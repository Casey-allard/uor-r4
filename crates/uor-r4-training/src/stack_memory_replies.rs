//! Reply-level memory evaluation (Lab 1; the milestone's recall measure, #973
//! comment 5884046794): greedy replies to a request panel with the relation
//! store in the loop, and the frozen answer key of the panel's ten memory
//! requests.
//!
//! At every generated position the model's own tags and triggers over the
//! whole window drive the prime-route store ([`crate::stack_prime_route`]:
//! semiprime-expert keys, sieve clause formation for reads and writes, the
//! learned write trigger), and the registers re-enter through the model's
//! memory branch. The `Silent` arm
//! keeps every register at status None, which for a zero-initialised branch
//! is the plain stack: the within-model control for what the store adds.
//! Turn marks come from the conversation itself (the user prefixes and the
//! replies generated so far), never from labels. Evaluation only.

use std::ops::Range;

use candle_core::D;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol};

use crate::stack_aerm::{AermModel, STATUS_NONE};
use crate::stack_dialogue::{check_panel, Reply, Request};
use crate::stack_prime_route::{
    trace_prime_route, PrimeRegistry, ReadPolicy, TurnMarks, WritePolicy,
};
use crate::{invalid, Result};

/// Where the registers come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryArm {
    /// The prime-route store driven by the model's own tags and triggers.
    Store,
    /// Every register silent (status None): no store.
    Silent,
}

/// One memory request's frozen answer: the final reply must contain one of
/// `accept` as a whole word and none of `reject`, case-insensitively.
#[derive(Clone, Copy, Debug)]
pub struct MemoryAnswer {
    pub id: &'static str,
    pub accept: &'static [&'static str],
    pub reject: &'static [&'static str],
}

/// The answer key of `development-requests.json` (SHA-256 `81268b51…`),
/// authored once from each request's first user turn and frozen before any
/// run (#973, comment 5884046794).
pub const PANEL_MEMORY_ANSWERS: [MemoryAnswer; 10] = [
    MemoryAnswer {
        id: "dev-mem-01",
        accept: &["alex"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-02",
        accept: &["momo"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-03",
        accept: &["green"],
        reject: &["blue"],
    },
    MemoryAnswer {
        id: "dev-mem-04",
        accept: &["teacher"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-05",
        accept: &["tokyo"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-06",
        accept: &["two", "2"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-07",
        accept: &["july"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-08",
        accept: &["blue"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-09",
        accept: &["piano"],
        reject: &[],
    },
    MemoryAnswer {
        id: "dev-mem-10",
        accept: &["pizza"],
        reject: &[],
    },
];

/// The lowercase whole words of `text` (runs of letters and digits).
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether `reply` meets `answer`.
pub fn meets(answer: &MemoryAnswer, reply: &str) -> bool {
    let words = words(reply);
    let has = |w: &&str| words.iter().any(|x| x == w);
    answer.accept.iter().any(has) && !answer.reject.iter().any(has)
}

/// Turn marks for `len` positions from reply spans: each reply's positions
/// are reply positions, and the position before each reply is a turn end.
fn marks(len: usize, replies: &[Range<usize>]) -> TurnMarks {
    let mut reply = vec![false; len];
    let mut turn_end = vec![false; len];
    for span in replies {
        for p in span.clone().filter(|&p| p < len) {
            reply[p] = true;
        }
        if span.start > 0 && span.start - 1 < len {
            turn_end[span.start - 1] = true;
        }
    }
    TurnMarks { turn_end, reply }
}

/// The greedy reply to `history` (the highest logit, ties to the lower id)
/// until EOS, a short terminal cycle or `cap` ids, with registers from `arm`.
/// `previous` holds the spans of this conversation's earlier replies. Each
/// step recomputes the whole window.
#[allow(clippy::too_many_arguments)]
pub fn memory_greedy_reply(
    model: &AermModel,
    history: &[u32],
    previous: &[Range<usize>],
    cap: usize,
    eos: u32,
    registry: &PrimeRegistry,
    arm: MemoryArm,
) -> Result<Reply> {
    let context = model.stack.config.context;
    let mut window = history.to_vec();
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if window.len() > context {
            return Err(invalid("the reply outgrew the context"));
        }
        let time = window.len();
        let bottom = model.bottom(&window, 1, time)?;
        let registers = match arm {
            MemoryArm::Silent => (
                vec![STATUS_NONE; time],
                vec![0; time],
                vec![STATUS_NONE; time],
                vec![0; time],
            ),
            MemoryArm::Store => {
                let tags = bottom.tags.argmax(D::Minus1)?.to_vec1::<u32>()?;
                let triggers = bottom.triggers.argmax(D::Minus1)?.to_vec1::<u32>()?;
                let mut spans = previous.to_vec();
                spans.push(history.len()..time);
                let run = trace_prime_route(
                    &window,
                    &tags,
                    &triggers,
                    &marks(time, &spans),
                    eos,
                    registry,
                    WritePolicy::Trigger,
                    ReadPolicy::Sieve,
                )?;
                let r = run.registers;
                (r.status, r.value, r.status_previous, r.value_previous)
            }
        };
        let logits = model.top(
            &bottom.hidden,
            &registers.0,
            &registers.1,
            &registers.2,
            &registers.3,
        )?;
        let last = logits.get(time - 1)?.to_vec1::<f32>()?;
        let mut best = 0usize;
        for (i, v) in last.iter().enumerate() {
            if *v > last[best] {
                best = i;
            }
        }
        let next = best as u32;
        ids.push(next);
        window.push(next);
        if let Some(reply) = Reply::stop(&ids, eos) {
            return Ok(reply);
        }
    }
    Ok(Reply {
        ids,
        eos: false,
        cycle: None,
        trace: Vec::new(),
        stopped_at: None,
        copy_stop: None,
    })
}

/// Answer every request with registers from `arm`, in the format of
/// [`crate::stack_dialogue::reply_panel`]: each user turn's prefix is
/// appended, a reply is generated, and a reply the model did not end is
/// closed with EOS before the next user turn. The panel is checked first.
#[allow(clippy::too_many_arguments)]
pub fn memory_reply_panel(
    model: &AermModel,
    encoder: &DialogueEncoder<'_>,
    protocol: &DialogueProtocol,
    requests: &[Request],
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    registry: &PrimeRegistry,
    arm: MemoryArm,
) -> Result<Value> {
    let context = model.stack.config.context;
    check_panel(encoder, requests, context, max_new_tokens)?;
    let mut rows = Vec::with_capacity(requests.len());
    for request in requests {
        let mut history = vec![protocol.bos_id];
        let mut spans: Vec<Range<usize>> = Vec::new();
        let mut turns = Vec::with_capacity(request.user_turns.len());
        for (turn, user) in request.user_turns.iter().enumerate() {
            let prefix = encoder.encode_user_prefix(user, turn != 0);
            if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                return Err(invalid(format!("request {}: turn {turn}", request.id)));
            }
            history.extend(&prefix.tokens);
            if history.len() + max_new_tokens > context {
                return Err(invalid(format!(
                    "request {}: the history and a capped reply exceed the context",
                    request.id
                )));
            }
            let start = history.len();
            let generated = memory_greedy_reply(
                model,
                &history,
                &spans,
                max_new_tokens,
                protocol.eos_id,
                registry,
                arm,
            )?;
            history.extend(&generated.ids);
            let closed = !generated.eos && turn + 1 < request.user_turns.len();
            if closed {
                history.push(protocol.eos_id);
            }
            spans.push(start..history.len());
            let text_ids: Vec<u32> = generated
                .ids
                .iter()
                .copied()
                .filter(|&id| id != protocol.eos_id)
                .collect();
            turns.push(json!({
                "turn": turn + 1, "user": user, "reply": decode(&text_ids),
                "reply_ids": generated.ids, "model_eos": generated.eos,
                "stop": generated.stop_record(),
                "caller_eos_inserted_before_next_request": closed,
            }));
        }
        rows.push(json!({
            "id": request.id, "category": request.category, "turns": turns,
            "history_ids": history,
        }));
    }
    Ok(json!({"rows": rows, "max_new_tokens": max_new_tokens, "arm": arm}))
}

/// Score a panel's memory rows against [`PANEL_MEMORY_ANSWERS`]: the final
/// turn's reply of each keyed request. Every key must be present in `panel`.
pub fn score_memory(panel: &Value) -> Result<Value> {
    let rows = panel["rows"]
        .as_array()
        .ok_or_else(|| invalid("a panel has rows"))?;
    let mut correct = 0usize;
    let mut per_request = Vec::with_capacity(PANEL_MEMORY_ANSWERS.len());
    for answer in &PANEL_MEMORY_ANSWERS {
        let row = rows
            .iter()
            .find(|r| r["id"] == answer.id)
            .ok_or_else(|| invalid(format!("the panel has no request {}", answer.id)))?;
        let reply = row["turns"]
            .as_array()
            .and_then(|turns| turns.last())
            .and_then(|t| t["reply"].as_str())
            .ok_or_else(|| invalid(format!("request {} has no final reply", answer.id)))?;
        let ok = meets(answer, reply);
        correct += usize::from(ok);
        per_request.push(json!({"id": answer.id, "reply": reply, "correct": ok}));
    }
    Ok(json!({
        "correct": correct,
        "of": PANEL_MEMORY_ANSWERS.len(),
        "requests": per_request,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_judge_matches_whole_words_and_rejects_distractors() {
        let alex = &PANEL_MEMORY_ANSWERS[0];
        assert!(meets(alex, "Your name is Alex."));
        assert!(meets(alex, "ALEX!"));
        assert!(!meets(alex, "Your name is Alexander."));
        assert!(!meets(alex, "I don't know."));
        let green = &PANEL_MEMORY_ANSWERS[2];
        assert!(meets(green, "Your favorite color is green."));
        assert!(!meets(green, "You like green and blue."));
        let two = &PANEL_MEMORY_ANSWERS[5];
        assert!(meets(two, "You have 2 brothers."));
        assert!(meets(two, "Two."));
        assert!(!meets(two, "You have twenty brothers."));
    }

    #[test]
    fn the_key_covers_ten_distinct_memory_requests() {
        let ids: std::collections::BTreeSet<&str> =
            PANEL_MEMORY_ANSWERS.iter().map(|a| a.id).collect();
        assert_eq!(ids.len(), 10);
        assert!(PANEL_MEMORY_ANSWERS.iter().all(|a| !a.accept.is_empty()));
    }

    #[test]
    fn marks_put_turn_ends_before_replies() {
        let m = marks(10, &[3..5, 8..10]);
        assert_eq!(
            m.reply,
            vec![false, false, false, true, true, false, false, false, true, true]
        );
        assert_eq!(
            m.turn_end,
            vec![false, false, true, false, false, false, false, true, false, false]
        );
        // An empty span (the first step of a reply) still marks its turn end.
        let m = marks(4, &[4..4]);
        assert_eq!(m.turn_end, vec![false, false, false, true]);
        assert!(m.reply.iter().all(|&r| !r));
    }

    #[test]
    fn scoring_reads_final_replies_and_requires_every_key() -> Result<()> {
        let mut rows: Vec<Value> = PANEL_MEMORY_ANSWERS
            .iter()
            .map(|a| {
                json!({"id": a.id, "category": "memory", "turns": [
                    {"reply": "Hello."},
                    {"reply": format!("It is {}.", a.accept[0])}
                ]})
            })
            .collect();
        let panel = json!({"rows": rows.clone()});
        assert_eq!(score_memory(&panel)?["correct"], 10);
        rows.pop();
        assert!(score_memory(&json!({"rows": rows})).is_err());
        Ok(())
    }
}
