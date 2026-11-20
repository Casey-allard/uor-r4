//! D31 reserved-TRAIN A1 preparation and fitted-state readback.
#![forbid(unsafe_code)]

#[cfg(feature = "cuda")]
#[path = "../../../../../labs/kn-reference/instrument/src/pick_device.rs"]
mod pick_device;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use uor_r4_discrete::kn_ceiling::{CeilingTable, EmStart, FrozenPickMixture, PickTarget};
use uor_r4_integer::lawful_likelihood::bits_q12;

type Failure = Box<dyn std::error::Error>;
const N: u32 = 10;
const WIDTH: usize = 11;
const EXPERTS: usize = 14;
const WORDS: usize = 528 * EXPERTS;
const PASSES: u32 = 96;
const REAL_ROWS: usize = 15_947_584;
const C1_SHA: &str = "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030";
const R_SHA: &str = "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a";
const ORIGINAL: &[u8] = include_bytes!("../../claude-439/run2/pick-p10.json");

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn new(path: &Path) -> Result<fs::File, Failure> {
    Ok(fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Failure> {
    let mut file = new(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn row_header(out: &mut impl Write, count: usize) -> Result<(), Failure> {
    out.write_all(b"A1EM0001")?;
    out.write_all(&(count as u64).to_le_bytes())?;
    Ok(())
}

fn write_rows(out: &mut impl Write, table: &CeilingTable) -> Result<(), Failure> {
    for (&bucket, masses) in table
        .buckets()
        .iter()
        .zip(table.masses().chunks_exact(EXPERTS))
    {
        out.write_all(&u64::from(bucket).to_le_bytes())?;
        for mass in masses {
            out.write_all(&mass.to_le_bytes())?;
        }
    }
    Ok(())
}

fn trace(path: &Path, rows: usize) -> Result<(Vec<u8>, Vec<Vec<u64>>), Failure> {
    let bytes = fs::read(path)?;
    let mut expected = *b"A1TR0001\0\0\0\0\0\0\0\0";
    expected[8..].copy_from_slice(&(rows as u64).to_le_bytes());
    if !bytes.starts_with(&expected) || bytes.len() != 16 + (PASSES as usize + 1) * (WORDS + 1) * 8
    {
        return Err(fail("invalid CUDA trace header or length"));
    }
    let values = bytes[16..]
        .chunks_exact((WORDS + 1) * 8)
        .map(|iteration| {
            iteration
                .chunks_exact(8)
                .map(|word| word.try_into().map(u64::from_le_bytes))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((bytes, values))
}

// The host oracle is bounded to this invented table. Every real update
// reads the separate CUDA trace, and this mode accepts no external rows.
fn finite(dir: &Path, device_trace: Option<&Path>) -> Result<(), Failure> {
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    let mut kn = Vec::new();
    let mut pick = Vec::new();
    for i in 0..128 {
        let highest = i % WIDTH;
        kn.push(1 + ((i as u64 * 65_537 * 524_287) % (1 << 48)));
        pick.push(PickTarget {
            class: (i % 3) as u8,
            hit: i % 3 != 0 && i % 2 == 0,
        });
        for k in 0..WIDTH {
            let total = if k <= highest { (k + i) % 29 + 1 } else { 0 };
            totals.push(total as u64);
            counts.push((i % (total + 1)) as u32);
        }
    }
    let table = CeilingTable::from_lookups_with_pick(N, &kn, &counts, &totals, &pick)?;
    if let Some(path) = device_trace {
        let (_, seen) = trace(path, table.targets())?;
        let selected = vec![true; table.targets()];
        let enabled = vec![true; EXPERTS];
        for (pass, row) in seen.iter().enumerate() {
            let expected =
                table.fit_em_from(&selected, &enabled, EmStart::Kn, pass as u32, bits_q12)?;
            if Some(&u128::from(row[0])) != expected.fit_bits.last() || row[1..] != expected.weights
            {
                return Err(fail(format!(
                    "CUDA differs from the Rust oracle after pass {pass}"
                )));
            }
        }
        println!("finite exact parity: 128 rows, 97 Q12 sums, 717024 Q32 words");
    } else {
        let mut out = BufWriter::new(new(&dir.join("finite.rows"))?);
        row_header(&mut out, table.targets())?;
        write_rows(&mut out, &table)?;
        out.flush()?;
        out.get_ref().sync_all()?;
    }
    Ok(())
}

fn seal(out: &Path) -> Result<(), Failure> {
    let (trace_bytes, steps) = trace(&out.join("fit.trace"), REAL_ROWS)?;
    let receipt_bytes = fs::read(out.join("prepare.json"))?;
    let prepared: Value = serde_json::from_slice(&receipt_bytes)?;
    if prepared["format"] != "uor-r4.d31.a1-preparation/1"
        || prepared["r_sha256"] != R_SHA
        || prepared["c_sha256"] != C1_SHA
        || prepared["rows"] != REAL_ROWS
        || prepared["device_source_sha256"] != digest(include_bytes!("device.cu"))
    {
        return Err(fail("invalid A1 preparation receipt"));
    }
    let original: Value = serde_json::from_slice(ORIGINAL)?;
    let a1 = &original["pick_arms"]["main"]["a1"];
    let expected = a1["fit_bits_q12"]
        .as_array()
        .ok_or_else(|| fail("original bits absent"))?;
    if expected.len() != steps.len() {
        return Err(fail("historical iteration count differs"));
    }
    for (pass, (step, want)) in steps.iter().zip(expected).enumerate() {
        if want
            .as_str()
            .ok_or_else(|| fail("missing original Q12 value"))?
            .parse::<u64>()?
            != step[0]
        {
            return Err(fail(format!(
                "historical Q12 score differs after pass {pass}"
            )));
        }
        FrozenPickMixture::new(N, [0; 32], step[1..].to_vec())?;
    }
    let final_step = steps
        .last()
        .ok_or_else(|| fail("missing final CUDA state"))?;
    let old_pick = a1["pick_weights"]
        .as_array()
        .ok_or_else(|| fail("original Pick column absent"))?;
    if old_pick.is_empty() {
        return Err(fail("no historical Pick words"));
    }
    for entry in old_pick {
        let bucket = entry["bucket"]
            .as_u64()
            .filter(|b| *b < 528)
            .ok_or_else(|| fail("invalid historical bucket"))? as usize;
        if entry["weight_q32"].as_u64() != Some(final_step[1 + bucket * EXPERTS + EXPERTS - 1]) {
            return Err(fail(format!(
                "historical Q32 Pick word differs at bucket {bucket}"
            )));
        }
    }
    let probe_bytes = fs::read(out.join("probes.json"))?;
    let ordinal_bytes = fs::read(out.join("probe-ordinals.u64"))?;
    let mass_bytes = fs::read(out.join("probe-masses.u64"))?;
    if prepared["probes_sha256"] != digest(&probe_bytes)
        || prepared["probe_ordinals_sha256"] != digest(&ordinal_bytes)
    {
        return Err(fail("A1 probe identity differs"));
    }
    let mut probes: Vec<Value> = serde_json::from_slice(&probe_bytes)?;
    if probes.len() < 64
        || probes.len() * 8 != mass_bytes.len()
        || mass_bytes.len() != ordinal_bytes.len()
    {
        return Err(fail("wrong A1 probe tape length"));
    }
    let manifest = serde_json::to_vec_pretty(&json!({
        "schema": "uor-r4.d31.a1-fit/1", "prepared": prepared,
        "preparation_sha256": digest(&receipt_bytes), "cuda_trace_sha256": digest(&trace_bytes),
        "historical_receipt_sha256": digest(ORIGINAL), "rows": REAL_ROWS, "passes": PASSES,
        "weight_format": "UORS5W01", "q32_weight_words": WORDS,
        "historical_q12_pass_sums_equal": steps.len(), "historical_q32_pick_words_equal": old_pick.len(),
        "cuda_probe_masses_sha256": digest(&mass_bytes), "cuda_probe_count": probes.len(),
        "final_fit_bits_q12": final_step[0]
    }))?;
    let binding = Sha256::digest(&manifest).into();
    let state = FrozenPickMixture::new(N, binding, final_step[1..].to_vec())?;
    let payload = state.to_bytes();
    let state_path = out.join("fit.weights");
    write_new(&state_path, &payload)?;
    let readback = fs::read(&state_path)?;
    let reloaded = FrozenPickMixture::from_bytes(&readback, binding)?;
    if readback != payload || reloaded != state {
        return Err(fail("serialized A1 state changed at reload"));
    }
    let mut reject_closed = 0;
    let mut suppress_open = 0;
    for ((probe, ordinal), mass) in probes
        .iter_mut()
        .zip(ordinal_bytes.chunks_exact(8))
        .zip(mass_bytes.chunks_exact(8))
    {
        let ordinal = u64::from_le_bytes(ordinal.try_into()?);
        let want = u64::from_le_bytes(mass.try_into()?);
        if probe["ordinal"] != ordinal {
            return Err(fail("ordinal differs at A1 probe"));
        }
        let kn = probe["kn_q48"]
            .as_u64()
            .ok_or_else(|| fail("no probe KN word"))?;
        let counts: [u32; WIDTH] = serde_json::from_value(probe["counts"].clone())?;
        let totals: [u64; WIDTH] = serde_json::from_value(probe["totals"].clone())?;
        let class: u8 = serde_json::from_value(probe["pick_class"].clone())?;
        let hit = probe["pick_hit"]
            .as_bool()
            .ok_or_else(|| fail("no probe Pick hit"))?;
        let (bucket, got) =
            reloaded.predict_mass(kn, &counts, &totals, PickTarget { class, hit })?;
        if got != want {
            return Err(fail(format!(
                "CUDA and reloaded mixture mass differ at ordinal {ordinal}"
            )));
        }
        if class == 0 {
            if !probe["successor"].is_null()
                || reloaded
                    .predict_mass(kn, &counts, &totals, PickTarget { class, hit: true })
                    .is_ok()
            {
                return Err(fail(format!(
                    "closed Pick gate leaked at ordinal {ordinal}"
                )));
            }
            reject_closed += 1;
        }
        if hit {
            let (_, without_pick) =
                reloaded.predict_mass(kn, &counts, &totals, PickTarget { class, hit: false })?;
            let contribution =
                reloaded.weights()[usize::from(bucket) * EXPERTS + EXPERTS - 1] << 16;
            if got.checked_sub(without_pick) != Some(contribution) {
                return Err(fail(format!(
                    "Pick suppression differs at ordinal {ordinal}"
                )));
            }
            suppress_open += usize::from(contribution != 0);
        }
        probe["bucket"] = json!(bucket);
        probe["mixed_q48"] = json!(got);
    }
    if reject_closed == 0 || suppress_open == 0 {
        return Err(fail(
            "A1 gate check lacked a closed or substantive fired probe",
        ));
    }
    write_new(
        &out.join("probes-scored.json"),
        &serde_json::to_vec_pretty(&probes)?,
    )?;
    write_new(&out.join("fit.manifest.json"), &manifest)?;
    println!(
        "{}",
        json!({"weight_bytes": payload.len(), "weight_sha256": digest(&payload),
               "manifest_sha256": digest(&manifest), "words": WORDS,
               "closed_gate_injections_rejected": reject_closed,
               "fired_gate_suppressions_with_nonzero_mass": suppress_open})
    );
    Ok(())
}

#[cfg(feature = "cuda")]
mod prepare;

fn main() -> Result<(), Failure> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        [mode, dir] if mode == Path::new("finite") => finite(dir, None),
        [mode, dir, result] if mode == Path::new("verify-finite") => finite(dir, Some(result)),
        [mode, output] if mode == Path::new("seal") => seal(output),
        #[cfg(feature = "cuda")]
        [mode, dir, trunk, counts, count_sha, out] if mode == Path::new("prepare") => {
            prepare::run(dir, trunk, counts, count_sha, out)
        }
        _ => Err(fail(concat!(
            "usage: d31-a1-fit finite OUT_DIR | verify-finite DIR DEVICE_TRACE | ",
            "prepare SEALED_CR_DIR TRUNK C1_COUNTS C1_COUNTS_SHA NEW_OUT_DIR | ",
            "seal OUT_DIR"
        ))),
    }
}
