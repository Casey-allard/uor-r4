//! D87 full-count / frozen-count replay through the D11 trunk loader, P1(a).
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use uor_r4_discrete::ngram_count::{triple_bytes, NgramAssembler, NgramTriple, PrimeRouteKey};
use uor_r4_discrete::ngram_count_device::NgramCounter;
use uor_r4_discrete::ngram_count_oracle::oracle_count;
use uor_r4_integer::discrete_d11::{
    IntegerDiscreteModel, DISCRETE_S2_SCHEMA, DISCRETE_TRUNK_SCHEMA, S2_SECTION,
};

type Fail = Box<dyn std::error::Error>;
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TRAIN: &str = "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sample(path: &Path) -> Result<Vec<u16>, Fail> {
    let mut f = std::io::BufReader::new(File::open(path)?);
    let mut head = [0; 64];
    f.read_exact(&mut head)?;
    if &head[..4] != b"UORT" {
        return Err("missing UORT".into());
    }
    let mut hash = Sha256::new();
    hash.update(head);
    let mut ids = Vec::new();
    let mut records = 0;
    let mut previous = 1;
    let mut total = 0;
    let mut buf = vec![0u8; 1 << 20];
    let mut remaining = f
        .get_ref()
        .metadata()?
        .len()
        .checked_sub(64)
        .ok_or("short stream")?;
    if remaining % 2 != 0 {
        return Err("partial token".into());
    }
    while remaining > 0 {
        let take = remaining.min(buf.len() as u64) as usize;
        f.read_exact(&mut buf[..take])?;
        hash.update(&buf[..take]);
        for pair in buf[..take].chunks_exact(2) {
            let t = u16::from_le_bytes([pair[0], pair[1]]);
            if t >= 4096 || (t == 0) != (previous == 1) {
                return Err("invalid TRAIN record".into());
            }
            if t == 0 {
                records += 1;
            }
            if (records - 1) % 100 == 0 {
                ids.push(t);
            }
            total += 1;
            previous = t;
        }
        remaining -= take as u64;
    }
    if format!("{:x}", hash.finalize()) != TRAIN
        || previous != 1
        || records != 519_580
        || total != 2_091_560_001
    {
        return Err("TRAIN identity, token or record count mismatch".into());
    }
    Ok(ids)
}

fn digest(rows: impl Iterator<Item = NgramTriple>) -> String {
    let mut hash = Sha256::new();
    for row in rows {
        hash.update(triple_bytes(&row));
    }
    format!("{:x}", hash.finalize())
}

// Independent bounded admission: stream one context's successors through four slots.
fn retain_oracle(raw: &[NgramTriple]) -> Vec<NgramTriple> {
    let mut retained = Vec::new();
    for row in raw.chunk_by(|a, b| a.key == b.key) {
        let mut slots = Vec::<NgramTriple>::with_capacity(5);
        for &x in row {
            if x.count == 1 {
                continue;
            }
            let slot = slots
                .partition_point(|y| y.count > x.count || (y.count == x.count && y.next < x.next));
            if slot < 4 {
                slots.insert(slot, x);
                slots.truncate(4);
            }
        }
        slots.sort_unstable_by_key(|x| x.next);
        for mut x in slots {
            x.count = 1 << (31 - x.count.leading_zeros());
            retained.push(x);
        }
    }
    retained
}

// Preserve the exact legacy payload; section offsets are relative to the aligned data start.
fn pack(base: &[u8], pins: &[u8], rows: &[NgramTriple]) -> Result<Vec<u8>, Fail> {
    if hex(&Sha256::digest(base)) != BASE || pins.len() != 96 {
        return Err("base/pin mismatch".into());
    }
    let size = u64::from_le_bytes(base[8..16].try_into()?) as usize;
    let mut header: Value = serde_json::from_slice(&base[16..16 + size])?;
    if header["schema"] != DISCRETE_TRUNK_SCHEMA {
        return Err("expected 19-table trunk".into());
    }
    let payload = &base[(16 + size).next_multiple_of(64)..];
    let offset = payload.len().next_multiple_of(64);
    let bytes = pins.len() + 20 * rows.len();
    header["schema"] = json!(DISCRETE_S2_SCHEMA);
    header["tokenizer_sha256"] = json!(TOKENIZER);
    header["tables"]
        .as_array_mut()
        .ok_or("missing tables")?
        .push(json!({
            "name":S2_SECTION, "kind":"u32", "len":bytes/4, "span":{"offset":offset,"bytes":bytes}
        }));
    let json = serde_json::to_vec(&header)?;
    let mut out = Vec::with_capacity(16 + json.len() + 128 + offset + bytes);
    out.extend_from_slice(&base[..8]);
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    let data_start = out.len().next_multiple_of(64);
    out.resize(data_start, 0);
    out.extend_from_slice(payload);
    out.resize(data_start + offset, 0);
    out.extend_from_slice(pins);
    for row in rows {
        out.extend_from_slice(&row.key.to_le_bytes());
        out.extend_from_slice(&row.next.to_le_bytes());
        out.push(row.count.ilog2() as u8);
        out.push(0);
    }
    if &out[data_start..data_start + payload.len()] != payload {
        return Err("changed base table words".into());
    }
    Ok(out)
}

fn run(tokens: &[u16], base_path: &Path, out: &Path, finite: bool) -> Result<Value, Fail> {
    let mut pins = Vec::new();
    for s in [
        BASE,
        if finite {
            "0000000000000000000000000000000000000000000000000000000000000000"
        } else {
            TRAIN
        },
    ] {
        for pair in s.as_bytes().chunks_exact(2) {
            pins.push(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?);
        }
    }
    let mut slice_sha = Sha256::new();
    for &token in tokens {
        slice_sha.update(token.to_le_bytes());
    }
    pins.extend_from_slice(&slice_sha.finalize());
    let counter = NgramCounter::new(0, tokens)?;
    if finite {
        let key = PrimeRouteKey::from_context(&[4095, 8, 16])?.word();
        let counts = [1, 3, 3, 3, 3, 3, 9, 2];
        let tie: Vec<_> = counts
            .into_iter()
            .enumerate()
            .map(|(i, count)| NgramTriple {
                key,
                next: 10 + i as u16,
                count,
            })
            .collect();
        let expected = [(11, 2), (12, 2), (13, 2), (16, 8)];
        let actual: Vec<_> = counter
            .freeze_s2(&tie)?
            .into_iter()
            .map(|r| (r.next, r.count))
            .collect();
        if actual != expected {
            return Err("CUDA prune/tie/rank fixture failed".into());
        }
    }
    let mut orders = Vec::new();
    let mut declared = Vec::new();
    for k in 0..=10 {
        let mut ass = NgramAssembler::new(k, true)?;
        let plan = counter.count_order(k, 1_000_000, &mut ass)?;
        let (gpu, mut raw, contexts) = ass.finish();
        drop(contexts);
        let kept = counter.freeze_s2(&raw)?;
        raw.clear();
        drop(raw);
        let (cpu, raw_ref, contexts_ref) = oracle_count(tokens, k)?;
        drop(contexts_ref);
        let kept_ref = retain_oracle(&raw_ref);
        if cpu != gpu || kept != kept_ref {
            return Err(format!("order {k}: raw or declared mismatch").into());
        }
        let declared_sha = digest(kept.iter().copied());
        let entry = json!({"order":k,"targets":gpu.targets,"exact_grams":gpu.grams,"contexts":gpu.contexts,
            "exact_sha256":gpu.triple_sha256,"contexts_sha256":gpu.context_sha256,"retained":kept.len(),
            "declared_sha256":declared_sha,"partitions":plan.len()});
        eprintln!("{entry}");
        orders.push(entry);
        declared.extend(kept);
    }
    let base = fs::read(base_path)?;
    let artifact = pack(&base, &pins, &declared)?;
    let mut file = File::options().write(true).create_new(true).open(out)?;
    file.write_all(&artifact)?;
    file.sync_all()?;
    drop(file);
    let size = artifact.len();
    if finite {
        let mut corrupt = artifact.clone();
        let last = corrupt.len() - 20;
        corrupt[last + 18] = 32;
        if IntegerDiscreteModel::from_bytes(&corrupt).is_ok() {
            return Err("malformed rank admitted".into());
        }
        corrupt = artifact.clone();
        corrupt.copy_within(last - 20..last, last);
        if IntegerDiscreteModel::from_bytes(&corrupt).is_ok() {
            return Err("duplicate record admitted".into());
        }
    }
    drop(artifact);
    let loaded = IntegerDiscreteModel::load(out)?;
    let s2 = loaded.frozen_s2().ok_or("missing S2")?;
    if s2.source_digests() != pins {
        return Err("wrong source digests".into());
    }
    let mut n = 0;
    for ((key, next, count), row) in s2.triples().zip(&declared) {
        if (row.key, row.next, row.count) != (key, next, count) || s2.count(key, next) != count {
            return Err("D11 lookup mismatch".into());
        }
        n += 1;
    }
    let word = digest(
        s2.triples()
            .map(|(key, next, count)| NgramTriple { key, next, count }),
    );
    if n != declared.len()
        || s2.triples().len() != n
        || digest(declared.iter().copied()) != word
        || s2.count(0, 0) != 0
    {
        return Err("D11 enumeration, digest or missing-key mismatch".into());
    }
    if finite {
        let ab = PrimeRouteKey::from_context(&[9, 18])?.word();
        let ba = PrimeRouteKey::from_context(&[18, 9])?.word();
        if s2.count(ab, 29) == 0 || s2.count(ba, 29) != 0 {
            return Err("reversed-context control failed".into());
        }
    }
    Ok(
        json!({"scope":if finite {"finite controls"} else {"P1(a) c420 every hundredth TRAIN record"},
        "base_sha256":BASE,"program_sha256":hex(&Sha256::digest(fs::read(std::env::current_exe()?)?)),
        "train_sha256":if finite {Value::Null} else {json!(TRAIN)}, "sample_sha256":hex(&pins[64..]),
        "records":tokens.iter().filter(|&&x|x==0).count(),"tokens":tokens.len(),"orders":orders,
        "declared_rows":n,"declared_sha256":word,"loaded_sha256":loaded.artifact_sha256(),"artifact_bytes":size}),
    )
}

fn main() -> Result<(), Fail> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4 {
        return Err("count-replay fixture|TRAIN BASE NEW-ARTIFACT".into());
    }
    let finite = a[1] == "fixture";
    let tokens = if finite {
        [
            0, 9, 18, 29, 1, 0, 9, 18, 42, 1, 0, 4095, 17, 8, 18, 77, 6, 98, 32, 4000, 9, 97, 1,
        ]
        .repeat(12)
    } else {
        sample(Path::new(&a[1]))?
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&run(&tokens, Path::new(&a[2]), Path::new(&a[3]), finite)?)?
    );
    Ok(())
}
