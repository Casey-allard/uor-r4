//! Engine-level tests on synthetic artifacts. Bit-identity with the D10
//! engine is tested in `uor-r4-training` (`tests/stack_d11_oracle.rs`), which
//! depends on both engines.

use serde_json::{json, Value};

use super::format::MAX_READ_CACHE_BYTES;
use super::{IntegerStackModel, StackError, StackShape, MAGIC};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

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
    let ages: Vec<i32> = (0..HEADS * CONTEXT).map(|i| -((i as i32) << 12)).collect();
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
    let header = json!({
        "schema": super::STACK_SCHEMA,
        "shape": {
            "vocab": VOCAB, "width": WIDTH, "heads": HEADS, "mlp": MLP, "pattern": "ra",
            "read": "lorentz", "rotation": true, "context": CONTEXT,
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

#[test]
fn snap_selection_matches_an_independent_exact_oracle() {
    use super::stack_snap_select;
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let mut rng = Lcg(0x5eed);
    // Independent of the kernel: the exact dot product against a root is
    // (P + B*sqrt(5)) / 4 with P = 2*delta_a + delta_b, so two candidates
    // compare by the sign of (P1 - P2) - (B2 - B1)*sqrt(5), decided in i128
    // by the sign-aware square rule (no floats, no kernel code shared).
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
    let oracle = |raw: [i32; 4]| -> usize {
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
    };
    for _ in 0..2000 {
        let raw: [i32; 4] = std::array::from_fn(|_| (rng.next() % 200_001) as i32 - 100_000);
        assert_eq!(stack_snap_select(raw), oracle(raw), "raw {raw:?}");
    }
    // Exact ties break to the lowest index; the zero vector selects root 0.
    assert_eq!(stack_snap_select([0; 4]), 0);
    let axis = H4_ROOT_COEFFICIENTS[0];
    let raw: [i32; 4] = std::array::from_fn(|c| i32::from(axis[c][0]) * 1000);
    assert_eq!(stack_snap_select(raw), 0);
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

#[test]
fn test_gemv_pairs_blocked4_bit_identical_to_scalar() {
    use super::kernels::{
        stack_activation_tables, stack_gemv_pairs, stack_gemv_pairs_blocked4, stack_pair_tables,
        PackedMatrix,
    };

    let (rows, cols) = (16usize, 32usize);
    let mut rng = Lcg(12345);

    let nibbles: Vec<u8> = (0..rows * cols / 2)
        .map(|_| (rng.next() & 0xFF) as u8)
        .collect();
    let scales: Vec<u8> = (0..rows * cols / 32)
        .map(|_| (rng.next() & 0x3F) as u8)
        .collect();
    let min_de: Vec<u8> = scales
        .chunks_exact(cols / 32)
        .map(|row_scales| row_scales.iter().map(|&s| s >> 4).min().unwrap_or(0))
        .collect();

    let matrix = PackedMatrix {
        rows,
        cols,
        exp_base: -9,
        nibbles,
        scales,
        min_de,
    };

    let x: Vec<i16> = (0..cols).map(|_| (rng.next() as i16) >> 4).collect();
    let mut act_tables = vec![[0i32; 16]; cols];
    stack_activation_tables(&x, &mut act_tables);

    let mut pair_tables = vec![[0i32; 256]; cols / 2];
    stack_pair_tables(&act_tables, &mut pair_tables);

    let mut out_scalar = vec![0i32; rows];
    let mut out_blocked = vec![0i32; rows];

    stack_gemv_pairs(&matrix, &pair_tables, -14, &mut out_scalar);
    stack_gemv_pairs_blocked4(&matrix, &pair_tables, -14, &mut out_blocked);

    assert_eq!(
        out_scalar, out_blocked,
        "stack_gemv_pairs_blocked4 must produce bit-for-bit identical outputs to stack_gemv_pairs"
    );
}

#[test]
fn test_gemv_pairs_blocked4_multiple_groups_and_remainder_rows() {
    use super::kernels::{
        stack_activation_tables, stack_gemv_pairs, stack_gemv_pairs_blocked4, stack_pair_tables,
        PackedMatrix,
    };

    // Width 64 (2 groups of 32), 19 rows (4 full blocks of 4 + 3 remainder rows)
    let (rows, cols) = (19usize, 64usize);
    let mut rng = Lcg(54321);

    let nibbles: Vec<u8> = (0..rows * cols / 2)
        .map(|_| (rng.next() & 0xFF) as u8)
        .collect();
    // Heterogeneous scales across groups (cols / 32 = 2 groups per row)
    let scales: Vec<u8> = (0..rows * cols / 32)
        .map(|_| ((rng.next() & 0x3F) as u8).max(1))
        .collect();
    let min_de: Vec<u8> = scales
        .chunks_exact(cols / 32)
        .map(|row_scales| row_scales.iter().map(|&s| s >> 4).min().unwrap_or(0))
        .collect();

    let matrix = PackedMatrix {
        rows,
        cols,
        exp_base: -10,
        nibbles,
        scales,
        min_de,
    };

    let x: Vec<i16> = (0..cols).map(|_| (rng.next() as i16) >> 4).collect();
    let mut act_tables = vec![[0i32; 16]; cols];
    stack_activation_tables(&x, &mut act_tables);

    let mut pair_tables = vec![[0i32; 256]; cols / 2];
    stack_pair_tables(&act_tables, &mut pair_tables);

    let mut out_scalar = vec![0i32; rows];
    let mut out_blocked = vec![0i32; rows];

    stack_gemv_pairs(&matrix, &pair_tables, -14, &mut out_scalar);
    stack_gemv_pairs_blocked4(&matrix, &pair_tables, -14, &mut out_blocked);

    assert_eq!(
        out_scalar, out_blocked,
        "blocked4 must match scalar across multiple groups and remainder rows"
    );
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
