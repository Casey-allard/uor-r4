//! Native R-only S5 fit. The host generates exact features and seals device state.
#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use uor_r4_discrete::kn_ceiling::{
    CeilingTable, EmStart, FrozenPickMixture, PickClassRule, PickTarget,
};
use uor_r4_integer::lawful_likelihood::bits_q12;

mod controls;
mod exact;
#[path = "../../../../../labs/kn-reference/instrument/src/pick_device.rs"]
mod pick_device;
#[cfg(feature = "cuda")]
mod prepare;

type Failure = Box<dyn std::error::Error>;
const N: u32 = 10;
const WIDTH: usize = 11;
const EXPERTS: usize = 14;
const PASSES: u32 = 96;
const REAL_ROWS: usize = 15_947_584;
const RECORDS: usize = 4_060;
const C_SHA: &str = "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a";
const COMPACT_SHA: &str = "064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8";
const R_SHA: &str = "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a";
const TRUNK_SHA: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const PREPARED: &str = "uor-r4.d37.r-preparation/1";
const ARMS: [(&str, PickClassRule); 2] =
    [("a1", PickClassRule::Three), ("a1s", PickClassRule::Five)];

fn fail(message: impl Into<String>) -> Failure {
    message.into().into()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_digest(path: &Path) -> Result<String, Failure> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut fs::File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn new(path: &Path) -> Result<fs::File, Failure> {
    Ok(fs::File::options()
        .write(true)
        .create_new(true)
        .open(path)?)
}

fn writer(path: &Path) -> Result<BufWriter<fs::File>, Failure> {
    Ok(BufWriter::with_capacity(1 << 20, new(path)?))
}

fn finish(mut writer: BufWriter<fs::File>) -> Result<(), Failure> {
    writer.flush()?;
    Ok(writer.get_ref().sync_all()?)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Failure> {
    let mut file = new(path)?;
    file.write_all(bytes)?;
    Ok(file.sync_all()?)
}

fn buckets(rule: PickClassRule) -> usize {
    176 * usize::from(rule.classes())
}

fn header(magic: &[u8; 8], count: usize, rule: PickClassRule) -> [u8; 24] {
    let mut out = [0; 24];
    out[..8].copy_from_slice(magic);
    out[8..16].copy_from_slice(&(count as u64).to_le_bytes());
    out[16..].copy_from_slice(&(buckets(rule) as u64).to_le_bytes());
    out
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

fn word(bytes: &[u8]) -> Result<u64, Failure> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

fn trace(
    path: &Path,
    rows: usize,
    rule: PickClassRule,
) -> Result<(String, Vec<Vec<u64>>), Failure> {
    let stride = (buckets(rule) * EXPERTS + 1) * 8;
    let size = 24 + (PASSES as usize + 1) * stride;
    if fs::metadata(path)?.len() != size as u64 {
        return Err(fail("wrong CUDA trace byte length"));
    }
    let bytes = fs::read(path)?;
    if bytes.len() != size || !bytes.starts_with(&header(b"R37TR001", rows, rule)) {
        return Err(fail("invalid CUDA trace header or length"));
    }
    let values = bytes[24..]
        .chunks_exact(stride)
        .map(|iteration| iteration.chunks_exact(8).map(word).collect())
        .collect::<Result<Vec<_>, Failure>>()?;
    Ok((digest(&bytes), values))
}

// This oracle accepts only invented inputs and never reads a real training tape.
fn finite(dir: &Path, verify: bool, masked: bool) -> Result<(), Failure> {
    const ROWS: usize = 128;
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    let mut kn = Vec::new();
    let distances = [None, Some(0), Some(1), Some(2), Some(3), Some(16)];
    for i in 0..ROWS {
        let highest = i % WIDTH;
        kn.push(1 + ((i as u64 * 65_537 * 524_287) % (1 << 48)));
        for order in 0..WIDTH {
            let total = if i == WIDTH - 1 {
                65_537
            } else if order <= highest {
                (order + i) % 29 + 1
            } else {
                0
            };
            totals.push(total as u64);
            counts.push((i % (total + 1)) as u32);
        }
    }
    let arms: &[(_, _)] = if masked {
        &[("a1-masked", PickClassRule::Three)]
    } else {
        &ARMS
    };
    for &(arm, rule) in arms {
        let pick: Vec<_> = (0..ROWS)
            .map(|i| {
                let distance = distances[i % distances.len()];
                PickTarget {
                    class: rule.classify(distance),
                    hit: !masked && distance.is_some() && i % 2 == 0,
                }
            })
            .collect();
        let table = CeilingTable::from_lookups_with_pick_classes(
            N,
            &kn,
            &counts,
            &totals,
            &pick,
            rule.classes(),
        )?;
        if usize::from(table.buckets()[WIDTH - 1]) != buckets(rule) - 1
            || table.masses()[(WIDTH - 1) * EXPERTS + EXPERTS - 1]
                != if masked { 0 } else { 1 << 48 }
        {
            return Err(fail(
                "finite fixture failed to reach the final live Pick bucket",
            ));
        }
        if verify {
            let (_, seen) = trace(&dir.join(format!("{arm}.trace")), ROWS, rule)?;
            let selected = vec![true; ROWS];
            let mut enabled = vec![true; EXPERTS];
            enabled[EXPERTS - 1] = !masked;
            let mut terminal = None;
            for (pass, row) in seen.iter().enumerate() {
                let expected =
                    table.fit_em_from(&selected, &enabled, EmStart::Kn, pass as u32, bits_q12)?;
                if Some(&u128::from(row[0])) != expected.fit_bits.last()
                    || row[1..] != expected.weights
                {
                    return Err(fail(format!(
                        "{arm} CUDA differs from Rust after pass {pass}"
                    )));
                }
                terminal = Some(expected.weights);
            }
            let oracle = FrozenPickMixture::with_rule(
                N,
                [0; 32],
                terminal.ok_or_else(|| fail("missing finite state"))?,
                rule,
            )?;
            let last = WIDTH - 1;
            let (bucket, mass) = oracle.predict_mass(
                kn[last],
                &counts[last * WIDTH..(last + 1) * WIDTH],
                &totals[last * WIDTH..(last + 1) * WIDTH],
                pick[last],
            )?;
            let probe = fs::read(dir.join(format!("{arm}.probe.u64")))?;
            if probe.len() != 16
                || word(&probe[..8])? != u64::from(bucket)
                || word(&probe[8..])? != mass
            {
                return Err(fail(format!("{arm} final bucket probe mismatch")));
            }
            println!(
                "{arm} finite parity with bucket {bucket}, {ROWS} rows, 97 sums and {} Q32 words",
                buckets(rule) * EXPERTS * 97
            );
        } else {
            let mut out = writer(&dir.join(format!("{arm}.rows")))?;
            out.write_all(&header(b"R37EM001", ROWS, rule))?;
            write_rows(&mut out, &table)?;
            finish(out)?;
        }
    }
    if !verify {
        write_new(
            &dir.join("finite-ordinal.u64"),
            &((WIDTH - 1) as u64).to_le_bytes(),
        )?;
    }
    Ok(())
}

fn seal(out: &Path, mode: &str, arm: &str) -> Result<(), Failure> {
    let rule = ARMS
        .iter()
        .find_map(|&(name, rule)| (name == arm).then_some(rule))
        .ok_or_else(|| fail("unknown fit arm"))?;
    let prep_bytes = fs::read(out.join("prepare.json"))?;
    let prepared: Value = serde_json::from_slice(&prep_bytes)?;
    if prepared["format"] != PREPARED
        || prepared["r_sha256"] != R_SHA
        || prepared["c_sha256"] != C_SHA
        || prepared["compact_sha256"] != COMPACT_SHA
        || prepared["trunk_sha256"] != TRUNK_SHA
        || prepared["rows"] != REAL_ROWS
        || prepared["records"] != RECORDS
        || prepared["pick_mode"] != mode
        || prepared["device_source_sha256"] != digest(include_bytes!("device.cu"))
        || prepared["host_source_sha256"] != digest(include_bytes!("main.rs"))
        || prepared["prep_source_sha256"] != digest(include_bytes!("prepare.rs"))
        || (mode == "masked"
            && prepared["masked_source_sha256"] != digest(include_bytes!("controls.rs")))
        || prepared["arms"][arm]["rows_sha256"] != file_digest(&out.join(format!("{arm}.rows")))?
    {
        return Err(fail("invalid R preparation receipt or row tape identity"));
    }
    let (trace_sha, steps) = trace(&out.join(format!("{arm}.trace")), REAL_ROWS, rule)?;
    for step in &steps {
        FrozenPickMixture::with_rule(N, [0; 32], step[1..].to_vec(), rule)?;
        if mode == "masked"
            && step[1..]
                .chunks_exact(EXPERTS)
                .any(|weights| weights[EXPERTS - 1] != 0)
        {
            return Err(fail("masked fit gave weight to the disabled Pick lane"));
        }
    }
    let final_step = steps
        .last()
        .ok_or_else(|| fail("missing final CUDA state"))?;
    let probe_bytes = fs::read(out.join("probes.json"))?;
    let ordinal_bytes = fs::read(out.join("probe-ordinals.u64"))?;
    let device_probes = fs::read(out.join(format!("{arm}.probe.u64")))?;
    if prepared["probes_sha256"] != digest(&probe_bytes)
        || prepared["probe_ordinals_sha256"] != digest(&ordinal_bytes)
    {
        return Err(fail("R probe identity differs"));
    }
    let mut probes: Vec<Value> = serde_json::from_slice(&probe_bytes)?;
    if !(64..=1024).contains(&probes.len())
        || probes.len() * 16 != device_probes.len()
        || probes.len() * 8 != ordinal_bytes.len()
    {
        return Err(fail("wrong R probe tape length"));
    }
    let manifest = serde_json::to_vec_pretty(&json!({
        "schema": "uor-r4.d37.r-fit/1", "prepared": prepared, "arm": arm,
        "preparation_sha256": digest(&prep_bytes), "cuda_trace_sha256": trace_sha,
        "rows": REAL_ROWS, "passes": PASSES, "pick_classes": rule.classes(),
        "weight_format": if rule == PickClassRule::Three { "UORS5W01" } else { "UORS5W02" },
        "q32_weight_words": buckets(rule) * EXPERTS,
        "cuda_probes_sha256": digest(&device_probes), "cuda_probe_count": probes.len(),
        "final_fit_bits_q12": final_step[0]
    }))?;
    let binding = Sha256::digest(&manifest).into();
    let state = FrozenPickMixture::with_rule(N, binding, final_step[1..].to_vec(), rule)?;
    let payload = state.to_bytes();
    let reloaded = FrozenPickMixture::from_bytes(&payload, binding)?;
    if reloaded != state {
        return Err(fail("serialized state changed at reload"));
    }
    let mut rejected_closed = 0;
    let mut substantive_hits = 0;
    for ((probe, ordinal), device) in probes
        .iter_mut()
        .zip(ordinal_bytes.chunks_exact(8))
        .zip(device_probes.chunks_exact(16))
    {
        let ordinal = word(ordinal)?;
        if probe["ordinal"] != ordinal {
            return Err(fail("ordinal differs at replay probe"));
        }
        let kn: u64 = serde_json::from_value(probe["kn_q48"].clone())?;
        let counts: [u32; WIDTH] = serde_json::from_value(probe["counts"].clone())?;
        let totals: [u64; WIDTH] = serde_json::from_value(probe["totals"].clone())?;
        let distance: Option<u32> = serde_json::from_value(probe["distance"].clone())?;
        let class = rule.classify(distance);
        let hit: bool = serde_json::from_value(probe["hit"].clone())?;
        let (bucket, got) =
            reloaded.predict_mass(kn, &counts, &totals, PickTarget { class, hit })?;
        if word(&device[..8])? != u64::from(bucket) || word(&device[8..])? != got {
            return Err(fail(format!(
                "device and native bucket or mass differ at R ordinal {ordinal}"
            )));
        }
        if class == 0 {
            if !probe["successor"].is_null()
                || reloaded
                    .predict_mass(kn, &counts, &totals, PickTarget { class, hit: true })
                    .is_ok()
            {
                return Err(fail(format!(
                    "closed Pick gate leaked at R ordinal {ordinal}"
                )));
            }
            rejected_closed += 1;
        }
        if hit {
            let (_, suppressed) =
                reloaded.predict_mass(kn, &counts, &totals, PickTarget { class, hit: false })?;
            let contribution =
                reloaded.weights()[usize::from(bucket) * EXPERTS + EXPERTS - 1] << 16;
            // The lowest representable mixed mass is one Q48 unit.
            if got != suppressed.saturating_add(contribution)
                && !(suppressed == 1 && got == contribution)
            {
                return Err(fail(format!(
                    "Pick suppression differs at R ordinal {ordinal}"
                )));
            }
            substantive_hits += usize::from(contribution != 0);
        }
        probe["bucket"] = json!(bucket);
        probe["mixed_q48"] = json!(got);
    }
    if rejected_closed == 0 || (!["disabled", "masked"].contains(&mode) && substantive_hits == 0) {
        return Err(fail(
            "real replay lacks a closed gate or a substantive fired hit",
        ));
    }
    let weight_path = out.join(format!("{arm}.weights"));
    write_new(&weight_path, &payload)?;
    let readback = fs::read(weight_path)?;
    if readback != payload || FrozenPickMixture::from_bytes(&readback, binding)? != state {
        return Err(fail("on-disk fitted state changed at reload"));
    }
    write_new(
        &out.join(format!("{arm}.probes-scored.json")),
        &serde_json::to_vec_pretty(&probes)?,
    )?;
    write_new(&out.join(format!("{arm}.manifest.json")), &manifest)?;
    println!(
        "{}",
        json!({
            "arm": arm, "pick_mode": mode, "weight_bytes": payload.len(),
            "weight_sha256": digest(&payload), "manifest_sha256": digest(&manifest),
            "closed_gate_injections_rejected": rejected_closed,
            "fired_gate_suppressions_with_nonzero_mass": substantive_hits
        })
    );
    Ok(())
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        [mode, dir] if mode == Path::new("finite") => finite(dir, false, false),
        [mode, dir] if mode == Path::new("verify-finite") => finite(dir, true, false),
        [mode, dir] if mode == Path::new("finite-masked") => finite(dir, false, true),
        [mode, dir] if mode == Path::new("verify-finite-masked") => finite(dir, true, true),
        [mode, path] if mode == Path::new("finite-c2") => exact::finite(path, None),
        [mode, path, tape] if mode == Path::new("verify-finite-c2") => {
            exact::finite(path, Some(tape))
        }
        [mode, dir, pick, arm] if mode == Path::new("seal") => seal(
            dir,
            pick.to_str()
                .ok_or_else(|| fail("Pick mode must be UTF-8"))?,
            arm.to_str().ok_or_else(|| fail("arm must be UTF-8"))?,
        ),
        [mode, identity, pin, out] if mode == Path::new("prepare-masked") => controls::masked(
            identity,
            pin.to_str()
                .ok_or_else(|| fail("identity pin must be UTF-8"))?,
            out,
        ),
        #[cfg(feature = "cuda")]
        [command, mode, r, trunk, compact, tape, pin, out] if command == Path::new("prepare") => {
            prepare::run(
                mode.to_str()
                    .ok_or_else(|| fail("Pick mode must be UTF-8"))?,
                r,
                trunk,
                compact,
                tape,
                pin.to_str()
                    .ok_or_else(|| fail("feature tape pin must be UTF-8"))?,
                out,
                None,
            )
        }
        #[cfg(feature = "cuda")]
        [command, r, trunk, compact, tape, pin, exact_tape, exact_pin, out]
            if command == Path::new("prepare-c2") =>
        {
            prepare::run(
                "exact",
                r,
                trunk,
                compact,
                tape,
                pin.to_str()
                    .ok_or_else(|| fail("count tape pin must be UTF-8"))?,
                out,
                Some((
                    exact_tape,
                    exact_pin
                        .to_str()
                        .ok_or_else(|| fail("C2 tape pin must be UTF-8"))?,
                )),
            )
        }
        _ => Err(fail(concat!(
            "usage: d37-r-fit finite DIR | verify-finite DIR | ",
            "prepare PICK_MODE R_UORT TRUNK COMPACT FEATURES FEATURES_SHA256 NEW_DIR | ",
            "prepare-c2 R_UORT TRUNK COMPACT FEATURES FEATURES_SHA256 C2_TAPE C2_SHA256 NEW_DIR | ",
            "prepare-masked SEALED_IDENTITY_DIR IDENTITY_PREP_SHA256 NEW_DIR | ",
            "seal DIR PICK_MODE ARM"
        ))),
    }
}
