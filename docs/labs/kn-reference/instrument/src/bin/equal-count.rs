//! Finite equal-byte comparison on the fingerprinted c420 DEV-tune data.
//! Inputs: c420 directory, immutable seed-1 D11, new output JSON.
//! A distinct "fixture" subcommand performs a synthetic CPU/CUDA differential.
#![cfg_attr(not(feature = "cuda"), allow(dead_code, unused_imports))]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uor_r4_discrete::equal_count::{Key, MAX_TOKENS, ROW_BYTES, VOCAB};
use uor_r4_integer::lawful_likelihood::bits_q12;

type Fail = Box<dyn std::error::Error>;
const TRAIN_SHA: &str = "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460";
const TUNE_SHA: &str = "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a";
const MASK_SHA: &str = "cf359a0ec251aef3a27f18e245699f4cb685b97c53e704857258c8629542c6f6";
const BYTE_SHA: &str = "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9";
fn digest(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn verified_prefix(path: &Path, expected: &str, limit: usize) -> Result<Vec<u8>, Fail> {
    let mut f = File::open(path)?;
    let mut sha = Sha256::new();
    let mut buf = vec![0; 1 << 20];
    let mut out = Vec::new();
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha.update(&buf[..n]);
        let keep = n.min(limit.saturating_sub(out.len()));
        out.extend_from_slice(&buf[..keep]);
    }
    if format!("{:x}", sha.finalize()) != expected {
        return Err(format!("checksum mismatch: {}", path.display()).into());
    }
    Ok(out)
}
fn stream(path: &Path, sha: &str, limit: usize) -> Result<Vec<u16>, Fail> {
    let b = verified_prefix(path, sha, 64 + 2 * (limit + 1))?;
    if b.len() < 66 || &b[..4] != b"UORT" {
        return Err("missing UORT header".into());
    }
    let mut ids: Vec<_> = b[64..]
        .chunks_exact(2)
        .map(|x| u16::from_le_bytes([x[0], x[1]]))
        .collect();
    let end = ids
        .iter()
        .rposition(|&t| t == 0)
        .ok_or("missing record boundary")?;
    ids.truncate(end);
    if ids.first() != Some(&0)
        || ids.last() != Some(&1)
        || ids.len() > limit
        || ids.windows(2).any(|p| (p[0] == 1) != (p[1] == 0))
        || ids.iter().any(|&t| usize::from(t) >= VOCAB)
    {
        return Err("bad whole-record V4096 prefix".into());
    }
    Ok(ids)
}

fn novel_from_train(train: &[u16], query: &[u16]) -> Vec<bool> {
    let mut grams = HashSet::<[u16; 13]>::new();
    for record in train.split_inclusive(|&t| t == 1) {
        for w in record.windows(13) {
            let mut key = [0; 13];
            key.copy_from_slice(w);
            grams.insert(key);
        }
    }
    let mut novel = vec![false; query.len()];
    let mut start = 0;
    for (j, &t) in query.iter().enumerate() {
        if t == 0 {
            start = j;
        }
        if j - start >= 13 {
            novel[j] = !grams.contains(&query[j - 13..j]);
        }
    }
    novel
}

#[derive(Default, Clone, Copy)]
struct Total {
    bits: u64,
    bytes: u64,
    targets: u64,
}
impl Total {
    fn add(&mut self, x: Self) {
        self.bits += x.bits;
        self.bytes += x.bytes;
        self.targets += x.targets;
    }
    fn report(self) -> Value {
        json!({"bits_q12":self.bits,"bytes":self.bytes,"targets":self.targets,"bpb_micro":micro(self.bits as i64,self.bytes)})
    }
}
fn micro(bits: i64, bytes: u64) -> Option<i64> {
    (bytes != 0).then(|| ((i128::from(bits) * 1_000_000) / (i128::from(bytes) * 4096)) as i64)
}
type Record = [Total; 3];
fn records(
    query: &[u16],
    mask: &[u8],
    byte_len: &[u8],
    novel: &[bool],
    masses: &[u64],
) -> Result<Vec<Record>, Fail> {
    if query.len() != mask.len()
        || query.len() != novel.len()
        || query.len() != masses.len()
        || byte_len.len() != VOCAB
        || mask.iter().any(|&m| m > 1)
    {
        return Err("target denominator length/label mismatch".into());
    }
    let mut result = Vec::<Record>::new();
    for (i, &id) in query.iter().enumerate() {
        if id == 0 {
            result.push(Default::default());
            continue;
        }
        let step = Total {
            bits: bits_q12(masses[i])?,
            bytes: u64::from(byte_len[usize::from(id)]),
            targets: 1,
        };
        let row = result.last_mut().ok_or("target before BOS")?;
        row[0].add(step);
        if novel[i] {
            row[1].add(step);
            if mask[i] == 1 {
                row[2].add(step);
            }
        }
    }
    Ok(result)
}
fn totals(records: &[Record], side: usize) -> Record {
    let mut out = Record::default();
    for r in records.iter().skip(side).step_by(2) {
        for (dst, x) in out.iter_mut().zip(r) {
            dst.add(*x);
        }
    }
    out
}
fn groups(r: Record) -> Value {
    json!({"all":r[0].report(),"novel":r[1].report(),"response_novel":r[2].report()})
}
fn paired(a: &[Record], b: &[Record]) -> Result<Value, Fail> {
    if a.len() != b.len() {
        return Err("unequal record sets".into());
    }
    let mut groups_out = Vec::new();
    for group in 0..3 {
        let mut deltas = Vec::new();
        for (x, y) in a.iter().zip(b).skip(1).step_by(2) {
            let (x, y) = (x[group], y[group]);
            if (x.targets, x.bytes) != (y.targets, y.bytes) {
                return Err("unequal paired denominators".into());
            }
            deltas.push((x.bits as i64 - y.bits as i64, x.bytes));
        }
        if deltas.is_empty() {
            return Err("no held-out comparison records".into());
        }
        let mut rng = 0xa4093822299f31d0u64;
        let mut estimates = Vec::new();
        for _ in 0..2000 {
            let (mut bits, mut bytes) = (0, 0);
            for _ in &deltas {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let (d, n) = deltas[(rng % deltas.len() as u64) as usize];
                bits += d;
                bytes += n;
            }
            if let Some(v) = micro(bits, bytes) {
                estimates.push(v);
            }
        }
        estimates.sort_unstable();
        let interval = (!estimates.is_empty()).then(|| {
            [
                estimates[estimates.len() * 25 / 1000],
                estimates[estimates.len() * 975 / 1000],
            ]
        });
        let x = totals(a, 1)[group];
        let y = totals(b, 1)[group];
        groups_out.push(json!({"delta_micro_bpb":micro(x.bits as i64-y.bits as i64,x.bytes),
            "paired_prompt_95_interval_micro_bpb":interval,"at_least_one_percent_fewer_bits":x.bits*100<=y.bits*99}));
    }
    Ok(json!({"all":groups_out[0],"novel":groups_out[1],"response_novel":groups_out[2]}))
}

#[cfg(feature = "cuda")]
fn run(dir: &Path, artifact: &Path, out: &Path) -> Result<(), Fail> {
    use uor_r4_discrete::equal_count_device::{compare_arm, kernel_hash};
    use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
    let model = IntegerDiscreteModel::load(artifact)?;
    if model.artifact_sha256() != "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d"
    {
        return Err("model differs from registered seed 1".into());
    }
    let roots = model.packed_codebook_roots();
    let train = stream(&dir.join("train.tokens.u16"), TRAIN_SHA, MAX_TOKENS)?;
    let query = stream(&dir.join("dev-tune.tokens.u16"), TUNE_SHA, MAX_TOKENS / 2)?;
    let mask = verified_prefix(&dir.join("dev-tune.mask.u8"), MASK_SHA, query.len())?;
    let lengths = verified_prefix(&dir.join("id-bytes.u8"), BYTE_SHA, VOCAB)?;
    let novel = novel_from_train(&train, &query);
    let mut reports = Vec::new();
    let mut graded = Vec::new();
    let keys = [Key::Transport, Key::Reseeded, Key::SplitMix]
        .into_iter()
        .chain((1..=12).map(Key::Ordered));
    for key in keys {
        let (arm, k) = key.words()?;
        let name = match key {
            Key::Transport => "transport".into(),
            Key::Reseeded => "reseeded".into(),
            Key::SplitMix => "splitmix".into(),
            Key::Ordered(n) => format!("suffix-{n}"),
        };
        let (bytes, masses) = compare_arm(&train, &query, &roots, key)?;
        let rows = records(&query, &mask, &lengths, &novel, &masses)?;
        let mut mh = Sha256::new();
        for &m in &masses {
            mh.update(m.to_le_bytes());
        }
        let mut rh = Sha256::new();
        for r in &rows {
            for x in r {
                for w in [x.bits, x.bytes, x.targets] {
                    rh.update(w.to_le_bytes());
                }
            }
        }
        let metadata = key.metadata(&roots)?.len();
        let table = &bytes[32 + metadata + VOCAB * 4..];
        let mut occupied = 0;
        let mut retained = 0u64;
        for row in table.chunks_exact(ROW_BYTES) {
            let mut total = 0u64;
            for entry in row.chunks_exact(8) {
                total += u64::from(u32::from_le_bytes(entry[4..8].try_into()?));
            }
            occupied += usize::from(total > 0);
            retained += total;
        }
        let per_record: Vec<_> = rows
            .iter()
            .map(|r| r.map(|x| [x.bits, x.bytes, x.targets]))
            .collect();
        reports.push(json!({
            "name":name, "arm":arm, "k":k,
            "frozen_image_sha256":digest(&bytes), "frozen_bytes":bytes.len(),
            "header_bytes":32, "key_bytes":metadata, "unigram_bytes":VOCAB*4,
            "row_bytes":ROW_BYTES, "rows":table.len()/ROW_BYTES,
            "index":"implicit direct row number, 0 extra bytes",
            "occupied_rows":occupied, "retained_train_events":retained,
            "masses_sha256":format!("{:x}",mh.finalize()),
            "record_tallies_sha256":format!("{:x}",rh.finalize()),
            "selection":groups(totals(&rows,0)), "heldout":groups(totals(&rows,1)),
            "record_tallies":per_record
        }));
        graded.push(rows);
    }
    let best = (3..graded.len())
        .min_by_key(|&i| totals(&graded[i], 0)[0].bits)
        .ok_or("no token context")?;
    let result = json!({"scope":"bounded c420 DEV-tune address diagnostic","model_sha256":model.artifact_sha256(),"kernel_sha256":kernel_hash(),
        "pins":{"train":TRAIN_SHA,"dev_tune":TUNE_SHA,"mask":MASK_SHA,"id_bytes":BYTE_SHA},
        "train_tokens":train.len(),"query_tokens":query.len(),"train_records":train.iter().filter(|&&t| t==0).count(),
        "heldout_records":graded[0].len()/2,"selection_records":graded[0].len().div_ceil(2),
        "selected_suffix_k":best-2,"candidate_minus_control":{"best_tuned_suffix":paired(&graded[0],&graded[best])?,
            "reseeded":paired(&graded[0],&graded[1])?,"splitmix":paired(&graded[0],&graded[2])?},"arms":reports});
    let mut result = result;
    result["record_tallies_schema"] = json!({
        "order":"original whole-record prefix order; even selects k; odd comparison",
        "groups":["all","novel","response_novel"],
        "fields":["bits_q12","bytes","targets"]
    });
    let payload = serde_json::to_vec(&result)?;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out)?;
    f.write_all(&payload)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn main() -> Result<(), Fail> {
    #[cfg(feature = "cuda")]
    {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() == 1 && args[0] == "fixture" {
            return parity_fixture();
        }
        if let [inputs, model, out] = args.as_slice() {
            return run(Path::new(inputs), Path::new(model), Path::new(out));
        }
    }
    Err("equal-count requires --features cuda and either fixture or INPUTS MODEL OUT".into())
}

// Independent, fixture-only arithmetic: no corpus arguments; this control
// never executes CPU training or evaluation on research data.
#[cfg(feature = "cuda")]
fn parity_fixture() -> Result<(), Fail> {
    use std::collections::BTreeMap;
    use uor_r4_discrete::equal_count::{image, row_count};
    use uor_r4_discrete::equal_count_device::compare_arm;
    use uor_r4_discrete::position_mechanism::compose_frames;
    fn mix(mut n: u64) -> u64 {
        n = (n ^ (n >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        n = (n ^ (n >> 27)).wrapping_mul(0x94d049bb133111eb);
        n ^ (n >> 31)
    }
    let roots: Vec<u32> = (0..VOCAB as u32)
        .map(|x| x.wrapping_mul(1297) & 65535)
        .collect();
    let mut train = Vec::new();
    let mut query = Vec::new();
    for r in 0..64u16 {
        train.push(0);
        query.push(0);
        for i in 0..20u16 {
            train.push(2 + (i + r * 3) % 57);
            query.push(2 + ((19 - i) * 3 + r) % 57);
        }
        train.push(1);
        query.push(1);
    }
    let mut distinct = HashSet::new();
    for key in [
        Key::Transport,
        Key::Reseeded,
        Key::Ordered(1),
        Key::Ordered(12),
        Key::SplitMix,
    ] {
        let (gpu, masses) = compare_arm(&train, &query, &roots, key)?;
        let meta = key.metadata(&roots)?;
        let rows = row_count(meta.len())?;
        let (_, k) = key.words()?;
        let address = |prefix: &[u16]| -> Result<usize, Fail> {
            let prefix = &prefix[prefix.len().saturating_sub(k as usize)..];
            let key_word = match key {
                Key::Transport | Key::Reseeded => {
                    let mut f = [0u8; 2];
                    for &id in prefix {
                        let n = roots[usize::from(id)];
                        for (b, frame) in f.iter_mut().enumerate() {
                            let root = ((n >> (4 * b)) & 15) as usize;
                            *frame = compose_frames(meta[8192 + b * 16 + root], *frame)?;
                        }
                    }
                    u64::from(f[0]) * 64 + u64::from(f[1])
                }
                Key::Ordered(_) | Key::SplitMix => {
                    let h = prefix.iter().fold(1u64, |h, &t| {
                        if matches!(key, Key::Ordered(_)) {
                            h.wrapping_mul(4099).wrapping_add(u64::from(t) + 1)
                        } else {
                            mix(h
                                .wrapping_add(0x9e3779b97f4a7c15)
                                .wrapping_add(u64::from(t)))
                        }
                    });
                    h ^ ((prefix.len() as u64) << 56)
                }
            };
            Ok((mix(key_word) % rows as u64) as usize)
        };
        let mut counts: BTreeMap<usize, BTreeMap<u16, u32>> = BTreeMap::new();
        let mut uni = vec![0u32; VOCAB];
        let mut targets = 0;
        for r in train.split_inclusive(|&t| t == 1) {
            for j in 1..r.len() {
                *counts
                    .entry(address(&r[..j])?)
                    .or_default()
                    .entry(r[j])
                    .or_default() += 1;
                uni[usize::from(r[j])] += 1;
                targets += 1;
            }
        }
        let mut words = vec![0; rows * 8];
        for (row, counts) in counts {
            let mut candidates: Vec<_> = counts.into_iter().collect();
            candidates.sort_by_key(|&(id, n)| (std::cmp::Reverse(n), id));
            for (i, (id, n)) in candidates.into_iter().take(4).enumerate() {
                words[row * 8 + 2 * i] = u32::from(id);
                words[row * 8 + 2 * i + 1] = n;
            }
        }
        let expected = image(key, &meta, &uni, &words, targets)?;
        if expected != gpu {
            return Err("finite CPU/GPU frozen count image disagreement".into());
        }
        let mut first = 0;
        for r in query.split_inclusive(|&t| t == 1) {
            for j in 1..r.len() {
                let p = usize::from(r[j]);
                let row = address(&r[..j])?;
                let mut hit = 0;
                let mut total = 0;
                for entry in words[row * 8..row * 8 + 8].chunks_exact(2) {
                    if entry[0] as usize == p {
                        hit += u64::from(entry[1]);
                    }
                    total += u64::from(entry[1]);
                }
                let global = ((u64::from(uni[p]) + 1) * (1u64 << 32)) / (u64::from(targets) + 4096);
                let q = (hit * (1u64 << 32))
                    .checked_div(total)
                    .map_or(global, |local| (global + local) / 2);
                if masses[first + j] != (q << 16) {
                    return Err("finite CPU/GPU target mass disagreement".into());
                }
            }
            first += r.len();
        }
        // Exercise all emittable IDs on an occupied and an empty row.
        // The unscored BOS has known zero TRAIN count; include its analytic
        // floor mass so the normalization check covers all 4096 vocabulary IDs.
        if key == Key::Ordered(1) {
            for (suffix, occupied) in [(2u16, true), (3000u16, false)] {
                let prefix = [0, suffix];
                let row = address(&prefix)?;
                let counts = &words[8 * row..8 * row + 8];
                if (counts[1] != 0) != occupied {
                    return Err("bad normalization fixture row".into());
                }
                let mut exhaustive = Vec::with_capacity(3 * (VOCAB - 1));
                for t in 1..VOCAB as u16 {
                    exhaustive.extend([0, suffix, t]);
                }
                let (_, all) = compare_arm(&train, &exhaustive, &roots, key)?;
                let bos = (1u64 << 32) / (u64::from(targets) + VOCAB as u64);
                let mut sum = if occupied { bos / 2 } else { bos };
                for (i, t) in (1..VOCAB).enumerate() {
                    let mass = all[3 * i + 2] >> 16;
                    let base =
                        ((u64::from(uni[t]) + 1) << 32) / (u64::from(targets) + VOCAB as u64);
                    if mass == 0 || (!occupied && mass != base) {
                        return Err("zero support or wrong empty-row fallback".into());
                    }
                    sum += mass;
                }
                if sum > (1u64 << 32) || (1u64 << 32) - sum > 2 * VOCAB as u64 {
                    return Err("Q32 support normalization outside roundoff bound".into());
                }
                println!(
                    "fixture normalized occupied={occupied}, Q32 deficit {}",
                    (1u64 << 32) - sum
                );
            }
        }
        let a = address(&[0, 7, 13, 19, 31])?;
        let b = address(&[0, 31, 19, 13, 7])?;
        if key == Key::Transport && a == b {
            return Err("fixture order swap failed to move transport".into());
        }
        distinct.insert(digest(
            &words
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect::<Vec<_>>(),
        ));
        println!(
            "fixture {:?}: {} bytes, {} target masses equal",
            key.words()?,
            gpu.len(),
            query.len() - 64
        );
    }
    if distinct.len() != 5 {
        return Err("fixture fails to distinguish its key controls".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn novelty_requires_thirteen_within_record_and_train_only() {
        let mut train = vec![0];
        train.extend(2..16);
        train.push(1);
        let mut query = train.clone();
        query.extend([0, 20, 21, 1]);
        query.extend([0, 20, 21, 1]);
        let flags = novel_from_train(&train, &query);
        assert!(flags.iter().all(|&f| !f));
        query[7] = 100;
        let flags = novel_from_train(&train, &query);
        assert!(flags[13..16].iter().all(|&f| f));
        assert!(flags[..13].iter().all(|&f| !f));
        assert!(flags[16..].iter().all(|&f| !f));
    }
    #[test]
    fn tallies_differentiate_all_novel_and_response_novel() -> Result<(), Fail> {
        let q = [0, 2, 3, 1];
        let mut len = vec![0; VOCAB];
        len[2] = 3;
        len[3] = 2;
        let r = records(
            &q,
            &[0, 0, 1, 1],
            &len,
            &[false, true, true, false],
            &[0, 1 << 47, 1 << 47, 1 << 47],
        )?;
        assert_eq!(
            (r[0][0].targets, r[0][1].targets, r[0][2].targets),
            (3, 2, 1)
        );
        assert_eq!((r[0][0].bytes, r[0][1].bytes, r[0][2].bytes), (5, 5, 2));
        assert!(records(&q, &[0, 0, 3, 1], &len, &[false; 4], &[0; 4]).is_err());
        Ok(())
    }
}
