//! Packed evidence tape for the GPU fit. The seal commits the manifest, which
//! commits the payload. The publication order makes an interrupted write
//! distinguishable from a reopenable frame.

use crate::{fail, record_parity, sha256_hex, Failure, CLASS_BOS, CLASS_NOVEL, KN_ONE, NGRAM_BOS};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, DirBuilder, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use uor_r4_discrete::kn_ceiling::{ceiling_expert_row, BUCKETS_PER_ORDER};
use uor_r4_discrete::kn_score::KN_BASE_MASS;

const SCHEMA: &str = "kn-ceiling-fit-frame/1";
pub const HIGHEST: u32 = 10;
pub const BUDGET: u64 = 16_777_216;
const LOOKUPS: usize = HIGHEST as usize + 1;
const LANES: usize = LOOKUPS + 2;
const BUCKETS: usize = LOOKUPS * BUCKETS_PER_ORDER;
const ROW_BYTES: usize = 5 + 8 * LANES;
const ALL_LANES: u16 = (1 << LANES) - 1;
const KN_U_LANES: u16 = (1 << (LANES - 1)) | 1;
const IO_BUFFER: usize = 1 << 20;
const FRAME: &str = "frame.bin";
const MANIFEST: &str = "manifest.json";
const SEAL: &str = "seal.sha256";

const INPUT_HASHES: [(&str, &str); 4] = [
    (
        "train_sha256",
        "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460",
    ),
    (
        "query_sha256",
        "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a",
    ),
    (
        "id_bytes_sha256",
        "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
    ),
    (
        "classes_sha256",
        "7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af",
    ),
];

pub fn hash_word(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn check_inputs(inputs: &Value, synthetic: bool) -> Result<(), Failure> {
    for (key, expected) in INPUT_HASHES {
        let actual = inputs[key]
            .as_str()
            .filter(|hash| hash_word(hash))
            .ok_or_else(|| fail(format!("missing or invalid {key}")))?;
        if actual == expected && synthetic && key != "id_bytes_sha256" {
            return Err(fail(format!(
                "protected {key} cannot be used as a synthetic fixture"
            )));
        }
        if actual != expected && !synthetic {
            return Err(fail(format!(
                "{key} differs from the frozen DEV-tune identity"
            )));
        }
    }
    Ok(())
}

fn lane_order() -> Value {
    let mut names = vec!["KN".to_owned()];
    names.extend((0..LOOKUPS).map(|i| format!("ML{i}")));
    names.push("uniform".to_owned());
    json!(names)
}

fn bucket_mask(bucket: usize) -> u16 {
    ((1 << (bucket / BUCKETS_PER_ORDER + 2)) - 1) | (1 << (LANES - 1))
}

fn bucket_masks() -> Value {
    json!((0..BUCKETS).map(bucket_mask).collect::<Vec<_>>())
}

fn mode_config() -> Value {
    let mode = |name: &str, start: Option<&str>, mask: u16, parity: (Option<u8>, Option<u8>)| {
        let primary = name == "u_count";
        let checkpoints: &[u8] = if primary {
            &[24, 32, 40, 48, 56, 64]
        } else if start.is_some() {
            &[24]
        } else {
            &[]
        };
        json!({
            "name": name, "start": start, "enabled_mask": mask,
            "fit_parity": parity.0, "score_parity": parity.1,
            "checkpoints": checkpoints, "plateau_window": primary.then_some(8),
            "plateau_numerator": primary.then_some(4096),
            "plateau_denominator": primary.then_some(10000),
        })
    };
    let all = (None, None);
    json!([
        mode("kn_only", None, 1, all),
        mode("u_count", Some("near_kn"), ALL_LANES, all),
        mode("u_count_uniform", Some("uniform"), ALL_LANES, all),
        mode("kn_u", Some("near_kn"), KN_U_LANES, all),
        mode("kn_u_uniform", Some("uniform"), KN_U_LANES, all),
        mode(
            "split_even_fit",
            Some("uniform"),
            ALL_LANES,
            (Some(0), Some(1))
        ),
        mode(
            "split_odd_fit",
            Some("uniform"),
            ALL_LANES,
            (Some(1), Some(0))
        ),
        mode("u_split", None, ALL_LANES, all),
    ])
}

struct Census {
    bucket_targets: [u64; BUCKETS],
    bucket_novel_targets: [u64; BUCKETS],
    parity_targets: [u64; 2],
    all_bytes: u64,
    novel_bytes: u64,
    novel_targets: u64,
    kn_mass_floor_targets: u64,
}

impl Default for Census {
    fn default() -> Self {
        Self {
            bucket_targets: [0; BUCKETS],
            bucket_novel_targets: [0; BUCKETS],
            parity_targets: [0; 2],
            all_bytes: 0,
            novel_bytes: 0,
            novel_targets: 0,
            kn_mass_floor_targets: 0,
        }
    }
}

impl Census {
    fn add(&mut self, bucket: usize, novel: bool, bytes: u8, parity: usize, kn: u64) {
        self.bucket_targets[bucket] += 1;
        self.parity_targets[parity] += 1;
        self.all_bytes += u64::from(bytes);
        self.kn_mass_floor_targets += u64::from(kn == 1);
        if novel {
            self.bucket_novel_targets[bucket] += 1;
            self.novel_targets += 1;
            self.novel_bytes += u64::from(bytes);
        }
    }

    fn json(&self) -> Value {
        json!({
            "bucket_targets": self.bucket_targets.as_slice(),
            "bucket_novel_targets": self.bucket_novel_targets.as_slice(),
            "parity_targets": self.parity_targets,
            "all_bytes": self.all_bytes,
            "novel_bytes": self.novel_bytes,
            "novel_targets": self.novel_targets,
            "kn_mass_floor_targets": self.kn_mass_floor_targets,
        })
    }
}

pub struct Rows<'a> {
    pub query: &'a [u16],
    pub id_bytes: &'a [u8],
    pub classes: &'a [u8],
    pub kn: &'a [u64],
    pub counts: &'a [u32],
    pub totals: &'a [u64],
}

fn check_full_census(manifest: &Value) -> Result<(), Failure> {
    if manifest["synthetic_fixture"] == false
        && manifest["per_mille"] == 1000
        && (manifest["inputs"]["train_tokens"] != 2_091_560_001_u64
            || manifest["inputs"]["query_tokens"] != 10_810_087_u64
            || manifest["target_count"] != 10_807_417_u64
            || manifest["all_bytes"] != 21_928_475_u64
            || manifest["novel_targets"] != 7_566_174_u64
            || manifest["novel_bytes"] != 15_995_822_u64
            || manifest["kn_mass_sha256"]
                != "ae8518d552bc42d8f1c3602680bd77586d0cd9c237962fbf5e2b36704b6bb2fd")
    {
        return Err(fail(
            "full frame differs from the frozen KN mass or target/byte census",
        ));
    }
    Ok(())
}

fn frame_bytes(count: u64) -> Result<u64, Failure> {
    count
        .checked_mul(ROW_BYTES as u64)
        .filter(|&length| length != 0)
        .ok_or_else(|| fail("invalid frame size"))
}

fn private_dir(path: &Path) -> Result<(), Failure> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    Ok(())
}

fn publish(path: &Path, name: &str, bytes: &[u8]) -> Result<(), Failure> {
    let temporary = path.join(format!("{name}.tmp"));
    let mut file = File::create_new(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path.join(name))?;
    Ok(())
}

fn seal_line(manifest_bytes: &[u8]) -> String {
    format!("{}  {MANIFEST}\n", sha256_hex(manifest_bytes))
}

pub fn write(path: &Path, rows: Rows<'_>, mut manifest: Value) -> Result<Value, Failure> {
    let target_count = u64::try_from(rows.kn.len())?;
    let length = frame_bytes(target_count)?;
    let want = rows.kn.len().checked_mul(LOOKUPS);
    let parity = record_parity(rows.query);
    if rows.query.len() != rows.classes.len()
        || parity.len() != rows.kn.len()
        || want != Some(rows.counts.len())
        || want != Some(rows.totals.len())
    {
        return Err(fail("incomplete frame target, lookup or class data"));
    }

    private_dir(path)?;
    let temporary = path.join("frame.bin.tmp");
    let mut writer = BufWriter::with_capacity(IO_BUFFER, File::create_new(&temporary)?);
    let mut frame_hash = Sha256::new();
    let mut raw_hash = Sha256::new();
    let mut kn_hash = Sha256::new();
    let mut census = Census::default();
    let mut t = 0;
    for (j, (&token, &class)) in rows.query.iter().zip(rows.classes).enumerate() {
        if class > CLASS_BOS || (token == NGRAM_BOS) != (class == CLASS_BOS) {
            return Err(fail(format!(
                "position {j}: token {token} has class {class}"
            )));
        }
        if token == NGRAM_BOS {
            continue;
        }
        let lookup = t * LOOKUPS..(t + 1) * LOOKUPS;
        let (counts, totals) = (&rows.counts[lookup.clone()], &rows.totals[lookup]);
        let kn = rows.kn[t];
        if !(1..=KN_ONE).contains(&kn) {
            return Err(fail(format!("row {t} has an invalid KN mass")));
        }
        if counts
            .iter()
            .zip(totals)
            .any(|(&count, &total)| u64::from(count) > total)
        {
            return Err(fail(format!(
                "row {t} has a raw count above its context total"
            )));
        }
        let mut masses = [0; LANES];
        let bucket = ceiling_expert_row(kn, counts, totals, &mut masses)?;
        let novel = class == CLASS_NOVEL;
        let bytes = *rows.id_bytes.get(usize::from(token)).ok_or_else(|| {
            fail(format!(
                "position {j}: missing byte credit for token {token}"
            ))
        })?;
        let mut packed = [0; ROW_BYTES];
        packed[..2].copy_from_slice(&bucket.to_le_bytes());
        packed[2..5].copy_from_slice(&[u8::from(novel), bytes, u8::from(parity[t])]);
        for (lane_bytes, mass) in packed[5..].chunks_exact_mut(8).zip(masses) {
            lane_bytes.copy_from_slice(&mass.to_le_bytes());
        }
        writer.write_all(&packed)?;
        frame_hash.update(packed);
        kn_hash.update(rows.kn[t].to_le_bytes());
        let mut raw = [0; 12 * LOOKUPS];
        for ((count, total), pair) in counts.iter().zip(totals).zip(raw.chunks_exact_mut(12)) {
            pair[..4].copy_from_slice(&count.to_le_bytes());
            pair[4..].copy_from_slice(&total.to_le_bytes());
        }
        raw_hash.update(raw);
        census.add(
            usize::from(bucket),
            novel,
            bytes,
            usize::from(parity[t]),
            rows.kn[t],
        );
        t += 1;
    }
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    fs::rename(temporary, path.join(FRAME))?;
    let data = json!({
        "schema": SCHEMA, "row_bytes": ROW_BYTES, "target_count": target_count,
        "frame_bytes": length, "frame_sha256": format!("{:x}", frame_hash.finalize()),
        "lane_order": lane_order(), "bucket_masks": bucket_masks(),
        "mode_config": mode_config(), "raw_lookup_sha256": format!("{:x}", raw_hash.finalize()),
        "kn_mass_sha256": format!("{:x}", kn_hash.finalize()),
    });
    let fields = manifest
        .as_object_mut()
        .ok_or_else(|| fail("frame metadata must be an object"))?;
    for derived in [data, census.json()] {
        if let Value::Object(derived) = derived {
            fields.extend(derived);
        }
    }
    check_full_census(&manifest)?;
    let mut canonical = serde_json::to_vec_pretty(&manifest)?;
    canonical.push(b'\n');
    publish(path, MANIFEST, &canonical)?;
    File::open(path)?.sync_all()?;
    publish(path, SEAL, seal_line(&canonical).as_bytes())?;
    File::open(path)?.sync_all()?;
    Ok(manifest)
}

fn read_word(bytes: &[u8]) -> Result<u64, Failure> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

pub fn verify(path: &Path) -> Result<Value, Failure> {
    let canonical = fs::read(path.join(MANIFEST))?;
    if fs::read_to_string(path.join(SEAL))? != seal_line(&canonical) {
        return Err(fail("manifest does not match its seal"));
    }
    let manifest: Value = serde_json::from_slice(&canonical)?;
    let mut serialized = serde_json::to_vec_pretty(&manifest)?;
    serialized.push(b'\n');
    if serialized != canonical {
        return Err(fail("manifest JSON is not canonical"));
    }
    if manifest["schema"] != SCHEMA
        || manifest["row_bytes"] != ROW_BYTES
        || manifest["highest_order"] != HIGHEST
        || manifest["budget"] != BUDGET
        || manifest["lane_order"] != lane_order()
        || manifest["bucket_masks"] != bucket_masks()
        || manifest["mode_config"] != mode_config()
    {
        return Err(fail("frame schema or fit configuration differs"));
    }
    if !matches!(manifest["per_mille"].as_u64(), Some(1..=1000)) {
        return Err(fail("invalid frame cut"));
    }
    for key in [
        "source_sha256",
        "binary_sha256",
        "kn_mass_sha256",
        "raw_lookup_sha256",
        "frame_sha256",
    ] {
        if !manifest[key].as_str().is_some_and(hash_word) {
            return Err(fail(format!("missing or invalid {key}")));
        }
    }
    let synthetic = manifest["synthetic_fixture"]
        .as_bool()
        .ok_or_else(|| fail("missing fixture flag"))?;
    check_inputs(&manifest["inputs"], synthetic)?;
    check_full_census(&manifest)?;
    let count = manifest["target_count"]
        .as_u64()
        .ok_or_else(|| fail("invalid target count"))?;
    let length = frame_bytes(count)?;
    let file = File::open(path.join(FRAME))?;
    if manifest["frame_bytes"] != length || file.metadata()?.len() != length {
        return Err(fail("frame length differs from its sealed target count"));
    }

    let mut reader = BufReader::with_capacity(IO_BUFFER, file);
    let mut hash = Sha256::new();
    let mut kn_hash = Sha256::new();
    let mut census = Census::default();
    let mut packed = [0; ROW_BYTES];
    for t in 0..count {
        reader.read_exact(&mut packed)?;
        let bucket = usize::from(u16::from_le_bytes([packed[0], packed[1]]));
        if bucket >= BUCKETS || packed[2] > 1 || packed[4] > 1 {
            return Err(fail(format!("row {t} has invalid bucket, class or parity")));
        }
        let mask = bucket_mask(bucket);
        for (lane, word) in packed[5..].chunks_exact(8).enumerate() {
            let mass = read_word(word)?;
            if mass > KN_ONE
                || (lane == 0 && mass == 0)
                || (lane == LANES - 1 && mass != KN_BASE_MASS)
                || (mask & (1 << lane) == 0 && mass != 0)
            {
                return Err(fail(format!("row {t} has invalid lane {lane} mass")));
            }
        }
        let kn = read_word(&packed[5..13])?;
        hash.update(packed);
        kn_hash.update(&packed[5..13]);
        census.add(
            bucket,
            packed[2] == 1,
            packed[3],
            usize::from(packed[4]),
            kn,
        );
    }
    if reader.read(&mut [0])? != 0 {
        return Err(fail("unexpected bytes after final target"));
    }
    if manifest["frame_sha256"] != format!("{:x}", hash.finalize())
        || manifest["kn_mass_sha256"] != format!("{:x}", kn_hash.finalize())
    {
        return Err(fail("frame or KN mass digest differs"));
    }
    if let Value::Object(derived) = census.json() {
        for (key, actual) in derived {
            if manifest[&key] != actual {
                return Err(fail(format!("manifest {key} differs from the packed rows")));
            }
        }
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("kn-fit-frame-{name}-{}", std::process::id()))
    }

    fn synthetic_metadata() -> Value {
        json!({
            "highest_order": HIGHEST, "budget": BUDGET, "per_mille": 1000,
            "synthetic_fixture": true, "source_sha256": "a".repeat(64),
            "binary_sha256": "b".repeat(64),
            "inputs": {
                "train_sha256": "0".repeat(64), "query_sha256": "1".repeat(64),
                "id_bytes_sha256": "2".repeat(64), "classes_sha256": "3".repeat(64),
            },
        })
    }

    #[test]
    fn packed_columns_and_sealed_reopen_follow_the_direct_collection() {
        let path = scratch("cols");
        let mut counts = [0; 4 * LOOKUPS];
        let mut totals = counts.map(u64::from);
        for t in 0..4 {
            counts[t * LOOKUPS] = 2;
            totals[t * LOOKUPS] = 8;
        }
        counts[LOOKUPS - 1] = 1;
        totals[LOOKUPS - 1] = 4;
        let kn = [1, 2, 3, KN_ONE];
        let rows = Rows {
            query: &[NGRAM_BOS, 2, 1, NGRAM_BOS, 3, 1],
            id_bytes: &[0, 0, 11, 99],
            classes: &[CLASS_BOS, CLASS_NOVEL, 2, CLASS_BOS, 0, CLASS_NOVEL],
            kn: &kn,
            counts: &counts,
            totals: &totals,
        };
        let manifest = write(&path, rows, synthetic_metadata()).unwrap();
        assert_eq!(manifest, verify(&path).unwrap());
        assert_eq!(manifest["all_bytes"], 110);
        assert_eq!(manifest["novel_bytes"], 11);
        assert_eq!(manifest["novel_targets"], 2);
        assert_eq!(manifest["parity_targets"], json!([2, 2]));
        assert_eq!(manifest["kn_mass_floor_targets"], 1);

        let payload = fs::read(path.join(FRAME)).unwrap();
        let packed: Vec<_> = payload.chunks_exact(ROW_BYTES).collect();
        assert_eq!(&packed[0][..5], &[162, 0, 1, 11, 1]);
        assert_eq!(&packed[1][..5], &[3, 0, 0, 0, 1]);
        assert_eq!(&packed[2][..5], &[3, 0, 0, 99, 0]);
        assert_eq!(&packed[3][..5], &[3, 0, 1, 0, 0]);
        let table =
            uor_r4_discrete::kn_ceiling::CeilingTable::from_lookups(HIGHEST, &kn, &counts, &totals)
                .unwrap();
        for (t, row) in packed.iter().enumerate() {
            assert_eq!(table.buckets()[t].to_le_bytes(), row[..2]);
            for (lane, word) in row[5..].chunks_exact(8).enumerate() {
                assert_eq!(table.masses()[t * LANES + lane].to_le_bytes(), word);
            }
        }

        let mut invalid_payload = payload.clone();
        invalid_payload[ROW_BYTES + 5 + 2 * 8] = 1; // absent ML1 in bucket 3
        fs::write(path.join(FRAME), &invalid_payload).unwrap();
        assert!(verify(&path).unwrap_err().to_string().contains("lane 2"));
        fs::write(path.join(FRAME), &payload[..payload.len() - 1]).unwrap();
        assert!(verify(&path).unwrap_err().to_string().contains("length"));
        fs::write(path.join(FRAME), &payload).unwrap();
        fs::write(path.join(SEAL), "missing hash").unwrap();
        assert!(verify(&path).unwrap_err().to_string().contains("seal"));
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn incomplete_target_rows_never_publish_a_seal() {
        let path = scratch("incomplete");
        let rows = Rows {
            query: &[NGRAM_BOS, 2, 1],
            id_bytes: &[0, 0, 11],
            classes: &[CLASS_BOS, CLASS_NOVEL, 2],
            kn: &[1],
            counts: &[0; LOOKUPS],
            totals: &[3; LOOKUPS],
        };
        assert!(write(&path, rows, synthetic_metadata()).is_err());
        assert!(!path.exists());
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn cuda_fixture_count_lookup_and_cold_reopen() {
        fn uort(tokens: &[u16]) -> Vec<u8> {
            let mut bytes = vec![0; 64];
            bytes[..4].copy_from_slice(b"UORT");
            bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
            bytes.extend(tokens.iter().flat_map(|&token| token.to_le_bytes()));
            bytes
        }

        let parent = scratch("cuda-reopen");
        private_dir(&parent).unwrap();
        let base_record = [0, 42, 2, 3, 16, 5, 6, 7, 8, 9, 10, 1];
        let other_record = [0, 43, 2, 3, 16, 5, 6, 7, 8, 9, 10, 1];
        let train = [base_record, base_record, other_record].concat();
        let query = [base_record, other_record, base_record].concat();
        let mut bytes = [1; 4096];
        bytes[1] = 0;
        bytes[42] = 2;
        bytes[43] = 3;
        let classes: Vec<_> = query
            .iter()
            .map(|&id| match id {
                NGRAM_BOS => CLASS_BOS,
                42 | 43 => CLASS_NOVEL,
                _ => 2,
            })
            .collect();
        let paths = ["train", "query", "bytes", "classes"].map(|name| parent.join(name));
        fs::write(&paths[0], uort(&train)).unwrap();
        fs::write(&paths[1], uort(&query)).unwrap();
        fs::write(&paths[2], bytes).unwrap();
        fs::write(&paths[3], classes).unwrap();
        for name in ["direct", "recollect"] {
            let out = parent.join(name);
            let mut args: Vec<_> = paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            args.extend([
                HIGHEST.to_string(),
                BUDGET.to_string(),
                out.to_string_lossy().into_owned(),
                "--fit-frame".to_owned(),
                "--source-sha256".to_owned(),
                "a".repeat(64),
                "--synthetic-fixture".to_owned(),
                "--check-host".to_owned(),
            ]);
            crate::run(&args).unwrap();
        }
        let direct = verify(&parent.join("direct")).unwrap();
        let recollected = verify(&parent.join("recollect")).unwrap();
        assert_eq!(direct, recollected);
        assert_eq!(direct["parity_targets"], json!([11, 22]));
        assert_eq!(direct["all_bytes"], 34);
        assert_eq!(direct["novel_bytes"], 7);
        assert_eq!(direct["novel_targets"], 3);
        // Target 42 occurs twice in 33 raw targets, and twice after the
        // three BOS positions. Adjusted continuation counts differ here.
        let payload = fs::read(parent.join("direct").join(FRAME)).unwrap();
        assert_eq!(&payload[..5], &[17, 0, 1, 2, 1]);
        assert_eq!(read_word(&payload[13..21]).unwrap(), (2 * KN_ONE) / 33);
        assert_eq!(read_word(&payload[21..29]).unwrap(), (2 * KN_ONE) / 3);
        // Keep the sealed directories and input fixture for standalone fit/replay.
        eprintln!("cold CUDA fixture at {}", parent.display());
    }
}
