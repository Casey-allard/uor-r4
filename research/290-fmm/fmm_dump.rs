//! FMM §5.1 data dump (issue #290): reproduce the cover that produced
//! `.uor-models/compiled/SmolLM2-135M-Instruct-7e27bd9f9532/graph-cover/cover.r4g1`
//! bit-for-bit (verified via the pinned kappas in cover_report.json),
//! then dump train observation vectors, sign signatures, region
//! prototypes, radii and top-1 member lists for offline SVD analysis.
//!
//! Usage: fmm_dump <corpus_meta> <corpus_recs> <artifacts> <out_dir>

use std::fs;
use std::path::Path;

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_compiler::induction;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: fmm_dump <corpus_meta> <corpus_recs> <artifacts> <out_dir>".into());
    }
    let artifact_container = fs::read(&args[3]).map_err(|e| format!("{}: {e}", args[3]))?;
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let meta_bytes = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let recs_bytes = fs::read(&args[2]).map_err(|e| e.to_string())?;
    let mut h = blake3::Hasher::new();
    h.update(&meta_bytes);
    h.update(&recs_bytes);
    let corpus_kappa = format!("blake3:{}", h.finalize().to_hex());

    let corpus = compiler::load_corpus_from(&args[1], &args[2])
        .ok_or_else(|| "corpus incomplete".to_owned())?;
    let artifacts = compiler::parse_artifacts(&artifact_container)
        .ok_or_else(|| "not a TLA3/TLA4/TLA5 artifact container".to_owned())?;

    // Config pinned by graph-cover/cover_report.json (schema 1).
    let config = induction::CoverConfig {
        depths: 5,
        k0: 16,
        regions_budget: 2048,
        memory_budget_bytes: 2147483648,
        threads: 8,
        min_support: 32,
        entropy_gain_bits: 0.1,
        radius_quantile_numerator: 80,
        radius_quantile_denominator: 100,
    };
    let (train_positions, _held_out) = induction::split_positions(&corpus);
    eprintln!("building {} train observations...", train_positions.len());
    let train = induction::build_observations(&artifacts, &corpus, &train_positions);
    eprintln!("inducing cover...");
    let induced = induction::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)?;
    let cover_kappa = induced.cover.kappa();

    // Verification against the pinned report values.
    let per_depth: Vec<usize> = (1..=induced.cover.max_depth)
        .map(|d| induced.cover.regions_at_depth(d).len())
        .collect();
    let checks = [
        ("artifact_kappa", artifact_kappa.as_str(), "blake3:c09164697949ec7189b58b0ad83713ac30565a59f6a8637afab571baa88e3567"),
        ("corpus_kappa", corpus_kappa.as_str(), "blake3:74491d1d80f426f675a35f22a98e6ca0a7de83bbfd1f87bcb9ad763ebe96f12a"),
        ("cover_kappa", cover_kappa.as_str(), "blake3:67d957c3bdfae5d78697e484faeb553c3a137d2b848e9fb69758c52853aba651"),
    ];
    let mut ok = true;
    for (name, got, want) in checks {
        let pass = got == want;
        ok &= pass;
        eprintln!("check {name}: {got} {} (pinned {want})", if pass { "MATCH" } else { "MISMATCH" });
    }
    eprintln!(
        "regions: total {} per_depth {:?} (pinned 362 / [16, 30, 58, 98, 160])",
        induced.cover.regions.len(),
        per_depth
    );
    ok &= induced.cover.regions.len() == 362 && per_depth == [16, 30, 58, 98, 160];

    // Dump.
    let out = Path::new(&args[4]);
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let d = compiler::D;
    let n = train.len();

    let mut vbuf = Vec::with_capacity(n * d * 4);
    for o in &train {
        for &x in &o.vector {
            vbuf.extend_from_slice(&x.to_le_bytes());
        }
    }
    fs::write(out.join("vectors.bin"), &vbuf).map_err(|e| e.to_string())?;

    let mut sbuf = Vec::with_capacity(n * compiler::SIG_BYTES);
    for o in &train {
        sbuf.extend_from_slice(&o.sig);
    }
    fs::write(out.join("sigs.bin"), &sbuf).map_err(|e| e.to_string())?;

    let regions = &induced.cover.regions;
    let mut pbuf = Vec::with_capacity(regions.len() * d * 4);
    for r in regions {
        for &x in &r.prototype {
            pbuf.extend_from_slice(&x.to_le_bytes());
        }
    }
    fs::write(out.join("prototypes.bin"), &pbuf).map_err(|e| e.to_string())?;

    let regions_json: Vec<serde_json::Value> = regions
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "depth": r.depth,
                "parent": r.parent,
                "radius": r.radius,
                "support": r.support,
                "members": induced.cover.members[r.id as usize],
            })
        })
        .collect();
    let meta = serde_json::json!({
        "d": d,
        "sig_bytes": compiler::SIG_BYTES,
        "n_train": n,
        "n_regions": regions.len(),
        "max_depth": induced.cover.max_depth,
        "artifact_kappa": artifact_kappa,
        "corpus_kappa": corpus_kappa,
        "cover_kappa": cover_kappa,
        "per_depth": per_depth,
        "reproduction_verified": ok,
    });
    fs::write(
        out.join("regions.json"),
        serde_json::to_string(&regions_json).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        out.join("meta.json"),
        serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    eprintln!("dump complete; reproduction_verified = {ok}");
    if ok {
        Ok(())
    } else {
        Err("reproduction checks FAILED — do not use this dump".into())
    }
}
