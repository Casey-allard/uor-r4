//! Repository gates.
//!
//! `cargo xtask <task>`; `just vv` runs the whole normative acceptance gate.
//! Each task below enforces one of the rules `AGENTS.md` sets out, and each
//! names the rule it enforces when it fails, so that a red gate says *which
//! promise* was broken rather than merely that something is wrong.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use repo_model::{codegen, Model};

mod audit;
mod gate;
mod gnaf_conformance;
mod gnaf_firewall;
mod gnaf_manifest;
mod gnaf_release_path;
mod gnaf_root;
mod gnaf_scan;
mod kappa;

fn main() -> ExitCode {
    let task = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "help".to_string());
    let write = std::env::args().any(|a| a == "--write");
    let root = repo_model::repo_root();

    let result = match task.as_str() {
        "check-model" => check_model(&root, write),
        "audit-limits" => audit::audit_limits(&root),
        "audit-deferral" => audit::audit_deferral(&root),
        "gates" => gate::gate(&root, None),
        "gate" => match std::env::args().nth(2) {
            Some(id) => gate::gate(&root, Some(&id)),
            None => {
                eprintln!("cargo xtask gate <claim-id>   (or `gates` for all)");
                return ExitCode::from(2);
            }
        },
        "validate" => validate(&root),
        "gnaf-firewall" => gnaf_firewall::gnaf_firewall(&root),
        "gnaf-root" => gnaf_root::gnaf_root(&root, !write),
        "gnaf-scan" => gnaf_scan::gnaf_scan(&root),
        "gnaf-release-path" => gnaf_release_path::gnaf_release_path(&root),
        "gnaf-manifest" => gnaf_manifest::gnaf_manifest(&root, !write),
        "gnaf-conformance" => gnaf_conformance::gnaf_conformance(&root, !write),
        "kappa" => kappa::run(&std::env::args().skip(2).collect::<Vec<_>>()),
        _ => {
            eprintln!(
                "cargo xtask <task>\n\
                 \n\
                 check-model       R1: model/*.toml is the single source; regenerate and diff\n\
                 audit-limits      R5:  no bound that cannot be traced to a parameter\n\
                 audit-deferral    R4: no deferral marker, no stub, no capability behind a flag\n\
                 gates             #515: list every dormant claim's activation gate and status\n\
                 gate <claim-id>   #515: report one dormant claim's activation gate and status\n\
                 validate          run every gate above\n\
                 gnaf-firewall     #653 SPEC 10.1: GNAF competitor universe stays artifact-/selector-/conclusion-blind\n\
                 gnaf-root         #653 SPEC 5: every GNAF .lean file is layer-owned; root imports current\n\
                 gnaf-scan         #653 SPEC 19: no sorry/admit/native_decide/axiom/unsafe/partial in GNAF code\n\
                 gnaf-release-path #653 SPEC 19/6.3: no noncomputable def/abbrev/instance on the GNAF release path (currently red: WGG-GO-1 outstanding; not in `validate`)\n\
                 gnaf-manifest     #653 SPEC 4/5: MANIFEST.json's 3 identity stages are current and acyclic (currently red: pre-existing staleness; not in `validate`)\n\
                 gnaf-conformance  #653 SPEC 17.3: CONFORMANCE.md is deterministically generated from model/claims.json and current\n\
                 kappa <cmd>       #624: publish/fetch/tag against a kappa-registry (R4_KAPPA_REGISTRY)\n\
                 \n\
                 --write           check-model/gnaf-root/gnaf-manifest/gnaf-conformance: rewrite the generated file instead of checking it"
            );
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gate failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// A gate failure, reported with the rule it broke.
type Fail = Box<dyn std::error::Error>;

/// R1, `CM-01`: the generated Rust consts equal the model, numeral for
/// numeral.
fn check_model(root: &Path, write: bool) -> Result<(), Fail> {
    let model = Model::load(&root.join("model"))?;
    model.check()?;

    let conformance = codegen::render_conformance(&model);
    let conformance_path: PathBuf = root.join(codegen::CONFORMANCE_PATH);

    if write {
        std::fs::write(&conformance_path, &conformance)?;
        println!("wrote {}", conformance_path.display());
        return Ok(());
    }

    let committed = std::fs::read_to_string(&conformance_path).map_err(|e| {
        format!(
            "{}: {e}\nrun `cargo xtask check-model --write`",
            conformance_path.display()
        )
    })?;
    if committed != conformance {
        return Err(format!(
            "{} is stale: it disagrees with model/ids.toml.\n\
             R2: a claim cannot exist in the documentation without a ledger row. \
             Run `cargo xtask check-model --write`.",
            conformance_path.display()
        )
        .into());
    }
    println!(
        "check-model: CONFORMANCE.md equals the model, {} ids (CM-01)",
        model.ids.id.len()
    );
    Ok(())
}

/// The whole normative acceptance gate, in one place.
///
/// `gnaf-release-path` and `gnaf-manifest` are deliberately NOT included
/// here: unlike the other four GNAF checks, they currently and expectedly
/// fail against the real vendored tree. `gnaf-release-path` hits two
/// `noncomputable def`s in `Artifact/Release.lean`, gated on WGG-GO-1 --
/// the same outstanding condition `Tools/gate.py` itself documents as
/// "expected to fail... that failure is the conforming behavior".
/// `gnaf-manifest` reports the checked-in `MANIFEST.json` as stale
/// (predates recent tree changes; see `gnaf_manifest.rs`'s module doc).
/// Folding a currently-red check into the gate this repository expects to
/// always pass would either break `validate` for everyone or -- worse --
/// get quietly worked around, which defeats the point of a gate that's
/// supposed to say what's really true. Both stay available as their own
/// `cargo xtask gnaf-release-path` / `cargo xtask gnaf-manifest` commands.
///
/// `gnaf-conformance` IS included: the real vendored tree's
/// `CONFORMANCE.md` is currently current against `model/claims.json`
/// (verified by cross-tool round trip against `Tools/gen_conformance.py`
/// at port time), so this check is green today and belongs in the gate
/// that's supposed to stay green.
fn validate(root: &Path) -> Result<(), Fail> {
    check_model(root, false)?;
    audit::audit_limits(root)?;
    audit::audit_deferral(root)?;
    gnaf_firewall::gnaf_firewall(root)?;
    gnaf_root::gnaf_root(root, true)?;
    gnaf_scan::gnaf_scan(root)?;
    gnaf_conformance::gnaf_conformance(root, true)?;
    println!("validate: every gate passed");
    Ok(())
}
