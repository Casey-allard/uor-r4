//! Prepare only compact sections and original IDs for AOT CUDA. Compare the
//! resulting device words separately against native and independent witnesses.

use memmap2::{Advice, MmapOptions};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use uor_r4_count_runtime::{kn_score::KnDiscounts, ngram_count::PrimeRouteKey, CompactCountLookup};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ORDERS: usize = 11;
const RESULT_BYTES: usize = 32;
const ARTIFACT_MAX: u64 = 4_000_000_000;
const PROBE_MAX: usize = 18;

fn get32(bytes: &[u8]) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes.try_into()?))
}

fn get64(bytes: &[u8]) -> Result<u64> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

struct Case {
    values: Value,
    probes: Vec<Vec<u16>>,
    artifact: [u8; 32],
    train: [u8; 32],
    bytes: u64,
}

impl Case {
    fn open(witness: &Path) -> Result<Self> {
        let values: Value = serde_json::from_reader(File::open(witness)?)?;
        let pin = |name: &str| -> Result<[u8; 32]> {
            let mut pin = [0; 32];
            hex::decode_to_slice(
                values[name].as_str().ok_or("missing witness digest")?,
                &mut pin,
            )?;
            Ok(pin)
        };
        let artifact = pin("candidate_sha256")?;
        let train = pin("train_sha256")?;
        let bytes = values["candidate_bytes"]
            .as_u64()
            .ok_or("missing witness candidate size")?;
        let probes = values["paired_native_lookups"]
            .as_array()
            .ok_or("missing independent probes")?
            .iter()
            .map(|probe| -> Result<Vec<u16>> {
                let tokens: Vec<u16> = serde_json::from_value(probe["tokens"].clone())?;
                if tokens.is_empty()
                    || tokens.len() > ORDERS
                    || tokens.last() == Some(&0)
                    || tokens.iter().any(|&id| id >= 4096)
                {
                    return Err("invalid finite original-ID probe".into());
                }
                Ok(tokens)
            })
            .collect::<Result<Vec<_>>>()?;
        if probes.is_empty() || probes.len() > PROBE_MAX {
            return Err("invalid finite probe count".into());
        }
        Ok(Self {
            values,
            probes,
            artifact,
            train,
            bytes,
        })
    }
}

fn metadata(path: &Path, case: &Case) -> Result<Vec<u8>> {
    let mut file = BufReader::with_capacity(1 << 20, File::open(path)?);
    if file.get_ref().metadata()?.len() != case.bytes || case.bytes > ARTIFACT_MAX {
        return Err("wrong witness compact length".into());
    }
    let mut digest = Sha256::new();
    if io::copy(&mut file, &mut digest)? != case.bytes || digest.finalize()[..] != case.artifact {
        return Err("wrong witness compact identity".into());
    }
    file.seek(SeekFrom::Start(0))?;
    let mut main = [0; 44];
    file.read_exact(&mut main)?;
    if &main[..8] != b"UORKNC02"
        || get32(&main[8..12])? + 1 != ORDERS as u32
        || main[12..] != case.train
    {
        return Err("compact format/order/TRAIN identity mismatch".into());
    }
    let mut at = main.len() as u64;
    let mut out = Vec::with_capacity(ORDERS * 48);
    for k in 0..ORDERS {
        file.seek(SeekFrom::Start(at))?;
        let mut h = [0; 72];
        file.read_exact(&mut h)?;
        let followers = get32(&h[..4])?;
        let contexts = get32(&h[4..8])?;
        let keys = at + h.len() as u64;
        let rows = keys + u64::from(contexts) * ((12u64 * k as u64).div_ceil(8) + 4);
        at = rows + u64::from(followers) * if k + 1 == ORDERS { 6 } else { 10 };
        if at > case.bytes || (contexts == 0) != (followers == 0) {
            return Err("invalid compact section lengths".into());
        }
        let mut adjusted = [0; 4];
        for (count, bytes) in adjusted.iter_mut().zip(h[40..].chunks_exact(8)) {
            *count = get64(bytes)?;
        }
        let (discount, _) = KnDiscounts::with_fallback(adjusted)?;
        out.extend(keys.to_le_bytes());
        out.extend(rows.to_le_bytes());
        out.extend(contexts.to_le_bytes());
        out.extend(followers.to_le_bytes());
        for word in discount.dq {
            out.extend(word.to_le_bytes());
        }
    }
    if at != case.bytes {
        return Err("trailing compact bytes".into());
    }
    Ok(out)
}

fn prepare(case: &Case, artifact: &Path, packet: &Path) -> Result<()> {
    let orders = metadata(artifact, case)?;
    let mut probes = Vec::with_capacity(24 * case.probes.len());
    for tokens in &case.probes {
        probes.extend(u16::try_from(tokens.len())?.to_le_bytes());
        for i in 0..ORDERS {
            probes.extend(tokens.get(i).copied().unwrap_or(0).to_le_bytes());
        }
    }
    fs::create_dir(packet)?;
    for (name, bytes) in [("orders.bin", orders), ("probes.bin", probes)] {
        let mut output = File::options()
            .create_new(true)
            .write(true)
            .open(packet.join(name))?;
        output.write_all(&bytes)?;
        output.sync_all()?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Order {
    raw: u32,
    adjusted: u32,
    raw_total: u64,
    adjusted_total: u64,
    q48: u64,
}

impl Order {
    fn decode(row: &[u8]) -> Result<Self> {
        Ok(Self {
            raw: get32(&row[0..4])?,
            adjusted: get32(&row[4..8])?,
            raw_total: get64(&row[8..16])?,
            adjusted_total: get64(&row[16..24])?,
            q48: get64(&row[24..32])?,
        })
    }
}

fn verify(case: &Case, artifact: &Path, results: &Path) -> Result<()> {
    let file = File::open(artifact)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || case.bytes > ARTIFACT_MAX || metadata.len() != case.bytes {
        return Err("wrong immutable compact length for native comparison".into());
    }
    // SAFETY: The receipt-pinned artifact is a regular file that the caller
    // keeps immutable until replay completes. This mapping is read-only.
    let data = unsafe { MmapOptions::new().map(&file)? };
    data.advise(Advice::Sequential)?;
    let native = CompactCountLookup::from_bytes(&data, case.train, case.artifact)?;
    let result_bytes = case.probes.len() * ORDERS * RESULT_BYTES;
    if fs::metadata(results)?.len() != result_bytes as u64 {
        return Err("CUDA output length mismatch".into());
    }
    let device = fs::read(results)?;
    let mut expected = vec![[Order::default(); ORDERS]; case.probes.len()];
    let mut contexts = BTreeMap::<(u32, u128), Vec<(usize, u16)>>::new();
    let mut retained = 0;
    let mut dropped = 0;
    for (i, tokens) in case.probes.iter().enumerate() {
        let last = tokens.len() - 1;
        let witness = &case.values["paired_native_lookups"][i];
        if witness["order"].as_u64() != Some(last as u64) {
            return Err("witness label does not describe full probe history".into());
        }
        for k in 0..=last {
            let suffix = &tokens[last - k..];
            let key = PrimeRouteKey::from_context(&suffix[..k])?.word();
            contexts
                .entry((k as u32, key))
                .or_default()
                .push((i, tokens[last]));
            let mut counts = [0; ORDERS];
            let mut totals = [0; ORDERS];
            let q48 = native.lookup(suffix, k, &mut counts, &mut totals)?;
            expected[i][k] = Order {
                raw: counts[k],
                raw_total: totals[k],
                q48,
                ..Order::default()
            };
            if k == last {
                let witness_counts: [u32; ORDERS] =
                    serde_json::from_value(witness["candidate_counts"].clone())?;
                let witness_totals: [u64; ORDERS] =
                    serde_json::from_value(witness["candidate_totals"].clone())?;
                if counts != witness_counts
                    || totals != witness_totals
                    || Some(q48) != witness["candidate_q48"].as_u64()
                {
                    return Err(
                        format!("ordinary lookup differs from independent probe {i}").into(),
                    );
                }
                match witness["retained"].as_bool() {
                    Some(true) if totals[k] > 0 && counts[k] > 0 => retained += 1,
                    Some(false) if k > 0 && totals[k] == 0 && expected[i][k - 1].q48 == q48 => {
                        dropped += 1
                    }
                    _ => return Err(format!("invalid retained/dropped control {i}").into()),
                }
            }
        }
    }
    native.visit_counts(|k, key, next, _, adjusted| {
        if let Some(wanted) = contexts.get(&(k, key)) {
            for &(probe, target) in wanted {
                let words = &mut expected[probe][k as usize];
                words.adjusted_total += u64::from(adjusted);
                if next == target {
                    words.adjusted = adjusted;
                }
            }
        }
    })?;
    for (i, bytes) in device.chunks_exact(ORDERS * RESULT_BYTES).enumerate() {
        for (k, serialized) in bytes.chunks_exact(RESULT_BYTES).enumerate() {
            let actual = Order::decode(serialized)?;
            if actual != expected[i][k] {
                return Err(format!(
                    "CUDA probe {i} order {k}: expected {:?}, got {actual:?}",
                    expected[i][k]
                )
                .into());
            }
        }
    }
    let after = file.metadata()?;
    if after.len() != metadata.len() || after.modified()? != metadata.modified()? {
        return Err("compact input changed during native comparison".into());
    }
    println!(
        "{}",
        json!({
            "artifact_sha256": hex::encode(case.artifact),
            "training_sha256": hex::encode(case.train),
            "output_sha256": hex::encode(Sha256::digest(device)),
            "probes": case.probes.len(),
            "retained_controls": retained,
            "dropped_controls": dropped,
            "order_records_compared": case.probes.len() * ORDERS,
        })
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, witness, artifact, dest] = args.as_slice() else {
        return Err("usage: d36-device-witness prepare WITNESS ARTIFACT NEW-PACKET-DIR | verify WITNESS ARTIFACT DEVICE-OUTPUT".into());
    };
    let case = Case::open(Path::new(witness))?;
    match mode.as_str() {
        "prepare" => prepare(&case, Path::new(artifact), Path::new(dest)),
        "verify" => verify(&case, Path::new(artifact), Path::new(dest)),
        _ => Err("unknown witness command".into()),
    }
}
