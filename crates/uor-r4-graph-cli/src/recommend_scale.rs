//! `recommend-scale` --- size an observation run for a teacher of any weight
//! class (#514).
//!
//! The substrate has to cover the teacher's induced context state-space. Two
//! demands scale with the teacher: *coverage* (enough records that held-out
//! contexts have an exact match --- low EXCT-miss) and *resolution* (a deeper,
//! wider teacher conditions on finer context, so more distinct keys to fill).
//! Coverage is mostly a corpus n-gram property and saturates near ~2M records
//! for natural text; resolution scales with a config-only capacity proxy.
//!
//! Closed form, calibrated on the runs we have:
//!
//! ```text
//! S(model)  = d_model * n_layers * log2(vocab)          // capacity proxy
//! N_needed  = N_REF * (S / S_REF) ^ BETA                // records to observe
//! ```
//!
//! Anchors (documented in docs/scaling_law.md):
//!   N_REF = 2_000_000 records --- the wiki coverage knee (#432: 97.8% EXCT
//!           full-code resolution at 2.11M vs 85.4% at 500k).
//!   S_REF = S(SmolLM2-360M)   --- the teacher this baseline pins (#509/#516).
//!   BETA  = 0.5 (PROVISIONAL) --- bounded by evidence in [0.45 (the measured
//!           cover-capacity exponent, #460), 1.0]; pinned by one sub-sample
//!           saturation sweep per teacher (#514). Override with --beta.
//!
//! The closed form is an estimate. When an observation stream exists, the
//! sub-sample sweep (docs/scaling_law.md) measures the real knee, which
//! supersedes this number.

use uor_r4_model_source::SourceUnavailable;

/// Records that saturate coverage on natural (wiki) text --- the knee, not the
/// ceiling. See #432.
const N_REF: f64 = 2_000_000.0;
/// Provisional resolution exponent. Bounded by [0.45, 1.0]; see module docs.
const BETA_DEFAULT: f64 = 0.5;
/// Mean observation records per Simple-Wiki article at full-article observation
/// (#432: 2.11M records / 10k articles).
const RECS_PER_ARTICLE_WIKI: f64 = 211.0;
/// Mean records per teacher-generated story (#432: 500k / 2,507).
const RECS_PER_ARTICLE_STORIES: f64 = 199.0;

/// The three measured coverage anchors: (records, EXCT full-code miss rate).
/// #509 (21k), #432 (500k, 2.11M). Used to interpolate an expected miss rate.
const COVERAGE_ANCHORS: [(f64, f64); 3] =
    [(21_235.0, 0.625), (500_000.0, 0.146), (2_110_111.0, 0.022)];

/// `d_model * n_layers * log2(vocab)`.
fn capacity_proxy(d_model: f64, n_layers: f64, vocab: f64) -> f64 {
    d_model * n_layers * vocab.log2()
}

/// SmolLM2-360M: d=960, layers=32, vocab=49152. The reference teacher.
fn s_ref() -> f64 {
    capacity_proxy(960.0, 32.0, 49152.0)
}

/// Expected EXCT full-code miss rate at `n` records: piecewise-linear in
/// `log10(n)` through the measured anchors, clamped to the observed range. A
/// corpus property, reported as a rough guide, not a promise.
fn expected_exct_miss(n: f64) -> f64 {
    let x = n.max(1.0).log10();
    let a = COVERAGE_ANCHORS;
    if x <= a[0].0.log10() {
        return a[0].1;
    }
    for w in a.windows(2) {
        let (x0, y0) = (w[0].0.log10(), w[0].1);
        let (x1, y1) = (w[1].0.log10(), w[1].1);
        if x <= x1 {
            let t = (x - x0) / (x1 - x0);
            return y0 + t * (y1 - y0);
        }
    }
    // Beyond the last anchor, hold the floor: coverage has plateaued.
    a[a.len() - 1].1
}

struct Config {
    d_model: f64,
    n_layers: f64,
    vocab: f64,
    name: String,
}

/// Read `hidden_size` / `num_hidden_layers` / `vocab_size` from an HF
/// `config.json` (a directory or the file itself).
fn config_from_hf(path: &str) -> Result<Config, SourceUnavailable> {
    let p = std::path::Path::new(path);
    let file = if p.is_dir() {
        p.join("config.json")
    } else {
        p.to_path_buf()
    };
    let text = std::fs::read_to_string(&file)
        .map_err(|e| SourceUnavailable::new(format!("cannot read {}: {e}", file.display())))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        SourceUnavailable::new(format!("{} is not valid JSON: {e}", file.display()))
    })?;
    let get = |k: &str| -> Result<f64, SourceUnavailable> {
        v.get(k).and_then(serde_json::Value::as_f64).ok_or_else(|| {
            SourceUnavailable::new(format!("{} has no numeric `{k}`", file.display()))
        })
    };
    Ok(Config {
        d_model: get("hidden_size")?,
        n_layers: get("num_hidden_layers")?,
        vocab: get("vocab_size")?,
        name: file
            .parent()
            .and_then(|d| d.file_name())
            .map_or_else(|| "model".to_string(), |n| n.to_string_lossy().to_string()),
    })
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// `recommend-scale` entry point.
pub fn run(args: &[String]) -> Result<(), SourceUnavailable> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "recommend-scale --- size an observe run for a teacher (#514)\n\
             usage: recommend-scale (--config <hf dir|config.json> | --d-model N --n-layers N --vocab N)\n\
             optional: [--corpus wiki|stories] [--beta B] [--n-ref R] [--s-ref S] [--recs-per-article R]\n\
             prints the recommended record count and article count, with a provisional-beta caveat."
        );
        return Ok(());
    }

    let cfg = if let Some(path) = parse_flag(args, "--config") {
        config_from_hf(&path)?
    } else {
        let need = |f: &str| -> Result<f64, SourceUnavailable> {
            parse_flag(args, f)
                .ok_or_else(|| {
                    SourceUnavailable::new(format!("missing {f} (or pass --config <hf dir>)"))
                })?
                .parse::<f64>()
                .map_err(|_| SourceUnavailable::new(format!("{f} must be a number")))
        };
        Config {
            d_model: need("--d-model")?,
            n_layers: need("--n-layers")?,
            vocab: need("--vocab")?,
            name: "model".to_string(),
        }
    };

    let corpus = parse_flag(args, "--corpus").unwrap_or_else(|| "wiki".to_string());
    let recs_per_article = parse_flag(args, "--recs-per-article")
        .and_then(|s| s.parse().ok())
        .unwrap_or(if corpus == "stories" {
            RECS_PER_ARTICLE_STORIES
        } else {
            RECS_PER_ARTICLE_WIKI
        });
    let beta = parse_flag(args, "--beta")
        .and_then(|s| s.parse().ok())
        .unwrap_or(BETA_DEFAULT);
    let n_ref = parse_flag(args, "--n-ref")
        .and_then(|s| s.parse().ok())
        .unwrap_or(N_REF);
    let s_reference = parse_flag(args, "--s-ref")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(s_ref);

    let s = capacity_proxy(cfg.d_model, cfg.n_layers, cfg.vocab);
    let n_needed = n_ref * (s / s_reference).powf(beta);
    let articles = n_needed / recs_per_article;
    let miss = expected_exct_miss(n_needed);

    println!("recommend-scale (#514) --- provisional, beta={beta} pending calibration");
    println!(
        "  teacher `{}`: d_model={} layers={} vocab={}",
        cfg.name, cfg.d_model as u64, cfg.n_layers as u64, cfg.vocab as u64
    );
    println!(
        "  capacity proxy S = {s:.0}  (S_ref[360M] = {s_reference:.0}, ratio {:.3})",
        s / s_reference
    );
    println!(
        "  recommended observation records: {:.0}  (~{:.0} {corpus} articles at {recs_per_article:.0}/article)",
        n_needed, articles
    );
    println!(
        "  expected EXCT full-code miss at that scale: ~{:.1}% (coverage guide, corpus property)",
        miss * 100.0
    );
    println!(
        "  note: closed-form estimate; a sub-sample saturation sweep on a real observe measures the true knee and supersedes this (docs/scaling_law.md)."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s_360m_is_the_reference() {
        // The 360M proxy is the anchor; ratio to itself is one.
        let s = capacity_proxy(960.0, 32.0, 49152.0);
        assert!((s / s_ref() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn needed_records_increase_with_teacher_capacity() {
        // A larger teacher needs at least as much corpus: 15M < 135M < 360M.
        let s15 = capacity_proxy(288.0, 6.0, 32000.0);
        let s135 = capacity_proxy(576.0, 30.0, 49152.0);
        let s360 = s_ref();
        assert!(s15 < s135 && s135 < s360);
        let n = |s: f64| N_REF * (s / s360).powf(BETA_DEFAULT);
        assert!(n(s15) < n(s135) && n(s135) < n(s360));
        // The reference lands at the coverage knee.
        assert!((n(s360) - N_REF).abs() < 1e-6);
    }

    #[test]
    fn coverage_interpolation_matches_anchors() {
        // At the measured points, the interpolation returns the measured miss.
        for (n, miss) in COVERAGE_ANCHORS {
            assert!((expected_exct_miss(n) - miss).abs() < 1e-9, "anchor {n}");
        }
        // Monotone non-increasing across the range.
        assert!(expected_exct_miss(100_000.0) > expected_exct_miss(1_000_000.0));
    }
}
