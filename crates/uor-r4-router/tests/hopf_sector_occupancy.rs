//! Issue #303: Hopf sector occupancy of the R⁴ router over the D3 held-out
//! fixtures.
//!
//! Drives the production routing path (`evolve_state` →
//! `route_query_to_manifold_native_with_hopf_input`, the same sequence as
//! `src/server.rs` POST /api/chat) over the held-out split of the D3 corpus
//! and measures which of the 512 Hopf sectors are reachable in practice.
//!
//! Run:
//!   cargo test -p uor-r4-router --release --offline \
//!     --test hopf_sector_occupancy -- --ignored --nocapture
//!
//! Skips vacuously when the D3 corpus is absent (same convention as
//! `uor-r4-core/tests/kappa_reproduction.rs`). The corpus fetch and the
//! held-out split rule (`blake3(id)[0] % 5 == 0`) are documented in
//! `scripts/fetch_d3_corpus.py` and the corpus manifest.

use std::collections::BTreeMap;
use std::path::PathBuf;

use uor_r4_router::UorR4Router;

/// Fixed session-evolution gain (the value the router tests use; the server
/// autotunes γ per request, which would make runs non-comparable).
const GAMMA: f64 = 0.85;
/// Article text is fed to the session in chunks of this many chars.
const CHUNK_CHARS: usize = 2000;
/// Sector budget K passed at the production call site.
const SECTOR_CAP: usize = 512;

struct Sample {
    sector_id: u64,
    chi_bin: u64,
    delta_bin: u64,
    alpha_bin: u64,
    chi_u: f64,
    u_delta: f64,
    u_alpha: f64,
    phase_transport_lambda: f64,
    /// The 512-d vector `get_state_4d_projection` reduced to the Hopf input.
    hopf_input: Vec<f64>,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus_path() -> PathBuf {
    workspace_root().join(".uor-models/corpora/simple-wiki-20231101/articles.jsonl")
}

fn report_path() -> PathBuf {
    std::env::var("HOPF_OCCUPANCY_REPORT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root().join("target/hopf_sector_occupancy/report.json"))
}

/// Canonical D3 split rule: `blake3(id as utf-8)[0] % 5 == 0` → held-out.
fn is_held_out(id: &str) -> bool {
    blake3::hash(id.as_bytes()).as_bytes()[0].is_multiple_of(5)
}

/// Split `text` into chunks of at most `CHUNK_CHARS` on char boundaries.
fn chunks(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let end = rest
            .char_indices()
            .nth(CHUNK_CHARS)
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = &rest[end..];
    }
    out
}

/// Deterministic per-block unit vector for the signed-projection diagnostic
/// (issue #303 scope item 4): 128 bytes from a keyed blake3 hash, centered
/// and L2-normalized. Fixed across runs — no RNG, no clock.
fn signed_probe(block: usize) -> Vec<f64> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4 issue-303 signed-projection probe");
    hasher.update(&(block as u64).to_le_bytes());
    let digest = hasher.finalize();
    let raw: Vec<f64> = digest
        .as_bytes()
        .iter()
        .cycle()
        .take(128)
        .map(|&b| (b as f64 / 255.0) - 0.5)
        .collect();
    let norm = raw.iter().map(|x| x * x).sum::<f64>().sqrt();
    raw.iter().map(|x| x / norm).collect()
}

fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut vx = 0.0;
    let mut vy = 0.0;
    for (&x, &y) in xs.iter().zip(ys) {
        cov += (x - mx) * (y - my);
        vx += (x - mx) * (x - mx);
        vy += (y - my) * (y - my);
    }
    if vx <= 0.0 || vy <= 0.0 {
        return 0.0;
    }
    cov / (vx.sqrt() * vy.sqrt())
}

fn mean_std(xs: &[f64]) -> (f64, f64) {
    let n = xs.len() as f64;
    let m = xs.iter().sum::<f64>() / n;
    let v = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n;
    (m, v.sqrt())
}

#[test]
#[ignore = "D3 corpus measurement; run explicitly per issue #303"]
fn hopf_sector_occupancy_d3() {
    let corpus = corpus_path();
    if !corpus.exists() {
        eprintln!(
            "D3 corpus absent at {}; skipping (fetch via scripts/fetch_d3_corpus.py)",
            corpus.display()
        );
        return;
    }

    let text = std::fs::read_to_string(&corpus).expect("read D3 corpus");
    let articles: Vec<serde_json::Value> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse corpus JSONL"))
        .collect();
    let held_out: Vec<&serde_json::Value> = articles
        .iter()
        .filter(|a| is_held_out(a["id"].as_str().expect("article id")))
        .collect();
    assert!(!held_out.is_empty(), "held-out split is empty");

    let mut router = UorR4Router::new(0.5);
    let mut samples: Vec<Sample> = Vec::new();

    for article in &held_out {
        let id = article["id"].as_str().expect("article id");
        let body = article["text"].as_str().expect("article text");
        for chunk in chunks(body) {
            // Same sequence as src/server.rs POST /api/chat: evolve the
            // session state, then route.
            router.evolve_state(id, chunk, GAMMA);
            let (routing, hopf_input) =
                router.route_query_to_manifold_native_with_hopf_input(chunk, id);
            let hopf = &routing.routed.hopf;
            samples.push(Sample {
                sector_id: hopf.sector_id,
                chi_bin: hopf.chi_bin,
                delta_bin: hopf.delta_bin,
                alpha_bin: hopf.alpha_bin,
                chi_u: hopf.chi.sin() * hopf.chi.sin(),
                u_delta: (hopf.delta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI),
                u_alpha: (hopf.transported_alpha + std::f64::consts::PI)
                    / (2.0 * std::f64::consts::PI),
                phase_transport_lambda: hopf.phase_transport_lambda,
                hopf_input,
            });
        }
    }
    assert!(!samples.is_empty(), "no routing samples collected");

    // ── Occupancy ────────────────────────────────────────────────────────
    let distinct_sectors = samples
        .iter()
        .map(|s| s.sector_id)
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let histogram = |pick: fn(&Sample) -> u64| -> BTreeMap<u64, usize> {
        let mut h: BTreeMap<u64, usize> = BTreeMap::new();
        for s in &samples {
            *h.entry(pick(s)).or_insert(0) += 1;
        }
        h
    };
    let chi_hist = histogram(|s| s.chi_bin);
    let delta_hist = histogram(|s| s.delta_bin);
    let alpha_hist = histogram(|s| s.alpha_bin);

    let range = |pick: fn(&Sample) -> f64| -> (f64, f64) {
        samples
            .iter()
            .map(pick)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            })
    };
    let chi_u_range = range(|s| s.chi_u);
    let u_delta_range = range(|s| s.u_delta);
    let u_alpha_range = range(|s| s.u_alpha);

    // ── Magnitude-only diagnostic (scope item 4) ─────────────────────────
    // Per 128-dim block: Pearson correlation across samples between the
    // block L2 norm (what the production projection keeps) and the signed
    // projection of the same block onto a fixed deterministic unit vector
    // (what it discards). Low |r| means the norm carries none of the
    // directional structure the signed summary sees.
    let probes: Vec<Vec<f64>> = (0..4).map(signed_probe).collect();
    let mut diagnostic = Vec::new();
    for (block, probe) in probes.iter().enumerate() {
        let norms: Vec<f64> = samples
            .iter()
            .map(|s| {
                s.hopf_input[block * 128..(block + 1) * 128]
                    .iter()
                    .map(|x| x * x)
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();
        let signed: Vec<f64> = samples
            .iter()
            .map(|s| {
                s.hopf_input[block * 128..(block + 1) * 128]
                    .iter()
                    .zip(probe)
                    .map(|(x, p)| x * p)
                    .sum()
            })
            .collect();
        let (norm_mean, norm_std) = mean_std(&norms);
        let (signed_mean, signed_std) = mean_std(&signed);
        diagnostic.push(serde_json::json!({
            "block": block,
            "pearson_r_norm_vs_signed": pearson(&norms, &signed),
            "norm_mean": norm_mean,
            "norm_std": norm_std,
            "signed_mean": signed_mean,
            "signed_std": signed_std,
        }));
    }

    let lambdas: Vec<f64> = samples.iter().map(|s| s.phase_transport_lambda).collect();
    let (lambda_min, lambda_max) = lambdas
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });

    let report = serde_json::json!({
        "issue": 303,
        "corpus": corpus.display().to_string(),
        "protocol": {
            "split_rule": "blake3(id as utf-8)[0] % 5 == 0 -> held-out",
            "articles_total": articles.len(),
            "articles_held_out": held_out.len(),
            "gamma": GAMMA,
            "chunk_chars": CHUNK_CHARS,
            "session_per": "article (identity = article id)",
            "sequence": "evolve_state(identity, chunk, gamma) then route per chunk",
        },
        "samples": samples.len(),
        "sector_cap": SECTOR_CAP,
        "distinct_sector_ids": distinct_sectors,
        "occupancy_fraction": distinct_sectors as f64 / SECTOR_CAP as f64,
        "bin_histograms": {
            "chi_bin": chi_hist,
            "delta_bin": delta_hist,
            "alpha_bin": alpha_hist,
        },
        "empirical_ranges": {
            "chi_u": [chi_u_range.0, chi_u_range.1],
            "u_delta": [u_delta_range.0, u_delta_range.1],
            "u_alpha": [u_alpha_range.0, u_alpha_range.1],
            "phase_transport_lambda": [lambda_min, lambda_max],
        },
        "magnitude_only_diagnostic": diagnostic,
    });

    let out = report_path();
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create report dir");
    }
    std::fs::write(
        &out,
        serde_json::to_string_pretty(&report).expect("serialize report"),
    )
    .expect("write report");

    println!("=== issue #303: Hopf sector occupancy (D3 held-out) ===");
    println!("held-out articles: {} / {}", held_out.len(), articles.len());
    println!("routing samples:   {}", samples.len());
    println!(
        "distinct sectors:  {} / {} ({:.1}%)",
        distinct_sectors,
        SECTOR_CAP,
        100.0 * distinct_sectors as f64 / SECTOR_CAP as f64
    );
    println!("chi_bin hist:      {chi_hist:?}");
    println!("delta_bin hist:    {delta_hist:?}");
    println!("alpha_bin hist:    {alpha_hist:?}");
    println!(
        "chi_u range:       [{:.4}, {:.4}]",
        chi_u_range.0, chi_u_range.1
    );
    println!(
        "u_delta range:     [{:.4}, {:.4}] (binning assumes [0, 1])",
        u_delta_range.0, u_delta_range.1
    );
    println!(
        "u_alpha range:     [{:.4}, {:.4}] (binning assumes [0, 1])",
        u_alpha_range.0, u_alpha_range.1
    );
    for d in diagnostic {
        println!(
            "block {}: r(norm, signed) = {:.4} (norm σ {:.4}, signed σ {:.4})",
            d["block"],
            d["pearson_r_norm_vs_signed"].as_f64().unwrap(),
            d["norm_std"].as_f64().unwrap(),
            d["signed_std"].as_f64().unwrap()
        );
    }
    println!("report written to {}", out.display());
}
