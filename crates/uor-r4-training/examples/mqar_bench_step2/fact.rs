//! `layout=fact`: the Step 2 deployment-parity MQAR bench (References #820;
//! `docs/research/barrier-assessment-2026-10-05/final-assessment.md` Step 2
//! and `completeness-critique.md` points 2, 5 and 6).
//!
//! Each window of at most 384 tokens (the trained and served context; the
//! served engine refuses positions beyond it) holds facts and their queries
//! written in pieces of the real #1017 tokenizer:
//!
//! - **Fact:** `key ++ gap ++ value`. A key is 1-3 BPE pieces: a lowercase
//!   space-prefixed word piece followed by 0-2 lowercase continuation pieces,
//!   kept only when the tokenizer encodes its own decoded text back to exactly
//!   those pieces (a canonical tuple). The gap (copula) is `g` pieces,
//!   `g` in `gaps` (default 0..=3), drawn from a pool of function-word pieces
//!   (" is", " =", ":", ...) plus random word pieces of the key/value pools,
//!   so gap and key/value vocabularies overlap. A value is one word piece, a
//!   canonical two-piece word, or a numeral `" d"` encoded as the tokenizer
//!   writes it (a bare space piece and a digit piece).
//! - **Query**, after its fact at a distance drawn inside a bucket:
//!   - `rehearse`: `key ++ gap ++ value`, the fact's own pieces, so the
//!     position predicting the first value piece holds the last copula piece
//!     (as in D19 "{k} is"), or the last key piece when `g = 0`;
//!   - `bare`: `key ++ BARE_PREFIX ++ value`, the "It's {v}" reply form, which
//!     never repeats the copula.
//!
//!   The targets are the value pieces (teacher forced after the first).
//!
//! Inside one window all key pieces are distinct from every other key and
//! value piece and from every gap piece, value tuples are distinct, and the
//! filler excludes every key and value piece: the query identifies its fact
//! uniquely. Pairings are fresh in every window. The held-out evaluation
//! draws keys whose first piece id is `0 mod 4`; training never draws one.
//!
//! Scores per cell `(form, g, key pieces)`: the first piece, the first
//! content piece (after a whitespace-only piece) and the full value (every
//! piece, teacher forced). Every item is also scored by the non-learned
//! rehearse + R-nlet reference rule ([`reference_rule`]) so that arms are
//! compared with it.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

use super::*;

/// The trained and served context of the D19 rungs.
pub(super) const SERVED_CONTEXT: usize = 384;
pub(super) const FACT_SCHEMA: &str = "uor-r4/mqar-bench/fact-record/v1";
/// The pools are task identity: one fixed seed, independent of `seed`.
const POOL_SEED: u64 = 0x5354_4550_3250;
/// Canonical multi-piece tuples kept per length.
const POOL_TARGET: usize = 4000;
const POOL_ATTEMPTS: usize = 400_000;
const GAP_WORDS: [&str; 16] = [
    " is", " was", " are", " =", ":", " has", " of", " the", " to", " in", " and", " a", ",",
    " called", " means", " -",
];
/// Random key/value word pieces added to the gap pool (the overlap).
const GAP_OVERLAP: usize = 16;
const BARE_PREFIX: &str = "? It's";
/// The longest n-let of the reference rule's backoff.
pub(super) const RULE_MAX_N: usize = 4;
const FACT_TRAIN_DOMAIN: u64 = 0x6661_6374_74;
const FACT_CURVE_DOMAIN: u64 = 0x6661_6374_63;
const FACT_CURVE_IN_CLASS_DOMAIN: u64 = 0x6661_6374_69;
const FACT_HELD_OUT_DOMAIN: u64 = 0x6661_6374_68;
const FACT_IN_CLASS_DOMAIN: u64 = 0x6661_6374_6B;
const FACT_PROBE_DOMAIN: u64 = 0x6661_6374_70;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Form {
    Rehearse,
    Bare,
}

impl Form {
    pub(super) fn name(self) -> &'static str {
        match self {
            Form::Rehearse => "rehearse",
            Form::Bare => "bare",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueKind {
    Word1,
    Word2,
    Numeral,
}

impl ValueKind {
    fn name(self) -> &'static str {
        match self {
            ValueKind::Word1 => "word1",
            ValueKind::Word2 => "word2",
            ValueKind::Numeral => "numeral",
        }
    }
}

/// The cell name of `(form, g, key pieces)`.
pub(super) fn cell_name(form: Form, gap: usize, key_len: usize) -> String {
    format!("{}.g{gap}.k{key_len}", form.name())
}

/// Pieces of the #1017 tokenizer the facts are written in.
pub(super) struct FactVocab {
    pub(super) vocab: usize,
    tokenizer_sha256: String,
    tokenizer_address: String,
    /// Decoded text of every id.
    texts: Vec<String>,
    /// Canonical key tuples by length - 1, split by class (`[train, held]`).
    keys: [[Vec<Vec<u32>>; 2]; 3],
    word_values: [Vec<Vec<u32>>; 2],
    numerals: Vec<Vec<u32>>,
    gap_pool: Vec<u32>,
    bare_prefix: Vec<u32>,
    filler: Vec<u32>,
    pools_sha256: String,
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn lower_alpha(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_lowercase())
}

impl FactVocab {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
            .ok_or_else(|| invalid("tokenizer= is not a byte-level BPE tokenizer.json"))?;
        Self::from_tokenizer(&tokenizer, hex_sha256(&bytes))
    }

    fn from_tokenizer(tokenizer: &ByteBpeTokenizer, sha256: String) -> Result<Self> {
        let vocab = tokenizer.vocab_size();
        let texts: Vec<String> = (0..vocab as u32)
            .map(|id| tokenizer.decode(&[id]))
            .collect();
        let special = |text: &str| text.starts_with("<|") && text.ends_with("|>");
        let canonical = |ids: &[u32]| tokenizer.encode(&tokenizer.decode(ids)) == ids;
        let starts: Vec<u32> = (0..vocab as u32)
            .filter(|&id| {
                let text = &texts[id as usize];
                text.len() >= 3
                    && text.starts_with(' ')
                    && lower_alpha(&text[1..])
                    && canonical(&[id])
            })
            .collect();
        let continuations: Vec<u32> = (0..vocab as u32)
            .filter(|&id| lower_alpha(&texts[id as usize]))
            .collect();
        if starts.len() < 64 || continuations.len() < 64 {
            return Err(invalid("the tokenizer has too few lowercase word pieces"));
        }
        let mut rng = Rng::new(POOL_SEED, 1, 0);
        let mut tuples = |length: usize| -> Result<Vec<Vec<u32>>> {
            let mut seen = BTreeSet::new();
            let mut kept = Vec::new();
            for _ in 0..POOL_ATTEMPTS {
                if kept.len() == POOL_TARGET {
                    break;
                }
                let mut ids = vec![starts[(rng.next() % starts.len() as u64) as usize]];
                while ids.len() < length {
                    ids.push(continuations[(rng.next() % continuations.len() as u64) as usize]);
                }
                if canonical(&ids) && seen.insert(ids.clone()) {
                    kept.push(ids);
                }
            }
            if kept.len() < 256 {
                return Err(invalid(format!(
                    "only {} canonical {length}-piece keys",
                    kept.len()
                )));
            }
            Ok(kept)
        };
        let singles: Vec<Vec<u32>> = starts.iter().map(|&id| vec![id]).collect();
        let (pairs, triples) = (tuples(2)?, tuples(3)?);
        let split = |pool: &[Vec<u32>]| -> [Vec<Vec<u32>>; 2] {
            let (held, train): (Vec<_>, Vec<_>) =
                pool.iter().cloned().partition(|key| key[0] % 4 == 0);
            [train, held]
        };
        let keys = [split(&singles), split(&pairs), split(&triples)];
        let numerals: Vec<Vec<u32>> = (0..10)
            .map(|digit| tokenizer.encode(&format!(" {digit}")))
            .filter(|ids| (1..=2).contains(&ids.len()))
            .collect();
        if numerals.len() < 5 {
            return Err(invalid(
                "the tokenizer writes too few one- or two-piece numerals",
            ));
        }
        let mut gap_pool: Vec<u32> = GAP_WORDS
            .iter()
            .map(|word| tokenizer.encode(word))
            .filter(|ids| ids.len() == 1)
            .map(|ids| ids[0])
            .collect();
        let mut overlap_rng = Rng::new(POOL_SEED, 2, 0);
        let mut added = 0;
        while added < GAP_OVERLAP {
            let id = starts[(overlap_rng.next() % starts.len() as u64) as usize];
            if !gap_pool.contains(&id) {
                gap_pool.push(id);
                added += 1;
            }
        }
        let bare_prefix = tokenizer.encode(BARE_PREFIX);
        let filler: Vec<u32> = (0..vocab as u32)
            .filter(|&id| !texts[id as usize].is_empty() && !special(&texts[id as usize]))
            .collect();
        let pools = json!({
            "keys": keys, "word_values": [&singles, &pairs], "numerals": numerals,
            "gap_pool": gap_pool, "bare_prefix": bare_prefix, "filler": filler,
        });
        Ok(Self {
            vocab,
            tokenizer_sha256: sha256,
            tokenizer_address: tokenizer.address(),
            pools_sha256: hex_sha256(&serde_json::to_vec(&pools)?),
            texts,
            keys,
            word_values: [singles, pairs],
            numerals,
            gap_pool,
            bare_prefix,
            filler,
        })
    }

    fn decoded(&self, ids: &[u32]) -> Vec<&str> {
        ids.iter()
            .map(|&id| self.texts[id as usize].as_str())
            .collect()
    }

    fn record(&self) -> Value {
        json!({
            "vocab": self.vocab,
            "tokenizer_sha256": self.tokenizer_sha256,
            "tokenizer_address": self.tokenizer_address,
            "pools_sha256": self.pools_sha256,
            "pool_seed": POOL_SEED,
            "key_pool_sizes_train_held": self.keys.iter()
                .map(|split| [split[0].len(), split[1].len()]).collect::<Vec<_>>(),
            "word_value_pool_sizes": [self.word_values[0].len(), self.word_values[1].len()],
            "numerals": self.numerals.iter()
                .map(|ids| json!({"ids": ids, "pieces": self.decoded(ids)})).collect::<Vec<_>>(),
            "gap_pool": self.gap_pool.iter()
                .map(|&id| json!({"id": id, "piece": self.texts[id as usize]})).collect::<Vec<_>>(),
            "bare_prefix": {"text": BARE_PREFIX, "ids": self.bare_prefix,
                "pieces": self.decoded(&self.bare_prefix)},
            "filler_pool_size": self.filler.len(),
        })
    }

    /// The index of the value's first piece that is not whitespace only.
    fn first_content(&self, value: &[u32]) -> usize {
        value
            .iter()
            .position(|&id| !self.texts[id as usize].trim().is_empty())
            .unwrap_or(0)
    }
}

/// The fact task: the vocabulary and the cells it draws.
pub(super) struct FactTask {
    pub(super) vocab: FactVocab,
    tokenizer_path: PathBuf,
    gaps: Vec<usize>,
    forms: Vec<Form>,
}

impl FactTask {
    pub(super) fn parse(args: &mut Args, common: &Common) -> Result<Self> {
        let tokenizer_path = PathBuf::from(
            args.take("tokenizer")
                .ok_or_else(|| invalid("layout=fact needs tokenizer=TOKENIZER_JSON"))?,
        );
        let gaps = args
            .take("gaps")
            .unwrap_or_else(|| "0,1,2,3".into())
            .split(',')
            .map(|part| {
                part.parse::<usize>()
                    .ok()
                    .filter(|&g| g <= 3)
                    .ok_or_else(|| invalid(format!("invalid gap {part} (0..=3)")))
            })
            .collect::<Result<BTreeSet<_>>>()?
            .into_iter()
            .collect();
        let forms = args
            .take("forms")
            .unwrap_or_else(|| "rehearse,bare".into())
            .split(',')
            .map(|part| match part {
                "rehearse" => Ok(Form::Rehearse),
                "bare" => Ok(Form::Bare),
                other => Err(invalid(format!("invalid form {other}"))),
            })
            .collect::<Result<BTreeSet<_>>>()?
            .into_iter()
            .collect();
        if common.context > SERVED_CONTEXT {
            return Err(invalid(format!(
                "layout=fact needs context <= {SERVED_CONTEXT} (the served context)"
            )));
        }
        Ok(Self {
            vocab: FactVocab::load(&tokenizer_path)?,
            tokenizer_path,
            gaps,
            forms,
        })
    }

    fn record(&self) -> Value {
        json!({
            "layout": "fact",
            "tokenizer_path": self.tokenizer_path,
            "vocabulary": self.vocab.record(),
            "gaps": self.gaps,
            "forms": self.forms.iter().map(|f| f.name()).collect::<Vec<_>>(),
            "key_pieces": [1, 2, 3],
            "value_kinds": ["word1", "word2", "numeral"],
            "held_out_split": "keys whose first piece id is 0 mod 4; training never draws one",
            "targets": "every value piece of every query (teacher forced after the first)",
            "reference_rule": format!(
                "rehearse + R-nlet backoff: the longest n <= {RULE_MAX_N} whose last-n pieces before the value recur earlier; copy the pieces after that occurrence (bare: the fixed bare prefix is stripped first)"
            ),
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct FactItem {
    form: Form,
    gap: usize,
    key_len: usize,
    kind: ValueKind,
    bucket: usize,
    distance: usize,
    key: Vec<u32>,
    gap_tokens: Vec<u32>,
    value: Vec<u32>,
    fact_start: usize,
    query_start: usize,
    /// Positions whose next-token targets are `value[i]`.
    predict: Vec<usize>,
}

impl FactItem {
    #[cfg(test)]
    fn fact_len(&self) -> usize {
        self.key.len() + self.gap + self.value.len()
    }

    fn key_last(&self) -> usize {
        self.fact_start + self.key.len() - 1
    }

    fn value_first(&self) -> usize {
        self.fact_start + self.key.len() + self.gap
    }
}

#[derive(Clone, Debug)]
pub(super) struct FactSequence {
    tokens: Vec<u32>,
    items: Vec<FactItem>,
}

fn pick<'a, T>(rng: &mut Rng, pool: &'a [T]) -> &'a T {
    &pool[(rng.next() % pool.len() as u64) as usize]
}

fn generate_fact(
    rng: &mut Rng,
    task: &FactTask,
    context: usize,
    buckets: &[Bucket],
    pairs_per_bucket: usize,
    pairing: Pairing,
) -> Result<FactSequence> {
    let vocab = &task.vocab;
    let class = usize::from(pairing == Pairing::HeldOut);
    let mut key_pieces = BTreeSet::new();
    let mut used_pieces = BTreeSet::new();
    let mut values_seen = BTreeSet::new();
    let mut order: Vec<usize> = (0..buckets.len()).collect();
    order.sort_by_key(|&b| std::cmp::Reverse(buckets[b].high));
    let mut items = Vec::new();
    // 1. Keys and values, all pieces of every key distinct from the rest.
    for &b in &order {
        for _ in 0..pairs_per_bucket {
            let form = *pick(rng, &task.forms);
            let gap = *pick(rng, &task.gaps);
            let key_len = rng.range(1, 3);
            let pool = &vocab.keys[key_len - 1][class];
            let key = (0..PLACEMENT_ATTEMPTS)
                .map(|_| pick(rng, pool))
                .find(|key| key.iter().all(|id| !used_pieces.contains(id)))
                .cloned()
                .ok_or_else(|| invalid("no key with fresh pieces"))?;
            key_pieces.extend(key.iter().copied());
            used_pieces.extend(key.iter().copied());
            let kinds = [ValueKind::Word1, ValueKind::Word2, ValueKind::Numeral];
            let first = *pick(rng, &kinds);
            // The drawn kind first; if its pool has no fresh value left in this
            // sequence (the numeral pool holds only ten values), fall back to
            // the other kinds in a fixed order rather than failing the draw.
            let order = std::iter::once(first).chain(kinds.into_iter().filter(|k| *k != first));
            let fresh = |value: &&Vec<u32>| {
                !values_seen.contains(*value) && value.iter().all(|id| !key_pieces.contains(id))
            };
            let (kind, value) = order
                .filter_map(|kind| {
                    let pool = match kind {
                        ValueKind::Word1 => &vocab.word_values[0],
                        ValueKind::Word2 => &vocab.word_values[1],
                        ValueKind::Numeral => &vocab.numerals,
                    };
                    (0..PLACEMENT_ATTEMPTS)
                        .map(|_| pick(rng, pool))
                        .find(fresh)
                        .or_else(|| pool.iter().find(fresh))
                        .map(|value| (kind, value.clone()))
                })
                .next()
                .ok_or_else(|| invalid("no fresh value"))?;
            values_seen.insert(value.clone());
            used_pieces.extend(value.iter().copied());
            items.push(FactItem {
                form,
                gap,
                key_len,
                kind,
                bucket: b,
                distance: 0,
                key,
                gap_tokens: Vec::new(),
                value,
                fact_start: 0,
                query_start: 0,
                predict: Vec::new(),
            });
        }
    }
    // 2. Gaps, after every key: no gap piece is a key piece of the window.
    for item in &mut items {
        for _ in 0..item.gap {
            let piece = (0..PLACEMENT_ATTEMPTS)
                .map(|_| *pick(rng, &vocab.gap_pool))
                .find(|id| !key_pieces.contains(id))
                .ok_or_else(|| invalid("no gap piece outside the keys"))?;
            item.gap_tokens.push(piece);
        }
    }
    // 3. Placement: fact span, then its query span `distance` later.
    let mut tokens: Vec<u32> = Vec::with_capacity(context);
    while tokens.len() < context {
        let id = *pick(rng, &vocab.filler);
        if !used_pieces.contains(&id) {
            tokens.push(id);
        }
    }
    let mut used = vec![false; context];
    for item in &mut items {
        let middle: Vec<u32> = match item.form {
            Form::Rehearse => item.gap_tokens.clone(),
            Form::Bare => vocab.bare_prefix.clone(),
        };
        let query: Vec<u32> = [item.key.as_slice(), &middle, &item.value].concat();
        let fact: Vec<u32> = [item.key.as_slice(), &item.gap_tokens, &item.value].concat();
        let bucket = buckets[item.bucket];
        let mut placed = None;
        for _ in 0..PLACEMENT_ATTEMPTS {
            let distance = rng.range(bucket.low, bucket.high);
            if distance < fact.len() || distance + query.len() > context {
                continue;
            }
            let start = rng.range(0, context - distance - query.len());
            let free = |from: usize, len: usize| (from..from + len).all(|p| !used[p]);
            if free(start, fact.len()) && free(start + distance, query.len()) {
                placed = Some((start, distance));
                break;
            }
        }
        let (start, distance) =
            placed.ok_or_else(|| invalid("could not place a fact in the window"))?;
        let query_start = start + distance;
        for (offset, &id) in fact.iter().enumerate() {
            tokens[start + offset] = id;
            used[start + offset] = true;
        }
        for (offset, &id) in query.iter().enumerate() {
            tokens[query_start + offset] = id;
            used[query_start + offset] = true;
        }
        let value_start = query_start + item.key.len() + middle.len();
        item.distance = distance;
        item.fact_start = start;
        item.query_start = query_start;
        item.predict = (0..item.value.len()).map(|i| value_start - 1 + i).collect();
    }
    items.sort_by_key(|item| item.query_start);
    Ok(FactSequence { tokens, items })
}

fn fact_arrays(sequences: &[FactSequence], context: usize) -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let total = sequences.len() * context;
    let (mut ids, mut targets, mut weights) = (
        Vec::with_capacity(total),
        vec![0u32; total],
        vec![0f32; total],
    );
    for (s, sequence) in sequences.iter().enumerate() {
        ids.extend_from_slice(&sequence.tokens);
        for item in &sequence.items {
            for (&position, &piece) in item.predict.iter().zip(&item.value) {
                targets[s * context + position] = piece;
                weights[s * context + position] = 1.0;
            }
        }
    }
    (ids, targets, weights)
}

/// The R-nlet rule: the latest occurrence, ending before `predict`, of the
/// `n` pieces ending at `predict`; returns the `len` pieces after it.
pub(super) fn nlet_rule(tokens: &[u32], predict: usize, n: usize, len: usize) -> Option<Vec<u32>> {
    if n == 0 || n > predict + 1 {
        return None;
    }
    let pattern = &tokens[predict + 1 - n..=predict];
    (n - 1..predict)
        .rev()
        .find(|&end| &tokens[end + 1 - n..=end] == pattern)
        .map(|end| tokens[end + 1..(end + 1 + len).min(tokens.len())].to_vec())
}

/// Backoff over `n = RULE_MAX_N..=1`: the longest n-let that recurs.
fn nlet_backoff(tokens: &[u32], predict: usize, len: usize) -> Option<Vec<u32>> {
    (1..=RULE_MAX_N)
        .rev()
        .find_map(|n| nlet_rule(tokens, predict, n, len))
}

/// The non-learned rehearse + R-nlet reference: on a rehearsed query the
/// n-let backoff at the position before the value; on a bare query the fixed
/// bare prefix (known to the rule) is stripped first, so the n-let ends at
/// the key and the rule copies what followed the key in the fact (the value
/// only when `g = 0`). It is told the value's piece count.
pub(super) fn reference_rule(tokens: &[u32], item: &FactItem, bare_len: usize) -> bool {
    let predict = item.predict[0];
    let end = match item.form {
        Form::Rehearse => predict,
        Form::Bare => predict - bare_len,
    };
    nlet_backoff(tokens, end, item.value.len()).as_deref() == Some(item.value.as_slice())
}

#[derive(Clone, Default)]
struct FactTally {
    total: usize,
    first: usize,
    first_content: usize,
    full: usize,
    nll_first_content: f64,
    rule: usize,
    rule_n: [usize; RULE_MAX_N],
}

impl FactTally {
    fn record(&self) -> Value {
        let rate = |n: usize| n as f64 / self.total.max(1) as f64;
        json!({
            "total": self.total,
            "first_piece_accuracy": rate(self.first),
            "first_content_piece_accuracy": rate(self.first_content),
            "first_content_correct": self.first_content,
            "full_accuracy": rate(self.full),
            "full_correct": self.full,
            "mean_nll_first_content_piece": self.nll_first_content / self.total.max(1) as f64,
            "rule_full_accuracy": rate(self.rule),
            "rule_n_full_accuracy": self.rule_n.iter().map(|&n| rate(n)).collect::<Vec<_>>(),
        })
    }
}

/// Per-item outcome before tallying.
struct Outcome {
    first: bool,
    first_content: bool,
    full: bool,
    nll_first_content: f64,
    rule: bool,
    rule_n: [bool; RULE_MAX_N],
}

fn tally(map: &mut BTreeMap<String, FactTally>, key: String, outcome: &Outcome) {
    let entry = map.entry(key).or_default();
    entry.total += 1;
    entry.first += usize::from(outcome.first);
    entry.first_content += usize::from(outcome.first_content);
    entry.full += usize::from(outcome.full);
    entry.nll_first_content += outcome.nll_first_content;
    entry.rule += usize::from(outcome.rule);
    for (n, hit) in outcome.rule_n.iter().enumerate() {
        entry.rule_n[n] += usize::from(*hit);
    }
}

fn argmax(scores: &[f32]) -> usize {
    scores
        .iter()
        .enumerate()
        .fold((0usize, f32::NEG_INFINITY), |best, (i, &v)| {
            if v > best.1 {
                (i, v)
            } else {
                best
            }
        })
        .0
}

/// The decision cells: `g >= 1` with a multi-piece key, every form.
pub(super) fn relevant_cell(gap: usize, key_len: usize) -> bool {
    gap >= 1 && key_len >= 2
}

/// The reference rule over a set alone (no arm): per-cell rule accuracy.
fn rule_only(task: &FactTask, sequences: &[FactSequence]) -> Value {
    let bare_len = task.vocab.bare_prefix.len();
    let mut cells = BTreeMap::new();
    for sequence in sequences {
        for item in &sequence.items {
            let rule = reference_rule(&sequence.tokens, item, bare_len);
            let outcome = Outcome {
                first: false,
                first_content: false,
                full: false,
                nll_first_content: 0.0,
                rule,
                rule_n: rule_n(&sequence.tokens, item),
            };
            tally(
                &mut cells,
                cell_name(item.form, item.gap, item.key_len),
                &outcome,
            );
        }
    }
    json!({
        "cells": cells.iter().map(|(k, t)| (k.clone(), json!({
            "total": t.total,
            "rule_full_accuracy": t.rule as f64 / t.total.max(1) as f64,
            "rule_n_full_accuracy": t.rule_n.iter().map(|&n| n as f64 / t.total.max(1) as f64).collect::<Vec<_>>(),
        }))).collect::<serde_json::Map<_, _>>(),
    })
}

fn rule_n(tokens: &[u32], item: &FactItem) -> [bool; RULE_MAX_N] {
    let mut hits = [false; RULE_MAX_N];
    for (n, hit) in hits.iter_mut().enumerate() {
        *hit = nlet_rule(tokens, item.predict[0], n + 1, item.value.len()).as_deref()
            == Some(item.value.as_slice());
    }
    hits
}

/// Recall per cell and marginal: argmax over the whole vocabulary at every
/// value position, teacher forced.
fn evaluate_fact(
    arm: &dyn ContextArm,
    task: &FactTask,
    sequences: &[FactSequence],
    context: usize,
    buckets: &[Bucket],
    chunk: usize,
) -> Result<Value> {
    let bare_len = task.vocab.bare_prefix.len();
    let mut cells: BTreeMap<String, FactTally> = BTreeMap::new();
    let mut marginals: BTreeMap<String, FactTally> = BTreeMap::new();
    for group in sequences.chunks(chunk.max(1)) {
        let (ids, _, _) = fact_arrays(group, context);
        let logits = arm.logits(&ids, group.len(), context)?;
        let mut rows = Vec::new();
        for (s, sequence) in group.iter().enumerate() {
            for item in &sequence.items {
                rows.extend(item.predict.iter().map(|&p| (s * context + p) as u32));
            }
        }
        let index = Tensor::from_vec(rows.clone(), rows.len(), arm.device())?;
        let selected = logits
            .index_select(&index, 0)?
            .to_device(&Device::Cpu)?
            .to_vec2::<f32>()?;
        let mut row = 0;
        for sequence in group {
            for item in &sequence.items {
                let content = task.vocab.first_content(&item.value);
                let mut hits = Vec::with_capacity(item.value.len());
                let mut nll = 0.0;
                for (i, &piece) in item.value.iter().enumerate() {
                    let scores = &selected[row];
                    row += 1;
                    hits.push(argmax(scores) == piece as usize);
                    if i == content {
                        let maximum = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                        let sum: f64 = scores.iter().map(|&v| f64::from(v - maximum).exp()).sum();
                        nll = sum.ln() - f64::from(scores[piece as usize] - maximum);
                    }
                }
                let outcome = Outcome {
                    first: hits[0],
                    first_content: hits[content],
                    full: hits.iter().all(|&h| h),
                    nll_first_content: nll,
                    rule: reference_rule(&sequence.tokens, item, bare_len),
                    rule_n: rule_n(&sequence.tokens, item),
                };
                tally(
                    &mut cells,
                    cell_name(item.form, item.gap, item.key_len),
                    &outcome,
                );
                for key in [
                    "all".to_string(),
                    format!("form.{}", item.form.name()),
                    format!("gap.{}", item.gap),
                    format!("key_pieces.{}", item.key_len),
                    format!("value.{}", item.kind.name()),
                    format!("bucket.{}", buckets[item.bucket].name),
                    format!("relevant.{}", relevant_cell(item.gap, item.key_len)),
                ] {
                    tally(&mut marginals, key, &outcome);
                }
            }
        }
    }
    let relevant_min = cells
        .iter()
        .filter(|(name, _)| {
            let parts: Vec<&str> = name.split('.').collect();
            let gap = parts[1][1..].parse::<usize>().unwrap_or(0);
            let key_len = parts[2][1..].parse::<usize>().unwrap_or(0);
            relevant_cell(gap, key_len)
        })
        .map(|(_, t)| t.full as f64 / t.total.max(1) as f64)
        .fold(f64::INFINITY, f64::min);
    let distance: BTreeMap<String, f64> = buckets
        .iter()
        .enumerate()
        .map(|(b, bucket)| {
            let (sum, count) = sequences
                .iter()
                .flat_map(|s| &s.items)
                .filter(|item| item.bucket == b)
                .fold((0usize, 0usize), |(sum, count), item| {
                    (sum + item.distance, count + 1)
                });
            (bucket.name.to_string(), sum as f64 / count.max(1) as f64)
        })
        .collect();
    Ok(json!({
        "cells": cells.iter().map(|(k, t)| (k.clone(), t.record())).collect::<serde_json::Map<_, _>>(),
        "marginals": marginals.iter().map(|(k, t)| (k.clone(), t.record())).collect::<serde_json::Map<_, _>>(),
        "relevant_cell_min_full_accuracy": if relevant_min.is_finite() { json!(relevant_min) } else { Value::Null },
        "mean_distance_by_bucket": distance,
    }))
}

/// The final read probe: each read head's weight, at the position that
/// predicts the first value piece, on the fact's first value piece and on
/// its last key piece, by gap (one item per probe sequence and gap).
fn fact_probe(
    arm: &dyn ContextArm,
    task: &FactTask,
    sequences: &[FactSequence],
    context: usize,
) -> Result<Value> {
    let heads = arm.read_heads();
    if heads.is_empty() {
        return Ok(json!({"status": "UNAVAILABLE: the arm has no read heads"}));
    }
    let (ids, _, _) = fact_arrays(sequences, context);
    let mut by_gap = serde_json::Map::new();
    for &gap in &task.gaps {
        let picks: Vec<(usize, &FactItem)> = sequences
            .iter()
            .enumerate()
            .filter_map(|(s, sequence)| {
                sequence
                    .items
                    .iter()
                    .find(|item| item.gap == gap && item.form == Form::Rehearse)
                    .map(|item| (s, item))
            })
            .collect();
        if picks.is_empty() {
            continue;
        }
        let mut per_head = Vec::new();
        let (mut best_value, mut best_key) = (0f64, 0f64);
        for &(layer, head) in &heads {
            let mut means = [0f64; 2];
            for (which, source) in [(0usize, true), (1, false)] {
                let rows: Vec<(usize, usize, Vec<usize>)> = picks
                    .iter()
                    .map(|(s, item)| {
                        let position = if source {
                            item.value_first()
                        } else {
                            item.key_last()
                        };
                        (*s, item.predict[0], vec![position])
                    })
                    .collect();
                let masses = arm.source_mass(&ids, sequences.len(), context, layer, head, &rows)?;
                means[which] =
                    masses.iter().map(|&m| f64::from(m)).sum::<f64>() / masses.len().max(1) as f64;
            }
            best_value = best_value.max(means[0]);
            best_key = best_key.max(means[1]);
            per_head.push(json!({
                "layer": layer, "head": head,
                "mean_weight_on_fact_value_first": means[0],
                "mean_weight_on_fact_key_last": means[1],
            }));
        }
        by_gap.insert(
            format!("g{gap}"),
            json!({
                "form": "rehearse",
                "queries": picks.len(),
                "best_head_mean_weight_on_fact_value_first": best_value,
                "best_head_mean_weight_on_fact_key_last": best_key,
                "heads": per_head,
            }),
        );
    }
    Ok(json!({"by_gap": by_gap}))
}

fn fact_sequences(
    s: &Common,
    task: &FactTask,
    buckets: &[Bucket],
    domain: u64,
    count: usize,
    pairing: Pairing,
) -> Result<Vec<FactSequence>> {
    (0..count)
        .map(|i| {
            let mut rng = Rng::new(s.seed, domain, i as u64);
            generate_fact(
                &mut rng,
                task,
                s.context,
                buckets,
                s.pairs_per_bucket,
                pairing,
            )
        })
        .collect()
}

fn summary(result: &Value) -> String {
    let m = &result["marginals"];
    format!(
        "full {:.3} first-content {:.3} rehearse {:.3} bare {:.3} relevant-min {:.3} (rule {:.3})",
        m["all"]["full_accuracy"].as_f64().unwrap_or(f64::NAN),
        m["all"]["first_content_piece_accuracy"]
            .as_f64()
            .unwrap_or(f64::NAN),
        m["form.rehearse"]["full_accuracy"]
            .as_f64()
            .unwrap_or(f64::NAN),
        m["form.bare"]["full_accuracy"].as_f64().unwrap_or(f64::NAN),
        result["relevant_cell_min_full_accuracy"]
            .as_f64()
            .unwrap_or(f64::NAN),
        m["all"]["rule_full_accuracy"].as_f64().unwrap_or(f64::NAN),
    )
}

/// Trains `arm` on the fact task and scores it.
fn run_fact(
    s: &Common,
    task: &FactTask,
    arm: &mut dyn ContextArm,
    log: &mut fs::File,
) -> Result<Value> {
    use std::io::Write;
    let started = Instant::now();
    let buckets = buckets_for(s.context);
    let line = format!(
        "{} {}: {} parameters, context {}, fact layout, buckets {:?}",
        arm.kind(),
        arm.record(),
        arm.parameters(),
        s.context,
        buckets.iter().map(|b| b.name).collect::<Vec<_>>()
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let curve_in_class = fact_sequences(
        s,
        task,
        &buckets,
        FACT_CURVE_IN_CLASS_DOMAIN,
        s.curve_sequences,
        Pairing::Train,
    )?;
    let curve_held_out = fact_sequences(
        s,
        task,
        &buckets,
        FACT_CURVE_DOMAIN,
        s.curve_sequences,
        Pairing::HeldOut,
    )?;
    let chunk = s.batch;
    let evaluate = |arm: &dyn ContextArm, set: &[FactSequence]| {
        evaluate_fact(arm, task, set, s.context, &buckets, chunk)
    };
    let mut curve = vec![json!({
        "step": 0,
        "in_class": evaluate(&*arm, &curve_in_class)?,
        "held_out": evaluate(&*arm, &curve_held_out)?,
    })];
    let (mut window_loss, mut window_steps) = (0f64, 0usize);
    let mut step_seconds = Vec::new();
    let mut stopped_early = false;
    let mut step = 0;
    while step < s.steps {
        if started.elapsed().as_secs_f64() > s.max_seconds {
            stopped_early = true;
            break;
        }
        let clock = Instant::now();
        let lr = cosine_rate(s.lr, s.warmup, s.min_lr, s.steps, step);
        let batch: Vec<FactSequence> = (0..s.batch)
            .map(|b| {
                let mut rng = Rng::new(s.seed, FACT_TRAIN_DOMAIN, (step * s.batch + b) as u64);
                generate_fact(
                    &mut rng,
                    task,
                    s.context,
                    &buckets,
                    s.pairs_per_bucket,
                    Pairing::Train,
                )
            })
            .collect::<Result<_>>()?;
        let (ids, targets, weights) = fact_arrays(&batch, s.context);
        let loss = arm.loss(&ids, &targets, &weights, s.batch, s.context)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!("nonfinite loss at step {step}")));
        }
        let grad_norm = arm.update(&loss, lr)?;
        step += 1;
        step_seconds.push(clock.elapsed().as_secs_f64());
        window_loss += value;
        window_steps += 1;
        if step % s.eval_every == 0 || step == s.steps {
            let in_class = evaluate(&*arm, &curve_in_class)?;
            let held_out = evaluate(&*arm, &curve_held_out)?;
            let train_loss = window_loss / window_steps.max(1) as f64;
            let held_out_all = &held_out["marginals"]["all"];
            let held_out_total = held_out_all["total"].as_u64().unwrap_or(0);
            let line = format!(
                "step {step} lr {lr:.2e} train value NLL {train_loss:.4} grad {grad_norm:.3} \
in-class {} | held-out (n={held_out_total}) full {:.3} {}/{held_out_total} \
first-content {:.3} {}/{held_out_total} ({:.0}s)",
                summary(&in_class),
                held_out_all["full_accuracy"].as_f64().unwrap_or(f64::NAN),
                held_out_all["full_correct"].as_u64().unwrap_or(0),
                held_out_all["first_content_piece_accuracy"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                held_out_all["first_content_correct"].as_u64().unwrap_or(0),
                started.elapsed().as_secs_f64()
            );
            eprintln!("{line}");
            writeln!(log, "{line}")?;
            curve.push(json!({
                "step": step, "lr": lr, "train_value_nll": train_loss,
                "grad_norm": grad_norm, "in_class": in_class, "held_out": held_out,
                "elapsed_seconds": started.elapsed().as_secs_f64(),
            }));
            window_loss = 0.0;
            window_steps = 0;
        }
    }
    let train_seconds: f64 = step_seconds.iter().sum();
    let eval_clock = Instant::now();
    let in_class_set = fact_sequences(
        s,
        task,
        &buckets,
        FACT_IN_CLASS_DOMAIN,
        s.final_sequences,
        Pairing::Train,
    )?;
    let in_class = evaluate(&*arm, &in_class_set)?;
    let held_out_set = fact_sequences(
        s,
        task,
        &buckets,
        FACT_HELD_OUT_DOMAIN,
        s.final_sequences / 2,
        Pairing::HeldOut,
    )?;
    let held_out = evaluate(&*arm, &held_out_set)?;
    let line = format!(
        "final in-class fresh {}; held-out-class {}",
        summary(&in_class),
        summary(&held_out)
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let final_eval_seconds = eval_clock.elapsed().as_secs_f64();
    let probe = if s.probe_steps.is_empty() {
        json!({"status": "not requested (probe_steps=none)"})
    } else {
        let probe_set = fact_sequences(
            s,
            task,
            &buckets,
            FACT_PROBE_DOMAIN,
            s.probe_sequences,
            Pairing::Train,
        )?;
        fact_probe(&*arm, task, &probe_set, s.context)?
    };
    let mut sorted = step_seconds.clone();
    sorted.sort_by(f64::total_cmp);
    let mean_positions = (0..s.context)
        .map(|t| arm.positions_scored(t) as f64)
        .sum::<f64>()
        / s.context as f64;
    let targets_per_step: usize = (0..s.batch)
        .map(|b| {
            let mut rng = Rng::new(s.seed, FACT_TRAIN_DOMAIN, b as u64);
            generate_fact(
                &mut rng,
                task,
                s.context,
                &buckets,
                s.pairs_per_bucket,
                Pairing::Train,
            )
            .map(|seq| seq.items.iter().map(|i| i.value.len()).sum::<usize>())
            .unwrap_or(0)
        })
        .sum();
    Ok(json!({
        "context_access": {
            "positions_scored_per_token_mean_over_window": mean_positions,
            "positions_scored_at_last_position": arm.positions_scored(s.context - 1),
            "counts": arm.access_note(),
        },
        "steps_completed": step,
        "stopped_early_at_max_seconds": stopped_early,
        "final_in_class_fresh_pairings": in_class,
        "final_held_out_class_pairings": held_out,
        "final_read_probe": probe,
        "curve": curve,
        "train_seconds": train_seconds,
        "median_step_seconds": sorted.get(sorted.len() / 2).copied(),
        "final_eval_seconds": final_eval_seconds,
        "wall_seconds": started.elapsed().as_secs_f64(),
        "facts_seen": step * s.batch * buckets.len() * s.pairs_per_bucket,
        "value_targets_in_first_batch": targets_per_step,
    }))
}

/// `layout=fact`: claim, train (or `mode=task-baselines`: the reference
/// rule alone), report and seal.
pub(super) fn main(
    common: &Common,
    spec: &ArmSpec,
    baselines_only: bool,
    task: &FactTask,
) -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    let task_record = json!({"common": common.record(), "fact": task.record()});
    // Claimed exclusively after argument validation, before any model work.
    report_output::claim(&common.out)?;
    if baselines_only {
        let buckets = buckets_for(common.context);
        let in_class = fact_sequences(
            common,
            task,
            &buckets,
            FACT_IN_CLASS_DOMAIN,
            common.final_sequences,
            Pairing::Train,
        )?;
        let held_out = fact_sequences(
            common,
            task,
            &buckets,
            FACT_HELD_OUT_DOMAIN,
            common.final_sequences / 2,
            Pairing::HeldOut,
        )?;
        let report = json!({
            "schema": FACT_SCHEMA, "status": "complete", "mode": "task-baselines",
            "task": task_record, "argv": argv,
            "final_in_class_fresh_pairings": rule_only(task, &in_class),
            "final_held_out_class_pairings": rule_only(task, &held_out),
        });
        write_json(&common.out.join("report.json"), &report)?;
        report_output::seal(&common.out)?;
        report_output::verify(&common.out)?;
        return Ok(());
    }
    write_json(
        &common.out.join("config.json"),
        &json!({"schema": FACT_SCHEMA, "task": task_record, "arm_label": spec.label(), "argv": argv}),
    )?;
    let mut log = fs::File::create(common.out.join("log.txt"))?;
    let result = (|| -> Result<(Value, Value)> {
        let device = uor_r4_training::baseline_protocol::device(&common.device_name)?;
        let mut arm = spec.build(common, &device)?;
        let arm_record = json!({
            "label": spec.label(),
            "kind": arm.kind(),
            "parameters": arm.parameters(),
            "config": arm.record(),
        });
        let mut results = run_fact(common, task, arm.as_mut(), &mut log)?;
        results["model_save"] = save_model(common, arm.as_ref());
        Ok((arm_record, results))
    })();
    drop(log);
    let report = match &result {
        Ok((arm, results)) => json!({
            "schema": FACT_SCHEMA, "status": "complete", "task": task_record, "argv": argv,
            "arm": arm, "results": results,
        }),
        Err(error) => json!({
            "schema": FACT_SCHEMA, "status": "failed", "task": task_record, "argv": argv,
            "arm": {"label": spec.label()}, "error": error.to_string(),
        }),
    };
    write_json(&common.out.join("report.json"), &report)?;
    report_output::seal(&common.out)?;
    report_output::verify(&common.out)?;
    result.map(|_| ())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// The real #1017 tokenizer when present (the tests that need it skip
    /// with a note otherwise; they are not counted as passing evidence then).
    pub(in super::super) fn tokenizer_path() -> Option<PathBuf> {
        let path = std::env::var_os("UOR_R4_FACT_TOKENIZER")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| {
                    PathBuf::from(home).join(
                        "uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json",
                    )
                })
            })?;
        path.exists().then_some(path)
    }

    pub(in super::super) fn task() -> Option<FactTask> {
        let path = tokenizer_path()?;
        Some(FactTask {
            vocab: FactVocab::load(&path).expect("tokenizer"),
            tokenizer_path: path,
            gaps: vec![0, 1, 2, 3],
            forms: vec![Form::Rehearse, Form::Bare],
        })
    }

    #[test]
    fn the_nlet_rule_copies_after_the_latest_earlier_occurrence() {
        let tokens = [9, 1, 2, 5, 6, 1, 2, 7, 8, 1, 2];
        // Last 2-let (1, 2) at 9..=10; latest earlier occurrence ends at 6.
        assert_eq!(nlet_rule(&tokens, 10, 2, 2), Some(vec![7, 8]));
        assert_eq!(nlet_rule(&tokens, 10, 1, 1), Some(vec![7]));
        assert_eq!(nlet_rule(&tokens, 10, 4, 1), None);
        assert_eq!(nlet_rule(&tokens, 2, 2, 1), None);
        assert_eq!(nlet_backoff(&tokens, 10, 1), Some(vec![7]));
    }

    #[test]
    fn fact_windows_hold_their_facts_queries_and_targets() {
        let Some(task) = task() else {
            eprintln!("SKIPPED: the #1017 tokenizer is absent");
            return;
        };
        let buckets = buckets_for(SERVED_CONTEXT);
        assert_eq!(buckets.len(), 3);
        let vocab = &task.vocab;
        assert!(vocab.vocab >= 4096);
        // Numerals are written as the tokenizer writes them: " " then a digit.
        for numeral in &vocab.numerals {
            assert_eq!(vocab.first_content(numeral), numeral.len() - 1);
        }
        let mut cells = BTreeSet::new();
        let mut rule_rehearse = (0usize, 0usize);
        for (pairing, domain) in [
            (Pairing::Train, FACT_TRAIN_DOMAIN),
            (Pairing::HeldOut, FACT_HELD_OUT_DOMAIN),
        ] {
            for index in 0..64 {
                let mut rng = Rng::new(5, domain, index);
                let sequence = generate_fact(&mut rng, &task, SERVED_CONTEXT, &buckets, 4, pairing)
                    .expect("window");
                assert_eq!(sequence.tokens.len(), SERVED_CONTEXT);
                assert_eq!(sequence.items.len(), 12);
                let mut key_pieces = BTreeSet::new();
                for item in &sequence.items {
                    assert_eq!(item.key.len(), item.key_len);
                    assert!((1..=3).contains(&item.key_len) && item.gap <= 3);
                    assert_eq!(item.key[0] % 4 == 0, pairing == Pairing::HeldOut);
                    for &piece in &item.key {
                        assert!(key_pieces.insert(piece), "a key piece repeats");
                    }
                    let t = &sequence.tokens;
                    let f = item.fact_start;
                    assert_eq!(&t[f..f + item.key_len], item.key.as_slice());
                    assert_eq!(
                        &t[f + item.key_len..f + item.key_len + item.gap],
                        item.gap_tokens.as_slice()
                    );
                    assert_eq!(
                        &t[item.value_first()..item.value_first() + item.value.len()],
                        item.value.as_slice()
                    );
                    assert!(item.query_start >= f + item.fact_len());
                    assert_eq!(item.query_start - f, item.distance);
                    let bucket = buckets[item.bucket];
                    assert!((bucket.low..=bucket.high).contains(&item.distance));
                    let q = item.query_start;
                    assert_eq!(&t[q..q + item.key_len], item.key.as_slice());
                    let middle = match item.form {
                        Form::Rehearse => item.gap_tokens.clone(),
                        Form::Bare => vocab.bare_prefix.clone(),
                    };
                    assert_eq!(
                        &t[q + item.key_len..q + item.key_len + middle.len()],
                        middle.as_slice()
                    );
                    // The predicting position holds the last copula piece
                    // (or the last key piece when g = 0, or the bare prefix).
                    let p0 = item.predict[0];
                    let expected = match (item.form, item.gap) {
                        (Form::Rehearse, 0) => *item.key.last().expect("key"),
                        (Form::Rehearse, _) => *item.gap_tokens.last().expect("gap"),
                        (Form::Bare, _) => *vocab.bare_prefix.last().expect("prefix"),
                    };
                    assert_eq!(t[p0], expected);
                    for (i, &piece) in item.value.iter().enumerate() {
                        assert_eq!(t[item.predict[i] + 1], piece);
                    }
                    // The key's full tuple occurs exactly twice: fact and query.
                    let occurrences = (0..=t.len() - item.key_len)
                        .filter(|&p| t[p..p + item.key_len] == item.key[..])
                        .count();
                    assert_eq!(occurrences, 2);
                    cells.insert(cell_name(item.form, item.gap, item.key_len));
                    if item.form == Form::Rehearse {
                        rule_rehearse.1 += 1;
                        rule_rehearse.0 +=
                            usize::from(reference_rule(t, item, vocab.bare_prefix.len()));
                    }
                }
                // Gap pieces are never key pieces of the window.
                for item in &sequence.items {
                    assert!(item.gap_tokens.iter().all(|g| !key_pieces.contains(g)));
                }
                // Weights fall on value pieces only.
                let (_, targets, weights) =
                    fact_arrays(std::slice::from_ref(&sequence), SERVED_CONTEXT);
                let weighted = weights.iter().filter(|&&w| w > 0.0).count();
                let pieces: usize = sequence.items.iter().map(|i| i.value.len()).sum();
                assert_eq!(weighted, pieces);
                for item in &sequence.items {
                    assert_eq!(targets[item.predict[0]], item.value[0]);
                }
            }
        }
        assert_eq!(cells.len(), 24, "every (form, gap, key pieces) cell occurs");
        // A rehearsed query is solved by the n-let backoff (the rule's ceiling).
        assert_eq!(rule_rehearse.0, rule_rehearse.1);
    }
}
