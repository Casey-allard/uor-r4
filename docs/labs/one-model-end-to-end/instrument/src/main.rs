//! The held-out-phrasing panel writer (one-model-end-to-end, claude-204).
//!
//! ```text
//! heldout-panel out=NEW_DIR tokenizer=TOKENIZER.json cell=PHRASING,VALUE [conversations=400] [seed=1] [protocol=2]
//! heldout-panel decode split=DIR tokenizer=TOKENIZER.json [docs=2] [skip=0]
//! ```
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

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::{Rng, Split};
use uor_r4_training::milestone_world_v2::{Cell, MWorld2, Meter, Mix, CONTEXT};

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
    if std::env::args().nth(1).as_deref() == Some("decode") {
        let args: BTreeMap<String, String> = std::env::args()
            .skip(2)
            .filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_owned(), v.to_owned())))
            .collect();
        return decode_split(&args);
    }
    let args: BTreeMap<String, String> = std::env::args()
        .skip(1)
        .map(|a| {
            a.split_once('=')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .ok_or_else(|| format!("argument {a:?} is not KEY=VALUE"))
        })
        .collect::<std::result::Result<_, String>>()?;
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
    let mut rng = Rng::new(seed);

    let (mut tokens, mut mask) = (Vec::<u16>::new(), Vec::<u8>::new());
    for index in 0..conversations {
        let conversation = world.conversation_in(&mut rng, cell)?;
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
        let metered = if version == 1 {
            conversation.tokens
        } else {
            spaced_meter.document(&conversation.turns)
        };
        if encoded.tokens.len() != metered {
            return Err(format!(
                "conversation {index}: the meter counted {metered} tokens, the protocol {}",
                encoded.tokens.len()
            )
            .into());
        }
        if encoded.tokens.len() > CONTEXT {
            return Err(format!(
                "conversation {index} has {} tokens, over the {CONTEXT}-token context",
                encoded.tokens.len()
            )
            .into());
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
    let manifest = json!({
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
        "max_tokens": CONTEXT,
        "tokenizer": {
            "vocab_size": vocab,
            "bos_id": 0,
            "sha256": tokenizer_sha256,
        },
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("{manifest}");
    Ok(())
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
}
