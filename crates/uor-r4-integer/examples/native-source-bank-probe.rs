//! Source-free fidelity probe. Inputs contain source/query/history, never targets.
//! This allocating driver is not a product session or a performance benchmark.
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::{
    geometric_occurrence_read::{FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity},
    geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer, SourceBankSegment},
    report_output,
};
#[path = "support/source_probe.rs"]
mod support;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const INPUT_LIMIT: u64 = 4 * 1024 * 1024;

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
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
impl Segment {
    fn frame(&self) -> Option<SelectedRecordFrame<'_>> {
        match self {
            Self::Source {
                record,
                commit,
                scope,
                entity,
                relation,
                view,
                original_source_ids,
                ..
            } => Some(SelectedRecordFrame {
                identity: SourceIdentity {
                    record: *record,
                    commit: *commit,
                },
                metadata: FrameMetadata {
                    scope: scope.as_bytes(),
                    entity,
                    relation: *relation,
                    view: *view,
                    status: FrameStatus::Found,
                },
                token_ids: original_source_ids,
            }),
            Self::Context { .. } => None,
        }
    }
}

struct Args {
    artifact: PathBuf,
    binding: PathBuf,
    inputs: PathBuf,
    out: PathBuf,
    generation_tokens: usize,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn args() -> Result<Args> {
    let mut values = std::collections::BTreeMap::new();
    let mut argv = std::env::args().skip(1);
    while let Some(key) = argv.next() {
        if ![
            "--artifact",
            "--binding",
            "--inputs",
            "--out",
            "--generation-tokens",
        ]
        .contains(&key.as_str())
        {
            return Err(invalid(format!("unknown argument {key}")).into());
        }
        let value = argv
            .next()
            .ok_or_else(|| invalid(format!("value absent for {key}")))?;
        if values.insert(key, value).is_some() {
            return Err(invalid("duplicate argument").into());
        }
    }
    let mut take = |name: &str| {
        values
            .remove(name)
            .ok_or_else(|| invalid(format!("required {name}")))
    };
    let result = Args {
        artifact: take("--artifact")?.into(),
        binding: take("--binding")?.into(),
        inputs: take("--inputs")?.into(),
        out: take("--out")?.into(),
        generation_tokens: take("--generation-tokens")?.parse()?,
    };
    if result.generation_tokens > 64 {
        return Err(invalid("generation token cap exceeds 64").into());
    }
    let output = support::prospective_output(&result.out)?;
    if output.starts_with(fs::canonicalize(&result.artifact)?) {
        return Err(invalid("output beneath artifact is forbidden").into());
    }
    for input in [&result.artifact, &result.binding, &result.inputs] {
        let canonical = fs::canonicalize(input)?;
        for ancestor in canonical.ancestors() {
            if ancestor.is_dir()
                && ancestor.join(report_output::MANIFEST_FILE).exists()
                && output.starts_with(ancestor)
            {
                return Err(invalid("output beneath a sealed input ancestor is forbidden").into());
            }
        }
    }
    Ok(result)
}

fn read_capped(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > maximum {
        return Err(invalid("input byte cap exceeded").into());
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(invalid("input grew beyond byte cap").into());
    }
    Ok(bytes)
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn run() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let outcome = (|| -> Result<()> {
        let start = Instant::now();
        let input_bytes = read_capped(&a.inputs, INPUT_LIMIT)?;
        let binding_bytes = read_capped(&a.binding, 16 * 1024)?;
        let input: Inputs = serde_json::from_slice(&input_bytes)?;
        if input.schema != "uor-r4.native-source-bank-probe-input/1"
            || input.cases.is_empty()
            || input.cases.len() > 64
        {
            return Err(invalid("input schema/case count differs").into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for packet in &input.cases {
            if packet.id.is_empty()
                || !ids.insert(&packet.id)
                || packet.segments.is_empty()
                || packet.segments.len() > 128
                || packet.query_ids.len() > 128
                || packet.actual_prefix_ids.len() > 128
            {
                return Err(invalid("packet identity/size admission differs").into());
            }
        }
        let binding: NativeArtifactBinding = serde_json::from_slice(&binding_bytes)?;
        let native = NativeSourceRealizer::load_native(&a.artifact, &binding)?;
        let eos = native.binding().eos_token_id();
        let mut rows = Vec::with_capacity(input.cases.len());
        for packet in input.cases {
            if start.elapsed().as_secs() >= 300 {
                return Err(invalid("declared replay wall cap reached").into());
            }
            let mut views = Vec::with_capacity(packet.segments.len());
            for segment in &packet.segments {
                views.push(match segment.frame() {
                    Some(frame) => Some(native.compile_view(frame.token_ids)?),
                    None => None,
                });
            }
            let mut segments = Vec::with_capacity(packet.segments.len());
            for (i, segment) in packet.segments.iter().enumerate() {
                segments.push(match segment {
                    Segment::Source { event, .. } => SourceBankSegment::Source {
                        frame: segment
                            .frame()
                            .ok_or_else(|| invalid("source frame absent"))?,
                        view: views[i]
                            .as_ref()
                            .ok_or_else(|| invalid("source view absent"))?,
                        event: *event,
                    },
                    Segment::Context {
                        token_ids,
                        event,
                        role,
                    } => SourceBankSegment::Context {
                        token_ids,
                        event: *event,
                        role: *role,
                    },
                });
            }
            let initial =
                native.read_bank(&segments, &packet.query_ids, &packet.actual_prefix_ids)?;
            let single_parent = if segments.len() == 1 {
                if let SourceBankSegment::Source { frame, view, .. } = segments[0] {
                    let old =
                        native.read(frame, view, &packet.query_ids, &packet.actual_prefix_ids)?;
                    if initial.actions != old.actions
                        || initial.context != old.period_context
                        || initial.heads != old.source.view_kernel_trace.heads
                        || initial.period_q24 != old.period_q24
                    {
                        return Err(invalid("single bank parent parity differs").into());
                    }
                    Some(old)
                } else {
                    None
                }
            } else {
                None
            };
            let mut prefix = packet.actual_prefix_ids.clone();
            let mut emitted = Vec::new();
            let mut traces = Vec::new();
            let mut parent_traces = Vec::new();
            let mut terminated = false;
            for step in 0..a.generation_tokens {
                if start.elapsed().as_secs() >= 300 {
                    return Err(invalid("declared replay wall cap reached").into());
                }
                let trace = if step == 0 {
                    initial.clone()
                } else {
                    native.read_bank(&segments, &packet.query_ids, &prefix)?
                };
                if segments.len() == 1 {
                    if let SourceBankSegment::Source { frame, view, .. } = segments[0] {
                        let old = native.read(frame, view, &packet.query_ids, &prefix)?;
                        if trace.actions != old.actions
                            || trace.context != old.period_context
                            || trace.heads != old.source.view_kernel_trace.heads
                            || trace.period_q24 != old.period_q24
                        {
                            return Err(
                                invalid("single bank own-prefix parent parity differs").into()
                            );
                        }
                        parent_traces.push(old);
                    }
                }
                let token = trace.actions.chosen_token_id;
                traces.push(trace);
                emitted.push(token);
                if token == eos {
                    terminated = true;
                    break;
                }
                prefix.push(token);
            }
            rows.push(
                json!({"id":packet.id,"single_record_parent":single_parent,"initial_trace":initial,
            "generated_ids_including_eos":emitted,"own_prefix_traces":traces,"single_record_own_prefix_parent_traces":parent_traces,
            "terminated_with_eos":terminated}),
            );
        }
        // A source-free chroot need not expose /proc/self/exe. The supervising
        // receipt independently pins the executable; its absolute argv0 is usable
        // for recording the same bytes inside that restricted filesystem.
        let (executable_path, executable_lookup) = match std::env::current_exe() {
            Ok(path) => (path, "current_exe"),
            Err(_) => {
                let path = PathBuf::from(
                    std::env::args_os()
                        .next()
                        .ok_or_else(|| invalid("executable argv0 absent"))?,
                );
                if !path.is_absolute() {
                    return Err(
                        invalid("restricted replay requires absolute executable argv0").into(),
                    );
                }
                (path, "absolute_argv0; independently bound by supervisor")
            }
        };
        let executable = read_capped(&executable_path, 64 * 1024 * 1024)?;
        let report = json!({"schema":"uor-r4.native-source-bank-probe/1",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":digest(&executable),
        "executable_lookup":executable_lookup,
        "inputs_sha256":digest(&input_bytes),"trusted_binding_sha256":digest(&binding_bytes),
        "generation_token_cap":a.generation_tokens,"optimizer_updates":0,
        "elapsed_millis":start.elapsed().as_millis(),"native_stats":native.stats(),"rows":rows,
        "scope":"native causal-bank entry; source/query/prefix only; single-source exact parent control; roles provenance only; bank-dependent feedback NOT_IMPLEMENTED; no general chat or performance qualification"});
        let output = serde_json::to_vec_pretty(&report)?;
        if output.len() > 64 * 1024 * 1024 {
            return Err(invalid("report byte cap exceeded").into());
        }
        fs::write(a.out.join("report.json"), output)?;
        Ok(())
    })();
    if let Err(ref error) = outcome {
        fs::write(
            a.out.join("failure.json"),
            serde_json::to_vec_pretty(
                &json!({"status":"failed","error":error.to_string(),"no_adopted_model":true}),
            )?,
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    outcome
}

fn main() {
    if let Err(error) = run() {
        eprintln!("native source-realizer probe: {error}");
        std::process::exit(1);
    }
}
