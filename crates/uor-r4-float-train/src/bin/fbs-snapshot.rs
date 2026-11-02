//! Convert a saved SpiralCore/FBS float trunk (`StackModel::save`) to the
//! integer crate's snapshot JSON (d11-exporter-float-trunk, claude-123).
//!
//! It runs on one GPU, because the routes are read back through the model's
//! CUDA projection (`fbs_export`). The serving build's `fbs-export` turns the
//! JSON into the `UORLUT01` artifact. The optional readouts, for the serving
//! build's `fbs-compare`, are the float trunk's own outputs, in order:
//! - `READOUT_JSON`: plain;
//! - `EXPORT_READOUT_JSON`: in the float sheets' D11 export mode;
//! - `D11_READOUT_JSON`: in the D11 mode, after rounding the loaded trunk's
//!   tables as exported (claude-125).
//!
//! ```text
//! fbs-snapshot [--dev DATA_ROOT] MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]
//! fbs-snapshot [--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON
//! ```
//!
//! `--dev DATA_ROOT` (claude-240) takes every readout, and the trace, on a sample of DEV in
//! place of the fixed sequences: `DATA_ROOT/ft/dev`, loaded with its pins, cut into the
//! trainer's windows of `SHEET_CONTEXT` and sampled every `PARITY_WINDOW_STRIDE`-th window, as
//! claude-126 sampled it. The snapshot itself does not depend on it.
//!
//! The second form (claude-235) writes the D11 readout of a checkpoint with
//! Hamming reads and what each read chose on its fixed sequences
//! (`uor-r4.hamming-trace/1`), for one arm:
//! - `d11`: with the integer session's tie rule turned off, so the GPU sort's own order breaks
//!   the reads' ties (the first form's D11 readout before claude-239);
//! - `d11-tie`: with the rule, as the first form's D11 readout since claude-239, the rule's
//!   default;
//! - `d11-rounded`: `d11` with the reads' parameters rounded as exported;
//! - `d11-tie-rounded`: both.
//!
//! The third form (claude-240) makes an evaluation copy of a checkpoint saved before claude-179:
//!
//! ```text
//! fbs-snapshot --rebind-fbs-backward MODEL_DIR OUT_MODEL_DIR
//! ```
//!
//! claude-179 (#3367) changed only the projection's backward kernel, so the forward a readout
//! runs is unchanged. But each plan's fingerprint hashes the whole `spiralcore_fbs.cu`, and
//! `StackModel::load` refuses the older checkpoints. The copy's config names the kernel as it is
//! now, and its tensors and run config are copied byte for byte. The copy is refused unless:
//! - the kernel's text through `spiralcore_fbs_fwd` is the text before claude-179;
//! - every saved fingerprint (the plans', and the vector head's `geometry`, the 8/8 plan's) is
//!   the current plan's with only the kernel's hash put back;
//! - the copy then loads, with every check `StackModel::load` makes.
//!
//! The fourth form (claude-240) is `train-in-space --inspect` of one run, with the run config's
//! rate, fold and floor readings and seed, and the reads' ties broken one way:
//!
//! ```text
//! fbs-snapshot --inspect as-trained|tie-rule DATA_ROOT RUN_DIR
//! ```
//!
//! - `as-trained`: the GPU sort's own order, as a checkpoint saved before claude-239 trained and
//!   was inspected. On a rebound copy, its lines are compared with the run's sealed inspection;
//! - `tie-rule`: the integer session's rule, the read's default since claude-239.
//!
//! The fifth form (claude-252, Q5's side T) makes an evaluation copy of a checkpoint with every
//! parameter the integer artifact holds rounded as `fbs-export --raw-gates --raw-transition`
//! rounds it (`fbs_export::write_rounded_copy`), on the CPU:
//!
//! ```text
//! fbs-snapshot --round-as-exported MODEL_DIR OUT_MODEL_DIR
//! ```
//!
//! The trunk's tables, the Hamming reads and the turn memory's weights are rounded; the routes,
//! the memory's tables and both configs are as they are. The copy's step holds the marker
//! `rounded-as-exported.json`, with each file's sha256 before and after, on which
//! `train-in-space --score-rows --decision-trace` writes side T.
//!
//! It never replaces or leaves a partial output file, and it prints each
//! file's sha256.
//!
//! The snapshot carries every read the checkpoint holds, or it is refused
//! (claude-198): the turn memory (`turn_memory*.safetensors`) and the Hamming
//! reads (`hamming_read.safetensors`, claude-200) are carried, and any other
//! read file (the Flock, content and opening reads, the prompt register) is
//! refused by name until its export exists.

use std::path::{Path, PathBuf};

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::{
    check_checkpoint_reads, check_readout_reads, fbs_float_readout, fbs_float_readout_on,
    fbs_hamming_snapshot, fbs_memory_snapshot, fbs_trunk_snapshot, round_hamming_as_exported,
    round_tables_as_exported, run_rate_reading, write_rounded_copy, READOUT_SEQUENCES,
};
use uor_r4_float_train::flock_read_hamming::{
    set_hamming_reads, set_hamming_tie_rule, set_hamming_trace, take_hamming_trace, HammingReads,
    HAMMING_DEPTHS, HAMMING_KEPT,
};
use uor_r4_float_train::flock_read_memory::TurnMemory;
use uor_r4_float_train::geometric_stack::{set_retention_rates, StackModel};
use uor_r4_float_train::hamilton_sheet::{
    build_document_windows, load_pinned_uort_split, CausalWindow, EXPECTED_DEV_ELIGIBLE_TARGETS,
    EXPECTED_DEV_TOKENS, PINNED_DEV_MASK_SHA256, PINNED_DEV_TOKENS_SHA256, SHEET_BOS_ID,
    SHEET_CONTEXT, SHEET_VOCAB,
};
use uor_r4_float_train::in_space_long::{inspect_run, root_separation};
use uor_r4_float_train::spiralcore_fbs::{ProjectionPlan, FBS_SOURCE_SHA256, SCHEMA};
use uor_r4_float_train::spiralcore_fbs_arms::FbsArm;
use uor_r4_float_train::spiralcore_fbs_sheet::PARITY_WINDOW_STRIDE;
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_snapshot::{
    run_readout_nearest, write_new_file, FbsFloatReadout, FbsReadoutMode, HammingTrace,
    HAMMING_TRACE_SCHEMA,
};

fn write(path: &str, json: &[u8]) -> Result<()> {
    write_new_file(Path::new(path), json)?;
    println!("{}  {path}", hex::encode(Sha256::digest(json)));
    Ok(())
}

/// The run's `config.json`, two levels above `MODEL_DIR` (`models/step-N`).
fn run_config(dir: &Path) -> Result<String> {
    let path = dir
        .parent()
        .and_then(Path::parent)
        .map(|run| run.join("config.json"))
        .ok_or_else(|| {
            FloatTrainError::Invalid("MODEL_DIR is not a run's models/step-N".to_owned())
        })?;
    Ok(std::fs::read_to_string(path)?)
}

/// A trace arm's switches: the tie rule, and the reads rounded as exported.
fn trace_arm(name: &str) -> Option<(bool, bool)> {
    match name {
        "d11" => Some((false, false)),
        "d11-tie" => Some((true, false)),
        "d11-rounded" => Some((false, true)),
        "d11-tie-rounded" => Some((true, true)),
        _ => None,
    }
}

/// Where an output that does not exist yet would be: its folder, resolved, and its name.
fn output_place(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let folder = path
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| FloatTrainError::Invalid(format!("{} names no file", path.display())))?;
    Ok(std::fs::canonicalize(folder)?.join(name))
}

/// claude-240: every `PARITY_WINDOW_STRIDE`-th window, from the first (claude-126's rule).
fn sample_windows<T>(windows: Vec<T>) -> Vec<T> {
    windows.into_iter().step_by(PARITY_WINDOW_STRIDE).collect()
}

/// claude-240: `--dev DATA_ROOT`'s sample. DEV is loaded with its pins and cut into the
/// trainer's windows (`in_space_long.rs`), and every `PARITY_WINDOW_STRIDE`-th is kept. It is
/// read before the GPU opens, so a wrong data root costs no GPU time.
fn dev_sample(data_root: &str) -> Result<Vec<Vec<u32>>> {
    let dev = Path::new(data_root).join("ft/dev");
    let (tokens, mask) = load_pinned_uort_split(
        &dev.join("tokens.u16"),
        &dev.join("response_mask.u8"),
        PINNED_DEV_TOKENS_SHA256,
        PINNED_DEV_MASK_SHA256,
        EXPECTED_DEV_TOKENS,
        EXPECTED_DEV_ELIGIBLE_TARGETS,
        SHEET_VOCAB,
        SHEET_BOS_ID,
    )?;
    let (windows, _) = build_document_windows(&tokens, &mask, SHEET_CONTEXT, SHEET_BOS_ID)?;
    let scored: usize = windows.iter().map(|w| w.scored_targets.len()).sum();
    if scored != EXPECTED_DEV_ELIGIBLE_TARGETS {
        return Err(FloatTrainError::Invalid(format!(
            "DEV's windows score {scored} targets, not {EXPECTED_DEV_ELIGIBLE_TARGETS}"
        )));
    }
    let all = windows.len();
    let sample: Vec<Vec<u32>> = sample_windows(windows).iter().map(scored_prefix).collect();
    let positions: usize = sample.iter().map(Vec::len).sum();
    // Up to t = HAMMING_KEPT + 1 a read keeps every earlier position, so only the later ones
    // can choose: the parity bars count those (the prep sheet's P2).
    let choosing: usize = sample
        .iter()
        .map(|w| w.len().saturating_sub(HAMMING_KEPT + 2))
        .sum();
    println!(
        "dev: {all} windows, {} sampled, {positions} positions, {choosing} past t = {}",
        sample.len(),
        HAMMING_KEPT + 1
    );
    Ok(sample)
}

/// A window's inputs up to its last scored position. The trainer pads each window with BOS to
/// `SHEET_CONTEXT`, and the model is causal, so the cut changes no scored prediction and keeps the
/// padding out of the parity counts. `build_document_windows` keeps only windows that score.
fn scored_prefix(window: &CausalWindow) -> Vec<u32> {
    let last = window.scored_offsets.iter().max().copied().unwrap_or(0);
    window.input_ids[..=last].to_vec()
}

/// A readout on the DEV sample when there is one, else on the fixed sequences. The trainer cut
/// DEV at `SHEET_CONTEXT`, so a model with another context, or a window past it, is refused.
fn readout_on(
    model: &StackModel,
    mode: FbsReadoutMode,
    dev: Option<&[Vec<u32>]>,
) -> Result<FbsFloatReadout> {
    let Some(sample) = dev else {
        return fbs_float_readout(model, mode);
    };
    let context = model.config.context;
    if context != SHEET_CONTEXT || sample.iter().any(|w| w.is_empty() || w.len() > context) {
        return Err(FloatTrainError::Invalid(format!(
            "the model's context is {context}, and DEV was cut at {SHEET_CONTEXT} into windows of 1 to {SHEET_CONTEXT}"
        )));
    }
    fbs_float_readout_on(model, mode, sample)
}

/// `[--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON` (claude-235;
/// the DEV sample, claude-240).
fn hamming_trace(
    arm: &str,
    model_dir: &str,
    readout: &str,
    trace: &str,
    dev: Option<&[Vec<u32>]>,
) -> Result<()> {
    let (tie, rounded) = trace_arm(arm).ok_or_else(|| {
        FloatTrainError::Invalid(format!(
            "unknown trace arm {arm}: d11, d11-tie, d11-rounded or d11-tie-rounded"
        ))
    })?;
    // Both outputs are refused before the GPU runs, so a run never leaves a readout without its
    // trace. They are compared by where they would be, so `./t.json` and `t.json` are one file.
    if output_place(readout)? == output_place(trace)? {
        return Err(FloatTrainError::Invalid(format!(
            "D11_READOUT_JSON and TRACE_JSON are both {readout}"
        )));
    }
    if let Some(path) = [readout, trace].into_iter().find(|p| Path::new(p).exists()) {
        return Err(FloatTrainError::Invalid(format!(
            "{path} exists, and the trace never replaces a file"
        )));
    }
    let dir = Path::new(model_dir);
    check_checkpoint_reads(dir)?;
    check_readout_reads(dir, 1)?;
    set_retention_rates(run_rate_reading(&run_config(dir)?)?);
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(dir, &device)?;
    let reads = HammingReads::load(dir, &device)?.ok_or_else(|| {
        FloatTrainError::Invalid("the checkpoint has no Hamming reads to trace".to_owned())
    })?;
    if rounded {
        round_hamming_as_exported(&reads)?;
    }
    set_hamming_reads(Some(reads));
    set_hamming_tie_rule(tie);
    round_tables_as_exported(&model)?;
    set_hamming_trace(true);
    let d11 = readout_on(&model, FbsReadoutMode::D11, dev)?;
    let windows = take_hamming_trace().unwrap_or_default();
    set_hamming_trace(false);
    // Only the sequences reach a read (a single token has no earlier position): each must be
    // traced once, at every read depth.
    let sequences = dev.map_or(READOUT_SEQUENCES, <[Vec<u32>]>::len);
    let depths: Vec<Vec<usize>> = windows
        .iter()
        .map(|w| w.reads.iter().map(|r| r.depth).collect())
        .collect();
    if windows.len() != sequences || depths.iter().any(|d| d[..] != HAMMING_DEPTHS[..]) {
        return Err(FloatTrainError::Invalid(format!(
            "the trace holds {} windows with depths {depths:?}, not {sequences} at {HAMMING_DEPTHS:?}",
            windows.len()
        )));
    }
    let heads = windows[0].reads[0].heads.len();
    let trace_json = serde_json::to_vec(&HammingTrace {
        schema: HAMMING_TRACE_SCHEMA.to_owned(),
        side: "float-d11".to_owned(),
        arm: arm.to_owned(),
        heads,
        kept: HAMMING_KEPT,
        windows,
    })?;
    write(readout, &serde_json::to_vec(&d11)?)?;
    write(trace, &trace_json)
}

/// claude-240: `spiralcore_fbs.cu` before claude-179 (as at `df1d7892`, where claude-146 trained),
/// and its text through the end of `spiralcore_fbs_fwd`, which claude-179 left unchanged.
const PRE_179_KERNEL_SHA256: &str =
    "517bffcc3c2029ddd171e75fd8fc1eb532755b30e683c66bd230382416480b0d";
const FORWARD_TEXT_SHA256: &str =
    "c98de18bc4724f682da5dc68712a76c2d4c80003e0e48b937185450ddea4c680";

/// A projection kernel's text through the end of `spiralcore_fbs_fwd`: up to the first line `}`
/// after its header. Every function the forward calls is defined above it.
fn forward_text(kernel: &str) -> Option<&str> {
    let header = kernel.find("__global__ void spiralcore_fbs_fwd(")?;
    let end = header + kernel[header..].find("\n}\n")? + 3;
    Some(&kernel[..end])
}

/// The fingerprint a plan had before claude-179: its serialized form, as `ProjectionPlan::new`
/// hashes it, with the kernel's hash put back to `PRE_179_KERNEL_SHA256`. Refused unless the
/// serialized form is the fingerprint's own and names the current kernel exactly once.
fn pre_179_fingerprint(plan: &ProjectionPlan, kernel_sha256: &str) -> Result<String> {
    let json = serde_json::to_string(plan)?;
    if hex::encode(Sha256::digest(json.as_bytes())) != plan.fingerprint()
        || json.matches(kernel_sha256).count() != 1
    {
        return Err(FloatTrainError::Invalid(
            "a plan's serialized form is not its fingerprint's, or names the kernel twice"
                .to_owned(),
        ));
    }
    let legacy = json.replace(kernel_sha256, PRE_179_KERNEL_SHA256);
    Ok(hex::encode(Sha256::digest(legacy.as_bytes())))
}

/// `--rebind-fbs-backward MODEL_DIR OUT_MODEL_DIR` (claude-240): the module docs' third form.
fn rebind_fbs_backward(model_dir: &str, out_dir: &str) -> Result<()> {
    let invalid = |m: String| FloatTrainError::Invalid(m);
    let kernel = include_str!("../spiralcore_fbs.cu");
    let forward = forward_text(kernel).map(|t| hex::encode(Sha256::digest(t.as_bytes())));
    if forward.as_deref() != Some(FORWARD_TEXT_SHA256) {
        return Err(invalid(
            "the projection's forward kernel is not the one before claude-179".to_owned(),
        ));
    }
    let kernel_sha256 = hex::encode(Sha256::digest(kernel.as_bytes()));
    let (dir, out) = (Path::new(model_dir), Path::new(out_dir));
    let run_of = |d: &Path| d.parent().and_then(Path::parent).map(Path::to_path_buf);
    let out_run =
        run_of(out).ok_or_else(|| invalid(format!("{out_dir} is not a run's models/step-N")))?;
    if out_run.exists() {
        return Err(invalid(format!(
            "{} exists, and the copy never replaces one",
            out_run.display()
        )));
    }
    let bytes = std::fs::read(dir.join("config.json"))?;
    let saved: serde_json::Value = serde_json::from_slice(&bytes)?;
    let binding = saved
        .get("spiralcore_fbs_projection")
        .ok_or_else(|| invalid("the checkpoint carries no projection identity".to_owned()))?;
    if binding.get("schema").and_then(|v| v.as_str()) != Some(SCHEMA)
        || binding.get("fbs_source_sha256").and_then(|v| v.as_str()) != Some(FBS_SOURCE_SHA256)
    {
        return Err(invalid(
            "the identity's schema or FBS source is not this build's".to_owned(),
        ));
    }
    let plans = binding
        .get("plans")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("the identity has no plans".to_owned()))?;
    // Each fingerprint the config saves: the projection's plans, and the vector head's
    // `geometry`, the 8/8 plan's (`WriterSpec::binding`).
    let mut saved_fps: Vec<(String, &str)> = plans
        .iter()
        .map(|(key, fp)| (key.clone(), fp.as_str().unwrap_or_default()))
        .collect();
    if let Some(head) = saved.get("native_vector_head") {
        let geometry = head.get("geometry").and_then(|v| v.as_str());
        saved_fps.push((
            "8/8".to_owned(),
            geometry.ok_or_else(|| invalid("the vector head has no geometry".to_owned()))?,
        ));
    }
    let mut text = String::from_utf8(bytes).map_err(|e| invalid(e.to_string()))?;
    for (key, saved_fp) in &saved_fps {
        let (key, saved_fp) = (key.as_str(), *saved_fp);
        let (input, output) = key
            .split_once('/')
            .and_then(|(i, o)| Some((i.parse::<usize>().ok()?, o.parse::<usize>().ok()?)))
            .ok_or_else(|| invalid(format!("plan {key} is not INPUT/OUTPUT")))?;
        let plan = ProjectionPlan::new(input, output)?;
        if pre_179_fingerprint(&plan, &kernel_sha256)? != saved_fp {
            return Err(invalid(format!(
                "plan {key}: the saved fingerprint is not this plan's with the kernel before claude-179"
            )));
        }
        // The config is otherwise copied byte for byte: each saved fingerprint appears once.
        if text.matches(saved_fp).count() != 1 {
            return Err(invalid(format!(
                "plan {key}'s fingerprint is not in the config once"
            )));
        }
        text = text.replace(saved_fp, plan.fingerprint());
        println!("plan {key}: {saved_fp} -> {}", plan.fingerprint());
    }
    // The tensors, every other file of the step and the run's config, byte for byte.
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(invalid(format!("{} is not a file", entry.path().display())));
        }
        files.push(entry.file_name());
    }
    std::fs::create_dir_all(out)?;
    std::fs::copy(
        run_of(dir)
            .ok_or_else(|| invalid(format!("{model_dir} is not a run's models/step-N")))?
            .join("config.json"),
        out_run.join("config.json"),
    )?;
    for name in files.iter().filter(|n| *n != "config.json") {
        std::fs::copy(dir.join(name), out.join(name))?;
    }
    write(&out.join("config.json").to_string_lossy(), text.as_bytes())?;
    // The copy loads, with every check the loader makes, on the CPU before any GPU opens. A copy
    // that does not load is removed: the run folder did not exist before this call.
    let loads =
        StackModel::load(out, &Device::Cpu).and_then(|_| HammingReads::load(out, &Device::Cpu));
    if let Err(e) = loads {
        std::fs::remove_dir_all(&out_run)?;
        return Err(e);
    }
    println!(
        "rebound: {} fingerprints, {} files; the copy loads",
        saved_fps.len(),
        files.len()
    );
    Ok(())
}

/// An inspection's tie order: `as-trained` (the GPU sort's own) or `tie-rule` (the integer
/// session's).
fn inspect_ties(name: &str) -> Option<bool> {
    match name {
        "as-trained" => Some(false),
        "tie-rule" => Some(true),
        _ => None,
    }
}

/// `--inspect TIES DATA_ROOT RUN_DIR` (claude-240): the module docs' fourth form.
fn inspect(ties: &str, data_root: &str, run: &str) -> Result<()> {
    let Some(tie) = inspect_ties(ties) else {
        usage();
    };
    let run = Path::new(run);
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(run.join("config.json"))?)?;
    let flag = |name: &str| {
        config[name]
            .as_bool()
            .ok_or_else(|| FloatTrainError::Invalid(format!("the run config has no {name}")))
    };
    let seed = config["seed"]
        .as_u64()
        .ok_or_else(|| FloatTrainError::Invalid("the run config has no seed".to_owned()))?;
    let device = Device::new_cuda(0)?;
    // `inspect_run` reads on this thread, so the rule set here is the one its reads use.
    set_hamming_tie_rule(tie);
    let (nearest, farthest) = root_separation()?;
    println!(
        "{}",
        serde_json::json!({"address_root_separation": {"nearest": nearest, "farthest": farthest}})
    );
    for reading in inspect_run(
        Path::new(data_root),
        run,
        flag("retain_rates")?,
        flag("fold_scores")?,
        flag("reflect_floor")?,
        seed,
        FbsArm::OneAtomActive,
        &device,
        None,
    )? {
        println!("{}", serde_json::to_string(&reading)?);
    }
    Ok(())
}

fn usage() -> ! {
    eprintln!(
        "usage: fbs-snapshot [--dev DATA_ROOT] MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]\n       fbs-snapshot [--dev DATA_ROOT] --hamming-trace ARM MODEL_DIR D11_READOUT_JSON TRACE_JSON\n       fbs-snapshot --rebind-fbs-backward MODEL_DIR OUT_MODEL_DIR\n       fbs-snapshot --inspect as-trained|tie-rule DATA_ROOT RUN_DIR\n       fbs-snapshot --round-as-exported MODEL_DIR OUT_MODEL_DIR"
    );
    std::process::exit(2);
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|flag| flag == "--rebind-fbs-backward")
    {
        let [_, model_dir, out_dir] = args.as_slice() else {
            usage();
        };
        return rebind_fbs_backward(model_dir, out_dir);
    }
    if args.first().is_some_and(|flag| flag == "--inspect") {
        let [_, ties, data_root, run] = args.as_slice() else {
            usage();
        };
        return inspect(ties, data_root, run);
    }
    if args
        .first()
        .is_some_and(|flag| flag == "--round-as-exported")
    {
        let [_, model_dir, out_dir] = args.as_slice() else {
            usage();
        };
        let copy = write_rounded_copy(Path::new(model_dir), Path::new(out_dir))?;
        let run = &copy.run_config;
        println!(
            "{}  run {} (from {})",
            run.copy_sha256, run.name, run.source_sha256
        );
        for file in &copy.files {
            println!(
                "{}  {} (from {})",
                file.copy_sha256, file.name, file.source_sha256
            );
        }
        println!(
            "rounded as exported: {} files; the copy reads back as rounded, and re-rounding it changes nothing",
            copy.files.len()
        );
        return Ok(());
    }
    let dev = if args.first().is_some_and(|flag| flag == "--dev") {
        if args.len() < 2 {
            usage();
        }
        let root = args.drain(..2).nth(1).unwrap_or_default();
        Some(dev_sample(&root)?)
    } else {
        None
    };
    let dev = dev.as_deref();
    if args.first().is_some_and(|flag| flag == "--hamming-trace") {
        let [_, arm, model_dir, readout, trace] = args.as_slice() else {
            usage();
        };
        return hamming_trace(arm, model_dir, readout, trace, dev);
    }
    let (model_dir, out, readouts) = match args.as_slice() {
        [model_dir, out, readouts @ ..] if readouts.len() <= 3 => (model_dir, out, readouts),
        _ => usage(),
    };
    let dir = Path::new(model_dir);
    // Every read the checkpoint holds is carried, or the export is refused (claude-198), before
    // the GPU is opened.
    check_checkpoint_reads(dir)?;
    check_readout_reads(dir, readouts.len())?;
    // The run's rate reading (integer-retention-rates, claude-226), from its `config.json` two
    // levels above `models/step-N`. The snapshot carries it, and the float readouts below are
    // taken under it, as the run trained and scored.
    let config = run_config(dir)?;
    let retain = run_rate_reading(&config)?;
    // And its readout (integer-nearest-readout, claude-231): a cell or nearest objective decodes
    // as the whole table's nearest token.
    let nearest = run_readout_nearest(&config).map_err(FloatTrainError::Invalid)?;
    set_retention_rates(retain);
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(dir, &device)?;
    let mut snapshot = fbs_trunk_snapshot(&model)?;
    snapshot.memory = TurnMemory::load(dir, &device)?
        .map(|memory| fbs_memory_snapshot(&memory, SHEET_CONTEXT))
        .transpose()?;
    let hamming = HammingReads::load(dir, &Device::Cpu)?
        .map(|reads| fbs_hamming_snapshot(&reads, snapshot.layers.len(), snapshot.width))
        .transpose()?;
    snapshot.hamming = hamming;
    snapshot.retain_rates = retain;
    snapshot.readout_nearest = nearest;
    write(out, &serde_json::to_vec(&snapshot)?)?;
    // The readouts are the float model with its Hamming reads (claude-200): `StackModel::hidden`
    // applies the reads set on this thread, here on the trunk's GPU.
    set_hamming_reads(HammingReads::load(dir, &device)?);
    for (path, mode) in readouts.iter().zip([
        FbsReadoutMode::Plain,
        FbsReadoutMode::Export,
        FbsReadoutMode::D11,
    ]) {
        if mode == FbsReadoutMode::D11 {
            round_tables_as_exported(&model)?;
        }
        write(path, &serde_json::to_vec(&readout_on(&model, mode, dev)?)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        forward_text, inspect_ties, output_place, pre_179_fingerprint, sample_windows,
        scored_prefix, CausalWindow, ProjectionPlan, FORWARD_TEXT_SHA256, PARITY_WINDOW_STRIDE,
    };
    use sha2::{Digest, Sha256};

    /// claude-240: the inspection reproduces a sealed one with the rule off, as the checkpoint
    /// trained; the rule is the other reading. No other name is taken.
    #[test]
    fn the_inspection_as_trained_turns_the_tie_rule_off() {
        assert_eq!(inspect_ties("as-trained"), Some(false));
        assert_eq!(inspect_ties("tie-rule"), Some(true));
        assert_eq!(inspect_ties("d11"), None);
    }

    /// claude-240: claude-179 changed the projection kernel after `spiralcore_fbs_fwd` only, so
    /// its text through the forward is the text before claude-179. An edit inside the forward
    /// moves the hash; one after it does not.
    #[test]
    fn the_forward_kernel_is_the_one_before_claude_179() {
        let hash = |k: &str| forward_text(k).map(|t| hex::encode(Sha256::digest(t.as_bytes())));
        let kernel = include_str!("../spiralcore_fbs.cu");
        assert_eq!(hash(kernel).as_deref(), Some(FORWARD_TEXT_SHA256));
        let after = format!("{kernel}\n// a later kernel\n");
        assert_eq!(hash(&after).as_deref(), Some(FORWARD_TEXT_SHA256));
        let inside = kernel.replacen("spiralcore_fbs_fwd(", "spiralcore_fbs_fwd( ", 1);
        assert_ne!(hash(&inside).as_deref(), Some(FORWARD_TEXT_SHA256));
    }

    /// claude-240: claude-146's checkpoints (`df1d7892`, all three seeds) saved these five plan
    /// fingerprints and the vector head's `geometry` (8/8). Each is this build's plan with only
    /// the kernel's hash put back, and none is this build's own, which is why `StackModel::load`
    /// refuses them.
    #[test]
    fn claude_146s_fingerprints_are_this_builds_plans_with_the_kernel_before_claude_179() {
        let kernel = hex::encode(Sha256::digest(include_bytes!("../spiralcore_fbs.cu")));
        for (input, output, saved) in [
            (
                256,
                256,
                "2653ea424190eb64e231b66f112eae74e52df8fd2a6760a24c5a09771ee6fbbf",
            ),
            (
                256,
                320,
                "b5da61c685f8d329260d5800fda2949ff904965fcdded8eb109b5db587a8b434",
            ),
            (
                256,
                384,
                "21945c345063470c9451eaad8f3c5b847cd0552418299087695c7e20bdce4c43",
            ),
            (
                256,
                512,
                "60ab6ef386dcea9fef54500ce3cd65a7f615be0c4c1de70308bc8224d5a2cf9b",
            ),
            (
                384,
                256,
                "afd76e112421753939844c60dfe6418ad32c3a7ba701d87b354d3c91f4d11479",
            ),
            (
                8,
                8,
                "5048316155f79ce6a908ad2f19794025fe633671aae61d10081ff194e2678bc5",
            ),
        ] {
            let plan = ProjectionPlan::new(input, output).unwrap();
            assert_eq!(
                pre_179_fingerprint(&plan, &kernel).unwrap(),
                saved,
                "{input}/{output}"
            );
            assert_ne!(plan.fingerprint(), saved, "{input}/{output}");
        }
    }

    /// claude-240: a DEV window is cut after its last scored position, so its BOS padding never
    /// reaches the parity counts, and every scored position and its prefix stay.
    #[test]
    fn a_dev_window_is_cut_after_its_last_scored_position() {
        let window = CausalWindow {
            doc_id: 0,
            input_ids: vec![0, 5, 6, 7, 8, 0, 0, 0],
            scored_offsets: vec![1, 3],
            scored_targets: vec![6, 8],
            scored_corpus_positions: vec![2, 4],
        };
        assert_eq!(scored_prefix(&window), vec![0, 5, 6, 7]);
    }

    /// claude-240: the DEV sample is claude-126's, every 8th window from the first, in order.
    #[test]
    fn the_dev_sample_keeps_every_eighth_window_from_the_first() {
        assert_eq!(PARITY_WINDOW_STRIDE, 8);
        let windows: Vec<Vec<u32>> = (0..17u32).map(|w| vec![w; 3]).collect();
        assert_eq!(
            sample_windows(windows),
            vec![vec![0; 3], vec![8; 3], vec![16; 3]]
        );
        // DEV's 428 windows give 54.
        assert_eq!(sample_windows(vec![0u8; 428]).len(), 54);
    }

    /// claude-235: two spellings of one output are one place, so the trace refuses them.
    #[test]
    fn two_spellings_of_one_output_are_one_place() {
        assert_eq!(
            output_place("./trace.json").unwrap(),
            output_place("trace.json").unwrap()
        );
        assert_ne!(
            output_place("readout.json").unwrap(),
            output_place("trace.json").unwrap()
        );
        assert!(output_place("..").is_err(), "names no file");
    }
}
