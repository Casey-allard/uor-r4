//! Independent custody check against the typed runtime extractor. No role scorer.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::ops::Range;
use std::path::Path;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
use uor_r4_integer::ordered_role::{orbit_action_digest, RoleFraming, RoleMode};
use uor_r4_integer::paired_role::{PairedRoleState, RoleSpan};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn json(path: &Path) -> Result<Vec<Value>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn index(value: &Value) -> Result<usize> {
    Ok(serde_json::from_value(value.clone())?)
}

fn tape(value: &Value) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(value.clone())?)
}

fn framing(tokenizer: &ByteBpeTokenizer) -> Result<RoleFraming> {
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let probe = [Message {
        role: "user",
        content: "x",
    }];
    let history = encoder.encode_open_history(&probe).tokens;
    let full = encoder.encode_assistant_prefix(&probe).tokens;
    let prefix = history
        .strip_suffix(tokenizer.encode(" x").as_slice())
        .ok_or("unfaithful probe content")?;
    let suffix = full
        .strip_prefix(history.as_slice())
        .ok_or("unfaithful probe suffix")?;
    Ok(RoleFraming::new(
        prefix.to_vec(),
        suffix.to_vec(),
        encoder.encode_assistant_prefix(&[]).tokens,
    )?)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [model, tokenizer, bank, panel] = args.as_slice() else {
        return Err(
            "usage: check MODEL_WITH_FROZEN_ROLES TOKENIZER FROZEN_ROLE_BANK PANEL_ROOT".into(),
        );
    };
    let model_bytes = fs::read(model)?;
    let model_sha = format!("{:x}", Sha256::digest(&model_bytes));
    let model = IntegerDiscreteModel::from_bytes(&model_bytes)?;
    let tokenizer_bytes = fs::read(tokenizer)?;
    let tokenizer_sha = format!("{:x}", Sha256::digest(&tokenizer_bytes));
    if model.tokenizer_sha256() != Some(tokenizer_sha.as_str()) {
        return Err("paired artifact does not bind this tokenizer".into());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("invalid pinned tokenizer")?;
    let framing = framing(&tokenizer)?;
    let sites = model.ordered_role_sites()?;
    let classes: Vec<_> = sites.word_classes().collect();
    let bank = PairedRoleState::from_bytes(&fs::read(bank)?)?;
    if classes
        .iter()
        .copied()
        .map(u32::from)
        .ne(bank.sites().iter().map(|site| site[1]))
    {
        return Err("embedded role word classes differ from separately frozen bank".into());
    }
    let panel = Path::new(panel);
    let mut rows = 0;
    let mut candidates = 0;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    for split in ["train", "held"] {
        let blind = json(&panel.join(format!("{split}-blind.json")))?;
        let bounds = json(&panel.join(format!("{split}-bounds.json")))?;
        let census = json(&panel.join(format!("{split}-census.json")))?;
        if blind.len() != bounds.len() || blind.len() != census.len() {
            return Err("row alignment lost".into());
        }
        for (i, ((blind, bound), candidates_json)) in
            blind.iter().zip(bounds).zip(census).enumerate()
        {
            let text = blind["text"].as_str().ok_or("expected ordinary text")?;
            let ids = tape(&blind["prefix"])?;
            if encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: text,
                }])
                .tokens
                != ids
            {
                return Err(format!("{split}:{i} public framing difference").into());
            }
            let content = framing.content(&ids)?;
            let prompt = sites.prompt(&ids, content.clone())?;
            let start = index(&bound["start"])?;
            let end = index(&bound["end"])?;
            let bounded = start..end;
            let available: BTreeSet<_> = prompt
                .candidates()
                .filter(|span| span.start >= start && span.end <= end)
                .map(|span| [span.start, span.end])
                .collect();
            let recorded: BTreeSet<[usize; 2]> = candidates_json
                .as_array()
                .ok_or("invalid census array")?
                .iter()
                .filter(|candidate| candidate["admitted"] == true)
                .map(|candidate| serde_json::from_value(candidate["span"].clone()))
                .collect::<serde_json::Result<_>>()?;
            if available != recorded {
                return Err(format!(
                    "{split}:{i} candidate admission differs: {available:?} versus {recorded:?}"
                )
                .into());
            }
            for candidate in candidates_json
                .as_array()
                .into_iter()
                .flatten()
                .filter(|candidate| candidate["admitted"] == true)
            {
                let [start, end] = serde_json::from_value(candidate["span"].clone())?;
                let span = RoleSpan { start, end };
                check_candidate(&prompt, &ids, bounded.clone(), span, candidate)
                    .map_err(|error| format!("{split}:{i} {span:?}: {error}"))?;
                candidates += 1;
            }
            rows += 1;
        }
    }
    let orbit = orbit_action_digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    println!("{model_sha} {tokenizer_sha} {orbit} {rows} exact public tapes {candidates} bounded candidates: both vectors and original slot/state/ID tuples exact");
    Ok(())
}

fn check_candidate(
    prompt: &uor_r4_integer::ordered_role::RolePrompt<'_>,
    ids: &[u32],
    bounded: Range<usize>,
    span: RoleSpan,
    candidate: &Value,
) -> Result<()> {
    for (mode, key) in [
        (RoleMode::Identity, "identity"),
        (RoleMode::Positional, "positional"),
    ] {
        let expected: [i16; 32] = serde_json::from_value(candidate[key].clone())?;
        if prompt.feature(mode, span, bounded.clone())? != expected {
            return Err(format!("{key} feature words differ").into());
        }
    }
    let mut positions = prompt
        .feature_positions(span, bounded)?
        .into_iter()
        .enumerate()
        .filter_map(|(i, position)| {
            position.map(|position| {
                let state = if i < 12 { i + 1 } else { 33 + i - 12 };
                (position, state, ids[position])
            })
        })
        .collect::<Vec<_>>();
    let mut expected: Vec<(usize, usize, u32)> =
        serde_json::from_value(candidate["slots"].clone())?;
    positions.sort_unstable();
    expected.sort_unstable();
    if positions != expected {
        return Err("original positions, frame states or token IDs differ".into());
    }
    Ok(())
}
