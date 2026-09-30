//! The A1 probe instruments beside M-world v2 (the council's amendments on
//! issue 1511): the sealed English retrieval probe, the probe-overlap
//! exclusion for corpora, and teacher-forced answer-span scoring.
//!
//! **The probe** ([`probe`], `data/a1-english-probe.json`) is 40 items of
//! natural English: MQAR-style recall at short and long distance, a relation
//! stated, updated and asked, and exact copy. Its names, places and numbers are
//! outside every syllable, v1 and v2 pool ([`crate::milestone_world_v2::in_generated_universe`]
//! and [`crate::milestone_world_v2::reserved_words`]), and no user turn contains a
//! three-word fragment of any v1 or v2 template. It is authored data, pinned by
//! SHA-256 ([`probe_sha256`]); each item's checks are derived from its typed
//! fields by [`ProbeItem::checks`] with the same [`Check2`] machinery the M-world
//! oracle uses, so the probe is judged by [`judge_v2`] like every other item.
//! Training excludes it by an overlap rule only: [`conversation_excluding_probe`]
//! drops any world conversation that shares an 8-word user-turn n-gram with it.
//! User turns shorter than 8 words and role-marker lines inside a document can
//! escape that rule, so it is not complete decontamination and does not by
//! itself make the probe a final held-out set.
//!
//! **Teacher-forced answer-span scores** ([`answer_layout`], [`teacher_forced`])
//! give the negative log-likelihood, in nats, of the gold reply and of the
//! gold value inside it, given the reference history and the assistant marker.
//! They call [`StackModel::score_targets`], so a pointer model scores through
//! its mixture, and they report the pointer's gate and hits over the answer
//! tokens. They sit beside greedy accuracy: a model can copy the value under
//! teacher forcing and still fail to generate it.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_stack::StackModel;
use crate::milestone_world::{normalized, words, Check, Split};
use crate::milestone_world_v2::{
    contains_slice, judge_v2, Category2, Check2, Conversation2, Kind, MWorld2, Meter, Pool, Rule,
    Tag, Turn2,
};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// The probe file, embedded so the pinned bytes are the bytes evaluated.
const PROBE_JSON: &str = include_str!("../data/a1-english-probe.json");

/// The probe file's schema name.
pub const PROBE_SCHEMA: &str = "uor-r4.a1-english-probe/1";

/// The probe's item count.
pub const PROBE_ITEMS: usize = 40;

/// The word n-gram length of the corpus exclusion.
pub const EXCLUSION_NGRAM: usize = 8;

/// The kinds of probe item, as reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbeGroup {
    /// Facts stated in one turn, asked for at once.
    MqarShort,
    /// The same, with three unrelated exchanges between statement and query.
    MqarLong,
    /// A relation stated, sometimes updated, another stated after it, then asked.
    Relation,
    /// A line to send back unchanged.
    Copy,
}

impl ProbeGroup {
    pub const ALL: [ProbeGroup; 4] = [
        ProbeGroup::MqarShort,
        ProbeGroup::MqarLong,
        ProbeGroup::Relation,
        ProbeGroup::Copy,
    ];

    /// The group's name in the file and in reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::MqarShort => "mqar-short",
            Self::MqarLong => "mqar-long",
            Self::Relation => "relation",
            Self::Copy => "copy",
        }
    }

    /// How many items of this group the probe holds.
    pub const fn count(self) -> usize {
        match self {
            Self::MqarShort => 8,
            Self::MqarLong => 8,
            Self::Relation => 14,
            Self::Copy => 10,
        }
    }

    fn kind(self) -> Kind {
        match self {
            Self::MqarShort | Self::MqarLong => Kind::Mqar,
            Self::Relation => Kind::Relation,
            Self::Copy => Kind::Copy,
        }
    }

    fn category(self) -> Category2 {
        match self {
            Self::MqarShort | Self::MqarLong => Category2::Mqar,
            Self::Relation => Category2::Relation,
            Self::Copy => Category2::Copy,
        }
    }

    /// The turns an item of this group has: statement and query, statement
    /// with three exchanges and query, two to four, or one.
    const fn turns(self) -> (usize, usize) {
        match self {
            Self::MqarShort => (2, 2),
            Self::MqarLong => (5, 5),
            Self::Relation => (3, 4),
            Self::Copy => (1, 1),
        }
    }
}

/// One turn of a probe item. The last turn of an item is the scored one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeTurn {
    pub user: String,
    /// The reference reply: the item's own earlier replies are the history the
    /// scored turn is answered after.
    pub reply: String,
    /// The values (names, places, numbers) this user turn states.
    #[serde(default)]
    pub values: Vec<String>,
    /// The scored turn of a recall item: the value the reply must state.
    #[serde(default)]
    pub answer: Option<String>,
    /// The values the reply must not state (other facts, an earlier value).
    #[serde(default)]
    pub stale: Vec<String>,
    /// The scored turn of a copy item: the exact words to send back.
    #[serde(default)]
    pub copy: Option<String>,
}

/// One probe item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeItem {
    pub id: String,
    pub group: ProbeGroup,
    pub turns: Vec<ProbeTurn>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeFile {
    schema: String,
    #[allow(dead_code)]
    description: String,
    items: Vec<ProbeItem>,
}

impl ProbeItem {
    fn scored(&self) -> Result<&ProbeTurn> {
        self.turns
            .last()
            .ok_or_else(|| invalid(format!("probe item {} has no turns", self.id)))
    }

    /// The checks of the scored turn, derived from its typed fields: recall
    /// items accept the answer and refuse every stale value; copy items accept
    /// exactly the copied words.
    pub fn checks(&self) -> Result<Vec<Check2>> {
        let turn = self.scored()?;
        match (&turn.answer, &turn.copy) {
            (Some(answer), None) if self.group != ProbeGroup::Copy => {
                let mut checks = vec![Check2::V1(Check::AnyOf(vec![answer.to_lowercase()]))];
                if !turn.stale.is_empty() {
                    checks.push(Check2::V1(Check::NoneOf(
                        turn.stale.iter().map(|s| s.to_lowercase()).collect(),
                    )));
                }
                Ok(checks)
            }
            (None, Some(text)) if self.group == ProbeGroup::Copy => {
                let target = words(text);
                if target.is_empty() {
                    return Err(invalid(format!("probe item {} copies no words", self.id)));
                }
                Ok(vec![Check2::CopyExact(target)])
            }
            _ => Err(invalid(format!(
                "probe item {}: the scored turn's typed fields do not fit its group",
                self.id
            ))),
        }
    }

    /// The item as a conversation of typed turns; the last is the scored turn
    /// and every other turn is context. `tokens` counts the whole document
    /// under `meter`.
    pub fn conversation(&self, meter: &Meter<'_>) -> Result<Conversation2> {
        let last = self.turns.len().saturating_sub(1);
        let checks = self.checks()?;
        let mut turns = Vec::with_capacity(self.turns.len());
        for (index, turn) in self.turns.iter().enumerate() {
            let (intent, category, checks, tag) = if index == last {
                (
                    format!("probe_{}", self.group.name()),
                    self.group.category(),
                    checks.clone(),
                    Tag {
                        pool: (self.group != ProbeGroup::Copy).then_some(Pool::Open),
                        answer: turn.answer.clone(),
                        ..Tag::default()
                    },
                )
            } else {
                (
                    "probe_context".to_owned(),
                    Category2::Responsive,
                    Vec::new(),
                    Tag {
                        values: turn.values.clone(),
                        ..Tag::default()
                    },
                )
            };
            turns.push(Turn2 {
                intent,
                category,
                user: turn.user.clone(),
                reply: turn.reply.clone(),
                checks,
                tag,
            });
        }
        let tokens = meter.document(&turns);
        Ok(Conversation2 {
            kind: self.group.kind(),
            turns,
            tokens,
        })
    }
}

/// The probe's items, checked: 40 items in the four groups, each scored turn's
/// typed fields fitting its group, every reference reply passing its own
/// checks, and every stated value in its turn's text.
pub fn probe() -> Result<Vec<ProbeItem>> {
    let file: ProbeFile = serde_json::from_str(PROBE_JSON)?;
    if file.schema != PROBE_SCHEMA {
        return Err(invalid(format!(
            "the probe file's schema is {}, not {PROBE_SCHEMA}",
            file.schema
        )));
    }
    validate(&file.items)?;
    Ok(file.items)
}

/// SHA-256 of the probe file's bytes.
pub fn probe_sha256() -> String {
    hex::encode(Sha256::digest(PROBE_JSON.as_bytes()))
}

fn validate(items: &[ProbeItem]) -> Result<()> {
    if items.len() != PROBE_ITEMS {
        return Err(invalid(format!(
            "the probe has {} items, not {PROBE_ITEMS}",
            items.len()
        )));
    }
    for group in ProbeGroup::ALL {
        let count = items.iter().filter(|i| i.group == group).count();
        if count != group.count() {
            return Err(invalid(format!(
                "the probe has {count} {} items, not {}",
                group.name(),
                group.count()
            )));
        }
    }
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    for item in items {
        if !ids.insert(item.id.as_str()) || !item.id.starts_with(item.group.name()) {
            return Err(invalid(format!(
                "probe item id {} is repeated or misnamed",
                item.id
            )));
        }
        let (fewest, most) = item.group.turns();
        if !(fewest..=most).contains(&item.turns.len()) {
            return Err(invalid(format!(
                "probe item {} has {} turns",
                item.id,
                item.turns.len()
            )));
        }
        if item
            .turns
            .iter()
            .any(|t| t.user.trim().is_empty() || t.reply.trim().is_empty())
        {
            return Err(invalid(format!("probe item {} has an empty turn", item.id)));
        }
        let checks = item.checks()?;
        let scored = item.scored()?;
        if !judge_v2(&checks, &scored.user, &scored.reply) {
            return Err(invalid(format!(
                "probe item {}: the reference reply fails its own checks",
                item.id
            )));
        }
        let context = &item.turns[..item.turns.len() - 1];
        if let Some(copy) = &scored.copy {
            if scored.reply != *copy || !scored.user.ends_with(copy.as_str()) {
                return Err(invalid(format!(
                    "probe item {}: a copy item's user turn ends with, and its reply is, the copied text",
                    item.id
                )));
            }
        } else {
            let stated: BTreeSet<&str> = context
                .iter()
                .flat_map(|t| t.values.iter().map(String::as_str))
                .collect();
            let answered = scored
                .answer
                .as_deref()
                .is_some_and(|answer| stated.contains(answer));
            if !answered || !scored.stale.iter().all(|s| stated.contains(s.as_str())) {
                return Err(invalid(format!(
                    "probe item {}: the answer and every stale value are stated earlier",
                    item.id
                )));
            }
        }
        for turn in context {
            let user = words(&turn.user);
            for value in &turn.values {
                if !contains_slice(&user, &words(value)) {
                    return Err(invalid(format!(
                        "probe item {}: {value} is not in the turn that states it",
                        item.id
                    )));
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The corpus exclusion.

/// The `n`-word windows of a text's words (lowercased, without punctuation).
fn word_windows(text: &str, n: usize) -> Vec<Vec<String>> {
    let all = words(text);
    if n == 0 || all.len() < n {
        return Vec::new();
    }
    all.windows(n).map(<[String]>::to_vec).collect()
}

/// Every [`EXCLUSION_NGRAM`]-word window of the probe's user turns.
pub fn probe_ngrams(items: &[ProbeItem]) -> BTreeSet<Vec<String>> {
    let mut grams = BTreeSet::new();
    for item in items {
        for turn in &item.turns {
            grams.extend(word_windows(&turn.user, EXCLUSION_NGRAM));
        }
    }
    grams
}

/// Whether any user turn of `turns` contains one of `grams`.
pub fn shares_ngram(turns: &[Turn2], grams: &BTreeSet<Vec<String>>) -> bool {
    turns.iter().any(|turn| {
        word_windows(&turn.user, EXCLUSION_NGRAM)
            .iter()
            .any(|gram| grams.contains(gram))
    })
}

/// What [`conversation_excluding_probe`] rejected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rejected {
    /// Drawn episodes with a user turn equal to an excluded panel turn.
    pub panel: usize,
    /// Drawn episodes that share an 8-word n-gram with the probe.
    pub probe: usize,
}

impl Rejected {
    pub fn total(&self) -> usize {
        self.panel + self.probe
    }
}

/// The next episode of `split` none of whose user turns equals an `excluded`
/// panel turn (compared by [`normalized`]) and none of which shares an
/// [`EXCLUSION_NGRAM`]-word n-gram with the probe (`grams`, from
/// [`probe_ngrams`]). The rejected draws are counted in `rejected`.
pub fn conversation_excluding_probe(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    split: Split,
    excluded: &BTreeSet<String>,
    grams: &BTreeSet<Vec<String>>,
    rejected: &mut Rejected,
) -> Result<Conversation2> {
    for _ in 0..10_000 {
        let conversation = world.conversation(rng, split)?;
        if conversation
            .turns
            .iter()
            .any(|t| excluded.contains(&normalized(&t.user)))
        {
            rejected.panel += 1;
        } else if shares_ngram(&conversation.turns, grams) {
            rejected.probe += 1;
        } else {
            return Ok(conversation);
        }
    }
    Err(invalid(
        "the exclusion lists reject every M-world v2 conversation",
    ))
}

// ---------------------------------------------------------------------------
// Teacher-forced answer-span scores.

/// The messages of `turns[..=upto]`: every exchange before `upto` complete, then
/// the user turn at `upto` alone.
pub fn history_messages(turns: &[Turn2], upto: usize) -> Vec<Message<'_>> {
    let mut messages = Vec::with_capacity(2 * upto + 1);
    for (i, turn) in turns.iter().enumerate().take(upto + 1) {
        messages.push(Message {
            role: "user",
            content: &turn.user,
        });
        if i < upto {
            messages.push(Message {
                role: "assistant",
                content: &turn.reply,
            });
        }
    }
    messages
}

/// The inputs, targets and weights that teacher-force one turn's reference
/// reply after its reference history and the assistant marker.
#[derive(Clone, Debug, PartialEq)]
pub struct AnswerLayout {
    /// The document without its last token.
    pub inputs: Vec<u32>,
    /// The document without its first token: the target of each input position.
    pub targets: Vec<u32>,
    /// 1 on the positions whose targets are reply tokens, 0 elsewhere.
    pub weights: Vec<f32>,
    /// The positions whose targets are the reply's content tokens and its EOS.
    pub reply: Range<usize>,
    /// The positions whose targets are tokens of the gold value; empty when the
    /// turn states no value or the value's tokens cannot be located.
    pub answer: Vec<usize>,
}

/// The layout of turn `index` of `turns`. The reference reply is encoded as
/// the corpus encodes an assistant turn (its tokens, then EOS), and the
/// prefix through "Assistant: " must be a prefix of the document.
pub fn answer_layout(
    encoder: &DialogueEncoder<'_>,
    tokenizer: &ByteBpeTokenizer,
    turns: &[Turn2],
    index: usize,
) -> Result<AnswerLayout> {
    let turn = turns
        .get(index)
        .ok_or_else(|| invalid("no such turn to score"))?;
    let mut messages = history_messages(turns, index);
    let prefix = encoder.encode_assistant_prefix(&messages);
    messages.push(Message {
        role: "assistant",
        content: &turn.reply,
    });
    let document = encoder.encode_document(&messages);
    let start = prefix.tokens.len();
    let n = document.tokens.len();
    if prefix.emitted_turns != messages.len() - 1
        || document.emitted_turns != messages.len()
        || prefix.special_token_occurrences != 0
        || document.special_token_occurrences != 0
        || n < start + 2
        || document.tokens[..start] != prefix.tokens[..]
    {
        return Err(invalid(format!(
            "turn {index} does not encode as a prefix and a reply"
        )));
    }
    // The reply's content tokens sit between the marker and the closing EOS.
    let normalized_reply = turn.reply.replace("\r\n", "\n").replace('\r', "\n");
    let content = normalized_reply.trim();
    let content_ids = &document.tokens[start..n - 1];
    if tokenizer.encode(content) != content_ids {
        return Err(invalid(format!(
            "turn {index}: the reply's tokens are not its content's encoding"
        )));
    }
    let reply = (start - 1)..(n - 1);
    let mut weights = vec![0.0f32; n - 1];
    for weight in &mut weights[reply.clone()] {
        *weight = 1.0;
    }
    let answer = match &turn.tag.answer {
        Some(value) => answer_positions(tokenizer, content, content_ids, value, start - 1),
        None => Vec::new(),
    };
    Ok(AnswerLayout {
        inputs: document.tokens[..n - 1].to_vec(),
        targets: document.tokens[1..].to_vec(),
        weights,
        reply,
        answer,
    })
}

/// The last occurrence of `needle` in `haystack`, ignoring ASCII case: its byte
/// range.
fn find_last_ci(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    if needle.is_empty() {
        return None;
    }
    let (haystack, needle) = (haystack.to_ascii_lowercase(), needle.to_ascii_lowercase());
    haystack
        .rfind(&needle)
        .map(|start| (start, start + needle.len()))
}

/// The byte range of each token of `ids` in `text`, when the tokens spell
/// exactly `text` (a byte-level BPE does); `None` otherwise.
fn token_byte_spans(
    tokenizer: &ByteBpeTokenizer,
    ids: &[u32],
    text: &str,
) -> Option<Vec<(usize, usize)>> {
    let mut spans = Vec::with_capacity(ids.len());
    let mut at = 0usize;
    for &id in ids {
        let length = tokenizer.decode_bytes(&[id]).len();
        spans.push((at, at + length));
        at += length;
    }
    let joined = tokenizer.decode_bytes(ids);
    let bytes = text.as_bytes();
    if joined == bytes {
        Some(spans)
    } else if joined.len() == bytes.len() + 1 && joined[0] == b' ' && joined[1..] == *bytes {
        // A tokenizer that prepends a space shifts every span one byte left.
        Some(
            spans
                .into_iter()
                .map(|(from, to)| (from.saturating_sub(1), to.saturating_sub(1)))
                .collect(),
        )
    } else {
        None
    }
}

/// The positions whose targets are the tokens that overlap the last occurrence
/// of `value` in the reply `content` (tokens `ids`); `first` is the position of
/// the first content token's target.
fn answer_positions(
    tokenizer: &ByteBpeTokenizer,
    content: &str,
    ids: &[u32],
    value: &str,
    first: usize,
) -> Vec<usize> {
    let Some((from, to)) = find_last_ci(content, value) else {
        return Vec::new();
    };
    let Some(spans) = token_byte_spans(tokenizer, ids, content) else {
        return Vec::new();
    };
    spans
        .iter()
        .enumerate()
        .filter_map(|(j, span)| (span.0 < to && span.1 > from).then_some(first + j))
        .collect()
}

/// What a pointer head did over some scored targets: sums, for the reader to
/// divide by `scored`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointerTotals {
    pub scored: usize,
    /// The most attended source held the target.
    pub hits: usize,
    /// Some attended source held the target: the ceiling of `hits`.
    pub reachable: usize,
    pub gate: f64,
    pub copy_mass: f64,
}

/// The teacher-forced scores of one turn's reference reply.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TeacherForced {
    /// Reply content tokens and the closing EOS.
    pub reply_tokens: usize,
    /// Their summed negative log-likelihood, in nats (the mixture's, for a
    /// pointer model).
    pub reply_nll: f64,
    /// Tokens of the gold value, and their summed NLL.
    pub answer_tokens: usize,
    pub answer_nll: f64,
    /// The pointer head over the value's tokens; all zero for a model without
    /// one.
    pub pointer: PointerTotals,
}

/// Score `layout` under `model`. `None` when the document does not fit the
/// model's context.
pub fn teacher_forced(model: &StackModel, layout: &AnswerLayout) -> Result<Option<TeacherForced>> {
    let time = layout.inputs.len();
    if time == 0 || time > model.config.context {
        return Ok(None);
    }
    let scores = model.score_targets(
        &layout.inputs,
        &layout.targets,
        Some(&layout.weights),
        1,
        time,
    )?;
    if scores.nll.len() != time {
        return Err(invalid("the scorer returned a row per position or none"));
    }
    let mut forced = TeacherForced {
        reply_tokens: layout.reply.len(),
        reply_nll: layout.reply.clone().map(|k| scores.nll[k]).sum::<f64>(),
        answer_tokens: layout.answer.len(),
        answer_nll: layout.answer.iter().map(|&k| scores.nll[k]).sum::<f64>(),
        pointer: PointerTotals::default(),
    };
    if let Some(rows) = &scores.pointer {
        for &k in &layout.answer {
            if let Some(Some(stats)) = rows.get(k) {
                forced.pointer.scored += 1;
                forced.pointer.hits += usize::from(stats.hit);
                forced.pointer.reachable += usize::from(stats.reachable);
                forced.pointer.gate += stats.gate;
                forced.pointer.copy_mass += stats.copy_mass;
            }
        }
    }
    Ok(Some(forced))
}

/// The keys a retrieval turn is counted under: the same names
/// [`crate::milestone_world_v2::Scorecard`] uses for its category, MQAR
/// distance and pool cells.
fn retrieval_keys(turn: &Turn2) -> Vec<String> {
    let mut keys = vec![format!("category/{:?}", turn.category)];
    if let Some(mqar) = &turn.tag.mqar {
        keys.push(format!("mqar/distance/{}", mqar.target_distance));
        keys.push("mqar/all".to_owned());
    }
    if turn.category == Category2::Copy {
        keys.push("copy".to_owned());
    }
    if turn.category == Category2::Relation {
        if let Some(pool) = turn.tag.pool {
            let pool = match pool {
                Pool::Open => "open",
                Pool::Closed => "closed",
            };
            let kind = if turn.tag.abstain {
                "abstain"
            } else {
                "recall"
            };
            keys.push(format!("relation/{pool}/{kind}"));
        }
    }
    keys
}

#[derive(Clone, Debug, Default)]
struct Tally {
    items: usize,
    reply_tokens: usize,
    reply_nll: f64,
    answer_items: usize,
    answer_tokens: usize,
    answer_nll: f64,
    pointer: PointerTotals,
}

/// Teacher-forced scores by category, MQAR distance and pool.
#[derive(Default)]
pub struct NllCard {
    keys: BTreeMap<String, Tally>,
}

impl NllCard {
    /// Count one scored turn.
    pub fn record(&mut self, turn: &Turn2, forced: &TeacherForced) {
        for key in retrieval_keys(turn) {
            let tally = self.keys.entry(key).or_default();
            tally.items += 1;
            tally.reply_tokens += forced.reply_tokens;
            tally.reply_nll += forced.reply_nll;
            if forced.answer_tokens > 0 {
                tally.answer_items += 1;
                tally.answer_tokens += forced.answer_tokens;
                tally.answer_nll += forced.answer_nll;
            }
            let pointer = &mut tally.pointer;
            pointer.scored += forced.pointer.scored;
            pointer.hits += forced.pointer.hits;
            pointer.reachable += forced.pointer.reachable;
            pointer.gate += forced.pointer.gate;
            pointer.copy_mass += forced.pointer.copy_mass;
        }
    }

    /// Mean NLL per token of the whole reply and of the value, and per value.
    pub fn to_json(&self) -> Value {
        let per = |sum: f64, n: usize| -> Value {
            if n == 0 {
                Value::Null
            } else {
                json!(sum / n as f64)
            }
        };
        let rows: BTreeMap<&str, Value> = self
            .keys
            .iter()
            .map(|(key, t)| {
                let mut row = json!({
                    "items": t.items,
                    "reply_tokens": t.reply_tokens,
                    "reply_nll_per_token": per(t.reply_nll, t.reply_tokens),
                    "answer_items": t.answer_items,
                    "answer_tokens": t.answer_tokens,
                    "answer_nll_per_token": per(t.answer_nll, t.answer_tokens),
                    "answer_nll_per_item": per(t.answer_nll, t.answer_items),
                });
                if t.pointer.scored > 0 {
                    let n = t.pointer.scored as f64;
                    row["pointer"] = json!({
                        "answer_tokens_scored": t.pointer.scored,
                        "hit_rate": t.pointer.hits as f64 / n,
                        "reachable_rate": t.pointer.reachable as f64 / n,
                        "mean_gate": t.pointer.gate / n,
                        "mean_copy_mass": t.pointer.copy_mass / n,
                    });
                }
                (key.as_str(), row)
            })
            .collect();
        json!(rows)
    }
}

// ---------------------------------------------------------------------------
// The model-free report of the probe.

/// Every item's token count, whether it fits `context`, its distance from the
/// first turn to the scored query, and the two untrained rules' replies and
/// passes, with the per-group rates. Needs no model.
pub fn static_report(items: &[ProbeItem], meter: &Meter<'_>, context: usize) -> Result<Value> {
    let mut rows = Vec::with_capacity(items.len());
    let mut groups: BTreeMap<ProbeGroup, (usize, usize, BTreeMap<&'static str, usize>)> =
        BTreeMap::new();
    for item in items {
        let conversation = item.conversation(meter)?;
        let Some(last) = conversation.turns.len().checked_sub(1) else {
            continue;
        };
        let scored = &conversation.turns[last];
        let fits = conversation.tokens <= context;
        let group = groups.entry(item.group).or_default();
        group.0 += 1;
        group.1 += usize::from(fits);
        let mut rules = serde_json::Map::new();
        for rule in Rule::ALL {
            let reply = rule.reply(&conversation.turns[..last], scored);
            let pass = judge_v2(&scored.checks, &scored.user, &reply);
            *group.2.entry(rule.name()).or_default() += usize::from(pass);
            rules.insert(
                rule.name().to_owned(),
                json!({"reply": reply, "pass": pass}),
            );
        }
        rows.push(json!({
            "id": item.id,
            "group": item.group,
            "tokens": conversation.tokens,
            "fits_context": fits,
            "distance": (last >= 1).then(|| meter.distance(&conversation.turns)),
            "rules": rules,
        }));
    }
    let by_group: BTreeMap<&str, Value> = groups
        .iter()
        .map(|(group, (count, fit, passes))| {
            let rates: BTreeMap<&str, Value> = passes
                .iter()
                .map(|(rule, pass)| {
                    (
                        *rule,
                        json!({"pass": pass, "of": count, "rate": *pass as f64 / *count as f64}),
                    )
                })
                .collect();
            (
                group.name(),
                json!({"items": count, "fit_context": fit, "rules": rates}),
            )
        })
        .collect();
    Ok(json!({
        "schema": "uor-r4.m-world-probe-static/1",
        "probe_sha256": probe_sha256(),
        "context": context,
        "items": rows,
        "by_group": by_group,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{ReadScore, StackArch, StackConfig};
    use crate::milestone_world::RELATIONS;
    use crate::milestone_world_v2::{in_generated_universe, reserved_words, Cell, Mix};
    use candle_core::Device;
    use uor_r4_tokenizer::dialogue::DialogueProtocol;

    /// The file's SHA-256, computed with `shasum -a 256` after it was written.
    const PROBE_SHA256: &str = "79433e90e4199a9c05391932590708b76c4e2ea70d7efeb7b38ed39712097add";

    /// About one token per three characters.
    fn toy(text: &str) -> usize {
        text.chars().count().div_ceil(3)
    }

    fn world() -> MWorld2<'static> {
        MWorld2::new(&toy, Mix::default()).expect("a valid mix")
    }

    #[test]
    fn the_probe_file_is_pinned() {
        assert_eq!(probe_sha256(), PROBE_SHA256);
    }

    #[test]
    fn the_probe_holds_forty_typed_items() {
        let items = probe().expect("the probe loads and validates");
        assert_eq!(items.len(), PROBE_ITEMS);
        for group in ProbeGroup::ALL {
            assert_eq!(
                items.iter().filter(|i| i.group == group).count(),
                group.count(),
                "{}",
                group.name()
            );
        }
        let meter = Meter::new(&toy);
        for item in &items {
            let conversation = item.conversation(&meter).expect("a conversation");
            let last = conversation.turns.last().expect("a scored turn");
            // The reference reply passes; a hedge, an echo of the question and a
            // reply that names a stale value do not.
            assert!(
                judge_v2(&last.checks, &last.user, &last.reply),
                "{}",
                item.id
            );
            assert!(
                !judge_v2(&last.checks, &last.user, "I'm not sure."),
                "{}",
                item.id
            );
            assert!(
                !judge_v2(&last.checks, &last.user, &last.user),
                "{}",
                item.id
            );
            let scored = item.turns.last().expect("a turn");
            for stale in &scored.stale {
                let both = format!(
                    "{} and {stale}",
                    scored.answer.as_deref().unwrap_or_default()
                );
                assert!(!judge_v2(&last.checks, &last.user, &both), "{}", item.id);
                let only = format!("It is {stale}.");
                assert!(!judge_v2(&last.checks, &last.user, &only), "{}", item.id);
            }
            if item.group == ProbeGroup::Copy {
                assert_eq!(last.category, Category2::Copy);
                let copy = scored.copy.as_deref().expect("copy text");
                let shouted = copy.to_uppercase();
                assert!(judge_v2(&last.checks, &last.user, &shouted));
                let extra = format!("Sure: {copy}");
                assert!(!judge_v2(&last.checks, &last.user, &extra));
            } else {
                assert_eq!(last.tag.pool, Some(Pool::Open));
                assert!(last.tag.answer.is_some());
            }
            assert_eq!(conversation.tokens, meter.document(&conversation.turns));
        }
    }

    #[test]
    fn the_probe_values_are_outside_every_pool() {
        let items = probe().expect("the probe");
        let reserved = reserved_words();
        let mut closed: BTreeSet<String> = BTreeSet::new();
        for relation in RELATIONS {
            closed.extend(
                relation
                    .train_values
                    .iter()
                    .chain(relation.development_values)
                    .map(|v| v.to_lowercase()),
            );
        }
        let mut checked = BTreeSet::new();
        for item in &items {
            for turn in &item.turns {
                let stated = turn
                    .values
                    .iter()
                    .chain(turn.answer.iter())
                    .chain(turn.stale.iter());
                for value in stated {
                    checked.insert(value.clone());
                    let lower = value.to_lowercase();
                    assert!(
                        !in_generated_universe(value),
                        "{} in {}: a syllable, digit or town value of the world",
                        value,
                        item.id
                    );
                    assert!(!reserved.contains(&lower), "{value} in {}", item.id);
                    assert!(!closed.contains(&lower), "{value} in {}", item.id);
                }
                // No number of two to four digits (the number pool) anywhere.
                for text in [&turn.user, &turn.reply] {
                    for word in words(text) {
                        let pool_number = (2..=4).contains(&word.len())
                            && word.bytes().all(|b| b.is_ascii_digit());
                        assert!(!pool_number, "{word} in {}", item.id);
                    }
                }
            }
        }
        assert!(checked.len() > 60, "{}", checked.len());
    }

    /// The maximal literal fragments between the slots of a template.
    fn fragments(template: &str) -> Vec<Vec<String>> {
        template
            .split(['{', '}'])
            .map(words)
            .filter(|fragment| fragment.len() >= 3)
            .collect()
    }

    #[test]
    fn no_probe_phrasing_matches_a_world_template() {
        let items = probe().expect("the probe");
        let mut templates: Vec<Vec<Vec<String>>> = Vec::new();
        for split in [Split::Train, Split::Development] {
            templates.extend(MWorld2::templates(split).into_iter().map(fragments));
        }
        assert!(templates.len() > 100);
        for item in &items {
            for turn in &item.turns {
                let user = words(&turn.user);
                for template in &templates {
                    for fragment in template {
                        assert!(
                            !contains_slice(&user, fragment),
                            "{} contains the template fragment {fragment:?}: {}",
                            item.id,
                            turn.user
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn no_world_conversation_shares_an_8_gram_with_the_probe() {
        let items = probe().expect("the probe");
        let grams = probe_ngrams(&items);
        assert!(grams.len() > 300, "{}", grams.len());
        for cell in Cell::ALL {
            let mut world = world();
            let mut rng = Rng::new(11);
            for _ in 0..500 {
                let conversation = world.conversation_in(&mut rng, cell).expect("an episode");
                assert!(
                    !shares_ngram(&conversation.turns, &grams),
                    "{cell:?}: {:?}",
                    conversation.turns
                );
            }
        }
    }

    #[test]
    fn the_corpus_exclusion_drops_a_conversation_that_shares_an_8_gram() {
        // Find a world conversation with a user turn of at least 8 words and
        // exclude one 8-gram of it, as a probe that quoted it would.
        let mut scan = world();
        let mut rng = Rng::new(5);
        let (mut index, mut gram) = (0usize, None);
        while gram.is_none() {
            let conversation = scan
                .conversation(&mut rng, Split::Train)
                .expect("an episode");
            gram = conversation
                .turns
                .iter()
                .find_map(|t| word_windows(&t.user, EXCLUSION_NGRAM).into_iter().next());
            if gram.is_none() {
                index += 1;
            }
            assert!(index < 200, "no conversation with an 8-word user turn");
        }
        let excluded_gram = gram.expect("an 8-gram");
        let grams: BTreeSet<Vec<String>> = [excluded_gram].into_iter().collect();
        let panel = BTreeSet::new();
        // The same seed again: the draws before that conversation are kept,
        // that one is rejected, and nothing returned contains the n-gram.
        let mut world = world();
        let mut rng = Rng::new(5);
        let mut rejected = Rejected::default();
        for _ in 0..=index {
            let kept = conversation_excluding_probe(
                &mut world,
                &mut rng,
                Split::Train,
                &panel,
                &grams,
                &mut rejected,
            )
            .expect("an episode");
            assert!(!shares_ngram(&kept.turns, &grams));
        }
        assert!(rejected.probe >= 1, "{rejected:?}");
        assert_eq!(rejected.panel, 0);
        // With nothing to exclude, the first draw is kept.
        let mut world = self::world();
        let mut rng = Rng::new(5);
        let mut none = Rejected::default();
        let empty = BTreeSet::new();
        conversation_excluding_probe(
            &mut world,
            &mut rng,
            Split::Train,
            &panel,
            &empty,
            &mut none,
        )
        .expect("an episode");
        assert_eq!(none.total(), 0);
        // A panel turn is still excluded by equality.
        let mut world = self::world();
        let mut rng = Rng::new(5);
        let first = self::world()
            .conversation(&mut Rng::new(5), Split::Train)
            .expect("an episode");
        let turns: BTreeSet<String> = first.turns.iter().map(|t| normalized(&t.user)).collect();
        let mut both = Rejected::default();
        let kept = conversation_excluding_probe(
            &mut world,
            &mut rng,
            Split::Train,
            &turns,
            &empty,
            &mut both,
        )
        .expect("an episode");
        assert!(both.panel >= 1 && both.probe == 0, "{both:?}");
        assert!(kept
            .turns
            .iter()
            .all(|t| !turns.contains(&normalized(&t.user))));
    }

    #[test]
    fn the_rules_on_the_probe_stay_below_half() {
        let items = probe().expect("the probe");
        let meter = Meter::new(&toy);
        let report = static_report(&items, &meter, 256).expect("a report");
        assert_eq!(report["probe_sha256"], json!(PROBE_SHA256));
        for (group, cell) in report["by_group"].as_object().expect("groups") {
            for (rule, row) in cell["rules"].as_object().expect("rules") {
                let rate = row["rate"].as_f64().expect("a rate");
                assert!(rate < 0.5, "{rule} on {group}: {rate}");
            }
        }
        assert_eq!(report["items"].as_array().map(Vec::len), Some(PROBE_ITEMS));
        // No rule copies: the copy group has no history to read.
        assert_eq!(
            report["by_group"]["copy"]["rules"]["R-nlet"]["pass"],
            json!(0)
        );
    }

    // -- Teacher forcing ------------------------------------------------------

    /// GPT-2's byte-to-character alphabet.
    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut table = vec!['\0'; 256];
        let mut extra = 0;
        for byte in 0u32..256 {
            table[byte as usize] = if printable.contains(&byte) {
                char::from_u32(byte).expect("a scalar value")
            } else {
                extra += 1;
                char::from_u32(255 + extra).expect("a scalar value")
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
        .expect("a byte-level tokenizer")
    }

    const VOCAB: usize = 288;

    fn stack() -> StackModel {
        let mut config = StackConfig::transformer_control(5);
        config.arch = StackArch::Geometric;
        config.vocab_size = VOCAB;
        config.width = 32;
        config.heads = 2;
        config.mlp_hidden = 64;
        config.pattern = "ra".to_owned();
        config.read = ReadScore::Lorentz;
        StackModel::new(config, &Device::Cpu).expect("a stack")
    }

    fn hand(user: &str, reply: &str, answer: Option<&str>) -> Turn2 {
        Turn2 {
            intent: "hand_built".into(),
            category: Category2::Mqar,
            user: user.into(),
            reply: reply.into(),
            checks: Vec::new(),
            tag: Tag {
                answer: answer.map(str::to_owned),
                ..Tag::default()
            },
        }
    }

    #[test]
    fn the_answer_layout_teacher_forces_the_reply_and_finds_the_value() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).expect("a protocol");
        let encoder = protocol.bind(&tokenizer).expect("an encoder");
        let turns = [
            hand("bol is 4721, tam is kavu.", "Noted.", None),
            hand("What is bol?", "Bol is 4721.", Some("4721")),
        ];
        let layout = answer_layout(&encoder, &tokenizer, &turns, 1).expect("a layout");
        let time = layout.inputs.len();
        assert_eq!(layout.targets.len(), time);
        assert_eq!(layout.weights.len(), time);
        // The targets are the inputs shifted by one, then the closing EOS.
        assert_eq!(&layout.inputs[1..], &layout.targets[..time - 1]);
        assert_eq!(layout.targets[time - 1], protocol.eos_id);
        // The weighted positions are exactly the reply's content and its EOS.
        let weighted: Vec<usize> = (0..time).filter(|&k| layout.weights[k] == 1.0).collect();
        assert_eq!(weighted, layout.reply.clone().collect::<Vec<usize>>());
        let reply_ids: Vec<u32> = layout.reply.clone().map(|k| layout.targets[k]).collect();
        assert_eq!(reply_ids.last(), Some(&protocol.eos_id));
        assert_eq!(
            tokenizer.decode(&reply_ids[..reply_ids.len() - 1]),
            "Bol is 4721."
        );
        // The value's positions decode to the value alone.
        let value_ids: Vec<u32> = layout.answer.iter().map(|&k| layout.targets[k]).collect();
        assert_eq!(tokenizer.decode(&value_ids), "4721");
        assert!(layout.answer.iter().all(|k| layout.reply.contains(k)));
        // A turn with no value scores its reply only.
        let plain = answer_layout(&encoder, &tokenizer, &turns, 0).expect("a layout");
        assert!(plain.answer.is_empty());
        assert!(answer_layout(&encoder, &tokenizer, &turns, 2).is_err());
    }

    #[test]
    fn the_spans_and_the_search_are_exact() {
        let tokenizer = tokenizer();
        let text = "Amara covers Wednesday.";
        let ids = tokenizer.encode(text);
        let spans = token_byte_spans(&tokenizer, &ids, text).expect("spans");
        assert_eq!(spans.len(), text.len());
        assert_eq!(spans[0], (0, 1));
        assert_eq!(spans[text.len() - 1], (text.len() - 1, text.len()));
        assert!(token_byte_spans(&tokenizer, &ids, "Amara covers Monday.").is_none());
        // The last occurrence, ignoring case.
        assert_eq!(find_last_ci("Amara is amara", "AMARA"), Some((9, 14)));
        assert_eq!(find_last_ci("Amara", "Zed"), None);
        assert_eq!(find_last_ci("Amara", ""), None);
        let positions = answer_positions(&tokenizer, text, &ids, "wednesday", 10);
        assert_eq!(positions, (10 + 13..10 + 22).collect::<Vec<usize>>());
    }

    #[test]
    fn teacher_forced_scores_are_the_models_own_target_nll() {
        let tokenizer = tokenizer();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).expect("a protocol");
        let encoder = protocol.bind(&tokenizer).expect("an encoder");
        let turns = [
            hand("bol is 4721.", "Noted.", None),
            hand("What is bol?", "Bol is 4721.", Some("4721")),
        ];
        let layout = answer_layout(&encoder, &tokenizer, &turns, 1).expect("a layout");
        let model = stack();
        let forced = teacher_forced(&model, &layout)
            .expect("scores")
            .expect("the document fits the context");
        let time = layout.inputs.len();
        let all = model
            .target_nll(&layout.inputs, &layout.targets, 1, time)
            .expect("target nll");
        let reply: f64 = layout.reply.clone().map(|k| all[k]).sum();
        let answer: f64 = layout.answer.iter().map(|&k| all[k]).sum();
        assert!((forced.reply_nll - reply).abs() < 1e-9);
        assert!((forced.answer_nll - answer).abs() < 1e-9);
        assert_eq!(forced.reply_tokens, layout.reply.len());
        assert_eq!(forced.answer_tokens, layout.answer.len());
        assert_eq!(forced.answer_tokens, 4);
        assert_eq!(forced.pointer, PointerTotals::default());
        assert!(forced.reply_nll > forced.answer_nll && forced.answer_nll > 0.0);
        // The card averages what was recorded, per token and per value.
        let mut card = NllCard::default();
        let mut turn = turns[1].clone();
        turn.category = Category2::Mqar;
        card.record(&turn, &forced);
        let report = card.to_json();
        let per_token = report["category/Mqar"]["answer_nll_per_token"]
            .as_f64()
            .expect("a number");
        assert!((per_token - forced.answer_nll / 4.0).abs() < 1e-9);
        assert_eq!(report["category/Mqar"]["answer_items"], json!(1));
    }
}
