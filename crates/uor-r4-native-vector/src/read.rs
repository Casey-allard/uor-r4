//! Integer-only first-pick read over one Q31 stream.
//!
//! Each row of a window is addressed twice: a query address through the query
//! gates and a key address through the key gates. An address cuts the row into
//! quaternion blocks, rotates each block by its own gate with
//! [`q31_hamilton_rotate`], and folds the rotated lanes into 16 exact `i64`
//! sums. Row `t` picks the earlier row `s < t` of the same window whose key
//! address has the least L1 distance to its query address, with a tie going to
//! the latest `s`. It then reads the table row of the token that followed the
//! pick, scales it lane by lane by integer weights, and adds the result to the
//! stream with Q31 saturation. No float, no squashing and no dense product.

use crate::{
    q31_coordinate_admitted, q31_hamilton_rotate, q31_saturate, Error, Q31Quaternion, Result,
    MAX_Q31_BATCH_ITEMS,
};

/// Largest admitted read width in lanes.
pub const MAX_Q31_READ_WIDTH: usize = 4096;
/// Lanes in one folded address.
const ADDRESS_LANES: usize = 16;
/// Largest admitted gain shift; the payload shift is `31 - gain_shift`.
const MAX_GAIN_SHIFT: u32 = 31;
const READ_VERSION: u8 = 1;
const HEADER_BYTES: usize = 9;
const ADMITTED_MAGNITUDE: u64 = 1 << 31;

// Every folded address word sums at most `width / 16` rotated lanes, each of
// magnitude at most 2^31; one distance term is at most twice that, and one
// distance sums 16 terms. All three bounds sit far below `i64::MAX`.
const _: () = assert!(
    (MAX_Q31_READ_WIDTH / ADDRESS_LANES) as u64 * ADMITTED_MAGNITUDE * 2 * ADDRESS_LANES as u64
        <= i64::MAX as u64
);

fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}

fn read_width_admitted(width: usize) -> bool {
    width != 0 && width.is_multiple_of(ADDRESS_LANES) && width <= MAX_Q31_READ_WIDTH
}

/// One first-pick read: query gates, key gates, per-lane weights and a gain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q31FirstPickRead {
    query_gates: Vec<Q31Quaternion>,
    key_gates: Vec<Q31Quaternion>,
    weights: Vec<i64>,
    gain_shift: u32,
}

/// The picks and the stream after the read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q31ReadOutput {
    /// `picks[b * time + t]` is `s + 1` for the picked row `s`, or `0` for row 0.
    pub picks: Vec<u32>,
    /// The stream plus the scaled payload, row-major like the input stream.
    pub out: Vec<i64>,
}

/// Fold one row into its 16-lane address under the given gates.
///
/// Block `j` is `row[4j..4j+4]`, rotated by `gates[j]`; lane `l` of the
/// rotated row adds into address word `l % 16` as an exact `i64` sum.
pub fn q31_read_address(gates: &[Q31Quaternion], row: &[i64]) -> Result<[i64; ADDRESS_LANES]> {
    let width = row.len();
    if !read_width_admitted(width) {
        return Err(invalid(
            "native Q31 read row width must be a nonzero multiple of 16 up to 4096",
        ));
    }
    if gates.len() != width / 4 {
        return Err(invalid("native Q31 read needs one gate per row block"));
    }
    let mut address = [0i64; ADDRESS_LANES];
    for (j, (&gate, block)) in gates.iter().zip(row.chunks_exact(4)).enumerate() {
        let rotated = q31_hamilton_rotate(gate, [block[0], block[1], block[2], block[3]])?;
        let base = (4 * j) % ADDRESS_LANES;
        for (k, word) in rotated.into_iter().enumerate() {
            address[base + k] = address[base + k]
                .checked_add(word)
                .expect("folded address stays inside the const-asserted bound");
        }
    }
    Ok(address)
}

fn read_distance(query: &[i64; ADDRESS_LANES], key: &[i64; ADDRESS_LANES]) -> u64 {
    query.iter().zip(key).fold(0u64, |total, (&q, &k)| {
        let term = q
            .checked_sub(k)
            .expect("address difference stays inside the const-asserted bound")
            .unsigned_abs();
        total
            .checked_add(term)
            .expect("address distance stays inside the const-asserted bound")
    })
}

impl Q31FirstPickRead {
    /// Construct one read only when every shape and word is admitted.
    pub fn new(
        query_gates: Vec<Q31Quaternion>,
        key_gates: Vec<Q31Quaternion>,
        weights: Vec<i64>,
        gain_shift: u32,
    ) -> Result<Self> {
        let width = weights.len();
        if !read_width_admitted(width) {
            return Err(invalid(
                "native Q31 read width must be a nonzero multiple of 16 up to 4096",
            ));
        }
        if query_gates.len() != width / 4 || key_gates.len() != width / 4 {
            return Err(invalid(
                "native Q31 read needs width / 4 query gates and width / 4 key gates",
            ));
        }
        let words_admitted = query_gates
            .iter()
            .chain(&key_gates)
            .flatten()
            .chain(&weights)
            .all(|&word| q31_coordinate_admitted(word));
        if !words_admitted {
            return Err(invalid("native Q31 read words must be in -2^31..=2^31"));
        }
        if gain_shift > MAX_GAIN_SHIFT {
            return Err(invalid("native Q31 read gain shift must be at most 31"));
        }
        Ok(Self {
            query_gates,
            key_gates,
            weights,
            gain_shift,
        })
    }

    /// Lanes per row.
    pub fn width(&self) -> usize {
        self.weights.len()
    }

    pub fn query_gates(&self) -> &[Q31Quaternion] {
        &self.query_gates
    }

    pub fn key_gates(&self) -> &[Q31Quaternion] {
        &self.key_gates
    }

    pub fn weights(&self) -> &[i64] {
        &self.weights
    }

    pub fn gain_shift(&self) -> u32 {
        self.gain_shift
    }

    /// Run the read over `batch` windows of `time` rows each.
    ///
    /// `stream` is `batch * time * width` words, `ids` is `batch * time` token
    /// ids, and `table` is `vocab * width` words. Any refusal returns an error
    /// and no partial output.
    pub fn apply(
        &self,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
    ) -> Result<Q31ReadOutput> {
        let width = self.width();
        let rows = batch
            .checked_mul(time)
            .filter(|&rows| rows <= MAX_Q31_BATCH_ITEMS)
            .ok_or_else(|| invalid("native Q31 read admits at most 2^30 rows"))?;
        let words = rows
            .checked_mul(width)
            .ok_or_else(|| invalid("native Q31 read stream length overflows usize"))?;
        if stream.len() != words {
            return Err(invalid(
                "native Q31 read stream must hold batch * time * width words",
            ));
        }
        if ids.len() != rows {
            return Err(invalid("native Q31 read needs batch * time ids"));
        }
        if table.is_empty() || !table.len().is_multiple_of(width) {
            return Err(invalid(
                "native Q31 read table must be a nonempty multiple of width",
            ));
        }
        if !stream
            .iter()
            .chain(table)
            .all(|&word| q31_coordinate_admitted(word))
        {
            return Err(invalid(
                "native Q31 read stream and table words must be in -2^31..=2^31",
            ));
        }
        let vocab = table.len() / width;

        let mut queries = Vec::with_capacity(rows);
        let mut keys = Vec::with_capacity(rows);
        for row in stream.chunks_exact(width) {
            queries.push(q31_read_address(&self.query_gates, row)?);
            keys.push(q31_read_address(&self.key_gates, row)?);
        }

        let shift = MAX_GAIN_SHIFT - self.gain_shift;
        let round = if shift > 0 { 1i64 << (shift - 1) } else { 0 };
        let mut picks = vec![0u32; rows];
        let mut out = stream.to_vec();
        for window in 0..batch {
            let first = window * time;
            for t in 1..time {
                let row = first + t;
                let mut best = 0usize;
                let mut best_distance = u64::MAX;
                for s in 0..t {
                    let distance = read_distance(&queries[row], &keys[first + s]);
                    // `<=` sends a tie to the latest earlier row.
                    if distance <= best_distance {
                        best_distance = distance;
                        best = s;
                    }
                }
                let id = ids[first + best + 1] as usize;
                if id >= vocab {
                    return Err(invalid("native Q31 read id must be below the table vocab"));
                }
                // `best + 1 <= t < rows <= 2^30`, so the pick fits `u32`.
                picks[row] = (best + 1) as u32;
                let payload = &table[id * width..(id + 1) * width];
                let target = &mut out[row * width..(row + 1) * width];
                for ((word, &weight), &value) in target.iter_mut().zip(&self.weights).zip(payload) {
                    let scaled = q31_saturate((weight * value + round) >> shift);
                    *word = q31_saturate(*word + scaled);
                }
            }
        }
        Ok(Q31ReadOutput { picks, out })
    }

    /// Lossless little-endian bytes: version, width, gain shift, then words.
    pub fn to_bytes(&self) -> Vec<u8> {
        let width = self.width();
        let mut bytes = Vec::with_capacity(HEADER_BYTES + 24 * width);
        bytes.push(READ_VERSION);
        // `width <= 4096`, so it fits `u32`.
        bytes.extend_from_slice(&(width as u32).to_le_bytes());
        bytes.extend_from_slice(&self.gain_shift.to_le_bytes());
        let words = self
            .query_gates
            .iter()
            .chain(&self.key_gates)
            .flatten()
            .chain(&self.weights);
        for word in words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Restore one read; refuses a wrong version, missing or trailing bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_BYTES {
            return Err(invalid("native Q31 read bytes are missing the header"));
        }
        if bytes[0] != READ_VERSION {
            return Err(invalid("native Q31 read bytes have an unknown version"));
        }
        let width = u32::from_le_bytes(bytes[1..5].try_into().expect("four header bytes")) as usize;
        let gain_shift = u32::from_le_bytes(bytes[5..9].try_into().expect("four header bytes"));
        if !read_width_admitted(width) {
            return Err(invalid(
                "native Q31 read bytes carry a width outside the admitted set",
            ));
        }
        if bytes.len() != HEADER_BYTES + 24 * width {
            return Err(invalid(
                "native Q31 read bytes must hold exactly 3 * width words",
            ));
        }
        let words: Vec<i64> = bytes[HEADER_BYTES..]
            .chunks_exact(8)
            .map(|chunk| i64::from_le_bytes(chunk.try_into().expect("eight word bytes")))
            .collect();
        let gates = |words: &[i64]| -> Vec<Q31Quaternion> {
            words
                .chunks_exact(4)
                .map(|g| [g[0], g[1], g[2], g[3]])
                .collect()
        };
        Self::new(
            gates(&words[..width]),
            gates(&words[width..2 * width]),
            words[2 * width..].to_vec(),
            gain_shift,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: i64 = 1 << 31;
    const IDENTITY: Q31Quaternion = [ONE, 0, 0, 0];

    fn identity_read(width: usize, weights: Vec<i64>, gain_shift: u32) -> Q31FirstPickRead {
        Q31FirstPickRead::new(
            vec![IDENTITY; width / 4],
            vec![IDENTITY; width / 4],
            weights,
            gain_shift,
        )
        .unwrap()
    }

    fn lane0_row(width: usize, lane0: i64) -> Vec<i64> {
        let mut row = vec![0; width];
        row[0] = lane0;
        row
    }

    fn refused<T: std::fmt::Debug>(result: Result<T>, needle: &str) {
        match result {
            Err(Error::Invalid(message)) => {
                assert!(message.contains(needle), "{message:?} lacks {needle:?}")
            }
            Ok(value) => panic!("expected a refusal containing {needle:?}, got {value:?}"),
        }
    }

    #[test]
    fn tie_goes_to_latest_and_closer_beats_later() {
        let read = identity_read(16, vec![0; 16], 0);
        let table = vec![0; 16];
        let ids = [0, 0, 0];

        let tied = [lane0_row(16, 10), lane0_row(16, 10), lane0_row(16, 12)].concat();
        let output = read.apply(&tied, &ids, &table, 1, 3).unwrap();
        assert_eq!(output.picks, vec![0, 1, 2]);

        let closer = [lane0_row(16, 12), lane0_row(16, 20), lane0_row(16, 12)].concat();
        let output = read.apply(&closer, &ids, &table, 1, 3).unwrap();
        assert_eq!(output.picks, vec![0, 1, 1]);
    }

    #[test]
    fn identity_fold_at_width_32_is_the_two_group_sum() {
        let blocks = [[ONE, -ONE, 3, -3], [1, -1, 0, 7], [-ONE, ONE, -1, 1]];
        for block in blocks {
            assert_eq!(q31_hamilton_rotate(IDENTITY, block).unwrap(), block);
        }
        let mut row: Vec<i64> = (0..32).map(|lane| lane * 1000 - 9001).collect();
        row[0] = ONE;
        row[16] = ONE;
        row[5] = -ONE;
        row[21] = -ONE;
        let address = q31_read_address(&[IDENTITY; 8], &row).unwrap();
        for (c, &word) in address.iter().enumerate() {
            assert_eq!(word, row[c] + row[16 + c], "lane {c}");
        }
        assert_eq!(address[0], 1 << 32);
        assert_eq!(address[5], -(1 << 32));
        refused(
            q31_read_address(&[IDENTITY; 7], &row),
            "one gate per row block",
        );
        refused(q31_read_address(&[IDENTITY; 3], &row[..12]), "row width");
    }

    #[test]
    fn row_zero_reads_nothing_and_picks_stay_in_their_window() {
        let read = identity_read(16, vec![ONE; 16], 0);
        let table: Vec<i64> = (0..3).flat_map(|v| vec![100 * (v + 1); 16]).collect();
        let lane0 = [1000, 2000, 7777, -5000, 7777, 7777];
        let stream: Vec<i64> = lane0.iter().flat_map(|&x| lane0_row(16, x)).collect();
        let ids = [0, 1, 2, 2, 0, 1];
        let output = read.apply(&stream, &ids, &table, 2, 3).unwrap();
        assert_eq!(output.picks, vec![0, 1, 2, 0, 1, 2]);

        let row = |r: usize| &output.out[r * 16..(r + 1) * 16];
        let added = |r: usize, value: i64| {
            let mut expected = lane0_row(16, lane0[r]);
            expected.iter_mut().for_each(|word| *word += value);
            expected
        };
        assert_eq!(row(0), &stream[..16]);
        assert_eq!(row(3), &stream[48..64]);
        assert_eq!(row(1), added(1, 200).as_slice());
        assert_eq!(row(2), added(2, 300).as_slice());
        assert_eq!(row(4), added(4, 100).as_slice());
        assert_eq!(row(5), added(5, 200).as_slice());
    }

    /// Row 1 always picks row 0 and reads table row `ids[1] = 1`.
    fn payload_row(gain_shift: u32, weights: &[i64], payload: &[i64], base: &[i64]) -> Vec<i64> {
        let pad = |words: &[i64]| {
            let mut padded = words.to_vec();
            padded.resize(16, 0);
            padded
        };
        let read = identity_read(16, pad(weights), gain_shift);
        let table = [vec![0; 16], pad(payload)].concat();
        let stream = [vec![0; 16], pad(base)].concat();
        let output = read.apply(&stream, &[0, 1], &table, 1, 2).unwrap();
        assert_eq!(output.picks, vec![0, 1]);
        assert_eq!(&output.out[..16], &[0; 16]);
        output.out[16..].to_vec()
    }

    #[test]
    fn payload_words_at_gain_shifts_0_4_and_31() {
        let half = 1 << 30;
        let weights = [half, half, half, ONE, ONE, -ONE, 1, 3];
        let payload = [3, -3, -5, -7, ONE, ONE, -1, -half];
        let base = [0, 0, 0, 10, ONE, -ONE, 4, 0];
        let out = payload_row(0, &weights, &payload, &base);
        assert_eq!(&out[..8], &[2, -1, -2, 3, ONE, -ONE, 4, -1]);
        assert_eq!(&out[8..], &[0; 8]);
        // Lanes 4 and 5 saturate in the add, so the bare product shows only elsewhere.
        for lane in [0, 1, 2, 3, 6, 7] {
            let crate_product =
                q31_hamilton_rotate([weights[lane], 0, 0, 0], [payload[lane], 0, 0, 0]).unwrap()[0];
            assert_eq!(out[lane] - base[lane], crate_product, "lane {lane}");
        }

        let weights = [1, 1, 1 << 27, ONE, ONE, -ONE];
        let payload = [-(3 << 25), 5 << 25, -9, 1 << 27, ONE, ONE];
        let base = [0, 0, 0, 1, -ONE, 5];
        let out = payload_row(4, &weights, &payload, &base);
        assert_eq!(&out[..6], &[-1, 1, -9, ONE, 0, -ONE + 5]);

        let weights = [3, 1 << 16, -(1 << 16), ONE];
        let payload = [-5, 1 << 16, 1 << 16, ONE];
        let base = [0, -1, 0, ONE];
        let out = payload_row(31, &weights, &payload, &base);
        assert_eq!(&out[..4], &[-15, ONE - 1, -ONE, ONE]);
    }

    #[test]
    fn refusals_return_no_output() {
        let width = 16;
        let gates = || vec![IDENTITY; 4];
        refused(
            Q31FirstPickRead::new(vec![], vec![], vec![], 0),
            "nonzero multiple of 16",
        );
        refused(
            Q31FirstPickRead::new(vec![IDENTITY; 6], vec![IDENTITY; 6], vec![0; 24], 0),
            "nonzero multiple of 16",
        );
        refused(
            Q31FirstPickRead::new(vec![IDENTITY; 1028], vec![IDENTITY; 1028], vec![0; 4112], 0),
            "nonzero multiple of 16",
        );
        refused(
            Q31FirstPickRead::new(vec![IDENTITY; 3], gates(), vec![0; width], 0),
            "width / 4 query gates",
        );
        refused(
            Q31FirstPickRead::new(gates(), vec![IDENTITY; 5], vec![0; width], 0),
            "width / 4 query gates",
        );
        let mut bad_weights = vec![0; width];
        bad_weights[3] = ONE + 1;
        refused(
            Q31FirstPickRead::new(gates(), gates(), bad_weights, 0),
            "words must be",
        );
        refused(
            Q31FirstPickRead::new(vec![[0, -ONE - 1, 0, 0]; 4], gates(), vec![0; width], 0),
            "words must be",
        );
        refused(
            Q31FirstPickRead::new(gates(), vec![[0, 0, 0, ONE + 1]; 4], vec![0; width], 0),
            "words must be",
        );
        refused(
            Q31FirstPickRead::new(gates(), gates(), vec![0; width], 32),
            "gain shift",
        );
        assert!(Q31FirstPickRead::new(
            vec![IDENTITY; 1024],
            vec![IDENTITY; 1024],
            vec![0; 4096],
            31
        )
        .is_ok());

        let read = identity_read(width, vec![ONE; width], 0);
        let stream = vec![1; 2 * width];
        let table = vec![0; 2 * width];
        let ids = [0, 1];
        assert!(read.apply(&stream, &ids, &table, 1, 2).is_ok());

        let mut bad_stream = stream.clone();
        bad_stream[7] = ONE + 1;
        refused(
            read.apply(&bad_stream, &ids, &table, 1, 2),
            "stream and table words",
        );
        let mut bad_table = table.clone();
        bad_table[20] = -ONE - 1;
        refused(
            read.apply(&stream, &ids, &bad_table, 1, 2),
            "stream and table words",
        );
        refused(
            read.apply(&stream[1..], &ids, &table, 1, 2),
            "batch * time * width",
        );
        refused(
            read.apply(&stream, &ids, &table, 2, 2),
            "batch * time * width",
        );
        refused(
            read.apply(&stream, &ids[..1], &table, 1, 2),
            "batch * time ids",
        );
        refused(read.apply(&stream, &[], &table, 1, 2), "batch * time ids");
        refused(
            read.apply(&stream, &ids, &table[1..], 1, 2),
            "nonempty multiple",
        );
        refused(read.apply(&stream, &ids, &[], 1, 2), "nonempty multiple");
        refused(
            read.apply(&stream, &[0, 2], &table, 1, 2),
            "below the table vocab",
        );
        // Id 0 is never read: row 0 reads nothing and row 1 reads `ids[1]`.
        assert!(read.apply(&stream, &[99, 1], &table, 1, 2).is_ok());
        refused(
            read.apply(&stream, &ids, &table, usize::MAX, 2),
            "at most 2^30 rows",
        );
        refused(
            read.apply(&stream, &ids, &table, MAX_Q31_BATCH_ITEMS + 1, 1),
            "at most 2^30 rows",
        );
    }

    #[test]
    fn bytes_round_trip_and_refuse_bad_buffers() {
        let query: Vec<Q31Quaternion> = (0..8).map(|j| [ONE - j, -j, j * 7, -ONE + j]).collect();
        let key: Vec<Q31Quaternion> = (0..8).map(|j| [j, ONE, -ONE, 3 - j]).collect();
        let weights: Vec<i64> = (0..32).map(|l| (l - 16) * 123_456_789).collect();
        let read = Q31FirstPickRead::new(query, key, weights, 9).unwrap();
        let bytes = read.to_bytes();
        assert_eq!(bytes.len(), 9 + 24 * 32);
        assert_eq!(bytes[0], 1);
        assert_eq!(&bytes[1..5], &32u32.to_le_bytes());
        assert_eq!(&bytes[5..9], &9u32.to_le_bytes());
        assert_eq!(Q31FirstPickRead::from_bytes(&bytes).unwrap(), read);

        refused(
            Q31FirstPickRead::from_bytes(&bytes[..bytes.len() - 1]),
            "exactly",
        );
        refused(Q31FirstPickRead::from_bytes(&bytes[..40]), "exactly");
        refused(
            Q31FirstPickRead::from_bytes(&bytes[..8]),
            "missing the header",
        );
        let mut trailing = bytes.clone();
        trailing.push(0);
        refused(Q31FirstPickRead::from_bytes(&trailing), "exactly");
        let mut version = bytes.clone();
        version[0] = 2;
        refused(Q31FirstPickRead::from_bytes(&version), "unknown version");
        let mut width = bytes.clone();
        width[1..5].copy_from_slice(&24u32.to_le_bytes());
        refused(Q31FirstPickRead::from_bytes(&width), "width outside");
        let mut gain = bytes.clone();
        gain[5..9].copy_from_slice(&32u32.to_le_bytes());
        refused(Q31FirstPickRead::from_bytes(&gain), "gain shift");
        let mut word = bytes.clone();
        word[9..17].copy_from_slice(&(ONE + 1).to_le_bytes());
        refused(Q31FirstPickRead::from_bytes(&word), "words must be");
    }

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn small(&mut self) -> i64 {
            (self.next() % (1 << 21)) as i64 - (1 << 20)
        }

        fn admitted(&mut self) -> i64 {
            (self.next() % ((1 << 32) + 1)) as i64 - ONE
        }
    }

    #[test]
    fn seeded_fixture_is_deterministic_and_one_key_lsb_moves_a_pick() {
        let (width, batch, time, vocab) = (16, 2, 8, 5);
        let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
        let weights: Vec<i64> = (0..width).map(|_| rng.admitted()).collect();
        let table: Vec<i64> = (0..vocab * width).map(|_| rng.small()).collect();
        let ids: Vec<u32> = (0..batch * time)
            .map(|_| (rng.next() % vocab as u64) as u32)
            .collect();
        let mut stream: Vec<i64> = (0..batch * time * width).map(|_| rng.small()).collect();

        // Row 6 queries at lane 0 = 3 * 2^29; rows 2 and 4 sit 1000 above and
        // below it with every other lane equal, so they tie exactly, while all
        // random rows sit near 2^30 farther away.
        let near = 3i64 << 29;
        let gap = 1000;
        stream[6 * width] = near;
        let query_row = stream[6 * width..7 * width].to_vec();
        for (row, lane0) in [(2, near + gap), (4, near - gap)] {
            stream[row * width..(row + 1) * width].copy_from_slice(&query_row);
            stream[row * width] = lane0;
        }

        let read = identity_read(width, weights.clone(), 0);
        let base = read.apply(&stream, &ids, &table, batch, time).unwrap();
        assert_eq!(
            base,
            read.apply(&stream, &ids, &table, batch, time).unwrap()
        );
        let restored = Q31FirstPickRead::from_bytes(&read.to_bytes()).unwrap();
        assert_eq!(
            base,
            restored.apply(&stream, &ids, &table, batch, time).unwrap()
        );
        assert_eq!(base.picks[6], 5, "the tie goes to row 4");

        // One LSB off the first key gate's real word shrinks every key lane
        // above 2^30 by one: row 2 moves closer and row 4 farther.
        let mut key_gates = vec![IDENTITY; width / 4];
        key_gates[0][0] = ONE - 1;
        let nudged = Q31FirstPickRead::new(vec![IDENTITY; width / 4], key_gates, weights, 0)
            .unwrap()
            .apply(&stream, &ids, &table, batch, time)
            .unwrap();
        assert_eq!(nudged.picks[6], 3, "row 2 is now strictly closer");
        assert_ne!(nudged, base);
    }
}
