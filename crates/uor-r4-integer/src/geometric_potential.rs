//! Native finite geometric potentials: integer table lookup and exact addition.
//!
//! An offline compiler evaluates seven learned potential families on the fixed
//! signed H4 geometry and rounds each table entry to signed Q24. These entries
//! are four-byte score values, not four-bit linear-map weights. This module
//! performs no placement, matrix multiplication, normalization or floating
//! arithmetic. The enclosing artifact loader binds the source model, codebook,
//! quantizer and complete table bytes; this constructor validates their layout.
//!
//! Every lane contributes two presence scores, up to two directed unaries, two
//! radial scores, and one cross score. Absent endpoints suppress the respective
//! unary/radial channel; cross requires both channels present. Presence is
//! always scored, including both-absent. Query/key order and antipodes remain
//! distinct. Age, NoRead, admission, softmax and values are outside this scorer.
//!
//! Admission allocates and partitions tables by head and lane. `score` allocates
//! nothing and uses table reads, integer shifts and addition, with no source
//! multiplication or division. Compiled instruction evidence is separate.

use std::fmt;

use crate::h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT};

pub const FRACTIONAL_BITS: u32 = 24;
pub const MAX_HEADS: usize = 64;
pub const MAX_LANES_PER_HEAD: usize = 32;
pub const DIRECTION_ENTRIES: usize = ROOT_COUNT;
pub const PAIR_STRIDE: usize = 128;
pub const PAIR_ENTRIES: usize = PAIR_STRIDE * PAIR_STRIDE;
pub const RADIUS_BINS: usize = 32;
pub const RADIUS_ENTRIES: usize = RADIUS_BINS * RADIUS_BINS;
pub const PRESENCE_ENTRIES: usize = 4;
pub const CONTENT_UNARY_OFFSET: usize = 0;
pub const CONTEXT_UNARY_OFFSET: usize = CONTENT_UNARY_OFFSET + DIRECTION_ENTRIES;
pub const PAIR_OFFSET: usize = CONTEXT_UNARY_OFFSET + DIRECTION_ENTRIES;
pub const CONTENT_RADIUS_OFFSET: usize = PAIR_OFFSET + PAIR_ENTRIES;
pub const CONTEXT_RADIUS_OFFSET: usize = CONTENT_RADIUS_OFFSET + RADIUS_ENTRIES;
pub const CONTENT_PRESENCE_OFFSET: usize = CONTEXT_RADIUS_OFFSET + RADIUS_ENTRIES;
pub const CONTEXT_PRESENCE_OFFSET: usize = CONTENT_PRESENCE_OFFSET + PRESENCE_ENTRIES;
pub const ENTRIES_PER_LANE: usize = CONTEXT_PRESENCE_OFFSET + PRESENCE_ENTRIES;
pub const TERMS_PER_LANE: usize = 7;
/// Includes the asymmetric i32 minimum. This is less than 2^39 and fits i64.
pub const MAX_ABS_SCORE: i64 = MAX_LANES_PER_HEAD as i64 * TERMS_PER_LANE as i64 * (1_i64 << 31);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressInput {
    ContentQuery,
    ContentKey,
    ContextQuery,
    ContextKey,
}

/// Fixed-size admission/scoring errors; no numerical error string allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PotentialError {
    InvalidRoot(u8),
    InvalidRadius(u8),
    NoncanonicalAbsent {
        root: u8,
        radius_bin: u8,
    },
    InvalidHeads(usize),
    InvalidLanes(usize),
    DimensionOverflow,
    TableLength {
        expected: usize,
        actual: usize,
    },
    NonzeroPadding {
        head: usize,
        lane: usize,
        row: usize,
        column: usize,
    },
    InvalidHead {
        head: usize,
        heads: usize,
    },
    WidthMismatch {
        input: AddressInput,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for PotentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot(root) => write!(f, "geometric potential root {root} is outside 0..120"),
            Self::InvalidRadius(bin) => write!(f, "geometric potential radius bin {bin} is outside 0..32"),
            Self::NoncanonicalAbsent { root, radius_bin } => write!(f, "absent geometric lane needs identity root 1 and radius bin 0, got {root}/{radius_bin}"),
            Self::InvalidHeads(heads) => write!(f, "geometric potential heads {heads} are outside 1..={MAX_HEADS}"),
            Self::InvalidLanes(lanes) => write!(f, "geometric potential lanes {lanes} are outside 1..={MAX_LANES_PER_HEAD}"),
            Self::DimensionOverflow => f.write_str("geometric potential table dimensions overflow"),
            Self::TableLength { expected, actual } => write!(f, "geometric potential tables need {expected} entries, got {actual}"),
            Self::NonzeroPadding { head, lane, row, column } => write!(f, "nonzero geometric pair padding at head {head}, lane {lane}, row {row}, column {column}"),
            Self::InvalidHead { head, heads } => write!(f, "geometric potential head {head} is outside 0..{heads}"),
            Self::WidthMismatch { input, expected, actual } => write!(f, "geometric potential {input:?} needs {expected} lanes, got {actual}"),
        }
    }
}

impl std::error::Error for PotentialError {}

pub type PotentialResult<T> = Result<T, PotentialError>;

/// A validated geometric address lane. Missingness is separate from a present
/// identity or minimum-radius code; absent lanes have one canonical encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressLane {
    root: H4Code,
    radius_bin: u8,
    present: bool,
}

impl AddressLane {
    pub fn new(root: u8, radius_bin: u8, present: bool) -> PotentialResult<Self> {
        let code = H4Code::try_from(root).map_err(|_| PotentialError::InvalidRoot(root))?;
        if usize::from(radius_bin) >= RADIUS_BINS {
            return Err(PotentialError::InvalidRadius(radius_bin));
        }
        if !present && (code != H4Code::IDENTITY || radius_bin != 0) {
            return Err(PotentialError::NoncanonicalAbsent { root, radius_bin });
        }
        Ok(Self {
            root: code,
            radius_bin,
            present,
        })
    }

    pub fn root(self) -> u8 {
        self.root.index()
    }
    pub fn radius_bin(self) -> u8 {
        self.radius_bin
    }
    pub fn present(self) -> bool {
        self.present
    }
}

#[derive(Debug)]
struct HeadTables {
    lanes: Box<[Box<[i32]>]>,
}

/// Immutable scores ordered head, lane, then CU120, RU120, Pair128x128,
/// CR32x32, RR32x32, CP4, RP4. Every pair entry outside 120x120 is zero.
#[derive(Debug)]
pub struct NativePotentialTables {
    heads: Box<[HeadTables]>,
    lanes: usize,
}

impl NativePotentialTables {
    pub fn new(heads: usize, lanes: usize, flat: &[i32]) -> PotentialResult<Self> {
        if !(1..=MAX_HEADS).contains(&heads) {
            return Err(PotentialError::InvalidHeads(heads));
        }
        if !(1..=MAX_LANES_PER_HEAD).contains(&lanes) {
            return Err(PotentialError::InvalidLanes(lanes));
        }
        let per_head = lanes
            .checked_mul(ENTRIES_PER_LANE)
            .ok_or(PotentialError::DimensionOverflow)?;
        let expected = heads
            .checked_mul(per_head)
            .ok_or(PotentialError::DimensionOverflow)?;
        if flat.len() != expected {
            return Err(PotentialError::TableLength {
                expected,
                actual: flat.len(),
            });
        }
        let mut partitioned = Vec::with_capacity(heads);
        for (head, values) in flat.chunks_exact(per_head).enumerate() {
            let mut head_lanes = Vec::with_capacity(lanes);
            for (lane, values) in values.chunks_exact(ENTRIES_PER_LANE).enumerate() {
                for row in 0..PAIR_STRIDE {
                    for column in 0..PAIR_STRIDE {
                        if (row >= ROOT_COUNT || column >= ROOT_COUNT)
                            && values[PAIR_OFFSET + row * PAIR_STRIDE + column] != 0
                        {
                            return Err(PotentialError::NonzeroPadding {
                                head,
                                lane,
                                row,
                                column,
                            });
                        }
                    }
                }
                head_lanes.push(values.into());
            }
            partitioned.push(HeadTables {
                lanes: head_lanes.into_boxed_slice(),
            });
        }
        Ok(Self {
            heads: partitioned.into_boxed_slice(),
            lanes,
        })
    }

    pub fn heads(&self) -> usize {
        self.heads.len()
    }
    pub fn lanes(&self) -> usize {
        self.lanes
    }

    /// Exact Q24 sum over one head. All input lengths and the head are checked
    /// before lookup. At most 32*7 i32 values are widened and added, so neither
    /// accumulation nor any intermediate sum can overflow i64. No saturation,
    /// per-term rescale, normalization or score centering occurs here.
    pub fn score(
        &self,
        head: usize,
        cq: &[AddressLane],
        ck: &[AddressLane],
        rq: &[AddressLane],
        rk: &[AddressLane],
        tables: &HistoricalH4Tables,
    ) -> PotentialResult<i64> {
        for (input, values) in [
            (AddressInput::ContentQuery, cq),
            (AddressInput::ContentKey, ck),
            (AddressInput::ContextQuery, rq),
            (AddressInput::ContextKey, rk),
        ] {
            if values.len() != self.lanes {
                return Err(PotentialError::WidthMismatch {
                    input,
                    expected: self.lanes,
                    actual: values.len(),
                });
            }
        }
        let head = self.heads.get(head).ok_or(PotentialError::InvalidHead {
            head,
            heads: self.heads.len(),
        })?;
        let mut total = 0i64;
        for ((((values, cq), ck), rq), rk) in head.lanes.iter().zip(cq).zip(ck).zip(rq).zip(rk) {
            let content_presence = (usize::from(cq.present) << 1) + usize::from(ck.present);
            let context_presence = (usize::from(rq.present) << 1) + usize::from(rk.present);
            total += i64::from(values[CONTENT_PRESENCE_OFFSET + content_presence]);
            total += i64::from(values[CONTEXT_PRESENCE_OFFSET + context_presence]);
            let content = if cq.present && ck.present {
                let relative = tables.relative(cq.root, ck.root);
                total += i64::from(values[CONTENT_UNARY_OFFSET + usize::from(relative.index())]);
                let radius = (usize::from(cq.radius_bin) << 5) + usize::from(ck.radius_bin);
                total += i64::from(values[CONTENT_RADIUS_OFFSET + radius]);
                Some(relative)
            } else {
                None
            };
            let context = if rq.present && rk.present {
                let relative = tables.relative(rq.root, rk.root);
                total += i64::from(values[CONTEXT_UNARY_OFFSET + usize::from(relative.index())]);
                let radius = (usize::from(rq.radius_bin) << 5) + usize::from(rk.radius_bin);
                total += i64::from(values[CONTEXT_RADIUS_OFFSET + radius]);
                Some(relative)
            } else {
                None
            };
            if let (Some(content), Some(context)) = (content, context) {
                let pair = (usize::from(content.index()) << 7) + usize::from(context.index());
                total += i64::from(values[PAIR_OFFSET + pair]);
            }
        }
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn geometry() -> Result<HistoricalH4Tables, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/historical-h4-tables-v1.bin"
        ))?;
        Ok(HistoricalH4Tables::from_bytes(&bytes)?)
    }

    fn uniform(lanes: usize, value: i32) -> Vec<i32> {
        let mut flat = vec![value; lanes * ENTRIES_PER_LANE];
        for lane in flat.chunks_exact_mut(ENTRIES_PER_LANE) {
            for row in 0..PAIR_STRIDE {
                for column in 0..PAIR_STRIDE {
                    if row >= ROOT_COUNT || column >= ROOT_COUNT {
                        lane[PAIR_OFFSET + row * PAIR_STRIDE + column] = 0;
                    }
                }
            }
        }
        flat
    }

    #[test]
    fn native_geometric_potential_directed_signed_relative_and_cross() -> TestResult {
        let geometry = geometry()?;
        let mut flat = uniform(1, 0);
        for content in 0..ROOT_COUNT {
            flat[CONTENT_UNARY_OFFSET + content] = content as i32 * 10;
            flat[CONTEXT_UNARY_OFFSET + content] = content as i32 * 100;
            for context in 0..ROOT_COUNT {
                flat[PAIR_OFFSET + content * PAIR_STRIDE + context] =
                    content as i32 * 1000 + context as i32;
            }
        }
        let scores = NativePotentialTables::new(1, 1, &flat)?;
        let i = [AddressLane::new(3, 16, true)?];
        let j = [AddressLane::new(5, 16, true)?];
        let minus_i = [AddressLane::new(2, 16, true)?];
        // inverse(i)*j=-k (6); inverse(j)*i=k (7).
        assert_eq!(scores.score(0, &i, &j, &j, &i, &geometry)?, 60 + 700 + 6007);
        assert_eq!(scores.score(0, &j, &i, &j, &i, &geometry)?, 70 + 700 + 7007);
        assert_eq!(
            scores.score(0, &minus_i, &j, &j, &i, &geometry)?,
            70 + 700 + 7007
        );
        for root in 0..ROOT_COUNT as u8 {
            let same = [AddressLane::new(root, 0, true)?];
            assert_eq!(
                scores.score(0, &same, &same, &same, &same, &geometry)?,
                10 + 100 + 1001
            );
        }
        Ok(())
    }

    #[test]
    fn native_geometric_potential_all_presence_states_and_radius_indices() -> TestResult {
        let geometry = geometry()?;
        let mut flat = uniform(1, 0);
        flat[CONTENT_UNARY_OFFSET..CONTEXT_UNARY_OFFSET].fill(40);
        flat[CONTEXT_UNARY_OFFSET..PAIR_OFFSET].fill(50);
        for content in 0..ROOT_COUNT {
            for context in 0..ROOT_COUNT {
                flat[PAIR_OFFSET + content * PAIR_STRIDE + context] = 70;
            }
        }
        for index in 0..RADIUS_ENTRIES {
            flat[CONTENT_RADIUS_OFFSET + index] = 100 + index as i32;
            flat[CONTEXT_RADIUS_OFFSET + index] = 200 + index as i32;
        }
        for state in 0..4 {
            flat[CONTENT_PRESENCE_OFFSET + state] = 1000 + state as i32;
            flat[CONTEXT_PRESENCE_OFFSET + state] = 2000 + state as i32;
        }
        let scores = NativePotentialTables::new(1, 1, &flat)?;
        let lane = |present, bin| AddressLane::new(1, if present { bin } else { 0 }, present);
        for cp in 0..4 {
            for rp in 0..4 {
                let cq = [lane(cp & 2 != 0, 31)?];
                let ck = [lane(cp & 1 != 0, 7)?];
                let rq = [lane(rp & 2 != 0, 2)?];
                let rk = [lane(rp & 1 != 0, 31)?];
                let expected = 3000
                    + cp
                    + rp
                    + if cp == 3 { 40 + 100 + 999 } else { 0 }
                    + if rp == 3 { 50 + 200 + 95 } else { 0 }
                    + if cp == 3 && rp == 3 { 70 } else { 0 };
                assert_eq!(scores.score(0, &cq, &ck, &rq, &rk, &geometry)?, expected);
            }
        }
        let absent = [AddressLane::new(1, 0, false)?];
        for q in 0..32 {
            for k in 0..32 {
                let query = [AddressLane::new(1, q, true)?];
                let key = [AddressLane::new(1, k, true)?];
                assert_eq!(
                    scores.score(0, &query, &key, &absent, &absent, &geometry)?,
                    3003 + 40 + 100 + i64::from(q) * 32 + i64::from(k)
                );
                assert_eq!(
                    scores.score(0, &absent, &absent, &query, &key, &geometry)?,
                    3003 + 50 + 200 + i64::from(q) * 32 + i64::from(k)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn native_geometric_potential_admission_and_input_errors_do_not_change_scores() -> TestResult {
        let geometry = geometry()?;
        for root in [120, 255] {
            assert_eq!(
                AddressLane::new(root, 0, false),
                Err(PotentialError::InvalidRoot(root))
            );
        }
        for radius in [32, 255] {
            assert_eq!(
                AddressLane::new(1, radius, false),
                Err(PotentialError::InvalidRadius(radius))
            );
        }
        assert!(matches!(
            AddressLane::new(0, 0, false),
            Err(PotentialError::NoncanonicalAbsent { .. })
        ));
        assert!(matches!(
            AddressLane::new(1, 1, false),
            Err(PotentialError::NoncanonicalAbsent { .. })
        ));
        for heads in [0, MAX_HEADS + 1, usize::MAX] {
            assert!(matches!(
                NativePotentialTables::new(heads, 1, &[]),
                Err(PotentialError::InvalidHeads(_))
            ));
        }
        for lanes in [0, MAX_LANES_PER_HEAD + 1, usize::MAX] {
            assert!(matches!(
                NativePotentialTables::new(1, lanes, &[]),
                Err(PotentialError::InvalidLanes(_))
            ));
        }
        assert!(matches!(
            NativePotentialTables::new(1, 1, &[]),
            Err(PotentialError::TableLength { .. })
        ));
        for (row, column) in [(120, 0), (0, 120), (127, 127)] {
            let mut bad = uniform(1, 0);
            bad[PAIR_OFFSET + row * PAIR_STRIDE + column] = 1;
            assert!(matches!(
                NativePotentialTables::new(1, 1, &bad),
                Err(PotentialError::NonzeroPadding { .. })
            ));
        }
        let mut flat = uniform(4, 0);
        for (lane, value) in [1, 2, 5, 7].into_iter().enumerate() {
            flat[lane * ENTRIES_PER_LANE + CONTENT_PRESENCE_OFFSET] = value;
        }
        let scores = NativePotentialTables::new(2, 2, &flat)?;
        assert_eq!((scores.heads(), scores.lanes()), (2, 2));
        let absent = [AddressLane::new(1, 0, false)?; 2];
        assert_eq!(
            scores.score(0, &absent, &absent, &absent, &absent, &geometry)?,
            3
        );
        assert_eq!(
            scores.score(1, &absent, &absent, &absent, &absent, &geometry)?,
            12
        );
        for index in 0..4 {
            let mut input: [&[AddressLane]; 4] = [&absent; 4];
            input[index] = &absent[..1];
            assert!(matches!(
                scores.score(0, input[0], input[1], input[2], input[3], &geometry),
                Err(PotentialError::WidthMismatch { .. })
            ));
        }
        assert!(matches!(
            scores.score(usize::MAX, &absent, &absent, &absent, &absent, &geometry),
            Err(PotentialError::InvalidHead { .. })
        ));
        assert_eq!(
            scores.score(1, &absent, &absent, &absent, &absent, &geometry)?,
            12
        );
        Ok(())
    }

    #[test]
    fn native_geometric_potential_extrema_accumulate_exactly_in_i64() -> TestResult {
        let geometry = geometry()?;
        assert_eq!(ENTRIES_PER_LANE, 18680);
        assert_eq!(
            [
                CONTENT_UNARY_OFFSET,
                CONTEXT_UNARY_OFFSET,
                PAIR_OFFSET,
                CONTENT_RADIUS_OFFSET,
                CONTEXT_RADIUS_OFFSET,
                CONTENT_PRESENCE_OFFSET,
                CONTEXT_PRESENCE_OFFSET
            ],
            [0, 120, 240, 16624, 17648, 18672, 18676]
        );
        assert!(MAX_ABS_SCORE < (1_i64 << 39));
        let present = [AddressLane::new(1, 31, true)?; MAX_LANES_PER_HEAD];
        let absent = [AddressLane::new(1, 0, false)?; MAX_LANES_PER_HEAD];
        for value in [i32::MIN, i32::MAX] {
            let scores = NativePotentialTables::new(
                1,
                MAX_LANES_PER_HEAD,
                &uniform(MAX_LANES_PER_HEAD, value),
            )?;
            assert_eq!(
                scores.score(0, &present, &present, &present, &present, &geometry)?,
                MAX_LANES_PER_HEAD as i64 * 7 * i64::from(value)
            );
            assert_eq!(
                scores.score(0, &absent, &absent, &absent, &absent, &geometry)?,
                MAX_LANES_PER_HEAD as i64 * 2 * i64::from(value)
            );
        }
        Ok(())
    }
}
