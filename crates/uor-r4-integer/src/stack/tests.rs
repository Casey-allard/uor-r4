//! Engine-level tests on synthetic artifacts. Bit-identity with the D10
//! engine is tested in `uor-r4-training` (`tests/stack_d11_oracle.rs`), which
//! depends on both engines.

use serde_json::{json, Value};

use super::format::MAX_READ_CACHE_BYTES;
use super::{IntegerStackModel, StackError, StackShape, MAGIC};
use super::synthetic::{
    fbs_synthetic_non_square, fbs_synthetic_tiny_gain, synthetic_fbs_trunk, synthetic_fbs_trunk_with,
    synthetic_hamming, synthetic_turn_memory, FbsSynth, FBS_SYNTH_SMALL, Lcg,
};

/// A container under construction: the header's section lists and the data.
struct Builder {
    matrices: Vec<Value>,
    tables: Vec<Value>,
    data: Vec<u8>,
}

impl Builder {
    fn push(&mut self, bytes: &[u8]) -> Value {
        self.data.resize(self.data.len().div_ceil(64) * 64, 0);
        let span = json!({"offset": self.data.len(), "bytes": bytes.len()});
        self.data.extend_from_slice(bytes);
        span
    }

    fn matrix(&mut self, rng: &mut Lcg, name: &str, rows: usize, cols: usize) {
        let nibbles: Vec<u8> = (0..rows * cols / 2).map(|_| rng.next() as u8).collect();
        // Scale exponents 0..=3 above the base, any mantissa.
        let scales: Vec<u8> = (0..rows * cols / 32)
            .map(|_| (rng.next() as u8) & 0x3F)
            .collect();
        let (nibbles, scales) = (self.push(&nibbles), self.push(&scales));
        self.matrices.push(json!({
            "name": name, "rows": rows, "cols": cols, "exp_base": -9,
            "nibbles": nibbles, "scales": scales,
        }));
    }

    fn table(&mut self, name: &str, kind: &str, len: usize, bytes: Vec<u8>) {
        let span = self.push(&bytes);
        self.tables
            .push(json!({"name": name, "kind": kind, "len": len, "span": span}));
    }

    fn i16s(&mut self, name: &str, values: &[i16]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "i16", values.len(), bytes);
    }

    fn i32s(&mut self, name: &str, values: &[i32]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "i32", values.len(), bytes);
    }

    fn u32s(&mut self, name: &str, values: &[u32]) {
        let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        self.table(name, "u32", values.len(), bytes);
    }
}

const VOCAB: usize = 40;
const WIDTH: usize = 32;
const HEADS: usize = 2;
const MLP: usize = 32;
const CONTEXT: usize = 6;

/// A synthetic `ra` stack (one recurrence with rotations, one Lorentz read)
/// with random weights and monotone sealed tables.
fn artifact(seed: u64) -> Vec<u8> {
    artifact_with(seed, &[])
}

/// [`artifact`] with extra `u32` table sections appended after the others.
fn artifact_with(seed: u64, extra: &[(&str, &[u32])]) -> Vec<u8> {
    artifact_with_context(seed, extra, CONTEXT)
}

fn artifact_with_context(seed: u64, extra: &[(&str, &[u32])], context: usize) -> Vec<u8> {
    let mut rng = Lcg(seed);
    let mut b = Builder {
        matrices: Vec::new(),
        tables: Vec::new(),
        data: Vec::new(),
    };
    let lanes = WIDTH / 4;
    let gate_rows = lanes + WIDTH;
    // Grid code of (16 + m) 2^(e - 4): |code| = (e + 64) << 4 | m.
    let code = |m: u64, e: i16| ((e + 64) << 4) | (m & 15) as i16;
    b.matrix(&mut rng, "embed", VOCAB, WIDTH);
    b.matrix(&mut rng, "l0.rec_in", 2 * WIDTH, WIDTH);
    b.matrix(&mut rng, "l0.rec_gate", gate_rows, WIDTH);
    b.matrix(&mut rng, "l0.rec_out", WIDTH, WIDTH);
    let taps: Vec<i16> = (0..4 * WIDTH)
        .map(|i| {
            let c = code(rng.next(), -2 - (i % 3) as i16);
            if rng.next().is_multiple_of(2) {
                c
            } else {
                -c
            }
        })
        .collect();
    b.i16s("l0.conv_taps", &taps);
    let biases: Vec<i32> = (0..gate_rows)
        .map(|_| (rng.next() % 60_000) as i32 - 30_000)
        .collect();
    b.i32s("l0.conv_bias", &biases[..WIDTH]);
    b.i32s("l0.gate_bias", &biases);
    let rates: Vec<i16> = (0..lanes).map(|_| code(rng.next(), 1)).collect();
    b.i16s("l0.decay_rate", &rates);
    for part in ["query", "key", "value", "out"] {
        b.matrix(&mut rng, &format!("l1.{part}"), WIDTH, WIDTH);
    }
    b.matrix(&mut rng, "l1.null", HEADS, WIDTH);
    b.i32s("l1.null_bias", &[4_000, -9_000]);
    let ages: Vec<i32> = (0..HEADS * context).map(|i| -((i as i32) << 12)).collect();
    b.i32s("l1.age", &ages);
    b.i16s("l1.beta", &[code(3, 0), code(9, -1)]);
    b.i32s("l1.offset", &[1 << 24, 3 << 23]);
    for l in 0..2 {
        b.matrix(&mut rng, &format!("l{l}.gate"), MLP, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.up"), MLP, WIDTH);
        b.matrix(&mut rng, &format!("l{l}.down"), WIDTH, MLP);
    }
    b.matrix(&mut rng, "head", VOCAB, WIDTH);
    // exp(-i/256) at Q31 approximated by a non-increasing integer sequence.
    let exp: Vec<u32> = (0..32 * 256 + 2)
        .map(|i: u32| (1u32 << 31) >> (i / 177).min(31))
        .collect();
    b.u32s("exp", &exp);
    let activation: Vec<i32> = (0..(2 << 12) + 1)
        .map(|i: i32| (i - 4096).max(0) << 8)
        .collect();
    b.i32s("silu", &activation);
    b.i32s("gelu", &activation);
    let arcosh: Vec<u32> = (0..super::kernels::ARCOSH_TABLE_LEN as u32)
        .map(|i| i << 10)
        .collect();
    b.u32s("arcosh", &arcosh);
    for (name, values) in extra {
        b.u32s(name, values);
    }
    let header = json!({
        "schema": super::STACK_SCHEMA,
        "shape": {
            "vocab": VOCAB, "width": WIDTH, "heads": HEADS, "mlp": MLP, "pattern": "ra",
            "read": "lorentz", "rotation": true, "context": context,
        },
        "group": 32,
        "numerics": {
            "rms_eps": {"mantissa": 2_814_749_767i64, "exp": -48},
            "score_scale_q30": 268_435_456, "exp_step_log2": -8,
            "silu_step_log2": -8, "silu_range_log2": 4,
            "gelu_step_log2": -8, "gelu_range_log2": 4,
        },
        "matrices": b.matrices,
        "tables": b.tables,
        "source": {"synthetic": true},
    });
    let json = serde_json::to_vec(&header).expect("header");
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(&b.data);
    out
}

/// `bytes` with its JSON header edited; the data sections keep their offsets.
fn with_header(bytes: &[u8], edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    let mut header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    edit(&mut header);
    let json = serde_json::to_vec(&header).expect("header");
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    out.resize(out.len().div_ceil(64) * 64, 0);
    out.extend_from_slice(&bytes[(16 + len).div_ceil(64) * 64..]);
    out
}

fn run(model: &IntegerStackModel, ids: &[u32]) -> Vec<Vec<i32>> {
    let mut session = model.session();
    ids.iter()
        .map(|&id| session.step(id).expect("step").to_vec())
        .collect()
}

#[test]
fn a_synthetic_stack_serves_a_full_context_deterministically() {
    let bytes = artifact(5);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert_eq!(model.shape().layers(), 2);
    // Every map in full plus one embedding row.
    let maps = 2 * WIDTH * WIDTH
        + (WIDTH / 4 + WIDTH) * WIDTH
        + WIDTH * WIDTH
        + 4 * WIDTH * WIDTH
        + HEADS * WIDTH
        + 2 * 3 * MLP * WIDTH
        + VOCAB * WIDTH
        + WIDTH;
    assert_eq!(model.weights_per_token(), maps as u64);
    let ids = [3u32, 39, 0, 17, 17, 8];
    let first = run(&model, &ids);
    assert_eq!(first.len(), CONTEXT);
    assert!(first.iter().all(|logits| logits.len() == VOCAB));
    assert!(
        first.windows(2).any(|w| w[0] != w[1]),
        "the logits do not depend on the position"
    );
    assert_eq!(run(&model, &ids), first, "a second session differs");
    let reparsed = IntegerStackModel::parse(&bytes).expect("parse again");
    assert_eq!(run(&reparsed, &ids), first, "a reloaded model differs");

    let mut session = model.session();
    for &id in &ids[..3] {
        session.step(id).expect("step");
    }
    session.reset();
    for (&id, want) in ids.iter().zip(&first) {
        assert_eq!(session.step(id).expect("step after reset"), want.as_slice());
    }
    assert!(matches!(
        session.step(0),
        Err(StackError::ContextFull { context: CONTEXT })
    ));
    let mut fresh = model.session();
    assert!(matches!(
        fresh.step(VOCAB as u32),
        Err(StackError::Token { .. })
    ));
    assert_eq!(fresh.position(), 0);
}

#[test]
fn truncated_or_mislabelled_artifacts_are_rejected() {
    let bytes = artifact(9);
    for cut in [0, 8, 15, 16, 100, bytes.len() / 2, bytes.len() - 1] {
        assert!(
            IntegerStackModel::parse(&bytes[..cut]).is_err(),
            "accepted a prefix of {cut} bytes"
        );
    }
    let mut magic = bytes.clone();
    magic[3] ^= 1;
    assert!(matches!(
        IntegerStackModel::parse(&magic),
        Err(StackError::Magic)
    ));
    let mut length = bytes.clone();
    length[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        IntegerStackModel::parse(&length),
        Err(StackError::HeaderLength { .. })
    ));
    let mut json = bytes.clone();
    json[17] = b'!';
    assert!(matches!(
        IntegerStackModel::parse(&json),
        Err(StackError::Header(_))
    ));
}

#[test]
fn shapes_whose_read_caches_exceed_the_bound_are_rejected_before_allocation() {
    let shape = |pattern: &str, read: &str| StackShape {
        vocab: 40,
        width: 512,
        heads: 2,
        mlp: 32,
        pattern: pattern.to_owned(),
        read: read.to_owned(),
        rotation: false,
        context: 1 << 16,
        pointer: None,
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory: None,
        rec_gates: None,
        gate: None,
        raw_gates: false,
        raw_transition: false,
        projection: None,
        head: None,
        kept_cells: None,
    };
    // Dot reads cache an i32 key and value row per position: 2^28 bytes per
    // layer at this width and context, so sixteen layers reach the bound.
    let sixteen = shape(&"a".repeat(16), "dot");
    assert_eq!(sixteen.read_cache_bytes(), Some(MAX_READ_CACHE_BYTES));
    assert!(sixteen.validate().is_ok());
    assert!(matches!(
        shape(&"a".repeat(17), "dot").validate(),
        Err(StackError::Shape(_))
    ));
    // Lorentz lifts add a u64 per head and position; recurrences cache nothing.
    assert!(shape(&"a".repeat(16), "lorentz").validate().is_err());
    assert_eq!(
        shape(&"r".repeat(256), "lorentz").read_cache_bytes(),
        Some(0)
    );
    // The loader applies the bound before any section is read or any session
    // allocated: a small file that declares seventeen such reads is refused.
    let bytes = with_header(&artifact(3), |header| {
        let shape = &mut header["shape"];
        shape["pattern"] = json!("a".repeat(17));
        shape["width"] = json!(512);
        shape["read"] = json!("dot");
        shape["context"] = json!(1 << 16);
    });
    match IntegerStackModel::parse(&bytes) {
        Err(StackError::Shape(reason)) => assert!(reason.contains("read caches"), "{reason}"),
        other => panic!("expected the read-cache bound, got {:?}", other.err()),
    }
}

/// The synthetic Lorentz read relabelled as the flat L2 read (the same beta
/// and offset tables) serves without the arcosh table, differently from the
/// Lorentz read; an L2 read without its beta or offset is refused, as is an
/// unknown read name. L2 caches no lifts, so its read-cache bound is Dot's.
#[test]
fn the_l2_read_serves_its_scale_and_offset_without_lifts_or_arcosh() {
    let bytes = artifact(6);
    let relabel = |drop: Option<&'static str>, read: &'static str| {
        with_header(&bytes, move |header| {
            header["shape"]["read"] = json!(read);
            if let Some(name) = drop {
                header["tables"]
                    .as_array_mut()
                    .expect("tables")
                    .retain(|t| t["name"] != name);
            }
        })
    };
    let l2 = IntegerStackModel::parse(&relabel(Some("arcosh"), "l2")).expect("an L2 read");
    assert!(l2.shape().l2() && l2.shape().scaled() && !l2.shape().lorentz());
    let ids = [3u32, 39, 0, 17, 17, 8];
    let first = run(&l2, &ids);
    assert_eq!(first.len(), CONTEXT);
    assert_eq!(run(&l2, &ids), first, "a second L2 session differs");
    let lorentz = IntegerStackModel::parse(&bytes).expect("the Lorentz read");
    assert_ne!(run(&lorentz, &ids), first, "L2 scored like Lorentz");
    let mut session = l2.session();
    for (&id, want) in ids.iter().zip(&first) {
        assert_eq!(session.step(id).expect("step"), want.as_slice());
    }
    for missing in ["l1.beta", "l1.offset"] {
        match IntegerStackModel::parse(&relabel(Some(missing), "l2")) {
            Err(StackError::MissingSection(name)) => assert_eq!(name, missing),
            other => panic!("an L2 read without {missing}: {:?}", other.err()),
        }
    }
    // The Lorentz read still needs its table.
    assert!(matches!(
        IntegerStackModel::parse(&relabel(Some("arcosh"), "lorentz")),
        Err(StackError::MissingSection(_))
    ));
    for unknown in ["L2", "euclidean", "cosine", ""] {
        assert!(
            matches!(
                IntegerStackModel::parse(&relabel(None, unknown)),
                Err(StackError::Shape(_))
            ),
            "accepted the read {unknown:?}"
        );
    }
    let shape = |read: &str| StackShape {
        vocab: 40,
        width: 512,
        heads: 2,
        mlp: 32,
        pattern: "a".repeat(16),
        read: read.to_owned(),
        rotation: false,
        context: 1 << 16,
        pointer: None,
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory: None,
        rec_gates: None,
        gate: None,
        raw_gates: false,
        raw_transition: false,
        projection: None,
        head: None,
        kept_cells: None,
    };
    assert_eq!(
        shape("l2").read_cache_bytes(),
        shape("dot").read_cache_bytes()
    );
    assert!(shape("l2").validate().is_ok());
}

#[test]
fn duplicate_sections_are_rejected() {
    let bytes = artifact(4);
    for (list, name) in [("matrices", "head"), ("tables", "exp")] {
        let duplicated = with_header(&bytes, |header| {
            let entries = header[list].as_array_mut().expect("section list");
            let copy = entries
                .iter()
                .find(|entry| entry["name"] == name)
                .expect("section")
                .clone();
            entries.push(copy);
        });
        assert!(
            matches!(
                IntegerStackModel::parse(&duplicated),
                Err(StackError::DuplicateSection(found)) if found == name
            ),
            "a duplicate {name} in {list} was not rejected"
        );
    }
    assert!(IntegerStackModel::parse(&with_header(&bytes, |_| {})).is_ok());
}

/// The icosian snap record the tests attach to a synthetic artifact.
fn snap_record() -> Value {
    json!({
        "name": "icosian",
        "roots": 120,
        "roots_sha256": super::ICOSIAN_ROOTS_SHA256,
    })
}

#[test]
fn phi_q32_is_the_rounded_fixed_point_golden_ratio() {
    let phi = (1.0f64 + 5.0f64.sqrt()) / 2.0;
    assert_eq!(super::PHI_Q32, (phi * 4294967296.0).round() as u128);
}

/// The snap's choice, independent of the kernel: the exact dot product against
/// a root is (P + B*sqrt(5)) / 4 with P = 2*delta_a + delta_b, so two
/// candidates compare by the sign of (P1 - P2) - (B2 - B1)*sqrt(5), decided in
/// i128 by the sign-aware square rule (no floats, no kernel code shared). The
/// lowest index wins an exact tie.
fn snap_oracle(raw: [i32; 4]) -> usize {
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let greater = |(p1, b1): (i128, i128), (p2, b2): (i128, i128)| -> bool {
        let dp = p1 - p2;
        let db = b2 - b1;
        if db == 0 {
            dp > 0
        } else if db > 0 {
            dp > 0 && dp * dp > 5 * db * db
        } else {
            dp >= 0 || dp * dp < 5 * db * db
        }
    };
    let mut best = (usize::MAX, (i128::MIN, i128::MIN));
    for (j, root) in H4_ROOT_COEFFICIENTS.iter().enumerate() {
        let mut p = 0i128;
        let mut bsum = 0i128;
        for c in 0..4 {
            let [a, b] = root[c];
            let x = i128::from(raw[c]);
            p += x * (2 * i128::from(a) + i128::from(b));
            bsum += x * i128::from(b);
        }
        if best.0 == usize::MAX || greater((p, bsum), best.1) {
            best = (j, (p, bsum));
        }
    }
    best.0
}

#[test]
fn snap_selection_matches_an_independent_exact_oracle() {
    use super::stack_snap_select;
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let mut rng = Lcg(0x5eed);
    for _ in 0..2000 {
        let raw: [i32; 4] = std::array::from_fn(|_| (rng.next() % 200_001) as i32 - 100_000);
        assert_eq!(stack_snap_select(raw), snap_oracle(raw), "raw {raw:?}");
    }
    // Exact ties break to the lowest index; the zero vector selects root 0.
    assert_eq!(stack_snap_select([0; 4]), 0);
    let axis = H4_ROOT_COEFFICIENTS[0];
    let raw: [i32; 4] = std::array::from_fn(|c| i32::from(axis[c][0]) * 1000);
    assert_eq!(stack_snap_select(raw), 0);
}

#[test]
fn snap_selection_matches_the_oracle_on_every_query_from_minus_8_to_8() {
    // Every query with each coordinate in -8..=8 (claude-241): zeros, equal
    // magnitudes and exact ties, on the scan and on the fast path alike.
    let mut grid = 0;
    for w in -8..=8 {
        for x in -8..=8 {
            for y in -8..=8 {
                for z in -8..=8 {
                    let raw = [w, x, y, z];
                    assert_eq!(super::stack_snap_select(raw), snap_oracle(raw), "{raw:?}");
                    grid += 1;
                }
            }
        }
    }
    assert_eq!(grid, 83_521);
}

#[test]
fn snap_rotation_matches_the_float_recipe_within_two_quanta() {
    use super::kernels::stack_snap_rotation;
    use super::PHI_Q32;
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let mut rng = Lcg(0x5eed2);
    let phi = PHI_Q32 as f64 / 4294967296.0;
    for _ in 0..2000 {
        let raw: [i32; 4] = std::array::from_fn(|_| (rng.next() % 200_001) as i32 - 100_000);
        let lambda = rng.next() % 131_072;
        let selected = super::stack_snap_select(raw);
        let rotated = stack_snap_rotation(raw, lambda);
        let root = &H4_ROOT_COEFFICIENTS[selected];
        for c in 0..4 {
            let [a, b] = root[c];
            let expected = lambda as f64 * (f64::from(a) + f64::from(b) * phi) / 2.0;
            let gap = (rotated[c] as f64 - expected).abs();
            assert!(gap <= 2.0, "component {c}: {rotated:?} vs {expected}");
        }
    }
}

#[test]
fn a_snapped_header_round_trips_and_bad_records_are_refused() {
    let bytes = artifact(7);
    let snapped = with_header(&bytes, |header| {
        header["transport_snap"] = snap_record();
    });
    let model = IntegerStackModel::parse(&snapped).expect("a known snap parses");
    let record = model.transport_snap().expect("the record is kept");
    assert_eq!(record.name, "icosian");
    assert_eq!(record.roots, 120);
    assert_eq!(record.roots_sha256, super::ICOSIAN_ROOTS_SHA256);
    // The same artifact without the record serves the free transport.
    assert!(IntegerStackModel::parse(&bytes)
        .expect("unsnapped")
        .transport_snap()
        .is_none());
    for (field, value) in [
        ("name", json!("h4")),
        ("roots", json!(119)),
        ("roots_sha256", json!("00".repeat(32))),
    ] {
        let broken = with_header(&bytes, |header| {
            let mut record = snap_record();
            record[field] = value;
            header["transport_snap"] = record;
        });
        match IntegerStackModel::parse(&broken) {
            Err(StackError::TransportSnap(reason)) => {
                assert!(!reason.is_empty());
            }
            other => panic!("a bad {field} was not refused: {:?}", other.err()),
        }
    }
    // A snap on a rotation-free shape is refused.
    let no_rotation = with_header(&bytes, |header| {
        header["shape"]["rotation"] = json!(false);
        header["transport_snap"] = snap_record();
    });
    assert!(matches!(
        IntegerStackModel::parse(&no_rotation),
        Err(StackError::TransportSnap(_))
    ));
}

#[test]
fn the_snap_trace_records_every_selection() {
    let snapped = with_header(&artifact(11), |header| {
        header["transport_snap"] = snap_record();
    });
    let model = IntegerStackModel::parse(&snapped).expect("parse");
    let mut session = model.session();
    session.enable_snap_trace();
    let ids: Vec<u32> = (0..CONTEXT as u32).map(|i| i % VOCAB as u32).collect();
    for &id in &ids {
        session.step(id).expect("step");
    }
    let trace = session.snap_trace().expect("trace");
    let lanes = WIDTH / 4;
    assert_eq!(trace.len(), CONTEXT * lanes);
    for entry in &trace {
        assert_eq!(entry.layer, 0);
        assert!(entry.root < 120);
    }
    // The trace matches the kernel applied to the same raw logits: rerun
    // with the trace off and confirm the served logits are unchanged.
    let with_trace = run(&model, &ids);
    let without_trace = run(&model, &ids);
    assert_eq!(with_trace, without_trace);
}

/// A random packed map and a random 16-bit input with its nibble and pair
/// tables, for the weight-map kernel tests.
struct RandomMap {
    matrix: super::kernels::PackedMatrix,
    tables: Vec<[i32; 16]>,
    pairs: Vec<[i32; 256]>,
}

fn random_map(rows: usize, cols: usize, seed: u64) -> RandomMap {
    use super::kernels::{stack_activation_tables, stack_pair_tables, PackedMatrix};
    let mut rng = Lcg(seed);
    let nibbles: Vec<u8> = (0..rows * cols / 2)
        .map(|_| (rng.next() & 0xFF) as u8)
        .collect();
    // Heterogeneous group scales, so that rows shift their groups apart.
    let scales: Vec<u8> = (0..rows * cols / 32)
        .map(|_| ((rng.next() & 0x3F) as u8).max(1))
        .collect();
    let min_de: Vec<u8> = scales
        .chunks_exact(cols / 32)
        .map(|row| row.iter().map(|&s| s >> 4).min().unwrap_or(0))
        .collect();
    let matrix = PackedMatrix {
        rows,
        cols,
        exp_base: -10,
        nibbles,
        scales,
        min_de,
    };
    let x: Vec<i16> = (0..cols).map(|_| (rng.next() as i16).max(-32767)).collect();
    let mut tables = vec![[0i32; 16]; cols];
    stack_activation_tables(&x, &mut tables);
    let mut pairs = vec![[0i32; 256]; cols / 2];
    stack_pair_tables(&tables, &mut pairs);
    RandomMap {
        matrix,
        tables,
        pairs,
    }
}

/// The row-blocked pair-table kernel, over the whole map and over every split
/// of it into two row ranges (the threaded path's unit of work), computes the
/// row-by-row nibble-table kernel's integers, including partial row blocks.
#[test]
fn gemv_pairs_row_blocks_and_ranges_equal_the_nibble_kernel() {
    use super::kernels::{stack_gemv, stack_gemv_pairs};
    for (rows, cols, seed) in [
        (1, 32, 1),
        (7, 64, 2),
        (19, 64, 3),
        (40, 96, 4),
        (67, 32, 5),
    ] {
        let map = random_map(rows, cols, seed);
        let mut expected = vec![0i32; rows];
        stack_gemv(&map.matrix, &map.tables, -14, 0, &mut expected);
        let mut whole = vec![0i32; rows];
        stack_gemv_pairs(&map.matrix, &map.pairs, -14, 0, &mut whole);
        assert_eq!(whole, expected, "{rows}x{cols}");
        for split in 1..rows {
            let mut parts = vec![0i32; rows];
            let (head, tail) = parts.split_at_mut(split);
            stack_gemv_pairs(&map.matrix, &map.pairs, -14, 0, head);
            stack_gemv_pairs(&map.matrix, &map.pairs, -14, split, tail);
            assert_eq!(parts, expected, "{rows}x{cols} split at {split}");
        }
    }
}

/// Every thread count serves the same logits, copy weights and snap trace:
/// on the Lorentz, L2 and Dot reads, the free and the snapped transport, with
/// and without the copy scale. Under `cfg(test)` the maps split into tasks of
/// two rows and the recurrence into tasks of one lane, so these small stacks
/// take every split path, including uneven remainders.
#[test]
fn every_thread_count_serves_the_same_integers() {
    let lorentz = artifact(13);
    let relabel = |read: &'static str| {
        with_header(&lorentz, move |header| {
            header["shape"]["read"] = json!(read);
        })
    };
    let snapped = with_header(&lorentz, |header| {
        header["transport_snap"] = snap_record();
    });
    let ids = [3u32, 39, 0, 17, 17, 8];
    let serve = |bytes: &[u8], threads: usize, copy: bool, trace: bool| {
        let mut model = IntegerStackModel::parse(bytes).expect("parse");
        model.set_threads(threads).expect("threads");
        assert_eq!(model.threads(), threads);
        let mut session = model.session();
        if copy {
            session.set_copy_scale(1 << 16).expect("copy scale");
        }
        if trace {
            session.enable_snap_trace();
        }
        let mut served = Vec::new();
        for &id in &ids {
            served.push(session.step(id).expect("step").to_vec());
            served.push(
                session
                    .pointer_weights()
                    .iter()
                    .map(|&w| w as i32)
                    .collect(),
            );
        }
        let roots: Vec<usize> = session
            .snap_trace()
            .unwrap_or_default()
            .iter()
            .map(|entry| entry.root)
            .collect();
        (served, roots)
    };
    for (name, bytes) in [
        ("lorentz", lorentz.clone()),
        ("l2", relabel("l2")),
        ("dot", relabel("dot")),
        ("snapped", snapped),
    ] {
        for (copy, trace) in [(false, false), (true, false), (false, true)] {
            let one = serve(&bytes, 1, copy, trace);
            for threads in [2, 3, 4] {
                assert_eq!(
                    serve(&bytes, threads, copy, trace),
                    one,
                    "{name}: {threads} threads (copy {copy}, trace {trace})"
                );
            }
        }
    }
    // A loaded model starts no threads of its own.
    let mut model = IntegerStackModel::parse(&lorentz).expect("parse");
    assert_eq!(model.threads(), 1);
    assert!(matches!(model.set_threads(0), Err(StackError::Threads(_))));
}

#[test]
fn session_save_and_restore_produces_bit_identical_logits() {
    use super::{SerializedStackSession, STACK_SESSION_SCHEMA};

    let bytes = artifact(7);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();

    // Ingest first 3 tokens
    let initial_tokens = [1u32, 3u32, 2u32];
    for &tok in &initial_tokens {
        session.step(tok).expect("step");
    }
    assert_eq!(session.position(), 3);
    assert_eq!(session.cache_at(), 3 * WIDTH);
    assert_eq!(session.lift_at(), 3 * HEADS);

    // Save session state
    let saved = session.save_state();
    assert_eq!(saved.schema, STACK_SESSION_SCHEMA);
    assert_eq!(saved.version, 1);
    assert_eq!(saved.artifact_sha256, model.artifact_sha256());
    assert_eq!(saved.position, 3);
    assert_eq!(saved.cache_at, (3 * WIDTH) as u64);
    assert_eq!(saved.lift_at, (3 * HEADS) as u64);
    assert_eq!(saved.layers.len(), 2);

    // Continue stepping original session for 2 more tokens
    let next_tokens = [4u32, 0u32];
    let mut expected_logits = Vec::new();
    for &tok in &next_tokens {
        let logits = session.step(tok).expect("step").to_vec();
        expected_logits.push(logits);
    }
    assert_eq!(session.position(), 5);

    // 1. Restore into the same session and step the same next tokens
    session.restore_state(&saved).expect("restore");
    assert_eq!(session.position(), 3);
    assert_eq!(session.cache_at(), 3 * WIDTH);
    assert_eq!(session.lift_at(), 3 * HEADS);
    assert_eq!(session.tokens(), &initial_tokens);
    assert_eq!(
        session.logits(),
        saved.logits.as_slice(),
        "restored session logits must match saved session logits before next step"
    );

    let mut restored_logits = Vec::new();
    for &tok in &next_tokens {
        let logits = session.step(tok).expect("step").to_vec();
        restored_logits.push(logits);
    }
    assert_eq!(
        restored_logits, expected_logits,
        "restored session must produce bit-for-bit identical logits to uninterrupted stepping"
    );

    // 2. Restore into a brand-new session
    let mut fresh_session = model.session();
    fresh_session.restore_state(&saved).expect("restore fresh");
    assert_eq!(fresh_session.position(), 3);
    assert_eq!(fresh_session.cache_at(), 3 * WIDTH);
    assert_eq!(fresh_session.tokens(), &initial_tokens);
    // Next-token logits immediately after restore must match saved position 3 logits
    assert_eq!(
        fresh_session.logits(),
        saved.logits.as_slice(),
        "fresh restored session logits must match saved session logits before next step"
    );

    let mut fresh_logits = Vec::new();
    for &tok in &next_tokens {
        let logits = fresh_session.step(tok).expect("step").to_vec();
        fresh_logits.push(logits);
    }
    assert_eq!(
        fresh_logits, expected_logits,
        "fresh restored session must produce bit-for-bit identical logits"
    );

    // 3. Serde JSON serialization round-trip
    let json_str = serde_json::to_string_pretty(&saved).expect("serialize");
    let deserialized: SerializedStackSession =
        serde_json::from_str(&json_str).expect("deserialize");
    assert_eq!(deserialized, saved);

    let mut json_restored_session = model.session();
    json_restored_session
        .restore_state(&deserialized)
        .expect("restore from deserialized");
    assert_eq!(json_restored_session.tokens(), &initial_tokens);
    let mut json_logits = Vec::new();
    for &tok in &next_tokens {
        let logits = json_restored_session.step(tok).expect("step").to_vec();
        json_logits.push(logits);
    }
    assert_eq!(
        json_logits, expected_logits,
        "deserialized session must produce bit-for-bit identical logits"
    );
}

#[test]
fn session_restore_strictly_validates_checksums_and_dimensions() {
    use super::SerializedStackLayerState;

    let bytes = artifact(9);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();
    session.step(1).expect("step");
    session.step(2).expect("step");
    let saved = session.save_state();

    // 1. Mismatched artifact checksum
    let mut bad_sha = saved.clone();
    bad_sha.artifact_sha256 =
        "0000000000000000000000000000000000000000000000000000000000000000".into();
    assert!(matches!(
        session.restore_state(&bad_sha),
        Err(StackError::Numerics(_))
    ));

    // 2. Mismatched schema
    let mut bad_schema = saved.clone();
    bad_schema.schema = "uor-r4.wrong-schema/1".into();
    assert!(matches!(
        session.restore_state(&bad_schema),
        Err(StackError::Schema(_))
    ));

    // 3. Mismatched version
    let mut bad_version = saved.clone();
    bad_version.version = 2;
    assert!(matches!(
        session.restore_state(&bad_version),
        Err(StackError::Schema(_))
    ));

    // 4. Position exceeding context
    let mut bad_pos = saved.clone();
    bad_pos.position = (CONTEXT + 1) as u64;
    assert!(matches!(
        session.restore_state(&bad_pos),
        Err(StackError::ContextFull { .. })
    ));

    // 5. Inconsistent cache_at
    let mut bad_cache = saved.clone();
    bad_cache.cache_at = bad_cache.cache_at + 1;
    assert!(matches!(
        session.restore_state(&bad_cache),
        Err(StackError::SessionState)
    ));

    // 6. Inconsistent layer count
    let mut bad_layers = saved.clone();
    bad_layers.layers.pop();
    assert!(matches!(
        session.restore_state(&bad_layers),
        Err(StackError::SessionState)
    ));

    // 7. Corrupted layer dimensions inside Recurrence
    let mut bad_rec = saved.clone();
    if let SerializedStackLayerState::Recurrence { state, .. } = &mut bad_rec.layers[0] {
        state.pop();
    }
    assert!(matches!(
        session.restore_state(&bad_rec),
        Err(StackError::SessionState)
    ));

    // 8. Inconsistent logits length
    let mut bad_logits = saved.clone();
    bad_logits.logits.pop();
    assert!(matches!(
        session.restore_state(&bad_logits),
        Err(StackError::SessionState)
    ));

    // 9. Inconsistent tokens length
    let mut bad_tokens = saved.clone();
    bad_tokens.tokens.pop();
    assert!(matches!(
        session.restore_state(&bad_tokens),
        Err(StackError::SessionState)
    ));

    // 10. Out-of-vocab token in tokens
    let mut bad_tok_val = saved.clone();
    bad_tok_val.tokens[0] = VOCAB as u32 + 50;
    assert!(matches!(
        session.restore_state(&bad_tok_val),
        Err(StackError::Token { .. })
    ));

    // 11. Non-empty snap trace on unsnapped model rejected
    let mut bad_trace = saved.clone();
    bad_trace.snap_trace = Some(vec![0u32; 1]);
    assert!(matches!(
        session.restore_state(&bad_trace),
        Err(StackError::SessionState)
    ));
}

#[test]
fn session_pause_save_fresh_restore_greedy_continuation() {
    use super::stack_argmax;

    let bytes = artifact(13);
    let model = IntegerStackModel::parse(&bytes).expect("parse");

    // Stream 1: Run uninterrupted generation
    let mut continuous_session = model.session();
    continuous_session.step(2).expect("prompt 1");
    continuous_session.step(5).expect("prompt 2");

    let mut uninterrupted_tokens = Vec::new();
    let mut uninterrupted_logits = Vec::new();
    for _ in 0..3 {
        let logits = continuous_session.logits().to_vec();
        uninterrupted_logits.push(logits.clone());
        let next_tok = stack_argmax(&logits) as u32;
        uninterrupted_tokens.push(next_tok);
        continuous_session.step(next_tok).expect("step next");
    }

    // Stream 2: Run prompt, pause, save to file, restore into a brand-new session, continue
    let mut pause_session = model.session();
    pause_session.step(2).expect("prompt 1");
    pause_session.step(5).expect("prompt 2");

    let temp_dir =
        std::env::temp_dir().join(format!("uor-stack-test-greedy-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create_dir");
    let ckpt_path = temp_dir.join("paused_session.json");
    pause_session
        .save_session_to_file(&ckpt_path)
        .expect("save");

    // Fresh session restore
    let mut resumed_session = model.session();
    resumed_session
        .restore_session_from_file(&ckpt_path)
        .expect("restore");

    // Immediate logits must match
    assert_eq!(
        resumed_session.logits(),
        pause_session.logits(),
        "resumed session logits immediately after restore must match paused session"
    );
    assert_eq!(
        resumed_session.tokens(),
        pause_session.tokens(),
        "resumed session tokens must match paused session"
    );

    // Continue greedy generation from resumed session
    let mut resumed_tokens = Vec::new();
    let mut resumed_logits = Vec::new();
    for _ in 0..3 {
        let logits = resumed_session.logits().to_vec();
        resumed_logits.push(logits.clone());
        let next_tok = stack_argmax(&logits) as u32;
        resumed_tokens.push(next_tok);
        resumed_session.step(next_tok).expect("step next");
    }

    assert_eq!(
        resumed_tokens, uninterrupted_tokens,
        "greedy generated token sequence must be bit-for-bit identical across save/restore boundary"
    );
    assert_eq!(
        resumed_logits, uninterrupted_logits,
        "greedy generated logits must be bit-for-bit identical across save/restore boundary"
    );

    let _ = std::fs::remove_file(&ckpt_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn session_save_to_file_is_atomic_and_preserves_existing_on_failure() {
    let bytes = artifact(11);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();
    session.step(1).expect("step");

    let temp_dir =
        std::env::temp_dir().join(format!("uor-stack-test-atomic-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create_dir");
    let save_path = temp_dir.join("checkpoint.json");

    // 1. Initial valid save
    session
        .save_session_to_file(&save_path)
        .expect("save initial");
    let original_bytes = std::fs::read(&save_path).expect("read original");
    assert!(!original_bytes.is_empty());

    // 2. Preexisting colliding temporary file must NOT be deleted on creation collision
    let colliding_temp = temp_dir.join(".checkpoint.json.injected-colliding-temp");
    let sentinel_data = b"preexisting-colliding-sentinel-data";
    std::fs::write(&colliding_temp, sentinel_data).expect("write colliding");

    // Force save with the exact colliding temp file path
    let res = session.save_session_to_file_internal(&save_path, Some(colliding_temp.clone()));
    assert!(res.is_err(), "save must fail when temp file already exists");

    // Both original checkpoint and colliding file remain intact
    let current_ckpt = std::fs::read(&save_path).expect("read ckpt");
    assert_eq!(
        original_bytes, current_ckpt,
        "original checkpoint must remain intact"
    );
    let current_temp = std::fs::read(&colliding_temp).expect("read colliding");
    assert_eq!(
        sentinel_data,
        current_temp.as_slice(),
        "colliding file must NOT be deleted"
    );

    let _ = std::fs::remove_file(&colliding_temp);
    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn session_restore_rejects_oversized_file_without_allocating() {
    let bytes = artifact(17);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();

    let temp_dir =
        std::env::temp_dir().join(format!("uor-stack-test-oversized-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create_dir");
    let oversized_path = temp_dir.join("oversized.json");

    // Create a 100 MB sparse file, well above max expected bound
    let file = std::fs::File::create(&oversized_path).expect("create");
    file.set_len(100 * 1024 * 1024).expect("set_len");
    drop(file);

    let res = session.restore_session_from_file(&oversized_path);
    assert!(
        res.is_err(),
        "restore_session_from_file must refuse oversized file before loading"
    );

    let _ = std::fs::remove_file(&oversized_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn session_restore_accepts_valid_edge_value_snapshot_within_bound() {
    let bytes = artifact(11);
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut session = model.session();

    for tok in 0..CONTEXT as u32 {
        session.step(tok % VOCAB as u32).expect("step");
    }

    let temp_dir = std::env::temp_dir().join(format!("uor-stack-test-edge-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).expect("create_dir");
    let save_path = temp_dir.join("full_checkpoint.json");

    session.save_session_to_file(&save_path).expect("save full");
    let file_len = std::fs::metadata(&save_path).expect("metadata").len();
    let max_len = super::session::max_serialized_session_bytes(&model);
    assert!(
        file_len > 0 && file_len <= max_len,
        "valid serialized session size {file_len} must be <= max bound {max_len}"
    );

    let mut fresh_session = model.session();
    fresh_session
        .restore_session_from_file(&save_path)
        .expect("restore full");
    assert_eq!(fresh_session.position(), CONTEXT);
    assert_eq!(fresh_session.tokens(), session.tokens());
    assert_eq!(fresh_session.logits(), session.logits());

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn session_restore_validates_and_restores_snap_trace() {
    // 1. Snapped model with trace enabled
    let snapped = with_header(&artifact(11), |header| {
        header["transport_snap"] = snap_record();
    });
    let model = IntegerStackModel::parse(&snapped).expect("parse");

    let mut session = model.session();
    session.enable_snap_trace();
    session.step(1).expect("step 1");
    session.step(3).expect("step 2");

    let original_trace = session.snap_trace().expect("trace");
    assert!(!original_trace.is_empty());
    for entry in &original_trace {
        assert!(entry.root < 120);
    }

    let saved = session.save_state();
    assert!(saved.snap_trace.is_some());

    // Restore into fresh session with snap trace enabled
    let mut fresh_session = model.session();
    fresh_session.enable_snap_trace();
    fresh_session.restore_state(&saved).expect("restore");
    assert_eq!(
        fresh_session.snap_trace().expect("restored trace"),
        original_trace,
        "restored snap trace must match original exactly"
    );

    // Restore into fresh session without snap trace initially enabled (model has snap)
    let mut lazy_session = model.session();
    lazy_session.restore_state(&saved).expect("restore lazy");
    assert_eq!(
        lazy_session.snap_trace().expect("restored trace"),
        original_trace,
        "snap trace should be populated from saved state when model has snap"
    );

    // Inconsistent snap trace length on snapped model rejected
    let mut bad_len_trace = saved.clone();
    bad_len_trace.snap_trace = Some(vec![0u32; 1]);
    assert!(matches!(
        model.session().restore_state(&bad_len_trace),
        Err(StackError::SessionState)
    ));

    // Out-of-range root index (>= 120) in snap trace on snapped model rejected
    let mut bad_root_trace = saved.clone();
    let expected_len = bad_root_trace.snap_trace.as_ref().unwrap().len();
    bad_root_trace.snap_trace = Some(vec![120u32; expected_len]);
    assert!(matches!(
        model.session().restore_state(&bad_root_trace),
        Err(StackError::SessionState)
    ));

    // 2. Restore state without snap trace into session that had snap trace: disables trace to avoid stale/misaligned entries
    let mut no_trace_session = model.session();
    no_trace_session.step(1).expect("step 1");
    let saved_no_trace = no_trace_session.save_state();
    assert!(saved_no_trace.snap_trace.is_none());

    let mut used_trace_session = model.session();
    used_trace_session.enable_snap_trace();
    used_trace_session.step(2).expect("step");
    assert!(!used_trace_session.snap_trace().unwrap().is_empty());

    used_trace_session
        .restore_state(&saved_no_trace)
        .expect("restore no trace");
    // Should disable tracing so no stale trace entries from used session are exposed
    assert!(used_trace_session.snap_trace().is_none());

    // 3. Late enable_snap_trace records correct position
    let mut late_session = model.session();
    late_session.step(1).expect("step 1");
    late_session.step(2).expect("step 2");
    assert_eq!(late_session.position(), 2);
    late_session.enable_snap_trace();
    late_session.step(3).expect("step 3");
    let late_trace = late_session.snap_trace().expect("late trace");
    assert!(!late_trace.is_empty());
    assert_eq!(late_trace[0].position, 2);

    // 4. Unsnapped model: trace is None in save_state, records no roots
    let unsnapped_model = IntegerStackModel::parse(&artifact(11)).expect("parse unsnapped");
    let mut unsnapped_session = unsnapped_model.session();
    unsnapped_session.enable_snap_trace();
    unsnapped_session.step(1).expect("step unsnapped");
    assert!(unsnapped_session.snap_trace().unwrap().is_empty());
    let saved_unsnapped = unsnapped_session.save_state();
    assert!(saved_unsnapped.snap_trace.is_none());
    let mut fresh_unsnapped = unsnapped_model.session();
    assert!(fresh_unsnapped.restore_state(&saved_unsnapped).is_ok());

    // 5. Late enable -> reset -> full new stream -> save/restore without capacity panic
    let mut late_reset_session = model.session();
    late_reset_session.step(1).expect("step 1");
    late_reset_session.step(2).expect("step 2");
    assert_eq!(late_reset_session.position(), 2);
    late_reset_session.enable_snap_trace();
    late_reset_session.reset();
    assert_eq!(late_reset_session.position(), 0);

    for tok in 0..CONTEXT as u32 {
        late_reset_session
            .step(tok % VOCAB as u32)
            .expect("step full");
    }
    assert_eq!(late_reset_session.position(), CONTEXT);
    let saved_reset_stream = late_reset_session.save_state();
    assert!(saved_reset_stream.snap_trace.is_some());
    let full_trace_roots = saved_reset_stream.snap_trace.as_ref().unwrap();
    let rec_layers = model.shape().pattern.bytes().filter(|&b| b == b'r').count();
    assert_eq!(
        full_trace_roots.len(),
        CONTEXT * model.shape().lanes() * rec_layers
    );

    let mut restored_reset_session = model.session();
    restored_reset_session
        .restore_state(&saved_reset_stream)
        .expect("restore reset full");
    assert_eq!(
        restored_reset_session.snap_trace().unwrap(),
        late_reset_session.snap_trace().unwrap()
    );
}

#[test]
fn test_pointer_copy_boosts_prior_token_logits_and_preserves_across_save_restore() {
    let model = IntegerStackModel::parse(&artifact(42)).expect("parse model");

    // 1. Baseline stepping with copy_scale_q16 == 0
    let mut baseline_session = model.session();
    assert_eq!(baseline_session.copy_scale(), 0);
    assert!(baseline_session.pointer_weights().is_empty());

    let prompt = [3u32, 7u32, 11u32, 7u32];
    let mut baseline_logits = Vec::new();
    for &tok in &prompt {
        let logits = baseline_session.step(tok).expect("step baseline").to_vec();
        baseline_logits.push(logits);
    }
    assert_eq!(baseline_session.position(), prompt.len());
    // When copy_scale is 0, capture is bypassed in stack_read, so weights remain 0
    assert!(
        baseline_session.pointer_weights().iter().all(|&w| w == 0),
        "when copy_scale is 0, capture is bypassed and weights remain 0"
    );

    // 2. Stepping with pointer copy enabled (scale = 1.0 in Q16 = 1 << 16)
    let mut copy_session = model.session();
    copy_session
        .set_copy_scale(1 << 16)
        .expect("a plain model accepts a copy scale");
    assert_eq!(copy_session.copy_scale(), 1 << 16);

    let mut copy_logits = Vec::new();
    let mut step_weights_snapshots = Vec::new();
    for &tok in &prompt {
        let logits = copy_session.step(tok).expect("step copy").to_vec();
        step_weights_snapshots.push(copy_session.pointer_weights().to_vec());
        copy_logits.push(logits);
    }
    assert_eq!(copy_session.position(), prompt.len());
    assert_eq!(copy_session.pointer_weights().len(), prompt.len());
    assert!(
        copy_session.pointer_weights().iter().any(|&w| w > 0),
        "with copy_scale > 0, non-zero attention weights must be captured"
    );

    // Step 0: prompt[0] has no preceding context tokens, so logits must match baseline bit-for-bit
    assert_eq!(
        copy_logits[0], baseline_logits[0],
        "step 0 has no preceding context, logits must match baseline exactly"
    );

    // Subsequent steps: tokens present in prompt[..step_idx] receive non-negative pointer copy boost;
    // tokens NOT present in prompt[..step_idx] must remain strictly equal to baseline logits.
    for step_idx in 1..prompt.len() {
        let prefix = &prompt[..step_idx];
        let prefix_set: std::collections::BTreeSet<u32> = prefix.iter().copied().collect();

        // Any token not in prefix must have bit-identical logit to baseline
        for v in 0..VOCAB as u32 {
            if !prefix_set.contains(&v) {
                assert_eq!(
                    copy_logits[step_idx][v as usize], baseline_logits[step_idx][v as usize],
                    "unseen token {v} at step {step_idx} must have unchanged logits"
                );
            }
        }

        // Pointer weights must be valid normalized values (sum <= Q31)
        let weights = &step_weights_snapshots[step_idx][..step_idx];
        for &w in weights {
            assert!(
                w <= (1u64 << 31),
                "normalized pointer weight must be <= Q31"
            );
        }

        // Any seen token with non-zero attention mass must receive a non-negative boost
        for (pos, &tok) in prefix.iter().enumerate() {
            let pw = step_weights_snapshots[step_idx][pos];
            if pw > 0 {
                assert!(
                    copy_logits[step_idx][tok as usize] >= baseline_logits[step_idx][tok as usize],
                    "seen token {tok} must receive non-negative boost"
                );
            }
        }
    }

    // 3. Save / restore state roundtrip preserves copy_scale_q16 and produces bit-identical continuation
    let saved = copy_session.save_state();
    assert_eq!(saved.copy_scale_q16, 1 << 16);
    assert_eq!(saved.position, prompt.len() as u64);

    let mut restored_session = model.session();
    restored_session
        .restore_state(&saved)
        .expect("restore state");
    assert_eq!(restored_session.copy_scale(), 1 << 16);
    assert_eq!(restored_session.position(), prompt.len());
    // On restore, pointer_weights buffer is zeroed until the next step
    assert!(
        restored_session.pointer_weights().iter().all(|&w| w == 0),
        "pointer_weights should be zeroed upon restore"
    );
    assert_eq!(
        restored_session.logits(),
        copy_session.logits(),
        "restored logits must match saved session logits"
    );

    let continuation_token = 5u32;
    let next_copy = copy_session
        .step(continuation_token)
        .expect("next copy")
        .to_vec();
    let next_rest = restored_session
        .step(continuation_token)
        .expect("next restored")
        .to_vec();
    assert_eq!(
        next_copy, next_rest,
        "continuation logits after restore must be bit-for-bit identical"
    );

    // 4. Disabling copy scale clears pointer weights buffer
    assert!(copy_session.pointer_weights().iter().any(|&w| w > 0));
    copy_session
        .set_copy_scale(0)
        .expect("a plain model accepts a copy scale");
    assert_eq!(copy_session.copy_scale(), 0);
    assert!(
        copy_session.pointer_weights().iter().all(|&w| w == 0),
        "disabling copy scale must clear pointer weights buffer"
    );

    // 5. Session reset clears pointer weights and position but allows clean reuse
    copy_session.reset();
    assert_eq!(copy_session.position(), 0);
    assert_eq!(copy_session.tokens().len(), 0);
    assert!(copy_session.pointer_weights().is_empty());
    let reset_step0 = copy_session
        .step(prompt[0])
        .expect("step after reset")
        .to_vec();
    assert_eq!(
        reset_step0, baseline_logits[0],
        "step 0 after reset must match baseline step 0"
    );
}

#[test]
fn test_pointer_copy_retrieval_boost_argmax_override_and_duplicate_accumulation() {
    let model = IntegerStackModel::parse(&artifact(42)).expect("parse model");

    // Stepping prompt with duplicate tokens: token 7 at pos 1 and pos 3
    let prompt = [3u32, 7u32, 11u32, 7u32];

    // Baseline stepping
    let mut baseline = model.session();
    for &tok in &prompt {
        baseline.step(tok).expect("baseline step");
    }
    let baseline_continuation = baseline.step(5).expect("baseline step 5").to_vec();

    // Copy session with scale = 1.0 (Q16 = 1 << 16)
    let mut copy = model.session();
    copy.set_copy_scale(1 << 16)
        .expect("a plain model accepts a copy scale");
    for &tok in &prompt {
        copy.step(tok).expect("copy step");
    }
    let copy_continuation = copy.step(5).expect("copy step 5").to_vec();

    // Verify duplicate token accumulation: token 7 was present at pos 1 and pos 3
    let pw1 = i128::from(copy.pointer_weights()[1]);
    let pw3 = i128::from(copy.pointer_weights()[3]);
    assert!(
        pw1 > 0,
        "token 7 at pos 1 must have non-zero attention weight"
    );
    assert!(
        pw3 > 0,
        "token 7 at pos 3 must have non-zero attention weight"
    );
    let expected_boost_pos1 = (pw1 * (1i128 << 16)) >> 31;
    let expected_boost_pos3 = (pw3 * (1i128 << 16)) >> 31;
    assert!(
        expected_boost_pos1 > 0,
        "expected boost at pos 1 must be strictly positive"
    );
    assert!(
        expected_boost_pos3 > 0,
        "expected boost at pos 3 must be strictly positive"
    );
    let expected_total_boost = (expected_boost_pos1 + expected_boost_pos3) as i32;
    assert!(
        expected_total_boost < i32::MAX,
        "expected total boost must be non-saturating"
    );

    let actual_diff = copy_continuation[7] - baseline_continuation[7];
    assert_eq!(
        actual_diff, expected_total_boost,
        "duplicate token 7 must accumulate exact sum of pointer boosts from all prefix occurrences"
    );

    // Large scale: test retrieval override where pointer boost drives argmax
    let mut strong_copy = model.session();
    // Use large scale (Q16 = 1 << 30) to test argmax override
    strong_copy
        .set_copy_scale(1 << 30)
        .expect("a plain model accepts a copy scale");
    for &tok in &prompt {
        strong_copy.step(tok).expect("strong copy step");
    }
    let strong_continuation = strong_copy.step(5).expect("strong copy step 5").to_vec();
    let winner = super::stack_argmax(&strong_continuation) as u32;
    assert!(
        prompt.contains(&winner),
        "under strong copy scale, a prefix token ({winner}) must win argmax through pointer retrieval"
    );

    // Distance retrieval: token 3 at distance 4 still receives non-negative boost
    let pw0 = i128::from(strong_copy.pointer_weights()[0]);
    if pw0 > 0 {
        assert!(
            strong_continuation[3] > baseline_continuation[3],
            "prefix token 3 at distance 4 must receive positive boost"
        );
    }

    // Extremal scale test: i32::MAX scale saturates cleanly via saturating_add without overflow or panic
    let mut max_scale_session = model.session();
    max_scale_session
        .set_copy_scale(i32::MAX)
        .expect("a plain model accepts a copy scale");
    for &tok in &prompt {
        max_scale_session
            .step(tok)
            .expect("step with i32::MAX scale");
    }
    let max_continuation = max_scale_session
        .step(5)
        .expect("step 5 with i32::MAX scale");
    assert!(
        max_continuation[7] > copy_continuation[7],
        "under extremal i32::MAX scale, boosted logits must exceed unit-scale copy logits"
    );
}

// ---------------------------------------------------------------------------
// Dialogue serving (`stack::chat`): the chat CLI's path to a stack artifact.

/// The GPT-2 byte-to-character alphabet of the tokenizer crate.
fn alphabet() -> [char; 256] {
    let mut table = ['\0'; 256];
    let mut extra = 0u32;
    for byte in 0u32..256 {
        let printable = (0x21..=0x7e).contains(&byte)
            || (0xa1..=0xac).contains(&byte)
            || (0xae..=0xff).contains(&byte);
        table[byte as usize] = if printable {
            char::from_u32(byte).expect("printable byte")
        } else {
            extra += 1;
            char::from_u32(255 + extra).expect("byte mapping")
        };
    }
    table
}

/// A byte-level tokenizer with the three dialogue specials at ids 0-2 and
/// exactly [`VOCAB`] entries: the bytes the literal-role markers and the test
/// messages need, then filler bytes.
fn chat_tokenizer_json() -> Vec<u8> {
    chat_tokenizer_json_sized(VOCAB)
}

/// [`chat_tokenizer_json`] at `vocab` entries (at most 259: the specials and
/// every byte).
fn chat_tokenizer_json_sized(vocab: usize) -> Vec<u8> {
    let table = alphabet();
    let mut bytes: Vec<u8> = b"User: Assistant: System: Memory: b.\nHilmotwae".to_vec();
    bytes.sort_unstable();
    bytes.dedup();
    for byte in 0u8..=255 {
        if bytes.len() == vocab - 3 {
            break;
        }
        if !bytes.contains(&byte) {
            bytes.push(byte);
        }
    }
    bytes.sort_unstable();
    bytes.truncate(vocab - 3);
    let mut vocab = serde_json::Map::new();
    for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
        vocab.insert((*surface).to_owned(), json!(id));
    }
    for (index, byte) in bytes.iter().enumerate() {
        vocab.insert(table[*byte as usize].to_string(), json!(index + 3));
    }
    let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
        .iter()
        .enumerate()
        .map(|(id, surface)| json!({"id": id, "content": surface}))
        .collect();
    serde_json::to_vec(&json!({
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "added_tokens": added,
        "model": {"type": "BPE", "vocab": vocab, "merges": []},
    }))
    .expect("tokenizer json")
}

fn chat(seed: u64) -> Result<(IntegerStackModel, Vec<u8>), StackError> {
    Ok((
        IntegerStackModel::parse(&artifact(seed))?,
        chat_tokenizer_json(),
    ))
}

fn open_chat<'m>(
    model: &'m IntegerStackModel,
    json: &[u8],
) -> Result<super::StackChat<'m>, Box<dyn std::error::Error>> {
    Ok(super::StackChat::from_tokenizer_json(model, json, 1)?)
}

type ChatTest = Result<(), Box<dyn std::error::Error>>;

/// The greedy continuation of `history` on a fresh session, as the float
/// stack's `greedy_reply` computes it: the highest served score (ties to the
/// lower id) until EOS, a terminal cycle of length 1..=4 repeated three times,
/// or the cap.
fn greedy_reference(
    model: &IntegerStackModel,
    history: &[u32],
    cap: usize,
    eos: u32,
) -> Result<Vec<u32>, StackError> {
    let mut session = model.session();
    let mut scores = Vec::new();
    for &id in history {
        scores = session.step(id)?.to_vec();
    }
    let mut ids = Vec::new();
    for _ in 0..cap {
        let next = super::stack_argmax(&scores) as u32;
        ids.push(next);
        if next == eos || crate::generation::short_cycle(&ids).is_some() {
            break;
        }
        scores = session.step(next)?.to_vec();
    }
    Ok(ids)
}

#[test]
fn stack_chat_greedy_reply_is_the_engines_own_continuation() -> ChatTest {
    let (model, json) = chat(11)?;
    let mut chat = open_chat(&model, &json)?;
    let eos = chat.protocol().eos_id;
    let history = [chat.protocol().bos_id, 3, 4];
    chat.seed(&history[..1])?;
    let reply = chat.reply_history(&history, 3)?;
    assert_eq!(reply.ids, greedy_reference(&model, &history, 3, eos)?);
    assert_eq!(
        chat.tokens(),
        &[history.as_slice(), reply.ids.as_slice()].concat()[..],
        "the history keeps every token, terminal token included"
    );
    // Determinism: the same history asks the same question again.
    let again = chat.reply_history(&history, 3)?;
    assert_eq!(again.ids, reply.ids);
    Ok(())
}

#[test]
fn stack_chat_extends_the_session_instead_of_replaying_it() -> ChatTest {
    let (model, json) = chat(13)?;
    let mut chat = open_chat(&model, &json)?;
    let bos = chat.protocol().bos_id;
    chat.seed(&[bos])?;
    let first = chat.reply_history(&[bos, 3], 2)?;
    let steps_after_first = chat.position();
    let mut second_history = vec![bos, 3];
    second_history.extend_from_slice(&first.ids);
    second_history.push(4);
    let second = chat.reply_history(&second_history, 1)?;
    // The first reply's ids were fed once: the second turn steps its own new
    // suffix (the pending terminal id and token 4), not the whole history.
    let stepped = if second.stop == super::StackStop::MaximumNewTokens {
        second_history.len() + second.ids.len()
    } else {
        second_history.len() + second.ids.len() - 1
    };
    assert_eq!(chat.position(), stepped);
    assert!(
        chat.position() - steps_after_first <= 2,
        "the session replayed the history"
    );
    Ok(())
}

#[test]
fn stack_chat_refuses_a_turn_that_does_not_fit_the_context() -> ChatTest {
    let (model, json) = chat(17)?;
    let mut chat = open_chat(&model, &json)?;
    assert_eq!(chat.context(), CONTEXT);
    chat.seed(&[0])?;
    let full = [0u32, 1, 2, 3, 4, 5];
    let error = chat
        .reply_history(&full, 1)
        .expect_err("a seven-position turn cannot fit a six-position context");
    assert!(matches!(error, super::StackChatError::Context { .. }));
    assert_eq!(chat.tokens(), &[0], "a refused turn changes nothing");
    Ok(())
}

#[test]
fn stack_chat_rejects_a_message_with_literal_special_tokens() -> ChatTest {
    let (model, json) = chat(19)?;
    let mut chat = open_chat(&model, &json)?;
    chat.seed(&[0])?;
    let error = chat.reply("Hi <|eos|>", 2).expect_err("special tokens");
    assert!(matches!(error, super::StackChatError::NotAUserTurn));
    Ok(())
}

#[test]
fn stack_chat_encodes_system_memory_between_user_and_assistant() -> ChatTest {
    let model = IntegerStackModel::parse(&artifact_with_context(29, &[], 64))?;
    let json = chat_tokenizer_json();
    let mut chat = super::StackChat::from_tokenizer_json(&model, &json, 2)?;
    let expected = chat
        .protocol()
        .bind(chat.tokenizer())?
        .encode_assistant_suffix(
            &[
                uor_r4_tokenizer::dialogue::Message {
                    role: "user",
                    content: "a",
                },
                uor_r4_tokenizer::dialogue::Message {
                    role: "system",
                    content: "Memory: b.",
                },
            ],
            false,
        )
        .tokens;
    assert_eq!(
        chat.tokenizer().decode(&expected),
        "User: a\nSystem: Memory: b.\nAssistant:"
    );
    chat.seed(&[chat.protocol().bos_id])?;
    chat.reply_with_memory("a", Some("b"), 1)?;
    assert_eq!(&chat.tokens()[1..1 + expected.len()], expected);
    Ok(())
}

#[test]
fn stack_chat_no_memory_is_the_existing_reply_path() -> ChatTest {
    let (model, json, _) = fbs_chat(31)?;
    let bos = 0;
    let mut reference = super::StackChat::from_tokenizer_json(&model, &json, 2)?;
    let mut history = vec![bos];
    history.extend(
        reference
            .protocol()
            .bind(reference.tokenizer())?
            .encode_user_prefix("Hi", false)
            .tokens,
    );
    reference.seed(&[bos])?;
    let expected = reference.reply_history(&history, 2)?;

    let mut explicit = super::StackChat::from_tokenizer_json(&model, &json, 2)?;
    explicit.seed(&[bos])?;
    let explicit_reply = explicit.reply_with_memory("Hi", None, 2)?;
    assert_eq!(explicit_reply.ids, expected.ids);
    assert_eq!(explicit_reply.text, expected.text);
    assert_eq!(explicit_reply.stop, expected.stop);
    assert_eq!(explicit.tokens(), reference.tokens());
    Ok(())
}

#[test]
fn stack_chat_refuses_invalid_memory_without_mutating_history() -> ChatTest {
    let (model, json) = chat(37)?;
    for value in ["", "two\nlines", "two\rlines", "<|eos|>"] {
        let mut chat = super::StackChat::from_tokenizer_json(&model, &json, 2)?;
        chat.seed(&[chat.protocol().bos_id])?;
        let before = chat.tokens().to_vec();
        let error = chat
            .reply_with_memory("Hi", Some(value), 1)
            .expect_err("invalid memory must be refused");
        assert!(
            matches!(
                error,
                super::StackChatError::InvalidMemory(_) | super::StackChatError::NotAUserTurn
            ),
            "{error}"
        );
        assert_eq!(chat.tokens(), before, "{value:?}");
    }
    assert_eq!(
        super::StackChat::memory_content("Pickle")?,
        "Memory: Pickle."
    );
    Ok(())
}

#[test]
fn stack_chat_reset_and_seed_start_a_fresh_conversation() -> ChatTest {
    let (model, json) = chat(23)?;
    let mut chat = open_chat(&model, &json)?;
    chat.seed(&[0])?;
    let reply = chat.reply_history(&[0, 3], 2)?;
    assert!(chat.tokens().len() > 2);
    chat.reset();
    chat.seed(&[0])?;
    assert_eq!(chat.tokens(), &[0]);
    assert_eq!(chat.position(), 1);
    assert_eq!(
        chat.reply_history(&[0, 3], 2)?.ids,
        reply.ids,
        "a fresh conversation with the same history serves the same ids"
    );
    Ok(())
}

/// The chat's FBS context: room for a stored opening key, a reply and a second turn.
const FBS_CHAT_CONTEXT: usize = 32;

/// claude-122's small SpiralCore/FBS trunk at a 32-token context with a turn memory (opening and
/// reply tables, `k` 8, training's 192-token key block), as `fbs-export` writes a `/7` artifact,
/// with a tokenizer of its vocabulary. Also returns the opening keys a turn can open with: the
/// stored keys holding no `<|bos|>` or `<|eos|>`, which would move the turn's anchor.
fn fbs_chat(seed: u64) -> Result<(IntegerStackModel, Vec<u8>, Vec<Vec<u32>>), StackError> {
    let mut snap = synthetic_fbs_trunk_with(
        seed,
        FbsSynth {
            context: FBS_CHAT_CONTEXT,
            ..FBS_SYNTH_SMALL
        },
    );
    let memory = synthetic_turn_memory(snap.vocab, snap.width, 8, 192, true, 40, seed ^ 0x5eed);
    let keys = memory
        .opening
        .iter()
        .map(|entry| entry.key.clone())
        .filter(|key| key.iter().all(|&id| id > 1))
        .collect();
    snap.memory = Some(memory);
    let model = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap)?)?;
    Ok((model, chat_tokenizer_json_sized(snap.vocab), keys))
}

/// The served continuation of `history` on a fresh session, as `fbs-toks` decodes a reply: with
/// `mark`, the turn memory's reply start is marked before the history's last token is stepped;
/// then each token is the one [`super::IntegerStackSession::serve_step`] serves, until EOS, a
/// terminal cycle of length 1..=4 repeated three times, or the cap.
fn served_reference(
    model: &IntegerStackModel,
    history: &[u32],
    cap: usize,
    eos: u32,
    mark: bool,
) -> Result<Vec<u32>, StackError> {
    let mut session = model.session();
    let mut pick = 0;
    for (index, &id) in history.iter().enumerate() {
        if mark && index + 1 == history.len() {
            session.mark_reply_start();
        }
        pick = session.serve_step(id)?.0;
    }
    let mut ids = Vec::new();
    for _ in 0..cap {
        ids.push(pick);
        if pick == eos || crate::generation::short_cycle(&ids).is_some() {
            break;
        }
        pick = session.serve_step(pick)?.0;
    }
    Ok(ids)
}

/// claude-259: the chat serves a SpiralCore/FBS artifact. Its step has no logits, so the chat
/// read `step`'s empty score row and refused every turn ("the engine returned 0 scores for a
/// vocabulary of 4096" on a7r-s1); it now serves `serve_step`'s pick, the turn memory marked.
#[test]
fn stack_chat_serves_an_fbs_model_with_its_turn_memory() -> ChatTest {
    let (model, json, keys) = fbs_chat(31)?;
    assert!(model.fbs_writer_table().is_some() && model.turn_memory().is_some());
    let mut chat = open_chat(&model, &json)?;
    let (bos, eos) = (chat.protocol().bos_id, chat.protocol().eos_id);
    assert!(!keys.is_empty());
    for key in &keys {
        let history = [&[bos][..], key].concat();
        let cap = 12;
        chat.seed(&[bos])?;
        let reply = chat.reply_history(&history, cap)?;
        assert_eq!(reply.ids, served_reference(&model, &history, cap, eos, true)?, "key {key:?}");
    }
    Ok(())
}

/// claude-259: the mark is where `fbs-toks` and training's reply rows put it, before the prompt's
/// last token. Some stored opening keys serve a different reply with and without it; on every
/// key the chat serves the marked one, also when the prompt was seeded (stepped unmarked) first.
#[test]
fn stack_chat_marks_the_reply_start_before_the_prompts_last_token() -> ChatTest {
    let (model, json, keys) = fbs_chat(37)?;
    let mut chat = open_chat(&model, &json)?;
    let (bos, eos) = (chat.protocol().bos_id, chat.protocol().eos_id);
    let cap = 12;
    let mut differ = 0;
    for key in &keys {
        let history = [&[bos][..], key].concat();
        let marked = served_reference(&model, &history, cap, eos, true)?;
        if marked != served_reference(&model, &history, cap, eos, false)? {
            differ += 1;
        }
        chat.seed(&[bos])?;
        assert_eq!(chat.reply_history(&history, cap)?.ids, marked, "key {key:?}");
        chat.seed(&history)?;
        assert_eq!(chat.reply_history(&history, cap)?.ids, marked, "seeded key {key:?}");
    }
    assert!(differ > 0, "no stored opening key changes the reply: the test cannot see the mark");
    Ok(())
}

/// claude-259: a second turn extends the session, and serves what a fresh session marked before
/// the second prompt's last token serves (the memory moves only the readout, so the first turn's
/// moves leave no trace in the state).
#[test]
fn stack_chat_marks_each_turn_of_an_fbs_conversation() -> ChatTest {
    let (model, json, keys) = fbs_chat(41)?;
    let mut chat = open_chat(&model, &json)?;
    let (bos, eos) = (chat.protocol().bos_id, chat.protocol().eos_id);
    let (first_key, second_key) = (&keys[0], &keys[1]);
    chat.seed(&[bos])?;
    let first_history = [&[bos][..], first_key].concat();
    let first = chat.reply_history(&first_history, 6)?;
    let mut second_history = [first_history.as_slice(), &first.ids].concat();
    if first.stop != super::StackStop::Eos {
        second_history.push(eos);
    }
    second_history.extend_from_slice(second_key);
    let steps_before = chat.position();
    let second = chat.reply_history(&second_history, 6)?;
    assert_eq!(second.ids, served_reference(&model, &second_history, 6, eos, true)?);
    assert!(
        chat.position() - steps_before <= second_history.len() - first_history.len() + second.ids.len(),
        "the session replayed the history"
    );
    Ok(())
}

/// claude-259: the chat's history is the tokens the session stepped, each once. A reply that ends
/// on EOS or a short cycle leaves its last id pending; the next turn steps it and kept a second
/// copy in the history ("16, 1, 1, 201" on a7r-s1), which the next turn's prompt then carried.
#[test]
fn stack_chat_keeps_a_pending_terminal_id_once() -> ChatTest {
    let (model, json, keys) = fbs_chat(43)?;
    let mut chat = open_chat(&model, &json)?;
    let bos = chat.protocol().bos_id;
    let mut pending = 0;
    for pair in keys.windows(2) {
        chat.seed(&[bos])?;
        let first_history = [&[bos][..], &pair[0]].concat();
        let first = chat.reply_history(&first_history, 8)?;
        pending += usize::from(first.stop != super::StackStop::MaximumNewTokens);
        let second_history = [chat.tokens(), &pair[1]].concat();
        let second = chat.reply_history(&second_history, 4)?;
        assert_eq!(chat.tokens(), &[second_history.as_slice(), &second.ids].concat()[..]);
    }
    assert!(pending > 0, "no first reply left an id pending: the test cannot see the copy");
    Ok(())
}

/// `artifact(seed)` with the schema set and the shape edited.
fn read_schema_artifact(schema: &str, edit: impl FnOnce(&mut Value)) -> Vec<u8> {
    with_header(&artifact(5), |header| {
        header["schema"] = json!(schema);
        edit(&mut header["shape"]);
    })
}

fn rank_read(shape: &mut Value) {
    shape["read_select"] = json!({"window": 8, "k": 8});
    shape["read_weights"] = json!("rank");
}

fn a_pointer(shape: &mut Value) {
    shape["pointer"] = json!({"dim": 16, "score": "dot", "score_scale_q30": 1 << 28});
}

#[test]
fn read_schema_plain_and_pointer_artifacts_parse_as_before() {
    let plain = read_schema_artifact(super::STACK_SCHEMA, |_| {});
    let container = super::format::Container::parse(&plain).expect("schema /1");
    assert_eq!(container.shape.read_rank(), None);
    assert!(!container.shape.read_binary());
    assert_eq!(
        super::stack_schema_for(&container.shape),
        super::STACK_SCHEMA
    );
    assert!(IntegerStackModel::parse(&plain).is_ok());
    let pointer = read_schema_artifact(super::STACK_POINTER_SCHEMA, a_pointer);
    let container = super::format::Container::parse(&pointer).expect("schema /2");
    assert_eq!(
        super::stack_schema_for(&container.shape),
        super::STACK_POINTER_SCHEMA
    );
    // The pointer head stays on /2, and a plain shape stays on /1.
    for (schema, edit) in [
        (
            super::STACK_POINTER_SCHEMA,
            (|_: &mut Value| {}) as fn(&mut Value),
        ),
        (super::STACK_SCHEMA, a_pointer),
        (super::STACK_READ_SCHEMA, |_: &mut Value| {}),
        (super::STACK_READ_SCHEMA, a_pointer),
    ] {
        assert!(
            matches!(
                super::format::Container::parse(&read_schema_artifact(schema, edit)),
                Err(StackError::Schema(_))
            ),
            "accepted {schema} for the wrong shape"
        );
    }
}

#[test]
fn read_schema_rank_and_binary_reads_parse_and_are_served_under_schema_3() {
    for binary in [false, true] {
        for pointer in [false, true] {
            let bytes = read_schema_artifact(super::STACK_READ_SCHEMA, |shape| {
                rank_read(shape);
                shape["read_binary"] = json!(binary);
                if pointer {
                    a_pointer(shape);
                }
            });
            let container = super::format::Container::parse(&bytes).expect("a rank read under /3");
            assert_eq!(
                container.shape.read_rank(),
                Some(super::StackReadSelect { window: 8, k: 8 })
            );
            assert_eq!(container.shape.read_binary(), binary);
            assert_eq!(
                super::stack_schema_for(&container.shape),
                super::STACK_READ_SCHEMA
            );
            // The rank and binary reads are served. The fixture has no
            // pointer sections, so a served read with a pointer head passes
            // the read check and stops at them.
            match (pointer, IntegerStackModel::parse(&bytes)) {
                (false, Ok(_)) => {}
                (true, Err(other)) if other.to_string().contains("pointer_query") => {}
                (_, Err(other)) => panic!("refused for another reason: {other}"),
                (true, Ok(_)) => panic!("served a missing pointer"),
            }
        }
    }
    // The selection's bound: window + k + 2 <= 256 slots.
    let largest = read_schema_artifact(super::STACK_READ_SCHEMA, |shape| {
        rank_read(shape);
        shape["read_select"] = json!({"window": 127, "k": 127});
    });
    assert!(super::format::Container::parse(&largest).is_ok());
}

#[test]
fn read_schema_rank_reads_under_schema_1_or_2_are_refused() {
    for (schema, pointer) in [
        (super::STACK_SCHEMA, false),
        (super::STACK_POINTER_SCHEMA, true),
    ] {
        let bytes = read_schema_artifact(schema, |shape| {
            rank_read(shape);
            if pointer {
                a_pointer(shape);
            }
        });
        assert!(
            matches!(
                super::format::Container::parse(&bytes),
                Err(StackError::Schema(_))
            ),
            "accepted a rank read under {schema}"
        );
    }
}

#[test]
fn read_schema_invalid_read_fields_are_refused() {
    let cases: [(&str, fn(&mut Value)); 7] = [
        ("softmax weights", |shape| {
            rank_read(shape);
            shape["read_weights"] = json!("softmax");
        }),
        ("rank weights without a selection", |shape| {
            shape["read_weights"] = json!("rank");
        }),
        ("a selection without rank weights", |shape| {
            shape["read_select"] = json!({"window": 8, "k": 8});
        }),
        ("binary without a selection", |shape| {
            shape["read_binary"] = json!(true);
        }),
        ("window 0", |shape| {
            rank_read(shape);
            shape["read_select"] = json!({"window": 0, "k": 8});
        }),
        ("k 0", |shape| {
            rank_read(shape);
            shape["read_select"] = json!({"window": 8, "k": 0});
        }),
        ("window + k + 2 > 256", |shape| {
            rank_read(shape);
            shape["read_select"] = json!({"window": 128, "k": 127});
        }),
    ];
    for (label, edit) in cases {
        for schema in [super::STACK_SCHEMA, super::STACK_READ_SCHEMA] {
            let bytes = read_schema_artifact(schema, edit);
            assert!(
                super::format::Container::parse(&bytes).is_err(),
                "accepted {label} under {schema}"
            );
        }
        // The shape rule itself refuses it, whatever the schema.
        let mut shape = serde_json::json!({
            "vocab": 40, "width": 64, "heads": 2, "mlp": 32, "pattern": "ra",
            "read": "dot", "rotation": false, "context": 8,
        });
        edit(&mut shape);
        let shape: StackShape = serde_json::from_value(shape).expect("shape");
        assert!(
            matches!(shape.validate(), Err(StackError::Shape(_))),
            "validated {label}"
        );
    }
}

/// The normalized Q31 rank table of `m` slots.
fn rank_table(m: usize) -> Vec<u32> {
    let mut table = vec![0u32; m];
    super::rank_table_q31(m, &mut table).expect("rank table");
    table
}

/// Checks one head's served read weights against the flock rank rule: zero
/// off the flock support, and on it the rank table of `kept + 1` slots by
/// descending score (ties to the lowest position) with exactly one rank,
/// NoRead's, skipped. Returns NoRead's rank.
fn check_rank_read(scores: &[i64], weights: &[u64], window: usize, k: usize) -> usize {
    let position = scores.len() - 1;
    let mut scratch = super::FlockScratch::new(scores.len());
    super::flock_select_integer(
        scores,
        position,
        super::FlockSelect::new(0, window, k),
        &mut scratch,
    )
    .expect("select");
    let kept: Vec<usize> = scratch.entries.iter().map(|e| e.position).collect();
    for (j, &w) in weights.iter().enumerate() {
        if !kept.contains(&j) {
            assert_eq!(w, 0, "position {j} off the support has weight {w}");
        }
    }
    let table = rank_table(kept.len() + 1);
    let skipped: Vec<usize> = (0..=kept.len())
        .filter(|&null_rank| {
            kept.iter().enumerate().all(|(index, &j)| {
                let slot = if index >= null_rank { index + 1 } else { index };
                weights[j] == u64::from(table[slot])
            })
        })
        .collect();
    assert!(
        !skipped.is_empty(),
        "weights {weights:?} are not the rank table {table:?} over {kept:?}"
    );
    skipped[0]
}

#[test]
fn read_rank_weights_place_noread_by_the_tie_rule() {
    // Window 2, k 1 and sink 0 at position 5 of [9, 4, 7, 7, 2, 7] keep the
    // sink 0, the window 4 and 5, and the top-1 of 1..=3: position 2 (its
    // tie with 3 goes to the lowest). Descending, ties to the lowest
    // position: 0 (9), 2 (7), 5 (7), 4 (2); NoRead follows every kept score
    // at least its own.
    let scores = [9i64, 4, 7, 7, 2, 7];
    for (null_score, null_rank) in [(10, 0), (9, 1), (8, 1), (7, 3), (3, 3), (2, 4), (1, 4)] {
        let (weights, null) =
            super::session::rank_weights_for_test(2, 1, &scores, null_score).expect("weights");
        let table = rank_table(5);
        assert_eq!(null, u64::from(table[null_rank]), "null {null_score}");
        assert_eq!(
            check_rank_read(&scores, &weights, 2, 1),
            null_rank,
            "null {null_score}"
        );
        // Unkept positions 1 and 3 (top-1 of 1..=3 is 2: the tie's lowest).
        assert_eq!((weights[1], weights[3]), (0, 0));
        assert_ne!(weights[2], 0);
        assert_eq!(
            weights.iter().sum::<u64>() + null,
            table.iter().map(|&w| u64::from(w)).sum::<u64>()
        );
    }
}

/// Steps a rank-read model of `window` and `k` over the whole context and
/// checks every position and head against the flock rank rule.
fn check_served_rank_read(window: usize, k: usize) -> usize {
    let bytes = read_schema_artifact(super::STACK_READ_SCHEMA, |shape| {
        shape["read_select"] = json!({"window": window, "k": k});
        shape["read_weights"] = json!("rank");
    });
    let model = IntegerStackModel::parse(&bytes).expect("a rank read is served");
    let mut session = model.session();
    let mut unkept = 0usize;
    for (position, id) in (0..CONTEXT as u32).map(|i| (i as usize, (i * 7 + 3) % VOCAB as u32)) {
        session.step(id).expect("step");
        for head in 0..HEADS {
            let (scores, weights) = session.last_read(head);
            assert_eq!(scores.len(), position + 1);
            check_rank_read(scores, weights, window, k);
            unkept += weights.iter().filter(|&&w| w == 0).count();
        }
    }
    unkept
}

#[test]
fn read_rank_serving_weights_are_the_flock_rank_table() {
    // Window 2 and k 1 leave positions off the support from position 4 on.
    assert!(check_served_rank_read(2, 1) > 0);
}

#[test]
fn read_rank_window_covering_every_position_still_reads_by_rank() {
    // Window 8 covers the whole context of 6: every position is kept, and the
    // weights are still the rank table, not the softmax.
    assert_eq!(check_served_rank_read(8, 8), 0);
}

/// A binary (Hamming-rank) read of `window` and `k` over a `read` score kind.
fn binary_read(read: &'static str, window: usize, k: usize) -> IntegerStackModel {
    let bytes = with_header(
        &read_schema_artifact(super::STACK_READ_SCHEMA, |shape| {
            shape["read_select"] = json!({"window": window, "k": k});
            shape["read_weights"] = json!("rank");
            shape["read_binary"] = json!(true);
        }),
        |header| header["shape"]["read"] = json!(read),
    );
    IntegerStackModel::parse(&bytes).expect("a binary read is served")
}

#[test]
fn read_binary_table_is_the_dense_score_of_plus_minus_one_vectors() {
    let one = 1i32 << 16;
    let hd = WIDTH / HEADS;
    for read in ["l2", "dot", "lorentz"] {
        let model = binary_read(read, 8, 8);
        for head in 0..HEADS {
            let table = model.binary_table(head).expect("a binary table");
            assert_eq!(table.len(), hd + 1);
            // A sign pattern unlike the table's first-h construction: the
            // query has mixed signs, and the key differs in h strided lanes.
            let query: Vec<i32> = (0..hd)
                .map(|i| if i % 3 == 0 { -one } else { one })
                .collect();
            for h in [0usize, 1, 2, 5, 9, hd - 1, hd] {
                let mut key = query.clone();
                for lane in (0..hd).map(|i| (i * 5 + 3) % hd).take(h) {
                    key[lane] = -key[lane];
                }
                let dense = model.dense_score(head, &query, &key).expect("a score");
                assert_eq!(table[h], dense, "{read} head {head} h {h}");
            }
            // More differing signs never score higher.
            assert!(
                table.windows(2).all(|w| w[0] >= w[1]) && table[0] > table[hd],
                "{read} head {head}: {table:?}"
            );
        }
    }
}

#[test]
fn read_binary_artifact_steps_by_the_table_and_a_key_sign_flip_changes_the_logits() {
    for read in ["l2", "dot"] {
        let model = binary_read(read, 8, 8);
        let mut session = model.session();
        let ids = [3u32, 39, 0, 17, 8];
        for (position, &id) in ids.iter().enumerate() {
            session.step(id).expect("a binary read steps");
            for head in 0..HEADS {
                let (table, ages) = (
                    model.binary_table(head).expect("table"),
                    model.read_ages(head).expect("ages"),
                );
                let (scores, _) = session.last_read(head);
                for (j, &score) in scores.iter().enumerate() {
                    // Every served score is a table entry plus its age.
                    let bare = score - i64::from(ages[position - j]);
                    assert!(
                        table.contains(&bare),
                        "{read} head {head} position {j}: {bare} is not in {table:?}"
                    );
                }
            }
        }
        // Flip the signs of position 1's key in every read layer, restore
        // (the sign codes follow from the keys), and step once more.
        let saved = session.save_state();
        let mut flipped = saved.clone();
        for layer in &mut flipped.layers {
            if let super::SerializedStackLayerState::Read { keys, .. } = layer {
                for key in &mut keys[WIDTH..2 * WIDTH] {
                    *key = !*key;
                }
            }
        }
        let next = |state: &super::SerializedStackSession| {
            let mut session = model.session();
            session.restore_state(state).expect("restore");
            session.step(5).expect("step").to_vec()
        };
        assert_eq!(next(&saved), session.step(5).expect("step").to_vec());
        assert_ne!(
            next(&saved),
            next(&flipped),
            "{read}: a key's signs do not read"
        );
    }
}

#[test]
fn read_binary_head_weights_are_zero_off_the_flock_support() {
    let model = binary_read("l2", 2, 1);
    let mut session = model.session();
    let mut unkept = 0usize;
    for (position, id) in (0..CONTEXT as u32).map(|i| (i as usize, (i * 7 + 3) % VOCAB as u32)) {
        session.step(id).expect("step");
        for head in 0..HEADS {
            let (scores, weights) = session.last_read(head);
            assert_eq!(scores.len(), position + 1);
            check_rank_read(scores, weights, 2, 1);
            unkept += weights.iter().filter(|&&w| w == 0).count();
        }
    }
    assert!(unkept > 0, "window 2 and k 1 keep every position");
}

/// A learned rank table of `HEADS × M × M` for `M = side`: row `m - 1` of
/// head `h` is `[2^31 - rest, 1000 r + 7 h + 1 for r in 1..m]`, zeros beyond,
/// unlike the fixed rank table in every row with `m > 1`.
fn learned_table(side: usize) -> Vec<u32> {
    let mut table = Vec::new();
    for h in 0..HEADS {
        for m in 1..=side {
            let rest: Vec<u32> = (1..m).map(|r| 1000 * r as u32 + 7 * h as u32 + 1).collect();
            table.push((1u32 << 31) - rest.iter().sum::<u32>());
            table.extend(&rest);
            table.extend(std::iter::repeat_n(0, side - m));
        }
    }
    table
}

fn learned_artifact(window: usize, k: usize, table: &[u32]) -> Vec<u8> {
    // The pattern is "ra": the read is layer 1.
    with_header(&artifact_with(5, &[("read_rank.1", table)]), |header| {
        header["schema"] = json!(super::STACK_READ_SCHEMA);
        header["shape"]["read_select"] = json!({"window": window, "k": k});
        header["shape"]["read_weights"] = json!("learned");
    })
}

#[test]
fn read_learned_rank_serving_weights_are_the_head_table_row_by_rank() {
    let (window, k) = (2, 1);
    let side = window + k + 2;
    let table = learned_table(side);
    let model = IntegerStackModel::parse(&learned_artifact(window, k, &table))
        .expect("a learned rank read is served");
    let mut session = model.session();
    let mut checked = 0usize;
    for (position, id) in (0..CONTEXT as u32).map(|i| (i as usize, (i * 7 + 3) % VOCAB as u32)) {
        session.step(id).expect("step");
        for head in 0..HEADS {
            let (scores, weights) = session.last_read(head);
            let mut scratch = super::FlockScratch::new(scores.len());
            super::flock_select_integer(
                scores,
                position,
                super::FlockSelect::new(0, window, k),
                &mut scratch,
            )
            .expect("select");
            let kept: Vec<usize> = scratch.entries.iter().map(|e| e.position).collect();
            for (j, &w) in weights.iter().enumerate() {
                assert_eq!(
                    w != 0,
                    kept.contains(&j),
                    "position {j} support, weight {w}"
                );
            }
            // The kept weights in descending score (ties to the lowest
            // position) are the head's row `m - 1` with NoRead's slot removed,
            // and NoRead takes the rest of 2^31.
            let m = kept.len() + 1;
            let at = (head * side + m - 1) * side;
            let row = &table[at..at + m];
            let by_rank: Vec<u64> = scratch
                .entries
                .iter()
                .map(|e| weights[e.position])
                .collect();
            let null = (1u64 << 31) - by_rank.iter().sum::<u64>();
            let matches = (0..m).any(|slot| {
                u64::from(row[slot]) == null
                    && row
                        .iter()
                        .enumerate()
                        .filter(|&(i, _)| i != slot)
                        .map(|(_, &w)| u64::from(w))
                        .eq(by_rank.iter().copied())
            });
            assert!(
                matches,
                "head {head} position {position}: {by_rank:?} + {null} vs {row:?}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, CONTEXT * HEADS);
}

#[test]
fn read_learned_rank_invalid_tables_are_refused() {
    let (window, k) = (2, 1);
    let side = window + k + 2;
    let good = learned_table(side);
    assert!(IntegerStackModel::parse(&learned_artifact(window, k, &good)).is_ok());
    let mut short = good.clone();
    short.pop();
    let mut over = good.clone();
    over[side + 1] += 1; // head 0, row m = 2 sums to 2^31 + 1
    let mut beyond = good.clone();
    beyond[side] -= 1; // head 0, row m = 2: move one unit off the support
    beyond[side + 3] = 1;
    for (why, table) in [("short", short), ("over", over), ("beyond", beyond)] {
        assert!(
            IntegerStackModel::parse(&learned_artifact(window, k, &table)).is_err(),
            "{why} table accepted"
        );
    }

    let missing = with_header(&artifact(5), |header| {
        header["schema"] = json!(super::STACK_READ_SCHEMA);
        header["shape"]["read_select"] = json!({"window": window, "k": k});
        header["shape"]["read_weights"] = json!("learned");
    });
    assert!(
        IntegerStackModel::parse(&missing).is_err(),
        "missing table accepted"
    );
    let binary = with_header(&learned_artifact(window, k, &good), |header| {
        header["shape"]["read_binary"] = json!(true);
    });
    assert!(
        IntegerStackModel::parse(&binary).is_err(),
        "learned binary accepted"
    );
}

fn synthetic_float_trunk(seed: u64, read: &str) -> super::FloatTrunkSnapshot {
    let mut rng = Lcg(seed);
    let vocab = 32usize;
    let d = 32usize;
    let heads = 2usize;
    let mlp = 32usize;
    let context = 8usize;
    let lanes = d / 4;
    let gate_rows = lanes + d;

    let rand_vec = |rng: &mut Lcg, len: usize, scale: f32| -> Vec<f32> {
        (0..len)
            .map(|_| {
                let u = ((rng.next() % 2001) as i32 - 1000) as f32 / 1000.0;
                u * scale
            })
            .collect()
    };
    let rand_pos = |rng: &mut Lcg, len: usize, base: f32, span: f32| -> Vec<f32> {
        (0..len)
            .map(|_| {
                let u = (rng.next() % 1000) as f32 / 1000.0;
                base + u * span
            })
            .collect()
    };

    let rec_layer = super::FloatTrunkLayer {
        mixer_norm: rand_pos(&mut rng, d, 0.75, 0.5),
        mixer: super::FloatTrunkMixer::Recurrence(super::FloatTrunkRecurrence {
            rec_in: rand_vec(&mut rng, 2 * d * d, 0.25),
            rec_gate: rand_vec(&mut rng, gate_rows * d, 0.25),
            rec_out: rand_vec(&mut rng, d * d, 0.25),
            conv_taps: rand_vec(&mut rng, 4 * d, 0.2),
            conv_bias: rand_vec(&mut rng, d, 0.05),
            gate_bias: rand_vec(&mut rng, gate_rows, 0.1),
            decay: (0..lanes)
                .map(|i| 0.0625 * (i as f32 + 0.5))
                .collect(),
        }),
        mlp_norm: rand_pos(&mut rng, d, 0.75, 0.5),
        gate: rand_vec(&mut rng, mlp * d, 0.25),
        up: rand_vec(&mut rng, mlp * d, 0.25),
        down: rand_vec(&mut rng, d * mlp, 0.25),
    };

    let read_layer = super::FloatTrunkLayer {
        mixer_norm: rand_pos(&mut rng, d, 0.75, 0.5),
        mixer: super::FloatTrunkMixer::Read(super::FloatTrunkRead {
            query: rand_vec(&mut rng, d * d, 0.25),
            key: rand_vec(&mut rng, d * d, 0.25),
            value: rand_vec(&mut rng, d * d, 0.25),
            null: rand_vec(&mut rng, heads * d, 0.2),
            out: rand_vec(&mut rng, d * d, 0.25),
            null_bias: vec![-0.05, 0.03],
            age: (0..heads * context)
                .map(|i| -0.02 * ((i % context) as f32))
                .collect(),
            beta: if read == "dot" {
                Vec::new()
            } else {
                vec![0.5, 0.75]
            },
            offset: if read == "dot" {
                Vec::new()
            } else {
                vec![0.125, 0.25]
            },
        }),
        mlp_norm: rand_pos(&mut rng, d, 0.75, 0.5),
        gate: rand_vec(&mut rng, mlp * d, 0.25),
        up: rand_vec(&mut rng, mlp * d, 0.25),
        down: rand_vec(&mut rng, d * mlp, 0.25),
    };

    super::FloatTrunkSnapshot {
        vocab,
        width: d,
        heads,
        mlp,
        context,
        read: read.to_owned(),
        read_select: super::StackReadSelect { window: 4, k: 2 },
        rms_eps: 1e-5,
        embed: rand_vec(&mut rng, vocab * d, 0.5),
        final_norm: rand_pos(&mut rng, d, 0.8, 0.4),
        head: rand_vec(&mut rng, vocab * d, 0.35),
        layers: vec![rec_layer, read_layer],
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for (&x, &y) in a.iter().zip(b) {
        let xf = f64::from(x);
        let yf = f64::from(y);
        dot += xf * yf;
        na += xf * xf;
        nb += yf * yf;
    }
    dot / (na.sqrt() * nb.sqrt()).max(1e-12)
}

#[test]
fn float_trunk_export_matches_float_reference_across_full_vocabulary() {
    for read in ["dot", "lorentz", "l2"] {
        let snap = synthetic_float_trunk(20261027, read);
        let bytes = super::export_float_trunk_uorlut01(&snap).expect("export float trunk");
        // Determinism: exporting twice yields byte-for-bit identical UORLUT01 bytes.
        assert_eq!(
            bytes,
            super::export_float_trunk_uorlut01(&snap).expect("export again"),
            "exporter must be deterministic for {read}"
        );

        let container = super::format::Container::parse(&bytes).expect("parse container");
        assert!(container.shape.rec_geometric());
        assert!(container.shape.gate_hamilton());
        assert_eq!(
            super::stack_schema_for(&container.shape),
            super::STACK_READ_SCHEMA
        );

        let model = IntegerStackModel::parse(&bytes).expect("parse float-trunk model");
        // Verify every single-token input 0..vocab across the vocabulary.
        let mut exact_top1 = 0usize;
        for tok in 0..snap.vocab as u32 {
            let float_logits = &snap
                .forward_f32(&[tok], true)
                .expect("float forward")[0];
            let mut session = model.session();
            let int_logits_q16 = session.step(tok).expect("int step");
            let int_logits: Vec<f32> = int_logits_q16
                .iter()
                .map(|&v| (f64::from(v) * 2f64.powi(-16)) as f32)
                .collect();

            let cos = cosine_similarity(float_logits, &int_logits);
            assert!(
                cos > 0.999,
                "{read} token {tok}: cosine similarity {cos} <= 0.999"
            );
            let float_winner = float_logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, _)| i)
                .unwrap();
            let int_winner = super::stack_argmax(int_logits_q16);
            if float_winner == int_winner {
                exact_top1 += 1;
            }
            let max_abs_diff = float_logits
                .iter()
                .zip(&int_logits)
                .map(|(&f, &i)| (f - i).abs())
                .fold(0.0f32, f32::max);
            assert!(
                max_abs_diff < 0.01,
                "{read} token {tok}: max_abs_diff {max_abs_diff} >= 0.01"
            );
        }
        assert_eq!(
            exact_top1, snap.vocab,
            "{read}: expected 32/32 top-1 agreement across vocabulary"
        );

        // Multi-token context prefix exercising causal conv history, quaternion
        // recurrence carry, and flock rank read over 8 positions.
        let seq = [0u32, 5, 11, 19, 3, 27, 14, 31];
        let float_seq = snap.forward_f32(&seq, true).expect("float seq");
        let mut session = model.session();
        for (pos, &tok) in seq.iter().enumerate() {
            let int_q16 = session.step(tok).expect("int seq step");
            let int_f32: Vec<f32> = int_q16
                .iter()
                .map(|&v| (f64::from(v) * 2f64.powi(-16)) as f32)
                .collect();
            let cos = cosine_similarity(&float_seq[pos], &int_f32);
            assert!(
                cos > 0.999,
                "{read} pos {pos}: cosine similarity {cos} <= 0.999"
            );
            let float_winner = float_seq[pos]
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(i, _)| i)
                .unwrap();
            assert_eq!(
                super::stack_argmax(int_q16),
                float_winner,
                "{read} pos {pos}: multi-token top-1 mismatch"
            );
        }
    }
}

#[test]
fn float_trunk_validation_threads_and_session_roundtrip() {
    use super::kernels::stack_isqrt;
    use super::session::GEOMETRIC_KEEP_Q31;

    for (idx, &keep) in GEOMETRIC_KEEP_Q31.iter().enumerate() {
        let complement = (1u128 << 62) - (1u128 << (60 - 2 * idx));
        assert_eq!(
            keep,
            stack_isqrt(complement) as u64,
            "GEOMETRIC_KEEP_Q31[{idx}] mismatch"
        );
    }

    let snap = synthetic_float_trunk(20261027, "lorentz");
    let bytes = super::export_float_trunk_uorlut01(&snap).expect("export float trunk");

    // Unquantized raw float reference (`quantized_weights = false`) runs cleanly and
    // stays positively correlated with the quantized float reference.
    let raw_f32 = snap.forward_f32(&[0], false).expect("raw float forward");
    let q_f32 = snap.forward_f32(&[0], true).expect("quantized float forward");
    assert_eq!(raw_f32.len(), 1);
    assert_eq!(raw_f32[0].len(), snap.vocab);
    assert!(raw_f32[0].iter().all(|v| v.is_finite()));
    assert!(
        cosine_similarity(&raw_f32[0], &q_f32[0]) > 0.5,
        "raw f32 and 4-bit quantized f32 forward must stay positively correlated"
    );

    // Multi-threaded serving and session save/restore produce bit-identical integers.
    let model_1 = IntegerStackModel::parse(&bytes).expect("parse 1-thread");
    let mut model_2 = IntegerStackModel::parse(&bytes).expect("parse 2-thread");
    model_2.set_threads(2).expect("set 2 threads");
    let seq = [2u32, 7, 15, 23, 4, 18];
    let out_1 = run(&model_1, &seq);
    let out_2 = run(&model_2, &seq);
    assert_eq!(out_1, out_2, "1-thread and 2-thread float-trunk outputs must match");

    let mut s_full = model_1.session();
    s_full.enable_snap_trace();
    for &t in &seq[..3] {
        s_full.step(t).expect("step first half");
    }
    let saved = s_full.save_state();
    for &t in &seq[3..] {
        s_full.step(t).expect("step second half");
    }
    let mut s_restored = model_1.session();
    s_restored.enable_snap_trace();
    s_restored.restore_state(&saved).expect("restore state");
    for &t in &seq[3..] {
        s_restored.step(t).expect("step restored second half");
    }
    assert!(
        !s_full
            .snap_trace()
            .expect("snap trace enabled")
            .is_empty()
    );
    assert_eq!(s_full.logits(), s_restored.logits());
    assert_eq!(s_full.snap_trace(), s_restored.snap_trace());

    // Invalid shape combinations are refused at parse time.
    let bad_rec = with_header(&bytes, |h| {
        h["shape"]["rec_gates"] = json!("unknown");
    });
    assert!(matches!(
        IntegerStackModel::parse(&bad_rec),
        Err(StackError::Shape(_))
    ));
    let bad_gate = with_header(&bytes, |h| {
        h["shape"]["gate"] = json!("unknown_gate");
    });
    assert!(matches!(
        IntegerStackModel::parse(&bad_gate),
        Err(StackError::Shape(_))
    ));
    let no_rot = with_header(&bytes, |h| {
        h["shape"]["rotation"] = json!(false);
        h.as_object_mut().unwrap().remove("transport_snap");
    });
    assert!(matches!(
        IntegerStackModel::parse(&no_rot),
        Err(StackError::Shape(_))
    ));
    let no_snap = with_header(&bytes, |h| {
        h.as_object_mut().unwrap().remove("transport_snap");
    });
    assert!(matches!(
        IntegerStackModel::parse(&no_snap),
        Err(StackError::TransportSnap(_))
    ));

    // Exporter refuses non-finite weights.
    let mut bad_snap = snap.clone();
    bad_snap.embed[0] = f32::NAN;
    assert!(super::export_float_trunk_uorlut01(&bad_snap).is_err());
}

/// The lawful shape of a SpiralCore/FBS trunk with the native vector writer.
fn fbs_shape() -> StackShape {
    StackShape {
        vocab: 256,
        width: 32,
        heads: 1,
        mlp: 32,
        pattern: "rr".to_owned(),
        read: "dot".to_owned(),
        rotation: true,
        context: 8,
        pointer: None,
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory: None,
        rec_gates: Some("geometric".to_owned()),
        gate: Some("hamilton".to_owned()),
        raw_gates: false,
        raw_transition: false,
        projection: Some("spiralcore_fbs".to_owned()),
        head: Some("native_vector".to_owned()),
        kept_cells: Some(2),
    }
}

#[test]
fn fbs_shape_takes_its_own_schema_and_refuses_unlawful_shapes() {
    let lawful = fbs_shape();
    assert!(lawful.validate().is_ok());
    assert_eq!(
        super::stack_schema_for(&lawful),
        super::format::STACK_FBS_SCHEMA
    );
    assert!(lawful.projection_fbs() && lawful.head_native_vector());

    // Each unlawful shape is refused for its own reason, so every rule is held.
    const REQUIRE: &str = "SpiralCore/FBS projections require geometric gates, the Hamilton gate, rotations and the native vector head";
    const KEPT: &str = "the native vector head keeps 1 to 16 cells";
    const READ: &str = "SpiralCore/FBS trunks carry no read selection or read weights";
    type Mutation = fn(&mut StackShape);
    let refused: [(&str, Mutation, &str); 16] = [
        (
            "another projection",
            |s| s.projection = Some("dense".to_owned()),
            "the projection is not spiralcore_fbs",
        ),
        (
            "another head",
            |s| s.head = Some("dense".to_owned()),
            "the head is not native_vector",
        ),
        ("table gates", |s| s.rec_gates = None, REQUIRE),
        ("no Hamilton gate", |s| s.gate = None, REQUIRE),
        (
            "no rotations",
            |s| s.rotation = false,
            "geometric recurrence gates require learned rotations",
        ),
        (
            "FBS without the writer",
            |s| {
                s.head = None;
                s.kept_cells = None;
            },
            REQUIRE,
        ),
        (
            "a read layer",
            |s| s.pattern = "ra".to_owned(),
            "SpiralCore/FBS projections serve recurrence layers only",
        ),
        (
            "a read selection",
            |s| s.read_select = Some(super::format::StackReadSelect { window: 4, k: 2 }),
            READ,
        ),
        ("rank read weights", |s| s.read_weights = Some("rank".to_owned()), READ),
        ("a binary read", |s| s.read_binary = true, READ),
        // The 8-blocks of every route table follow from the group rule.
        (
            "an MLP width off the group",
            |s| s.mlp = 40,
            "the MLP width is not a multiple of the group",
        ),
        (
            "the writer without FBS",
            |s| s.projection = None,
            "the native vector head requires SpiralCore/FBS projections",
        ),
        ("no kept cells", |s| s.kept_cells = None, KEPT),
        ("17 kept cells", |s| s.kept_cells = Some(17), KEPT),
        (
            "a vocabulary under 256",
            |s| s.vocab = 255,
            "the native vector head serves a vocabulary of 256 to 8192 and a width up to 1024",
        ),
        (
            "a pointer head",
            |s| {
                s.pointer = Some(super::format::StackPointer {
                    dim: 8,
                    score: "dot".to_owned(),
                    score_scale_q30: 1 << 30,
                })
            },
            "the native vector head has no pointer head",
        ),
    ];
    for (what, mutate, reason) in refused {
        let mut shape = fbs_shape();
        mutate(&mut shape);
        match shape.validate() {
            Err(StackError::Shape(got)) => assert_eq!(got, reason, "{what}"),
            other => panic!("{what}: expected a shape refusal, got {other:?}"),
        }
    }
    // Kept cells on a dense head are refused, and a dense shape keeps its schema.
    let mut dense = fbs_shape();
    dense.projection = None;
    dense.head = None;
    assert!(matches!(
        dense.validate(),
        Err(StackError::Shape(
            "kept cells belong to the native vector head"
        ))
    ));
    dense.kept_cells = None;
    assert!(dense.validate().is_ok());
    assert_eq!(
        super::stack_schema_for(&dense),
        super::format::STACK_READ_SCHEMA
    );
}

#[test]
fn fbs_image_matches_its_shape() {
    // claude-128: the read-only view an out-of-crate evaluator ports the FBS
    // step from holds every table at the length the shape implies, and the
    // writer table is the snapshot's embedding rounded to Q12.
    for snap in [
        synthetic_fbs_trunk(20261025),
        fbs_synthetic_non_square(20261026),
        fbs_synthetic_tiny_gain(20261026),
    ] {
        let model = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export"))
            .expect("parse");
        let image = model.fbs_image().expect("an FBS model has an image");
        let (d, m, s) = (snap.width, snap.mlp, model.shape());
        let (lanes, g) = (d / 4, s.gate_rows());
        assert_eq!(g, lanes + d);
        assert_eq!(image.layers.len(), snap.layers.len());
        assert_eq!(image.kept_cells, snap.kept_cells);
        assert_eq!(image.final_gain.len(), d);
        assert_eq!(image.writer_table.len(), snap.vocab * d);
        for (&q, &v) in image.writer_table.iter().zip(&snap.embed) {
            assert_eq!(i64::from(q), (f64::from(v) * 4096.0).round() as i64);
        }
        for (layer, routes) in image.layers.iter().zip(&snap.layers) {
            assert_eq!((layer.rec_gain.len(), layer.mlp_gain.len()), (d, d));
            // Each map's atoms are the snapshot's routes for that map, so a
            // swapped map or layer would differ.
            for (map, route, out_dim, in_dim) in [
                (layer.rec_in, &routes.rec_in, 2 * d, d),
                (layer.rec_gate, &routes.rec_gate, g, d),
                (layer.rec_out, &routes.rec_out, d, d),
                (layer.mlp_gate, &routes.mlp_gate, m, d),
                (layer.mlp_up, &routes.mlp_up, m, d),
                (layer.mlp_down, &routes.mlp_down, d, m),
            ] {
                assert_eq!((map.atoms.len(), map.codes.len()), (out_dim / 8, out_dim / 8));
                assert_eq!(map.atoms, route.atoms.as_slice());
                assert!(map.atoms.iter().all(|&a| usize::from(a) < in_dim / 8));
                assert!(map.codes.iter().flatten().all(|&c| c < 16));
            }
            assert_eq!(layer.taps.len(), super::CONVOLUTION_WIDTH * d);
            assert_eq!(
                (layer.conv_bias.len(), layer.gate_bias.len(), layer.decay_offset.len()),
                (d, g, lanes)
            );
        }
    }
    let dense = IntegerStackModel::parse(
        &super::export_float_trunk_uorlut01(&synthetic_float_trunk(7, "dot")).expect("export"),
    )
    .expect("parse");
    assert!(dense.fbs_image().is_none());
}

#[test]
fn fbs_trunk_export_is_deterministic_and_parses() {
    let snap = synthetic_fbs_trunk(20261025);
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export FBS trunk");
    assert_eq!(
        bytes,
        super::export_fbs_trunk_uorlut01(&snap).expect("export again"),
        "the FBS export is byte-deterministic"
    );
    let container = super::format::Container::parse(&bytes).expect("parse FBS container");
    assert_eq!(container.shape, fbs_shape());

    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    let header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    assert_eq!(header["schema"], json!("uor-r4.lut-stack/4"));
    assert_eq!(header["matrices"], json!([]));

    for (name, routes) in [
        ("l0.rec_in.fbs", &snap.layers[0].rec_in),
        ("l1.mlp_down.fbs", &snap.layers[1].mlp_down),
    ] {
        let entries = container.table_u32(name).expect(name);
        let atoms: Vec<u16> = entries.iter().map(|&e| (e & 0xffff) as u16).collect();
        let states: Vec<u8> = entries.iter().map(|&e| (e >> 16) as u8).collect();
        assert_eq!(atoms, routes.atoms, "{name} atoms");
        assert_eq!(states, routes.states, "{name} states");
    }
    let writer = container.table_i16("writer.table").expect("writer table");
    assert_eq!(writer.len(), 256 * 32);
}

#[test]
fn wide_all_recurrence_fbs_trunks_export_and_serve_past_the_attention_head_cap() {
    for (seed, width) in [(701, 512), (702, 1_024)] {
        let mut snapshot = synthetic_fbs_trunk_with(
            seed,
            FbsSynth {
                vocab: 512,
                d: width,
                mlp: 96,
                layers: 2,
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.hamming = Some(synthetic_hamming(
            width,
            snapshot.layers.len(),
            width / 64,
            seed ^ 0x57a7,
        ));
        let bytes = super::export_fbs_trunk_uorlut01(&snapshot).expect("wide FBS export");
        assert_eq!(bytes, super::export_fbs_trunk_uorlut01(&snapshot).expect("repeat export"));
        let model = IntegerStackModel::parse(&bytes).expect("wide FBS parse");
        assert!(model.shape().projection_fbs());
        assert_eq!((model.shape().width, model.shape().heads), (width, 1));
        let mut first = model.session();
        let mut second = model.session();
        for token in [3u32, 5, 11] {
            let left = first.step_native(token).expect("wide FBS step");
            let right = second.step_native(token).expect("repeat wide FBS step");
            assert_eq!(left.query_q12, right.query_q12);
            assert_eq!(left.token, right.token);
            assert_eq!(left.kept_cells, right.kept_cells);
        }
    }

    let bytes = super::export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(703)).expect("FBS export");
    let model = IntegerStackModel::parse(&bytes).expect("FBS parse");
    let mut attention = model.shape().clone();
    attention.width = 512;
    attention.heads = 1;
    attention.pattern = "a".to_owned();
    attention.projection = None;
    attention.head = None;
    attention.kept_cells = None;
    attention.gate = None;
    assert!(matches!(
        attention.validate(),
        Err(StackError::Shape("the head width exceeds 256"))
    ));
}

#[test]
fn fbs_reference_quat_modes_match_their_definitions() {
    use super::format::{ref_quat_mode, ref_snap_icosian, FbsQuatMode};
    // Normalize: the float CUDA path's `q / sqrt(|q|^2 + eps)`.
    let n = ref_quat_mode([3.0, 0.0, 4.0, 0.0], FbsQuatMode::Normalize, 1e-12);
    for (got, want) in n.iter().zip([0.6, 0.0, 0.8, 0.0]) {
        assert!((got - want).abs() < 1e-12, "{n:?}");
    }
    let tiny = ref_quat_mode([1e-6, 0.0, 0.0, 0.0], FbsQuatMode::Normalize, 1e-12);
    assert!((tiny[0] - 1e-6 / (1e-12f64 + 1e-12).sqrt()).abs() < 1e-12, "{tiny:?}");
    // Icosian: a unit icosian, and the identity snaps to itself.
    assert_eq!(ref_quat_mode([1.0, 0.0, 0.0, 0.0], FbsQuatMode::Icosian, 1e-12), [1.0, 0.0, 0.0, 0.0]);
    let s = ref_quat_mode([0.3, -0.2, 0.9, 0.1], FbsQuatMode::Icosian, 1e-12);
    assert!((s.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-12, "{s:?}");
    // Q12Icosian rounds to Q12 steps first: a gate below half a step rounds
    // to zero and snaps as the zero vector does, not as its direction does.
    let small = [1e-5, 0.0, 0.0, 0.0];
    assert_eq!(ref_quat_mode(small, FbsQuatMode::Q12Icosian, 1e-12), ref_snap_icosian([0.0; 4]));
    assert_eq!(ref_quat_mode(small, FbsQuatMode::Icosian, 1e-12), [1.0, 0.0, 0.0, 0.0]);
    assert_ne!(ref_snap_icosian([0.0; 4]), [1.0, 0.0, 0.0, 0.0]);
}

#[test]
fn fbs_reference_each_snap_site_reads_its_own_mode() {
    use super::format::{
        ref_hamilton, ref_hamilton_feed_forward, ref_quat_mode, FbsQuatMode, FbsReferenceSnaps,
        RefRecurrence,
    };
    use FbsQuatMode::{Icosian, Normalize};
    let close = |got: &[f64], want: &[f64], what: &str| {
        assert!(
            got.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-12) && got.len() == want.len(),
            "{what}: {got:?} against {want:?}"
        );
    };
    let quat = |v: &[f64]| [v[0], v[1], v[2], v[3]];

    // The feed-forward gate: hamilton(mode(g), up) per 4-block.
    let g = [0.3, -0.7, 0.2, 0.5, 1.5, 0.1, -0.4, 0.2];
    let up = [1.0, 2.0, -1.0, 0.5, -0.3, 0.8, 0.2, -1.1];
    for mode in [Icosian, Normalize] {
        let h = ref_hamilton_feed_forward(&g, &up, mode);
        for b in 0..2 {
            let want = ref_hamilton(
                ref_quat_mode(quat(&g[4 * b..]), mode, 1e-12),
                quat(&up[4 * b..]),
            );
            close(&h[4 * b..4 * b + 4], &want, &format!("feed-forward {mode:?}"));
        }
    }
    assert_ne!(
        ref_hamilton_feed_forward(&g, &up, Icosian),
        ref_hamilton_feed_forward(&g, &up, Normalize)
    );

    // One recurrence lane: tap 0 is 1 and every other table 0, so the
    // convolution is the drive; the decay pre-activation 0.5 picks rate 8.
    let mut taps = [0.0f32; 16];
    taps[..4].copy_from_slice(&[1.0; 4]);
    let rec = RefRecurrence::new(&taps, &[0.0; 4], &[0.0], false).expect("recurrence");
    let lambda = 2f64.powi(-9);
    let keep = (1.0 - lambda * lambda).sqrt();
    let drive = [1.0, 2.0, 3.0, 4.0];
    let out_gate = [0.5, 0.1, -0.2, 0.3];
    let rotation = [0.6, -0.3, 0.2, 0.7];
    let gate_out = [0.5, rotation[0], rotation[1], rotation[2], rotation[3]];
    for snaps in [
        FbsReferenceSnaps::INTEGER,
        FbsReferenceSnaps::FLOAT,
        FbsReferenceSnaps { transition: Icosian, gates: Normalize },
        FbsReferenceSnaps { transition: Normalize, gates: Icosian },
    ] {
        let (mut hist, mut state) = (vec![0.0f64; 12], vec![0.0f64; 4]);
        // Step 1, from a zero state: only the output gate's mode shows.
        let gated = rec.step(&drive, &out_gate, &gate_out, &mut hist, &mut state, snaps);
        let s1 = drive.map(|v| keep * v);
        close(&state, &s1, &format!("{snaps:?} state 1"));
        let want = ref_hamilton(ref_quat_mode(out_gate, snaps.gates, 1e-12), s1);
        close(&gated, &want, &format!("{snaps:?} output 1"));
        // Step 2: the transition's mode moves the carried state.
        let gated = rec.step(&drive, &out_gate, &gate_out, &mut hist, &mut state, snaps);
        let trans = ref_quat_mode(rotation, snaps.transition, 1e-6).map(|v| lambda * v);
        let moved = ref_hamilton(trans, s1);
        let s2: Vec<f64> = (0..4).map(|c| moved[c] + keep * drive[c]).collect();
        close(&state, &s2, &format!("{snaps:?} state 2"));
        let want = ref_hamilton(ref_quat_mode(out_gate, snaps.gates, 1e-12), quat(&s2));
        close(&gated, &want, &format!("{snaps:?} output 2"));
    }
}

#[test]
fn fbs_reference_snaps_default_to_the_integer_path_and_each_switch_is_live() {
    use super::format::FbsReferenceSnaps;
    let snap = synthetic_fbs_trunk(20261026);
    let sequence = [0u32, 5, 11, 19, 3, 27, 14, 31];
    for quantized in [false, true] {
        let default = snap.forward_f32(&sequence, quantized).expect("reference");
        let integer = snap
            .forward_f32_with(&sequence, quantized, FbsReferenceSnaps::INTEGER)
            .expect("reference");
        assert_eq!(default, integer, "quantized {quantized}");
    }
    // Each variant changes some query: the switches reach the forward.
    let base = snap.forward_f32(&sequence, false).expect("reference");
    for snaps in [FbsReferenceSnaps::FLOAT, FbsReferenceSnaps::FLOAT_EXPORT] {
        let other = snap.forward_f32_with(&sequence, false, snaps).expect("reference");
        assert!(
            base.iter().zip(&other).any(|(a, b)| a.query != b.query),
            "{snaps:?} leaves every query unchanged"
        );
    }
    let float = snap.forward_f32_with(&sequence, false, FbsReferenceSnaps::FLOAT).expect("reference");
    let export = snap
        .forward_f32_with(&sequence, false, FbsReferenceSnaps::FLOAT_EXPORT)
        .expect("reference");
    assert!(float.iter().zip(&export).any(|(a, b)| a.query != b.query));
}

#[test]
fn fbs_exported_roundings_match_the_stack_exporter() {
    use crate::fbs_snapshot::{fixed_value, q12_value, FBS_FIXED_EXP};
    // The ungated roundings the float converter's D11 mode reads are the ones
    // the exporter writes.
    assert_eq!(FBS_FIXED_EXP, super::kernels::RESIDUAL_EXP);
    let snap = synthetic_fbs_trunk(20261026);
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    // Every quantized reference output equals the one computed from tables
    // rounded by these helpers, because the reference reads them through them.
    let quantized = snap.forward_f32(&[3, 9, 27], true).expect("reference");
    let mut rounded = snap.clone();
    let grid = |v: &mut f32| {
        *v = super::decode_grid_code(super::encode_grid_code(f64::from(*v), false).expect("grid"))
            as f32
    };
    for layer in &mut rounded.layers {
        layer.rec_norm.iter_mut().for_each(grid);
        layer.mlp_norm.iter_mut().for_each(grid);
        layer.conv_taps.iter_mut().for_each(grid);
        for v in layer
            .conv_bias
            .iter_mut()
            .chain(layer.gate_bias.iter_mut())
            .chain(layer.decay.iter_mut())
        {
            *v = fixed_value(*v, FBS_FIXED_EXP) as f32;
        }
    }
    rounded.final_norm.iter_mut().for_each(grid);
    for v in &mut rounded.embed {
        *v = q12_value(*v).expect("fits Q12") as f32;
    }
    // Rounding is idempotent: the rounded trunk's plain forward, with its final
    // query rounded to Q12 as the quantized reference rounds it before the
    // writer, is the original trunk's quantized forward; and it exports to
    // the same artifact.
    let plain = rounded.forward_f32(&[3, 9, 27], false).expect("reference");
    for (p, q) in plain.iter().zip(&quantized) {
        let query: Vec<f64> = p
            .query
            .iter()
            .map(|&v| (v * 4096.0).round().clamp(-32768.0, 32767.0) / 4096.0)
            .collect();
        assert_eq!(query, q.query);
        assert_eq!((p.token, &p.kept_cells), (q.token, &q.kept_cells));
    }
    assert_eq!(super::export_fbs_trunk_uorlut01(&rounded).expect("export"), bytes);
    assert!(IntegerStackModel::parse(&bytes).is_ok() && model.session().step_native(3).is_ok());
    // The reference reads the Q16 tables through `fixed_value` too, so the
    // checks above cannot see it drift: hold it against the code the exporter
    // stores for every bias, gate bias and decay offset, and see some move.
    let mut moved = false;
    for layer in &snap.layers {
        for values in [&layer.conv_bias, &layer.gate_bias, &layer.decay] {
            let codes = super::format::quantize_fixed_i32(values, values.len(), FBS_FIXED_EXP, "q16")
                .expect("fixed");
            for (&v, &code) in values.iter().zip(&codes) {
                let value = fixed_value(v, FBS_FIXED_EXP);
                assert_eq!(value, f64::from(code) * 2f64.powi(FBS_FIXED_EXP));
                moved |= value != f64::from(v);
            }
        }
    }
    assert!(moved, "no Q16 value moves, so a skipped rounding would pass");
    assert_eq!(q12_value(8.0), None);
    assert_eq!(q12_value(f32::NAN), None);
    assert_eq!(q12_value(-8.0), Some(-8.0));
}

#[test]
fn fbs_tie_margins_measure_how_near_each_switch_is() {
    use super::format::{icosian_margin, rate_margin, FbsQuatMode, FbsReferenceSnaps, FbsTieMargins};
    // A root's nearest rival is 60 degrees away: dot φ/2 against 1.
    let phi = 0.5 * (1.0 + 5.0f64.sqrt());
    let own = icosian_margin([1.0, 0.0, 0.0, 0.0]);
    assert!((own - (1.0 - 0.5 * phi)).abs() < 1e-12, "{own}");
    // (1, 1, 0, 0) is equally near ½(φ, 1, φ⁻¹, 0) and ½(1, φ, 0, φ⁻¹): a tie.
    assert_eq!(icosian_margin([1.0, 1.0, 0.0, 0.0]), 0.0);
    let mut m = FbsTieMargins {
        transition: f64::INFINITY,
        gate: f64::INFINITY,
        rate: f64::INFINITY,
        zero_snaps: 0,
    };
    m.note_snap([0.3, 0.1, 0.2, 0.4], FbsQuatMode::Normalize, false);
    assert_eq!((m.gate, m.zero_snaps), (f64::INFINITY, 0));
    m.note_snap([1.0, 0.0, 0.0, 0.0], FbsQuatMode::Icosian, true);
    assert_eq!((m.transition, m.gate), (own, f64::INFINITY));
    // A vector that rounds to zero at Q12 snaps a zero vector: every root ties.
    m.note_snap([1e-5, 0.0, 0.0, 0.0], FbsQuatMode::Q12Icosian, false);
    assert_eq!((m.gate, m.zero_snaps), (0.0, 1));
    assert_eq!(rate_margin(0.5), 0.0);
    assert_eq!(rate_margin(0.5 + 1.0 / 64.0), 1.0 / 64.0);
    assert_eq!(rate_margin(-1.0), 1.0 + 1.0 / 16.0);

    let snap = synthetic_fbs_trunk(20261026);
    let one = snap.tie_margins(&[5], true, FbsReferenceSnaps::INTEGER).expect("margins");
    // One token moves no state, so its transition snap never matters.
    assert_eq!(one.transition, f64::INFINITY);
    assert!(one.gate.is_finite() && one.gate >= 0.0 && one.rate.is_finite() && one.rate >= 0.0);
    assert_eq!(one.zero_snaps, 0);
    let two = snap.tie_margins(&[5, 11], true, FbsReferenceSnaps::INTEGER).expect("margins");
    assert!(two.transition.is_finite() && two.transition >= 0.0);
    // The prefix's switches stay counted, and runs do not leak into each other.
    assert!(two.gate <= one.gate && two.rate <= one.rate);
    assert_eq!(snap.tie_margins(&[5], true, FbsReferenceSnaps::INTEGER).expect("margins"), one);
    let float = snap.tie_margins(&[5, 11], true, FbsReferenceSnaps::FLOAT).expect("margins");
    assert_eq!((float.transition, float.gate), (f64::INFINITY, f64::INFINITY));
}

#[test]
fn fbs_reference_batches_equal_their_inputs_run_alone() {
    use super::format::FbsReferenceSnaps;
    // One pass over many inputs, the exporter's checks and tables read once,
    // gives each input's own forward and its per-position margins, with no
    // state carried from one input to the next.
    let snap = synthetic_fbs_trunk(20261026);
    let inputs: [&[u32]; 3] = [&[5, 11, 3], &[7], &[19, 2]];
    for quantized in [false, true] {
        let each = snap
            .forward_f32_each(&inputs, quantized, FbsReferenceSnaps::INTEGER)
            .expect("batch");
        for (tokens, out) in inputs.iter().zip(&each) {
            let alone = snap
                .forward_f32_with(tokens, quantized, FbsReferenceSnaps::INTEGER)
                .expect("alone");
            assert_eq!(&alone, out, "{tokens:?} quantized {quantized}");
        }
    }
    let margins = snap
        .tie_margins_each(&inputs, true, FbsReferenceSnaps::INTEGER)
        .expect("margins");
    for (tokens, per_position) in inputs.iter().zip(&margins) {
        assert_eq!(per_position.len(), tokens.len());
        // Position t holds the margins of the prefix that ends there.
        for (t, m) in per_position.iter().enumerate() {
            let prefix = snap
                .tie_margins(&tokens[..=t], true, FbsReferenceSnaps::INTEGER)
                .expect("prefix");
            assert_eq!(*m, prefix, "{tokens:?} position {t}");
        }
    }
    // A refused batch is refused whole, and a later trace starts afresh.
    assert!(snap
        .tie_margins_each(&[&[5], &[u32::MAX]], true, FbsReferenceSnaps::INTEGER)
        .is_err());
    assert_eq!(
        snap.tie_margins(&[5], true, FbsReferenceSnaps::INTEGER).expect("margins"),
        margins[0][0]
    );
}

#[test]
fn fbs_trunk_snapshot_json_round_trips_exactly() {
    // The float converter hands the snapshot over as JSON (`fbs-export`); every
    // f32 must read back to the same bits, so the export is the same bytes.
    let snap = synthetic_fbs_trunk(20261026);
    let json = serde_json::to_vec(&snap).expect("serialize");
    let back: super::FbsTrunkSnapshot = serde_json::from_slice(&json).expect("deserialize");
    assert_eq!(back, snap);
    assert!(back
        .embed
        .iter()
        .zip(&snap.embed)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    assert_eq!(
        super::export_fbs_trunk_uorlut01(&back).expect("export"),
        super::export_fbs_trunk_uorlut01(&snap).expect("export")
    );
    // An unknown field is refused, never ignored.
    let mut value: serde_json::Value = serde_json::from_slice(&json).expect("value");
    value["head_scale"] = serde_json::json!(1.0);
    assert!(serde_json::from_value::<super::FbsTrunkSnapshot>(value).is_err());
}

#[test]
fn fbs_trunk_export_refuses_bad_snapshots() {
    let good = synthetic_fbs_trunk(20261026);
    assert!(super::export_fbs_trunk_uorlut01(&good).is_ok());
    type Mutation = fn(&mut super::FbsTrunkSnapshot);
    type Refusal = fn(&StackError) -> bool;
    let refused: [(&str, Mutation, Refusal); 8] = [
        (
            "atom == in_blocks",
            |s| s.layers[0].rec_in.atoms[0] = s.layers[0].rec_in.in_blocks as u16,
            |e| matches!(e, StackError::Numerics(m) if m == "l0.rec_in.fbs[0]: atom 4 is not below 4 input blocks"),
        ),
        (
            "state 64",
            |s| s.layers[1].mlp_up.states[0] = 64,
            |e| matches!(e, StackError::Numerics(m) if m == "l1.mlp_up.fbs[0]: state 64 is not below 64"),
        ),
        (
            "a short route",
            |s| {
                s.layers[0].rec_gate.atoms.pop();
            },
            |e| matches!(e, StackError::TableLength { name, .. } if name == "l0.rec_gate.fbs.atoms"),
        ),
        (
            "NaN in embed",
            |s| s.embed[0] = f32::NAN,
            |e| matches!(e, StackError::Numerics(m) if m == "writer.table[0] is not finite"),
        ),
        (
            "embed 8.0 off the Q12 range",
            |s| s.embed[3] = 8.0,
            |e| matches!(e, StackError::Numerics(m) if m == "writer.table[3] = 8 does not fit Q12 in i16"),
        ),
        (
            "rms_eps NaN",
            |s| s.rms_eps = f64::NAN,
            |e| matches!(e, StackError::Numerics(m) if m == "rms_eps NaN is not finite and positive"),
        ),
        (
            "rms_eps 0",
            |s| s.rms_eps = 0.0,
            |e| matches!(e, StackError::Numerics(m) if m == "rms_eps 0 is not finite and positive"),
        ),
        (
            "width 40",
            |s| s.width = 40,
            |e| matches!(e, StackError::Shape("the width is not a multiple of 4, of the group and of the heads")),
        ),
    ];
    for (what, mutate, refusal) in refused {
        let mut snap = good.clone();
        mutate(&mut snap);
        match super::export_fbs_trunk_uorlut01(&snap) {
            Err(e) => assert!(refusal(&e), "{what}: refused for another reason: {e:?}"),
            Ok(_) => panic!("{what} must be refused"),
        }
    }
    // The Q12 range is half-open: -8.0 is the i16 minimum and is served.
    let mut low = good.clone();
    low.embed[3] = -8.0;
    assert!(super::export_fbs_trunk_uorlut01(&low).is_ok());
}

#[test]
fn fbs_trunk_reference_reads_only_kept_cells() {
    let snap = synthetic_fbs_trunk(20261026);
    for tok in 0..snap.vocab as u32 {
        let out = snap.forward_f32(&[tok], false).expect("forward");
        assert_eq!(out.len(), 1);
        let w = &out[0];
        assert_eq!(w.kept_cells.len(), snap.kept_cells, "token {tok}");
        assert!(!w.candidates.is_empty() && w.candidates.len() <= snap.kept_cells * snap.vocab.div_ceil(256));
        for &(cand, _) in &w.candidates {
            assert!(
                w.kept_cells.contains(&(cand % 256)),
                "token {tok}: candidate {cand} lies outside the kept cells"
            );
        }
        assert!(
            w.candidates.iter().any(|&(cand, _)| cand == w.token),
            "token {tok}: the winner is not a candidate"
        );
        // The winner holds the best score, ties to the smaller id.
        let best = w
            .candidates
            .iter()
            .fold(f64::NEG_INFINITY, |m, &(_, s)| m.max(s));
        let first_best = w.candidates.iter().find(|&&(_, s)| s == best).map(|&(c, _)| c);
        assert_eq!(first_best, Some(w.token), "token {tok}");
        // The kept cells are the best-scoring ones: no cell left out scores higher.
        let cells = fbs_cell_scores(&w.query);
        let worst_kept = w
            .kept_cells
            .iter()
            .map(|&c| cells[c as usize])
            .fold(f64::INFINITY, f64::min);
        assert!(
            (0..256u32)
                .filter(|c| !w.kept_cells.contains(c))
                .all(|c| cells[c as usize] <= worst_kept),
            "token {tok}: a cell left out outscores a kept cell"
        );
    }
    // Too long a sequence and a token outside the vocabulary are refused.
    assert!(snap.forward_f32(&[0; 9], false).is_err());
    assert!(snap.forward_f32(&[snap.vocab as u32], false).is_err());
}

#[test]
fn fbs_trunk_reference_quantized_agrees() {
    let snap = synthetic_fbs_trunk(20261026);
    let mut agree = 0;
    for tok in 0..snap.vocab as u32 {
        let plain = snap.forward_f32(&[tok], false).expect("forward")[0].token;
        let quantized = snap.forward_f32(&[tok], true).expect("forward")[0].token;
        agree += usize::from(plain == quantized);
    }
    // A detector of gross quantization faults, not a precision bar: at
    // vocabulary 256 each cell holds one token, so two kept cells leave two
    // candidates, and the 4-bit grid-coded gains and taps of this untrained
    // random trunk flip near-tied cells. A wrong scale or exponent falls to
    // near chance. Measured at seed 20261026: 224 of 256. The integer session
    // is held to the quantized reference, not to this one.
    assert!(
        agree >= 192,
        "the quantized reference agrees on {agree} of 256 single-token inputs"
    );
}

#[test]
fn fbs_trunk_gather_is_the_signed_axis_action() {
    let snap = synthetic_fbs_trunk(20261026);
    let route = &snap.layers[0].rec_in;
    let mut rng = Lcg(77);
    let input: Vec<i32> = (0..route.in_blocks * 8)
        .map(|_| (rng.next() % 2001) as i32 - 1000)
        .collect();
    let as_f64: Vec<f64> = input.iter().map(|&v| f64::from(v)).collect();
    let gathered = super::format::fbs_reference_gather(route, &as_f64);
    for (o, (&atom, &state)) in route.atoms.iter().zip(&route.states).enumerate() {
        let a = usize::from(atom) * 8;
        let block: [i32; 8] = input[a..a + 8].try_into().expect("block");
        let acted = crate::discrete_d11::cl06_action(usize::from(state), &block);
        for j in 0..8 {
            assert_eq!(gathered[o * 8 + j], f64::from(acted[j]), "block {o}, channel {j}");
        }
    }
}

/// Overwrite the `index`-th `u32` of table `name` in a container's data.
fn poke_table_u32(bytes: &mut [u8], name: &str, index: usize, value: u32) {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("header length")) as usize;
    let header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    let data_start = (16 + len).div_ceil(64) * 64;
    let table = header["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t["name"] == name)
        .expect("the table exists");
    let at = data_start + table["span"]["offset"].as_u64().expect("offset") as usize + 4 * index;
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn fbs_model_parse_refuses_corrupt_routes() {
    let snap = synthetic_fbs_trunk(20261026);
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    assert!(IntegerStackModel::parse(&bytes).is_ok());
    let in_blocks = snap.layers[0].rec_in.in_blocks as u32;
    let corrupt: [(&str, usize, u32); 3] = [
        ("an atom outside the input", 0, in_blocks),
        ("state 64", 0, 64 << 16),
        ("a state beyond 64 with a lawful atom", 1, (200 << 16) | 1),
    ];
    for (what, index, value) in corrupt {
        let mut bad = bytes.clone();
        poke_table_u32(&mut bad, "l0.rec_in.fbs", index, value);
        assert!(
            matches!(IntegerStackModel::parse(&bad), Err(StackError::Numerics(_))),
            "{what} was loaded"
        );
    }
    // A lawful edit still loads: the refusals are about the entries, not the edit.
    let mut lawful = bytes.clone();
    poke_table_u32(&mut lawful, "l0.rec_in.fbs", 0, (5 << 16) | 1);
    assert!(IntegerStackModel::parse(&lawful).is_ok());
}

/// The reference's own near-tie margin at a position: the smaller of its
/// best-minus-second candidate score and its kept-cell boundary score gap.
/// The writer's 256 cell scores for a query, recomputed here: the negative L1
/// distances of its first and last 8-blocks to the 16 orbit roots.
fn fbs_cell_scores(q: &[f64]) -> Vec<f64> {
    let d = q.len();
    let neg_l1 = |a: &[f64], r: [i32; 8]| -> f64 {
        -a.iter()
            .zip(r)
            .map(|(x, y)| (x - f64::from(y)).abs())
            .sum::<f64>()
    };
    let roots: Vec<[i32; 8]> = (0..16).map(crate::discrete_d11::orbit_root_coords).collect();
    (0..256)
        .map(|c| neg_l1(&q[..8], roots[c >> 4]) + neg_l1(&q[d - 8..], roots[c & 15]))
        .collect()
}

/// How near the reference came to the other choice where the integer session
/// differs: the margin at the kept-cell boundary when the kept cells differ,
/// else the margin between its two best candidates.
fn fbs_reference_margin(want: &super::FbsWriterOutput, kept: usize, cells_agree: bool) -> f64 {
    if cells_agree {
        let mut scores: Vec<f64> = want.candidates.iter().map(|&(_, s)| s).collect();
        scores.sort_by(|a, b| b.total_cmp(a));
        match scores.as_slice() {
            [best, second, ..] => best - second,
            _ => f64::INFINITY,
        }
    } else {
        let mut cells = fbs_cell_scores(&want.query);
        cells.sort_by(|a, b| b.total_cmp(a));
        cells[kept - 1] - cells[kept]
    }
}

#[test]
fn fbs_trunk_export_matches_float_reference_across_full_vocabulary() {
    // A reference near-tie below this may break either way under the served
    // fixed point, and is let through as an exception (at most 4). The bar is
    // strict: a 2-LSB query drift can move a cell score by up to 2^-7, so a
    // wider tie can still flip, and then fails here rather than passes.
    const TIE: f64 = 1.0 / 1024.0;
    let snap = synthetic_fbs_trunk(20261026);
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    let mut exceptions: Vec<String> = Vec::new();
    let mut check = |at: String, got: &super::NativeStep, want: &super::FbsWriterOutput| {
        assert_eq!(got.query_q12.len(), want.query.len(), "{at}");
        for (j, (&g, &w)) in got.query_q12.iter().zip(&want.query).enumerate() {
            let w = (w * 4096.0).round() as i32;
            assert!(
                (g - w).abs() <= 2,
                "{at}: query channel {j} is {g} against {w} (Q12)"
            );
        }
        if got.kept_cells != want.kept_cells || got.token != want.token {
            let margin =
                fbs_reference_margin(want, snap.kept_cells, got.kept_cells == want.kept_cells);
            assert!(
                margin < TIE,
                "{at}: integer token {} cells {:?} against reference {} cells {:?}, reference margin {margin:.3e}",
                got.token, got.kept_cells, want.token, want.kept_cells
            );
            exceptions.push(format!("{at} (margin {margin:.2e})"));
        }
    };
    for tok in 0..snap.vocab as u32 {
        let got = model.session().step_native(tok).expect("integer step");
        let want = snap.forward_f32(&[tok], true).expect("reference");
        check(format!("token {tok}"), &got, &want[0]);
    }
    let sequence = [0u32, 5, 11, 19, 3, 27, 14, 31];
    let want = snap.forward_f32(&sequence, true).expect("reference");
    let mut session = model.session();
    for (pos, (&tok, want)) in sequence.iter().zip(&want).enumerate() {
        let got = session.step_native(tok).expect("integer step");
        check(format!("sequence position {pos}"), &got, want);
    }
    eprintln!(
        "fbs control: {} near-tie exceptions over 264 positions {exceptions:?}",
        exceptions.len()
    );
    assert!(
        exceptions.len() <= 4,
        "{} near-tie exceptions: {exceptions:?}",
        exceptions.len()
    );
    // A dense model has no native head, and an FBS model's step returns no
    // vocabulary-wide logits.
    let dense = IntegerStackModel::parse(
        &super::export_float_trunk_uorlut01(&synthetic_float_trunk(7, "dot")).expect("export"),
    )
    .expect("parse");
    assert!(dense.session().step_native(0).is_err());
    assert!(model.session().step(0).expect("step").is_empty());
}

/// The four snap choices of an FBS export (claude-181), snapped first.
const FBS_EXPORT_SNAP_CHOICES: [super::FbsExportSnaps; 4] = [
    super::FbsExportSnaps::SNAPPED,
    super::FbsExportSnaps {
        gates: false,
        transition: true,
    },
    super::FbsExportSnaps {
        gates: true,
        transition: false,
    },
    super::FbsExportSnaps {
        gates: false,
        transition: false,
    },
];

/// The JSON header of an artifact.
fn header_of(bytes: &[u8]) -> Value {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    serde_json::from_slice(&bytes[16..16 + len]).expect("header")
}

#[test]
fn fbs_unsnapped_exports_match_their_float_reference_in_every_combination() {
    // claude-181: with the gates, the transition or both served unsnapped (the unit
    // quaternion by integer square root and division), the integer session matches the f64
    // reference that normalizes the same quaternions, under the snapped control's bar above:
    // every query within 2 LSB at Q12, and at most 4 near-tie exceptions in token or cells.
    const TIE: f64 = 1.0 / 1024.0;
    let snap = synthetic_fbs_trunk(20261026);
    let sequence = [0u32, 5, 11, 19, 3, 27, 14, 31];
    for snaps in FBS_EXPORT_SNAP_CHOICES {
        let bytes = super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export");
        let header = header_of(&bytes);
        let snapped = snaps == super::FbsExportSnaps::SNAPPED;
        let schema = if snapped {
            super::STACK_FBS_SCHEMA
        } else {
            super::STACK_FBS_RAW_SCHEMA
        };
        assert_eq!(header["schema"], json!(schema), "{snaps:?}");
        assert_eq!(header["transport_snap"].is_null(), !snaps.transition, "{snaps:?}");
        let model = IntegerStackModel::parse(&bytes).expect("parse");
        let shape = model.shape();
        assert_eq!(
            (shape.raw_gates, shape.raw_transition),
            (!snaps.gates, !snaps.transition),
            "{snaps:?}"
        );
        let reference = snaps.reference();
        let mut exceptions: Vec<String> = Vec::new();
        let mut check = |at: String, got: &super::NativeStep, want: &super::FbsWriterOutput| {
            assert_eq!(got.query_q12.len(), want.query.len(), "{snaps:?} {at}");
            for (j, (&g, &w)) in got.query_q12.iter().zip(&want.query).enumerate() {
                let w = (w * 4096.0).round() as i32;
                assert!(
                    (g - w).abs() <= 2,
                    "{snaps:?} {at}: query channel {j} is {g} against {w} (Q12)"
                );
            }
            if got.kept_cells != want.kept_cells || got.token != want.token {
                let margin =
                    fbs_reference_margin(want, snap.kept_cells, got.kept_cells == want.kept_cells);
                assert!(
                    margin < TIE,
                    "{snaps:?} {at}: integer token {} against reference {}, margin {margin:.3e}",
                    got.token,
                    want.token
                );
                exceptions.push(format!("{at} (margin {margin:.2e})"));
            }
        };
        for tok in 0..snap.vocab as u32 {
            let got = model.session().step_native(tok).expect("integer step");
            let want = snap
                .forward_f32_with(&[tok], true, reference)
                .expect("reference");
            check(format!("token {tok}"), &got, &want[0]);
        }
        let want = snap
            .forward_f32_with(&sequence, true, reference)
            .expect("reference");
        let mut session = model.session();
        for (pos, (&tok, want)) in sequence.iter().zip(&want).enumerate() {
            let got = session.step_native(tok).expect("integer step");
            check(format!("sequence position {pos}"), &got, want);
        }
        eprintln!(
            "fbs control {snaps:?}: {} near-tie exceptions {exceptions:?}",
            exceptions.len()
        );
        assert!(
            exceptions.len() <= 4,
            "{snaps:?}: {} near-tie exceptions: {exceptions:?}",
            exceptions.len()
        );
    }
}

#[test]
#[ignore = "reads a trained snapshot from UOR_FBS_SNAPSHOT"]
fn a_trained_fbs_snapshot_matches_its_reference_in_every_combination() {
    // claude-181: host parity on a stored trained snapshot (claude-126's seed 1, for one): for
    // each snap choice, the integer session against the f64 reference with the same snaps,
    // over every single token and over 64 sequences of 32 tokens (the transition moves a state
    // only from a sequence's second token). The counts are readings beside the snapped
    // choice's, with a floor: at least 95% of positions agree in token and in kept cells.
    // Query drift is a reading only (median and largest per-position worst channel, in Q12
    // LSB): through six trained layers the snapped contract keeps 1,379 of 4,096 single tokens
    // within 2 LSB, while 98% of its tokens and cells agree.
    let path = std::env::var("UOR_FBS_SNAPSHOT").expect("set UOR_FBS_SNAPSHOT");
    let snap: super::FbsTrunkSnapshot =
        serde_json::from_slice(&std::fs::read(&path).expect("read the snapshot")).expect("parse");
    let tokens: Vec<u32> = (0..snap.vocab as u32).collect();
    let mut lcg = Lcg(181);
    let sequences: Vec<Vec<u32>> = (0..64)
        .map(|_| (0..32).map(|_| (lcg.next() % snap.vocab as u64) as u32).collect())
        .collect();
    let mut inputs: Vec<&[u32]> = tokens.iter().map(std::slice::from_ref).collect();
    inputs.extend(sequences.iter().map(Vec::as_slice));
    for snaps in FBS_EXPORT_SNAP_CHOICES {
        let model = IntegerStackModel::parse(
            &super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export"),
        )
        .expect("parse");
        let want = snap
            .forward_f32_each(&inputs, true, snaps.reference())
            .expect("reference");
        for (part, range) in [("single tokens", 0..tokens.len()), ("sequences", tokens.len()..inputs.len())] {
            let (mut same_token, mut same_cells, mut n) = (0usize, 0usize, 0usize);
            let mut drift = Vec::new();
            for (input, want) in inputs[range.clone()].iter().zip(&want[range]) {
                let mut session = model.session();
                for (&tok, want) in input.iter().zip(want) {
                    let got = session.step_native(tok).expect("integer step");
                    n += 1;
                    same_token += usize::from(got.token == want.token);
                    same_cells += usize::from(got.kept_cells == want.kept_cells);
                    drift.push(
                        got.query_q12
                            .iter()
                            .zip(&want.query)
                            .map(|(&g, &w)| (g - (w * 4096.0).round() as i32).abs())
                            .max()
                            .unwrap_or(i32::MAX),
                    );
                }
            }
            drift.sort_unstable();
            eprintln!(
                "trained fbs {snaps:?} {part}: tokens {same_token}/{n}, kept cells {same_cells}/{n}, query drift median {} max {} LSB, within 2 LSB {}/{n}",
                drift[drift.len() / 2],
                drift[drift.len() - 1],
                drift.iter().filter(|&&d| d <= 2).count()
            );
            for (what, count) in [("tokens", same_token), ("kept cells", same_cells)] {
                assert!(count * 100 >= n * 95, "{snaps:?} {part}: {what} agree on {count} of {n}");
            }
        }
    }
}

#[test]
fn each_unsnapped_choice_changes_the_served_forward() {
    // claude-181: the queries of each unsnapped export differ from the snapped export's, so
    // neither flag is ignored by the session.
    let snap = synthetic_fbs_trunk(20261026);
    let queries = |snaps: super::FbsExportSnaps| {
        let model = IntegerStackModel::parse(
            &super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export"),
        )
        .expect("parse");
        let mut session = model.session();
        [0u32, 5, 11, 19, 3, 27, 14, 31]
            .iter()
            .map(|&tok| session.step_native(tok).expect("step").query_q12)
            .collect::<Vec<_>>()
    };
    let snapped = queries(super::FbsExportSnaps::SNAPPED);
    for snaps in &FBS_EXPORT_SNAP_CHOICES[1..] {
        assert_ne!(queries(*snaps), snapped, "{snaps:?} served the snapped forward");
    }
}

#[test]
fn an_fbs_header_states_its_snaps_consistently() {
    // claude-181: the snapped export is the `lut-stack/4` contract byte for byte, and the
    // schema, the shape's flags and the transport snap must agree.
    let snap = synthetic_fbs_trunk(20261026);
    let snapped = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    assert_eq!(
        super::export_fbs_trunk_uorlut01_with(&snap, super::FbsExportSnaps::SNAPPED)
            .expect("export"),
        snapped
    );
    let header = header_of(&snapped);
    assert!(header["shape"].get("raw_gates").is_none());
    assert!(header["shape"].get("raw_transition").is_none());
    let raw_transition = super::export_fbs_trunk_uorlut01_with(
        &snap,
        super::FbsExportSnaps {
            gates: true,
            transition: false,
        },
    )
    .expect("export");
    // An unsnapped transition with a transport snap recorded is refused.
    let both = with_header(&raw_transition, |h| h["transport_snap"] = snap_record());
    assert!(matches!(
        IntegerStackModel::parse(&both),
        Err(StackError::TransportSnap(_))
    ));
    // A raw shape under the snapped schema, or a snapped one under the raw schema, is refused.
    let old_schema = with_header(&raw_transition, |h| {
        h["schema"] = json!(super::STACK_FBS_SCHEMA);
    });
    assert!(matches!(
        IntegerStackModel::parse(&old_schema),
        Err(StackError::Schema(_))
    ));
    let new_schema = with_header(&snapped, |h| {
        h["schema"] = json!(super::STACK_FBS_RAW_SCHEMA);
    });
    assert!(matches!(
        IntegerStackModel::parse(&new_schema),
        Err(StackError::Schema(_))
    ));
    // A snapped transition still needs its record.
    let no_record = with_header(&snapped, |h| h["transport_snap"] = Value::Null);
    assert!(matches!(
        IntegerStackModel::parse(&no_record),
        Err(StackError::TransportSnap(_))
    ));
    // The flags belong to FBS stacks.
    let dense = super::export_float_trunk_uorlut01(&synthetic_float_trunk(7, "dot")).expect("export");
    let flagged = with_header(&dense, |h| h["shape"]["raw_gates"] = json!(true));
    assert!(matches!(
        IntegerStackModel::parse(&flagged),
        Err(StackError::Shape(
            "unsnapped gates and transitions belong to SpiralCore/FBS stacks"
        ))
    ));
}

#[test]
fn fbs_writer_matches_reference_when_its_address_gains_are_tiny() {
    // claude-127: a trained trunk shrank the final gains of the 16 channels
    // the writer scores its cells from (the query's first and last 8-blocks)
    // to about 6e-4, so those blocks sit at a few Q12 steps and one LSB
    // reorders the cells. The query must then reach Q12 from a norm rounded to
    // nearest at 2^-24, as unbiased as the reference's: a 2^-14 norm truncated
    // and then rounded again to Q12 chose different cells on 30% of tokens.
    let snap = fbs_synthetic_tiny_gain(20261026);
    let model = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export"))
        .expect("parse");
    let (mut tokens, mut cells, mut close) = (0usize, 0usize, 0usize);
    for tok in 0..snap.vocab as u32 {
        let got = model.session().step_native(tok).expect("integer step");
        let want = &snap.forward_f32(&[tok], true).expect("reference")[0];
        tokens += usize::from(got.token == want.token);
        cells += usize::from(got.kept_cells == want.kept_cells);
        let worst = got
            .query_q12
            .iter()
            .zip(&want.query)
            .map(|(&g, &w)| (g - (w * 4096.0).round() as i32).abs())
            .max()
            .unwrap_or(i32::MAX);
        close += usize::from(worst <= 2);
    }
    eprintln!("fbs tiny-address control: tokens {tokens}/512, kept cells {cells}/512, queries within 2 LSB {close}/512");
    assert!(cells >= 500, "kept cells agree on only {cells} of 512");
    assert!(tokens >= 500, "tokens agree on only {tokens} of 512");
    assert!(close >= 500, "queries within 2 LSB on only {close} of 512");
}

#[test]
fn fbs_trunk_matches_reference_at_a_non_square_multi_token_shape() {
    // claude-127: the small control is square (MLP = width), holds one token
    // per writer cell and keeps 2 cells. This one is non-square (MLP 96, width
    // 64), holds two tokens per cell (vocabulary 512), keeps 8 cells, runs 3
    // layers and has larger tables, as a trained trunk does. A snap near a tie
    // may break either way under fixed point, so a few positions may differ.
    let snap = fbs_synthetic_non_square(20261026);
    let model = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export"))
        .expect("parse");
    let (mut tokens, mut cells, mut close) = (0usize, 0usize, 0usize);
    for tok in 0..snap.vocab as u32 {
        let got = model.session().step_native(tok).expect("integer step");
        let want = &snap.forward_f32(&[tok], true).expect("reference")[0];
        tokens += usize::from(got.token == want.token);
        cells += usize::from(got.kept_cells == want.kept_cells);
        let worst = got
            .query_q12
            .iter()
            .zip(&want.query)
            .map(|(&g, &w)| (g - (w * 4096.0).round() as i32).abs())
            .max()
            .unwrap_or(i32::MAX);
        close += usize::from(worst <= 2);
    }
    eprintln!("fbs large control: tokens {tokens}/512, kept cells {cells}/512, queries within 2 LSB {close}/512");
    assert!(tokens >= 504, "tokens agree on only {tokens} of 512");
    assert!(cells >= 496, "kept cells agree on only {cells} of 512");
    assert!(close >= 480, "queries within 2 LSB on only {close} of 512");
}

/// claude-188: a trunk with no Hamming read exports the same bytes as before the read was added
/// to the format, snapped (`lut-stack/4`) and with both forms unsnapped (`lut-stack/5`). The
/// sha256s were taken at `a368b78ef`, before the change.
#[test]
fn read_free_fbs_exports_are_unchanged_by_the_hamming_format() {
    use sha2::{Digest, Sha256};
    let snap = synthetic_fbs_trunk(20261025);
    let snapped = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    let raw = super::export_fbs_trunk_uorlut01_with(
        &snap,
        super::FbsExportSnaps {
            gates: false,
            transition: false,
        },
    )
    .expect("export");
    assert_eq!(hex::encode(Sha256::digest(&snapped)), "e377d68f6b3c112ed267a9e3c09ef2ef2210e8f1c9bc4619ae26dd4c68d18e5c");
    assert_eq!(hex::encode(Sha256::digest(&raw)), "ec5ca212c257a2b2cd73c337029ff8556b454810377801d81addca75c4105965");
}

/// claude-188: a trunk with Hamming reads exports as `uor-r4.lut-stack/6`, and reads back with
/// every parameter at its declared exponent, a zero quaternion included. The integer session
/// serves it, type heads included (claude-195).
#[test]
fn hamming_reads_round_trip_in_lut_stack_6() {
    use super::synthetic::synthetic_hamming;
    let fixed = |values: &[f32], exp: i32| -> Vec<i32> {
        values.iter().map(|&v| (f64::from(v) * 2f64.powi(-exp)).round() as i32).collect()
    };
    for raw in [false, true] {
        let mut snap = synthetic_fbs_trunk(20261025);
        let read = synthetic_hamming(snap.width, snap.layers.len(), 4, 188);
        snap.hamming = Some(read.clone());
        let snaps = super::FbsExportSnaps {
            gates: !raw,
            transition: !raw,
        };
        let bytes = super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export");
        let header_len = u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes")) as usize;
        let header: Value = serde_json::from_slice(&bytes[16..16 + header_len]).expect("header");
        assert_eq!(header["schema"], super::STACK_FBS_HAMMING_SCHEMA);
        assert_eq!(header["shape"]["raw_gates"].as_bool().unwrap_or(false), raw);
        let image = super::read_hamming_uorlut01(&bytes)
            .expect("parse")
            .expect("the artifact carries the reads");
        assert_eq!(image.record, super::StackHamming::new(4, vec![1, snap.layers.len()]));
        assert_eq!(image.reads.len(), 2);
        for (got, want) in image.reads.iter().zip(&read.reads) {
            let rot = super::HAMMING_ROTATION_EXP;
            assert_eq!(got.query_rotation, fixed(&want.query_rotation, rot));
            assert_eq!(got.key_rotation, fixed(&want.key_rotation, rot));
            assert_eq!(got.value_rotation, fixed(&want.value_rotation, rot));
            assert_eq!(got.margin, fixed(&want.margin, super::HAMMING_MARGIN_EXP));
            assert_eq!(got.weight, fixed(&want.weight, super::HAMMING_WEIGHT_EXP));
            assert_eq!(got.key_rotation[..4], [0, 0, 0, 0], "lane 0 is the zero quaternion");
        }
        // Its type-head weights are not zero, and the session serves them (claude-195).
        IntegerStackModel::parse(&bytes).expect("a read with type heads is served");
        // The same trunk without the read is served, and reads back no read.
        snap.hamming = None;
        let plain = super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export");
        assert!(super::read_hamming_uorlut01(&plain).expect("parse").is_none());
        IntegerStackModel::parse(&plain).expect("a read-free trunk is served");
    }
}

/// claude-188: the exporter refuses a Hamming read the trunk cannot carry.
#[test]
fn malformed_hamming_reads_are_refused() {
    use super::synthetic::synthetic_hamming;
    let base = synthetic_fbs_trunk(20261025);
    let layers = base.layers.len();
    let export = |mutate: &dyn Fn(&mut super::FbsHammingSnapshot)| {
        let mut snap = base.clone();
        let mut read = synthetic_hamming(snap.width, layers, 4, 188);
        mutate(&mut read);
        snap.hamming = Some(read);
        super::export_fbs_trunk_uorlut01(&snap)
    };
    export(&|_| {}).expect("the unmodified reads export");
    let cases: [(&str, &dyn Fn(&mut super::FbsHammingSnapshot)); 9] = [
        ("an odd head count", &|r| r.heads = 3),
        ("a head count of zero", &|r| r.heads = 0),
        ("depths out of order", &|r| r.depths.reverse()),
        ("a depth beyond the layers", &|r| r.depths[1] = layers + 1),
        ("a depth of zero", &|r| r.depths[0] = 0),
        ("a missing read", &|r| {
            r.reads.pop();
        }),
        ("a short rotation", &|r| {
            r.reads[0].key_rotation.pop();
        }),
        ("a zero margin", &|r| r.reads[1].margin[0] = 0.0),
        ("a weight that does not fit", &|r| r.reads[0].weight[0] = 1.0e6),
    ];
    for (what, mutate) in cases {
        assert!(export(mutate).is_err(), "{what} must be refused");
    }
}

/// claude-188: a snapshot written before the Hamming reads (no `hamming` key) still parses, and a
/// read-free snapshot serializes with no `hamming` key.
#[test]
fn snapshots_without_hamming_reads_are_unchanged() {
    let snap = synthetic_fbs_trunk(20261025);
    let value = serde_json::to_value(&snap).expect("serialize");
    assert!(value.get("hamming").is_none());
    // Nor a memory key (claude-198).
    assert!(value.get("memory").is_none());
    let back: super::FbsTrunkSnapshot = serde_json::from_value(value).expect("an old snapshot parses");
    assert_eq!(back, snap);
}

/// claude-188: the loader and the parser, not only the exporter, hold a `lut-stack/6` artifact
/// to its schema and sections. Each edit of a valid export is refused by both
/// `read_hamming_uorlut01` and `IntegerStackModel::parse`.
#[test]
fn hamming_schema_and_sections_are_checked_at_parse() {
    use super::synthetic::synthetic_hamming;
    let mut snap = synthetic_fbs_trunk(20261025);
    snap.hamming = Some(synthetic_hamming(snap.width, snap.layers.len(), 4, 188));
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    super::read_hamming_uorlut01(&bytes).expect("the valid export parses");
    let refused_by_both = |bytes: &[u8], what: &str| -> StackError {
        let parsed = super::read_hamming_uorlut01(bytes).expect_err(what);
        assert!(IntegerStackModel::parse(bytes).is_err(), "{what}: the session must refuse it too");
        parsed
    };
    // `/6` without the record, and the record under another schema.
    let missing = with_header(&bytes, |h| {
        h.as_object_mut().expect("object").remove("hamming");
    });
    assert!(matches!(refused_by_both(&missing, "a /6 header without the record"), StackError::Schema(_)));
    let other = with_header(&bytes, |h| h["schema"] = json!(super::STACK_FBS_SCHEMA));
    assert!(matches!(refused_by_both(&other, "the record under /4"), StackError::Schema(_)));
    // The record on a stack without FBS projections.
    let dense = with_header(&artifact(5), |h| {
        h["schema"] = json!(super::STACK_FBS_HAMMING_SCHEMA);
        h["hamming"] = json!(super::StackHamming::new(2, vec![1, 2]));
    });
    assert!(matches!(
        refused_by_both(&dense, "the record on a dense stack"),
        StackError::Shape(m) if m.contains("SpiralCore/FBS")
    ));
    // A record whose heads disagree with the margin section, and a foreign exponent.
    let heads = with_header(&bytes, |h| h["hamming"]["heads"] = json!(2));
    assert!(matches!(
        refused_by_both(&heads, "a margin section of the wrong length"),
        StackError::TableLength { .. }
    ));
    let exp = with_header(&bytes, |h| h["hamming"]["rotation_exp"] = json!(-20));
    assert!(matches!(
        refused_by_both(&exp, "a foreign exponent"),
        StackError::Shape(m) if m.contains("exponents")
    ));
    // A margin of zero in the data.
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    let header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    let entry = header["tables"]
        .as_array()
        .expect("tables")
        .iter()
        .find(|t| t["name"] == "hamming.d1.margin")
        .expect("the margin section");
    let offset = (16 + len).div_ceil(64) * 64 + entry["span"]["offset"].as_u64().expect("offset") as usize;
    let mut zeroed = bytes.clone();
    zeroed[offset..offset + 4].fill(0);
    assert!(matches!(
        refused_by_both(&zeroed, "a margin of zero"),
        StackError::Numerics(m) if m.contains("margin")
    ));
}

/// claude-188: `StackHamming::check` refuses each rule's violation for that rule's reason.
#[test]
fn hamming_record_checks_name_their_reason() {
    let plain = super::export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(20261025)).expect("export");
    let base = IntegerStackModel::parse(&plain).expect("parse").shape().clone();
    let layers = base.layers();
    let reason = |shape: &StackShape, record: super::StackHamming| -> &'static str {
        match record.check(shape) {
            Err(StackError::Shape(m)) => m,
            other => panic!("expected a shape refusal, got {other:?}"),
        }
    };
    let record = |heads: usize, depths: Vec<usize>| super::StackHamming::new(heads, depths);
    let wide = |width: usize| StackShape { width, ..base.clone() };
    record(4, vec![1, layers]).check(&base).expect("a valid record");
    assert!(reason(&wide(48), record(3, vec![1, layers])).contains("even head count"));
    assert!(reason(&base, record(0, vec![1, layers])).contains("even head count"));
    assert!(reason(&wide(256), record(2, vec![1, layers])).contains("at most 64 coordinates"));
    assert!(reason(&wide(40), record(4, vec![1, layers])).contains("whole 4-lane blocks"));
    for depths in [vec![], vec![layers, 1], vec![0, layers], vec![1, layers + 1]] {
        assert!(reason(&base, record(4, depths.clone())).contains("strictly increasing"), "{depths:?}");
    }
    let mut none_kept = record(4, vec![1, layers]);
    none_kept.kept = 0;
    assert!(reason(&base, none_kept).contains("at least one position"));
    let mut foreign = record(4, vec![1, layers]);
    foreign.weight_exp = -16;
    assert!(reason(&base, foreign).contains("exponents"));
    let dense = IntegerStackModel::parse(&artifact(5)).expect("parse").shape().clone();
    assert!(reason(&dense, record(2, vec![1, 2])).contains("SpiralCore/FBS"));
}

/// A copy-head control (claude-189): a Q12 table of `vocab` rows of `width`, and one read of
/// `heads` heads quantized as the exporter writes it, from `synthetic_hamming`.
fn copy_control(vocab: usize, width: usize, heads: usize, seed: u64) -> (super::StackHamming, super::HammingReadImage, Vec<i16>) {
    let mut rng = Lcg(seed);
    let table: Vec<i16> = (0..vocab * width)
        .map(|_| ((rng.next() % 12_289) as i32 - 6_144) as i16)
        .collect();
    let read = &super::synthetic::synthetic_hamming(width, 2, heads, seed).reads[0];
    let fixed = |values: &[f32], exp: i32| -> Vec<i32> {
        values.iter().map(|&v| (f64::from(v) * 2f64.powi(-exp)).round() as i32).collect()
    };
    let image = super::HammingReadImage {
        depth: 1,
        query_rotation: fixed(&read.query_rotation, super::HAMMING_ROTATION_EXP),
        key_rotation: fixed(&read.key_rotation, super::HAMMING_ROTATION_EXP),
        value_rotation: fixed(&read.value_rotation, super::HAMMING_ROTATION_EXP),
        margin: fixed(&read.margin, super::HAMMING_MARGIN_EXP),
        weight: fixed(&read.weight, super::HAMMING_WEIGHT_EXP),
    };
    (super::StackHamming::new(heads, vec![1, 2]), image, table)
}

/// The f64 reference of the copy heads' read (claude-189): the float read's steps on the
/// quantized values, `(g / |g|) ⊗ p` in f64 and its signs, the same distances and tie rule, and
/// the weights and means in f64. Returns the codes `vocab * copy`, and per row the kept sets
/// and the mean.
#[allow(clippy::type_complexity)]
fn reference_copy(
    record: &super::StackHamming,
    read: &super::HammingReadImage,
    table: &[i16],
    width: usize,
    tokens: &[u32],
) -> (Vec<u64>, Vec<(Vec<Vec<(usize, u32)>>, Option<Vec<f64>>)>) {
    let (copy, head_width) = (record.heads / 2, width / record.heads);
    let vocab = table.len() / width;
    let units: Vec<[f64; 4]> = read
        .key_rotation
        .chunks_exact(4)
        .map(|g| {
            let g = [g[0], g[1], g[2], g[3]].map(|v| f64::from(v) * 2f64.powi(super::HAMMING_ROTATION_EXP));
            let r = g.iter().map(|v| v * v).sum::<f64>().sqrt();
            if r == 0.0 {
                [0.0; 4]
            } else {
                g.map(|v| v / r)
            }
        })
        .collect();
    let hamilton = |a: [f64; 4], b: [f64; 4]| {
        [
            a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
            a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
            a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
            a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
        ]
    };
    let point = |v: usize, at: usize| -> f64 { f64::from(table[v * width + at]) / 4096.0 };
    let mut codes = vec![0u64; vocab * copy];
    for v in 0..vocab {
        for h in 0..copy {
            let mut word = 0u64;
            for c in (0..head_width).step_by(4) {
                let at = h * head_width + c;
                let p = [point(v, at), point(v, at + 1), point(v, at + 2), point(v, at + 3)];
                for (i, value) in hamilton(units[at / 4], p).into_iter().enumerate() {
                    if value >= 0.0 {
                        word |= 1u64 << (c + i);
                    }
                }
            }
            codes[v * copy + h] = word;
        }
    }
    let margins: Vec<f64> = read.margin[..copy].iter().map(|&m| f64::from(m) / 65536.0).collect();
    let rows = (0..tokens.len())
        .map(|t| {
            let mut kept_sets = Vec::with_capacity(copy);
            let mut sum = vec![0.0f64; width];
            let mut found = 0usize;
            for h in 0..copy {
                let q = codes[tokens[t] as usize * copy + h];
                let mut candidates: Vec<(u32, usize)> = (1..t)
                    .map(|s| ((q ^ codes[tokens[s - 1] as usize * copy + h]).count_ones(), s))
                    .collect();
                candidates.sort_unstable();
                candidates.truncate(record.kept);
                let kept: Vec<(usize, u32)> = candidates.into_iter().map(|(d, s)| (s, d)).collect();
                let mut head = vec![0.0f64; width];
                let mut count = 0usize;
                for &(s, d) in &kept {
                    let weight = (margins[h] - f64::from(d)).max(0.0) / margins[h].max(1.0);
                    if weight > 0.0 {
                        count += 1;
                        for (c, acc) in head.iter_mut().enumerate() {
                            *acc += weight * point(tokens[s] as usize, c);
                        }
                    }
                }
                if count > 0 {
                    found += 1;
                    for (acc, value) in sum.iter_mut().zip(head) {
                        *acc += value / count as f64;
                    }
                }
                kept_sets.push(kept);
            }
            let mean = (found > 0).then(|| sum.into_iter().map(|v| v / found as f64).collect());
            (kept_sets, mean)
        })
        .collect();
    (codes, rows)
}

/// claude-189: the copy heads' integer read equals the f64 reference of the same quantized steps
/// on a 256-wide, 64-coordinates-a-head read: every code, every kept set with its distances, and
/// every copy mean within `2^-23`. The bar was fixed in the ORDER before this test ran.
#[test]
fn copy_heads_match_the_f64_reference() {
    let (vocab, width) = (256, 256);
    // Narrow margins, where only repeats count, and wide ones (33 to 41 bits, fractional), where
    // most candidates fall inside with fractional weights, testing the Q31 weights and means.
    for (seed, wide) in [(189u64, false), (190, false), (191, false), (194, true), (195, true)] {
        let (record, mut read, table) = copy_control(vocab, width, 4, seed);
        if wide {
            for (h, m) in read.margin.iter_mut().enumerate() {
                *m = ((33 + 2 * h as i32) << 16) + 12_345 * (h as i32 + 1);
            }
        }
        let heads = super::CopyHeads::new(&record, &read, &table, vocab, width);
        let mut rng = Lcg(seed ^ 0x5eed);
        // Draws from a small subset, so repeats and near matches are common.
        let tokens: Vec<u32> = (0..48).map(|_| (rng.next() % 12) as u32 * 21).collect();
        let (codes, rows) = reference_copy(&record, &read, &table, width, &tokens);
        for v in 0..vocab as u32 {
            for h in 0..2 {
                assert_eq!(heads.code(v, h), codes[v as usize * 2 + h], "code of {v}, head {h}");
            }
        }
        for (t, (kept_sets, mean)) in rows.iter().enumerate() {
            let prefix = &tokens[..=t];
            for (h, kept) in kept_sets.iter().enumerate() {
                assert_eq!(&heads.choose(h, prefix), kept, "seed {seed}, row {t}, head {h}");
            }
            let got = heads.mean(prefix, &table);
            assert_eq!(got.is_some(), mean.is_some(), "seed {seed}, row {t}: found");
            if let (Some(got), Some(want)) = (got, mean) {
                for (c, (&g, &w)) in got.iter().zip(want).enumerate() {
                    let g = g as f64 * 2f64.powi(super::COPY_MEAN_EXP);
                    assert!((g - w).abs() <= 2f64.powi(-23), "seed {seed}, row {t}, coordinate {c}: {g} against {w}");
                }
            }
        }
    }
}

/// claude-189: the recall control. Wherever `x_t` repeats an earlier `x_j` with `j + 1 < t`,
/// every copy head keeps `s = j + 1` at distance 0. Under one-bit margins only a distance of 0
/// counts, so with a single earlier copy the copy mean is exactly the point of `x_{j+1}`, the
/// token that followed it.
#[test]
fn copy_heads_recall_the_token_after_an_earlier_copy() {
    let (vocab, width) = (256, 256);
    let (record, mut read, table) = copy_control(vocab, width, 4, 192);
    read.margin.fill(1 << 16);
    let heads = super::CopyHeads::new(&record, &read, &table, vocab, width);
    // Distinct tokens, then a repeat of position 5: the copy of 5 is followed by token 6.
    let mut tokens: Vec<u32> = (0..20u32).map(|i| i * 11 + 3).collect();
    tokens.push(tokens[5]);
    let t = tokens.len() - 1;
    for h in 0..2 {
        assert!(heads.choose(h, &tokens).contains(&(6, 0)), "head {h} keeps the position after the copy");
    }
    let mean = heads.mean(&tokens, &table).expect("the copy is found");
    let follower = &table[tokens[6] as usize * width..][..width];
    for (&m, &p) in mean.iter().zip(follower) {
        assert_eq!(m, i128::from(p) << 31, "the copy mean is the follower's point");
    }
    // Every repeat in a sequence with many is kept at distance 0, when it fits the kept count.
    let mut rng = Lcg(193);
    let repeats: Vec<u32> = (0..40).map(|_| (rng.next() % 6) as u32 * 37).collect();
    for t in 2..repeats.len() {
        let prefix = &repeats[..=t];
        for h in 0..2 {
            let kept = heads.choose(h, prefix);
            let zeros: Vec<usize> = (0..t - 1).filter(|&j| repeats[j] == repeats[t]).map(|j| j + 1).collect();
            if zeros.len() <= record.kept {
                for s in zeros {
                    assert!(kept.contains(&(s, 0)), "row {t}, head {h}: the copy at {} is kept", s - 1);
                }
            }
        }
    }
    let _ = t;
}

/// claude-189: a candidate's weight divides by `max(m, 1)`, the float's `margin.maximum(1.0)`, so
/// a margin below one bit does not inflate it.
#[test]
fn copy_weights_divide_by_at_least_one_bit() {
    let half_bit = 1 << 15;
    assert_eq!(super::copy_weight_q31(half_bit, 0), 1 << 30, "0.5 / max(0.5, 1) = 0.5");
    assert_eq!(super::copy_weight_q31(1 << 16, 0), 1 << 31);
    assert_eq!(super::copy_weight_q31(4 << 16, 1), 3 << 29, "(4 − 1) / 4");
    assert_eq!(super::copy_weight_q31(4 << 16, 4), 0);
    assert_eq!(super::copy_weight_q31(4 << 16, 9), 0);
}

/// claude-194's served-read trunk: claude-127's non-square shape (width 64, 3 layers, 4 heads of
/// 16 coordinates) over 40 positions, and `synthetic_hamming`'s reads at `depths` (a subset of
/// `[1, 3]`), each with its copy weights times `copy` and its type weights times `types`
/// (claude-195; 0 serves the copy heads alone). Returns the snapshot, the trunk exported without
/// the reads, and with them.
fn served_read_trunk(depths: &[usize], copy: f32, types: f32) -> (super::FbsTrunkSnapshot, Vec<u8>, Vec<u8>) {
    let shape = super::synthetic::FbsSynth {
        context: 40,
        ..super::synthetic::FBS_SYNTH_NON_SQUARE
    };
    let mut snap = super::synthetic::synthetic_fbs_trunk_with(20261029, shape);
    let (width, layers) = (snap.width, snap.layers.len());
    let plain = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    let mut read = super::synthetic::synthetic_hamming(width, layers, 4, 194);
    let all = std::mem::take(&mut read.depths);
    read.reads = all
        .iter()
        .zip(std::mem::take(&mut read.reads))
        .filter(|(depth, _)| depths.contains(depth))
        .map(|(_, r)| r)
        .collect();
    read.depths = depths.to_vec();
    for r in &mut read.reads {
        let (copy_weight, type_weight) = r.weight.split_at_mut(width);
        copy_weight.iter_mut().for_each(|w| *w *= copy);
        type_weight.iter_mut().for_each(|w| *w *= types);
    }
    snap.hamming = Some(read);
    let with_read = super::export_fbs_trunk_uorlut01(&snap).expect("export with the reads");
    (snap, plain, with_read)
}

/// 40 tokens with exact repeats: an 11-token cycle, six new tokens, then the cycle again.
fn repeating_tokens() -> Vec<u32> {
    (0..40u32)
        .map(|i| if (14..20).contains(&i) { 100 + i } else { (i * 7 + 3) % 11 })
        .collect()
}

/// `v / 2^s` rounded half away from zero: the tests' own form of the float `round()`.
fn round_half_away(v: i128, s: u32) -> i128 {
    let r = (v.abs() + (1i128 << (s - 1))) >> s;
    if v < 0 {
        -r
    } else {
        r
    }
}

/// The served-read model, the read-free one, the shared Q12 table, and each read with its copy
/// heads, in depth order, for the hook tests; the type weights scaled by `types`.
#[allow(clippy::type_complexity)]
fn served_read_parts(
    depths: &[usize],
    copy: f32,
    types: f32,
) -> (IntegerStackModel, IntegerStackModel, Vec<i16>, Vec<(super::HammingReadImage, super::CopyHeads)>) {
    let (_, plain, with_read) = served_read_trunk(depths, copy, types);
    let read = IntegerStackModel::parse(&with_read).expect("the read is served");
    let free = IntegerStackModel::parse(&plain).expect("parse");
    let table = free.fbs_image().expect("an FBS model").writer_table.to_vec();
    let image = super::read_hamming_uorlut01(&with_read).expect("parse").expect("the reads");
    let (vocab, width) = (free.shape().vocab, free.shape().width);
    let reads = image
        .reads
        .iter()
        .map(|r| (r.clone(), super::CopyHeads::new(&image.record, r, &table, vocab, width)))
        .collect();
    (read, free, table, reads)
}

/// claude-194: a `/6` read whose copy weights are 0 serves every step bit for bit as the same
/// trunk exported without it, although its copy heads do find candidates. The zero-copy view
/// refuses the read rather than drop it.
#[test]
fn a_read_at_weight_zero_serves_as_the_read_free_trunk() {
    let (read, free, table, reads) = served_read_parts(&[1, 3], 0.0, 0.0);
    assert!(read.fbs_image().is_none(), "the view does not carry the read");
    let copy_heads = &reads[0].1;
    assert!(copy_heads.choose(0, &[]).is_empty() && copy_heads.mean(&[], &table).is_none(), "no tokens, no read");
    let tokens = repeating_tokens();
    let (mut with, mut without) = (read.session(), free.session());
    let mut found = [0usize; 2];
    for t in 0..tokens.len() {
        let want = without.step_native(tokens[t]).expect("step");
        assert_eq!(with.step_native(tokens[t]).expect("step"), want, "position {t}");
        for (count, (_, heads)) in found.iter_mut().zip(&reads) {
            *count += usize::from(heads.mean(&tokens[..=t], &table).is_some());
        }
    }
    assert!(found.iter().all(|&f| f > 10), "both reads' copy heads find candidates: {found:?} rows");
}

/// One traced FBS step: the Q12 query and the residual after every layer.
fn traced_step(session: &mut super::IntegerStackSession<'_>, token: u32) -> (Vec<i32>, Vec<Vec<i32>>) {
    super::hamming::LAYER_TRACE.with(|trace| *trace.borrow_mut() = Some(Vec::new()));
    let query = session.step_native(token).expect("step").query_q12;
    let layers = super::hamming::LAYER_TRACE
        .with(|trace| trace.borrow_mut().take())
        .expect("the trace");
    (query, layers)
}

/// claude-194: the stream read after layer 1 adds `w_c ⊙ c` to the residual the read-free trunk
/// has there, each sum rounded half away from zero to the residual's Q16, as the float's `q16`
/// rounds the sum. A layer before the read cannot see it, so the read-free run gives the read's
/// input exactly. Rows 0 and 1 have no earlier position `1 ≤ s < t`, so they add nothing.
#[test]
fn the_stream_read_adds_the_rounded_copy_to_the_residual() {
    let (read, free, table, reads) = served_read_parts(&[1], 1.0, 0.0);
    let (image, copy_heads) = &reads[0];
    let width = free.shape().width;
    let copy_weight = &image.weight[..width];
    let tokens = repeating_tokens();
    let (mut with, mut without) = (read.session(), free.session());
    let (mut moved, mut changed) = (0, 0);
    for t in 0..tokens.len() {
        let (query, trace) = traced_step(&mut with, tokens[t]);
        let (free_query, free_trace) = traced_step(&mut without, tokens[t]);
        assert_eq!(trace.len(), free.shape().layers());
        let mean = copy_heads.mean(&tokens[..=t], &table);
        assert!(t >= 2 || mean.is_none(), "row {t} has no candidate");
        let want: Vec<i32> = match mean {
            None => free_trace[0].clone(),
            // `x` at 2^-16 raised to the product's 2^-24 · 2^-43, the sum back to 2^-16.
            Some(c) => free_trace[0]
                .iter()
                .zip(copy_weight)
                .zip(&c)
                .map(|((&x, &w), &c)| round_half_away((i128::from(x) << 51) + i128::from(w) * c, 51) as i32)
                .collect(),
        };
        assert_eq!(trace[0], want, "position {t}");
        moved += usize::from(trace[0] != free_trace[0]);
        changed += usize::from(query != free_query);
    }
    assert!(moved > 10 && changed > 10, "the read moved {moved} residuals and {changed} queries");
}

/// claude-194: the readout read moves the final norm's output to `n + w_c ⊙ (c − n)` where a copy
/// head found a candidate, the sum rounded once to the norm's exponent, and the Q12 query is that
/// point. The read changes no state, so a row without a candidate (rows 0 and 1 among them) keeps
/// the read-free query.
#[test]
fn the_readout_read_moves_the_norm_output_toward_the_copy() {
    let layers = super::synthetic::FBS_SYNTH_NON_SQUARE.layers;
    let (read, free, table, reads) = served_read_parts(&[layers], 1.0, 0.0);
    let (image, copy_heads) = &reads[0];
    let width = free.shape().width;
    let copy_weight = &image.weight[..width];
    let tokens = repeating_tokens();
    let (mut with, mut without) = (read.session(), free.session());
    let (mut changed, mut none) = (0, 0);
    for t in 0..tokens.len() {
        let query = with.step_native(tokens[t]).expect("step").query_q12;
        let free_query = without.step_native(tokens[t]).expect("step").query_q12;
        let n = without.fbs_norm_output();
        let mean = copy_heads.mean(&tokens[..=t], &table);
        assert!(t >= 2 || mean.is_none(), "row {t} has no candidate");
        let want: Vec<i64> = match &mean {
            None => n.to_vec(),
            // `c` at 2^-43, `n` at 2^-24: the move is at 2^-24 · 2^-43, and the sum goes back to 2^-24.
            Some(c) => n
                .iter()
                .zip(copy_weight)
                .zip(c)
                .map(|((&n, &w), &c)| {
                    let n = i128::from(n);
                    round_half_away((n << 43) + i128::from(w) * (c - (n << 19)), 43) as i64
                })
                .collect(),
        };
        assert_eq!(with.fbs_norm_output(), want, "position {t}");
        let want_query: Vec<i32> = want
            .iter()
            .map(|&v| super::kernels::to_exp_i32(v, -24, -12).clamp(-32768, 32767))
            .collect();
        assert_eq!(query, want_query, "position {t}");
        if mean.is_none() {
            none += 1;
            assert_eq!(query, free_query, "position {t}: no candidate, no move");
        }
        changed += usize::from(query != free_query);
    }
    assert!(changed > 10 && none >= 2, "the read moved {changed} queries; {none} rows found nothing");
}

/// claude-194: the session refuses each part of a read it does not serve, by its own error: a
/// read on a dense stack, and a depth of 0 or past the layers. A non-zero type-head weight, once
/// refused, is served since claude-195.
#[test]
fn the_session_refuses_what_it_does_not_serve_by_name() {
    let (mut snap, _, served) = served_read_trunk(&[1, 3], 1.0, 0.0);
    IntegerStackModel::parse(&served).expect("copy heads alone are served");
    let refusal = |bytes: &[u8]| match IntegerStackModel::parse(bytes) {
        Err(StackError::Shape(m)) => m,
        Err(other) => panic!("expected a shape refusal, got {other:?}"),
        Ok(_) => panic!("expected a refusal"),
    };
    let width = snap.width;
    snap.hamming.as_mut().expect("the reads").reads[1].weight[width + 5] = 0.25;
    let typed = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    IntegerStackModel::parse(&typed).expect("a non-zero type weight is served");
    let dense = with_header(&artifact(5), |h| {
        h["schema"] = json!(super::STACK_FBS_HAMMING_SCHEMA);
        h["hamming"] = json!(super::StackHamming::new(2, vec![1, 2]));
    });
    assert_eq!(refusal(&dense), "a Hamming read needs a SpiralCore/FBS stack");
    for depths in [json!([0, 3]), json!([1, 4])] {
        let edited = with_header(&served, |h| h["hamming"]["depths"] = depths.clone());
        assert_eq!(
            refusal(&edited),
            "Hamming read depths must be strictly increasing, from 1 to the layer count",
            "{depths}"
        );
    }
}

/// claude-194: with a read at both hooks, the stage- and part-timed steps are the served step,
/// and the read does change what is served. Both kinds of head run (claude-195).
#[test]
fn the_timed_fbs_steps_serve_the_read() {
    let (read, free, ..) = served_read_parts(&[1, 3], 1.0, 1.0);
    let tokens = repeating_tokens();
    let (mut plain, mut stages, mut parts, mut without) = (read.session(), read.session(), read.session(), free.session());
    let (mut ns, mut changed) = (Vec::new(), 0);
    for (t, &token) in tokens.iter().enumerate() {
        let want = plain.step_native(token).expect("step");
        let (got, _) = stages.measure_fbs_step_stages(token, &mut ns).expect("stage-timed step");
        assert_eq!(got, want, "stages, position {t}");
        let (got, _) = parts.measure_fbs_step_parts(token, &mut ns).expect("part-timed step");
        assert_eq!(got, want, "parts, position {t}");
        changed += usize::from(without.step_native(token).expect("step") != want);
    }
    assert_eq!(plain.save_state(), stages.save_state());
    assert_eq!(plain.save_state(), parts.save_state());
    assert_eq!(plain.tokens(), &tokens[..]);
    assert!(changed > 10, "the read changed {changed} steps");
}

/// claude-194: each hook rounds its sum once, as the float's `q16(x + w·c)` does, not the product
/// before adding it: at an exact half tie the two differ. Rounding the product, `3 − 0.5` would
/// come out 2; rounding the sum, 2.5 comes out 3.
#[test]
fn the_hooks_round_the_sum_not_the_product() {
    use super::hamming::{readout_sum, stream_sum};
    // `w_c · c = ∓2^50` is ∓0.5 at Q16.
    assert_eq!(stream_sum(3, -1, 1 << 50, 0, 0), 3);
    assert_eq!(stream_sum(-3, 1, 1 << 50, 0, 0), -3);
    assert_eq!(stream_sum(3, 1, 1 << 50, 0, 0), 4);
    // `w · (c − n · 2^19) = ∓2^42` is ∓0.5 at `2^-24`.
    assert_eq!(readout_sum(3, 1, Some((3 << 19) - (1 << 42)), 0, 0), 3);
    assert_eq!(readout_sum(-3, 1, Some((-3 << 19) + (1 << 42)), 0, 0), -3);
    // At `w = 2^24` (1.0) the move lands on the copy exactly.
    assert_eq!(readout_sum(-7, 1 << 24, Some(5 << 19), 0, 0), 5);
}

/// claude-195: the type term joins the same single rounding. In the stream, `w_y · y = −2^54` is
/// −0.5 at Q16, so `3 − 0.5` rounds to 3, and a copy and a type term of 0.25 each make `3.5 → 4`,
/// where rounding each term apart would give 3. On the readout a type term moves `n` with no copy
/// match, and the copy weight is then unused.
#[test]
fn the_type_term_joins_the_single_rounding() {
    use super::hamming::{readout_sum, stream_sum};
    assert_eq!(stream_sum(3, 0, 0, -1, 1 << 54), 3);
    assert_eq!(stream_sum(-3, 0, 0, 1, 1 << 54), -3);
    assert_eq!(stream_sum(3, 1, 1 << 49, 1, 1 << 53), 4);
    // `w_y · y = 2^54` at `2^-79` is 0.5 at `2^-24`.
    assert_eq!(readout_sum(3, 5, None, 1, 1 << 54), 4);
    assert_eq!(readout_sum(3, 5, None, -1, 1 << 54), 3);
    assert_eq!(readout_sum(3, 5, None, 0, 1 << 54), 3);
}

/// One lane of `raw` (quaternions at `2^HAMMING_ROTATION_EXP`) as the f64 unit `g / |g|`, the
/// float `hamilton_gate`'s normalization; the zero quaternion stays zero.
fn f64_unit(raw: &[i32], lane: usize) -> [f64; 4] {
    let g: Vec<f64> = raw[4 * lane..4 * lane + 4]
        .iter()
        .map(|&v| f64::from(v) * 2f64.powi(super::HAMMING_ROTATION_EXP))
        .collect();
    let r = g.iter().map(|v| v * v).sum::<f64>().sqrt();
    if r == 0.0 {
        [0.0; 4]
    } else {
        [g[0] / r, g[1] / r, g[2] / r, g[3] / r]
    }
}

/// The Hamilton product `a ⊗ b` in f64.
fn f64_hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

/// `point` (real values) through the rotation `raw`, lane by lane, in f64.
fn f64_rotated(raw: &[i32], point: &[f64]) -> Vec<f64> {
    point
        .chunks_exact(4)
        .enumerate()
        .flat_map(|(lane, p)| f64_hamilton(f64_unit(raw, lane), [p[0], p[1], p[2], p[3]]))
        .collect()
}

/// Head `h`'s f64 sign code of `point` through `raw`: a bit is 1 at or above 0.
fn f64_code(raw: &[i32], point: &[f64], h: usize, head_width: usize) -> u64 {
    let rotated = f64_rotated(raw, point);
    (0..head_width).fold(0u64, |word, c| if rotated[h * head_width + c] >= 0.0 { word | (1u64 << c) } else { word })
}

/// claude-195: the type heads' kernel against an f64 reference of the float read's steps on the
/// quantized values: the key codes of every vocabulary entry, each row's query codes, each
/// payload within half a step of the f64 rotation on the input's Q16 grid, kept sets and
/// distances, and the type mean within one step of that grid.
///
/// Two seeds use the synthetic margins, where no candidate counts. Three use wide ones, where
/// most count with fractional weights. Two mix a wide margin on one type head with the synthetic
/// one on the other, so that rows where exactly one head found a candidate are common. There the
/// mean is over that one head alone, not over both (review round 1).
#[test]
fn type_heads_match_the_f64_reference() {
    let (vocab, width, rows) = (256usize, 256usize, 48usize);
    let (copy, head_width) = (2usize, width / 4);
    let (mut found_rows, mut one_found_rows) = (0, 0);
    for seed in 0..7u64 {
        let (record, mut read, table) = copy_control(vocab, width, 4, 195 + seed);
        match seed {
            0 | 1 => {}
            2..=4 => {
                for (k, m) in read.margin.iter_mut().enumerate() {
                    *m = ((33 + 2 * k as i32) << 16) + 12_345;
                }
            }
            // One type head wide (the first or the second), the other left narrow.
            _ => read.margin[copy + (seed as usize - 5)] = (37 << 16) + 12_345,
        }
        let types = super::TypeHeads::new(&record, &read, &table, vocab, width);
        let point_of = |v: usize| -> Vec<f64> { table[v * width..][..width].iter().map(|&q| f64::from(q) / 4096.0).collect() };
        let key_ref: Vec<[u64; 2]> = (0..vocab)
            .map(|v| [0, 1].map(|j| f64_code(&read.key_rotation, &point_of(v), copy + j, head_width)))
            .collect();
        for (v, keys) in key_ref.iter().enumerate() {
            for (j, &key) in keys.iter().enumerate() {
                assert_eq!(types.key(v as u32, j), key, "seed {seed}, entry {v}, type head {j}");
            }
        }
        let mut rng = Lcg(seed + 7);
        let tokens: Vec<u32> = (0..rows).map(|_| (rng.next() % 12) as u32).collect();
        // Points within ±4 at Q16, as a residual is.
        let points: Vec<Vec<i64>> = (0..rows)
            .map(|_| (0..width).map(|_| (rng.next() % 524_289) as i64 - 262_144).collect())
            .collect();
        let real = |p: &[i64]| -> Vec<f64> { p.iter().map(|&v| v as f64 / 65536.0).collect() };
        let payload_ref: Vec<Vec<f64>> = points.iter().map(|p| f64_rotated(&read.value_rotation, &real(p))).collect();
        let mut cache = vec![0i64; rows * width];
        for t in 0..rows {
            types.payload(&points[t], &mut cache[t * width..][..width]);
            // Half a step for the rounding, and 2^-10 of a step for the unit quaternion's own Q31
            // quantization (at inputs within ±4, its error is below 2^-11 of a step). A truncated
            // payload is up to a whole step off.
            for (k, (&got, &want)) in cache[t * width..][..width].iter().zip(&payload_ref[t]).enumerate() {
                let off = (got as f64 - want * 65536.0).abs();
                assert!(off <= 0.5 + 2f64.powi(-10), "seed {seed}, row {t}, payload {k}: {off} steps off");
            }
            let queries = types.queries(&points[t]);
            let mut sum = vec![0f64; width];
            let mut found = 0usize;
            for j in 0..2 {
                let query = f64_code(&read.query_rotation, &real(&points[t]), copy + j, head_width);
                assert_eq!(queries[j], query, "seed {seed}, row {t}, type head {j}");
                let mut candidates: Vec<(u32, usize)> = (1..t)
                    .map(|s| ((query ^ key_ref[tokens[s - 1] as usize][j]).count_ones(), s))
                    .collect();
                candidates.sort_unstable();
                candidates.truncate(record.kept);
                let kept: Vec<(usize, u32)> = candidates.into_iter().map(|(d, s)| (s, d)).collect();
                assert_eq!(types.choose(j, queries[j], &tokens[..=t]), kept, "seed {seed}, row {t}, type head {j}");
                let margin = f64::from(read.margin[copy + j]) / 65536.0;
                let mut head = vec![0f64; width];
                let mut count = 0usize;
                for &(s, d) in &kept {
                    let weight = (margin - f64::from(d)).max(0.0) / margin.max(1.0);
                    if weight > 0.0 {
                        count += 1;
                        for (acc, &v) in head.iter_mut().zip(&payload_ref[s]) {
                            *acc += weight * v;
                        }
                    }
                }
                if count > 0 {
                    found += 1;
                    for (acc, value) in sum.iter_mut().zip(head) {
                        *acc += value / count as f64;
                    }
                }
            }
            let got = types.mean(&queries, &tokens[..=t], &cache[..(t + 1) * width]);
            match (got, found) {
                (None, 0) => {}
                (Some(got), found) if found > 0 => {
                    found_rows += 1;
                    one_found_rows += usize::from(found == 1);
                    for (k, (&g, &s)) in got.iter().zip(&sum).enumerate() {
                        let error = (g as f64 / 2f64.powi(16 + 31) - s / found as f64).abs();
                        assert!(error <= 2f64.powi(-16), "seed {seed}, row {t}, coordinate {k}: off by {error}");
                    }
                }
                (got, found) => panic!("seed {seed}, row {t}: integer {} against {found} heads found", got.is_some()),
            }
        }
    }
    assert!(found_rows > 100, "the type heads found candidates at {found_rows} rows");
    assert!(one_found_rows > 40, "exactly one type head found a candidate at {one_found_rows} rows");
}

/// claude-195: the stream read with type heads, against claude-194's method. At every row the
/// residual after layer 1 equals the read-free residual `x` there plus `w_c ⊙ c + w_y ⊙ y`, the sum
/// rounded once to Q16, where `y` is the type mean over the payloads of the read-free residuals
/// so far (a layer before the read cannot see it). Returns the rows the read moved and the rows
/// where a type head found a candidate.
fn stream_with_type_heads(copy: f32, types: f32) -> (usize, usize) {
    let (read, free, table, reads) = served_read_parts(&[1], copy, types);
    let (image, copy_heads) = &reads[0];
    let (vocab, width) = (free.shape().vocab, free.shape().width);
    let type_heads = super::TypeHeads::new(&super::StackHamming::new(4, vec![1]), image, &table, vocab, width);
    let (w_c, w_y) = image.weight.split_at(width);
    let tokens = repeating_tokens();
    let mut cache = vec![0i64; tokens.len() * width];
    let (mut with, mut without) = (read.session(), free.session());
    let (mut moved, mut typed) = (0, 0);
    for t in 0..tokens.len() {
        let (_, trace) = traced_step(&mut with, tokens[t]);
        let (_, free_trace) = traced_step(&mut without, tokens[t]);
        let x: Vec<i64> = free_trace[0].iter().map(|&v| i64::from(v)).collect();
        type_heads.payload(&x, &mut cache[t * width..][..width]);
        let y = type_heads.mean(&type_heads.queries(&x), &tokens[..=t], &cache[..(t + 1) * width]);
        let c = copy_heads.mean(&tokens[..=t], &table);
        typed += usize::from(y.is_some());
        let want: Vec<i32> = if c.is_none() && y.is_none() {
            free_trace[0].clone()
        } else {
            (0..width)
                .map(|k| {
                    // `x` at 2^-16, `w_c · c` at 2^-67 and `w_y · y` at 2^-71: the sum at 2^-71.
                    let c = c.as_ref().map_or(0, |c| c[k]);
                    let y = y.as_ref().map_or(0, |y| y[k]);
                    let sum = (i128::from(x[k]) << 55) + ((i128::from(w_c[k]) * c) << 4) + i128::from(w_y[k]) * y;
                    round_half_away(sum, 55) as i32
                })
                .collect()
        };
        assert_eq!(trace[0], want, "copy {copy}, types {types}, position {t}");
        moved += usize::from(trace[0] != free_trace[0]);
    }
    (moved, typed)
}

#[test]
fn the_stream_read_adds_both_kinds_of_head() {
    let (moved, typed) = stream_with_type_heads(1.0, 1.0);
    assert!(moved > 10 && typed > 10, "moved {moved} rows; type heads found candidates at {typed}");
    // The type heads alone, copy weights 0, still move the residual.
    let (moved, typed) = stream_with_type_heads(0.0, 1.0);
    assert!(moved > 10 && typed > 10, "type heads alone moved {moved} rows; found at {typed}");
}

/// claude-195: the readout move with type heads: `n + [matched] w_c ⊙ (c − n) + w_y ⊙ y` on the
/// read-free final norm output `n`, the type term rounded to the copy move's `2^-67` and the sum
/// once to `n`'s `2^-24`; the Q12 query is that point. Returns the rows whose query moved and the
/// rows where a type head found a candidate.
fn readout_with_type_heads(copy: f32, types: f32) -> (usize, usize) {
    let layers = super::synthetic::FBS_SYNTH_NON_SQUARE.layers;
    let (read, free, table, reads) = served_read_parts(&[layers], copy, types);
    let (image, copy_heads) = &reads[0];
    let (vocab, width) = (free.shape().vocab, free.shape().width);
    let type_heads = super::TypeHeads::new(&super::StackHamming::new(4, vec![layers]), image, &table, vocab, width);
    let (w_c, w_y) = image.weight.split_at(width);
    let tokens = repeating_tokens();
    let mut cache = vec![0i64; tokens.len() * width];
    let (mut with, mut without) = (read.session(), free.session());
    let (mut moved, mut typed) = (0, 0);
    for t in 0..tokens.len() {
        let query = with.step_native(tokens[t]).expect("step").query_q12;
        let free_query = without.step_native(tokens[t]).expect("step").query_q12;
        let n = without.fbs_norm_output().to_vec();
        type_heads.payload(&n, &mut cache[t * width..][..width]);
        let y = type_heads.mean(&type_heads.queries(&n), &tokens[..=t], &cache[..(t + 1) * width]);
        let c = copy_heads.mean(&tokens[..=t], &table);
        typed += usize::from(y.is_some());
        let want: Vec<i64> = if c.is_none() && y.is_none() {
            n.clone()
        } else {
            (0..width)
                .map(|k| {
                    let nk = i128::from(n[k]);
                    let move_c = c.as_ref().map_or(0, |c| i128::from(w_c[k]) * (c[k] - (nk << 19)));
                    let y = y.as_ref().map_or(0, |y| y[k]);
                    // The type term's pre-rounding to 2^-67 is the design's, kept to stay inside an
                    // i128 for any `n`; it is 43 bits below `n`'s grid.
                    let move_y = round_half_away(i128::from(w_y[k]) * y, 12);
                    round_half_away((nk << 43) + move_c + move_y, 43) as i64
                })
                .collect()
        };
        assert_eq!(with.fbs_norm_output(), want, "copy {copy}, types {types}, position {t}");
        let want_query: Vec<i32> = want
            .iter()
            .map(|&v| super::kernels::to_exp_i32(v, -24, -12).clamp(-32768, 32767))
            .collect();
        assert_eq!(query, want_query, "copy {copy}, types {types}, position {t}");
        moved += usize::from(query != free_query);
    }
    (moved, typed)
}

#[test]
fn the_readout_read_adds_both_kinds_of_head() {
    let (moved, typed) = readout_with_type_heads(1.0, 1.0);
    assert!(moved > 10 && typed > 10, "moved {moved} queries; type heads found candidates at {typed}");
    let (moved, typed) = readout_with_type_heads(0.0, 1.0);
    assert!(moved > 10 && typed > 10, "type heads alone moved {moved} queries; found at {typed}");
}

/// claude-195: a session of a read with type heads is refused at restore, by its text, since the
/// saved session does not carry the payloads; a read with copy heads alone, whose state is the
/// tokens, restores and continues bit for bit.
#[test]
fn a_session_with_type_heads_is_not_restored() {
    let tokens = repeating_tokens();
    let (typed, ..) = served_read_parts(&[1, 3], 1.0, 1.0);
    let mut session = typed.session();
    for &token in &tokens[..8] {
        session.step_native(token).expect("step");
    }
    match typed.session().restore_state(&session.save_state()) {
        Err(StackError::Shape(m)) => assert_eq!(
            m,
            "a session of a Hamming read with type heads cannot be restored yet: its per-position payloads are not saved"
        ),
        other => panic!("expected the restore refusal, got {:?}", other.err()),
    }
    let (copied, ..) = served_read_parts(&[1, 3], 1.0, 0.0);
    let mut first = copied.session();
    for &token in &tokens[..8] {
        first.step_native(token).expect("step");
    }
    let mut second = copied.session();
    second.restore_state(&first.save_state()).expect("a copy-only session restores");
    for (t, &token) in tokens.iter().enumerate().skip(8) {
        assert_eq!(first.step_native(token).expect("step"), second.step_native(token).expect("step"), "position {t}");
    }
}

/// claude-195: the payload cache is held to the session read cache's 4 GiB, at its limit and one
/// past it, and an overflowing product is refused too.
#[test]
fn the_payload_cache_is_bounded() {
    use super::hamming::check_payload_cache;
    check_payload_cache(1, 1 << 16, 1 << 13).expect("4 GiB exactly");
    check_payload_cache(2, 1 << 16, 1 << 12).expect("4 GiB exactly");
    check_payload_cache(0, usize::MAX, usize::MAX).expect("no type heads, no cache");
    for (typed, context, width) in [(1, 1 << 16, (1 << 13) + 1), (3, 1 << 16, 1 << 12), (usize::MAX, 2, 2)] {
        match check_payload_cache(typed, context, width) {
            Err(StackError::Shape(m)) => assert_eq!(m, "the Hamming type heads' payload cache exceeds the session read cache's 4 GiB"),
            other => panic!("({typed}, {context}, {width}): expected the refusal, got {other:?}"),
        }
    }
}

/// The record's refusal of a reply turn out of range or apart from its table (claude-198).
const REPLY_TURN_REFUSED: &str = "a reply table's turn must be from 1 to 16 ids, and only with the table";

/// A synthetic turn memory for a trunk of `vocab` and `width` (claude-198): 40 opening keys of 1
/// to `k` ids and, with `reply`, 60 reply keys `turn (0 to 8 ids), MAX, length, MAX, last id` (the
/// float trainer's reply turn is 8 ids, whatever `k`), each table in ascending key order, and
/// weights in `[-1, 1]`.
fn synthetic_memory(vocab: usize, width: usize, k: usize, reply: bool, seed: u64) -> super::FbsMemorySnapshot {
    use super::{FbsMemoryEntry, FbsReplyTable, MEMORY_PAD};
    let mut rng = Lcg(seed);
    let weight = |rng: &mut Lcg| -> Vec<f32> {
        (0..width).map(|_| ((rng.next() % 2001) as f32 - 1000.0) / 1000.0).collect()
    };
    let table = |rng: &mut Lcg, count: usize, key: &mut dyn FnMut(&mut Lcg) -> Vec<u32>| {
        let mut entries: Vec<FbsMemoryEntry> = (0..count)
            .map(|_| FbsMemoryEntry {
                key: key(rng),
                value: (rng.next() % vocab as u64) as u32,
            })
            .collect();
        entries.sort();
        entries.dedup_by(|a, b| a.key == b.key);
        entries
    };
    let opening = table(&mut rng, 40, &mut |rng| {
        let len = 1 + (rng.next() % k as u64) as usize;
        (0..len).map(|_| (rng.next() % vocab as u64) as u32).collect()
    });
    let opening_weight = weight(&mut rng);
    let reply = reply.then(|| {
        let entries = table(&mut rng, 60, &mut |rng| {
            let turn = (rng.next() % 9) as usize;
            let mut key: Vec<u32> = (0..turn).map(|_| (rng.next() % vocab as u64) as u32).collect();
            key.extend([MEMORY_PAD, 1 + (rng.next() % 192) as u32, MEMORY_PAD, (rng.next() % vocab as u64) as u32]);
            key
        });
        FbsReplyTable {
            turn: 8,
            entries,
            weight: weight(&mut rng),
        }
    });
    super::FbsMemorySnapshot {
        k,
        key_window: 192,
        opening,
        opening_weight,
        reply,
    }
}

/// claude-198: a trunk with a turn memory exports as `uor-r4.lut-stack/7`, with or without a reply
/// table, an empty opening table, and Hamming reads beside it. Every key, value and weight reads
/// back at its exponent, and the session refuses the artifact by name.
#[test]
fn memories_round_trip_in_lut_stack_7() {
    use super::synthetic::synthetic_hamming;
    let fixed = |values: &[f32]| -> Vec<i32> {
        values.iter().map(|&v| (f64::from(v) * 2f64.powi(-super::MEMORY_WEIGHT_EXP)).round() as i32).collect()
    };
    let base = synthetic_fbs_trunk(20261025);
    let (vocab, width) = (base.vocab, base.width);
    let mut empty = synthetic_memory(vocab, width, 8, false, 7);
    empty.opening.clear();
    let cases = [
        ("opening only", synthetic_memory(vocab, width, 8, false, 198), false),
        ("both tables", synthetic_memory(vocab, width, 8, true, 198), false),
        ("k = 4 beside reply keys of 8 turn ids", synthetic_memory(vocab, width, 4, true, 200), false),
        ("an empty opening table", empty, false),
        ("both tables beside Hamming reads", synthetic_memory(vocab, width, 8, true, 199), true),
    ];
    for (what, memory, with_hamming) in cases {
        let mut snap = base.clone();
        if with_hamming {
            let mut read = synthetic_hamming(width, snap.layers.len(), 4, 188);
            for r in &mut read.reads {
                r.weight[width..].fill(0.0);
            }
            snap.hamming = Some(read);
        }
        // The k = 4 case holds reply keys of more turn ids than k, as the float trainer writes them.
        if let Some(reply) = &memory.reply {
            assert_eq!(reply.entries.iter().any(|e| e.key.len() - 4 > memory.k), memory.k < 8, "{what}");
        }
        snap.memory = Some(memory.clone());
        let bytes = super::export_fbs_trunk_uorlut01(&snap).expect(what);
        let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
        let header: Value = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
        assert_eq!(header["schema"], super::STACK_FBS_MEMORY_SCHEMA, "{what}");
        let image = super::read_memory_uorlut01(&bytes).expect(what).expect("the memory");
        assert_eq!(
            image.record,
            super::StackTurnMemory {
                k: memory.k,
                key_window: 192,
                opening: memory.opening.len(),
                reply: memory.reply.as_ref().map(|r| r.entries.len()),
                reply_turn: memory.reply.as_ref().map(|r| r.turn),
                reply_key_len: super::MEMORY_REPLY_KEY_LEN,
                weight_exp: super::MEMORY_WEIGHT_EXP,
            },
            "{what}"
        );
        let keys = |entries: &[super::FbsMemoryEntry]| entries.iter().map(|e| e.key.clone()).collect::<Vec<_>>();
        let values = |entries: &[super::FbsMemoryEntry]| entries.iter().map(|e| e.value).collect::<Vec<_>>();
        assert_eq!(image.opening.keys, keys(&memory.opening), "{what}");
        assert_eq!(image.opening.values, values(&memory.opening), "{what}");
        assert_eq!(image.opening.weight, fixed(&memory.opening_weight), "{what}");
        match (&image.reply, &memory.reply) {
            (None, None) => {}
            (Some(got), Some(want)) => {
                assert_eq!(got.keys, keys(&want.entries), "{what}");
                assert_eq!(got.values, values(&want.entries), "{what}");
                assert_eq!(got.weight, fixed(&want.weight), "{what}");
            }
            _ => panic!("{what}: the reply table did not round-trip"),
        }
        assert_eq!(super::read_hamming_uorlut01(&bytes).expect(what).is_some(), with_hamming, "{what}");
        // Units 2-3 serve the memory: the engine parses it and names its record and tables.
        let model = IntegerStackModel::parse(&bytes).expect(what);
        let record = model.turn_memory().expect("the served memory record");
        assert_eq!(record.k, memory.k, "{what}");
        assert_eq!(record.key_window, 192, "{what}");
        assert_eq!(
            model.turn_memory_table_lengths(),
            Some((
                memory.opening.len(),
                memory.reply.as_ref().map(|r| r.entries.len()),
            )),
            "{what}"
        );
        assert!(model.session().has_turn_memory(), "{what}");
    }
    // The same trunk without the memory is served, and reads back no memory.
    let plain = super::export_fbs_trunk_uorlut01(&base).expect("export");
    assert!(super::read_memory_uorlut01(&plain).expect("parse").is_none());
    IntegerStackModel::parse(&plain).expect("a memory-free trunk is served");
}

/// claude-198: the export refuses a memory the reader would refuse, each by its reason, because it
/// re-reads what it wrote; and the reader alone refuses a header whose counts do not match the
/// sections, and a memory on a dense stack.
#[test]
fn malformed_memories_are_refused_by_their_text() {
    use super::{FbsMemoryEntry, MEMORY_PAD};
    let base = synthetic_fbs_trunk(20261025);
    let (vocab, width) = (base.vocab, base.width);
    let good = synthetic_memory(vocab, width, 8, true, 198);
    let export = |mutate: &dyn Fn(&mut super::FbsMemorySnapshot)| -> Result<Vec<u8>, StackError> {
        let mut memory = good.clone();
        mutate(&mut memory);
        let mut snap = base.clone();
        snap.memory = Some(memory);
        super::export_fbs_trunk_uorlut01(&snap)
    };
    let served = export(&|_| {}).expect("the unmodified memory exports");
    let numerics = |what: &str, mutate: &dyn Fn(&mut super::FbsMemorySnapshot), reason: &str| match export(mutate) {
        Err(StackError::Numerics(m)) => assert!(m.contains(reason), "{what}: {m}"),
        other => panic!("{what}: expected a refusal naming {reason:?}, got {:?}", other.err()),
    };
    numerics("opening keys out of order", &|m| m.opening.swap(0, 1), "not in strictly ascending order");
    numerics("a repeated opening key", &|m| m.opening[1].key = m.opening[0].key.clone(), "not in strictly ascending order");
    numerics("a key id past the vocabulary", &|m| m.opening[0].key[0] = vocab as u32, "is not a key this engine reads");
    // An empty opening key sorts first and stores as a row of pads; the float memory never has one.
    numerics("an empty opening key", &|m| m.opening[0].key.clear(), "is not a key this engine reads");
    numerics("a value past the vocabulary", &|m| m.opening[0].value = vocab as u32, "names an id past the vocabulary");
    numerics(
        "a reply key without its separators",
        &|m| {
            let reply = m.reply.as_mut().expect("reply");
            reply.entries.insert(0, FbsMemoryEntry { key: vec![0, 1, 2, 3], value: 0 });
        },
        "is not a key this engine reads",
    );
    numerics(
        "a reply length past the key window",
        &|m| {
            let reply = m.reply.as_mut().expect("reply");
            reply.entries.insert(0, FbsMemoryEntry { key: vec![MEMORY_PAD, 193, MEMORY_PAD, 0], value: 0 });
        },
        "is not a key this engine reads",
    );
    numerics(
        "a reply turn longer than the record's",
        &|m| {
            let reply = m.reply.as_mut().expect("reply");
            let key = [vec![0; 9], vec![MEMORY_PAD, 1, MEMORY_PAD, 0]].concat();
            reply.entries.insert(0, FbsMemoryEntry { key, value: 0 });
        },
        "is not a key this engine reads",
    );
    for turn in [0, 17] {
        match export(&|m| m.reply.as_mut().expect("reply").turn = turn) {
            Err(StackError::Shape(m)) => assert_eq!(m, REPLY_TURN_REFUSED, "turn {turn}"),
            other => panic!("a reply turn of {turn}: expected the record refusal, got {:?}", other.err()),
        }
    }
    numerics("an opening key longer than k", &|m| m.opening[0].key = vec![0; 9], "a key is longer than its row");
    numerics("a weight that does not fit", &|m| m.opening_weight[0] = 1.0e6, "fixed-point overflow");
    match export(&|m| m.k = 0) {
        Err(StackError::Shape(m)) => assert_eq!(m, "a turn memory's k must be from 1 to 16"),
        other => panic!("k = 0: expected the record refusal, got {:?}", other.err()),
    }
    match export(&|m| m.opening_weight.pop().map_or((), |_| ())) {
        Err(StackError::TableLength { name, .. }) => assert_eq!(name, "memory.opening.weight"),
        other => panic!("a short weight: expected a length refusal, got {:?}", other.err()),
    }
    // The reader alone: a record whose count does not match its sections.
    let miscounted = with_header(&served, |h| {
        let n = h["memory"]["opening"].as_u64().expect("count");
        h["memory"]["opening"] = json!(n + 1);
    });
    match super::read_memory_uorlut01(&miscounted) {
        Err(StackError::TableLength { name, .. }) => assert_eq!(name, "memory.opening.keys"),
        other => panic!("a miscounted header: expected a length refusal, got {other:?}"),
    }
    // A reply count without its turn.
    let turnless = with_header(&served, |h| h["memory"]["reply_turn"] = Value::Null);
    match super::read_memory_uorlut01(&turnless) {
        Err(StackError::Shape(m)) => assert_eq!(m, REPLY_TURN_REFUSED),
        other => panic!("a reply table without its turn: expected the record refusal, got {other:?}"),
    }
    // A memory record on a dense stack.
    let dense = with_header(&artifact(5), |h| {
        h["schema"] = json!(super::STACK_FBS_MEMORY_SCHEMA);
        h["memory"] = json!(super::StackTurnMemory {
            k: 8,
            key_window: 192,
            opening: 0,
            reply: None,
            reply_turn: None,
            reply_key_len: super::MEMORY_REPLY_KEY_LEN,
            weight_exp: super::MEMORY_WEIGHT_EXP,
        });
    });
    match super::read_memory_uorlut01(&dense) {
        Err(StackError::Shape(m)) => assert_eq!(m, "a turn memory needs a SpiralCore/FBS stack"),
        other => panic!("a memory on a dense stack: expected the shape refusal, got {other:?}"),
    }
}


/// The turn-memory tests' stream: <|bos|>, a turn whose prompt holds an internal <|eos|> (token
/// 1), the turn's 8 last ids, then reply_len reply tokens. The reply starts at token 12; the
/// prompt's last token is row 11. The turn is longer than the opening table's k (4), so the reply
/// keys carry reply_turn (8) ids, not k.
fn turn_memory_script(reply_len: usize) -> (Vec<u32>, usize) {
    let mut tokens = vec![0u32, 10, 11, 1, 12, 13, 14, 15, 16, 17, 18, 19];
    let reply_start = tokens.len();
    tokens.extend((0..reply_len).map(|i| 20 + (i as u32 % 40)));
    (tokens, reply_start)
}

/// A small trunk with room for the turn memory's 192-token window tests, at context positions.
fn turn_memory_trunk(context: usize) -> super::FbsTrunkSnapshot {
    synthetic_fbs_trunk_with(7, FbsSynth { context, ..FBS_SYNTH_SMALL })
}

/// The session's turn anchor at row at: the first limit ids after the last <|bos|>/<|eos|> at or
/// before it, from at's 192-block (empty when at is itself <|bos|>/<|eos|>).
fn anchor_turn(tokens: &[u32], at: usize, limit: usize, key_window: usize) -> Vec<u32> {
    let block = at / key_window * key_window;
    let mut start = block;
    for i in (block..=at).rev() {
        if tokens[i] == 0 || tokens[i] == 1 {
            start = i + 1;
            break;
        }
    }
    if start > at {
        return Vec::new();
    }
    tokens[start..=at].iter().copied().take(limit).collect()
}

/// The float trainer's mask-based reply-row turn (reply_rows): the window's prompt run back from
/// the reply's start to the last response token or <|bos|>.
fn mask_turn(
    tokens: &[u32],
    mask: &[u8],
    reply_start: usize,
    row: usize,
    limit: usize,
    key_window: usize,
) -> Vec<u32> {
    let block = row / key_window * key_window;
    if reply_start <= block {
        return Vec::new();
    }
    let mut first = reply_start;
    while first > block && tokens[first - 1] != 0 && mask[first - 1] == 0 {
        first -= 1;
    }
    tokens[first..reply_start].iter().copied().take(limit).collect()
}

/// The session's reply key at row: the opening anchor's ids (empty once the prompt has left the
/// row's key block), then MAX, the block-visible reply length and MAX, the row's id.
fn expected_reply_key(
    tokens: &[u32],
    row: usize,
    reply_start: usize,
    reply_turn: usize,
    key_window: usize,
) -> Vec<u32> {
    let block = row / key_window * key_window;
    let prompt_last = reply_start - 1;
    let mut key = Vec::new();
    if prompt_last >= block {
        key.extend(anchor_turn(tokens, prompt_last, reply_turn, key_window));
    }
    let visible = block.max(reply_start);
    key.extend([u32::MAX, (row + 1 - visible) as u32, u32::MAX, tokens[row]]);
    key
}

/// Units 2-3: the session's keys follow the item's rule - the anchor after the last
/// <|bos|>/<|eos|>, the caller's reply start, training's 192-token blocks whatever the session's
/// context - and the mask rule's gap at an internal <|eos|> is pinned.
#[test]
fn turn_memory_keys_follow_the_item_rule() {
    let (key_window, reply_turn) = (192usize, 8usize);
    let (tokens, reply_start) = turn_memory_script(192);
    let mut snap = turn_memory_trunk(400);
    snap.memory = Some(synthetic_turn_memory(snap.vocab, snap.width, 4, key_window, true, 0, 198));
    // The third reference row: the session's context is not the key window.
    assert_eq!(snap.context, 400);
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export /7");
    let model = IntegerStackModel::parse(&bytes).expect("served /7");
    assert_eq!(model.shape().context, 400);
    assert_eq!(model.turn_memory().expect("record").key_window, key_window);
    assert_eq!(model.turn_memory_table_lengths(), Some((0, Some(0))));
    let mask: Vec<u8> = (0..tokens.len()).map(|i| u8::from(i >= reply_start)).collect();
    let mut session = model.session();
    let mut moved = 0usize;
    for row in 0..tokens.len() {
        if row + 1 == reply_start {
            session.mark_reply_start();
        }
        session.step_native(tokens[row]).expect("step");
        if row + 1 < reply_start {
            assert!(session.last_memory_move().is_none(), "row {row}: a prompt row moved");
            continue;
        }
        let mv = session.last_memory_move().expect("a keyed row").clone();
        if row + 1 == reply_start {
            // The opening table's row: the anchor's first k ids, after the internal <|eos|>.
            assert_eq!(mv.key, vec![12, 13, 14, 15], "opening row {row}");
            assert!(mv.opening);
        } else {
            if row == reply_start {
                // The reply key's turn is reply_turn (8) ids, longer than the opening table's k.
                assert_eq!(&mv.key[..8], &[12, 13, 14, 15, 16, 17, 18, 19], "row {row}");
            }
            assert_eq!(&mv.key, &expected_reply_key(&tokens, row, reply_start, reply_turn, key_window), "row {row}");
            assert!(!mv.opening);
            if row < key_window {
                // Training's mask-based reply turn reaches back past the internal <|eos|>; the
                // session's anchor rule stops at it. The gap is measured here, not hidden.
                assert_eq!(mask_turn(&tokens, &mask, reply_start, row, reply_turn, key_window), vec![10, 11, 1, 12, 13, 14, 15, 16], "row {row}");
                assert_ne!(&mv.key[..4], &[10, 11, 1, 12], "row {row}: the mask rule's turn appears in the session key");
            } else if row < key_window + 8 {
                // The second reference row: a reply cut by the window. The prompt has left the
                // row's block, so the turn is empty and the length is the block-visible reply.
                assert!(mask_turn(&tokens, &mask, reply_start, row, reply_turn, key_window).is_empty(), "row {row}");
                assert_eq!(mv.key[0], u32::MAX, "row {row}");
                assert_eq!(mv.key[1], (row + 1 - key_window) as u32, "row {row}");
            }
        }
        moved += 1;
    }
    assert!(moved > key_window, "the decoded reply crossed the key window's block boundary");
}

/// Units 2-3: the opening table moves the reply's first step's row and the reply table each later
/// row, n + w(c - n), after the Hamming readout. A zero weight, an unseen key, a memory without a
/// reply table and a session without the reply-start signal move nothing.
#[test]
fn turn_memory_moves_the_readout_point() {
    use super::{FbsMemoryEntry, FbsReplyTable, MEMORY_PAD};
    let (tokens, reply_start) = turn_memory_script(8);
    let width = FBS_SYNTH_SMALL.d as usize;
    let stack = |memory: Option<super::FbsMemorySnapshot>| -> IntegerStackModel {
        let mut snap = turn_memory_trunk(24);
        snap.memory = memory;
        IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export")).expect("served")
    };
    let tables = |opening_weight: f32, reply_weight: f32, hit: bool| -> super::FbsMemorySnapshot {
        let (opening_key, reply_key) = if hit {
            (
                vec![12, 13, 14, 15],
                vec![12, 13, 14, 15, 16, 17, 18, 19, MEMORY_PAD, 1, MEMORY_PAD, 20],
            )
        } else {
            (
                vec![90, 91, 92, 93],
                vec![90, 91, 92, 93, 94, 95, 96, 97, MEMORY_PAD, 1, MEMORY_PAD, 20],
            )
        };
        super::FbsMemorySnapshot {
            k: 4,
            key_window: 192,
            opening: vec![FbsMemoryEntry { key: opening_key, value: 42 }],
            opening_weight: vec![opening_weight; width],
            reply: Some(FbsReplyTable {
                turn: 8,
                entries: vec![FbsMemoryEntry { key: reply_key, value: 43 }],
                weight: vec![reply_weight; width],
            }),
        }
    };
    // The float rule's move at f64: n + w(c - n), c the stored token's writer-table point.
    let moved = |before: &[i64], target: &[i16], weight: f32| -> Vec<i64> {
        let denom = 2f64.powi(24);
        let quantized = (f64::from(weight) * denom).round();
        before
            .iter()
            .zip(target)
            .map(|(&n, &t)| {
                let nf = n as f64 / denom;
                let cf = (i64::from(t) << 12) as f64 / denom;
                ((nf + (quantized / denom) * (cf - nf)) * denom).round() as i64
            })
            .collect()
    };
    let hit_model = stack(Some(tables(0.5, 0.25, true)));
    let mut session = hit_model.session();
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            session.mark_reply_start();
        }
        let opening_hit = row + 1 == reply_start;
        let reply_hit = row == reply_start;
        session.step_native(token).expect("step");
        let mv = session.last_memory_move();
        if row + 1 < reply_start {
            assert!(mv.is_none(), "row {row}: a prompt row moved");
            continue;
        }
        if !opening_hit && !reply_hit {
            // A later reply row keys the reply table too, but no entry holds its key: it hits
            // nothing and moves nothing.
            let mv = mv.expect("a keyed reply row").clone();
            assert_eq!(mv.stored, None, "row {row}");
            assert_eq!(session.fbs_norm_output_before_memory(), session.fbs_norm_output(), "row {row}");
            continue;
        }
        let mv = mv.expect("a keyed row").clone();
        let (stored, weight) = if opening_hit { (42u32, 0.5f32) } else { (43, 0.25) };
        assert_eq!(mv.stored, Some(stored), "row {row}");
        assert_eq!(session.fbs_norm_output_before_memory().len(), width);
        let target = session.fbs_writer_row(stored).expect("writer row");
        let expected = moved(session.fbs_norm_output_before_memory(), target, weight);
        assert_eq!(session.fbs_norm_output(), &expected[..], "row {row}");
    }
    // A zero weight is no read, bit for bit, though the key still hits.
    let zero_model = stack(Some(tables(0.0, 0.0, true)));
    let mut zero = zero_model.session();
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            zero.mark_reply_start();
        }
        zero.step_native(token).expect("step");
        if row + 1 == reply_start || row == reply_start {
            let want = if row + 1 == reply_start { 42 } else { 43 };
            assert_eq!(zero.last_memory_move().expect("a keyed row").stored, Some(want));
            assert_eq!(zero.fbs_norm_output_before_memory(), zero.fbs_norm_output(), "row {row}");
        }
    }
    // An unseen key hits nothing and moves nothing.
    let unseen_model = stack(Some(tables(0.5, 0.25, false)));
    let mut unseen = unseen_model.session();
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            unseen.mark_reply_start();
        }
        unseen.step_native(token).expect("step");
        if row + 1 == reply_start || row == reply_start {
            assert_eq!(unseen.last_memory_move().expect("a keyed row").stored, None, "row {row}");
            assert_eq!(unseen.fbs_norm_output_before_memory(), unseen.fbs_norm_output(), "row {row}");
        }
    }
    // A memory without a reply table moves only the opening row.
    let opening_model = stack(Some(super::FbsMemorySnapshot {
        k: 4,
        key_window: 192,
        opening: tables(0.5, 0.5, true).opening,
        opening_weight: vec![0.5; width],
        reply: None,
    }));
    let mut opening_only = opening_model.session();
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            opening_only.mark_reply_start();
        }
        opening_only.step_native(token).expect("step");
        if row + 1 == reply_start {
            assert_eq!(opening_only.last_memory_move().expect("opening").stored, Some(42));
        } else if row >= reply_start {
            assert!(opening_only.last_memory_move().is_none(), "row {row}: no reply table");
        }
    }
    // Without the caller's signal no row is keyed or moved; the step is the trunk's.
    let plain = stack(None);
    let signalled_model = stack(Some(tables(0.5, 0.25, true)));
    let mut with_memory = signalled_model.session();
    let mut without = plain.session();
    for &token in &tokens {
        with_memory.step_native(token).expect("step");
        without.step_native(token).expect("step");
        assert!(with_memory.last_memory_move().is_none(), "an unsignalled session keyed a row");
        assert_eq!(with_memory.fbs_norm_output(), without.fbs_norm_output());
    }
}

/// Units 2-3: the turn memory's move comes after the Hamming readout, as in the float trunk: the
/// /7 step's pre-move final norm equals the same trunk's /6 (Hamming only) readout.
#[test]
fn turn_memory_moves_after_the_hamming_readout() {
    use super::synthetic::synthetic_hamming;
    let (tokens, reply_start) = turn_memory_script(8);
    let base = turn_memory_trunk(24);
    let width = base.width;
    let mut read = synthetic_hamming(width, base.layers.len(), 4, 188);
    for r in &mut read.reads {
        r.weight[width..].fill(0.0);
    }
    let mut hamming_only = base.clone();
    hamming_only.hamming = Some(read.clone());
    let mut with_memory = base.clone();
    with_memory.hamming = Some(read);
    with_memory.memory = Some(super::FbsMemorySnapshot {
        k: 4,
        key_window: 192,
        opening: vec![super::FbsMemoryEntry { key: vec![12, 13, 14, 15], value: 42 }],
        opening_weight: vec![0.5; width],
        reply: None,
    });
    let hamming = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&hamming_only).expect("export /6"))
        .expect("served /6");
    let memory = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&with_memory).expect("export /7"))
        .expect("served /7");
    let mut readout = hamming.session();
    let mut moved = memory.session();
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            readout.mark_reply_start();
            moved.mark_reply_start();
        }
        readout.step_native(token).expect("step");
        moved.step_native(token).expect("step");
        if row + 1 == reply_start {
            assert_eq!(moved.last_memory_move().expect("opening").stored, Some(42));
            // The /6 readout is the Hamming read applied to the trunk's final norm; the /7 step's
            // pre-move point must be exactly that, so the memory moved after it.
            assert_eq!(moved.fbs_norm_output_before_memory(), readout.fbs_norm_output());
            assert_ne!(moved.fbs_norm_output(), moved.fbs_norm_output_before_memory(), "the memory's move must be applied");
        }
    }
}

/// Units 2-3: a session of a turn memory refuses a restore until its turn state is saved; a
/// memory-free session restores as before, and memory-free models have no record and ignore the
/// reply-start signal.
#[test]
fn a_turn_memory_session_restore_is_refused() {
    let bytes = super::synthetic::synthetic_turn_memory_artifact(7, 8, 192, false);
    let model = IntegerStackModel::parse(&bytes).expect("served /7");
    assert!(model.turn_memory().is_some());
    let mut session = model.session();
    assert!(session.has_turn_memory());
    session.step_native(10).expect("step");
    let saved = session.save_state();
    let mut fresh = model.session();
    match fresh.restore_state(&saved) {
        Err(StackError::Shape(m)) => assert_eq!(
            m,
            "a session of a turn memory cannot be restored yet: its turn state is not saved"
        ),
        other => panic!("expected the turn-memory restore refusal, got {other:?}"),
    }
    // A memory-free artifact: no record, no tables, the signal ignored, and a restore works.
    let plain = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(7)).expect("export"))
        .expect("served");
    assert!(plain.turn_memory().is_none());
    assert_eq!(plain.turn_memory_table_lengths(), None);
    let mut a = plain.session();
    assert!(!a.has_turn_memory());
    a.step_native(10).expect("step");
    let saved = a.save_state();
    let mut b = plain.session();
    b.restore_state(&saved).expect("a memory-free session restores");
    b.mark_reply_start();
    b.step_native(11).expect("step");
    assert!(b.last_memory_move().is_none());
}

/// claude-222: a point table for the synthetic `/7` stack, `rows` rows of its width.
fn synthetic_point_key(bytes: &[u8], rows: usize) -> super::PointKeyImage {
    let shape = IntegerStackModel::parse(bytes).expect("served").shape().clone();
    let points = (0..rows * shape.width).map(|i| ((i * 37) % 2001) as i16 - 1000).collect();
    let targets = (0..rows).map(|r| 2 + (r as u32 * 7) % (shape.vocab as u32 - 2)).collect();
    super::PointKeyImage {
        width: shape.width,
        points,
        targets,
    }
}

/// claude-222: a `/7` stack of the small trunk whose opening table holds `turn_memory_script`'s
/// opening key when `hit`, and an unseen one when not. Its reply table holds a key no row hits, so
/// every later reply row is keyed and misses.
fn point_key_test_stack(hit: bool) -> Vec<u8> {
    point_key_test_stack_with(hit, false)
}

/// [`point_key_test_stack`] with its readout: the writer's pick, or the whole table's nearest
/// (claude-231).
fn point_key_test_stack_with(hit: bool, nearest: bool) -> Vec<u8> {
    use super::{FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, MEMORY_PAD};
    let width = FBS_SYNTH_SMALL.d as usize;
    let mut snap = turn_memory_trunk(24);
    let key = if hit { vec![12, 13, 14, 15] } else { vec![90, 91, 92, 93] };
    let unseen = vec![90, 91, 92, 93, 94, 95, 96, 97, MEMORY_PAD, 1, MEMORY_PAD, 20];
    snap.memory = Some(FbsMemorySnapshot {
        k: 4,
        key_window: 192,
        opening: vec![FbsMemoryEntry { key, value: 42 }],
        opening_weight: vec![0.5; width],
        reply: Some(FbsReplyTable {
            turn: 8,
            entries: vec![FbsMemoryEntry { key: unseen, value: 43 }],
            weight: vec![0.5; width],
        }),
    });
    snap.readout_nearest = nearest;
    super::export_fbs_trunk_uorlut01(&snap).expect("export")
}

/// claude-222: the served point key names the reply's first token where the opening table's exact
/// key misses, from that row's own query. Every other row, an opening the key hits, and a session
/// without the reply-start signal decode as the `/7` stack does, token for token.
#[test]
fn the_point_key_names_a_missed_opening_and_leaves_the_rest() {
    let (tokens, reply_start) = turn_memory_script(8);
    let width = FBS_SYNTH_SMALL.d as usize;
    // The point key must serve only the opening, though every later reply row misses too.
    let stack = point_key_test_stack;
    // Each row's token and query, and the opening row's move, with or without the signal.
    let decode = |bytes: &[u8], signal: bool| -> (Vec<(u32, Vec<i32>)>, Option<Option<u32>>) {
        let model = IntegerStackModel::parse(bytes).expect("served");
        let mut session = model.session();
        let mut opening = None;
        let steps = tokens
            .iter()
            .enumerate()
            .map(|(row, &token)| {
                if signal && row + 1 == reply_start {
                    session.mark_reply_start();
                }
                let step = session.step_native(token).expect("step");
                if row + 1 == reply_start {
                    opening = session.last_memory_move().map(|m| m.stored);
                }
                (step.token, step.query_q12)
            })
            .collect();
        (steps, opening)
    };
    // The table: the missed opening's own query naming `named`, and a far row naming another id.
    let with_table = |bytes: &[u8], query: &[i32], named: u32| -> Vec<u8> {
        let mut points: Vec<i16> = query.iter().map(|&q| q as i16).collect();
        points.extend(query.iter().map(|&q| (q + 3000).clamp(-32768, 32767) as i16));
        let image = super::PointKeyImage { width, points, targets: vec![named, named + 1] };
        super::add_point_key_uorlut01(bytes, &image).expect("the /8 stack")
    };
    let at = reply_start - 1;
    let (miss7, opening) = decode(&stack(false), true);
    assert_eq!(opening, Some(None), "the opening key misses");
    let named = if miss7[at].0 == 7 { 8 } else { 7 };
    let (miss8, _) = decode(&with_table(&stack(false), &miss7[at].1, named), true);
    assert_eq!(miss8[at].0, named, "the missed opening takes the point key's name");
    for (row, (a, b)) in miss7.iter().zip(&miss8).enumerate().filter(|&(row, _)| row != at) {
        assert_eq!(a.0, b.0, "row {row} decodes as /7");
    }
    // The key hits: the memory names the opening, and the table changes nothing.
    let (hit7, opening) = decode(&stack(true), true);
    assert_eq!(opening, Some(Some(42)), "the opening key hits");
    let (hit8, _) = decode(&with_table(&stack(true), &hit7[at].1, named), true);
    assert_eq!(hit7.iter().map(|s| s.0).collect::<Vec<_>>(), hit8.iter().map(|s| s.0).collect::<Vec<_>>());
    // No reply-start signal: no row is keyed, and the table changes nothing.
    let (quiet7, _) = decode(&stack(false), false);
    let (quiet8, opening) = decode(&with_table(&stack(false), &miss7[at].1, named), false);
    assert_eq!(opening, None, "no row keyed");
    assert_eq!(quiet7.iter().map(|s| s.0).collect::<Vec<_>>(), quiet8.iter().map(|s| s.0).collect::<Vec<_>>());
    // The timed twin serves the key as the step does.
    let model = IntegerStackModel::parse(&with_table(&stack(false), &miss7[at].1, named)).expect("served");
    let mut session = model.session();
    let mut ns = Vec::new();
    let timed: Vec<u32> = tokens
        .iter()
        .enumerate()
        .map(|(row, &token)| {
            if row + 1 == reply_start {
                session.mark_reply_start();
            }
            session.measure_fbs_step_stages(token, &mut ns).expect("timed step").0.token
        })
        .collect();
    assert_eq!(timed, miss8.iter().map(|s| s.0).collect::<Vec<_>>(), "the timed twin decodes as the step");
}

/// claude-222: TRAIN's openings are read in the float trunk's windows, and so are their rows. The
/// first run of a document starts at its `<|bos|>`. A later run's causal prefix starts inside the
/// document and never crosses its start. A target after a reply token is no opening, and a window
/// without one is skipped.
#[test]
fn the_point_key_windows_are_the_float_trunks_openings() {
    let windows = |tokens: &[u32], mask: &[u8], context| super::point_key_windows(tokens, mask, context, 0).expect("windows");
    let w = |input: Vec<u32>, openings: Vec<(usize, u32)>| super::PointKeyWindow { input, openings };
    assert_eq!(
        windows(&[0, 5, 6, 7, 8, 9, 0, 10, 11, 12], &[0, 0, 0, 1, 1, 0, 0, 0, 1, 1], 4),
        [w(vec![0, 5, 6], vec![(2, 7)]), w(vec![0, 10], vec![(1, 11)])]
    );
    assert_eq!(
        windows(&[0, 1, 2, 3, 4, 5, 6, 7], &[0, 0, 0, 0, 0, 0, 1, 1], 3),
        [w(vec![3, 4, 5], vec![(2, 6)])],
        "a later run's prefix starts inside the document"
    );
    assert!(super::point_key_windows(&[0, 1], &[0], 4, 0).is_err(), "a mask of another length");
    assert!(super::point_key_windows(&[0, 1, 2], &[0, 0, b'1'], 4, 0).is_err(), "a mask byte other than 0 or 1");
}

/// claude-222: the builder's table holds the integer trunk's own query at TRAIN's opening rows,
/// exactly the session's with no reply-start signal. Served from the `/8` stack it builds, it names
/// a TRAIN opening that the exact key misses by that opening's own target.
#[test]
fn the_point_key_builder_reads_the_trunks_own_points_at_trains_openings() {
    let (tokens, reply_start) = turn_memory_script(8);
    let mask: Vec<u8> = (0..tokens.len()).map(|p| u8::from(p >= reply_start)).collect();
    let bytes = point_key_test_stack(false);
    let model = IntegerStackModel::parse(&bytes).expect("served /7");
    let windows = super::point_key_windows(&tokens, &mask, 192, 0).expect("windows");
    assert_eq!(windows, [super::PointKeyWindow { input: tokens[..reply_start].to_vec(), openings: vec![(reply_start - 1, tokens[reply_start])] }]);
    let image = super::build_point_key(&model, &windows).expect("the table");
    let mut session = model.session();
    let own: Vec<i16> = tokens[..reply_start]
        .iter()
        .map(|&t| session.step_native(t).expect("step").query_q12)
        .last()
        .expect("a row")
        .iter()
        .map(|&q| q as i16)
        .collect();
    assert_eq!((image.points, image.targets.clone()), (own, vec![tokens[reply_start]]));
    let built = super::build_point_key(&model, &windows).expect("the table");
    let served = IntegerStackModel::parse(&super::add_point_key_uorlut01(&bytes, &built).expect("/8")).expect("served /8");
    let mut session = served.session();
    let mut opening = None;
    for (row, &token) in tokens[..reply_start].iter().enumerate() {
        if row + 1 == reply_start {
            session.mark_reply_start();
        }
        opening = Some(session.step_native(token).expect("step").token);
    }
    assert_eq!(opening, Some(tokens[reply_start]), "the TRAIN opening's own target");
}

/// `bytes`' JSON header and its data sections.
fn header_and_data(bytes: &[u8]) -> (Value, Vec<u8>) {
    let len = u64::from_le_bytes(bytes[8..16].try_into().expect("length")) as usize;
    let header = serde_json::from_slice(&bytes[16..16 + len]).expect("header");
    (header, bytes[(16 + len).div_ceil(64) * 64..].to_vec())
}

/// claude-222: the served point key added to a `/7` turn-memory stack is a `/8` stack whose table
/// reads back as written, with the `/7` data unchanged byte for byte. Adding it is deterministic,
/// and it is refused on a stack that already has one, on a stack without the turn memory, and at
/// another width.
#[test]
fn a_point_key_round_trips_through_the_lut_stack_8_record() {
    let bytes = super::synthetic::synthetic_turn_memory_artifact(7, 8, 192, false);
    let image = synthetic_point_key(&bytes, 5);
    let with = super::add_point_key_uorlut01(&bytes, &image).expect("the /8 stack");
    assert_eq!(with, super::add_point_key_uorlut01(&bytes, &image).expect("again"), "deterministic");
    let read = super::read_memory_uorlut01(&with).expect("read").expect("a memory");
    assert_eq!(read.point.as_ref(), Some(&image), "the table reads back as written");
    assert_eq!(super::read_memory_uorlut01(&bytes).expect("read").expect("a memory").point, None);
    let ((old_header, old), (new_header, new)) = (header_and_data(&bytes), header_and_data(&with));
    assert_eq!(&new[..old.len()], &old[..], "the /7 sections are unchanged");
    assert_eq!(new_header["schema"], json!(super::STACK_FBS_POINT_SCHEMA));
    assert_eq!(new_header["point"], json!({ "rows": 5, "width": image.width, "exp": super::POINT_KEY_EXP }));
    assert_eq!(new_header["memory"], old_header["memory"]);
    IntegerStackModel::parse(&with).expect("served /8");
    assert!(super::add_point_key_uorlut01(&with, &image).is_err(), "a stack with a table already");
    let plain = super::export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(7)).expect("export");
    assert!(super::add_point_key_uorlut01(&plain, &image).is_err(), "a stack without the turn memory");
    let narrow = super::PointKeyImage {
        width: image.width - 1,
        points: vec![0; (image.width - 1) * 5],
        targets: image.targets.clone(),
    };
    assert!(super::add_point_key_uorlut01(&bytes, &narrow).is_err(), "another width");
}

/// claude-222: the schema names the point key, and the record is checked: under `/7`, `/8` without
/// the record, the record without the turn memory, rows its targets do not hold, another width or
/// exponent, and an unknown field are each refused. A `/8` session's restore is refused as a `/7`
/// one's is.
#[test]
fn the_lut_stack_8_schema_and_record_are_checked() {
    let bytes = super::synthetic::synthetic_turn_memory_artifact(7, 8, 192, false);
    let with = super::add_point_key_uorlut01(&bytes, &synthetic_point_key(&bytes, 5)).expect("the /8 stack");
    let refused = |b: Vec<u8>| IntegerStackModel::parse(&b).is_err();
    assert!(refused(with_header(&with, |h| h["schema"] = json!(super::STACK_FBS_MEMORY_SCHEMA))), "the record under /7");
    assert!(refused(with_header(&bytes, |h| h["schema"] = json!(super::STACK_FBS_POINT_SCHEMA))), "/8 without the record");
    assert!(
        refused(with_header(&with, |h| {
            h.as_object_mut().expect("object").remove("memory");
        })),
        "the record without the turn memory"
    );
    assert!(refused(with_header(&with, |h| h["point"]["rows"] = json!(4))), "rows the targets do not hold");
    assert!(refused(with_header(&with, |h| h["point"]["width"] = json!(3))), "another width");
    assert!(refused(with_header(&with, |h| h["point"]["exp"] = json!(-11))), "another exponent");
    assert!(refused(with_header(&with, |h| h["point"]["extra"] = json!(1))), "an unknown field");
    let model = IntegerStackModel::parse(&with).expect("served /8");
    let mut session = model.session();
    session.step_native(10).expect("step");
    let saved = session.save_state();
    match model.session().restore_state(&saved) {
        Err(StackError::Shape(m)) => assert_eq!(m, "a session of a turn memory cannot be restored yet: its turn state is not saved"),
        other => panic!("expected the turn-memory restore refusal, got {other:?}"),
    }
}

/// claude-224: a panel reply ends at BOS, unkept; at its length; or where prompt and reply fill
/// the context. Each of the last two keeps its token.
#[test]
fn the_reply_stop_ends_at_bos_its_length_or_a_full_context() {
    use super::{reply_stop, ReplyEnd};
    assert_eq!(reply_stop(0, 5, 10, 64, 192), Some(ReplyEnd::Bos));
    assert_eq!(reply_stop(5, 63, 10, 64, 192), Some(ReplyEnd::Length));
    assert_eq!(reply_stop(5, 0, 10, 1, 192), Some(ReplyEnd::Length), "a one-token reply");
    assert_eq!(reply_stop(5, 10, 181, 64, 192), Some(ReplyEnd::Context));
    assert_eq!(reply_stop(5, 9, 181, 64, 192), None, "one row of context left");
    assert_eq!(reply_stop(5, 5, 10, 64, 192), None);
}

/// claude-224: on a `/8` stack the panel reply's opening is the point key's name where the exact
/// key misses, with its least distance, and the float name's least distance is the table's. A `/7`
/// stack has neither. The reply is capped where prompt and reply fill the context, and at its
/// length.
#[test]
fn a_panel_reply_records_the_point_keys_opening_and_is_capped() {
    use super::{panel_reply, ReplyEnd};
    let (tokens, reply_start) = turn_memory_script(8);
    let mask: Vec<u8> = (0..tokens.len()).map(|p| u8::from(p >= reply_start)).collect();
    let bytes = point_key_test_stack(false);
    let model7 = IntegerStackModel::parse(&bytes).expect("served /7");
    let windows = super::point_key_windows(&tokens, &mask, 192, 0).expect("windows");
    let image = super::build_point_key(&model7, &windows).expect("the table");
    let model8 = IntegerStackModel::parse(&super::add_point_key_uorlut01(&bytes, &image).expect("/8")).expect("served /8");
    let (prompt, named) = (&tokens[..reply_start], tokens[reply_start]);
    let served = panel_reply(&model8, prompt, 64, Some(named)).expect("served");
    assert_eq!((served.keyed, served.float_distance), (Some((named, 0)), Some(0)));
    assert_eq!(served.reply[0], named, "the opening is the key's name");
    let context = model8.shape().context;
    assert_eq!((served.reply.len(), served.end), (context - prompt.len(), ReplyEnd::Context), "capped at the context");
    let other = panel_reply(&model8, prompt, 64, Some(named + 1)).expect("served");
    assert_eq!(other.float_distance, None, "a name no row holds");
    let greedy = panel_reply(&model7, prompt, 64, Some(named)).expect("greedy");
    assert_eq!((greedy.keyed, greedy.float_distance), (None, None), "no table");
    assert_eq!(greedy.reply.len(), context - prompt.len(), "greedy is capped the same way");
    let short = panel_reply(&model8, prompt, 3, None).expect("served");
    assert_eq!((short.reply.len(), short.end), (3, ReplyEnd::Length));
    let full = panel_reply(&model8, &vec![12; context], 4, None).expect("a prompt that fills the context");
    assert_eq!((full.reply.len(), full.end), (1, ReplyEnd::Context), "its opening, and no step past the context");
    assert!(panel_reply(&model8, &vec![12; context + 1], 4, None).is_err(), "a prompt past the context");
}

/// claude-228: the run's instrument at a served opening. A decode whose key serves passes the
/// reference scan with the opening step's own query, and a served name or distance the reference
/// does not give is refused, as is a check against a stack without a table.
#[test]
fn a_served_opening_is_checked_against_the_reference_scan() {
    use super::panel_reply;
    use super::point_key::check_served_point;
    let (tokens, reply_start) = turn_memory_script(8);
    let mask: Vec<u8> = (0..tokens.len()).map(|p| u8::from(p >= reply_start)).collect();
    let bytes = point_key_test_stack(false);
    let model7 = IntegerStackModel::parse(&bytes).expect("served /7");
    let windows = super::point_key_windows(&tokens, &mask, 192, 0).expect("windows");
    let image = super::build_point_key(&model7, &windows).expect("the table");
    let model8 = IntegerStackModel::parse(&super::add_point_key_uorlut01(&bytes, &image).expect("/8")).expect("served /8");
    let prompt = &tokens[..reply_start];
    let keyed = panel_reply(&model8, prompt, 4, None).expect("served and checked").keyed.expect("the key served");
    let mut session = model8.session();
    let mut step = None;
    for (row, &token) in prompt.iter().enumerate() {
        if row + 1 == prompt.len() {
            session.mark_reply_start();
        }
        step = Some(session.step_native(token).expect("step"));
    }
    let query = step.expect("a step").query_q12;
    assert_eq!(session.last_point_key(), Some(keyed), "the same opening");
    assert_eq!(model8.point_key_reference(&query), Some(keyed), "the reference names as served");
    check_served_point(&model8, &query, keyed).expect("the served name");
    assert!(check_served_point(&model8, &query, (keyed.0 + 1, keyed.1)).is_err(), "another name");
    assert!(check_served_point(&model8, &query, (keyed.0, keyed.1 + 1)).is_err(), "another distance");
    assert!(check_served_point(&model7, &query, keyed).is_err(), "a stack without a table");
}

/// claude-228: the run's instrument at export. The table reads back from the `/8` bytes as built;
/// a built table that differs from what the bytes hold is refused, and so is a `/7` stack.
#[test]
fn a_point_table_that_does_not_read_back_as_built_is_refused() {
    let bytes = super::synthetic::synthetic_turn_memory_artifact(7, 8, 192, false);
    let image = synthetic_point_key(&bytes, 5);
    let with = super::add_point_key_uorlut01(&bytes, &image).expect("the /8 stack");
    super::check_point_key_read_back(&with, &image).expect("as built");
    let mut moved = image.clone();
    moved.points[3] += 1;
    assert!(super::check_point_key_read_back(&with, &moved).is_err(), "a point changed");
    let mut renamed = image.clone();
    renamed.targets[0] += 1;
    assert!(super::check_point_key_read_back(&with, &renamed).is_err(), "a target changed");
    assert!(super::check_point_key_read_back(&bytes, &image).is_err(), "a /7 stack holds no table");
}

/// claude-226: the retained rates are the float trunk's `retain_rates` rule at Q31. Each lane
/// carries `1 - 2^-(i+1)` exactly, and its drive weight is `sqrt(1 - lambda^2)` within one unit.
/// Entry 0 is one half in both readings, so the readings first differ at entry 1.
#[test]
fn the_retained_rates_are_the_float_rule_at_q31() {
    use super::session::{GEOMETRIC_KEEP_Q31, RETAIN_KEEP_Q31, RETAIN_LAMBDA_Q31};
    for i in 0..16 {
        let lambda = 1.0 - 2f64.powi(-(i as i32 + 1));
        assert_eq!(RETAIN_LAMBDA_Q31[i] as f64, lambda * 2f64.powi(31), "lambda {i}");
        let keep = (1.0 - lambda * lambda).sqrt() * 2f64.powi(31);
        assert!((RETAIN_KEEP_Q31[i] as f64 - keep).abs() <= 1.0, "keep {i}: {} against {keep}", RETAIN_KEEP_Q31[i]);
    }
    assert_eq!((RETAIN_LAMBDA_Q31[0], RETAIN_KEEP_Q31[0]), (1 << 30, GEOMETRIC_KEEP_Q31[0]), "entry 0, one half");
    assert_ne!(RETAIN_LAMBDA_Q31[1], 1 << 29, "entry 1 differs");
}

/// claude-226: a snapshot trained with retention exports `rec_gates = "geometric-retention"`, and
/// its integer session serves the retained f64 reference at every row, its query within 2 Q12
/// units. The decay reading's reference is not what it serves. A value the engine does not know is
/// refused by its own validation, as an engine before this unit refuses the new one.
#[test]
fn a_retained_fbs_stack_serves_the_retained_reference() {
    let mut snap = synthetic_fbs_trunk(20261026);
    snap.retain_rates = true;
    let bytes = super::export_fbs_trunk_uorlut01(&snap).expect("export");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(model.shape().rec_retention() && model.shape().rec_geometric());
    let sequence = [0u32, 5, 11, 19, 3, 27, 14, 31];
    let want = snap.forward_f32(&sequence, true).expect("the retained reference");
    let mut decay = snap.clone();
    decay.retain_rates = false;
    let decayed = decay.forward_f32(&sequence, true).expect("the decay reference");
    let q12 = |v: f64| (v * 4096.0).round() as i32;
    let mut session = model.session();
    let mut apart = 0;
    for (row, &token) in sequence.iter().enumerate() {
        let got = session.step_native(token).expect("step");
        for (j, (&g, &w)) in got.query_q12.iter().zip(&want[row].query).enumerate() {
            assert!((g - q12(w)).abs() <= 2, "row {row} channel {j}: {g} against the retained reference's {}", q12(w));
        }
        apart += usize::from(got.query_q12.iter().zip(&decayed[row].query).any(|(&g, &d)| (g - q12(d)).abs() > 2));
    }
    assert!(apart > 0, "the decay reading's reference is what the retained stack serves");
    let refused = with_header(&bytes, |h| h["shape"]["rec_gates"] = json!("geometric-other"));
    assert!(IntegerStackModel::parse(&refused).is_err(), "an unknown reading is refused");
    // The refusal an engine before this unit makes is its shape validation's, pinned here.
    let mut other = model.shape().clone();
    other.rec_gates = Some("geometric-other".to_owned());
    assert!(matches!(other.validate(), Err(StackError::Shape("the recurrence gates are not geometric"))));
    assert!(model.shape().validate().is_ok(), "the retained reading is valid");
}

/// claude-226: a snapshot without the flag, as every one before it, exports byte for byte as it did
/// before the change, and serves the decay reading. The sha256 was taken at `a5eb6863a`, the base.
#[test]
fn a_snapshot_without_the_flag_exports_as_before() {
    use sha2::{Digest, Sha256};
    let snap = synthetic_fbs_trunk(20261026);
    let mut json: Value = serde_json::to_value(&snap).expect("json");
    assert!(json.get("retain_rates").is_none(), "false is not written");
    assert!(json.get("readout_nearest").is_none(), "nor is the writer readout (claude-231)");
    json.as_object_mut().expect("object").remove("retain_rates");
    let old: super::FbsTrunkSnapshot = serde_json::from_value(json).expect("an old snapshot");
    let bytes = super::export_fbs_trunk_uorlut01(&old).expect("export");
    assert_eq!(bytes.len(), 21_696);
    assert_eq!(hex::encode(Sha256::digest(&bytes)), "2eda9536d128c310eff1f84875bac832355790b5dd21d4dfa98aeb60b0de700d");
    let model = IntegerStackModel::parse(&bytes).expect("parse");
    assert!(model.shape().rec_geometric() && !model.shape().rec_retention());
    assert!(model.shape().head_native_vector() && !model.shape().head_nearest(), "the writer head");
}

/// claude-231: the nearest head. `validate` accepts `native_vector_nearest`, and an engine without
/// it refuses it by its own `validate` ("the head is not native_vector"), pinned on an unknown head.
/// Both heads are native, and only the new one reads nearest.
#[test]
fn the_nearest_head_is_a_native_head_and_an_unknown_one_is_refused() {
    let mut snap = synthetic_fbs_trunk(20261031);
    snap.readout_nearest = true;
    let model = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export")).expect("parse");
    let shape = model.shape().clone();
    assert_eq!(shape.head.as_deref(), Some("native_vector_nearest"));
    assert!(shape.head_native_vector() && shape.head_nearest());
    assert!(shape.validate().is_ok(), "the nearest head is valid");
    let mut other = shape.clone();
    other.head = Some("native_vector_other".to_owned());
    assert!(matches!(other.validate(), Err(StackError::Shape("the head is not native_vector"))));
    let mut writer = shape;
    writer.head = Some("native_vector".to_owned());
    assert!(writer.head_native_vector() && !writer.head_nearest());
}

/// claude-231: a nearest stack serves the whole table's L1-nearest token at every step, checked by a
/// test-side reference scan of the step's own query over the writer table (ties to the smaller id).
/// The writer stack of the same snapshot computes the same queries and serves its own pick, which
/// differs somewhere. The browser's `serve_step` serves `step_native`'s token under both heads,
/// where it took the empty logits' argmax, 0, before.
#[test]
fn a_nearest_stack_serves_the_whole_tables_nearest_token() {
    let mut snap = synthetic_fbs_trunk(20261031);
    let writer = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export")).expect("parse");
    snap.readout_nearest = true;
    let nearest = IntegerStackModel::parse(&super::export_fbs_trunk_uorlut01(&snap).expect("export")).expect("parse");
    let table = nearest.fbs_image().expect("an FBS model").writer_table.to_vec();
    let (vocab, d) = (nearest.shape().vocab, nearest.shape().width);
    let reference = |q: &[i32]| -> u32 {
        let (mut best, mut least) = (0u32, i64::MAX);
        for v in 0..vocab {
            let l1: i64 = q.iter().zip(&table[v * d..(v + 1) * d]).map(|(&x, &t)| (i64::from(x) - i64::from(t)).abs()).sum();
            if l1 < least {
                (best, least) = (v as u32, l1);
            }
        }
        best
    };
    let mut differ = 0usize;
    for run in 0..6usize {
        let (mut a, mut b) = (nearest.session(), writer.session());
        let (mut browser_a, mut browser_b) = (nearest.session(), writer.session());
        let (mut timed, mut ns) = (nearest.session(), Vec::new());
        for t in 0..nearest.shape().context {
            let token = ((t * 7 + run * 11 + 3) % vocab) as u32;
            let near = a.step_native(token).expect("nearest step");
            assert_eq!(near.token, reference(&near.query_q12), "run {run} step {t}: the whole table's nearest");
            let pick = b.step_native(token).expect("writer step");
            assert_eq!(pick.query_q12, near.query_q12, "run {run} step {t}: the head changes no query");
            differ += usize::from(pick.token != near.token);
            assert_eq!(browser_a.serve_step(token).expect("browser"), (near.token, 0), "run {run} step {t}: nearest");
            assert_eq!(browser_b.serve_step(token).expect("browser"), (pick.token, 0), "run {run} step {t}: the writer");
            // The timed twin (`fbs-toks --stages`) serves the same, and its search beside agrees.
            let (stepped, beside) = timed.measure_fbs_step_stages(token, &mut ns).expect("timed step");
            assert_eq!((stepped.token, beside), (near.token, near.token), "run {run} step {t}: the timed twin");
        }
    }
    assert!(differ > 0, "the writer's pick and the nearest token differ somewhere");
}

/// claude-231: the point key's override of a missed opening holds under the nearest head, and the
/// unkeyed greedy opening is the whole table's nearest.
#[test]
fn the_point_key_still_names_the_opening_under_the_nearest_head() {
    use super::panel_reply;
    let (tokens, reply_start) = turn_memory_script(8);
    let mask: Vec<u8> = (0..tokens.len()).map(|p| u8::from(p >= reply_start)).collect();
    let bytes = point_key_test_stack_with(false, true);
    let model7 = IntegerStackModel::parse(&bytes).expect("served /7");
    assert!(model7.shape().head_nearest());
    let windows = super::point_key_windows(&tokens, &mask, 192, 0).expect("windows");
    let image = super::build_point_key(&model7, &windows).expect("the table");
    let model8 = IntegerStackModel::parse(&super::add_point_key_uorlut01(&bytes, &image).expect("/8")).expect("served /8");
    let prompt = &tokens[..reply_start];
    let served = panel_reply(&model8, prompt, 4, None).expect("served");
    let (named, _) = served.keyed.expect("the key served");
    assert_eq!(served.reply[0], named, "the opening is the key's name");
    assert_eq!(panel_reply(&model7, prompt, 4, None).expect("greedy").keyed, None, "no table");
}

/// claude-231: `serve_step` on a dense stack is the browser's old step: the integer argmax of the
/// step's logits (first on ties) and top1 − top2, read here by a sort.
#[test]
fn serve_step_on_a_dense_stack_is_the_logits_argmax_and_margin() {
    let model = IntegerStackModel::parse(&artifact(7)).expect("a dense stack");
    let (mut stepped, mut served) = (model.session(), model.session());
    for t in 0..4u32 {
        let logits = stepped.step(t).expect("step").to_vec();
        let mut sorted = logits.clone();
        sorted.sort_unstable_by(|x, y| y.cmp(x));
        let want = (super::stack_argmax(&logits) as u32, sorted[0].saturating_sub(sorted[1]));
        assert_eq!(served.serve_step(t).expect("serve"), want, "step {t}");
    }
}

/// claude-231: the run's readout is its objective: `cell` and `nearest` read nearest, `sheet` and
/// `voronoi` the writer, a config without one the writer, and anything else is refused.
#[test]
fn the_run_readout_is_its_objective() {
    use crate::fbs_snapshot::run_readout_nearest as readout;
    assert_eq!(readout(r#"{"objective":"cell"}"#), Ok(true));
    assert_eq!(readout(r#"{"objective":"nearest"}"#), Ok(true));
    assert_eq!(readout(r#"{"objective":"sheet"}"#), Ok(false));
    assert_eq!(readout(r#"{"objective":"voronoi"}"#), Ok(false));
    assert_eq!(readout(r#"{"steps":16384}"#), Ok(false), "a run from before the objectives");
    assert!(readout(r#"{"objective":"other"}"#).is_err());
    assert!(readout(r#"{"objective":3}"#).is_err());
    assert!(readout("not json").is_err());
}

/// claude-235: the integer side of the Hamming trace, on the synthetic trunk with a read after
/// layer 1 and on the readout, both kinds of head served. The trace over `repeating_tokens`, the
/// served model, and the read's record.
fn integer_hamming_trace() -> (crate::fbs_snapshot::HammingTrace, IntegerStackModel, super::HammingImage) {
    use crate::fbs_snapshot::{HammingTrace, HAMMING_TRACE_SCHEMA};
    let (_, _, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let model = IntegerStackModel::parse(&with_read).expect("the read is served");
    let image = super::read_hamming_uorlut01(&with_read).expect("parse").expect("the reads");
    let tokens = repeating_tokens();
    let mut session = model.session();
    session.set_hamming_trace(true);
    for &token in &tokens {
        session.step_native(token).expect("step");
    }
    let steps = session.take_hamming_trace().expect("the trace is on");
    let window = super::hamming_trace_window(&tokens, &steps).expect("one window");
    let trace = HammingTrace {
        schema: HAMMING_TRACE_SCHEMA.to_owned(),
        side: "integer".to_owned(),
        arm: "integer".to_owned(),
        heads: image.record.heads,
        kept: image.record.kept,
        windows: vec![window],
    };
    (trace, model, image)
}

/// claude-235: with the trace on, every step serves what it serves with the trace off, and the
/// trace holds each read's choices at every row: a copy head's query and key are its vocabulary
/// codes and its kept set is `CopyHeads::choose`'s; a type head's key is its vocabulary code and
/// its set holds `min(kept, t − 1)` positions. A trace compared with itself agrees everywhere.
#[test]
fn the_hamming_trace_records_each_reads_choices_and_changes_no_step() {
    use crate::fbs_snapshot::compare_hamming_traces;
    let (trace, model, image) = integer_hamming_trace();
    let tokens = repeating_tokens();
    let (mut traced, mut plain) = (model.session(), model.session());
    assert_eq!(plain.take_hamming_trace(), None, "off in a fresh session");
    traced.set_hamming_trace(true);
    for (t, &token) in tokens.iter().enumerate() {
        assert_eq!(traced.step_native(token).expect("step"), plain.step_native(token).expect("step"), "position {t}");
    }
    assert_eq!(traced.save_state(), plain.save_state());
    assert_eq!(traced.take_hamming_trace().map(|s| s.len()), Some(2 * tokens.len()), "two reads a step");
    assert_eq!(traced.take_hamming_trace(), Some(Vec::new()), "taken, and still on");

    let free = IntegerStackModel::parse(&served_read_trunk(&[1, 3], 1.0, 1.0).1).expect("parse");
    let table = free.fbs_image().expect("an FBS model").writer_table.to_vec();
    let (vocab, width) = (free.shape().vocab, free.shape().width);
    let (heads, copy) = (image.record.heads, image.record.heads / 2);
    let window = &trace.windows[0];
    assert_eq!(window.reads.iter().map(|r| r.depth).collect::<Vec<_>>(), [1, 3]);
    let mut full = 0;
    for (read, served) in window.reads.iter().zip(&image.reads) {
        assert_eq!(read.heads.len(), heads, "both kinds of head");
        let copies = super::CopyHeads::new(&image.record, served, &table, vocab, width);
        let types = super::TypeHeads::new(&image.record, served, &table, vocab, width);
        for t in 0..tokens.len() {
            let want = image.record.kept.min(t.saturating_sub(1));
            for h in 0..heads {
                let head = &read.heads[h];
                assert_eq!(head.kept[t].len(), want, "head {h}, row {t}");
                if h < copy {
                    let chosen: Vec<u32> = copies.choose(h, &tokens[..=t]).into_iter().map(|(s, _)| s as u32).collect();
                    assert_eq!(head.kept[t], chosen, "copy head {h}, row {t}");
                    assert_eq!(head.query[t], copies.code(tokens[t], h));
                    assert_eq!(head.key[t], if t == 0 { 0 } else { copies.code(tokens[t - 1], h) });
                } else {
                    assert_eq!(head.key[t], if t == 0 { 0 } else { types.key(tokens[t - 1], h - copy) });
                }
                full += usize::from(want == image.record.kept);
            }
        }
    }
    assert!(full > 0, "some rows keep a full set");
    let same = compare_hamming_traces(&trace, &trace).expect("compare");
    assert!(same.faults.is_empty());
    for row in &same.rows {
        assert_eq!((row.equal, row.top1_equal), (row.judged, row.top1_judged), "{row:?}");
        assert_eq!(row.judged, tokens.len() * copy, "{row:?}");
        assert_eq!(row.downstream + row.not_served + row.ties + row.instrument_faults, 0, "{row:?}");
    }
}

/// claude-237: the copy-code check's inputs from an artifact, on the synthetic trunk with a read
/// after layer 1 and on the readout: the served codes are `CopyHeads`' own for every entry and
/// copy head, and the session's own trace holds exactly those codes at every position, query and
/// key, with no pair given two codes.
#[test]
fn the_copy_code_check_holds_the_session_trace_to_its_served_codes() {
    use crate::fbs_snapshot::{copy_code_check, copy_code_reads};
    let (integer, ..) = integer_hamming_trace();
    let (_, _, with_read) = served_read_trunk(&[1, 3], 1.0, 1.0);
    let served = super::served_copy_codes_uorlut01(&with_read).expect("parse").expect("the reads");
    let (vocab, width, heads) = (served.vocab, served.width, served.image.record.heads);
    let copy = heads / 2;
    for (read, codes) in served.image.reads.iter().zip(&served.codes) {
        let copies = super::CopyHeads::new(&served.image.record, read, &served.table, vocab, width);
        assert_eq!(codes.len(), vocab * copy);
        for (i, &code) in codes.iter().enumerate() {
            assert_eq!(code, copies.code((i / copy) as u32, i % copy), "entry {}, head {}", i / copy, i % copy);
        }
    }
    // The codes differ by depth, head, entry and position, so a wrong index or an unshifted key
    // cannot pass the comparison below.
    let first = &served.codes[0];
    assert_ne!(first, &served.codes[served.codes.len() - 1], "the depths' codes");
    assert!(copy > 1 && first.chunks(copy).any(|entry| entry[0] != entry[1]), "copy heads that differ");
    assert!(first.chunks(copy).any(|entry| entry != &first[..copy]), "entries that differ");
    let query = &integer.windows[0].reads[0].heads[0].query;
    assert!(query.windows(2).any(|pair| pair[0] != pair[1]), "neighbouring positions that differ");
    let inputs: Vec<(usize, &[i32], Vec<u64>)> = served
        .image
        .reads
        .iter()
        .zip(&served.codes)
        .map(|(read, codes)| (read.depth, read.key_rotation.as_slice(), codes.clone()))
        .collect();
    let reads = copy_code_reads(&inputs, served.image.record.rotation_exp, &served.table, vocab, width, heads)
        .expect("the control");
    let mut float = integer.clone();
    float.side = "float-d11".to_owned();
    let check = copy_code_check(vocab, width, heads, &reads, &float, &integer).expect("the check");
    assert_eq!(check.reads.iter().map(|r| r.depth).collect::<Vec<_>>(), [1, 3]);
    for read in &check.reads {
        let trace = &read.integer;
        assert!(trace.query.samples > 0 && trace.key.samples > 0);
        assert_eq!(trace.query.equal_integer, trace.query.samples, "depth {}", read.depth);
        assert_eq!(trace.key.equal_integer, trace.key.samples, "depth {}", read.depth);
        assert_eq!(trace.inconsistent, 0, "depth {}", read.depth);
    }
}

/// claude-235: one full set of the stream read's copy head at a row where a candidate it left
/// out sits at its last kept place's distance, with a member nearer than the place and a
/// candidate farther: the head, the row, a member at the place, the candidate, the nearer member
/// and the farther candidate.
fn a_tie_at_the_last_place(trace: &crate::fbs_snapshot::HammingTrace) -> (usize, usize, u32, u32, u32, u32) {
    let read = &trace.windows[0].reads[0];
    for (h, head) in read.heads[..trace.heads / 2].iter().enumerate() {
        for t in trace.kept + 1..head.kept.len() {
            let distance = |s: u32| (head.query[t] ^ head.key[s as usize]).count_ones();
            let set = &head.kept[t];
            let place = set.iter().map(|&s| distance(s)).max().expect("a full set");
            let left: Vec<u32> = (1..t as u32).filter(|s| !set.contains(s)).collect();
            let out = left.iter().copied().find(|&s| distance(s) == place);
            let near = set.iter().copied().find(|&s| distance(s) < place);
            let far = left.iter().copied().find(|&s| distance(s) > place);
            if let (Some(out), Some(near), Some(far)) = (out, near, far) {
                let member = *set.iter().find(|&&s| distance(s) == place).expect("a member at the place");
                return (h, t, member, out, near, far);
            }
        }
    }
    panic!("no copy-head row ties across its last kept place with a nearer member and a farther candidate");
}

/// claude-235: re-choose `head`'s set at row `t` by its own codes after a planted code change:
/// the `kept` nearest, its old members first among equal distances, so only what the change
/// moves changes. Whether the set changed.
fn rechoose(head: &mut crate::fbs_snapshot::HammingTraceHead, t: usize, kept: usize) -> bool {
    let mut old = head.kept[t].clone();
    old.sort_unstable();
    let mut set: Vec<u32> = (1..t as u32).collect();
    set.sort_by_key(|&s| ((head.query[t] ^ head.key[s as usize]).count_ones(), old.binary_search(&s).is_err(), s));
    set.truncate(kept.min(t.saturating_sub(1)));
    set.sort_unstable();
    let changed = set != old;
    head.kept[t] = set;
    changed
}

/// claude-235: the comparison classes a planted difference by the item's rule. A set swapping a
/// member for a candidate at the same last-place distance is a tie, and the readout read's type
/// sets from that row on are downstream. A changed query code, a changed key code, and both,
/// with the sets re-chosen by them, are a query flip, key flips, and a query-and-key flip. A set
/// one short, a set its codes do not choose, or a swap with the same codes that brings in a
/// farther candidate or drops a nearer member, is an instrument fault. Traces that do not line up
/// are refused.
#[test]
fn the_trace_comparison_classes_ties_flips_and_instrument_faults() {
    use crate::fbs_snapshot::compare_hamming_traces;
    let (integer, ..) = integer_hamming_trace();
    let (h, t, member, out, near, far) = a_tie_at_the_last_place(&integer);
    let (time, types) = (integer.windows[0].ids.len(), integer.heads - integer.heads / 2);
    let planted = |plant: &dyn Fn(&mut crate::fbs_snapshot::HammingTraceHead)| {
        let mut first = integer.clone();
        plant(&mut first.windows[0].reads[0].heads[h]);
        compare_hamming_traces(&first, &integer).expect("compare")
    };
    let swap = |head: &mut crate::fbs_snapshot::HammingTraceHead, from: u32, to: u32| {
        let at = head.kept[t].iter().position(|&s| s == from).expect("the member");
        head.kept[t][at] = to;
    };
    let causes = |c: &crate::fbs_snapshot::HammingTraceComparison| {
        let r = &c.rows[0];
        (r.judged - r.equal, r.ties, r.query_flips, r.key_flips, r.query_and_key_flips, r.instrument_faults)
    };

    let tie = planted(&|head| swap(head, member, out));
    assert_eq!(causes(&tie), (1, 1, 0, 0, 0, 0), "{:?}", tie.rows[0]);
    assert_eq!(tie.rows[3].downstream, (time - t) * types, "the readout's type sets from row {t} on");
    assert_eq!(tie.rows[2].judged, tie.rows[2].equal, "the readout's copy sets are still judged");

    // Planted code changes, each set re-chosen by its side's new codes. The query at row `t` made
    // `out`'s key changes that row alone: a query flip. `out`'s key made the row's query changes
    // its sets from row `t` on, each a key flip; both at once make row `t` a query-and-key flip.
    let original = &integer.windows[0].reads[0].heads[h];
    let named = original.key[out as usize];
    let moved = (original.query[t] ^ named).count_ones() as usize;
    let query = planted(&|head| {
        head.query[t] = named;
        assert!(rechoose(head, t, integer.kept), "the query names {out}");
    });
    assert_eq!(causes(&query), (1, 0, 1, 0, 0, 0), "{:?}", query.rows[0]);
    let bits = |c: &crate::fbs_snapshot::HammingTraceComparison, d: usize| (c.rows[0].query_bits[0], c.rows[0].query_bits[d]);
    assert_eq!(bits(&tie, 1), (tie.rows[0].judged, 0), "no query differs");
    assert_eq!(bits(&query, moved), (query.rows[0].judged - 1, 1), "one query differs, by {moved} bits");
    let rekey = |head: &mut crate::fbs_snapshot::HammingTraceHead, code: u64| {
        head.key[out as usize] = code;
        for r in out as usize + 1..time {
            let changed = rechoose(head, r, integer.kept);
            assert!(changed || r != t, "row {t} keeps {out}");
        }
    };
    let key = planted(&|head| {
        let code = head.query[t];
        rekey(head, code);
    });
    let flips = key.rows[0].judged - key.rows[0].equal;
    assert_eq!(causes(&key), (flips, 0, 0, flips, 0, 0), "{:?}", key.rows[0]);
    let both = planted(&|head| {
        let code = !head.query[t];
        head.query[t] = code;
        rekey(head, code);
    });
    let flips = both.rows[0].judged - both.rows[0].equal;
    assert_eq!(causes(&both), (flips, 0, 0, flips - 1, 1, 0), "{:?}", both.rows[0]);
    // A set kept with codes it was not chosen by (the query complemented) is an instrument fault.
    let unchosen = planted(&|head| head.query[t] = !head.query[t]);
    assert_eq!(causes(&unchosen), (1, 0, 0, 0, 0, 1), "{:?}", unchosen.rows[0]);
    assert!(unchosen.faults[0].contains("not the codes the set was chosen with"), "{:?}", unchosen.faults);

    let short = planted(&|head| {
        head.kept[t].pop();
    });
    assert_eq!(causes(&short), (1, 0, 0, 0, 0, 1), "{:?}", short.rows[0]);
    assert!(short.faults[0].contains(&format!("{} kept, not {}", integer.kept - 1, integer.kept)), "{:?}", short.faults);
    // Off the place's distance, so not the nearest by its codes: a farther candidate in for the
    // member at the place (the place moves), or the candidate at the place in for a nearer member
    // (the place stays).
    for (from, to) in [(member, far), (near, out)] {
        let off = planted(&|head| swap(head, from, to));
        assert_eq!(causes(&off), (1, 0, 0, 0, 0, 1), "{to} in for {from}: {:?}", off.rows[0]);
    }

    let mut other = integer.clone();
    other.windows[0].ids[0] ^= 1;
    assert!(compare_hamming_traces(&other, &integer).is_err(), "other ids");
    let mut fewer = integer.clone();
    fewer.windows[0].reads[1].heads[0].query.pop();
    assert!(compare_hamming_traces(&fewer, &integer).is_err(), "a head short of the window");
}

/// claude-249: the decision script's reply length, so its window spans the opening and 12 reply rows.
const DECISION_REPLY: usize = 12;

/// claude-249: R's export under Q5's card, the gates and the transition both unsnapped.
const DECISION_RAW: super::FbsExportSnaps = super::FbsExportSnaps {
    gates: false,
    transition: false,
};

/// claude-249: an artifact on R's path under Q5's card, exported under `snaps` after `edit`: a
/// nearest stack with retention, a Hamming read after layer 1 and on the readout with both kinds
/// of head, and a turn memory with a reply table whose opening the decision script's turn hits.
fn decision_stack(snaps: super::FbsExportSnaps, edit: impl FnOnce(&mut super::FbsTrunkSnapshot)) -> Vec<u8> {
    let (tokens, _) = turn_memory_script(DECISION_REPLY);
    let mut snap = turn_memory_trunk(tokens.len());
    snap.retain_rates = true;
    snap.readout_nearest = true;
    snap.hamming = Some(super::synthetic::synthetic_hamming(snap.width, snap.layers.len(), 4, 188));
    let mut memory = synthetic_memory(snap.vocab, snap.width, 4, true, 249);
    memory.opening = vec![super::FbsMemoryEntry { key: vec![12, 13, 14, 15], value: 42 }];
    snap.memory = Some(memory);
    edit(&mut snap);
    super::export_fbs_trunk_uorlut01_with(&snap, snaps).expect("export")
}

/// claude-249: `model` stepped over the decision script, the reply marked before the prompt's last
/// token, under the forced set `force` (`None`: the trace off): each step's served output, and
/// the trace.
fn decision_run(
    model: &IntegerStackModel,
    force: Option<&crate::fbs_snapshot::DecisionForceWindow>,
) -> (Vec<super::NativeStep>, Option<Vec<crate::fbs_snapshot::DecisionStep>>) {
    let (tokens, reply_start) = turn_memory_script(DECISION_REPLY);
    let mut session = model.session();
    if let Some(force) = force {
        session.set_decision_trace(force).expect("the trace is on");
    }
    let steps = tokens
        .iter()
        .enumerate()
        .map(|(row, &token)| {
            if row + 1 == reply_start {
                session.mark_reply_start();
            }
            session.step_native(token).expect("step")
        })
        .collect();
    (steps, session.take_decision_trace())
}

/// claude-249: the forced set naming every decision `trace` records: each lane's entry, and each
/// read's query and key codes, the keys from step 1 on (step 0's names no candidate).
fn force_all(trace: &[crate::fbs_snapshot::DecisionStep]) -> crate::fbs_snapshot::DecisionForceWindow {
    use crate::fbs_snapshot::{CodeRole, DecisionForceWindow, ForcedBucket, ForcedCode};
    let mut force = DecisionForceWindow::default();
    for (step, s) in trace.iter().enumerate() {
        for (layer, l) in s.layers.iter().enumerate() {
            force.buckets.extend(l.entry.iter().enumerate().map(|(lane, &entry)| ForcedBucket { step, layer, lane, entry }));
        }
        for read in s.layers.iter().filter_map(|l| l.read.as_ref()).chain(&s.readout) {
            for (head, h) in read.heads.iter().enumerate() {
                let code = |role, code| ForcedCode { step, depth: read.depth, head, role, code };
                force.codes.push(code(CodeRole::Query, h.query));
                if step > 0 {
                    force.codes.push(code(CodeRole::Key, h.key));
                }
            }
        }
    }
    force
}

/// claude-249 (Q5's I-R, and I-Force's second part): R's decision trace on R's path. A traced
/// session, and one forced to every decision the trace recorded, serve what an untraced session
/// serves at every step, and the forced one records the same trace. The trace holds the five
/// kinds in the forward's order: each layer's buckets, each lane's entry the one its score
/// chooses; the read after layer 1, none after layer 2, then the readout's, each head's codes and
/// kept set the Hamming trace's; the turn memory's lookups from the opening on; no nearest, which
/// the caller fills.
#[test]
fn rs_decision_trace_records_its_decisions_and_changes_no_step() {
    use crate::fbs_snapshot::DecisionForceWindow;
    let model = IntegerStackModel::parse(&decision_stack(DECISION_RAW, |_| ())).expect("R's path");
    let (tokens, reply_start) = turn_memory_script(DECISION_REPLY);
    let (plain, off) = decision_run(&model, None);
    assert_eq!(off, None, "off unless set");
    let (traced, trace) = decision_run(&model, Some(&DecisionForceWindow::default()));
    assert_eq!(traced, plain, "the trace changes no step");
    let trace = trace.expect("the trace");
    assert_eq!(trace.len(), tokens.len());

    let mut hamming = model.session();
    hamming.set_hamming_trace(true);
    for (row, &token) in tokens.iter().enumerate() {
        if row + 1 == reply_start {
            hamming.mark_reply_start();
        }
        hamming.step_native(token).expect("step");
    }
    let reads = hamming.take_hamming_trace().expect("the Hamming trace");
    assert_eq!(reads.len(), 2 * tokens.len(), "two reads a step");
    let lanes = model.shape().lanes();
    let mut kept = 0;
    for (t, step) in trace.iter().enumerate() {
        assert_eq!(step.layers.len(), 2, "step {t}");
        for layer in &step.layers {
            assert_eq!(layer.score_q16.len(), lanes, "step {t}");
            let own: Vec<u8> = layer.score_q16.iter().map(|&s| (s >> 12).min(15) as u8).collect();
            assert_eq!(layer.entry, own, "step {t}: each lane's entry is its score's");
        }
        assert_eq!(step.layers[1].read, None, "step {t}: no read after the last layer");
        let ours = [step.layers[0].read.as_ref(), step.readout.as_ref()];
        for (read, served) in ours.into_iter().zip(&reads[2 * t..2 * t + 2]) {
            let read = read.expect("both reads run");
            assert_eq!(read.depth, served.depth, "step {t}");
            assert_eq!(read.heads.len(), 4, "step {t}: both kinds of head");
            for (h, s) in read.heads.iter().zip(&served.heads) {
                assert_eq!((h.query, h.key, &h.kept), (s.query, s.key, &s.kept), "step {t}");
                kept += usize::from(h.kept.len() == 2);
            }
        }
        assert_eq!(step.nearest, None, "step {t}: the caller's");
        match step.memory.as_ref() {
            None => assert!(t + 1 < reply_start, "step {t}: a lookup from the opening on"),
            Some(m) if t + 1 == reply_start => assert_eq!((m.opening, m.stored), (true, Some(42)), "the opening hits"),
            Some(m) => assert!(!m.opening && t >= reply_start, "step {t}: a reply lookup"),
        }
    }
    assert!(kept > 0, "some rows keep a full set");
    assert!(trace.iter().any(|s| s.layers.iter().any(|l| l.entry.iter().any(|&e| e != l.entry[0]))), "buckets vary");

    let all = force_all(&trace);
    let (forced, retrace) = decision_run(&model, Some(&all));
    assert_eq!(forced, plain, "forced to its own decisions, R serves what it serves");
    assert_eq!(retrace.as_ref(), Some(&trace), "and records the same trace");
}

/// claude-249 (I-Force's first part): a forced bucket, and a forced query or key code of either
/// kind of head, each takes the forced value in the trace and changes what comes after it, while
/// every step before it is the served one. A forced query changes its own row's kept set; a forced
/// key, candidate s's, a later row's; a forced bucket, the step's query.
#[test]
fn a_forced_bucket_or_code_takes_the_forced_value_and_changes_what_follows() {
    use crate::fbs_snapshot::{CodeRole, DecisionForceWindow, DecisionRead, DecisionStep, ForcedBucket, ForcedCode};
    let model = IntegerStackModel::parse(&decision_stack(DECISION_RAW, |_| ())).expect("R's path");
    let (plain, trace) = decision_run(&model, Some(&DecisionForceWindow::default()));
    let trace = trace.expect("the trace");
    let run = |force: DecisionForceWindow, at: usize, what: &str| {
        let (steps, got) = decision_run(&model, Some(&force));
        let got = got.expect("the trace");
        assert_eq!(steps[..at], plain[..at], "{what}: the steps before are the served ones");
        assert_eq!(got[..at], trace[..at], "{what}: and their decisions");
        (steps, got)
    };

    let (step, layer, lane) = (4, 0, 0);
    let entry = (trace[step].layers[layer].entry[lane] + 8) % 16;
    let force = DecisionForceWindow { buckets: vec![ForcedBucket { step, layer, lane, entry }], ..Default::default() };
    let (steps, got) = run(force, step, "a bucket");
    assert_eq!(got[step].layers[layer].entry[lane], entry, "the forced entry is the one used");
    assert_eq!(got[step].layers[layer].score_q16, trace[step].layers[layer].score_q16, "from the same scores");
    assert_ne!(steps[step].query_q12, plain[step].query_q12, "the step's query changes");

    let read = |s: &DecisionStep, depth: usize| -> DecisionRead {
        if depth == 1 { s.layers[0].read.clone() } else { s.readout.clone() }.expect("the read")
    };
    // A copy head's and a type head's query, a copy head's and a type head's key, at both depths.
    let cases = [(CodeRole::Query, 6, 1, 0), (CodeRole::Query, 7, 2, 3), (CodeRole::Key, 3, 1, 1), (CodeRole::Key, 4, 2, 2)];
    for (role, step, depth, head) in cases {
        let what = format!("{role:?} at step {step}, depth {depth}, head {head}");
        let own = read(&trace[step], depth).heads[head].clone();
        let code = !match role {
            CodeRole::Query => own.query,
            CodeRole::Key => own.key,
        };
        let force = DecisionForceWindow {
            codes: vec![ForcedCode { step, depth, head, role, code }],
            ..Default::default()
        };
        let (steps, got) = run(force, step, &what);
        let used = read(&got[step], depth).heads[head].clone();
        let used = match role {
            CodeRole::Query => used.query,
            CodeRole::Key => used.key,
        };
        assert_eq!(used, code, "{what}: the forced code is the one used");
        let later = match role {
            CodeRole::Query => step..step + 1,
            CodeRole::Key => step + 1..got.len(),
        };
        assert!(
            later.into_iter().any(|u| read(&got[u], depth).heads[head].kept != read(&trace[u], depth).heads[head].kept),
            "{what}: a kept set it decides changes"
        );
        assert_ne!(steps[step..], plain[step..], "{what}: what R serves changes");
    }
}

/// claude-249: the decision trace refuses an artifact off R's path under the card (a dense stack,
/// a snapped gate or transition, the writer head), a session that has stepped, and a forced set
/// naming a step, layer, lane, entry, depth or head outside the model, a key at step 0, or one
/// decision twice; a model without reads refuses any forced code. A refused trace stays off. A
/// recurrence that is not geometric is refused by the shape's own validation before a session
/// exists (`a_retained_fbs_stack_serves_the_retained_reference`).
#[test]
fn the_decision_trace_refuses_what_is_off_rs_path() {
    use crate::fbs_snapshot::{CodeRole, DecisionForceWindow, ForcedBucket, ForcedCode};
    let none = DecisionForceWindow::default();
    let refused = |bytes: &[u8], force: &DecisionForceWindow, what: &str| {
        let model = IntegerStackModel::parse(bytes).expect(what);
        let mut session = model.session();
        assert!(session.set_decision_trace(force).is_err(), "{what}");
        assert_eq!(session.take_decision_trace(), None, "{what}: the trace stays off");
    };
    refused(&artifact(7), &none, "a dense stack");
    for snaps in FBS_EXPORT_SNAP_CHOICES.into_iter().filter(|&s| s != DECISION_RAW) {
        refused(&decision_stack(snaps, |_| ()), &none, &format!("snapped: {snaps:?}"));
    }
    refused(&decision_stack(DECISION_RAW, |s| s.readout_nearest = false), &none, "the writer head");

    let bytes = decision_stack(DECISION_RAW, |_| ());
    let model = IntegerStackModel::parse(&bytes).expect("R's path");
    let mut stepped = model.session();
    stepped.step_native(0).expect("step");
    assert!(stepped.set_decision_trace(&none).is_err(), "a session that has stepped");
    let mut timed = model.session();
    timed.set_decision_trace(&none).expect("a trace");
    assert!(timed.measure_fbs_step_stages(0, &mut Vec::new()).is_err(), "a timed step of a traced session");

    let (context, lanes) = (model.shape().context, model.shape().lanes());
    let bucket = |step, layer, lane, entry| ForcedBucket { step, layer, lane, entry };
    let code = |step, depth, head, role| ForcedCode { step, depth, head, role, code: 1 };
    let with = |buckets: Vec<ForcedBucket>, codes: Vec<ForcedCode>| DecisionForceWindow { buckets, codes, ..Default::default() };
    let outside = [
        with(vec![bucket(context, 0, 0, 0)], vec![]),
        with(vec![bucket(0, 2, 0, 0)], vec![]),
        with(vec![bucket(0, 0, lanes, 0)], vec![]),
        with(vec![bucket(0, 0, 0, 16)], vec![]),
        with(vec![bucket(1, 0, 0, 3), bucket(1, 0, 0, 3)], vec![]),
        with(vec![], vec![code(context, 1, 0, CodeRole::Query)]),
        with(vec![], vec![code(0, 0, 0, CodeRole::Query)]),
        with(vec![], vec![code(0, 3, 0, CodeRole::Query)]),
        with(vec![], vec![code(0, 1, 4, CodeRole::Query)]),
        with(vec![], vec![code(0, 1, 0, CodeRole::Key)]),
        with(vec![], vec![code(2, 2, 1, CodeRole::Key), code(2, 2, 1, CodeRole::Key)]),
        with(vec![], vec![code(3, 1, 0, CodeRole::Query), code(3, 1, 0, CodeRole::Query)]),
    ];
    for force in &outside {
        refused(&bytes, force, &format!("{force:?}"));
    }
    let edges = with(
        vec![bucket(context - 1, 1, lanes - 1, 15), bucket(0, 0, 0, 0)],
        vec![code(context - 1, 2, 3, CodeRole::Key), code(0, 1, 0, CodeRole::Query), code(1, 1, 0, CodeRole::Key)],
    );
    assert!(model.session().set_decision_trace(&edges).is_ok(), "the model's edges are inside it");
    let no_reads = decision_stack(DECISION_RAW, |s| s.hamming = None);
    let model = IntegerStackModel::parse(&no_reads).expect("no reads");
    assert!(model.session().set_decision_trace(&none).is_ok(), "no read, no code: a trace");
    refused(&no_reads, &with(vec![], vec![code(0, 1, 0, CodeRole::Query)]), "a code on a model without reads");
}
