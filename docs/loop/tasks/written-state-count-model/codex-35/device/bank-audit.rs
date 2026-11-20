//! Independent TRAIN packet, fitted bank and exposed writer checks on Wick.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use uor_r4_integer::discrete_d11::{ExactCopyState, ROLE_COPY_STOPS};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Span = [u32; 2];
const WIDTH: usize = 32;
const D26: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

#[derive(Deserialize)]
struct Input {
    row: u32,
    oracle: [u32; 4],
}

#[derive(Deserialize)]
struct Capture {
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    artifact_sha256: String,
    tokenizer_sha256: String,
    pair_rank_project: String,
    role_replay: Replay,
}

#[derive(Deserialize)]
struct Replay {
    contextual_sources: Vec<LegacyPool>,
    pair_rank_candidates: Vec<Pool>,
}

#[derive(Deserialize)]
struct LegacyPool {
    fact_span: Span,
    candidates: Vec<(Candidate, Eligibility)>,
}

#[derive(Deserialize)]
struct Eligibility {
    source: u32,
    eligible: bool,
}

#[derive(Deserialize)]
struct Pool {
    fact_span: Span,
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    span: Site,
    local: [i16; WIDTH],
    contextual: [i32; WIDTH],
    completed_source_range: Span,
}

#[derive(Deserialize, PartialEq)]
struct Site {
    start: u32,
    end: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Index {
    packet_row: usize,
    row: u32,
    fact: Span,
    candidates: Vec<Span>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Omission {
    row: u32,
    gold_source: Span,
    candidate_source_ranges: Vec<Span>,
}

#[derive(Debug, PartialEq)]
struct Pair {
    row: u32,
    fact: Span,
    gold: Span,
    rival: Span,
}

fn require(condition: bool, reason: impl FnOnce() -> String) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(reason().into())
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn verify_bytes(expected: &[u8], path: &Path) -> Result<()> {
    let actual = fs::read(path)?;
    if let Some(offset) = expected.iter().zip(&actual).position(|(a, b)| a != b) {
        return Err(format!("{} differs at byte {offset}", path.display()).into());
    }
    require(expected.len() == actual.len(), || {
        format!(
            "{} expected {} bytes, found {}",
            path.display(),
            expected.len(),
            actual.len()
        )
    })
}

fn capture_pool<'a>(
    capture: &'a Capture,
    input: &Input,
    geometry: &str,
) -> Result<Vec<&'a Candidate>> {
    let [query_start, query_end, start, end] = input.oracle;
    let [legacy] = capture.role_replay.contextual_sources.as_slice() else {
        return Err(format!("row {} does not have exactly one original fact", input.row).into());
    };
    let [pool] = capture.role_replay.pair_rank_candidates.as_slice() else {
        return Err(format!("row {} does not have exactly one projected fact", input.row).into());
    };
    let ids = &capture.prompt_ids;
    require(
        capture.generated_ids.is_empty()
            && capture.artifact_sha256 == D26
            && capture.tokenizer_sha256 == TOKENIZER
            && capture.pair_rank_project == geometry
            && pool.fact_span == [start, end]
            && legacy.fact_span == pool.fact_span
            && end <= query_start
            && ids.get(start as usize..end as usize).is_some_and(|fact| {
                !fact.is_empty() && Some(fact) == ids.get(query_start as usize..query_end as usize)
            })
            && legacy.candidates.len() == pool.candidates.len(),
        || {
            format!(
                "capture identity or supplied fact mismatch for row {}",
                input.row
            )
        },
    )?;
    let mut eligible = Vec::new();
    for ((old, score), candidate) in legacy.candidates.iter().zip(&pool.candidates) {
        let [source, fence] = candidate.completed_source_range;
        require(
            old.span == candidate.span
                && old.local == candidate.local
                && old.completed_source_range == candidate.completed_source_range
                && score.source == candidate.span.start
                && source >= end
                && source == candidate.span.start
                && source < candidate.span.end
                && candidate.span.end <= fence
                && fence < query_start
                && ids
                    .get(fence as usize)
                    .is_some_and(|id| ROLE_COPY_STOPS.contains(id))
                && (geometry != "fact_prefix" || old.contextual == candidate.contextual),
            || {
                format!(
                    "changed site, local coordinate or fence for row {}",
                    input.row
                )
            },
        )?;
        if score.eligible {
            eligible.push(candidate);
        }
    }
    require(
        eligible
            .windows(2)
            .all(|sites| sites[0].span.start < sites[1].span.start),
        || format!("unordered or repeated candidate site for row {}", input.row),
    )?;
    Ok(eligible)
}

struct Audit {
    pairs: Vec<Pair>,
    index: Vec<Index>,
    omissions: Vec<Omission>,
    context_pairs: usize,
    bank_bytes: usize,
}

fn audit_geometry(
    directory: &Path,
    bank_root: &Path,
    geometry: &str,
    inputs: &[Input],
    labels: &BTreeMap<u32, Span>,
) -> Result<Audit> {
    let mut packet = Vec::new();
    let mut prototypes = Vec::new();
    let mut pairs = Vec::new();
    let mut index = Vec::new();
    let mut omissions = Vec::new();
    let mut context_pairs = 0;

    for input in inputs {
        let capture: Capture = read_json(
            &directory
                .join("train.projection")
                .join(format!("{}.json", input.row)),
        )?;
        let candidates = capture_pool(&capture, input, geometry)?;
        let target = labels
            .get(&input.row)
            .ok_or("supervised TRAIN source missing")?;
        let gold = candidates
            .iter()
            .find(|c| &c.completed_source_range == target);
        let Some(gold) = gold.filter(|_| candidates.len() >= 2) else {
            omissions.push(Omission {
                row: input.row,
                gold_source: *target,
                candidate_source_ranges: candidates
                    .iter()
                    .map(|c| c.completed_source_range)
                    .collect(),
            });
            continue;
        };
        let [_, _, start, end] = input.oracle;
        index.push(Index {
            packet_row: index.len(),
            row: input.row,
            fact: [start, end],
            candidates: candidates
                .iter()
                .map(|c| [c.span.start, c.span.end])
                .collect(),
        });
        packet.extend([u32::try_from(candidates.len())?, gold.span.start]);
        for candidate in &candidates {
            packet.extend([candidate.span.start, candidate.span.end]);
            packet.extend(candidate.local.map(|x| i32::from(x) as u32));
            packet.extend(candidate.contextual.map(|x| x as u32));
        }
        for rival in candidates
            .iter()
            .filter(|candidate| candidate.span != gold.span)
        {
            pairs.push(Pair {
                row: input.row,
                fact: [start, end],
                gold: [gold.span.start, gold.span.end],
                rival: [rival.span.start, rival.span.end],
            });
            let local = gold
                .local
                .iter()
                .zip(rival.local)
                .flat_map(|(&a, b)| (i32::from(a) - i32::from(b)).to_le_bytes());
            let contextual = gold
                .contextual
                .iter()
                .zip(rival.contextual)
                .flat_map(|(&a, b)| (i64::from(a) - i64::from(b)).to_le_bytes());
            context_pairs += usize::from(gold.contextual != rival.contextual);
            prototypes.extend(local.chain(contextual));
        }
    }

    let packet = [0x3135_3344, 1, u32::try_from(index.len())?]
        .into_iter()
        .chain(packet)
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();
    verify_bytes(&packet, &directory.join("train.bin"))?;
    require(
        read_json::<Vec<Index>>(&directory.join("train.index.json"))? == index
            && read_json::<Vec<Omission>>(&directory.join("train.omitted.json"))? == omissions,
        || format!("{geometry} packet admission sidecars differ"),
    )?;
    require(!pairs.is_empty() && pairs.len() <= 8192, || {
        format!("{geometry} invalid prototype count")
    })?;
    let mut bank = b"D35PAIR1".to_vec();
    bank.extend(u32::try_from(pairs.len())?.to_le_bytes());
    bank.extend(prototypes);
    for mode in ["primary", "cold"] {
        verify_bytes(&bank, &bank_root.join(mode).join("bank.bin"))?;
    }
    Ok(Audit {
        pairs,
        index,
        omissions,
        context_pairs,
        bank_bytes: bank.len(),
    })
}

fn audit_train(train: &Path, projected: &Path, fit: &Path) -> Result<Value> {
    let inputs = fs::read_to_string(train.join("supplied.jsonl"))?
        .lines()
        .map(serde_json::from_str::<Input>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let label_rows: Vec<(u32, Span)> = read_json(&train.join("labels.json"))?;
    let labels = label_rows.iter().copied().collect::<BTreeMap<_, _>>();
    require(
        !inputs.is_empty()
            && inputs.windows(2).all(|rows| rows[0].row < rows[1].row)
            && inputs.len() == labels.len()
            && inputs.len() == label_rows.len(),
        || "duplicate or unpaired TRAIN identities".into(),
    )?;
    let fact = audit_geometry(
        &projected.join("fact-prefix"),
        &fit.join("fact-prefix"),
        "fact_prefix",
        &inputs,
        &labels,
    )?;
    let candidate = audit_geometry(
        &projected.join("candidate-prefix"),
        &fit.join("candidate-prefix"),
        "candidate_prefix",
        &inputs,
        &labels,
    )?;
    require(
        fact.index == candidate.index
            && fact.omissions == candidate.omissions
            && fact.pairs == candidate.pairs,
        || "geometries changed admitted rows or ordered TRAIN pair identities".into(),
    )?;
    require(
        fact.context_pairs == 0 && candidate.context_pairs > 0,
        || "TRAIN fact cancellation or candidate-prefix variation check failed".into(),
    )?;
    Ok(json!({
        "train_rows": inputs.len(),
        "train_rows_admitted_both": fact.index.len(),
        "train_rows_omitted_both": fact.omissions.len(),
        "identical_ordered_train_pairs": fact.pairs.len(),
        "fact_prefix_nonzero_contextual_pairs": fact.context_pairs,
        "candidate_prefix_nonzero_contextual_pairs": candidate.context_pairs,
        "bank_bytes_each": fact.bank_bytes,
        "bank_files_checked": 4,
        "prototype_bytes_equal": true
    }))
}

#[derive(Deserialize)]
struct ExposedGold {
    row: u32,
    gold_source: Option<u32>,
    original_ids: Vec<u32>,
    expected_tape: Vec<u32>,
}

#[derive(Deserialize)]
struct ExposedCapture {
    prompt_ids: Vec<u32>,
    role_replay: ExposedReplay,
}

#[derive(Deserialize)]
struct ExposedReplay {
    contextual_sources: Vec<LegacyPool>,
}

fn write(prompt: &[u32], source: Option<u32>) -> Result<Vec<[u32; 8]>> {
    const FALLBACK: u32 = 0;
    let mut state = ExactCopyState::new(source, u32::try_from(prompt.len())?);
    let mut trace = Vec::new();
    for _ in 0..=8 {
        let step = state
            .write(prompt, FALLBACK, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        trace.push(step.words());
        if step.copied == 0 {
            return Ok(trace);
        }
        state = step.state;
    }
    Err("writer did not release within its bound".into())
}

fn audit_writer(gold_path: &Path, capture_path: &Path, row: u32) -> Result<Value> {
    let gold = read_json::<Vec<ExposedGold>>(gold_path)?
        .into_iter()
        .find(|item| item.row == row)
        .ok_or("exposed gold row missing")?;
    let capture: ExposedCapture = read_json(capture_path)?;
    let onset = gold
        .gold_source
        .ok_or("writer fixture needs a known positive onset")?;
    require(capture.prompt_ids == gold.original_ids, || {
        "exposed row prompt differs from gold".into()
    })?;
    let pools = &capture.role_replay.contextual_sources;
    let pool = pools
        .iter()
        .find(|pool| {
            pool.candidates
                .iter()
                .any(|(candidate, score)| score.eligible && candidate.span.start == onset)
        })
        .ok_or("gold onset does not have an eligible candidate in the exposed fixture")?;
    let (rival, _) = pool
        .candidates
        .iter()
        .find(|(candidate, score)| score.eligible && candidate.span.start != onset)
        .ok_or("exposed fixture needs an admitted rival")?;
    let correct = write(&gold.original_ids, Some(onset))?;
    let adverse = write(&gold.original_ids, Some(rival.span.start))?;
    let disabled = write(&gold.original_ids, None)?;
    let correct_ids = correct.iter().map(|words| words[4]).collect::<Vec<_>>();
    let rival_ids = adverse.iter().map(|words| words[4]).collect::<Vec<_>>();
    let [start, end] = rival.completed_source_range;
    let expected_rival = gold
        .original_ids
        .get(start as usize..end as usize)
        .ok_or("rival range is outside the exposed prompt")?
        .iter()
        .copied()
        .chain([1])
        .collect::<Vec<_>>();
    require(
        correct_ids == gold.expected_tape
            && rival_ids == expected_rival
            && rival_ids != gold.expected_tape
            && disabled.len() == 1
            && disabled[0][4..] == [0, u32::MAX, 0, 0],
        || "exposed positive, adverse or no-source writer check failed".into(),
    )?;
    Ok(json!({
        "exposed_d34_row": row,
        "frozen_source_fact": pool.fact_span,
        "oracle_onset": onset,
        "oracle_emitted_ids": correct_ids,
        "oracle_writer_words": correct,
        "rival_onset": rival.span.start,
        "rival_emitted_ids": rival_ids,
        "rival_writer_words": adverse,
        "no_source_writer_words": disabled
    }))
}

fn main() -> Result<()> {
    let host = Command::new("hostname").arg("-s").output()?;
    require(host.status.success() && host.stdout == b"wick\n", || {
        "run this audit on Wick".into()
    })?;
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let output = match args.as_slice() {
        [mode, train, projected, fit] if mode == Path::new("train") => audit_train(train, projected, fit)?,
        [mode, gold, capture, row] if mode == Path::new("writer") => {
            audit_writer(gold, capture, row.to_str().ok_or("non-UTF8 row ID")?.parse()?)?
        }
        _ => return Err("usage: bank-audit train TRAIN_ONLY_DIR TRAIN_PROJECTION FIT_OUTPUT | writer EXPOSED_D34_GOLD EXPOSED_D34_CAPTURE ROW_ID".into()),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
