//! Check architectural commitments, not experiment order or duplicated prose.
use serde_json::Value;
use std::path::Path;

fn check(root: &Path) -> Result<(), String> {
    let path = root.join("docs/integration/agent-execution-policy.json");
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let policy: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    for (pointer, expected) in [
        ("/schema", Value::from("uor-r4.agent-execution-policy/4")),
        ("/mode", Value::from("native_geometric_ai")),
        ("/architecture/implementation_language", Value::from("rust")),
        (
            "/architecture/training_matrix_multiplication_allowed",
            Value::from(true),
        ),
        (
            "/architecture/python_model_dependency_allowed",
            Value::from(false),
        ),
        (
            "/architecture/dense_transformer_disguised_as_lookup_allowed",
            Value::from(false),
        ),
        ("/delivery/protected_pull_request", Value::from(true)),
        ("/delivery/direct_push_to_main", Value::from(false)),
        (
            "/evidence/preserve_unique_evidence_and_user_material",
            Value::from(true),
        ),
        (
            "/evidence/unavailable_is_model_evidence",
            Value::from(false),
        ),
        (
            "/execution/machine_budget/global_fixed_time_or_retry_quota",
            Value::from(false),
        ),
    ] {
        if policy.pointer(pointer) != Some(&expected) {
            return Err(format!(
                "architecture commitment differs at {pointer}: expected {expected}"
            ));
        }
    }
    for mechanism in [
        "prime_registry_and_ordered_nlets",
        "fixed_zeta_zero_phase_channels",
        "r4_s3_h4_causal_state_and_transport",
        "z_phi_radial_and_orientation_state",
        "typed_paired_h4_icosian_bridge",
        "learned_geometric_operators",
        "uor_canonical_identity",
    ] {
        if !policy["architecture"]["primary_mechanisms"]
            .as_array()
            .is_some_and(|items| items.contains(&Value::from(mechanism)))
        {
            return Err(format!("missing primary mechanism: {mechanism}"));
        }
    }
    for capability in ["conversation_and_memory", "coding_and_reasoning"] {
        if !policy["project_track"]["alpha_capabilities"]
            .as_array()
            .is_some_and(|items| items.contains(&Value::from(capability)))
        {
            return Err(format!("missing alpha capability: {capability}"));
        }
    }
    for field in ["canonical_plan", "current_state"] {
        let relative = policy["project_track"][field]
            .as_str()
            .ok_or_else(|| format!("missing project_track.{field}"))?;
        if Path::new(relative).is_absolute()
            || Path::new(relative)
                .components()
                .any(|p| p == std::path::Component::ParentDir)
            || !root.join(relative).is_file()
        {
            return Err(format!(
                "project_track.{field} must name an existing repository file"
            ));
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    let root = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match check(&root) {
        Ok(()) => {
            println!("Native geometric architecture policy: PASS (no model behavior evaluated)");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
