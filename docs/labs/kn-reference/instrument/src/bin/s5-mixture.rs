//! Package the full S5 weight vector or inspect one historical Pick contribution.
#![forbid(unsafe_code)]

#[path = "../pick_device.rs"]
mod pick_device;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, io::Write, path::Path};
use uor_r4_discrete::{
    kn_ceiling::{
        bucket_count, ceiling_bucket, CeilingTable, EmStart, FrozenPickMixture, PickTarget,
        WEIGHT_ONE,
    },
    kn_pick::{code_pick, code_pick_rows},
};

type Failure = Box<dyn std::error::Error>;
const ORIGINAL_REPORT: &str = "28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53";

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [action, report, output] if action == "pack" => pack(report, output),
        [action, output] if action == "fixture" => fixture(output),
        [action, report, trunk, order, total, prefix] if action == "historical" => {
            historical(report, trunk, order.parse()?, total.parse()?, prefix)
        }
        _ => Err(fail(concat!(
            "usage: s5-mixture pack FULL_S5_REPORT NEW_WEIGHTS; ",
            "s5-mixture historical PINNED_S5_REPORT PINNED_TRUNK ORDER TOTAL PREFIX_CSV; ",
            "s5-mixture fixture NEW_WEIGHTS"
        ))),
    }
}

fn pack(report: &str, output: &str) -> Result<(), Failure> {
    let bytes = fs::read(report)?;
    let receipt: Value = serde_json::from_slice(&bytes)?;
    let highest = order_from(&receipt)?;
    let words = receipt["pick_arms"]["main"]["a1"]["weights_q32"]
        .as_array()
        .ok_or_else(|| fail("full A1 weight vector absent from S5 report"))?
        .iter()
        .map(|word| word.as_u64().ok_or_else(|| fail("invalid A1 weight word")))
        .collect::<Result<Vec<_>, _>>()?;
    let binding = Sha256::digest(&bytes).into();
    let fit = FrozenPickMixture::new(highest, binding, words)?;
    let (readback, payload) = write_weights(&fit, binding, output)?;
    if readback != fit {
        return Err(fail("S5 weight readback mismatch"));
    }
    println!(
        "{}",
        json!({"weight_bytes": payload.len(), "report_sha256": format!("{:x}", Sha256::digest(&bytes)),
               "weight_sha256": format!("{:x}", Sha256::digest(&payload))})
    );
    Ok(())
}

fn write_weights(
    fit: &FrozenPickMixture,
    binding: [u8; 32],
    output: &str,
) -> Result<(FrozenPickMixture, Vec<u8>), Failure> {
    let payload = fit.to_bytes();
    let mut file = fs::File::options()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(&payload)?;
    file.sync_all()?;
    drop(file);
    let readback = fs::read(output)?;
    if readback != payload {
        return Err(fail("S5 byte readback mismatch"));
    }
    Ok((FrozenPickMixture::from_bytes(&readback, binding)?, readback))
}

fn fixture(output: &str) -> Result<(), Failure> {
    const STREAM: [u16; 18] = [0, 2, 3, 4, 2, 3, 4, 2, 5, 1, 0, 2, 3, 4, 2, 3, 4, 1];
    const CONTEXT: usize = 8;
    const HIGHEST: u32 = 2;
    const FIT_TARGETS: usize = 9;
    const EM_ITERATIONS: u32 = 96;
    let codes: Vec<_> = (0..6).map(|id| [1u16 << id]).collect();
    let pick: Vec<_> = code_pick_rows(&STREAM, &codes, CONTEXT)?
        .iter()
        .map(|row| PickTarget {
            class: row.class,
            hit: row.hit,
        })
        .collect();
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    for target in 0..pick.len() {
        counts.extend([4, 2, u32::from(target % 2 == 1)]);
        totals.extend([18, 4, if target % 2 == 1 { 2 } else { 0 }]);
    }
    let kn = vec![1 << 45; pick.len()];
    let table = CeilingTable::from_lookups_with_pick(HIGHEST, &kn, &counts, &totals, &pick)?;
    let fit_mask: Vec<_> = (0..pick.len()).map(|target| target < FIT_TARGETS).collect();
    let fit = table.fit_em_from(
        &fit_mask,
        &vec![true; table.experts()],
        EmStart::Kn,
        EM_ITERATIONS,
        uor_r4_integer::lawful_likelihood::bits_q12,
    )?;
    let expected = table.mixed_masses(&fit.weights)?;
    let manifest = json!({
        "protocol": "D28 finite S5 fixture v1", "stream": STREAM, "codes": codes, "context": CONTEXT,
        "kn": kn, "counts": counts, "totals": totals, "fit_mask": fit_mask, "highest": HIGHEST,
        "em_iterations": EM_ITERATIONS, "initialization": "KN", "objective": "bits_q12"
    });
    let input_hash = Sha256::digest(serde_json::to_vec(&manifest)?);
    let binding = input_hash.into();
    let frozen = FrozenPickMixture::new(HIGHEST, binding, fit.weights)?;
    let (reloaded, payload) = write_weights(&frozen, binding, output)?;
    if reloaded != frozen {
        return Err(fail("finite S5 weight readback mismatch"));
    }
    let mut replay = Vec::new();
    for (target, &want) in expected.iter().enumerate() {
        let span = target * 3..(target + 1) * 3;
        let (bucket, got) = reloaded.predict_mass(
            kn[target],
            &counts[span.clone()],
            &totals[span],
            pick[target],
        )?;
        if (bucket, got) != (table.buckets()[target], want) {
            return Err(fail(format!("finite S5 replay differs at target {target}")));
        }
        replay.push(
            json!({"bucket": bucket, "mass_q48": got, "pick_class": pick[target].class,
                           "pick_hit": pick[target].hit, "fit": fit_mask[target]}),
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "bytes": payload.len(), "sha256": format!("{:x}", Sha256::digest(&payload)),
            "manifest_sha256": format!("{input_hash:x}"), "weight_words": reloaded.weights().len(),
        "fit_targets": FIT_TARGETS, "replay_targets": replay.len(),
            "initial_bits_q12": fit.fit_bits.first(), "final_bits_q12": fit.fit_bits.last(), "replay": replay
        }))?
    );
    Ok(())
}

fn order_from(receipt: &Value) -> Result<u32, Failure> {
    let order = receipt["highest_order"]
        .as_u64()
        .ok_or_else(|| fail("missing highest_order"))?;
    if order > 10 {
        return Err(fail("S5 order exceeds ten"));
    }
    Ok(order as u32)
}

fn historical(
    report: &str,
    trunk: &str,
    order: usize,
    total: u64,
    ids: &str,
) -> Result<(), Failure> {
    let bytes = fs::read(report)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    if digest != ORIGINAL_REPORT {
        return Err(fail(format!(
            "unexpected historical S5 report digest {digest}"
        )));
    }
    let receipt: Value = serde_json::from_slice(&bytes)?;
    let highest = order_from(&receipt)?;
    let model = pick_device::load_trunk(Path::new(trunk))?;
    let (blocks, _, _) = model.identity_pick_inventory()?;
    let codes = pick_device::root_codes(&blocks)?;
    let ids = ids
        .split(',')
        .map(str::parse::<u16>)
        .collect::<Result<Vec<_>, _>>()?;
    if ids.first() != Some(&0)
        || ids[1..]
            .iter()
            .any(|&id| id < 2 || usize::from(id) >= codes.len())
        || ids.len() > model.shape().context as usize
        || order > highest as usize
        || order > ids.len()
        || total == 0
    {
        return Err(fail(
            "expected one causal BOS-framed prefix and a positive seen total",
        ));
    }
    let (position, distance, ties) =
        code_pick(&codes, &ids).ok_or_else(|| fail("historical prefix does not admit Pick"))?;
    let class = if distance == 0 { 1 } else { 2 };
    let mut totals = vec![0; highest as usize + 1];
    totals[order] = total;
    let (_, plain) = ceiling_bucket(&totals)?;
    let bucket = class * bucket_count(highest) + usize::from(plain);
    let weight = receipt["pick_arms"]["main"]["a1"]["pick_weights"]
        .as_array()
        .ok_or_else(|| fail("missing historical Pick column"))?
        .iter()
        .find(|entry| entry["bucket"].as_u64() == Some(bucket as u64));
    let value = weight
        .and_then(|entry| entry["weight_q32"].as_u64())
        .filter(|&weight| weight <= WEIGHT_ONE)
        .ok_or_else(|| fail("historical report has no fitted Pick weight for this bucket"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "report_sha256": digest, "trunk_sha256": model.artifact_sha256(),
            "seen_order_supplied": order, "seen_total_supplied": total, "prefix_ids": ids,
            "pick_class": class, "pick_position": position, "pick_distance": distance, "pick_ties": ties,
            "copied_id": ids[position + 1], "bucket": bucket, "fit_targets": weight.and_then(|e| e["fit_targets"].as_u64()),
            "pick_weight_q32": value, "copied_target_pick_contribution_q48": value << 16,
            "any_other_target_pick_contribution_q48": 0
        }))?
    );
    Ok(())
}
