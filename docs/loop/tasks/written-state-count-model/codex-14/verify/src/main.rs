#![forbid(unsafe_code)]

mod history;
mod result;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

type Failure = Box<dyn std::error::Error>;

const LANES: usize = 13;
const BUCKETS: usize = 176;
const ROW_BYTES: usize = 109;
const MASS_ONE: u64 = 1 << 48;
const WEIGHT_ONE: u64 = 1 << 32;
const UNIFORM: u64 = MASS_ONE / 4096;
const PRIMARY: [u64; 6] = [24, 32, 40, 48, 56, 64];
const ALL_MASK: u16 = (1 << LANES) - 1;
const RESTRICTED_MASK: u16 = 1 | (1 << (LANES - 1));

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
struct Tally {
    targets: u64,
    bytes: u64,
    bits_q12: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
struct Score {
    all: Tally,
    novel: Tally,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: String,
    row_bytes: usize,
    target_count: u64,
    frame_bytes: u64,
    frame_sha256: String,
    kn_mass_sha256: String,
    raw_lookup_sha256: String,
    source_sha256: String,
    binary_sha256: String,
    inputs: Value,
    highest_order: u64,
    budget: u64,
    per_mille: u64,
    orders: Value,
    raw_orders: Value,
    mode_config: Value,
    lane_order: Vec<String>,
    bucket_masks: Vec<u16>,
    bucket_targets: Vec<u64>,
    bucket_novel_targets: Vec<u64>,
    parity_targets: [u64; 2],
    all_bytes: u64,
    novel_bytes: u64,
    novel_targets: u64,
    kn_mass_floor_targets: u64,
    synthetic_fixture: bool,
    cuda_images: Value,
}

#[derive(Clone, Copy)]
struct Mode {
    key: &'static str,
    start: Option<&'static str>,
    enabled: u16,
    fit: Option<usize>,
    score: Option<usize>,
}

const MODES: [Mode; 8] = [
    Mode {
        key: "kn_only",
        start: None,
        enabled: 1,
        fit: None,
        score: None,
    },
    Mode {
        key: "u_count",
        start: Some("near_kn"),
        enabled: ALL_MASK,
        fit: None,
        score: None,
    },
    Mode {
        key: "u_count_uniform",
        start: Some("uniform"),
        enabled: ALL_MASK,
        fit: None,
        score: None,
    },
    Mode {
        key: "kn_u",
        start: Some("near_kn"),
        enabled: RESTRICTED_MASK,
        fit: None,
        score: None,
    },
    Mode {
        key: "kn_u_uniform",
        start: Some("uniform"),
        enabled: RESTRICTED_MASK,
        fit: None,
        score: None,
    },
    Mode {
        key: "split_even_fit",
        start: Some("uniform"),
        enabled: ALL_MASK,
        fit: Some(0),
        score: Some(1),
    },
    Mode {
        key: "split_odd_fit",
        start: Some("uniform"),
        enabled: ALL_MASK,
        fit: Some(1),
        score: Some(0),
    },
    Mode {
        key: "u_split",
        start: None,
        enabled: ALL_MASK,
        fit: None,
        score: None,
    },
];

fn ensure(condition: bool, label: impl Into<String>) -> Result<(), Failure> {
    if !condition {
        return Err(label.into().into());
    }
    Ok(())
}

fn array(value: &Value) -> Result<&[Value], Failure> {
    Ok(value.as_array().ok_or("expected JSON array")?)
}

fn number(value: &Value) -> Result<u64, Failure> {
    match value {
        Value::String(text) => Ok(text.parse()?),
        _ => Ok(value.as_u64().ok_or("expected unsigned integer")?),
    }
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest_file(path: &Path) -> Result<String, Failure> {
    let mut hasher = Sha256::new();
    io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn mask(bucket: usize) -> u16 {
    ((1 << (bucket / 16 + 2)) - 1) | (1 << (LANES - 1))
}

fn mode_config(mode: &Mode) -> Value {
    let primary = mode.key == "u_count";
    let checkpoints: &[u64] = if primary {
        &PRIMARY
    } else if mode.start.is_some() {
        &[24]
    } else {
        &[]
    };
    serde_json::json!({
        "name": mode.key, "start": mode.start, "enabled_mask": mode.enabled,
        "fit_parity": mode.fit, "score_parity": mode.score, "checkpoints": checkpoints,
        "plateau_window": primary.then_some(8),
        "plateau_numerator": primary.then_some(4096),
        "plateau_denominator": primary.then_some(10_000),
    })
}

impl Manifest {
    fn validate(&self, source: &str, binary: &str, history: Option<&Value>) -> Result<(), Failure> {
        ensure(
            self.schema == "kn-ceiling-fit-frame/1" && self.row_bytes == ROW_BYTES,
            "frame schema/width",
        )?;
        ensure(
            self.target_count > 0 && self.target_count < 1 << 24,
            "registered target-count bound",
        )?;
        ensure(
            self.frame_bytes == self.target_count * ROW_BYTES as u64,
            "frame bytes vs rows",
        )?;
        ensure(
            self.highest_order == 10 && self.budget == 16_777_216,
            "registered N and budget",
        )?;
        ensure((1..=1000).contains(&self.per_mille), "per-mille cut range")?;
        for field in [
            "train_sha256",
            "query_sha256",
            "id_bytes_sha256",
            "classes_sha256",
        ] {
            ensure(
                self.inputs[field].as_str().is_some_and(valid_sha),
                format!("input digest {field}"),
            )?;
        }
        for name in ["count", "lookup", "kn_score"] {
            let image = array(&self.cuda_images[name])?;
            ensure(
                image.len() == 3 && image.iter().all(|x| x.as_str().is_some_and(valid_sha)),
                format!("{name} source, PTX and cubin digests"),
            )?;
        }
        for digest in [
            &self.frame_sha256,
            &self.kn_mass_sha256,
            &self.raw_lookup_sha256,
            &self.source_sha256,
            &self.binary_sha256,
        ] {
            ensure(valid_sha(digest), "malformed manifest digest")?;
        }
        ensure(
            self.source_sha256 == source,
            "collector source digest mismatch",
        )?;
        ensure(
            self.binary_sha256 == binary,
            "collector binary digest mismatch",
        )?;
        let lane_order: Vec<String> = std::iter::once("KN".to_owned())
            .chain((0..=10).map(|order| format!("ML{order}")))
            .chain(std::iter::once("uniform".to_owned()))
            .collect();
        ensure(self.lane_order == lane_order, "lane order")?;
        ensure(
            self.bucket_masks.iter().copied().eq((0..BUCKETS).map(mask)),
            "ordered bucket masks",
        )?;
        ensure(
            self.bucket_targets.len() == BUCKETS && self.bucket_novel_targets.len() == BUCKETS,
            "bucket census width",
        )?;
        ensure(
            array(&self.mode_config)?
                .iter()
                .cloned()
                .eq(MODES.iter().map(mode_config)),
            "registered ordered mode configurations",
        )?;
        for (label, orders) in [("adjusted", &self.orders), ("raw", &self.raw_orders)] {
            let orders = array(orders)?;
            ensure(orders.len() == 11, format!("{label} count order width"))?;
            for (n, order) in orders.iter().enumerate() {
                ensure(
                    number(&order["order"])? == n as u64,
                    format!("{label} count order {n}"),
                )?;
                for field in ["triple_sha256", "context_sha256"] {
                    ensure(
                        order[field].as_str().is_some_and(valid_sha),
                        format!("{label} order {n} {field}"),
                    )?;
                }
            }
        }
        for field in ["triple_sha256", "context_sha256"] {
            ensure(
                self.orders[10][field] == self.raw_orders[10][field],
                format!("top raw/adjusted {field}"),
            )?;
        }
        if let Some(history) = history {
            ensure(
                self.inputs == history["inputs"],
                "registered inputs or record-boundary cut changed",
            )?;
            ensure(
                history["masses_sha256"].as_str() == Some(&self.kn_mass_sha256),
                "historical KN mass digest",
            )?;
            for (n, order) in array(&self.orders)?.iter().enumerate() {
                for field in ["triple_sha256", "context_sha256"] {
                    ensure(
                        order[field] == history["orders"][n][field],
                        format!("historical adjusted order {n} {field}"),
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct Census {
    total: Score,
    by_parity: [Score; 2],
    bucket_targets: Vec<[u64; 2]>,
    bucket_novel: Vec<u64>,
    kn_at_floor: u64,
}

fn add_target(score: &mut Score, novel: usize, bytes: u64) {
    score.all.targets += 1;
    score.all.bytes += bytes;
    if novel == 1 {
        score.novel.targets += 1;
        score.novel.bytes += bytes;
    }
}

struct Query {
    tokens: Vec<u16>,
    classes: Vec<u8>,
    id_bytes: Vec<u8>,
    end: usize,
    position: usize,
    parity: usize,
}

impl Query {
    fn open(directory: &Path, manifest: &Manifest) -> Result<Self, Failure> {
        let read = |file: &str, hash: &str| -> Result<Vec<u8>, Failure> {
            let bytes = fs::read(directory.join(file))?;
            ensure(
                manifest.inputs[hash].as_str() == Some(&sha(&bytes)),
                format!("{file} digest"),
            )?;
            Ok(bytes)
        };
        let query = read("dev-tune.tokens.u16", "query_sha256")?;
        let classes = read("dev-tune.novel.u8", "classes_sha256")?;
        let id_bytes = read("id-bytes.u8", "id_bytes_sha256")?;
        ensure(
            query.len() >= 64 && &query[..4] == b"UORT",
            "query UORT header",
        )?;
        let declared = u64::from_le_bytes(query[8..16].try_into()?);
        let payload = &query[64..];
        ensure(
            declared == classes.len() as u64
                && declared.checked_mul(2) == Some(payload.len() as u64)
                && id_bytes.len() == 4096,
            "query input widths",
        )?;
        let tokens: Vec<_> = payload
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        let end = usize::try_from(number(&manifest.inputs["query_tokens"])?)?;
        ensure(
            end > 0 && end <= tokens.len() && tokens[0] == 0 && tokens[end - 1] == 1,
            "query records must be complete",
        )?;
        let expected_end = if manifest.per_mille == 1000 {
            tokens.len()
        } else {
            let mark = u64::try_from(tokens.len())? * manifest.per_mille / 1000;
            tokens[..=usize::try_from(mark)?]
                .iter()
                .rposition(|&token| token == 0)
                .ok_or("query cut has no BOS")?
        };
        ensure(end == expected_end, "query record-boundary cut")?;
        Ok(Self {
            tokens,
            classes,
            id_bytes,
            end,
            position: 0,
            parity: 0,
        })
    }

    fn next(&mut self) -> Result<Option<[u8; 3]>, Failure> {
        while self.position < self.end {
            let token = usize::from(self.tokens[self.position]);
            let class = self.classes[self.position];
            self.position += 1;
            if token >= 4096 || class > 3 || (token == 0) != (class == 3) {
                return Err(format!("query token/class at {}", self.position - 1).into());
            }
            if token == 0 {
                self.parity ^= 1;
                continue;
            }
            return Ok(Some([
                u8::from(class == 1),
                self.id_bytes[token],
                self.parity as u8,
            ]));
        }
        Ok(None)
    }
}

fn decode_row(row: &[u8; ROW_BYTES]) -> Result<(usize, usize, u64, usize, [u64; LANES]), Failure> {
    let bucket = usize::from(u16::from_le_bytes([row[0], row[1]]));
    let novel = usize::from(row[2]);
    let parity = usize::from(row[4]);
    ensure(
        bucket < BUCKETS && novel < 2 && parity < 2,
        "row bucket, novel or parity out of range",
    )?;

    let mut masses = [0; LANES];
    for (lane, bytes) in row[5..].chunks_exact(8).enumerate() {
        masses[lane] = u64::from_le_bytes(bytes.try_into()?);
    }
    ensure(
        masses[0] > 0 && masses[LANES - 1] == UNIFORM,
        "row KN or uniform mass",
    )?;
    ensure(
        masses.iter().all(|&mass| mass <= MASS_ONE),
        "row mass above Q48 one",
    )?;
    ensure(
        masses
            .iter()
            .enumerate()
            .all(|(lane, &mass)| mask(bucket) & (1 << lane) != 0 || mass == 0),
        "absent lane has mass",
    )?;
    Ok((bucket, novel, u64::from(row[3]), parity, masses))
}

fn scan_frame(
    mut reader: impl Read,
    manifest: &Manifest,
    mut query: Option<Query>,
) -> Result<Census, Failure> {
    let mut payload = Sha256::new();
    let mut kn = Sha256::new();
    let mut census = Census {
        bucket_targets: vec![[0, 0]; BUCKETS],
        bucket_novel: vec![0; BUCKETS],
        ..Census::default()
    };
    let mut row = [0u8; ROW_BYTES];
    for index in 0..manifest.target_count {
        reader
            .read_exact(&mut row)
            .map_err(|e| format!("frame row {index}: {e}"))?;
        let (bucket, novel, bytes, parity, masses) =
            decode_row(&row).map_err(|e| format!("frame row {index}: {e}"))?;
        if let Some(query) = &mut query {
            let expected: [u8; 3] = row[2..5].try_into()?;
            if query.next()? != Some(expected) {
                return Err(
                    format!("frame row {index} query class, byte credit or record parity").into(),
                );
            }
        }
        payload.update(row);
        kn.update(&row[5..13]);
        census.kn_at_floor += u64::from(masses[0] == 1);
        census.bucket_targets[bucket][parity] += 1;
        census.bucket_novel[bucket] += novel as u64;
        add_target(&mut census.total, novel, bytes);
        add_target(&mut census.by_parity[parity], novel, bytes);
    }
    ensure(reader.read(&mut row[..1])? == 0, "trailing frame bytes")?;
    if let Some(query) = &mut query {
        ensure(
            query.next()?.is_none(),
            "frame ended before the query's final target",
        )?;
    }
    ensure(
        format!("{:x}", payload.finalize()) == manifest.frame_sha256,
        "packed-frame digest mismatch",
    )?;
    ensure(
        format!("{:x}", kn.finalize()) == manifest.kn_mass_sha256,
        "packed-frame KN digest mismatch",
    )?;
    ensure(
        census
            .bucket_targets
            .iter()
            .map(|counts| counts[0] + counts[1])
            .eq(manifest.bucket_targets.iter().copied()),
        "packed-frame bucket census",
    )?;
    ensure(
        census.bucket_novel == manifest.bucket_novel_targets,
        "packed-frame novel bucket census",
    )?;
    ensure(
        census.by_parity.map(|score| score.all.targets) == manifest.parity_targets,
        "packed-frame parity census",
    )?;
    ensure(
        census.kn_at_floor == manifest.kn_mass_floor_targets,
        "packed-frame KN floor census",
    )?;
    ensure(
        census.total.all.bytes == manifest.all_bytes
            && census.total.novel.bytes == manifest.novel_bytes
            && census.total.novel.targets == manifest.novel_targets,
        "packed-frame all/novel denominators",
    )?;
    Ok(census)
}

fn run() -> Result<Value, Failure> {
    let usage = "usage: kn-ceiling-verify FRAME-DIR SOURCE-SHA256 COLLECTOR-BIN [--query INPUT-DIR] [--fit FIT-DIR FITTER-BIN] [--fixture]";
    let mut args = std::env::args_os().skip(1);
    let dir = PathBuf::from(args.next().ok_or(usage)?);
    let source = args.next().ok_or(usage)?.into_string().map_err(|_| usage)?;
    let binary = digest_file(Path::new(&args.next().ok_or(usage)?))?;
    let mut fixture = false;
    let mut query_dir = None;
    let mut fit = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--query") if query_dir.is_none() => {
                query_dir = Some(PathBuf::from(args.next().ok_or(usage)?))
            }
            Some("--fit") if fit.is_none() => {
                fit = Some((
                    PathBuf::from(args.next().ok_or(usage)?),
                    PathBuf::from(args.next().ok_or(usage)?),
                ));
            }
            Some("--fixture") if !fixture => fixture = true,
            _ => return Err(usage.into()),
        }
    }

    let manifest_bytes = fs::read(dir.join("manifest.json"))?;
    let seal = sha(&manifest_bytes);
    ensure(
        fs::read(dir.join("seal.sha256"))? == format!("{seal}  manifest.json\n").as_bytes(),
        "manifest seal digest or line format",
    )?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    let manifest_value: Value = serde_json::from_slice(&manifest_bytes)?;
    let mut canonical = serde_json::to_vec_pretty(&manifest_value)?;
    canonical.push(b'\n');
    ensure(
        manifest_bytes == canonical,
        "noncanonical sealed manifest JSON",
    )?;
    ensure(
        manifest.synthetic_fixture == fixture,
        "use --fixture for synthetic frames only",
    )?;
    let historical = (!fixture)
        .then(|| history::read(manifest.per_mille))
        .transpose()?;
    manifest.validate(&source, &binary, historical.as_ref())?;
    let query = query_dir
        .as_deref()
        .map(|path| Query::open(path, &manifest))
        .transpose()?;
    let file = File::open(dir.join("frame.bin"))?;
    ensure(
        file.metadata()?.len() == manifest.frame_bytes,
        "packed frame length",
    )?;
    let census = scan_frame(
        BufReader::with_capacity(1024 * 1024, file),
        &manifest,
        query,
    )?;
    if let Some(history) = &historical {
        let old = history::score(history)?;
        ensure(
            census.total.all.targets == old.all.targets
                && census.total.all.bytes == old.all.bytes
                && census.total.novel.targets == old.novel.targets
                && census.total.novel.bytes == old.novel.bytes,
            "historical all/novel denominators",
        )?;
    }
    let reading = fit
        .as_ref()
        .map(|(dir, fitter)| {
            result::verify(dir, fitter, &manifest, &seal, &census, historical.as_ref())
        })
        .transpose()?;
    Ok(serde_json::json!({
        "schema": "kn-ceiling-independent-verification/1",
        "manifest_sha256": seal,
        "frame_sha256": manifest.frame_sha256,
        "kn_mass_sha256": manifest.kn_mass_sha256,
        "raw_lookup_sha256": manifest.raw_lookup_sha256,
        "source_sha256": manifest.source_sha256,
        "collector_sha256": binary,
        "per_mille": manifest.per_mille,
        "target_count": manifest.target_count,
        "novel_target_count": manifest.novel_targets,
        "parity_targets": manifest.parity_targets,
        "verified_query_rows": query_dir.is_some(),
        "historical_frame_match": historical.is_some(),
        "historical_fit_controls_match": historical.is_some() && reading.is_some(),
        "synthetic_fixture": fixture,
        "reading": reading,
    }))
}

fn main() -> Result<(), Failure> {
    let output = run()?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(bucket: u8, novel: u8, parity: u8) -> [u8; ROW_BYTES] {
        let mut row = [0u8; ROW_BYTES];
        row[..5].copy_from_slice(&[bucket, 0, novel, 3, parity]);
        row[5..13].copy_from_slice(&(MASS_ONE / 2).to_le_bytes());
        row[101..].copy_from_slice(&UNIFORM.to_le_bytes());
        row
    }

    #[test]
    fn rejects_bad_class_parity_bucket_and_masses_even_before_hash_checks() {
        let clean = row(0, 1, 1);
        assert!(decode_row(&clean).is_ok());
        for (index, value) in [(0, 176), (2, 2), (4, 2), (10, 0), (21, 1), (105, 0)] {
            let mut damaged = clean;
            damaged[index] = value;
            assert!(
                decode_row(&damaged).is_err(),
                "accepted changed byte {index}"
            );
        }
        let mut above_one = clean;
        above_one[13..21].copy_from_slice(&(MASS_ONE + 1).to_le_bytes());
        assert!(decode_row(&above_one).is_err());
    }

    #[test]
    fn packed_frame_requires_exact_rows_hash_and_query_alignment() {
        let packed_rows = [row(15, 1, 1), row(31, 0, 0)];
        let packed = packed_rows.concat();
        let mut kn = Sha256::new();
        let mut bucket_targets = vec![0; BUCKETS];
        let mut bucket_novel = vec![0; BUCKETS];
        for row in &packed_rows {
            kn.update(&row[5..13]);
            bucket_targets[usize::from(row[0])] += 1;
            bucket_novel[usize::from(row[0])] += u64::from(row[2]);
        }
        let mut manifest: Manifest = serde_json::from_value(serde_json::json!({
            "schema": "kn-ceiling-fit-frame/1", "row_bytes": ROW_BYTES, "target_count": 2,
            "frame_bytes": packed.len(), "frame_sha256": sha(&packed),
            "kn_mass_sha256": format!("{:x}", kn.finalize()),
            "raw_lookup_sha256": "", "source_sha256": "", "binary_sha256": "",
            "inputs": null, "highest_order": 10, "budget": 16_777_216, "per_mille": 1000,
            "orders": null, "raw_orders": null, "mode_config": null, "lane_order": [],
            "bucket_masks": [], "bucket_targets": bucket_targets, "bucket_novel_targets": bucket_novel,
            "parity_targets": [1, 1], "all_bytes": 6, "novel_bytes": 3, "novel_targets": 1,
            "kn_mass_floor_targets": 0, "synthetic_fixture": true, "cuda_images": null,
        })).unwrap();
        let query = |classes| Query {
            tokens: vec![0, 1, 0, 1],
            classes,
            id_bytes: vec![0, 3],
            end: 4,
            position: 0,
            parity: 0,
        };
        let ok_query = || Some(query(vec![3, 1, 3, 2]));
        assert!(scan_frame(packed.as_slice(), &manifest, ok_query()).is_ok());
        assert!(scan_frame(&packed[..packed.len() - 1], &manifest, None).is_err());
        let mut extra = packed.clone();
        extra.push(0);
        assert!(scan_frame(extra.as_slice(), &manifest, None).is_err());

        let mut swapped = [packed_rows[1], packed_rows[0]].concat();
        assert!(scan_frame(swapped.as_slice(), &manifest, None).is_err());
        manifest.frame_sha256 = sha(&swapped);
        assert!(scan_frame(swapped.as_slice(), &manifest, ok_query()).is_err());
        swapped.copy_from_slice(&packed);
        manifest.frame_sha256 = sha(&packed);
        assert!(scan_frame(swapped.as_slice(), &manifest, Some(query(vec![3, 2, 3, 1]))).is_err());
        manifest.parity_targets = [2, 0];
        assert!(scan_frame(packed.as_slice(), &manifest, None).is_err());
    }

    #[test]
    fn historical_all_slices_reopen_and_match_plain_kn() {
        for per_mille in [1, 10, 100, 1000] {
            let history = history::read(per_mille).unwrap();
            assert_eq!(array(&history["orders"]).unwrap().len(), 11);
            for trace in [
                &history["ceiling"]["u_count"]["fit_bits_q12"],
                &history["ceiling"]["kn_u"]["fit_bits_q12"],
                &history["ceiling"]["u_split"]["fit_bits_q12_even"],
                &history["ceiling"]["u_split"]["fit_bits_q12_odd"],
            ] {
                assert_eq!(array(trace).unwrap().len(), 25);
            }
        }
    }
}
