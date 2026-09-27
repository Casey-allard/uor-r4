//! Explicit offline retained R1d -> dialogue576 development conversion.
//!
//! This calibrates the existing signed4/signed16 grids, exports packed bytes,
//! and reloads them through the F32 emulator and explicit integer profile. It
//! performs no forward, generation, backward or optimizer/model update calls.
//! Numerical retention and actual dialogue output require a separate projected
//! observation; successful conversion does not promote a model.
#![forbid(unsafe_code)]

use std::{collections::BTreeMap, error::Error, fs, io, path::Path, path::PathBuf, time::Instant};

use candle_core::Device;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_integer::{config::ServingProfile, IntegerModel};
use uor_r4_training::{
    dialogue_artifact::LegacyDialogueArtifact,
    joint_model::JointModel,
    joint_quantization::{CALIBRATION_RULE, HARD_PARAMETERS_FILE, HARD_PARAMETERS_MANIFEST_FILE},
    sha256_file,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 5 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "usage: dialogue-integer-bridge FIT_ROOT CORPUS_MANIFEST TOKENIZER TABLE_ROOT NEW_REPORT_ROOT (explicit dialogue576 development profile)").into());
    }
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("");
    if source.len() != 40 || !source.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "build with full UOR_BUILD_SOURCE_COMMIT",
        )
        .into());
    }
    let report = &args[4];
    report_output::claim(report)?;
    // Both output roots are exclusively claimed before opening any model.
    let packed = report.join("packed");
    report_output::claim(&packed)?;
    let outcome = convert(&args, &packed, source);
    if let Err(error) = &outcome {
        let failure = json!({
            "status":"CONVERSION_FAILED","error":error.to_string(),
            "conversion_source_commit":source,"serving_profile":"dialogue576",
            "scope":"Incomplete development conversion; not a model-quality result"
        });
        write_json(&report.join("failed-attempt.json"), &failure)?;
        // A successful sealed child remains immutable even if later reload
        // fails. Failures before sealing retain a sealed partial child.
        if !packed.join("manifest.json").try_exists()? {
            write_json(&packed.join("failed-attempt.json"), &failure)?;
            report_output::seal(&packed)?;
        }
    }
    report_output::verify(&packed)?;
    report_output::seal(report)?;
    report_output::verify(report)?;
    outcome?;
    println!(
        "{}",
        json!({"status":"CONVERTED_AND_RELOADED_DEVELOPMENT","report_root":report,"packed_root":packed})
    );
    Ok(())
}

fn convert(args: &[PathBuf], packed: &Path, source: &str) -> Result<(), Box<dyn Error>> {
    let clock = Instant::now();
    let report = &args[4];
    let profile = ServingProfile::Dialogue576;
    let executable_sha256 = sha256_file(&std::env::current_exe()?)?;
    let source_sha256 = source_hashes();
    report_output::verify(&args[3])?;
    let artifact = LegacyDialogueArtifact::load(&args[0], &args[1], &args[2], &Device::Cpu)?;
    let (model, parent) = JointModel::calibrated_dialogue_integer_parent(artifact)?;
    if sha256_file(&args[2])? != parent.tokenizer_sha256
        || sha256_file(&args[1])? != parent.prepared_manifest_sha256
    {
        return Err(io::Error::other("conversion input changed after verified import").into());
    }
    let manifest = model.save_dialogue_integer_hard(
        packed,
        &parent,
        source,
        &executable_sha256,
        &source_sha256,
    )?;
    report_output::seal(packed)?;
    report_output::verify(packed)?;

    let hard = JointModel::load_hard_for_profile(packed, &Device::Cpu, profile)?;
    if hard.config != model.config
        || hard.quantization() != model.quantization()
        || hard.admission_policy() != model.admission_policy()
    {
        return Err(io::Error::other("packed F32 reload configuration/scale/clock differs").into());
    }
    let integer = IntegerModel::load_with_tables_profile(packed, &args[3], profile)?;
    if integer.config() != &model.config || integer.serving_profile() != profile {
        return Err(io::Error::other("integer reload configuration/profile differs").into());
    }
    // Parameter hashes are provenance of the actually reloaded dyadic values,
    // not a claim that quantization preserved their floating parent or behavior.
    let mut reloaded_parameters = Vec::new();
    for (name, variable) in hard.variables() {
        let values = variable.detach().flatten_all()?.to_vec1::<f32>()?;
        let mut digest = Sha256::new();
        for value in values {
            digest.update(value.to_le_bytes());
        }
        reloaded_parameters.push(json!({
            "name":name,"shape":variable.dims(),
            "sha256_le_f32":hex::encode(digest.finalize())
        }));
    }
    write_json(
        &report.join("result.json"),
        &json!({
            "schema":"uor-r4.native-dialogue576-bridge/1",
            "status":"CONVERTED_AND_RELOADED_DEVELOPMENT",
            "serving_profile":profile,"admission":"full",
            "conversion_source_commit":source,"conversion_executable_sha256":executable_sha256,
            "source_sha256":source_sha256,
            "parent":parent,
            "inputs":{
                "fit_root":args[0],"corpus_manifest":identity(&args[1])?,
                "tokenizer":identity(&args[2])?,"tables_root":args[3],
                "tables_manifest":identity(&args[3].join("tables.json"))?,
                "tables_payload":identity(&args[3].join("tables.bin"))?,
                "tables_seal":identity(&args[3].join("manifest.json"))?
            },
            "packed":{
                "root":packed,"manifest":identity(&packed.join("hard-model.json"))?,
                "parameter_manifest":identity(&packed.join(HARD_PARAMETERS_MANIFEST_FILE))?,
                "payload":identity(&packed.join(HARD_PARAMETERS_FILE))?,
                "seal":identity(&packed.join("manifest.json"))?
            },
            "quantization":model.quantization(),"calibration_rule":CALIBRATION_RULE,
            "parameter_statistics":manifest["parameter_manifest"]["parameter_statistics"],
            "reloaded_f32_parameters":reloaded_parameters,
            "integer_coefficient_storage":integer.coefficient_storage(),
            "integer_numerical_contract":manifest["numerical_contract"],
            "f32_emulator_numerical_contract":hard.numerical_contract(),
            "loaded_configurations_scales_clocks_match":true,
            "new_forward_calls":0,"new_backward_calls":0,"new_generation_calls":0,
            "new_optimizer_updates":0,"new_model_updates":0,
            "quantized_numerical_retention":"NOT_RUN",
            "generated_dialogue_observation":"NOT_RUN",
            "serving_performance_energy":"NOT_RUN",
            "elapsed_seconds":clock.elapsed().as_secs_f64(),
            "scope":"Offline parameter calibration/export and explicit-profile reload only. No fitting, QAT, optimizer resume, decoder selection or model promotion. Integer-profile RMS arithmetic and F32 emulation need not agree bitwise. Original historical provenance limitations remain; dense parameter access remains."
        }),
    )?;
    Ok(())
}

fn identity(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(json!({"path":path,"bytes":fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}

fn write_json(path: &Path, value: &Value) -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    std::io::Write::write_all(&mut file, b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn source_hashes() -> BTreeMap<String, String> {
    [
        (
            "example.rs",
            include_bytes!("dialogue-integer-bridge.rs").as_slice(),
        ),
        (
            "dialogue_artifact.rs",
            include_bytes!("../src/dialogue_artifact.rs").as_slice(),
        ),
        (
            "joint_model.rs",
            include_bytes!("../src/joint_model.rs").as_slice(),
        ),
        (
            "joint_quantization.rs",
            include_bytes!("../src/joint_quantization.rs").as_slice(),
        ),
        (
            "integer/config.rs",
            include_bytes!("../../uor-r4-integer/src/config.rs").as_slice(),
        ),
        (
            "integer/model.rs",
            include_bytes!("../../uor-r4-integer/src/model.rs").as_slice(),
        ),
        (
            "integer/packed_rows.rs",
            include_bytes!("../../uor-r4-integer/src/packed_rows.rs").as_slice(),
        ),
        (
            "integer/format.rs",
            include_bytes!("../../uor-r4-integer/src/format.rs").as_slice(),
        ),
        (
            "integer/math.rs",
            include_bytes!("../../uor-r4-integer/src/math.rs").as_slice(),
        ),
        (
            "integer/tables.rs",
            include_bytes!("../../uor-r4-integer/src/tables.rs").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, bytes)| (name.to_owned(), hex::encode(Sha256::digest(bytes))))
    .collect()
}
