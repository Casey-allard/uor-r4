//! Repository gates.
//!
//! `cargo xtask <task>`; `just vv` runs the whole normative acceptance gate.
//! Each task below enforces one of the rules `AGENTS.md` sets out, and each
//! names the rule it enforces when it fails, so that a red gate says *which
//! promise* was broken rather than merely that something is wrong.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use repo_model::{codegen, Model};

mod archive_verify;
mod audit;
mod book_check;
mod book_stage;
mod docs_catalog;
mod docs_inventory;
mod docs_links;
mod docs_move;
mod docs_relink;
mod docs_rewrite_refs;
mod docs_stubs;
mod figures;
mod figures_charts;
mod figures_math;
mod gate;
mod gnaf_conformance;
mod gnaf_firewall;
mod gnaf_manifest;
mod gnaf_release_path;
mod gnaf_root;
mod gnaf_scan;
mod kappa;
mod qubo_codec;
mod roadmap;
mod roadmap_sync;

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
        "figures" => figures::run(&root, write),
        "figures-math" => figures_math::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "figures-charts" => {
            figures_charts::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
        }
        "docs-inventory" => {
            docs_inventory::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
                .map_err(Into::into)
        }
        "links" => docs_links::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Fail::from),
        "docs-move" => {
            docs_move::run(&root, &std::env::args().skip(2).collect::<Vec<_>>()).map_err(Into::into)
        }
        "docs-relink" => docs_relink::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "docs-rewrite-refs" => {
            docs_rewrite_refs::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
                .map_err(Into::into)
        }
        "docs-catalog" => docs_catalog::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "roadmap" => {
            roadmap::run(&root, &std::env::args().skip(2).collect::<Vec<_>>()).map_err(Into::into)
        }
        "docs-stubs" => docs_stubs::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "book-check" => book_check::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "book-stage" => book_stage::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
            .map_err(Into::into),
        "archive-verify" => {
            archive_verify::run(&root, &std::env::args().skip(2).collect::<Vec<_>>())
                .map_err(Into::into)
        }
        "qubo-codec" => {
            let seed = std::env::args()
                .nth(2)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0xD11);
            let size = std::env::args()
                .nth(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(32);
            let sweeps = std::env::args()
                .nth(4)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(64);
            let (rtn, sa, ratio) = qubo_codec::benchmark(seed, size, sweeps);
            println!(
                "{{\"schema\":\"uor-r4.qubo-codec-benchmark/1\",\"seed\":{seed},\"group_size\":{size},\"sweeps\":{sweeps},\"rtn_err\":{rtn},\"sa_err\":{sa},\"sa_over_rtn\":{ratio},\"note\":\"AdaRound-QUBO solved classically (SA); no quantum device expedites this today (SIMULATED)\"}}"
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "cargo xtask <task>\n\
                 \n\
                 check-model       R1: conformance/*.toml is the single source; regenerate and diff\n\
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
                 figures           D27: the README/geometry status diagrams are generated and current\n\
                 figures-math      D27: computed geometry figures (E8, 600-cell, flock, SpiralCore, route) are current [--write] [--out DIR]\n\
                 figures-charts    D27: the docs data charts (status vs baselines, results share, timelines) drawn from the research catalog [--write] [--catalog FILE] [--out DIR]\n\
                 links [..]        docs reorganization: tracked-markdown link check (--json OUT, --baseline FILE, --renames-from REV, --simulate MAP.tsv); a moved target linked from an unchanged archive/ or prep record is reported as MOVED, not failed\n\
                 docs-inventory --out FILE.json [--prefix P ..]  docs reorganization: docs file inventory and by-string code references\n\
                 docs-move         move files/dirs with git mv from --map MAP.tsv (dry run unless --apply)\n\
                 docs-relink       rewrite live markdown links into moved paths from --map MAP.tsv [--exclude P ..] [--json OUT] (dry run unless --apply)\n\
                 docs-rewrite-refs rewrite by-string code references into moved paths from --map MAP.tsv [--exclude P ..] [--json OUT] (dry run unless --apply)\n\
                 docs-catalog      generate archive/CATALOG.md and archive/catalog/*.md from archive/catalog.jsonl [--input FILE] [--check]\n\
                 roadmap           check docs/roadmap/items/*.md, or render docs/roadmap/README.md from them: check | render [--check]\n\
                 roadmap sync      create or update one GitHub issue per roadmap item and the M1..M8 checklists through gh [--repo R] (dry run unless --apply)\n\
                 docs-stubs        pointer stubs at archive/ positions from --list FILE (dry run unless --apply)\n\
                 book-check        check that every relative link in the built book resolves [--dir DIR] (default book)\n\
                 book-stage        stage the mdBook source from docs/SUMMARY.md: links out to GitHub, $…$ maths to MathJax [--out DIR] (default target/book-src)\n\
                 archive-verify    SHA256SUMS manifests vs the tree [--json OUT] [--baseline FILE]; archive/ immutability [--base REV]\n\
                 \n\
                 --write           check-model/gnaf-root/gnaf-manifest/gnaf-conformance/figures: rewrite the generated file instead of checking it"
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
    let model = Model::load(&root.join("conformance"))?;
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
            "{} is stale: it disagrees with conformance/ids.toml.\n\
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
