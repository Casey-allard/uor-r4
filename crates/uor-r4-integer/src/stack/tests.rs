//! Engine-level tests on synthetic artifacts. Bit-identity with the D10
//! engine is tested in `uor-r4-training` (`tests/stack_d11_oracle.rs`), which
//! depends on both engines.

use serde_json::{json, Value};

use super::{IntegerStackModel, StackError, MAGIC};

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
