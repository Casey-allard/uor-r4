//! The native Q31 first-pick read in a served stack (claude-390, schema `uor-r4.lut-stack/14`).
//!
//! The operator is `uor_r4_native_vector::Q31FirstPickRead` (claude-389); this module is the
//! artifact's and the session's side: the header record, the table that holds the read's own
//! bytes, the image read from a parsed container, and the served form with its payload table.
//!
//! Scales. The session's residual is i32 at `2^-16` and the writer table is i16 at `2^-12`. The
//! read's weights are Q31 words meaning `w · 2^(35 - g)` for gain shift `g`, so the operator's
//! `(w·e + r) >> (31 - g)` lands a table row on the residual's `2^-16` with no rescale at serve
//! time. The served row is the residual widened to i64 exactly, and the out row is written back
//! only when every word fits i32: an out word of `+2^31` is refused, never clamped.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_native_vector::{Q31FirstPickRead, Q31ReadHistory};

use super::format::Container;
use super::read_out::ReadOut;
use super::{StackError, StackShape};

/// The table that holds the read's bytes, as little-endian u32 words, the last zero-padded.
pub const Q31_READ_TABLE: &str = "q31.read";

/// The header record of a stack's native Q31 read.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StackQ31Read {
    /// The table name, [`Q31_READ_TABLE`].
    pub table: String,
    /// The read runs after layer `depth - 1`'s MLP add; `1..=layers`.
    pub depth: usize,
    /// Lanes per row: the stack's width.
    pub width: usize,
    /// The exact length of `Q31FirstPickRead::to_bytes()`.
    pub bytes: usize,
    /// SHA-256 over those exact bytes.
    pub sha256: String,
}

impl StackQ31Read {
    /// Refuse a record this reader does not serve on `shape`.
    pub(crate) fn check(&self, shape: &StackShape) -> Result<(), StackError> {
        if !shape.projection_fbs() {
            return Err(StackError::Shape(
                "a native Q31 read is served on a SpiralCore/FBS stack",
            ));
        }
        if self.table != Q31_READ_TABLE {
            return Err(StackError::Shape(
                "a native Q31 read's bytes are in the q31.read table",
            ));
        }
        if self.depth == 0 || self.depth > shape.layers() {
            return Err(StackError::Shape(
                "a native Q31 read's depth is from 1 to the stack's layers",
            ));
        }
        if self.width != shape.width {
            return Err(StackError::Shape(
                "a native Q31 read is as wide as the stack",
            ));
        }
        Ok(())
    }
}

/// A native Q31 first-pick read and the depth it is served at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Q31SessionRead {
    depth: usize,
    read: Q31FirstPickRead,
}

impl Q31SessionRead {
    /// A read served after layer `depth - 1`; `depth` is checked against a stack when it is added.
    pub fn new(depth: usize, read: Q31FirstPickRead) -> Self {
        Self { depth, read }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn read(&self) -> &Q31FirstPickRead {
        &self.read
    }

    /// The header record for these exact bytes.
    pub fn record(&self) -> StackQ31Read {
        let bytes = self.read.to_bytes();
        StackQ31Read {
            table: Q31_READ_TABLE.to_owned(),
            depth: self.depth,
            width: self.read.width(),
            bytes: bytes.len(),
            sha256: hex::encode(Sha256::digest(&bytes)),
        }
    }

    /// The read's bytes as little-endian u32 words, the last word zero-padded.
    pub(crate) fn words(&self) -> Vec<u32> {
        let mut bytes = self.read.to_bytes();
        bytes.resize(bytes.len().div_ceil(4) * 4, 0);
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }
}

/// The native Q31 read an artifact holds, or none when its header has no record. Refuses a table
/// whose words are not exactly the record's bytes (length, zero padding, digest) or whose bytes
/// `Q31FirstPickRead::from_bytes` refuses.
pub(crate) fn q31_read_image(
    artifact: &Container<'_>,
) -> Result<Option<Q31SessionRead>, StackError> {
    let Some(record) = &artifact.q31_read else {
        return Ok(None);
    };
    record.check(&artifact.shape)?;
    let words = artifact.table_u32(&record.table)?;
    if words.len() != record.bytes.div_ceil(4) {
        return Err(StackError::TableSection(record.table.clone()));
    }
    let mut bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    if bytes[record.bytes..].iter().any(|&b| b != 0) {
        return Err(StackError::Numerics(
            "q31.read padding must be zero".to_owned(),
        ));
    }
    bytes.truncate(record.bytes);
    if hex::encode(Sha256::digest(&bytes)) != record.sha256 {
        return Err(StackError::Numerics(
            "q31.read sha256 does not match the stored words".to_owned(),
        ));
    }
    let read = Q31FirstPickRead::from_bytes(&bytes)
        .map_err(|e| StackError::Numerics(format!("q31.read: {e}")))?;
    if read.width() != record.width {
        return Err(StackError::Numerics(
            "q31.read width does not match its record".to_owned(),
        ));
    }
    Ok(Some(Q31SessionRead::new(record.depth, read)))
}

/// The native Q31 read of the `UORLUT01` artifact `bytes`, or none when it serves none.
pub fn read_q31_read_uorlut01(bytes: &[u8]) -> Result<Option<Q31SessionRead>, StackError> {
    q31_read_image(&Container::parse(bytes)?)
}

/// A loaded read: the operator, its depth, and the writer table widened to i64 once at load.
#[derive(Clone, Debug)]
pub(crate) struct ServedQ31Read {
    pub depth: usize,
    read: Q31FirstPickRead,
    table: Vec<i64>,
}

/// One served step of the read: its pick, the row handed to it, and the row it wrote back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Q31ReadStep {
    pub pick: u32,
    pub x_in: Vec<i64>,
    pub out: Vec<i64>,
}

impl ServedQ31Read {
    pub fn new(image: Q31SessionRead, writer_table: &[i16]) -> Self {
        Self {
            depth: image.depth,
            read: image.read,
            table: writer_table.iter().map(|&v| i64::from(v)).collect(),
        }
    }

    /// The read's common result at this row: the picked position (1-based, 0 for none) and the row
    /// it writes. Mutates `history` exactly as [`Self::serve`] does; `x` is not written.
    pub fn read_out(
        &self,
        history: &mut Q31ReadHistory,
        token: u32,
        x: &[i32],
    ) -> Result<ReadOut, StackError> {
        let x_in: Vec<i64> = x.iter().map(|&v| i64::from(v)).collect();
        let (pick, out) = self
            .read
            .step(history, &x_in, token, &self.table)
            .map_err(|e| StackError::Numerics(format!("q31.read: {e}")))?;
        if out.iter().any(|&w| i32::try_from(w).is_err()) {
            return Err(StackError::Numerics(
                "q31.read: an out word of +2^31 does not fit the residual".to_owned(),
            ));
        }
        Ok(ReadOut::first_pick(
            pick,
            out.iter().map(|&word| i128::from(word)).collect(),
        ))
    }

    /// Read into `x` at this row: `x` widened, stepped, and written back. Refuses an out word that
    /// does not fit i32 (`+2^31`); the history then holds this row, so the session is reset before
    /// it serves again.
    pub fn serve(
        &self,
        history: &mut Q31ReadHistory,
        trace: Option<&mut Vec<Q31ReadStep>>,
        token: u32,
        x: &mut [i32],
    ) -> Result<(), StackError> {
        let x_in: Vec<i64> = x.iter().map(|&v| i64::from(v)).collect();
        // The served read consumes the common result: the pick and the row it writes.
        let result = self.read_out(history, token, x)?;
        let payload = result
            .payload
            .as_deref()
            .expect("a first-pick read writes a row");
        for (x, &word) in x.iter_mut().zip(payload) {
            *x = i32::try_from(word).expect("read_out admitted every out word as i32");
        }
        if let Some(trace) = trace {
            trace.push(Q31ReadStep {
                pick: result.pick(),
                x_in,
                out: payload.iter().map(|&word| word as i64).collect(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::format::Container;
    use super::super::synthetic::{synthetic_fbs_trunk_with, FbsSynth, FBS_SYNTH_SMALL};
    use super::super::{export_fbs_trunk_uorlut01, IntegerStackModel, NativeStep, StackError};
    use super::*;

    /// A prompt with repeats, so later rows read back to earlier ones.
    const PROMPT: [u32; 16] = [5, 9, 5, 9, 17, 5, 9, 17, 3, 5, 9, 17, 3, 5, 200, 9];

    fn plain_trunk() -> Vec<u8> {
        let shape = FbsSynth {
            context: 24,
            ..FBS_SYNTH_SMALL
        };
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(20261115, shape)).expect("export")
    }

    /// Seeded words in `-limit..=limit`.
    fn words(seed: u64, count: usize, limit: i64) -> Vec<i64> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((state >> 11) as i64).rem_euclid(2 * limit + 1) - limit
            })
            .collect()
    }

    fn gates(seed: u64, width: usize) -> Vec<[i64; 4]> {
        words(seed, width, 1 << 30)
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect()
    }

    /// A seeded read whose weights land a Q12 row well inside the residual at gain `g`.
    fn drawn_read(seed: u64, width: usize, g: u32) -> Q31FirstPickRead {
        let limit = 1i64 << (31 - g.min(27));
        Q31FirstPickRead::new(
            gates(seed, width),
            gates(seed ^ 0x9e37, width),
            words(seed ^ 7, width, limit),
            g,
        )
        .expect("an admitted read")
    }

    fn with_read(plain: &[u8], depth: usize, read: Q31FirstPickRead) -> Vec<u8> {
        super::super::add_q31_read_uorlut01(plain, &Q31SessionRead::new(depth, read))
            .expect("the read is added")
    }

    /// Serve `PROMPT` with the trace on: every step's native output and the read's trace.
    fn serve(bytes: &[u8]) -> (Vec<NativeStep>, Vec<Q31ReadStep>) {
        let model = IntegerStackModel::parse(bytes).expect("parse");
        let mut session = model.session();
        session.set_q31_read_trace(true).expect("trace");
        let steps = PROMPT
            .iter()
            .map(|&t| session.step_native(t).expect("step"))
            .collect();
        (steps, session.q31_read_trace().expect("on").to_vec())
    }

    /// The refusal's message, so a test names which rule refused and not only that one did.
    fn refusal<T>(result: Result<T, StackError>) -> String {
        match result {
            Ok(_) => panic!("expected a refusal"),
            Err(e) => e.to_string(),
        }
    }

    fn writer_table(bytes: &[u8]) -> Vec<i64> {
        let container = Container::parse(bytes).expect("parse");
        let table = container.table_i16("writer.table").expect("table");
        table.into_iter().map(i64::from).collect()
    }

    #[test]
    fn a_served_q31_read_equals_the_batch_operator_on_its_own_stream() {
        let plain = plain_trunk();
        let table = writer_table(&plain);
        let layers = Container::parse(&plain).expect("parse").shape.layers();
        let model = IntegerStackModel::parse(&plain).expect("parse");
        let mut session = model.session();
        let unread: Vec<NativeStep> = PROMPT
            .iter()
            .map(|&t| session.step_native(t).expect("step"))
            .collect();
        assert!(layers > 1, "the depths below differ");
        let mut streams = Vec::new();
        for depth in [1, layers] {
            for (seed, g) in [(11u64, 0u32), (12, 4), (13, 31)] {
                let read = drawn_read(seed, 32, g);
                let (steps, trace) = serve(&with_read(&plain, depth, read.clone()));
                streams.push((
                    depth,
                    seed,
                    trace.iter().map(|s| s.x_in.clone()).collect::<Vec<_>>(),
                ));
                assert_ne!(
                    steps, unread,
                    "depth {depth} g {g}: the read's rows reach the step"
                );
                assert_eq!(
                    trace.len(),
                    PROMPT.len(),
                    "depth {depth} g {g}: one read a step"
                );
                let stream: Vec<i64> = trace.iter().flat_map(|s| s.x_in.clone()).collect();
                let want = read
                    .apply(&stream, &PROMPT, &table, 1, PROMPT.len())
                    .expect("apply");
                let picks: Vec<u32> = trace.iter().map(|s| s.pick).collect();
                let out: Vec<i64> = trace.iter().flat_map(|s| s.out.clone()).collect();
                assert_eq!(picks, want.picks, "depth {depth} g {g}: picks");
                assert_eq!(out, want.out, "depth {depth} g {g}: out words");
                let past = picks
                    .iter()
                    .enumerate()
                    .skip(1)
                    .any(|(t, &p)| p as usize != t);
                assert!(
                    past,
                    "depth {depth} g {g}: some row reads past its predecessor: {picks:?}"
                );
                assert!(out != stream, "depth {depth} g {g}: the read writes");
            }
        }
        // The read sits after layer `depth - 1`: the residual it reads differs between depths.
        for (depth, seed, stream) in streams.iter().filter(|(d, _, _)| *d == 1) {
            let (_, _, deeper) = streams
                .iter()
                .find(|(d, s, _)| *d == layers && s == seed)
                .expect("the deeper read");
            assert_ne!(
                stream, deeper,
                "seed {seed}: depth {depth} reads another residual than depth {layers}"
            );
        }
    }

    #[test]
    fn a_q31_read_with_zero_weights_serves_the_plain_trunk() {
        let plain = plain_trunk();
        let read =
            Q31FirstPickRead::new(gates(21, 32), gates(22, 32), vec![0; 32], 4).expect("read");
        let (with, _) = serve(&with_read(&plain, 1, read));
        let model = IntegerStackModel::parse(&plain).expect("parse");
        let mut session = model.session();
        let without: Vec<NativeStep> = PROMPT
            .iter()
            .map(|&t| session.step_native(t).expect("step"))
            .collect();
        assert_eq!(with, without);
        assert!(
            session.set_q31_read_trace(true).is_err(),
            "a plain trunk has no read to trace"
        );
    }

    #[test]
    fn the_stage_timed_step_serves_the_q31_read() {
        let bytes = with_read(&plain_trunk(), 2, drawn_read(31, 32, 4));
        let model = IntegerStackModel::parse(&bytes).expect("parse");
        let (mut plain, mut timed) = (model.session(), model.session());
        let mut ns = Vec::new();
        for &t in &PROMPT {
            let want = plain.step_native(t).expect("step");
            let (got, _) = timed.measure_fbs_step_stages(t, &mut ns).expect("timed");
            assert_eq!(got, want);
        }
    }

    #[test]
    fn a_q31_read_round_trips_and_damage_is_refused() {
        let plain = plain_trunk();
        let image = Q31SessionRead::new(2, drawn_read(41, 32, 4));
        let bytes = super::super::add_q31_read_uorlut01(&plain, &image).expect("added");
        assert_eq!(
            read_q31_read_uorlut01(&bytes).expect("read"),
            Some(image.clone())
        );
        assert_eq!(read_q31_read_uorlut01(&plain).expect("read"), None);
        let meta = super::super::inspect_uorlut01(&bytes).expect("inspect");
        assert_eq!(meta.schema, super::super::STACK_FBS_Q31_READ_SCHEMA);
        assert_eq!(meta.q31_read, Some(image.record()));
        let plain_meta =
            serde_json::to_value(super::super::inspect_uorlut01(&plain).expect("inspect"))
                .expect("json");
        assert!(
            plain_meta.get("q31_read").is_none(),
            "a stack without a read inspects as before"
        );
        // Every earlier section keeps its bytes; adding again gives the same artifact; the read's
        // own bytes come back unchanged, and a re-added image serves the same prompt.
        assert_eq!(
            super::super::add_q31_read_uorlut01(&plain, &image).expect("again"),
            bytes
        );
        let back = read_q31_read_uorlut01(&bytes).expect("read").expect("some");
        assert_eq!(back.read().to_bytes(), image.read().to_bytes());
        let again = super::super::add_q31_read_uorlut01(&plain, &back).expect("re-added");
        assert_eq!(serve(&again), serve(&bytes));

        let add = |bytes: &[u8], depth: usize, read: Q31FirstPickRead| {
            super::super::add_q31_read_uorlut01(bytes, &Q31SessionRead::new(depth, read))
        };
        let added: [(&str, Vec<u8>, usize, usize, &str); 4] = [
            (
                "a read is added once",
                bytes.clone(),
                1,
                32,
                "added to a stack without one",
            ),
            ("depth 0", plain.clone(), 0, 32, "depth is from 1"),
            (
                "depth past the layers",
                plain.clone(),
                3,
                32,
                "depth is from 1",
            ),
            (
                "another width",
                plain.clone(),
                1,
                16,
                "as wide as the stack",
            ),
        ];
        for (name, onto, depth, width, rule) in added {
            let message = refusal(add(&onto, depth, drawn_read(42, width, 4)));
            assert!(message.contains(rule), "{name}: {message}");
        }

        // Parse refusals: every table byte is bound by the digest, and the header by its schema.
        let declared = u64::from_le_bytes(bytes[8..16].try_into().expect("eight")) as usize;
        let header: serde_json::Value =
            serde_json::from_slice(&bytes[16..16 + declared]).expect("header");
        let data_start = (16 + declared).div_ceil(64) * 64;
        let tables = header["tables"].as_array().expect("tables");
        let span = &tables
            .iter()
            .find(|t| t["name"] == Q31_READ_TABLE)
            .expect("table")["span"];
        let offset = data_start + span["offset"].as_u64().expect("offset") as usize;
        let len = span["bytes"].as_u64().expect("bytes") as usize;
        for at in [0, 9, len / 2, len - 1] {
            let mut damaged = bytes.clone();
            damaged[offset + at] ^= 1;
            let message = refusal(IntegerStackModel::parse(&damaged));
            assert!(
                message.contains("q31.read"),
                "a flipped byte at {at}: {message}"
            );
        }
        // An address record from a real address stack, to set beside the read in the header.
        let layers = Container::parse(&plain).expect("parse").shape.layers();
        let address_image = super::super::AddressImage::checked(
            super::super::AddressArm::E8,
            0,
            32,
            vec![vec![0; 32 / 8 * 240 * 32]; layers],
        )
        .expect("an address table");
        let addressed =
            super::super::add_address_uorlut01(&plain, &address_image).expect("address");
        let addressed_len =
            u64::from_le_bytes(addressed[8..16].try_into().expect("eight")) as usize;
        let addressed_header: serde_json::Value =
            serde_json::from_slice(&addressed[16..16 + addressed_len]).expect("header");
        let address_record = addressed_header["address"].clone();
        assert!(
            address_record.is_object(),
            "the address stack carries its record"
        );
        let rewrite = |edit: &dyn Fn(&mut serde_json::Value)| {
            let mut h = header.clone();
            edit(&mut h);
            let hb = serde_json::to_vec(&h).expect("json");
            let mut out = Vec::new();
            out.extend_from_slice(&bytes[..8]);
            out.extend_from_slice(&(hb.len() as u64).to_le_bytes());
            out.extend_from_slice(&hb);
            out.resize(out.len().div_ceil(64) * 64, 0);
            out.extend_from_slice(&bytes[data_start..]);
            IntegerStackModel::parse(&out).map(|_| ())
        };
        assert!(
            rewrite(&|_| {}).is_ok(),
            "the rewrite itself keeps a valid artifact"
        );
        type HeaderEdit<'a> = &'a dyn Fn(&mut serde_json::Value);
        let beside_address = |h: &mut serde_json::Value| h["address"] = address_record.clone();
        let refused: [(&str, HeaderEdit, &str); 10] = [
            (
                "/14 without the record",
                &|h| {
                    h.as_object_mut().expect("object").remove("q31_read");
                },
                "schema",
            ),
            (
                "the record under /12",
                &|h| h["schema"] = "uor-r4.lut-stack/12".into(),
                "schema",
            ),
            (
                "another table name",
                &|h| h["q31_read"]["table"] = "q31.state".into(),
                "in the q31.read table",
            ),
            (
                "a record without its table",
                &|h| {
                    let tables = h["tables"].as_array_mut().expect("tables");
                    tables.retain(|t| t["name"] != Q31_READ_TABLE);
                },
                "q31.read",
            ),
            (
                "a byte length the table does not hold",
                &|h| h["q31_read"]["bytes"] = (len - 8).into(),
                "q31.read",
            ),
            (
                "depth 0",
                &|h| h["q31_read"]["depth"] = 0.into(),
                "depth is from 1",
            ),
            (
                "a depth past the layers",
                &|h| h["q31_read"]["depth"] = 3.into(),
                "depth is from 1",
            ),
            (
                "a width other than the stack's",
                &|h| h["q31_read"]["width"] = 16.into(),
                "as wide as the stack",
            ),
            (
                "a wrong digest",
                &|h| h["q31_read"]["sha256"] = "00".into(),
                "sha256",
            ),
            (
                "a read beside an address table",
                &beside_address,
                "no other optional record",
            ),
        ];
        for (name, edit, rule) in refused {
            let message = refusal(rewrite(edit));
            assert!(message.contains(rule), "{name}: {message}");
        }
        assert!(
            rewrite(&|h| h["q31_read"]["depth"] = 1.into()).is_ok(),
            "the depth is the record's"
        );
    }

    #[test]
    fn a_planted_q31_read_fault_changes_the_served_read() {
        let plain = plain_trunk();
        // One weight word moved by one LSB at g = 31 adds the table entry itself to every read.
        let base = drawn_read(52, 32, 31);
        let (steps, trace) = serve(&with_read(&plain, 2, base.clone()));
        let mut weights = base.weights().to_vec();
        weights[0] += 1;
        let faulted = Q31FirstPickRead::new(
            base.query_gates().to_vec(),
            base.key_gates().to_vec(),
            weights,
            31,
        )
        .expect("read");
        let (fsteps, ftrace) = serve(&with_read(&plain, 2, faulted));
        assert_ne!(ftrace, trace, "a 1-LSB weight fault changes an out word");
        assert_ne!(fsteps, steps, "and the served step");
        // One key-gate word moved by `2^k`, for each of the 32 words: a fault that moves a rounded
        // address changes a pick, and then an out word and the served step. Measured: no 1-LSB
        // key-gate fault reaches the address at residual magnitudes (each gate product is
        // rounded by 2^31), so the replay detects a key-gate fault only from the delta printed.
        let base = drawn_read(51, 32, 4);
        let (steps, trace) = serve(&with_read(&plain, 2, base.clone()));
        let largest = trace
            .iter()
            .flat_map(|s| s.x_in.iter())
            .map(|w| w.unsigned_abs())
            .max();
        eprintln!(
            "largest |x_in| word at the read: {largest:?} (2^31 = {})",
            1u64 << 31
        );
        let mut first_moving = None;
        for k in 0..31 {
            let mut moved = 0usize;
            for word in 0..32 {
                let mut keys = base.key_gates().to_vec();
                keys[word / 4][word % 4] += 1i64 << k;
                if !keys.iter().flatten().all(|&w| w.abs() <= 1 << 31) {
                    continue;
                }
                let weights = base.weights().to_vec();
                let faulted = Q31FirstPickRead::new(base.query_gates().to_vec(), keys, weights, 4)
                    .expect("read");
                let (fsteps, ftrace) = serve(&with_read(&plain, 2, faulted));
                if ftrace != trace {
                    moved += 1;
                    assert_ne!(
                        fsteps, steps,
                        "2^{k} word {word}: a changed read changes the step"
                    );
                }
            }
            eprintln!("key-gate faults of 2^{k} that change the served read: {moved} of 32");
            if moved > 0 {
                first_moving = Some(k);
                break;
            }
        }
        // Some key-gate fault inside the 2^31 bound must reach the served read; which `k` first
        // does is a measurement, printed above, not a bar.
        assert!(
            first_moving.is_some(),
            "no key-gate fault of 2^0..2^30 moved the read"
        );
    }

    #[test]
    fn a_q31_read_session_resets_and_refuses_restore() {
        let bytes = with_read(&plain_trunk(), 1, drawn_read(61, 32, 4));
        let model = IntegerStackModel::parse(&bytes).expect("parse");
        let mut session = model.session();
        let first: Vec<NativeStep> = PROMPT
            .iter()
            .map(|&t| session.step_native(t).expect("step"))
            .collect();
        let saved = session.save_state();
        let message = refusal(session.restore_state(&saved));
        assert!(
            message.contains("its read history is not saved"),
            "{message}"
        );
        session.reset();
        let again: Vec<NativeStep> = PROMPT
            .iter()
            .map(|&t| session.step_native(t).expect("step"))
            .collect();
        assert_eq!(first, again);
    }

    #[test]
    fn the_read_out_is_the_served_step() {
        let plain = plain_trunk();
        let container = Container::parse(&plain).expect("parse");
        let table = container
            .table_i16("writer.table")
            .expect("the writer table");
        let read = drawn_read(11, 32, 0);
        let served = ServedQ31Read::new(Q31SessionRead::new(1, read), &table);
        let rows = [words(31, 32, 1 << 20), words(32, 32, 1 << 20)];
        let mut by_read = Q31ReadHistory::new();
        let mut by_serve = Q31ReadHistory::new();
        for (token, row) in [5u32, 9u32].iter().zip(rows.iter()) {
            let x: Vec<i32> = row.iter().map(|&v| v as i32).collect();
            let out = served
                .read_out(&mut by_read, *token, &x)
                .expect("the common result");
            assert_eq!(out.check(), Ok(()));
            let mut written = x.clone();
            served
                .serve(&mut by_serve, None, *token, &mut written)
                .expect("the served step");
            assert_eq!(
                out.payload.expect("a row"),
                written.iter().map(|&v| i128::from(v)).collect::<Vec<_>>()
            );
            assert_eq!(out.support.len(), usize::from(!out.support.is_empty()));
        }
        // The histories are equivalent if a further row produces the same result.
        let x: Vec<i32> = words(33, 32, 1 << 20).iter().map(|&v| v as i32).collect();
        let by_read_out = served.read_out(&mut by_read, 17, &x).expect("read_out");
        let mut written = x.clone();
        served
            .serve(&mut by_serve, None, 17, &mut written)
            .expect("serve");
        assert_eq!(
            by_read_out.payload.expect("a row"),
            written.iter().map(|&v| i128::from(v)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_q31_read_out_word_past_the_residual_is_refused() {
        // Every weight at 2^31 with g = 31 adds `2^31 * e`: a read row whose table entry is
        // positive saturates to +2^31, which the i32 residual cannot hold.
        let read = Q31FirstPickRead::new(gates(71, 32), gates(72, 32), vec![1 << 31; 32], 31)
            .expect("read");
        let model = IntegerStackModel::parse(&with_read(&plain_trunk(), 1, read)).expect("parse");
        let mut session = model.session();
        match PROMPT.iter().find_map(|&t| session.step_native(t).err()) {
            Some(StackError::Numerics(message)) => assert!(message.contains("+2^31"), "{message}"),
            other => panic!("expected the out-word refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_refused_q31_read_step_refuses_every_step_until_reset() {
        // The refused step has already written its layer rows and the read's history, so the
        // session refuses to serve past it rather than serve from a half-advanced state.
        let read = Q31FirstPickRead::new(gates(71, 32), gates(72, 32), vec![1 << 31; 32], 31)
            .expect("read");
        let model = IntegerStackModel::parse(&with_read(&plain_trunk(), 1, read)).expect("parse");
        let mut session = model.session();
        let mut served = Vec::new();
        let at = loop {
            let t = PROMPT[served.len()];
            match session.step_native(t) {
                Ok(step) => served.push(step),
                Err(StackError::Numerics(message)) => {
                    assert!(message.contains("+2^31"), "{message}");
                    break served.len();
                }
                Err(other) => panic!("expected the out-word refusal, got {other:?}"),
            }
        };
        assert!(
            at > 0,
            "some steps serve before the refusal, so reset has steps to repeat"
        );
        for &t in &PROMPT[..3] {
            let message = refusal(session.step_native(t));
            assert!(message.contains("reset the session"), "{message}");
            let mut ns = Vec::new();
            let message = refusal(session.measure_fbs_step_stages(t, &mut ns));
            assert!(message.contains("reset the session"), "timed: {message}");
        }
        session.reset();
        let again: Vec<NativeStep> = PROMPT[..at]
            .iter()
            .map(|&t| session.step_native(t).expect("served again after reset"))
            .collect();
        assert_eq!(again, served, "reset serves the same steps");
        match session.step_native(PROMPT[at]) {
            Err(StackError::Numerics(message)) => assert!(message.contains("+2^31"), "{message}"),
            other => panic!("the same step is refused by its own rule again, got {other:?}"),
        }
    }

    /// claude-384: pair rows are not served beside a native Q31 read, whichever is added first,
    /// and a header that names both is refused by that rule.
    #[test]
    fn pair_rows_are_not_served_with_a_q31_read() {
        let plain = plain_trunk();
        let shape = Container::parse(&plain).expect("parse").shape;
        let primes = crate::fbs_pair_rows::token_primes(shape.vocab).expect("the vocabulary");
        let marks = super::super::PairMarks {
            user: vec![1, 2],
            assistant: vec![3, 4],
        };
        let image = super::super::PairRowsImage::checked(
            shape.vocab,
            shape.width,
            marks,
            vec![primes[1] * primes[2]],
            vec![7; shape.width],
        )
        .expect("pair rows of the stack's size");
        let read = with_read(&plain, 2, drawn_read(51, 32, 4));
        let rows = super::super::add_pair_rows_uorlut01(&plain, &image).expect("the rows are added");
        let rule = "a native Q31 record and pair rows are not served together";
        let first_read = refusal(super::super::add_pair_rows_uorlut01(&read, &image));
        assert!(first_read.contains(rule), "{first_read}");
        let first_rows = refusal(super::super::add_q31_read_uorlut01(
            &rows,
            &Q31SessionRead::new(2, drawn_read(51, 32, 4)),
        ));
        assert!(first_rows.contains("no other optional record"), "{first_rows}");
        let read_len = u64::from_le_bytes(read[8..16].try_into().expect("eight")) as usize;
        let read_header: serde_json::Value =
            serde_json::from_slice(&read[16..16 + read_len]).expect("header");
        let len = u64::from_le_bytes(rows[8..16].try_into().expect("eight")) as usize;
        let mut header: serde_json::Value = serde_json::from_slice(&rows[16..16 + len]).expect("header");
        header["q31_read"] = read_header["q31_read"].clone();
        let hb = serde_json::to_vec(&header).expect("json");
        let mut both = rows[..8].to_vec();
        both.extend_from_slice(&(hb.len() as u64).to_le_bytes());
        both.extend_from_slice(&hb);
        both.resize(both.len().div_ceil(64) * 64, 0);
        both.extend_from_slice(&rows[(16 + len).div_ceil(64) * 64..]);
        let named = refusal(IntegerStackModel::parse(&both).map(|_| ()));
        assert!(named.contains(rule), "{named}");
    }
}
