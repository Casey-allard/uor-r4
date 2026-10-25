//! Host controls for the address-metric pre-flight configs (#2834, claude-110):
//! the six arms of `docs/harness/prep/m1-address-key-full-vector-2026-10-25.md`.
//! A8-Outer is the committed A8 seed-1 config byte for byte. Each treatment
//! differs from it only in its pinned table, its address metric and (FV16) K.
//! Every arm validates for the attention pre-flight scope (K8 arms) or the
//! driver (FV16), and every arm has its own lineage scope.
use serde_json::Value;
use std::collections::BTreeSet;
use uor_r4_discrete::address_cells::AddressMetric;
use uor_r4_discrete::controller::attention_learning;
use uor_r4_discrete::training::{CodebookInitialization, DriverConfig};

const A8: &[u8] = include_bytes!(
    "../../../docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json"
);
const FV8_ENV: &[u8] =
    include_bytes!("../../../docs/labs/m1-address-key-full-vector/configs/fv8-env.json");
const FV8_TWO: &[u8] =
    include_bytes!("../../../docs/labs/m1-address-key-full-vector/configs/fv8-two-scale.json");
const FVS8_ENV: &[u8] =
    include_bytes!("../../../docs/labs/m1-address-key-full-vector/configs/fvs8-env.json");
const FVS8_TWO: &[u8] =
    include_bytes!("../../../docs/labs/m1-address-key-full-vector/configs/fvs8-two-scale.json");
const FV16_TWO: &[u8] =
    include_bytes!("../../../docs/labs/m1-address-key-full-vector/configs/fv16-two-scale.json");

const A8_TABLE: &str = "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1";
const FV_TABLE: &str = "2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2";
const FVS_TABLE: &str = "5e5abf336bfb039d6c3bde336cf5d03f207ae9ff7bc8cfb21e71691c622313c4";

fn table(config: &DriverConfig) -> &str {
    match &config.codebook_initialization {
        CodebookInitialization::PinnedTableV1 { table } => &table.sha256,
        CodebookInitialization::SeededDeviceV1 => "",
    }
}

/// The config with its table, metric and K removed: what every arm must share.
fn common(bytes: &[u8]) -> Value {
    let mut v: Value = serde_json::from_slice(bytes).unwrap();
    let object = v.as_object_mut().unwrap();
    object.remove("codebook_initialization");
    object.remove("address_metric");
    object["spec"].as_object_mut().unwrap().remove("kept_cells");
    v
}

#[test]
fn the_six_pre_flight_arms_differ_only_in_table_metric_and_k() {
    let arms: [(&str, &[u8], &str, AddressMetric, u32); 6] = [
        ("a8-outer", A8, A8_TABLE, AddressMetric::OuterL1V1, 8),
        (
            "fv8-env",
            FV8_ENV,
            FV_TABLE,
            AddressMetric::CellEnvelopeL1V1,
            8,
        ),
        (
            "fv8-two-scale",
            FV8_TWO,
            FV_TABLE,
            AddressMetric::FullVectorCellL1V1,
            8,
        ),
        (
            "fvs8-env",
            FVS8_ENV,
            FVS_TABLE,
            AddressMetric::CellEnvelopeL1V1,
            8,
        ),
        (
            "fvs8-two-scale",
            FVS8_TWO,
            FVS_TABLE,
            AddressMetric::FullVectorCellL1V1,
            8,
        ),
        (
            "fv16-two-scale",
            FV16_TWO,
            FV_TABLE,
            AddressMetric::FullVectorCellL1V1,
            16,
        ),
    ];
    let base = common(A8);
    let raw: Value = serde_json::from_slice(A8).unwrap();
    assert!(
        raw.get("address_metric").is_none(),
        "A8-Outer is the default key"
    );
    let mut scopes = BTreeSet::new();
    for (name, bytes, sha, metric, k) in arms {
        let config: DriverConfig = serde_json::from_slice(bytes).unwrap();
        config.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(table(&config), sha, "{name}");
        assert_eq!(config.address_metric, metric, "{name}");
        assert_eq!(config.spec.kept_cells, k, "{name}");
        assert_eq!(
            common(bytes),
            base,
            "{name} differs from A8 beyond table/metric/K"
        );
        if k == 8 {
            attention_learning::validate_config(&config).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                scopes.insert(attention_learning::scope(&config).unwrap()),
                "{name} scope repeats"
            );
        }
    }
    assert_eq!(scopes.len(), 5);
}
