//! Independent compact-format and finite native replay oracle for D36.
use memmap2::Mmap;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{error::Error, fs::File, path::Path};
use uor_r4_count_runtime::CompactCountLookup;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Spectrum = [[u64; 4]; 2];

const ORDERS: usize = 11;
const Q48: u128 = 1 << 48;

fn slice<'a>(bytes: &mut &'a [u8], len: usize) -> Result<&'a [u8]> {
    let (prefix, rest) = bytes
        .split_at_checked(len)
        .ok_or("truncated compact table")?;
    *bytes = rest;
    Ok(prefix)
}

fn little(bytes: &[u8]) -> u64 {
    bytes.iter().rev().fold(0, |n, &x| (n << 8) | u64::from(x))
}

#[derive(Clone, Copy)]
struct Group<'a> {
    key: &'a [u8],
    rows: &'a [u8],
    width: usize,
}

impl Group<'_> {
    fn followers(&self) -> impl Iterator<Item = (u16, u32, u32)> + '_ {
        self.rows.chunks_exact(self.width).map(|row| {
            let raw = little(&row[2..6]) as u32;
            let adjusted = if self.width == 6 {
                raw
            } else {
                little(&row[6..]) as u32
            };
            (little(&row[..2]) as u16, raw, adjusted)
        })
    }

    fn tokens(&self, order: usize) -> Vec<u16> {
        let packed = self
            .key
            .iter()
            .fold(0u128, |n, &x| (n << 8) | u128::from(x));
        let mut tokens: Vec<u16> = (0..order)
            .map(|slot| ((packed >> (slot * 12)) & 4095) as u16)
            .collect();
        tokens.push(little(&self.rows[..2]) as u16);
        tokens
    }
}

struct Section<'a> {
    index: &'a [u8],
    followers: &'a [u8],
    stride: usize,
    width: usize,
    spectrum: Spectrum,
}

impl<'a> Section<'a> {
    fn count(&self) -> usize {
        self.index.len() / self.stride
    }

    fn key(&self, i: usize) -> &'a [u8] {
        &self.index[i * self.stride..(i + 1) * self.stride - 4]
    }

    fn offset(&self, i: usize) -> usize {
        if i == self.count() {
            return self.followers.len();
        }
        let end = (i + 1) * self.stride;
        little(&self.index[end - 4..end]) as usize * self.width
    }

    fn group(&self, i: usize) -> Result<Group<'a>> {
        let rows = self
            .followers
            .get(self.offset(i)..self.offset(i + 1))
            .filter(|rows| !rows.is_empty())
            .ok_or("invalid follower span")?;
        Ok(Group {
            key: self.key(i),
            rows,
            width: self.width,
        })
    }

    fn lookup(&self, context: &[u16]) -> Result<Option<Group<'a>>> {
        let packed = context
            .iter()
            .rev()
            .fold(0u128, |n, &id| (n << 12) | u128::from(id));
        let bytes = packed.to_be_bytes();
        let key = &bytes[20 - self.stride..];
        let mut lo = 0;
        let mut hi = self.count();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.key(mid) < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if lo < self.count() && self.key(lo) == key {
            return self.group(lo).map(Some);
        }
        Ok(None)
    }
}

fn decode(mut bytes: &[u8], train: [u8; 32]) -> Result<Vec<Section<'_>>> {
    if slice(&mut bytes, 8)? != b"UORKNC02"
        || little(slice(&mut bytes, 4)?) != 10
        || slice(&mut bytes, 32)? != train
    {
        return Err("wrong compact header, maximum order, or TRAIN pin".into());
    }
    let mut sections = Vec::new();
    for order in 0..ORDERS {
        let rows = little(slice(&mut bytes, 4)?) as usize;
        let groups = little(slice(&mut bytes, 4)?) as usize;
        let mut spectrum = [[0; 4]; 2];
        for word in spectrum.iter_mut().flatten() {
            *word = little(slice(&mut bytes, 8)?);
        }
        let stride = (order * 12).div_ceil(8) + 4;
        let width = if order == 10 { 6 } else { 10 };
        let index = slice(&mut bytes, groups * stride)?;
        let followers = slice(&mut bytes, rows * width)?;
        let section = Section {
            index,
            followers,
            stride,
            width,
            spectrum,
        };
        if section.offset(0) != 0 {
            return Err(format!("order {order} does not begin at follower row 0").into());
        }
        sections.push(section);
    }
    if !bytes.is_empty() {
        return Err("trailing compact bytes".into());
    }
    Ok(sections)
}

#[derive(Default)]
struct Totals {
    groups: u64,
    followers: u64,
    raw: u64,
    adjusted: u64,
    spectrum: Spectrum,
}

impl Totals {
    fn add(&mut self, group: Group<'_>) -> Result<u64> {
        self.groups += 1;
        let mut support = 0;
        let mut previous = 0;
        for (next, raw, adjusted) in group.followers() {
            if next <= previous || next >= 4096 || raw == 0 || adjusted == 0 {
                return Err("invalid or unordered source follower".into());
            }
            previous = next;
            self.followers += 1;
            support += u64::from(raw);
            self.adjusted += u64::from(adjusted);
            for (spectrum, count) in self.spectrum.iter_mut().zip([raw, adjusted]) {
                if count <= 4 {
                    spectrum[count as usize - 1] += 1;
                }
            }
        }
        self.raw += support;
        Ok(support)
    }

    fn report(&self) -> serde_json::Value {
        json!({
            "groups": self.groups, "followers": self.followers,
            "raw_support": self.raw, "adjusted_support": self.adjusted,
            "raw_spectrum": self.spectrum[0], "adjusted_spectrum": self.spectrum[1],
        })
    }
}

fn discounts(spectrum: [u64; 4]) -> [u128; 3] {
    let fallback = [Q48 / 2, Q48, 3 * Q48 / 2];
    if spectrum.contains(&0) {
        return fallback;
    }
    let n = spectrum.map(u128::from);
    let scale = n[0] + 2 * n[1];
    let mut words = [0; 3];
    for (j, word) in words.iter_mut().enumerate() {
        let rank = j as u128 + 1;
        let divisor = scale * n[j];
        let subtracted = (rank + 1) * n[0] * n[j + 1];
        let undiscounted = rank * divisor;
        if subtracted > undiscounted {
            return fallback;
        }
        *word = (undiscounted - subtracted) * Q48 / divisor;
    }
    words
}

#[derive(Debug, PartialEq)]
struct Lookup {
    counts: [u32; ORDERS],
    totals: [u64; ORDERS],
    mass: u64,
}

fn reference(sections: &[Section<'_>], tokens: &[u16]) -> Result<Lookup> {
    let at = tokens.len() - 1;
    let mut result = Lookup {
        counts: [0; ORDERS],
        totals: [0; ORDERS],
        mass: 0,
    };
    let mut mass = Q48 / 4096;
    for (order, section) in sections.iter().take(at + 1).enumerate() {
        let Some(group) = section.lookup(&tokens[at - order..at])? else {
            continue;
        };
        let discount = discounts(section.spectrum[1]);
        let mut removed = 0;
        let mut kept = 0;
        let mut total = 0;
        for (id, raw, adj) in group.followers() {
            let d = discount[(adj as usize).min(3) - 1];
            removed += d;
            total += u128::from(adj);
            result.totals[order] += u64::from(raw);
            if id == tokens[at] {
                result.counts[order] = raw;
                kept = u128::from(adj) * Q48 - d;
            }
        }
        mass = kept / total + (removed / total) * mass / Q48;
    }
    result.mass = mass.max(1) as u64;
    Ok(result)
}

fn native(reader: &CompactCountLookup<'_>, tokens: &[u16]) -> Result<Lookup> {
    let mut result = Lookup {
        counts: [0; ORDERS],
        totals: [0; ORDERS],
        mass: 0,
    };
    result.mass = reader.lookup(
        tokens,
        tokens.len() - 1,
        &mut result.counts,
        &mut result.totals,
    )?;
    Ok(result)
}

struct Audit {
    sections: Vec<serde_json::Value>,
    probes: Vec<(usize, bool, Vec<u16>)>,
    prior_bytes: u64,
    floor_bytes: u64,
    prior_tied_groups: u64,
    prior_smallest_group: Option<u64>,
}

fn audit(source: &[Section<'_>], candidate: &[Section<'_>], threshold: u64) -> Result<Audit> {
    let mut sections = Vec::new();
    let mut probes = Vec::new();
    let mut prior_bytes = 44 + 72 * ORDERS as u64;
    let mut floor_bytes = prior_bytes;
    let mut prior_tied_groups = 0;
    let mut prior_smallest_group: Option<u64> = None;
    for (order, (before, after)) in source.iter().zip(candidate).enumerate() {
        let (mut all, mut kept, mut dropped) =
            (Totals::default(), Totals::default(), Totals::default());
        let mut previous = None;
        let mut witnessed = [false; 2];
        for i in 0..before.count() {
            let group = before.group(i)?;
            if previous.is_some_and(|key| key >= group.key) || (order % 2 == 1 && group.key[0] > 15)
            {
                return Err(
                    format!("invalid or unordered source key at order {order}, group {i}").into(),
                );
            }
            previous = Some(group.key);
            let support = all.add(group)?;
            let retains = order < 4 || support >= threshold;
            let group_bytes = (before.stride + group.rows.len()) as u64;
            if order < 4 || support >= threshold.saturating_sub(1) {
                prior_bytes += group_bytes;
            }
            if order < 4 {
                floor_bytes += group_bytes;
            } else if !retains && support == threshold - 1 {
                prior_tied_groups += 1;
                prior_smallest_group = Some(
                    prior_smallest_group.map_or(group_bytes, |current| current.min(group_bytes)),
                );
            }
            if retains {
                let j = kept.groups as usize;
                if j == after.count() {
                    return Err(
                        format!("omitted retained key at order {order}, source group {i}").into(),
                    );
                }
                let emitted = after.group(j)?;
                if group.key != emitted.key || group.rows != emitted.rows {
                    return Err(format!("retained key or follower words differ at order {order}, source group {i}, candidate group {j}").into());
                }
                kept.add(group)?;
            } else {
                dropped.add(group)?;
            }
            if !witnessed[usize::from(retains)] {
                probes.push((order, retains, group.tokens(order)));
                witnessed[usize::from(retains)] = true;
            }
        }
        if kept.groups as usize != after.count()
            || kept.followers as usize * after.width != after.followers.len()
            || all.spectrum != before.spectrum
            || kept.spectrum != after.spectrum
        {
            return Err(format!("group, follower or spectrum mismatch at order {order}").into());
        }
        sections.push(json!({
            "order": order, "original": all.report(), "retained": kept.report(),
            "dropped": dropped.report(), "section_data_bytes": after.index.len() + after.followers.len(),
        }));
    }
    Ok(Audit {
        sections,
        probes,
        prior_bytes,
        floor_bytes,
        prior_tied_groups,
        prior_smallest_group,
    })
}

fn pin(value: &str) -> Result<[u8; 32]> {
    let mut digest = [0; 32];
    hex::decode_to_slice(value, &mut digest)?;
    Ok(digest)
}

fn map(path: &Path) -> Result<Mmap> {
    let file = File::open(path)?;
    // The audited artifacts are immutable for the lifetime of this process.
    Ok(unsafe { Mmap::map(&file) }?)
}

fn pair(
    before: &[Section<'_>],
    after: &[Section<'_>],
    source: &CompactCountLookup<'_>,
    candidate: &CompactCountLookup<'_>,
    probes: &[(usize, bool, Vec<u16>)],
) -> Result<Vec<serde_json::Value>> {
    let mut report = Vec::new();
    for (order, kept, tokens) in probes {
        let k = *order;
        let original = native(source, tokens)?;
        let pruned = native(candidate, tokens)?;
        if original != reference(before, tokens)?
            || pruned != reference(after, tokens)?
            || original.counts[k] == 0
            || pruned.mass <= 1
        {
            return Err(
                format!("native/reference disagreement or clamped mass at order {k}").into(),
            );
        }
        if *kept {
            if pruned.counts[k] != original.counts[k] || pruned.totals[k] != original.totals[k] {
                return Err(format!("native retained-group disagreement at order {k}").into());
            }
        } else {
            // Removing the oldest token removes precisely the top order for this probe.
            let backed_off = native(candidate, &tokens[1..])?;
            if pruned.counts[k] != 0
                || pruned.totals[k] != 0
                || pruned.mass != backed_off.mass
                || pruned.counts != backed_off.counts
                || pruned.totals != backed_off.totals
                || !backed_off.totals.iter().any(|&n| n > 0)
            {
                return Err(format!("native backoff disagreement at order {k}").into());
            }
        }
        report.push(json!({
            "order": k, "retained": kept, "tokens": tokens,
            "source_q48": original.mass, "candidate_q48": pruned.mass,
            "source_counts": original.counts, "source_totals": original.totals,
            "candidate_counts": pruned.counts, "candidate_totals": pruned.totals,
        }));
    }
    Ok(report)
}

fn corruption_control(
    source: &[Section<'_>],
    candidate: &[u8],
    train: [u8; 32],
    threshold: u64,
) -> Result<serde_json::Value> {
    if candidate.len() > 32 * 1024 * 1024 {
        return Err("corruption control is restricted to the finite fixture".into());
    }
    let sections = decode(candidate, train)?;
    let first = &sections[0];
    let offset = 44 + 72 + first.index.len();
    let (row_number, word) = first
        .followers
        .chunks_exact(first.width)
        .enumerate()
        .map(|(i, row)| (i, little(&row[2..6]) as u32))
        .find(|&(_, raw)| raw > 4 && raw < u32::MAX)
        .ok_or("finite fixture has no raw word outside the spectrum bins")?;
    let start = offset + row_number * first.width + 2;
    let mut bad = candidate.to_vec();
    bad[start..start + 4].copy_from_slice(&(word + 1).to_le_bytes());
    let pinned: [u8; 32] = Sha256::digest(candidate).into();
    let digest_error = CompactCountLookup::from_bytes(&bad, train, pinned)
        .err()
        .ok_or("native reader accepted changed bytes at the original pin")?;
    let repinned: [u8; 32] = Sha256::digest(&bad).into();
    CompactCountLookup::from_bytes(&bad, train, repinned)?;
    let exact_error = audit(source, &decode(&bad, train)?, threshold)
        .err()
        .ok_or("exact verifier accepted repinned altered words")?;
    if !exact_error
        .to_string()
        .starts_with("retained key or follower words differ")
    {
        return Err(exact_error);
    }
    Ok(json!({
        "byte_offset": start, "before": word, "after": word + 1,
        "altered_sha256": hex::encode(repinned), "old_pin_rejection": digest_error.to_string(),
        "repinned_native_load": "PASS", "exact_witness_rejection": exact_error.to_string(),
    }))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [source, candidate, threshold, cap, train, source_pin, candidate_pin, extra @ ..] =
        args.as_slice()
    else {
        return Err("usage: d36-retention-verify SOURCE CANDIDATE THRESHOLD COMPACT_CAP TRAIN_SHA256 SOURCE_SHA256 CANDIDATE_SHA256 [--corruption-control]".into());
    };
    if !extra.is_empty() && extra != ["--corruption-control"] {
        return Err("unsupported argument".into());
    }
    let threshold: u64 = threshold.parse()?;
    let cap: u64 = cap.parse()?;
    let (train, source_pin, candidate_pin) = (pin(train)?, pin(source_pin)?, pin(candidate_pin)?);
    let source = map(Path::new(source))?;
    let candidate = map(Path::new(candidate))?;
    if threshold == 0 || candidate.len() as u64 > cap {
        return Err("candidate exceeds compact cap or threshold is zero".into());
    }
    if Sha256::digest(&source)[..] != source_pin || Sha256::digest(&candidate)[..] != candidate_pin
    {
        return Err("whole-artifact SHA-256 mismatch".into());
    }
    let before = decode(&source, train)?;
    let after = decode(&candidate, train)?;
    let audit = audit(&before, &after, threshold)?;
    if audit.probes.is_empty() {
        return Err("compact audit has no context for paired native lookup".into());
    }
    if threshold > 1 && audit.prior_bytes <= cap {
        return Err(format!("prior threshold fits: {} <= {cap}", audit.prior_bytes).into());
    }
    if (threshold == 1
        || audit
            .sections
            .iter()
            .all(|section| section["dropped"]["groups"] == 0))
        && source[..] != candidate[..]
    {
        return Err("all-retained artifact differs byte-for-byte from the immutable input".into());
    }
    let original_reader = CompactCountLookup::from_bytes(&source, train, source_pin)?;
    let candidate_reader = CompactCountLookup::from_bytes(&candidate, train, candidate_pin)?;
    let pairs = pair(
        &before,
        &after,
        &original_reader,
        &candidate_reader,
        &audit.probes,
    )?;
    let corruption = if extra.is_empty() {
        None
    } else {
        Some(corruption_control(&before, &candidate, train, threshold)?)
    };
    let report = json!({
        "train_sha256": hex::encode(train), "source_sha256": hex::encode(source_pin),
        "candidate_sha256": hex::encode(candidate_pin), "source_bytes": source.len(),
        "candidate_bytes": candidate.len(), "threshold": threshold, "compact_cap": cap,
        "floor_bytes": audit.floor_bytes,
        "prior_threshold_bytes": (threshold > 1).then_some(audit.prior_bytes),
        "prior_threshold_added_contexts": audit.prior_tied_groups,
        "prior_threshold_smallest_added_context_bytes": audit.prior_smallest_group,
        "native_cold_load": "PASS", "sections": audit.sections, "paired_native_lookups": pairs,
        "corruption_control": corruption,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
