#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Failure = Box<dyn std::error::Error>;
const LANES: usize = 13;
const BUCKETS: usize = 176;
const ROW_BYTES: usize = 109;
const MASS_ONE: u64 = 1 << 48;
const WEIGHT_ONE: u64 = 1 << 32;
const MAX_SCORE_PER_ROW: u64 = 48 * 4096;

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    row_bytes: usize,
    target_count: u32,
    frame_bytes: u64,
    frame_sha256: String,
    bucket_masks: Vec<u16>,
    bucket_targets: Vec<u32>,
    synthetic_fixture: bool,
}

#[derive(Deserialize)]
struct Verified {
    schema: String,
    manifest_sha256: String,
    frame_sha256: String,
    target_count: u32,
    synthetic_fixture: bool,
    verified_query_rows: bool,
    reading: Value,
}

#[derive(Deserialize)]
struct Fit {
    schema: String,
    manifest_sha256: Option<String>,
    frame_sha256: String,
    targets: u32,
    kn_only: KnMode,
    u_count: FitMode,
    u_count_uniform: FitMode,
}

#[derive(Deserialize)]
struct KnMode {
    scored: Score,
}

#[derive(Deserialize)]
struct FitMode {
    fit_bits_q12: Vec<u64>,
    best_pass: usize,
    endpoint: usize,
    scored: Score,
    buckets: Vec<FitBucket>,
}

#[derive(Deserialize)]
struct Score {
    all: Tally,
}

#[derive(Deserialize)]
struct Tally {
    targets: u32,
    bits_q12: u64,
}

#[derive(Deserialize)]
struct FitBucket {
    bucket: usize,
    targets: u32,
    mask: u16,
    start_q32: [u64; LANES],
    weights_q32: [u64; LANES],
}

struct Anchor {
    name: &'static str,
    pass: usize,
    score_q12: u64,
    weights: Option<Vec<[u64; LANES]>>,
}

#[derive(Serialize)]
struct Bucket {
    bucket: usize,
    targets: u32,
    mask: u16,
    max_lane: usize,
    sums_q32: [Option<String>; LANES],
}

#[derive(Serialize)]
struct AnchorReceipt {
    name: &'static str,
    pass: usize,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    score_q12: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    buckets: Option<Vec<Bucket>>,
}

#[derive(Serialize)]
struct Receipt {
    schema: &'static str,
    manifest_sha256: String,
    frame_sha256: String,
    targets: u32,
    fit_result_sha256: String,
    verified_fit_sha256: String,
    certificate_binary_sha256: String,
    certificate_host_sha256: String,
    anchors: Vec<AnchorReceipt>,
    rowmax_q12: u64,
}

fn require(condition: bool, error: &'static str) -> Result<(), Failure> {
    if condition {
        Ok(())
    } else {
        Err(error.into())
    }
}

fn digest_file(path: &Path) -> Result<String, Failure> {
    let mut hash = Sha256::new();
    io::copy(&mut File::open(path)?, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn mask(bucket: usize) -> u16 {
    ((1 << (bucket / 16 + 2)) - 1) | (1 << (LANES - 1))
}

impl Manifest {
    fn validate(&self) -> Result<(), Failure> {
        require(
            self.schema == "kn-ceiling-fit-frame/1"
                && self.target_count > 0
                && self.target_count < 1 << 24
                && self.row_bytes == ROW_BYTES
                && self.frame_bytes == u64::from(self.target_count) * ROW_BYTES as u64,
            "frame shape",
        )?;
        require(
            self.bucket_masks.iter().copied().eq((0..BUCKETS).map(mask))
                && self.bucket_targets.len() == BUCKETS
                && self
                    .bucket_targets
                    .iter()
                    .map(|&n| u64::from(n))
                    .sum::<u64>()
                    == u64::from(self.target_count),
            "frame bucket census or mask",
        )
    }
}

impl FitMode {
    fn anchor(&self, name: &'static str, manifest: &Manifest) -> Result<Anchor, Failure> {
        let (first_best, &score_q12) = self
            .fit_bits_q12
            .iter()
            .enumerate()
            .min_by_key(|(_, score)| *score)
            .ok_or("empty fit trace")?;
        require(
            self.endpoint.checked_add(1) == Some(self.fit_bits_q12.len())
                && self.best_pass == first_best
                && self.scored.all.targets == manifest.target_count
                && Some(&self.scored.all.bits_q12) == self.fit_bits_q12.last()
                && self.buckets.len() == BUCKETS,
            "fit pass, score or bucket census",
        )?;
        let mut selected = Vec::with_capacity(BUCKETS);
        for (b, bucket) in self.buckets.iter().enumerate() {
            require(
                bucket.bucket == b
                    && bucket.mask == manifest.bucket_masks[b]
                    && bucket.targets == manifest.bucket_targets[b],
                "fit bucket identity",
            )?;
            for weights in [&bucket.start_q32, &bucket.weights_q32] {
                require(
                    weights.iter().enumerate().all(|(e, &weight)| {
                        weight <= WEIGHT_ONE && ((weight > 0) == ((bucket.mask & (1 << e)) != 0))
                    }) && weights.iter().sum::<u64>() == WEIGHT_ONE,
                    "anchor weights outside Q32 simplex",
                )?;
            }
            selected.push(if first_best == 0 {
                bucket.start_q32
            } else {
                bucket.weights_q32
            });
        }
        Ok(Anchor {
            name,
            pass: first_best,
            score_q12,
            weights: (first_best == self.endpoint || first_best == 0).then_some(selected),
        })
    }
}

fn stage_rows(frame: &Path, manifest: &Manifest) -> Result<Vec<[u64; LANES]>, Failure> {
    let input = File::open(frame)?;
    require(
        input.metadata()?.is_file() && input.metadata()?.len() == manifest.frame_bytes,
        "frame payload length",
    )?;
    let mut starts = [0; BUCKETS + 1];
    for (b, &count) in manifest.bucket_targets.iter().enumerate() {
        starts[b + 1] = starts[b] + count as usize;
    }
    let mut cursors = starts;
    let mut rows = Vec::new();
    rows.try_reserve_exact(manifest.target_count as usize)?;
    rows.resize(manifest.target_count as usize, [0; LANES]);
    let mut reader = BufReader::with_capacity(1 << 20, input);
    let mut hash = Sha256::new();
    let mut row = [0; ROW_BYTES];
    for _ in 0..manifest.target_count {
        reader.read_exact(&mut row)?;
        hash.update(row);
        let bucket = u16::from_le_bytes([row[0], row[1]]) as usize;
        require(
            bucket < BUCKETS && row[2] <= 1 && row[4] <= 1,
            "frame row metadata outside domain",
        )?;
        let index = cursors[bucket];
        require(index < starts[bucket + 1], "frame bucket overflow")?;
        cursors[bucket] += 1;
        let present = manifest.bucket_masks[bucket];
        for (lane, bytes) in row[5..].chunks_exact(8).enumerate() {
            let mass = u64::from_le_bytes(bytes.try_into()?);
            require(
                mass <= MASS_ONE
                    && (lane != 0 || mass > 0)
                    && (lane != LANES - 1 || mass == MASS_ONE / 4096)
                    && (present & (1 << lane) != 0 || mass == 0),
                "frame row mass outside domain",
            )?;
            rows[index][lane] = mass;
        }
    }
    let mut extra = [0];
    require(
        reader.read(&mut extra)? == 0
            && cursors[..BUCKETS] == starts[1..]
            && format!("{:x}", hash.finalize()) == manifest.frame_sha256,
        "frame EOF, bucket census or payload SHA-256 mismatch",
    )?;
    Ok(rows)
}

fn write_worker(
    writer: impl Write,
    manifest: &Manifest,
    anchors: &[Anchor],
    rows: &[[u64; LANES]],
) -> Result<(), Failure> {
    let mut output = BufWriter::with_capacity(1 << 20, writer);
    let count = anchors.iter().filter(|a| a.weights.is_some()).count() as u32;
    output.write_all(&manifest.target_count.to_le_bytes())?;
    output.write_all(&count.to_le_bytes())?;
    for &targets in &manifest.bucket_targets {
        output.write_all(&targets.to_le_bytes())?;
    }
    for weights in anchors.iter().filter_map(|anchor| anchor.weights.as_ref()) {
        for &word in weights.iter().flatten() {
            output.write_all(&word.to_le_bytes())?;
        }
    }
    for &word in rows.iter().flatten() {
        output.write_all(&word.to_le_bytes())?;
    }
    output.flush()?;
    Ok(())
}

fn run_worker(
    worker: &Path,
    manifest: &Manifest,
    anchors: &[Anchor],
    rows: &[[u64; LANES]],
) -> Result<Vec<u8>, Failure> {
    let mut child = Command::new(worker)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let written = child
        .stdin
        .take()
        .ok_or("missing CUDA worker stdin")
        .map_err(Failure::from)
        .and_then(|input| write_worker(input, manifest, anchors, rows));
    if let Err(error) = written {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let out = child.wait_with_output()?;
    require(out.status.success(), "CUDA worker failed")?;
    Ok(out.stdout)
}

fn next_word(input: &mut &[u8]) -> Result<u128, Failure> {
    let mut bytes = [0; 16];
    input.read_exact(&mut bytes)?;
    Ok(u128::from_le_bytes(bytes))
}

fn parse_worker(
    mut input: &[u8],
    manifest: &Manifest,
    anchors: &[Anchor],
) -> Result<(Vec<AnchorReceipt>, u64), Failure> {
    let mut receipts = Vec::with_capacity(anchors.len());
    let mut rowmax_q12 = 0_u64;
    for anchor in anchors {
        let mut receipt = AnchorReceipt {
            name: anchor.name,
            pass: anchor.pass,
            status: "weights_unavailable",
            score_q12: None,
            buckets: None,
        };
        if anchor.weights.is_some() {
            let mut buckets = Vec::with_capacity(BUCKETS);
            let mut q12 = 0_u64;
            for (bucket, &targets) in manifest.bucket_targets.iter().enumerate() {
                let mask = manifest.bucket_masks[bucket];
                let mut max_lane = 0;
                let mut max_sum = 0;
                let mut sums_q32 = std::array::from_fn(|_| None);
                let max_term = u128::from(WEIGHT_ONE) * (u128::from(MASS_ONE) + 1);
                for (e, entry) in sums_q32.iter_mut().enumerate() {
                    let sum = next_word(&mut input)?;
                    if mask & (1 << e) == 0 {
                        require(sum == 0, "CUDA summed an absent lane")?;
                    } else {
                        require(
                            sum <= u128::from(targets) * max_term && sum >= u128::from(targets),
                            "CUDA tangent sum outside integer bounds",
                        )?;
                        *entry = Some(sum.to_string());
                        if sum > max_sum {
                            max_sum = sum;
                            max_lane = e;
                        }
                    }
                }
                let score = u64::try_from(next_word(&mut input)?)?;
                let rowmax = u64::try_from(next_word(&mut input)?)?;
                require(
                    score <= u64::from(targets) * MAX_SCORE_PER_ROW
                        && rowmax <= score
                        && (anchor.name == "pure_kn" || rowmax == 0),
                    "CUDA bucket score outside registry bounds",
                )?;
                q12 = q12.checked_add(score).ok_or("score overflow")?;
                rowmax_q12 = rowmax_q12.checked_add(rowmax).ok_or("rowmax overflow")?;
                buckets.push(Bucket {
                    bucket,
                    targets,
                    mask,
                    max_lane,
                    sums_q32,
                });
            }
            require(
                q12 == anchor.score_q12,
                "CUDA anchor vs. fit score mismatch",
            )?;
            receipt.status = "evaluated";
            receipt.score_q12 = Some(q12);
            receipt.buckets = Some(buckets);
        }
        receipts.push(receipt);
    }
    require(input.is_empty(), "extra CUDA worker output bytes")?;
    Ok((receipts, rowmax_q12))
}

fn main() -> Result<(), Failure> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [frame_dir, verified_path, fit_dir, worker, output] = args.as_slice() else {
        return Err(
            "usage: kn-ceiling-certificate FRAME_DIR VERIFIED_JSON FIT_DIR CUDA_WORKER OUT_DIR"
                .into(),
        );
    };
    let manifest_bytes = fs::read(frame_dir.join("manifest.json"))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    manifest.validate()?;
    let manifest_sha256 = format!("{:x}", Sha256::digest(&manifest_bytes));
    let seal = fs::read_to_string(frame_dir.join("seal.sha256"))?;
    require(
        seal == format!("{manifest_sha256}  manifest.json\n"),
        "frame manifest seal mismatch",
    )?;
    let verified_bytes = fs::read(verified_path)?;
    let verified: Verified = serde_json::from_slice(&verified_bytes)?;
    require(
        verified.schema == "kn-ceiling-independent-verification/1"
            && verified.manifest_sha256 == manifest_sha256
            && verified.frame_sha256 == manifest.frame_sha256
            && verified.target_count == manifest.target_count
            && verified.synthetic_fixture == manifest.synthetic_fixture
            && (verified.verified_query_rows || verified.synthetic_fixture)
            && verified.reading.is_object(),
        "independent frame and fit verification does not bind requested frame",
    )?;
    let fit_bytes = fs::read(fit_dir.join("result.json"))?;
    let fit: Fit = serde_json::from_slice(&fit_bytes)?;
    require(
        fit.schema == "kn-ceiling-fit-result/1"
            && (fit.manifest_sha256.as_deref() == Some(manifest_sha256.as_str())
                || (manifest.synthetic_fixture && fit.manifest_sha256.is_none()))
            && fit.frame_sha256 == manifest.frame_sha256
            && fit.targets == manifest.target_count
            && fit.kn_only.scored.all.targets == manifest.target_count,
        "fit result does not bind requested frame",
    )?;
    let mut pure = [0; LANES];
    pure[0] = WEIGHT_ONE;
    let anchors = [
        Anchor {
            name: "pure_kn",
            pass: 0,
            score_q12: fit.kn_only.scored.all.bits_q12,
            weights: Some(vec![pure; BUCKETS]),
        },
        fit.u_count.anchor("near_kn", &manifest)?,
        fit.u_count_uniform.anchor("uniform", &manifest)?,
    ];
    require(
        verified.reading["kn_only"]["all"]["bits_q12"].as_u64() == Some(anchors[0].score_q12)
            && verified.reading["best_pass"].as_u64() == Some(anchors[1].pass as u64)
            && verified.reading["best_pass_bits_q12"].as_u64() == Some(anchors[1].score_q12)
            && verified.reading["endpoint"].as_u64() == Some(fit.u_count.endpoint as u64)
            && verified.reading["u_count"]["all"]["bits_q12"].as_u64()
                == Some(fit.u_count.scored.all.bits_q12)
            && verified.reading["u_count_uniform"]["all"]["bits_q12"].as_u64()
                == Some(fit.u_count_uniform.scored.all.bits_q12),
        "fit trace and score disagree with independently verified reading",
    )?;
    let worker = fs::canonicalize(worker)?;
    let certificate_binary_sha256 = digest_file(&worker)?;
    let certificate_host_sha256 = digest_file(&std::env::current_exe()?)?;
    fs::create_dir(output)?;
    let rows = stage_rows(&frame_dir.join("frame.bin"), &manifest)?;
    let result = run_worker(&worker, &manifest, &anchors, &rows)?;
    require(
        digest_file(&worker)? == certificate_binary_sha256,
        "CUDA worker SHA-256 changed across execution",
    )?;
    let (certified_anchors, rowmax_q12) = parse_worker(&result, &manifest, &anchors)?;
    let receipt = Receipt {
        schema: "kn-ceiling-device-certificate/1",
        manifest_sha256,
        frame_sha256: manifest.frame_sha256,
        targets: manifest.target_count,
        fit_result_sha256: format!("{:x}", Sha256::digest(&fit_bytes)),
        verified_fit_sha256: format!("{:x}", Sha256::digest(&verified_bytes)),
        certificate_binary_sha256,
        certificate_host_sha256,
        anchors: certified_anchors,
        rowmax_q12,
    };
    let temp = output.join("device-certificate.json.part");
    let mut report = BufWriter::new(File::create_new(&temp)?);
    serde_json::to_writer(&mut report, &receipt)?;
    report.write_all(b"\n")?;
    report.flush()?;
    report.get_ref().sync_all()?;
    drop(report);
    fs::rename(temp, output.join("device-certificate.json"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent finite differential using Rust u128 division. The run uses
    // synthetic mass rows only and is explicitly invoked under the GPU lock.
    #[test]
    #[ignore = "requires a CUDA certificate worker and the Wick GPU lock"]
    fn cuda_tangent_matches_widened_finite_oracle() -> Result<(), Failure> {
        let worker = std::env::var_os("KN_CERT_TEST_WORKER").ok_or("set KN_CERT_TEST_WORKER")?;
        let mut rows = Vec::new();
        let uniform = MASS_ONE / 4096;
        let mut m = [0; LANES];
        m[LANES - 1] = uniform;
        // ML0 sits at its one-unit weight floor in the near-KN anchor,
        // and the pure ML0 vertex is optimal for the entire first bucket.
        // The pure-KN sums force 80-bit terms and cross CUDA CTA boundaries.
        m[0] = 1;
        m[1] = MASS_ONE;
        rows.resize(129, m);
        // Bucket 16 has zero mass on present lanes, distinct row-max lanes
        // and fractional tangent ratios. Bucket 1 is empty.
        m[0] = 3;
        m[1] = MASS_ONE - 1;
        rows.push(m);
        m[0] = MASS_ONE - 1;
        m[1] = 0;
        rows.push(m);

        let mut counts = vec![0; BUCKETS];
        counts[0] = 129;
        counts[16] = 2;
        let manifest = Manifest {
            schema: "kn-ceiling-fit-frame/1".into(),
            row_bytes: ROW_BYTES,
            target_count: rows.len() as u32,
            frame_bytes: (rows.len() * ROW_BYTES) as u64,
            frame_sha256: String::new(),
            bucket_masks: (0..BUCKETS).map(mask).collect(),
            bucket_targets: counts,
            synthetic_fixture: true,
        };
        let mut pure = [0; LANES];
        pure[0] = WEIGHT_ONE;
        let near = (0..BUCKETS)
            .map(|b| {
                let present = mask(b);
                let mut w = std::array::from_fn(|e| u64::from(present & (1 << e) != 0));
                w[0] = WEIGHT_ONE - u64::from(present.count_ones()) + 1;
                w
            })
            .collect();
        let anchors = [
            Anchor {
                name: "pure_kn",
                pass: 0,
                score_q12: 0,
                weights: Some(vec![pure; BUCKETS]),
            },
            Anchor {
                name: "near_kn",
                pass: 0,
                score_q12: 0,
                weights: Some(near),
            },
        ];
        let actual = run_worker(Path::new(&worker), &manifest, &anchors, &rows)?;
        let mut words = actual.as_slice();
        for anchor in anchors {
            let weights = anchor.weights.ok_or("missing test anchor")?;
            let mut start = 0;
            for (b, &n) in manifest.bucket_targets.iter().enumerate() {
                let end = start + n as usize;
                let mut expected = [0_u128; LANES];
                for row in &rows[start..end] {
                    let scaled: u128 = row
                        .iter()
                        .zip(weights[b])
                        .map(|(&p, w)| u128::from(p) * u128::from(w))
                        .sum();
                    let s = (scaled >> 32).max(1);
                    for e in 0..LANES {
                        if mask(b) & (1 << e) != 0 {
                            let numerator = u128::from(row[e] + 1) << 32;
                            expected[e] += numerator.div_ceil(s);
                        }
                    }
                }
                for (lane, sum) in expected.into_iter().enumerate() {
                    assert_eq!(
                        next_word(&mut words)?,
                        sum,
                        "{} bucket {b} lane {lane}",
                        anchor.name
                    );
                }
                let score = next_word(&mut words)?;
                let rowmax = next_word(&mut words)?;
                if n == 0 {
                    assert_eq!((score, rowmax), (0, 0));
                }
                if b == 0 && anchor.name == "pure_kn" {
                    assert_eq!(score, 129 * u128::from(MAX_SCORE_PER_ROW));
                    assert_eq!(rowmax, 0);
                }
                start = end;
            }
        }
        assert!(words.is_empty());
        Ok(())
    }
}
