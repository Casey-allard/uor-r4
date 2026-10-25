//! Snapshot storage types and exact serialization hash domains; no model execution.

use crate::address_cells::AddressMetric;
use crate::normalization_mechanism::NormalizationMode;
use crate::position_mechanism::PositionConfig;
use crate::{Error, ObjectivePolicy, ReadInitialization, ReadSpec, Result, Spec};
fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// Artifact storage only. Values are admitted on the GPU when loaded; no
/// host score, route or transport is computed during serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: ReadSpec,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicy::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicy,
    #[serde(default, skip_serializing_if = "ReadInitialization::is_zero_gates")]
    pub read_initialization: ReadInitialization,
    #[serde(
        default,
        skip_serializing_if = "NormalizationMode::is_scale_matched_snap"
    )]
    pub normalization_mode: NormalizationMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionConfig>,
    #[serde(default, skip_serializing_if = "AddressMetric::is_outer")]
    pub address_metric: AddressMetric,
    pub seed: u32,
    pub epoch: u64,
    pub table: Vec<i32>,
    pub routes: Vec<u32>,
    /// Explicit full [blocks,stages] trit inventory in both modes. It is
    /// inactive in Route mode; there is no schema1/default-mode migration.
    pub gates: Vec<i32>,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

/// Two encodings of one snapshot, never interchangeable with a raw FilePin or
/// the encoding of the enclosing DriverCheckpoint. This is storage metadata;
/// computing these hashes does not construct a Device or evaluate a model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotHashDomains {
    /// Domain A: serialization of the serde_json::Value stored in a checkpoint.
    pub model_storage_sha256: String,
    /// Domain B: serialization of the typed Snapshot in declared field order.
    pub model_snapshot_sha256: String,
}

pub fn snapshot_hash_domains(snapshot: &Snapshot) -> Result<SnapshotHashDomains> {
    let stored = serde_json::to_value(snapshot).map_err(|e| invalid(e.to_string()))?;
    stored_snapshot_hash_domains(&stored)
}

pub fn stored_snapshot_hash_domains(stored: &serde_json::Value) -> Result<SnapshotHashDomains> {
    let snapshot: Snapshot =
        serde_json::from_value(stored.clone()).map_err(|e| invalid(e.to_string()))?;
    Ok(SnapshotHashDomains {
        // Hash the original stored Value, not a normalized typed round trip.
        model_storage_sha256: crate::corpus::sha256(
            &serde_json::to_vec(stored).map_err(|e| invalid(e.to_string()))?,
        ),
        model_snapshot_sha256: crate::corpus::sha256(
            &serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn stored() -> serde_json::Value {
        json!({"schema":crate::SCHEMA,"spec":{"vocabulary":8,"blocks":2,"context":4,"kept_cells":2},
            "read_spec":{"kind":"route"},"seed":1,"epoch":2,"table":[1,-1],"routes":[0],"gates":[0],
            "operator_kappa":"a","composition_kappa":"b","kernel_sha256":"1".repeat(64),
            "source_sha256":"2".repeat(64),"geometry_sha256":"3".repeat(64)})
    }

    #[test]
    fn original_typed_encoding_and_explicit_default_storage_domains() {
        let mut value = stored();
        // ReadSpec's established serialization is taken from its public type.
        value["read_spec"] = serde_json::to_value(crate::ReadSpec::Route).unwrap();
        let snapshot: Snapshot = serde_json::from_value(value.clone()).unwrap();
        let typed = serde_json::to_vec(&snapshot).unwrap();
        let mut explicit = value.clone();
        explicit["objective_policy"] =
            serde_json::to_value(crate::ObjectivePolicy::default()).unwrap();
        explicit["read_initialization"] =
            serde_json::to_value(crate::ReadInitialization::default()).unwrap();
        explicit["normalization_mode"] =
            serde_json::to_value(NormalizationMode::default()).unwrap();
        let a = stored_snapshot_hash_domains(&value).unwrap();
        let b = stored_snapshot_hash_domains(&explicit).unwrap();
        assert_eq!(a.model_snapshot_sha256, crate::corpus::sha256(&typed));
        assert_eq!(a.model_snapshot_sha256, b.model_snapshot_sha256);
        assert_ne!(a.model_storage_sha256, b.model_storage_sha256);
        assert_eq!(
            snapshot_hash_domains(&snapshot)
                .unwrap()
                .model_snapshot_sha256,
            a.model_snapshot_sha256
        );
        let text = String::from_utf8(typed).unwrap();
        let golden = concat!(
            "{\"schema\":\"uor-r4.discrete-spiralcore-fbs/2\",",
            "\"spec\":{\"vocabulary\":8,\"blocks\":2,\"context\":4,\"kept_cells\":2},",
            "\"read_spec\":{\"kind\":\"route\"},\"seed\":1,\"epoch\":2,",
            "\"table\":[1,-1],\"routes\":[0],\"gates\":[0],",
            "\"operator_kappa\":\"a\",\"composition_kappa\":\"b\",\"kernel_sha256\":\""
        )
        .to_owned()
            + &"1".repeat(64)
            + "\",\"source_sha256\":\""
            + &"2".repeat(64)
            + "\",\"geometry_sha256\":\""
            + &"3".repeat(64)
            + "\"}";
        assert_eq!(text, golden);
        assert!(!text.contains("objective_policy"));
        assert!(!text.contains("read_initialization"));
        assert!(!text.contains("normalization_mode"));
        value["unexpected"] = true.into();
        assert!(stored_snapshot_hash_domains(&value).is_err());
    }

    #[test]
    fn unit_snap_control_mode_is_stored_and_reloaded() {
        let mut value = stored();
        value["read_spec"] = serde_json::to_value(crate::ReadSpec::Route).unwrap();
        let mut snapshot: Snapshot = serde_json::from_value(value).unwrap();
        assert_eq!(snapshot.normalization_mode, NormalizationMode::default());
        let default_domains = snapshot_hash_domains(&snapshot).unwrap();
        snapshot.normalization_mode = NormalizationMode::UnitSnapControlV1;
        let text = serde_json::to_string(&snapshot).unwrap();
        assert!(text.contains("\"normalization_mode\":\"unit_snap_control_v1\""));
        let reloaded: Snapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(
            reloaded.normalization_mode,
            NormalizationMode::UnitSnapControlV1
        );
        let control_domains = snapshot_hash_domains(&snapshot).unwrap();
        assert_ne!(
            default_domains.model_snapshot_sha256,
            control_domains.model_snapshot_sha256
        );
    }

    #[test]
    fn optional_position_config_is_stored_and_reloaded() {
        use crate::position_mechanism::{PositionConfig, PositionMode, PositionStepTable};

        let mut value = stored();
        value["read_spec"] = serde_json::to_value(crate::ReadSpec::Route).unwrap();
        let mut snapshot: Snapshot = serde_json::from_value(value).unwrap();
        assert_eq!(snapshot.position, None);
        let default_text = serde_json::to_string(&snapshot).unwrap();
        assert!(!default_text.contains("\"position\":"));

        snapshot.position = Some(PositionConfig::new(
            PositionMode::PrefixHolonomyV1,
            PositionStepTable::identity(2).unwrap(),
        ));
        let text = serde_json::to_string(&snapshot).unwrap();
        assert!(text.contains("\"mode\":\"prefix_holonomy_v1\""));
        assert!(text.contains("\"candidates\":[0,0"));
        let reloaded: Snapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(reloaded.position, snapshot.position);
    }

    #[test]
    fn stored_and_typed_hashes_change_for_snapshot_mutation() {
        let mut value = stored();
        value["read_spec"] = serde_json::to_value(crate::ReadSpec::Route).unwrap();
        let before = stored_snapshot_hash_domains(&value).unwrap();
        value["epoch"] = 3.into();
        let after = stored_snapshot_hash_domains(&value).unwrap();
        assert_ne!(before.model_storage_sha256, after.model_storage_sha256);
        assert_ne!(before.model_snapshot_sha256, after.model_snapshot_sha256);
    }
}
