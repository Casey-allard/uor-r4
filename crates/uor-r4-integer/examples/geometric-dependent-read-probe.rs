//! Source-free integer dependent-read interface probe; no learned feedback claim.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_integer::{
    geometric_no_read::CANONICAL_BASIS_Q25,
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_read_feedback::{FeedbackInputMode, NativeReadFeedback},
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer},
    geometric_value_q4::{pack_coefficients, ValueQ4Config},
    report_output,
};
#[path = "support/source_probe.rs"]
mod support;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_REPORT_BYTES: usize = 128 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    record: u64,
    commit: u64,
    scope: String,
    entity: Vec<u32>,
    relation: u32,
    view: u32,
    status: String,
    original_source_ids: Vec<u32>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
struct Args {
    artifact: PathBuf,
    binding: PathBuf,
    inputs: PathBuf,
    out: PathBuf,
    mode: String,
    feedback_input: String,
    generation_tokens: usize,
}
fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn read_capped(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(invalid("input byte cap exceeded").into());
    }
    Ok(bytes)
}
fn args() -> Result<Args> {
    let mut values = BTreeMap::new();
    let mut argv = std::env::args().skip(1);
    while let Some(key) = argv.next() {
        if ![
            "--artifact",
            "--binding",
            "--inputs",
            "--out",
            "--mode",
            "--feedback-input",
            "--generation-tokens",
        ]
        .contains(&key.as_str())
        {
            return Err(invalid(format!("unknown argument {key}")).into());
        }
        let value = argv
            .next()
            .ok_or_else(|| invalid("argument value absent"))?;
        if values.insert(key, value).is_some() {
            return Err(invalid("duplicate argument").into());
        }
    }
    let mut take = |key: &str| {
        values
            .remove(key)
            .ok_or_else(|| invalid(format!("required {key}")))
    };
    let a = Args {
        artifact: take("--artifact")?.into(),
        binding: take("--binding")?.into(),
        inputs: take("--inputs")?.into(),
        out: take("--out")?.into(),
        mode: take("--mode")?,
        feedback_input: take("--feedback-input")?,
        generation_tokens: take("--generation-tokens")?.parse()?,
    };
    if !["baseline", "identity", "bridge"].contains(&a.mode.as_str())
        || !["joint-copy", "role-surface"].contains(&a.feedback_input.as_str())
        || a.generation_tokens > 64
    {
        return Err(invalid("mode/input/generation admission differs").into());
    }
    let output = support::prospective_output(&a.out)?;
    for input in [&a.artifact, &a.binding, &a.inputs] {
        let canonical = fs::canonicalize(input)?;
        if input == &a.artifact && output.starts_with(&canonical) {
            return Err(invalid("output beneath artifact").into());
        }
        for ancestor in canonical.ancestors() {
            if ancestor.is_dir()
                && ancestor.join(report_output::MANIFEST_FILE).exists()
                && output.starts_with(ancestor)
            {
                return Err(invalid("output beneath sealed input ancestor").into());
            }
        }
    }
    Ok(a)
}
fn coefficient(root: usize, coordinate: usize) -> i8 {
    // Offline integer rounding of the pinned basis; no semantic token hash.
    let numerator = i64::from(CANONICAL_BASIS_Q25[root][coordinate]) * 7;
    let half = 1i64 << 24;
    let rounded = if numerator >= 0 {
        (numerator + half) >> 25
    } else {
        -((-numerator + half) >> 25)
    };
    rounded.clamp(-7, 7) as i8
}
fn synthetic_value(config: ValueQ4Config) -> Result<Vec<u8>> {
    let mut values = vec![0i8; config.coefficient_count()?];
    let mut at = 0;
    for (name, shape) in config.coefficient_shapes()? {
        let count: usize = shape.iter().product();
        if name == "own_root" {
            for index in 0..count {
                values[at + index] = coefficient((index / 4) % 120, index % 4);
            }
        } else if name == "token_category" {
            let classes = *shape
                .last()
                .ok_or_else(|| invalid("category shape absent"))?;
            for offset in (0..count).step_by(classes) {
                values[at + offset + 1] = 7;
            }
        }
        at += count;
    }
    Ok(pack_coefficients(&values)?)
}
fn feedback(native: &NativeSourceRealizer, active: bool) -> Result<NativeReadFeedback> {
    let context = native.context_config();
    let value = synthetic_value(NativeReadFeedback::value_config(context))?;
    let count = NativeReadFeedback::bridge_coefficient_count(context)?;
    let mut bridge = vec![0i8; count];
    if active {
        // [lane][action][83]: bias/own/neighbor then atom0 root+category,
        // atom1 root+category. Only the actual atom0 root contributes here.
        if count % (120 * 83) != 0 {
            return Err(invalid("bridge layout differs").into());
        }
        for lane in 0..count / (120 * 83) {
            for action in 0..120 {
                for coordinate in 0..4 {
                    bridge[(lane * 120 + action) * 83 + 9 + coordinate] =
                        coefficient(action, coordinate);
                }
            }
        }
    }
    Ok(NativeReadFeedback::compile(
        native.artifact_binding().clone(),
        context,
        &value,
        &pack_coefficients(&bridge)?,
    )?)
}
fn deadline(start: Instant) -> Result<()> {
    if start.elapsed().as_secs() >= 300 {
        return Err(invalid("300s native driver wall cap reached").into());
    }
    Ok(())
}
fn execute(a: &Args) -> Result<()> {
    let start = Instant::now();
    let bytes = read_capped(&a.inputs, 4 * 1024 * 1024)?;
    let binding_bytes = read_capped(&a.binding, 16 * 1024)?;
    let inputs: Inputs = serde_json::from_slice(&bytes)?;
    if inputs.schema != "uor-r4.native-source-realizer-probe-input/1"
        || inputs.cases.is_empty()
        || inputs.cases.len() > 64
    {
        return Err(invalid("input schema/count differs").into());
    }
    let mut ids = BTreeSet::new();
    for p in &inputs.cases {
        if p.id.is_empty()
            || !ids.insert(&p.id)
            || p.scope.is_empty()
            || p.entity.len() > 128
            || p.original_source_ids.len() > 128
            || p.query_ids.len() > 128
            || p.actual_prefix_ids.len() > 128
        {
            return Err(invalid("source packet identity/length differs").into());
        }
    }
    let binding: NativeArtifactBinding = serde_json::from_slice(&binding_bytes)?;
    let native = NativeSourceRealizer::load_native(&a.artifact, &binding)?;
    let update = if a.mode == "baseline" {
        None
    } else {
        Some(feedback(&native, a.mode == "bridge")?)
    };
    let input_mode = if a.feedback_input == "joint-copy" {
        FeedbackInputMode::JointCopy
    } else {
        FeedbackInputMode::RoleSurface
    };
    let mut total_bytes = 0usize;
    let mut rows = Vec::new();
    if let Some(update) = &update {
        fs::write(
            a.out.join("feedback-metadata.json"),
            serde_json::to_vec_pretty(update.metadata())?,
        )?;
        fs::write(a.out.join("value-q4.bin"), update.value_packed())?;
        fs::write(a.out.join("bridge-q4.bin"), update.bridge_packed())?;
        total_bytes += update.value_packed().len() + update.bridge_packed().len();
    }
    for (row_index, p) in inputs.cases.into_iter().enumerate() {
        deadline(start)?;
        if p.status != "Found" {
            return Err(invalid("only admitted Found source supported").into());
        }
        let frame = SelectedRecordFrame {
            identity: SourceIdentity {
                record: p.record,
                commit: p.commit,
            },
            metadata: FrameMetadata {
                scope: p.scope.as_bytes(),
                entity: &p.entity,
                relation: p.relation,
                view: p.view,
                status: FrameStatus::Found,
            },
            token_ids: &p.original_source_ids,
        };
        let view = native.compile_view(&p.original_source_ids)?;
        let mut prefix = p.actual_prefix_ids.clone();
        let mut emitted = Vec::new();
        let mut steps = Vec::new();
        let mut terminated = false;
        // Even generation0 performs one admitted read for causal construction.
        for step in 0..a.generation_tokens.max(1) {
            deadline(start)?;
            let baseline = native.read(frame, &view, &p.query_ids, &prefix)?;
            let (token, trace): (u32, Value) = if let Some(update) = &update {
                let dependent = native.read_dependent(
                    frame,
                    &view,
                    &p.query_ids,
                    &prefix,
                    update,
                    input_mode,
                )?;
                if dependent.stage1 != baseline {
                    return Err(invalid("provisional stage differs from frozen parent").into());
                }
                if a.mode == "identity" && dependent.stage2 != baseline {
                    return Err(invalid("identity feedback differs from frozen parent").into());
                }
                (
                    dependent.stage2.actions.chosen_token_id,
                    serde_json::to_value(&dependent)?,
                )
            } else {
                (
                    baseline.actions.chosen_token_id,
                    serde_json::to_value(&baseline)?,
                )
            };
            steps.push(json!({"step":step,"actual_prefix_ids":prefix,"chosen_token_id":token,"trace":trace}));
            if a.generation_tokens == 0 {
                break;
            }
            emitted.push(token);
            if token == native.binding().eos_token_id() {
                terminated = true;
                break;
            }
            prefix.push(token);
        }
        let row = json!({"id":p.id,"source_view":view,"steps":steps,"generated_ids_including_eos":emitted,
            "eos":terminated,"stop":if a.generation_tokens==0{"read-only"}else if terminated{"eos"}else{"budget"}});
        let serialized = serde_json::to_vec(&row)?;
        total_bytes = total_bytes
            .checked_add(serialized.len())
            .ok_or_else(|| invalid("report size overflow"))?;
        if total_bytes > MAX_REPORT_BYTES - 1024 * 1024 {
            return Err(invalid("128MiB report cap reached").into());
        }
        let name = format!("row-{row_index:04}.json");
        fs::write(a.out.join(&name), &serialized)?;
        rows.push(json!({"id":row["id"],"path":name,"sha256":digest(&serialized),"eos":terminated,"steps":steps.len()}));
    }
    let (executable, executable_lookup) = match std::env::current_exe() {
        Ok(p) => (p, "current_exe"),
        Err(_) => {
            let p = PathBuf::from(
                std::env::args_os()
                    .next()
                    .ok_or_else(|| invalid("executable path absent"))?,
            );
            if !p.is_absolute() {
                return Err(invalid("restricted replay requires absolute executable argv0").into());
            }
            (p, "absolute_argv0; independently bound by supervisor")
        }
    };
    let executable_bytes = read_capped(&executable, 64 * 1024 * 1024)?;
    let report = json!({"schema":"uor-r4.geometric-dependent-read-probe/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "executable_sha256":digest(&executable_bytes),"executable_lookup":executable_lookup,"inputs_sha256":digest(&bytes),"trusted_binding_sha256":digest(&binding_bytes),
        "mode":a.mode,"feedback_input":a.feedback_input,"generation_token_cap":a.generation_tokens,"optimizer_updates":0,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"rows":rows,"native_stats":native.stats(),
        "construction":"producer approximate canonical-root own-state classifier with constant nonzero category; bridge actual atom0 root basis only; allzero bridge for identity",
        "scope":"source-free native interface and unlearned causal construction; no answer labels, learned feedback, language improvement, session, energy or performance claim"});
    fs::write(
        a.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    Ok(())
}
fn run() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let result = execute(&a);
    if let Err(error) = &result {
        // Preserve a claimed failed attempt rather than reusing its directory.
        if !a.out.join(report_output::MANIFEST_FILE).exists() {
            fs::write(
                a.out.join("failure.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema":"uor-r4.geometric-dependent-read-probe-failure/1",
                    "mode":a.mode,"feedback_input":a.feedback_input,"optimizer_updates":0,
                    "error":error.to_string(),"status":"failed; no capability claim"
                }))?,
            )?;
            report_output::seal(&a.out)?;
            report_output::verify(&a.out)?;
        }
    }
    result
}
fn main() {
    if let Err(error) = run() {
        eprintln!("geometric dependent-read probe: {error}");
        std::process::exit(1);
    }
}
