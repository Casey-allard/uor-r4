//! Frozen ft/dev identity Pick and suffix support, with a separate label join.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
use uor_r4_discrete::{
    device::{Device, FLOCK_TRACE_HEADER_WORDS, FLOCK_TRACE_LEVEL_WORDS},
    normalization_mechanism::NormalizationMode,
    position_mechanism::{PositionConfig, PositionMode, PositionStepTable},
    Identity, ObjectivePolicy, ReadInitialization, ReadSpec, Spec,
};
use uor_r4_integer::{
    count_mixer::{CONTEXT_MAX_ORDER as DEPTH, VOCAB},
    discrete_d11::IntegerDiscreteModel,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PANE: &str = "d933c533b8b95b4ed6cb54cf96fe83c25a221e3e3226f9a212f52c7bfc1a7074";
const REFERENCE: &str = "9c88ee8d20518a0063f1a8eabb0b37ed84861d52cce561604093404840c8caac";
const TOKENS: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const MASK: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
const META: &str = "5193006637f640deec7ac6d442db75b2892f44ed432ee23071603a807105ac7e";
const TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const CARRIERS: [(&str, &str, usize, u64, usize); 2] = [
    (
        "b544f22f44cbb670aba39289233821504712d30f7d0ca7c2450cada551c02e51",
        "ff199225c52bd3364b938e61d496ad14bd66d559b9265d399a47ad6101181567",
        3,
        93_180_496,
        1_553,
    ),
    (
        "d8744ebba0ec11470c22dcd6371a0cb4ce479c32c9e7fdc829d056cb20b9eea2",
        "0f8a039e257ef4016fb6f5eaedeb4c7ba1907b368fa62d6f30f3aa2a055feef2",
        4,
        125_998_708,
        2_306,
    ),
];
const ROWS: usize = 14_287;
const ABSENT: u32 = u32::MAX;
const BLOCKS: usize = 4;
const VECTOR: usize = BLOCKS * 8;
const PICK_WORDS: usize = VECTOR + BLOCKS * 5 + 1;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn field(row: &Value, name: &str) -> Result<usize> {
    Ok(usize::try_from(
        row[name].as_u64().ok_or("missing unsigned field")?,
    )?)
}

fn fixture(dir: &Path) -> Result<(Vec<u16>, Vec<u8>, Vec<Value>)> {
    let tokens = fs::read(dir.join("tokens.u16"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    let metadata_bytes = fs::read(dir.join("rows.json"))?;
    let metadata: Value = serde_json::from_slice(&metadata_bytes)?;
    let rows = metadata["row_metadata"]
        .as_array()
        .ok_or("missing row metadata")?;
    if hash(&metadata_bytes) != META
        || hash(&tokens) != TOKENS
        || hash(&mask) != MASK
        || metadata["rows_sha256"] != PANE
        || rows.len() != ROWS
        || mask.iter().filter(|&&mark| mark == 1).count() != ROWS
        || tokens.len() != 64 + mask.len() * 2
    {
        return Err("frozen ft/dev fixture digest or shape mismatch".into());
    }
    let ids = tokens[64..]
        .chunks_exact(2)
        .map(|word| u16::from_le_bytes([word[0], word[1]]))
        .collect();
    Ok((ids, mask, rows.to_vec()))
}

fn prefix_valid(prefix: &[u32], context: usize) -> bool {
    !prefix.is_empty()
        && prefix.len() <= context
        && prefix
            .iter()
            .enumerate()
            .all(|(at, &id)| id < VOCAB as u32 && id != 1 && (id != 0 || at == 0))
}

// Historical Relation spans can contain earlier in-band response EOS
// commits. The registered Pick and exact-neighbor seams end there.
fn record_prefix(prefix: &[u32]) -> &[u32] {
    let start = prefix
        .iter()
        .rposition(|&id| id <= 1)
        .map_or(0, |at| at + usize::from(prefix[at] == 1));
    &prefix[start..]
}

fn capture_prefixes(dir: &Path, out: &Path) -> Result<()> {
    let (ids, mask, meta) = fixture(dir)?;
    let mut canonical = Vec::with_capacity(ROWS);
    let mut effective = Vec::with_capacity(ROWS);
    let mut previous = None;
    for (ordinal, row) in meta.iter().enumerate() {
        let at = field(row, "target_position")?;
        let len = field(row, "prefix_len")?;
        if field(row, "ordinal")? != ordinal
            || at >= ids.len()
            || len > at
            || previous.is_some_and(|old| old >= at)
            || mask[at] != 1
        {
            return Err("frozen ft/dev row order or eligibility mismatch".into());
        }
        previous = Some(at);
        let prefix: Vec<_> = ids[at - len..at].iter().copied().map(u32::from).collect();
        let current = record_prefix(&prefix);
        if !prefix_valid(current, 192) {
            return Err(format!("noncausal frozen prefix at row {ordinal}").into());
        }
        effective.push(current.to_vec());
        canonical.push(prefix);
    }
    fs::write(
        out.join("canonical-prefixes.json"),
        serde_json::to_vec(&canonical)?,
    )?;
    write_prefixes(&effective, out)
}

fn write_prefixes(prefixes: &[Vec<u32>], out: &Path) -> Result<()> {
    fs::write(out.join("prefixes.json"), serde_json::to_vec(prefixes)?)?;
    let mut suffixes = BufWriter::new(File::create(out.join("suffixes.u16"))?);
    for prefix in prefixes {
        let tail = &prefix[prefix.len().saturating_sub(DEPTH)..];
        for id in std::iter::repeat_n(1, DEPTH - tail.len()).chain(tail.iter().copied()) {
            suffixes.write_all(&(id as u16).to_le_bytes())?;
        }
    }
    suffixes.flush()?;
    Ok(())
}

fn prefixes(path: &Path, context: usize) -> Result<Vec<Vec<u32>>> {
    let rows: Vec<Vec<u32>> = serde_json::from_slice(&fs::read(path)?)?;
    if rows.is_empty() || rows.iter().any(|row| !prefix_valid(row, context)) {
        return Err("invalid or empty causal prefix batch".into());
    }
    Ok(rows)
}

fn trunk(path: &Path) -> Result<IntegerDiscreteModel> {
    let model = IntegerDiscreteModel::load(path)?;
    if model.artifact_sha256() != TRUNK
        || model.shape().blocks as usize != BLOCKS
        || model.shape().vocabulary as usize != VOCAB
        || model.shape().context != 192
    {
        return Err("unexpected seed-1 Relation D11 source".into());
    }
    Ok(model)
}

fn gpu(seed: &Path, input: &Path, out: &Path, swap: bool) -> Result<()> {
    let model = trunk(seed)?;
    let shape = model.shape();
    let rows = prefixes(input, shape.context as usize)?;
    let position = PositionConfig::new(
        PositionMode::IdentityControlV1,
        PositionStepTable::identity(shape.blocks)?,
    );
    let fresh = Device::new_with_position(
        Spec {
            vocabulary: shape.vocabulary,
            blocks: shape.blocks,
            context: shape.context,
            kept_cells: shape.kept_cells,
        },
        ReadSpec::Pick,
        ObjectivePolicy::NearestAddressMarginV1,
        ReadInitialization::ActiveLeafV1,
        NormalizationMode::ScaleMatchedSnapV1,
        Some(position),
        shape.seed,
        0,
    )?;
    let mut snapshot = fresh.snapshot()?;
    drop(fresh);
    (snapshot.table, snapshot.routes, snapshot.gates) = model.identity_pick_inventory()?;
    if swap {
        let p = &mut snapshot.table[2 * VECTOR..4 * VECTOR];
        let (id2, id3) = p.split_at_mut(VECTOR);
        id2.swap_with_slice(id3);
    }
    snapshot.epoch = shape.epoch;
    let snapshot_bytes = serde_json::to_vec(&snapshot)?;
    let path = out.with_extension("snapshot.json");
    fs::write(path, &snapshot_bytes)?;
    let expected_geometry = snapshot.table.clone();
    let expected_gates = snapshot.gates.clone();
    let expected_routes = snapshot.routes.clone();
    let device = Device::from_snapshot(snapshot, 0)?;
    let returned = device.snapshot()?;
    if returned.table != expected_geometry
        || returned.gates != expected_gates
        || returned.routes != expected_routes
        || returned
            .position
            .as_ref()
            .map(|config| config.mode.device_word())
            != Some(3)
    {
        return Err("CUDA Pick state readback differs from explicit identity snapshot".into());
    }
    let mut tape = BufWriter::new(File::create(out)?);
    let stride = FLOCK_TRACE_HEADER_WORDS + shape.stages as usize * FLOCK_TRACE_LEVEL_WORDS;
    for (chunk, batch) in rows.chunks(256).enumerate() {
        let windows: Vec<Vec<_>> = batch
            .iter()
            .enumerate()
            .map(|(row, prefix)| {
                prefix
                    .iter()
                    .enumerate()
                    .map(|(pos, &token)| Identity {
                        token,
                        occurrence: pos as u64,
                        record: (chunk * 256 + row) as u64,
                        version: 0,
                    })
                    .collect()
            })
            .collect();
        // The existing traced forward API also accepts a loss target. Reuse the
        // current prefix ID here. The actual held target is unavailable to CUDA.
        let placeholder: Vec<_> = batch.iter().filter_map(|p| p.last().copied()).collect();
        let forward = device.forward_traced(&device.carry_windows(&windows)?, &placeholder)?;
        let vector = forward.vectors()?;
        let trace = forward.flock_trace()?.ok_or("missing Pick device trace")?;
        if vector.len() != batch.len() * VECTOR || trace.len() != batch.len() * BLOCKS * stride {
            return Err("unexpected Pick tape shape".into());
        }
        for (row, prefix) in batch.iter().enumerate() {
            for &word in &vector[row * VECTOR..(row + 1) * VECTOR] {
                tape.write_all(&word.to_le_bytes())?;
            }
            for block in 0..BLOCKS {
                let rec = &trace[(row * BLOCKS + block) * stride..][..stride];
                if rec[5..].iter().any(|&word| word != ABSENT) {
                    return Err("unexpected nonempty Pick trace remainder".into());
                }
                for &word in &rec[..5] {
                    tape.write_all(&word.to_le_bytes())?;
                }
            }
            let pick = trace[row * BLOCKS * stride];
            let successor = if pick == ABSENT {
                ABSENT
            } else {
                *prefix
                    .get(pick as usize + 1)
                    .ok_or("Pick successor outside causal prefix")?
            };
            tape.write_all(&successor.to_le_bytes())?;
        }
    }
    tape.flush()?;
    println!(
        "{}",
        json!({"source":TRUNK,"snapshot_sha256":hash(&snapshot_bytes),
        "transport_word":3,"rows":rows.len(),"geometry_fault":swap})
    );
    Ok(())
}

fn exact_neighbor(prefix: &[u32]) -> Option<(usize, u32)> {
    let t = prefix.len().checked_sub(1)?;
    let query = prefix.get(t.checked_sub(1)?..=t)?;
    (1..t)
        .rev()
        .find(|&p| &prefix[p - 1..=p] == query)
        .map(|p| (p, prefix[p + 1]))
}

fn verify(
    seed: &Path,
    d11: &Path,
    native: &Path,
    input: &Path,
    pick: &Path,
    count: &Path,
    out: &Path,
) -> Result<()> {
    let model = trunk(seed)?;
    let counted = IntegerDiscreteModel::load(d11)?;
    let mixer = counted.count_mixer().ok_or("missing context mixer")?;
    let native = fs::read(native)?;
    let carrier = CARRIERS
        .iter()
        .find(|row| row.0 == counted.artifact_sha256())
        .ok_or("unregistered count D11")?;
    if mixer.words() != native
        || hash(&native) != carrier.1
        || mixer.context_order() != carrier.2
        || fs::metadata(d11)?.len() != carrier.3
    {
        return Err("count D11 native words or carrier shape mismatch".into());
    }
    let rows = prefixes(input, model.shape().context as usize)?;
    let pick_bytes = fs::read(pick)?;
    let count_bytes = fs::read(count)?;
    let count_header = rows.len() * (DEPTH + 2) * 8;
    if rows.len() * PICK_WORDS * 4 != pick_bytes.len()
        || count_bytes.len() != count_header + rows.len() * VOCAB
    {
        return Err("incorrect full-word diagnostic tape sizes".into());
    }
    let mut records = Vec::with_capacity(rows.len());
    let mut spans: BTreeMap<usize, Vec<(usize, u32)>> = BTreeMap::new();
    for (i, prefix) in rows.iter().enumerate() {
        let rust = model.observe_identity_pick(prefix)?;
        let mut want = Vec::with_capacity(PICK_WORDS * 4);
        for word in rust.vector {
            want.extend(word.to_le_bytes());
        }
        for word in rust.block_words.iter().flatten() {
            want.extend(word.to_le_bytes());
        }
        want.extend(rust.successor.unwrap_or(ABSENT).to_le_bytes());
        if pick_bytes[i * PICK_WORDS * 4..(i + 1) * PICK_WORDS * 4] != want {
            return Err(format!("identity Pick full-word mismatch at row {i}").into());
        }
        let position = rust.block_words[0][0];
        if let Some(id) = rust.successor {
            if id <= 1 {
                return Err("Pick successor outside eligible original IDs".into());
            }
            spans.entry(prefix.len()).or_default().push((i, id));
        }

        let support = mixer
            .context_support(prefix)
            .ok_or("missing context support")?;
        let winner = mixer.top1_for_prefix(prefix);
        let mass = mixer.mass_q32_for_prefix(prefix, winner);
        let routes = mixer.context_routes(prefix);
        let count_words: Vec<_> = [u64::from(winner), mass]
            .into_iter()
            .chain(routes)
            .flat_map(u64::to_le_bytes)
            .collect();
        let at = i * count_words.len();
        let support_at = count_header + i * VOCAB;
        if support[usize::from(winner)] == 0
            || count_bytes[at..at + count_words.len()] != count_words
            || count_bytes[support_at..support_at + VOCAB] != support
        {
            return Err(format!(
                "suffix support, routes or incumbent full-word mismatch at row {i}"
            )
            .into());
        }
        let exact = exact_neighbor(prefix);
        records.push(
            json!({"ordinal":i,"length":prefix.len(),"routes":routes,"winner":winner,
            "winner_mass_q32":mass,"support_count":support.iter().filter(|&&s|s!=0).count(),
            "support_sha256":hash(&support),"exact_neighbor":exact,"pick_successor":rust.successor,
            "pick_position":(position != ABSENT).then_some(position),
            "pick_distance":(position != ABSENT).then_some(rust.block_words[0][1]),
            "pick_ties":rust.block_words[0][2],"pick_blocks":rust.block_words,
            "shuffled_successor":null,"shuffled_donor":null}),
        );
    }
    let mut singleton = 0;
    let mut unchanged_rotations = 0;
    for span in spans.values() {
        singleton += usize::from(span.len() == 1);
        for (i, &(receiver, original)) in span.iter().enumerate() {
            let (donor, id) = span[(i + 1) % span.len()];
            unchanged_rotations += usize::from(original == id);
            records[receiver]["shuffled_successor"] = json!(id);
            records[receiver]["shuffled_donor"] = json!(donor);
        }
    }
    fs::write(
        out,
        serde_json::to_vec(&json!({"schema":"uor-r4.ft-dev-pick-support/1",
        "source":TRUNK,"carrier":carrier.0,"native_sha256":carrier.1,
        "carrier_bytes":carrier.3,"native_bytes":native.len(),"depth":carrier.2,
        "transport_word":3,"unigram_min_weight":8,"uniform_weight":0,
        "prefix_sha256":hash(&fs::read(input)?),"pick_sha256":hash(&pick_bytes),
        "count_sha256":hash(&count_bytes),"rows":rows.len(),"support_strata":spans.len(),
        "singleton_strata":singleton,"unchanged_rotations":unchanged_rotations,"records":records}))?,
    )?;
    Ok(())
}

const ARMS: [&str; 5] = ["count_top1", "s2", "s2_exact", "s2_pick", "s2_rotated"];

#[derive(Default)]
struct Paired {
    rows: usize,
    baseline: usize,
    hits: usize,
    gains: usize,
    harms: usize,
}

impl Paired {
    fn add(&mut self, reachable: bool, correct_baseline: bool) {
        self.rows += 1;
        self.baseline += usize::from(correct_baseline);
        self.hits += usize::from(reachable);
        self.gains += usize::from(reachable && !correct_baseline);
        self.harms += usize::from(!reachable && correct_baseline);
    }

    fn json(&self) -> Value {
        let net = self.gains as i64 - self.harms as i64;
        let discordant = 4 * (self.gains + self.harms);
        let root = discordant.isqrt();
        let band = root + usize::from(root * root < discordant);
        json!({"rows":self.rows,"baseline_correct":self.baseline,"hits":self.hits,
            "gains":self.gains,"harms":self.harms,"net":net,"paired_band":band,
            "verdict":if net < -(band as i64) {"below"} else if net > band as i64 {"above"} else {"in_band"}})
    }
}

fn grade(
    dir: &Path,
    d11: &Path,
    input: &Path,
    count: &Path,
    blind: &Path,
    blind_sha: &str,
    out: &Path,
) -> Result<()> {
    let model = IntegerDiscreteModel::load(d11)?;
    let mixer = model.count_mixer().ok_or("missing count model")?;
    let carrier = CARRIERS
        .iter()
        .find(|row| row.0 == model.artifact_sha256())
        .ok_or("unregistered graded carrier")?;
    let capture = fs::read(blind)?;
    let count = fs::read(count)?;
    let input_sha = hash(&fs::read(input)?);
    let receipt: Value = serde_json::from_slice(&capture)?;
    let records = receipt["records"].as_array().ok_or("no blind records")?;
    let count_header = ROWS * (DEPTH + 2) * 8;
    if hash(&capture) != blind_sha
        || receipt["carrier"] != carrier.0
        || receipt["count_sha256"] != hash(&count)
        || receipt["prefix_sha256"] != input_sha
        || records.len() != ROWS
        || count.len() != count_header + ROWS * VOCAB
    {
        return Err("uncaptured, shifted or tampered blind support receipt".into());
    }
    let (ids, mask, meta) = fixture(dir)?;
    let (prior_name, prior_sha) = match carrier.2 {
        3 => (
            "1-depth3-equal.gpu",
            "4327e8aa37bdcc565eaa80e5557861bba0b68100718d8703cfd14e0603abc0fc",
        ),
        4 => (
            "100-depth4-equal.gpu",
            "f6a87aaba5129dba59931c24b3dd589b37a4169d95519a1339faa722e872fa1f",
        ),
        _ => return Err("no independent incumbent tape for carrier".into()),
    };
    let prior = fs::read(dir.join(prior_name))?;
    if prior.len() != ids.len() * 61 || hash(&prior) != prior_sha {
        return Err("unexpected frozen count-only predecessor tape".into());
    }
    let current = prefixes(input, 192)?;
    let original_path = input.with_file_name("canonical-prefixes.json");
    let original_bytes = fs::read(original_path)?;
    let original: Vec<Vec<u32>> = serde_json::from_slice(&original_bytes)?;
    let reference = fs::read(dir.join("baseline.jsonl"))?;
    if hash(&reference) != REFERENCE || current.len() != ROWS || original.len() != ROWS {
        return Err("frozen relation reference or prefix cardinality mismatch".into());
    }
    let reference = std::str::from_utf8(&reference)?;
    if reference.lines().count() != ROWS {
        return Err("incorrect relation reference row count".into());
    }
    let mut canonical = Sha256::new();
    let mut by_stratum: BTreeMap<&str, [Paired; ARMS.len()]> = BTreeMap::new();
    let mut grades = BufWriter::new(File::create(out)?);
    let mut response_pos = 0;
    let mut absent = 0;
    let mut all_zero_trits_with_pick = 0;
    let mut exact_extra = 0;
    let mut pick_extra = 0;
    let mut rotated_extra = 0;
    let mut pick_not_exact = 0;
    let mut exact_not_pick = 0;
    let mut max_next_support = 0;
    let mut distance_hist = BTreeMap::<u64, usize>::new();
    let mut support_total = 0;
    let mut values_equal = 0;
    let mut overlap = 0;
    let mut same_correct = 0;
    let mut reset_spans = 0;
    for (i, (meta, line)) in meta.iter().zip(reference.lines()).enumerate() {
        let rec = &records[i];
        let base: Value = serde_json::from_str(line)?;
        let position = field(meta, "target_position")?;
        let len = field(meta, "prefix_len")?;
        let target = field(meta, "target_token")?;
        if field(meta, "ordinal")? != i
            || field(rec, "ordinal")? != i
            || field(&base, "ordinal")? != i
            || field(&base, "status")? != 0
            || field(&base, "target")? != target
            || target == 0
            || target >= VOCAB
            || position >= ids.len()
            || ids[position] as usize != target
            || len > position
            || mask[position] != 1
            || record_prefix(&original[i]) != current[i]
            || original[i]
                .iter()
                .copied()
                .ne(ids[position - len..position].iter().copied().map(u32::from))
        {
            return Err(
                format!("misaligned frozen target or captured causal prefix at row {i}").into(),
            );
        }
        canonical.update((i as u64).to_le_bytes());
        canonical.update((position as u64).to_le_bytes());
        canonical.update((target as u32).to_le_bytes());
        canonical.update((len as u32).to_le_bytes());
        for &token in &original[i] {
            canonical.update(token.to_le_bytes());
        }
        reset_spans += usize::from(original[i].len() != current[i].len());
        if position == 0 || mask[position - 1] != 1 || ids[position - 1] == 1 {
            response_pos = 0;
        } else {
            response_pos += 1;
        }
        let response_stratum = match response_pos {
            0 => "response_0",
            1 => "response_1",
            2 => "response_2",
            _ => "response_3_plus",
        };
        let start = count_header + i * VOCAB;
        let support = &count[start..start + VOCAB];
        if rec["support_sha256"] != hash(support) {
            return Err(format!("support row digest mismatch at row {i}").into());
        }
        let support_size = field(rec, "support_count")?;
        support_total += support_size;
        max_next_support = max_next_support.max(support_size);
        let suffix = support[target] != 0;
        let exact = rec["exact_neighbor"].get(1).and_then(Value::as_u64);
        let pick = rec["pick_successor"].as_u64();
        let rotated = rec["shuffled_successor"].as_u64();
        let exact_hit = exact == Some(target as u64);
        let pick_hit = pick == Some(target as u64);
        let rotated_hit = rotated == Some(target as u64);
        if let Some(distance) = rec["pick_distance"].as_u64() {
            *distance_hist.entry(distance).or_default() += 1;
        }
        absent += usize::from(pick.is_none());
        all_zero_trits_with_pick += usize::from(
            pick.is_some()
                && rec["pick_blocks"]
                    .as_array()
                    .is_some_and(|blocks| blocks.iter().all(|block| block[3] == 1)),
        );
        values_equal += usize::from(pick.is_some() && pick == rotated);
        overlap += usize::from(pick.is_some() && exact.is_some());
        same_correct += usize::from(pick_hit && exact_hit);
        exact_extra += usize::from(!suffix && exact_hit);
        pick_extra += usize::from(!suffix && pick_hit);
        rotated_extra += usize::from(!suffix && rotated_hit);
        pick_not_exact += usize::from(!suffix && pick_hit && !exact_hit);
        exact_not_pick += usize::from(!suffix && exact_hit && !pick_hit);

        let winner = field(rec, "winner")?;
        let prior_at = ids.len() * 13 + position * (DEPTH + 2) * 8;
        let count_at = i * (DEPTH + 2) * 8;
        if prior[prior_at..prior_at + (DEPTH + 2) * 8]
            != count[count_at..count_at + (DEPTH + 2) * 8]
        {
            return Err(
                format!("frozen predecessor winner, mass or routes diverge at row {i}").into(),
            );
        }
        let base_correct = field(&base, "prediction")? == target;
        let hits = [
            target == winner,
            suffix,
            suffix || exact_hit,
            suffix || pick_hit,
            suffix || rotated_hit,
        ];
        let strata = [
            Some("overall"),
            Some(response_stratum),
            (response_pos <= 2).then_some("early_0_to_2"),
            Some(if target == 1 { "eos" } else { "non_eos" }),
        ];
        for name in strata.into_iter().flatten() {
            for (score, hit) in by_stratum.entry(name).or_default().iter_mut().zip(hits) {
                score.add(hit, base_correct);
            }
        }
        let masses = mixer
            .context_masses(&current[i], target as u16)
            .ok_or("missing graded expert words")?;
        let best_count = masses[..=carrier.2]
            .iter()
            .copied()
            .max()
            .ok_or("no count experts")?;
        let max_count = 8 * u64::from(masses[0]) + 248 * u64::from(best_count);
        let max_pick =
            8 * u64::from(masses[0]) + 248 * u64::from(if pick_hit { 1 << 24 } else { best_count });
        serde_json::to_writer(
            &mut grades,
            &json!({"ordinal":i,"target":target,"canonical_prefix_len":original[i].len(),
            "record_prefix_len":current[i].len(),
            "reference":field(&base,"prediction")?,"count_winner":winner,"support_arms":hits,
            "count_masses_q24":masses,"expert_mass_bound_q32":[max_count,max_pick],
            "response_pos":response_pos,"eligible_position":position}),
        )?;
        grades.write_all(b"\n")?;
    }
    grades.flush()?;
    let overall = by_stratum.get("overall").ok_or("empty grade")?;
    if format!("{:x}", canonical.finalize()) != PANE
        || overall[0].baseline != 7002
        || overall[0].hits != carrier.4
    {
        return Err("frozen panel, incumbent or historical M1 floor mismatch".into());
    }
    let graded: BTreeMap<_, _> = by_stratum
        .iter()
        .map(|(&name, scores)| {
            let arms: BTreeMap<_, _> = ARMS
                .into_iter()
                .zip(scores.iter().map(Paired::json))
                .collect();
            (name, arms)
        })
        .collect();
    println!(
        "{}",
        json!({"rows":ROWS,"panel":PANE,"baseline":REFERENCE,"carrier":carrier.0,
        "canonical_prefix_manifest":hash(&original_bytes),"reset_spans":reset_spans,
        "carrier_bytes":carrier.3,"depth":carrier.2,"native":carrier.1,"frozen_incumbent_tape":prior_sha,
        "blind_capture":blind_sha,"target_blind_count_support":receipt["count_sha256"],
        "target_blind_pick_tape":receipt["pick_sha256"],"rotated_strata":receipt["support_strata"],
        "singleton_strata":receipt["singleton_strata"],"unchanged_rotations":values_equal,
        "no_raw_pick":absent,"raw_pick_all_gates_zero":all_zero_trits_with_pick,
        "pick_distance_histogram":distance_hist,"max_count_support":max_next_support,
        "sum_count_support":support_total,"both_pick_and_exact_present":overlap,
        "both_predict_target":same_correct,"exact_additions":exact_extra,"pick_additions":pick_extra,
        "rotation_additions":rotated_extra,"pick_only_over_exact_and_s2":pick_not_exact,
        "exact_only_over_pick_and_s2":exact_not_pick,"grades_by_stratum":graded})
    );
    Ok(())
}

fn finite(seed: &Path, out: &Path) -> Result<()> {
    let model = trunk(seed)?;
    let context = model.shape().context as usize;
    let support = [true, false, true, true, true, false];
    // A tied unigram leader, two sparse suffix experts, and one raw Pick.
    // Each distribution has exactly 17 arbitrary units, allowing an exact
    // exhaustive Q8 winner search with no normalization or float.
    let scores = [
        [5, 5, 3, 2, 1, 1],
        [0, 0, 0, 17, 0, 0],
        [0, 0, 0, 4, 13, 0],
        [0, 0, 17, 0, 0, 0],
    ];
    let mut admissible = 0;
    let mut winners = [false; 6];
    for unigram in 8..=256 {
        for suffix1 in 0..=256 - unigram {
            for suffix2 in 0..=256 - unigram - suffix1 {
                let raw_pick = 256 - unigram - suffix1 - suffix2;
                let w = [unigram, suffix1, suffix2, raw_pick];
                let mass = |id: usize| {
                    w.iter()
                        .zip(scores)
                        .map(|(&weight, lane)| weight * lane[id])
                        .sum::<u32>()
                };
                let winner = (0..6)
                    .max_by_key(|&id| (mass(id), std::cmp::Reverse(id)))
                    .ok_or("finite empty vocabulary")?;
                if !support[winner] {
                    return Err("finite exhaustive oracle escaped registered support".into());
                }
                winners[winner] = true;
                admissible += 1;
            }
        }
    }
    let maximum: Vec<_> = std::iter::once(0)
        .chain((1..context).map(|pos| (pos % 31 + 2) as u32))
        .collect();
    let rows = vec![
        vec![0],
        vec![2],
        vec![3],
        vec![0, 2],
        vec![0, 2, 3],
        vec![0, 2, 2, 2, 2, 2],
        vec![2, 3, 4, 2, 3, 5, 2, 3],
        maximum,
    ];
    let tie = model.observe_identity_pick(&rows[5])?;
    if tie.block_words[0][..3] != [4, 0, 3]
        || exact_neighbor(&rows[6]) != Some((4, 5))
        || record_prefix(&[0, 2, 3, 1, 2, 3]) != [2, 3]
        || !record_prefix(&[0, 2, 1]).is_empty()
        || record_prefix(&[0, 2, 1, 0, 3]) != [0, 3]
        || model.observe_identity_pick(&[0, 2, 1]).is_ok()
        || model.observe_identity_pick(&[2, 0, 3]).is_ok()
        || model.observe_identity_pick(&[0, VOCAB as u32]).is_ok()
        || model.observe_identity_pick(&vec![2; context + 1]).is_ok()
    {
        return Err("finite boundary, latest tie or exact original-ID failure".into());
    }
    write_prefixes(&rows, out)?;
    let mut shifted = rows;
    shifted.rotate_left(1);
    fs::write(
        out.join("shifted-prefixes.json"),
        serde_json::to_vec(&shifted)?,
    )?;
    println!(
        "{}",
        json!({"admissible_weight_vectors_checked":admissible,"can_win":winners,
        "tie_position":4,"tie_distance":0,"tie_count":3,"max_prefix":context,
        "out_of_range_id_rejected":true,"committed_eos_rejected":true,
        "interior_bos_rejected":true,"overflowing_prefix_rejected":true})
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let path = |index: usize| Path::new(args[index].as_str());
    match (args.first().map(String::as_str), args.len()) {
        (Some("prefixes"), 3) => capture_prefixes(path(1),path(2)),
        (Some("gpu"), 4) => gpu(path(1),path(2),path(3),false),
        (Some("gpu-swap"), 4) => gpu(path(1),path(2),path(3),true),
        (Some("verify"), 8) => verify(path(1),path(2),path(3),path(4),path(5),path(6),path(7)),
        (Some("grade"), 8) => grade(path(1),path(2),path(3),path(4),path(5), &args[6],path(7)),
        (Some("finite"), 3) => finite(path(1),path(2)),
        _ => Err("expected prefixes FIXTURE OUT | gpu|gpu-swap SEED PREFIXES OUT | verify SEED MODEL NATIVE PREFIXES PICK SUFFIX OUT | grade FIXTURE MODEL PREFIXES SUFFIX BLIND BLIND_SHA OUT | finite SEED OUT".into()),
    }
}
