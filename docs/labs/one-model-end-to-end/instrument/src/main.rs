//! The held-out-phrasing panel writer (one-model-end-to-end, claude-204).
//!
//! ```text
//! heldout-panel out=NEW_DIR tokenizer=TOKENIZER.json cell=PHRASING,VALUE [conversations=400] [seed=1] [protocol=2] [chain=N]
//! heldout-panel decode split=DIR tokenizer=TOKENIZER.json [docs=2] [skip=0]
//! heldout-panel check-queries split=DIR tokenizer=TOKENIZER.json
//! ```
//!
//! **Chained panels (claude-275).** With `chain=N` (at least `2 * CONTEXT`), every conversation
//! is `MWorld2::chained` in `cell`, up to N tokens, metered as `m-world-recall-corpus chain=`
//! meters TRAIN's chained draw: under the spaced meter in protocol 2, with a meter mismatch
//! counted, not refused. No turn is excluded. Beside the split it writes:
//! - `queries.json`: every chained query's conversation and turn, its document's first position,
//!   its reply's first position and length (the reply's mask run, its end marker included), its
//!   answer and its `Chain` record. Positions index `tokens.u16` past its 64-byte header.
//! - `user_turns.json`: one row of `user_turns` per conversation, the form
//!   `m-world-recall-corpus exclude=` reads, so TRAIN can be drawn without the panel's turns.
//!
//! `check-queries` re-reads a chained panel and checks every query against it: the hashes match,
//! a mask run of exactly the reply's length opens at the reply's position, the reply's text holds
//! the answer, and positions increase.
//!
//! It draws `conversations` M-world v2 conversations of one cell. A cell pairs `train` or `dev`
//! user-turn phrasings with `train` or `dev` values (`milestone_world_v2::Cell`). The draw is at
//! the default mix.
//!
//! **No probe screen.** `m-world corpus` drew TRAIN under a sealed-probe screen, which redraws a
//! conversation that shares an 8-word user-turn n-gram with the probe. Its module,
//! `milestone_world_v2_probe`, is preserved under `#[cfg(any())]` and never builds, so no
//! screen is applied here.
//! - This is no matter for a test panel: the screen kept the probe out of *training*.
//! - It is a small difference from how TRAIN was drawn, for the `train,train` control.
//!
//! The panel is written as a UORT split, in the literal-roles dialogue protocol `m-world corpus`
//! writes:
//! - `tokens.u16`;
//! - `response_mask.u8`;
//! - `manifest.json`.
//!
//! **Why a separate tool.** `crates/uor-r4-training/examples/m-world.rs` has not been a build
//! target since the cutover. The crate sets `autoexamples = false`, and the example's other modes
//! import modules that are now configured out. So this writes the panels its `corpus_v2` would,
//! from the same library calls.
//!
//! **The cells.**
//! - `cell=dev,dev` is the held-out-phrasing panel.
//! - `cell=train,dev` keeps trained phrasings and uses new values.
//! - `cell=train,train` is the positive control, drawn as TRAIN was.
//!
//! The assistant's reply templates are shared by every cell.
//!
//! It never replaces `out`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::{Rng, Split};
use uor_r4_training::milestone_world_v2::{
    Cell, Chain, ChainRedraws, MWorld2, Meter, Mix, CONTEXT,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// `PHRASING,VALUE`, each `train` or `dev`.
fn cell_of(text: &str) -> Result<Cell> {
    let part = |s: &str| match s.trim() {
        "train" => Ok(Split::Train),
        "dev" => Ok(Split::Development),
        other => Err(format!("cell= takes train or dev, not {other:?}")),
    };
    match text.split_once(',') {
        Some((phrasing, value)) => Ok(Cell::new(part(phrasing)?, part(value)?)),
        None => Err("cell= takes PHRASING,VALUE, each train or dev".into()),
    }
}

fn split_name(split: Split) -> &'static str {
    match split {
        Split::Train => "train",
        Split::Development => "dev",
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// One chained query's place in a panel, a row of `queries.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Query {
    conversation: usize,
    turn: usize,
    document_start: usize,
    reply_start: usize,
    reply_tokens: usize,
    answer: String,
    chain: Chain,
}

/// Each maximal run of mask 1, as (first position, length): a document's k-th run is turn k's
/// reply with its end marker.
fn reply_runs(mask: &[u8]) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (position, &m) in mask.iter().enumerate() {
        if m != 1 {
            continue;
        }
        match runs.last_mut() {
            Some((start, len)) if *start + *len == position => *len += 1,
            _ => runs.push((position, 1)),
        }
    }
    runs
}

/// What is wrong with `queries` against a panel's tokens (BOS = 0) and mask, one line a fault.
fn query_faults(
    tokens: &[u32],
    mask: &[u8],
    queries: &[Query],
    decode: impl Fn(&[u32]) -> String,
) -> Vec<String> {
    let mut faults = Vec::new();
    let mut last = None;
    for (index, query) in queries.iter().enumerate() {
        let (start, len) = (query.reply_start, query.reply_tokens);
        let end = start + len;
        let fault = if len == 0 || end > tokens.len() || end > mask.len() {
            Some("its reply is empty or runs past the panel".to_owned())
        } else if query.document_start >= start || tokens[query.document_start] != 0 {
            Some("its document does not open with BOS before the reply".to_owned())
        } else if mask[start - 1] != 0
            || mask[start..end].iter().any(|&m| m != 1)
            || mask.get(end).is_some_and(|&m| m == 1)
        {
            Some("no mask run of its reply's length opens at its reply".to_owned())
        } else if query.answer.is_empty() || !decode(&tokens[start..end]).contains(&query.answer) {
            Some(format!("its reply does not state {:?}", query.answer))
        } else if last.is_some_and(|l| l >= start) {
            Some("its reply does not follow the last query's".to_owned())
        } else {
            None
        };
        if let Some(fault) = fault {
            faults.push(format!(
                "query {index} (conversation {}, turn {}): {fault}",
                query.conversation, query.turn
            ));
        }
        last = Some(start);
    }
    faults
}

/// The token ids of a UORT `tokens.u16` file, past its 64-byte header.
fn token_ids(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() < 64 || bytes.len() % 2 != 0 {
        return Err("tokens.u16 is not a UORT token file".into());
    }
    Ok(bytes[64..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect())
}

/// `heldout-panel check-queries split=DIR tokenizer=FILE`: check a chained panel's
/// `queries.json` against its split, printing `checked N queries, F failed`.
fn check_queries(args: &BTreeMap<String, String>) -> Result<()> {
    let dir = PathBuf::from(args.get("split").ok_or("split= is required")?);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
        args.get("tokenizer").ok_or("tokenizer= is required")?,
    )?)
    .ok_or("tokenizer JSON is not a supported byte-level BPE")?;
    let token_bytes = fs::read(dir.join("tokens.u16"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    let record: Value = serde_json::from_slice(&fs::read(dir.join("queries.json"))?)?;
    let mut faults = Vec::new();
    for (key, actual) in [
        ("tokens_sha256", sha256(&token_bytes)),
        ("mask_sha256", sha256(&mask)),
    ] {
        if record[key].as_str() != Some(actual.as_str()) {
            faults.push(format!("queries.json's {key} is not the split's {actual}"));
        }
    }
    let queries: Vec<Query> = serde_json::from_value(record["queries"].clone())?;
    faults.extend(query_faults(
        &token_ids(&token_bytes)?,
        &mask,
        &queries,
        |ids| tokenizer.decode(ids),
    ));
    for fault in &faults {
        eprintln!("{fault}");
    }
    println!("checked {} queries, {} failed", queries.len(), faults.len());
    if queries.is_empty() || !faults.is_empty() {
        return Err("the queries do not check".into());
    }
    Ok(())
}

/// `heldout-panel decode split=DIR tokenizer=FILE [docs=2] [skip=0]`: print documents of a UORT
/// split as text, one line per mask run, marked `P` (mask 0) or `R` (mask 1). A diagnostic for
/// comparing a panel's encoding with TRAIN's.
fn decode_split(args: &BTreeMap<String, String>) -> Result<()> {
    let dir = PathBuf::from(args.get("split").ok_or("split= is required")?);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
        args.get("tokenizer").ok_or("tokenizer= is required")?,
    )?)
    .ok_or("tokenizer JSON is not a supported byte-level BPE")?;
    let docs: usize = args.get("docs").map_or(Ok(2), |v| v.parse())?;
    let skip: usize = args.get("skip").map_or(Ok(0), |v| v.parse())?;
    let bytes = fs::read(dir.join("tokens.u16"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    let tokens: Vec<u32> = bytes[64..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    let (mut doc, mut run, mut run_mask) = (0usize, Vec::<u32>::new(), 0u8);
    let flush = |run: &mut Vec<u32>, run_mask: u8, doc: usize| {
        if !run.is_empty() && doc > skip && doc <= skip + docs {
            let kind = if run_mask == 1 { 'R' } else { 'P' };
            println!(
                "{doc} {kind} {:?} {:?}",
                tokenizer.decode(run),
                &run[..run.len().min(12)]
            );
        }
        run.clear();
    };
    for (&t, &m) in tokens.iter().zip(&mask) {
        if t == 0 {
            flush(&mut run, run_mask, doc);
            doc += 1;
            if doc > skip + docs {
                break;
            }
            continue;
        }
        if !run.is_empty() && m != run_mask {
            flush(&mut run, run_mask, doc);
        }
        run_mask = m;
        run.push(t);
    }
    flush(&mut run, run_mask, doc);
    Ok(())
}

fn main() -> Result<()> {
    let mode = std::env::args().nth(1);
    if matches!(mode.as_deref(), Some("decode" | "check-queries")) {
        let args: BTreeMap<String, String> = std::env::args()
            .skip(2)
            .filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_owned(), v.to_owned())))
            .collect();
        return match mode.as_deref() {
            Some("decode") => decode_split(&args),
            _ => check_queries(&args),
        };
    }
    let args: BTreeMap<String, String> = std::env::args()
        .skip(1)
        .map(|a| {
            a.split_once('=')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .ok_or_else(|| format!("argument {a:?} is not KEY=VALUE"))
        })
        .collect::<std::result::Result<_, String>>()?;
    println!("{}", write_panel(&args)?);
    Ok(())
}

/// Write the panel `args` names and return its manifest.
fn write_panel(args: &BTreeMap<String, String>) -> Result<Value> {
    let get = |k: &str| args.get(k).cloned();
    let out = PathBuf::from(get("out").ok_or("out= is required")?);
    if out.exists() {
        return Err(format!("{} exists; the panel writer never replaces", out.display()).into());
    }
    let tokenizer_path = PathBuf::from(get("tokenizer").ok_or("tokenizer= is required")?);
    let cell = cell_of(&get("cell").ok_or("cell= is required")?)?;
    let conversations: usize = get("conversations").map_or(Ok(400), |v| v.parse())?;
    let seed: u64 = get("seed").map_or(Ok(1), |v| v.parse())?;
    // TRAIN (`mw-balp/train`) is in dialogue protocol 2. The `train,train` control matched TRAIN's
    // role-marker tokens only under it; under protocol 1 its exact-prompt share was 0 (claude-204).
    let version: u8 = get("protocol").map_or(Ok(2), |v| v.parse())?;
    let chain: Option<usize> = get("chain").map(|v| v.parse()).transpose()?;
    if chain.is_some_and(|budget| budget < 2 * CONTEXT) {
        return Err(format!("chain= takes at least {} tokens", 2 * CONTEXT).into());
    }
    let max_tokens = chain.unwrap_or(CONTEXT);

    let tokenizer_bytes = fs::read(&tokenizer_path)?;
    let tokenizer_sha256 = sha256(&tokenizer_bytes);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer JSON is not a supported byte-level BPE")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)
        .map_err(|e| format!("protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("protocol: {e}"))?;
    let vocab = u32::try_from(tokenizer.vocab_size())?;
    let count = |text: &str| tokenizer.encode(text).len();
    let spaced_meter = Meter::spaced(&count);
    let mut world = MWorld2::new(&count, Mix::default())?;
    // As `m-world-recall-corpus` draws a chained TRAIN: metered under protocol 2's layout.
    if chain.is_some() && version == 2 {
        world = world.with_spaced_meter();
    }
    let mut rng = Rng::new(seed);

    let (mut tokens, mut mask) = (Vec::<u16>::new(), Vec::<u8>::new());
    let (mut queries, mut user_turns) = (Vec::<Query>::new(), Vec::<Value>::new());
    let (mut redraws, mut metered_mismatches) = (ChainRedraws::default(), 0usize);
    for index in 0..conversations {
        let conversation = match chain {
            None => world.conversation_in(&mut rng, cell)?,
            Some(budget) => {
                world.chained(&mut rng, cell, budget, &BTreeSet::new(), &mut redraws)?
            }
        };
        let messages: Vec<Message<'_>> = conversation
            .turns
            .iter()
            .flat_map(|turn| {
                [
                    Message {
                        role: "user",
                        content: &turn.user,
                    },
                    Message {
                        role: "assistant",
                        content: &turn.reply,
                    },
                ]
            })
            .collect();
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len() || encoded.special_token_occurrences != 0 {
            return Err(format!("conversation {index} did not encode").into());
        }
        // `corpus_v2`'s checks, as TRAIN was drawn: the conversation is metered under version 1;
        // a version-2 document must match the spaced meter, and every document fits the context.
        // A chained draw is counted as `m-world-recall-corpus` counts it: against its own meter,
        // a mismatch recorded, not refused.
        let metered = if chain.is_some() || version == 1 {
            conversation.tokens
        } else {
            spaced_meter.document(&conversation.turns)
        };
        if chain.is_some() {
            metered_mismatches += usize::from(encoded.tokens.len() != metered);
        } else if encoded.tokens.len() != metered {
            return Err(format!(
                "conversation {index}: the meter counted {metered} tokens, the protocol {}",
                encoded.tokens.len()
            )
            .into());
        }
        if encoded.tokens.len() > max_tokens {
            return Err(format!(
                "conversation {index} has {} tokens, over the {max_tokens}-token context",
                encoded.tokens.len()
            )
            .into());
        }
        if chain.is_some() {
            let runs = reply_runs(&encoded.response_mask);
            if runs.len() != conversation.turns.len() {
                return Err(format!(
                    "conversation {index}: {} reply runs for {} turns",
                    runs.len(),
                    conversation.turns.len()
                )
                .into());
            }
            let document_start = tokens.len();
            for (turn, (item, &(start, len))) in conversation.turns.iter().zip(&runs).enumerate() {
                let Some(record) = &item.tag.chain else {
                    continue;
                };
                queries.push(Query {
                    conversation: index,
                    turn,
                    document_start,
                    reply_start: document_start + start,
                    reply_tokens: len,
                    answer: item
                        .tag
                        .answer
                        .clone()
                        .ok_or_else(|| format!("conversation {index}: a query has no answer"))?,
                    chain: record.clone(),
                });
            }
            let turns: Vec<&str> = conversation.turns.iter().map(|t| t.user.as_str()).collect();
            user_turns.push(json!({"conversation": index, "user_turns": turns}));
        }
        for &id in &encoded.tokens {
            if id >= vocab {
                return Err("a token is outside the vocabulary".into());
            }
            tokens.push(u16::try_from(id)?);
        }
        mask.extend(&encoded.response_mask);
    }
    if mask.len() != tokens.len() {
        return Err("the response mask does not cover the tokens".into());
    }

    fs::create_dir_all(&out)?;
    let mut writer = CorpusWriter::create(out.join("tokens.u16"), vocab)
        .map_err(|e| format!("token store: {e}"))?;
    writer
        .write_tokens(&tokens)
        .map_err(|e| format!("token store: {e}"))?;
    let total = writer.finish().map_err(|e| format!("token store: {e}"))?;
    fs::write(out.join("response_mask.u8"), &mask)?;
    let mut manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": split_name(cell.phrasing),
        "writer": "one-model-end-to-end heldout-panel (claude-204)",
        "world": "m-world-v2",
        "cell": {"phrasing": split_name(cell.phrasing), "value": split_name(cell.value)},
        "mix": "default",
        "seed": seed,
        "conversations": conversations,
        "dialogue_protocol_version": version,
        "probe_screen": "none (milestone_world_v2_probe is preserved under cfg(any()))",
        "tokens": total,
        "tokens_bytes": tokens.len() * 2 + 64,
        "tokens_sha256": sha256(&fs::read(out.join("tokens.u16"))?),
        "response_tokens": mask.iter().filter(|&&m| m == 1).count(),
        "mask_bytes": mask.len(),
        "mask_sha256": sha256(&mask),
        "max_tokens": max_tokens,
        "tokenizer": {
            "vocab_size": vocab,
            "bos_id": 0,
            "sha256": tokenizer_sha256,
        },
    });
    if let Some(budget) = chain {
        if queries.is_empty() {
            return Err("the chained panel has no query".into());
        }
        let record = json!({
            "schema": "uor-r4-chain-queries/v1",
            "writer": "heldout-panel (claude-275)",
            "tokens_sha256": manifest["tokens_sha256"],
            "mask_sha256": manifest["mask_sha256"],
            "conversations": conversations,
            "queries": queries,
        });
        let queries_bytes = serde_json::to_vec_pretty(&record)?;
        let user_turns_bytes = serde_json::to_vec_pretty(&user_turns)?;
        fs::write(out.join("queries.json"), &queries_bytes)?;
        fs::write(out.join("user_turns.json"), &user_turns_bytes)?;
        manifest["writer"] = json!("one-model-end-to-end heldout-panel (claude-275)");
        manifest["chain"] = json!({
            "budget": budget,
            "excluded": "none",
            "queries": queries.len(),
            "queries_sha256": sha256(&queries_bytes),
            "user_turns_sha256": sha256(&user_turns_bytes),
            "metered_mismatches": metered_mismatches,
            "redraws": redraws,
        });
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_names_its_phrasing_and_value_split() {
        let cell = cell_of("dev,train").unwrap();
        assert_eq!(
            (cell.phrasing, cell.value),
            (Split::Development, Split::Train)
        );
        assert_eq!(
            cell_of(" train , dev ").unwrap(),
            Cell::new(Split::Train, Split::Development)
        );
        assert!(cell_of("dev").is_err());
        assert!(cell_of("dev,test").is_err());
    }

    fn query(reply_start: usize, reply_tokens: usize, answer: &str) -> Query {
        Query {
            conversation: 0,
            turn: 0,
            document_start: 0,
            reply_start,
            reply_tokens,
            answer: answer.to_owned(),
            chain: Chain {
                notes: 1,
                queried: 0,
                episodes: 2,
                episode: 0,
                distance: 1,
                tokens: 7,
            },
        }
    }

    #[test]
    fn a_reply_run_is_a_maximal_run_of_mask_ones() {
        assert_eq!(
            reply_runs(&[0, 1, 1, 0, 0, 1, 0, 1]),
            vec![(1, 2), (5, 1), (7, 1)]
        );
        assert!(reply_runs(&[0, 0]).is_empty());
    }

    #[test]
    fn a_query_must_sit_on_a_reply_of_its_length_that_states_its_answer() {
        // BOS, a user turn, a two-token reply, a user turn, a one-token reply.
        let tokens = [0, 5, 6, 7, 8, 9, 10];
        let mask = [0, 0, 0, 1, 1, 0, 1];
        let decode = |ids: &[u32]| ids.iter().map(|i| format!("<{i}>")).collect::<String>();
        let good = vec![query(3, 2, "<8>"), query(6, 1, "<10>")];
        assert!(query_faults(&tokens, &mask, &good, decode).is_empty());
        for bad in [
            vec![query(4, 1, "<8>")],
            vec![query(3, 1, "<7>")],
            vec![query(3, 2, "<9>")],
            vec![query(3, 2, "")],
            vec![query(6, 2, "<10>")],
            vec![good[1].clone(), good[0].clone()],
        ] {
            assert_eq!(query_faults(&tokens, &mask, &bad, decode).len(), 1, "{bad:?}");
        }
    }

    /// A byte-per-token tokenizer leaves no M-world episode inside the 256-token cap, so this,
    /// like `m-world-recall-corpus`'s pinned test, reads the tokenizer TRAIN was written with.
    #[test]
    #[ignore = "needs the pinned d36d3e87 tokenizer JSON in UOR_R4_PINNED_TOKENIZER"]
    fn a_chained_panel_places_every_query_on_its_reply() {
        let tokenizer_path = std::env::var("UOR_R4_PINNED_TOKENIZER")
            .expect("set UOR_R4_PINNED_TOKENIZER to the d36d3e87 tokenizer JSON");
        assert_eq!(
            sha256(&fs::read(&tokenizer_path).unwrap()),
            "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89"
        );
        let tokenizer = tokenizer_path.as_str();
        let root =
            std::env::temp_dir().join(format!("heldout-panel-chain-{}", std::process::id()));
        let out = root.join("panel");
        let args = |pairs: &[(&str, &str)]| -> BTreeMap<String, String> {
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect()
        };
        let panel = args(&[
            ("out", out.to_str().unwrap()),
            ("tokenizer", tokenizer),
            ("cell", "dev,dev"),
            ("conversations", "3"),
            ("seed", "7"),
            ("chain", "512"),
        ]);
        let manifest = write_panel(&panel).unwrap();
        let queries = manifest["chain"]["queries"].as_u64().unwrap();
        assert!(queries >= 3, "{manifest}");
        assert_eq!(manifest["max_tokens"], 512);
        let rows: Vec<Value> =
            serde_json::from_slice(&fs::read(out.join("user_turns.json")).unwrap()).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|r| r["user_turns"].as_array().is_some_and(|t| !t.is_empty())));
        check_queries(&args(&[
            ("split", out.to_str().unwrap()),
            ("tokenizer", tokenizer),
        ]))
        .unwrap();
        assert!(write_panel(&panel).is_err(), "the writer never replaces");
        let mut short = panel.clone();
        short.insert("out".into(), root.join("short").to_str().unwrap().into());
        short.insert("chain".into(), "511".into());
        assert!(write_panel(&short).is_err(), "a chain shorter than two contexts");
        fs::remove_dir_all(&root).unwrap();
    }
}
