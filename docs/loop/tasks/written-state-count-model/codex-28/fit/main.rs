use all_starts_native::packet::{decode, digest, hex, le, save, seal, Result, Rows, CAP};
use serde::Deserialize;
use serde_json::json;
use std::{collections::BTreeMap, env, fs, path::Path};
use uor_r4_integer::{
    all_starts::original_candidates,
    discrete_d11::{IntegerDiscreteModel, ROLE_COPY_STOPS},
};

const PARENT: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const FIT_MAGIC: u32 = 0x38324644;
const FIT_WIDTH: u32 = 13;
const TRAIN_START: u32 = 28000;
const TRAIN_ROWS: usize = 68;

#[derive(Deserialize)]
struct Blind {
    row: u32,
    ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Grade {
    row: u32,
    family: String,
    fact_end: usize,
    query_start: usize,
    fence: usize,
    gold_source: usize,
    prefix_sha256: String,
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    source: usize,
    word_end: usize,
    fence: usize,
    descriptor_ids: Vec<u32>,
    descriptor_classes: Vec<u32>,
    descriptor_depth: usize,
    word_width: usize,
    positive: bool,
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<Vec<T>> {
    std::str::from_utf8(bytes)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn prepare(train: &Path, parent: &Path, out: &Path) -> Result<()> {
    let blob = fs::read(parent)?;
    if hex(digest(&blob)) != PARENT {
        return Err("wrong frozen D21 parent identity".into());
    }
    let model = IntegerDiscreteModel::from_bytes(&blob)?;
    let roots = model.packed_codebook_roots();
    let sites = model.frozen_role_replay()?;
    let blind_bytes = fs::read(train.join("train.blind.jsonl"))?;
    let grade_bytes = fs::read(train.join("train.gold.jsonl"))?;
    let blind = parse::<Blind>(&blind_bytes)?;
    let grade = parse::<Grade>(&grade_bytes)?;
    if blind.len() != TRAIN_ROWS || grade.len() != TRAIN_ROWS {
        return Err("wrong TRAIN roster".into());
    }
    let mut families = BTreeMap::new();
    let mut descriptors = BTreeMap::new();
    let mut profiles = BTreeMap::new();
    let mut packet = vec![FIT_MAGIC, 0, FIT_WIDTH, 0];
    let mut positive_count = 0;
    for (index, (blind, grade)) in blind.into_iter().zip(grade).enumerate() {
        let row = blind.row;
        let expected_row = TRAIN_START + index as u32;
        if row != expected_row || grade.row != expected_row {
            return Err(format!(
                "TRAIN roster differs at index {index}: expected {expected_row}, blind {row}, grade {}",
                grade.row
            ).into());
        }
        let ids = &blind.ids;
        if hex(digest(&le(ids.iter().copied()))) != grade.prefix_sha256
            || grade.fact_end >= grade.fence
            || grade.fence >= grade.query_start
            || grade.query_start >= ids.len()
            || !sites.delimiters.contains(&ids[grade.fence])
            || ids.iter().any(|&id| id as usize >= roots.len())
        {
            return Err(format!("TRAIN row {row} provenance/coordinates disagree").into());
        }
        let next = families.len() as u32;
        let family = *families.entry(grade.family).or_insert(next);
        if family != (row - TRAIN_START) / 4 {
            return Err(format!("TRAIN row {row} family order disagrees").into());
        }
        let native = original_candidates(
            ids,
            &roots,
            sites.assertion_classes,
            grade.fact_end,
            grade.query_start,
            ROLE_COPY_STOPS,
        )?;
        if native.fence() != Some(grade.fence as u32) {
            return Err(format!(
                "TRAIN row {row} first native fence {:?} differs from declared {}",
                native.fence(),
                grade.fence
            )
            .into());
        }
        let class = |i: usize| sites.assertion_classes[ids[i] as usize];
        let candidates: Vec<_> = native
            .filter_map(|candidate| {
                candidate
                    .descriptor
                    .map(|descriptor| (candidate, descriptor))
            })
            .collect();
        if candidates.len() != grade.candidates.len() {
            return Err(format!("TRAIN row {row} tokenizer vs loaded site count differs").into());
        }
        let mut positives = 0;
        for ((native, geometry), c) in candidates.into_iter().zip(grade.candidates) {
            let source = native.source as usize;
            let end = native.word_end as usize;
            let depth = geometry.depth;
            let width = end - source;
            let start = source - depth;
            let classes: Vec<_> = (start..end).map(class).collect();
            if c.source != source
                || c.word_end != end
                || c.fence != grade.fence
                || c.descriptor_depth != depth
                || c.word_width != width
                || c.descriptor_classes != classes
                || c.descriptor_ids != ids[start..end]
                || c.positive != (source == grade.gold_source)
            {
                return Err(
                    format!("TRAIN row {row} loaded site differs at source {source}").into(),
                );
            }
            positives += u32::from(c.positive);
            let label = u32::from(!c.positive);
            let shape = (depth | (width << 8)) as u32;
            let signature = geometry.signature;
            let mut packed = [0; 5];
            for (word, pair) in packed.iter_mut().zip(geometry.roots.chunks_exact(2)) {
                *word = u32::from(pair[0]) | (u32::from(pair[1]) << 16);
            }
            if matches!(descriptors.insert((shape, signature, packed), label), Some(prior) if prior != label)
            {
                return Err(
                    format!("TRAIN row {row} contains an opposed geometry collision").into(),
                );
            }
            let census = profiles.entry((shape, signature)).or_insert([0usize; 2]);
            census[label as usize] += 1;
            packet.extend([
                row,
                source as u32,
                end as u32,
                grade.fence as u32,
                family,
                shape,
                signature,
                label,
            ]);
            packet.extend(packed);
        }
        if positives != 1 {
            return Err(format!("TRAIN row {row} lacks unique original positive").into());
        }
        positive_count += positives;
    }
    let len = (packet.len() - 4) / FIT_WIDTH as usize;
    if len > CAP || len <= positive_count as usize {
        return Err("invalid TRAIN bank occupancy or class census".into());
    }
    packet[1] = len as u32;
    let packet = le(packet);
    fs::create_dir(out)?;
    save(&out.join("train.packet"), &packet)?;
    save(
        &out.join("train.binding.json"),
        &serde_json::to_vec_pretty(&json!({
            "schema":"uor-r4.d28-onset-fit/1",
            "parent_sha256": PARENT,
            "tokenizer_sha256": model.tokenizer_sha256().ok_or("missing tokenizer")?,
            "blind_sha256":hex(digest(&blind_bytes)),
            "gold_sha256":hex(digest(&grade_bytes)),
            "packet_sha256":hex(digest(&packet)),
            "families":families,
            "records":len,
            "positives":positive_count,
            "negatives":len-positive_count as usize,
            "profiles":profiles.into_iter().map(|((shape, signature), support)| json!({
                "shape":shape, "signature":signature, "support":support,
            })).collect::<Vec<_>>(),
        }))?,
    )?;
    Ok(())
}

fn seal_result(parent: &Path, stage: &Path, raw: &Path, output: &Path) -> Result<()> {
    let blob = fs::read(parent)?;
    if hex(digest(&blob)) != PARENT {
        return Err("wrong frozen parent SHA-256".into());
    }
    let _model = IntegerDiscreteModel::from_bytes(&blob)?;
    let header = 16 + usize::try_from(u64::from_le_bytes(blob[8..16].try_into()?))?;
    let original = &blob[header.next_multiple_of(64)..];
    let original_bindings = [
        original[16..48].try_into()?,
        original[48..80].try_into()?,
        original[80..112].try_into()?,
        [0; 32],
    ];
    let input = fs::read(stage.join("train.packet"))?;
    let bind = fs::read(stage.join("train.binding.json"))?;
    let receipt: serde_json::Value = serde_json::from_slice(&bind)?;
    if receipt["parent_sha256"] != PARENT || receipt["packet_sha256"] != hex(digest(&input)) {
        return Err("stale fit stage binding".into());
    }
    let raw = fs::read(raw)?;
    let data = decode(&raw)?;
    let training = decode(&input)?;
    if data.get(1) != Some(&0) || data.get(2) != training.get(1) {
        return Err("fit output mode or record census differs".into());
    }
    let len = training[1] as usize;
    for (index, (record, row)) in data
        .get(4..)
        .ok_or("missing raw fit records")?
        .chunks_exact(8)
        .zip(training[4..].chunks_exact(FIT_WIDTH as usize))
        .take(len)
        .enumerate()
    {
        if record[0] != row[5] | (row[7] << 16) || record[1] != row[6] || record[3..] != row[8..] {
            return Err(format!("fit record {index} differs from supervised input").into());
        }
    }
    let bindings = Rows {
        provenance: original_bindings,
        rows: Vec::new(),
    };
    save(output, &seal(&bindings, &input, &bind, &raw)?)
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, train, parent, out] if mode == "prepare" => {
            prepare(Path::new(train), Path::new(parent), Path::new(out))
        }
        [mode, parent, stage, raw, output] if mode == "seal" => seal_result(
            Path::new(parent),
            Path::new(stage),
            Path::new(raw),
            Path::new(output),
        ),
        _ => Err("use prepare TRAIN_DIR PARENT NEW_STAGE or seal PARENT STAGE RAW NEW_BANK".into()),
    }
}
