//! Matched-byte native suffix counts. GPU counting, TRAIN fit, D11 reload and DEV replay.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use uor_r4_discrete::{
    ngram_count::{NgramAssembler, NgramOrderDigest, NgramTriple},
    ngram_count_device::NgramCounter,
    ngram_count_oracle::oracle_count,
};
use uor_r4_integer::{
    count_mixer::{
        self, CountMixer, CONTEXT_FLOOR_WEIGHT as FLOOR, CONTEXT_HEADER as HEADER,
        CONTEXT_MAX_ORDER as DEPTH, CONTEXT_ROW_BYTES as ROW_BYTES, VOCAB,
    },
    discrete_d11::IntegerDiscreteModel,
    lawful_likelihood::bits_q12,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Weights = [u32; DEPTH + 1];
type Row = (u64, u32);
const TRAIN: &str = "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460";
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TOKENIZER: &str = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const CARRIER_TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const ALLOCATIONS: [[usize; DEPTH]; 4] = [
    [100, 0, 0, 0],
    [75, 25, 0, 0],
    [50, 35, 15, 0],
    [40, 30, 20, 10],
];
const ARM_NAMES: [&str; 4] = [
    "depth1-full",
    "depth2-equal",
    "depth3-equal",
    "depth4-equal",
];

fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn verified(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    if sha(&bytes) != expected {
        return Err(format!("identity mismatch: {}", path.display()).into());
    }
    Ok(bytes)
}

fn ids_bytes(ids: &[u16]) -> Vec<u8> {
    ids.iter().flat_map(|id| id.to_le_bytes()).collect()
}

fn tokenizer_projection(bytes: &[u8]) -> Result<Value> {
    let source: Value = serde_json::from_slice(bytes)?;
    let mut model = source["model"].clone();
    let merges = model["merges"].as_array_mut().ok_or("missing BPE merges")?;
    for entry in merges {
        if let Some(pair) = entry.as_array() {
            let [left, right] = pair.as_slice() else {
                return Err("invalid BPE merge pair".into());
            };
            *entry = json!(format!(
                "{} {}",
                left.as_str().ok_or("invalid left merge")?,
                right.as_str().ok_or("invalid right merge")?
            ));
        }
    }
    Ok(
        json!({"model":model,"pre_tokenizer":source["pre_tokenizer"],
              "added_tokens":source["added_tokens"]}),
    )
}

fn identity(ids: &[u16]) -> Value {
    let mut hash = Sha256::new();
    let mut records = 0;
    for chunk in ids.chunks(1 << 18) {
        records += chunk.iter().filter(|&&v| v == 0).count();
        hash.update(ids_bytes(chunk));
    }
    json!({"sha256":format!("{:x}",hash.finalize()),"tokens":ids.len(),"records":records})
}

fn sample(path: &Path, fraction: usize) -> Result<(Vec<u16>, Vec<u16>)> {
    let mut reader = BufReader::new(fs::File::open(path)?);
    let mut header = [0; 64];
    reader.read_exact(&mut header)?;
    if &header[..4] != b"UORT" {
        return Err("bad TRAIN header".into());
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut left = reader.get_ref().metadata()?.len() - 64;
    if left % 2 != 0 {
        return Err("unaligned TRAIN words".into());
    }
    let mut buffer = vec![0; 1 << 20];
    let mut train = Vec::with_capacity((2_091_560_001u64 * fraction as u64 / 100) as usize);
    let mut fit = Vec::new();
    let (mut records, mut previous, mut tokens) = (0usize, 1u16, 0u64);
    let (mut for_train, mut for_fit) = (false, false);
    while left > 0 {
        let length = left.min(buffer.len() as u64) as usize;
        reader.read_exact(&mut buffer[..length])?;
        hash.update(&buffer[..length]);
        left -= length as u64;
        for word in buffer[..length].chunks_exact(2) {
            let id = u16::from_le_bytes([word[0], word[1]]);
            if usize::from(id) >= VOCAB || (id == 0) != (previous == 1) {
                return Err("invalid TRAIN framing".into());
            }
            if id == 0 {
                for_fit = records % 1000 == 1;
                for_train = !for_fit && records % (100 / fraction) == 0;
                records += 1;
            }
            if for_train {
                train.push(id);
            } else if for_fit {
                fit.push(id);
            }
            previous = id;
            tokens += 1;
        }
    }
    if format!("{:x}", hash.finalize()) != TRAIN
        || previous != 1
        || records != 519_580
        || tokens != 2_091_560_001
    {
        return Err("bad TRAIN identity".into());
    }
    Ok((train, fit))
}

struct Panel {
    ids: Vec<u16>,
    novel: Vec<u8>,
    bytes: Vec<u8>,
}

impl Panel {
    fn load(dir: &Path) -> Result<Self> {
        let words = verified(
            &dir.join("dev-tune.tokens.u16"),
            "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a",
        )?;
        let ids: Vec<_> = words[64..]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes([word[0], word[1]]))
            .collect();
        let novel = verified(
            &dir.join("dev-tune.novel.u8"),
            "7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af",
        )?;
        let bytes = verified(
            &dir.join("id-bytes.u8"),
            "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
        )?;
        if ids.len() != 10_810_087 || novel.len() != ids.len() || bytes.len() != VOCAB {
            return Err("bad query shape".into());
        }
        Ok(Self { ids, novel, bytes })
    }
}

fn compact(triple: &NgramTriple) -> Row {
    let depth = (triple.key >> 120) as u64;
    let digits = ((triple.key >> 72) as u64) & 0xffff_ffff_ffff;
    (
        (depth << 60) | (digits << 12) | u64::from(triple.next),
        triple.count,
    )
}

fn count_order(
    device: &NgramCounter,
    train: &[u16],
    order: usize,
    maximum: Option<usize>,
    oracle: bool,
) -> Result<(Vec<Row>, NgramOrderDigest)> {
    let start = Instant::now();
    let partition_budget = match train.len() {
        0..=30_000_000 => 1_000_000,
        30_000_001..=300_000_000 => 8_000_000,
        _ => 32_000_000,
    };
    let mut assembly = NgramAssembler::new(order as u32, false)?;
    let mut all = Vec::new();
    let mut best = BinaryHeap::<(Reverse<u32>, u64)>::new();
    for range in device.plan(order as u32, partition_budget)? {
        let partition = device.count_partition(order as u32, &range)?;
        let records = partition.triples.iter().map(compact);
        if let Some(maximum) = maximum {
            for (key, count) in records {
                let ranked = (Reverse(count), key);
                if best.len() < maximum {
                    best.push(ranked);
                } else if let Some(mut worst) = best.peek_mut() {
                    if ranked < *worst {
                        *worst = ranked;
                    }
                }
            }
        } else {
            all.extend(records);
        }
        assembly.push_owned(partition)?;
    }
    let (digest, _, _) = assembly.finish();
    if oracle {
        let (host, _, _) = oracle_count(train, order as u32)?;
        if host != digest {
            return Err(format!("CPU/CUDA canonical order-{order} stream mismatch").into());
        }
    }
    if maximum.is_some() {
        all = best
            .into_vec()
            .into_iter()
            .map(|(Reverse(count), key)| (key, count))
            .collect();
    }
    all.sort_unstable_by_key(|&(key, count)| (Reverse(count), key));
    eprintln!(
        "order {order}: {} triples, {} retained, {} ms",
        digest.grams,
        all.len(),
        start.elapsed().as_millis()
    );
    Ok((all, digest))
}

fn weights_for_order(depth: usize) -> Weights {
    let mut weights = [0; DEPTH + 1];
    weights[0] = FLOOR;
    for k in 0..depth {
        let begin = (256 - FLOOR) * k as u32 / depth as u32;
        let end = (256 - FLOOR) * (k + 1) as u32 / depth as u32;
        weights[k + 1] = end - begin;
    }
    weights
}

fn encode(uni: &[Row], levels: &[Vec<Row>], limits: &[usize; DEPTH]) -> Result<Vec<u8>> {
    let depth = limits
        .iter()
        .rposition(|&x| x != 0)
        .ok_or("no conditional rows")?
        + 1;
    let mut words = Vec::with_capacity(HEADER + limits.iter().sum::<usize>() * ROW_BYTES);
    for weight in weights_for_order(depth) {
        words.extend(weight.to_le_bytes());
    }
    let mut base = [0u32; VOCAB];
    for &(id, count) in uni {
        base[id as usize] = count;
    }
    for count in base {
        words.extend(count.to_le_bytes());
    }
    for (rows, &limit) in levels.iter().zip(limits) {
        let mut selected = rows
            .get(..limit)
            .ok_or("not enough distinct context rows")?
            .to_vec();
        selected.sort_unstable();
        for (key, count) in selected {
            words.extend(key.to_le_bytes());
            words.extend(count.to_le_bytes());
        }
    }
    Ok(words)
}

fn device_run(
    device: &Path,
    kind: &str,
    state: &Path,
    ids: &Path,
    out: &Path,
    stride: usize,
) -> Result<()> {
    if !Command::new(device)
        .arg(kind)
        .arg(state)
        .arg(ids)
        .arg(out)
        .arg(stride.to_string())
        .status()?
        .success()
    {
        return Err(format!("CUDA {kind} failed").into());
    }
    Ok(())
}

fn read_u32(words: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(
        words[index * 4..index * 4 + 4]
            .try_into()
            .expect("validated tape length"),
    )
}

fn read_u64(words: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(
        words[index * 8..index * 8 + 8]
            .try_into()
            .expect("validated tape length"),
    )
}

fn fit(
    device: &Path,
    out: &Path,
    name: &str,
    words: &mut [u8],
    ids: &[u16],
) -> Result<Vec<Weights>> {
    let model = CountMixer::from_context_words(words)?;
    let depth = model.context_order();
    let fit_state = out.join(format!("{name}-fit.words"));
    let tape = out.join(format!("{name}-fit.u32"));
    fs::write(&fit_state, &*words)?;
    device_run(device, "fit", &fit_state, &out.join("fit.u16"), &tape, 1)?;
    let updates = fs::read(tape)?;
    if updates.len() != 4 * (DEPTH + 1) * 4 {
        return Err("invalid fit update tape".into());
    }
    let experts: Vec<_> = ids
        .iter()
        .enumerate()
        .filter(|&(_, &id)| id != 0)
        .map(|(i, &id)| {
            model
                .context_masses(&ids[..i], id)
                .ok_or("missing context model")
        })
        .collect::<std::result::Result<_, _>>()?;
    let mut weights = weights_for_order(depth);
    let mut sequence = Vec::new();
    for pass in 0..4 {
        let mut totals = [0u64; DEPTH];
        for masses in &experts {
            let weighted: [u64; DEPTH + 1] =
                std::array::from_fn(|k| u64::from(weights[k]) * u64::from(masses[k]));
            let mixture: u64 = weighted.iter().sum();
            for (k, sum) in totals.iter_mut().enumerate().take(depth) {
                *sum += ((weighted[k + 1] << 16) + mixture / 2) / mixture;
            }
        }
        let amount: u64 = totals.iter().sum();
        if amount != 0 {
            let mut before = 0;
            let mut cumulative = 0;
            for k in 0..depth {
                cumulative += totals[k];
                let after = u64::from(256 - FLOOR - depth as u32) * cumulative / amount;
                weights[k + 1] = 1 + (after - before) as u32;
                before = after;
            }
        }
        let gpu: Weights = std::array::from_fn(|k| read_u32(&updates, pass * (DEPTH + 1) + k));
        if weights != gpu {
            return Err(format!("fit word mismatch in {name}, pass {pass}").into());
        }
        sequence.push(weights);
    }
    for (k, weight) in weights.into_iter().enumerate() {
        words[k * 4..k * 4 + 4].copy_from_slice(&weight.to_le_bytes());
    }
    Ok(sequence)
}

type DocBits = [u64; 2];
type Metric = [u64; 3];

fn score(
    device: &Path,
    out: &Path,
    name: &str,
    model: &CountMixer,
    panel: &Panel,
    fixture: bool,
) -> Result<(Value, Vec<DocBits>)> {
    let start = Instant::now();
    let raw_words = out.join(format!("{name}.reload.words"));
    let tape = out.join(format!("{name}.gpu"));
    let stride = if fixture { 1 } else { 2639 };
    fs::write(&raw_words, model.words())?;
    device_run(
        device,
        "score",
        &raw_words,
        &out.join("query.u16"),
        &tape,
        stride,
    )?;
    let raw = fs::read(tape)?;
    let n = panel.ids.len();
    let probes = n.div_ceil(stride);
    if raw.len() != n * 13 + probes * (DEPTH + 2) * 8 {
        return Err("invalid score tape".into());
    }
    let (mass, rest) = raw.split_at(n * 8);
    let (bits, rest) = rest.split_at(n * 4);
    let (hits, witnesses) = rest.split_at(n);
    let mut totals = [Metric::default(), Metric::default()];
    let mut documents: Vec<DocBits> = Vec::new();
    let mut fallbacks = [0u64; DEPTH];
    let mut minimum = u64::MAX;
    let mut examples = Vec::new();
    let (mut checked, mut correct) = (0u64, 0u64);
    for (i, &target) in panel.ids.iter().enumerate() {
        let mass = read_u64(mass, i);
        let bits = u64::from(read_u32(bits, i));
        let prefix = &panel.ids[..i];
        if target == 0 {
            if (mass, bits, hits[i], panel.novel[i]) != (0, 0, 0, 3) {
                return Err("BOS incorrectly scored".into());
            }
            documents.push([0; 2]);
            continue;
        }
        let expected = model.mass_q32_for_prefix(prefix, target);
        let routes = model.context_routes(prefix);
        let exact = routes.iter().enumerate().fold(0, |mask, (k, key)| {
            mask | (u8::from((key >> 48) as usize == k + 1) << k)
        });
        if mass == 0 || mass != expected || bits != bits_q12(mass << 16)? || hits[i] != exact {
            return Err(
                format!("target mass/bits/route occupancy mismatch in {name} at {i}").into(),
            );
        }
        for (k, missing) in fallbacks.iter_mut().enumerate().take(model.context_order()) {
            *missing += u64::from(exact & (1 << k) == 0);
        }
        minimum = minimum.min(mass);
        let bytes = u64::from(panel.bytes[usize::from(target)]);
        let doc = documents.last_mut().ok_or("target without BOS")?;
        for subset in 0..=usize::from(panel.novel[i] == 1) {
            totals[subset][0] += 1;
            totals[subset][1] += bytes;
            totals[subset][2] += bits;
            doc[subset] += bits;
        }
        if i % stride == 0 {
            let at = i / stride * (DEPTH + 2);
            let winner = model.top1_for_prefix(prefix);
            let maximum = model.mass_q32_for_prefix(prefix, winner);
            if read_u64(witnesses, at) != u64::from(winner)
                || read_u64(witnesses, at + 1) != maximum
                || routes
                    .iter()
                    .enumerate()
                    .any(|(k, &key)| read_u64(witnesses, at + k + 2) != key)
            {
                return Err(format!("winner/read-route replay mismatch in {name} at {i}").into());
            }
            checked += 1;
            correct += u64::from(winner == target);
            if examples.len() < 12 {
                examples.push(json!({"index":i,"recent":&prefix[prefix.len().saturating_sub(DEPTH)..],"target":target,"q32":mass,"winner":winner,"winner_q32":maximum,"routes":routes}));
            }
        }
    }
    let metrics = |row: Metric| json!({"targets":row[0],"bytes":row[1],"bits_q12":row[2]});
    Ok((
        json!({
            "overall":metrics(totals[0]),"novel":metrics(totals[1]),"fallbacks":fallbacks,
            "greedy_probes":checked,"greedy_correct_probes":correct,"minimum_observed_q32":minimum,
            "device_tape_sha256":sha(&raw),"score_and_parity_ms":start.elapsed().as_millis(),"examples":examples
        }),
        documents,
    ))
}

fn bootstrap(baseline: &[DocBits], changed: &[DocBits]) -> [[i64; 3]; 2] {
    // Results in integer parts per million. Identical policies resample paired documents.
    let mut rng = 0x8592_4d91_faa3_907eu64;
    let mut draws = [Vec::with_capacity(2000), Vec::with_capacity(2000)];
    let ppm = |before: u64, after: u64| {
        ((i128::from(before) - i128::from(after)) * 1_000_000 / i128::from(before)) as i64
    };
    for _ in 0..2000 {
        let (mut base, mut arm) = ([0u64; 2], [0u64; 2]);
        for _ in baseline {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let i = (rng % baseline.len() as u64) as usize;
            for subset in 0..2 {
                base[subset] += baseline[i][subset];
                arm[subset] += changed[i][subset];
            }
        }
        for subset in 0..2 {
            if base[subset] > 0 {
                draws[subset].push(ppm(base[subset], arm[subset]));
            }
        }
    }
    std::array::from_fn(|subset| {
        let values = &mut draws[subset];
        values.sort_unstable();
        let base = baseline.iter().map(|doc| doc[subset]).sum();
        let arm = changed.iter().map(|doc| doc[subset]).sum();
        if values.is_empty() {
            [0; 3]
        } else {
            [
                ppm(base, arm),
                values[values.len() / 40],
                values[values.len() * 39 / 40],
            ]
        }
    })
}

fn digest_json(digest: &NgramOrderDigest) -> Value {
    json!({"depth":digest.order,"targets":digest.targets,"triples":digest.grams,"contexts":digest.contexts,
           "triples_sha256":digest.triple_sha256,"contexts_sha256":digest.context_sha256})
}

fn finite_model(model: &CountMixer) -> Result<()> {
    let controls = [
        [0u16, 20, 3],
        [0, 21, 3],
        [0, 4095, 3],
        [1, 0, 3],
        [1, 1, 0],
    ];
    for prefix in controls {
        let masses: Vec<_> = (1..VOCAB as u16)
            .map(|id| model.mass_q32_for_prefix(&prefix, id))
            .collect();
        if masses.iter().sum::<u64>() != 1 << 32
            || masses.iter().any(|&mass| mass < u64::from(FLOOR))
            || model.mass_q32_for_prefix(&prefix, 0) != 0
        {
            return Err("support/conservation mismatch".into());
        }
        let mut expected: Vec<_> = (1..VOCAB as u16).collect();
        expected.sort_unstable_by_key(|&id| (Reverse(masses[usize::from(id) - 1]), id));
        if model.top8_for_prefix(&prefix) != expected[..8] {
            return Err("sparse top-eight mismatch".into());
        }
    }
    if model.context_order() >= 2 && model.context_routes(&[0u16, 4095, 3])[1] >> 48 != 1 {
        return Err("missing suffix did not read shorter context".into());
    }
    Ok(())
}

fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.len() != 5 {
        return Err("expected fixture|1|10|100 INPUTS DEVICE BASE_D11 NEW_OUTPUT_DIRECTORY".into());
    }
    let started = Instant::now();
    let fixture = arguments[0] == "fixture";
    let fraction = if fixture { 1 } else { arguments[0].parse()? };
    if ![1, 10, 100].contains(&fraction) {
        return Err("invalid TRAIN fraction".into());
    }
    let inputs = PathBuf::from(&arguments[1]);
    let device = PathBuf::from(&arguments[2]);
    let out = PathBuf::from(&arguments[4]);
    fs::create_dir(&out)?;
    let base = verified(Path::new(&arguments[3]), BASE)?;
    let query_tokenizer = verified(&inputs.join("tokenizer.json"), TOKENIZER)?;
    let carrier_tokenizer = verified(
        &inputs
            .parent()
            .ok_or("missing carrier tokenizer directory")?
            .join("tokenizer.json"),
        CARRIER_TOKENIZER,
    )?;
    let tokenizer_projection_value = tokenizer_projection(&query_tokenizer)?;
    if tokenizer_projection_value != tokenizer_projection(&carrier_tokenizer)? {
        return Err("corpus/carrier BPE rules or token IDs differ".into());
    }
    let tokenizer_projection_sha = sha(&serde_json::to_vec(&tokenizer_projection_value)?);
    let (train, held, panel) = if fixture {
        let mut train = Vec::new();
        for repetition in 0..40 {
            train.extend([0, 20, 3, 4, 8, 1]);
            train.extend([0, 21, 3, 5, 9, 1]);
            for offset in 0..6 {
                train.extend([
                    0,
                    50 + offset,
                    52 + repetition,
                    80 + repetition,
                    400 + offset,
                    1,
                ]);
            }
        }
        let held = vec![0, 20, 3, 4, 8, 1, 0, 21, 3, 5, 9, 1, 0, 4095, 3, 7, 1];
        let novel = held.iter().map(|&id| if id == 0 { 3 } else { 1 }).collect();
        let panel = Panel {
            ids: held.clone(),
            novel,
            bytes: vec![1; VOCAB],
        };
        (train, held, panel)
    } else {
        let (train, fit) = sample(&inputs.join("train.tokens.u16"), fraction)?;
        (train, fit, Panel::load(&inputs)?)
    };
    let train_identity = identity(&train);
    let fit_identity = identity(&held);
    let query_identity = identity(&panel.ids);
    fs::write(out.join("fit.u16"), ids_bytes(&held))?;
    fs::write(out.join("query.u16"), ids_bytes(&panel.ids))?;
    let device_count = NgramCounter::new(0, &train)?;
    let (uni, u) = count_order(&device_count, &train, 0, None, fraction == 1)?;
    let (first, b) = count_order(&device_count, &train, 1, None, fraction == 1)?;
    let budget = first.len();
    let second_limit = (fraction != 1).then_some(budget * 35 / 100);
    let (second, c) = count_order(&device_count, &train, 2, second_limit, fraction == 1)?;
    let (third, d) = count_order(
        &device_count,
        &train,
        3,
        Some(budget * 20 / 100),
        fraction == 1,
    )?;
    let (fourth, e) = count_order(
        &device_count,
        &train,
        4,
        Some(budget * 10 / 100),
        fraction == 1,
    )?;
    let counts = [u, b, c, d, e];
    drop(device_count);
    drop(train);
    let levels = [first, second, third, fourth];
    let profiles = ALLOCATIONS.iter().map(|allocation| {
        let mut limits = allocation.map(|portion| budget * portion / 100);
        limits[0] += budget - limits.iter().sum::<usize>();
        limits
    });
    let mut suite: Vec<_> = ARM_NAMES.into_iter().zip(profiles).collect();
    if fraction == 1 && levels[1].len() <= 5_000_000 {
        suite.push(("depth2-unpruned", [budget, levels[1].len(), 0, 0]));
    }
    let mut sizes = Vec::new();
    let mut results = Vec::new();
    let mut baseline = Vec::new();
    let mut baseline_words = None;
    for (name, limits) in suite {
        let mut words = encode(&uni, &levels, &limits)?;
        let fitting = fit(&device, &out, name, &mut words, &held)?;
        let packed = count_mixer::pack_context(&base, &words)?;
        let model_path = out.join(format!("{name}.d11"));
        let d11_sha = sha(&packed);
        let d11_bytes = packed.len();
        fs::write(&model_path, &packed)?;
        if fixture {
            let mut corrupt = packed;
            let index = corrupt.len() - base.len() - 1;
            corrupt[index] ^= 1;
            if IntegerDiscreteModel::from_bytes(&corrupt).is_ok() {
                return Err("corrupted context accepted".into());
            }
        }
        let loaded = IntegerDiscreteModel::load(&model_path)?;
        let model = loaded.count_mixer().ok_or("missing D11 count state")?;
        if model.words() != words
            || loaded.artifact_sha256() != d11_sha
            || loaded.tokenizer_sha256() != Some(CARRIER_TOKENIZER)
        {
            return Err("count D11 round-trip identity failure".into());
        }
        if fixture {
            finite_model(model)?;
            if name == "depth2-unpruned" {
                let (left, right) = (
                    loaded.evaluate_prefix(&[0, 20, 3])?,
                    loaded.evaluate_prefix(&[0, 21, 3])?,
                );
                if (left.prediction, right.prediction) != (4, 5) {
                    return Err("same last ID did not produce distinct winners through D11".into());
                }
            }
        }
        let (scores, docs) = score(&device, &out, name, model, &panel, fixture)?;
        let words_sha = sha(&words);
        if name == "depth1-full" {
            baseline = docs.clone();
            baseline_words = Some(words);
        }
        sizes.push(d11_bytes);
        let paired = bootstrap(&baseline, &docs);
        eprintln!(
            "{name}: {} bytes, weights {:?}; overall {} novel {}",
            d11_bytes,
            model.context_weights(),
            scores["overall"]["bits_q12"],
            scores["novel"]["bits_q12"]
        );
        results.push(json!({"arm":name,"limits":limits,"d11_bytes":d11_bytes,"state_bytes":model.words().len(),
                            "d11_sha256":d11_sha,"words_sha256":words_sha,"train_fit_q8":fitting,
                            "against_depth1_relative_saved_ppm_estimate_low_high":paired,"scores":scores}));
    }
    if sizes.len() < ARM_NAMES.len()
        || sizes[..ARM_NAMES.len()]
            .iter()
            .any(|&bytes| bytes != sizes[0])
    {
        return Err("matched physical D11 bytes mismatch".into());
    }
    let first_words = baseline_words.ok_or("missing order-one control")?;
    let mut legacy = Vec::with_capacity((VOCAB + 1) * 4 + budget * 8);
    legacy.extend(242u32.to_le_bytes());
    legacy.extend(&first_words[(DEPTH + 1) * 4..HEADER]);
    for word in first_words[HEADER..].chunks_exact(ROW_BYTES) {
        let packed = read_u64(word, 0);
        legacy.extend((((packed >> 48) & 0xfff) as u16).to_le_bytes());
        legacy.extend(((packed & 0xfff) as u16).to_le_bytes());
        legacy.extend(&word[8..]);
    }
    let legacy = CountMixer::from_words(&legacy)?;
    let mut old = [Metric::default(); 2];
    for (i, &target) in panel.ids.iter().enumerate().filter(|&(_, &id)| id != 0) {
        let q12 = bits_q12(legacy.mass_q32(panel.ids[i - 1], target) << 16)?;
        for metric in old.iter_mut().take(1 + usize::from(panel.novel[i] == 1)) {
            metric[0] += 1;
            metric[1] += u64::from(panel.bytes[usize::from(target)]);
            metric[2] += q12;
        }
    }
    if fraction == 1
        && !fixture
        && old
            != [
                [10_807_417, 21_928_475, 254_667_414_476],
                [7_566_174, 15_995_822, 193_248_297_371],
            ]
    {
        return Err(format!("legacy order-one comparator mismatch: {old:?}").into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "fixture":fixture,"train_fraction_percent":fraction,"train":train_identity,"fit":fit_identity,
        "query":query_identity,"novel_sha256":sha(&panel.novel),"corpus_tokenizer_sha256":TOKENIZER,
        "carrier_tokenizer_sha256":CARRIER_TOKENIZER,"shared_tokenizer_projection_sha256":tokenizer_projection_sha,
            "corpus_train_sha256":TRAIN,"base_d11_sha256":BASE,"train_count_oracle_pass":fraction == 1,
            "count_streams":counts.iter().map(digest_json).collect::<Vec<_>>(),
            "legacy_gpu_score_scope":"Historical GPU replay at 1%; current readback/denominator cross-check on host",
            "legacy_order1_weight242_host":{"overall_targets_bytes_bits_q12":old[0],"novel_targets_bytes_bits_q12":old[1]},
            "fixed_unigram_q8":FLOOR,"row_bytes":ROW_BYTES,"arms":results,"wall_ms":started.elapsed().as_millis()
        }))?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
