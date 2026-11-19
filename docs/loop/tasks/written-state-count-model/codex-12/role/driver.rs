use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
use uor_r4_integer::paired_role::{
    role_candidates, validate_role_ids, PairedRoleState, RoleSpan, ROLE_CONTEXT, ROLE_NIL,
    ROLE_OFFSETS, ROLE_ROOTS, ROLE_VOCAB,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}

#[derive(Deserialize)]
struct Grade {
    pair: String,
    prefix: Vec<u32>,
    prompt: String,
    kind: String,
    query_start: Option<usize>,
    query_end: Option<usize>,
    excluded_start: Option<usize>,
    excluded_end: Option<usize>,
}

fn hex(digest: impl AsRef<[u8]>) -> String {
    digest.as_ref().iter().map(|x| format!("{x:02x}")).collect()
}

fn words_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|&x| x.to_le_bytes()).collect()
}

fn digest(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

fn bare(bytes: &[u8]) -> Result<Vec<Bare>> {
    let tape: Vec<Bare> = serde_json::from_slice(bytes)?;
    if tape.is_empty() || tape.len() > 8192 {
        return Err("bad prompt row count".into());
    }
    for row in &tape {
        validate_role_ids(&row.prefix)?;
    }
    Ok(tape)
}

fn label_start(span: Option<RoleSpan>) -> u32 {
    span.map_or(ROLE_NIL, |s| s.start as u32)
}

fn role_sites(
    model: &IntegerDiscreteModel,
    tokenizer: &[u8],
    span: &[u8],
) -> Result<Vec<[u32; 2]>> {
    let tokenizer: Value = serde_json::from_slice(tokenizer)?;
    let vocab = tokenizer["model"]["vocab"]
        .as_object()
        .ok_or("missing vocabulary")?;
    if tokenizer["stack_sha256"] != model.artifact_sha256() {
        return Err("tokenizer model binding".into());
    }
    let classes: Vec<u32> =
        serde_json::from_value(serde_json::from_slice::<Value>(span)?["bank"]["classes"].clone())?;
    let roots = model.packed_codebook_roots();
    if classes.len() != ROLE_VOCAB || roots.len() != ROLE_VOCAB || classes.iter().any(|&c| c > 2) {
        return Err("invalid role geometry dimensions".into());
    }
    let mut sites = Vec::new();
    sites.extend(
        roots
            .iter()
            .zip(classes)
            .map(|(&root, class)| [root, class]),
    );
    for (text, id) in vocab {
        let id = id.as_u64().ok_or("invalid vocabulary ID")? as usize;
        if id >= ROLE_VOCAB {
            return Err("vocabulary ID out of range".into());
        }
        if let Some(case_fold) = vocab.get(&text.to_ascii_lowercase()) {
            let case_fold = case_fold.as_u64().ok_or("invalid fold vocabulary ID")? as usize;
            sites[id][0] = *roots
                .get(case_fold)
                .ok_or("fold vocabulary ID out of range")?;
        }
    }
    Ok(sites)
}

fn labels(
    tape: &[Bare],
    grade: &[Grade],
    sites: &[[u32; 2]],
) -> Result<Vec<[Option<RoleSpan>; 2]>> {
    if tape.len() != grade.len() {
        return Err("TRAIN grade length differs".into());
    }
    let mut groups = BTreeMap::<_, Vec<&Grade>>::new();
    let mut result = Vec::new();
    for (row, g) in tape.iter().zip(grade) {
        if row.prefix != g.prefix {
            return Err("TRAIN prefix and labels differ".into());
        }
        let spans = role_candidates(sites, &row.prefix)?
            .map(|(span, _)| span)
            .collect::<Vec<_>>();
        let endpoint = |start: Option<usize>, end: Option<usize>| -> Result<_> {
            match (start, end) {
                (None, None) => Ok(None),
                (Some(start), Some(end)) if spans.contains(&RoleSpan { start, end }) => {
                    Ok(Some(RoleSpan { start, end }))
                }
                _ => Err(format!(
                    "TRAIN endpoint {start:?}:{end:?} cannot be proposed on {}",
                    g.prompt
                )
                .into()),
            }
        };
        let q = endpoint(g.query_start, g.query_end)?;
        let e = endpoint(g.excluded_start, g.excluded_end)?;
        if q.is_some() && q == e {
            return Err("TRAIN class collision".into());
        }
        result.push([q, e]);
        groups.entry(&g.pair).or_default().push(g);
    }
    for rows in groups.values() {
        match rows.as_slice() {
            [g] if g.kind == "ordinary" => {}
            [a, b]
                if a.prefix != b.prefix
                    && a.query_start.is_some() == b.query_start.is_some()
                    && a.excluded_start.is_some() == b.excluded_start.is_some() => {}
            _ => return Err("TRAIN absent or inconsistent role-swap peer".into()),
        }
    }
    Ok(result)
}

fn launch(
    exe: &str,
    sites: &[[u32; 2]],
    cells: &[[u32; 4]],
    tape: &[Bare],
    labels: Option<&[[Option<RoleSpan>; 2]]>,
) -> Result<Vec<u32>> {
    let mut words = vec![
        u32::from(labels.is_none()),
        tape.len() as u32,
        cells.len() as u32,
    ];
    words.extend(sites.iter().flatten());
    words.extend(cells.iter().flatten());
    for (i, row) in tape.iter().enumerate() {
        words.extend(&row.prefix);
        words.resize(words.len() + ROLE_CONTEXT - row.prefix.len(), 0);
        words.push(row.prefix.len() as u32);
        let pair = labels.map_or([None, None], |l| l[i]);
        words.extend(pair.map(label_start));
    }
    let packet = words_bytes(&words);
    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing CUDA stdin")?
        .write_all(&packet)?;
    let output = child.wait_with_output()?;
    let length = if labels.is_some() {
        3 * ROLE_OFFSETS * ROLE_ROOTS
    } else {
        8 * tape.len()
    };
    if !output.status.success() || output.stdout.len() != 4 * length {
        return Err(format!(
            "bad CUDA response {} bytes, status {}",
            output.stdout.len(),
            output.status
        )
        .into());
    }
    Ok(output
        .stdout
        .chunks_exact(4)
        .map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))
        .collect())
}

fn fit(mode: &str, args: &[String]) -> Result<()> {
    let [exe, model_path, tok_path, span_path, train_bare, train_grade, out] = args else {
        return Err(
            "fit/preflight needs DEVICE MODEL TOKENIZER SPAN TRAIN_BARE TRAIN_GRADE OUT".into(),
        );
    };
    let model = IntegerDiscreteModel::load(Path::new(model_path))?;
    let model_bytes = fs::read(model_path)?;
    let tokenizer = fs::read(tok_path)?;
    let frozen_span = fs::read(span_path)?;
    let train = fs::read(train_bare)?;
    let grades = fs::read(train_grade)?;
    let tape = bare(&train)?;
    let sites = role_sites(&model, &tokenizer, &frozen_span)?;
    let labels = labels(
        &tape,
        &serde_json::from_slice::<Vec<Grade>>(&grades)?,
        &sites,
    )?;
    if mode == "preflight" {
        println!(
            "{} TRAIN rows with representable, disjoint role endpoints",
            tape.len()
        );
        return Ok(());
    }
    let output = launch(exe, &sites, &[], &tape, Some(&labels))?;
    let mut reference = vec![0; output.len()];
    for (row, [q, e]) in tape.iter().zip(labels) {
        for (span, keys) in role_candidates(&sites, &row.prefix)? {
            let role = if q == Some(span) {
                1
            } else if e == Some(span) {
                2
            } else {
                0
            };
            for key in keys {
                reference[3 * key as usize + role] += 1;
            }
        }
    }
    if reference != output {
        return Err("device/reference fitted count mismatch".into());
    }
    let cells = output
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, c)| c.iter().any(|&n| n != 0))
        .map(|(i, c)| [i as u32, c[0], c[1], c[2]])
        .collect();
    let state = PairedRoleState::new(
        [
            digest(&model_bytes),
            digest(&tokenizer),
            digest(&train),
            digest(&grades),
        ],
        sites,
        cells,
    )?;
    let bytes = state.to_bytes();
    if PairedRoleState::from_bytes(&bytes)?.to_bytes() != bytes {
        return Err("state pack/reload differs".into());
    }
    File::options()
        .write(true)
        .create_new(true)
        .open(out)?
        .write_all(&bytes)?;
    println!(
        "{} fitted words matched, {} occupied cells, {} bytes, bank SHA-256 {}",
        output.len(),
        state.cells().len(),
        bytes.len(),
        hex(digest(&bytes))
    );
    Ok(())
}

fn blind(args: &[String]) -> Result<()> {
    let [exe, model_path, tok_path, state_path, tape_path, out] = args else {
        return Err("blind needs DEVICE MODEL TOKENIZER BANK BARE OUT".into());
    };
    let model = digest(&fs::read(model_path)?);
    let tokenizer = digest(&fs::read(tok_path)?);
    let tape_bytes = fs::read(tape_path)?;
    let state_bytes = fs::read(state_path)?;
    let state = PairedRoleState::from_bytes(&state_bytes)?;
    if state.bindings()[..2] != [model, tokenizer] {
        return Err("role bank model/tokenizer binding differs".into());
    }
    let tape = bare(&tape_bytes)?;
    let output = launch(exe, state.sites(), state.cells(), &tape, None)?;
    let mut rows = Vec::new();
    for (i, (words, row)) in output.chunks_exact(8).zip(&tape).enumerate() {
        let decision = state.predict(&row.prefix)?;
        if words != decision.words() {
            return Err(format!("device/runtime decision mismatch row {i}").into());
        }
        let start = |s: Option<RoleSpan>| s.map(|s| s.start);
        let end = |s: Option<RoleSpan>| s.map(|s| s.end);
        rows.push(json!({
            "prefix_sha256":hex(digest(&words_bytes(&row.prefix))),
            "query_start":start(decision.requested),"query_end":end(decision.requested),
            "excluded_start":start(decision.excluded),"excluded_end":end(decision.excluded),
            "requested_score":decision.requested_score,"excluded_score":decision.excluded_score,
            "requested_runner_up":decision.requested_runner_up,"excluded_runner_up":decision.excluded_runner_up,
            "words":words,
        }));
    }
    let mut file = File::options().write(true).create_new(true).open(out)?;
    serde_json::to_writer(
        &mut file,
        &json!({"schema":"uor-r4.paired-role-decisions/1","artifact_sha256":hex(model),"tokenizer_sha256":hex(tokenizer),"bare_sha256":hex(digest(&tape_bytes)),"state_sha256":hex(digest(&state_bytes)),"rows":rows}),
    )?;
    writeln!(file)?;
    println!("{} signed route and exact-span words matched", output.len());
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((mode, rest)) if mode == "fit" || mode == "preflight" => fit(mode, rest),
        Some((mode, rest)) if mode == "blind" => blind(rest),
        Some((mode, rest)) if mode == "finite" && rest.len() == 1 => finite(&rest[0]),
        _ => Err("expected fit, preflight or blind".into()),
    }
}
fn finite(exe: &str) -> Result<()> {
    let mut sites = vec![[0, 0]; ROLE_VOCAB];
    for (i, site) in sites.iter_mut().enumerate() {
        site[0] = i as u32;
    }
    sites[4][1] = 1;
    sites[5][1] = 2;
    sites[9][1] = 1;
    let input = Bare {
        prefix: vec![0, 2, 3, 6, 4, 5, 6, 7, 8, 9, 10, 11],
    };
    let [q, e] = [
        RoleSpan { start: 4, end: 6 },
        RoleSpan { start: 9, end: 10 },
    ];
    let fit_tape = [input];
    let fitted = launch(exe, &sites, &[], &fit_tape, Some(&[[Some(q), Some(e)]]))?;
    let mut expected = vec![0; fitted.len()];
    for (s, label) in [(q, 1), (e, 2)] {
        let keys = role_candidates(&sites, &fit_tape[0].prefix)?
            .find(|(span, _)| *span == s)
            .ok_or("missing finite candidate")?
            .1;
        for key in keys {
            expected[3 * key as usize + label] = 1;
        }
    }
    if fitted != expected {
        return Err("finite fitted state mismatch".into());
    }
    let cells: Vec<_> = fitted
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, c)| c.iter().any(|&n| n != 0))
        .map(|(i, c)| [i as u32, c[0], c[1], c[2]])
        .collect();
    let state = PairedRoleState::new([[0; 32]; 4], sites, cells)?;
    let reloaded = PairedRoleState::from_bytes(&state.to_bytes())?;
    let predictions = [
        Bare {
            prefix: fit_tape[0].prefix.clone(),
        },
        Bare {
            prefix: vec![0, 2, 3],
        },
        Bare {
            prefix: vec![0, 2, 3, 6, 4, 5, 6, 7, 8, 2, 3, 6, 4, 5, 6, 7, 8],
        },
    ];
    let words = launch(exe, reloaded.sites(), reloaded.cells(), &predictions, None)?;
    for (row, actual) in predictions.iter().zip(words.chunks_exact(8)) {
        if actual != reloaded.predict(&row.prefix)?.words() {
            return Err("finite device/runtime parity mismatch".into());
        }
    }
    if words[..4] != [4, 6, 9, 10]
        || words[8..12] != [ROLE_NIL; 4]
        || words[16..18] != [ROLE_NIL; 2]
        || PairedRoleState::from_bytes(&state.to_bytes()[..100]).is_ok()
        || bare(br#"[{"prefix":[0,2],"query_start":1}]"#).is_ok()
    {
        return Err("finite role, null, tie, truncation or label boundary".into());
    }
    println!("PASS GPU and Rust fit/reload decisions, exact multi-piece endpoints, no candidate, tied candidates, truncated state and blind grade rejection");
    Ok(())
}
