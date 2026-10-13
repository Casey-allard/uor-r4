//! Fixed D29 qualification controller; Rust orchestration, never model math.
//! Build with rustc --edition=2021 on the admitted pod. Requires a FIT sheet
//! and exact source pin. Runs only the 31 frozen tests, never an optimizer.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitCode, Stdio},
};

// Only this owned telemetry child is stopped on completion/error. The samples
// are observed GPU memory use, not a claim of an unsampled absolute peak.
struct DeviceSamples(Child);
impl DeviceSamples {
    fn start(report: &Path) -> Result<Self, String> {
        let out =
            fs::File::create(report.join("gpu-memory-samples.csv")).map_err(|e| e.to_string())?;
        let err = fs::File::create(report.join("gpu-memory-samples.stderr.log"))
            .map_err(|e| e.to_string())?;
        Command::new("nvidia-smi")
            .args([
                "--query-gpu=timestamp,uuid,memory.used,utilization.gpu",
                "--format=csv",
                "--loop-ms=200",
            ])
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()
            .map(Self)
            .map_err(|e| e.to_string())
    }
}
impl Drop for DeviceSamples {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

const LIB: &[&str] = &[
    "geometric_stack::tests::geometric_rate_cuda_uses_signed_edges_and_exact_keep_table",
    "geometric_stack::tests::geometric_cuda_clipping_matches_stored_analytic_values",
    "geometric_stack::tests::geometric_rung0_cuda_composition_matches_zero_carrier_golden",
    "geometric_stack::tests::geometric_rung0_cuda_ordinary_path_has_learning_credit",
    "native_vector_writer::contract_tests::binding_and_cpu_refusal",
    "native_vector_writer::cuda_tests::ties_identity_and_shared_channel",
    "native_vector_writer::cuda_tests::local_distance_adjoints_and_omitted_target",
    "native_vector_writer::cuda_tests::numerical_and_device_boundaries",
    "native_vector_writer::cuda_tests::device_metrics_keep_token_and_address_bars_distinct",
    "native_vector_writer::cuda_tests::rung0_full_batch_writer_forward_backward_without_updates",
    "native_vector_writer::cuda_tests::actual_stack_store_channel_and_artifact_identity",
    "spiralcore_fbs::native_cuda_tests::native_cuda_frontier_forward_exact_input_adjoint_and_secant",
    "spiralcore_fbs::native_cuda_tests::native_cuda_complete_ternary_children_and_signed_orientation",
    "spiralcore_fbs::native_cuda_tests::native_cuda_shared_batch_and_output_credit",
    "spiralcore_fbs::native_cuda_tests::native_cuda_numerical_and_shape_refusals",
];
const CLI: &[&str] = &[
    "native_rung0::tests::document_boundaries_and_overlap_tail_score_every_mask_once",
    "native_rung0::tests::exact_window_and_shifted_mask_contract_is_metadata_only",
    "native_rung0::tests::masked_count_inventory_is_validated_metadata_only",
    "native_rung0::tests::uort_header_inventory_is_not_token_payload_or_tail",
    "native_rung0_argument_tests::complete_native_argument_contract_refuses_legacy_modes",
    "native_rung0_argument_tests::native_metric_record_never_mislabels_raw_margins_as_likelihood",
    "native_rung0_argument_tests::native_process_flags_refuse_sampling_host_diagnostics_and_bad_estimates",
    "native_rung0_argument_tests::native_reference_metadata_rejects_invalid_masks_and_count_claims",
    "native_rung0_argument_tests::native_train_reference_arguments_require_one_masked_stream_and_positive_count",
    "native_rung0::cuda_fixed_tests::cuda_masked_train_reference_excludes_unmasked_ids_and_separates_dev_oracle",
    "native_rung0::cuda_fixed_tests::cuda_document_evaluator_known_answers_and_native_artifact_roundtrip",
];
const MODULE: &[&str] = &[
    "compiled_sources_carry_no_banned_operation",
    "no_excluded_historical_kernel_is_compiled",
    "nothing_in_the_crate_calls_a_runtime_compiler",
    "quarantined_dense_contraction_modules_stay_out_of_the_build",
    "built_ptx_modules_carry_no_banned_instruction",
];

fn capture(program: &Path, args: &[&str], report: &Path, label: &str) -> Result<String, String> {
    let start = std::time::Instant::now();
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    let text = format!(
        "command: {} {:?}\nstatus: {}\nelapsed_s: {:.6}\nstdout:\n{}\nstderr:\n{}",
        program.display(),
        args,
        output.status,
        start.elapsed().as_secs_f64(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(report.join(format!("{label}.log")), &text).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("{label} failed; preserved in {}", report.display()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 7 {
        return Err("usage: qualify-geometric REPO SOURCE_SHA LIB_TEST CLI_TEST REPORT_ROOT FIT_RECEIPT_URL MODULE_TEST".into());
    }
    if !cfg!(target_os = "linux")
        || env::var("CUDA_VISIBLE_DEVICES")
            .unwrap_or_default()
            .is_empty()
    {
        return Err(
            "qualification requires the admitted Linux CUDA pod and explicit CUDA_VISIBLE_DEVICES"
                .into(),
        );
    }
    let repo = Path::new(&args[0])
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if args[1].len() != 40 || !args[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SOURCE_SHA must be the full frozen git commit".into());
    }
    if !args[5].starts_with("https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-") {
        return Err("FIT receipt must identify the prospective milestone comment".into());
    }
    env::set_current_dir(&repo).map_err(|e| e.to_string())?;
    let sheet =
        fs::read_to_string(repo.join("docs/harness/prep/m1-d29-vendor-finite-2026-10-13.md"))
            .map_err(|e| e.to_string())?;
    if !sheet.lines().any(|l| l.trim() == "TEST FITNESS: FIT") {
        return Err("finite sheet is not FIT; no controls executed".into());
    }
    let report = PathBuf::from(&args[4]);
    if !report.is_absolute() || !report.starts_with("/workspace/uor-r4/jobs") {
        return Err(
            "exclusive report must be under /workspace/uor-r4/jobs for admitted preservation"
                .into(),
        );
    }
    fs::create_dir(&report).map_err(|e| format!("exclusive report: {e}"))?;
    fs::write(
        report.join("scope.txt"),
        format!(
            "source={}\nFIT={}\ntraining=HELD\noptimizer_updates=0\ncontrols=31\n",
            args[1], args[5]
        ),
    )
    .map_err(|e| e.to_string())?;
    let head = capture(Path::new("git"), &["rev-parse", "HEAD"], &report, "source")?;
    if head.trim() != args[1] {
        return Err("source pin mismatch".into());
    }
    let dirty = capture(
        Path::new("git"),
        &["status", "--porcelain", "--untracked-files=no"],
        &report,
        "tracked-status",
    )?;
    if !dirty.is_empty() {
        return Err("tracked source is dirty".into());
    }
    capture(
        Path::new("nvidia-smi"),
        &[
            "--query-gpu=name,uuid,driver_version,memory.total",
            "--format=csv",
        ],
        &report,
        "device",
    )?;
    let binaries = [
        Path::new(&args[2])
            .canonicalize()
            .map_err(|e| e.to_string())?,
        Path::new(&args[3])
            .canonicalize()
            .map_err(|e| e.to_string())?,
        Path::new(&args[6])
            .canonicalize()
            .map_err(|e| e.to_string())?,
    ];
    let target = binaries[0]
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("library test does not have a Cargo target/debug/deps parent")?;
    if !target.starts_with("/workspace") || !binaries.iter().all(|p| p.starts_with(target)) {
        return Err("test binaries must share the owned /workspace target directory".into());
    }
    // Tests use this variable only as their artifact-output parent. Keep the
    // tiny round-trip artifacts with the archived report, not the build cache.
    let fixture_root = report.join("fixture-artifacts");
    fs::create_dir(&fixture_root).map_err(|e| e.to_string())?;
    env::set_var("CARGO_TARGET_DIR", &fixture_root);
    env::set_var("UOR_FINITE_REPORT_ROOT", &report);
    capture(
        Path::new("sha256sum"),
        &[
            binaries[0].to_str().ok_or("non-UTF8 lib path")?,
            binaries[1].to_str().ok_or("non-UTF8 cli path")?,
            binaries[2].to_str().ok_or("non-UTF8 module path")?,
        ],
        &report,
        "executables",
    )?;
    // Preflight every exact test before any numerical execution. A zero-test
    // success or ignored test is never accepted as qualification.
    for (i, names) in [LIB, CLI, MODULE].iter().enumerate() {
        let list = capture(&binaries[i], &["--list"], &report, &format!("list-{i}"))?;
        for name in *names {
            if list
                .lines()
                .filter(|line| *line == format!("{name}: test"))
                .count()
                != 1
            {
                return Err(format!("missing or ambiguous frozen control {name}"));
            }
        }
    }
    let _samples = DeviceSamples::start(&report)?;
    let mut passed = 0;
    for (i, names) in [LIB, CLI, MODULE].iter().enumerate() {
        for name in *names {
            let out = capture(
                &binaries[i],
                &[name, "--exact", "--nocapture", "--test-threads=1"],
                &report,
                &format!("control-{:02}", passed + 1),
            )?;
            if !out.contains("test result: ok. 1 passed; 0 failed; 0 ignored;")
                || out.contains("D29:")
            {
                return Err(format!(
                    "control {name} did not execute exactly one passing nonignored test"
                ));
            }
            passed += 1;
            println!("PASS {passed}/31 {name}");
        }
    }
    fs::write(
        report.join("RESULT.txt"),
        "31/31 fixed controls PASS\nTraining HELD; no optimizer or corpus run\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("BLOCKER: {error}");
            ExitCode::FAILURE
        }
    }
}
