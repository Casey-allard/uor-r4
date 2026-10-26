//! Stage-1 wider code carrier (`m1-wider-code-carrier`, #2847):
//! - Host grading of stored zero-training GPU evaluation receipts across $B \in \{4, 6, 8\}$,
//! - D11 (`UORLUT01`) integer export verification at $B = 6$ and $B = 8$,
//! - Finite CUDA qualification (`qualify`, `#[cfg(feature = "cuda")]`), and
//! - Zero-training GPU capacity pre-flight runner (`evaluate_arm` / `run_preflight_suite`,
//!   `#[cfg(feature = "cuda")]`).

use crate::address_key_grade::{
    clears, paired_psucc, Arm, Paired, A8_FULL_ADMITTED, A8_PSUCC_ADMITTED, A8_TOP1, DEV_ROWS,
    PSUCC_ROWS, RUN_BAR,
};
use crate::controller::attention_learning::evidence::B3Summary;
use crate::{Error, Result, Spec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

pub const WIDER_CARRIER_QUALIFICATION_SCHEMA: &str = "uor-r4.wider-carrier-qualification/1";
pub const WIDER_CARRIER_ARM_SCHEMA: &str = "uor-r4.wider-carrier-arm/1";
pub const WIDER_CARRIER_GRADE_SCHEMA: &str = "uor-r4.wider-carrier-grade/1";

/// Sealed `#2834` (`claude-110`) $B = 4$ reference numbers on the same 14,287-row DEV panel.
pub const FV8_B4_ENV_PSUCC: u64 = 1_980;
pub const FV8_B4_ENV_FULL: u64 = 5_709;
pub const FV8_B4_ENV_TOP1: u64 = 118;
pub const FV8_B4_ENV_TOP8: u64 = 1_126;

pub const FV8_B4_TWOSCALE_PSUCC: u64 = 1_802;
pub const FV8_B4_TWOSCALE_FULL: u64 = 5_666;
pub const FV8_B4_TWOSCALE_TOP1: u64 = 118;
pub const FV8_B4_TWOSCALE_TOP8: u64 = 1_179;

pub const QUALIFICATION_NAMES: &[&str] = &[
    "wider_carrier_b6_host_gpu_distance_parity",
    "wider_carrier_b8_host_gpu_distance_parity",
    "wider_carrier_outer_invariance_across_b4_b6_b8",
    "wider_carrier_d11_export_and_state_restoration",
];

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn require(ok: bool, why: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(invalid(why))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiderArmRecord {
    pub schema: String,
    pub label: String,
    pub config_sha256: String,
    pub blocks: u32,
    pub width: u32,
    pub address_metric: String,
    pub mode: String,
    pub model_snapshot_sha256: String,
    pub distinct_vectors: u64,
    pub occupied_cells: u64,
    pub within_cell_tie_tokens: u64,
    pub within_cell_tie_dev_rows: u64,
    pub dev_alias_ceiling_rows: u64,
    pub scored_rows: u64,
    pub full_admitted: u64,
    pub psucc_rows: u64,
    pub psucc_admitted: u64,
    pub complement_admitted: u64,
    pub top1: u64,
    pub top8: u64,
    pub admitted_strict_win_rows: u64,
    pub eval_ms: u64,
    pub decode_tokens_per_s: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d11_artifact_sha256: Option<String>,
    pub b3: B3Summary,
}

fn paired_json(p: Paired) -> Value {
    let diff = p.b as i64 - p.c as i64;
    json!({
        "b": p.b,
        "c": p.c,
        "b_minus_c": diff,
        "clears": clears(p),
    })
}

fn check_arm_panel(record: &WiderArmRecord) -> Result<()> {
    if record.scored_rows != DEV_ROWS
        || record.psucc_rows != PSUCC_ROWS
        || record.b3.full.rows != DEV_ROWS
        || record.b3.psucc.rows != PSUCC_ROWS
        || record.b3.rows.len() as u64 != DEV_ROWS
    {
        return Err(invalid(format!(
            "{} does not score the registered 14,287-row DEV panel with 3,847 P_succ rows",
            record.label
        )));
    }
    let ordinals: std::collections::BTreeSet<u64> =
        record.b3.rows.iter().map(|r| r.row_ordinal).collect();
    let full_adm = record.b3.rows.iter().filter(|r| r.target_admitted).count() as u64;
    let psucc_adm = record
        .b3
        .rows
        .iter()
        .filter(|r| r.is_psucc && r.target_admitted)
        .count() as u64;
    if ordinals.len() as u64 != DEV_ROWS
        || full_adm != record.full_admitted
        || full_adm != record.b3.full.target_admitted
        || psucc_adm != record.psucc_admitted
        || psucc_adm != record.b3.psucc.target_admitted
    {
        return Err(invalid(format!(
            "{} B3 summary counts disagree with its row records",
            record.label
        )));
    }
    Ok(())
}

pub fn check_g_occ(record: &WiderArmRecord) -> bool {
    record.distinct_vectors == 4_096
        && record.occupied_cells == 256
        && record.within_cell_tie_tokens == 0
        && record.within_cell_tie_dev_rows == 0
        && record.dev_alias_ceiling_rows == DEV_ROWS
}

/// Grade one wider-carrier treatment arm against `A8-Outer`, its matched-width
/// shuffled control (`WBS6` / `WBS8`), and an optional $B = 4$ or XOR-pad reference.
pub fn grade_wider_treatment(
    a8_outer: &WiderArmRecord,
    treatment: &WiderArmRecord,
    shuffled: &WiderArmRecord,
    other_control: Option<&WiderArmRecord>,
    g_exact_all: bool,
) -> Result<Value> {
    check_arm_panel(a8_outer)?;
    check_arm_panel(treatment)?;
    check_arm_panel(shuffled)?;
    if let Some(other) = other_control {
        check_arm_panel(other)?;
    }
    let _ = Arm {
        b3: a8_outer.b3.clone(),
        top1: a8_outer.top1,
    };
    let g_occ_treat = check_g_occ(treatment);
    let g_occ_shuf = check_g_occ(shuffled);
    let g_occ = g_occ_treat && g_occ_shuf;
    let vs_a8 = paired_psucc(&treatment.b3, &a8_outer.b3)?;
    let vs_shuffled = paired_psucc(&treatment.b3, &shuffled.b3)?;
    let vs_other = match other_control {
        Some(other) => Some(paired_psucc(&treatment.b3, &other.b3)?),
        None => None,
    };
    let reach = treatment.psucc_admitted > A8_PSUCC_ADMITTED && clears(vs_a8);
    let capacity = treatment
        .dev_alias_ceiling_rows
        .min(treatment.full_admitted)
        > RUN_BAR;
    let outcome = if !g_exact_all || !g_occ {
        "INSTRUMENT"
    } else if reach && capacity && clears(vs_shuffled) {
        "PASS"
    } else if reach && capacity {
        "PARTIAL"
    } else {
        "FAIL"
    };
    Ok(json!({
        "schema": WIDER_CARRIER_GRADE_SCHEMA,
        "arm": treatment.label,
        "blocks": treatment.blocks,
        "width": treatment.width,
        "address_metric": treatment.address_metric,
        "g_exact": g_exact_all,
        "g_occ": {
            "treatment": g_occ_treat,
            "shuffled": g_occ_shuf,
            "within_cell_tie_tokens": treatment.within_cell_tie_tokens,
            "within_cell_tie_dev_rows": treatment.within_cell_tie_dev_rows,
            "dev_alias_ceiling_rows": treatment.dev_alias_ceiling_rows,
        },
        "a8_outer": {
            "psucc_admitted": a8_outer.psucc_admitted,
            "full_admitted": a8_outer.full_admitted,
            "top1": a8_outer.top1,
            "top8": a8_outer.top8,
        },
        "treatment": {
            "psucc_admitted": treatment.psucc_admitted,
            "full_admitted": treatment.full_admitted,
            "top1": treatment.top1,
            "top8": treatment.top8,
            "eval_ms": treatment.eval_ms,
            "decode_tokens_per_s": treatment.decode_tokens_per_s,
        },
        "shuffled": {
            "label": shuffled.label,
            "psucc_admitted": shuffled.psucc_admitted,
            "full_admitted": shuffled.full_admitted,
            "top1": shuffled.top1,
            "top8": shuffled.top8,
        },
        "vs_a8": paired_json(vs_a8),
        "vs_shuffled": paired_json(vs_shuffled),
        "vs_other_control": vs_other.map(|p| {
            let other = other_control.expect("other control present");
            json!({
                "control_label": other.label,
                "control_psucc_admitted": other.psucc_admitted,
                "control_full_admitted": other.full_admitted,
                "control_top1": other.top1,
                "control_top8": other.top8,
                "paired": paired_json(p),
            })
        }),
        "reach": reach,
        "capacity": capacity,
        "outcome": outcome,
    }))
}

/// Verify that a wider-carrier `Spec` ($B \in \{6, 8\}$) and placed table export
/// cleanly through `d11_export::export_d11` (`UORLUT01`) and round-trip through
/// `d11_export::import_d11`.
pub fn verify_d11_export_for_table(spec: Spec, table: &[i32]) -> Result<Value> {
    let stages = crate::metadata::BinderPlan::new(spec.context as usize)?.stages();
    let gate_len = spec.blocks as usize * stages + 5 * stages;
    let mut gates = vec![0i32; gate_len];
    for b in 0..spec.blocks as usize {
        gates[b * stages] = 1;
    }
    let routes = vec![0u32; spec.blocks as usize * stages];
    let snapshot = crate::snapshot::Snapshot {
        schema: crate::SCHEMA.to_owned(),
        spec,
        read_spec: crate::ReadSpec::Relation { kept: 8, sweeps: 1 },
        objective_policy: crate::ObjectivePolicy::NearestAddressMarginV1,
        read_initialization: crate::ReadInitialization::ActiveLeafV1,
        normalization_mode: crate::normalization_mechanism::NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        seed: 1,
        epoch: 0,
        table: table.to_vec(),
        routes,
        gates,
        operator_kappa: geometry::OPERATOR_KAPPA.to_owned(),
        composition_kappa: geometry::COMPOSITION_KAPPA.to_owned(),
        kernel_sha256: "22".repeat(32),
        source_sha256: "00".repeat(32),
        geometry_sha256: "11".repeat(32),
    };
    let artifact = crate::d11_export::export_d11(&snapshot)?;
    let restored = crate::d11_export::import_d11(&artifact)?;
    let expected_table_ints = spec.vocabulary as usize * spec.blocks as usize * 8;
    let expected_routes_len = spec.blocks as usize * stages;
    require(
        &artifact[..8] == crate::d11_export::MAGIC
            && restored.spec == spec
            && restored.table == snapshot.table
            && restored.routes == snapshot.routes
            && restored.gates == snapshot.gates
            && restored.table.len() == expected_table_ints
            && restored.routes.len() == expected_routes_len,
        "D11 wider-carrier artifact round-trip fields mismatch",
    )?;
    let artifact_sha256 = crate::corpus::sha256(&artifact);
    Ok(json!({
        "format": crate::d11_export::DISCRETE_LUT_SCHEMA,
        "vocabulary": spec.vocabulary,
        "blocks": spec.blocks,
        "context": spec.context,
        "kept_cells": spec.kept_cells,
        "artifact_bytes": artifact.len(),
        "artifact_sha256": artifact_sha256,
        "table_ints": expected_table_ints,
        "routes_len": expected_routes_len,
    }))
}

/// Load a stored `WiderArmRecord` from `arm_dir` (`arm_summary.json` + `dev_b3.json`).
pub fn load_arm_record(arm_dir: &std::path::Path) -> Result<WiderArmRecord> {
    let summary_bytes = std::fs::read(arm_dir.join("arm_summary.json")).map_err(|e| {
        invalid(format!(
            "read {}: {e}",
            arm_dir.join("arm_summary.json").display()
        ))
    })?;
    let mut summary_val: Value =
        serde_json::from_slice(&summary_bytes).map_err(|e| invalid(e.to_string()))?;
    let b3_bytes = std::fs::read(arm_dir.join("dev_b3.json")).map_err(|e| {
        invalid(format!(
            "read {}: {e}",
            arm_dir.join("dev_b3.json").display()
        ))
    })?;
    let b3_val: Value = serde_json::from_slice(&b3_bytes).map_err(|e| invalid(e.to_string()))?;
    if let Some(obj) = summary_val.as_object_mut() {
        obj.insert("b3".to_owned(), b3_val);
    }
    let rec: WiderArmRecord =
        serde_json::from_value(summary_val).map_err(|e| invalid(e.to_string()))?;
    check_arm_panel(&rec)?;
    Ok(rec)
}

/// Grade a full wider-carrier pre-flight directory and qualification report,
/// write `out_path`, and return the graded JSON record.
pub fn grade_wider_suite(
    run_dir: &std::path::Path,
    qual_path: &std::path::Path,
    out_path: &std::path::Path,
) -> Result<Value> {
    let qual_bytes = std::fs::read(qual_path)
        .map_err(|e| invalid(format!("read {}: {e}", qual_path.display())))?;
    let qual_sha256 = crate::corpus::sha256(&qual_bytes);
    let qual_val: Value =
        serde_json::from_slice(&qual_bytes).map_err(|e| invalid(e.to_string()))?;
    let qual_passed = qual_val.get("status").and_then(Value::as_str) == Some("PASS");

    let a8 = load_arm_record(&run_dir.join("a8-outer"))?;
    let fv8_b4_env = load_arm_record(&run_dir.join("fv8-b4-env"))?;
    let fv8_b4_twoscale = load_arm_record(&run_dir.join("fv8-b4-two-scale"))?;
    let wb6_outer = load_arm_record(&run_dir.join("wb6-outer"))?;
    let wb6_env = load_arm_record(&run_dir.join("wb6-env"))?;
    let wb6_twoscale = load_arm_record(&run_dir.join("wb6-two-scale"))?;
    let wbs6_env = load_arm_record(&run_dir.join("wbs6-env"))?;
    let wbs6_twoscale = load_arm_record(&run_dir.join("wbs6-two-scale"))?;
    let wb8_outer = load_arm_record(&run_dir.join("wb8-outer"))?;
    let wb8_env = load_arm_record(&run_dir.join("wb8-env"))?;
    let wb8_twoscale = load_arm_record(&run_dir.join("wb8-two-scale"))?;
    let wbs8_env = load_arm_record(&run_dir.join("wbs8-env"))?;
    let wbs8_twoscale = load_arm_record(&run_dir.join("wbs8-two-scale"))?;
    let fv8_b8_pad_env = load_arm_record(&run_dir.join("fv8-b8-pad-env"))?;
    let fv8_b8_pad_twoscale = load_arm_record(&run_dir.join("fv8-b8-pad-two-scale"))?;

    let g_exact_a8 = a8.psucc_admitted == A8_PSUCC_ADMITTED
        && a8.full_admitted == A8_FULL_ADMITTED
        && a8.top1 == A8_TOP1;
    let g_exact_fv8_b4 = fv8_b4_env.psucc_admitted == FV8_B4_ENV_PSUCC
        && fv8_b4_env.full_admitted == FV8_B4_ENV_FULL
        && fv8_b4_env.top1 == FV8_B4_ENV_TOP1
        && fv8_b4_env.top8 == FV8_B4_ENV_TOP8
        && fv8_b4_twoscale.psucc_admitted == FV8_B4_TWOSCALE_PSUCC
        && fv8_b4_twoscale.full_admitted == FV8_B4_TWOSCALE_FULL
        && fv8_b4_twoscale.top1 == FV8_B4_TWOSCALE_TOP1
        && fv8_b4_twoscale.top8 == FV8_B4_TWOSCALE_TOP8;
    let g_d11_and_outer_controls = a8.d11_artifact_sha256.is_some()
        && wb6_outer.d11_artifact_sha256.is_some()
        && wb8_outer.d11_artifact_sha256.is_some()
        && check_g_occ(&wb6_outer)
        && check_g_occ(&wb8_outer)
        && qual_val
            .pointer("/controls/2/status")
            .and_then(Value::as_str)
            == Some("PASS");
    let suite_gate_ok = qual_passed && g_exact_a8 && g_exact_fv8_b4 && g_d11_and_outer_controls;

    let grade_wb6_env =
        grade_wider_treatment(&a8, &wb6_env, &wbs6_env, Some(&fv8_b4_env), suite_gate_ok)?;
    let grade_wb6_twoscale = grade_wider_treatment(
        &a8,
        &wb6_twoscale,
        &wbs6_twoscale,
        Some(&fv8_b4_twoscale),
        suite_gate_ok,
    )?;
    let grade_wb8_env = grade_wider_treatment(
        &a8,
        &wb8_env,
        &wbs8_env,
        Some(&fv8_b8_pad_env),
        suite_gate_ok,
    )?;
    let grade_wb8_twoscale = grade_wider_treatment(
        &a8,
        &wb8_twoscale,
        &wbs8_twoscale,
        Some(&fv8_b8_pad_twoscale),
        suite_gate_ok,
    )?;

    let wb8_env_vs_fv8_b4 = paired_psucc(&fv8_b4_env.b3, &wb8_env.b3)?;
    let wb8_twoscale_vs_fv8_b4 = paired_psucc(&fv8_b4_twoscale.b3, &wb8_twoscale.b3)?;
    let wb8_env_vs_wb6_env = paired_psucc(&wb6_env.b3, &wb8_env.b3)?;
    let wb8_twoscale_vs_wb6_twoscale = paired_psucc(&wb6_twoscale.b3, &wb8_twoscale.b3)?;
    let wb6_env_vs_wb6_outer = paired_psucc(&wb6_env.b3, &wb6_outer.b3)?;
    let wb6_twoscale_vs_wb6_outer = paired_psucc(&wb6_twoscale.b3, &wb6_outer.b3)?;
    let wb8_env_vs_wb8_outer = paired_psucc(&wb8_env.b3, &wb8_outer.b3)?;
    let wb8_twoscale_vs_wb8_outer = paired_psucc(&wb8_twoscale.b3, &wb8_outer.b3)?;
    let wb6_outer_vs_a8 = paired_psucc(&wb6_outer.b3, &a8.b3)?;
    let wb8_outer_vs_a8 = paired_psucc(&wb8_outer.b3, &a8.b3)?;

    let any_pass = [
        &grade_wb6_env,
        &grade_wb6_twoscale,
        &grade_wb8_env,
        &grade_wb8_twoscale,
    ]
    .iter()
    .any(|g| g.get("outcome").and_then(Value::as_str) == Some("PASS"));
    let overall_outcome = if !suite_gate_ok {
        "INSTRUMENT"
    } else if any_pass {
        "PASS"
    } else {
        "DIAGNOSTIC_ONLY"
    };

    let arm_summary_entry = |r: &WiderArmRecord| -> Value {
        json!({
            "label": r.label,
            "blocks": r.blocks,
            "width": r.width,
            "address_metric": r.address_metric,
            "mode": r.mode,
            "config_sha256": r.config_sha256,
            "model_snapshot_sha256": r.model_snapshot_sha256,
            "distinct_vectors": r.distinct_vectors,
            "occupied_cells": r.occupied_cells,
            "within_cell_tie_tokens": r.within_cell_tie_tokens,
            "within_cell_tie_dev_rows": r.within_cell_tie_dev_rows,
            "dev_alias_ceiling_rows": r.dev_alias_ceiling_rows,
            "psucc_admitted": r.psucc_admitted,
            "full_admitted": r.full_admitted,
            "complement_admitted": r.complement_admitted,
            "top1": r.top1,
            "top8": r.top8,
            "admitted_strict_win_rows": r.admitted_strict_win_rows,
            "eval_ms": r.eval_ms,
            "decode_tokens_per_s": r.decode_tokens_per_s,
            "d11_artifact_sha256": r.d11_artifact_sha256,
        })
    };

    let out = json!({
        "schema": WIDER_CARRIER_GRADE_SCHEMA,
        "driver_source_sha256": crate::training::driver_source_sha256(),
        "qualification_sha256": qual_sha256,
        "instrument_gates": {
            "g_qualification_passed": qual_passed,
            "g_exact_a8_outer": g_exact_a8,
            "g_exact_fv8_b4": g_exact_fv8_b4,
            "g_d11_and_outer_controls": g_d11_and_outer_controls,
            "all_passed": suite_gate_ok,
        },
        "arms": [
            arm_summary_entry(&a8),
            arm_summary_entry(&fv8_b4_env),
            arm_summary_entry(&fv8_b4_twoscale),
            arm_summary_entry(&wb6_outer),
            arm_summary_entry(&wb6_env),
            arm_summary_entry(&wb6_twoscale),
            arm_summary_entry(&wbs6_env),
            arm_summary_entry(&wbs6_twoscale),
            arm_summary_entry(&wb8_outer),
            arm_summary_entry(&wb8_env),
            arm_summary_entry(&wb8_twoscale),
            arm_summary_entry(&wbs8_env),
            arm_summary_entry(&wbs8_twoscale),
            arm_summary_entry(&fv8_b8_pad_env),
            arm_summary_entry(&fv8_b8_pad_twoscale),
        ],
        "treatments": {
            "WB6-Env": grade_wb6_env,
            "WB6-TwoScale": grade_wb6_twoscale,
            "WB8-Env": grade_wb8_env,
            "WB8-TwoScale": grade_wb8_twoscale,
        },
        "cross_width_paired_psucc": {
            "WB8-Env_vs_FV8-B4-Env": paired_json(wb8_env_vs_fv8_b4),
            "WB8-TwoScale_vs_FV8-B4-TwoScale": paired_json(wb8_twoscale_vs_fv8_b4),
            "WB8-Env_vs_WB6-Env": paired_json(wb8_env_vs_wb6_env),
            "WB8-TwoScale_vs_WB6-TwoScale": paired_json(wb8_twoscale_vs_wb6_twoscale),
            "WB6-Env_vs_WB6-Outer": paired_json(wb6_env_vs_wb6_outer),
            "WB6-TwoScale_vs_WB6-Outer": paired_json(wb6_twoscale_vs_wb6_outer),
            "WB8-Env_vs_WB8-Outer": paired_json(wb8_env_vs_wb8_outer),
            "WB8-TwoScale_vs_WB8-Outer": paired_json(wb8_twoscale_vs_wb8_outer),
            "WB6-Outer_vs_A8-Outer": paired_json(wb6_outer_vs_a8),
            "WB8-Outer_vs_A8-Outer": paired_json(wb8_outer_vs_a8),
        },
        "overall_outcome": overall_outcome,
    });
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| invalid(e.to_string()))?;
    }
    std::fs::write(
        out_path,
        serde_json::to_vec_pretty(&out).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    Ok(out)
}

#[cfg(feature = "cuda")]
pub mod cuda_runner {
    use super::*;
    use crate::address_cells::{
        admit_cells_by_distances, build, cell_envelope_l1, cell_outer_l1, full_vector_cell_l1,
        token_full_l1, AddressCells, AddressCellsMode, AddressMetric, BigramCounts, CELLS,
    };
    use crate::controller::attention_learning::evidence;
    use crate::corpus::{self, PreparedCorpus};
    use crate::device::{compiled_wider_carrier_image_sha256, Device};
    use crate::training::{DriverConfig, PreparedDriver};
    use crate::{Identity, ObjectivePolicy, Parameter, ReadInitialization, ReadSpec};
    use std::fs;
    use std::path::Path;
    use std::time::Instant;

    fn ident(token: u32, occurrence: u64) -> Identity {
        Identity {
            token,
            occurrence,
            record: 17,
            version: 9,
        }
    }

    fn synthetic_fixture() -> (BigramCounts, Vec<u16>, Vec<Vec<Identity>>, Vec<u32>) {
        let mut tokens = Vec::new();
        for t in 1..512u32 {
            let reps = 1 + (512 - t) / 64;
            for _ in 0..reps {
                tokens.push(t / 2);
                tokens.push(t);
            }
        }
        let eligible: Vec<usize> = (1..tokens.len()).step_by(2).collect();
        let counts = BigramCounts::count(&tokens, &eligible, 512).expect("fixture counts");
        let fallback: Vec<u16> = (0..512u32).map(|t| (t % 256) as u16).collect();
        let mut windows = Vec::new();
        let mut targets = Vec::new();
        let mut occ = 1u64;
        for row in 0..16u32 {
            let base = (row * 17 + 9) % 256;
            let s = (base + 1) % 512;
            let t = ((s * 2) + (row % 2)) % 512;
            let w = vec![
                ident((base + 3) % 512, occ),
                ident((base + 7) % 512, occ + 1),
                ident((base + 11) % 512, occ + 2),
                ident(s, occ + 3),
            ];
            occ += 4;
            windows.push(w);
            targets.push(t);
        }
        (counts, fallback, windows, targets)
    }

    fn build_placed_device(
        spec: Spec,
        placed: &AddressCells,
        metric: AddressMetric,
    ) -> Result<Device> {
        let mut dev = Device::new_with_position(
            spec,
            ReadSpec::Relation { kept: 8, sweeps: 1 },
            ObjectivePolicy::NearestAddressMarginV1,
            ReadInitialization::ActiveLeafV1,
            crate::normalization_mechanism::NormalizationMode::default(),
            None,
            1,
            0,
        )?;
        dev.set_address_metric(metric);
        let mut snap = dev.snapshot()?;
        snap.table = placed.table.clone();
        let dev = Device::from_snapshot(snap, 0)?;
        require(
            dev.address_metric() == metric,
            "address metric not preserved",
        )?;
        Ok(dev)
    }

    fn verify_host_gpu_parity_for_blocks(blocks: u32) -> Result<Value> {
        let (counts, fallback, windows, targets) = synthetic_fixture();
        let spec = Spec {
            vocabulary: 512,
            blocks,
            context: 12,
            kept_cells: 8,
        };
        let placed = build(spec, &counts, &fallback, AddressCellsMode::WiderBlocksV1)?;
        let width = blocks as usize * 8;
        let mut checked_rows = 0u64;
        let mut checked_token_pairs = 0u64;

        for metric in [
            AddressMetric::OuterL1V1,
            AddressMetric::CellEnvelopeL1V1,
            AddressMetric::FullVectorCellL1V1,
        ] {
            let dev = build_placed_device(spec, &placed, metric)?;
            let before = dev.snapshot()?;
            let kernels = dev.wider_carrier_kernels()?;
            let batch = dev.carry_windows(&windows)?;
            let inspected = dev.inspect_wider_carrier_rows(&batch, &targets, &kernels)?;
            let evaluated = dev.evaluate(&batch, &targets)?;
            require(
                inspected.len() == windows.len() && evaluated.rows.len() == windows.len(),
                "inspected or evaluated row count differs from batch",
            )?;

            let mut queries_flat = Vec::with_capacity(windows.len() * width);
            for (r, row_insp) in inspected.iter().enumerate() {
                queries_flat.extend_from_slice(&row_insp.query);
                let q = &row_insp.query;
                let mut host_dists = [0u64; CELLS];
                for c in 0..CELLS {
                    let host_env = cell_envelope_l1(q, &placed.table, &placed.cells, spec, c)?;
                    let host_two_scale =
                        full_vector_cell_l1(q, &placed.table, &placed.cells, spec, c)?;
                    let host_sum = host_two_scale - (host_env << 8);
                    require(
                        u64::from(row_insp.env[c]) == host_env,
                        "GPU env[c] differs from host cell_envelope_l1",
                    )?;
                    require(
                        row_insp.sum[c] == host_sum,
                        "GPU sum[c] differs from host cell distance sum",
                    )?;
                    let expected_dist = match metric {
                        AddressMetric::OuterL1V1 => cell_outer_l1(q, blocks as usize, c)?,
                        AddressMetric::CellEnvelopeL1V1 => host_env,
                        AddressMetric::FullVectorCellL1V1 => host_two_scale,
                    };
                    require(
                        row_insp.cell_dist[c] == expected_dist,
                        "GPU wider_cell_distance differs from host oracle",
                    )?;
                    host_dists[c] = expected_dist;
                }
                let host_adm: Vec<u32> = admit_cells_by_distances(&host_dists, 8)
                    .into_iter()
                    .map(|c| c as u32)
                    .collect();
                require(
                    row_insp.admitted_cells == host_adm,
                    "GPU admitted cells differ from host admit_cells_by_distances",
                )?;
                let target_cell = u32::from(placed.cells[targets[r] as usize]);
                let host_target_admitted = host_adm.contains(&target_cell);
                let eval_row = &evaluated.rows[r];
                let gpu_target_admitted = eval_row.admitted_cells > eval_row.address_rivals;
                require(
                    host_target_admitted == gpu_target_admitted,
                    "Device::evaluate target cell admission disagrees with wider-carrier inspection",
                )?;
                checked_rows += 1;
            }

            let tok_dists = dev.inspect_wider_carrier_token_distances(
                &queries_flat,
                windows.len() as u32,
                &kernels,
            )?;
            require(
                tok_dists.len() == windows.len(),
                "token distance row count mismatch",
            )?;
            for (r, td_row) in tok_dists.iter().enumerate() {
                let q = &inspected[r].query;
                for t in 0..512usize {
                    let full = token_full_l1(q, &placed.table[t * width..(t + 1) * width])?;
                    let outer = cell_outer_l1(q, blocks as usize, usize::from(placed.cells[t]))?;
                    require(
                        u64::from(td_row.full_l1[t]) == full
                            && u64::from(td_row.outer_l1[t]) == outer
                            && u64::from(td_row.interior_l1[t]) == full - outer,
                        "GPU token full/outer/interior L1 differs from host",
                    )?;
                    checked_token_pairs += 1;
                }
            }

            let after = dev.snapshot()?;
            require(
                before.table == after.table
                    && before.routes == after.routes
                    && before.gates == after.gates
                    && before.epoch == after.epoch,
                "wider-carrier parity check mutated device state or epoch",
            )?;
        }

        Ok(json!({
            "blocks": blocks,
            "width": width,
            "checked_rows": checked_rows,
            "checked_token_pairs": checked_token_pairs,
        }))
    }

    fn verify_outer_invariance_across_b4_b6_b8() -> Result<Value> {
        let (counts, fallback, windows, targets) = synthetic_fixture();
        let mut baseline_outer_q: Option<Vec<(Vec<i32>, Vec<i32>)>> = None;
        let mut baseline_admitted: Option<Vec<Vec<u32>>> = None;
        let mut baseline_eval_admitted: Option<u64> = None;

        for blocks in [4u32, 6, 8] {
            let spec = Spec {
                vocabulary: 512,
                blocks,
                context: 12,
                kept_cells: 8,
            };
            let placed = build(spec, &counts, &fallback, AddressCellsMode::WiderBlocksV1)?;
            let dev = build_placed_device(spec, &placed, AddressMetric::OuterL1V1)?;
            let kernels = dev.wider_carrier_kernels()?;
            let batch = dev.carry_windows(&windows)?;
            let inspected = dev.inspect_wider_carrier_rows(&batch, &targets, &kernels)?;
            let evaluated = dev.evaluate(&batch, &targets)?;
            let width = blocks as usize * 8;
            let outer_q: Vec<(Vec<i32>, Vec<i32>)> = inspected
                .iter()
                .map(|r| (r.query[0..8].to_vec(), r.query[width - 8..width].to_vec()))
                .collect();
            let adm: Vec<Vec<u32>> = inspected.iter().map(|r| r.admitted_cells.clone()).collect();
            if let Some(ref want_q) = baseline_outer_q {
                require(
                    &outer_q == want_q,
                    "outer blocks (q[0], q[B-1]) changed across B = 4, 6, 8",
                )?;
            } else {
                baseline_outer_q = Some(outer_q);
            }
            if let Some(ref want_adm) = baseline_admitted {
                require(
                    &adm == want_adm,
                    "OuterL1V1 admitted cells changed across B = 4, 6, 8",
                )?;
            } else {
                baseline_admitted = Some(adm);
            }
            if let Some(want_eval) = baseline_eval_admitted {
                require(
                    evaluated.aggregate.target_admitted_rows == want_eval,
                    "OuterL1V1 target_admitted_rows changed across B = 4, 6, 8",
                )?;
            } else {
                baseline_eval_admitted = Some(evaluated.aggregate.target_admitted_rows);
            }
        }
        Ok(json!({
            "blocks_tested": [4, 6, 8],
            "outer_query_blocks_identical": true,
            "outer_admitted_cells_identical": true,
            "target_admitted_rows": baseline_eval_admitted.unwrap_or(0),
        }))
    }

    fn verify_d11_and_state_restoration() -> Result<Value> {
        let (counts, fallback, windows, targets) = synthetic_fixture();
        let mut d11_records = Vec::new();
        for blocks in [6u32, 8] {
            let spec = Spec {
                vocabulary: 512,
                blocks,
                context: 12,
                kept_cells: 8,
            };
            let placed = build(spec, &counts, &fallback, AddressCellsMode::WiderBlocksV1)?;
            let dev_outer = build_placed_device(spec, &placed, AddressMetric::OuterL1V1)?;
            let before = dev_outer.snapshot()?;
            let exported_cells = dev_outer.export_token_cells()?;
            require(
                exported_cells == placed.cells,
                "Device::export_token_cells differs from placed.cells at B = 6 or 8",
            )?;
            let d11_bytes = crate::d11_export::export_d11(&before)?;
            let restored = crate::d11_export::import_d11(&d11_bytes)?;
            require(
                restored.spec.blocks == blocks
                    && restored.spec.vocabulary == 512
                    && restored.table == before.table
                    && restored.routes == before.routes
                    && restored.gates == before.gates,
                "D11 export round-trip mismatch on GPU snapshot",
            )?;
            let d11_sha256 = corpus::sha256(&d11_bytes);
            d11_records.push(json!({
                "blocks": blocks,
                "artifact_bytes": d11_bytes.len(),
                "artifact_sha256": d11_sha256,
            }));

            let kernels = dev_outer.wider_carrier_kernels()?;
            // Non-outer address metrics must be refused by D11 export, and foreign
            // carried batches must be refused by inspect_wider_carrier_rows.
            for metric in [
                AddressMetric::CellEnvelopeL1V1,
                AddressMetric::FullVectorCellL1V1,
            ] {
                let dev_fv = build_placed_device(spec, &placed, metric)?;
                let fv_snap = dev_fv.snapshot()?;
                require(
                    crate::d11_export::export_d11(&fv_snap).is_err(),
                    "d11_export must refuse non-outer address metric",
                )?;
                let foreign_batch = dev_fv.carry_windows(&windows)?;
                require(
                    dev_outer
                        .inspect_wider_carrier_rows(&foreign_batch, &targets, &kernels)
                        .is_err(),
                    "inspect_wider_carrier_rows must refuse foreign CarriedBatch",
                )?;
            }

            let outer_batch = dev_outer.carry_windows(&windows)?;
            let _proposal = dev_outer.propose(&outer_batch, &targets, Parameter::Gate(0))?;
            require(
                dev_outer.epoch()? == 0,
                "Device::propose mutated epoch at B = 6 or 8",
            )?;

            let after = dev_outer.snapshot()?;
            require(
                before.table == after.table
                    && before.routes == after.routes
                    && before.gates == after.gates
                    && before.epoch == after.epoch,
                "export_token_cells, propose, or D11 export mutated device state",
            )?;
        }
        Ok(json!({
            "d11_exports": d11_records,
            "non_outer_refusals_verified": true,
            "state_unchanged": true,
        }))
    }

    fn timing_probe() -> Result<Value> {
        let mut cases = Vec::new();
        for blocks in [4u32, 6, 8] {
            for metric in [
                AddressMetric::OuterL1V1,
                AddressMetric::CellEnvelopeL1V1,
                AddressMetric::FullVectorCellL1V1,
            ] {
                let spec = Spec {
                    vocabulary: 4096,
                    blocks,
                    context: 192,
                    kept_cells: 8,
                };
                let mut dev = Device::new_with_position(
                    spec,
                    ReadSpec::Relation { kept: 8, sweeps: 1 },
                    ObjectivePolicy::NearestAddressMarginV1,
                    ReadInitialization::ActiveLeafV1,
                    crate::normalization_mechanism::NormalizationMode::default(),
                    None,
                    1,
                    0,
                )?;
                dev.set_address_metric(metric);
                let batch_windows: Vec<Vec<Identity>> = (0..64u64)
                    .map(|r| {
                        (0..192u64)
                            .map(|p| Identity {
                                token: ((r * 192 + p) % 4096) as u32,
                                occurrence: r * 192 + p + 1,
                                record: 17,
                                version: 9,
                            })
                            .collect()
                    })
                    .collect();
                let targets: Vec<u32> = (0..64u32).map(|r| (r * 37 + 11) % 4096).collect();
                let carried = dev.carry_windows(&batch_windows)?;
                // Warmup
                let _ = dev.evaluate(&carried, &targets)?;
                dev.synchronize()?;
                let mut samples_us = Vec::new();
                for _ in 0..5 {
                    let started = Instant::now();
                    let _ = dev.evaluate(&carried, &targets)?;
                    dev.synchronize()?;
                    samples_us.push(started.elapsed().as_micros() as u64);
                }
                samples_us.sort_unstable();
                let median_us = samples_us[2].max(1);
                let rows_per_s = (64 * 1_000_000) / median_us;
                cases.push(json!({
                    "blocks": blocks,
                    "width": blocks * 8,
                    "address_metric": format!("{metric:?}"),
                    "batch_rows": 64,
                    "median_eval_us": median_us,
                    "decode_rows_per_s": rows_per_s,
                }));
            }
        }
        Ok(json!({
            "shape": "V4096, B in {4,6,8}, C192, K8, Relation(8,1) ActiveLeafV1, 64 rows, 1 warmup + 5 timed, median",
            "cases": cases,
        }))
    }

    /// Run all four finite CUDA qualification controls and write `wider-carrier-qualification.json`.
    pub fn qualify(report_dir: &Path) -> Result<Value> {
        fs::create_dir_all(report_dir).map_err(|e| invalid(format!("report dir: {e}")))?;
        let mut controls = Vec::new();
        let mut failed = false;

        let runners: [(&str, fn() -> Result<Value>); 4] = [
            (QUALIFICATION_NAMES[0], || {
                verify_host_gpu_parity_for_blocks(6)
            }),
            (QUALIFICATION_NAMES[1], || {
                verify_host_gpu_parity_for_blocks(8)
            }),
            (
                QUALIFICATION_NAMES[2],
                verify_outer_invariance_across_b4_b6_b8,
            ),
            (QUALIFICATION_NAMES[3], verify_d11_and_state_restoration),
        ];

        for (index, (name, run_fn)) in runners.into_iter().enumerate() {
            let rec = match run_fn() {
                Ok(detail) => json!({
                    "index": index + 1,
                    "name": name,
                    "status": "PASS",
                    "detail": detail,
                }),
                Err(err) => {
                    failed = true;
                    json!({
                        "index": index + 1,
                        "name": name,
                        "status": "FAIL",
                        "error": err.to_string(),
                    })
                }
            };
            println!("{rec}");
            controls.push(rec);
        }

        let timing = if failed {
            json!("NOT_RUN: parity failed")
        } else {
            match timing_probe() {
                Ok(t) => t,
                Err(err) => {
                    failed = true;
                    json!({"status": "FAIL", "error": err.to_string()})
                }
            }
        };

        let payload = json!({
            "schema": WIDER_CARRIER_QUALIFICATION_SCHEMA,
            "suite": "wider-code-carrier-qualification",
            "status": if failed { "FAILED" } else { "PASS" },
            "driver_source_sha256": crate::training::driver_source_sha256(),
            "wider_carrier_image_sha256": compiled_wider_carrier_image_sha256(),
            "controls": controls,
            "timing": timing,
            "training": "HELD",
            "applied_updates": 0,
        });
        let out_file = report_dir.join("wider-carrier-qualification.json");
        fs::write(
            &out_file,
            serde_json::to_vec_pretty(&payload).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        require(
            !failed,
            "one or more wider-carrier qualification controls failed",
        )?;
        Ok(payload)
    }

    fn dev_expected_rows(config: &DriverConfig) -> Result<(Vec<(u64, u32, bool)>, Vec<u32>)> {
        let source = PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
        let mut expected = Vec::with_capacity(source.eligible_len());
        let mut target_tokens = Vec::with_capacity(source.eligible_len());
        let mut first = 0u64;
        while first < source.eligible_len() as u64 {
            let batch = source.batch(first, config.spec.context, config.batch_rows)?;
            for ((window, &position), &target) in batch
                .windows
                .iter()
                .zip(&batch.target_positions)
                .zip(&batch.targets)
            {
                expected.push((
                    position,
                    target,
                    evidence::is_copy_successor(window, target),
                ));
                target_tokens.push(target);
            }
            first = batch.next_target;
        }
        let psucc_count = expected.iter().filter(|&&(_, _, p)| p).count() as u64;
        require(
            expected.len() as u64 == DEV_ROWS && psucc_count == PSUCC_ROWS,
            "DEV split did not reproduce 14,287 total and 3,847 P_succ rows",
        )?;
        Ok((expected, target_tokens))
    }

    /// Evaluate one zero-training wider-carrier or baseline `DriverConfig` arm on CUDA,
    /// write `census.json`, `eval.json`, `dev_b3.json`, and `arm_summary.json` into `out_dir`,
    /// and return the `WiderArmRecord`.
    pub fn evaluate_arm(label: &str, config_path: &Path, out_dir: &Path) -> Result<WiderArmRecord> {
        fs::create_dir_all(out_dir)
            .map_err(|e| invalid(format!("create {}: {e}", out_dir.display())))?;
        let config_bytes = fs::read(config_path)
            .map_err(|e| invalid(format!("read {}: {e}", config_path.display())))?;
        let config_sha256 = corpus::sha256(&config_bytes);
        let config: DriverConfig =
            serde_json::from_slice(&config_bytes).map_err(|e| invalid(e.to_string()))?;

        let driver = PreparedDriver::load(config.clone())?;
        let model = driver.new_model(0)?;
        let checkpoint = driver.checkpoint(&model, driver.initial_cursor())?;
        let snap = model.snapshot()?;

        // Export token cells on CUDA and run coverage census.
        let cells_u16 = model.export_token_cells()?;
        let (dev_expected, dev_targets) = dev_expected_rows(&config)?;
        let census_val = crate::coverage_census::census(
            &snap.table,
            config.spec,
            &dev_targets,
            Some(&cells_u16),
            Some(RUN_BAR),
        )?;
        fs::write(
            out_dir.join("census.json"),
            serde_json::to_vec_pretty(&census_val).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;

        let d11_artifact_sha256 = if config.address_metric.is_outer() {
            let artifact = crate::d11_export::export_d11(&snap)?;
            let restored = crate::d11_export::import_d11(&artifact)?;
            require(
                restored.spec == snap.spec && restored.table == snap.table,
                "D11 export round-trip mismatch on wider-carrier OuterL1V1 arm",
            )?;
            let art_sha = corpus::sha256(&artifact);
            let info = json!({
                "format": crate::d11_export::DISCRETE_LUT_SCHEMA,
                "vocabulary": snap.spec.vocabulary,
                "blocks": snap.spec.blocks,
                "context": snap.spec.context,
                "kept_cells": snap.spec.kept_cells,
                "artifact_bytes": artifact.len(),
                "artifact_sha256": art_sha,
            });
            fs::write(
                out_dir.join("d11_inspection.json"),
                serde_json::to_vec_pretty(&info).map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
            Some(art_sha)
        } else {
            None
        };

        let started = Instant::now();
        let eval_val = driver.evaluate_checkpoint(&model, &checkpoint)?;
        model.synchronize()?;
        let eval_ms = (started.elapsed().as_millis() as u64).max(1);
        let decode_tokens_per_s = (DEV_ROWS * 1_000) / eval_ms;

        fs::write(
            out_dir.join("eval.json"),
            serde_json::to_vec(&eval_val).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;

        let b3 = evidence::summarize(&eval_val, &dev_expected)?;
        fs::write(
            out_dir.join("dev_b3.json"),
            serde_json::to_vec(&b3).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;

        let agg = eval_val
            .get("aggregate")
            .ok_or_else(|| invalid("evaluation missing aggregate"))?;
        let word = |ptr: &str, v: &Value| -> Result<u64> {
            v.pointer(ptr)
                .and_then(Value::as_u64)
                .ok_or_else(|| invalid(format!("missing u64 at {ptr}")))
        };
        let scored_rows = word("/scored_rows", agg)?;
        let full_admitted = word("/target_admitted_rows", agg)?;
        let top1 = word("/top1_correct", agg)?;
        let top8 = word("/top8_correct", agg)?;
        let admitted_strict_win_rows = word("/admitted_strict_win_rows", agg)?;

        let mode_label = match &config.codebook_initialization {
            crate::training::CodebookInitialization::SeededDeviceV1 => "seeded_device_v1".into(),
            crate::training::CodebookInitialization::PinnedTableV1 { table } => {
                let tf: crate::training::CodebookTable =
                    serde_json::from_slice(&table.read()?).map_err(|e| invalid(e.to_string()))?;
                tf.mode
            }
        };

        let record = WiderArmRecord {
            schema: WIDER_CARRIER_ARM_SCHEMA.into(),
            label: label.to_owned(),
            config_sha256,
            blocks: config.spec.blocks,
            width: config.spec.blocks * 8,
            address_metric: format!("{:?}", config.address_metric),
            mode: mode_label,
            model_snapshot_sha256: checkpoint.model_sha256,
            distinct_vectors: word("/distinct_vectors", &census_val)?,
            occupied_cells: word("/cells/occupied_cells", &census_val)?,
            within_cell_tie_tokens: word(
                "/within_cell_tie_mass/tokens_in_alias_groups",
                &census_val,
            )?,
            within_cell_tie_dev_rows: word(
                "/within_cell_tie_mass/dev_rows_targeting_non_representative",
                &census_val,
            )?,
            dev_alias_ceiling_rows: word("/dev_alias_ceiling_rows", &census_val)?,
            scored_rows,
            full_admitted,
            psucc_rows: b3.psucc.rows,
            psucc_admitted: b3.psucc.target_admitted,
            complement_admitted: b3.complement.target_admitted,
            top1,
            top8,
            admitted_strict_win_rows,
            eval_ms,
            decode_tokens_per_s,
            d11_artifact_sha256,
            b3,
        };
        check_arm_panel(&record)?;
        let mut summary_json = serde_json::to_value(&record).map_err(|e| invalid(e.to_string()))?;
        if let Some(obj) = summary_json.as_object_mut() {
            obj.remove("b3");
        }
        fs::write(
            out_dir.join("arm_summary.json"),
            serde_json::to_vec_pretty(&summary_json).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        println!("{summary_json}");
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address_cells::{build, AddressCellsMode, BigramCounts};
    use crate::controller::attention_learning::evidence::{AdmissionRow, B3Count};

    fn synthetic_arm(
        label: &str,
        blocks: u32,
        psucc: u64,
        rest: u64,
        top1: u64,
        top8: u64,
    ) -> WiderArmRecord {
        let rows: Vec<AdmissionRow> = (0..DEV_ROWS)
            .map(|i| {
                let is_psucc = i < PSUCC_ROWS;
                let target_admitted = if is_psucc {
                    i < psucc
                } else {
                    i - PSUCC_ROWS < rest
                };
                AdmissionRow {
                    row_ordinal: i,
                    target_position: i + 1,
                    target_token: (i % 4_096) as u32,
                    is_psucc,
                    target_admitted,
                }
            })
            .collect();
        let count = |f: &dyn Fn(&AdmissionRow) -> bool| B3Count {
            rows: rows.iter().filter(|r| f(r)).count() as u64,
            target_admitted: rows.iter().filter(|r| f(r) && r.target_admitted).count() as u64,
        };
        let b3 = B3Summary {
            full: count(&|_| true),
            psucc: count(&|r| r.is_psucc),
            complement: count(&|r| !r.is_psucc),
            rows,
        };
        WiderArmRecord {
            schema: WIDER_CARRIER_ARM_SCHEMA.into(),
            label: label.into(),
            config_sha256: "a".repeat(64),
            blocks,
            width: blocks * 8,
            address_metric: "CellEnvelopeL1V1".into(),
            mode: "address_wider_blocks_v1".into(),
            model_snapshot_sha256: "b".repeat(64),
            distinct_vectors: 4_096,
            occupied_cells: 256,
            within_cell_tie_tokens: 0,
            within_cell_tie_dev_rows: 0,
            dev_alias_ceiling_rows: DEV_ROWS,
            scored_rows: DEV_ROWS,
            full_admitted: psucc + rest,
            psucc_rows: PSUCC_ROWS,
            psucc_admitted: psucc,
            complement_admitted: rest,
            top1,
            top8,
            admitted_strict_win_rows: top1,
            eval_ms: 4_000,
            decode_tokens_per_s: 3_571,
            d11_artifact_sha256: None,
            b3,
        }
    }

    #[test]
    fn wider_carrier_d11_export_round_trips_at_b6_and_b8() {
        let mut tokens = Vec::new();
        for t in 1..512u32 {
            tokens.push(t / 2);
            tokens.push(t);
        }
        let eligible: Vec<usize> = (1..tokens.len()).step_by(2).collect();
        let counts = BigramCounts::count(&tokens, &eligible, 512).expect("counts");
        let fallback: Vec<u16> = (0..512u32).map(|t| (t % 256) as u16).collect();
        for blocks in [6u32, 8] {
            let spec = Spec {
                vocabulary: 512,
                blocks,
                context: 12,
                kept_cells: 8,
            };
            let placed =
                build(spec, &counts, &fallback, AddressCellsMode::WiderBlocksV1).expect("build");
            let info = verify_d11_export_for_table(spec, &placed.table).expect("d11 export");
            assert_eq!(info["blocks"], blocks);
            assert_eq!(info["table_ints"], 512 * blocks * 8);
            assert_eq!(info["routes_len"], blocks * 3);
        }
    }

    #[test]
    fn wider_carrier_grader_checks_bars_and_instrument_gates() {
        let a8 = synthetic_arm(
            "A8-Outer",
            4,
            A8_PSUCC_ADMITTED,
            A8_FULL_ADMITTED - A8_PSUCC_ADMITTED,
            A8_TOP1,
            513,
        );
        let wb8 = synthetic_arm("WB8-Env", 8, 2_120, 4_100, 145, 1_240);
        let wbs8 = synthetic_arm("WBS8-Env", 8, 690, 1_450, 38, 880);
        let pad8 = synthetic_arm("FV8-B8-Pad-Env", 8, 1_750, 3_600, 110, 1_050);

        let graded =
            grade_wider_treatment(&a8, &wb8, &wbs8, Some(&pad8), true).expect("grade pass");
        assert_eq!(graded["outcome"], "PASS");
        assert_eq!(graded["reach"], true);
        assert_eq!(graded["capacity"], true);
        assert_eq!(graded["vs_a8"]["clears"], true);
        assert_eq!(graded["vs_shuffled"]["clears"], true);
        assert_eq!(graded["vs_other_control"]["paired"]["clears"], true);

        let instr =
            grade_wider_treatment(&a8, &wb8, &wbs8, Some(&pad8), false).expect("grade instr");
        assert_eq!(instr["outcome"], "INSTRUMENT");

        let mut bad_occ = wb8.clone();
        bad_occ.within_cell_tie_tokens = 2;
        let occ_fail =
            grade_wider_treatment(&a8, &bad_occ, &wbs8, None, true).expect("grade occ fail");
        assert_eq!(occ_fail["outcome"], "INSTRUMENT");
    }

    #[test]
    fn committed_wider_carrier_configs_validate_and_match_pins() {
        use crate::address_cells::AddressMetric;
        use crate::training::DriverConfig;
        use std::path::Path;

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let a8_bytes = std::fs::read(
            root.join("docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json"),
        )
        .expect("read a8");
        let a8: DriverConfig = serde_json::from_slice(&a8_bytes).expect("parse a8");

        let expected: [(&str, u32, AddressMetric, &str, &str); 12] = [
            (
                "wb6-outer.json",
                6,
                AddressMetric::OuterL1V1,
                "fd2c79277ef20c7fa1cac136c5d8f6fcb2586f5563aecfd889afb5c4b6748374",
                "4e1545948a9cb4d3560590f7d5e3db64acf0efbe063eef48dd32d0c978c21c3a",
            ),
            (
                "wb6-env.json",
                6,
                AddressMetric::CellEnvelopeL1V1,
                "fd2c79277ef20c7fa1cac136c5d8f6fcb2586f5563aecfd889afb5c4b6748374",
                "e86d0f157310d928bb9c515291566a563d0acffe7f1e1c6089c63aacf98160b2",
            ),
            (
                "wb6-two-scale.json",
                6,
                AddressMetric::FullVectorCellL1V1,
                "fd2c79277ef20c7fa1cac136c5d8f6fcb2586f5563aecfd889afb5c4b6748374",
                "49a2822bb65e5d895fe8e9820bf3a450a27b552c80e7f9365d5424a6e33593f7",
            ),
            (
                "wbs6-env.json",
                6,
                AddressMetric::CellEnvelopeL1V1,
                "67031b7f9da75e6672aced861b299b9f0301ba7894d2571d6b81525bec873df5",
                "ec86db0fe1c537cdda2401df525cce84c81a6b68d4192f7a3c63851818e1582f",
            ),
            (
                "wbs6-two-scale.json",
                6,
                AddressMetric::FullVectorCellL1V1,
                "67031b7f9da75e6672aced861b299b9f0301ba7894d2571d6b81525bec873df5",
                "c3499a619b21a9264792d8f2aed9177de302b7bf5d8d5e02a48e0d5263ab0d2f",
            ),
            (
                "wb8-outer.json",
                8,
                AddressMetric::OuterL1V1,
                "d5c3cb97f02210942778fe26fd874aaabf967d605438715925f501f2214a3222",
                "6a72404fef373dc2f3c59d3141819ccbdd168fa5004bbef2b4803221a3f19af0",
            ),
            (
                "wb8-env.json",
                8,
                AddressMetric::CellEnvelopeL1V1,
                "d5c3cb97f02210942778fe26fd874aaabf967d605438715925f501f2214a3222",
                "840431c40a0ea25567727a2ff1f6c026f70b9bcf906291ace9982941cb8628ba",
            ),
            (
                "wb8-two-scale.json",
                8,
                AddressMetric::FullVectorCellL1V1,
                "d5c3cb97f02210942778fe26fd874aaabf967d605438715925f501f2214a3222",
                "c31490a11dd9525ba702534511ef92621790f8e138913df6a343267896fbc16e",
            ),
            (
                "wbs8-env.json",
                8,
                AddressMetric::CellEnvelopeL1V1,
                "1490c1b8c8878221d65f39bd975259ccaadc2a338b94bdc91c2061f0fc535dd2",
                "cdcd9d102172d6f47e13ad7293f51f447cc00c756b7b0ab78bde4ee09bc38edc",
            ),
            (
                "wbs8-two-scale.json",
                8,
                AddressMetric::FullVectorCellL1V1,
                "1490c1b8c8878221d65f39bd975259ccaadc2a338b94bdc91c2061f0fc535dd2",
                "9992e290a7940bc377057faade753bd7d8448e603c7a03ffc9e076e2f44e7733",
            ),
            (
                "fv8-b8-pad-env.json",
                8,
                AddressMetric::CellEnvelopeL1V1,
                "2b933bc5d4f0da4eb602e5490876f80cf8ce21bdbe47306a8f27da7e90df5620",
                "b67157b8dbf39ff3a240eb3ab0cb1f9c2a3654e720d51b09b2f6eb813a337a7e",
            ),
            (
                "fv8-b8-pad-two-scale.json",
                8,
                AddressMetric::FullVectorCellL1V1,
                "2b933bc5d4f0da4eb602e5490876f80cf8ce21bdbe47306a8f27da7e90df5620",
                "d025c109a30dfce732c6e846c9f8947a53b17eca5459a9d3e4bf138b09d321d2",
            ),
        ];

        for (file, blocks, metric, table_sha, config_sha) in expected {
            let path = root
                .join("docs/labs/m1-wider-code-carrier/configs")
                .join(file);
            let bytes = std::fs::read(&path).expect("read config");
            assert_eq!(crate::corpus::sha256(&bytes), config_sha, "{file} sha256");
            let cfg: DriverConfig = serde_json::from_slice(&bytes).expect("parse config");
            cfg.validate().expect("validate config");
            assert_eq!(cfg.spec.blocks, blocks);
            assert_eq!(cfg.address_metric, metric);
            assert_eq!(cfg.spec.vocabulary, a8.spec.vocabulary);
            assert_eq!(cfg.spec.context, a8.spec.context);
            assert_eq!(cfg.spec.kept_cells, a8.spec.kept_cells);
            assert_eq!(cfg.read_spec, a8.read_spec);
            assert_eq!(cfg.read_initialization, a8.read_initialization);
            assert_eq!(cfg.train, a8.train);
            assert_eq!(cfg.dev, a8.dev);
            assert_eq!(cfg.tokenizer, a8.tokenizer);
            match &cfg.codebook_initialization {
                crate::training::CodebookInitialization::PinnedTableV1 { table } => {
                    assert_eq!(table.sha256, table_sha);
                }
                _ => panic!("expected pinned_table_v1"),
            }
        }
    }
}
