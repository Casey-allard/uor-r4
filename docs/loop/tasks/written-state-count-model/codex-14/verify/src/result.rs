use super::{
    array, digest_file, ensure, history, mask, number, Census, Failure, Manifest, Mode, Score,
    Tally, BUCKETS, LANES, MODES, PRIMARY, WEIGHT_ONE,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const MAX_BITS_PER_TARGET: u64 = 48 * 4096;

#[derive(Deserialize)]
struct Fit {
    fit_bits_q12: Vec<u64>,
    endpoint: u64,
    converged: bool,
    best_pass: u64,
    best_checkpoint_pass: u64,
    rise_passes: Vec<u64>,
    scored: Score,
    buckets: Vec<Bucket>,
}

#[derive(Deserialize)]
struct Bucket {
    bucket: usize,
    targets: u64,
    mask: u16,
    start_q32: [u64; LANES],
    weights_q32: [u64; LANES],
    floor_dwell: [u64; LANES],
}

fn plateau(trace: &[u64], checkpoint: u64, tolerance: u64) -> bool {
    let endpoint = checkpoint as usize;
    let Some(window) = trace.get(endpoint - 8..=endpoint) else {
        return false;
    };
    window[0].abs_diff(window[8]) <= tolerance
        && window
            .windows(2)
            .all(|pair| pair[0].abs_diff(pair[1]) <= tolerance)
}

fn start_weights(mode: Mode, mask: u16) -> [u64; LANES] {
    let live = u64::from(mask.count_ones());
    let share = if mode.start == Some("near_kn") {
        1
    } else {
        WEIGHT_ONE / live
    };
    std::array::from_fn(|lane| match lane {
        0 => WEIGHT_ONE - share * (live - 1),
        _ if mask & (1 << lane) != 0 => share,
        _ => 0,
    })
}

fn validate_score(actual: Score, census: Score, key: &str) -> Result<(), Failure> {
    for (class, receipt, frame) in [
        ("all", actual.all, census.all),
        ("novel", actual.novel, census.novel),
    ] {
        ensure(
            receipt.targets == frame.targets
                && receipt.bytes == frame.bytes
                && receipt.bits_q12 <= receipt.targets * MAX_BITS_PER_TARGET,
            format!("{key} {class} score bounds or target/byte census"),
        )?;
    }
    ensure(
        actual.novel.bits_q12 <= actual.all.bits_q12,
        format!("{key} novel score exceeds all"),
    )?;
    Ok(())
}

fn validate_fit(fit: &Fit, mode: Mode, census: &Census) -> Result<(), Failure> {
    let key = mode.key;
    let primary = key == "u_count";
    let trace = &fit.fit_bits_q12;
    let total_fit = mode.fit.map_or(census.total.all.targets, |p| {
        census.by_parity[p].all.targets
    });
    let tolerance = 4096 * total_fit / 10_000;
    let expected = if primary {
        PRIMARY
            .into_iter()
            .find(|&checkpoint| plateau(trace, checkpoint, tolerance))
    } else {
        plateau(trace, 24, tolerance).then_some(24)
    };
    let endpoint = if primary { expected.unwrap_or(64) } else { 24 };
    ensure(
        fit.endpoint == endpoint && trace.len() == endpoint as usize + 1,
        format!("{key} registered endpoint and trace extent"),
    )?;
    ensure(
        fit.converged == expected.is_some(),
        format!("{key} reported convergence"),
    )?;
    ensure(
        trace
            .iter()
            .all(|&bits| bits <= total_fit * MAX_BITS_PER_TARGET),
        format!("{key} objective bound"),
    )?;
    let (best, _) = trace
        .iter()
        .enumerate()
        .min_by_key(|(_, value)| **value)
        .ok_or("empty objective")?;
    ensure(
        fit.best_pass == best as u64,
        format!("{key} earliest best objective pass"),
    )?;
    let best_checkpoint = PRIMARY
        .into_iter()
        .take_while(|&pass| pass <= endpoint)
        .min_by_key(|&pass| trace[pass as usize])
        .ok_or("empty checkpoint trace")?;
    ensure(
        fit.best_checkpoint_pass == best_checkpoint,
        format!("{key} earliest best checkpoint"),
    )?;
    let rises = trace
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[1] > pair[0])
        .map(|(i, _)| i as u64 + 1);
    ensure(
        fit.rise_passes.iter().copied().eq(rises),
        format!("{key} objective rise passes"),
    )?;
    let score_census = mode.score.map_or(census.total, |p| census.by_parity[p]);
    validate_score(fit.scored, score_census, key)?;
    if mode.fit == mode.score {
        ensure(
            fit.scored.all.bits_q12 == trace[endpoint as usize],
            format!("{key} in-sample objective/score identity"),
        )?;
    }
    ensure(fit.buckets.len() == BUCKETS, format!("{key} bucket width"))?;
    for (id, bucket) in fit.buckets.iter().enumerate() {
        let expected_mask = mask(id) & mode.enabled;
        let by_parity = census.bucket_targets[id];
        let targets = mode
            .fit
            .map_or(by_parity[0] + by_parity[1], |p| by_parity[p]);
        ensure(
            id == bucket.bucket && bucket.mask == expected_mask && bucket.targets == targets,
            format!("{key} bucket {id} census or mask"),
        )?;
        ensure(
            bucket.start_q32 == start_weights(mode, expected_mask),
            format!("{key} bucket {id} start"),
        )?;
        if targets == 0 {
            ensure(
                bucket.start_q32 == bucket.weights_q32,
                format!("{key} bucket {id} changed without fit data"),
            )?;
        }
        ensure(
            bucket.weights_q32.iter().all(|&w| w <= WEIGHT_ONE)
                && bucket.weights_q32.iter().sum::<u64>() == WEIGHT_ONE,
            format!("{key} bucket {id} Q32 sum"),
        )?;
        for lane in 0..LANES {
            let live = expected_mask & (1 << lane) != 0;
            let weight = bucket.weights_q32[lane];
            let dwell = bucket.floor_dwell[lane];
            ensure(
                (weight != 0) == live
                    && dwell <= endpoint * u64::from(live)
                    && (weight != 1 || dwell > 0)
                    && (dwell != endpoint || weight == 1)
                    && (targets != 0 || dwell == endpoint * u64::from(weight == 1)),
                format!("{key} bucket {id} lane {lane} floor or absent weight"),
            )?;
        }
    }
    Ok(())
}

fn check_historical(run: &Value, historical: &Value) -> Result<(), Failure> {
    let ceiling = &historical["ceiling"];
    for (mode, trace) in [
        ("u_count_uniform", &ceiling["u_count"]["fit_bits_q12"]),
        ("kn_u_uniform", &ceiling["kn_u"]["fit_bits_q12"]),
        ("split_even_fit", &ceiling["u_split"]["fit_bits_q12_even"]),
        ("split_odd_fit", &ceiling["u_split"]["fit_bits_q12_odd"]),
    ] {
        let actual: Vec<u64> = serde_json::from_value(run[mode]["fit_bits_q12"].clone())?;
        let expected: Vec<u64> = array(trace)?.iter().map(number).collect::<Result<_, _>>()?;
        ensure(
            actual == expected,
            format!("{mode} historical 25-word objective"),
        )?;
    }
    for (mode, receipt) in [
        ("kn_only", historical),
        ("u_count_uniform", &ceiling["u_count"]["scored"]),
        ("kn_u_uniform", &ceiling["kn_u"]["scored"]),
        ("u_split", &ceiling["u_split"]["scored"]),
    ] {
        let scored: Score = serde_json::from_value(run[mode]["scored"].clone())?;
        ensure(
            scored == history::score(receipt)?,
            format!("{mode} historical all/novel score"),
        )?;
    }
    for bucket in array(&ceiling["u_count"]["buckets"])? {
        let id = usize::try_from(number(&bucket["bucket"])?)?;
        let expected: Vec<u64> = array(&bucket["weights_q32"])?
            .iter()
            .map(number)
            .collect::<Result<_, _>>()?;
        let actual: Bucket = serde_json::from_value(run["u_count_uniform"]["buckets"][id].clone())?;
        ensure(
            number(&bucket["targets"])? == actual.targets && expected == actual.weights_q32,
            format!("uniform U_count historical weights and census in bucket {id}"),
        )?;
    }
    Ok(())
}

fn add(a: Tally, b: Tally) -> Tally {
    Tally {
        targets: a.targets + b.targets,
        bytes: a.bytes + b.bytes,
        bits_q12: a.bits_q12 + b.bits_q12,
    }
}

fn branch(bits: i128, reference: i128) -> &'static str {
    if 100 * bits <= 85 * reference {
        "counts carry"
    } else if 100 * bits > 90 * reference {
        "geometry critical"
    } else {
        "both open"
    }
}

fn caveat(trace: &[u64], at: usize, reference: i128) -> Value {
    let score = i128::from(trace[at]);
    let last_drop = i128::from(trace[at - 1].saturating_sub(trace[at]));
    let projected = score - 4 * last_drop;
    json!({
        "pass": at, "bits_q12": score, "projected_q12": projected,
        "branch": branch(score, reference), "projected_branch": branch(projected, reference),
        "crosses_cutoff": branch(score, reference) != branch(projected, reference),
    })
}

fn reading(run: &Value, per_mille: u64, synthetic_fixture: bool) -> Result<Value, Failure> {
    let scored = |key: &str| -> Result<Score, Failure> {
        Ok(serde_json::from_value(run[key]["scored"].clone())?)
    };
    let kn = scored("kn_only")?;
    let primary: Fit = serde_json::from_value(run["u_count"].clone())?;
    let restricted = scored("kn_u")?;
    let gains = |score: Score| {
        let gain = |model: Tally, base: Tally| {
            json!({
                "bits_q12": model.bits_q12,
                "kn_minus_model_q12": i128::from(base.bits_q12) - i128::from(model.bits_q12),
                "ratio_ppm": (base.bits_q12 > 0).then(|| model.bits_q12 * 1_000_000 / base.bits_q12),
            })
        };
        json!({ "all": gain(score.all, kn.all), "novel": gain(score.novel, kn.novel) })
    };
    let contains_kn = [primary.scored, restricted].into_iter().all(|mode| {
        mode.all.bits_q12 <= kn.all.bits_q12 && mode.novel.bits_q12 <= kn.novel.bits_q12
    });
    let reference = i128::from(kn.all.bits_q12);
    let bits = i128::from(primary.scored.all.bits_q12);
    let split_bits = i128::from(scored("u_split")?.all.bits_q12);
    let is_full = per_mille == 1000 && !synthetic_fixture;
    let applicable = is_full && primary.converged && contains_kn;
    Ok(json!({
        "per_mille": per_mille,
        "classification": if synthetic_fixture { "synthetic fixture" }
            else if !contains_kn { "unread: KN containment" }
            else if !primary.converged { "unread: registered convergence" }
            else if applicable { branch(bits, reference) }
            else { "same-cut diagnostics" },
        "contains_kn_all_and_novel": contains_kn,
        "converged": primary.converged,
        "endpoint": primary.endpoint,
        "best_pass": primary.best_pass,
        "best_pass_bits_q12": primary.fit_bits_q12[primary.best_pass as usize],
        "best_checkpoint_pass": primary.best_checkpoint_pass,
        "best_checkpoint_bits_q12": primary.fit_bits_q12[primary.best_checkpoint_pass as usize],
        "kn_only": kn,
        "u_count": gains(primary.scored),
        "u_count_uniform": gains(scored("u_count_uniform")?),
        "kn_u": gains(restricted),
        "kn_u_uniform": gains(scored("kn_u_uniform")?),
        "u_split": gains(scored("u_split")?),
        "split_gap_flag": 2 * (split_bits - bits) > reference - bits,
        "pass_24_sensitivity": is_full.then(|| caveat(&primary.fit_bits_q12, 24, reference)),
        "selected_endpoint_sensitivity": is_full.then(|| caveat(&primary.fit_bits_q12, primary.endpoint as usize, reference)),
        "feasible_upper_bound_q12": is_full.then_some(bits),
    }))
}

fn identity_matches(identity: Option<&Value>, seal: &str, synthetic: bool) -> bool {
    match identity {
        Some(Value::String(digest)) => digest == seal,
        Some(Value::Null) => synthetic,
        _ => false,
    }
}

pub(super) fn verify(
    directory: &Path,
    fitter: &Path,
    manifest: &Manifest,
    seal: &str,
    census: &Census,
    historical: Option<&Value>,
) -> Result<Value, Failure> {
    let run: Value = serde_json::from_slice(&fs::read(directory.join("result.json"))?)?;
    ensure(
        run["schema"] == "kn-ceiling-fit-result/1",
        "fit result schema",
    )?;
    ensure(
        run["targets"].as_u64() == Some(manifest.target_count),
        "fit result target count",
    )?;
    ensure(
        identity_matches(run.get("manifest_sha256"), seal, manifest.synthetic_fixture),
        "fit result manifest digest",
    )?;
    ensure(
        run["frame_sha256"].as_str() == Some(&manifest.frame_sha256),
        "fit result frame digest",
    )?;
    ensure(
        run["fitter_sha256"].as_str() == Some(&digest_file(fitter)?),
        "fit result fitter executable digest",
    )?;
    let tolerance = 4096 * manifest.target_count / 10_000;
    ensure(
        run["tolerance_q12"].as_u64() == Some(tolerance),
        "fit result registered tolerance",
    )?;
    for mode in MODES {
        let receipt_path = directory.join(format!("{}.json", mode.key));
        let receipt: Value = serde_json::from_slice(&fs::read(receipt_path)?)?;
        ensure(
            receipt == run[mode.key],
            format!("{} aggregate and standalone receipts differ", mode.key),
        )?;
        for field in ["manifest_sha256", "frame_sha256", "fitter_sha256"] {
            ensure(
                receipt.get(field) == run.get(field),
                format!("{} {field} identity", mode.key),
            )?;
        }
        if mode.start.is_some() {
            let fit: Fit = serde_json::from_value(run[mode.key].clone())?;
            validate_fit(&fit, mode, census)?;
        }
    }
    let kn: Score = serde_json::from_value(run["kn_only"]["scored"].clone())?;
    let even_fit: Score = serde_json::from_value(run["split_even_fit"]["scored"].clone())?;
    let odd_fit: Score = serde_json::from_value(run["split_odd_fit"]["scored"].clone())?;
    let split: Score = serde_json::from_value(run["u_split"]["scored"].clone())?;
    validate_score(kn, census.total, "kn_only")?;
    validate_score(split, census.total, "u_split")?;
    ensure(
        split
            == Score {
                all: add(even_fit.all, odd_fit.all),
                novel: add(even_fit.novel, odd_fit.novel),
            },
        "u_split must be the sum of held folds",
    )?;
    if let Some(history) = historical {
        check_historical(&run, history)?;
    }
    reading(&run, manifest.per_mille, manifest.synthetic_fixture)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_branch_boundaries_and_signed_projection() {
        assert_eq!(branch(85, 100), "counts carry");
        assert_eq!(branch(86, 100), "both open");
        assert_eq!(branch(90, 100), "both open");
        assert_eq!(branch(91, 100), "geometry critical");
        let details = caveat(&[20, 1], 1, 100);
        assert_eq!(details["projected_q12"], -75);
    }

    #[test]
    fn plateau_requires_net_and_every_individual_change() {
        let mut trace = vec![100; 25];
        trace[20] = 104;
        assert!(!plateau(&trace, 24, 3));
        assert!(plateau(&trace, 24, 4));
        for (i, x) in trace[16..].iter_mut().enumerate() {
            *x = 100 - i as u64;
        }
        assert!(!plateau(&trace, 24, 1));
        assert!(plateau(&trace, 24, 8));
        assert!(!plateau(&trace, 32, 8));
    }

    #[test]
    fn synthetic_can_use_production_identity_and_always_rejects_wrong_digest() {
        assert!(identity_matches(Some(&json!("seal")), "seal", true));
        assert!(identity_matches(Some(&json!("seal")), "seal", false));
        assert!(identity_matches(Some(&Value::Null), "seal", true));
        assert!(!identity_matches(Some(&Value::Null), "seal", false));
        assert!(!identity_matches(Some(&json!("wrong")), "seal", true));
        assert!(!identity_matches(None, "seal", true));
    }
}
