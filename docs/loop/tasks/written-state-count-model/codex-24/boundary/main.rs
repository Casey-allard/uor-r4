use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{
    discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS},
    paired_role::{PairedRoleDecision, RoleSpan},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MODEL: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

struct Pair {
    name: &'static str,
    sentence: &'static str,
    question: &'static str,
    contexts: [&'static str; 2],
    literals: [&'static str; 2],
}

const PAIRS: [Pair; 2] = [
    Pair {
        name: "Morgan",
        sentence: "Morgan can help train drivers.",
        question: "Quote only the full noun phrase in that sentence naming the people who can receive Morgan’s help.",
        contexts: [
            "At the railway, the locomotive crews have a payroll problem. Morgan is an accountant and is forbidden to give driving instruction.",
            "At the trucking school, Morgan is a driving instructor. Everyone she is working with drives on roads, and nobody works on a railway.",
        ],
        literals: ["train drivers", "drivers"],
    },
    Pair {
        name: "Jo",
        sentence: "Jo can help guide dogs.",
        question: "Quote only the full noun phrase in that sentence naming the animals Jo can help.",
        contexts: [
            "At the assistance-animal clinic, Jo can pay for treatment but cannot handle or lead an animal. The clinic serves only animals trained to lead blind people.",
            "At the ordinary pet shelter, none of the animals is trained to lead a person. During the drill Jo leads the animals to the outside gate.",
        ],
        literals: ["guide dogs", "dogs"],
    },
];

fn token_at(boundaries: &[usize], byte: usize) -> Result<usize> {
    boundaries
        .binary_search(&byte)
        .map_err(|_| format!("byte {byte} splits an original token").into())
}

fn cursor(ids: &[u32], onset: usize, fence: usize) -> Result<Value> {
    let mut state = ExactCopyState::new(Some(u32::try_from(onset)?), u32::try_from(ids.len())?);
    let expected = [ids[onset..fence].to_vec(), vec![1]].concat();
    let mut emitted = Vec::new();
    let mut words = Vec::new();
    for _ in 0..8 {
        // A non-EOS fallback distinguishes completion from ordinary generation.
        let step = state
            .write(ids, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        emitted.push(step.emitted);
        words.push(step.words());
        state = step.state;
        if step.emitted == 1 {
            if step.completed_span() && emitted == expected {
                return Ok(
                    json!({"original_start": onset, "expected_ids": expected, "ids": emitted, "steps": words}),
                );
            }
            break;
        }
    }
    Err("forced native cursor did not write the original literal through EOS".into())
}

fn inspect(
    model: &IntegerDiscreteModel,
    tokenizer: &ByteBpeTokenizer,
    pair: &Pair,
    reading: usize,
) -> Result<(Value, Value)> {
    let context = pair.contexts[reading];
    let text = format!("{context} {}", pair.sentence) + " " + pair.question;
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer)?;
    let encoder = protocol.bind(tokenizer)?;
    let ids = encoder
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: &text,
        }])
        .tokens;
    let decoded = tokenizer.decode(&ids);
    let framing = "<|bos|>User: ";
    if decoded != format!("{framing}{text}\nAssistant:") {
        return Err("prefix did not decode byte-for-byte".into());
    }
    let mut boundaries = vec![0];
    for &id in &ids {
        boundaries.push(
            boundaries.last().ok_or("missing zero byte boundary")?
                + tokenizer.decode_bytes(&[id]).len(),
        );
    }

    let target_byte = framing.len() + context.len();
    let query_byte = framing.len()
        + text
            .rfind(&format!(" {}", pair.name))
            .ok_or("query name absent")?;
    let start = token_at(&boundaries, query_byte)?;
    let end = token_at(&boundaries, query_byte + 1 + pair.name.len())?;
    let query = RoleSpan { start, end };
    let fact_start = token_at(&boundaries, target_byte)?;
    let fact_end = token_at(&boundaries, target_byte + 1 + pair.name.len())?;
    let fence = token_at(&boundaries, target_byte + pair.sentence.len())?;
    let onsets: Vec<_> = pair
        .literals
        .iter()
        .map(|literal| {
            let offset = pair.sentence.find(literal).ok_or("literal absent")?;
            token_at(&boundaries, target_byte + offset)
        })
        .collect::<Result<_>>()?;
    for (&onset, literal) in onsets.iter().zip(pair.literals) {
        if tokenizer.decode(&ids[onset..fence]) != format!(" {literal}") {
            return Err("gold or rival does not decode to its adjudicated noun phrase".into());
        }
    }
    let roles = PairedRoleDecision {
        requested: None,
        excluded: None,
        requested_score: 0,
        excluded_score: 0,
        requested_runner_up: 0,
        excluded_runner_up: 0,
    };
    let (decision, witnesses) = model.paired_copy_witness(&ids, roles, Some(query))?;
    let witness = witnesses
        .iter()
        .find(|w| w.fact_span == [fact_start, fact_end])
        .ok_or("intended factual sentence did not bind")?;
    if witness.assertion.fence != Some(fence as u32) {
        return Err("intended fact scans to a different fence".into());
    }
    let roots = model.ordered_role_tables()?.0;
    let classes = model.frozen_role_replay()?.assertion_classes;
    let word_start_mask = witness.assertion.value.word_start_mask;
    let eligible_mask = witness.assertion.value.eligible_mask;
    let mask = eligible_mask & word_start_mask;
    let last = fact_end - 1;
    let mut offsets = Vec::new();
    let mut relative = Vec::new();
    for i in 0..3 {
        let source = last + i + 2;
        let id = ids[source];
        let eligible = eligible_mask & (1 << i) != 0;
        let word_start = word_start_mask & (1 << i) != 0;
        let admitted = mask & (1 << i) != 0;
        if eligible && (classes[id as usize] == 1) != word_start {
            return Err("word-start audit disagrees with the original span bank".into());
        }
        let word_end = if admitted {
            Some(
                (source + 1..fence)
                    .find(|&i| classes[ids[i] as usize] != 2)
                    .unwrap_or(fence),
            )
        } else {
            None
        };
        let reachable = admitted && fence - source < 8;
        let descriptor = word_end.filter(|_| reachable).map(|end| {
            ids[fact_end..end]
                .iter()
                .map(|&token| [classes[token as usize], roots[token as usize]])
                .collect::<Vec<_>>()
        });
        offsets.push(json!({
            "after_last_fact_token": i + 2, "absolute_source": source, "id": id,
            "piece": tokenizer.decode(&[id]), "word_class": classes[id as usize],
            "eligible": eligible, "word_start": word_start, "joint_mask": admitted,
            "absolute_word_end": word_end, "before_cursor_cap": reachable,
            "whole_word": word_end.map(|end| tokenizer.decode(&ids[source..end])),
            "projected_classes_and_roots": descriptor
        }));
        relative.push(json!({
            "after_last_fact_token": i + 2, "word_end_relative_to_last": word_end.map(|end| end - last),
            "eligible": eligible, "word_start": word_start, "before_cursor_cap": reachable,
            "projected_classes_and_roots": descriptor
        }));
    }
    let key = json!({"candidate_descriptors": relative, "fence_relative_to_last": fence - last});
    let starts = [0, 1].map(|i| {
        let distance = onsets[i] - last;
        let admissible = (2..=4).contains(&distance) && mask & (1 << (distance - 2)) != 0;
        json!({"literal": pair.literals[i], "is_gold": reading == i, "start": onsets[i],
               "after_last_fact_token": distance, "joint_mask": admissible})
    });
    let cuts = [
        cursor(&ids, onsets[0], fence)?,
        cursor(&ids, onsets[1], fence)?,
    ];
    let intended_tokens = (fact_start..=fence).map(|at| {
        let id = ids[at];
        json!({"at": at, "id": id, "piece": tokenizer.decode(&[id]), "word_class": classes[id as usize], "packed_root": roots[id as usize]})
    }).collect::<Vec<_>>();
    let raw = json!({
        "context": context, "text": text, "prefix_ids": ids, "supplied_query_span": [start, end],
        "intended_fact_span": [fact_start, fact_end], "intended_fact_tokens_including_fence": intended_tokens,
        "intended_witness": witness, "all_witnesses": witnesses, "binder": decision,
        "starts": starts, "offsets": offsets, "forced_cursor": cuts
    });
    Ok((raw, key))
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [model_path, tokenizer_path] = args.as_slice() else {
        return Err("usage: d24-boundary-witness FROZEN_D11 TOKENIZER".into());
    };
    let model = IntegerDiscreteModel::load(Path::new(model_path))?;
    let tokenizer_data = fs::read(tokenizer_path)?;
    let tokenizer_hash = format!("{:x}", Sha256::digest(&tokenizer_data));
    if model.artifact_sha256() != MODEL
        || tokenizer_hash != TOKENIZER
        || model.tokenizer_sha256() != Some(TOKENIZER)
    {
        return Err("artifact or tokenizer pin mismatch".into());
    }
    let onset_mode = model
        .onset_mode()
        .ok_or("artifact has no fitted onset bank")?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_data).ok_or("invalid tokenizer")?;
    let mut pairs = Vec::new();
    for pair in PAIRS {
        let (early, early_key) = inspect(&model, &tokenizer, &pair, 0)?;
        let (late, late_key) = inspect(&model, &tokenizer, &pair, 1)?;
        pairs.push(json!({
            "shared_sentence": pair.sentence, "query": pair.question, "name": pair.name,
            "same_relative_candidate_descriptor": early_key == late_key,
            "relative_candidate_descriptor": {"early":early_key, "late":late_key},
            "early": early, "late": late
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "model_sha256": MODEL, "tokenizer_sha256": TOKENIZER, "onset_mode":onset_mode, "pairs": pairs
        }))?
    );
    Ok(())
}
