//! Dialogue learning and replies for the geometric stack
//! ([`crate::geometric_stack`]).
//!
//! The stack learns the prepared literal-role corpus
//! (`uor_r4_tokenizer::dialogue`, schema `uor-r4.literal-role-dialogue/1`)
//! through the retained dialogue study's episodes
//! ([`crate::dialogue_episodes`]): the same contract, eligibility,
//! response-uniform sampler and source-stratified development panel
//! ([`crate::dialogue_development::select`]). The contract is built the way
//! that study builds it: UNK pads, and the assistant marker is the empty
//! assistant prefix after BOS. Replies to a request panel also follow the
//! study: greedy, stopped at EOS, at a short terminal cycle
//! ([`crate::reference_eval::short_cycle_period`]) or at a token cap, and an
//! unfinished reply is closed with EOS by the caller before the next user
//! turn.
//!
//! A prepared split is a UORT token store ([`MmapCorpusReader`]), one
//! response-mask byte per token, and the split manifest, whose `files` give
//! each source's label and token count in store order.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use candle_core::Tensor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::dialogue_episodes::{
    EpisodeBatch, EpisodeContract, EpisodeIndex, PrefixPolicy, SourceSpan, EPISODE_CONTEXT,
};
use crate::geometric_stack::{PointerRowStats, StackModel, TargetScores};
use crate::reference_eval::short_cycle_period;
use crate::{invalid, Result};

/// One prepared split of the dialogue corpus.
pub struct DialogueSplit {
    reader: MmapCorpusReader,
    mask: Vec<u8>,
    sources: Vec<SourceSpan>,
}

impl DialogueSplit {
    /// Read a split's token store, response mask and manifest. The manifest
    /// must record no literal special-token text, overall and per source, as
    /// the retained study requires.
    pub fn load(tokens: &Path, mask: &Path, manifest: &Path) -> Result<Self> {
        let reader = MmapCorpusReader::open(tokens)
            .map_err(|e| invalid(format!("dialogue token store: {e}")))?;
        let mask = fs::read(mask)?;
        let manifest: Value = serde_json::from_slice(&fs::read(manifest)?)?;
        if reader.as_slice().len() != mask.len() || !cfg!(target_endian = "little") {
            return Err(invalid("dialogue tokens and mask differ in length"));
        }
        if manifest["drops"]["special_token_occurrences"] != 0 {
            return Err(invalid("the split holds literal special-token text"));
        }
        let files = manifest["files"]
            .as_array()
            .ok_or_else(|| invalid("the split manifest has no files"))?;
        let mut sources = Vec::with_capacity(files.len());
        let mut start = 0usize;
        for file in files {
            let tokens = file["tokens"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| invalid("a source without a token count"))?;
            if file["special_token_occurrences"] != 0 {
                return Err(invalid("a source holds literal special-token text"));
            }
            let label = file["label"]
                .as_str()
                .ok_or_else(|| invalid("a source without a label"))?;
            let end = start
                .checked_add(tokens)
                .ok_or_else(|| invalid("source lengths overflow"))?;
            sources.push(SourceSpan {
                label: label.to_owned(),
                start,
                end,
            });
            start = end;
        }
        if start != mask.len() {
            return Err(invalid("the sources do not cover the token store"));
        }
        Ok(Self {
            reader,
            mask,
            sources,
        })
    }

    /// The store's declared vocabulary size.
    pub fn vocab_size(&self) -> usize {
        self.reader.vocab_size() as usize
    }

    /// Every episode of the split under `contract`.
    pub fn index(&self, contract: EpisodeContract) -> Result<EpisodeIndex<'_>> {
        EpisodeIndex::new(self.reader.as_slice(), &self.mask, contract, &self.sources)
    }
}

/// The literal-role protocol of `tokenizer` and the episode contract the
/// retained study builds from it.
pub fn episode_contract(
    tokenizer: &ByteBpeTokenizer,
    vocab_size: usize,
) -> Result<(DialogueProtocol, EpisodeContract)> {
    let protocol =
        DialogueProtocol::literal_roles_v1(tokenizer).map_err(|e| invalid(e.to_string()))?;
    let encoder = protocol
        .bind(tokenizer)
        .map_err(|e| invalid(e.to_string()))?;
    let marker = encoder.encode_assistant_prefix(&[]);
    if marker.tokens.first() != Some(&protocol.bos_id) {
        return Err(invalid(
            "the empty assistant prefix does not start with BOS",
        ));
    }
    let contract = EpisodeContract {
        context: EPISODE_CONTEXT,
        vocab_size,
        bos_id: protocol.bos_id,
        eos_id: protocol.eos_id,
        unk_id: protocol.unk_id,
        padding_id: protocol.unk_id,
        assistant_marker_ids: marker.tokens[1..].to_vec(),
    };
    Ok((protocol, contract))
}

/// An episode batch without the columns after its longest episode: inputs,
/// targets, weights and the trimmed time. Padding follows each episode's last
/// target and the stack is causal, so no weighted target changes; only the
/// wasted positions go.
pub struct Trimmed {
    pub inputs: Vec<u32>,
    pub targets: Vec<u32>,
    pub weights: Vec<f32>,
    pub time: usize,
}

pub fn trim(batch: &EpisodeBatch) -> Trimmed {
    let time = batch
        .rows
        .iter()
        .map(|row| row.counts.real_input_positions)
        .max()
        .unwrap_or(1)
        .max(1);
    let keep = |values: &[u32]| -> Vec<u32> {
        values
            .chunks(batch.time)
            .flat_map(|row| row[..time].iter().copied())
            .collect()
    };
    Trimmed {
        inputs: keep(&batch.inputs),
        targets: keep(&batch.targets),
        weights: batch
            .weights
            .chunks(batch.time)
            .flat_map(|row| row[..time].iter().copied())
            .collect(),
        time,
    }
}

/// What a pointer head did over the scored targets of a panel.
#[derive(Clone, Default)]
struct PointerTotals {
    scored: usize,
    gate: f64,
    copy_mass: f64,
    hits: usize,
    reachable: usize,
}

impl PointerTotals {
    fn add(&mut self, row: &PointerRowStats) {
        self.scored += 1;
        self.gate += row.gate;
        self.copy_mass += row.copy_mass;
        self.hits += usize::from(row.hit);
        self.reachable += usize::from(row.reachable);
    }

    fn merge(&mut self, other: &Self) {
        self.scored += other.scored;
        self.gate += other.gate;
        self.copy_mass += other.copy_mass;
        self.hits += other.hits;
        self.reachable += other.reachable;
    }

    fn report(&self) -> Option<Value> {
        (self.scored != 0).then(|| {
            let n = self.scored as f64;
            json!({
                "scored_targets": self.scored,
                "mean_gate": self.gate / n,
                "pointer_hit_rate": self.hits as f64 / n,
                "target_reachable_rate": self.reachable as f64 / n,
                "mean_copy_mass": self.copy_mass / n,
                "definitions": "over the scored targets: mean_gate is the mean gate g_t; \
                    pointer_hit_rate is the fraction whose most attended source (lowest position \
                    on a tie) holds the target token; target_reachable_rate is the fraction \
                    whose target token is held by any source with attention (the hit rate's \
                    ceiling); mean_copy_mass is the mean p_copy(target) before the gate",
            })
        })
    }
}

#[derive(Clone, Default)]
struct Totals {
    responses: usize,
    targets: usize,
    nll: f64,
    first_targets: usize,
    first_nll: f64,
    eos_targets: usize,
    pointer: PointerTotals,
}

impl Totals {
    fn merge(&mut self, other: &Self) {
        self.responses += other.responses;
        self.targets += other.targets;
        self.nll += other.nll;
        self.first_targets += other.first_targets;
        self.first_nll += other.first_nll;
        self.eos_targets += other.eos_targets;
        self.pointer.merge(&other.pointer);
    }

    fn report(&self) -> Value {
        let mut report = json!({
            "selected_responses": self.responses,
            "supervised_targets": self.targets,
            "eos_targets": self.eos_targets,
            "response_mean_nll": (self.targets != 0).then(|| self.nll / self.targets as f64),
            "first_four_targets": self.first_targets,
            "first_four_response_targets_mean_nll":
                (self.first_targets != 0).then(|| self.first_nll / self.first_targets as f64),
        });
        if let Some(pointer) = self.pointer.report() {
            report["pointer"] = pointer;
        }
        report
    }
}

/// Score a stack on the development responses `ids` under their full original
/// prefixes: the token-mean response NLL and the NLL of each response's first
/// four targets, pooled and per source. The fields are those of the retained
/// study's panel report (`dialogue_development::evaluate`); here every
/// target's NLL is computed in f64 and summed in f64. `score_fn` scores one
/// batch of `(inputs, targets, response weights, batch, time)`; the pointer
/// statistics it returns, if any, are pooled over the response targets.
fn evaluate_development_nll<F>(
    model: &StackModel,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch: usize,
    scope_description: &'static str,
    mut score_fn: F,
) -> Result<Value>
where
    F: FnMut(&[u32], &[u32], &[f32], usize, usize) -> Result<TargetScores>,
{
    let contract = index.contract();
    if !(1..=64).contains(&batch)
        || ids.is_empty()
        || model.config.context != contract.context
        || model.config.vocab_size != contract.vocab_size
    {
        return Err(invalid(
            "development needs responses, a batch of 1..64 and the model's context and vocabulary",
        ));
    }
    let unique: BTreeSet<_> = ids.iter().collect();
    if unique.len() != ids.len() {
        return Err(invalid("duplicate development response"));
    }
    let mut totals = vec![Totals::default(); index.sources().len()];
    let mut selected = vec![Vec::new(); index.sources().len()];
    for chunk in ids.chunks(batch) {
        let episodes = index.materialize(chunk, PrefixPolicy::FullPrefix)?;
        let trimmed = trim(&episodes);
        let time = trimmed.time;
        let scores = score_fn(
            &trimmed.inputs,
            &trimmed.targets,
            &trimmed.weights,
            episodes.batch,
            time,
        )?;
        let nll = &scores.nll;
        for (lane, row) in episodes.rows.iter().enumerate() {
            let total = &mut totals[row.source_index];
            selected[row.source_index].push(row.response_id);
            total.responses += 1;
            total.eos_targets += row.counts.eos_targets;
            let start = lane * time;
            let mut first = 0usize;
            for (offset, (weight, value)) in trimmed.weights[start..start + time]
                .iter()
                .zip(&nll[start..start + time])
                .enumerate()
            {
                if *weight == 1.0 {
                    if let Some(Some(stats)) = scores.pointer.as_ref().map(|p| &p[start + offset]) {
                        total.pointer.add(stats);
                    }
                    total.targets += 1;
                    total.nll += value;
                    if first < 4 {
                        total.first_targets += 1;
                        total.first_nll += value;
                        first += 1;
                    }
                }
            }
        }
    }
    let mut pooled = Totals::default();
    let mut sources = Vec::with_capacity(totals.len());
    for (source_index, total) in totals.iter().enumerate() {
        pooled.merge(total);
        let mut report = total.report();
        report["source_index"] = json!(source_index);
        report["label"] = json!(index.sources()[source_index].label);
        report["eligible_responses"] =
            json!(index.population().sources[source_index].eligible_responses);
        report["response_ids"] = json!(selected[source_index]);
        sources.push(report);
    }
    let mut report = pooled.report();
    report["schema"] = json!("uor-r4.stack-dialogue-development/1");
    report["response_ids"] = json!(ids);
    report["per_source"] = json!(sources);
    report["conditioning"] = json!("full_original_prefix");
    report["scope"] = json!(scope_description);
    Ok(report)
}

/// Score a stack on the development responses `ids` under their full original
/// prefixes using an explicit output head tensor: the token-mean response NLL
/// and the NLL of each response's first four targets, pooled and per source.
/// The explicit head scores raw logits, which are not a pointer model's
/// distribution (the mixture), so a pointer model is refused.
pub fn development_with_head(
    model: &StackModel,
    head: &Tensor,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch: usize,
) -> Result<Value> {
    if model.config.pointer.is_some() {
        return Err(invalid(
            "development_with_head scores raw logits through an explicit head; a pointer \
             model's distribution is its mixture (use development)",
        ));
    }
    evaluate_development_nll(
        model,
        index,
        ids,
        batch,
        "Token means over the selected development responses using an explicit output head, not the full corpus or an equal-source mean.",
        |inputs, targets, _weights, batch_size, time| {
            Ok(TargetScores {
                nll: model.target_nll_with_head(inputs, targets, head, batch_size, time)?,
                pointer: None,
            })
        },
    )
}

/// Score a stack on the development responses `ids` under their full original
/// prefixes: the token-mean response NLL and the NLL of each response's first
/// four targets, pooled and per source. The fields are those of the retained
/// study's panel report (`dialogue_development::evaluate`); here every
/// target's NLL is computed in f64 and summed in f64.
pub fn development(
    model: &StackModel,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch: usize,
) -> Result<Value> {
    evaluate_development_nll(
        model,
        index,
        ids,
        batch,
        "Token means over the selected development responses, not the full corpus or an equal-source mean.",
        // Only the response targets are read, so a pointer head is run on them
        // alone (the mixture, with its statistics); a model without one scores
        // every position, as `target_nll` does.
        |inputs, targets, weights, batch_size, time| {
            model.score_targets(inputs, targets, Some(weights), batch_size, time)
        },
    )
}

/// One request of a reply panel: user turns answered in order, the format of
/// the retained study's request files.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub category: String,
    pub user_turns: Vec<String>,
}

/// The retained study's panel limits: requests per panel, user turns per
/// request and generated ids per reply.
pub const MAX_REQUESTS: usize = 128;
pub const MAX_USER_TURNS: usize = 8;
pub const MAX_NEW_TOKENS: usize = 128;

/// Read a request panel: 1 to [`MAX_REQUESTS`] requests with distinct ids,
/// each with 1 to [`MAX_USER_TURNS`] nonblank user turns, as the retained
/// study requires.
pub fn load_requests(path: &Path) -> Result<Vec<Request>> {
    let requests: Vec<Request> = serde_json::from_slice(&fs::read(path)?)?;
    let ids: BTreeSet<&str> = requests.iter().map(|r| r.id.as_str()).collect();
    if requests.is_empty()
        || requests.len() > MAX_REQUESTS
        || ids.len() != requests.len()
        || requests.iter().any(|r| {
            r.user_turns.is_empty()
                || r.user_turns.len() > MAX_USER_TURNS
                || r.user_turns.iter().any(|t| t.trim().is_empty())
        })
    {
        return Err(invalid(format!(
            "a request panel needs 1 to {MAX_REQUESTS} requests with distinct ids, each with 1 to \
             {MAX_USER_TURNS} nonblank user turns"
        )));
    }
    Ok(requests)
}

/// Check that the whole panel can be answered before anything is generated
/// (or trained): a positive cap of at most [`MAX_NEW_TOKENS`], every user
/// turn one protocol turn with no special tokens, and, as in the retained
/// study, every request's history within `context` even if each reply reaches
/// the cap and is closed with EOS by the caller. Nothing is truncated.
pub fn check_panel(
    encoder: &DialogueEncoder<'_>,
    requests: &[Request],
    context: usize,
    max_new_tokens: usize,
) -> Result<()> {
    if max_new_tokens == 0 || max_new_tokens > MAX_NEW_TOKENS {
        return Err(invalid(format!(
            "max_new_tokens must be 1 to {MAX_NEW_TOKENS}"
        )));
    }
    for request in requests {
        let mut longest = 1usize;
        for (turn, user) in request.user_turns.iter().enumerate() {
            let prefix = encoder.encode_user_prefix(user, turn != 0);
            if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                return Err(invalid(format!(
                    "request {}: turn {} is not one plain user turn",
                    request.id,
                    turn + 1
                )));
            }
            longest += prefix.tokens.len() + max_new_tokens;
            if longest > context {
                return Err(invalid(format!(
                    "request {}: with every reply at {max_new_tokens} ids the history reaches \
                     {longest} of {context} positions by turn {}",
                    request.id,
                    turn + 1
                )));
            }
            if turn + 1 < request.user_turns.len() {
                longest += 1;
            }
        }
    }
    Ok(())
}

/// A generated reply: its ids (ending in EOS when the model ended it),
/// whether the model ended it, and the period of the short terminal cycle
/// that stopped it, if one did.
pub struct Reply {
    pub ids: Vec<u32>,
    pub eos: bool,
    pub cycle: Option<usize>,
}

impl Reply {
    /// The retained study's stop rules after each generated id: EOS, then a
    /// short terminal cycle. `None` means the reply continues.
    pub fn stop(ids: &[u32], eos: u32) -> Option<Self> {
        let last = *ids.last()?;
        if last == eos {
            return Some(Self {
                ids: ids.to_vec(),
                eos: true,
                cycle: None,
            });
        }
        short_cycle_period(ids).map(|period| Self {
            ids: ids.to_vec(),
            eos: false,
            cycle: Some(period),
        })
    }

    /// How the reply stopped: `"eos"`, `{"short_cycle": period}` or
    /// `"max_new_tokens"`.
    pub fn stop_record(&self) -> Value {
        match (self.eos, self.cycle) {
            (true, _) => json!("eos"),
            (false, Some(period)) => json!({"short_cycle": period}),
            (false, None) => json!("max_new_tokens"),
        }
    }
}

/// Answer every request. For each user turn, the turn's prefix is appended
/// to the history and `reply(history, cap)` generates up to `cap` ids. A reply
/// the model did not end is closed with EOS before the next user turn. The
/// panel is checked first ([`check_panel`]), so the whole history fits the
/// context with every reply at its cap; nothing is truncated.
pub fn reply_panel(
    encoder: &DialogueEncoder<'_>,
    protocol: &DialogueProtocol,
    requests: &[Request],
    context: usize,
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    reply: &mut dyn FnMut(&[u32], usize) -> Result<Reply>,
) -> Result<Value> {
    check_panel(encoder, requests, context, max_new_tokens)?;
    let mut rows = Vec::with_capacity(requests.len());
    for request in requests {
        let mut history = vec![protocol.bos_id];
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
            let generated = reply(&history, max_new_tokens)?;
            history.extend(&generated.ids);
            let closed = !generated.eos && turn + 1 < request.user_turns.len();
            if closed {
                history.push(protocol.eos_id);
            }
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
    Ok(json!({"rows": rows, "max_new_tokens": max_new_tokens}))
}

/// The float stack's greedy reply to `history`: the highest logit (ties to the
/// lower id) until EOS, a short terminal cycle or `cap` ids. Each step
/// recomputes the whole window.
pub fn greedy_reply(model: &StackModel, history: &[u32], cap: usize, eos: u32) -> Result<Reply> {
    let mut window = history.to_vec();
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if window.len() > model.config.context {
            return Err(invalid("the reply outgrew the context"));
        }
        // The last position's logits, or a pointer model's mixture scores.
        let last = model.next_scores(&window)?;
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue_development;
    use crate::geometric_stack::{ReadScore, StackAdamW, StackArch, StackConfig};
    use candle_core::Device;
    use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
    use uor_r4_tokenizer::dialogue::Message;

    /// GPT-2's byte-to-character alphabet.
    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut table = vec!['\0'; 256];
        let mut extra = 0;
        for byte in 0u32..256 {
            table[byte as usize] = if printable.contains(&byte) {
                char::from_u32(byte).unwrap()
            } else {
                extra += 1;
                char::from_u32(255 + extra).unwrap()
            };
        }
        table
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2.
    fn tokenizer() -> ByteBpeTokenizer {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        for (byte, ch) in alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    const VOCAB: usize = 288;

    /// Two sources of one- and two-turn conversations whose answers follow
    /// from the questions, written as a prepared split.
    fn split(directory: &Path, tokenizer: &ByteBpeTokenizer) -> DialogueSplit {
        let protocol = DialogueProtocol::literal_roles_v1(tokenizer).unwrap();
        let encoder = protocol.bind(tokenizer).unwrap();
        let mut tokens = Vec::new();
        let mut mask = Vec::new();
        let mut files = Vec::new();
        for (label, pairs) in [
            ("colors", [("sky?", "blue"), ("grass?", "green")]),
            ("sounds", [("cat?", "meow"), ("dog?", "woof")]),
        ] {
            let start = tokens.len();
            let turn = |(question, answer): (&'static str, &'static str)| {
                [
                    Message {
                        role: "user",
                        content: question,
                    },
                    Message {
                        role: "assistant",
                        content: answer,
                    },
                ]
            };
            let (a, b) = (turn(pairs[0]), turn(pairs[1]));
            let documents = [a.to_vec(), b.to_vec(), [a, b].concat(), [b, a].concat()];
            for _ in 0..3 {
                for messages in &documents {
                    let document = encoder.encode_document(messages);
                    tokens.extend(document.tokens.iter().map(|&id| id as u16));
                    mask.extend(document.response_mask);
                }
            }
            files.push(json!({"label": label, "tokens": tokens.len() - start,
                "special_token_occurrences": 0}));
        }
        let paths = [
            directory.join("tokens.uort"),
            directory.join("mask.bin"),
            directory.join("manifest.json"),
        ];
        CorpusWriter::write_file(&paths[0], VOCAB as u32, &tokens).unwrap();
        fs::write(&paths[1], &mask).unwrap();
        fs::write(
            &paths[2],
            json!({"files": files, "drops": {"special_token_occurrences": 0}}).to_string(),
        )
        .unwrap();
        DialogueSplit::load(&paths[0], &paths[1], &paths[2]).unwrap()
    }

    fn stack() -> StackModel {
        let mut config = StackConfig::transformer_control(5);
        config.arch = StackArch::Geometric;
        config.vocab_size = VOCAB;
        config.width = 32;
        config.heads = 2;
        config.mlp_hidden = 64;
        config.pattern = "ra".to_owned();
        config.read = ReadScore::Lorentz;
        StackModel::new(config, &Device::Cpu).unwrap()
    }

    #[test]
    fn the_stack_learns_responses_of_a_prepared_split_and_answers_requests() {
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-stack-dialogue-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let tokenizer = tokenizer();
        let split = split(&directory, &tokenizer);
        let (protocol, contract) = episode_contract(&tokenizer, split.vocab_size()).unwrap();
        assert_eq!(contract.padding_id, protocol.unk_id);
        assert_eq!(
            tokenizer.decode(&contract.assistant_marker_ids),
            "Assistant: "
        );
        let index = split.index(contract).unwrap();
        // Per source, three copies of two one-turn and two two-turn documents.
        assert_eq!(index.episodes().len(), 2 * 3 * (1 + 1 + 2 + 2));
        let panel = dialogue_development::select(&index, 7, 2).unwrap();
        assert_eq!(panel.len(), 4);

        let model = stack();
        let before = development(&model, &index, &panel, 4).unwrap();
        // Trimming the padding columns leaves the weighted loss unchanged.
        let ids = index.sample_ids(3, 0, 8).unwrap();
        let batch = index.materialize(&ids, PrefixPolicy::FullPrefix).unwrap();
        let trimmed = trim(&batch);
        assert!(trimmed.time < 64);
        let full = model
            .weighted_loss(
                &batch.inputs,
                &batch.targets,
                &batch.weights,
                batch.batch,
                batch.time,
            )
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        let short = model
            .weighted_loss(
                &trimmed.inputs,
                &trimmed.targets,
                &trimmed.weights,
                batch.batch,
                trimmed.time,
            )
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        assert!(
            (full - short).abs() < 1e-5,
            "trimmed loss {short} against {full}"
        );
        let mut optimizer = StackAdamW::new(&model, 0.0, 1.0).unwrap();
        for step in 0..240u64 {
            let ids = index.sample_ids(3, step, 8).unwrap();
            let batch = index.materialize(&ids, PrefixPolicy::FullPrefix).unwrap();
            let trimmed = trim(&batch);
            // Only response targets carry weight.
            for ((&weight, &target), &input) in trimmed
                .weights
                .iter()
                .zip(&trimmed.targets)
                .zip(&trimmed.inputs)
            {
                if weight != 0.0 {
                    assert_ne!(target, protocol.bos_id);
                    assert_ne!(input, index.contract().padding_id);
                }
            }
            let loss = model
                .weighted_loss(
                    &trimmed.inputs,
                    &trimmed.targets,
                    &trimmed.weights,
                    batch.batch,
                    trimmed.time,
                )
                .unwrap();
            let grads = loss.backward().unwrap();
            optimizer.update(&model, &grads, 0.01).unwrap();
        }
        let after = development(&model, &index, &panel, 4).unwrap();
        let nll = |report: &Value| report["response_mean_nll"].as_f64().unwrap();
        assert!(
            nll(&after) < 0.5 * nll(&before),
            "response NLL {} -> {}",
            nll(&before),
            nll(&after)
        );
        assert_eq!(after["per_source"].as_array().unwrap().len(), 2);
        assert_eq!(after["selected_responses"], 4);

        let requests = vec![Request {
            id: "r1".into(),
            category: "test".into(),
            user_turns: vec!["sky?".into(), "grass?".into()],
        }];
        let encoder = protocol.bind(&tokenizer).unwrap();
        let panel = reply_panel(
            &encoder,
            &protocol,
            &requests,
            model.config.context,
            8,
            &|ids| tokenizer.decode(ids),
            &mut |history, cap| greedy_reply(&model, history, cap, protocol.eos_id),
        )
        .unwrap();
        let turns = panel["rows"][0]["turns"].as_array().unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0]["reply"], "blue");
        assert_eq!(turns[0]["model_eos"], true);
        assert_eq!(turns[0]["stop"], "eos");
        assert_eq!(turns[1]["reply"], "green");
        let history: Vec<u32> =
            serde_json::from_value(panel["rows"][0]["history_ids"].clone()).unwrap();
        assert_eq!(
            tokenizer.decode(&history),
            "<|bos|>User: sky?\nAssistant: blue<|eos|>\nUser: grass?\nAssistant: green<|eos|>"
        );
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_pointer_stack_learns_the_split_and_reports_its_gate_and_hits() {
        use crate::flock::FlockSelect;
        use crate::geometric_stack::PointerConfig;
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-stack-dialogue-pointer-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let tokenizer = tokenizer();
        let split = split(&directory, &tokenizer);
        let (protocol, contract) = episode_contract(&tokenizer, split.vocab_size()).unwrap();
        let index = split.index(contract).unwrap();
        let panel = dialogue_development::select(&index, 7, 2).unwrap();
        let mut config = stack().config.clone();
        // A Lorentz pointer that softmaxes over every source (its own
        // selection is none), beside a flock on the reads, which the pointer
        // does not use.
        config.pointer = Some(PointerConfig {
            score: ReadScore::Lorentz,
            ..PointerConfig::new(8)
        });
        config.select = Some(FlockSelect {
            sink: 0,
            window: 8,
            k: 2,
        });
        let mut model = StackModel::new(config, &Device::Cpu).unwrap();
        let before = development(&model, &index, &panel, 4).unwrap();
        let pointer = &before["pointer"];
        // The head is scored on exactly the supervised targets, and its gate
        // starts near sigmoid(-2).
        assert_eq!(pointer["scored_targets"], before["supervised_targets"]);
        let gate = pointer["mean_gate"].as_f64().unwrap();
        assert!((0.05..0.3).contains(&gate), "initial mean gate {gate}");
        assert!(before["per_source"][0]["pointer"]["pointer_hit_rate"].is_number());
        let mut optimizer = StackAdamW::new(&model, 0.0, 1.0).unwrap();
        for step in 0..240u64 {
            let ids = index.sample_ids(3, step, 8).unwrap();
            let batch = index.materialize(&ids, PrefixPolicy::FullPrefix).unwrap();
            let trimmed = trim(&batch);
            let loss = model
                .weighted_loss(
                    &trimmed.inputs,
                    &trimmed.targets,
                    &trimmed.weights,
                    batch.batch,
                    trimmed.time,
                )
                .unwrap();
            let grads = loss.backward().unwrap();
            optimizer.update(&model, &grads, 0.01).unwrap();
        }
        let after = development(&model, &index, &panel, 4).unwrap();
        let nll = |report: &Value| report["response_mean_nll"].as_f64().unwrap();
        assert!(
            nll(&after) < 0.5 * nll(&before),
            "response NLL {} -> {}",
            nll(&before),
            nll(&after)
        );
        assert!(after["pointer"]["mean_gate"].is_number());
        // The same weights scored with the single-source pointer, post hoc: a
        // single kept source copies with weight 1 or 0, so on every scored
        // target the copy mass is the hit and the reachable indicator.
        model
            .set_pointer_select(Some(crate::geometric_stack::PointerSelect::TopK(1)))
            .unwrap();
        let single = development(&model, &index, &panel, 4).unwrap();
        assert_eq!(
            single["pointer"]["scored_targets"],
            after["pointer"]["scored_targets"]
        );
        let rate = |key: &str| single["pointer"][key].as_f64().unwrap();
        assert!(
            (rate("mean_copy_mass") - rate("pointer_hit_rate")).abs() < 1e-9,
            "copy mass {} against hit rate {}",
            rate("mean_copy_mass"),
            rate("pointer_hit_rate")
        );
        assert!((rate("target_reachable_rate") - rate("pointer_hit_rate")).abs() < 1e-9);
        model.set_pointer_select(None).unwrap();
        // Greedy replies come from the mixture and stop at EOS.
        let requests = vec![Request {
            id: "r1".into(),
            category: "test".into(),
            user_turns: vec!["sky?".into()],
        }];
        let encoder = protocol.bind(&tokenizer).unwrap();
        let replies = reply_panel(
            &encoder,
            &protocol,
            &requests,
            model.config.context,
            8,
            &|ids| tokenizer.decode(ids),
            &mut |history, cap| greedy_reply(&model, history, cap, protocol.eos_id),
        )
        .unwrap();
        assert_eq!(replies["rows"][0]["turns"][0]["reply"], "blue");
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_panel_is_checked_whole_before_anything_is_generated() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let request = |id: &str, turns: &[&str]| Request {
            id: id.into(),
            category: "test".into(),
            user_turns: turns.iter().map(|t| (*t).to_owned()).collect(),
        };
        let two = [request("r1", &["sky?", "grass?"])];
        let first = encoder.encode_user_prefix("sky?", false).tokens.len();
        let second = encoder.encode_user_prefix("grass?", true).tokens.len();
        // BOS, both prefixes, both replies at the cap and the caller's EOS
        // between the turns.
        let cap = 8;
        let worst = 1 + first + cap + 1 + second + cap;
        assert!(check_panel(&encoder, &two, worst, cap).is_ok());
        assert!(check_panel(&encoder, &two, worst - 1, cap).is_err());
        assert!(check_panel(&encoder, &two, 256, 0).is_err());
        assert!(check_panel(&encoder, &two, 4096, MAX_NEW_TOKENS + 1).is_err());
        let special = [request("r2", &["say <|eos|> now"])];
        assert!(check_panel(&encoder, &special, 256, cap).is_err());
        // A panel whose second turn cannot fit is refused before the first
        // reply is generated.
        let mut calls = 0;
        let refused = reply_panel(
            &encoder,
            &protocol,
            &two,
            worst - 1,
            cap,
            &|ids| tokenizer.decode(ids),
            &mut |_, _| {
                calls += 1;
                Ok(Reply {
                    ids: vec![protocol.eos_id],
                    eos: true,
                    cycle: None,
                })
            },
        );
        assert!(refused.is_err());
        assert_eq!(calls, 0);

        let directory = std::env::temp_dir().join(format!(
            "uor-r4-stack-panel-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let load = |name: &str, panel: Value| {
            let path = directory.join(name);
            fs::write(&path, panel.to_string()).unwrap();
            load_requests(&path)
        };
        let row = |id: String, turns: usize| json!({"id": id, "category": "c", "user_turns": vec!["hi"; turns]});
        assert!(load("ok.json", json!([row("a".into(), 1), row("b".into(), 8)])).is_ok());
        assert!(load("empty.json", json!([])).is_err());
        assert!(load("same.json", json!([row("a".into(), 1), row("a".into(), 1)])).is_err());
        assert!(load("turns.json", json!([row("a".into(), 9)])).is_err());
        assert!(load("none.json", json!([row("a".into(), 0)])).is_err());
        assert!(load(
            "blank.json",
            json!([{"id": "a", "category": "c", "user_turns": ["hi", "  "]}])
        )
        .is_err());
        let many: Vec<Value> = (0..=MAX_REQUESTS)
            .map(|i| row(format!("r{i}"), 1))
            .collect();
        assert!(load("many.json", Value::Array(many)).is_err());
        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn replies_stop_at_eos_then_at_a_short_terminal_cycle() {
        let stop = |ids: &[u32]| Reply::stop(ids, 1).map(|r| (r.eos, r.cycle));
        assert_eq!(stop(&[5, 6, 1]), Some((true, None)));
        assert_eq!(stop(&[5, 6, 7]), None);
        assert_eq!(stop(&[7, 7, 7]), Some((false, Some(1))));
        assert_eq!(stop(&[3, 8, 9, 8, 9, 8, 9]), Some((false, Some(2))));
        assert_eq!(stop(&[8, 9, 8, 9]), None);
        // EOS is checked first, as in the retained study.
        assert_eq!(stop(&[1, 1, 1]), Some((true, None)));
    }
}
