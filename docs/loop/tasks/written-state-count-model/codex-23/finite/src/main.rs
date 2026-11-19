//! Finite planted-state fixture. No corpus reads, counting, fitting or GPU access.
mod validate;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use uor_r4_discrete::ngram_count::PrimeRouteKey;
use uor_r4_integer::discrete_d11::{DISCRETE_S2_SCHEMA, DISCRETE_TRUNK_SCHEMA, S2_SECTION};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Fail = Box<dyn std::error::Error>;
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const TEXTS: [&str; 2] = ["red saw the oak", "blue saw the oak"];
const COPIES: [u16; 2] = [677, 924];
const BACKOFF: u16 = 1710;
const VOCABULARY: u16 = 4096;
const TOTAL: u64 = 1 << 32;
type Row = (u128, u16, u8);

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn checked(condition: bool, why: &str) -> Result<(), Fail> {
    if !condition {
        return Err(why.to_owned().into());
    }
    Ok(())
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), Fail> {
    let mut file = File::options().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), Fail> {
    write(path, &serde_json::to_vec_pretty(value)?)
}

fn key(context: &[u16]) -> Result<u128, Fail> {
    Ok(PrimeRouteKey::from_context(context)?.word())
}

fn key_hex(key: u128) -> String {
    format!("{key:032x}")
}

fn plant(rows: &mut Vec<Row>, context: &[u16], tokens_and_ranks: &[(u16, u8)]) -> Result<(), Fail> {
    let word = key(context)?;
    rows.extend(
        tokens_and_ranks
            .iter()
            .map(|&(token, rank)| (word, token, rank)),
    );
    Ok(())
}

fn data_parts(artifact: &[u8]) -> Result<(Value, &[u8]), Fail> {
    let size = usize::try_from(u64::from_le_bytes(
        artifact.get(8..16).ok_or("short file header")?.try_into()?,
    ))?;
    let end = 16_usize.checked_add(size).ok_or("header length overflow")?;
    let header = serde_json::from_slice(artifact.get(16..end).ok_or("short JSON header")?)?;
    let data = artifact
        .get(
            end.checked_next_multiple_of(64)
                .ok_or("data start overflow")?..,
        )
        .ok_or("short data start")?;
    Ok((header, data))
}

// Same byte layout as docs/labs/kn-reference/instrument/src/bin/count-replay.rs:
// schema descriptor, unchanged carrier payload, aligned 96-byte digest header, then rows.
fn pack(base: &[u8], source: &[u8], rows: &[u8]) -> Result<Vec<u8>, Fail> {
    let (mut header, payload) = data_parts(base)?;
    checked(
        header["schema"] == DISCRETE_TRUNK_SCHEMA,
        "expected 19-table trunk",
    )?;
    let offset = payload.len().next_multiple_of(64);
    let bytes = source.len() + rows.len();
    header["schema"] = json!(DISCRETE_S2_SCHEMA);
    header["tokenizer_sha256"] = json!(TOKENIZER);
    header["tables"]
        .as_array_mut()
        .ok_or("missing tables")?
        .push(json!({
            "name": S2_SECTION,
            "kind": "u32",
            "len": bytes / 4,
            "span": {"offset": offset, "bytes": bytes}
        }));
    let header = serde_json::to_vec(&header)?;
    let mut out = Vec::with_capacity(16 + header.len() + 64 + offset + bytes);
    out.extend_from_slice(&base[..8]);
    out.extend_from_slice(&(header.len() as u64).to_le_bytes());
    out.extend_from_slice(&header);
    let data_start = out.len().next_multiple_of(64);
    out.resize(data_start, 0);
    out.extend_from_slice(payload);
    out.resize(data_start + offset, 0);
    out.extend_from_slice(source);
    out.extend_from_slice(rows);
    Ok(out)
}

// Explicit expected contexts select keys through the upstream PrimeRouteKey.
// The dense reference is diagnostic-only and enumerates legal IDs in token order.
fn oracle(rows: &[Row], context: Option<&[u16]>) -> Result<Value, Fail> {
    let word = context.map(key).transpose()?;
    let reads: Vec<(u16, u64)> = rows
        .iter()
        .filter(|&&(key, _, _)| Some(key) == word)
        .map(|&(_, token, rank)| (token, 1 << rank))
        .collect();
    checked(
        word.is_none() == reads.is_empty(),
        "wrong oracle fixture route",
    )?;
    let uniform = if reads.is_empty() { TOTAL } else { TOTAL / 32 };
    let legal = u64::from(VOCABULARY - 1);
    let floor = uniform / legal;
    let remainder = uniform % legal;
    let mut mass = vec![0_u64; usize::from(VOCABULARY)];
    for token in 1..VOCABULARY {
        mass[usize::from(token)] = floor + u64::from(u64::from(token) <= remainder);
    }
    let retained_sum: u64 = reads.iter().map(|read| read.1).sum();
    let mut previous = 0;
    let mut count = 0;
    for &(token, increment) in &reads {
        count += increment;
        let allocated =
            ((u128::from(TOTAL - uniform) * u128::from(count)) / u128::from(retained_sum)) as u64;
        mass[usize::from(token)] += allocated - previous;
        previous = allocated;
    }
    checked(
        mass.iter().sum::<u64>() == TOTAL && mass[1..].iter().all(|&value| value >= floor),
        "broken oracle floor or total",
    )?;
    let mut ranked: Vec<_> = (1..VOCABULARY)
        .map(|token| (token, mass[usize::from(token)]))
        .collect();
    ranked.sort_by_key(|&(token, mass)| (std::cmp::Reverse(mass), token));
    Ok(json!({
        "order": context.map_or(0, <[u16]>::len),
        "key": word.map(key_hex),
        "reads": reads,
        "mass": mass,
        "top1": ranked[0].0,
        "top8": &ranked[..8]
    }))
}

fn fixture(base_path: &Path, tokenizer_path: &Path, out: &Path) -> Result<(), Fail> {
    let base = fs::read(base_path)?;
    checked(sha(&base) == BASE, "wrong public M1 seed1 artifact")?;
    let bytes = fs::read(tokenizer_path)?;
    checked(sha(&bytes) == TOKENIZER, "wrong original tokenizer")?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
    checked(protocol.eos_id == 1, "fixture expects original EOS ID 1")?;
    let bound = protocol.bind(&tokenizer)?;
    let prompts: Vec<Vec<u16>> = TEXTS
        .iter()
        .map(|&text| {
            bound
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: text,
                }])
                .tokens
                .into_iter()
                .map(u16::try_from)
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<_, _>>()?;
    let suffixes: Vec<_> = prompts
        .iter()
        .map(|prompt| &prompt[prompt.len() - 10..])
        .collect();
    checked(
        suffixes[0][1..] == suffixes[1][1..]
            && suffixes[0][0] == COPIES[0]
            && suffixes[1][0] == COPIES[1],
        "order-ten prompts do not share exactly nine trailing tokens and preserve the copy source",
    )?;
    let shared = &suffixes[0][1..];
    let marker = shared[8];
    let appends = [[marker, COPIES[0]], [marker, COPIES[1]]];
    let mut rolled = prompts.clone();
    let mut rows = Vec::new();
    for index in 0..2 {
        let copy = COPIES[index];
        plant(
            &mut rows,
            suffixes[index],
            &[(copy, 7), (COPIES[1 - index], 3)],
        )?;
        rolled[index].push(copy);
        plant(&mut rows, &appends[index], &[(1, 5)])?;
    }
    plant(&mut rows, shared, &[(BACKOFF, 4)])?;
    plant(
        &mut rows,
        &[3000],
        &[(12, 31), (13, 31), (14, 31), (15, 31)],
    )?;
    plant(&mut rows, &[0, 3000], &[(41, 4)])?;
    let short = [23, 4000];
    let tie = [22, 3000];
    let bos = [31, 0, 3000];
    let eos = [31, 1, 3000];
    // Higher-order decoys would win if either document boundary were crossed.
    plant(&mut rows, &bos, &[(50, 6)])?;
    plant(&mut rows, &eos, &[(51, 6)])?;
    let unknown = [500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400];
    let ninth_only = [&[999_u16][..], shared].concat();
    let cases: [(&str, &[u16], Option<&[u16]>); 10] = [
        ("red-copy", &prompts[0], Some(suffixes[0])),
        ("red-eos", &rolled[0], Some(&appends[0])),
        ("blue-copy", &prompts[1], Some(suffixes[1])),
        ("blue-eos", &rolled[1], Some(&appends[1])),
        ("order-nine-miss", &ninth_only, Some(shared)),
        ("short-uniform", &short, None),
        ("full-uniform", &unknown, None),
        ("four-way-tie", &tie, Some(&tie[1..])),
        ("bos-boundary", &bos, Some(&bos[1..])),
        ("eos-boundary", &eos, Some(&eos[2..])),
    ];
    rows.sort_unstable();
    checked(
        rows.windows(2)
            .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1)),
        "duplicate planted key/token",
    )?;
    let mut row_bytes = Vec::new();
    for (key, token, rank) in rows.iter().copied() {
        row_bytes.extend_from_slice(&key.to_le_bytes());
        row_bytes.extend_from_slice(&token.to_le_bytes());
        row_bytes.extend_from_slice(&[rank, 0]);
    }
    let mut source = Vec::from(Sha256::digest(&base).as_slice());
    source.extend_from_slice(&[0; 32]);
    source.extend_from_slice(&Sha256::digest(&row_bytes));
    let passive = pack(&base, &source, &row_bytes)?;
    let mut queries = String::new();
    let mut oracle_rows = Vec::new();
    let mut summary = Vec::new();
    for (name, prefix, expected_context) in cases {
        let ids = prefix
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        queries.push_str(&ids);
        queries.push('\n');
        let row = oracle(&rows, expected_context)?;
        summary.push(
            json!({"case": name, "prefix": prefix, "order": row["order"],
            "key_hex": row["key"], "reads": row["reads"], "top8": row["top8"]}),
        );
        oracle_rows.push(row);
    }
    fs::create_dir(out)?;
    write(&out.join("fixture-passive.d11"), &passive)?;
    write(&out.join("fixture.rows"), &row_bytes)?;
    write(&out.join("queries.txt"), queries.as_bytes())?;
    write_json(&out.join("oracle.json"), &json!(oracle_rows))?;
    write_json(
        &out.join("plan.json"),
        &json!({
            "base_sha256": BASE,
            "tokenizer_sha256": TOKENIZER,
            "passive_sha256": sha(&passive),
            "rows_sha256": sha(&row_bytes),
            "passive_bytes": passive.len(),
            "planted_rows": rows.len(),
            "texts": TEXTS,
            "queries": summary
        }),
    )?;
    Ok(())
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [verb, base, tokenizer, out] if verb == "make" => {
            fixture(Path::new(base), Path::new(tokenizer), Path::new(out))
        }
        a if a.first().is_some_and(|verb| verb == "validate") => validate::run(&a[1..]),
        _ => Err("active-s2-finite make BASE TOKENIZER NEW_RUN_DIRECTORY".into()),
    }
}
