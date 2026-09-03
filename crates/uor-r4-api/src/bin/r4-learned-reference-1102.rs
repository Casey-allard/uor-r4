//! Research-only stdin harness for the separately admitted #1102 comparison.
//! Hex is this harness's lossless IPC encoding, not a public serving protocol.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Read, Write};
use uor_r4_api::learned_reference::{
    floating_point_environment, sha256, ComparisonAdmission, ExpectedBinding,
    LoadedResearchReference, NativeError, RawRequest, RuntimeIdentity, ValidationAudit,
    CONTRACT_SHA256,
};

type Failure = Box<dyn std::error::Error>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    schema: String,
    text_hex: String,
    request_extras: serde_json::Map<String, Value>,
}
fn emit(value: &Value) -> Result<(), Failure> {
    let mut out = io::stdout().lock();
    serde_json::to_writer(&mut out, value)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}
fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Failure> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("missing string {key}").into())
}
fn hex(bytes: impl IntoIterator<Item = u8>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::new();
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}
fn unhex(text: &str) -> Result<Vec<u8>, Failure> {
    if text.len() % 2 != 0 || text.len() > 32768 {
        return Err("invalid raw hex length".into());
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|x| {
            let digit = |b: u8| -> Result<u8, Failure> {
                match b {
                    b'0'..=b'9' => Ok(b - b'0'),
                    b'a'..=b'f' => Ok(b - b'a' + 10),
                    _ => Err("invalid raw hex".into()),
                }
            };
            Ok(digit(x[0])? * 16 + digit(x[1])?)
        })
        .collect()
}
fn f32hex(values: &[f32]) -> String {
    hex(values.iter().flat_map(|x| x.to_le_bytes()))
}

#[derive(Default, Serialize)]
struct Work {
    model_load_started: bool,
    validation_audit: Option<ValidationAudit>,
    model_loads: u32,
    logical_forwards: u32,
    refusal_rows: u32,
}
fn run(work: &mut Work) -> Result<(), Failure> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).ok_or("missing mode")?;
    let binary_sha = sha256(&fs::read(std::env::current_exe()?)?);
    if mode == "metadata" {
        return emit(
            &json!({"schema":"uor-r4.native-runtime-metadata/1", "binary_sha256":binary_sha,
            "arch":std::env::consts::ARCH,"os":std::env::consts::OS,"fpcr":floating_point_environment()?}),
        );
    }
    if mode != "gate" && mode != "run" {
        return Err("invalid mode".into());
    }
    if args.len() != 6 {
        return Err("expected mode release release_sha256 artifact expected_binding".into());
    }
    let release_bytes = fs::read(&args[2])?;
    if sha256(&release_bytes) != args[3] {
        return Err("release identity mismatch".into());
    }
    let release: Value = serde_json::from_slice(&release_bytes)?;
    if release["schema"] != "uor-r4.native-bridge-release/1"
        || release["issue"] != 1102
        || release["contract_sha256"] != CONTRACT_SHA256
    {
        return Err("release schema mismatch".into());
    }
    let native = &release["native"];
    let runtime: RuntimeIdentity = serde_json::from_value(native["runtime"].clone())?;
    if runtime.native_binary_sha256 != binary_sha {
        return Err("native binary identity mismatch".into());
    }
    let probes = native["probes"].as_object().ok_or("missing probes")?;
    if probes.len() != 4 {
        return Err("missing probe classes".into());
    }
    for key in ["corpus", "reference", "history", "results"] {
        let path = string(&native["probes"], key)?;
        match fs::File::open(path) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {}
            _ => return Err(format!("isolation probe was not denied: {key}").into()),
        }
    }
    let fpcr_before = floating_point_environment()?;
    let expected: ExpectedBinding = serde_json::from_slice(&fs::read(&args[5])?)?;
    let mut artifact = Vec::new();
    fs::File::open(&args[4])?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut artifact)?;
    work.model_load_started = true;
    let (loaded, validation_audit) = LoadedResearchReference::load_audited(artifact, &expected);
    work.model_loads = if loaded.is_ok() { 2 } else { 0 };
    work.validation_audit = Some(validation_audit.clone());
    if mode == "gate" {
        return match loaded {
            Err(error) => emit(
                &json!({"kind":"gate","error":error,"logical_forwards":0,"model_loads":0,"validation_audit":validation_audit,"fpcr_after":floating_point_environment()?}),
            ),
            Ok(engine) => {
                let answer = engine.answer(RawRequest {
                    schema: "uor-r4.text-to-clauses/1",
                    text: b"",
                });
                let error = answer.err().ok_or("unqualified engine answered")?;
                emit(
                    &json!({"kind":"gate","error":null,"missing_qualification":error,
                    "capability":engine.capability(),"native_state_sha256":engine.manifest().native_state_sha256,
                    "owned_artifact_bytes":engine.owned_artifact_bytes(),"validation_audit":validation_audit,"logical_forwards":0,"model_loads":2,
                    "fpcr_before":fpcr_before,"fpcr_after":floating_point_environment()?}),
                )
            }
        };
    }
    let engine = loaded?;
    work.model_loads = 2;
    let admission = ComparisonAdmission::from_trusted_release(&release_bytes, &args[3], runtime)?;
    emit(&json!({"kind":"ready","model_loads":2,"logical_forwards":0,
        "native_state_sha256":engine.manifest().native_state_sha256,"capability":engine.capability(),"fpcr":fpcr_before}))?;
    let mut input = io::stdin().lock();
    let mut valid = 0u32;
    let mut refusals = 0u32;
    loop {
        let mut line = String::new();
        let count = (&mut input).take(65537).read_line(&mut line)?;
        if count == 0 {
            break;
        }
        if count > 65536 || !line.ends_with('\n') {
            return Err("invalid packet size".into());
        }
        let packet: Packet = serde_json::from_str(&line)?;
        let raw = unhex(&packet.text_hex)?;
        if valid + refusals >= 336 {
            return Err("row cap exceeded".into());
        }
        // Unknown original request fields are refused at the original first
        // precedence, before raw-text interpretation or model work.
        let schema = if packet.request_extras.is_empty() {
            packet.schema.as_str()
        } else {
            ""
        };
        let mut attempted = 0;
        let evaluated = engine.compare(
            RawRequest { schema, text: &raw },
            &admission,
            320 - work.logical_forwards,
            &mut attempted,
        );
        work.logical_forwards += attempted;
        let output = evaluated?;
        let mut tensors = json!({});
        let mut diagnostics = Value::Null;
        let logical_forwards = if let Some(d) = &output.diagnostics {
            valid += 1;
            tensors = json!({"role_attention":f32hex(&d.role_attention),"role_vectors":f32hex(&d.role_vectors),
                "binding_attention":f32hex(&d.binding_attention),"logits":f32hex(&d.logits)});
            diagnostics = json!({"role_argmax":d.role_argmax,"token_frame_indices":d.token_frame_indices,
                "clause_frame_indices":d.clause_frame_indices});
            1
        } else {
            refusals += 1;
            work.refusal_rows += 1;
            0
        };
        if valid > 320 || refusals > 16 {
            return Err("per-kind cap exceeded".into());
        }
        emit(
            &json!({"kind":"result","result":output.result,"parsed":output.parsed,
            "tensors":tensors,"diagnostics":diagnostics,"logical_forwards":logical_forwards,"receipt":output.receipt}),
        )?;
    }
    if valid != 320 || refusals != 16 {
        return Err("incomplete population".into());
    }
    emit(
        &json!({"kind":"done","valid_rows":valid,"refusal_rows":refusals,"logical_forwards":valid,
        "model_loads":2,"parameter_updates":0,"native_state_sha256":engine.manifest().native_state_sha256,
        "fpcr_after":floating_point_environment()?}),
    )
}
fn main() {
    let mut work = Work::default();
    if let Err(error) = run(&mut work) {
        let fpcr = floating_point_environment().ok();
        let native_error = error.downcast_ref::<NativeError>();
        let _ = emit(
            &json!({"kind":"error","message":error.to_string(),"fpcr_after":fpcr,"work":work,"native_error":native_error}),
        );
        std::process::exit(1);
    }
}
