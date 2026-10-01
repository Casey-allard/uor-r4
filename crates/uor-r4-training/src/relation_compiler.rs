//! E3 of the log-sieve design (`docs/integration/log-sieve-retrieval-design-2026-10-01.md`
//! §2.3): can a learned head name the relation and the act of a user turn
//! under paraphrase? This is the language-to-atom compiler #958 lacked.
//!
//! Every user turn of M-world v2 is labelled from the generator's typed intent:
//! its relation (one of [`relation_names`], or [`NONE`]) and its act
//! (assert, update, query, or [`NONE`]; a query of an unstated relation is a
//! query). Two softmax classifiers are fitted on turns drawn with *training*
//! phrasings and scored on turns drawn with *development* phrasings, whose
//! templates the fit never sees:
//!
//! - **trunk**: the frozen stack's final normalized states for the turn read
//!   alone (BOS, the user prefix and the assistant marker), last position and
//!   mean over positions;
//! - **lexical**: the words of the turn (a control: development phrasings
//!   change content words, so words unseen in training carry nothing).
//!
//! The fit is full-batch gradient descent on standardized features, in f64,
//! with no randomness, so a run is a function of its inputs.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::DialogueEncoder;

use crate::geometric_stack::StackModel;
use crate::milestone_world::words;
use crate::milestone_world_v2::{relation_names, Cell, MWorld2, Turn2};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// The label of a turn that states, updates or asks no relation.
pub const NONE: &str = "none";
/// The acts, in label order.
pub const ACTS: [&str; 4] = ["assert", "update", "query", NONE];

/// A user turn and its labels.
#[derive(Clone, Debug)]
pub struct Example {
    pub text: String,
    pub relation: String,
    pub act: &'static str,
    /// The phrasing template (or teacher paraphrase) the turn was drawn
    /// from, slot unfilled.
    pub template: Option<String>,
}

impl Example {
    /// The value filling the template's `{v}` slot in the text, if the turn
    /// has a one-slot template that the text matches.
    pub fn slot_value(&self) -> Option<&str> {
        let (start, end) = self.slot_span()?;
        Some(&self.text[start..end])
    }

    /// The half-open byte span of [`Self::slot_value`] in the text.
    pub fn slot_span(&self) -> Option<(usize, usize)> {
        let template = self.template.as_deref()?;
        let (before, after) = template.split_once("{v}")?;
        if after.contains("{v}") {
            return None;
        }
        let value = self.text.strip_prefix(before)?.strip_suffix(after)?;
        (!value.trim().is_empty()).then_some((before.len(), before.len() + value.len()))
    }
}

/// Training examples from teacher paraphrases (one JSON object per line with
/// `relation`, `act` and `text`): each `{v}` is filled with a value the
/// training draws gave that relation, in turn (a paraphrase whose relation
/// has no recorded value is skipped). Returns the examples and the count
/// skipped.
pub fn paraphrase_examples(jsonl: &str, training: &[Example]) -> Result<(Vec<Example>, usize)> {
    let mut values: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for example in training {
        if let Some(value) = example.slot_value() {
            let pool = values.entry(example.relation.as_str()).or_default();
            if !pool.contains(&value) {
                pool.push(value);
            }
        }
    }
    let mut next: BTreeMap<String, usize> = BTreeMap::new();
    let (mut examples, mut skipped) = (Vec::new(), 0usize);
    for line in jsonl.lines().filter(|line| !line.trim().is_empty()) {
        let row: Value = serde_json::from_str(line)?;
        let (Some(relation), Some(act), Some(text)) = (
            row["relation"].as_str(),
            row["act"].as_str(),
            row["text"].as_str(),
        ) else {
            return Err(invalid("a paraphrase row needs relation, act and text"));
        };
        let act = ACTS
            .iter()
            .copied()
            .find(|a| *a == act && *a != NONE)
            .ok_or_else(|| invalid(format!("unknown paraphrase act {act}")))?;
        if !relation_names().contains(&relation) {
            return Err(invalid(format!("unknown paraphrase relation {relation}")));
        }
        let pattern = text;
        let text = if pattern.contains("{v}") {
            let Some(pool) = values.get(relation).filter(|pool| !pool.is_empty()) else {
                skipped += 1;
                continue;
            };
            let i = next.entry(relation.to_owned()).or_default();
            let value = pool[*i % pool.len()];
            *i += 1;
            pattern.replace("{v}", value)
        } else {
            pattern.to_owned()
        };
        examples.push(Example {
            text,
            relation: relation.to_owned(),
            act,
            template: Some(pattern.to_owned()),
        });
    }
    Ok((examples, skipped))
}

/// A turn's relation and act from its typed intent.
pub fn label(turn: &Turn2) -> (String, &'static str) {
    for name in relation_names() {
        if let Some(act) = turn
            .intent
            .strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('_'))
        {
            let act = match act {
                "assert" => "assert",
                "update" => "update",
                "query" | "absent" => "query",
                _ => continue,
            };
            return (name.to_owned(), act);
        }
    }
    (NONE.to_owned(), NONE)
}

/// Every user turn of `conversations` episodes of `cell`, labelled.
pub fn collect(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    cell: Cell,
    conversations: usize,
) -> Result<Vec<Example>> {
    let mut examples = Vec::new();
    for _ in 0..conversations {
        for turn in world.conversation_in(rng, cell)?.turns {
            let (relation, act) = label(&turn);
            examples.push(Example {
                text: turn.user,
                relation,
                act,
                template: turn.tag.template,
            });
        }
    }
    Ok(examples)
}

/// The frozen trunk's features of a user turn read alone: the final
/// normalized state at the last position (the assistant marker) and the mean
/// over all positions, `2 * width` values.
pub fn trunk_features(
    model: &StackModel,
    encoder: &DialogueEncoder<'_>,
    bos: u32,
    text: &str,
) -> Result<Vec<f64>> {
    let prefix = encoder.encode_user_prefix(text, false);
    if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
        return Err(invalid("a user turn did not encode as one plain turn"));
    }
    let mut ids = vec![bos];
    ids.extend(&prefix.tokens);
    if ids.len() > model.config.context {
        return Err(invalid("a user turn is longer than the model's context"));
    }
    let width = model.config.width;
    let states = model
        .hidden(&ids, 1, ids.len())?
        .flatten_all()?
        .to_vec1::<f32>()?;
    let rows = states.len() / width;
    let mut features = vec![0f64; 2 * width];
    for (i, value) in states[(rows - 1) * width..].iter().enumerate() {
        features[i] = f64::from(*value);
    }
    for row in states.chunks(width) {
        for (i, value) in row.iter().enumerate() {
            features[width + i] += f64::from(*value) / rows as f64;
        }
    }
    Ok(features)
}

/// Binary bag-of-words features over a fixed vocabulary.
pub struct Lexicon {
    index: BTreeMap<String, usize>,
    /// Per word, `sqrt((1 - p) / p)` for its document frequency `p` in the
    /// fitted texts: the value a present word takes when standardized, so a
    /// sparse table can weigh rare words as a standardized fit does.
    scale: Vec<f64>,
}

impl Lexicon {
    /// The vocabulary of `texts` (lowercased words without punctuation).
    pub fn fit<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut index = BTreeMap::new();
        let mut documents: Vec<usize> = Vec::new();
        let mut total = 0usize;
        for text in texts {
            total += 1;
            let mut seen = std::collections::BTreeSet::new();
            for word in words(text) {
                let next = index.len();
                let i = *index.entry(word).or_insert(next);
                if i == documents.len() {
                    documents.push(0);
                }
                if seen.insert(i) {
                    documents[i] += 1;
                }
            }
        }
        let scale = documents
            .iter()
            .map(|&d| {
                let p = d as f64 / total.max(1) as f64;
                if p >= 1.0 {
                    1.0
                } else {
                    ((1.0 - p) / p).sqrt()
                }
            })
            .collect();
        Self { index, scale }
    }

    /// The present words of `text` with their standardized scale, sorted.
    pub fn scaled(&self, text: &str) -> Vec<(usize, f64)> {
        self.active(text)
            .into_iter()
            .map(|i| (i, self.scale[i]))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// The vocabulary indices of `text`'s words, sorted and distinct.
    pub fn active(&self, text: &str) -> Vec<usize> {
        let mut active: Vec<usize> = words(text)
            .iter()
            .filter_map(|word| self.index.get(word).copied())
            .collect();
        active.sort_unstable();
        active.dedup();
        active
    }

    /// The features of `text`; words outside the vocabulary carry nothing.
    pub fn features(&self, text: &str) -> Vec<f64> {
        let mut features = vec![0f64; self.index.len()];
        for word in words(text) {
            if let Some(&i) = self.index.get(&word) {
                features[i] = 1.0;
            }
        }
        features
    }
}

/// A softmax classifier on standardized features.
pub struct Softmax {
    classes: usize,
    dim: usize,
    mean: Vec<f64>,
    scale: Vec<f64>,
    weights: Vec<f64>,
    bias: Vec<f64>,
}

impl Softmax {
    /// Fit by `steps` full-batch gradient steps of size `rate` on the mean
    /// cross-entropy plus `l2` times the squared weights.
    pub fn fit(
        x: &[Vec<f64>],
        y: &[usize],
        classes: usize,
        steps: usize,
        rate: f64,
        l2: f64,
    ) -> Result<Self> {
        let dim = x.first().map_or(0, Vec::len);
        if x.is_empty()
            || x.len() != y.len()
            || classes < 2
            || y.iter().any(|&label| label >= classes)
            || x.iter().any(|row| row.len() != dim)
        {
            return Err(invalid("a classifier needs labelled rows of one width"));
        }
        let n = x.len() as f64;
        let mut mean = vec![0f64; dim];
        for row in x {
            for (m, v) in mean.iter_mut().zip(row) {
                *m += v / n;
            }
        }
        let mut scale = vec![0f64; dim];
        for row in x {
            for ((s, v), m) in scale.iter_mut().zip(row).zip(&mean) {
                *s += (v - m) * (v - m) / n;
            }
        }
        for s in &mut scale {
            *s = if *s > 1e-12 { s.sqrt() } else { 1.0 };
        }
        let mut model = Self {
            classes,
            dim,
            mean,
            scale,
            weights: vec![0f64; classes * dim],
            bias: vec![0f64; classes],
        };
        let standardized: Vec<Vec<f64>> = x.iter().map(|row| model.standardize(row)).collect();
        for _ in 0..steps {
            let mut grad_w = vec![0f64; classes * dim];
            let mut grad_b = vec![0f64; classes];
            for (row, &label) in standardized.iter().zip(y) {
                let p = model.probabilities_standardized(row);
                for c in 0..classes {
                    let g = (p[c] - f64::from(u8::from(c == label))) / n;
                    grad_b[c] += g;
                    for (w, v) in grad_w[c * dim..(c + 1) * dim].iter_mut().zip(row) {
                        *w += g * v;
                    }
                }
            }
            for (w, g) in model.weights.iter_mut().zip(&grad_w) {
                *w -= rate * (g + l2 * *w);
            }
            for (b, g) in model.bias.iter_mut().zip(&grad_b) {
                *b -= rate * g;
            }
        }
        Ok(model)
    }

    fn standardize(&self, row: &[f64]) -> Vec<f64> {
        row.iter()
            .zip(&self.mean)
            .zip(&self.scale)
            .map(|((v, m), s)| (v - m) / s)
            .collect()
    }

    fn probabilities_standardized(&self, row: &[f64]) -> Vec<f64> {
        let logits: Vec<f64> = (0..self.classes)
            .map(|c| {
                self.bias[c]
                    + self.weights[c * self.dim..(c + 1) * self.dim]
                        .iter()
                        .zip(row)
                        .map(|(w, v)| w * v)
                        .sum::<f64>()
            })
            .collect();
        let top = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = logits.iter().map(|l| (l - top).exp()).collect();
        let total: f64 = exp.iter().sum();
        exp.iter().map(|e| e / total).collect()
    }

    /// The most probable class of a raw feature row (ties to the lower class).
    pub fn predict(&self, row: &[f64]) -> usize {
        let p = self.probabilities_standardized(&self.standardize(row));
        let mut best = 0;
        for c in 1..p.len() {
            if p[c] > p[best] {
                best = c;
            }
        }
        best
    }
}

/// A softmax classifier on sparse features (each row's present indices with
/// their values), so a row costs only its present features. With the
/// lexicon's standardized scale ([`Lexicon::scaled`]) it weighs rare words as a
/// standardized fit does. In serving it is a table of word weights summed per
/// class.
pub struct SparseSoftmax {
    classes: usize,
    dim: usize,
    weights: Vec<f64>,
    bias: Vec<f64>,
}

impl SparseSoftmax {
    /// Full-batch gradient descent on the mean cross-entropy plus `l2` times
    /// the squared weights (applied as a shrink each step).
    pub fn fit(
        rows: &[Vec<(usize, f64)>],
        y: &[usize],
        classes: usize,
        dim: usize,
        steps: usize,
        rate: f64,
        l2: f64,
    ) -> Result<Self> {
        if rows.is_empty()
            || rows.len() != y.len()
            || classes < 2
            || y.iter().any(|&label| label >= classes)
            || rows
                .iter()
                .flatten()
                .any(|&(i, v)| i >= dim || !v.is_finite())
        {
            return Err(invalid(
                "a sparse classifier needs labelled rows inside its width",
            ));
        }
        let n = rows.len() as f64;
        let mut model = Self {
            classes,
            dim,
            weights: vec![0f64; classes * dim],
            bias: vec![0f64; classes],
        };
        for _ in 0..steps {
            let mut grad_w = vec![0f64; classes * dim];
            let mut grad_b = vec![0f64; classes];
            for (row, &label) in rows.iter().zip(y) {
                let p = model.probabilities(row);
                for c in 0..classes {
                    let g = (p[c] - f64::from(u8::from(c == label))) / n;
                    grad_b[c] += g;
                    for &(i, v) in row {
                        grad_w[c * dim + i] += g * v;
                    }
                }
            }
            let shrink = 1.0 - rate * l2;
            for (w, g) in model.weights.iter_mut().zip(&grad_w) {
                *w = *w * shrink - rate * g;
            }
            for (b, g) in model.bias.iter_mut().zip(&grad_b) {
                *b -= rate * g;
            }
        }
        Ok(model)
    }

    fn probabilities(&self, row: &[(usize, f64)]) -> Vec<f64> {
        let logits: Vec<f64> = (0..self.classes)
            .map(|c| {
                self.bias[c]
                    + row
                        .iter()
                        .map(|&(i, v)| self.weights[c * self.dim + i] * v)
                        .sum::<f64>()
            })
            .collect();
        let top = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let exp: Vec<f64> = logits.iter().map(|l| (l - top).exp()).collect();
        let total: f64 = exp.iter().sum();
        exp.iter().map(|e| e / total).collect()
    }

    /// The most probable class of a row (ties to the lower class).
    pub fn predict(&self, row: &[(usize, f64)]) -> usize {
        let p = self.probabilities(row);
        let mut best = 0;
        for c in 1..p.len() {
            if p[c] > p[best] {
                best = c;
            }
        }
        best
    }

    fn logit(&self, row: &[(usize, f64)], class: usize) -> f64 {
        self.bias[class]
            + row
                .iter()
                .map(|&(i, v)| self.weights[class * self.dim + i] * v)
                .sum::<f64>()
    }

    /// Class 1's logit minus class 0's, for a two-class head.
    fn margin(&self, row: &[(usize, f64)]) -> f64 {
        self.logit(row, 1) - self.logit(row, 0)
    }
}

/// The standardization of dense features appended to a sparse word row.
struct DenseNorm {
    mean: Vec<f64>,
    scale: Vec<f64>,
}

/// The relation channel of the log-sieve design (§2.3) as a route: a word
/// table naming each user turn's relation and act, fitted on labelled turns,
/// optionally with a frozen trunk's features of the turn appended to each word
/// row at their standardized values.
pub struct RelationRoute {
    lexicon: Lexicon,
    relation_head: SparseSoftmax,
    act_head: SparseSoftmax,
    relations: Vec<String>,
    trunk: Option<DenseNorm>,
}

/// A turn's row: its present words at their standardized scale, then any
/// trunk features, standardized, after the vocabulary.
fn route_row(
    lexicon: &Lexicon,
    norm: Option<&DenseNorm>,
    text: &str,
    dense: Option<&[f64]>,
) -> Result<Vec<(usize, f64)>> {
    let mut row = lexicon.scaled(text);
    match (norm, dense) {
        (None, None) => {}
        (Some(norm), Some(x)) if x.len() == norm.mean.len() => {
            let base = lexicon.len();
            row.extend(
                x.iter()
                    .enumerate()
                    .map(|(j, v)| (base + j, (v - norm.mean[j]) / norm.scale[j])),
            );
        }
        _ => {
            return Err(invalid(
                "trunk features must accompany exactly a route fitted with them",
            ))
        }
    }
    Ok(row)
}

impl RelationRoute {
    pub fn fit(train: &[Example], steps: usize, rate: f64, l2: f64) -> Result<Self> {
        Self::fit_with(train, None, steps, rate, l2)
    }

    /// [`Self::fit`] with `trunk[i]`, the trunk's features of `train[i]`,
    /// appended to each word row.
    pub fn fit_with(
        train: &[Example],
        trunk: Option<&[Vec<f64>]>,
        steps: usize,
        rate: f64,
        l2: f64,
    ) -> Result<Self> {
        let lexicon = Lexicon::fit(train.iter().map(|e| e.text.as_str()));
        let relations: Vec<String> = relation_names()
            .iter()
            .map(|name| (*name).to_owned())
            .chain([NONE.to_owned()])
            .collect();
        let norm = match trunk {
            None => None,
            Some(x) => {
                let width = x.first().map_or(0, Vec::len);
                if x.len() != train.len() || width == 0 || x.iter().any(|row| row.len() != width) {
                    return Err(invalid(
                        "one trunk feature row of one width per training turn",
                    ));
                }
                let n = x.len() as f64;
                let mut mean = vec![0f64; width];
                for row in x {
                    for (m, v) in mean.iter_mut().zip(row) {
                        *m += v / n;
                    }
                }
                let mut scale = vec![0f64; width];
                for row in x {
                    for ((s, v), m) in scale.iter_mut().zip(row).zip(&mean) {
                        *s += (v - m) * (v - m) / n;
                    }
                }
                for s in &mut scale {
                    *s = if *s > 1e-12 { s.sqrt() } else { 1.0 };
                }
                Some(DenseNorm { mean, scale })
            }
        };
        let rows: Vec<Vec<(usize, f64)>> = train
            .iter()
            .enumerate()
            .map(|(i, e)| {
                route_row(
                    &lexicon,
                    norm.as_ref(),
                    &e.text,
                    trunk.map(|x| x[i].as_slice()),
                )
            })
            .collect::<Result<_>>()?;
        let relation_y: Vec<usize> = train
            .iter()
            .map(|e| {
                relations
                    .iter()
                    .position(|r| *r == e.relation)
                    .unwrap_or(relations.len() - 1)
            })
            .collect();
        let act_y: Vec<usize> = train
            .iter()
            .map(|e| {
                ACTS.iter()
                    .position(|a| *a == e.act)
                    .unwrap_or(ACTS.len() - 1)
            })
            .collect();
        let dim = (lexicon.len() + norm.as_ref().map_or(0, |n| n.mean.len())).max(1);
        Ok(Self {
            relation_head: SparseSoftmax::fit(
                &rows,
                &relation_y,
                relations.len(),
                dim,
                steps,
                rate,
                l2,
            )?,
            act_head: SparseSoftmax::fit(&rows, &act_y, ACTS.len(), dim, steps, rate, l2)?,
            lexicon,
            relations,
            trunk: norm,
        })
    }

    /// For a turn known to be a statement: `update` when the act head scores
    /// it above `assert`, otherwise `assert`.
    pub fn statement_act(&self, text: &str, dense: Option<&[f64]>) -> Result<&'static str> {
        let row = route_row(&self.lexicon, self.trunk.as_ref(), text, dense)?;
        // ACTS[0] is assert and ACTS[1] update.
        let (assert, update) = (0, 1);
        Ok(
            if self.act_head.logit(&row, update) > self.act_head.logit(&row, assert) {
                ACTS[update]
            } else {
                ACTS[assert]
            },
        )
    }

    /// Whether the route reads trunk features beside the words.
    pub fn needs_trunk(&self) -> bool {
        self.trunk.is_some()
    }

    /// The relation and act the table names for a user turn, given its trunk
    /// features exactly when the route was fitted with them.
    pub fn classify(&self, text: &str, dense: Option<&[f64]>) -> Result<(&str, &'static str)> {
        let row = route_row(&self.lexicon, self.trunk.as_ref(), text, dense)?;
        Ok((
            self.relations[self.relation_head.predict(&row)].as_str(),
            ACTS[self.act_head.predict(&row)],
        ))
    }

    /// The value the log gives for a relation query: the latest earlier user
    /// turn the table names as the asked relation, and its words outside the
    /// world's fixed vocabulary (`reserved`, instrument knowledge, as R-sieve's
    /// content cut). With `fact_acts` the turn must also be named an assert or
    /// update; without, the value's presence alone marks a statement (a query
    /// states no value). `None` when the query names no relation or no such
    /// turn holds a value.
    /// `trunk` gives a turn's trunk features (`None` for a route fitted
    /// without them).
    pub fn value(
        &self,
        history: &[Turn2],
        query: &Turn2,
        reserved: &std::collections::BTreeSet<String>,
        fact_acts: bool,
        trunk: &mut dyn FnMut(&str) -> Result<Option<Vec<f64>>>,
    ) -> Result<Option<String>> {
        let dense = trunk(&query.user)?;
        let (asked, _) = self.classify(&query.user, dense.as_deref())?;
        if asked == NONE {
            return Ok(None);
        }
        for turn in history.iter().rev() {
            let dense = trunk(&turn.user)?;
            let (relation, act) = self.classify(&turn.user, dense.as_deref())?;
            if relation != asked || (fact_acts && !matches!(act, "assert" | "update")) {
                continue;
            }
            let value: Vec<String> = words(&turn.user)
                .into_iter()
                .filter(|word| !reserved.contains(word))
                .collect();
            if !value.is_empty() {
                return Ok(Some(value.join(" ")));
            }
        }
        Ok(None)
    }
}

/// A word of a text and its half-open byte span in that text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordSpan {
    pub start: usize,
    pub end: usize,
    /// Lowercased, with curly apostrophes made straight.
    pub word: String,
}

fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '\u{2018}' | '\u{2019}')
}

/// The words of `text` with their byte spans: the same words, in the same
/// order, as the world's `words`, so a value can be cut from the unchanged
/// source.
pub fn word_spans(text: &str) -> Vec<WordSpan> {
    let mut spans = Vec::new();
    let mut push = |start: usize, end: usize| {
        let piece = &text[start..end];
        let lead = piece.len() - piece.trim_start_matches(is_apostrophe).len();
        let trail = piece.len() - piece.trim_end_matches(is_apostrophe).len();
        if lead + trail < piece.len() {
            let (start, end) = (start + lead, end - trail);
            spans.push(WordSpan {
                start,
                end,
                word: text[start..end]
                    .replace(['\u{2018}', '\u{2019}'], "'")
                    .to_lowercase(),
            });
        }
    };
    let mut open = None;
    for (i, c) in text.char_indices() {
        let part = c.is_alphanumeric() || is_apostrophe(c);
        match (part, open) {
            (true, None) => open = Some(i),
            (false, Some(start)) => {
                push(start, i);
                open = None;
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        push(start, text.len());
    }
    spans
}

/// The value-span head: which words of a statement are its value. Each word
/// is classified in or out of the value from its context, and the value is
/// the contiguous run of at most `max_words` words whose summed margin is
/// largest and positive; no such run means no write.
///
/// A word is seen only through the *frame* vocabulary: the training words
/// that never fall inside a value. Any other word, a value word included,
/// reads as `<v>`, so the head cannot recall a value it was trained on and
/// must find values from their context, shape and position.
pub struct SpanHead {
    frame: std::collections::BTreeSet<String>,
    index: BTreeMap<String, usize>,
    head: SparseSoftmax,
    max_words: usize,
}

/// The features of word `i` of `text`, by name.
fn span_features(
    frame: &std::collections::BTreeSet<String>,
    text: &str,
    words: &[WordSpan],
    i: usize,
) -> Vec<String> {
    let at = |j: isize| -> &str {
        match usize::try_from(j).ok().and_then(|j| words.get(j)) {
            None if j < 0 => "<s>",
            None => "</s>",
            Some(word) if frame.contains(&word.word) => &word.word,
            Some(_) => "<v>",
        }
    };
    let j = i as isize;
    let surface = &text[words[i].start..words[i].end];
    let shape = if surface.chars().all(|c| c.is_ascii_digit()) {
        "digit"
    } else if surface.chars().next().is_some_and(char::is_uppercase) {
        "cap"
    } else {
        "lower"
    };
    let position = if i == 0 {
        "first"
    } else if i + 1 == words.len() {
        "last"
    } else {
        "mid"
    };
    vec![
        format!("w0={}", at(j)),
        format!("l1={}", at(j - 1)),
        format!("l2={}", at(j - 2)),
        format!("r1={}", at(j + 1)),
        format!("r2={}", at(j + 2)),
        format!("l1r1={}|{}", at(j - 1), at(j + 1)),
        format!("shape={shape}|{position}"),
    ]
}

/// Whether a turn with these labels writes a value.
fn writes(relation: &str, act: &str) -> bool {
    relation != NONE && matches!(act, "assert" | "update")
}

impl SpanHead {
    /// Fit on labelled turns: a statement's slot words are in its value,
    /// every other word, and every word of a query or of a turn naming no
    /// relation, is out (the no-write credit). A statement whose slot cannot
    /// be recovered from its template gives no supervision and is skipped.
    pub fn fit(
        train: &[Example],
        steps: usize,
        rate: f64,
        l2: f64,
        max_words: usize,
    ) -> Result<Self> {
        if max_words == 0 {
            return Err(invalid("a value span needs at least one word"));
        }
        let mut labelled = Vec::new();
        for example in train {
            let slot = if writes(&example.relation, example.act) {
                match example.slot_span() {
                    Some(slot) => Some(slot),
                    None => continue,
                }
            } else {
                None
            };
            let words = word_spans(&example.text);
            let inside: Vec<bool> = words
                .iter()
                .map(|w| slot.is_some_and(|(s, e)| w.start >= s && w.end <= e))
                .collect();
            labelled.push((example.text.as_str(), words, inside));
        }
        let (mut outside, mut inside) = (
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        );
        for (_, words, marks) in &labelled {
            for (word, &mark) in words.iter().zip(marks) {
                if mark {
                    inside.insert(word.word.clone());
                } else {
                    outside.insert(word.word.clone());
                }
            }
        }
        let frame: std::collections::BTreeSet<String> =
            outside.difference(&inside).cloned().collect();
        let mut index = BTreeMap::new();
        let (mut rows, mut y) = (Vec::new(), Vec::new());
        for (text, words, marks) in &labelled {
            for (i, &mark) in marks.iter().enumerate() {
                let mut row: Vec<(usize, f64)> = span_features(&frame, text, words, i)
                    .into_iter()
                    .map(|name| {
                        let next = index.len();
                        (*index.entry(name).or_insert(next), 1.0)
                    })
                    .collect();
                row.sort_by_key(|&(i, _)| i);
                row.dedup_by_key(|&mut (i, _)| i);
                rows.push(row);
                y.push(usize::from(mark));
            }
        }
        let head = SparseSoftmax::fit(&rows, &y, 2, index.len().max(1), steps, rate, l2)?;
        Ok(Self {
            frame,
            index,
            head,
            max_words,
        })
    }

    fn row(&self, text: &str, words: &[WordSpan], i: usize) -> Vec<(usize, f64)> {
        let mut row: Vec<(usize, f64)> = span_features(&self.frame, text, words, i)
            .iter()
            .filter_map(|name| self.index.get(name).map(|&i| (i, 1.0)))
            .collect();
        row.sort_by_key(|&(i, _)| i);
        row.dedup_by_key(|&mut (i, _)| i);
        row
    }

    /// The value's half-open byte span in `text`, or `None` for no write.
    /// Ties keep the earlier, then the shorter, run.
    pub fn decode(&self, text: &str) -> Option<(usize, usize)> {
        let words = word_spans(text);
        let margins: Vec<f64> = (0..words.len())
            .map(|i| self.head.margin(&self.row(text, &words, i)))
            .collect();
        let mut best: Option<(f64, usize, usize)> = None;
        for a in 0..words.len() {
            let mut sum = 0.0;
            for (b, margin) in margins.iter().enumerate().skip(a).take(self.max_words) {
                sum += margin;
                if sum > 0.0 && best.is_none_or(|(top, _, _)| sum > top) {
                    best = Some((sum, a, b));
                }
            }
        }
        best.map(|(_, a, b)| (words[a].start, words[b].end))
    }
}

/// The fitting settings of a saved compiler.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompilerSettings {
    pub table_steps: usize,
    pub table_rate: f64,
    pub table_l2: f64,
    pub span_steps: usize,
    pub span_rate: f64,
    pub span_l2: f64,
    pub span_max_words: usize,
    pub act_rule: ActRule,
}

/// Which head decides whether a turn naming a relation is a statement or a
/// query.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActRule {
    /// The table's act head names assert, update, query or none.
    #[default]
    Table,
    /// A decoded value span makes the turn a statement, and the act head
    /// only chooses assert or update; no span makes it a query.
    Span,
}

impl ActRule {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "table" => Ok(Self::Table),
            "span" => Ok(Self::Span),
            other => Err(invalid(format!("unknown act rule {other}"))),
        }
    }
}

impl Default for CompilerSettings {
    /// The table's settings are `recall=route`'s (E4); the span head's are
    /// the same gradient settings, with values of at most four words.
    fn default() -> Self {
        Self {
            table_steps: 400,
            table_rate: 0.5,
            table_l2: 1e-4,
            span_steps: 400,
            span_rate: 0.5,
            span_l2: 1e-4,
            span_max_words: 4,
            act_rule: ActRule::Table,
        }
    }
}

pub const COMPILER_SCHEMA: &str = "uor-r4.relation-compiler/1";
pub const COMPILER_LABEL_SCHEMA: &str = "m-world-v2.relations-acts/1";
const COMPILER_FEATURES: &str = "table: words at standardized scale; span: frame words in a \
     two-word window, the l1|r1 pair, shape and position";

/// Serialize `f64`s by their bits, so a saved artifact reloads exactly.
mod f64_bits {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(values: &[f64], s: S) -> Result<S::Ok, S::Error> {
        values
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<u64>>()
            .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
        Ok(Vec::<u64>::deserialize(d)?
            .into_iter()
            .map(f64::from_bits)
            .collect())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HeadParts {
    classes: usize,
    dim: usize,
    #[serde(with = "f64_bits")]
    weights: Vec<f64>,
    #[serde(with = "f64_bits")]
    bias: Vec<f64>,
}

impl HeadParts {
    fn of(head: &SparseSoftmax) -> Self {
        Self {
            classes: head.classes,
            dim: head.dim,
            weights: head.weights.clone(),
            bias: head.bias.clone(),
        }
    }

    fn head(self, classes: usize, dim: usize) -> Result<SparseSoftmax> {
        if self.classes != classes
            || self.dim != dim
            || self.weights.len() != classes * dim
            || self.bias.len() != classes
            || self
                .weights
                .iter()
                .chain(&self.bias)
                .any(|v| !v.is_finite())
        {
            return Err(invalid(
                "a saved head has the wrong shape or a non-finite weight",
            ));
        }
        Ok(SparseSoftmax {
            classes,
            dim,
            weights: self.weights,
            bias: self.bias,
        })
    }
}

/// The saved artifact. Field order is the encoding; [`SavedCompiler`] loads
/// only its own canonical bytes.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CompilerArtifact {
    schema: String,
    label_schema: String,
    features: String,
    /// The tokenizer whose token counts metered the training draws, and the
    /// one the session's emitter must use.
    tokenizer_sha256: String,
    /// The table's relation labels in class order, [`NONE`] last.
    relations: Vec<String>,
    acts: Vec<String>,
    /// The table's vocabulary in index order, and each word's scale.
    words: Vec<String>,
    #[serde(with = "f64_bits")]
    word_scale: Vec<f64>,
    relation_head: HeadParts,
    act_head: HeadParts,
    /// The span head's frame vocabulary, sorted, and features in index order.
    span_frame: Vec<String>,
    span_features: Vec<String>,
    span_head: HeadParts,
    span_max_words: usize,
    table_steps: usize,
    #[serde(with = "f64_bits")]
    table_rate_l2: Vec<f64>,
    span_steps: usize,
    #[serde(with = "f64_bits")]
    span_rate_l2: Vec<f64>,
    /// Where the training turns came from (draws, files and digests).
    training: Value,
    /// `span` for [`ActRule::Span`]; absent for [`ActRule::Table`], so
    /// artifacts saved before the rule existed keep their bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    act_rule: Option<String>,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A fitted relation/act table and value-span head, loaded from its saved
/// bytes: the compiler of the grounded session (#962, D19). It predicts
/// from a turn's text alone; it never sees an evaluator label.
#[derive(Clone)]
pub struct SavedCompiler {
    bytes: Vec<u8>,
    inner: std::sync::Arc<(RelationRoute, SpanHead)>,
    act_rule: ActRule,
    identity: crate::stack_grounded_session::CompilerIdentity,
}

impl SavedCompiler {
    /// Fit the table and the span head on `train` and save them, with
    /// `training`, the provenance of `train`, as the artifact. The returned
    /// compiler is loaded back from those bytes.
    pub fn fit(
        train: &[Example],
        tokenizer_sha256: &str,
        training: Value,
        settings: CompilerSettings,
    ) -> Result<Self> {
        let route = RelationRoute::fit(
            train,
            settings.table_steps,
            settings.table_rate,
            settings.table_l2,
        )?;
        let span = SpanHead::fit(
            train,
            settings.span_steps,
            settings.span_rate,
            settings.span_l2,
            settings.span_max_words,
        )?;
        let bytes = encode(&route, &span, tokenizer_sha256, training, settings)?;
        Self::from_bytes(bytes)
    }

    /// Load a saved compiler. Only canonical bytes of this schema load.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let artifact: CompilerArtifact = serde_json::from_slice(&bytes)?;
        if artifact.schema != COMPILER_SCHEMA
            || artifact.label_schema != COMPILER_LABEL_SCHEMA
            || artifact.features != COMPILER_FEATURES
        {
            return Err(invalid("not a relation compiler of this schema"));
        }
        if !is_sha256(&artifact.tokenizer_sha256) {
            return Err(invalid("the compiler's tokenizer digest is malformed"));
        }
        let distinct = |names: &[String]| {
            names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == names.len()
                && names.iter().all(|name| !name.is_empty())
        };
        if artifact.relations.len() < 2
            || artifact.relations.last().map(String::as_str) != Some(NONE)
            || !distinct(&artifact.relations)
            || artifact.acts != ACTS
            || !distinct(&artifact.words)
            || artifact.word_scale.len() != artifact.words.len()
            || artifact
                .word_scale
                .iter()
                .any(|s| !s.is_finite() || *s <= 0.0)
            || !distinct(&artifact.span_features)
            || !artifact.span_frame.windows(2).all(|w| w[0] < w[1])
            || artifact.span_max_words == 0
            || artifact.table_rate_l2.len() != 2
            || artifact.span_rate_l2.len() != 2
        {
            return Err(invalid(
                "a saved compiler's labels or vocabularies are malformed",
            ));
        }
        let settings = CompilerSettings {
            table_steps: artifact.table_steps,
            table_rate: artifact.table_rate_l2[0],
            table_l2: artifact.table_rate_l2[1],
            span_steps: artifact.span_steps,
            span_rate: artifact.span_rate_l2[0],
            span_l2: artifact.span_rate_l2[1],
            span_max_words: artifact.span_max_words,
            act_rule: match artifact.act_rule.as_deref() {
                None => ActRule::Table,
                Some("span") => ActRule::Span,
                Some(_) => return Err(invalid("a saved compiler names an unknown act rule")),
            },
        };
        let table_dim = artifact.words.len().max(1);
        let lexicon = Lexicon {
            index: artifact
                .words
                .iter()
                .enumerate()
                .map(|(i, w)| (w.clone(), i))
                .collect(),
            scale: artifact.word_scale,
        };
        let route = RelationRoute {
            lexicon,
            relation_head: artifact
                .relation_head
                .head(artifact.relations.len(), table_dim)?,
            act_head: artifact.act_head.head(ACTS.len(), table_dim)?,
            relations: artifact.relations,
            trunk: None,
        };
        let span = SpanHead {
            frame: artifact.span_frame.into_iter().collect(),
            index: artifact
                .span_features
                .iter()
                .enumerate()
                .map(|(i, f)| (f.clone(), i))
                .collect(),
            head: artifact
                .span_head
                .head(2, artifact.span_features.len().max(1))?,
            max_words: artifact.span_max_words,
        };
        if encode(
            &route,
            &span,
            &artifact.tokenizer_sha256,
            artifact.training.clone(),
            settings,
        )? != bytes
        {
            return Err(invalid("a saved compiler's bytes are not canonical"));
        }
        let relations = route
            .relations
            .iter()
            .filter(|name| name.as_str() != NONE)
            .enumerate()
            .map(|(i, name)| {
                Ok(crate::stack_grounded_session::RelationLabel {
                    id: u32::try_from(i + 1).map_err(|_| invalid("too many relations"))?,
                    name: name.clone(),
                })
            })
            .collect::<Result<_>>()?;
        let identity = crate::stack_grounded_session::CompilerIdentity {
            schema: COMPILER_SCHEMA.to_owned(),
            artifact_sha256: uor_r4_core::native_geometric::learner::realtext_support::sha256_hex(
                &bytes,
            ),
            tokenizer_sha256: artifact.tokenizer_sha256,
            label_schema: COMPILER_LABEL_SCHEMA.to_owned(),
            relations,
            encoder: None,
        };
        Ok(Self {
            bytes,
            inner: std::sync::Arc::new((route, span)),
            act_rule: settings.act_rule,
            identity,
        })
    }

    pub fn act_rule(&self) -> ActRule {
        self.act_rule
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn route(&self) -> &RelationRoute {
        &self.inner.0
    }

    pub fn span(&self) -> &SpanHead {
        &self.inner.1
    }

    /// The store relation ID of a relation label: its 1-based position
    /// among the table's relations, [`NONE`] excluded.
    pub fn relation_id(&self, name: &str) -> Option<u32> {
        self.identity
            .relations
            .iter()
            .find(|label| label.name == name)
            .map(|label| label.id)
    }

    /// The action for a user turn: a query of the named relation; a
    /// statement or correction with the span head's value; or unresolved
    /// when the table names no relation. Under [`ActRule::Table`] a turn is
    /// also unresolved when the table names no act, or names a statement
    /// whose value the span head does not mark.
    pub fn action(&self, source: &str) -> Result<crate::stack_grounded_session::CompiledAction> {
        use crate::stack_grounded_session::{CompiledAction, SourceSpan};
        let unresolved = |reason: &str| {
            Ok(CompiledAction::Unresolved {
                reason: reason.to_owned(),
            })
        };
        let (relation, act) = self.route().classify(source, None)?;
        if relation == NONE {
            return unresolved("the table names no relation");
        }
        let id = self
            .relation_id(relation)
            .ok_or_else(|| invalid("the table named a relation outside its labels"))?;
        if self.act_rule == ActRule::Span {
            return Ok(match self.span().decode(source) {
                None => CompiledAction::QueryCurrent { relation: id },
                Some((start, end)) => {
                    let span = SourceSpan { start, end };
                    if self.route().statement_act(source, None)? == "update" {
                        CompiledAction::Correct { relation: id, span }
                    } else {
                        CompiledAction::Assert { relation: id, span }
                    }
                }
            });
        }
        match act {
            "query" => Ok(CompiledAction::QueryCurrent { relation: id }),
            "assert" | "update" => match self.span().decode(source) {
                None => unresolved("the span head marks no value"),
                Some((start, end)) => {
                    let span = SourceSpan { start, end };
                    Ok(if act == "assert" {
                        CompiledAction::Assert { relation: id, span }
                    } else {
                        CompiledAction::Correct { relation: id, span }
                    })
                }
            },
            _ => unresolved("the table names no act"),
        }
    }
}

fn encode(
    route: &RelationRoute,
    span: &SpanHead,
    tokenizer_sha256: &str,
    training: Value,
    settings: CompilerSettings,
) -> Result<Vec<u8>> {
    if route.trunk.is_some() {
        return Err(invalid("a saved compiler has no trunk features"));
    }
    // JSON floats need not reload to the same bits; only the f64 fields
    // above are stored by their bits.
    fn has_float(value: &Value) -> bool {
        match value {
            Value::Number(n) => n.is_f64(),
            Value::Array(items) => items.iter().any(has_float),
            Value::Object(map) => map.values().any(has_float),
            _ => false,
        }
    }
    if has_float(&training) {
        return Err(invalid(
            "a compiler's training record holds integers and strings only",
        ));
    }
    let mut words: Vec<(usize, &String)> =
        route.lexicon.index.iter().map(|(w, &i)| (i, w)).collect();
    words.sort();
    let mut features: Vec<(usize, &String)> = span.index.iter().map(|(f, &i)| (i, f)).collect();
    features.sort();
    let artifact = CompilerArtifact {
        schema: COMPILER_SCHEMA.to_owned(),
        label_schema: COMPILER_LABEL_SCHEMA.to_owned(),
        features: COMPILER_FEATURES.to_owned(),
        tokenizer_sha256: tokenizer_sha256.to_owned(),
        relations: route.relations.clone(),
        acts: ACTS.iter().map(|a| (*a).to_owned()).collect(),
        words: words.into_iter().map(|(_, w)| w.clone()).collect(),
        word_scale: route.lexicon.scale.clone(),
        relation_head: HeadParts::of(&route.relation_head),
        act_head: HeadParts::of(&route.act_head),
        span_frame: span.frame.iter().cloned().collect(),
        span_features: features.into_iter().map(|(_, f)| f.clone()).collect(),
        span_head: HeadParts::of(&span.head),
        span_max_words: span.max_words,
        table_steps: settings.table_steps,
        table_rate_l2: vec![settings.table_rate, settings.table_l2],
        span_steps: settings.span_steps,
        span_rate_l2: vec![settings.span_rate, settings.span_l2],
        training,
        act_rule: match settings.act_rule {
            ActRule::Table => None,
            ActRule::Span => Some("span".to_owned()),
        },
    };
    Ok(serde_json::to_vec(&artifact)?)
}

impl crate::stack_grounded_session::TurnCompiler for SavedCompiler {
    fn identity(&self) -> &crate::stack_grounded_session::CompilerIdentity {
        &self.identity
    }

    fn artifact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn compile(
        &self,
        source: &str,
    ) -> std::result::Result<
        crate::stack_grounded_session::CompiledAction,
        crate::stack_grounded_session::GroundedSessionError,
    > {
        self.action(source).map_err(|e| {
            crate::stack_grounded_session::GroundedSessionError::Compiler(e.to_string())
        })
    }
}

/// Accuracy of `predicted` against `truth` over the rows `keep` selects,
/// with per-label tallies and the most frequent confusions.
pub fn score(
    labels: &[String],
    truth: &[usize],
    predicted: &[usize],
    keep: impl Fn(usize) -> bool,
) -> Value {
    let (mut pass, mut of) = (0usize, 0usize);
    let mut per: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut confusions: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for (&t, &p) in truth.iter().zip(predicted) {
        if !keep(t) {
            continue;
        }
        of += 1;
        let tally = per.entry(labels[t].as_str()).or_default();
        tally.1 += 1;
        if t == p {
            pass += 1;
            tally.0 += 1;
        } else {
            *confusions
                .entry((labels[t].as_str(), labels[p].as_str()))
                .or_default() += 1;
        }
    }
    let mut top: Vec<((&str, &str), usize)> = confusions.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(8);
    json!({
        "pass": pass,
        "of": of,
        "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 },
        "per_label": per
            .iter()
            .map(|(label, (p, o))| (label.to_string(), json!({"pass": p, "of": o})))
            .collect::<BTreeMap<_, _>>(),
        "top_confusions": top
            .iter()
            .map(|((t, p), n)| json!({"truth": t, "predicted": p, "count": n}))
            .collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::milestone_world_v2::{Category2, Tag};

    fn turn(intent: &str) -> Turn2 {
        Turn2 {
            intent: intent.into(),
            category: Category2::Relation,
            user: String::new(),
            reply: String::new(),
            checks: Vec::new(),
            tag: Tag::default(),
        }
    }

    #[test]
    fn turns_are_labelled_from_their_typed_intent() {
        assert_eq!(
            label(&turn("friend_name_assert")),
            ("friend_name".into(), "assert")
        );
        assert_eq!(
            label(&turn("user_name_update")),
            ("user_name".into(), "update")
        );
        assert_eq!(
            label(&turn("lucky_number_query")),
            ("lucky_number".into(), "query")
        );
        // A query of a relation never stated is still a query of it.
        assert_eq!(
            label(&turn("hometown_absent")),
            ("hometown".into(), "query")
        );
        for other in ["mqar_query", "mqar_assert", "copy", "capital", "thanks"] {
            assert_eq!(label(&turn(other)), (NONE.into(), NONE), "{other}");
        }
        assert!(relation_names().len() >= 10);
    }

    #[test]
    fn the_softmax_fit_separates_classes_and_is_deterministic() -> Result<()> {
        let x: Vec<Vec<f64>> = (0..30)
            .map(|i| {
                let class = i % 3;
                vec![class as f64 + 0.1 * (i % 5) as f64, (i % 7) as f64]
            })
            .collect();
        let y: Vec<usize> = (0..30).map(|i| i % 3).collect();
        let a = Softmax::fit(&x, &y, 3, 400, 0.5, 1e-4)?;
        let b = Softmax::fit(&x, &y, 3, 400, 0.5, 1e-4)?;
        let predicted: Vec<usize> = x.iter().map(|row| a.predict(row)).collect();
        assert_eq!(
            predicted,
            x.iter().map(|row| b.predict(row)).collect::<Vec<_>>()
        );
        assert_eq!(predicted, y);
        assert!(Softmax::fit(&x, &y[..29], 3, 1, 0.5, 0.0).is_err());
        let labels: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let report = score(&labels, &y, &predicted, |label| label != 2);
        assert_eq!(report["of"], 20);
        assert_eq!(report["rate"], 1.0);
        Ok(())
    }

    #[test]
    fn paraphrases_are_filled_with_the_relations_training_values() -> Result<()> {
        let example = |text: &str, relation: &str, act, template: &str| Example {
            text: text.into(),
            relation: relation.into(),
            act,
            template: Some(template.into()),
        };
        let training = [
            example(
                "My friend is Sam.",
                "friend_name",
                "assert",
                "My friend is {v}.",
            ),
            example(
                "Tam is my friend.",
                "friend_name",
                "assert",
                "{v} is my friend.",
            ),
            example(
                "Who is my friend?",
                "friend_name",
                "query",
                "Who is my friend?",
            ),
        ];
        assert_eq!(training[0].slot_value(), Some("Sam"));
        assert_eq!(training[1].slot_value(), Some("Tam"));
        assert_eq!(training[2].slot_value(), None);
        let jsonl = concat!(
            r#"{"relation":"friend_name","act":"assert","text":"My buddy is {v}."}"#,
            "\n",
            r#"{"relation":"friend_name","act":"assert","text":"{v} is a pal of mine."}"#,
            "\n",
            r#"{"relation":"friend_name","act":"query","text":"Who's my pal?"}"#,
            "\n",
            r#"{"relation":"hometown","act":"assert","text":"I grew up in {v}."}"#,
            "\n",
        );
        let (examples, skipped) = paraphrase_examples(jsonl, &training)?;
        // Values are used in turn; a relation without one is skipped.
        assert_eq!(skipped, 1);
        let texts: Vec<&str> = examples.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(
            texts,
            ["My buddy is Sam.", "Tam is a pal of mine.", "Who's my pal?"]
        );
        assert_eq!(examples[2].act, "query");
        assert!(
            paraphrase_examples(r#"{"relation":"nope","act":"query","text":"x"}"#, &training)
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn the_relation_route_finds_the_latest_statement_of_the_asked_relation() -> Result<()> {
        let example = |text: &str, relation: &str, act| Example {
            text: text.into(),
            relation: relation.into(),
            act,
            template: None,
        };
        let train = [
            example("My friend is Sam.", "friend_name", "assert"),
            example("Who is my friend?", "friend_name", "query"),
            example("My dog is Rex.", "pet_name", "assert"),
            example("What is my dog called?", "pet_name", "query"),
            example("The weather is nice.", NONE, NONE),
        ];
        let route = RelationRoute::fit(&train, 300, 0.5, 1e-4)?;
        assert!(!route.needs_trunk());
        assert_eq!(
            route.classify("Who is my friend?", None)?,
            ("friend_name", "query")
        );
        assert_eq!(
            route.classify("My dog is Max.", None)?,
            ("pet_name", "assert")
        );
        // A route without trunk features refuses them, and one with them
        // refuses a turn without them.
        assert!(route.classify("Who is my friend?", Some(&[0.5])).is_err());
        let trunk: Vec<Vec<f64>> = (0..train.len()).map(|i| vec![i as f64, 1.0]).collect();
        let with = RelationRoute::fit_with(&train, Some(&trunk), 300, 0.5, 1e-4)?;
        assert!(with.needs_trunk());
        assert!(with.classify("Who is my friend?", None).is_err());
        assert!(with
            .classify("Who is my friend?", Some(&[1.0, 1.0]))
            .is_ok());
        let mut none = |_: &str| -> Result<Option<Vec<f64>>> { Ok(None) };
        let user = |text: &str| Turn2 {
            intent: String::new(),
            category: Category2::Relation,
            user: text.into(),
            reply: String::new(),
            checks: Vec::new(),
            tag: Tag::default(),
        };
        let history = [
            user("My friend is Zorvik."),
            user("My dog is Plimbo."),
            user("My friend is Quandle."),
        ];
        // Words outside the fixed vocabulary are the value; the latest wins.
        let reserved: std::collections::BTreeSet<String> =
            ["my", "friend", "is", "dog", "who", "what", "called"]
                .iter()
                .map(|w| (*w).to_owned())
                .collect();
        assert_eq!(
            route
                .value(
                    &history,
                    &user("Who is my friend?"),
                    &reserved,
                    true,
                    &mut none
                )?
                .as_deref(),
            Some("quandle")
        );
        assert_eq!(
            route
                .value(
                    &history,
                    &user("What is my dog called?"),
                    &reserved,
                    false,
                    &mut none
                )?
                .as_deref(),
            Some("plimbo")
        );
        assert_eq!(
            route.value(&[], &user("Who is my friend?"), &reserved, true, &mut none)?,
            None
        );
        // The sparse fit refuses an index outside its width.
        assert!(SparseSoftmax::fit(&[vec![(3, 1.0)]], &[0], 2, 3, 1, 0.5, 0.0).is_err());
        Ok(())
    }

    #[test]
    fn word_spans_are_the_worlds_words_cut_from_the_source() {
        for text in [
            "My name is Zorvak.",
            "  'Tis Ana’s dog, called ‘Rex’ -- or O'Neil?",
            "Lucky 42, code-word: bliv.",
            "''",
            "Ünïcode Ök, naïve café",
        ] {
            let spans = word_spans(text);
            assert_eq!(
                spans.iter().map(|s| s.word.clone()).collect::<Vec<_>>(),
                words(text),
                "{text}"
            );
            for span in &spans {
                assert!(text.is_char_boundary(span.start) && text.is_char_boundary(span.end));
                assert_eq!(
                    text[span.start..span.end]
                        .replace(['\u{2018}', '\u{2019}'], "'")
                        .to_lowercase(),
                    span.word
                );
            }
        }
        assert_eq!(
            word_spans("call me Kel.")[2],
            WordSpan {
                start: 8,
                end: 11,
                word: "kel".into()
            }
        );
    }

    fn templated(template: &str, value: &str, relation: &str, act: &'static str) -> Example {
        Example {
            text: template.replace("{v}", value),
            relation: relation.into(),
            act,
            template: Some(template.into()),
        }
    }

    fn small_world() -> Vec<Example> {
        let mut train = Vec::new();
        let names = ["Sam", "Tam", "Rook", "Vel", "Bram", "Ilo"];
        let towns = ["Hobton", "Marsk", "Pelford", "Quill"];
        for (i, name) in names.iter().enumerate() {
            train.push(templated("My name is {v}.", name, "user_name", "assert"));
            train.push(templated(
                "Actually, my name is {v}.",
                names[(i + 1) % names.len()],
                "user_name",
                "update",
            ));
            train.push(templated(
                "Call me {v}, please.",
                name,
                "user_name",
                "assert",
            ));
            train.push(templated("What is my name?", name, "user_name", "query"));
            train.push(templated("The weather is nice today.", name, NONE, NONE));
        }
        for (i, town) in towns.iter().enumerate() {
            train.push(templated("I grew up in {v}.", town, "hometown", "assert"));
            train.push(templated(
                "Sorry, I grew up in {v}.",
                towns[(i + 1) % towns.len()],
                "hometown",
                "update",
            ));
            train.push(templated("Where am I from?", town, "hometown", "query"));
        }
        train
    }

    #[test]
    fn the_span_head_finds_unseen_values_from_context_and_marks_no_write() -> Result<()> {
        let train = small_world();
        assert_eq!(train[0].slot_span(), Some((11, 14)));
        assert_eq!(train[3].slot_span(), None);
        let head = SpanHead::fit(&train, 300, 0.5, 1e-4, 4)?;
        // Training values never enter the frame vocabulary.
        assert!(!head.frame.contains("sam") && head.frame.contains("name"));
        for (text, value) in [
            ("My name is Zorvak.", Some("Zorvak")),
            ("Actually, my name is Plimbo.", Some("Plimbo")),
            ("I grew up in Dunmere.", Some("Dunmere")),
            ("What is my name?", None),
            ("Where am I from?", None),
            ("The weather is nice today.", None),
        ] {
            let decoded = head.decode(text).map(|(s, e)| &text[s..e]);
            assert_eq!(decoded, value, "{text}");
        }
        assert!(SpanHead::fit(&train, 1, 0.5, 0.0, 0).is_err());
        Ok(())
    }

    #[test]
    fn a_saved_compiler_reloads_exactly_and_compiles_turns() -> Result<()> {
        use crate::stack_grounded_session::{CompiledAction, SourceSpan, TurnCompiler};
        let train = small_world();
        let tokenizer = "ab".repeat(32);
        let training = json!({"draw": "small world", "turns": train.len()});
        let saved = SavedCompiler::fit(
            &train,
            &tokenizer,
            training.clone(),
            CompilerSettings::default(),
        )?;
        let again = SavedCompiler::from_bytes(saved.bytes().to_vec())?;
        assert_eq!(again.bytes(), saved.bytes());
        assert_eq!(again.identity(), saved.identity());
        // A refit from the same turns gives the same bytes.
        let refit = SavedCompiler::fit(&train, &tokenizer, training, CompilerSettings::default())?;
        assert_eq!(refit.bytes(), saved.bytes());
        let identity = saved.identity();
        assert_eq!(identity.tokenizer_sha256, tokenizer);
        assert_eq!(identity.relations.len(), relation_names().len());
        assert_eq!(identity.relations[0].id, 1);
        let name = saved
            .relation_id("user_name")
            .ok_or_else(|| invalid("no id"))?;
        let town = saved
            .relation_id("hometown")
            .ok_or_else(|| invalid("no id"))?;
        let compile = |text: &str| saved.compile(text).map_err(|e| invalid(e.to_string()));
        assert_eq!(
            compile("My name is Zorvak.")?,
            CompiledAction::Assert {
                relation: name,
                span: SourceSpan { start: 11, end: 17 }
            }
        );
        assert_eq!(
            compile("Sorry, I grew up in Dunmere.")?,
            CompiledAction::Correct {
                relation: town,
                span: SourceSpan { start: 20, end: 27 }
            }
        );
        assert_eq!(
            compile("Where am I from?")?,
            CompiledAction::QueryCurrent { relation: town }
        );
        assert!(matches!(
            compile("The weather is nice today.")?,
            CompiledAction::Unresolved { .. }
        ));
        // Only canonical bytes of this schema load.
        let mut pretty: Value = serde_json::from_slice(saved.bytes())?;
        assert!(SavedCompiler::from_bytes(serde_json::to_vec_pretty(&pretty)?).is_err());
        pretty["acts"][0] = json!("tell");
        assert!(SavedCompiler::from_bytes(serde_json::to_vec(&pretty)?).is_err());
        assert!(SavedCompiler::fit(
            &train,
            &tokenizer,
            json!({"rate": 0.5}),
            CompilerSettings::default()
        )
        .is_err());
        // The table rule writes no act-rule field, so its bytes predate it.
        assert!(!String::from_utf8_lossy(saved.bytes()).contains("act_rule"));
        Ok(())
    }

    #[test]
    fn under_the_span_rule_a_value_makes_a_statement_and_none_a_query() -> Result<()> {
        use crate::stack_grounded_session::{CompiledAction, SourceSpan, TurnCompiler};
        let train = small_world();
        let settings = CompilerSettings {
            act_rule: ActRule::Span,
            ..CompilerSettings::default()
        };
        let saved = SavedCompiler::fit(&train, &"cd".repeat(32), json!({}), settings)?;
        assert_eq!(saved.act_rule(), ActRule::Span);
        let again = SavedCompiler::from_bytes(saved.bytes().to_vec())?;
        assert_eq!(again.act_rule(), ActRule::Span);
        assert_eq!(again.bytes(), saved.bytes());
        let name = saved
            .relation_id("user_name")
            .ok_or_else(|| invalid("no id"))?;
        let compile = |text: &str| saved.compile(text).map_err(|e| invalid(e.to_string()));
        assert_eq!(
            compile("My name is Zorvak.")?,
            CompiledAction::Assert {
                relation: name,
                span: SourceSpan { start: 11, end: 17 }
            }
        );
        assert_eq!(
            compile("Actually, my name is Plimbo.")?,
            CompiledAction::Correct {
                relation: name,
                span: SourceSpan { start: 21, end: 27 }
            }
        );
        assert_eq!(
            compile("What is my name?")?,
            CompiledAction::QueryCurrent { relation: name }
        );
        let mut artifact: Value = serde_json::from_slice(saved.bytes())?;
        assert_eq!(artifact["act_rule"], "span");
        artifact["act_rule"] = json!("guess");
        assert!(SavedCompiler::from_bytes(serde_json::to_vec(&artifact)?).is_err());
        assert!(ActRule::parse("span").is_ok() && ActRule::parse("both").is_err());
        Ok(())
    }

    #[test]
    fn the_lexicon_ignores_words_it_never_saw() {
        let lexicon = Lexicon::fit(["My friend is Sam.", "What is my name?"]);
        assert_eq!(lexicon.len(), 6);
        let features = lexicon.features("My buddy is Tam.");
        assert_eq!(features.iter().filter(|&&v| v == 1.0).count(), 2);
    }
}
