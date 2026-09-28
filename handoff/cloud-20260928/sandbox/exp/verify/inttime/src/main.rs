//! Scratch timing/exactness harness for the uor-r4-integer serving step.
//! Builds a RANDOM-WEIGHT artifact in the documented on-disk format (no owner
//! artifacts exist on this box), loads it through the public loader, and times
//! `IntegerModel::step` over a full 256-token session. Also runs a scratch copy
//! whose `product`/`divide` use hardware multiply/divide, and checks that both
//! produce bit-identical distributions.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use uor_r4_integer::config::QuantizedTrainingState;
use uor_r4_integer::format::{
    ParameterQuantization, QuantizationSpec, CALIBRATION_RULE, HARD_SCHEMA, SPEC_SCHEMA,
};
use uor_r4_integer::{JointConfig, ReadMode, Transport};

const ENCODING: &str = "signed4-low-nibble-first-reserved-minus8;signed16-le-reserved-minus32768";

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut s = self.0;
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        self.0 = s;
        s
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % ((hi - lo + 1) as u64)) as i64
    }
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_tables(dir: &Path) {
    uor_r4_integer::report_output::claim(dir).unwrap();
    let mut bytes = Vec::with_capacity(65535 * 16);
    let total = (1u64 << 48) as f64;
    for index in 0..65535usize {
        let x = (index as f64 - 32767.0) / 256.0;
        let sigmoid = (32768.0 / (1.0 + (-x).exp())).round() as i32;
        let tanh = (16384.0 * x.tanh()).round() as i32;
        let exp = ((-(index as f64) / 256.0).exp() * total).round() as u64;
        bytes.extend_from_slice(&sigmoid.to_le_bytes());
        bytes.extend_from_slice(&tanh.to_le_bytes());
        bytes.extend_from_slice(&exp.to_le_bytes());
    }
    fs::write(dir.join("tables.bin"), &bytes).unwrap();
    let meta = json!({"schema":"uor-r4.joint-integer-tables/1","entries":65535,"probability_bits":48,
        "payload_sha256":sha(&bytes)});
    fs::write(dir.join("tables.json"), serde_json::to_vec_pretty(&meta).unwrap()).unwrap();
    uor_r4_integer::report_output::seal(dir).unwrap();
}

/// Plausible exponents for a trained model of this shape (Glorot-scale rows).
fn exponent(name: &str) -> i16 {
    match name {
        "embedding.weight" => -8,
        "recurrent.input.weight" | "recurrent.state.weight" => -7,
        "read.query.weight" | "read.key.weight" | "read.no_read.weight" => -6,
        "read.value.weight" | "update.weight" => -7,
        "update.gate.weight" | "copy.gate.weight" => -7,
        "output.norm.weight" => -2,
        "read.age" => -13,
        "recurrent.bias" => -12,
        _ => -12, // other biases
    }
}

fn write_model(dir: &Path, transport: Transport, seed: u64) {
    fs::create_dir_all(dir).unwrap();
    let cfg = JointConfig {
        vocab_size: 4096,
        width: 256,
        read_width: 64,
        context: 256,
        transport,
        seed: 0,
    };
    let mut rng = Rng(seed | 1);
    let shapes = cfg.shapes();
    let mut spec_params = BTreeMap::new();
    let mut codes: BTreeMap<String, Vec<i32>> = BTreeMap::new();
    for (name, shape) in &shapes {
        let bits: u8 = if name.ends_with(".weight") { 4 } else { 16 };
        let rows = if shape.len() == 2 { shape[0] } else { 1 };
        let count: usize = shape.iter().product();
        let e = exponent(name);
        let values: Vec<i32> = (0..count)
            .map(|i| {
                if bits == 4 {
                    if name == "output.norm.weight" {
                        rng.range(3, 5) as i32
                    } else {
                        rng.range(-7, 7) as i32
                    }
                } else if name == "read.age" {
                    -((i as i32 + 1) * 128)
                } else if name == "recurrent.bias" {
                    if (256..512).contains(&i) {
                        // z-gate: logit(1/tau), tau log-spaced 4..64 per 4-lane
                        let lane = (i - 256) / 4;
                        let tau = 4.0 * 16f64.powf(lane as f64 / 63.0);
                        let z = 1.0 / tau;
                        ((z / (1.0 - z)).ln() * 4096.0).round() as i32
                    } else {
                        rng.range(-400, 400) as i32
                    }
                } else if name == "update.gate.bias" {
                    -2048
                } else if name == "copy.gate.bias" {
                    -4096
                } else {
                    rng.range(-2000, 2000) as i32
                }
            })
            .collect();
        spec_params.insert(
            name.clone(),
            ParameterQuantization {
                shape: shape.clone(),
                bits,
                row_exponents: vec![e; rows],
            },
        );
        codes.insert(name.clone(), values);
    }
    let spec = QuantizationSpec {
        schema: SPEC_SCHEMA.into(),
        parameters: spec_params,
    };
    // Payload in BTreeMap order.
    let mut payload = Vec::new();
    let mut entries = Vec::new();
    let mut stats = serde_json::Map::new();
    for (name, p) in &spec.parameters {
        let v = &codes[name];
        let start = payload.len() as u64;
        if p.bits == 4 {
            for pair in v.chunks(2) {
                let lo = (pair[0] & 0x0f) as u8;
                let hi = if pair.len() > 1 { (pair[1] & 0x0f) as u8 } else { 0 };
                payload.push(lo | (hi << 4));
            }
        } else {
            for &c in v {
                payload.extend_from_slice(&(c as i16).to_le_bytes());
            }
        }
        let bytes = payload.len() as u64 - start;
        entries.push(json!({"name":name,"offset":start,"elements":v.len(),"bytes":bytes}));
        stats.insert(
            name.clone(),
            json!({"elements":v.len(),"clipped_values":0,"zero_codes":0,"distinct_codes":0,
                   "max_absolute_error":0.0,"squared_error_sum":0.0}),
        );
    }
    fs::write(dir.join("hard-parameters.bin"), &payload).unwrap();
    let spec_bytes = serde_json::to_vec(&spec).unwrap();
    let mut h = Sha256::new();
    h.update(HARD_SCHEMA.as_bytes());
    h.update([0]);
    h.update((spec_bytes.len() as u64).to_le_bytes());
    h.update(&spec_bytes);
    h.update((payload.len() as u64).to_le_bytes());
    h.update(&payload);
    let binding = hex::encode(h.finalize());
    let descriptor = json!({
        "schema":HARD_SCHEMA,"encoding":ENCODING,"calibration_rule":CALIBRATION_RULE,
        "specification":spec,"specification_sha256":sha(&spec_bytes),
        "payload_bytes":payload.len(),"payload_sha256":sha(&payload),"binding_sha256":binding,
        "parameters":entries,"parameter_statistics":stats,
        "numerical_scope":"scratch random-weight timing fixture (not a trained model)"});
    let descriptor_bytes = serde_json::to_vec_pretty(&descriptor).unwrap();
    fs::write(dir.join("hard-parameters.json"), &descriptor_bytes).unwrap();
    let state = QuantizedTrainingState {
        start_step: 0,
        ramp_steps: 1,
        completed_step: 0,
        spec,
    };
    let manifest = json!({
        "schema":"uor-r4.joint-recurrent-packed-emulator/1",
        "numerical_contract":uor_r4_integer::config::quantized_numerical_contract().unwrap(),
        "parameter_manifest_sha256":sha(&descriptor_bytes),
        "parameter_manifest":descriptor,
        "model":cfg,
        "quantization":state});
    fs::write(
        dir.join("hard-model.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn stats(label: &str, ns: &[u128]) {
    let mean = |a: &[u128]| a.iter().sum::<u128>() as f64 / a.len() as f64 / 1e6;
    println!(
        "{label}: total256={:.2} ms | mean/step={:.3} ms | pos0-15={:.3} | pos120-135={:.3} | pos240-255={:.3} ms",
        ns.iter().sum::<u128>() as f64 / 1e6,
        mean(ns),
        mean(&ns[0..16]),
        mean(&ns[120..136]),
        mean(&ns[240..256])
    );
}

macro_rules! run_session {
    ($krate:ident, $model:expr, $tokens:expr, $mode:expr) => {{
        let model = $model;
        let mut session = model.new_session();
        let mut ns = Vec::with_capacity(256);
        let mut probs: Vec<Vec<u64>> = Vec::with_capacity(256);
        let mut nulls = Vec::new();
        for &t in $tokens.iter() {
            let clock = Instant::now();
            let step = model.step(&mut session, t, $mode).unwrap();
            ns.push(clock.elapsed().as_nanos());
            nulls.push(step.no_read_mass);
            probs.push(step.probabilities);
        }
        (ns, probs, nulls)
    }};
}

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("root dir"));
    let reps: usize = std::env::args()
        .nth(2)
        .map(|s| s.parse().unwrap())
        .unwrap_or(3);
    fs::create_dir(&root).expect("fresh root");
    let tables = root.join("tables");
    write_tables(&tables);
    let mut rng = Rng(0x1234_5678_9abc_def1);
    let tokens: Vec<u32> = (0..256).map(|_| (rng.next() % 4096) as u32).collect();
    for (label, transport) in [
        ("quaternion", Transport::Quaternion),
        ("householder", Transport::HouseholderPair),
    ] {
        let dir = root.join(format!("model-{label}"));
        write_model(&dir, transport, 0xdead_beef);
        let load = Instant::now();
        let sw = uor_r4_integer::IntegerModel::load_with_tables(&dir, &tables).unwrap();
        println!("[{label}] load (hash-verify+decode) {:.1} ms", load.elapsed().as_secs_f64() * 1e3);
        let hw = uor_r4_integer_hw::IntegerModel::load_with_tables(&dir, &tables).unwrap();
        let hw2 = uor_r4_integer_hw2::IntegerModel::load_with_tables(&dir, &tables).unwrap();
        let hw2_mode = |m: ReadMode| match m {
            ReadMode::Enabled => uor_r4_integer_hw2::ReadMode::Enabled,
            ReadMode::NoRead => uor_r4_integer_hw2::ReadMode::NoRead,
        };
        let hw_mode = |m: ReadMode| match m {
            ReadMode::Enabled => uor_r4_integer_hw::ReadMode::Enabled,
            ReadMode::NoRead => uor_r4_integer_hw::ReadMode::NoRead,
        };
        for mode in [ReadMode::Enabled, ReadMode::NoRead] {
            let mut best_sw: Option<Vec<u128>> = None;
            let mut best_hw: Option<Vec<u128>> = None;
            let mut best_hw2: Option<Vec<u128>> = None;
            let mut identical = true;
            let mut last_nulls = Vec::new();
            let mut max_p = 0u64;
            for _ in 0..reps {
                let (ns_sw, p_sw, nulls) = run_session!(uor_r4_integer, &sw, tokens, mode);
                let (ns_hw, p_hw, _) = run_session!(uor_r4_integer_hw, &hw, tokens, hw_mode(mode));
                let (ns_hw2, p_hw2, _) = run_session!(uor_r4_integer_hw2, &hw2, tokens, hw2_mode(mode));
                identical &= p_sw == p_hw && p_sw == p_hw2;
                if best_hw2.as_ref().map_or(true, |b: &Vec<u128>| ns_hw2.iter().sum::<u128>() < b.iter().sum::<u128>()) {
                    best_hw2 = Some(ns_hw2);
                }
                max_p = p_sw.last().unwrap().iter().copied().max().unwrap();
                last_nulls = nulls;
                let better = |best: &Option<Vec<u128>>, new: &Vec<u128>| {
                    best.as_ref().map_or(true, |b| new.iter().sum::<u128>() < b.iter().sum::<u128>())
                };
                if better(&best_sw, &ns_sw) {
                    best_sw = Some(ns_sw);
                }
                if better(&best_hw, &ns_hw) {
                    best_hw = Some(ns_hw);
                }
            }
            println!(
                "[{label}/{mode:?}] best of {reps}; sw vs hw distributions bit-identical over all 256 steps x {reps}: {identical}; last-step max prob={:.4}; null mass at pos 255={:.4}",
                max_p as f64 / (1u64 << 48) as f64,
                *last_nulls.last().unwrap() as f64 / (1u64 << 48) as f64
            );
            stats("  software-multiply (repo)", best_sw.as_ref().unwrap());
            stats("  hardware-multiply (scratch)", best_hw.as_ref().unwrap());
            stats("  hw-multiply + direct MAC dense maps (scratch)", best_hw2.as_ref().unwrap());
        }
    }
}
