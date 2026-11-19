use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{RoleSpan, ROLE_CONTEXT},
};

#[derive(Deserialize)]
struct Fixtures {
    rows: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    seed: Option<u32>,
    ids: Vec<u32>,
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    name: String,
    span: [usize; 2],
    clause: [usize; 2],
}

fn put(out: &mut Vec<u8>, words: impl IntoIterator<Item = u32>) {
    for w in words {
        out.extend_from_slice(&w.to_le_bytes());
    }
}

fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let [_, seed, model_path, state_path, tokenizer_path, fixtures_path, target] = args.as_slice()
    else {
        return Err("usage: clause-coordinate-finite SEED PAIRED_D11 ORIGINAL_ROLE TOKENIZER FIXTURES NEW_OUTDIR".into());
    };
    let seed: u32 = seed.parse()?;
    let state_bytes = fs::read(state_path)?;
    let tok_bytes = fs::read(tokenizer_path)?;
    let model_bytes = fs::read(model_path)?;
    let model = IntegerDiscreteModel::load(Path::new(model_path))?;
    let replay = model.frozen_role_replay()?;
    let roles = replay.roles;
    if roles.to_bytes() != state_bytes
        || roles.bindings()[1] != <[u8; 32]>::from(Sha256::digest(&tok_bytes))
    {
        return Err("embedded role state or bound tokenizer differs".into());
    }
    let raw = fs::read(fixtures_path)?;
    let fixtures: Fixtures = serde_json::from_slice(&raw)?;
    let rows: Vec<_> = fixtures
        .rows
        .iter()
        .filter(|r| r.seed.is_none_or(|s| s == seed))
        .collect();
    let n: usize = rows.iter().map(|r| r.candidates.len()).sum();
    if n == 0 || n > 64 || rows.iter().any(|r| r.candidates.len() != 2) {
        return Err(
            "finite fixture requires two candidates per row and at most 64 candidates".into(),
        );
    }

    let mut packet = Vec::new();
    put(
        &mut packet,
        [0x31435043, n as u32, roles.cells().len() as u32],
    );
    for &[root, class] in roles.sites() {
        put(&mut packet, [root, class, 0, 0]);
    }
    for &cell in roles.cells() {
        put(&mut packet, cell);
    }
    let mut expected = Vec::new();
    let mut observations = Vec::new();
    for row in rows {
        if row.ids.len() > ROLE_CONTEXT {
            return Err("fixture exceeds the original role context".into());
        }
        for c in &row.candidates {
            let span = RoleSpan {
                start: c.span[0],
                end: c.span[1],
            };
            let (keys, scores) =
                roles.project_candidate_in_clause(&row.ids, span, c.clause[0]..c.clause[1])?;
            put(
                &mut packet,
                row.ids
                    .iter()
                    .copied()
                    .chain(std::iter::repeat(0))
                    .take(ROLE_CONTEXT),
            );
            put(
                &mut packet,
                [
                    row.ids.len(),
                    c.clause[0],
                    c.clause[1],
                    span.start,
                    span.end,
                ]
                .map(|v| v as u32),
            );
            put(&mut expected, keys);
            put(&mut expected, scores.map(|v| v as u32));
            observations.push(json!({"id":row.id,"name":c.name,"span":c.span,"clause":c.clause,"keys":keys,"scores":scores}));
        }
    }

    let output = Path::new(target);
    fs::create_dir(output)?;
    fs::write(output.join("packet.bin"), &packet)?;
    fs::write(output.join("native.bin"), &expected)?;
    let record = json!({"seed":seed,"artifact_sha256":digest(&model_bytes),"role_sha256":digest(&state_bytes),
        "tokenizer_sha256":digest(&tok_bytes),"fixtures_sha256":digest(&raw),"packet_sha256":digest(&packet),
        "native_sha256":digest(&expected),"rows":observations});
    fs::write(
        output.join("native.json"),
        serde_json::to_string_pretty(&record)?,
    )?;
    Ok(())
}
