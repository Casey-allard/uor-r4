//! Frozen KN-only mixture for a finite native client transport witness.
#![forbid(unsafe_code)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};
use uor_r4_count_runtime::{CompactCountLookup, FrozenPickMixture};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const C128: &str = "1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6";
const C1: &str = "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030";
const FULL: &str = "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a";
const TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = fs::File::create_new(path)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [counts, expected, probes, output] = args.as_slice() else {
        return Err(
            "usage: d36-count-fixture COUNTS COUNT_SHA256 INDEPENDENT_PROBE_REPORT NEW_DIR".into(),
        );
    };
    let verified: Value = serde_json::from_reader(fs::File::open(probes)?)?;
    let train = verified["train_sha256"]
        .as_str()
        .ok_or("missing TRAIN identity")?;
    let ref_field = if verified["candidate_sha256"] == expected.as_str() {
        "candidate_q48"
    } else if train == C128 && verified["source_sha256"] == expected.as_str() {
        "source_q48"
    } else {
        return Err("unrelated probe report".into());
    };
    let cap = match train {
        C128 => 16_459_406,
        C1 => 268_435_456,
        FULL => 3_912_936_331,
        _ => return Err("unknown TRAIN identity".into()),
    };
    let file_bytes = fs::metadata(counts)?.len();
    if file_bytes > cap
        || (ref_field == "candidate_q48"
            && verified["candidate_bytes"].as_u64() != Some(file_bytes))
    {
        return Err("compact size outside the audited fixture bound".into());
    }
    let bytes = fs::read(counts)?;
    let table = CompactCountLookup::from_bytes(
        &bytes,
        hex::decode(train)?
            .try_into()
            .map_err(|_| "training digest")?,
        hex::decode(expected)?
            .try_into()
            .map_err(|_| "artifact digest")?,
    )?;
    let mut witnesses = Vec::new();
    for p in verified["paired_native_lookups"]
        .as_array()
        .ok_or("missing probes")?
    {
        let order = p["order"].as_u64().ok_or("missing order")?;
        if order < 4 {
            continue;
        }
        let mut tape: Vec<u16> = serde_json::from_value(p["tokens"].clone())?;
        let at = tape.len().checked_sub(1).ok_or("empty probe")?;
        if at == 0
            || tape[..at].contains(&1)
            || tape[..at]
                .iter()
                .rposition(|&id| id == 0)
                .is_some_and(|pos| pos != 0)
        {
            continue;
        }
        let probe_target = tape[at];
        let mut expected_at_probe = None;
        let mut hash = Sha256::new();
        let mut winner = 0;
        let mut max = 0;
        let mut ties = 0;
        for id in 1..4096 {
            tape[at] = id;
            let mass = table.lookup(&tape, at, &mut [0; 11], &mut [0; 11])?;
            if id == probe_target {
                expected_at_probe = Some(mass)
            }
            if mass > max {
                winner = id;
                max = mass;
                ties = 1;
            } else if mass == max {
                ties += 1;
            }
            hash.update(mass.to_le_bytes());
        }
        if expected_at_probe != p[ref_field].as_u64() {
            return Err("independent lookup mass differs".into());
        }
        witnesses.push(json!({"order":order,"retained":p["retained"],"context":&tape[..at],"token":winner,"mass_q48":max,"top_ties":ties,"mass_sha256":hex::encode(hash.finalize())}));
    }
    if ![true, false]
        .iter()
        .all(|&v| witnesses.iter().any(|w| w["retained"] == v))
    {
        return Err("missing kept or omitted fixture".into());
    }
    let out = Path::new(output);
    fs::create_dir(out)?;
    let fit = serde_json::to_vec(
        &json!({"schema":"uor-r4.d31.a1-fit/1","fixture":"finite integer transport with unit KN mass in every bucket; no fitting","prepared":{"format":"uor-r4.d31.a1-preparation/1","c_sha256":train,"trunk_sha256":TRUNK,"compact_sha256":expected}}),
    )?;
    let mut q32 = vec![0; 528 * 14];
    for bucket in q32.chunks_exact_mut(14) {
        bucket[0] = 1u64 << 32;
    }
    let weights = FrozenPickMixture::new(10, Sha256::digest(&fit).into(), q32)?.to_bytes();
    write_new(&out.join("fit.json"), &fit)?;
    write_new(&out.join("weights.bin"), &weights)?;
    write_new(
        &out.join("witnesses.json"),
        &serde_json::to_vec_pretty(&witnesses)?,
    )?;
    Ok(())
}
