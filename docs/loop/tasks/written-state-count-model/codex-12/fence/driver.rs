use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};
use uor_r4_integer::{
    assertion_value::{self, AssertionValue},
    content_selector::{BOUNDARY, WIDTH},
    continuation_selector::Continuation,
    discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS},
    span_selector::{Bank, SpanChoice, CONT, START},
    value_selector::ExactBank,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tape {
    prefix: Vec<u32>,
}
#[derive(Deserialize)]
struct AnchorGrade {
    prefix: Vec<u32>,
    query_start: Option<u32>,
    supports: Vec<FactGrade>,
}
#[derive(Deserialize)]
struct FactGrade {
    fact_start: u32,
}
#[derive(Deserialize)]
struct LearnedRequest {
    schema: String,
    artifact_sha256: String,
    bare_sha256: String,
    tokenizer_sha256: String,
    rows: Vec<RequestRow>,
}
#[derive(Deserialize)]
struct RequestRow {
    prefix_sha256: String,
    query_start: Option<usize>,
    query_end: Option<usize>,
}
#[derive(Deserialize)]
struct PriorBank {
    model: String,
    span: String,
    token: String,
    delimiters: Vec<u32>,
}
#[derive(Deserialize)]
struct State<B> {
    bank: B,
}
#[derive(Deserialize)]
struct Trace {
    schema: String,
    artifact_sha256: String,
    bare_sha256: String,
    span_state_sha256: String,
    exact_state_sha256: String,
    rows: Vec<TraceRow>,
}
#[derive(Deserialize)]
struct TraceRow {
    prefix_sha256: String,
    frozen: SpanChoice,
    continuation: Continuation,
}
#[derive(Serialize)]
struct Decision {
    prefix_sha256: String,
    frozen: SpanChoice,
    old_source: Option<u32>,
    source: Option<u32>,
    continuation: Continuation,
    bounded: AssertionValue,
    words: [u32; 13],
}

fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn digest_ids(ids: &[u32]) -> String {
    hash(
        &ids.iter()
            .copied()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    )
}
fn packet(
    executable: &str,
    roots: &[u32],
    bank: &Bank,
    exact: &ExactBank,
    rows: &[(&[u32], SpanChoice, AssertionValue)],
) -> Result<()> {
    let mut words = vec![
        rows.len() as u32,
        bank.relations.len() as u32,
        exact.connectors.len() as u32,
    ];
    words.extend(ROLE_COPY_STOPS);
    words.extend(roots);
    words.extend(&bank.classes);
    words.extend(bank.relations.as_flattened());
    words.extend(exact.connectors.as_flattened());
    for (ids, frozen, _) in rows {
        let start = words.len();
        words.extend(*ids);
        words.resize(start + WIDTH, 0);
        words.extend(assertion_value::endpoints(ids, *frozen)?.unwrap_or([BOUNDARY; 2]));
    }
    let bytes: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing CUDA stdin")?
        .write_all(&bytes)?;
    let result = child.wait_with_output()?;
    if !result.status.success() || result.stdout.len() != rows.len() * 13 * 4 {
        return Err("invalid CUDA packet".into());
    }
    for (index, (got, (_, _, expected))) in result.stdout.chunks_exact(13 * 4).zip(rows).enumerate()
    {
        let want: Vec<_> = expected
            .words()
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect();
        if got != want {
            return Err(format!("CUDA/Rust assertion words differ at row {index}").into());
        }
    }
    Ok(())
}

fn apply(args: &[String]) -> Result<()> {
    let [executable, model, span, exact, bare, trace, output] = args else {
        return Err("apply DEVICE MODEL SPAN EXACT BARE TRACE OUTPUT".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(model))?;
    let sb = fs::read(span)?;
    let eb = fs::read(exact)?;
    let bb = fs::read(bare)?;
    let tb = fs::read(trace)?;
    let span_hash = hash(&sb);
    let exact_hash = hash(&eb);
    let bare_hash = hash(&bb);
    let trace_hash = hash(&tb);
    let trace: Trace = serde_json::from_slice(&tb)?;
    let tape: Vec<Tape> = serde_json::from_slice(&bb)?;
    if trace.schema != "uor-r4.entity-pair-chain/1"
        || trace.artifact_sha256 != model.artifact_sha256()
        || trace.bare_sha256 != bare_hash
        || trace.span_state_sha256 != span_hash
        || trace.exact_state_sha256 != exact_hash
        || trace.rows.len() != tape.len()
        || !(1..=8192).contains(&tape.len())
    {
        return Err("unbound trace, panel or frozen state".into());
    }
    let bank = serde_json::from_slice::<State<Bank>>(&sb)?.bank;
    let exact = serde_json::from_slice::<State<ExactBank>>(&eb)?.bank;
    bank.validate()?;
    exact.validate()?;
    let roots = model.packed_codebook_roots();
    let mut rows = Vec::with_capacity(tape.len());
    let mut raw = Vec::with_capacity(tape.len());
    for (tape, row) in tape.iter().zip(trace.rows) {
        let prefix = &tape.prefix;
        if prefix.is_empty()
            || prefix.len() > WIDTH
            || prefix.iter().any(|&id| id >= 4096)
            || row.prefix_sha256 != digest_ids(prefix)
        {
            return Err("invalid tape or row binding".into());
        }
        let bounded =
            assertion_value::decide(prefix, &roots, &bank, &exact, row.frozen, ROLE_COPY_STOPS)?;
        raw.push((prefix.as_slice(), row.frozen, bounded));
        rows.push(Decision {
            prefix_sha256: row.prefix_sha256,
            frozen: row.frozen,
            old_source: row.continuation.source,
            source: bounded.source,
            continuation: Continuation {
                source: bounded.source,
                ..bounded.continuation
            },
            bounded,
            words: bounded.words(),
        });
    }
    packet(executable, &roots, &bank, &exact, &raw)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "schema":"uor-r4.entity-pair-chain/1", "arm":"action",
            "artifact_sha256":model.artifact_sha256(), "bare_sha256":bare_hash,
            "span_state_sha256":span_hash, "exact_state_sha256":exact_hash,
            "trace_sha256":trace_hash, "cuda_parity_words":rows.len() * 13,
            "rows":rows
        }),
    )?;
    file.write_all(b"\n")?;
    Ok(())
}

fn anchors(args: &[String]) -> Result<()> {
    let [bare, grade, artifact, state, output] = args else {
        return Err("anchors BARE GRADE ARTIFACT_SHA STATE_SHA OUTPUT".into());
    };
    let bytes = fs::read(bare)?;
    let tape: Vec<Tape> = serde_json::from_slice(&bytes)?;
    let grade: Vec<AnchorGrade> = serde_json::from_slice(&fs::read(grade)?)?;
    if tape.len() != grade.len() {
        return Err("oracle row count".into());
    }
    let mut rows = vec![];
    for (tape, grade) in tape.iter().zip(grade) {
        if tape.prefix != grade.prefix {
            return Err("oracle prompt differs".into());
        }
        let (q, f) = match (grade.query_start, grade.supports.as_slice()) {
            (Some(q), [fact]) => (Some(q), Some(fact.fact_start)),
            _ => (None, None),
        };
        rows.push(json!({
            "q":q, "f":f, "boundary":null, "forced":null,
            "prefix_sha256":digest_ids(&tape.prefix),
        }));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "arm":"role", "artifact":artifact, "bare":hash(&bytes), "state":state, "rows":rows
        }),
    )?;
    file.write_all(b"\n")?;
    Ok(())
}

fn nominate(args: &[String]) -> Result<()> {
    let [bare, requests, span, prior, output] = args else {
        return Err("nominate BARE LEARNED_REQUESTS SPAN PRIOR_BANK OUTPUT".into());
    };
    let bytes = fs::read(bare)?;
    let tape: Vec<Tape> = serde_json::from_slice(&bytes)?;
    let request_bytes = fs::read(requests)?;
    let request: LearnedRequest = serde_json::from_slice(&request_bytes)?;
    let span_bytes = fs::read(span)?;
    let bank = serde_json::from_slice::<State<Bank>>(&span_bytes)?.bank;
    bank.validate()?;
    let prior_bytes = fs::read(prior)?;
    let prior: PriorBank = serde_json::from_slice(&prior_bytes)?;
    let bare_hash = hash(&bytes);
    if request.schema != "uor-r4.paired-role-decisions/1"
        || request.bare_sha256 != bare_hash
        || request.artifact_sha256 != prior.model
        || request.tokenizer_sha256 != prior.token
        || prior.span != hash(&span_bytes)
        || request.rows.len() != tape.len()
    {
        return Err("learned request, prior fact delimiter or tape mismatch".into());
    }
    let mut rows = Vec::with_capacity(tape.len());
    for (request, tape) in request.rows.into_iter().zip(&tape) {
        let prefix = &tape.prefix;
        if prefix.is_empty()
            || prefix.len() > WIDTH
            || prefix.iter().any(|&id| id >= 4096)
            || request.prefix_sha256 != digest_ids(prefix)
        {
            return Err("learned request row binding".into());
        }
        let class = |position: usize| bank.classes[prefix[position] as usize];
        let count = match (request.query_start, request.query_end) {
            (None, None) => 0,
            (Some(q), Some(end))
                if q < end
                    && end <= prefix.len()
                    && class(q) == START
                    && (q + 1..end).all(|p| class(p) == CONT)
                    && (end == prefix.len() || class(end) != CONT) =>
            {
                (1..q.saturating_sub(1))
                    .filter(|&f| {
                        if class(f) != START || !prior.delimiters.contains(&prefix[f - 1]) {
                            return false;
                        }
                        let next = f + (end - q);
                        next <= q && class(next) != CONT && prefix[f..next] == prefix[q..end]
                    })
                    .count()
            }
            _ => return Err("incomplete learned request endpoints".into()),
        };
        rows.push(json!({
            "prefix_sha256":request.prefix_sha256,
            "nominee":request.query_start,
            "query_end":request.query_end,
            "count":count,
        }));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer(
        &mut file,
        &json!({
            "arm":"both", "artifact_sha256":prior.model,
            "bare_sha256":bare_hash,"state_sha256":hash(&prior_bytes),
            "learned_requests_sha256":hash(&request_bytes), "rows":rows,
        }),
    )?;
    file.write_all(b"\n")?;
    Ok(())
}

fn fixture(executable: &str) -> Result<()> {
    let mut classes = vec![1; 4096];
    classes[16] = 0;
    classes[31] = 2;
    let roots = vec![0; 4096];
    let bank = Bank {
        classes,
        relations: vec![[1, 0, BOUNDARY, BOUNDARY, 0], [3, 0, 0, 0, 0]],
    };
    let exact = ExactBank {
        connectors: vec![[1, 30, BOUNDARY, BOUNDARY]],
    };
    let prefixes: [&[u32]; 7] = [
        &[29, 30, 32, 16, 44, 29],
        &[29, 30, 32, 31, 16, 29],
        &[29, 16, 32, 77, 44, 29],
        &[29, 30, 32, 77, 44, 29],
        &[29, 30, 32, 31, 31, 31, 31, 31, 31, 16, 29],
        &[29, 30, 32, 31, 31, 31, 31, 31, 31, 31, 16, 29],
        &[29, 30, 32, 16, 44, 29],
    ];
    let expected = [Some(2), Some(2), None, None, Some(2), None, None];
    let mut rows = vec![];
    for (index, (prefix, expected)) in prefixes.into_iter().zip(expected).enumerate() {
        let frozen = if index == 6 {
            SpanChoice::default()
        } else {
            SpanChoice {
                fact_start: Some(0),
                fact_end: Some(0),
                query_start: Some(prefix.len() as u32 - 1),
                query_end: Some(prefix.len() as u32 - 1),
                ..Default::default()
            }
        };
        let bounded =
            assertion_value::decide(prefix, &roots, &bank, &exact, frozen, ROLE_COPY_STOPS)?;
        if bounded.source != expected {
            return Err(format!("assertion finite {index}: {bounded:?}").into());
        }
        if let Some(start) = expected {
            let mut state = ExactCopyState::new(Some(start), prefix.len() as u32);
            let stop = bounded.fence.ok_or("missing finite fence")?;
            for &id in &prefix[start as usize..stop as usize] {
                let step = state.write(prefix, 77, ROLE_COPY_STOPS)?;
                if step.emitted != id || step.copied != 1 {
                    return Err("finite exact copy".into());
                }
                state = step.state;
            }
            let release = state
                .write(prefix, 77, ROLE_COPY_STOPS)?
                .finish_with_eos(true, 1);
            if release.emitted != 1 || release.release != 1 {
                return Err("finite delimiter must release with EOS".into());
            }
        }
        rows.push((prefix, frozen, bounded));
    }
    packet(executable, &roots, &bank, &exact, &rows)?;
    println!("PASS 91 device/Rust words; crossed stop, multi-piece value, empty value, no stop, seven-token EOS, eight-token cap and no-read");
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, exe] if cmd == "fixture" => fixture(exe),
        [cmd, rest @ ..] if cmd == "apply" => apply(rest),
        [cmd, rest @ ..] if cmd == "anchors" => anchors(rest),
        [cmd, rest @ ..] if cmd == "nominate" => nominate(rest),
        _ => Err(io::Error::other("expected apply, fixture, anchors or nominate").into()),
    }
}
