#[cfg(any(feature = "cuda", test))]
use super::Source;
use super::{DiagnosticPin, Pin};
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "cuda", test))]
use serde_json::{json, Value};
#[cfg(any(feature = "cuda", test))]
use sha2::{Digest, Sha256};

pub const ORIGINAL_ARTIFACT: &str =
    "9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4";
pub const ORIGINAL_SOURCE: &str =
    "f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7";
pub const TARGET_SOURCE: &str = "4c0bf3ab97a82c3fed5187eb448b5d007a7dbe7271908b4f491e66d27697ea45";
pub const KERNEL: &str = "1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28";
pub const GEOMETRY: &str = "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327";
pub const CONFIG: &str = "eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgePin {
    pub schema: String,
    pub original_snapshot_sha256: String,
    pub original_source_sha256: String,
    pub target_source_sha256: String,
    pub evaluation_only: bool,
    pub training_resume: bool,
}

fn refused(pin: &Pin, reason: &str) -> String {
    let artifact = pin
        .checkpoint0
        .as_ref()
        .map(|checkpoint| checkpoint.snapshot_path.display().to_string())
        .unwrap_or_else(|| "<missing checkpoint0>".into());
    format!("checkpoint0 identity bridge refused artifact {artifact}; pre-source={ORIGINAL_SOURCE}; post-source={TARGET_SOURCE}: {reason}")
}

pub fn validate_pin(
    pin: &Pin,
    diagnostic: &DiagnosticPin,
    bridge: &BridgePin,
) -> Result<(), String> {
    if bridge.schema != "uor-r4.checkpoint0-source-bridge-pin/1"
        || bridge.original_snapshot_sha256 != ORIGINAL_ARTIFACT
        || bridge.original_source_sha256 != ORIGINAL_SOURCE
        || bridge.target_source_sha256 != TARGET_SOURCE
        || !bridge.evaluation_only
        || bridge.training_resume
        || diagnostic.source_sha256 != TARGET_SOURCE
        || diagnostic.kernel_sha256 != KERNEL
        || diagnostic.geometry_sha256 != GEOMETRY
        || diagnostic.config_file_sha256 != CONFIG
        || pin.schema != super::DIAGNOSTIC_SCHEMA
        || pin.cells.iter().any(|cell| cell.source != "checkpoint0")
        || !pin.checkpoint0.as_ref().is_some_and(|checkpoint| {
            checkpoint.snapshot_sha256 == ORIGINAL_ARTIFACT
                && checkpoint.seed == 1
                && checkpoint.epoch == 0
        })
    {
        return Err(refused(pin, "mapping, artifact, config or evaluation-only scope differs from the one declared mapping"));
    }
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
pub fn contextualize(pin: &Pin, error: &str) -> String {
    if pin
        .diagnostic
        .as_ref()
        .is_some_and(|diagnostic| diagnostic.checkpoint0_identity_bridge.is_some())
    {
        refused(pin, error)
    } else {
        error.to_owned()
    }
}

#[cfg(any(feature = "cuda", test))]
pub fn prepare(
    pin: &Pin,
    source: Source,
    bytes: &[u8],
    compiled: &Value,
) -> Result<(Vec<u8>, Option<Value>), String> {
    let Some((diagnostic, bridge)) = pin.diagnostic.as_ref().and_then(|diagnostic| {
        diagnostic
            .checkpoint0_identity_bridge
            .as_ref()
            .map(|bridge| (diagnostic, bridge))
    }) else {
        return Ok((bytes.to_vec(), None));
    };
    validate_pin(pin, diagnostic, bridge)?;
    super::validate(pin).map_err(|error| refused(pin, &error))?;
    if source != Source::Checkpoint0 {
        return Err(refused(
            pin,
            "only checkpoint0 can use this bridge; Q4 NOT_EXECUTABLE",
        ));
    }
    for (key, expected) in [
        ("source_sha256", TARGET_SOURCE),
        ("kernel_sha256", KERNEL),
        ("geometry_sha256", GEOMETRY),
    ] {
        if compiled.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(refused(
                pin,
                &format!(
                    "compiled {key} differs: observed {}",
                    compiled.get(key).unwrap_or(&Value::Null)
                ),
            ));
        }
    }
    let original_digest = format!("{:x}", Sha256::digest(bytes));
    if original_digest != ORIGINAL_ARTIFACT {
        return Err(refused(pin, &format!("original artifact checksum differs: observed {original_digest}, expected {ORIGINAL_ARTIFACT}")));
    }
    let mut snapshot: Value = serde_json::from_slice(bytes)
        .map_err(|error| refused(pin, &format!("snapshot JSON: {error}")))?;
    for (key, expected) in [
        ("schema", "uor-r4.discrete-spiralcore-fbs/2"),
        ("source_sha256", ORIGINAL_SOURCE),
        ("kernel_sha256", KERNEL),
        ("geometry_sha256", GEOMETRY),
    ] {
        if snapshot.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(refused(pin, &format!("snapshot {key} differs")));
        }
    }
    if snapshot["seed"] != json!(1)
        || snapshot["epoch"] != json!(0)
        || snapshot["spec"] != json!({"vocabulary":4096,"blocks":4,"context":192,"kept_cells":16})
        || snapshot["read_spec"] != json!({"kind":"route"})
    {
        return Err(refused(
            pin,
            "snapshot is not the sealed seed1 epoch0 K16 Route artifact",
        ));
    }
    let before = snapshot.clone();
    snapshot["source_sha256"] = json!(TARGET_SOURCE);
    let mut restored = snapshot.clone();
    restored["source_sha256"] = json!(ORIGINAL_SOURCE);
    if restored != before {
        return Err(refused(pin, "non-source snapshot fields changed"));
    }
    let rebound =
        serde_json::to_vec(&snapshot).map_err(|error| refused(pin, &error.to_string()))?;
    let mut payload = before;
    payload
        .as_object_mut()
        .ok_or_else(|| refused(pin, "snapshot is not an object"))?
        .remove("source_sha256");
    let payload = serde_json::to_vec(&payload).map_err(|error| refused(pin, &error.to_string()))?;
    let receipt = json!({
        "schema":"uor-r4.checkpoint0-source-bridge-receipt/1",
        "record_kind":"checkpoint0_identity_bridge",
        "bridge_used":true,
        "status":"mapping_prepared; device admission and zero-update checks recorded separately",
        "original_snapshot_path":pin.checkpoint0.as_ref().map(|checkpoint| &checkpoint.snapshot_path),
        "original_snapshot_sha256":original_digest,
        "in_memory_snapshot_sha256":format!("{:x}",Sha256::digest(&rebound)),
        "pre_source_sha256":ORIGINAL_SOURCE,
        "post_source_sha256":TARGET_SOURCE,
        "kernel_sha256":KERNEL,
        "geometry_sha256":GEOMETRY,
        "config_file_sha256":CONFIG,
        "unchanged_non_source_payload_sha256":format!("{:x}",Sha256::digest(&payload)),
        "changed_fields":["source_sha256"],
        "seed":1,"epoch":0,"evaluation_only":true,"training_resume":false,
        "rebound_checkpoint_written":false,
        "reason":"Kernel unchanged; source-set identity moved because profiling sources are part of the module source."
    });
    Ok((rebound, Some(receipt)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin() -> Pin {
        serde_json::from_str(include_str!(
            "../../../propose-subphase-diagnostic-2026-10-16/diagnostic-pin.json"
        ))
        .expect("registered pin")
    }

    fn compiled() -> Value {
        json!({"source_sha256":TARGET_SOURCE,"kernel_sha256":KERNEL,"geometry_sha256":GEOMETRY})
    }

    #[test]
    fn mapping_is_exact_and_cannot_authorize_training_q4_or_other_artifacts() {
        let original = serde_json::to_value(pin()).expect("pin JSON");
        assert!(super::super::validate(&pin()).is_ok());
        for (path, wrong) in [
            (
                "/diagnostic/checkpoint0_identity_bridge/schema",
                json!("wrong"),
            ),
            (
                "/diagnostic/checkpoint0_identity_bridge/original_source_sha256",
                json!(TARGET_SOURCE),
            ),
            (
                "/diagnostic/checkpoint0_identity_bridge/target_source_sha256",
                json!(ORIGINAL_SOURCE),
            ),
            (
                "/diagnostic/checkpoint0_identity_bridge/original_snapshot_sha256",
                json!("0".repeat(64)),
            ),
            (
                "/diagnostic/checkpoint0_identity_bridge/evaluation_only",
                json!(false),
            ),
            (
                "/diagnostic/checkpoint0_identity_bridge/training_resume",
                json!(true),
            ),
            ("/diagnostic/source_sha256", json!(ORIGINAL_SOURCE)),
            ("/diagnostic/kernel_sha256", json!("0".repeat(64))),
            ("/diagnostic/geometry_sha256", json!("0".repeat(64))),
            ("/diagnostic/config_file_sha256", json!("0".repeat(64))),
            ("/checkpoint0/snapshot_sha256", json!("0".repeat(64))),
            ("/checkpoint0/seed", json!(2)),
            ("/checkpoint0/epoch", json!(1)),
            ("/cells/0/source", json!("sealed_seed1_q4")),
        ] {
            let mut changed = original.clone();
            *changed.pointer_mut(path).expect("field") = wrong;
            let changed: Pin = serde_json::from_value(changed).expect("typed pin");
            assert!(super::super::validate(&changed).is_err(), "accepted {path}");
            let error =
                prepare(&changed, Source::Checkpoint0, b"{}", &compiled()).expect_err("refused");
            assert!(error.contains(ORIGINAL_SOURCE) && error.contains(TARGET_SOURCE));
            assert!(error.contains("checkpoint0-timing-snapshot.json"));
        }
    }

    #[test]
    fn same_kernel_is_not_an_artifact_or_source_wildcard() {
        for field in ["source_sha256", "kernel_sha256", "geometry_sha256"] {
            let mut wrong = compiled();
            wrong[field] = json!("0".repeat(64));
            assert!(prepare(&pin(), Source::Checkpoint0, b"{}", &wrong)
                .expect_err("compiled refusal")
                .contains(field));
        }
        assert!(prepare(&pin(), Source::Checkpoint0, b"{}", &compiled())
            .expect_err("artifact refusal")
            .contains("artifact checksum"));
        assert!(prepare(&pin(), Source::SealedSeed1Q4, b"{}", &compiled()).is_err());
    }

    #[test]
    fn no_bridge_is_a_byte_preserving_noop_not_implicit_admission() {
        let mut pin = pin();
        pin.diagnostic
            .as_mut()
            .expect("diagnostic")
            .checkpoint0_identity_bridge = None;
        let (bytes, receipt) =
            prepare(&pin, Source::Checkpoint0, b"unchanged", &json!({})).expect("no adapter");
        assert_eq!(bytes, b"unchanged");
        assert!(receipt.is_none());
        assert_eq!(contextualize(&pin, "failure"), "failure");
    }

    #[test]
    #[ignore = "requires archived exact checkpoint bytes via UOR_SEALED_CHECKPOINT0; metadata hashing only"]
    fn authentic_sealed_checkpoint_changes_only_source_identity_and_records_mapping() {
        let path = std::env::var("UOR_SEALED_CHECKPOINT0").expect("sealed artifact path");
        let original = std::fs::read(path).expect("read sealed bytes");
        let (rebound, receipt) =
            prepare(&pin(), Source::Checkpoint0, &original, &compiled()).expect("exact bridge");
        let before: Value = serde_json::from_slice(&original).expect("original JSON");
        let mut after: Value = serde_json::from_slice(&rebound).expect("rebound JSON");
        assert_eq!(after["source_sha256"], TARGET_SOURCE);
        after["source_sha256"] = json!(ORIGINAL_SOURCE);
        assert_eq!(after, before);
        let receipt = receipt.expect("receipt");
        assert_eq!(receipt["bridge_used"], true);
        assert_eq!(receipt["pre_source_sha256"], ORIGINAL_SOURCE);
        assert_eq!(receipt["post_source_sha256"], TARGET_SOURCE);
        assert_eq!(receipt["training_resume"], false);
        assert_eq!(receipt["rebound_checkpoint_written"], false);
        let mut corrupted = original.clone();
        corrupted.push(b' ');
        assert!(prepare(&pin(), Source::Checkpoint0, &corrupted, &compiled()).is_err());
        assert!(prepare(&pin(), Source::Checkpoint0, &rebound, &compiled()).is_err());
        println!(
            "{}",
            serde_json::to_string_pretty(&receipt).expect("receipt JSON")
        );
    }
}
