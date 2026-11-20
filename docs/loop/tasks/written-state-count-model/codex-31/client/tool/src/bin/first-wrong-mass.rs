//! Post-score inspection of the sealed CUDA client candidate tapes.
#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

type Failure = Box<dyn std::error::Error>;
const N_IDS: usize = 4095;
const N_BUCKETS: usize = 528;
const LANES: usize = 14;
const BUCKET_HEADER: usize = 44;
const ROW_HEADER: usize = 16;
const ROW_BYTES: usize = (LANES + 1) * 8;
const KEY_SHA: &str = "95d556eaf457b2f517123f01b5b3b5a39784e947ce106033119fe97a195238b5";
const PANEL_SHA: &str = "e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11";

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn word(bytes: &[u8], at: usize) -> Result<u64, Failure> {
    let bytes = bytes
        .get(at..at + 8)
        .ok_or_else(|| fail("short native word"))?;
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

fn integer(row: &Value, field: &str) -> Result<usize, Failure> {
    Ok(row[field]
        .as_u64()
        .ok_or_else(|| fail(format!("missing integer field {field}")))?
        .try_into()?)
}

fn components(
    row_bytes: &[u8],
    weights: &[u8],
    id: usize,
    q48: u64,
    full: bool,
) -> Result<Value, Failure> {
    let start = ROW_HEADER + (id - 1) * ROW_BYTES;
    let bucket = usize::try_from(word(row_bytes, start)?)?;
    if bucket >= N_BUCKETS {
        return Err(fail("candidate bucket is outside the fitted state"));
    }
    let offset = BUCKET_HEADER + bucket * LANES * 8;
    let mut mixture = 0u128;
    let mut pick = (0, 0);
    let mut weight_total = 0u64;
    let mut lane_floors = [0u64; LANES];
    for lane in 0..LANES {
        let weight = word(weights, offset + lane * 8)?;
        let mass = word(row_bytes, start + (lane + 1) * 8)?;
        if weight > 1 << 32 || mass > 1 << 48 {
            return Err(fail("candidate exceeds fitted native word bounds"));
        }
        let weighted = u128::from(weight) * u128::from(mass);
        mixture += weighted;
        lane_floors[lane] = u64::try_from(weighted >> 32)?;
        weight_total += weight;
        if lane + 1 == LANES {
            pick = (weight, mass);
        }
    }
    let without_pick = ((mixture - u128::from(pick.0) * u128::from(pick.1)) >> 32).max(1);
    if weight_total != 1 << 32 || u128::from(q48) != (mixture >> 32).max(1) {
        return Err(fail(
            "actual GPU mass differs from exact fitted word reconstruction",
        ));
    }
    let mut result = json!({"bucket":bucket,"longest_active_order":(bucket % 176) / 16,
        "pick_weight_q32":pick.0,"pick_expert_mass_q48":pick.1,
        "without_pick_same_bucket_q48":u64::try_from(without_pick)?,
        "pick_contribution_q48":q48 - u64::try_from(without_pick)?});
    if full {
        result["weighted_lane_floors_q48"] = json!(lane_floors);
        result["rounding_carry_and_floor_q48"] = json!(q48 - lane_floors.iter().sum::<u64>());
    }
    Ok(result)
}

fn inspect(panel: &Path, step: &Value, gold: usize, weights: &[u8]) -> Result<Value, Failure> {
    let id = integer(step, "id")?;
    let index = integer(step, "step")?;
    let read = step["read_enabled"]
        .as_bool()
        .ok_or_else(|| fail("missing read setting"))?;
    let actual = integer(step, "token")?;
    if !(1..=N_IDS).contains(&gold) || !(1..=N_IDS).contains(&actual) || actual == gold {
        return Err(fail(
            "first wrong comparison requires two distinct legal original IDs",
        ));
    }
    let folder = panel.join(format!("{id}-{}-{index:02}", u8::from(read)));
    let mass_bytes = fs::read(folder.join("candidate-masses.u64"))?;
    let row_bytes = fs::read(folder.join("candidate.rows"))?;
    if mass_bytes.len() != N_IDS * 8
        || sha(&mass_bytes) != step["mass_sha256"]
        || row_bytes.len() != ROW_HEADER + N_IDS * ROW_BYTES
        || !row_bytes.starts_with(b"A1EM0001")
        || word(&row_bytes, 8)? != N_IDS as u64
    {
        return Err(fail(
            "the recorded first-wrong CUDA tape is malformed or misbound",
        ));
    }
    let masses = mass_bytes
        .chunks_exact(8)
        .map(|cell| cell.try_into().map(u64::from_le_bytes))
        .collect::<Result<Vec<_>, _>>()?;
    if masses.iter().any(|&mass| !(1..=1 << 48).contains(&mass)) {
        return Err(fail("GPU candidate score outside Q48"));
    }
    let rank = |target: usize| {
        let score = masses[target - 1];
        1 + masses
            .iter()
            .enumerate()
            .filter(|&(i, &mass)| mass > score || (mass == score && i + 1 < target))
            .count()
    };
    let actual_mass = masses[actual - 1];
    let gold_mass = masses[gold - 1];
    if rank(actual) != 1
        || step["mass_q48"] != actual_mass
        || integer(step, "top_ties")? != masses.iter().filter(|&&mass| mass == actual_mass).count()
    {
        return Err(fail(
            "the recorded winner does not match the complete CUDA vector",
        ));
    }
    let actual_components = components(&row_bytes, weights, actual, actual_mass, true)?;
    let gold_components = components(&row_bytes, weights, gold, gold_mass, false)?;
    if actual_components["bucket"] != step["bucket"]
        || actual_components["bucket"] != gold_components["bucket"]
    {
        return Err(fail(
            "candidate bucket differs from the observed prefix choice",
        ));
    }
    let successor = step["pick_successor"].as_u64();
    if !read && (successor.is_some() || !step["pick_words"].is_null()) {
        return Err(fail("read-suppressed step carried a Pick successor"));
    }
    for (candidate, score) in [(gold, &gold_components), (actual, &actual_components)] {
        let expected = if successor == Some(candidate as u64) {
            1u64 << 48
        } else {
            0
        };
        if score["pick_expert_mass_q48"] != expected {
            return Err(fail(
                "candidate expert mass does not match its original Pick successor",
            ));
        }
    }
    let picked = if let Some(successor) = successor {
        let id = usize::try_from(successor)?;
        if !(1..=N_IDS).contains(&id) {
            return Err(fail("Pick successor exceeds original vocabulary"));
        }
        let mass = masses[id - 1];
        let picked_components = components(&row_bytes, weights, id, mass, false)?;
        if picked_components["pick_expert_mass_q48"] != 1u64 << 48 {
            return Err(fail(
                "recorded Pick successor is missing its fixed expert mass",
            ));
        }
        Some(json!({"id":id,"rank":rank(id),"mass_q48":mass,"s5":picked_components}))
    } else {
        None
    };
    Ok(
        json!({"id":id,"read_enabled":read,"first_wrong_step_zero_based":index,
        "gold_id":gold,"gold_mass_q48":gold_mass,"gold_rank":rank(gold),"gold_s5":gold_components,
        "winner_id":actual,"winner_mass_q48":actual_mass,"winner_s5":actual_components,
        "pick_words":step["pick_words"],"pick_successor":successor,"pick_equals_gold":successor == Some(gold as u64),
        "picked_candidate":picked,
        "prefix_sha256":step["prefix_sha256"],"candidate_masses_sha256":step["mass_sha256"],
        "candidate_expert_rows_sha256":sha(&row_bytes)}),
    )
}

fn contrasts(panel: &Path, steps: &[Value]) -> Result<Vec<Value>, Failure> {
    let mut output = Vec::new();
    for (left, right, changed) in [
        (31001, 31002, "asked_person"),
        (31003, 31004, "asked_person"),
        (31001, 31003, "fact_order"),
        (31002, 31004, "fact_order"),
    ] {
        for read in [true, false] {
            let step = |id: u32| {
                steps
                    .iter()
                    .find(|row| row["id"] == id && row["read_enabled"] == read && row["step"] == 0)
                    .ok_or_else(|| fail("missing step zero for the crossed cause"))
            };
            let read_words = |id: u32| -> Result<(Vec<u8>, Vec<u8>), Failure> {
                let at = panel.join(format!("{id}-{}-00", u8::from(read)));
                let rows = fs::read(at.join("candidate.rows"))?;
                let masses = fs::read(at.join("candidate-masses.u64"))?;
                if rows.len() != ROW_HEADER + N_IDS * ROW_BYTES
                    || !rows.starts_with(b"A1EM0001")
                    || word(&rows, 8)? != N_IDS as u64
                    || masses.len() != N_IDS * 8
                    || sha(&masses) != step(id)?["mass_sha256"]
                {
                    return Err(fail("malformed first-step contrast tape"));
                }
                Ok((rows, masses))
            };
            let (left_rows, left_mass) = read_words(left)?;
            let (right_rows, right_mass) = read_words(right)?;
            let mut changed_lanes = [0usize; LANES + 1];
            for i in 0..N_IDS {
                let start = ROW_HEADER + i * ROW_BYTES;
                for (lane, count) in changed_lanes.iter_mut().enumerate() {
                    let at = start + lane * 8;
                    *count += usize::from(word(&left_rows, at)? != word(&right_rows, at)?);
                }
            }
            let left_step = step(left)?;
            let right_step = step(right)?;
            let mixture_changed = left_mass
                .chunks_exact(8)
                .zip(right_mass.chunks_exact(8))
                .filter(|(a, b)| a != b)
                .count();
            output.push(
                json!({"ids":[left,right],"changed_input":changed,"read_enabled":read,
                "input_prefix_hash_equal":left_step["prefix_sha256"]==right_step["prefix_sha256"],
                "candidate_bucket_words_changed":changed_lanes[0],
                "expert_mass_words_changed_by_lane":&changed_lanes[1..],
                "mixed_candidate_masses_changed":mixture_changed,
                "pick_words":[left_step["pick_words"],right_step["pick_words"]],
                "pick_successors":[left_step["pick_successor"],right_step["pick_successor"]],
                "winning_ids":[left_step["token"],right_step["token"]],
                "winning_masses_q48":[left_step["mass_q48"],right_step["mass_q48"]]}),
            );
        }
    }
    Ok(output)
}

fn run(panel: &Path, key: &Path, weight_path: &Path, out: &Path) -> Result<(), Failure> {
    let key_bytes = fs::read(key)?;
    let weights = fs::read(weight_path)?;
    let steps_bytes = fs::read(panel.join("steps.jsonl"))?;
    let receipt_bytes = fs::read(panel.join("receipt.json"))?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    if sha(&key_bytes) != KEY_SHA
        || receipt["panel_sha256"] != PANEL_SHA
        || sha(&steps_bytes) != receipt["steps_sha256"]
        || sha(&weights) != receipt["weights_sha256"]
        || weights.len() != BUCKET_HEADER + N_BUCKETS * LANES * 8
        || !weights.starts_with(b"UORS5W01")
        || weights[8..12] != 10u32.to_le_bytes()
        || hex::encode(&weights[12..44]) != receipt["fit_manifest_sha256"]
    {
        return Err(fail(
            "wrong blinded panel, answer key, fit or device receipt",
        ));
    }
    let keys: Vec<Value> = serde_json::from_slice(&key_bytes)?;
    let steps = std::str::from_utf8(&steps_bytes)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()?;
    if steps.len() != integer(&receipt, "steps")? {
        return Err(fail("device receipt step count differs"));
    }
    let mut rows = Vec::with_capacity(12);
    for id in 31001..=31006 {
        let key = keys
            .iter()
            .find(|x| x["id"] == id)
            .ok_or_else(|| fail("missing golden panel row"))?;
        let gold: Vec<usize> = serde_json::from_value(key["gold"].clone())?;
        if gold.last() != Some(&1) {
            return Err(fail("gold must include EOS"));
        }
        for read in [true, false] {
            let arm = steps
                .iter()
                .filter(|step| step["id"] == id && step["read_enabled"] == read)
                .collect::<Vec<_>>();
            if arm.is_empty()
                || arm.len() > 12
                || arm.iter().enumerate().any(|(i, step)| step["step"] != i)
            {
                return Err(fail("device arm is absent or out of order"));
            }
            let index = (0..arm.len().max(gold.len())).find(|&i| {
                arm.get(i).and_then(|step| step["token"].as_u64())
                    != gold.get(i).map(|&id| id as u64)
            });
            let row = match index {
                Some(i) if i < arm.len() && i < gold.len() => {
                    inspect(panel, arm[i], gold[i], &weights)?
                }
                Some(i) => json!({"id":id,"read_enabled":read,"first_wrong_step_zero_based":i,
                    "observed_stop":arm.last().map(|step| &step["stop"]),"gold_unexecuted_id":gold.get(i)}),
                None => {
                    json!({"id":id,"read_enabled":read,"first_wrong_step_zero_based":null,"exact_through_eos":true})
                }
            };
            rows.push(row);
        }
    }
    let result = json!({"schema":"uor-r4.d31.first-wrong-mass/1","device_receipt_sha256":sha(&receipt_bytes),
        "device_steps_sha256":receipt["steps_sha256"],"answer_key_sha256":KEY_SHA,"weights_sha256":receipt["weights_sha256"],
        "ranked_ids":N_IDS,
        "expert_lanes":["kn","ml_0","ml_1","ml_2","ml_3","ml_4","ml_5","ml_6","ml_7","ml_8","ml_9","ml_10","uniform","s5_pick"],
        "rows":rows,"first_step_contrasts":contrasts(panel,&steps)?});
    let mut file = fs::File::options().write(true).create_new(true).open(out)?;
    serde_json::to_writer_pretty(&mut file, &result)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<(), Failure> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if let [panel, key, weights, output] = args.as_slice() {
        return run(
            Path::new(panel),
            Path::new(key),
            Path::new(weights),
            Path::new(output),
        );
    }
    Err(fail(
        "usage: first-wrong-mass SEALED-CUDA-PANEL PRIVATE-KEY FIT-WEIGHTS NEW-OUTPUT",
    ))
}
