//! Exclusive report directories for experiment drivers.
//!
//! A report root is claimed once, with exclusive creation semantics, before any
//! model work; an existing path fails instead of being reused or overwritten.
//! A completed attempt is sealed with a manifest naming every file and its
//! BLAKE3 digest, so retries and interrupted attempts keep distinct directories,
//! logs and charges. This is a small executable guard, not an evidence framework.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Sentinel written into a freshly claimed report directory.
pub const ATTEMPT_FILE: &str = "attempt.json";
/// Manifest written once when an attempt is sealed.
pub const MANIFEST_FILE: &str = "manifest.json";

/// Create `path` exclusively as a new report directory and write its attempt
/// sentinel. Parents are created as containers; the leaf must not exist. An
/// existing leaf, including an existing completed report, fails before any
/// caller work and is left byte-identical.
pub fn claim(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "report directory {} was not created exclusively ({e}); choose a new attempt path instead of reusing or overwriting an existing report",
                path.display()
            ),
        )
    })?;
    let argv: Vec<String> = std::env::args().collect();
    let sentinel = serde_json::json!({
        "schema": "uor-r4.report-attempt/1",
        "claimed_at": humantime_now(),
        "pid": std::process::id(),
        "argv": argv,
        "rule": "exclusively created before model work; never reused for a retry"
    });
    let mut file = fs::File::create_new(path.join(ATTEMPT_FILE))?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&sentinel)?.as_bytes(),
    )?;
    Ok(())
}

/// Seal a completed attempt: write `manifest.json` (exclusively) listing every
/// regular file below `path` with its size and BLAKE3 digest. Sealing twice fails.
pub fn seal(path: &Path) -> io::Result<PathBuf> {
    let manifest_path = path.join(MANIFEST_FILE);
    if manifest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is already sealed", path.display()),
        ));
    }
    let mut files = Vec::new();
    collect(path, path, &mut files)?;
    files.sort();
    let mut entries = Vec::with_capacity(files.len());
    for relative in files {
        let bytes = fs::read(path.join(&relative))?;
        entries.push(serde_json::json!({
            "path": relative,
            "bytes": bytes.len(),
            "blake3": blake3::hash(&bytes).to_hex().to_string(),
        }));
    }
    let manifest = serde_json::json!({
        "schema": "uor-r4.report-manifest/1",
        "sealed_at": humantime_now(),
        "root": path,
        "files": entries,
    });
    let mut file = fs::File::create_new(&manifest_path)?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&manifest)?.as_bytes(),
    )?;
    Ok(manifest_path)
}

/// Verify a sealed attempt: every listed file must exist with its recorded size and
/// BLAKE3 digest, and the directory must hold no unlisted regular file besides the
/// manifest. Returns the sorted list of unlisted files on success (empty) or an error
/// naming the first listed mismatch; unlisted files are reported as an error too, so
/// a complete file set is checked, not only the listed hashes.
pub fn verify(path: &Path) -> io::Result<Vec<String>> {
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path.join(MANIFEST_FILE))?)?;
    let listed = manifest["files"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest without files"))?;
    let mut names = std::collections::BTreeSet::new();
    for entry in listed {
        let relative = entry["path"].as_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "manifest entry without path")
        })?;
        let bytes = fs::read(path.join(relative))?;
        if entry["bytes"].as_u64() != Some(bytes.len() as u64)
            || entry["blake3"].as_str() != Some(blake3::hash(&bytes).to_hex().as_str())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("sealed file changed: {relative}"),
            ));
        }
        names.insert(relative.to_owned());
    }
    let mut actual = Vec::new();
    collect(path, path, &mut actual)?;
    let unlisted: Vec<String> = actual
        .into_iter()
        .filter(|f| f != MANIFEST_FILE && !names.contains(f))
        .collect();
    if !unlisted.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "sealed directory holds unlisted files: {}",
                unlisted.join(", ")
            ),
        ));
    }
    Ok(unlisted)
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect(root, &path, out)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| io::Error::other(e.to_string()))?;
            out.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn humantime_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{now}")
}

/// Two-sided 95% Student-$t$ critical value $t_{0.975, \text{df}}$ for degrees of freedom $\text{df} = n - 1$.
///
/// Exact tabulated values for $1 \le \text{df} \le 30$; second-order Cornish-Fisher expansion ($O(\nu^{-2})$)
/// around $z_{0.975} = 1.959963984540054$ for $\text{df} > 30$.
pub fn student_t_0975(df: usize) -> f64 {
    match df {
        0 => f64::INFINITY,
        1 => 12.706_204_736_4,
        2 => 4.302_652_729_91,
        3 => 3.182_446_305_28,
        4 => 2.776_445_105_2,
        5 => 2.570_581_836_61,
        6 => 2.446_911_851_14,
        7 => 2.364_624_251_6,
        8 => 2.306_004_135_2,
        9 => 2.262_157_162_85,
        10 => 2.228_138_851_96,
        11 => 2.200_985_160_08,
        12 => 2.178_812_829_66,
        13 => 2.160_368_656_46,
        14 => 2.144_786_687_92,
        15 => 2.131_449_545_56,
        16 => 2.119_905_299_22,
        17 => 2.109_815_577_83,
        18 => 2.100_922_040_24,
        19 => 2.093_024_054_41,
        20 => 2.085_963_447_27,
        21 => 2.079_613_844_73,
        22 => 2.073_873_067_9,
        23 => 2.068_657_610_42,
        24 => 2.063_898_561_63,
        25 => 2.059_538_552_75,
        26 => 2.055_529_438_64,
        27 => 2.051_830_516_48,
        28 => 2.048_407_141_8,
        29 => 2.045_229_642_13,
        30 => 2.042_272_456_3,
        _ => {
            let z = 1.959_963_984_540_054_f64;
            let d = df as f64;
            z + (z.powi(3) + z) / (4.0 * d)
                + (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / (96.0 * d * d)
        }
    }
}

/// Alias for [`student_t_0975`].
pub fn student_t_975(df: usize) -> f64 {
    student_t_0975(df)
}

/// Unbiased sample mean, sample standard deviation ($n - 1$ denominator), and two-sided
/// 95% Student-$t$ confidence interval half-width ($t_{0.975, n-1} \cdot s / \sqrt{n}$).
/// Returns `None` when $n < 2$.
pub fn sample_mean_sd_t_ci95(samples: &[f64]) -> Option<(f64, f64, f64)> {
    let n = samples.len();
    if n < 2 {
        return None;
    }
    let nf = n as f64;
    let mean = samples.iter().sum::<f64>() / nf;
    let var = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / ((n - 1) as f64);
    let sd = var.sqrt();
    let sem = sd / nf.sqrt();
    let ci95 = student_t_0975(n - 1) * sem;
    Some((mean, sd, ci95))
}

/// Validation failure returned by [`validate_protocol_v3_energy_receipt`].
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolV3ValidationError {
    /// `work_rate_w` disagrees with `gross_j_per_tok * tokens_per_s` by more than 2%.
    PowerThroughputInconsistency {
        reported_work_rate_w: f64,
        implied_work_rate_w: f64,
        relative_error: f64,
    },
    /// Reported `ci95` disagrees with Student's $t_{0.975, n-1} \cdot s / \sqrt{n}$ by more than 2%.
    InvalidStudentTCi95 {
        metric: String,
        reported_ci95: f64,
        expected_t_ci95: f64,
        relative_error: f64,
    },
    /// Sample count $n < 2$, so unbiased sample variance and Student-$t$ degrees of freedom $\nu = n - 1$ are undefined.
    InvalidSampleCount { metric: String, count: usize },
    /// `work_rate_w <= idle_rate_w` in strict qualification mode (zero or negative net power above idle).
    NonPositiveNetPower { work_rate_w: f64, idle_rate_w: f64 },
    /// One or more external inference daemons (`ollama`, `llama-server`, `vllm`, etc.) were running.
    ExternalInferenceDaemonsDetected(Vec<String>),
    /// Required Protocol v3 full-path field is missing from the receipt.
    MissingRequiredV3Field(&'static str),
}

/// Validate an energy + full-path cost receipt against strict Protocol v3 qualification requirements
/// (`allow_provisional = false`).
pub fn validate_protocol_v3_energy_receipt(
    receipt: &serde_json::Value,
) -> Vec<ProtocolV3ValidationError> {
    validate_protocol_v3_energy_receipt_with_mode(receipt, false)
}

/// Validate an energy + full-path cost receipt against Protocol v3 requirements, with an explicit
/// `allow_provisional` toggle for non-quiescent host rehearsals.
pub fn validate_protocol_v3_energy_receipt_with_mode(
    receipt: &serde_json::Value,
    allow_provisional: bool,
) -> Vec<ProtocolV3ValidationError> {
    use serde_json::Value;
    let mut errors = Vec::new();

    let work_rate_w = receipt
        .pointer("/power/work_rate_w")
        .or_else(|| receipt.pointer("/serving/energy/work_rate_w"))
        .and_then(Value::as_f64);
    let idle_rate_w = receipt
        .pointer("/power/idle_rate_w")
        .or_else(|| receipt.pointer("/serving/energy/idle_power_w"))
        .and_then(Value::as_f64);
    let gross_j = receipt
        .pointer("/energy_per_token/gross_j_per_tok")
        .or_else(|| receipt.pointer("/serving/energy/gross_j_per_tok/mean"))
        .and_then(Value::as_f64);
    let tok_s = receipt
        .pointer("/throughput/tokens_per_s")
        .or_else(|| receipt.pointer("/serving/tokens_per_s/mean"))
        .and_then(Value::as_f64);

    if let (Some(work_rate_w), Some(gross_j), Some(tok_s)) = (work_rate_w, gross_j, tok_s) {
        let implied = gross_j * tok_s;
        let rel_err = (work_rate_w - implied).abs() / implied.max(1e-12);
        if rel_err > 0.02 {
            errors.push(ProtocolV3ValidationError::PowerThroughputInconsistency {
                reported_work_rate_w: work_rate_w,
                implied_work_rate_w: implied,
                relative_error: rel_err,
            });
        }
    } else {
        errors.push(ProtocolV3ValidationError::MissingRequiredV3Field(
            "power.work_rate_w / energy_per_token.gross_j_per_tok / throughput.tokens_per_s",
        ));
    }

    if !allow_provisional {
        if let (Some(work_w), Some(idle_w)) = (work_rate_w, idle_rate_w) {
            if work_w <= idle_w {
                errors.push(ProtocolV3ValidationError::NonPositiveNetPower {
                    work_rate_w: work_w,
                    idle_rate_w: idle_w,
                });
            }
        }
    }

    let top_num_runs = receipt
        .pointer("/num_runs")
        .or_else(|| receipt.pointer("/throughput/trials"))
        .or_else(|| receipt.pointer("/serving/num_runs"))
        .or_else(|| receipt.pointer("/serving/energy/num_runs"))
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .or_else(|| {
            receipt
                .pointer("/serving/tokens_per_s/runs")
                .and_then(Value::as_array)
                .map(Vec::len)
        });
    if let Some(n) = top_num_runs {
        if n < 2 {
            errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                metric: "num_runs".to_string(),
                count: n,
            });
        }
    }

    for (name, ptr) in [
        ("gross_mj_per_tok", "/energy_per_token/gross_mj_per_tok"),
        ("net_mj_per_tok", "/energy_per_token/net_mj_per_tok"),
    ] {
        if let Some(block) = receipt.pointer(ptr) {
            let samples: Option<Vec<f64>> = block
                .get("samples")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_f64).collect());
            let reported_ci95 = block.get("ci95").and_then(Value::as_f64);
            let reported_sd = block.get("sd").and_then(Value::as_f64);
            let block_n = block
                .get("n")
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .or(top_num_runs);

            if let Some(ref s) = samples {
                if s.len() < 2 {
                    errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                        metric: name.to_string(),
                        count: s.len(),
                    });
                } else if let (Some((_mean, _sd, expected_ci95)), Some(reported)) =
                    (sample_mean_sd_t_ci95(s), reported_ci95)
                {
                    let rel_err = (reported - expected_ci95).abs() / expected_ci95.max(1e-12);
                    if rel_err > 0.02 {
                        errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                            metric: name.to_string(),
                            reported_ci95: reported,
                            expected_t_ci95: expected_ci95,
                            relative_error: rel_err,
                        });
                    }
                }
            } else if let (Some(sd), Some(n), Some(reported)) =
                (reported_sd, block_n, reported_ci95)
            {
                if n < 2 {
                    errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                        metric: name.to_string(),
                        count: n,
                    });
                } else {
                    let expected_ci95 = student_t_0975(n - 1) * sd / (n as f64).sqrt();
                    let rel_err = (reported - expected_ci95).abs() / expected_ci95.max(1e-12);
                    if rel_err > 0.02 {
                        errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                            metric: name.to_string(),
                            reported_ci95: reported,
                            expected_t_ci95: expected_ci95,
                            relative_error: rel_err,
                        });
                    }
                }
            } else if reported_ci95.is_some() {
                // Never fall back to min/mean/max or to another metric, and never silently skip
                // when `ci95` is reported without `samples` or `(sd, n >= 2)`.
                errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                    metric: name.to_string(),
                    count: block_n.unwrap_or(0),
                });
            }
        }
    }

    // Check `/serving/energy/net_j_per_tok/ci95` (legacy v2 schema):
    // 1. If `/serving/energy/net_j_per_tok/samples` is present, validate for any n >= 2.
    // 2. Else if `/serving/energy/net_j_per_tok/sd` and `num_runs >= 2` are present, validate
    //    `ci95 = t_{0.975, n-1} * sd / sqrt(n)` for any n >= 2.
    // 3. Else `/serving/energy/net_j_per_tok` omitted its own `samples` and `sd`, which is always
    //    flagged via `InvalidSampleCount`; additionally, on a 3-run legacy v2 receipt such as
    //    `archive/ops/measurement_report_rnd.json` where `gross_j_per_tok` records
    //    `min = 0.003135, mean = 0.003172, max = 0.003199` J/tok (`n = 3`), reconstruct the exact
    //    3 gross observations via the 3-element sum identity `x_2 = 3 * mean - min - max`
    //    (`[3.135, 3.182, 3.199]` mJ) to diagnose the v2 normal-z (`1.96 * SE = 0.0375 mJ`) vs
    //    Student-t_2 (`4.302653 * SE = 0.08235 mJ`) defect under constant-idle variance reuse.
    if let Some(net_ci95_j) = receipt
        .pointer("/serving/energy/net_j_per_tok/ci95")
        .and_then(Value::as_f64)
    {
        if let Some(samples_j) = receipt
            .pointer("/serving/energy/net_j_per_tok/samples")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<f64>>())
        {
            if samples_j.len() < 2 {
                errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                    metric: "serving.energy.net_j_per_tok".to_string(),
                    count: samples_j.len(),
                });
            } else if let Some((_m, _sd, expected_t_ci95_j)) = sample_mean_sd_t_ci95(&samples_j) {
                let rel_err = (net_ci95_j - expected_t_ci95_j).abs() / expected_t_ci95_j.max(1e-12);
                if rel_err > 0.02 {
                    errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                        metric: "serving.energy.net_j_per_tok".to_string(),
                        reported_ci95: net_ci95_j * 1000.0,
                        expected_t_ci95: expected_t_ci95_j * 1000.0,
                        relative_error: rel_err,
                    });
                }
            }
        } else if let (Some(sd_j), Some(n)) = (
            receipt
                .pointer("/serving/energy/net_j_per_tok/sd")
                .and_then(Value::as_f64),
            top_num_runs,
        ) {
            if n < 2 {
                errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                    metric: "serving.energy.net_j_per_tok".to_string(),
                    count: n,
                });
            } else {
                let expected_t_ci95_j = student_t_0975(n - 1) * sd_j / (n as f64).sqrt();
                let rel_err = (net_ci95_j - expected_t_ci95_j).abs() / expected_t_ci95_j.max(1e-12);
                if rel_err > 0.02 {
                    errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                        metric: "serving.energy.net_j_per_tok".to_string(),
                        reported_ci95: net_ci95_j * 1000.0,
                        expected_t_ci95: expected_t_ci95_j * 1000.0,
                        relative_error: rel_err,
                    });
                }
            }
        } else {
            errors.push(ProtocolV3ValidationError::InvalidSampleCount {
                metric: "serving.energy.net_j_per_tok".to_string(),
                count: 0,
            });
            if top_num_runs == Some(3) {
                if let Some(((min_j, mean_j), max_j)) = receipt
                    .pointer("/serving/energy/gross_j_per_tok/min")
                    .and_then(Value::as_f64)
                    .zip(
                        receipt
                            .pointer("/serving/energy/gross_j_per_tok/mean")
                            .and_then(Value::as_f64),
                    )
                    .zip(
                        receipt
                            .pointer("/serving/energy/gross_j_per_tok/max")
                            .and_then(Value::as_f64),
                    )
                {
                    let mid_j = 3.0 * mean_j - min_j - max_j;
                    let exact_3run_j = [min_j, mid_j, max_j];
                    if let Some((_m, _sd, expected_t_ci95_j)) = sample_mean_sd_t_ci95(&exact_3run_j)
                    {
                        let rel_err =
                            (net_ci95_j - expected_t_ci95_j).abs() / expected_t_ci95_j.max(1e-12);
                        if rel_err > 0.02 {
                            errors.push(ProtocolV3ValidationError::InvalidStudentTCi95 {
                                metric: "serving.energy.net_j_per_tok (vs reconstructed gross_j_per_tok 3-run SD)".to_string(),
                                reported_ci95: net_ci95_j * 1000.0,
                                expected_t_ci95: expected_t_ci95_j * 1000.0,
                                relative_error: rel_err,
                            });
                        }
                    }
                }
            }
        }
    }

    for required in [
        "/memory/parameter_breakdown",
        "/step_latency/phase_latency_us",
        "/tokens/logits_sha256",
        "/machine/external_inference_daemons_detected",
        "/memory/kernel_peak_footprint_bytes",
    ] {
        if receipt.pointer(required).is_none() {
            errors.push(ProtocolV3ValidationError::MissingRequiredV3Field(required));
        }
    }

    if !allow_provisional {
        if let Some(arr) = receipt
            .pointer("/machine/external_inference_daemons_detected")
            .and_then(Value::as_array)
        {
            let daemons: Vec<String> = arr
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect();
            if !daemons.is_empty() {
                errors.push(ProtocolV3ValidationError::ExternalInferenceDaemonsDetected(
                    daemons,
                ));
            }
        }
    }

    errors
}

/// All 42 `#[inline(never)]` `uor_r4_integer::stack` kernel and mixer symbol suffixes in the
/// `IntegerStackSession::step` call graph (matching `STACK_MANDATORY_SYMBOLS` in
/// `scripts/audit_zero_matmul_serving.py`; together with `IntegerStackSession::step`, this forms
/// the 43 mandatory stack serving symbols).
///
/// Note on D23 ratchet-tracked legacy kernels:
/// `stack_exp_neg` and `stack_sigmoid_q31` are D11 sealed integer-table kernels tracked as legacy
/// sites by the D23 ratchet baseline. Including them here verifies that their compiled machine code
/// contains zero hardware multiply, divide, or floating-point instructions, not CHARTER §3.2
/// mechanism compliance.
const STACK_CALL_GRAPH_KERNELS: &[&str] = &[
    "stack_step",
    "stack_map_pairs",
    "stack_map_nibbles",
    "stack_step_call",
    "stack_map_task",
    "stack_head_task",
    "stack_heads",
    "stack_lanes",
    "stack_recurrence",
    "stack_rotation",
    "stack_read",
    "stack_pointer",
    "stack_swiglu",
    "stack_gemv",
    "stack_gemv_pairs",
    "stack_pair_tables",
    "stack_activation_tables",
    "stack_dequant_row",
    "stack_rms_norm",
    "stack_quantize16",
    "stack_max_abs_i32",
    "stack_max_abs_i64",
    "stack_exp_neg",
    "stack_sigmoid_q31",
    "stack_activation",
    "stack_arcosh1p_q24",
    "stack_lift",
    "stack_lorentz_distance",
    "stack_l2_distance",
    "stack_query_tables",
    "stack_dot",
    "stack_mix_row",
    "stack_hamilton",
    "stack_mul_u128",
    "stack_mul_u64",
    "stack_square",
    "stack_div_u64",
    "stack_div_u128",
    "stack_isqrt",
    "stack_argmax",
    "stack_pointer_mixture",
    "stack_snap_select",
    "stack_snap_rotation",
];

fn is_step_root_symbol(sym: &str) -> bool {
    if sym.contains("measure_step_phases") || sym.contains("closure") || sym.contains("drop_glue") {
        return false;
    }
    (sym.contains("IntegerStackSession") && sym.ends_with("::step"))
        || (sym.contains("19IntegerStackSession") && sym.ends_with("4step"))
}

fn matches_mandatory_stack_kernel(sym: &str, kernel: &str) -> bool {
    if sym.contains("measure_step_phases")
        || sym.contains("closure")
        || sym.contains("drop_glue")
        || sym.contains("from_iter")
        || sym.contains("try_process")
        || sym.contains("GenericShunt")
    {
        return false;
    }
    let in_stack_mod = sym.contains("stack::") || sym.contains("5stack");
    if !in_stack_mod {
        return false;
    }
    let demangled_exact = format!("::{kernel}");
    let mangled_exact = format!("{}{kernel}", kernel.len());
    sym.ends_with(&demangled_exact) || sym.ends_with(&mangled_exact)
}

fn is_stack_serving_or_closure_symbol(sym: &str) -> bool {
    if sym.contains("measure_step_phases") {
        return false;
    }
    if is_step_root_symbol(sym) {
        return true;
    }
    let in_stack_mod = sym.contains("uor_r4_integer::stack")
        || sym.contains("14uor_r4_integer5stack")
        || sym.contains("stack::session::")
        || sym.contains("stack::kernels::");
    if !in_stack_mod {
        return false;
    }
    for &kernel in STACK_CALL_GRAPH_KERNELS {
        let demangled_exact = format!("::{kernel}");
        let demangled_closure = format!("::{kernel}::");
        let mangled_exact = format!("{}{kernel}", kernel.len());
        if sym.ends_with(&demangled_exact)
            || sym.contains(&demangled_closure)
            || sym.contains(&mangled_exact)
        {
            return true;
        }
    }
    false
}

/// Non-numerical formatting, panic, I/O, model-loading, and pure work-stealing scheduler symbols
/// excluded from transitive call-graph descent (matching `allow_patterns` and
/// `STACK_SCHEDULER_ALLOW` in `scripts/audit_zero_matmul_serving.py`).
/// Note: any scheduler generic instantiated over `uor_r4_integer::stack` is NOT allowlisted and
/// IS audited so inlined closure arithmetic cannot hide inside rayon/crossbeam.
fn is_call_graph_allowlisted(sym: &str) -> bool {
    const SCHEDULER_PATTERNS: &[&str] = &[
        "rayon_core",
        "crossbeam_deque",
        "crossbeam_epoch",
        "crossbeam_utils",
    ];
    let is_sched = SCHEDULER_PATTERNS.iter().any(|pat| sym.contains(pat));
    let own_stack = sym.contains("uor_r4_integer::stack") || sym.contains("14uor_r4_integer5stack");
    if is_sched && !own_stack {
        return true;
    }
    const EXACT_ALLOCATOR_ENTRY_POINTS: &[&str] = &[
        "__rust_alloc",
        "___rust_alloc",
        "__rust_dealloc",
        "___rust_dealloc",
        "__rust_realloc",
        "___rust_realloc",
        "__rust_alloc_zeroed",
        "___rust_alloc_zeroed",
        "__rust_no_alloc_shim_is_unstable",
        "___rust_no_alloc_shim_is_unstable",
        "__rust_alloc_error_handler",
        "___rust_alloc_error_handler",
        "malloc",
        "_malloc",
        "free",
        "_free",
        "realloc",
        "_realloc",
        "calloc",
        "_calloc",
        "posix_memalign",
        "_posix_memalign",
    ];
    let clean_sym = sym.split('+').next().unwrap_or(sym).trim();
    if EXACT_ALLOCATOR_ENTRY_POINTS.contains(&clean_sym) {
        return true;
    }
    const ALLOW_PATTERNS: &[&str] = &[
        "alloc::raw_vec",
        "5alloc7raw_vec",
        "handle_alloc_error",
        "precondition_check",
        "core::fmt",
        "4core3fmt",
        "panic",
        "fmt::Display",
        "fmt::Debug",
        "7Display",
        "5Debug",
        "unwind",
        "rust_begin_unwind",
        "std::io",
        "3std2io",
        "std::panicking",
        "9panicking",
        "Bundle",
        "Tokenizer",
        "tokenizer",
        "uor_r4_tokenizer",
        "core::str",
        "4core3str",
        "load",
        "from_file",
        "from_serialized",
        "dyld_stub_binder",
    ];
    ALLOW_PATTERNS.iter().any(|pat| sym.contains(pat))
}

/// True when a column or token sequence represents raw instruction encoding bytes in `objdump -d`:
/// either a single 8-hex-digit 32-bit word (AArch64 `d102c3ff`) or 1..=15 space-separated
/// 2-hex-digit bytes (x86-64 `48 89 e5`). Never matches any instruction mnemonic such as `fadd`
/// (4 chars), `fabd` (4 chars), `fcadd` (5 chars), or `add` (3 chars).
fn is_raw_encoding_column(col: &str) -> bool {
    let col = col.trim();
    if col.is_empty() {
        return false;
    }
    if col.len() == 8 && col.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    let mut count = 0usize;
    for tok in col.split_whitespace() {
        if tok.len() != 2 || !tok.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
        count += 1;
    }
    (1..=15).contains(&count)
}

/// Extract `(mnemonic, operands)` from a disassembled instruction line by column position after
/// the hex address and optional raw encoding bytes (never filtering mnemonics by hex characters,
/// so all-hex mnemonics such as `fadd`, `fabd`, `fcadd`, `add`, and `b` are preserved).
fn extract_mnemonic_and_operands(line: &str) -> Option<(String, String)> {
    let mut ws = line.split_whitespace();
    let first_tok = ws.next()?;
    let addr_clean = first_tok.trim_end_matches(':');
    if addr_clean.is_empty() || !addr_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    if line.contains('\t') {
        let cols: Vec<&str> = line
            .split('\t')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        if cols.len() < 2 {
            return None;
        }
        // In `llvm-objdump` (`100000888: d102c3ff\tsub\tsp, sp, #0xb0`) and `otool -tV`
        // (`0000000100000888\tsub\tsp, sp, #0xb0`), `cols[0]` does not end with `:` (or when
        // `--no-show-raw-insn` is used, `cols[1]` is not a raw encoding column), so `cols[1]` is
        // the mnemonic column. In GNU `objdump` or tab-separated lines with a separate encoding
        // column (`100001000:\t1e612800\tfadd\td0, d0, d1`), `cols[0]` ends with `:` and `cols[1]`
        // is an 8-hex-digit 32-bit word or 2-hex-digit byte sequence, so `cols[2]` is the mnemonic.
        let mnemonic_col_idx =
            if cols[0].ends_with(':') && cols.len() >= 3 && is_raw_encoding_column(cols[1]) {
                2
            } else {
                1
            };
        let col = cols.get(mnemonic_col_idx)?;
        let mut col_ws = col.split_whitespace();
        let mnemonic = col_ws.next()?.to_ascii_lowercase();
        let mut operands: Vec<&str> = col_ws.collect();
        for &extra_col in &cols[mnemonic_col_idx + 1..] {
            operands.push(extra_col);
        }
        Some((mnemonic, operands.join(" ")))
    } else {
        let rest: Vec<&str> = ws.collect();
        if rest.is_empty() {
            return None;
        }
        let mut idx = 0usize;
        if rest[0].len() == 8 && rest[0].chars().all(|c| c.is_ascii_hexdigit()) {
            idx = 1;
        } else {
            while idx < rest.len()
                && rest[idx].len() == 2
                && rest[idx].chars().all(|c| c.is_ascii_hexdigit())
            {
                idx += 1;
            }
        }
        let mnemonic = rest.get(idx)?.to_ascii_lowercase();
        let operands = rest[idx + 1..].join(" ");
        Some((mnemonic, operands))
    }
}

/// Extract a direct call or branch target symbol name from an instruction's `(mnemonic, operands)`
/// across `objdump --demangle -d` (`bl 0x1000008cc <sym>`) and `otool -tvV` (`bl _sym`).
fn extract_call_target(mnemonic: &str, operands: &str, current_sym: &str) -> Option<String> {
    let base = mnemonic.split('.').next().unwrap_or(mnemonic);
    if !matches!(base, "bl" | "b" | "call" | "callq" | "jmp" | "jmpq") {
        return None;
    }
    if let (Some(start), true) = (operands.find('<'), operands.ends_with('>')) {
        let mut target = &operands[start + 1..operands.len() - 1];
        if let Some(plus_idx) = target.rfind("+0x") {
            target = &target[..plus_idx];
        }
        if target != current_sym && !target.is_empty() {
            return Some(target.to_string());
        }
        return None;
    }
    // `otool -tvV` format: `bl _symbol` or `bl 0x10018de34 ; symbol stub for: _memcpy`
    let first_op = operands
        .split(';')
        .next()
        .unwrap_or(operands)
        .split_whitespace()
        .next()?;
    if !first_op.starts_with("0x")
        && !first_op.chars().all(|c| c.is_ascii_hexdigit())
        && first_op != current_sym
    {
        return Some(first_op.to_string());
    }
    None
}

fn is_scalar_float_register_token(tok: &str) -> bool {
    let clean = tok.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    if let Some(num_str) = clean
        .strip_prefix('s')
        .or_else(|| clean.strip_prefix('S'))
        .or_else(|| clean.strip_prefix('h'))
        .or_else(|| clean.strip_prefix('H'))
    {
        if let Ok(reg_num) = num_str.parse::<u32>() {
            return reg_num <= 31;
        }
    }
    false
}

/// Check whether `(mnemonic, operands)` is a forbidden Class I (hardware or compiler-rt multiply),
/// Class II (hardware or compiler-rt divide), Class III (floating-point / SIMD float transfer), or
/// unresolved indirect subroutine call (`blr` / `call *`) on AArch64 or x86-64.
fn is_forbidden_instruction(mnemonic: &str, operands: &str) -> bool {
    const FORBIDDEN_MNEMONICS: &[&str] = &[
        // AArch64 Class I: hardware multipliers
        "mul",
        "madd",
        "msub",
        "mneg",
        "smnegl",
        "umnegl",
        "smull",
        "umull",
        "smaddl",
        "umaddl",
        "smsubl",
        "umsubl",
        "smulh",
        "umulh",
        "sqdmulh",
        "sqrdmulh",
        "sqrdmlah",
        "sqrdmlsh",
        "sqrdmlah2",
        "sqrdmlsh2",
        "mla",
        "mls",
        "pmul",
        "pmull",
        "pmull2",
        "sdot",
        "udot",
        "usdot",
        "sudot",
        "smlal",
        "smlsl",
        "umlal",
        "umlsl",
        "smull2",
        "umull2",
        "smlal2",
        "smlsl2",
        "umlal2",
        "umlsl2",
        "sqdmull",
        "sqdmull2",
        "sqdmlal",
        "sqdmlal2",
        "sqdmlsl",
        "sqdmlsl2",
        "smmla",
        "ummla",
        "usmmla",
        // AArch64 Class II: hardware dividers
        "sdiv",
        "udiv",
        // AArch64 Class III: floating-point & scalar float register transfer
        "fmul",
        "fmov",
        "fadd",
        "fsub",
        "fdiv",
        "fmadd",
        "fmsub",
        "fnmadd",
        "fnmsub",
        "fnmul",
        "fsqrt",
        "fcmp",
        "fcmpe",
        "scvtf",
        "ucvtf",
        "fneg",
        "fabs",
        "fmla",
        "fmls",
        "fmax",
        "fmin",
        "fmaxnm",
        "fminnm",
        "fmaxp",
        "fminp",
        "fmaxnmp",
        "fminnmp",
        "fmaxv",
        "fminv",
        "fmaxnmv",
        "fminnmv",
        "fmulx",
        "fabd",
        "faddp",
        "facge",
        "facgt",
        "fcmeq",
        "fcmge",
        "fcmgt",
        "fcmle",
        "fcmlt",
        "fcsel",
        "fccmp",
        "fccmpe",
        "frecpe",
        "frecps",
        "frecpx",
        "frsqrte",
        "frsqrts",
        "fcmla",
        "fcadd",
        "fdot",
        "fmlal",
        "fmlsl",
        "fmmla",
        "bfdot",
        "bfmmla",
        "bfmlal",
        "bfmlalb",
        "bfmlalt",
        // AArch64 indirect subroutine calls
        "blr",
        "blrda",
        "blrdb",
        "blraa",
        "blrab",
        // x86-64 integer multipliers & dividers (including AT&T size suffixes)
        "imul",
        "imulb",
        "imulw",
        "imull",
        "imulq",
        "mulb",
        "mulw",
        "mull",
        "mulq",
        "mulx",
        "mulxl",
        "mulxq",
        "idiv",
        "idivb",
        "idivw",
        "idivl",
        "idivq",
        "div",
        "divb",
        "divw",
        "divl",
        "divq",
        // x86-64 SSE/AVX float, comparison, conversion & packed integer multipliers
        "mulss",
        "mulsd",
        "mulps",
        "mulpd",
        "vmulss",
        "vmulsd",
        "vmulps",
        "vmulpd",
        "addss",
        "addsd",
        "addps",
        "addpd",
        "vaddss",
        "vaddsd",
        "vaddps",
        "vaddpd",
        "subss",
        "subsd",
        "subps",
        "subpd",
        "vsubss",
        "vsubsd",
        "vsubps",
        "vsubpd",
        "divss",
        "divsd",
        "divps",
        "divpd",
        "vdivss",
        "vdivsd",
        "vdivps",
        "vdivpd",
        "sqrtss",
        "sqrtsd",
        "sqrtps",
        "sqrtpd",
        "vsqrtss",
        "vsqrtsd",
        "vsqrtps",
        "vsqrtpd",
        "rsqrtss",
        "rsqrtps",
        "vrsqrtss",
        "vrsqrtps",
        "rcpss",
        "rcpps",
        "vrcpss",
        "vrcpps",
        "ucomiss",
        "ucomisd",
        "comiss",
        "comisd",
        "vucomiss",
        "vucomisd",
        "vcomiss",
        "vcomisd",
        "cmpss",
        "cmpsd",
        "cmpps",
        "cmppd",
        "vcmpss",
        "vcmpsd",
        "vcmpps",
        "vcmppd",
        "cvtsi2ss",
        "cvtsi2sd",
        "cvtsi2ssl",
        "cvtsi2ssq",
        "cvtsi2sdl",
        "cvtsi2sdq",
        "cvttss2si",
        "cvttsd2si",
        "cvtss2si",
        "cvtsd2si",
        "cvtss2sd",
        "cvtsd2ss",
        "cvtdq2ps",
        "cvtdq2pd",
        "cvtps2dq",
        "cvtpd2dq",
        "cvttps2dq",
        "cvttpd2dq",
        "cvtps2pd",
        "cvtpd2ps",
        "vcvtsi2ss",
        "vcvtsi2sd",
        "vcvttss2si",
        "vcvttsd2si",
        "vcvtdq2ps",
        "vcvtdq2pd",
        "vcvtps2dq",
        "vcvtpd2dq",
        "vcvttps2dq",
        "vcvttpd2dq",
        "vcvtps2pd",
        "vcvtpd2ps",
        "pmulld",
        "pmullq",
        "pmuludq",
        "pmaddwd",
        "pmullw",
        "pmulhw",
        "pmulhuw",
        "pmuldq",
        "pmaddubsw",
        "vpmulld",
        "vpmullq",
        "vpmuludq",
        "vpmaddwd",
        "vpmullw",
        "vpmulhw",
        "vpmulhuw",
        "vpmuldq",
        "vpmaddubsw",
        "vpdpbusd",
        "vpdpbusds",
        "vpdpwssd",
        "vpdpwssds",
    ];

    let base = mnemonic.split('.').next().unwrap_or(mnemonic);
    if FORBIDDEN_MNEMONICS.contains(&base) {
        return true;
    }
    if base.starts_with("fcvt")
        || base.starts_with("frint")
        || base.starts_with("cvt")
        || base.starts_with("vcvt")
        || base.starts_with("vfmadd")
        || base.starts_with("vfmsub")
        || base.starts_with("vfnmadd")
        || base.starts_with("vfnmsub")
    {
        return true;
    }
    if (base.starts_with('f') || base.starts_with("bf")) && mnemonic.contains('.') {
        return true;
    }
    // Reject x86-64 indirect subroutine calls (`call *%rax` / `callq *(%rdi)`).
    if matches!(base, "call" | "callq") && operands.trim_start().starts_with('*') {
        return true;
    }
    // Check 16-bit (`h0..h31`) and 32-bit (`s0..s31`) scalar float loads/stores. (Note: on AArch64
    // NEON, 64-bit 2x`i32` integer SIMD instructions such as `sqadd.2s v0, v0, v1` in `stack_step`
    // use the `d0..d31` 64-bit register alias for 64-bit integer vector loads/stores, matching
    // `scripts/audit_zero_matmul_serving.py` and disclosed in `disassembly_audit.scope`.)
    if matches!(base, "ldr" | "str" | "ldur" | "stur" | "ldp" | "stp") {
        let code_operands = operands.split(';').next().unwrap_or(operands);
        if code_operands
            .split(|c: char| c == ',' || c.is_whitespace())
            .any(is_scalar_float_register_token)
        {
            return true;
        }
    }
    // Direct calls to compiler-rt / libgcc software multiply, divide, or float helpers.
    if matches!(base, "bl" | "b" | "call" | "callq" | "jmp" | "jmpq") {
        const FORBIDDEN_COMPILER_RT: &[&str] = &[
            "__multi3",
            "__muldi3",
            "__mulodi4",
            "__udivti3",
            "__divti3",
            "__umodti3",
            "__modti3",
            "__udivdi3",
            "__divdi3",
            "__floatditf",
            "__floatunditf",
            "__floattidf",
            "__floatuntidf",
            "__addtf3",
            "__subtf3",
            "__multf3",
            "__divtf3",
        ];
        if FORBIDDEN_COMPILER_RT.iter().any(|rt| operands.contains(rt)) {
            return true;
        }
    }
    false
}

/// Verify that disassembled text from `objdump --demangle -d` or `otool -tV` contains
/// `IntegerStackSession::step`, its 42 mandatory `uor_r4_integer::stack` mixer and kernel callees,
/// all `uor_r4_integer::stack` closure instantiations, and all non-allowlisted functions reachable
/// in the transitive call graph (`core`, `alloc`, `compiler_builtins`, `uor_r4_integer`) with zero
/// Class I (multiply), Class II (divide), or Class III (floating-point / SIMD transfer) instructions.
///
/// When `require_full_call_graph` is true, requires all 43 mandatory symbols
/// (`IntegerStackSession::step` + all 42 `STACK_CALL_GRAPH_KERNELS`) with zero missing symbols.
pub fn verify_step_disassembly_text(
    disasm_text: &str,
    require_full_call_graph: bool,
) -> Result<(usize, usize), String> {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    let mut functions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current_sym: Option<String> = None;

    for raw_line in disasm_text.lines() {
        let line = raw_line.trim();
        if line.ends_with(">:") {
            if let Some(start) = line.find('<') {
                let sym = line[start + 1..line.len() - 2].to_string();
                functions.entry(sym.clone()).or_default();
                current_sym = Some(sym);
                continue;
            }
        } else if line.ends_with(':') && !line.contains('\t') && !line.contains(' ') {
            let sym = line[..line.len() - 1].to_string();
            functions.entry(sym.clone()).or_default();
            current_sym = Some(sym);
            continue;
        }

        if let Some(ref sym) = current_sym {
            if extract_mnemonic_and_operands(line).is_some() {
                if let Some(lines) = functions.get_mut(sym) {
                    lines.push(line.to_string());
                }
            }
        }
    }

    let step_syms: Vec<String> = functions
        .keys()
        .filter(|s| is_step_root_symbol(s))
        .cloned()
        .collect();
    if step_syms.is_empty() {
        return Err("IntegerStackSession::step root symbol not found in disassembly".to_string());
    }

    let mut missing_mandatory = Vec::new();
    let mut audited_set: BTreeSet<String> = BTreeSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    for s in &step_syms {
        if audited_set.insert(s.clone()) {
            queue.push_back(s.clone());
        }
    }

    for &kernel in STACK_CALL_GRAPH_KERNELS {
        let matched: Vec<String> = functions
            .keys()
            .filter(|s| matches_mandatory_stack_kernel(s, kernel))
            .cloned()
            .collect();
        if matched.is_empty() {
            if require_full_call_graph {
                missing_mandatory.push(kernel);
            }
        } else {
            for m in matched {
                if audited_set.insert(m.clone()) {
                    queue.push_back(m);
                }
            }
        }
    }

    if !missing_mandatory.is_empty() {
        return Err(format!(
            "missing {} mandatory stack serving symbol(s) in full call-graph disassembly: {}",
            missing_mandatory.len(),
            missing_mandatory.join(", ")
        ));
    }

    // Include every `uor_r4_integer::stack` closure or rayon/crossbeam scheduler instantiation over
    // a stack closure present in the binary (matching `audit_zero_matmul_serving.py`).
    for sym in functions.keys() {
        let is_stack_or_closure = is_stack_serving_or_closure_symbol(sym)
            || (!sym.contains("measure_step_phases")
                && (sym.contains("uor_r4_integer::stack::session::stack_")
                    || sym.contains("uor_r4_integer::stack::kernels::stack_")
                    || sym.contains("14uor_r4_integer5stack7session")
                    || sym.contains("14uor_r4_integer5stack7kernels")));
        if is_stack_or_closure && audited_set.insert(sym.clone()) {
            queue.push_back(sym.clone());
        }
    }

    // Walk the transitive call graph (BFS) across all non-allowlisted reachable callees in the binary.
    while let Some(curr) = queue.pop_front() {
        if let Some(lines) = functions.get(&curr) {
            for line in lines {
                if let Some((mnemonic, operands)) = extract_mnemonic_and_operands(line) {
                    if let Some(callee) = extract_call_target(&mnemonic, &operands, &curr) {
                        if functions.contains_key(&callee)
                            && !is_call_graph_allowlisted(&callee)
                            && audited_set.insert(callee.clone())
                        {
                            queue.push_back(callee);
                        }
                    }
                }
            }
        }
    }

    let mut instructions_checked = 0usize;
    let mut violations = Vec::new();
    for sym in &audited_set {
        let is_rayon_or_tls_wrapper = sym.contains("rayon_core::")
            || sym.contains("10rayon_core")
            || sym.contains("LocalKey")
            || sym.contains("thread_local");
        let is_stack_session = sym.contains("uor_r4_integer::stack::session")
            || sym.contains("14uor_r4_integer5stack7session");
        if let Some(lines) = functions.get(sym) {
            let mut prev_tlv_thunk_reg: Option<String> = None;
            for line in lines {
                if let Some((mnemonic, operands)) = extract_mnemonic_and_operands(line) {
                    instructions_checked += 1;
                    let base = mnemonic.split('.').next().unwrap_or(&mnemonic);
                    let is_indirect_call =
                        matches!(base, "blr" | "blrda" | "blrdb" | "blraa" | "blrab")
                            || (matches!(base, "call" | "callq")
                                && operands.trim_start().starts_with('*'));
                    let is_inlined_macho_tlv_thunk = is_stack_session
                        && base == "blr"
                        && prev_tlv_thunk_reg.as_deref() == Some(operands.trim());
                    prev_tlv_thunk_reg = if base == "ldr" {
                        let compact = operands.replace(' ', "");
                        compact
                            .strip_suffix(",[x0]")
                            .filter(|r| r.starts_with('x'))
                            .map(str::to_string)
                    } else {
                        None
                    };
                    if is_indirect_call && (is_rayon_or_tls_wrapper || is_inlined_macho_tlv_thunk) {
                        // Rayon work-stealing and TLS runtime wrappers (`rayon_core::*`,
                        // `LocalKey::with`, `std::sys::thread_local::*`, and inlined Mach-O
                        // `TLVDescriptor::thunk` `ldr xN, [x0]; blr xN` sequences in `stack::session`
                        // for `WORKER_THREAD_STATE`) are permitted; every `uor_r4_integer::stack`
                        // closure in the binary is seeded directly into `audited_set` above, and
                        // all `stack::kernels::*`, `core::*`, and `alloc::*` functions forbid
                        // indirect calls unconditionally.
                        continue;
                    }
                    if is_forbidden_instruction(&mnemonic, &operands) {
                        violations.push(format!("{sym}: {mnemonic} in `{line}`"));
                    }
                }
            }
        }
    }

    let symbols_checked = audited_set.len();
    if symbols_checked == 0 || instructions_checked == 0 {
        return Err(format!(
            "no instructions checked in IntegerStackSession::step call graph (symbols={symbols_checked}, instructions={instructions_checked})"
        ));
    }
    if !violations.is_empty() {
        return Err(format!(
            "forbidden multiply/divide/float instructions in stack serving call-graph disassembly: {}",
            violations.join("; ")
        ));
    }
    Ok((symbols_checked, instructions_checked))
}

/// Run `objdump --demangle -d` (falling back to `otool -tV`) on `binary_path` and verify that
/// `IntegerStackSession::step`, all 42 mandatory `uor_r4_integer::stack` mixer and kernel symbols,
/// all stack closure instantiations, and all reachable non-allowlisted callees in the transitive
/// call graph contain zero multiply, divide, or floating-point instructions.
#[cfg(not(target_arch = "wasm32"))]
pub fn verify_step_objdump_zero_mul_zero_float(
    binary_path: &Path,
) -> Result<(usize, usize), String> {
    let disasm = std::process::Command::new("objdump")
        .args(["--demangle", "-d"])
        .arg(binary_path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .or_else(|| {
            std::process::Command::new("otool")
                .args(["-tV"])
                .arg(binary_path)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .ok_or_else(|| {
            format!(
                "failed to disassemble {} with objdump or otool",
                binary_path.display()
            )
        })?;
    verify_step_disassembly_text(&disasm, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-report-output-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn report_output_new_path_is_claimed_and_sealed_normally() {
        let base = scratch("new");
        let out = base.join("candidate-a").join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        assert!(out.join(ATTEMPT_FILE).is_file());
        fs::write(out.join("result.json"), b"{\"exact\":1}").unwrap();
        fs::create_dir(out.join("nested")).unwrap();
        fs::write(out.join("nested").join("rows.jsonl"), b"{}\n").unwrap();
        let manifest_path = seal(&out).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        let files = manifest["files"].as_array().unwrap();
        let names: Vec<_> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![ATTEMPT_FILE, "nested/rows.jsonl", "result.json"]
        );
        assert_eq!(
            files[2]["blake3"],
            blake3::hash(b"{\"exact\":1}").to_hex().to_string()
        );
        assert!(seal(&out).is_err(), "sealing twice must fail");
        assert!(
            verify(&out).unwrap().is_empty(),
            "a complete sealed set verifies"
        );
        // A file added beneath a sealed directory is detected as unlisted.
        fs::write(out.join("derived-input.json"), b"[]").unwrap();
        let error = verify(&out).unwrap_err().to_string();
        assert!(
            error.contains("unlisted") && error.contains("derived-input.json"),
            "{error}"
        );
        fs::remove_file(out.join("derived-input.json")).unwrap();
        // A changed listed file is detected before the file set is judged.
        fs::write(out.join("result.json"), b"{\"exact\":0}").unwrap();
        assert!(verify(&out)
            .unwrap_err()
            .to_string()
            .contains("result.json"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn report_output_existing_report_fails_before_work_and_stays_byte_identical() {
        let base = scratch("collision");
        let out = base.join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        let report = out.join("result.json");
        fs::write(&report, b"original rejected-candidate rows").unwrap();
        let sentinel_before = fs::read(out.join(ATTEMPT_FILE)).unwrap();
        let error = claim(&out).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains("new attempt path"));
        assert_eq!(
            fs::read(&report).unwrap(),
            b"original rejected-candidate rows"
        );
        assert_eq!(fs::read(out.join(ATTEMPT_FILE)).unwrap(), sentinel_before);
        // A plain pre-existing directory without a sentinel is also refused.
        let plain = base.join("plain");
        fs::create_dir(&plain).unwrap();
        fs::write(plain.join("result.json"), b"keep").unwrap();
        assert_eq!(
            claim(&plain).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(plain.join("result.json")).unwrap(), b"keep");
        // A distinct attempt path beside it completes normally.
        let next = base.join("evaluate").join("attempt-2");
        claim(&next).unwrap();
        assert!(next.join(ATTEMPT_FILE).is_file());
        fs::remove_dir_all(base).unwrap();
    }
}
