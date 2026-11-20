//! Materialize whole-context retention from a pinned UORKNC02 TRAIN artifact.

#[path = "full-compact-count/retention.rs"]
mod retention;

#[path = "prune-compact-count/source.rs"]
mod source;

use retention::{follower_width, keep, key_width, Budget, Selection, Stats, HEADER, ORDER_HEADER};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use source::Source;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;
use uor_r4_discrete::ngram_count::NGRAM_MAX_ORDER;

type Fail = Box<dyn std::error::Error>;

const N: u32 = NGRAM_MAX_ORDER;
const ORDERS: usize = N as usize + 1;

fn digest_arg(value: &str) -> Result<[u8; 32], Fail> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected a 64-character SHA-256 pin".into());
    }
    let mut digest = [0; 32];
    for (i, out) in digest.iter_mut().enumerate() {
        *out = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16)?;
    }
    Ok(digest)
}

struct OrderLayout {
    key_start: u64,
    follower_start: u64,
    end: u64,
    key_cursor: u64,
    follower_cursor: u32,
}

fn materialize(
    source: &mut Source,
    output: &Path,
    selection: Selection,
    retained: &[Stats; ORDERS],
) -> Result<String, Fail> {
    let key_file = File::options().write(true).create_new(true).open(output)?;
    let mut keys = BufWriter::with_capacity(1 << 20, key_file);
    let mut rows = BufWriter::with_capacity(1 << 20, File::options().write(true).open(output)?);
    keys.get_ref().set_len(selection.bytes)?;
    keys.write_all(&source.header)?;
    let mut layout = Vec::with_capacity(ORDERS);
    let mut start = HEADER;
    for (order, stats) in (0..=N).zip(retained) {
        keys.seek(SeekFrom::Start(start))?;
        keys.write_all(&u32::try_from(stats.followers)?.to_le_bytes())?;
        keys.write_all(&u32::try_from(stats.contexts)?.to_le_bytes())?;
        for spectrum in stats.spectra.iter().flatten() {
            keys.write_all(&spectrum.to_le_bytes())?;
        }
        let key_start = start + ORDER_HEADER;
        let follower_start = key_start + stats.contexts * (key_width(order) as u64 + 4);
        let end = start + ORDER_HEADER + stats.bytes(order, N);
        layout.push(OrderLayout {
            key_start,
            follower_start,
            end,
            key_cursor: key_start,
            follower_cursor: 0,
        });
        start = end;
    }
    if start != selection.bytes {
        return Err("retained section lengths disagree with the selected compact size".into());
    }

    let mut active_order = None;
    source.visit_groups(|order, key, stats, followers| {
        if !keep(order, stats.totals[0], selection.threshold) {
            return Ok(());
        }
        let section = &mut layout[order as usize];
        if active_order != Some(order) {
            keys.seek(SeekFrom::Start(section.key_start))?;
            rows.seek(SeekFrom::Start(section.follower_start))?;
            active_order = Some(order);
        }
        let next_key = section.key_cursor + key.len() as u64 + 4;
        let next_follower = section
            .follower_cursor
            .checked_add(u32::try_from(stats.followers)?)
            .ok_or("too many retained followers")?;
        if next_key > section.follower_start
            || section.follower_start + u64::from(next_follower) * follower_width(order, N)
                > section.end
        {
            return Err("retained group would overrun its compact section".into());
        }
        keys.write_all(key)?;
        keys.write_all(&section.follower_cursor.to_le_bytes())?;
        rows.write_all(followers)?;
        section.key_cursor = next_key;
        section.follower_cursor = next_follower;
        Ok(())
    })?;

    for (order, section) in (0..=N).zip(layout) {
        if section.key_cursor != section.follower_start
            || section.follower_start
                + u64::from(section.follower_cursor) * follower_width(order, N)
                != section.end
        {
            return Err("materialization did not fill retained order sections exactly".into());
        }
    }
    rows.flush()?;
    keys.flush()?;
    keys.get_ref().sync_all()?;
    let mut digest = Sha256::new();
    if io::copy(&mut File::open(output)?, &mut digest)? != selection.bytes {
        return Err("materialized file length differs from selected compact size".into());
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn census_json(stats: &[Stats; ORDERS]) -> Vec<Value> {
    (0..=N)
        .zip(stats)
        .map(|(order, stats)| {
            json!({
                "order": order,
                "contexts": stats.contexts,
                "followers": stats.followers,
                "raw_support": stats.totals[0],
                "adjusted_support": stats.totals[1],
                "raw_spectrum": stats.spectra[0],
                "adjusted_spectrum": stats.spectra[1],
                "data_bytes": stats.bytes(order, N),
            })
        })
        .collect()
}

fn prune(
    source_path: &Path,
    source_pin: [u8; 32],
    training_pin: [u8; 32],
    cap: u64,
    output_path: &Path,
) -> Result<Value, Fail> {
    if output_path.symlink_metadata().is_ok() {
        return Err("requested artifact path already exists".into());
    }
    let mut source = Source::open(source_path, source_pin, training_pin)?;
    let mut budget = Budget::new(N);
    let mut original = [Stats::default(); ORDERS];
    let mut by_support: [BTreeMap<u64, Stats>; ORDERS] = std::array::from_fn(|_| BTreeMap::new());
    source.visit_groups(|order, _, stats, _| {
        original[order as usize].add(stats);
        if order > retention::ALWAYS_KEEP_ORDER {
            by_support[order as usize]
                .entry(stats.totals[0])
                .or_default()
                .add(stats);
        }
        budget.observe(order, stats.totals[0], stats.followers)
    })?;
    let selection = budget.select(cap)?;
    let mut retained = original;
    for (order, histogram) in (0..=N).zip(by_support) {
        if order > retention::ALWAYS_KEEP_ORDER {
            let stats = &mut retained[order as usize];
            *stats = Stats::default();
            for (_, group) in histogram.range(selection.threshold..) {
                stats.add(*group);
            }
        }
    }

    let parent = output_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let staging = parent.join(format!(".prune-compact-count-{}", std::process::id()));
    fs::create_dir(&staging)?;
    let staged = staging.join("compact.counts");
    let written = materialize(&mut source, &staged, selection, &retained).and_then(|sha| {
        source.check_unchanged()?;
        // A same-filesystem link atomically publishes the sealed artifact
        // and fails if the requested path already belongs to any file.
        fs::hard_link(&staged, output_path)?;
        Ok(sha)
    });
    if staged.exists() {
        fs::remove_file(staged)?;
    }
    fs::remove_dir(staging)?;
    let artifact_sha256 = written?;
    Ok(json!({
        "training_sha256": format!("{:x}", sha2::digest::Output::<Sha256>::from(training_pin)),
        "source_artifact_sha256": format!("{:x}", sha2::digest::Output::<Sha256>::from(source_pin)),
        "source_artifact_bytes": source.bytes,
        "artifact_sha256": artifact_sha256,
        "compact_cap_bytes": cap,
        "artifact_bytes": selection.bytes,
        "mandatory_floor_bytes": selection.floor,
        "threshold": selection.threshold,
        "original": census_json(&original),
        "retained": census_json(&retained),
    }))
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [source, source_sha, training_sha, cap, output] = args.as_slice() else {
        return Err("usage: prune-compact-count SOURCE ARTIFACT_SHA256 TRAIN_SHA256 COMPACT_CAP NEW-ARTIFACT".into());
    };
    let receipt = prune(
        Path::new(source),
        digest_arg(source_sha)?,
        digest_arg(training_sha)?,
        cap.parse()?,
        Path::new(output),
    )?;
    println!("{receipt}");
    Ok(())
}
