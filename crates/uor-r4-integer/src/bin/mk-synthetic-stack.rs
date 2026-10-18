//! Write a synthetic `UORLUT01` stack artifact to disk (#1172 test fixture).
//!
//! This is the engine's own test-fixture builder (`stack/tests.rs`), lifted
//! to a binary so the browser harness and the reload-identical test have a
//! sealed artifact to load. The artifact is a random-weight `ra` stack (one
//! recurrence with rotations, one Lorentz read) with monotone sealed tables;
//! it is deterministic (same seed, same bytes) and the integer engine parses
//! and serves it.
//!
//! ```text
//! mk-synthetic-stack [SEED] [OUT_PATH]
//! ```

use serde_json::{json, Value};

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
const CONTEXT: usize = 64;
const MAGIC: &[u8; 8] = b"UORLUT01";
const STACK_SCHEMA: &str = "uor-r4.lut-stack/1";

fn artifact(seed: u64) -> Vec<u8> {
    let mut rng = Lcg(seed);
    let mut b = Builder {
        matrices: Vec::new(),
        tables: Vec::new(),
        data: Vec::new(),
    };
    let lanes = WIDTH / 4;
    let gate_rows = lanes + WIDTH;
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
    let exp: Vec<u32> = (0..32 * 256 + 2)
        .map(|i: u32| (1u32 << 31) >> (i / 177).min(31))
        .collect();
    b.u32s("exp", &exp);
    let activation: Vec<i32> = (0..(2 << 12) + 1)
        .map(|i: i32| (i - 4096).max(0) << 8)
        .collect();
    b.i32s("silu", &activation);
    b.i32s("gelu", &activation);
    // The sealed arcosh table must be non-decreasing and exactly
    // `ARCOSH_TABLE_LEN` entries (`kernels::ARCOSH_TABLE_LEN` = 89089).
    let arcosh: Vec<u32> = (0..89089u32).map(|i| i).collect();
    b.u32s("arcosh", &arcosh);
    let header = json!({
        "schema": STACK_SCHEMA,
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

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let out_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "synthetic-stack.lut".to_string());
    let bytes = artifact(seed);
    std::fs::write(&out_path, &bytes).expect("write artifact");
    println!(
        "wrote {} bytes to {} (seed {})",
        bytes.len(),
        out_path,
        seed
    );
}
