use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::ops::Range;
use std::process::{Command, Stdio};
use uor_r4_integer::paired_role::{
    validate_role_ids, PairedRoleState, RoleSpan, ROLE_CONTEXT, ROLE_NIL, ROLE_VOCAB,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}

#[derive(Deserialize)]
struct Region {
    prefix: Vec<u32>,
    start: usize,
    end: usize,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn words_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|&x| x.to_le_bytes()).collect()
}

fn launch(
    executable: &str,
    state: &PairedRoleState,
    rows: &[Bare],
    regions: &[Range<usize>],
) -> Result<Vec<u32>> {
    let mut packet = vec![rows.len() as u32, state.cells().len() as u32];
    packet.extend(state.sites().iter().flatten());
    packet.extend(state.cells().iter().flatten());
    for (row, region) in rows.iter().zip(regions) {
        packet.extend(&row.prefix);
        packet.resize(packet.len() + ROLE_CONTEXT - row.prefix.len(), 0);
        packet.extend([
            row.prefix.len() as u32,
            region.start as u32,
            region.end as u32,
        ]);
    }
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing CUDA stdin")?
        .write_all(&words_bytes(&packet))?;
    let output = child.wait_with_output()?;
    if !output.status.success() || output.stdout.len() != rows.len() * 32 {
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
        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        .collect())
}

fn run(executable: &str, args: &[String]) -> Result<()> {
    let [model_path, tokenizer_path, bank_path, tape_path, region_path, out] = args else {
        return Err("expected DEVICE MODEL TOKENIZER BANK BARE REGION|all OUT".into());
    };
    let model = Sha256::digest(fs::read(model_path)?);
    let tokenizer = Sha256::digest(fs::read(tokenizer_path)?);
    let tape_bytes = fs::read(tape_path)?;
    let bank = fs::read(bank_path)?;
    let state = PairedRoleState::from_bytes(&bank)?;
    if state.bindings()[..2] != [<[u8; 32]>::from(model), tokenizer.into()] {
        return Err("role bank model/tokenizer binding differs".into());
    }
    let tape: Vec<Bare> = serde_json::from_slice(&tape_bytes)?;
    if tape.is_empty() || tape.len() > 8192 {
        return Err("bad prompt row count".into());
    }
    for row in &tape {
        validate_role_ids(&row.prefix)?;
    }
    let regions = if region_path == "all" {
        tape.iter()
            .map(|row| 0..row.prefix.len())
            .collect::<Vec<_>>()
    } else {
        let regions: Vec<Region> = serde_json::from_slice(&fs::read(region_path)?)?;
        if regions.len() != tape.len()
            || regions
                .iter()
                .zip(&tape)
                .any(|(r, b)| r.prefix != b.prefix || r.start > r.end || r.end > b.prefix.len())
        {
            return Err("candidate region does not bind the original prompt tape".into());
        }
        regions.iter().map(|row| row.start..row.end).collect()
    };
    let output = launch(executable, &state, &tape, &regions)?;
    let mut rows = Vec::new();
    for (i, ((words, row), region)) in output.chunks_exact(8).zip(tape).zip(regions).enumerate() {
        let decision = state.predict_in_region(&row.prefix, region)?;
        if words != decision.words() {
            return Err(format!("CUDA/Rust candidate region mismatch at row {i}").into());
        }
        let start = |s: Option<RoleSpan>| s.map(|s| s.start);
        let end = |s: Option<RoleSpan>| s.map(|s| s.end);
        rows.push(json!({
            "prefix_sha256":digest(&words_bytes(&row.prefix)),
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
        &json!({
            "schema":"uor-r4.paired-role-decisions/1",
            "artifact_sha256":format!("{model:x}"),
            "tokenizer_sha256":format!("{tokenizer:x}"),
            "bare_sha256":digest(&tape_bytes),
            "state_sha256":digest(&bank),"rows":rows
        }),
    )?;
    writeln!(file)?;
    println!(
        "{} original span and signed role rank words matched",
        output.len()
    );
    Ok(())
}

fn finite(executable: &str) -> Result<()> {
    let mut sites = vec![[0, 0]; ROLE_VOCAB];
    sites[4] = [4, 1];
    sites[5] = [5, 2];
    sites[7] = [7, 1];
    // Evidence depends exclusively on three untouched context IDs after [4,6).
    let cells = vec![[3 * 65_537, 0, 10, 0], [4 * 65_537 + 7, 0, 10, 0]];
    let state = PairedRoleState::new([[0; 32]; 4], sites, cells)?;
    let regions = [0..8, 4..6, 4..5, 5..6, 4..4];
    let tape = regions.clone().map(|_| Bare {
        prefix: vec![0, 2, 3, 6, 4, 5, 6, 7],
    });
    let output = launch(executable, &state, &tape, &regions)?;
    for ((actual, row), region) in output.chunks_exact(8).zip(tape).zip(regions.clone()) {
        if actual != state.predict_in_region(&row.prefix, region)?.words() {
            return Err("CUDA/Rust finite region mismatch".into());
        }
    }
    if output[..2] != [4, 6]
        || output[..8] != output[8..16]
        || output[16..]
            .chunks_exact(8)
            .any(|row| row[..4] != [ROLE_NIL; 4])
        || state
            .predict_in_region(&[0, 4], Range { start: 2, end: 1 })
            .is_ok()
        || state.predict_in_region(&[0, 4], 0..3).is_ok()
    {
        return Err("finite whole-word mask or full-prefix context check failed".into());
    }
    println!("PASS full-prefix evidence, exact half-open multipart span, cut start/end rejection, empty mask, reversed and out-of-bounds refusal");
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [mode, executable] if mode == "finite" => finite(executable),
        [executable, args @ ..] => run(executable, args),
        [] => Err("missing device binary".into()),
    }
}
