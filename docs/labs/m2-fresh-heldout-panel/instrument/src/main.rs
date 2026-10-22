//! CLI entry point for the M2 fresh held-out panel (`m2-fresh256`) and `dev512` noise audit.

use m2_fresh_heldout_panel_instrument::{
    analyze_saved437_dev512_spread, generate_m2_fresh256_panel, verify_m2_fresh256_panel,
};
use std::path::Path;

fn main() -> Result<(), String> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .ok_or_else(|| "cannot resolve repo root".to_string())?;

    let dev_spread = analyze_saved437_dev512_spread(repo_root)?;
    let bundle = generate_m2_fresh256_panel()?;
    let audit = verify_m2_fresh256_panel(repo_root, &bundle)?;

    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--emit-dir") {
        let dir = args
            .get(pos + 1)
            .ok_or_else(|| "--emit-dir requires a path".to_string())?;
        let out_dir = Path::new(dir);
        std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("plan.json"), &bundle.plan_json).map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("inputs.json"), &bundle.inputs_json)
            .map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("labels.json"), &bundle.labels_json)
            .map_err(|e| e.to_string())?;
        std::fs::write(out_dir.join("manifest.json"), &bundle.manifest_json)
            .map_err(|e| e.to_string())?;
    }

    let summary = serde_json::json!({
        "dev512_spread": dev_spread,
        "fresh256_audit": audit,
        "fresh256_seals": {
            "plan_bytes": bundle.plan_json.len(),
            "plan_sha256": bundle.plan_sha256,
            "inputs_bytes": bundle.inputs_json.len(),
            "inputs_sha256": bundle.inputs_sha256,
            "labels_bytes": bundle.labels_json.len(),
            "labels_sha256": bundle.labels_sha256,
            "manifest_bytes": bundle.manifest_json.len(),
            "manifest_sha256": bundle.manifest_sha256,
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?
    );
    Ok(())
}
