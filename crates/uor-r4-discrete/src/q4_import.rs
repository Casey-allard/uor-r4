//! Evaluation-only importer for exactly one historical checkpoint: the sealed W2
//! Q4 completion checkpoint of 15 Oct (`4477eb5c…`, 16,404 batches, epoch 407).
//!
//! Today's code refuses Q4 twice: the driver lineage moved (`validate_checkpoint`) and
//! the snapshot's compiled source and kernel hashes no longer equal today's build
//! (`Device::from_snapshot`). This module changes only the second binding, in memory:
//! it authenticates the original bytes and the stored model Value, rebinds exactly the
//! snapshot's `source_sha256` and `kernel_sha256` to the caller's actual compiled
//! identity, proves that reversing those two fields restores the stored Value exactly,
//! and hands the result to the unchanged `Device::from_snapshot`.
//!
//! The imported model can only be read: [`ImportedDevice`] exposes carry, forward and
//! evaluate, never apply, save or snapshot export. No file is written, no lineage or
//! cursor is rewritten, and the production checkpoint path is untouched.

use crate::snapshot::Snapshot;
use crate::training::Cursor;
use crate::{ReadSpec, Spec};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The only admissible artifact: exact length and raw SHA-256 of the wrapper file.
pub const ORIGINAL_LEN: usize = 3_138_786;
pub const ORIGINAL_SHA256: &str =
    "4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf";
/// Domain A: SHA-256 of `serde_json::to_vec` of the stored `.checkpoint.model` Value.
pub const STORED_MODEL_SHA256: &str =
    "49fa448cb3349ac5711966082a9c53918a693d45e11f282221d240cbb463fa73";
pub const OLD_SOURCE_SHA256: &str =
    "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a";
pub const OLD_KERNEL_SHA256: &str =
    "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b";
pub const GEOMETRY_SHA256: &str =
    "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327";
pub const OPERATOR_KAPPA: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
pub const COMPOSITION_KAPPA: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";
const WRAPPER_SCHEMA: &str = "uor-r4.completion-checkpoint/1";
const CHECKPOINT_SCHEMA: &str = "uor-r4.discrete-driver-checkpoint/1";
const SPEC: Spec = Spec {
    vocabulary: 4096,
    blocks: 4,
    context: 192,
    kept_cells: 8,
};
const SEED: u32 = 1;
const EPOCH: u64 = 407;
const COMPLETED_BATCHES: u64 = 16_404;
const ACCEPTED_UPDATES: u64 = 407;
const NEXT_TARGET: u64 = 1_049_856;
/// The two snapshot fields the importer rebinds, by their serialized names.
pub const CHANGED_FIELDS: [&str; 2] = ["source_sha256", "kernel_sha256"];

/// The caller's actual compiled identity; the importer writes its source and kernel
/// into the snapshot and requires its geometry to equal Q4's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetIdentity {
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
}

impl TargetIdentity {
    /// Reads the three fields from `device::compiled_identity()`'s JSON.
    pub fn from_compiled_identity(identity: &Value) -> Result<Self, String> {
        let field = |name: &str| -> Result<String, String> {
            let value = identity
                .get(name)
                .and_then(Value::as_str)
                .ok_or_else(|| format!("compiled identity lacks string {name}"))?;
            require_hash(name, value)?;
            Ok(value.to_owned())
        };
        Ok(Self {
            source_sha256: field("source_sha256")?,
            kernel_sha256: field("kernel_sha256")?,
            geometry_sha256: field("geometry_sha256")?,
        })
    }

    /// This build's actual compiled identity.
    #[cfg(feature = "cuda")]
    pub fn compiled() -> Result<Self, String> {
        Self::from_compiled_identity(&crate::device::compiled_identity())
    }

    fn validate(&self) -> Result<(), String> {
        require_hash("target source_sha256", &self.source_sha256)?;
        require_hash("target kernel_sha256", &self.kernel_sha256)?;
        require_hash("target geometry_sha256", &self.geometry_sha256)?;
        if self.geometry_sha256 != GEOMETRY_SHA256 {
            return Err(
                "target geometry differs from Q4's; the importer rebinds only source and kernel"
                    .into(),
            );
        }
        Ok(())
    }
}

/// What the import did, for the verifier's `original_to_target_bridge`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportReceipt {
    pub schema: String,
    pub original_sha256: String,
    pub original_len: u64,
    pub stored_model_sha256: String,
    pub old_source_sha256: String,
    pub old_kernel_sha256: String,
    pub new_source_sha256: String,
    pub new_kernel_sha256: String,
    pub geometry_sha256: String,
    pub changed_fields: Vec<String>,
    pub evaluation_only: bool,
    pub no_apply: bool,
}
pub const RECEIPT_SCHEMA: &str = "uor-r4.q4-import-receipt/1";

/// The rebound snapshot and its receipt. The snapshot is private: its only exit is
/// [`ImportedQ4::into_device`], so it cannot be saved or re-exported.
#[derive(Debug)]
pub struct ImportedQ4 {
    snapshot: Snapshot,
    receipt: ImportReceipt,
}

impl ImportedQ4 {
    pub fn receipt(&self) -> &ImportReceipt {
        &self.receipt
    }

    /// Loads the rebound snapshot through the unchanged `Device::from_snapshot`.
    #[cfg(feature = "cuda")]
    pub fn into_device(self, ordinal: usize) -> crate::Result<ImportedDevice> {
        Ok(ImportedDevice {
            device: crate::device::Device::from_snapshot(self.snapshot, ordinal)?,
            receipt: self.receipt,
        })
    }
}

/// A read-only view of the imported Q4 model. The Device is private and only the
/// methods below reach it: no apply, propose, save or snapshot export.
#[cfg(feature = "cuda")]
pub struct ImportedDevice {
    device: crate::device::Device,
    receipt: ImportReceipt,
}

#[cfg(feature = "cuda")]
impl ImportedDevice {
    pub fn receipt(&self) -> &ImportReceipt {
        &self.receipt
    }
    pub fn spec(&self) -> Spec {
        self.device.spec()
    }
    pub fn read_spec(&self) -> ReadSpec {
        self.device.read_spec()
    }
    pub fn carry(
        &self,
        identities: &[crate::Identity],
    ) -> crate::Result<crate::device::CarriedBatch> {
        self.device.carry(identities)
    }
    pub fn carry_windows(
        &self,
        windows: &[Vec<crate::Identity>],
    ) -> crate::Result<crate::device::CarriedBatch> {
        self.device.carry_windows(windows)
    }
    pub fn forward(
        &self,
        batch: &crate::device::CarriedBatch,
        targets: &[u32],
    ) -> crate::Result<crate::device::Forward> {
        self.device.forward(batch, targets)
    }
    pub fn evaluate(
        &self,
        batch: &crate::device::CarriedBatch,
        targets: &[u32],
    ) -> crate::Result<crate::device::Evaluation> {
        self.device.evaluate(batch, targets)
    }
    /// Read-only: the accepted-update epoch the imported model carries.
    pub fn epoch(&self) -> crate::Result<u64> {
        self.device.epoch()
    }
    pub fn ordinal(&self) -> usize {
        self.device.ordinal()
    }
    /// Read-only CUDA aggregation of completed evaluation receipts.
    pub fn aggregate_evaluations(
        &self,
        evaluations: &[crate::device::Evaluation],
    ) -> crate::Result<crate::device::EvaluationAggregate> {
        self.device.aggregate_evaluations(evaluations)
    }
    /// Read-only CUDA export of each token's cell, indexed by token id.
    pub fn export_token_cells(&self) -> crate::Result<Vec<u16>> {
        self.device.export_token_cells()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Wrapper {
    schema: String,
    admission_sha256: Value,
    baseline_evaluation: Value,
    baseline_snapshot_metadata: Value,
    checkpoint: StoredCheckpoint,
    pilot_parent: Value,
    route: Value,
    vector: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct StoredCheckpoint {
    schema: String,
    lineage: Value,
    cursor: Cursor,
    model_sha256: String,
    model: Value,
}

/// Admits only the pinned Q4 bytes and rebinds them to `target`.
pub fn prepare(original_bytes: &[u8], target: &TargetIdentity) -> Result<ImportedQ4, String> {
    if original_bytes.len() != ORIGINAL_LEN {
        return Err(format!(
            "Q4 original length {} differs from the pinned {ORIGINAL_LEN}",
            original_bytes.len()
        ));
    }
    if crate::corpus::sha256(original_bytes) != ORIGINAL_SHA256 {
        return Err("Q4 original raw sha256 differs from the pin".into());
    }
    authenticate_and_rebind(original_bytes, STORED_MODEL_SHA256, target)
}

/// Everything after the raw pin. `stored_model_sha256` is the fixed constant in
/// production; tests pass another value only to reach the later checks.
fn authenticate_and_rebind(
    bytes: &[u8],
    stored_model_sha256: &str,
    target: &TargetIdentity,
) -> Result<ImportedQ4, String> {
    target.validate()?;
    let wrapper: Wrapper =
        serde_json::from_slice(bytes).map_err(|e| format!("Q4 wrapper does not parse: {e}"))?;
    if wrapper.schema != WRAPPER_SCHEMA || wrapper.checkpoint.schema != CHECKPOINT_SCHEMA {
        return Err("Q4 wrapper or checkpoint schema differs".into());
    }
    let checkpoint = wrapper.checkpoint;
    // Domain A, before any typed parse or default.
    let stored = &checkpoint.model;
    let domain = crate::corpus::sha256(&serde_json::to_vec(stored).map_err(|e| e.to_string())?);
    if checkpoint.model_sha256 != stored_model_sha256 || domain != stored_model_sha256 {
        return Err("Q4 stored model Value domain hash differs from the pin".into());
    }
    let cursor = &checkpoint.cursor;
    if cursor.completed_batches != COMPLETED_BATCHES
        || cursor.accepted_updates != ACCEPTED_UPDATES
        || cursor.corpus_pass != 0
        || cursor.next_target != NEXT_TARGET
        || cursor.next_coordinate != 0
        || cursor.block.is_some()
    {
        return Err("Q4 cursor differs from completed16404/accepted407".into());
    }
    let mut snapshot: Snapshot = serde_json::from_value(stored.clone())
        .map_err(|e| format!("Q4 model does not parse as today's Snapshot: {e}"))?;
    if snapshot.schema != crate::SCHEMA
        || snapshot.spec != SPEC
        || snapshot.read_spec != ReadSpec::Route
        || !snapshot.objective_policy.is_nearest_address_margin()
        || !snapshot.read_initialization.is_zero_gates()
        || !snapshot.normalization_mode.is_scale_matched_snap()
        || snapshot.position.is_some()
        || snapshot.seed != SEED
        || snapshot.epoch != EPOCH
        || snapshot.table.len() != (SPEC.vocabulary * SPEC.blocks * 8) as usize
        || snapshot.routes.len() != 20
        || snapshot.gates.len() != 20
        || snapshot.operator_kappa != OPERATOR_KAPPA
        || snapshot.composition_kappa != COMPOSITION_KAPPA
        || snapshot.source_sha256 != OLD_SOURCE_SHA256
        || snapshot.kernel_sha256 != OLD_KERNEL_SHA256
        || snapshot.geometry_sha256 != GEOMETRY_SHA256
    {
        return Err("Q4 snapshot is not the legacy V4096/B4/C192/K8 Route seed1 epoch407 configuration with the pinned identity".into());
    }
    snapshot.source_sha256 = target.source_sha256.clone();
    snapshot.kernel_sha256 = target.kernel_sha256.clone();
    require_reverse_equality(&snapshot, stored)?;
    Ok(ImportedQ4 {
        snapshot,
        receipt: ImportReceipt {
            schema: RECEIPT_SCHEMA.into(),
            original_sha256: ORIGINAL_SHA256.into(),
            original_len: ORIGINAL_LEN as u64,
            stored_model_sha256: domain,
            old_source_sha256: OLD_SOURCE_SHA256.into(),
            old_kernel_sha256: OLD_KERNEL_SHA256.into(),
            new_source_sha256: target.source_sha256.clone(),
            new_kernel_sha256: target.kernel_sha256.clone(),
            geometry_sha256: GEOMETRY_SHA256.into(),
            changed_fields: CHANGED_FIELDS.iter().map(|f| (*f).to_owned()).collect(),
            evaluation_only: true,
            no_apply: true,
        },
    })
}

/// Serializes the rebound snapshot, puts the two old identity values back, and
/// requires the whole Value to equal the stored one. Today's serde omits a field at
/// its default, so a stored key the legacy file lacked stays absent, and a stored key
/// that today's type would drop or re-encode fails here.
fn require_reverse_equality(rebound: &Snapshot, stored: &Value) -> Result<(), String> {
    let mut reversed = serde_json::to_value(rebound).map_err(|e| e.to_string())?;
    let object = reversed
        .as_object_mut()
        .ok_or("rebound snapshot is not a JSON object")?;
    for (field, old) in CHANGED_FIELDS
        .iter()
        .zip([OLD_SOURCE_SHA256, OLD_KERNEL_SHA256])
    {
        let slot = object
            .get_mut(*field)
            .ok_or_else(|| format!("rebound snapshot lacks {field}"))?;
        *slot = Value::String(old.to_owned());
    }
    if &reversed != stored {
        return Err(
            "reversing the source/kernel rebind does not restore the stored model Value exactly"
                .into(),
        );
    }
    Ok(())
}

fn require_hash(name: &str, value: &str) -> Result<(), String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(format!("{name} is not a lowercase sha256"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn target() -> TargetIdentity {
        TargetIdentity {
            source_sha256: "a".repeat(64),
            kernel_sha256: "b".repeat(64),
            geometry_sha256: GEOMETRY_SHA256.into(),
        }
    }

    fn model() -> Value {
        json!({"schema":crate::SCHEMA,
            "spec":{"vocabulary":4096,"blocks":4,"context":192,"kept_cells":8},
            "read_spec":{"kind":"route"},"seed":1,"epoch":407,
            "table":vec![0i32;131072],"routes":vec![0u32;20],"gates":vec![0i32;20],
            "operator_kappa":OPERATOR_KAPPA,"composition_kappa":COMPOSITION_KAPPA,
            "kernel_sha256":OLD_KERNEL_SHA256,"source_sha256":OLD_SOURCE_SHA256,
            "geometry_sha256":GEOMETRY_SHA256})
    }

    fn domain(model: &Value) -> String {
        crate::corpus::sha256(&serde_json::to_vec(model).unwrap())
    }

    /// A synthetic wrapper whose checkpoint carries `model` and the given model_sha256.
    fn wrapper(model: &Value, model_sha256: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"schema":WRAPPER_SCHEMA,"admission_sha256":"x",
            "baseline_evaluation":{},"baseline_snapshot_metadata":{},"pilot_parent":{},
            "route":{},"vector":{},
            "checkpoint":{"schema":CHECKPOINT_SCHEMA,"lineage":{},
                "cursor":{"completed_batches":16404,"accepted_updates":407,"corpus_pass":0,
                    "next_target":1049856,"next_coordinate":0},
                "model_sha256":model_sha256,"model":model}}))
        .unwrap()
    }

    fn synthetic(model: &Value) -> Result<ImportedQ4, String> {
        let sha = domain(model);
        authenticate_and_rebind(&wrapper(model, &sha), &sha, &target())
    }

    #[test]
    fn q4_import_real_file_maps() {
        let Ok(path) = std::env::var("UOR_W2_Q4") else {
            eprintln!("SKIP q4_import_real_file_maps: set UOR_W2_Q4 to the staged q4-checkpoint-16404.json");
            return;
        };
        let path = std::path::PathBuf::from(path);
        let path = if path.is_dir() {
            path.join("q4-checkpoint-16404.json")
        } else {
            path
        };
        let bytes = std::fs::read(&path).unwrap();
        let imported = prepare(&bytes, &target()).unwrap();
        let receipt = imported.receipt();
        assert_eq!(receipt.original_sha256, ORIGINAL_SHA256);
        assert_eq!(receipt.stored_model_sha256, STORED_MODEL_SHA256);
        assert_eq!(receipt.new_source_sha256, "a".repeat(64));
        assert_eq!(receipt.new_kernel_sha256, "b".repeat(64));
        assert_eq!(imported.snapshot.source_sha256, "a".repeat(64));
        assert_eq!(imported.snapshot.kernel_sha256, "b".repeat(64));
        assert_eq!(imported.snapshot.epoch, 407);
        // Reverse equality against the stored Value, re-derived independently here.
        let wrapper: Value = serde_json::from_slice(&bytes).unwrap();
        let stored = &wrapper["checkpoint"]["model"];
        require_reverse_equality(&imported.snapshot, stored).unwrap();
        // The legacy file carries none of the fields added since 15 Oct.
        for absent in [
            "objective_policy",
            "read_initialization",
            "normalization_mode",
            "position",
        ] {
            assert!(stored.get(absent).is_none(), "{absent} present in Q4");
        }
        // One flipped byte, same length: refused by the raw pin.
        let mut flipped = bytes.clone();
        flipped[ORIGINAL_LEN / 2] ^= 1;
        assert!(prepare(&flipped, &target())
            .unwrap_err()
            .contains("raw sha256"));
        // A tampered seed past the raw pin: refused by the stored-Value domain pin.
        let mut tampered = wrapper.clone();
        tampered["checkpoint"]["model"]["seed"] = 2.into();
        let tampered = serde_json::to_vec(&tampered).unwrap();
        assert!(
            authenticate_and_rebind(&tampered, STORED_MODEL_SHA256, &target())
                .unwrap_err()
                .contains("domain hash")
        );
    }

    #[test]
    fn q4_import_refuses_flipped_byte_and_wrong_length() {
        let pinned_length = vec![0u8; ORIGINAL_LEN];
        assert!(prepare(&pinned_length, &target())
            .unwrap_err()
            .contains("raw sha256"));
        for len in [0, ORIGINAL_LEN - 1, ORIGINAL_LEN + 1] {
            assert!(prepare(&vec![0u8; len], &target())
                .unwrap_err()
                .contains("length"));
        }
        // The synthetic wrapper, which maps on its own pins, still fails the fixed pins.
        let bytes = wrapper(&model(), &domain(&model()));
        assert!(prepare(&bytes, &target()).is_err());
    }

    #[test]
    fn q4_import_tampered_model_is_refused_by_domain_pin() {
        let mut tampered = model();
        tampered["seed"] = 2.into();
        // Past the raw pin, under the fixed domain pin.
        let bytes = wrapper(&tampered, STORED_MODEL_SHA256);
        assert!(
            authenticate_and_rebind(&bytes, STORED_MODEL_SHA256, &target())
                .unwrap_err()
                .contains("domain hash")
        );
        // A table entry: same refusal.
        let mut table = model();
        table["table"][7] = 1.into();
        let bytes = wrapper(&table, STORED_MODEL_SHA256);
        assert!(authenticate_and_rebind(&bytes, STORED_MODEL_SHA256, &target()).is_err());
        // Even with the domain pin re-pinned to the tampered Value, the legacy checks refuse.
        assert!(synthetic(&tampered).unwrap_err().contains("legacy"));
    }

    #[test]
    fn q4_import_tampered_explicit_default_is_refused_by_reverse_equality() {
        // Each parses to the legacy default, so only the reverse equality sees it.
        for (key, value) in [
            ("position", Value::Null),
            (
                "normalization_mode",
                serde_json::to_value(crate::normalization_mechanism::NormalizationMode::default())
                    .unwrap(),
            ),
            (
                "objective_policy",
                serde_json::to_value(crate::ObjectivePolicy::default()).unwrap(),
            ),
            (
                "read_initialization",
                serde_json::to_value(crate::ReadInitialization::default()).unwrap(),
            ),
        ] {
            let mut tampered = model();
            tampered[key] = value;
            let error = synthetic(&tampered).unwrap_err();
            assert!(error.contains("reversing"), "{key}: {error}");
        }
    }

    #[test]
    fn q4_import_refuses_non_legacy_config() {
        for (key, value) in [
            ("epoch", json!(408)),
            (
                "spec",
                json!({"vocabulary":4096,"blocks":4,"context":192,"kept_cells":16}),
            ),
            ("normalization_mode", json!("unit_snap_control_v1")),
            ("source_sha256", json!("c".repeat(64))),
            ("geometry_sha256", json!("d".repeat(64))),
        ] {
            let mut changed = model();
            changed[key] = value;
            assert!(synthetic(&changed).unwrap_err().contains("legacy"), "{key}");
        }
        let mut wrong_geometry = target();
        wrong_geometry.geometry_sha256 = "e".repeat(64);
        let sha = domain(&model());
        assert!(
            authenticate_and_rebind(&wrapper(&model(), &sha), &sha, &wrong_geometry)
                .unwrap_err()
                .contains("geometry")
        );
    }

    #[test]
    fn q4_import_reverse_equality_holds_and_target_is_written() {
        let imported = synthetic(&model()).unwrap();
        require_reverse_equality(&imported.snapshot, &model()).unwrap();
        let rebound = serde_json::to_value(&imported.snapshot).unwrap();
        let mut expected = model();
        expected["source_sha256"] = "a".repeat(64).into();
        expected["kernel_sha256"] = "b".repeat(64).into();
        assert_eq!(rebound, expected);
        let receipt = imported.receipt();
        assert_eq!(receipt.old_source_sha256, OLD_SOURCE_SHA256);
        assert_eq!(receipt.old_kernel_sha256, OLD_KERNEL_SHA256);
        assert_eq!(receipt.new_source_sha256, target().source_sha256);
        assert_eq!(receipt.new_kernel_sha256, target().kernel_sha256);
        assert_eq!(receipt.changed_fields, ["source_sha256", "kernel_sha256"]);
        assert!(receipt.evaluation_only && receipt.no_apply);
    }

    #[test]
    fn q4_import_target_identity_reads_compiled_identity_json() {
        let identity = json!({"source_sha256":"a".repeat(64),"kernel_sha256":"b".repeat(64),
            "geometry_sha256":GEOMETRY_SHA256,"cuda_images":{}});
        assert_eq!(
            TargetIdentity::from_compiled_identity(&identity).unwrap(),
            target()
        );
        let mut bad = identity.clone();
        bad["kernel_sha256"] = "B".repeat(64).into();
        assert!(TargetIdentity::from_compiled_identity(&bad).is_err());
    }

    #[test]
    fn q4_import_imported_device_exposes_no_apply_or_save() {
        let source = include_str!("q4_import.rs");
        let start = source.find("impl ImportedDevice {").unwrap();
        let body = &source[start..start + source[start..].find("\n}\n").unwrap()];
        let methods: Vec<&str> = body
            .split("pub fn ")
            .skip(1)
            .map(|rest| rest.split('(').next().unwrap())
            .collect();
        assert_eq!(
            methods,
            [
                "receipt",
                "spec",
                "read_spec",
                "carry",
                "carry_windows",
                "forward",
                "evaluate",
                "epoch",
                "ordinal",
                "aggregate_evaluations",
                "export_token_cells"
            ]
        );
        let imported_q4 = source.find("impl ImportedQ4 {").unwrap();
        let body = &source[imported_q4..imported_q4 + source[imported_q4..].find("\n}\n").unwrap()];
        assert_eq!(
            body.matches("pub fn ").count(),
            2,
            "receipt and into_device only"
        );
    }
}
