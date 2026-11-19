use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Write, path::Path};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const INPUT: u32 = 0x33415355;
pub const LABEL: u32 = 0x314c5355;
pub const RAW: u32 = 0x31425355;
pub const TRACE: u32 = 0x32545355;
const SEALED: u32 = 0x32425355;
pub const HEADER: usize = 36;
pub const WIDTH: usize = 35;
pub const MAX_ROWS: usize = 65536;
pub const CAP: usize = 256;
pub const RECORD: usize = 8;
pub const OUT: usize = 68;
pub const NIL: u32 = u32::MAX;
pub const NO_RADIUS: u32 = 65535;

#[derive(Clone, Copy)]
pub struct Row(pub [u32; WIDTH]);

pub struct Rows {
    pub provenance: [[u8; 32]; 4],
    pub rows: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Label {
    pub ordinal: u32,
    pub assertion_index: u32,
    pub family: u32,
    pub source: Option<u32>,
}

#[derive(Deserialize)]
struct Prefix {
    ordinal: u32,
    prefix_sha256: String,
    original_tape_len: u32,
    requested_span: Option<[u32; 2]>,
    assertions: Vec<Assertion>,
}
#[derive(Deserialize)]
struct Assertion {
    fact_span: [u32; 2],
    fence: Option<u32>,
    eligible_mask: u32,
    word_start_mask: u32,
    legacy_joint_mask: u32,
    legacy_candidates: Vec<Candidate>,
}
#[derive(Deserialize)]
struct Candidate {
    original_start: u32,
    first_word_end: u32,
    connector_slots: u32,
    word_slots: u32,
    writer_reachable: bool,
    slot_ids: Vec<u32>,
    slot_classes: Vec<u32>,
    packed_roots_u16: Vec<u16>,
    packed_roots_le: Vec<u8>,
}

pub fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
pub fn hex_digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid lowercase SHA-256".into());
    }
    let mut out = [0; 32];
    for (i, value) in out.iter_mut().enumerate() {
        *value = u8::from_str_radix(&text[2 * i..2 * i + 2], 16)?;
    }
    Ok(out)
}
pub fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn le(words: impl IntoIterator<Item = u32>) -> Vec<u8> {
    words.into_iter().flat_map(u32::to_le_bytes).collect()
}
pub fn decode(bytes: &[u8]) -> Result<Vec<u32>> {
    if !bytes.len().is_multiple_of(4) {
        return Err("truncated little-endian word".into());
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
        .collect())
}
pub fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

impl Row {
    pub fn new(
        ordinal: u32,
        assertion: u32,
        length: u32,
        fact: u32,
        query: u32,
        fence: u32,
        masks: [u32; 2],
    ) -> Self {
        let mut raw = [0; WIDTH];
        raw[..8].copy_from_slice(&[
            ordinal, assertion, length, fact, query, fence, masks[0], masks[1],
        ]);
        for at in [8, 17, 26] {
            raw[at] = NIL;
        }
        Self(raw)
    }

    pub fn candidate(&self, offset: usize) -> &[u32] {
        &self.0[8 + offset * 9..17 + offset * 9]
    }
    pub fn insert(
        &mut self,
        offset: usize,
        source: u32,
        end: u32,
        reachable: bool,
        classes: &[u32],
        roots: &[u16],
    ) -> Result<()> {
        let slot = self
            .0
            .get_mut(8 + offset * 9..17 + offset * 9)
            .ok_or("invalid candidate slot")?;
        if slot[0] != NIL
            || classes.len() != roots.len()
            || roots.len() > 10
            || classes.iter().any(|&c| c > 2)
        {
            return Err("duplicate or malformed candidate".into());
        }
        if reachable
            && roots.len()
                != (offset + 1) + end.checked_sub(source).ok_or("reversed word")? as usize
        {
            return Err("candidate geometry width differs from original span".into());
        }
        slot[..3].copy_from_slice(&[source, end, u32::from(reachable)]);
        if reachable {
            for (i, (&class, &root)) in classes.iter().zip(roots).enumerate() {
                slot[3] |= class << (2 * i);
                slot[4 + i / 2] |= u32::from(root) << (16 * (i % 2));
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        let row = &self.0;
        if !(1..=192).contains(&row[2]) || (row[6] | row[7]) > 7 {
            return Err("invalid original tape width or legacy masks".into());
        }
        if row[5] == NIL {
            if row[6] != 0
                || row[7] != 0
                || (0..3).any(|i| self.candidate(i) != [NIL, 0, 0, 0, 0, 0, 0, 0, 0])
            {
                return Err("absent fence has eligible candidates".into());
            }
            return Ok(());
        }
        if row[1] == NIL || !(0 < row[3] && row[3] < row[5] && row[5] < row[4] && row[4] < row[2]) {
            return Err("invalid fact/fence/query coordinates".into());
        }
        let mask = row[6] & row[7];
        for i in 0..3 {
            let c = self.candidate(i);
            if mask & (1 << i) == 0 {
                if c != [NIL, 0, 0, 0, 0, 0, 0, 0, 0] {
                    return Err("slot absent from legacy intersection".into());
                }
                continue;
            }
            let source = row[3] + 2 + i as u32;
            if c[0] != source
                || !(source < c[1] && c[1] <= row[5])
                || c[2] != u32::from(row[5] - source < 8)
            {
                return Err("source offset, end, or reachability mismatch".into());
            }
            if c[2] == 0 {
                if c[3..].iter().any(|&part| part != 0) {
                    return Err("cap-rejected slot carries geometry".into());
                }
            } else {
                validate_geometry(i + 1, (c[1] - source) as usize, c[3], &c[4..], false)?;
            }
        }
        for i in 0..3 {
            let a = self.candidate(i);
            if a[2] != 1 {
                continue;
            }
            for j in 0..i {
                let b = self.candidate(j);
                if b[2] != 1 {
                    continue;
                }
                let count = (a[1].min(b[1]) - row[3] - 1) as usize;
                for at in 0..count {
                    if ((a[3] >> (2 * at)) & 3) != ((b[3] >> (2 * at)) & 3)
                        || ((a[4 + at / 2] >> (16 * (at % 2))) & 65535)
                            != ((b[4 + at / 2] >> (16 * (at % 2))) & 65535)
                    {
                        return Err(
                            "original candidate geometry disagrees on an overlapping slot".into(),
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

fn validate_geometry(
    depth: usize,
    word: usize,
    signature: u32,
    roots: &[u32],
    position: bool,
) -> Result<()> {
    let n = depth + word;
    if !(1..=3).contains(&depth) || !(1..=7).contains(&word) || signature >> (2 * n) != 0 {
        return Err("invalid geometry width or padding".into());
    }
    if position {
        if signature != 0 || roots.iter().any(|&r| r != 0) {
            return Err("position-only record carries lexical geometry".into());
        }
        return Ok(());
    }
    if (signature >> (2 * depth)) & 3 != 1 {
        return Err("candidate onset is not START".into());
    }
    for i in 0..10 {
        let class = (signature >> (2 * i)) & 3;
        if i < n && (class > 2 || (i > depth && class != 2)) {
            return Err("invalid slot class".into());
        }
        if i >= n && (roots[i / 2] >> (16 * (i % 2))) & 65535 != 0 {
            return Err("unused geometry slot is nonzero".into());
        }
    }
    Ok(())
}

impl Rows {
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = le([INPUT, self.rows.len() as u32, WIDTH as u32, 0]);
        bytes.extend(self.provenance.into_iter().flatten());
        bytes.extend(le(self.rows.iter().flat_map(|r| r.0)));
        Ok(bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let words = decode(bytes)?;
        if words.len() < HEADER
            || words[0] != INPUT
            || words[2] != WIDTH as u32
            || words[3] != 0
            || words.len() != HEADER + words[1] as usize * WIDTH
        {
            return Err("candidate packet framing mismatch".into());
        }
        let provenance =
            core::array::from_fn(|i| bytes[16 + i * 32..48 + i * 32].try_into().unwrap());
        let rows = words[HEADER..]
            .chunks_exact(WIDTH)
            .map(|raw| Row(raw.try_into().unwrap()))
            .collect();
        let packet = Self { provenance, rows };
        packet.validate()?;
        Ok(packet)
    }

    fn validate(&self) -> Result<()> {
        if self.rows.is_empty() || self.rows.len() > MAX_ROWS {
            return Err("candidate count out of bounds".into());
        }
        let mut last = None;
        for row in &self.rows {
            let key = (row.0[0], row.0[1]);
            if last.is_some_and(|p| p >= key) {
                return Err("candidate keys are not strictly increasing".into());
            }
            row.validate()?;
            last = Some(key);
        }
        Ok(())
    }
}

/// Canonical site identity across same-seed blind TRAIN and inference captures.
pub fn site_key(metadata: &Value) -> Result<[u8; 32]> {
    let mut hash = Sha256::new();
    hash.update(b"uor-r4/all-starts-sites/1");
    for name in [
        "role_d11",
        "codebook_sha256",
        "classes_sha256",
        "span_state_sha256",
    ] {
        hash.update(hex_digest(
            metadata[name]
                .as_str()
                .ok_or("missing frozen site provenance")?,
        )?);
    }
    for word in uor_r4_integer::discrete_d11::spiralcore_orbit_roots() {
        hash.update(word.map(|x| x as u8));
    }
    Ok(hash.finalize().into())
}

pub fn pack_json(
    bytes: &[u8],
    model: [u8; 32],
    tokenizer: [u8; 32],
    sites: [u8; 32],
) -> Result<Rows> {
    let value: Value = serde_json::from_slice(bytes)?;
    let tape = if value.is_array() {
        value
    } else {
        if value["schema"] != "uor-r4.legacy-assertion-candidates/1"
            || value["copy_stops"] != serde_json::json!([16, 1, 0, 201])
        {
            return Err("unrecognized capture contract".into());
        }
        if value["mode"] != "synthetic"
            && (value["base_d11"] != hex(model)
                || value["tokenizer_sha256"] != hex(tokenizer)
                || site_key(&value)? != sites)
        {
            return Err("capture provenance conflicts with required model/tokenizer/sites".into());
        }
        value["rows"].clone()
    };
    let rows: Vec<Prefix> = serde_json::from_value(tape)?;
    let mut output = Vec::new();
    let mut previous = None;
    for p in rows {
        if previous.is_some_and(|old| old >= p.ordinal) {
            return Err("capture ordinals are not strictly increasing".into());
        }
        previous = Some(p.ordinal);
        hex_digest(&p.prefix_sha256)?;
        let query = p.requested_span.map_or(0, |range| range[0]);
        if p.assertions.is_empty() {
            output.push(Row::new(
                p.ordinal,
                NIL,
                p.original_tape_len,
                0,
                query,
                NIL,
                [0, 0],
            ));
        }
        for (index, a) in p.assertions.into_iter().enumerate() {
            let [start, end] = a.fact_span;
            let query_span = p
                .requested_span
                .ok_or("assertion has no original requested span")?;
            if !(0 < start
                && start < end
                && end < query_span[0]
                && query_span[0] < query_span[1]
                && query_span[1] <= p.original_tape_len)
            {
                return Err("bad original assertion or requested range".into());
            }
            if a.legacy_joint_mask != (a.eligible_mask & a.word_start_mask) {
                return Err("legacy mask intersection mismatch".into());
            }
            let mut row = Row::new(
                p.ordinal,
                index as u32,
                p.original_tape_len,
                end - 1,
                query,
                a.fence.unwrap_or(NIL),
                [a.eligible_mask, a.word_start_mask],
            );
            // A slot is bound to its original range, not to token equality in another row.
            let mut words = BTreeMap::new();
            for c in a.legacy_candidates {
                if !(1..=3).contains(&c.connector_slots)
                    || c.slot_ids.len() != c.slot_classes.len()
                    || c.slot_ids.len() != c.packed_roots_u16.len()
                    || c.slot_ids.iter().any(|&id| id >= 4096)
                    || c.packed_roots_u16
                        .iter()
                        .flat_map(|x| x.to_le_bytes())
                        .ne(c.packed_roots_le.iter().copied())
                    || c.first_word_end.checked_sub(c.original_start) != Some(c.word_slots)
                {
                    return Err("malformed capture candidate".into());
                }
                for (i, (&id, (&class, &root))) in c
                    .slot_ids
                    .iter()
                    .zip(c.slot_classes.iter().zip(&c.packed_roots_u16))
                    .enumerate()
                {
                    if let Some(old) = words.insert(end + i as u32, (id, class, root)) {
                        if old != (id, class, root) {
                            return Err("overlapping original slots disagree".into());
                        }
                    }
                }
                let (classes, roots) = if c.writer_reachable {
                    (&c.slot_classes[..], &c.packed_roots_u16[..])
                } else {
                    (&[][..], &[][..])
                };
                row.insert(
                    c.connector_slots as usize - 1,
                    c.original_start,
                    c.first_word_end,
                    c.writer_reachable,
                    classes,
                    roots,
                )?;
            }
            output.push(row);
        }
    }
    let packet = Rows {
        provenance: [model, tokenizer, sites, digest(bytes)],
        rows: output,
    };
    packet.validate()?;
    Ok(packet)
}

pub fn fit_labels(rows: &Rows, labels: &[Label]) -> Result<Vec<u8>> {
    let mut keyed = BTreeMap::new();
    for label in labels {
        if (label.source.is_some() && label.family == NIL)
            || label.source == Some(NIL)
            || keyed
                .insert((label.ordinal, label.assertion_index), label)
                .is_some()
        {
            return Err("duplicate label or omitted TRAIN family".into());
        }
    }
    let mut out = vec![LABEL, rows.rows.len() as u32, 4, 0];
    for row in &rows.rows {
        let key = (row.0[0], row.0[1]);
        let label = keyed.remove(&key);
        let src = label.and_then(|l| l.source).unwrap_or(NIL);
        if src != NIL && !(0..3).any(|i| row.candidate(i)[0] == src && row.candidate(i)[2] == 1) {
            return Err(format!("gold coverage failure at {:?}", key).into());
        }
        out.extend([key.0, key.1, label.map_or(NIL, |l| l.family), src]);
    }
    if !keyed.is_empty() {
        return Err("label without blind row".into());
    }
    Ok(le(out))
}

pub fn validate_raw(raw: &[u8]) -> Result<Vec<u32>> {
    let data = decode(raw)?;
    if data.len() != 4 + CAP * RECORD
        || data[0] != RAW
        || data[1] > 2
        || !(1..=CAP as u32).contains(&data[2])
        || data[3] != CAP as u32
    {
        return Err("raw bank framing mismatch".into());
    }
    let (mode, used) = (data[1], data[2] as usize);
    let records: Vec<_> = data[4..].chunks_exact(RECORD).collect();
    let mut shapes = BTreeMap::new();
    let mut labels = BTreeMap::new();
    let mut classes = [false; 2];
    for (i, r) in records.iter().enumerate() {
        if i >= used {
            if r.iter().any(|&a| a != 0) {
                return Err("nonzero unused bank record".into());
            }
            continue;
        }
        let label = r[0] >> 16;
        if label > 1 || (r[2] > 1280 && r[2] != NO_RADIUS) || (mode == 2 && r[2] != NO_RADIUS) {
            return Err("invalid class or radius".into());
        }
        classes[label as usize] = true;
        validate_geometry(
            (r[0] & 255) as usize,
            (r[0] >> 8 & 255) as usize,
            r[1],
            &r[3..],
            mode == 2,
        )?;
        let key = (r[0] & 65535, r[1]);
        if mode == 1 {
            let depth = (r[0] & 255) as usize;
            let length = ((r[0] >> 8) & 255) as usize;
            let roots: [u16; 10] =
                core::array::from_fn(|j| (r[3 + j / 2] >> (16 * (j % 2))) as u16);
            let mut normalized = roots;
            crate::score::canonical(&mut normalized[..depth + length], depth, r[1]);
            if normalized != roots {
                return Err("noncanonical neutral bank geometry".into());
            }
        }
        if let Some(old) = shapes.insert(key, r[2]) {
            if old != r[2] {
                return Err("two radii for one shape".into());
            }
        }
        if mode != 2 {
            if let Some(old) = labels.insert((key, &r[3..]), label) {
                if old != label {
                    return Err("opposed geometric labels".into());
                }
            }
        }
    }
    if classes != [true, true] {
        return Err("bank missing class".into());
    }
    Ok(data)
}

pub fn seal(rows: &Rows, input: &[u8], labels: &[u8], raw: &[u8]) -> Result<Vec<u8>> {
    validate_raw(raw)?;
    let mut bank = le([SEALED, 6, raw.len() as u32, 0]);
    bank.extend(rows.provenance[..3].iter().flatten());
    bank.extend(digest(input));
    bank.extend(digest(labels));
    bank.extend(digest(raw));
    bank.extend(raw);
    Ok(bank)
}

pub fn open<'a>(rows: &Rows, bank: &'a [u8]) -> Result<&'a [u8]> {
    const BINDINGS: usize = 208;
    if bank.len() < BINDINGS
        || decode(&bank[..16])? != [SEALED, 6, (bank.len() - BINDINGS) as u32, 0]
    {
        return Err("sealed bank framing mismatch".into());
    }
    for (i, expected) in rows.provenance[..3].iter().enumerate() {
        if &bank[16 + 32 * i..48 + 32 * i] != expected {
            return Err("bank model/tokenizer/sites binding mismatch".into());
        }
    }
    let raw = &bank[BINDINGS..];
    if bank[176..208] != digest(raw) {
        return Err("bank payload checksum mismatch".into());
    }
    validate_raw(raw)?;
    Ok(raw)
}
