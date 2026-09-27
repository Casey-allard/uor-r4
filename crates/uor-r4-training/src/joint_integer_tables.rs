//! Offline Rust construction and shared validated loading of integer tables.
//!
//! `export` is an offline compiler operation using F64 transcendentals. The
//! loaded tables and all accesses from the numerical model are integer only.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde_json::json;
use uor_r4_core::report_output;

use crate::Result;

use uor_r4_integer::lorentz::{
    arcosh_grid, ARCOSH_CODE_BITS, ARCOSH_ENTRIES, ARCOSH_FRACTION_BITS, ARCOSH_MANTISSA_BITS,
    ARCOSH_OUTPUT_BITS,
};
pub use uor_r4_integer::tables::{Tables, TOTAL};
use uor_r4_integer::tables::{ARCOSH_METADATA, ARCOSH_PAYLOAD, ARCOSH_SCHEMA};

const ENTRIES: usize = 65535;
const SCHEMA: &str = "uor-r4.joint-integer-tables/1";

/// Compile the complete finite input domains once, outside served computation.
pub fn export(directory: &Path) -> Result<()> {
    report_output::claim(directory)?;
    let mut file = File::create_new(directory.join("tables.bin"))?;
    for index in 0..ENTRIES {
        let x = (index as f64 - 32767.0) / 256.0;
        let sigmoid = (32768.0 / (1.0 + (-x).exp())).round() as i32;
        let tanh = (16384.0 * x.tanh()).round() as i32;
        let exp = ((-(index as f64) / 256.0).exp() * TOTAL as f64).round() as u64;
        file.write_all(&sigmoid.to_le_bytes())?;
        file.write_all(&tanh.to_le_bytes())?;
        file.write_all(&exp.to_le_bytes())?;
    }
    file.sync_all()?;
    let metadata = json!({
        "schema":SCHEMA, "entries":ENTRIES, "probability_bits":48,
        "payload_sha256":crate::sha256_file(&directory.join("tables.bin"))?,
        "layout":"Each little-endian row: sigmoid i32, tanh i32, exp u64",
        "sigmoid_tanh_domain":"signed affine codes -32767..32767 at exponent -8; output Q15/Q14 nearest ties away",
        "exp_domain":"negative affine differences 0..65534 at exponent -8; output Q48 nearest",
        "compiler":"offline Rust F64 exp/tanh; artifact hash binds actual table; no cross-platform bitwise claim",
        "runtime":"integer indexing only; table construction is not served computation"
    });
    fs::write(
        directory.join("tables.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    export_arcosh(directory)?;
    report_output::seal(directory)?;
    report_output::verify(directory)?;
    Ok(())
}

/// round(2^24 arcosh(1 + u)) at every grid point, for the Lorentz read.
pub fn arcosh_table() -> Vec<u32> {
    (0..ARCOSH_ENTRIES)
        .map(|index| {
            let u = arcosh_grid(index) as f64 * 2f64.powi(-(ARCOSH_FRACTION_BITS as i32));
            ((u + (u * (u + 2.0)).sqrt()).ln_1p() * 2f64.powi(ARCOSH_OUTPUT_BITS as i32)).round()
                as u32
        })
        .collect()
}

/// The Lorentz read's arcosh table, beside the retained tables. `tables.json`
/// and `tables.bin` are unchanged, so dot-read identities are unchanged.
fn export_arcosh(directory: &Path) -> Result<()> {
    let mut file = File::create_new(directory.join(ARCOSH_PAYLOAD))?;
    for value in arcosh_table() {
        file.write_all(&value.to_le_bytes())?;
    }
    file.sync_all()?;
    let metadata = json!({
        "schema":ARCOSH_SCHEMA, "entries":ARCOSH_ENTRIES,
        "argument_fraction_bits":ARCOSH_FRACTION_BITS, "points_per_octave_bits":ARCOSH_MANTISSA_BITS,
        "argument_code_bits":ARCOSH_CODE_BITS, "output_fraction_bits":ARCOSH_OUTPUT_BITS,
        "payload_sha256":crate::sha256_file(&directory.join(ARCOSH_PAYLOAD))?,
        "layout":"Little-endian u32 round(2^24 arcosh(1+u)) at u=code 2^-32: codes 0..1023 directly, then 1024 points per octave from 2^10 to 2^64, then the end point 2^64",
        "domain":"Lorentz read excess z-1 at Q32; the runtime interpolates linearly between grid points and rounds to nearest",
        "compiler":"offline Rust F64 sqrt/ln_1p; artifact hash binds actual table; no cross-platform bitwise claim",
        "runtime":"integer indexing, one crate::math product and a shift per lookup; table construction is not served computation"
    });
    fs::write(
        directory.join(ARCOSH_METADATA),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use candle_core::Device;
    use uor_r4_integer::lorentz::{arcosh1p_q24, excess_q32, exp_q32, squared_norm};
    use uor_r4_integer::IntegerModel;

    use super::*;
    use crate::invalid;
    use crate::joint_model::{JointConfig, JointModel, ReadGeometry, ReadMode, Transport};
    use crate::joint_model::{LORENTZ_LOG_BETA, LORENTZ_OFFSET};

    fn test_root(label: &str) -> Result<PathBuf> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock before epoch"))?
            .as_nanos();
        Ok(std::env::temp_dir().join(format!(
            "uor-joint-integer-{label}-{}-{nonce}",
            std::process::id()
        )))
    }

    fn arcosh1p(u: f64) -> f64 {
        (u + (u * (u + 2.0)).sqrt()).ln_1p()
    }

    /// Deterministic codes in [-limit, limit].
    struct Codes(u64);
    impl Codes {
        fn next(&mut self, limit: i32) -> i32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) % (2 * limit as u64 + 1)) as i32 - limit
        }
        fn vector(&mut self, limit: i32) -> Vec<i32> {
            (0..64).map(|_| self.next(limit)).collect()
        }
    }

    fn inner(a: &[i32], b: &[i32]) -> i128 {
        a.iter()
            .zip(b)
            .map(|(&x, &y)| i128::from(x) * i128::from(y))
            .sum()
    }

    /// The sealed table with its interpolation stays within 2e-7 of arcosh(1+u)
    /// from the first code to 2^64, including points between grid nodes.
    #[test]
    fn integer_arcosh_matches_the_function_across_every_octave() -> Result<()> {
        let table = arcosh_table();
        let mut worst = 0f64;
        for bits in 0..64u32 {
            for numerator in [1000u128, 1001, 1234, 1500, 1618, 1999, 2000] {
                let code = (numerator << bits) / 1000;
                if code >> 64 != 0 {
                    continue;
                }
                let u = code as f64 * 2f64.powi(-32);
                let integer = f64::from(arcosh1p_q24(code, &table)?) * 2f64.powi(-24);
                worst = worst.max((integer - arcosh1p(u)).abs());
            }
        }
        assert!(worst < 2e-7, "worst arcosh error {worst}");
        Ok(())
    }

    /// z - 1 from Q8 codes agrees with an F64 evaluation of q0 k0 - <q,k> - 1,
    /// across small, large, near-parallel and opposite vectors.
    #[test]
    fn integer_excess_matches_the_lorentz_inner_product() -> Result<()> {
        let mut codes = Codes(3);
        let mut worst = 0f64;
        for case in 0..400 {
            let limit = [8, 256, 4096, 32767][case % 4];
            let query = codes.vector(limit);
            let key: Vec<i32> = match case % 3 {
                0 => codes.vector(limit),
                // Near-parallel: small perturbation of a scaled query.
                1 => query
                    .iter()
                    .map(|&x| (x / 2 + codes.next(2)).clamp(-32767, 32767))
                    .collect(),
                _ => query.iter().map(|&x| -x).collect(),
            };
            let (q, k, d) = (
                squared_norm(&query)?,
                squared_norm(&key)?,
                inner(&query, &key),
            );
            let excess = excess_q32(q, k, d)? as f64 * 2f64.powi(-32);
            let scale = 2f64.powi(-16);
            let (q, k, d) = (q as f64 * scale, k as f64 * scale, d as f64 * scale);
            let reference = ((1.0 + q) * (1.0 + k)).sqrt() - d - 1.0;
            let tolerance = 2f64.powi(-31) + 1e-13 * (1.0 + q + k);
            worst = worst.max((excess - reference).abs() / tolerance);
        }
        assert!(worst <= 1.0, "worst excess error {worst} tolerances");
        Ok(())
    }

    /// The load-time scale agrees with exp to 2^-32 relative over the learned
    /// range, at every exponent the calibration can choose for a signed16 code.
    #[test]
    fn integer_scale_matches_exp() -> Result<()> {
        let mut codes = Codes(9);
        for exponent in -24..=-9 {
            for _ in 0..50 {
                let code = codes.next(32767);
                let x = f64::from(code) * 2f64.powi(exponent);
                if x >= 21.0 {
                    continue;
                }
                let integer = exp_q32(i128::from(code), exponent)? as f64;
                let reference = x.exp() * 2f64.powi(32);
                assert!(
                    (integer - reference).abs() <= 1.0 + reference * 2f64.powi(-40),
                    "exp({x}): {integer} against {reference}"
                );
            }
        }
        Ok(())
    }

    #[derive(Default)]
    struct Parity {
        probability: f64,
        state: f64,
        no_read: f64,
        read: f64,
        gate: f64,
        read_total: f64,
        /// Mean entropy (nats) of the emulator's read masses once 16 or more
        /// keys exist: near zero when one key takes the whole read.
        read_entropy: f64,
    }

    /// Run the packed F32 emulator and the integer runtime on the same tokens.
    fn parity(model: &JointModel, tables: &Path, tokens: &[u32]) -> Result<Parity> {
        let packed = test_root("packed")?;
        fs::create_dir(&packed)?;
        model.save_hard(&packed)?;
        let reference = JointModel::load_hard(&packed, &Device::Cpu)?;
        let integer = IntegerModel::load_with_tables(&packed, tables)?;
        let mut expected_session = reference.new_session(1)?;
        let mut actual_session = integer.new_session();
        let total = TOTAL as f64;
        let mut result = Parity::default();
        let mut spread_steps = 0usize;
        for &token in tokens {
            let actual = integer.step(&mut actual_session, token, ReadMode::Enabled)?;
            let expected = reference.step(&mut expected_session, &[token], ReadMode::Enabled)?;
            let probabilities = expected.probabilities.flatten_all()?.to_vec1::<f32>()?;
            for (&code, &value) in actual.probabilities.iter().zip(&probabilities) {
                result.probability = result
                    .probability
                    .max((code as f64 / total - f64::from(value)).abs());
            }
            let state = expected.state.flatten_all()?.to_vec1::<f32>()?;
            for (&code, &value) in actual.state.iter().zip(&state) {
                result.state = result
                    .state
                    .max((f64::from(code) / 2048.0 - f64::from(value)).abs());
            }
            let no_read = f64::from(expected.no_read_mass.flatten_all()?.to_vec1::<f32>()?[0]);
            result.no_read = result
                .no_read
                .max((actual.no_read_mass as f64 / total - no_read).abs());
            let reads = expected.read_masses.flatten_all()?.to_vec1::<f32>()?;
            if reads.len() != actual.read_masses.len() {
                return Err(invalid("read slot counts differ"));
            }
            for (&code, &value) in actual.read_masses.iter().zip(&reads) {
                result.read = result
                    .read
                    .max((code as f64 / total - f64::from(value)).abs());
            }
            result.read_total += 1.0 - actual.no_read_mass as f64 / total;
            if reads.len() >= 16 {
                spread_steps += 1;
                result.read_entropy += reads
                    .iter()
                    .map(|&mass| f64::from(mass))
                    .filter(|&mass| mass > 0.0)
                    .map(|mass| -mass * mass.ln())
                    .sum::<f64>();
            }
            let gate = f64::from(expected.copy_gate.flatten_all()?.to_vec1::<f32>()?[0]);
            result.gate = result
                .gate
                .max((f64::from(actual.copy_gate) / 32768.0 - gate).abs());
        }
        result.read_total /= tokens.len() as f64;
        result.read_entropy /= spread_steps.max(1) as f64;
        fs::remove_dir_all(packed)?;
        Ok(result)
    }

    /// Untrained width-256 and width-128 Lorentz models, quantized and packed, run in the
    /// integer runtime within the retained engineering drift limits (0.01 on
    /// probabilities and state) of their F32 emulator, as the dot read does.
    /// The Dot-matched initial scale reads almost only the oldest key at this
    /// width, so the flat (beta 1) and trained-scale starts cover spread reads.
    /// The runtime requires the arcosh table: a root without it serves the dot
    /// model and refuses the Lorentz one.
    #[test]
    fn packed_lorentz_read_serves_with_integers_within_drift_limits() -> Result<()> {
        let tables = test_root("tables")?;
        export(&tables)?;
        let mut codes = Codes(17);
        let block: Vec<u32> = (0..16).map(|_| (codes.next(2047) + 2048) as u32).collect();
        let tokens: Vec<u32> = block
            .iter()
            .chain(&block)
            .chain(block.iter().rev())
            .copied()
            .collect();
        let dot = JointConfig::default();
        let lorentz = JointConfig {
            read_geometry: ReadGeometry::Lorentz,
            ..JointConfig::default()
        };
        let householder = JointConfig {
            transport: Transport::HouseholderPair,
            seed: 5,
            ..lorentz.clone()
        };
        for (label, config, scalars, spread) in [
            ("dot", dot.clone(), None, true),
            ("lorentz", lorentz.clone(), None, false),
            (
                "lorentz-flat",
                lorentz.clone(),
                Some((0.0f32, 5.0f32)),
                true,
            ),
            // Scalars of a trained context-256 Lorentz model.
            (
                "lorentz-trained-scale",
                householder,
                Some((1.32, 4.94)),
                false,
            ),
            (
                "lorentz-flat-width128",
                JointConfig {
                    width: 128,
                    ..lorentz.clone()
                },
                Some((0.0, 5.0)),
                true,
            ),
        ] {
            let model = JointModel::new(config, &Device::Cpu)?;
            if let Some((log_beta, offset)) = scalars {
                let device = Device::Cpu;
                model.variables()[LORENTZ_LOG_BETA].set(&candle_core::Tensor::from_vec(
                    vec![log_beta],
                    (1,),
                    &device,
                )?)?;
                model.variables()[LORENTZ_OFFSET].set(&candle_core::Tensor::from_vec(
                    vec![offset],
                    (1,),
                    &device,
                )?)?;
            }
            let mut model = model;
            model.configure_quantization(0, 1)?;
            let result = parity(&model, &tables, &tokens)?;
            eprintln!(
                "{label}: probability {:.2e} state {:.2e} no-read {:.2e} read {:.2e} gate {:.2e} mean read mass {:.3} read entropy {:.2}",
                result.probability, result.state, result.no_read, result.read, result.gate, result.read_total, result.read_entropy
            );
            assert!(
                result.probability <= 0.01 && result.state <= 0.01,
                "{label}"
            );
            assert!(result.no_read <= 0.01 && result.read <= 0.01, "{label}");
            assert!(
                result.read_total > 0.5,
                "{label}: the read is not exercised"
            );
            if spread {
                assert!(result.read_entropy > 2.0, "{label}: the read is not spread");
            }
        }

        // A table root exported before the arcosh table existed.
        let legacy = test_root("legacy-tables")?;
        report_output::claim(&legacy)?;
        for name in ["tables.bin", "tables.json"] {
            fs::copy(tables.join(name), legacy.join(name))?;
        }
        report_output::seal(&legacy)?;
        let mut model = JointModel::new(lorentz, &Device::Cpu)?;
        model.configure_quantization(0, 1)?;
        let packed = test_root("packed-legacy")?;
        fs::create_dir(&packed)?;
        model.save_hard(&packed)?;
        let Err(error) = IntegerModel::load_with_tables(&packed, &legacy) else {
            return Err(invalid("Lorentz model loaded without an arcosh table"));
        };
        assert!(error.to_string().contains("arcosh"), "{error}");
        let mut model = JointModel::new(dot, &Device::Cpu)?;
        model.configure_quantization(0, 1)?;
        let dot_packed = test_root("packed-legacy-dot")?;
        fs::create_dir(&dot_packed)?;
        model.save_hard(&dot_packed)?;
        IntegerModel::load_with_tables(&dot_packed, &legacy)?;
        for root in [tables, legacy, packed, dot_packed] {
            fs::remove_dir_all(root)?;
        }
        Ok(())
    }

    /// A 4096-token byte-level tokenizer with `<|bos|>` = 0 and `<|eos|>` = 1:
    /// the 256 byte symbols, then unused pieces, and no merges.
    fn byte_level_tokenizer() -> Result<serde_json::Value> {
        let mut vocab = serde_json::Map::new();
        vocab.insert("<|bos|>".into(), json!(0));
        vocab.insert("<|eos|>".into(), json!(1));
        let mut shifted = 256u32;
        for byte in 0u32..256 {
            let printable = matches!(byte, 33..=126 | 161..=172 | 174..=255);
            let code = if printable {
                byte
            } else {
                shifted += 1;
                shifted - 1
            };
            let symbol = char::from_u32(code).ok_or_else(|| invalid("byte symbol"))?;
            vocab.insert(symbol.to_string(), json!(byte + 2));
        }
        for id in 258..4096 {
            vocab.insert(format!("<|unused{id}|>"), json!(id));
        }
        Ok(json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [{"id": 0, "content": "<|bos|>"}, {"id": 1, "content": "<|eos|>"}],
            "model": {"type": "BPE", "vocab": vocab, "merges": []}
        }))
    }

    /// A packed Lorentz model becomes a development serving bundle and
    /// generates text through the integer session; `pack` still refuses it,
    /// because it has no accepted parent.
    #[test]
    fn development_bundle_generates_with_a_lorentz_model() -> Result<()> {
        use uor_r4_integer::bundle::{self, Bundle};
        use uor_r4_integer::generation::{Request, Selection};
        let tables = test_root("bundle-tables")?;
        export(&tables)?;
        let mut model = JointModel::new(
            JointConfig {
                read_geometry: ReadGeometry::Lorentz,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        model.variables()[LORENTZ_LOG_BETA].set(&candle_core::Tensor::from_vec(
            vec![0f32],
            (1,),
            &Device::Cpu,
        )?)?;
        model.configure_quantization(0, 1)?;
        let packed = test_root("bundle-packed")?;
        fs::create_dir(&packed)?;
        model.save_hard(&packed)?;
        let tokenizer = test_root("bundle-tokenizer")?;
        fs::create_dir(&tokenizer)?;
        let tokenizer_file = tokenizer.join("tokenizer.json");
        fs::write(
            &tokenizer_file,
            serde_json::to_vec(&byte_level_tokenizer()?)?,
        )?;
        let root = test_root("bundle")?;
        bundle::pack_development(&packed, &tables, &tokenizer_file, &root)?;
        let served = Bundle::load(&root)?;
        let generation = served.generate(&Request {
            prompt: "Once upon a time".into(),
            max_new_tokens: 8,
            selection: Selection::Categorical { top_k: 40, seed: 1 },
            read_mode: ReadMode::Enabled,
            first_sentence: false,
        })?;
        let generated = generation.generated_token_ids.len();
        assert!((1..=8).contains(&generated), "{generated}");
        assert_eq!(
            generation.prompt_token_ids.len(),
            1 + "Once upon a time".len()
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("bundle.json"))?)?;
        assert!(metadata["provenance"]
            .as_str()
            .is_some_and(|text| text.starts_with("development")));
        assert!(metadata["numerical_contract"]
            .as_str()
            .is_some_and(|text| text.contains("lorentz read")));
        // Partitioned conversational memory scores by the dot read only, so it
        // refuses a Lorentz model instead of computing the wrong scores.
        let model = served.model();
        let mut conversation = model.new_conversational_session();
        let Err(error) = model.step_conversational(
            &mut conversation,
            5,
            uor_r4_integer::SlotTarget::Dialogue,
            ReadMode::Enabled,
        ) else {
            return Err(invalid("conversational step accepted a Lorentz model"));
        };
        assert!(error.to_string().contains("dot-read"), "{error}");
        let accepted = test_root("bundle-accepted")?;
        assert!(bundle::pack(&packed, &tables, &tokenizer_file, &accepted).is_err());
        for directory in [tables, packed, tokenizer, root, accepted] {
            if directory.exists() {
                fs::remove_dir_all(directory)?;
            }
        }
        Ok(())
    }
}
