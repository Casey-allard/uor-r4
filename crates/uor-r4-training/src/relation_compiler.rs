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

/// A frozen stack read as the compiler's encoder: its [`trunk_features`] of
/// a user turn read alone, bound to its saved files by digest.
pub struct Trunk {
    model: StackModel,
    tokenizer: uor_r4_tokenizer::ByteBpeTokenizer,
    protocol: uor_r4_tokenizer::dialogue::DialogueProtocol,
    tokenizer_sha256: String,
    identity: crate::stack_grounded_session::EncoderIdentity,
}

impl Trunk {
    /// Load the stack saved in `directory` (with its transport snap) and the
    /// tokenizer it reads turns with.
    pub fn load(
        directory: &std::path::Path,
        tokenizer_json: &[u8],
        device: &candle_core::Device,
    ) -> Result<Self> {
        if StackModel::saved_served_representation(directory)?.is_some() {
            return Err(invalid("a trunk is read in its float form, not served"));
        }
        let mut model = StackModel::load(directory, device)?;
        model.set_transport_snap(StackModel::saved_transport_snap(directory)?)?;
        let digest = |name: &str| -> Result<Option<String>> {
            let path = directory.join(name);
            path.exists().then(|| crate::sha256_file(&path)).transpose()
        };
        let identity = crate::stack_grounded_session::EncoderIdentity {
            config_sha256: digest("config.json")?
                .ok_or_else(|| invalid("a trunk needs config.json"))?,
            model_sha256: digest("model.safetensors")?
                .ok_or_else(|| invalid("a trunk needs model.safetensors"))?,
            transport_sha256: digest("transport.json")?,
        };
        let tokenizer =
            uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json)
                .ok_or_else(|| invalid("unreadable tokenizer.json"))?;
        if tokenizer.vocab_size() != model.config.vocab_size {
            return Err(invalid("the trunk and tokenizer vocabularies differ"));
        }
        let protocol = uor_r4_tokenizer::dialogue::DialogueProtocol::literal_roles_v1(&tokenizer)
            .map_err(|e| invalid(format!("protocol: {e}")))?;
        Ok(Self {
            model,
            tokenizer,
            protocol,
            tokenizer_sha256: uor_r4_core::native_geometric::learner::realtext_support::sha256_hex(
                tokenizer_json,
            ),
            identity,
        })
    }

    pub fn identity(&self) -> &crate::stack_grounded_session::EncoderIdentity {
        &self.identity
    }

    /// The trunk's features of a user turn read alone.
    pub fn features(&self, text: &str) -> Result<Vec<f64>> {
        let encoder = self
            .protocol
            .bind(&self.tokenizer)
            .map_err(|e| invalid(format!("protocol: {e}")))?;
        trunk_features(&self.model, &encoder, self.protocol.bos_id, text)
    }
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

    /// The logits of a raw feature row.
    fn logits(&self, row: &[f64]) -> Vec<f64> {
        let row = self.standardize(row);
        (0..self.classes)
            .map(|c| {
                self.bias[c]
                    + self.weights[c * self.dim..(c + 1) * self.dim]
                        .iter()
                        .zip(&row)
                        .map(|(w, v)| w * v)
                        .sum::<f64>()
            })
            .collect()
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

    pub(crate) fn probabilities(&self, row: &[(usize, f64)]) -> Vec<f64> {
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
    pub relation_mode: RelationMode,
}

/// Which heads name a turn's relation and act.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RelationMode {
    /// The sparse word table.
    #[default]
    Table,
    /// Dense softmax heads over a frozen [`Trunk`]'s features of the turn
    /// and its binary words, standardized (E3's `combined` head). The
    /// compiler then loads only with that trunk.
    Combined,
    /// The combined relation head names the relation; the table's act head
    /// keeps the act (and, under [`ActRule::Span`], assert vs update).
    CombinedRelation,
    /// A stack fine-tuned on `compile-corpus` documents generates each
    /// turn's op (`Op: assert user_name Ada`), and the value is located in
    /// the turn's own text ([`parse_op`]). The table and span head are
    /// saved alongside but not consulted.
    OpModel,
}

impl RelationMode {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "table" => Ok(Self::Table),
            "combined" => Ok(Self::Combined),
            "combined_relation" => Ok(Self::CombinedRelation),
            "op_model" => Ok(Self::OpModel),
            other => Err(invalid(format!("unknown relation mode {other}"))),
        }
    }
}

/// How a compiler with an op model combines it with its saved table (the
/// table and span head an op-model artifact carries).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpPolicy {
    /// The op model alone (the artifact's own mode).
    #[default]
    Op,
    /// The table alone; the op model is not consulted.
    Table,
    /// The table decides whether a turn is a statement, a query or neither;
    /// for a statement the op model's statement (relation and value) is
    /// used when it gives one.
    TableStatements,
    /// The op model's statement is used unless the table reads a query.
    UnlessQuery,
}

impl OpPolicy {
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "op" => Ok(Self::Op),
            "table" => Ok(Self::Table),
            "table_statements" => Ok(Self::TableStatements),
            "unless_query" => Ok(Self::UnlessQuery),
            other => Err(invalid(format!(
                "unknown op policy {other} (op, table, table_statements or unless_query)"
            ))),
        }
    }
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
            relation_mode: RelationMode::Table,
        }
    }
}

/// The system line that asks a stack to compile the next user turn.
pub const COMPILE_PROMPT: &str = "Compile.";
/// The longest op an op model may generate, in tokens.
const OP_MAX_TOKENS: usize = 24;

/// The op a labelled turn compiles to: `Op: none`, `Op: query <relation>`, or
/// `Op: assert|update <relation> <value>` with the template's slot value.
/// `None` for a statement whose slot cannot be recovered.
pub fn op_text(example: &Example) -> Option<String> {
    if example.relation == NONE || example.act == NONE {
        return Some("Op: none".to_owned());
    }
    match example.act {
        "query" => Some(format!("Op: query {}", example.relation)),
        act => example
            .slot_value()
            .map(|value| format!("Op: {act} {} {}", example.relation, value.trim())),
    }
}

/// The action of a generated op for `source`. A statement's value must occur
/// in the source (exactly, else ignoring ASCII case), and its span is that
/// occurrence; a statement whose value does not occur is a query of its
/// relation (the turn holds no value to store); anything else is unresolved
/// with a reason. `relation_id` maps
/// a relation name to its store ID.
pub fn parse_op(
    text: &str,
    source: &str,
    relation_id: impl Fn(&str) -> Option<u32>,
) -> crate::stack_grounded_session::CompiledAction {
    use crate::stack_grounded_session::{CompiledAction, SourceSpan};
    let unresolved = |reason: &str| CompiledAction::Unresolved {
        reason: reason.to_owned(),
    };
    let Some(rest) = text.trim().strip_prefix("Op:") else {
        return unresolved("the model produced no op");
    };
    let mut parts = rest.trim().splitn(3, ' ');
    let (act, relation, value) = (parts.next(), parts.next(), parts.next());
    let id = |name: &str| relation_id(name);
    match (act, relation, value) {
        (Some("none"), None, None) => unresolved("the op is none"),
        (Some("query"), Some(name), None) => match id(name) {
            Some(relation) => CompiledAction::QueryCurrent { relation },
            None => unresolved("the op names an unknown relation"),
        },
        (Some(act @ ("assert" | "update")), Some(name), Some(value)) => {
            let Some(relation) = id(name) else {
                return unresolved("the op names an unknown relation");
            };
            let value = value.trim().trim_end_matches(['.', '!', '?', ',']).trim();
            if value.is_empty() {
                return unresolved("the op has an empty value");
            }
            let start = source.find(value).or_else(|| {
                source
                    .to_ascii_lowercase()
                    .find(&value.to_ascii_lowercase())
            });
            // A turn that names a relation but holds no value for it cannot
            // be stored; it asks for the stored one ("Remind me what my job
            // is." generated as an assert with an invented value).
            let Some(start) = start else {
                return CompiledAction::QueryCurrent { relation };
            };
            let span = SourceSpan {
                start,
                end: start + value.len(),
            };
            if act == "assert" {
                CompiledAction::Assert { relation, span }
            } else {
                CompiledAction::Correct { relation, span }
            }
        }
        _ => unresolved("the op does not parse"),
    }
}

pub const COMPILER_SCHEMA: &str = "uor-r4.relation-compiler/1";
pub const COMPILER_LABEL_SCHEMA: &str = "m-world-v2.relations-acts/1";
const COMPILER_FEATURES: &str = "table: words at standardized scale; span: frame words in a \
     two-word window, the l1|r1 pair, shape and position";

/// Serialize `f64`s by their bits, so a saved artifact reloads exactly.
pub(crate) mod f64_bits {
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
pub(crate) struct HeadParts {
    classes: usize,
    dim: usize,
    #[serde(with = "f64_bits")]
    weights: Vec<f64>,
    #[serde(with = "f64_bits")]
    bias: Vec<f64>,
}

impl HeadParts {
    pub(crate) fn of(head: &SparseSoftmax) -> Self {
        Self {
            classes: head.classes,
            dim: head.dim,
            weights: head.weights.clone(),
            bias: head.bias.clone(),
        }
    }

    pub(crate) fn head(self, classes: usize, dim: usize) -> Result<SparseSoftmax> {
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

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DenseParts {
    classes: usize,
    dim: usize,
    #[serde(with = "f64_bits")]
    mean: Vec<f64>,
    #[serde(with = "f64_bits")]
    scale: Vec<f64>,
    #[serde(with = "f64_bits")]
    weights: Vec<f64>,
    #[serde(with = "f64_bits")]
    bias: Vec<f64>,
}

impl DenseParts {
    fn of(head: &Softmax) -> Self {
        Self {
            classes: head.classes,
            dim: head.dim,
            mean: head.mean.clone(),
            scale: head.scale.clone(),
            weights: head.weights.clone(),
            bias: head.bias.clone(),
        }
    }

    fn head(self, classes: usize, dim: usize) -> Result<Softmax> {
        if self.classes != classes
            || self.dim != dim
            || self.mean.len() != dim
            || self.scale.len() != dim
            || self.weights.len() != classes * dim
            || self.bias.len() != classes
            || self
                .mean
                .iter()
                .chain(&self.weights)
                .chain(&self.bias)
                .any(|v| !v.is_finite())
            || self.scale.iter().any(|s| !s.is_finite() || *s <= 0.0)
        {
            return Err(invalid(
                "a saved dense head has the wrong shape or a non-finite weight",
            ));
        }
        Ok(Softmax {
            classes,
            dim,
            mean: self.mean,
            scale: self.scale,
            weights: self.weights,
            bias: self.bias,
        })
    }
}

/// The combined heads and the trunk they read.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CombinedParts {
    trunk: crate::stack_grounded_session::EncoderIdentity,
    trunk_width: usize,
    relation_head: DenseParts,
    /// Absent under [`RelationMode::CombinedRelation`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    act_head: Option<DenseParts>,
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
    /// Present for the combined relation modes only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    combined: Option<CombinedParts>,
    /// The bound op model, for [`RelationMode::OpModel`] only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_model: Option<crate::stack_grounded_session::EncoderIdentity>,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The combined heads and the trunk they read.
struct Combined {
    trunk: Trunk,
    relation_head: Softmax,
    /// `None` under [`RelationMode::CombinedRelation`]: the table names acts.
    act_head: Option<Softmax>,
}

impl Combined {
    /// A turn's dense row: the trunk's features, then its binary words.
    fn row(&self, lexicon: &Lexicon, text: &str) -> Result<Vec<f64>> {
        let mut row = self.trunk.features(text)?;
        row.extend(lexicon.features(text));
        Ok(row)
    }
}

/// A fitted relation/act table and value-span head, loaded from its saved
/// bytes: the compiler of the grounded session (#962, D19). It predicts
/// from a turn's text alone; it never sees an evaluator label. Under
/// [`RelationMode::Combined`] dense heads over a frozen [`Trunk`] name the
/// relation and act instead of the table.
#[derive(Clone)]
pub struct SavedCompiler {
    bytes: Vec<u8>,
    inner: std::sync::Arc<(RelationRoute, SpanHead)>,
    combined: Option<std::sync::Arc<Combined>>,
    op: Option<std::sync::Arc<Trunk>>,
    act_rule: ActRule,
    op_policy: OpPolicy,
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
        Self::fit_with(train, tokenizer_sha256, training, settings, None)
    }

    /// [`Self::fit`], with the trunk that [`RelationMode::Combined`] needs.
    pub fn fit_with(
        train: &[Example],
        tokenizer_sha256: &str,
        training: Value,
        settings: CompilerSettings,
        trunk: Option<Trunk>,
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
        let (combined, op) = match (settings.relation_mode, trunk) {
            (RelationMode::Table, None) => (None, None),
            // The op model is trained separately (`compile-corpus`); the
            // compiler binds it and reads its generated ops.
            (RelationMode::OpModel, Some(trunk)) => (None, Some(trunk)),
            (RelationMode::Combined | RelationMode::CombinedRelation, Some(trunk)) => {
                let mut x = Vec::with_capacity(train.len());
                for example in train {
                    let mut row = trunk.features(&example.text)?;
                    row.extend(route.lexicon.features(&example.text));
                    x.push(row);
                }
                let relation_y: Vec<usize> = train
                    .iter()
                    .map(|e| {
                        route
                            .relations
                            .iter()
                            .position(|r| *r == e.relation)
                            .unwrap_or(route.relations.len() - 1)
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
                let fit = |y: &[usize], classes: usize| {
                    Softmax::fit(
                        &x,
                        y,
                        classes,
                        settings.table_steps,
                        settings.table_rate,
                        settings.table_l2,
                    )
                };
                let relation_head = fit(&relation_y, route.relations.len())?;
                let act_head = match settings.relation_mode {
                    RelationMode::Combined => Some(fit(&act_y, ACTS.len())?),
                    _ => None,
                };
                (
                    Some(Combined {
                        trunk,
                        relation_head,
                        act_head,
                    }),
                    None,
                )
            }
            (RelationMode::Table, Some(_)) => {
                return Err(invalid("the table relation mode reads no trunk"))
            }
            (_, None) => return Err(invalid("this relation mode needs a trunk")),
        };
        let bytes = encode(
            &route,
            &span,
            combined.as_ref(),
            op.as_ref().map(|t| &t.identity),
            tokenizer_sha256,
            training,
            settings,
        )?;
        Self::load(bytes, combined.map(|c| c.trunk).or(op))
    }

    /// Load a saved compiler that reads no trunk. Only canonical bytes of
    /// this schema load.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Self::load(bytes, None)
    }

    /// Load a saved compiler with the trunk its combined heads read, which
    /// must be the trunk, and read with the tokenizer, the artifact binds.
    pub fn load(bytes: Vec<u8>, trunk: Option<Trunk>) -> Result<Self> {
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
            relation_mode: match (&artifact.combined, &artifact.op_model) {
                (None, None) => RelationMode::Table,
                (None, Some(_)) => RelationMode::OpModel,
                (Some(parts), _) if parts.act_head.is_some() => RelationMode::Combined,
                (Some(_), _) => RelationMode::CombinedRelation,
            },
        };
        let words = artifact.words.len();
        let table_dim = words.max(1);
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
        let op_model = artifact.op_model.clone();
        let (combined, op) = match (artifact.combined, &op_model, trunk) {
            (None, None, None) => (None, None),
            (Some(_), Some(_), _) => {
                return Err(invalid(
                    "a compiler binds combined heads or an op model, not both",
                ))
            }
            (None, Some(bound), Some(trunk)) => {
                if *bound != trunk.identity || trunk.tokenizer_sha256 != artifact.tokenizer_sha256 {
                    return Err(invalid(
                        "the op model or its tokenizer is not the one the compiler binds",
                    ));
                }
                (None, Some(trunk))
            }
            (Some(parts), None, Some(trunk)) => {
                if parts.trunk != trunk.identity
                    || trunk.tokenizer_sha256 != artifact.tokenizer_sha256
                    || parts.trunk_width != 2 * trunk.model.config.width
                {
                    return Err(invalid(
                        "the trunk or its tokenizer is not the one the compiler binds",
                    ));
                }
                let dim = parts.trunk_width + words;
                (
                    Some(Combined {
                        trunk,
                        relation_head: parts.relation_head.head(route.relations.len(), dim)?,
                        act_head: parts
                            .act_head
                            .map(|head| head.head(ACTS.len(), dim))
                            .transpose()?,
                    }),
                    None,
                )
            }
            (_, _, None) => return Err(invalid("this compiler needs its trunk to load")),
            (None, None, Some(_)) => return Err(invalid("this compiler reads no trunk")),
        };
        if encode(
            &route,
            &span,
            combined.as_ref(),
            op_model.as_ref(),
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
            encoder: combined
                .as_ref()
                .map(|c| c.trunk.identity.clone())
                .or(op_model),
        };
        Ok(Self {
            bytes,
            inner: std::sync::Arc::new((route, span)),
            combined: combined.map(std::sync::Arc::new),
            op: op.map(std::sync::Arc::new),
            act_rule: settings.act_rule,
            op_policy: OpPolicy::Op,
            identity,
        })
    }

    /// The same compiler combining its op model and its table by `policy`
    /// (a load-time choice; the artifact is unchanged). Refused for a
    /// compiler without an op model unless `policy` is [`OpPolicy::Op`]
    /// or [`OpPolicy::Table`].
    pub fn with_op_policy(mut self, policy: OpPolicy) -> Result<Self> {
        if self.op.is_none() && !matches!(policy, OpPolicy::Op | OpPolicy::Table) {
            return Err(invalid("an op policy needs a compiler with an op model"));
        }
        self.op_policy = policy;
        Ok(self)
    }

    pub fn op_policy(&self) -> OpPolicy {
        self.op_policy
    }

    pub fn act_rule(&self) -> ActRule {
        self.act_rule
    }

    pub fn relation_mode(&self) -> RelationMode {
        if self.op.is_some() {
            return RelationMode::OpModel;
        }
        match &self.combined {
            None => RelationMode::Table,
            Some(combined) if combined.act_head.is_some() => RelationMode::Combined,
            Some(_) => RelationMode::CombinedRelation,
        }
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

    /// The op model's action for a turn read alone: the prompt is
    /// `System: Compile.` and the turn, the op is decoded greedily up to EOS
    /// ([`OP_MAX_TOKENS`] at most) and parsed by [`parse_op`].
    fn op_action(&self, source: &str) -> Result<crate::stack_grounded_session::CompiledAction> {
        use crate::stack_grounded_session::CompiledAction;
        use uor_r4_tokenizer::dialogue::Message;
        let op = self
            .op
            .as_ref()
            .ok_or_else(|| invalid("this compiler has no op model"))?;
        let encoder = op
            .protocol
            .bind(&op.tokenizer)
            .map_err(|e| invalid(format!("protocol: {e}")))?;
        let prompt = encoder.encode_assistant_prefix(&[
            Message {
                role: "system",
                content: COMPILE_PROMPT,
            },
            Message {
                role: "user",
                content: source,
            },
        ]);
        if prompt.emitted_turns != 2 || prompt.special_token_occurrences != 0 {
            return Ok(CompiledAction::Unresolved {
                reason: "the turn does not encode as a compile prompt".into(),
            });
        }
        if prompt.tokens.len() + OP_MAX_TOKENS + 1 > op.model.config.context {
            return Ok(CompiledAction::Unresolved {
                reason: "the turn is too long to compile".into(),
            });
        }
        let reply = crate::stack_dialogue::greedy_reply(
            &op.model,
            &prompt.tokens,
            OP_MAX_TOKENS,
            op.protocol.eos_id,
        )?;
        let ids: Vec<u32> = reply
            .ids
            .iter()
            .copied()
            .filter(|&id| id != op.protocol.eos_id)
            .collect();
        let text = op.tokenizer.decode(&ids);
        Ok(parse_op(&text, source, |name| self.relation_id(name)))
    }

    /// The relation and act the compiler's heads name for a turn: the
    /// combined heads when it has them, otherwise the table.
    pub fn classify(&self, source: &str) -> Result<(&str, &'static str)> {
        if self.op.is_some() {
            use crate::stack_grounded_session::CompiledAction;
            let (id, act) = match self.op_action(source)? {
                CompiledAction::Assert { relation, .. } => (relation, ACTS[0]),
                CompiledAction::Correct { relation, .. } => (relation, ACTS[1]),
                CompiledAction::QueryCurrent { relation }
                | CompiledAction::Query { relation, .. } => (relation, ACTS[2]),
                CompiledAction::Unresolved { .. } => return Ok((NONE, NONE)),
            };
            let name = self
                .identity
                .relations
                .iter()
                .find(|label| label.id == id)
                .map(|label| label.name.as_str())
                .ok_or_else(|| invalid("an op named a relation outside the labels"))?;
            return Ok((name, act));
        }
        let row = self.combined_row(source)?;
        self.classify_row(source, row.as_deref())
    }

    /// The combined heads' row of a turn (one trunk read), if they exist.
    fn combined_row(&self, source: &str) -> Result<Option<Vec<f64>>> {
        self.combined
            .as_ref()
            .map(|combined| combined.row(&self.route().lexicon, source))
            .transpose()
    }

    fn classify_row(&self, source: &str, row: Option<&[f64]>) -> Result<(&str, &'static str)> {
        match (&self.combined, row) {
            (None, None) => self.route().classify(source, None),
            (Some(combined), Some(row)) => Ok((
                self.route().relations[combined.relation_head.predict(row)].as_str(),
                match &combined.act_head {
                    Some(head) => ACTS[head.predict(row)],
                    None => self.route().classify(source, None)?.1,
                },
            )),
            _ => Err(invalid(
                "a combined row must accompany exactly combined heads",
            )),
        }
    }

    /// For a turn known to be a statement, `update` when the act head scores
    /// it above `assert`, otherwise `assert`.
    fn statement_act(&self, source: &str, row: Option<&[f64]>) -> Result<&'static str> {
        match (&self.combined, row) {
            (None, None) => self.route().statement_act(source, None),
            (Some(combined), Some(row)) => {
                let Some(head) = &combined.act_head else {
                    return self.route().statement_act(source, None);
                };
                let logits = head.logits(row);
                // ACTS[0] is assert and ACTS[1] update.
                Ok(if logits[1] > logits[0] {
                    ACTS[1]
                } else {
                    ACTS[0]
                })
            }
            _ => Err(invalid(
                "a combined row must accompany exactly combined heads",
            )),
        }
    }

    /// The action for a user turn: a query of the named relation; a
    /// statement or correction with the span head's value; or unresolved
    /// when the heads name no relation. Under [`ActRule::Table`] a turn is
    /// also unresolved when the heads name no act, or name a statement whose
    /// value the span head does not mark.
    pub fn action(&self, source: &str) -> Result<crate::stack_grounded_session::CompiledAction> {
        use crate::stack_grounded_session::CompiledAction;
        let statement = |a: &CompiledAction| {
            matches!(
                a,
                CompiledAction::Assert { .. } | CompiledAction::Correct { .. }
            )
        };
        if self.op.is_none() {
            return self.table_action(source);
        }
        match self.op_policy {
            OpPolicy::Op => self.op_action(source),
            OpPolicy::Table => self.table_action(source),
            OpPolicy::TableStatements => {
                let table = self.table_action(source)?;
                if !statement(&table) {
                    return Ok(table);
                }
                let op = self.op_action(source)?;
                Ok(if statement(&op) { op } else { table })
            }
            OpPolicy::UnlessQuery => {
                let table = self.table_action(source)?;
                if matches!(
                    table,
                    CompiledAction::QueryCurrent { .. } | CompiledAction::Query { .. }
                ) {
                    return Ok(table);
                }
                let op = self.op_action(source)?;
                Ok(if statement(&op) { op } else { table })
            }
        }
    }

    /// The table's (or combined heads') action for a turn, never the op
    /// model's.
    fn table_action(&self, source: &str) -> Result<crate::stack_grounded_session::CompiledAction> {
        use crate::stack_grounded_session::{CompiledAction, SourceSpan};
        let unresolved = |reason: &str| {
            Ok(CompiledAction::Unresolved {
                reason: reason.to_owned(),
            })
        };
        let row = self.combined_row(source)?;
        let (relation, act) = self.classify_row(source, row.as_deref())?;
        if relation == NONE {
            return unresolved("the heads name no relation");
        }
        let id = self
            .relation_id(relation)
            .ok_or_else(|| invalid("the heads named a relation outside the labels"))?;
        if self.act_rule == ActRule::Span {
            return Ok(match self.span().decode(source) {
                None => CompiledAction::QueryCurrent { relation: id },
                Some((start, end)) => {
                    let span = SourceSpan { start, end };
                    if self.statement_act(source, row.as_deref())? == "update" {
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
            _ => unresolved("the heads name no act"),
        }
    }
}

fn encode(
    route: &RelationRoute,
    span: &SpanHead,
    combined: Option<&Combined>,
    op_model: Option<&crate::stack_grounded_session::EncoderIdentity>,
    tokenizer_sha256: &str,
    training: Value,
    settings: CompilerSettings,
) -> Result<Vec<u8>> {
    if route.trunk.is_some() {
        return Err(invalid("a saved compiler's table has no trunk features"));
    }
    let combined_mode = matches!(
        settings.relation_mode,
        RelationMode::Combined | RelationMode::CombinedRelation
    );
    if (settings.relation_mode == RelationMode::OpModel) != op_model.is_some() {
        return Err(invalid("the relation mode and the op model disagree"));
    }
    if combined_mode != combined.is_some()
        || combined.is_some_and(|c| {
            c.act_head.is_some() != (settings.relation_mode == RelationMode::Combined)
        })
    {
        return Err(invalid("the relation mode and the combined heads disagree"));
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
        combined: combined.map(|c| CombinedParts {
            trunk: c.trunk.identity.clone(),
            trunk_width: 2 * c.trunk.model.config.width,
            relation_head: DenseParts::of(&c.relation_head),
            act_head: c.act_head.as_ref().map(DenseParts::of),
        }),
        op_model: op_model.cloned(),
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
        // A table-only compiler takes only the op and table policies.
        assert!(refit
            .clone()
            .with_op_policy(OpPolicy::TableStatements)
            .is_err());
        assert!(refit.clone().with_op_policy(OpPolicy::UnlessQuery).is_err());
        assert_eq!(
            refit.clone().with_op_policy(OpPolicy::Table)?.op_policy(),
            OpPolicy::Table
        );
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

    /// A byte-level tokenizer of 259 IDs: three specials, then the bytes.
    fn byte_tokenizer() -> Result<Vec<u8>> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, token) in specials.iter().enumerate() {
            vocab.insert((*token).into(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let code = if printable.contains(&byte) {
                byte
            } else {
                extra += 1;
                255 + extra
            };
            let symbol = char::from_u32(code).ok_or_else(|| invalid("alphabet"))?;
            vocab.insert(symbol.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, token)| json!({"id": id, "content": token}))
            .collect();
        Ok(serde_json::to_vec(&json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": added,
            "model": {"type": "BPE", "vocab": vocab, "merges": []}
        }))?)
    }

    /// A tiny stack saved under a fresh temporary directory.
    fn saved_trunk(name: &str, seed: u64) -> Result<std::path::PathBuf> {
        use crate::geometric_stack::{ReadScore, StackArch, StackConfig};
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "uor-relation-compiler-{}-{nonce}-{name}",
            std::process::id()
        ));
        let model = StackModel::new(
            StackConfig {
                arch: StackArch::Geometric,
                vocab_size: 259,
                width: 8,
                heads: 2,
                mlp_hidden: 16,
                context: 96,
                pattern: "ra".into(),
                read: ReadScore::Lorentz,
                rotation: true,
                rotation_group: Default::default(),
                seed,
                memory: None,
                select: None,
                pointer: None,
            },
            &candle_core::Device::Cpu,
        )?;
        model.save(&directory)?;
        Ok(directory)
    }

    #[test]
    fn combined_heads_load_only_with_the_trunk_they_were_fitted_on() -> Result<()> {
        use crate::stack_grounded_session::TurnCompiler;
        let device = candle_core::Device::Cpu;
        let tokenizer = byte_tokenizer()?;
        let digest =
            uor_r4_core::native_geometric::learner::realtext_support::sha256_hex(&tokenizer);
        let directory = saved_trunk("fitted", 7)?;
        let train = small_world();
        let settings = CompilerSettings {
            act_rule: ActRule::Span,
            relation_mode: RelationMode::Combined,
            table_steps: 60,
            ..CompilerSettings::default()
        };
        let saved = SavedCompiler::fit_with(
            &train,
            &digest,
            json!({"draw": "small world"}),
            settings,
            Some(Trunk::load(&directory, &tokenizer, &device)?),
        )?;
        assert_eq!(saved.relation_mode(), RelationMode::Combined);
        let encoder = saved
            .identity()
            .encoder
            .clone()
            .ok_or_else(|| invalid("no encoder identity"))?;
        assert_eq!(
            encoder.model_sha256,
            crate::sha256_file(&directory.join("model.safetensors"))?
        );
        // The same trunk reloads the same compiler; none, or another, refuses.
        let again = SavedCompiler::load(
            saved.bytes().to_vec(),
            Some(Trunk::load(&directory, &tokenizer, &device)?),
        )?;
        assert_eq!(again.bytes(), saved.bytes());
        for text in [
            "My name is Zorvak.",
            "Where am I from?",
            "The weather is nice today.",
        ] {
            assert_eq!(
                again.compile(text).map_err(|e| invalid(e.to_string()))?,
                saved.compile(text).map_err(|e| invalid(e.to_string()))?,
                "{text}"
            );
        }
        assert!(SavedCompiler::from_bytes(saved.bytes().to_vec()).is_err());
        let other = saved_trunk("other", 8)?;
        assert!(SavedCompiler::load(
            saved.bytes().to_vec(),
            Some(Trunk::load(&other, &tokenizer, &device)?)
        )
        .is_err());
        // A table compiler reads no trunk.
        let table = SavedCompiler::fit(&train, &digest, json!({}), CompilerSettings::default())?;
        assert!(SavedCompiler::load(
            table.bytes().to_vec(),
            Some(Trunk::load(&directory, &tokenizer, &device)?)
        )
        .is_err());
        assert!(RelationMode::parse("combined").is_ok() && RelationMode::parse("dense").is_err());
        // The split mode: combined relation names, the table's acts.
        let split = SavedCompiler::fit_with(
            &train,
            &digest,
            json!({"draw": "small world"}),
            CompilerSettings {
                relation_mode: RelationMode::CombinedRelation,
                ..settings
            },
            Some(Trunk::load(&directory, &tokenizer, &device)?),
        )?;
        assert_eq!(split.relation_mode(), RelationMode::CombinedRelation);
        let artifact: Value = serde_json::from_slice(split.bytes())?;
        assert!(artifact["combined"].get("act_head").is_none());
        let split_again = SavedCompiler::load(
            split.bytes().to_vec(),
            Some(Trunk::load(&directory, &tokenizer, &device)?),
        )?;
        assert_eq!(split_again.relation_mode(), RelationMode::CombinedRelation);
        for text in [
            "My name is Zorvak.",
            "Sorry, I grew up in Dunmere.",
            "Where am I from?",
        ] {
            assert_eq!(
                split.classify(text)?.1,
                split.route().classify(text, None)?.1,
                "{text}"
            );
            assert_eq!(
                split_again
                    .compile(text)
                    .map_err(|e| invalid(e.to_string()))?,
                split.compile(text).map_err(|e| invalid(e.to_string()))?
            );
        }
        assert!(RelationMode::parse("combined_relation").is_ok());
        // The op-model mode binds the model and reads its generated ops.
        let op = SavedCompiler::fit_with(
            &train,
            &digest,
            json!({"draw": "small world"}),
            CompilerSettings {
                relation_mode: RelationMode::OpModel,
                ..settings
            },
            Some(Trunk::load(&directory, &tokenizer, &device)?),
        )?;
        assert_eq!(op.relation_mode(), RelationMode::OpModel);
        let artifact: Value = serde_json::from_slice(op.bytes())?;
        assert!(artifact["op_model"]["model_sha256"].is_string());
        assert!(artifact.get("combined").is_none());
        // An untrained stack's output is not an op: unresolved, not an error.
        let action = op
            .compile("My name is Zorvak.")
            .map_err(|e| invalid(e.to_string()))?;
        assert!(matches!(
            action,
            crate::stack_grounded_session::CompiledAction::Unresolved { .. }
        ));
        assert!(SavedCompiler::from_bytes(op.bytes().to_vec()).is_err());
        assert!(SavedCompiler::load(
            op.bytes().to_vec(),
            Some(Trunk::load(&other, &tokenizer, &device)?)
        )
        .is_err());
        assert!(SavedCompiler::load(
            op.bytes().to_vec(),
            Some(Trunk::load(&directory, &tokenizer, &device)?)
        )
        .is_ok());
        let _ = std::fs::remove_dir_all(&directory);
        let _ = std::fs::remove_dir_all(&other);
        Ok(())
    }

    #[test]
    fn ops_are_written_from_labels_and_parsed_against_the_source() {
        use crate::stack_grounded_session::{CompiledAction, SourceSpan};
        let train = small_world();
        assert_eq!(
            op_text(&train[0]).as_deref(),
            Some("Op: assert user_name Sam")
        );
        assert_eq!(
            op_text(&train[1]).as_deref(),
            Some("Op: update user_name Tam")
        );
        assert_eq!(op_text(&train[3]).as_deref(), Some("Op: query user_name"));
        assert_eq!(op_text(&train[4]).as_deref(), Some("Op: none"));
        let ids = |name: &str| match name {
            "user_name" => Some(1),
            "hometown" => Some(4),
            _ => None,
        };
        let source = "Actually, call me Zorvak.";
        assert_eq!(
            parse_op("Op: update user_name Zorvak", source, ids),
            CompiledAction::Correct {
                relation: 1,
                span: SourceSpan { start: 18, end: 24 }
            }
        );
        // Case may differ; a trailing period is ignored.
        assert_eq!(
            parse_op(" Op: assert user_name zorvak.", source, ids),
            CompiledAction::Assert {
                relation: 1,
                span: SourceSpan { start: 18, end: 24 }
            }
        );
        assert_eq!(
            parse_op("Op: query hometown", "Where am I from?", ids),
            CompiledAction::QueryCurrent { relation: 4 }
        );
        // A statement whose value is not in the turn asks for the stored one.
        assert_eq!(
            parse_op("Op: assert user_name Plimbo", source, ids),
            CompiledAction::QueryCurrent { relation: 1 }
        );
        assert_eq!(
            parse_op("Op: update hometown Oslo", "Remind me where I live.", ids),
            CompiledAction::QueryCurrent { relation: 4 }
        );
        for (text, reason) in [
            ("Op: none", "the op is none"),
            ("Your name is Zorvak.", "the model produced no op"),
            ("Op: query pet_kind", "the op names an unknown relation"),
            ("Op: assert user_name", "the op does not parse"),
            ("Op: query hometown extra", "the op does not parse"),
        ] {
            assert_eq!(
                parse_op(text, source, ids),
                CompiledAction::Unresolved {
                    reason: reason.into()
                },
                "{text}"
            );
        }
    }

    #[test]
    fn an_op_policy_parses_and_needs_an_op_model() {
        for (text, policy) in [
            ("op", OpPolicy::Op),
            ("table", OpPolicy::Table),
            ("table_statements", OpPolicy::TableStatements),
            ("unless_query", OpPolicy::UnlessQuery),
        ] {
            assert_eq!(OpPolicy::parse(text).expect("a policy"), policy);
        }
        assert!(OpPolicy::parse("hybrid").is_err());
        assert_eq!(OpPolicy::default(), OpPolicy::Op);
    }

    #[test]
    fn the_lexicon_ignores_words_it_never_saw() {
        let lexicon = Lexicon::fit(["My friend is Sam.", "What is my name?"]);
        assert_eq!(lexicon.len(), 6);
        let features = lexicon.features("My buddy is Tam.");
        assert_eq!(features.iter().filter(|&&v| v == 1.0).count(), 2);
    }
}
