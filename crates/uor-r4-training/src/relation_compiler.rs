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
    /// The phrasing template the turn was drawn from, slot unfilled.
    pub template: Option<String>,
}

impl Example {
    /// The value filling the template's `{v}` slot in the text, if the turn
    /// has a one-slot template that the text matches.
    pub fn slot_value(&self) -> Option<&str> {
        let template = self.template.as_deref()?;
        let (before, after) = template.split_once("{v}")?;
        let value = self.text.strip_prefix(before)?.strip_suffix(after)?;
        (!value.trim().is_empty()).then_some(value)
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
        let text = if text.contains("{v}") {
            let Some(pool) = values.get(relation).filter(|pool| !pool.is_empty()) else {
                skipped += 1;
                continue;
            };
            let i = next.entry(relation.to_owned()).or_default();
            let value = pool[*i % pool.len()];
            *i += 1;
            text.replace("{v}", value)
        } else {
            text.to_owned()
        };
        examples.push(Example {
            text,
            relation: relation.to_owned(),
            act,
            template: None,
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
    fn the_lexicon_ignores_words_it_never_saw() {
        let lexicon = Lexicon::fit(["My friend is Sam.", "What is my name?"]);
        assert_eq!(lexicon.len(), 6);
        let features = lexicon.features("My buddy is Tam.");
        assert_eq!(features.iter().filter(|&&v| v == 1.0).count(), 2);
    }
}
