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
    artifact_with(seed, &[])
}

/// [`artifact`] with extra `u32` table sections appended after the others.
fn artifact_with(seed: u64, extra: &[(&str, &[u32])]) -> Vec<u8> {
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
    for (name, values) in extra {
        b.u32s(name, values);
    }
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
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory: None,
        rec_gates: None,
        gate: None,
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
    let table = alphabet();
    let mut bytes: Vec<u8> = b"User: Assistant: System:\nHilmotwae".to_vec();
    bytes.sort_unstable();
    bytes.dedup();
    for byte in 0u8..=255 {
        if bytes.len() == VOCAB - 3 {
            break;
        }
        if !bytes.contains(&byte) {
            bytes.push(byte);
        }
    }
    bytes.sort_unstable();
    bytes.truncate(VOCAB - 3);
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

/// The shape and value ranges of a synthetic FBS trunk (claude-127: a trained
/// trunk is wider, deeper, non-square and has larger tables than the small one).
#[derive(Clone, Copy, Debug)]
struct FbsSynth {
    vocab: usize,
    d: usize,
    mlp: usize,
    layers: usize,
    context: usize,
    kept: usize,
    /// RMS gains are drawn from `gain_lo + [0, gain_span)`.
    gain_lo: f32,
    gain_span: f32,
    /// Convolution taps and biases are drawn from `±table`.
    table: f32,
    /// Lane `i`'s decay offset is `decay_step (i + ½)`.
    decay_step: f32,
}

/// claude-122's small trunk.
const FBS_SYNTH_SMALL: FbsSynth = FbsSynth {
    vocab: 256,
    d: 32,
    mlp: 32,
    layers: 2,
    context: 8,
    kept: 2,
    gain_lo: 0.75,
    gain_span: 0.5,
    table: 0.25,
    decay_step: 0.0625,
};

/// A synthetic SpiralCore/FBS trunk whose export has exactly [`fbs_shape`].
fn synthetic_fbs_trunk(seed: u64) -> super::FbsTrunkSnapshot {
    synthetic_fbs_trunk_with(seed, FBS_SYNTH_SMALL)
}

fn synthetic_fbs_trunk_with(seed: u64, s: FbsSynth) -> super::FbsTrunkSnapshot {
    let mut rng = Lcg(seed);
    let (vocab, d, mlp, context) = (s.vocab, s.d, s.mlp, s.context);
    let lanes = d / 4;
    let g = lanes + d;
    let rand_vec = |rng: &mut Lcg, len: usize, scale: f32| -> Vec<f32> {
        (0..len)
            .map(|_| ((rng.next() % 2001) as i32 - 1000) as f32 / 1000.0 * scale)
            .collect()
    };
    let rand_pos = |rng: &mut Lcg, len: usize| -> Vec<f32> {
        (0..len)
            .map(|_| s.gain_lo + (rng.next() % 1000) as f32 / 1000.0 * s.gain_span)
            .collect()
    };
    let routes = |rng: &mut Lcg, in_dim: usize, out_dim: usize| -> super::FbsRoutes {
        let (in_blocks, out_blocks) = (in_dim / 8, out_dim / 8);
        super::FbsRoutes {
            in_blocks,
            out_blocks,
            atoms: (0..out_blocks)
                .map(|_| (rng.next() % in_blocks as u64) as u16)
                .collect(),
            states: (0..out_blocks).map(|_| (rng.next() % 64) as u8).collect(),
        }
    };
    let layers = (0..s.layers)
        .map(|_| super::FbsTrunkLayer {
            rec_norm: rand_pos(&mut rng, d),
            mlp_norm: rand_pos(&mut rng, d),
            rec_in: routes(&mut rng, d, 2 * d),
            rec_gate: routes(&mut rng, d, g),
            rec_out: routes(&mut rng, d, d),
            mlp_gate: routes(&mut rng, d, mlp),
            mlp_up: routes(&mut rng, d, mlp),
            mlp_down: routes(&mut rng, mlp, d),
            conv_taps: rand_vec(&mut rng, 4 * d, s.table),
            conv_bias: rand_vec(&mut rng, d, s.table),
            gate_bias: rand_vec(&mut rng, g, s.table),
            decay: (0..lanes).map(|i| s.decay_step * (i as f32 + 0.5)).collect(),
        })
        .collect();
    super::FbsTrunkSnapshot {
        vocab,
        width: d,
        mlp,
        context,
        rms_eps: 1e-5,
        kept_cells: s.kept,
        embed: rand_vec(&mut rng, vocab * d, 1.5),
        final_norm: rand_pos(&mut rng, d),
        layers,
    }
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

#[test]
fn fbs_writer_matches_reference_when_its_address_gains_are_tiny() {
    // claude-127: a trained trunk shrank the final gains of the 16 channels
    // the writer scores its cells from (the query's first and last 8-blocks)
    // to about 6e-4, so those blocks sit at a few Q12 steps and one LSB
    // reorders the cells. The query must then reach Q12 from a norm rounded to
    // nearest at 2^-24, as unbiased as the reference's: a 2^-14 norm truncated
    // and then rounded again to Q12 chose different cells on 30% of tokens.
    let s = FbsSynth {
        vocab: 512,
        d: 64,
        mlp: 64,
        layers: 1,
        context: 8,
        kept: 8,
        ..FBS_SYNTH_SMALL
    };
    let mut snap = synthetic_fbs_trunk_with(20261026, s);
    let d = snap.width;
    for (i, g) in (0..8).chain(d - 8..d).enumerate() {
        snap.final_norm[g] = 6e-4 * (1.0 + i as f32 / 16.0);
    }
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
    let s = FbsSynth {
        vocab: 512,
        d: 64,
        mlp: 96,
        layers: 3,
        context: 16,
        kept: 8,
        gain_lo: 0.75,
        gain_span: 1.5,
        table: 0.5,
        decay_step: 0.25,
    };
    let snap = synthetic_fbs_trunk_with(20261026, s);
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
