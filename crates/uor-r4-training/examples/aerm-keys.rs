//! G v2 frozen-trunk key probe (Lab 2, #973 comment 5890046442).
//!
//! A frozen S4 trunk is wrapped by four newly trained heads — a five-way tag
//! head, a four-way trigger head, an eight-way entity-atom head read at the
//! entity slot and a fourteen-way relation-atom head read at the trigger
//! position — on the D2-natural v2 world. The entity-atom head is a plain
//! K-way softmax in both arms; the one compared factor is the relation-atom
//! head parametrization: a learned linear map to a quaternion scored against
//! fixed unit icosians (`2I`), versus a plain K-way softmax over the same
//! atoms. Keys are the semiprime of the entity-atom prime and the relation-atom
//! prime, whose primes are reserved above the vocabulary band.
//!
//! ```text
//! aerm-keys run out=NEW_REPORT_ROOT trunk=TRUNK_DIR tokenizer=TOKENIZER.json \
//!   [splits=2,4] [steps=1500] [batch=16] [context=256] [lr=0.003] [warmup=100] \
//!   [weight_decay=0.1] [clip=1] [trigger_positive=5] [eval_episodes=512] \
//!   [eval_batch=16] [dev_episodes=256] [seed=9001] [data_seed=9002] \
//!   [eval_seed=9003] [dev_seed=9004]
//! aerm-keys decode out=NEW_REPORT_ROOT trunk=TRUNK_DIR tokenizer=TOKENIZER.json \
//!   [splits=2,4] [steps=30000] [train_episodes=1024] [eval_episodes=512] \
//!   [dev_episodes=256] [batch=16] [eval_batch=16] [context=256] [lr=0.01] \
//!   [warmup=100] [weight_decay=0] [clip=10] [seed=9001] [data_seed=9002] \
//!   [eval_seed=9003] [dev_seed=9004]
//! aerm-keys run-cached out=NEW_REPORT_ROOT trunk=TRUNK_DIR tokenizer=TOKENIZER.json \
//!   [splits=2,4] [steps=30000] [train_episodes=1024] [batch=16] [context=256] \
//!   [lr=0.01] [warmup=100] [weight_decay=0] [clip=10] [trigger_positive=5] \
//!   [eval_episodes=512] [eval_batch=16] [dev_episodes=256] [seed=9001] \
//!   [data_seed=9002] [eval_seed=9003] [dev_seed=9004] [plateau_window=2000] \
//!   [record_every=250]
//! aerm-keys roots
//! ```
//!
//! `decode` is the additive frozen-feature decodability diagnostic: it loads
//! the same pinned trunk, extracts the identical atom-supervised feature rows
//! the key probe uses, and drives one multinomial logistic map per atom head
//! to convergence, reporting per-split in-distribution and held-out atom
//! accuracy against the majority and chance baselines with the predeclared
//! exposure-limited/information-absent reading.
//!
//! `run-cached` is the exposure-only amendment of the pre-registered key
//! probe: it extracts the identical frozen-trunk features once over a fixed
//! training sample, keeping only the supervision rows the heads consume (the
//! `atom_rows` entity/relation rows, with their tag/trigger labels), then
//! trains the same four heads full-batch over those cached rows to
//! convergence (high lr, no weight decay, explicit plateau early-stop). The
//! compared 2I versus softmax factor, the metrics, the in-distribution guard
//! and the decision rules are unchanged from `run`.
//!
//! `run` claims an exclusive report root before loading anything, refuses a
//! trunk whose `model.safetensors` SHA-256 is not the pinned S4 arm-A value,
//! loads it through `AermModel::from_stack` with its saved icosian snap
//! restored, trains only the four heads (the trunk feature is detached, so no
//! gradient reaches it), evaluates held-out phrasings per phenomenon and per
//! class, and seals the report. `roots` prints the fixed 2I assignment. Set
//! `RAYON_NUM_THREADS=2` and `nice` for the fit.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{backprop::GradStore, DType, Device, Tensor, Var, D};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::Serialize;
use serde_json::{json, Value};

use uor_r4_core::prime_route_attention::{PrimeAtom, SemiprimeExpert};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::{logits_cross_entropy, StackModel, TransportSnap};
use uor_r4_training::stack_aerm::{
    entity_atom, AermModel, AtomStore, Clause, DialogueBatch, Episode, Phenomenon, RelationWorld,
    STATUS_ABSENT, STATUS_HIT, TAG_ENTITY, TAG_RELATION, TAG_VALUE, TRIGGER_NONE, TRIGGER_WRITE,
};
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

/// The pinned S4 arm-A trunk (`kimi-s4-fit-20260929/arm-a-snap-1/model`).
const TRUNK_SHA256: &str = "2b3b568141dfe213dd85e4d67e9ce0018e802f0654ee73ea10c7e552fc327b96";

const ENTITY_ATOMS: usize = 8;
const RELATION_ATOMS: usize = 14;
/// The achieved maximum pairwise `|q . r|` of the 14 relation roots, produced
/// by the deterministic greedy min-max search below and fixed before any fit.
/// The pre-registered `1/2` bound is infeasible: the largest set of unit
/// icosians with pairwise `|q . r| <= 1/2` has size 12 (the 24-cell `2T`
/// antipodal quotient; asserted in the tests), so the 14 relation roots take
/// the smallest bound the icosians allow.
const RELATION_ROOT_MAX_INNER: f32 = 0.809_017;
/// The pinned S4 trunk width; supervision-row features are stored as fixed
/// `[f32; FEATURE_WIDTH]` vectors.
const FEATURE_WIDTH: usize = 288;
/// The `run-cached` peak-RSS bound (1.5 GiB), asserted before sealing.
const PEAK_RSS_BOUND_KIB: u64 = 1_572_864;
#[cfg(test)]
const ROOT_BOUND_TOLERANCE: f32 = 1e-6;

const TAG_CLASSES: usize = 5;
const TAG5_OTHER: u32 = 0;
const TAG5_SPEAKER: u32 = 1;
const TAG5_ENTITY: u32 = 2;
const TAG5_RELATION: u32 = 3;
const TAG5_VALUE: u32 = 4;

const TRIGGER_CLASSES: usize = 4;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

// ---------------------------------------------------------------------------
// Arguments.

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing required argument {key}=")))
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }

    fn list<T: std::str::FromStr>(&self, key: &str, default: &str) -> Result<Vec<T>> {
        self.0
            .get(key)
            .map(String::as_str)
            .unwrap_or(default)
            .split(',')
            .map(|item| {
                item.trim()
                    .parse()
                    .map_err(|_| invalid(format!("invalid {key} item {item}")))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The fixed 2I atom roots.

struct AtomRoots {
    relation: Vec<[f32; 4]>,
    relation_indices: Vec<usize>,
    relation_bound: f32,
}

fn dot(left: &[f32; 4], right: &[f32; 4]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2] + left[3] * right[3]
}

fn pair_bound(set: &[usize], roots: &[[f32; 4]]) -> f32 {
    pair_score(set, roots).0
}

/// The largest pairwise `|q . r|` of `set` and how many pairs attain it.
fn pair_score(set: &[usize], roots: &[[f32; 4]]) -> (f32, usize) {
    let mut maximum = 0f32;
    let mut count = 0usize;
    for left in 0..set.len() {
        for right in left + 1..set.len() {
            let inner = dot(&roots[set[left]], &roots[set[right]]).abs();
            if inner > maximum + 1e-9 {
                maximum = inner;
                count = 1;
            } else if (inner - maximum).abs() <= 1e-9 {
                count += 1;
            }
        }
    }
    (maximum, count)
}

/// Deterministically selects `size` roots from `roots` that minimise the
/// largest pairwise `|q . r|`: every seed is tried and each greedy step adds
/// the candidate whose worst inner product to the chosen set is smallest
/// (ties to the lowest index).
fn select_dispersed(roots: &[[f32; 4]], size: usize) -> Vec<usize> {
    let mut best = Vec::new();
    let mut best_score = (f32::INFINITY, usize::MAX);
    for seed in 0..roots.len() {
        let mut chosen = vec![seed];
        while chosen.len() < size {
            let mut pick: Option<(usize, f32)> = None;
            for candidate in 0..roots.len() {
                if chosen.contains(&candidate) {
                    continue;
                }
                let worst = chosen
                    .iter()
                    .map(|&other| dot(&roots[other], &roots[candidate]).abs())
                    .fold(0.0f32, f32::max);
                if pick.is_none_or(|(index, value)| {
                    worst < value - 1e-9 || ((worst - value).abs() <= 1e-9 && candidate < index)
                }) {
                    pick = Some((candidate, worst));
                }
            }
            match pick {
                Some((candidate, _)) => chosen.push(candidate),
                None => break,
            }
        }
        if chosen.len() < size {
            continue;
        }
        let score = pair_score(&chosen, roots);
        if score.0 < best_score.0 - 1e-9
            || ((score.0 - best_score.0).abs() <= 1e-9 && score.1 < best_score.1)
        {
            best_score = score;
            best = chosen;
        }
    }
    best
}

/// The fixed relation-root assignment: the 14 unit icosians with the smallest
/// achievable maximum pairwise `|q . r|` ([`RELATION_ROOT_MAX_INNER`]), chosen
/// by the deterministic greedy min-max search and fixed before any fit. Only
/// the relation atoms carry 2I roots; the entity-atom head is a plain K-way
/// softmax in both arms.
fn select_roots() -> Result<AtomRoots> {
    let roots = TransportSnap::Icosian.roots();
    let relation_indices = select_dispersed(roots, RELATION_ATOMS);
    if relation_indices.len() < RELATION_ATOMS {
        return Err(invalid("the icosians cannot supply the relation roots"));
    }
    Ok(AtomRoots {
        relation: relation_indices.iter().map(|&index| roots[index]).collect(),
        relation_bound: pair_bound(&relation_indices, roots),
        relation_indices,
    })
}

/// The largest set of unit icosians with pairwise `|q . r| <= bound`, by
/// branch-and-bound with a greedy-colouring bound. Documents the
/// pre-registration's infeasibility: it is 12 at the required `1/2`.
#[cfg(test)]
fn max_compatible_set(roots: &[[f32; 4]], bound: f32) -> usize {
    let count = roots.len();
    let mut adjacency = vec![0u128; count];
    for left in 0..count {
        for right in 0..count {
            if left != right
                && dot(&roots[left], &roots[right]).abs() <= bound + ROOT_BOUND_TOLERANCE
            {
                adjacency[left] |= 1u128 << right;
            }
        }
    }
    let mut best = 0usize;
    expand_clique((1u128 << count) - 1, 0, &mut best, &adjacency);
    best
}

#[cfg(test)]
fn color_sort(candidates: u128, adjacency: &[u128]) -> (Vec<usize>, Vec<usize>) {
    let mut order = Vec::new();
    let mut colors = Vec::new();
    let mut remaining = candidates;
    let mut color = 0usize;
    while remaining != 0 {
        color += 1;
        let mut available = remaining;
        while available != 0 {
            let vertex = available.trailing_zeros() as usize;
            available &= !(1u128 << vertex);
            remaining &= !(1u128 << vertex);
            available &= !adjacency[vertex];
            order.push(vertex);
            colors.push(color);
        }
    }
    (order, colors)
}

#[cfg(test)]
fn expand_clique(candidates: u128, size: usize, best: &mut usize, adjacency: &[u128]) {
    if candidates == 0 {
        *best = (*best).max(size);
        return;
    }
    let (order, colors) = color_sort(candidates, adjacency);
    let mut remaining = candidates;
    for index in (0..order.len()).rev() {
        if size + colors[index] <= *best {
            return;
        }
        let vertex = order[index];
        expand_clique(remaining & adjacency[vertex], size + 1, best, adjacency);
        remaining &= !(1u128 << vertex);
    }
}

// ---------------------------------------------------------------------------
// Atom primes above the vocabulary band, and their semiprime keys.

/// The first `count` primes.
fn first_primes(count: usize) -> Result<Vec<u64>> {
    if count == 0 {
        return Err(invalid("the prime sieve needs at least one prime"));
    }
    let n = count as f64;
    let bound = if count < 6 {
        15
    } else {
        (n * (n.ln() + n.ln().ln())).ceil() as usize + 16
    };
    let mut composite = vec![false; bound + 1];
    let mut primes = Vec::with_capacity(count);
    for candidate in 2..=bound {
        if composite[candidate] {
            continue;
        }
        primes.push(candidate as u64);
        if primes.len() == count {
            break;
        }
        let mut multiple = candidate * candidate;
        while multiple <= bound {
            composite[multiple] = true;
            multiple += candidate;
        }
    }
    if primes.len() < count {
        return Err(invalid("the prime sieve produced too few primes"));
    }
    Ok(primes)
}

struct AtomKeys {
    entity: Vec<u64>,
    relation: Vec<u64>,
    vocab_prime_ceiling: u64,
}

impl AtomKeys {
    /// Reserves 22 primes above the vocabulary's primes. `uor-r4-core`'s
    /// `PrimeRegistry::compile` assigns its primes sequentially from 5 in
    /// semantic-ID order and cannot reserve arbitrary atom primes above the
    /// vocabulary band, so the reservation is this smallest correct mapping:
    /// the token `t` prime is the `(t + 1)`-th prime, the ceiling is the
    /// `vocab_size`-th prime and the atom primes are the next 22. Core's
    /// [`PrimeAtom`]/[`SemiprimeExpert`] validate the primes and give the
    /// order-independent semiprime identity.
    fn new(vocab_size: usize) -> Result<Self> {
        if vocab_size == 0 {
            return Err(invalid("the vocabulary must be non-empty"));
        }
        let primes = first_primes(vocab_size + ENTITY_ATOMS + RELATION_ATOMS)?;
        let ceiling = primes[vocab_size - 1];
        let entity = primes[vocab_size..vocab_size + ENTITY_ATOMS].to_vec();
        let relation = primes[vocab_size + ENTITY_ATOMS..].to_vec();
        for &prime in entity.iter().chain(relation.iter()) {
            PrimeAtom::new(prime as u32)
                .map_err(|error| invalid(format!("atom prime {prime} rejected: {error}")))?;
        }
        Ok(Self {
            entity,
            relation,
            vocab_prime_ceiling: ceiling,
        })
    }

    fn prime_of(&self, table: &[u64], atom: u32, kind: &str) -> Result<u64> {
        let index = atom
            .checked_sub(1)
            .ok_or_else(|| invalid(format!("{kind} atom 0 has no prime")))?
            as usize;
        table
            .get(index)
            .copied()
            .ok_or_else(|| invalid(format!("no {kind} prime for atom {atom}")))
    }

    fn semiprime(&self, entity_atom: u32, relation_atom: u32) -> Result<u64> {
        let left = PrimeAtom::new(self.prime_of(&self.entity, entity_atom, "entity")? as u32)
            .map_err(|error| invalid(format!("entity prime: {error}")))?;
        let right =
            PrimeAtom::new(self.prime_of(&self.relation, relation_atom, "relation")? as u32)
                .map_err(|error| invalid(format!("relation prime: {error}")))?;
        Ok(SemiprimeExpert::new(left, right)
            .map_err(|error| invalid(format!("semiprime key: {error}")))?
            .product())
    }
}

/// An exact atom store keyed by the semiprime, with the world `AtomStore`'s
/// version semantics.
#[derive(Default)]
struct SemiprimeStore {
    records: BTreeMap<u64, (u32, Option<u32>, u32)>,
}

impl SemiprimeStore {
    fn write(&mut self, key: u64, value: u32) {
        match self.records.get_mut(&key) {
            Some(record) => {
                if record.0 != value {
                    record.1 = Some(record.0);
                    record.0 = value;
                }
                record.2 += 1;
            }
            None => {
                self.records.insert(key, (value, None, 1));
            }
        }
    }

    fn read(&self, key: u64, previous: bool) -> (u32, u32) {
        match self.records.get(&key) {
            Some(record) => match (previous, record.1) {
                (false, _) => (STATUS_HIT, record.0),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None => (STATUS_ABSENT, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// The trained heads.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomKind {
    TwoI,
    Softmax,
}

impl AtomKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::TwoI => "2i",
            Self::Softmax => "softmax",
        }
    }
}

struct LinearHead {
    weight: Var,
    bias: Var,
}

impl LinearHead {
    fn new(classes: usize, width: usize, seed: u64, device: &Device) -> Result<Self> {
        let mut rng = Rng::new(seed);
        Ok(Self {
            weight: normal_var(&mut rng, &[classes, width], 0.02, device)?,
            bias: zeros_var(&[classes], device)?,
        })
    }

    fn logits(&self, feature: &Tensor) -> Result<Tensor> {
        Ok(feature
            .matmul(&self.weight.as_tensor().t()?)?
            .broadcast_add(self.bias.as_tensor())?)
    }
}

struct AtomArm {
    weight: Var,
    bias: Var,
    roots: Option<Tensor>,
}

impl AtomArm {
    fn new(
        kind: AtomKind,
        roots: &[[f32; 4]],
        classes: usize,
        width: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        let outputs = if kind == AtomKind::TwoI {
            if roots.len() != classes {
                return Err(invalid(format!(
                    "the 2I atom head needs {classes} roots, got {}",
                    roots.len()
                )));
            }
            4
        } else {
            classes
        };
        let mut rng = Rng::new(seed);
        let weight = normal_var(&mut rng, &[outputs, width], 0.02, device)?;
        let bias = zeros_var(&[outputs], device)?;
        let root_tensor = if kind == AtomKind::TwoI {
            let flat: Vec<f32> = roots.iter().flat_map(|root| root.iter().copied()).collect();
            Some(Tensor::from_vec(flat, (roots.len(), 4), device)?)
        } else {
            None
        };
        Ok(Self {
            weight,
            bias,
            roots: root_tensor,
        })
    }

    fn logits(&self, feature: &Tensor) -> Result<Tensor> {
        let projected = feature
            .matmul(&self.weight.as_tensor().t()?)?
            .broadcast_add(self.bias.as_tensor())?;
        match &self.roots {
            Some(roots) => Ok(projected.matmul(&roots.t()?)?),
            None => Ok(projected),
        }
    }

    fn parameter_count(&self) -> usize {
        self.weight.elem_count() + self.bias.elem_count()
    }
}

struct ProbeHeads {
    tag: LinearHead,
    trigger: LinearHead,
    entity: AtomArm,
    relation: AtomArm,
}

impl ProbeHeads {
    fn new(
        kind: AtomKind,
        roots: &AtomRoots,
        width: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        Ok(Self {
            tag: LinearHead::new(TAG_CLASSES, width, seed ^ 0x01, device)?,
            trigger: LinearHead::new(TRIGGER_CLASSES, width, seed ^ 0x02, device)?,
            entity: AtomArm::new(
                AtomKind::Softmax,
                &[],
                ENTITY_ATOMS,
                width,
                seed ^ 0x03,
                device,
            )?,
            relation: AtomArm::new(
                kind,
                &roots.relation,
                RELATION_ATOMS,
                width,
                seed ^ 0x04,
                device,
            )?,
        })
    }

    fn optimizer_groups(&self) -> (Vec<Var>, Vec<Var>) {
        (
            vec![
                self.tag.weight.clone(),
                self.trigger.weight.clone(),
                self.entity.weight.clone(),
                self.relation.weight.clone(),
            ],
            vec![
                self.tag.bias.clone(),
                self.trigger.bias.clone(),
                self.entity.bias.clone(),
                self.relation.bias.clone(),
            ],
        )
    }

    fn parameter_count(&self) -> usize {
        self.tag.weight.elem_count()
            + self.tag.bias.elem_count()
            + self.trigger.weight.elem_count()
            + self.trigger.bias.elem_count()
            + self.entity.parameter_count()
            + self.relation.parameter_count()
    }
}

fn normal_var(rng: &mut Rng, shape: &[usize], std: f64, device: &Device) -> Result<Var> {
    let count: usize = shape.iter().product();
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        let u1 = ((rng.next_u64() >> 11) as f64 + 1.0) / (1u64 << 53) as f64;
        let u2 = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let radius = (-2.0 * u1.ln()).sqrt();
        values.push((radius * (std::f64::consts::TAU * u2).cos() * std) as f32);
    }
    Ok(Var::from_vec(values, shape, device)?)
}

fn zeros_var(shape: &[usize], device: &Device) -> Result<Var> {
    Ok(Var::from_tensor(&Tensor::zeros(
        shape,
        DType::F32,
        device,
    )?)?)
}

// ---------------------------------------------------------------------------
// Frozen-trunk features and the head losses.

fn split_features(
    stack: &StackModel,
    split: usize,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<Tensor> {
    let x = stack.embed(ids, batch, time)?;
    let hidden = stack.run_layers(x, 0..split)?;
    let width = stack.config.width;
    let flat = hidden.reshape((batch * time, width))?;
    let rms = (flat.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?.sqrt()?;
    Ok(flat.broadcast_div(&rms)?.detach())
}

fn gold_tags5(batch: &DialogueBatch) -> Vec<u32> {
    (0..batch.batch * batch.time)
        .map(|row| match batch.tags[row] {
            TAG_ENTITY if batch.entity_atom[row] == entity_atom::SPEAKER => TAG5_SPEAKER,
            TAG_ENTITY => TAG5_ENTITY,
            TAG_RELATION => TAG5_RELATION,
            TAG_VALUE => TAG5_VALUE,
            _ => TAG5_OTHER,
        })
        .collect()
}

fn gather_rows(logits: &Tensor, rows: &[u32], device: &Device) -> Result<Tensor> {
    let index = Tensor::from_vec(rows.to_vec(), rows.len(), device)?;
    Ok(logits.index_select(&index, 0)?)
}

/// The atom supervision rows of a batch, exactly as the key probe's head
/// losses select them: entity rows are real positions carrying an entity atom
/// (target `entity_atom - 1`); relation rows are each clause's trigger
/// position (target `relation_atom - 1`). Shared by `head_losses` and the
/// `decode` mode so both read the identical feature rows.
fn atom_rows(batch: &DialogueBatch) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
    let rows = batch.batch * batch.time;
    let entity_rows: Vec<u32> = (0..rows)
        .filter(|&row| batch.entity_atom[row] != 0 && batch.real[row] > 0.0)
        .map(|row| row as u32)
        .collect();
    let entity_targets: Vec<u32> = entity_rows
        .iter()
        .map(|&row| batch.entity_atom[row as usize] - 1)
        .collect();

    let mut relation_rows = Vec::new();
    let mut relation_targets = Vec::new();
    for (episode_index, episode) in batch.episodes.iter().enumerate() {
        for clause in &episode.clauses {
            relation_rows.push((episode_index * batch.time + clause.trigger_position) as u32);
            relation_targets.push(clause.relation_atom - 1);
        }
    }
    (entity_rows, entity_targets, relation_rows, relation_targets)
}

fn head_losses(
    stack: &StackModel,
    split: usize,
    heads: &ProbeHeads,
    batch: &DialogueBatch,
    config: &ProbeConfig,
) -> Result<Tensor> {
    let device = stack.device();
    let feature = split_features(stack, split, &batch.ids, batch.batch, batch.time)?;

    let tag_logits = heads.tag.logits(&feature)?;
    let tag_loss = logits_cross_entropy(&tag_logits, &gold_tags5(batch), Some(&batch.real))?;

    let trigger_logits = heads.trigger.logits(&feature)?;
    let trigger_weights: Vec<f32> = batch
        .triggers
        .iter()
        .zip(&batch.real)
        .map(|(&trigger, &real)| {
            if trigger == TRIGGER_NONE {
                real
            } else {
                real * config.trigger_positive
            }
        })
        .collect();
    let trigger_loss =
        logits_cross_entropy(&trigger_logits, &batch.triggers, Some(&trigger_weights))?;

    let (entity_rows, entity_targets, relation_rows, relation_targets) = atom_rows(batch);

    let mut loss = (&tag_loss + &trigger_loss)?;
    if !entity_rows.is_empty() {
        let logits = heads.entity.logits(&feature)?;
        let selected = gather_rows(&logits, &entity_rows, device)?;
        loss = (&loss + &logits_cross_entropy(&selected, &entity_targets, None)?)?;
    }
    if !relation_rows.is_empty() {
        let logits = heads.relation.logits(&feature)?;
        let selected = gather_rows(&logits, &relation_rows, device)?;
        loss = (&loss + &logits_cross_entropy(&selected, &relation_targets, None)?)?;
    }
    Ok(loss)
}

fn clip_gradients(grads: &mut GradStore, vars: &[Var], max_norm: f64) -> Result<f64> {
    let mut total = 0f64;
    for var in vars {
        if let Some(grad) = grads.get(var.as_tensor()) {
            total += f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
        }
    }
    let norm = total.sqrt();
    if norm.is_finite() && norm > max_norm {
        let scale = max_norm / (norm + 1e-6);
        for var in vars {
            if let Some(grad) = grads.get(var.as_tensor()) {
                let scaled = (grad * scale)?;
                grads.insert(var.as_tensor(), scaled);
            }
        }
    }
    Ok(norm)
}

fn schedule(step: usize, warmup: usize, steps: usize) -> f64 {
    if step < warmup {
        return (step + 1) as f64 / warmup.max(1) as f64;
    }
    let span = steps.saturating_sub(warmup).max(1);
    let progress = (step - warmup) as f64 / span as f64;
    0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress.min(1.0)).cos())
}

#[derive(Clone, Debug, Serialize)]
struct StepRecord {
    step: usize,
    loss: f64,
    seconds: f64,
}

// ---------------------------------------------------------------------------
// Fit and evaluation.

#[derive(Clone)]
struct ProbeConfig {
    steps: usize,
    batch: usize,
    context: usize,
    learning_rate: f64,
    warmup: usize,
    weight_decay: f64,
    clip: f64,
    trigger_positive: f32,
    eval_episodes: usize,
    eval_batch: usize,
    dev_episodes: usize,
    seed: u64,
    data_seed: u64,
    eval_seed: u64,
    dev_seed: u64,
}

fn train_heads(
    stack: &StackModel,
    split: usize,
    kind: AtomKind,
    roots: &AtomRoots,
    world: &RelationWorld,
    config: &ProbeConfig,
) -> Result<(ProbeHeads, Vec<StepRecord>)> {
    let device = stack.device().clone();
    let width = stack.config.width;
    let heads = ProbeHeads::new(kind, roots, width, config.seed, &device)?;
    let (decayed, plain) = heads.optimizer_groups();
    let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
    let params = |weight_decay| ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
        weight_decay,
    };
    let mut decayed_optimizer = AdamW::new(decayed, params(config.weight_decay))?;
    let mut plain_optimizer = AdamW::new(plain, params(0.0))?;
    let mut rng = Rng::new(config.data_seed);
    let mut records = Vec::new();
    let started = Instant::now();
    for step in 0..config.steps {
        let lr = config.learning_rate * schedule(step, config.warmup, config.steps);
        decayed_optimizer.set_learning_rate(lr);
        plain_optimizer.set_learning_rate(lr);
        let batch = world.batch(&mut rng, config.batch, config.context, false)?;
        let loss = head_losses(stack, split, &heads, &batch, config)?;
        let mut grads = loss.backward()?;
        clip_gradients(&mut grads, &all, config.clip)?;
        decayed_optimizer.step(&grads)?;
        plain_optimizer.step(&grads)?;
        if step % 50 == 0 || step + 1 == config.steps {
            records.push(StepRecord {
                step,
                loss: f64::from(loss.to_scalar::<f32>()?),
                seconds: started.elapsed().as_secs_f64(),
            });
        }
    }
    Ok((heads, records))
}

#[derive(Clone, Debug, Default, Serialize)]
struct Counts {
    clauses: usize,
    key_correct: usize,
    writes: usize,
    write_key_correct: usize,
    reads: usize,
    read_key_correct: usize,
    register_correct: usize,
}

impl Counts {
    fn absorb(&mut self, other: &Counts) {
        self.clauses += other.clauses;
        self.key_correct += other.key_correct;
        self.writes += other.writes;
        self.write_key_correct += other.write_key_correct;
        self.reads += other.reads;
        self.read_key_correct += other.read_key_correct;
        self.register_correct += other.register_correct;
    }

    fn key_accuracy(&self) -> Option<f64> {
        (self.clauses > 0).then(|| self.key_correct as f64 / self.clauses as f64)
    }

    fn read_key_accuracy(&self) -> Option<f64> {
        (self.reads > 0).then(|| self.read_key_correct as f64 / self.reads as f64)
    }

    fn register_accuracy(&self) -> Option<f64> {
        (self.reads > 0).then(|| self.register_correct as f64 / self.reads as f64)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
struct EvalReport {
    episodes: usize,
    totals: Counts,
    phenomena: BTreeMap<String, Counts>,
    classes: BTreeMap<String, Counts>,
    entity_confusion: BTreeMap<String, usize>,
    relation_confusion: BTreeMap<String, usize>,
}

fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

fn clause_slots(episode: &Episode, clause: &Clause, turn_start: usize) -> Result<(usize, usize)> {
    let entity = (turn_start..=clause.trigger_position)
        .find(|&position| episode.entity_atom[position] == clause.entity_atom)
        .ok_or_else(|| invalid("a natural clause lacks its entity slot"))?;
    let relation = (turn_start..=clause.trigger_position)
        .find(|&position| episode.relation_atom[position] == clause.relation_atom)
        .ok_or_else(|| invalid("a natural clause lacks its relation slot"))?;
    Ok((entity, relation))
}

fn evaluate_heads(
    stack: &StackModel,
    split: usize,
    heads: &ProbeHeads,
    keys: &AtomKeys,
    world: &RelationWorld,
    config: &ProbeConfig,
    held: bool,
    seed: u64,
    episodes: usize,
) -> Result<EvalReport> {
    let mut report = EvalReport::default();
    let mut rng = Rng::new(seed);
    while report.episodes < episodes {
        let batch = world.batch(&mut rng, config.eval_batch, config.context, held)?;
        let feature = split_features(stack, split, &batch.ids, batch.batch, batch.time)?;
        let entity_argmax = argmax_rows(&heads.entity.logits(&feature)?)?;
        let relation_argmax = argmax_rows(&heads.relation.logits(&feature)?)?;
        for (episode_index, episode) in batch.episodes.iter().enumerate() {
            let base = episode_index * batch.time;
            let mut turn_start = 0usize;
            let mut gold = AtomStore::new();
            let mut model_store = SemiprimeStore::default();
            for clause in &episode.clauses {
                let (entity_slot, _) = clause_slots(episode, clause, turn_start)?;
                turn_start = clause.trigger_position + 1;
                let predicted_entity = entity_argmax[base + entity_slot] + 1;
                let predicted_relation = relation_argmax[base + clause.trigger_position] + 1;
                let key_correct = predicted_entity == clause.entity_atom
                    && predicted_relation == clause.relation_atom;
                let semiprime = keys.semiprime(predicted_entity, predicted_relation)?;
                let phenomenon = clause.phenomenon.as_str();
                let class = format!("{:?}", clause.class);
                let mut counts = Counts {
                    clauses: 1,
                    key_correct: usize::from(key_correct),
                    ..Counts::default()
                };
                if episode.triggers[clause.trigger_position] == TRIGGER_WRITE {
                    counts.writes = 1;
                    counts.write_key_correct = usize::from(key_correct);
                    let value = clause
                        .value
                        .ok_or_else(|| invalid("a natural write clause lacks its value"))?;
                    gold.write(clause.entity_atom, clause.relation_atom, value);
                    model_store.write(semiprime, value);
                } else {
                    counts.reads = 1;
                    counts.read_key_correct = usize::from(key_correct);
                    let gold_read =
                        gold.read(clause.entity_atom, clause.relation_atom, clause.previous);
                    let model_read = model_store.read(semiprime, clause.previous);
                    counts.register_correct = usize::from(gold_read == model_read);
                }
                if !key_correct {
                    if predicted_entity != clause.entity_atom {
                        *report
                            .entity_confusion
                            .entry(format!(
                                "{phenomenon}:{}->{predicted_entity}",
                                clause.entity_atom
                            ))
                            .or_default() += 1;
                    }
                    if predicted_relation != clause.relation_atom {
                        *report
                            .relation_confusion
                            .entry(format!(
                                "{phenomenon}:{}->{predicted_relation}",
                                clause.relation_atom
                            ))
                            .or_default() += 1;
                    }
                }
                report.totals.absorb(&counts);
                report
                    .phenomena
                    .entry(phenomenon.to_owned())
                    .or_default()
                    .absorb(&counts);
                report.classes.entry(class).or_default().absorb(&counts);
            }
            report.episodes += 1;
        }
        if report.episodes >= episodes {
            break;
        }
    }
    Ok(report)
}

// ---------------------------------------------------------------------------
// Reporting helpers.

fn counts_json(counts: &Counts) -> Value {
    json!({
        "clauses": counts.clauses,
        "key_correct": counts.key_correct,
        "key_accuracy": counts.key_accuracy(),
        "writes": counts.writes,
        "write_key_correct": counts.write_key_correct,
        "write_key_accuracy": (counts.writes > 0).then(|| counts.write_key_correct as f64 / counts.writes as f64),
        "reads": counts.reads,
        "read_key_correct": counts.read_key_correct,
        "read_key_accuracy": counts.read_key_accuracy(),
        "register_correct": counts.register_correct,
        "register_accuracy": counts.register_accuracy(),
    })
}

fn report_json(report: &EvalReport) -> Value {
    json!({
        "episodes": report.episodes,
        "totals": counts_json(&report.totals),
        "phenomena": report
            .phenomena
            .iter()
            .map(|(name, counts)| (name.clone(), counts_json(counts)))
            .collect::<BTreeMap<_, _>>(),
        "classes": report
            .classes
            .iter()
            .map(|(name, counts)| (name.clone(), counts_json(counts)))
            .collect::<BTreeMap<_, _>>(),
        "entity_confusion": report.entity_confusion,
        "relation_confusion": report.relation_confusion,
    })
}

fn phenomenon_accuracy(report: &EvalReport, phenomenon: Phenomenon) -> Option<f64> {
    report
        .phenomena
        .get(phenomenon.as_str())
        .and_then(Counts::key_accuracy)
}

/// The lexical phenomena whose held-out accuracy routes the < 0.5 rule.
const LEXICAL_PHENOMENA: [Phenomenon; 2] = [Phenomenon::SynonymsMorphology, Phenomenon::WhCarried];

fn decision_rules(arms: &[ArmResult]) -> Value {
    let two_i: Vec<&ArmResult> = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::TwoI)
        .collect();
    let softmax: Vec<&ArmResult> = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::Softmax)
        .collect();
    let best_two_i = two_i
        .iter()
        .max_by(|left, right| {
            left.held
                .totals
                .key_accuracy()
                .unwrap_or(0.0)
                .total_cmp(&right.held.totals.key_accuracy().unwrap_or(0.0))
        })
        .copied();
    let best_softmax = softmax
        .iter()
        .max_by(|left, right| {
            left.held
                .totals
                .key_accuracy()
                .unwrap_or(0.0)
                .total_cmp(&right.held.totals.key_accuracy().unwrap_or(0.0))
        })
        .copied();
    let (Some(best_two_i), Some(best_softmax)) = (best_two_i, best_softmax) else {
        return json!({"available": false, "reason": "both arms required"});
    };
    let per_phenomenon: BTreeMap<String, f64> = Phenomenon::ALL
        .iter()
        .filter_map(|&phenomenon| {
            phenomenon_accuracy(&best_two_i.held, phenomenon)
                .map(|accuracy| (phenomenon.as_str().to_owned(), accuracy))
        })
        .collect();
    let all_phenomena = Phenomenon::ALL.iter().all(|&phenomenon| {
        per_phenomenon
            .get(phenomenon.as_str())
            .is_some_and(|&a| a >= 0.90)
    });
    let in_range: BTreeMap<String, f64> = per_phenomenon
        .iter()
        .filter(|(_, &accuracy)| (0.5..0.9).contains(&accuracy))
        .map(|(name, &accuracy)| (name.clone(), accuracy))
        .collect();
    let lexical_below_half_both_splits = LEXICAL_PHENOMENA.iter().all(|&phenomenon| {
        two_i.iter().all(|arm| {
            phenomenon_accuracy(&arm.held, phenomenon).is_some_and(|accuracy| accuracy < 0.5)
        })
    });
    let two_i_accuracy = best_two_i.held.totals.key_accuracy().unwrap_or(0.0);
    let softmax_accuracy = best_softmax.held.totals.key_accuracy().unwrap_or(0.0);
    let gap = softmax_accuracy - two_i_accuracy;
    json!({
        "best_two_i_split": best_two_i.split,
        "best_softmax_split": best_softmax.split,
        "two_i_held_key_accuracy": two_i_accuracy,
        "softmax_held_key_accuracy": softmax_accuracy,
        "two_i_minus_softmax": two_i_accuracy - softmax_accuracy,
        "softmax_minus_two_i_gap": gap,
        "per_phenomenon_two_i_accuracy": per_phenomenon,
        "all_phenomena_at_least_0_90": all_phenomena,
        "heads_to_integration": all_phenomena,
        "geometric_atoms_used": gap <= 0.02,
        "phenomena_between_0_5_and_0_9": in_range,
        "lexical_below_0_5_at_both_splits": lexical_below_half_both_splits,
        "report_to_lab_1": lexical_below_half_both_splits,
        "lexical_phenomena": LEXICAL_PHENOMENA
            .iter()
            .map(|phenomenon| phenomenon.as_str())
            .collect::<Vec<_>>(),
    })
}

struct ArmResult {
    split: usize,
    kind: AtomKind,
    learnable_parameters: usize,
    held: EvalReport,
    dev: EvalReport,
    history: Value,
    plateau: Option<CachedPlateau>,
}

fn arm_json(arm: &ArmResult) -> Value {
    let mut value = json!({
        "split": arm.split,
        "atom_kind": arm.kind.as_str(),
        "learnable_parameters": arm.learnable_parameters,
        "held_out": report_json(&arm.held),
        "in_distribution": report_json(&arm.dev),
        "in_distribution_key_accuracy": arm.dev.totals.key_accuracy(),
        "history": arm.history.clone(),
    });
    if let Some(plateau) = &arm.plateau {
        value["plateau"] = json!(plateau);
    }
    value
}

// ---------------------------------------------------------------------------
// The `decode` mode: frozen-feature linear decodability of the atom heads.
//
// Where `run` trains four heads jointly for a bounded number of steps, the
// decode probe asks the narrower diagnostic question: do the frozen trunk's
// features at a split linearly encode the gold entity/relation atoms at all?
// It extracts the identical feature rows the key probe supervises (via
// [`split_features`] and [`atom_rows`]) on a fixed in-distribution sample,
// then drives one multinomial logistic map per atom head to convergence with
// full-batch Adam. The predeclared reading: in-distribution atom accuracy
// >= 0.98 for both heads at either split means the key probe's guard failure
// was exposure-limited; otherwise the frozen trunk does not expose the atom
// information at this seam.

#[derive(Clone)]
struct DecodeConfig {
    steps: usize,
    train_episodes: usize,
    eval_episodes: usize,
    dev_episodes: usize,
    batch: usize,
    eval_batch: usize,
    context: usize,
    learning_rate: f64,
    warmup: usize,
    weight_decay: f64,
    clip: f64,
    seed: u64,
    data_seed: u64,
    eval_seed: u64,
    dev_seed: u64,
}

/// The cached gold-atom rows of a fixed episode sample: the detached frozen
/// features at the supervised rows and their atom targets.
struct AtomSample {
    episodes: usize,
    entity_features: Tensor,
    entity_targets: Vec<u32>,
    relation_features: Tensor,
    relation_targets: Vec<u32>,
}

fn collect_atom_sample(
    stack: &StackModel,
    split: usize,
    world: &RelationWorld,
    config: &DecodeConfig,
    held: bool,
    seed: u64,
    episodes: usize,
) -> Result<AtomSample> {
    let device = stack.device().clone();
    let mut rng = Rng::new(seed);
    let mut collected = 0usize;
    let mut entity_features = Vec::new();
    let mut entity_targets = Vec::new();
    let mut relation_features = Vec::new();
    let mut relation_targets = Vec::new();
    while collected < episodes {
        let batch = world.batch(&mut rng, config.batch, config.context, held)?;
        let feature = split_features(stack, split, &batch.ids, batch.batch, batch.time)?;
        let (entity_rows, entity_row_targets, relation_rows, relation_row_targets) =
            atom_rows(&batch);
        if !entity_rows.is_empty() {
            entity_features.push(gather_rows(&feature, &entity_rows, &device)?);
            entity_targets.extend(entity_row_targets);
        }
        if !relation_rows.is_empty() {
            relation_features.push(gather_rows(&feature, &relation_rows, &device)?);
            relation_targets.extend(relation_row_targets);
        }
        collected += batch.episodes.len();
    }
    if entity_features.is_empty() || relation_features.is_empty() {
        return Err(invalid("the decode sample has no supervised atom rows"));
    }
    Ok(AtomSample {
        episodes: collected,
        entity_features: Tensor::cat(&entity_features, 0)?,
        entity_targets,
        relation_features: Tensor::cat(&relation_features, 0)?,
        relation_targets,
    })
}

#[derive(Clone, Debug, Serialize)]
struct DecodeStep {
    step: usize,
    loss: f64,
    train_accuracy: f64,
    seconds: f64,
}

/// Evidence that the train curve plateaued: the best train accuracy over the
/// final `window` steps against the best over everything before it.
#[derive(Clone, Debug, Serialize)]
struct Plateau {
    window: usize,
    best_accuracy_final_window: f64,
    best_accuracy_before_window: f64,
    stopped_improving: bool,
}

fn head_accuracy(head: &LinearHead, features: &Tensor, targets: &[u32]) -> Result<f64> {
    let predicted = argmax_rows(&head.logits(features)?)?;
    let correct = predicted
        .iter()
        .zip(targets)
        .filter(|(left, right)| left == right)
        .count();
    Ok(correct as f64 / targets.len().max(1) as f64)
}

/// Full-batch multinomial logistic regression on cached frozen features,
/// driven for `config.steps` Adam steps under the same cosine schedule shape
/// as the key probe, with the train curve recorded for the plateau check.
fn fit_decode_probe(
    features: &Tensor,
    targets: &[u32],
    classes: usize,
    width: usize,
    seed: u64,
    config: &DecodeConfig,
    device: &Device,
) -> Result<(LinearHead, Vec<DecodeStep>, Plateau)> {
    let head = LinearHead::new(classes, width, seed, device)?;
    let params = |weight_decay| ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
        weight_decay,
    };
    let mut weight_optimizer = AdamW::new(vec![head.weight.clone()], params(config.weight_decay))?;
    let mut bias_optimizer = AdamW::new(vec![head.bias.clone()], params(0.0))?;
    let all = [head.weight.clone(), head.bias.clone()];
    let mut history = Vec::new();
    let started = Instant::now();
    for step in 0..config.steps {
        let lr = config.learning_rate * schedule(step, config.warmup, config.steps);
        weight_optimizer.set_learning_rate(lr);
        bias_optimizer.set_learning_rate(lr);
        let loss = logits_cross_entropy(&head.logits(features)?, targets, None)?;
        let mut grads = loss.backward()?;
        clip_gradients(&mut grads, &all, config.clip)?;
        weight_optimizer.step(&grads)?;
        bias_optimizer.step(&grads)?;
        if step % 250 == 0 || step + 1 == config.steps {
            history.push(DecodeStep {
                step,
                loss: f64::from(loss.to_scalar::<f32>()?),
                train_accuracy: head_accuracy(&head, features, targets)?,
                seconds: started.elapsed().as_secs_f64(),
            });
        }
    }
    let window = 2000.min(config.steps / 2).max(1);
    let split_at = config.steps - window;
    let best = |records: &[&DecodeStep]| {
        records
            .iter()
            .map(|record| record.train_accuracy)
            .fold(0f64, f64::max)
    };
    let before: Vec<&DecodeStep> = history
        .iter()
        .filter(|record| record.step < split_at)
        .collect();
    let final_window: Vec<&DecodeStep> = history
        .iter()
        .filter(|record| record.step >= split_at)
        .collect();
    let plateau = Plateau {
        window,
        best_accuracy_final_window: best(&final_window),
        best_accuracy_before_window: best(&before),
        stopped_improving: best(&final_window) <= best(&before) + 1e-12,
    };
    Ok((head, history, plateau))
}

fn majority_baseline(train_targets: &[u32], targets: &[u32], classes: usize) -> Result<f64> {
    let mut counts = vec![0usize; classes];
    for &target in train_targets {
        let slot = counts
            .get_mut(target as usize)
            .ok_or_else(|| invalid(format!("atom target {target} outside {classes} classes")))?;
        *slot += 1;
    }
    let majority = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(class, _)| class as u32)
        .ok_or_else(|| invalid("no train targets for the majority baseline"))?;
    let correct = targets.iter().filter(|&&target| target == majority).count();
    Ok(correct as f64 / targets.len().max(1) as f64)
}

fn current_rss_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
}

fn decode_head_json(
    head: &str,
    classes: usize,
    chance: f64,
    width: usize,
    fitted: &(LinearHead, Vec<DecodeStep>, Plateau),
    train: &AtomSample,
    dev: &AtomSample,
    held: &AtomSample,
) -> Result<Value> {
    let (probe, history, plateau) = fitted;
    let (train_features, train_targets) = match head {
        "entity" => (&train.entity_features, &train.entity_targets),
        _ => (&train.relation_features, &train.relation_targets),
    };
    let (dev_features, dev_targets) = match head {
        "entity" => (&dev.entity_features, &dev.entity_targets),
        _ => (&dev.relation_features, &dev.relation_targets),
    };
    let (held_features, held_targets) = match head {
        "entity" => (&held.entity_features, &held.entity_targets),
        _ => (&held.relation_features, &held.relation_targets),
    };
    Ok(json!({
        "head": head,
        "classes": classes,
        "chance": chance,
        "parameter_count": probe.weight.elem_count() + probe.bias.elem_count(),
        "feature_width": width,
        "rows": {
            "train": train_targets.len(),
            "in_distribution": dev_targets.len(),
            "held_out": held_targets.len(),
        },
        "train_accuracy": head_accuracy(probe, train_features, train_targets)?,
        "in_distribution_accuracy": head_accuracy(probe, dev_features, dev_targets)?,
        "held_out_accuracy": head_accuracy(probe, held_features, held_targets)?,
        "majority_baseline": {
            "train": majority_baseline(train_targets, train_targets, classes)?,
            "in_distribution": majority_baseline(train_targets, dev_targets, classes)?,
            "held_out": majority_baseline(train_targets, held_targets, classes)?,
        },
        "plateau": plateau,
        "history": history,
    }))
}

fn decode(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let trunk_directory = PathBuf::from(args.required("trunk")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let splits: Vec<usize> = args.list("splits", "2,4")?;
    if splits.is_empty() || splits.iter().any(|&split| split == 0) {
        return Err(invalid("splits must be non-empty positive layer counts"));
    }
    let config = DecodeConfig {
        steps: args.number("steps", 30_000)?,
        train_episodes: args.number("train_episodes", 1024)?,
        eval_episodes: args.number("eval_episodes", 512)?,
        dev_episodes: args.number("dev_episodes", 256)?,
        batch: args.number("batch", 16)?,
        eval_batch: args.number("eval_batch", 16)?,
        context: args.number("context", 256)?,
        learning_rate: args.number("lr", 0.01)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.0)?,
        clip: args.number("clip", 10.0)?,
        seed: args.number("seed", 9001)?,
        data_seed: args.number("data_seed", 9002)?,
        eval_seed: args.number("eval_seed", 9003)?,
        dev_seed: args.number("dev_seed", 9004)?,
    };
    if config.steps < 2
        || config.batch == 0
        || config.eval_batch == 0
        || config.context < 2
        || config.train_episodes == 0
        || config.eval_episodes == 0
        || config.dev_episodes == 0
    {
        return Err(invalid(
            "steps, episodes, batch, eval_batch and context must be usable",
        ));
    }

    let trunk_weights = trunk_directory.join("model.safetensors");
    let trunk_sha256 = sha256_file(&trunk_weights)?;
    if trunk_sha256 != TRUNK_SHA256 {
        return Err(invalid(format!(
            "trunk {} is not the pinned S4 arm-A checkpoint: model.safetensors SHA-256 {trunk_sha256}",
            trunk_directory.display()
        )));
    }
    let device = Device::Cpu;
    let mut stack = StackModel::load(&trunk_directory, &device)?;
    let mut snap_restored = false;
    if let Some(snap) = StackModel::saved_transport_snap(&trunk_directory)? {
        stack.set_transport_snap(Some(snap))?;
        snap_restored = true;
    }
    if splits.iter().any(|&split| split >= stack.config.layers()) {
        return Err(invalid("a split must leave layers on both sides"));
    }
    let model = AermModel::from_stack(stack, splits[0], false, config.seed)?;
    let stack = &model.stack;
    let width = stack.config.width;

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|error| invalid(format!("protocol: {error}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::natural(&encode, protocol.bos_id, protocol.eos_id)?;

    let mut split_reports = Vec::new();
    for &split in &splits {
        let train = collect_atom_sample(
            stack,
            split,
            &world,
            &config,
            false,
            config.data_seed,
            config.train_episodes,
        )?;
        let dev_batch = DecodeConfig {
            batch: config.eval_batch,
            ..config.clone()
        };
        let dev = collect_atom_sample(
            stack,
            split,
            &world,
            &dev_batch,
            false,
            config.dev_seed,
            config.dev_episodes,
        )?;
        let held = collect_atom_sample(
            stack,
            split,
            &world,
            &dev_batch,
            true,
            config.eval_seed,
            config.eval_episodes,
        )?;
        let entity = fit_decode_probe(
            &train.entity_features,
            &train.entity_targets,
            ENTITY_ATOMS,
            width,
            config.seed ^ 0x11,
            &config,
            &device,
        )?;
        let relation = fit_decode_probe(
            &train.relation_features,
            &train.relation_targets,
            RELATION_ATOMS,
            width,
            config.seed ^ 0x12,
            &config,
            &device,
        )?;
        split_reports.push(json!({
            "split": split,
            "samples": {
                "train_episodes": train.episodes,
                "in_distribution_episodes": dev.episodes,
                "held_out_episodes": held.episodes,
            },
            "entity": decode_head_json("entity", ENTITY_ATOMS, 1.0 / ENTITY_ATOMS as f64, width, &entity, &train, &dev, &held)?,
            "relation": decode_head_json("relation", RELATION_ATOMS, 1.0 / RELATION_ATOMS as f64, width, &relation, &train, &dev, &held)?,
        }));
    }

    // Predeclared reading: in-distribution atom accuracy >= 0.98 for both
    // heads at either split means the key probe's guard failure was
    // exposure-limited; otherwise the frozen trunk does not expose the atom
    // information at this seam.
    let exposure_limited = split_reports.iter().any(|report| {
        report["entity"]["in_distribution_accuracy"]
            .as_f64()
            .is_some_and(|accuracy| accuracy >= 0.98)
            && report["relation"]["in_distribution_accuracy"]
                .as_f64()
                .is_some_and(|accuracy| accuracy >= 0.98)
    });
    let conclusion = if exposure_limited {
        "exposure_limited"
    } else {
        "information_absent"
    };

    let report = json!({
        "schema": "uor-r4.g2-decode-probe/1",
        "trunk": {
            "directory": trunk_directory.display().to_string(),
            "model_safetensors_sha256": trunk_sha256,
            "pinned_sha256": TRUNK_SHA256,
            "splits": splits,
            "loaded_through": "AermModel::from_stack",
            "transport_snap_restored": snap_restored,
            "layers": stack.config.layers(),
            "width": width,
        },
        "features": "identical to the key probe: split_features (RMS-normalized detached hidden at the split) at the atom_rows supervision rows",
        "config": {
            "steps": config.steps,
            "train_episodes": config.train_episodes,
            "eval_episodes": config.eval_episodes,
            "dev_episodes": config.dev_episodes,
            "batch": config.batch,
            "eval_batch": config.eval_batch,
            "context": config.context,
            "learning_rate": config.learning_rate,
            "warmup": config.warmup,
            "weight_decay": config.weight_decay,
            "clip": config.clip,
            "seed": config.seed,
            "data_seed": config.data_seed,
            "eval_seed": config.eval_seed,
            "dev_seed": config.dev_seed,
        },
        "splits": split_reports,
        "predeclared_reading": "in-distribution >= 0.98 for both heads at either split -> exposure_limited; otherwise -> information_absent",
        "conclusion": conclusion,
        "wall_seconds": started.elapsed().as_secs_f64(),
        "peak_rss_kib": current_rss_kib(),
        "peak_rss_note": "in-process sample via ps at report time; the true maximum resident set size is captured by running the sealed binary under /usr/bin/time -l",
    });
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// The `run-cached` mode: the pre-registered key probe at adequate exposure.
//
// The decode probe proved the frozen trunk's features linearly encode the
// atoms in-distribution, so the key probe's guard failure at 1500 stepped
// batches was predeclared exposure-limited. This mode changes only the
// exposure: the identical features (`split_features` at the same supervision
// rows) are extracted once over a fixed training sample, and the same four
// heads are trained full-batch to convergence. The compared 2I versus
// softmax factor, the store, the keys, the metrics, the guard and the
// decision rules are unchanged.

#[derive(Clone)]
struct CachedConfig {
    steps: usize,
    train_episodes: usize,
    batch: usize,
    context: usize,
    learning_rate: f64,
    warmup: usize,
    weight_decay: f64,
    clip: f64,
    trigger_positive: f32,
    eval_episodes: usize,
    eval_batch: usize,
    dev_episodes: usize,
    seed: u64,
    data_seed: u64,
    eval_seed: u64,
    dev_seed: u64,
    plateau_window: usize,
    record_every: usize,
}

/// One cached supervision row: the detached frozen-trunk feature at a
/// position the key probe's heads supervise (an `atom_rows` entity row or a
/// clause trigger position), with the labels the four heads consume and the
/// episode id. No per-token sequence or autodiff graph is retained.
struct SupervisionRow {
    episode: usize,
    feature: [f32; FEATURE_WIDTH],
    tag: u32,
    real: f32,
    trigger: u32,
    trigger_weight: f32,
    entity_atom: u32,
    relation_atom: u32,
}

/// The cached supervision rows of a fixed in-distribution training sample,
/// extracted episode by episode from the frozen trunk: the full kept-row
/// feature matrix for the tag/trigger heads and the gathered atom rows (via
/// [`atom_rows`], exactly as [`head_losses`] selects them) for the
/// entity/relation heads.
struct CachedTrain {
    episodes: usize,
    rows: Vec<SupervisionRow>,
    features: Tensor,
    tag_targets: Vec<u32>,
    real: Vec<f32>,
    triggers: Vec<u32>,
    trigger_weights: Vec<f32>,
    entity_features: Tensor,
    entity_targets: Vec<u32>,
    relation_features: Tensor,
    relation_targets: Vec<u32>,
}

fn collect_train_cache(
    stack: &StackModel,
    split: usize,
    world: &RelationWorld,
    config: &CachedConfig,
) -> Result<CachedTrain> {
    let device = stack.device().clone();
    if stack.config.width != FEATURE_WIDTH {
        return Err(invalid(format!(
            "the trunk width {} is not the pinned {FEATURE_WIDTH}",
            stack.config.width
        )));
    }
    let mut rng = Rng::new(config.data_seed);
    let mut episodes = 0usize;
    let mut rows: Vec<SupervisionRow> = Vec::new();
    while episodes < config.train_episodes {
        let batch = world.batch(&mut rng, 1, config.context, false)?;
        let feature = split_features(stack, split, &batch.ids, 1, batch.time)?;
        let tags = gold_tags5(&batch);
        let (entity_rows, _, relation_rows, relation_row_targets) = atom_rows(&batch);
        let mut kept: BTreeMap<usize, (u32, u32)> = BTreeMap::new();
        for &row in &entity_rows {
            kept.entry(row as usize).or_default().0 = batch.entity_atom[row as usize];
        }
        for (&row, &target) in relation_rows.iter().zip(&relation_row_targets) {
            kept.entry(row as usize).or_default().1 = target + 1;
        }
        for (row, (entity_atom, relation_atom)) in kept {
            let values = feature.get(row)?.to_vec1::<f32>()?;
            let feature_row: [f32; FEATURE_WIDTH] = values
                .as_slice()
                .try_into()
                .map_err(|_| invalid("a supervision row is not FEATURE_WIDTH wide"))?;
            let trigger = batch.triggers[row];
            let real = batch.real[row];
            rows.push(SupervisionRow {
                episode: episodes,
                feature: feature_row,
                tag: tags[row],
                real,
                trigger,
                trigger_weight: if trigger == TRIGGER_NONE {
                    real
                } else {
                    real * config.trigger_positive
                },
                entity_atom,
                relation_atom,
            });
        }
        episodes += 1;
    }
    let entity: Vec<&SupervisionRow> = rows.iter().filter(|row| row.entity_atom != 0).collect();
    let relation: Vec<&SupervisionRow> = rows.iter().filter(|row| row.relation_atom != 0).collect();
    if entity.is_empty() || relation.is_empty() {
        return Err(invalid(
            "the cached training sample has no supervised atom rows",
        ));
    }
    let mut rows_per_episode = vec![0usize; episodes];
    for row in &rows {
        rows_per_episode[row.episode] += 1;
    }
    if rows_per_episode.iter().any(|&count| count == 0) {
        return Err(invalid("a cached training episode has no supervision rows"));
    }
    let flat: Vec<f32> = rows.iter().flat_map(|row| row.feature).collect();
    let entity_flat: Vec<f32> = entity.iter().flat_map(|row| row.feature).collect();
    let relation_flat: Vec<f32> = relation.iter().flat_map(|row| row.feature).collect();
    Ok(CachedTrain {
        episodes,
        tag_targets: rows.iter().map(|row| row.tag).collect(),
        real: rows.iter().map(|row| row.real).collect(),
        triggers: rows.iter().map(|row| row.trigger).collect(),
        trigger_weights: rows.iter().map(|row| row.trigger_weight).collect(),
        entity_targets: entity.iter().map(|row| row.entity_atom - 1).collect(),
        relation_targets: relation.iter().map(|row| row.relation_atom - 1).collect(),
        features: Tensor::from_vec(flat, (rows.len(), FEATURE_WIDTH), &device)?,
        entity_features: Tensor::from_vec(entity_flat, (entity.len(), FEATURE_WIDTH), &device)?,
        relation_features: Tensor::from_vec(
            relation_flat,
            (relation.len(), FEATURE_WIDTH),
            &device,
        )?,
        rows,
    })
}

/// Row chunk for the full-batch tag/trigger loss and accuracy passes. The
/// weighted-mean cross-entropy decomposes exactly over chunks (each chunk
/// loss is scaled by its share of the total weight), so every row still
/// contributes to every step's gradient. The pieces are backpropped one at a
/// time and their gradient stores merged: candle's matmul backward
/// materializes the gradient wrt both operands, so one combined graph would
/// hold a features-sized gradient per chunk at once.
const FULL_BATCH_CHUNK_ROWS: usize = 131072;

fn weighted_cross_entropy_pieces(
    head: &LinearHead,
    features: &Tensor,
    targets: &[u32],
    weights: &[f32],
    pieces: &mut Vec<Tensor>,
) -> Result<()> {
    let rows = targets.len();
    let total_weight: f64 = weights.iter().map(|&w| f64::from(w)).sum();
    let before = pieces.len();
    for start in (0..rows).step_by(FULL_BATCH_CHUNK_ROWS) {
        let end = (start + FULL_BATCH_CHUNK_ROWS).min(rows);
        let chunk_weights = &weights[start..end];
        let chunk_weight: f64 = chunk_weights.iter().map(|&w| f64::from(w)).sum();
        if chunk_weight <= 0.0 {
            continue;
        }
        let chunk_features = features.narrow(0, start, end - start)?;
        let chunk_logits = head.logits(&chunk_features)?;
        let chunk_loss =
            logits_cross_entropy(&chunk_logits, &targets[start..end], Some(chunk_weights))?;
        pieces.push((chunk_loss * (chunk_weight / total_weight))?);
    }
    if pieces.len() == before {
        return Err(invalid("the cached training sample has no weighted rows"));
    }
    Ok(())
}

/// The same loss composition as [`head_losses`] — tag, trigger, entity atom,
/// relation atom, with the same weights — as separately differentiable
/// pieces over the cached features. Gathering the atom rows before the
/// linear heads is the same map as [`head_losses`]' gather after them,
/// because every head is linear in the feature.
fn cached_head_loss_pieces(heads: &ProbeHeads, cache: &CachedTrain) -> Result<Vec<Tensor>> {
    let mut pieces = Vec::new();
    // The small atom pieces come first: the first piece's gradient store is
    // reused as the accumulator and keeps its entries for the whole step, so
    // its untracked-input gradient should stay small.
    let entity_logits = heads.entity.logits(&cache.entity_features)?;
    pieces.push(logits_cross_entropy(
        &entity_logits,
        &cache.entity_targets,
        None,
    )?);
    let relation_logits = heads.relation.logits(&cache.relation_features)?;
    pieces.push(logits_cross_entropy(
        &relation_logits,
        &cache.relation_targets,
        None,
    )?);
    weighted_cross_entropy_pieces(
        &heads.tag,
        &cache.features,
        &cache.tag_targets,
        &cache.real,
        &mut pieces,
    )?;
    weighted_cross_entropy_pieces(
        &heads.trigger,
        &cache.features,
        &cache.triggers,
        &cache.trigger_weights,
        &mut pieces,
    )?;
    Ok(pieces)
}

/// The summed loss value and the merged gradient store of all pieces.
fn cached_head_loss_and_grads(
    heads: &ProbeHeads,
    cache: &CachedTrain,
    vars: &[Var],
) -> Result<(f64, GradStore)> {
    let mut combined: Option<GradStore> = None;
    let mut loss_value = 0f64;
    for piece in cached_head_loss_pieces(heads, cache)? {
        loss_value += f64::from(piece.to_scalar::<f32>()?);
        let piece_grads = piece.backward()?;
        match &mut combined {
            None => combined = Some(piece_grads),
            Some(acc) => {
                for var in vars {
                    if let Some(grad) = piece_grads.get(var.as_tensor()) {
                        let merged = match acc.get(var.as_tensor()) {
                            Some(existing) => (existing + grad)?,
                            None => grad.clone(),
                        };
                        acc.insert(var.as_tensor(), merged);
                    }
                }
            }
        }
    }
    match combined {
        Some(grads) => Ok((loss_value, grads)),
        None => Err(invalid(
            "the cached training sample produced no loss pieces",
        )),
    }
}

#[derive(Clone, Debug, Serialize)]
struct CachedStep {
    step: usize,
    loss: f64,
    tag_accuracy: f64,
    trigger_accuracy: f64,
    entity_accuracy: f64,
    relation_accuracy: f64,
    tracked_accuracy: f64,
    seconds: f64,
}

/// The plateau/early-stop evidence: the tracked train accuracy (the minimum
/// of the four head accuracies) over the final `window` steps against
/// everything before it, with the steps actually run.
#[derive(Clone, Debug, Serialize)]
struct CachedPlateau {
    window: usize,
    best_tracked_final_window: f64,
    best_tracked_before_window: f64,
    stopped_improving: bool,
    stopped_early: bool,
    steps_run: usize,
    final_tag_accuracy: f64,
    final_trigger_accuracy: f64,
    final_entity_accuracy: f64,
    final_relation_accuracy: f64,
}

fn masked_head_accuracy(
    head: &LinearHead,
    features: &Tensor,
    targets: &[u32],
    real: &[f32],
) -> Result<f64> {
    let mut predicted = Vec::with_capacity(targets.len());
    for start in (0..targets.len()).step_by(FULL_BATCH_CHUNK_ROWS) {
        let end = (start + FULL_BATCH_CHUNK_ROWS).min(targets.len());
        let logits = head.logits(&features.narrow(0, start, end - start)?)?;
        predicted.extend(argmax_rows(&logits)?);
    }
    let mut correct = 0usize;
    let mut total = 0usize;
    for ((&prediction, &target), &weight) in predicted.iter().zip(targets).zip(real) {
        if weight > 0.0 {
            total += 1;
            correct += usize::from(prediction == target);
        }
    }
    Ok(correct as f64 / total.max(1) as f64)
}

fn arm_accuracy(arm: &AtomArm, features: &Tensor, targets: &[u32]) -> Result<f64> {
    let predicted = argmax_rows(&arm.logits(features)?)?;
    let correct = predicted
        .iter()
        .zip(targets)
        .filter(|(left, right)| left == right)
        .count();
    Ok(correct as f64 / targets.len().max(1) as f64)
}

/// Full-batch training of the four probe heads on the cached features, with
/// the same initialization seeds, optimizer groups and schedule shape as
/// [`train_heads`]. Stops early when the tracked train accuracy (the minimum
/// of the four head accuracies) has not improved in the last
/// `plateau_window` steps.
fn train_heads_cached(
    width: usize,
    kind: AtomKind,
    roots: &AtomRoots,
    cache: &CachedTrain,
    config: &CachedConfig,
    device: &Device,
) -> Result<(ProbeHeads, Vec<CachedStep>, CachedPlateau)> {
    let heads = ProbeHeads::new(kind, roots, width, config.seed, device)?;
    let (decayed, plain) = heads.optimizer_groups();
    let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
    let params = |weight_decay| ParamsAdamW {
        lr: config.learning_rate,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
        weight_decay,
    };
    let mut decayed_optimizer = AdamW::new(decayed, params(config.weight_decay))?;
    let mut plain_optimizer = AdamW::new(plain, params(0.0))?;
    let mut records: Vec<CachedStep> = Vec::new();
    let started = Instant::now();
    let mut steps_run = 0usize;
    let mut stopped_early = false;
    for step in 0..config.steps {
        let lr = config.learning_rate * schedule(step, config.warmup, config.steps);
        decayed_optimizer.set_learning_rate(lr);
        plain_optimizer.set_learning_rate(lr);
        let (loss, mut grads) = cached_head_loss_and_grads(&heads, cache, &all)?;
        clip_gradients(&mut grads, &all, config.clip)?;
        decayed_optimizer.step(&grads)?;
        plain_optimizer.step(&grads)?;
        steps_run = step + 1;
        if step % config.record_every == 0 || step + 1 == config.steps {
            let tag_accuracy =
                masked_head_accuracy(&heads.tag, &cache.features, &cache.tag_targets, &cache.real)?;
            let trigger_accuracy = masked_head_accuracy(
                &heads.trigger,
                &cache.features,
                &cache.triggers,
                &cache.real,
            )?;
            let entity_accuracy =
                arm_accuracy(&heads.entity, &cache.entity_features, &cache.entity_targets)?;
            let relation_accuracy = arm_accuracy(
                &heads.relation,
                &cache.relation_features,
                &cache.relation_targets,
            )?;
            let tracked = tag_accuracy
                .min(trigger_accuracy)
                .min(entity_accuracy)
                .min(relation_accuracy);
            records.push(CachedStep {
                step,
                loss,
                tag_accuracy,
                trigger_accuracy,
                entity_accuracy,
                relation_accuracy,
                tracked_accuracy: tracked,
                seconds: started.elapsed().as_secs_f64(),
            });
            let window = config.plateau_window;
            if steps_run >= 2 * window {
                let split_at = steps_run - window;
                let best_final = records
                    .iter()
                    .filter(|record| record.step >= split_at)
                    .map(|record| record.tracked_accuracy)
                    .fold(0f64, f64::max);
                let best_before = records
                    .iter()
                    .filter(|record| record.step < split_at)
                    .map(|record| record.tracked_accuracy)
                    .fold(0f64, f64::max);
                if best_final <= best_before + 1e-12 {
                    stopped_early = true;
                    break;
                }
            }
        }
    }
    let window = (config.plateau_window.min(steps_run / 2)).max(1);
    let split_at = steps_run - window;
    let best_final = records
        .iter()
        .filter(|record| record.step >= split_at)
        .map(|record| record.tracked_accuracy)
        .fold(0f64, f64::max);
    let best_before = records
        .iter()
        .filter(|record| record.step < split_at)
        .map(|record| record.tracked_accuracy)
        .fold(0f64, f64::max);
    let last = records
        .last()
        .ok_or_else(|| invalid("the cached fit recorded no steps"))?;
    let plateau = CachedPlateau {
        window,
        best_tracked_final_window: best_final,
        best_tracked_before_window: best_before,
        stopped_improving: best_final <= best_before + 1e-12,
        stopped_early,
        steps_run,
        final_tag_accuracy: last.tag_accuracy,
        final_trigger_accuracy: last.trigger_accuracy,
        final_entity_accuracy: last.entity_accuracy,
        final_relation_accuracy: last.relation_accuracy,
    };
    Ok((heads, records, plateau))
}

/// One evaluation episode with the heads' atom predictions at its
/// supervision rows: entity predictions at the `atom_rows` entity positions
/// and relation predictions at the clause trigger positions. The store/sieve
/// evaluation below runs the real store path from these predictions.
struct EvalEpisode {
    episode: Episode,
    entity_predictions: BTreeMap<usize, u32>,
    relation_predictions: BTreeMap<usize, u32>,
}

/// Extracts the same supervision-row features as the training cache, once per
/// evaluation episode, and records the heads' argmax predictions at them.
fn collect_eval_episodes(
    stack: &StackModel,
    split: usize,
    heads: &ProbeHeads,
    world: &RelationWorld,
    context: usize,
    held: bool,
    seed: u64,
    episodes: usize,
) -> Result<Vec<EvalEpisode>> {
    let device = stack.device().clone();
    let mut rng = Rng::new(seed);
    let mut out = Vec::new();
    while out.len() < episodes {
        let batch = world.batch(&mut rng, 1, context, held)?;
        let feature = split_features(stack, split, &batch.ids, 1, batch.time)?;
        let (entity_rows, _, relation_rows, _) = atom_rows(&batch);
        let mut entity_predictions = BTreeMap::new();
        if !entity_rows.is_empty() {
            let selected = gather_rows(&feature, &entity_rows, &device)?;
            let predicted = argmax_rows(&heads.entity.logits(&selected)?)?;
            for (&row, &prediction) in entity_rows.iter().zip(&predicted) {
                entity_predictions.insert(row as usize, prediction + 1);
            }
        }
        let mut relation_predictions = BTreeMap::new();
        if !relation_rows.is_empty() {
            let selected = gather_rows(&feature, &relation_rows, &device)?;
            let predicted = argmax_rows(&heads.relation.logits(&selected)?)?;
            for (&row, &prediction) in relation_rows.iter().zip(&predicted) {
                relation_predictions.insert(row as usize, prediction + 1);
            }
        }
        let episode = batch
            .episodes
            .into_iter()
            .next()
            .ok_or_else(|| invalid("the eval batch has no episode"))?;
        out.push(EvalEpisode {
            episode,
            entity_predictions,
            relation_predictions,
        });
    }
    Ok(out)
}

/// The store/sieve evaluation over cached supervision-row predictions: the
/// same per-episode real store path as [`evaluate_heads`], keyed by the
/// semiprime of the predicted entity/relation atoms, against the v2 world's
/// `AtomStore` gold.
fn evaluate_cached_episodes(keys: &AtomKeys, episodes: &[EvalEpisode]) -> Result<EvalReport> {
    let mut report = EvalReport::default();
    for eval in episodes {
        let episode = &eval.episode;
        let mut turn_start = 0usize;
        let mut gold = AtomStore::new();
        let mut model_store = SemiprimeStore::default();
        for clause in &episode.clauses {
            let (entity_slot, _) = clause_slots(episode, clause, turn_start)?;
            turn_start = clause.trigger_position + 1;
            let predicted_entity = *eval
                .entity_predictions
                .get(&entity_slot)
                .ok_or_else(|| invalid("the eval cache lacks the entity slot row"))?;
            let predicted_relation = *eval
                .relation_predictions
                .get(&clause.trigger_position)
                .ok_or_else(|| invalid("the eval cache lacks the trigger row"))?;
            let key_correct = predicted_entity == clause.entity_atom
                && predicted_relation == clause.relation_atom;
            let semiprime = keys.semiprime(predicted_entity, predicted_relation)?;
            let phenomenon = clause.phenomenon.as_str();
            let class = format!("{:?}", clause.class);
            let mut counts = Counts {
                clauses: 1,
                key_correct: usize::from(key_correct),
                ..Counts::default()
            };
            if episode.triggers[clause.trigger_position] == TRIGGER_WRITE {
                counts.writes = 1;
                counts.write_key_correct = usize::from(key_correct);
                let value = clause
                    .value
                    .ok_or_else(|| invalid("a natural write clause lacks its value"))?;
                gold.write(clause.entity_atom, clause.relation_atom, value);
                model_store.write(semiprime, value);
            } else {
                counts.reads = 1;
                counts.read_key_correct = usize::from(key_correct);
                let gold_read =
                    gold.read(clause.entity_atom, clause.relation_atom, clause.previous);
                let model_read = model_store.read(semiprime, clause.previous);
                counts.register_correct = usize::from(gold_read == model_read);
            }
            if !key_correct {
                if predicted_entity != clause.entity_atom {
                    *report
                        .entity_confusion
                        .entry(format!(
                            "{phenomenon}:{}->{predicted_entity}",
                            clause.entity_atom
                        ))
                        .or_default() += 1;
                }
                if predicted_relation != clause.relation_atom {
                    *report
                        .relation_confusion
                        .entry(format!(
                            "{phenomenon}:{}->{predicted_relation}",
                            clause.relation_atom
                        ))
                        .or_default() += 1;
                }
            }
            report.totals.absorb(&counts);
            report
                .phenomena
                .entry(phenomenon.to_owned())
                .or_default()
                .absorb(&counts);
            report.classes.entry(class).or_default().absorb(&counts);
        }
        report.episodes += 1;
    }
    Ok(report)
}

fn run_cached(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let trunk_directory = PathBuf::from(args.required("trunk")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let splits: Vec<usize> = args.list("splits", "2,4")?;
    if splits.is_empty() || splits.iter().any(|&split| split == 0) {
        return Err(invalid("splits must be non-empty positive layer counts"));
    }
    let config = CachedConfig {
        steps: args.number("steps", 30_000)?,
        train_episodes: args.number("train_episodes", 1024)?,
        batch: args.number("batch", 16)?,
        context: args.number("context", 256)?,
        learning_rate: args.number("lr", 0.01)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.0)?,
        clip: args.number("clip", 10.0)?,
        trigger_positive: args.number("trigger_positive", 5.0)?,
        eval_episodes: args.number("eval_episodes", 512)?,
        eval_batch: args.number("eval_batch", 16)?,
        dev_episodes: args.number("dev_episodes", 256)?,
        seed: args.number("seed", 9001)?,
        data_seed: args.number("data_seed", 9002)?,
        eval_seed: args.number("eval_seed", 9003)?,
        dev_seed: args.number("dev_seed", 9004)?,
        plateau_window: args.number("plateau_window", 2000)?,
        record_every: args.number("record_every", 250)?,
    };
    if config.steps < 2
        || config.batch == 0
        || config.eval_batch == 0
        || config.context < 2
        || config.train_episodes == 0
        || config.eval_episodes == 0
        || config.dev_episodes == 0
        || config.plateau_window == 0
        || config.record_every == 0
    {
        return Err(invalid(
            "steps, episodes, batch, eval_batch, context, plateau_window and record_every must be usable",
        ));
    }

    let trunk_weights = trunk_directory.join("model.safetensors");
    let trunk_sha256 = sha256_file(&trunk_weights)?;
    if trunk_sha256 != TRUNK_SHA256 {
        return Err(invalid(format!(
            "trunk {} is not the pinned S4 arm-A checkpoint: model.safetensors SHA-256 {trunk_sha256}",
            trunk_directory.display()
        )));
    }
    let device = Device::Cpu;
    let mut stack = StackModel::load(&trunk_directory, &device)?;
    let mut snap_restored = false;
    if let Some(snap) = StackModel::saved_transport_snap(&trunk_directory)? {
        stack.set_transport_snap(Some(snap))?;
        snap_restored = true;
    }
    if splits.iter().any(|&split| split >= stack.config.layers()) {
        return Err(invalid("a split must leave layers on both sides"));
    }
    let model = AermModel::from_stack(stack, splits[0], false, config.seed)?;
    let stack = &model.stack;

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|error| invalid(format!("protocol: {error}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::natural(&encode, protocol.bos_id, protocol.eos_id)?;
    let roots = select_roots()?;
    let keys = AtomKeys::new(stack.config.vocab_size)?;

    let debug_rss = std::env::var_os("AERM_KEYS_DEBUG_RSS").is_some();
    let mut arms = Vec::new();
    let mut training_samples = Vec::new();
    for &split in &splits {
        let cache = collect_train_cache(stack, split, &world, &config)?;
        if debug_rss {
            eprintln!(
                "split {split}: cache collected ({} rows), rss {:?}",
                cache.rows.len(),
                current_rss_kib()
            );
        }
        training_samples.push(json!({
            "split": split,
            "episodes": cache.episodes,
            "rows": cache.features.dims()[0],
            "entity_rows": cache.entity_targets.len(),
            "relation_rows": cache.relation_targets.len(),
        }));
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, history, plateau) =
                train_heads_cached(stack.config.width, kind, &roots, &cache, &config, &device)?;
            if debug_rss {
                eprintln!(
                    "split {split} {:?}: trained {} steps, rss {:?}",
                    kind,
                    plateau.steps_run,
                    current_rss_kib()
                );
            }
            let held_episodes = collect_eval_episodes(
                stack,
                split,
                &heads,
                &world,
                config.context,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            let held = evaluate_cached_episodes(&keys, &held_episodes)?;
            let dev_episodes = collect_eval_episodes(
                stack,
                split,
                &heads,
                &world,
                config.context,
                false,
                config.dev_seed,
                config.dev_episodes,
            )?;
            let dev = evaluate_cached_episodes(&keys, &dev_episodes)?;
            arms.push(ArmResult {
                split,
                kind,
                learnable_parameters: heads.parameter_count(),
                held,
                dev,
                history: json!(history),
                plateau: Some(plateau),
            });
        }
    }

    let peak_rss_kib = current_rss_kib();
    if let Some(rss) = peak_rss_kib {
        if rss > PEAK_RSS_BOUND_KIB {
            return Err(invalid(format!(
                "peak RSS {rss} KiB exceeds the 1.5 GiB supervision-row cache bound"
            )));
        }
    }

    let decisions = decision_rules(&arms);
    let guard_passes = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::TwoI)
        .all(|arm| arm.dev.totals.key_accuracy().unwrap_or(0.0) >= 0.98);
    let report = json!({
        "schema": "uor-r4.g2-key-probe/1",
        "amendment": "exposure only (supervision-row feature cache); metrics, guard and decision rules unchanged",
        "attempt_note": "keys-2 failed under full-token feature caching; this run caches supervision rows only; both failed attempts are preserved",
        "expected": "the in-distribution guard should now pass; the discriminating measurement is held-out relation-atom and key accuracy, where the frozen trunk is expected to be weak (decode probe held-out relation 0.450/0.491)",
        "trunk": {
            "directory": trunk_directory.display().to_string(),
            "model_safetensors_sha256": trunk_sha256,
            "pinned_sha256": TRUNK_SHA256,
            "splits": splits,
            "loaded_through": "AermModel::from_stack",
            "transport_snap_restored": snap_restored,
            "layers": stack.config.layers(),
            "width": stack.config.width,
        },
        "roots": {
            "snap": TransportSnap::Icosian.name(),
            "root_count": TransportSnap::Icosian.roots().len(),
            "relation_indices": roots.relation_indices,
            "relation_bound": roots.relation_bound,
            "relation_root_max_inner": RELATION_ROOT_MAX_INNER,
            "entity_head": "plain 8-way softmax in both arms (no entity roots)",
            "deviation": "the pre-registered 22 roots at pairwise |q.r| <= 1/2 are infeasible: the largest such icosian set has 12 (the 2T antipodal quotient; see the test); only the 14 relation atoms carry 2I roots, at the smallest achievable bound",
        },
        "keys": {
            "vocabulary_size": stack.config.vocab_size,
            "vocab_prime_ceiling": keys.vocab_prime_ceiling,
            "entity_primes": keys.entity,
            "relation_primes": keys.relation,
            "registry": "uor-r4-token primes are the first vocab_size primes; atom primes are the next 22 above the ceiling, validated by uor_r4_core::prime_route_attention::PrimeAtom and combined by SemiprimeExpert",
        },
        "features": "identical to the key probe: split_features (RMS-normalized detached hidden at the split), extracted episode by episode over the fixed samples; only the atom_rows supervision rows (entity rows and clause trigger positions, with tag/trigger labels) are cached as [f32; 288] rows; no per-token sequences or autodiff graphs are retained",
        "training_samples": training_samples,
        "config": {
            "steps": config.steps,
            "train_episodes": config.train_episodes,
            "batch": config.batch,
            "context": config.context,
            "learning_rate": config.learning_rate,
            "warmup": config.warmup,
            "weight_decay": config.weight_decay,
            "clip": config.clip,
            "trigger_positive": config.trigger_positive,
            "eval_episodes": config.eval_episodes,
            "eval_batch": config.eval_batch,
            "dev_episodes": config.dev_episodes,
            "seed": config.seed,
            "data_seed": config.data_seed,
            "eval_seed": config.eval_seed,
            "dev_seed": config.dev_seed,
            "plateau_window": config.plateau_window,
            "record_every": config.record_every,
        },
        "arms": arms.iter().map(arm_json).collect::<Vec<_>>(),
        "guard": {
            "in_distribution_key_accuracy_at_least": 0.98,
            "passes": guard_passes,
        },
        "decisions": decisions,
        "parameter_comparison": {
            "2i_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::TwoI).map(|arm| arm.learnable_parameters),
            "softmax_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::Softmax).map(|arm| arm.learnable_parameters),
        },
        "wall_seconds": started.elapsed().as_secs_f64(),
        "peak_rss_kib": peak_rss_kib,
        "peak_rss_bound_kib": PEAK_RSS_BOUND_KIB,
        "peak_rss_note": "in-process sample via ps at report time, asserted <= peak_rss_bound_kib; the true maximum resident set size is captured by running the sealed binary under /usr/bin/time -l",
    });
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry points.

fn run(args: &Args, out: &Path) -> Result<()> {
    let trunk_directory = PathBuf::from(args.required("trunk")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let splits: Vec<usize> = args.list("splits", "2,4")?;
    if splits.is_empty() || splits.iter().any(|&split| split == 0) {
        return Err(invalid("splits must be non-empty positive layer counts"));
    }
    let config = ProbeConfig {
        steps: args.number("steps", 1500)?,
        batch: args.number("batch", 16)?,
        context: args.number("context", 256)?,
        learning_rate: args.number("lr", 0.003)?,
        warmup: args.number("warmup", 100)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        trigger_positive: args.number("trigger_positive", 5.0)?,
        eval_episodes: args.number("eval_episodes", 512)?,
        eval_batch: args.number("eval_batch", 16)?,
        dev_episodes: args.number("dev_episodes", 256)?,
        seed: args.number("seed", 9001)?,
        data_seed: args.number("data_seed", 9002)?,
        eval_seed: args.number("eval_seed", 9003)?,
        dev_seed: args.number("dev_seed", 9004)?,
    };
    if config.steps == 0 || config.batch == 0 || config.eval_batch == 0 || config.context < 2 {
        return Err(invalid(
            "steps, batch, eval_batch and context must be usable",
        ));
    }

    let trunk_weights = trunk_directory.join("model.safetensors");
    let trunk_sha256 = sha256_file(&trunk_weights)?;
    if trunk_sha256 != TRUNK_SHA256 {
        return Err(invalid(format!(
            "trunk {} is not the pinned S4 arm-A checkpoint: model.safetensors SHA-256 {trunk_sha256}",
            trunk_directory.display()
        )));
    }
    let device = Device::Cpu;
    let mut stack = StackModel::load(&trunk_directory, &device)?;
    let mut snap_restored = false;
    if let Some(snap) = StackModel::saved_transport_snap(&trunk_directory)? {
        stack.set_transport_snap(Some(snap))?;
        snap_restored = true;
    }
    if splits.iter().any(|&split| split >= stack.config.layers()) {
        return Err(invalid("a split must leave layers on both sides"));
    }
    let model = AermModel::from_stack(stack, splits[0], false, config.seed)?;
    let stack = &model.stack;

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|error| invalid(format!("protocol: {error}")))?;
    let encode = |text: &str| tokenizer.encode(text);
    let world = RelationWorld::natural(&encode, protocol.bos_id, protocol.eos_id)?;
    let roots = select_roots()?;
    let keys = AtomKeys::new(stack.config.vocab_size)?;

    let mut arms = Vec::new();
    for &split in &splits {
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, records) = train_heads(stack, split, kind, &roots, &world, &config)?;
            let held = evaluate_heads(
                stack,
                split,
                &heads,
                &keys,
                &world,
                &config,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            let dev = evaluate_heads(
                stack,
                split,
                &heads,
                &keys,
                &world,
                &config,
                false,
                config.dev_seed,
                config.dev_episodes,
            )?;
            arms.push(ArmResult {
                split,
                kind,
                learnable_parameters: heads.parameter_count(),
                held,
                dev,
                history: json!(records),
                plateau: None,
            });
        }
    }

    let decisions = decision_rules(&arms);
    let guard_passes = arms
        .iter()
        .filter(|arm| arm.kind == AtomKind::TwoI)
        .all(|arm| arm.dev.totals.key_accuracy().unwrap_or(0.0) >= 0.98);
    let report = json!({
        "schema": "uor-r4.g2-key-probe/1",
        "trunk": {
            "directory": trunk_directory.display().to_string(),
            "model_safetensors_sha256": trunk_sha256,
            "pinned_sha256": TRUNK_SHA256,
            "splits": splits,
            "loaded_through": "AermModel::from_stack",
            "transport_snap_restored": snap_restored,
            "layers": stack.config.layers(),
            "width": stack.config.width,
        },
        "roots": {
            "snap": TransportSnap::Icosian.name(),
            "root_count": TransportSnap::Icosian.roots().len(),
            "relation_indices": roots.relation_indices,
            "relation_bound": roots.relation_bound,
            "relation_root_max_inner": RELATION_ROOT_MAX_INNER,
            "entity_head": "plain 8-way softmax in both arms (no entity roots)",
            "deviation": "the pre-registered 22 roots at pairwise |q.r| <= 1/2 are infeasible: the largest such icosian set has 12 (the 2T antipodal quotient; see the test); only the 14 relation atoms carry 2I roots, at the smallest achievable bound",
        },
        "keys": {
            "vocabulary_size": stack.config.vocab_size,
            "vocab_prime_ceiling": keys.vocab_prime_ceiling,
            "entity_primes": keys.entity,
            "relation_primes": keys.relation,
            "registry": "uor-r4-token primes are the first vocab_size primes; atom primes are the next 22 above the ceiling, validated by uor_r4_core::prime_route_attention::PrimeAtom and combined by SemiprimeExpert",
        },
        "config": {
            "steps": config.steps,
            "batch": config.batch,
            "context": config.context,
            "learning_rate": config.learning_rate,
            "warmup": config.warmup,
            "weight_decay": config.weight_decay,
            "clip": config.clip,
            "trigger_positive": config.trigger_positive,
            "eval_episodes": config.eval_episodes,
            "eval_batch": config.eval_batch,
            "dev_episodes": config.dev_episodes,
            "seed": config.seed,
            "data_seed": config.data_seed,
            "eval_seed": config.eval_seed,
            "dev_seed": config.dev_seed,
        },
        "arms": arms.iter().map(arm_json).collect::<Vec<_>>(),
        "guard": {
            "in_distribution_key_accuracy_at_least": 0.98,
            "passes": guard_passes,
        },
        "decisions": decisions,
        "parameter_comparison": {
            "2i_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::TwoI).map(|arm| arm.learnable_parameters),
            "softmax_learnable_parameters": arms.iter().find(|arm| arm.kind == AtomKind::Softmax).map(|arm| arm.learnable_parameters),
        },
    });
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn roots_mode(arguments: &[String]) -> Result<()> {
    let roots = select_roots()?;
    let value = json!({
        "snap": TransportSnap::Icosian.name(),
        "relation_indices": roots.relation_indices,
        "relation_roots": roots.relation,
        "relation_bound": roots.relation_bound,
        "relation_root_max_inner": RELATION_ROOT_MAX_INNER,
    });
    println!("{}", serde_json::to_string_pretty(&value)?);
    let _ = arguments;
    Ok(())
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn execute(arguments: &[String]) -> Result<()> {
    let (mode, rest) = arguments
        .split_first()
        .ok_or_else(|| invalid("usage: aerm-keys {run|run-cached|decode|roots} ..."))?;
    match mode.as_str() {
        "run" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "trunk",
                    "tokenizer",
                    "splits",
                    "steps",
                    "batch",
                    "context",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "clip",
                    "trigger_positive",
                    "eval_episodes",
                    "eval_batch",
                    "dev_episodes",
                    "seed",
                    "data_seed",
                    "eval_seed",
                    "dev_seed",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = run(&args, &out);
            finish(&out, result)
        }
        "run-cached" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "trunk",
                    "tokenizer",
                    "splits",
                    "steps",
                    "train_episodes",
                    "batch",
                    "context",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "clip",
                    "trigger_positive",
                    "eval_episodes",
                    "eval_batch",
                    "dev_episodes",
                    "seed",
                    "data_seed",
                    "eval_seed",
                    "dev_seed",
                    "plateau_window",
                    "record_every",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = run_cached(&args, &out);
            finish(&out, result)
        }
        "decode" => {
            let args = Args::parse(
                rest,
                &[
                    "out",
                    "trunk",
                    "tokenizer",
                    "splits",
                    "steps",
                    "train_episodes",
                    "eval_episodes",
                    "dev_episodes",
                    "batch",
                    "eval_batch",
                    "context",
                    "lr",
                    "warmup",
                    "weight_decay",
                    "clip",
                    "seed",
                    "data_seed",
                    "eval_seed",
                    "dev_seed",
                ],
            )?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = decode(&args, &out);
            finish(&out, result)
        }
        "roots" => roots_mode(rest),
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = execute(&arguments) {
        eprintln!("aerm-keys: {error}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Focused tests (no heavy fit).

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashMap};

    use uor_r4_training::geometric_stack::{ReadScore, StackArch, StackConfig};

    use super::*;

    fn toy_encoder() -> impl Fn(&str) -> Vec<u32> {
        let vocab: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
        move |text: &str| {
            let mut pieces: Vec<String> = Vec::new();
            let mut current = String::new();
            for ch in text.chars() {
                let boundary = ch == ' ' || ch == '\'' || ch == '\n' || ch.is_ascii_punctuation();
                if boundary && !current.is_empty() && !(current == "'" && ch.is_alphabetic()) {
                    pieces.push(std::mem::take(&mut current));
                }
                current.push(ch);
                if ch.is_ascii_punctuation() && ch != '\'' {
                    pieces.push(std::mem::take(&mut current));
                }
            }
            if !current.is_empty() {
                pieces.push(current);
            }
            let mut vocab = vocab.borrow_mut();
            pieces
                .into_iter()
                .map(|piece| {
                    let next = vocab.len() as u32 + 3;
                    *vocab.entry(piece).or_insert(next)
                })
                .collect()
        }
    }

    fn natural_world() -> Result<RelationWorld> {
        RelationWorld::natural(&toy_encoder(), 0, 1)
    }

    fn tiny_stack() -> Result<StackModel> {
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 4096,
            width: FEATURE_WIDTH,
            heads: 4,
            mlp_hidden: 64,
            context: 256,
            pattern: "rar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 7,
            memory: None,
        };
        config.validate()?;
        StackModel::new(config, &Device::Cpu)
    }

    fn tiny_config() -> ProbeConfig {
        ProbeConfig {
            steps: 2,
            batch: 2,
            context: 256,
            learning_rate: 0.01,
            warmup: 1,
            weight_decay: 0.0,
            clip: 1.0,
            trigger_positive: 5.0,
            eval_episodes: 4,
            eval_batch: 2,
            dev_episodes: 2,
            seed: 11,
            data_seed: 13,
            eval_seed: 17,
            dev_seed: 19,
        }
    }

    #[test]
    fn relation_roots_are_complete_at_the_registered_bound() -> Result<()> {
        let icosians = TransportSnap::Icosian.roots();
        let roots = select_roots()?;
        assert_eq!(roots.relation.len(), RELATION_ATOMS);
        assert_eq!(roots.relation_indices.len(), RELATION_ATOMS);
        for (index, root) in roots.relation.iter().enumerate() {
            let norm = dot(root, root).sqrt();
            assert!(
                (norm - 1.0).abs() <= 1e-5,
                "root {index} is not unit: {norm}"
            );
        }
        assert_eq!(
            pair_bound(&roots.relation_indices, icosians),
            roots.relation_bound
        );
        assert!(
            (roots.relation_bound - RELATION_ROOT_MAX_INNER).abs() <= ROOT_BOUND_TOLERANCE,
            "the greedy min-max search achieved {}, not the registered {}",
            roots.relation_bound,
            RELATION_ROOT_MAX_INNER
        );
        Ok(())
    }

    #[test]
    fn fourteen_relation_roots_cannot_meet_the_half_bound() {
        let icosians = TransportSnap::Icosian.roots();
        let maximum = max_compatible_set(icosians, 0.5);
        assert_eq!(
            maximum, 12,
            "the pre-registration assumed 14 roots exist at |q.r| <= 1/2"
        );
        assert!(maximum < RELATION_ATOMS);
    }

    #[test]
    fn semiprime_keys_are_order_independent_and_collision_free() -> Result<()> {
        let keys = AtomKeys::new(4096)?;
        let mut products = BTreeSet::new();
        for entity in 1..=ENTITY_ATOMS as u32 {
            for relation in 1..=RELATION_ATOMS as u32 {
                let product = keys.semiprime(entity, relation)?;
                assert!(
                    products.insert(product),
                    "semiprime collision at ({entity}, {relation}) = {product}"
                );
            }
        }
        assert_eq!(products.len(), ENTITY_ATOMS * RELATION_ATOMS);

        let left =
            PrimeAtom::new(keys.entity[0] as u32).map_err(|error| invalid(format!("{error}")))?;
        let right =
            PrimeAtom::new(keys.relation[0] as u32).map_err(|error| invalid(format!("{error}")))?;
        assert_eq!(
            SemiprimeExpert::new(left, right)
                .map_err(|error| invalid(format!("{error}")))?
                .product(),
            SemiprimeExpert::new(right, left)
                .map_err(|error| invalid(format!("{error}")))?
                .product()
        );

        let vocabulary: BTreeSet<u64> = first_primes(4096)?.into_iter().collect();
        assert!(
            products.iter().all(|product| !vocabulary.contains(product)),
            "an atom semiprime collided with a vocabulary prime"
        );
        assert!(keys
            .entity
            .iter()
            .chain(&keys.relation)
            .all(|prime| !vocabulary.contains(prime)));
        Ok(())
    }

    #[test]
    fn head_step_leaves_the_frozen_trunk_without_gradient() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let heads = ProbeHeads::new(
            AtomKind::TwoI,
            &roots,
            stack.config.width,
            23,
            stack.device(),
        )?;
        let config = tiny_config();
        let world = natural_world()?;
        let mut rng = Rng::new(config.data_seed);
        let batch = world.batch(&mut rng, config.batch, config.context, false)?;
        let loss = head_losses(&stack, 1, &heads, &batch, &config)?;
        let grads = loss.backward()?;
        for (name, var) in stack.variables() {
            assert!(
                grads.get(var.as_tensor()).is_none(),
                "trunk variable {name} received a gradient from a head step"
            );
        }
        let (decayed, plain) = heads.optimizer_groups();
        assert!(decayed
            .iter()
            .any(|var| grads.get(var.as_tensor()).is_some()));

        let before = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
        let params = |weight_decay| ParamsAdamW {
            lr: 0.01,
            beta1: 0.9,
            beta2: 0.95,
            eps: 1e-8,
            weight_decay,
        };
        let mut step_grads = grads;
        clip_gradients(&mut step_grads, &all, 1.0)?;
        AdamW::new(decayed, params(0.0))?.step(&step_grads)?;
        AdamW::new(plain, params(0.0))?.step(&step_grads)?;
        let after = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(before, after, "an optimizer step changed the frozen trunk");
        Ok(())
    }

    #[test]
    fn decode_probe_smoke_runs_end_to_end() -> Result<()> {
        let stack = tiny_stack()?;
        let world = natural_world()?;
        let config = DecodeConfig {
            steps: 4,
            train_episodes: 4,
            eval_episodes: 4,
            dev_episodes: 2,
            batch: 2,
            eval_batch: 2,
            context: 256,
            learning_rate: 0.01,
            warmup: 1,
            weight_decay: 0.0,
            clip: 10.0,
            seed: 11,
            data_seed: 13,
            eval_seed: 17,
            dev_seed: 19,
        };
        let train = collect_atom_sample(&stack, 1, &world, &config, false, config.data_seed, 4)?;
        let dev = collect_atom_sample(&stack, 1, &world, &config, false, config.dev_seed, 2)?;
        let held = collect_atom_sample(&stack, 1, &world, &config, true, config.eval_seed, 4)?;
        assert_eq!(train.entity_features.dims()[0], train.entity_targets.len());
        assert_eq!(
            train.relation_features.dims()[0],
            train.relation_targets.len()
        );
        assert!(train
            .entity_targets
            .iter()
            .all(|&t| t < ENTITY_ATOMS as u32));
        assert!(train
            .relation_targets
            .iter()
            .all(|&t| t < RELATION_ATOMS as u32));
        let device = stack.device().clone();
        let width = stack.config.width;
        let (entity, entity_history, entity_plateau) = fit_decode_probe(
            &train.entity_features,
            &train.entity_targets,
            ENTITY_ATOMS,
            width,
            23,
            &config,
            &device,
        )?;
        let (relation, _, _) = fit_decode_probe(
            &train.relation_features,
            &train.relation_targets,
            RELATION_ATOMS,
            width,
            29,
            &config,
            &device,
        )?;
        assert_eq!(entity_history.len(), 2);
        assert!(entity_plateau.window >= 1);
        for (head, features, targets) in [
            (&entity, &dev.entity_features, &dev.entity_targets),
            (&relation, &held.relation_features, &held.relation_targets),
        ] {
            let accuracy = head_accuracy(head, features, targets)?;
            assert!((0.0..=1.0).contains(&accuracy));
        }
        let baseline = majority_baseline(&train.entity_targets, &dev.entity_targets, ENTITY_ATOMS)?;
        assert!((0.0..=1.0).contains(&baseline));
        // The decode fit must not touch the frozen trunk.
        let before = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let (entity_again, _, _) = fit_decode_probe(
            &train.entity_features,
            &train.entity_targets,
            ENTITY_ATOMS,
            width,
            23,
            &config,
            &device,
        )?;
        let _ = entity_again;
        let after = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(
            before, after,
            "a decode probe step changed the frozen trunk"
        );
        Ok(())
    }

    #[test]
    fn cached_probe_smoke_runs_end_to_end() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let keys = AtomKeys::new(stack.config.vocab_size)?;
        let world = natural_world()?;
        let config = CachedConfig {
            steps: 6,
            train_episodes: 4,
            batch: 2,
            context: 256,
            learning_rate: 0.01,
            warmup: 1,
            weight_decay: 0.0,
            clip: 10.0,
            trigger_positive: 5.0,
            eval_episodes: 4,
            eval_batch: 2,
            dev_episodes: 2,
            seed: 11,
            data_seed: 13,
            eval_seed: 17,
            dev_seed: 19,
            plateau_window: 2,
            record_every: 1,
        };
        let cache = collect_train_cache(&stack, 1, &world, &config)?;
        assert_eq!(cache.features.dims()[0], cache.tag_targets.len());
        assert_eq!(cache.features.dims()[0], cache.triggers.len());
        assert_eq!(cache.entity_features.dims()[0], cache.entity_targets.len());
        assert_eq!(
            cache.relation_features.dims()[0],
            cache.relation_targets.len()
        );
        let before = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let device = stack.device().clone();
        let width = stack.config.width;
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, history, plateau) =
                train_heads_cached(width, kind, &roots, &cache, &config, &device)?;
            assert!(!history.is_empty());
            assert!(plateau.steps_run >= 1 && plateau.steps_run <= config.steps);
            for accuracy in [
                plateau.final_tag_accuracy,
                plateau.final_trigger_accuracy,
                plateau.final_entity_accuracy,
                plateau.final_relation_accuracy,
            ] {
                assert!((0.0..=1.0).contains(&accuracy));
            }
            let held_episodes = collect_eval_episodes(
                &stack,
                1,
                &heads,
                &world,
                config.context,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            let held = evaluate_cached_episodes(&keys, &held_episodes)?;
            assert!(held.totals.clauses > 0, "no held clauses were scored");
            assert!(held.totals.reads > 0, "no held reads were scored");
        }
        let after = stack
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("the tiny stack has no variables"))?
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(
            before, after,
            "a cached probe step changed the frozen trunk"
        );
        Ok(())
    }

    #[test]
    fn cached_loss_and_merged_grads_match_single_graph() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let world = natural_world()?;
        let config = CachedConfig {
            steps: 2,
            train_episodes: 4,
            batch: 2,
            context: 256,
            learning_rate: 0.01,
            warmup: 1,
            weight_decay: 0.0,
            clip: 10.0,
            trigger_positive: 5.0,
            eval_episodes: 2,
            eval_batch: 2,
            dev_episodes: 2,
            seed: 11,
            data_seed: 13,
            eval_seed: 17,
            dev_seed: 19,
            plateau_window: 2,
            record_every: 1,
        };
        let cache = collect_train_cache(&stack, 1, &world, &config)?;
        let device = stack.device().clone();
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let heads = ProbeHeads::new(kind, &roots, stack.config.width, config.seed, &device)?;
            let (decayed, plain) = heads.optimizer_groups();
            let all: Vec<Var> = decayed.iter().chain(&plain).cloned().collect();
            let (loss_value, merged) = cached_head_loss_and_grads(&heads, &cache, &all)?;
            let pieces = cached_head_loss_pieces(&heads, &cache)?;
            let mut total = pieces[0].clone();
            for piece in &pieces[1..] {
                total = (&total + piece)?;
            }
            let single_value = f64::from(total.to_scalar::<f32>()?);
            assert!(
                (loss_value - single_value).abs() <= 1e-6,
                "merged loss {loss_value} != single-graph loss {single_value}"
            );
            let single = total.backward()?;
            for var in &all {
                let left = merged
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("the merged store lacks a gradient"))?;
                let right = single
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("the single store lacks a gradient"))?;
                let difference = (left - right)?.abs()?.max_all()?.to_scalar::<f32>()?;
                assert!(
                    difference <= 1e-5,
                    "merged and single-graph gradients differ by {difference}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn two_update_smoke_runs_end_to_end() -> Result<()> {
        let stack = tiny_stack()?;
        let roots = select_roots()?;
        let keys = AtomKeys::new(stack.config.vocab_size)?;
        let world = natural_world()?;
        let config = tiny_config();
        for kind in [AtomKind::TwoI, AtomKind::Softmax] {
            let (heads, records) = train_heads(&stack, 1, kind, &roots, &world, &config)?;
            assert!(!records.is_empty());
            let held = evaluate_heads(
                &stack,
                1,
                &heads,
                &keys,
                &world,
                &config,
                true,
                config.eval_seed,
                config.eval_episodes,
            )?;
            assert!(held.episodes >= config.eval_episodes.min(1));
            assert!(held.totals.clauses > 0, "no held clauses were scored");
            assert!(held.totals.reads > 0, "no held reads were scored");
            assert!(heads.parameter_count() > 0);
        }
        Ok(())
    }
}
