#![forbid(unsafe_code)]

mod fixture;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const BUCKETS: usize = 176;
const LANES: usize = 13;
const B: u128 = 1 << 32;
const M: u128 = 1 << 48;
const MAX_BITS_Q12: u64 = 48 * 4096;
const REFERENCE: u64 = 128_649_439_118;
const KN_MASS_SHA256: &str = "ae8518d552bc42d8f1c3602680bd77586d0cd9c237962fbf5e2b36704b6bb2fd";
const ANCHORS: [&str; 3] = ["pure_kn", "near_kn", "uniform"];
const MODES: [&str; 4] = ["kn_only", "u_count", "u_count_uniform", "kn_u"];

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    frame_sha256: String,
    kn_mass_sha256: String,
    source_sha256: String,
    binary_sha256: String,
    target_count: u64,
    per_mille: u64,
    bucket_targets: Vec<u64>,
    bucket_masks: Vec<u16>,
    inputs: Value,
    synthetic_fixture: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceReceipt {
    schema: String,
    manifest_sha256: String,
    frame_sha256: String,
    targets: u64,
    fit_result_sha256: String,
    verified_fit_sha256: String,
    certificate_binary_sha256: String,
    certificate_host_sha256: String,
    anchors: Vec<Anchor>,
    rowmax_q12: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Anchor {
    name: String,
    pass: usize,
    status: String,
    score_q12: Option<u64>,
    buckets: Option<Vec<Bucket>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bucket {
    bucket: usize,
    targets: u64,
    mask: u16,
    max_lane: usize,
    sums_q32: [Option<String>; LANES],
}

#[derive(Clone, Copy, Serialize)]
struct Point {
    mode: &'static str,
    pass: usize,
    score_q12: u64,
}

fn require(condition: bool, cause: impl Into<Error>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(cause.into())
    }
}

fn mask(bucket: usize) -> u16 {
    ((1 << (bucket / 16 + 2)) - 1) | (1 << (LANES - 1))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_digest(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn valid_sha(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn integer(value: &Value) -> Result<u64> {
    value
        .as_u64()
        .ok_or_else(|| "expected an unsigned JSON integer".into())
}

fn checked_add(a: u128, b: u128) -> Result<u128> {
    a.checked_add(b)
        .ok_or_else(|| "certificate addition overflow".into())
}

fn checked_mul(a: u128, b: u128) -> Result<u128> {
    a.checked_mul(b)
        .ok_or_else(|| "certificate multiplication overflow".into())
}

fn decimal(s: &str) -> Result<u128> {
    let n: u128 = s.parse()?;
    require(n.to_string() == s, "noncanonical directional sum")?;
    Ok(n)
}

fn bucket_residual(row: &Bucket, id: usize, targets: u64, live: u16) -> Result<u128> {
    require(
        row.bucket == id && row.targets == targets && row.mask == live,
        format!("bucket {id} identity, ordered census or mask"),
    )?;
    let n = u128::from(targets);
    let maximum_sum = checked_mul(checked_mul(n, B)?, checked_add(M, 1)?)?;
    let mut best = None;
    for (lane, sum) in row.sums_q32.iter().enumerate() {
        let present = live & (1 << lane) != 0;
        require(
            sum.is_some() == present,
            format!("bucket {id} lane {lane} presence"),
        )?;
        if let Some(sum) = sum {
            let value = decimal(sum)?;
            require(
                (n..=maximum_sum).contains(&value),
                format!("bucket {id} lane {lane} sum out of bounds"),
            )?;
            if best.is_none_or(|(_, score)| value > score) {
                best = Some((lane, value));
            }
        }
    }
    let (lane, maximum) = best.ok_or("missing live expert")?;
    require(
        row.max_lane == lane,
        format!("bucket {id} maximizing lane or tie order"),
    )?;
    maximum
        .checked_sub(checked_mul(n, B)?)
        .ok_or_else(|| format!("bucket {id} negative residual").into())
}

fn tangent_bound(score: u64, count: u64, residual: u128) -> Result<u64> {
    let log_allowance = checked_mul(4, u128::from(count))?;
    let descent_allowance = checked_mul(6144, residual)?.div_ceil(B);
    let allowance = checked_add(log_allowance, descent_allowance)?;
    Ok(u64::try_from(u128::from(score).saturating_sub(allowance))?)
}

fn branch(bits: u64, reference: u64) -> &'static str {
    let scaled = 100 * u128::from(bits);
    let reference = u128::from(reference);
    if scaled <= 85 * reference {
        "counts carry"
    } else if scaled > 90 * reference {
        "geometry critical"
    } else {
        "both open"
    }
}

fn classification(
    lower: Option<u64>,
    upper: u64,
    reference: u64,
    full: bool,
) -> (&'static str, Option<&'static str>) {
    let Some(lower) = lower else {
        return ("missing", None);
    };
    if !full {
        return ("same_cut_only", None);
    }
    let lower_branch = branch(lower, reference);
    let upper_branch = branch(upper, reference);
    if lower_branch == upper_branch {
        ("resolved", Some(upper_branch))
    } else {
        ("loose", None)
    }
}

fn validate_manifest(manifest: &Manifest, verified: &Value, seal: &str) -> Result<()> {
    require(
        manifest.schema == "kn-ceiling-fit-frame/1",
        "frame manifest schema",
    )?;
    require(
        manifest.target_count > 0 && manifest.target_count < (1 << 24),
        "registered target-count range",
    )?;
    require(
        manifest.bucket_targets.len() == BUCKETS,
        "manifest bucket census extent",
    )?;
    let targets = manifest
        .bucket_targets
        .iter()
        .try_fold(0u128, |sum, &count| checked_add(sum, u128::from(count)))?;
    require(
        targets == u128::from(manifest.target_count),
        "manifest incomplete target census",
    )?;
    require(
        manifest
            .bucket_masks
            .iter()
            .copied()
            .eq((0..BUCKETS).map(mask)),
        "manifest canonical lane mask",
    )?;
    for sha in [
        seal,
        &manifest.frame_sha256,
        &manifest.kn_mass_sha256,
        &manifest.source_sha256,
        &manifest.binary_sha256,
    ] {
        require(valid_sha(sha), "manifest or seal digest format")?;
    }
    require(
        verified["schema"] == "kn-ceiling-independent-verification/1"
            && verified["manifest_sha256"] == seal
            && verified["frame_sha256"] == manifest.frame_sha256
            && verified["kn_mass_sha256"] == manifest.kn_mass_sha256
            && verified["source_sha256"] == manifest.source_sha256
            && verified["collector_sha256"] == manifest.binary_sha256
            && verified["target_count"] == manifest.target_count
            && verified["per_mille"] == manifest.per_mille
            && verified["synthetic_fixture"] == manifest.synthetic_fixture,
        "independent frame verification identity",
    )?;
    if !manifest.synthetic_fixture {
        require(
            verified["verified_query_rows"] == true
                && verified["historical_frame_match"] == true
                && verified["historical_fit_controls_match"] == true,
            "production certification requires independent query, historical frame and fit checks",
        )?;
    }
    if !manifest.synthetic_fixture && manifest.per_mille == 1000 {
        for (field, hash) in [
            (
                "train_sha256",
                "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460",
            ),
            (
                "query_sha256",
                "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a",
            ),
            (
                "classes_sha256",
                "7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af",
            ),
            (
                "id_bytes_sha256",
                "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
            ),
        ] {
            require(
                manifest.inputs[field] == hash,
                format!("full frame {field}"),
            )?;
        }
        require(
            manifest.target_count == 10_807_417 && manifest.kn_mass_sha256 == KN_MASS_SHA256,
            "full frozen KN census or mass digest",
        )?;
    }
    Ok(())
}

fn best_points(
    fit: &Value,
    verified: &Value,
    manifest: &Manifest,
    seal: &str,
    fitter_sha: &str,
) -> Result<([Point; 4], [Point; 4])> {
    require(
        fit["schema"] == "kn-ceiling-fit-result/1"
            && (fit["manifest_sha256"] == seal
                || (manifest.synthetic_fixture && fit["manifest_sha256"].is_null()))
            && fit["frame_sha256"] == manifest.frame_sha256
            && fit["fitter_sha256"] == fitter_sha
            && fit["targets"] == manifest.target_count,
        "fitter executable, frame, manifest or target identity",
    )?;
    let r = &verified["reading"];
    let kn = &fit["kn_only"]["scored"];
    require(
        *kn == r["kn_only"] && kn["all"]["targets"] == manifest.target_count,
        "pure KN verified tally",
    )?;
    let reference = integer(&kn["all"]["bits_q12"])?;
    if !manifest.synthetic_fixture && manifest.per_mille == 1000 {
        require(
            reference == REFERENCE
                && kn["all"]["bytes"] == 21_928_475
                && kn["novel"]["targets"] == 7_566_174
                && kn["novel"]["bytes"] == 15_995_822
                && kn["novel"]["bits_q12"] == 114_471_363_642u64,
            "full frozen KN denominators and scores",
        )?;
    }
    let pure_kn = Point {
        mode: MODES[0],
        pass: 0,
        score_q12: reference,
    };
    let mut points = [pure_kn; MODES.len()];
    let mut endpoints = [pure_kn; MODES.len()];
    for (index, mode) in MODES.into_iter().enumerate().skip(1) {
        let receipt = &fit[mode];
        let trace: Vec<u64> = serde_json::from_value(receipt["fit_bits_q12"].clone())?;
        let endpoint = usize::try_from(integer(&receipt["endpoint"])?)?;
        require(
            trace.len().checked_sub(1) == Some(endpoint)
                && (if mode == "u_count" {
                    (24..=64).contains(&endpoint) && endpoint % 8 == 0
                } else {
                    endpoint == 24
                })
                && trace
                    .iter()
                    .all(|&score| score <= manifest.target_count * MAX_BITS_Q12)
                && receipt["scored"]["all"]["targets"] == manifest.target_count,
            format!("{mode} registered bounded in-sample trace"),
        )?;
        let end_score = integer(&receipt["scored"]["all"]["bits_q12"])?;
        require(
            trace[endpoint] == end_score
                && r[mode]["all"]["bits_q12"] == end_score
                && r[mode]["novel"]["bits_q12"] == receipt["scored"]["novel"]["bits_q12"],
            format!("{mode} scored endpoint and verifier agreement"),
        )?;
        let (pass, &score_q12) = trace
            .iter()
            .enumerate()
            .min_by_key(|(_, score)| *score)
            .ok_or("empty trace")?;
        require(
            receipt["best_pass"] == pass,
            format!("{mode} first best pass"),
        )?;
        if mode == "u_count" {
            require(
                r["best_pass"] == pass && r["best_pass_bits_q12"] == score_q12,
                "verified primary best point",
            )?;
        }
        points[index] = Point {
            mode,
            pass,
            score_q12,
        };
        endpoints[index] = Point {
            mode,
            pass: endpoint,
            score_q12: end_score,
        };
    }
    Ok((points, endpoints))
}

fn check_mode_receipts(directory: &Path, fit: &Value, manifest: &Manifest) -> Result<()> {
    for mode in MODES {
        let independent: Value =
            serde_json::from_slice(&fs::read(directory.join(format!("{mode}.json")))?)?;
        require(
            independent == fit[mode],
            format!("{mode} standalone receipt mismatch"),
        )?;
        if mode == "kn_only" {
            continue;
        }
        let buckets = independent["buckets"].as_array().ok_or("fit bucket list")?;
        require(
            buckets.len() == BUCKETS,
            format!("{mode} fit bucket extent"),
        )?;
        let enabled = if mode == "kn_u" {
            1 | (1 << (LANES - 1))
        } else {
            (1 << LANES) - 1
        };
        for (index, bucket) in buckets.iter().enumerate() {
            let live = manifest.bucket_masks[index] & enabled;
            require(
                bucket["bucket"] == index
                    && bucket["targets"] == manifest.bucket_targets[index]
                    && bucket["mask"] == live,
                format!("{mode} bucket {index} fit identity"),
            )?;
            for field in ["start_q32", "weights_q32"] {
                let weights: [u64; LANES] = serde_json::from_value(bucket[field].clone())?;
                require(
                    weights.iter().map(|&w| u128::from(w)).sum::<u128>() == B
                        && weights
                            .iter()
                            .enumerate()
                            .all(|(e, &w)| (w > 0) == (live & (1 << e) != 0)),
                    format!("{mode} bucket {index} feasible {field}"),
                )?;
            }
        }
    }
    Ok(())
}

fn verify_bounds(
    raw: DeviceReceipt,
    manifest: &Manifest,
    fit: &Value,
    points: &[Point; 4],
    fixture: Option<&fixture::Tape>,
) -> Result<(Option<u64>, Vec<Value>)> {
    require(
        raw.anchors.len() == ANCHORS.len(),
        "registered anchor count",
    )?;
    let mut lower = raw.rowmax_q12;
    if let (Some(tape), Some(rowmax)) = (fixture, raw.rowmax_q12) {
        require(
            tape.rowmax()? == rowmax,
            "finite row maximum score mismatch",
        )?;
    }
    let mut details = Vec::with_capacity(ANCHORS.len());
    for (index, anchor) in raw.anchors.into_iter().enumerate() {
        let point = points[index];
        require(
            anchor.name == ANCHORS[index] && anchor.pass == point.pass,
            "registered anchor order or best pass changed",
        )?;
        let weights_available =
            index == 0 || point.pass == 0 || fit[point.mode]["endpoint"] == point.pass;
        if !weights_available {
            require(
                anchor.status == "weights_unavailable"
                    && anchor.score_q12.is_none()
                    && anchor.buckets.is_none(),
                "unavailable anchor claims arithmetic",
            )?;
            details.push(json!({"name": anchor.name, "pass": point.pass, "status": anchor.status}));
            continue;
        }
        require(
            anchor.status == "evaluated" && anchor.score_q12 == Some(point.score_q12),
            "anchor score or status",
        )?;
        let buckets = anchor.buckets.ok_or("missing anchor bucket receipts")?;
        require(
            buckets.len() == BUCKETS,
            "incomplete ordered bucket receipts",
        )?;
        let residual = buckets
            .iter()
            .enumerate()
            .try_fold(0u128, |sum, (id, row)| {
                checked_add(
                    sum,
                    bucket_residual(
                        row,
                        id,
                        manifest.bucket_targets[id],
                        manifest.bucket_masks[id],
                    )?,
                )
            })?;
        if let Some(tape) = fixture {
            tape.verify_anchor(point, fit, &buckets)?;
        }
        let bound = tangent_bound(point.score_q12, manifest.target_count, residual)?;
        lower = Some(lower.map_or(bound, |old| old.max(bound)));
        details.push(json!({
            "name": anchor.name, "pass": point.pass, "status": anchor.status,
            "anchor_q12": point.score_q12, "residual_q32": residual.to_string(), "lower_q12": bound,
        }));
    }
    Ok((lower, details))
}

fn run() -> Result<Value> {
    let mut args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let reverify = if matches!(args.len(), 8 | 11) {
        let tail = args.split_off(args.len() - 4);
        require(
            tail[0] == Path::new("--reverify"),
            "expected --reverify option",
        )?;
        Some(tail)
    } else {
        None
    };
    require(matches!(args.len(), 4 | 7), concat!(
        "usage: kn-ceiling-certificate-verify FRAME_DIR VERIFIED_JSON FIT_DIR FITTER ",
        "[DEVICE_JSON CUDA_WORKER CERTIFICATE_HOST] [--reverify PRIMARY_VERIFIER COLLECTOR QUERY_DIR]"
    ))?;
    let file = fs::read(args[0].join("manifest.json"))?;
    let seal = digest(&file);
    require(
        fs::read(args[0].join("seal.sha256"))? == format!("{seal}  manifest.json\n").as_bytes(),
        "manifest seal mismatch",
    )?;
    let manifest: Manifest = serde_json::from_slice(&file)?;
    let verified_bytes = fs::read(&args[1])?;
    let verified_sha = digest(&verified_bytes);
    let verified: Value = serde_json::from_slice(&verified_bytes)?;
    validate_manifest(&manifest, &verified, &seal)?;
    let fit_file = fs::read(args[2].join("result.json"))?;
    let fit_sha = digest(&fit_file);
    let fit: Value = serde_json::from_slice(&fit_file)?;
    let fitter_sha = file_digest(&args[3])?;
    let mut primary_verifier_sha = None;
    if let Some(paths) = reverify {
        let primary_verifier = &paths[1];
        let verifier_sha = file_digest(primary_verifier)?;
        let mut command = Command::new(primary_verifier);
        command
            .arg(&args[0])
            .arg(&manifest.source_sha256)
            .arg(&paths[2])
            .arg("--query")
            .arg(&paths[3])
            .arg("--fit")
            .arg(&args[2])
            .arg(&args[3]);
        if manifest.synthetic_fixture {
            command.arg("--fixture");
        }
        let checked = command.output()?;
        require(
            checked.status.success(),
            format!(
                "independent fit recheck failed: {}",
                String::from_utf8_lossy(&checked.stderr)
            ),
        )?;
        let checked_receipt: Value = serde_json::from_slice(&checked.stdout)?;
        require(
            checked_receipt == verified
                && file_digest(&args[2].join("result.json"))? == fit_sha
                && file_digest(&args[3])? == fitter_sha
                && file_digest(primary_verifier)? == verifier_sha,
            "independent verification result or pinned executable changed",
        )?;
        primary_verifier_sha = Some(verifier_sha);
    } else {
        require(
            manifest.synthetic_fixture,
            "production certificate requires --reverify",
        )?;
    }
    check_mode_receipts(&args[2], &fit, &manifest)?;
    let (points, endpoints) = best_points(&fit, &verified, &manifest, &seal, &fitter_sha)?;
    let mut upper = *endpoints
        .iter()
        .min_by_key(|point| point.score_q12)
        .ok_or("no feasible scores")?;
    let reference = points[0].score_q12;

    let mut identities = Value::Null;
    let mut rowmax = None;
    let mut details = Vec::new();
    let mut lower = None;
    if args.len() == 7 {
        let raw_bytes = fs::read(&args[4])?;
        let device_sha = digest(&raw_bytes);
        let raw: DeviceReceipt = serde_json::from_slice(&raw_bytes)?;
        require(
            raw.schema == "kn-ceiling-device-certificate/1"
                && raw.manifest_sha256 == seal
                && raw.frame_sha256 == manifest.frame_sha256
                && raw.targets == manifest.target_count
                && raw.fit_result_sha256 == fit_sha
                && raw.verified_fit_sha256 == verified_sha,
            "device certificate frame, fit or verified receipt identity",
        )?;
        require(
            raw.certificate_binary_sha256 == file_digest(&args[5])?
                && raw.certificate_host_sha256 == file_digest(&args[6])?,
            "device certificate executable identity",
        )?;
        identities = json!({
            "device_receipt_sha256": device_sha, "worker_sha256": raw.certificate_binary_sha256,
            "orchestrator_sha256": raw.certificate_host_sha256,
        });
        rowmax = raw.rowmax_q12;
        let fixture = manifest
            .synthetic_fixture
            .then(|| fixture::Tape::read(&args[0], &manifest))
            .transpose()?;
        (lower, details) = verify_bounds(raw, &manifest, &fit, &points, fixture.as_ref())?;
        for (point, detail) in points.iter().zip(&details) {
            if point.pass == 0
                && detail["status"] == "evaluated"
                && point.score_q12 < upper.score_q12
            {
                upper = *point;
            }
        }
        require(
            lower.is_some_and(|bound| bound <= upper.score_q12),
            "invalid interval: lower exceeds feasible upper",
        )?;
    }
    let upper_q12 = upper.score_q12;
    let full = !manifest.synthetic_fixture && manifest.per_mille == 1000;
    let (status, conclusion) = classification(lower, upper_q12, reference, full);
    Ok(json!({
        "schema": "kn-ceiling-certificate-verification/1",
        "family": "one Q32 simplex vector per bucket, exact Q48 mixed-mass floor and integer Q12 score",
        "manifest_sha256": seal,
        "frame_sha256": manifest.frame_sha256,
        "fit_result_sha256": fit_sha,
        "verified_fit_sha256": verified_sha,
        "fitter_sha256": fitter_sha,
        "primary_verifier_sha256": primary_verifier_sha,
        "device": identities,
        "per_mille": manifest.per_mille,
        "target_count": manifest.target_count,
        "reference_q12": reference,
        "rowmax_q12": rowmax,
        "lower_q12": lower,
        "upper_q12": upper_q12,
        "upper_witness": upper,
        "fit_endpoints": endpoints,
        "best_observed_points": points,
        "anchors": details,
        "status": status,
        "proved_branch": conclusion,
    }))
}

fn main() {
    let result = run().and_then(|value| Ok(serde_json::to_string_pretty(&value)?));
    match result {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("certificate verification failed: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod finite;
