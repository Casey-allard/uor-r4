//! MQAR read diagnostic (References #820): train a small geometric stack
//! (`uor_r4_training::geometric_stack`) on synthetic multi-query associative
//! recall and measure in-context recall accuracy by query distance.
//!
//! ```text
//! mqar-bench out=NEW_REPORT_ROOT [device=cpu|metal|cuda] [context=512] [pairs_per_bucket=8] \
//!   [batch=8] [steps=1800] [lr=0.001] [warmup=100] [min_lr=0.1] [weight_decay=0.1] [clip=1.0] \
//!   [eval_every=100] [curve_sequences=8] [final_sequences=64] [seed=1] [max_seconds=1200] \
//!   [probe_steps=0,600,final|none] [probe_sequences=16] [save_model=false|true] \
//!   [dump_scores=false|true] \
//!   [mode=train|task-baselines] [layout=synthetic|fact] FACT OPTIONS [arm=stack] ARM OPTIONS
//! arm=stack: [pattern=aaaaaa] [read=l2|dot|lorentz] [rotation=true|false] [width=128] [heads=4] \
//!   [mlp=384] [age=default|flat|spread] [key_shift=false|true] \
//!   [lineage=none|f2|qk|qk_jj|identity|so4|conv|wprev] [learned_init=lag1|zero]
//! layout=fact: tokenizer=TOKENIZER_JSON [gaps=0,1,2,3] [forms=rehearse,bare]
//!   (context <= 384; defaults pairs_per_bucket=4, final_sequences=256)
//! mqar-bench mode=decide runs=DIR_OF_SEALED_FACT_ROOTS out=NEW_REPORT_ROOT
//! ```
//!
//! `layout=fact` (Step 2, the deployment-parity bench) and `mode=decide` (its
//! frozen decision table) are documented in `mqar_bench_step2/fact.rs` and
//! `mqar_bench_step2/decide.rs`. `lineage` selects the key/query lineage arm
//! (`StackModel::set_read_key_shift` for `f2`, `StackModel::set_read_lineage`
//! for the research-only controls); `key_shift=true` is kept as `lineage=f2`.
//!
//! `dump_scores=1` is a TEMPORARY T2 diagnostic: after the final evaluation it
//! writes the read's own softmax weight rows on the held-out set (every read
//! layer and head, at each item's first-content predicting position) into
//! `dump_scores/` under the report root, plus the gold source index, the
//! item's landmarks and the model's argmax there. It trains nothing, changes
//! no parameter and moves no tally; the default (`dump_scores=0`) path writes
//! no such file.
//!
//! Each sequence is a fixed-length window of filler tokens holding
//! `pairs_per_bucket` key-value pairs per distance bucket. A pair writes its
//! key at position `p` and its value at `p + 1`; its single query writes the key
//! again at `q = p + d`, and the training target at `q` (the only weighted
//! position) is the value. The distance `d = q - p` is drawn uniformly inside
//! the pair's bucket, so no fixed offset reaches the key. Keys, values and
//! filler come from disjoint token ranges; keys and values are distinct inside
//! a sequence. Every sequence draws fresh pairings from its own seed, so a key's
//! value cannot be memorized: a pairing recurs across sequences only at the rate
//! of independent uniform draws.
//!
//! Two final evaluations use fresh seeds. The primary one (`in_class`) draws
//! pairings from the training pairing classes. The pairing space is split by
//! `(key_index + value_index) % 4`; training never draws class 0, and the
//! second evaluation (`held_out_class`) draws only class 0. It is adversarial:
//! a model that learns the class rule as a prior is penalized there even when
//! it reads the context, so it measures prior-over-context, not recall alone.
//!
//! The arm (the context-access mechanism under test) is defined separately
//! from the task, training loop and scoring: see `ContextArm` and `ArmSpec`.
//! Each report records the arm in a fixed schema (`REPORT_SCHEMA`), including
//! its context-access cost in positions scored per query token.
//!
//! The report root is claimed exclusively before any model work and sealed
//! with its manifest at the end. Offline floating-point training only; nothing
//! here is a serving path, and there is no transformer control.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde_json::{json, Value};
use uor_r4_core::report_output;
#[cfg(test)]
use uor_r4_training::geometric_stack::quaternion_j_left;
use uor_r4_training::geometric_stack::{
    parse_pointer_route, PointerConfig, PrimeRoute, ReadBinding, ReadBindingTarget, ReadLineage,
    ReadScore, RotationGroup, StackAdamW, StackArch, StackConfig, StackModel,
};
use uor_r4_training::{Result, TrainingError};

#[path = "mqar_bench_step2/decide.rs"]
mod decide;
#[path = "mqar_bench_step2/fact.rs"]
mod fact;

const VOCAB: usize = 512;
/// Filler tokens: noise between the pairs and queries.
const FILLER: (u32, u32) = (8, 64);
/// Key tokens.
const KEYS: (u32, u32) = (64, 288);
/// Value tokens.
const VALUES: (u32, u32) = (288, 512);
/// Pairing classes `(key_index + value_index) % CLASSES`; class 0 is held out.
const CLASSES: u32 = 4;
const HELD_OUT_CLASS: u32 = 0;
/// Rejection-sampling attempts per pair placement.
const PLACEMENT_ATTEMPTS: usize = 10_000;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

/// An inclusive range of query-to-key distances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bucket {
    name: &'static str,
    low: usize,
    high: usize,
}

const ALL_BUCKETS: [Bucket; 5] = [
    Bucket {
        name: "d16",
        low: 12,
        high: 24,
    },
    Bucket {
        name: "d64",
        low: 48,
        high: 96,
    },
    Bucket {
        name: "d200",
        low: 150,
        high: 300,
    },
    Bucket {
        name: "d400",
        low: 350,
        high: 480,
    },
    Bucket {
        name: "d1000",
        low: 1000,
        high: 1100,
    },
];

/// The buckets a window of `context` tokens can hold (a query at `p + d`
/// needs `d <= context - 1`).
fn buckets_for(context: usize) -> Vec<Bucket> {
    ALL_BUCKETS
        .iter()
        .copied()
        .filter(|bucket| bucket.high < context)
        .collect()
}

/// SplitMix64.
#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64, domain: u64, index: u64) -> Self {
        let mut rng = Rng(seed ^ domain.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        rng.0 ^= rng
            .next()
            .wrapping_add(index.wrapping_mul(0xD1B5_4A32_D192_ED03));
        rng.next();
        rng
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `low..=high`.
    fn range(&mut self, low: usize, high: usize) -> usize {
        low + (self.next() % (high - low + 1) as u64) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pairing {
    /// Classes other than the held-out one.
    Train,
    /// The held-out class only.
    HeldOut,
}

impl Pairing {
    fn admits(self, key: u32, value: u32) -> bool {
        let class = ((key - KEYS.0) + (value - VALUES.0)) % CLASSES;
        match self {
            Pairing::Train => class != HELD_OUT_CLASS,
            Pairing::HeldOut => class == HELD_OUT_CLASS,
        }
    }
}

#[derive(Clone, Debug)]
struct Query {
    position: usize,
    distance: usize,
    bucket: usize,
    key: u32,
    value: u32,
}

#[derive(Clone, Debug)]
struct Sequence {
    tokens: Vec<u32>,
    queries: Vec<Query>,
}

fn generate(
    rng: &mut Rng,
    context: usize,
    buckets: &[Bucket],
    pairs_per_bucket: usize,
    pairing: Pairing,
) -> Result<Sequence> {
    let pairs = buckets.len() * pairs_per_bucket;
    if pairs == 0
        || pairs > (KEYS.1 - KEYS.0) as usize
        || pairs > (VALUES.1 - VALUES.0) as usize
        || 3 * pairs > context / 2
    {
        return Err(invalid("too many or too few pairs for the window"));
    }
    let filler = (FILLER.1 - FILLER.0) as u64;
    let mut tokens: Vec<u32> = (0..context)
        .map(|_| FILLER.0 + (rng.next() % filler) as u32)
        .collect();
    let mut used = vec![false; context];
    let mut keys_used = vec![false; (KEYS.1 - KEYS.0) as usize];
    let mut values_used = vec![false; (VALUES.1 - VALUES.0) as usize];
    let mut queries = Vec::with_capacity(pairs);
    // The longest buckets first: their keys have the fewest admissible slots.
    let mut order: Vec<usize> = (0..buckets.len()).collect();
    order.sort_by_key(|&b| std::cmp::Reverse(buckets[b].high));
    for &b in &order {
        let bucket = buckets[b];
        for _ in 0..pairs_per_bucket {
            let key = loop {
                let index = (rng.next() % keys_used.len() as u64) as usize;
                if !keys_used[index] {
                    keys_used[index] = true;
                    break KEYS.0 + index as u32;
                }
            };
            let mut value = None;
            for _ in 0..PLACEMENT_ATTEMPTS {
                let index = (rng.next() % values_used.len() as u64) as usize;
                let candidate = VALUES.0 + index as u32;
                if !values_used[index] && pairing.admits(key, candidate) {
                    values_used[index] = true;
                    value = Some(candidate);
                    break;
                }
            }
            let value = value.ok_or_else(|| invalid("no admissible value for a key"))?;
            let mut placed = None;
            for _ in 0..PLACEMENT_ATTEMPTS {
                let distance = rng.range(bucket.low, bucket.high);
                let key_position = rng.range(0, context - 1 - distance);
                let query = key_position + distance;
                if !used[key_position] && !used[key_position + 1] && !used[query] {
                    placed = Some((key_position, distance));
                    break;
                }
            }
            let (key_position, distance) =
                placed.ok_or_else(|| invalid("could not place a pair in the window"))?;
            let position = key_position + distance;
            for p in [key_position, key_position + 1, position] {
                used[p] = true;
            }
            tokens[key_position] = key;
            tokens[key_position + 1] = value;
            tokens[position] = key;
            queries.push(Query {
                position,
                distance,
                bucket: b,
                key,
                value,
            });
        }
    }
    queries.sort_by_key(|query| query.position);
    Ok(Sequence { tokens, queries })
}

/// Inputs, targets and weights for `weighted_loss`: weight 1 on queries only.
fn batch_arrays(sequences: &[Sequence], context: usize) -> (Vec<u32>, Vec<u32>, Vec<f32>) {
    let total = sequences.len() * context;
    let (mut ids, mut targets, mut weights) = (
        Vec::with_capacity(total),
        vec![0u32; total],
        vec![0f32; total],
    );
    for (s, sequence) in sequences.iter().enumerate() {
        ids.extend_from_slice(&sequence.tokens);
        for query in &sequence.queries {
            targets[s * context + query.position] = query.value;
            weights[s * context + query.position] = 1.0;
        }
    }
    (ids, targets, weights)
}

#[derive(Clone, Default)]
struct Tally {
    correct: usize,
    total: usize,
    nll: f64,
    distance: usize,
    /// Wrong answers that are another pair's value in the same window.
    other_pair_value: usize,
    /// Wrong answers that are a value token absent from the window.
    absent_value: usize,
    /// Wrong answers that echo the query key.
    echoed_key: usize,
    /// Wrong answers that are the value written most recently before the
    /// query (by another pair): a recency read instead of a key match.
    most_recent_value: usize,
}

// ---------------------------------------------------------------------------
// Context-access arms. The task generator above, and the training loop and
// per-distance scoring below, see an arm only through `ContextArm`. To add an
// arm (for example a routed or admitted read), add an `ArmSpec` variant, its
// parse branch and its constructor in `ArmSpec::build`; nothing else changes.
// ---------------------------------------------------------------------------

/// What every arm gives the bench.
trait ContextArm {
    /// Stable arm kind, e.g. `geometric_stack`.
    fn kind(&self) -> &'static str;
    /// The arm's own configuration, recorded verbatim in the report.
    fn record(&self) -> Value;
    fn parameters(&self) -> usize;
    fn device(&self) -> &Device;
    /// Weighted next-token loss over `batch` windows of `time` tokens.
    fn loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: &[f32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor>;
    /// Logits [batch * time, vocabulary].
    fn logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor>;
    /// One optimizer update from the loss's gradients; returns the gradient norm.
    fn update(&mut self, loss: &Tensor, lr: f64) -> Result<f64>;
    /// Context positions the arm scores for the token at `position`, summed
    /// over its layers and heads (a recurrence scores none; a full causal read
    /// head scores `position + 1`).
    fn positions_scored(&self, position: usize) -> usize;
    /// What `positions_scored` counts, for the report.
    fn access_note(&self) -> String;
    /// Saves the arm's weights under `directory` (`save_model=true`); an arm
    /// without a saved form refuses.
    fn save(&self, directory: &Path) -> Result<()>;
    /// The (layer, head) read heads whose source weights the probe observes;
    /// empty for an arm without position-scoring reads.
    fn read_heads(&self) -> Vec<(usize, usize)> {
        Vec::new()
    }
    /// The read weight of head (`layer`, `head`) on each row's sources, for
    /// rows of (batch item, query position, source positions), at most one
    /// row per batch item. Observation only: it changes no logits.
    fn source_mass(
        &self,
        _ids: &[u32],
        _batch: usize,
        _time: usize,
        _layer: usize,
        _head: usize,
        _rows: &[(usize, usize, Vec<usize>)],
    ) -> Result<Vec<f32>> {
        Err(invalid("this arm has no read heads to probe"))
    }
    /// TEMPORARY (T2 `dump_scores=1`): the read's full softmax weight rows at
    /// declared `(batch, query)` positions, per read layer, and the logits of
    /// the same forward (see `StackModel::read_weight_rows`). Observation only.
    fn read_weight_rows(
        &self,
        _ids: &[u32],
        _batch: usize,
        _time: usize,
        _rows: &[(usize, usize)],
    ) -> Result<(Vec<(usize, Tensor)>, Tensor)> {
        Err(invalid("this arm has no read weight rows"))
    }
    /// Stage 0: the concrete stack, so the offline readout swap can hold every
    /// learned vector fixed and change only the score function. `None` for an
    /// arm whose readout cannot be swapped.
    fn stack_mut(&mut self) -> Option<&mut StackModel> {
        None
    }
}

/// Initial values of the reads' learned per-distance age bias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgeInit {
    /// The stack's own: head `h` of `H` starts at slope `2^(-8 (h + 1) / H)`.
    Default,
    /// Zero at every distance.
    Flat,
    /// F1: slopes spread geometrically, `2^(-2 - 4 h)` for `h < H - 1`, and
    /// exactly zero for the last head, so it reaches every distance at init.
    Spread,
}

impl AgeInit {
    fn name(self) -> &'static str {
        match self {
            AgeInit::Default => "default",
            AgeInit::Flat => "flat",
            AgeInit::Spread => "spread",
        }
    }

    /// The slope of `head` of `heads`, or `None` to keep the stack's own.
    fn slope(self, head: usize, heads: usize) -> Option<f64> {
        match self {
            AgeInit::Default => None,
            AgeInit::Flat => Some(0.0),
            AgeInit::Spread if head + 1 == heads => Some(0.0),
            AgeInit::Spread => Some(2f64.powi(-2 - 4 * head as i32)),
        }
    }
}

/// The key/query lineage of a stack arm (Step 2 arms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LineageArm {
    /// No lineage: the plain read keys.
    None,
    /// F2: `k_t + j k_{t-1}` (`StackModel::set_read_key_shift`, saveable).
    F2,
    /// A research-only lineage (`StackModel::set_read_lineage`).
    Research(ReadLineage),
}

impl LineageArm {
    fn name(self) -> &'static str {
        match self {
            LineageArm::None => "none",
            LineageArm::F2 => "f2",
            LineageArm::Research(lineage) => lineage.name(),
        }
    }

    /// `lineage=NAME`; `seed` seeds the random SO(4) map, `lag1` the learned
    /// controls' initial lag-1 channel (identity) or its absence (zero).
    fn parse(name: &str, seed: u64, lag1: bool) -> Result<Self> {
        Ok(match name {
            "none" => LineageArm::None,
            "f2" => LineageArm::F2,
            "qk" => LineageArm::Research(ReadLineage::QueryKeyJ),
            "qk_jj" => LineageArm::Research(ReadLineage::QueryKeyJSame),
            "identity" => LineageArm::Research(ReadLineage::IdentityShift),
            "so4" => LineageArm::Research(ReadLineage::RandomSo4 { seed }),
            "conv" => LineageArm::Research(ReadLineage::LearnedConv { init_lag1: lag1 }),
            "wprev" => LineageArm::Research(ReadLineage::LearnedPrev {
                init_identity: lag1,
            }),
            other => return Err(invalid(format!("invalid lineage={other}"))),
        })
    }

    fn record(self) -> Value {
        match self {
            LineageArm::None | LineageArm::F2 => json!({"name": self.name()}),
            LineageArm::Research(lineage) => json!({"name": self.name(), "spec": lineage}),
        }
    }
}

/// The arm registry: one variant per context-access mechanism.
#[derive(Clone, Debug)]
enum ArmSpec {
    /// `arm=stack`: the geometric stack with a layer pattern of `r`
    /// (quaternion recurrence) and `a` (read) letters.
    Stack {
        pattern: String,
        read: ReadScore,
        rotation: bool,
        width: usize,
        heads: usize,
        mlp_hidden: usize,
        age: AgeInit,
        /// The key/query lineage. F2 (`key_shift=true` or `lineage=f2`):
        /// every read key also carries the previous position's key, turned by
        /// the unit quaternion `j` (`StackModel::set_read_key_shift`).
        lineage: LineageArm,
    },
}

impl ArmSpec {
    fn parse(args: &mut Args, seed: u64) -> Result<Self> {
        let key_shift: bool = args.parsed("key_shift", false)?;
        let lag1 = match args.take("learned_init").as_deref() {
            None | Some("lag1") => true,
            Some("zero") => false,
            Some(other) => return Err(invalid(format!("invalid learned_init={other}"))),
        };
        let lineage = match args.take("lineage") {
            None if key_shift => LineageArm::F2,
            None => LineageArm::None,
            Some(name) => {
                let lineage = LineageArm::parse(&name, seed, lag1)?;
                if key_shift && lineage != LineageArm::F2 {
                    return Err(invalid("key_shift=true is lineage=f2; give one of them"));
                }
                lineage
            }
        };
        match args.take("arm").as_deref() {
            None | Some("stack") => Ok(ArmSpec::Stack {
                pattern: args.take("pattern").unwrap_or_else(|| "aaaaaa".into()),
                read: match args.take("read").as_deref() {
                    None | Some("l2") => ReadScore::L2,
                    Some("dot") => ReadScore::Dot,
                    Some("lorentz") => ReadScore::Lorentz,
                    Some(other) => return Err(invalid(format!("invalid read={other}"))),
                },
                rotation: args.parsed("rotation", true)?,
                width: args.parsed("width", 128usize)?,
                heads: args.parsed("heads", 4usize)?,
                mlp_hidden: args.parsed("mlp", 384usize)?,
                age: match args.take("age").as_deref() {
                    None | Some("default") => AgeInit::Default,
                    Some("flat") => AgeInit::Flat,
                    Some("spread") => AgeInit::Spread,
                    Some(other) => return Err(invalid(format!("invalid age={other}"))),
                },
                lineage,
            }),
            Some(other) => Err(invalid(format!("unknown arm={other}"))),
        }
    }

    /// A short label for logs and report paths.
    fn label(&self) -> String {
        match self {
            ArmSpec::Stack {
                pattern,
                read,
                rotation,
                age,
                lineage,
                ..
            } => format!(
                "stack-{pattern}-{read:?}{}{}{}",
                match age {
                    AgeInit::Default => String::new(),
                    other => format!("-{}-age", other.name()),
                },
                match lineage {
                    LineageArm::None => String::new(),
                    LineageArm::F2 => "-key-shift".into(),
                    other => format!("-lineage-{}", other.name()),
                },
                if *rotation { "" } else { "-no-rotation" },
            )
            .to_lowercase(),
        }
    }

    /// Validates without building, so a bad arm fails before the report claim.
    fn validate(&self, common: &Common) -> Result<()> {
        match self {
            ArmSpec::Stack {
                pattern,
                lineage,
                width,
                ..
            } => {
                if *lineage != LineageArm::None && !pattern.contains('a') {
                    return Err(invalid("a key/query lineage needs a read layer"));
                }
                if *lineage != LineageArm::None && !width.is_multiple_of(4) {
                    return Err(invalid("a key/query lineage needs four-channel lanes"));
                }
                self.stack_config(common)?.validate()
            }
        }
    }

    fn stack_config(&self, common: &Common) -> Result<StackConfig> {
        match self {
            ArmSpec::Stack {
                pattern,
                read,
                rotation,
                width,
                heads,
                mlp_hidden,
                ..
            } => {
                let config = StackConfig {
                    arch: StackArch::Geometric,
                    vocab_size: common.vocab,
                    width: *width,
                    heads: *heads,
                    mlp_hidden: *mlp_hidden,
                    context: common.context,
                    pattern: pattern.clone(),
                    read: *read,
                    rotation: *rotation,
                    rotation_group: RotationGroup::Quaternion,
                    seed: common.seed,
                    memory: None,
                    select: None,
                    pointer: None,
                };
                config.validate()?;
                Ok(config)
            }
        }
    }

    fn build(&self, common: &Common, device: &Device) -> Result<Box<dyn ContextArm>> {
        match self {
            ArmSpec::Stack { age, lineage, .. } => {
                let config = self.stack_config(common)?;
                let mut model = StackModel::new(config.clone(), device)?;
                // The pointer needs BOTH a head and a route: StackConfig carries a
                // PointerConfig (the head), the route is installed on the model. Both are off
                // unless `pointer=` is given, which is what every prior run did -- so the
                // geometric `ngram` successor route (longest ordered n-let match) has never
                // been exercised here. This is D20 section 2 condition 3 in its exact form.
                if let Some(route) = common.pointer.clone() {
                    model.add_pointer(PointerConfig::new(config.width), common.seed)?;
                    model.set_pointer_route(Some(route))?;
                }
                if *age != AgeInit::Default {
                    for (name, var) in model.variables() {
                        if !name.ends_with(".read.age") {
                            continue;
                        }
                        let mut values = Vec::with_capacity(config.heads * config.context);
                        for head in 0..config.heads {
                            let slope = age
                                .slope(head, config.heads)
                                .ok_or_else(|| invalid("age init without a slope"))?;
                            values.extend((0..config.context).map(|d| (-slope * d as f64) as f32));
                        }
                        var.set(&Tensor::from_vec(
                            values,
                            (config.heads, config.context),
                            device,
                        )?)?;
                    }
                }
                // Before the optimizer: learned lineages add variables.
                match lineage {
                    LineageArm::None => {}
                    LineageArm::F2 => model.set_read_key_shift(true)?,
                    LineageArm::Research(research) => model.set_read_lineage(Some(*research))?,
                }
                let optimizer = StackAdamW::new(&model, common.weight_decay, common.clip)?;
                Ok(Box::new(StackArm {
                    model,
                    optimizer,
                    age: *age,
                    lineage: *lineage,
                }))
            }
        }
    }
}

struct StackArm {
    model: StackModel,
    optimizer: StackAdamW,
    age: AgeInit,
    lineage: LineageArm,
}

impl StackArm {
    fn read_layers(&self) -> usize {
        self.model
            .config
            .pattern
            .chars()
            .filter(|&c| c == 'a')
            .count()
    }
}

impl ContextArm for StackArm {
    fn kind(&self) -> &'static str {
        "geometric_stack"
    }

    fn record(&self) -> Value {
        let heads = self.model.config.heads;
        json!({
            "stack_config": self.model.config,
            "age_init": self.age.name(),
            "age_init_slopes": (0..heads)
                .map(|h| self.age.slope(h, heads)
                    .unwrap_or_else(|| 2f64.powf(-8.0 * (h + 1) as f64 / heads as f64)))
                .collect::<Vec<_>>(),
            "read_key_shift": self.model.read_key_shift(),
            "lineage": self.lineage.name(),
            "lineage_record": self.lineage.record(),
            "read_layers": self.read_layers(),
            "recurrence_layers": self.model.config.layers() - self.read_layers(),
            "recurrent_state_floats_per_layer": self.model.config.width,
        })
    }

    fn parameters(&self) -> usize {
        self.model.parameter_count()
    }

    fn device(&self) -> &Device {
        self.model.device()
    }

    fn loss(
        &self,
        ids: &[u32],
        targets: &[u32],
        weights: &[f32],
        batch: usize,
        time: usize,
    ) -> Result<Tensor> {
        self.model.weighted_loss(ids, targets, weights, batch, time)
    }

    fn logits(&self, ids: &[u32], batch: usize, time: usize) -> Result<Tensor> {
        self.model.forward(ids, batch, time)
    }

    fn update(&mut self, loss: &Tensor, lr: f64) -> Result<f64> {
        let grads = loss.backward()?;
        self.optimizer.update(&self.model, &grads, lr)
    }

    fn positions_scored(&self, position: usize) -> usize {
        self.read_layers() * self.model.config.heads * (position + 1)
    }

    fn access_note(&self) -> String {
        "read layers x heads x (position + 1): every causal position is scored by each read head (plus one NoRead slot, not counted); recurrence layers score no positions".into()
    }

    fn save(&self, directory: &Path) -> Result<()> {
        self.model.save(directory)
    }

    fn read_heads(&self) -> Vec<(usize, usize)> {
        let config = &self.model.config;
        (0..config.layers())
            .filter(|&layer| config.pattern.as_bytes()[layer] == b'a')
            .flat_map(|layer| (0..config.heads).map(move |head| (layer, head)))
            .collect()
    }

    fn source_mass(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        layer: usize,
        head: usize,
        rows: &[(usize, usize, Vec<usize>)],
    ) -> Result<Vec<f32>> {
        let target = ReadBindingTarget {
            layer,
            head,
            rows: rows
                .iter()
                .map(|(batch, query, sources)| ReadBinding {
                    batch: *batch,
                    query: *query,
                    sources: sources.clone(),
                })
                .collect(),
        };
        Ok(self
            .model
            .read_binding_masses(ids, batch, time, &target)?
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?)
    }

    fn read_weight_rows(
        &self,
        ids: &[u32],
        batch: usize,
        time: usize,
        rows: &[(usize, usize)],
    ) -> Result<(Vec<(usize, Tensor)>, Tensor)> {
        self.model.read_weight_rows(ids, batch, time, rows)
    }

    fn stack_mut(&mut self) -> Option<&mut StackModel> {
        Some(&mut self.model)
    }
}

/// The deciding probe: each read head's softmax weight on the true key
/// position `q - d` and on the value position `q - d + 1` after it, by bucket.
/// One query per probe sequence and bucket (its first of that bucket).
fn read_probe(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
) -> Result<Value> {
    let heads = arm.read_heads();
    if heads.is_empty() {
        return Ok(json!({"status": "UNAVAILABLE: the arm has no read heads"}));
    }
    let (ids, _, _) = batch_arrays(sequences, context);
    let mut by_bucket = serde_json::Map::new();
    for (b, bucket) in buckets.iter().enumerate() {
        let picks: Vec<(usize, &Query)> = sequences
            .iter()
            .enumerate()
            .filter_map(|(s, sequence)| {
                sequence
                    .queries
                    .iter()
                    .find(|query| query.bucket == b)
                    .map(|query| (s, query))
            })
            .collect();
        let uniform = picks
            .iter()
            .map(|(_, query)| 1.0 / (query.position + 1) as f64)
            .sum::<f64>()
            / picks.len().max(1) as f64;
        let mut per_head = Vec::new();
        let (mut best_key, mut best_value) = (0f64, 0f64);
        for &(layer, head) in &heads {
            let mut means = [0f64; 2];
            let mut over_half = [0usize; 2];
            for (which, offset) in [(0usize, 0usize), (1, 1)] {
                let rows: Vec<(usize, usize, Vec<usize>)> = picks
                    .iter()
                    .map(|(s, query)| {
                        (
                            *s,
                            query.position,
                            vec![query.position - query.distance + offset],
                        )
                    })
                    .collect();
                let masses = arm.source_mass(&ids, sequences.len(), context, layer, head, &rows)?;
                means[which] =
                    masses.iter().map(|&m| f64::from(m)).sum::<f64>() / masses.len().max(1) as f64;
                over_half[which] = masses.iter().filter(|&&m| m > 0.5).count();
            }
            best_key = best_key.max(means[0]);
            best_value = best_value.max(means[1]);
            per_head.push(json!({
                "layer": layer, "head": head,
                "mean_weight_on_key": means[0], "mean_weight_on_value": means[1],
                "queries_over_half_on_key": over_half[0],
                "queries_over_half_on_value": over_half[1],
            }));
        }
        by_bucket.insert(
            bucket.name.into(),
            json!({
                "queries": picks.len(),
                "uniform_weight_reference": uniform,
                "best_head_mean_weight_on_key": best_key,
                "best_head_mean_weight_on_value": best_value,
                "heads": per_head,
            }),
        );
    }
    Ok(json!({"by_bucket": by_bucket}))
}

fn probe_line(probe: &Value, buckets: &[Bucket]) -> String {
    buckets
        .iter()
        .map(|b| {
            let entry = &probe["by_bucket"][b.name];
            format!(
                "{} key {:.3} value {:.3} (uniform {:.4})",
                b.name,
                entry["best_head_mean_weight_on_key"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                entry["best_head_mean_weight_on_value"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
                entry["uniform_weight_reference"]
                    .as_f64()
                    .unwrap_or(f64::NAN),
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The value written most recently before `query` (by any pair): what a pure
/// recency read with no key match would answer.
fn most_recent_value(sequence: &Sequence, query: &Query) -> Option<u32> {
    sequence
        .queries
        .iter()
        .map(|pair| (pair.position - pair.distance + 1, pair.value))
        .filter(|&(value_position, _)| value_position < query.position)
        .max_by_key(|&(value_position, _)| value_position)
        .map(|(_, value)| value)
}

/// Data-only baselines per bucket, independent of any arm: the accuracy of
/// answering with the most recently written value (recency, no key match).
fn task_baselines(sequences: &[Sequence], buckets: &[Bucket]) -> Value {
    let mut hits = vec![(0usize, 0usize); buckets.len()];
    for sequence in sequences {
        for query in &sequence.queries {
            let entry = &mut hits[query.bucket];
            entry.1 += 1;
            entry.0 += usize::from(most_recent_value(sequence, query) == Some(query.value));
        }
    }
    let mut by_bucket = serde_json::Map::new();
    for (bucket, (hit, total)) in buckets.iter().zip(&hits) {
        by_bucket.insert(
            bucket.name.into(),
            json!({"recency_accuracy": *hit as f64 / (*total).max(1) as f64, "total": total}),
        );
    }
    json!({"by_bucket": by_bucket, "rule": "answer the value written most recently before the query"})
}

/// Recall by bucket: argmax over the whole vocabulary at each query.
fn evaluate(
    arm: &dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
    chunk: usize,
) -> Result<Value> {
    let mut tallies = vec![Tally::default(); buckets.len()];
    let mut scored = 0usize;
    for group in sequences.chunks(chunk.max(1)) {
        let (ids, _, _) = batch_arrays(group, context);
        let logits = arm.logits(&ids, group.len(), context)?;
        let mut rows = Vec::new();
        for (s, sequence) in group.iter().enumerate() {
            for query in &sequence.queries {
                rows.push((s * context + query.position) as u32);
            }
        }
        let index = Tensor::from_vec(rows.clone(), rows.len(), arm.device())?;
        let selected = logits
            .index_select(&index, 0)?
            .to_device(&Device::Cpu)?
            .to_vec2::<f32>()?;
        let mut row = 0;
        for sequence in group {
            let in_window: Vec<u32> = sequence.queries.iter().map(|q| q.value).collect();
            for query in &sequence.queries {
                let scores = &selected[row];
                row += 1;
                scored += arm.positions_scored(query.position);
                let (argmax, maximum) =
                    scores
                        .iter()
                        .enumerate()
                        .fold((0usize, f32::NEG_INFINITY), |best, (i, &v)| {
                            if v > best.1 {
                                (i, v)
                            } else {
                                best
                            }
                        });
                let sum: f64 = scores.iter().map(|&v| f64::from(v - maximum).exp()).sum();
                let nll = sum.ln() - f64::from(scores[query.value as usize] - maximum);
                let tally = &mut tallies[query.bucket];
                tally.total += 1;
                tally.correct += usize::from(argmax == query.value as usize);
                tally.nll += nll;
                tally.distance += query.distance;
                // The most recent value written by another pair.
                let most_recent = sequence
                    .queries
                    .iter()
                    .filter(|other| {
                        other.value != query.value
                            && other.position - other.distance + 1 < query.position
                    })
                    .max_by_key(|other| other.position - other.distance)
                    .map(|other| other.value);
                let predicted = argmax as u32;
                if predicted != query.value {
                    tally.most_recent_value += usize::from(most_recent == Some(predicted));
                    if in_window.contains(&predicted) {
                        tally.other_pair_value += 1;
                    } else if (VALUES.0..VALUES.1).contains(&predicted) {
                        tally.absent_value += 1;
                    } else if predicted == query.key {
                        tally.echoed_key += 1;
                    }
                }
            }
        }
    }
    let mut by_bucket = serde_json::Map::new();
    let (mut correct, mut total) = (0, 0);
    for (bucket, tally) in buckets.iter().zip(&tallies) {
        correct += tally.correct;
        total += tally.total;
        by_bucket.insert(
            bucket.name.into(),
            json!({
                "distance_low": bucket.low, "distance_high": bucket.high,
                "correct": tally.correct, "total": tally.total,
                "accuracy": tally.correct as f64 / tally.total.max(1) as f64,
                "mean_nll": tally.nll / tally.total.max(1) as f64,
                "mean_distance": tally.distance as f64 / tally.total.max(1) as f64,
                "wrong_other_pair_value": tally.other_pair_value,
                "wrong_absent_value": tally.absent_value,
                "wrong_echoed_key": tally.echoed_key,
                "wrong_most_recent_pair_value": tally.most_recent_value,
            }),
        );
    }
    Ok(json!({
        "by_bucket": by_bucket,
        "correct": correct,
        "total": total,
        "accuracy": correct as f64 / total.max(1) as f64,
        "chance_accuracy": 1.0 / f64::from(VALUES.1 - VALUES.0),
        "positions_scored_per_query_token_mean": scored as f64 / total.max(1) as f64,
        "task_baselines": task_baselines(sequences, buckets),
    }))
}

// ---------------------------------------------------------------------------
// Stage 0 (offline, no training): hold the frozen q/k vectors and vary only
// the readout.
//
// A1 is the arm's own fused score, whatever `ReadScore` it trained with. A2
// re-runs the *same* forward with `ReadScore::Dot` over the same learned q/k
// projections (no geometry, no learned per-head scale/offset), with the
// learned per-distance age bias kept (A2a) and zeroed (A2b). A3 is the exact
// token-identity sieve over the same rows and the same candidate set. Every
// arm is ranked over the same causal sources, so the comparison is
// like-for-like; nothing here is a serving path and no logit is changed
// outside this function.
// ---------------------------------------------------------------------------

/// Sequences the stage-0 readout observes (the first of the final in-class
/// set). Kept small on purpose: the read's weight dump materializes a
/// `batch x heads x time x (value_width + time)` tensor per read layer, so the
/// diagnostic's peak memory scales with this number, not with the row count.
const STAGE0_SEQUENCES: usize = 6;

/// One query row's identity on the panel.
struct Stage0Row {
    sequence: usize,
    position: usize,
    distance: usize,
    value: u32,
    bucket: usize,
    /// Pairs whose key was written strictly before this query: the MQAR
    /// candidate count `N` whose `1/N` is this row's chance level.
    pairs_before: usize,
}

/// The rank of `gold` among the causal sources `0..=limit` by weight,
/// descending; rank 1 is a strictly highest weight (ties count against it).
fn stage0_rank(row: &[f32], gold: usize, limit: usize) -> usize {
    let g = row[gold];
    1 + (0..=limit.min(row.len() - 1))
        .filter(|&j| j != gold && row[j] > g)
        .count()
}

#[derive(Default, Clone)]
struct Stage0Tally {
    rows: usize,
    head_rows: usize,
    top1_key: usize,
    top1_value: usize,
    r4_key: usize,
    r16_key: usize,
    r4_value: usize,
    r16_value: usize,
    rows_any_head_top1_key: usize,
    noread_mass_sum: f64,
    pairs_before_sum: usize,
    positions_sum: usize,
}

impl Stage0Tally {
    fn add(&mut self, other: &Stage0Tally) {
        self.rows += other.rows;
        self.head_rows += other.head_rows;
        self.top1_key += other.top1_key;
        self.top1_value += other.top1_value;
        self.r4_key += other.r4_key;
        self.r16_key += other.r16_key;
        self.r4_value += other.r4_value;
        self.r16_value += other.r16_value;
        self.rows_any_head_top1_key += other.rows_any_head_top1_key;
        self.noread_mass_sum += other.noread_mass_sum;
        self.pairs_before_sum += other.pairs_before_sum;
        self.positions_sum += other.positions_sum;
    }

    fn json(&self, note: &str) -> Value {
        let hr = self.head_rows.max(1) as f64;
        json!({
            "note": note,
            "sequences": STAGE0_SEQUENCES,
            "rows": self.rows,
            "head_rows": self.head_rows,
            "gold_key_top1": self.top1_key as f64 / hr,
            "gold_key_recall4": self.r4_key as f64 / hr,
            "gold_key_recall16": self.r16_key as f64 / hr,
            "gold_value_top1": self.top1_value as f64 / hr,
            "gold_value_recall4": self.r4_value as f64 / hr,
            "gold_value_recall16": self.r16_value as f64 / hr,
            "rows_any_head_gold_key_top1": self.rows_any_head_top1_key as f64
                / self.rows.max(1) as f64,
            "mean_noread_mass": self.noread_mass_sum / hr,
            "mean_source_mass": 1.0 - self.noread_mass_sum / hr,
            "mean_pairs_before_query": self.pairs_before_sum as f64 / self.rows.max(1) as f64,
            "chance_gold_key_top1": 1.0 / (self.pairs_before_sum as f64 / self.rows.max(1) as f64),
            "mean_causal_positions_ranked": self.positions_sum as f64 / self.rows.max(1) as f64,
        })
    }
}

/// One row's outcome under one arm: the bootstrap unit is the query row.
#[derive(Clone)]
struct Stage0RowOutcome {
    bucket: usize,
    pairs_before: usize,
    /// Fraction of read heads that rank the gold key 1st.
    top1_head_mean: f64,
    /// Fraction of read heads that rank the gold key in the top 16.
    r16_head_mean: f64,
    /// Any head ranks the gold key 1st.
    top1_any_head: bool,
    /// NoRead's share of the read's own softmax.
    noread_mass: f64,
}

/// Tally one weight dump (rows in `meta` order) over every read layer and head.
fn stage0_tally_dump(
    dump: &[(usize, Tensor)],
    meta: &[Stage0Row],
    buckets: usize,
) -> Result<(Stage0Tally, Vec<Stage0Tally>, Vec<Stage0RowOutcome>)> {
    let mut all = Stage0Tally::default();
    let mut per_bucket = vec![Stage0Tally::default(); buckets];
    let mut outcomes = Vec::with_capacity(meta.len());
    for (_layer, tensor) in dump {
        let weights = tensor.to_device(&Device::Cpu)?.to_vec3::<f32>()?;
        for (r, row_meta) in meta.iter().enumerate() {
            let heads = &weights[r];
            let limit = row_meta.position;
            let gold_key = row_meta.position - row_meta.distance;
            let gold_value = gold_key + 1;
            let mut any_top1 = false;
            let mut top1_heads = 0usize;
            let mut r16_heads = 0usize;
            let mut row_noread = 0f64;
            for head in heads {
                let rank_key = stage0_rank(head, gold_key, limit);
                let rank_value = stage0_rank(head, gold_value, limit);
                let mass: f32 = head[..=limit.min(head.len() - 1)].iter().sum();
                for tally in [&mut all, &mut per_bucket[row_meta.bucket]] {
                    tally.head_rows += 1;
                    tally.top1_key += usize::from(rank_key == 1);
                    tally.top1_value += usize::from(rank_value == 1);
                    tally.r4_key += usize::from(rank_key <= 4);
                    tally.r16_key += usize::from(rank_key <= 16);
                    tally.r4_value += usize::from(rank_value <= 4);
                    tally.r16_value += usize::from(rank_value <= 16);
                    tally.noread_mass_sum += 1.0 - f64::from(mass);
                }
                any_top1 |= rank_key == 1;
                top1_heads += usize::from(rank_key == 1);
                r16_heads += usize::from(rank_key <= 16);
                row_noread += 1.0 - f64::from(mass);
            }
            let head_count = heads.len().max(1) as f64;
            if *_layer == 0 {
                // The per-row bootstrap unit: one entry per query row, taken
                // from the first read layer (later layers add to the tallies).
                outcomes.push(Stage0RowOutcome {
                    bucket: row_meta.bucket,
                    pairs_before: row_meta.pairs_before,
                    top1_head_mean: top1_heads as f64 / head_count,
                    r16_head_mean: r16_heads as f64 / head_count,
                    top1_any_head: any_top1,
                    noread_mass: row_noread / head_count,
                });
            }
            for tally in [&mut all, &mut per_bucket[row_meta.bucket]] {
                tally.rows += 1;
                tally.rows_any_head_top1_key += usize::from(any_top1);
                tally.pairs_before_sum += row_meta.pairs_before;
                tally.positions_sum += limit + 1;
            }
        }
    }
    Ok((all, per_bucket, outcomes))
}

/// A3: the exact token-identity sieve over the same rows and candidates.
/// Admit every causal position whose token is the query's own key token,
/// rank the admitted set by recency (latest first), and read the value after
/// the winner. Keys are distinct inside a sequence, so the gold key position
/// is the unique admission.
fn stage0_sieve(
    ids: &[u32],
    set: &[Sequence],
    meta: &[Stage0Row],
    context: usize,
    buckets: usize,
) -> Result<(Value, Stage0Tally, Vec<Stage0Tally>, Vec<Stage0RowOutcome>)> {
    let _ = set;
    let mut all = Stage0Tally::default();
    let mut per_bucket = vec![Stage0Tally::default(); buckets];
    let mut outcomes = Vec::with_capacity(meta.len());
    let mut admitted_histogram = BTreeMap::new();
    let mut value_after_winner = 0usize;
    let mut winners = 0usize;
    for row_meta in meta {
        let base = row_meta.sequence * context;
        let query_token = ids[base + row_meta.position];
        let gold_key = row_meta.position - row_meta.distance;
        let mut admitted: Vec<usize> = (0..row_meta.position)
            .filter(|&j| ids[base + j] == query_token)
            .collect();
        admitted.sort_unstable();
        *admitted_histogram.entry(admitted.len()).or_insert(0usize) += 1;
        let winner = admitted.last().copied();
        let top1 = winner == Some(gold_key);
        let value_ok = winner.is_some_and(|w| {
            w + 1 < row_meta.position && ids[base + w + 1] == row_meta.value
        });
        winners += usize::from(winner.is_some());
        value_after_winner += usize::from(value_ok);
        outcomes.push(Stage0RowOutcome {
            bucket: row_meta.bucket,
            pairs_before: row_meta.pairs_before,
            top1_head_mean: f64::from(u8::from(top1)),
            r16_head_mean: f64::from(u8::from(top1)),
            top1_any_head: top1,
            // A3 is an exact sieve: it has no softmax and no NoRead slot.
            noread_mass: 0.0,
        });
        for tally in [&mut all, &mut per_bucket[row_meta.bucket]] {
            tally.rows += 1;
            tally.head_rows += 1;
            tally.top1_key += usize::from(top1);
            tally.top1_value += usize::from(value_ok);
            tally.r4_key += usize::from(top1);
            tally.r16_key += usize::from(top1);
            tally.r4_value += usize::from(value_ok);
            tally.r16_value += usize::from(value_ok);
            tally.rows_any_head_top1_key += usize::from(top1);
            tally.pairs_before_sum += row_meta.pairs_before;
            tally.positions_sum += row_meta.position + 1;
        }
    }
    Ok((
        json!({
            "rows": meta.len(),
            "rows_with_a_winner": winners,
            "admitted_matches_per_row_histogram": admitted_histogram,
            "unique_admission_rows": admitted_histogram.get(&1).copied().unwrap_or(0),
            "value_after_winner_rows": value_after_winner,
        }),
        all,
        per_bucket,
        outcomes,
    ))
}

/// Write a tensor's f32 payload to `path` as raw little-endian f32 and return a
/// manifest record for it. Deliberately dumb: the offline reader is Python and
/// re-derives nothing from the model, so every shape, count and range the
/// reader needs is recorded next to the bytes rather than assumed.
fn dump_f32(path: &Path, tensor: &Tensor) -> Result<Value> {
    let shape = tensor.dims().to_vec();
    let data = tensor
        .flatten_all()?
        .to_dtype(DType::F32)?
        .to_vec1::<f32>()?;
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for value in &data {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(path, &bytes)?;
    Ok(json!({
        "file": path.file_name().map(|n| n.to_string_lossy().to_string()),
        "shape": shape,
        "count": data.len(),
        "bytes": bytes.len(),
        "min": data.iter().copied().fold(f32::INFINITY, f32::min),
        "max": data.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    }))
}

/// Stack a per-layer weight dump (`[(layer, [rows, heads, time])]`) into one
/// `[layers, rows, heads, time]` tensor, and return the layer order with it: the
/// offline reader is given the order rather than trusting the dump's.
fn stack_layers(dump: &[(usize, Tensor)]) -> Result<(Tensor, Vec<usize>)> {
    let order: Vec<usize> = dump.iter().map(|(layer, _)| *layer).collect();
    let tensors: Vec<Tensor> = dump.iter().map(|(_, t)| t.clone()).collect();
    Ok((Tensor::stack(&tensors, 0)?, order))
}

fn stage0_readout(
    arm: &mut dyn ContextArm,
    sequences: &[Sequence],
    context: usize,
    buckets: &[Bucket],
    dump_dir: Option<&Path>,
) -> Result<Value> {
    if arm.read_heads().is_empty() {
        return Ok(json!({"status": "UNAVAILABLE: the arm has no read heads"}));
    }
    if arm.stack_mut().is_none() {
        return Ok(json!({
            "status": "UNAVAILABLE: the arm's readout cannot be swapped on frozen vectors"
        }));
    }
    let take = sequences.len().min(STAGE0_SEQUENCES);
    let set = &sequences[..take];
    let (ids, _, _) = batch_arrays(set, context);
    let mut rows: Vec<(usize, usize)> = Vec::new();
    let mut meta: Vec<Stage0Row> = Vec::new();
    for (b, sequence) in set.iter().enumerate() {
        for query in &sequence.queries {
            let pairs_before = sequence
                .queries
                .iter()
                .filter(|other| other.position - other.distance < query.position)
                .count();
            rows.push((b, query.position));
            meta.push(Stage0Row {
                sequence: b,
                position: query.position,
                distance: query.distance,
                value: query.value,
                bucket: query.bucket,
                pairs_before,
            });
        }
    }
    let nb = buckets.len();
    let batch = set.len();

    // A1: the arm's own fused score.
    let a1_dump = arm.read_weight_rows(&ids, batch, context, &rows)?;
    let layers = a1_dump.0.len();
    let (a1, a1_buckets, a1_rows) = stage0_tally_dump(&a1_dump.0, &meta, nb)?;

    // The frozen-vector dump (Stage-0 follow-up): the exact per-position query
    // and key projections the fused read scores with, plus its auxiliary
    // vector, so an offline readout can evaluate a score the model does not
    // itself implement -- cosine, and the norm/direction split -- on precisely
    // these vectors. Taken while the arm's *own* score is active: the
    // projections do not depend on the score function, only the score does, so
    // one dump serves every arm.
    let mut vector_dump = json!({"requested": false});
    if let Some(dir) = dump_dir {
        fs::create_dir_all(dir)?;
        let (read_score, qk, qk_logits) = {
            let model = arm
                .stack_mut()
                .ok_or_else(|| invalid("stage0 needs the stack arm"))?;
            let (qk, logits) = model.read_qk_rows(&ids, batch, context)?;
            (serde_json::to_value(model.config.read)?, qk, logits)
        };
        // Inertness, measured rather than asserted: `read_weight_rows` returns
        // the logits of its own (different capture, same forward) run. The qk
        // capture adds no value channel and no arithmetic, so the two must
        // agree exactly. If they do not, the dump is not observation only and
        // no number derived from it is interpretable.
        let inert_exact = {
            let a = qk_logits.flatten_all()?.to_vec1::<f32>()?;
            let b = a1_dump.1.flatten_all()?.to_vec1::<f32>()?;
            a.len() == b.len() && a == b
        };
        let mut per_layer = Vec::new();
        for (layer, row) in &qk {
            per_layer.push(json!({
                "layer": layer,
                "query": dump_f32(&dir.join(format!("layer{layer}.query.f32")), &row.query)?,
                "key": dump_f32(&dir.join(format!("layer{layer}.key.f32")), &row.key)?,
                "aux": dump_f32(&dir.join(format!("layer{layer}.aux.f32")), &row.aux)?,
            }));
        }
        let (l2_weights, weight_order) = stack_layers(&a1_dump.0)?;
        let l2_weights = dump_f32(&dir.join("weights_l2.f32"), &l2_weights)?;
        write_json(
            &dir.join("vectors.json"),
            &json!({
                "schema": "uor-r4.mqar-bench.stage0-vectors/1",
                "note": "raw frozen read inputs; the offline readout swaps only the score function",
                "arm_read_score": read_score,
                "sequences": take,
                "batch": batch,
                "time": context,
                "heads": qk.first().map(|(_, row)| row.query.dims4().map(|d| d.1)).transpose()?,
                "qk_layers": per_layer,
                "qk_logits_equal_to_weight_dump_logits": inert_exact,
                "weights_l2": {"dump": l2_weights, "layer_order": weight_order},
                "rows": meta.iter().map(|row| json!({
                    "sequence": row.sequence,
                    "position": row.position,
                    "distance": row.distance,
                    "value": row.value,
                    "bucket": buckets[row.bucket].name,
                    "pairs_before": row.pairs_before,
                })).collect::<Vec<_>>(),
            }),
        )?;
        vector_dump = json!({
            "requested": true,
            "dir": dir.display().to_string(),
            "layers": qk.len(),
            "rows": meta.len(),
            "inert_logits_exact": inert_exact,
        });
    }

    // A2a: the same forward, plain dot over the same frozen q/k.
    let saved_read = {
        let model = arm
            .stack_mut()
            .ok_or_else(|| invalid("stage0 needs the stack arm"))?;
        let saved = model.config.read;
        model.config.read = ReadScore::Dot;
        saved
    };
    let a2a_dump = arm.read_weight_rows(&ids, batch, context, &rows)?;
    let (a2a, a2a_buckets, a2a_rows) = stage0_tally_dump(&a2a_dump.0, &meta, nb)?;

    // A2b: plain dot with the learned per-distance age bias zeroed.
    let saved_age = {
        let model = arm
            .stack_mut()
            .ok_or_else(|| invalid("stage0 needs the stack arm"))?;
        let mut saved: Vec<(String, Tensor)> = Vec::new();
        for (name, var) in model.variables() {
            if !name.ends_with(".read.age") {
                continue;
            }
            let tensor = var.as_tensor();
            let data = tensor.flatten_all()?.to_vec1::<f32>()?;
            saved.push((
                name.clone(),
                Tensor::from_vec(data, tensor.shape().clone(), tensor.device())?,
            ));
            var.set(&Tensor::zeros_like(tensor)?)?;
        }
        saved
    };
    let a2b_dump = arm.read_weight_rows(&ids, batch, context, &rows)?;
    let (a2b, a2b_buckets, a2b_rows) = stage0_tally_dump(&a2b_dump.0, &meta, nb)?;
    // The plain-dot, age-zeroed weight dump: the library's own A2b, written from
    // the identical code path. It is both the instrument-gate control against
    // the sealed A2b numbers and the reference the offline dot recomputed from
    // the frozen vectors has to reproduce bit for bit.
    if let Some(dir) = dump_dir {
        let (dot_weights, dot_order) = stack_layers(&a2b_dump.0)?;
        let dot_weights = dump_f32(&dir.join("weights_dot_age0.f32"), &dot_weights)?;
        write_json(
            &dir.join("weights_dot_age0.json"),
            &json!({
                "schema": "uor-r4.mqar-bench.stage0-weights/1",
                "note": "the library's own plain-dot, age-zeroed read weights (Stage-0 A2b)",
                "score": "dot",
                "age_zeroed": true,
                "weights": dot_weights,
                "layer_order": dot_order,
            }),
        )?;
    }
    // Release both dot dumps before the restore dump: only A1's is needed for
    // the bitwise swap check, and the peak memory here is under pressure.
    drop(a2a_dump);
    drop(a2b_dump);

    // Restore the arm's own score and age exactly, then prove the swap was
    // reversible: A1 must reproduce bit for bit.
    let restored = {
        let model = arm
            .stack_mut()
            .ok_or_else(|| invalid("stage0 needs the stack arm"))?;
        for (name, tensor) in &saved_age {
            if let Some(var) = model.variables().get(name) {
                var.set(tensor)?;
            }
        }
        model.config.read = saved_read;
        true
    };
    let a1_again = arm.read_weight_rows(&ids, batch, context, &rows)?;
    let mut swap_exact = restored && a1_again.0.len() == a1_dump.0.len();
    if swap_exact {
        for ((la, ta), (lb, tb)) in a1_dump.0.iter().zip(a1_again.0.iter()) {
            if la != lb {
                swap_exact = false;
                break;
            }
            let va = ta.flatten_all()?.to_vec1::<f32>()?;
            let vb = tb.flatten_all()?.to_vec1::<f32>()?;
            if va != vb {
                swap_exact = false;
                break;
            }
        }
    }

    // A3: the exact-identity sieve over the same rows and candidate sets.
    let (a3_sieve, a3, a3_buckets, a3_rows) = stage0_sieve(&ids, set, &meta, context, nb)?;

    // The per-row bootstrap unit: one entry per query row, so a CI over rows
    // can be computed without re-running anything.
    let row_outcomes: Vec<Value> = meta
        .iter()
        .enumerate()
        .map(|(r, row_meta)| {
            json!({
                "row": r,
                "bucket": buckets[row_meta.bucket].name,
                "pairs_before": row_meta.pairs_before,
                "distance": row_meta.distance,
                "a1_top1": a1_rows[r].top1_head_mean,
                "a1_r16": a1_rows[r].r16_head_mean,
                "a1_any_head_top1": a1_rows[r].top1_any_head,
                "a1_noread_mass": a1_rows[r].noread_mass,
                "a2a_top1": a2a_rows[r].top1_head_mean,
                "a2a_r16": a2a_rows[r].r16_head_mean,
                "a2a_any_head_top1": a2a_rows[r].top1_any_head,
                "a2a_noread_mass": a2a_rows[r].noread_mass,
                "a2b_top1": a2b_rows[r].top1_head_mean,
                "a2b_r16": a2b_rows[r].r16_head_mean,
                "a2b_any_head_top1": a2b_rows[r].top1_any_head,
                "a2b_noread_mass": a2b_rows[r].noread_mass,
                "a3_top1": a3_rows[r].top1_head_mean,
                "a3_r16": a3_rows[r].r16_head_mean,
                "a3_any_head_top1": a3_rows[r].top1_any_head,
            })
        })
        .collect();

    let mut by_bucket = serde_json::Map::new();
    for (b, bucket) in buckets.iter().enumerate() {
        by_bucket.insert(
            bucket.name.into(),
            json!({
                "distance_low": bucket.low, "distance_high": bucket.high,
                "a1_fused": a1_buckets[b].json("the arm's own score"),
                "a2a_dot_with_age": a2a_buckets[b].json("plain dot, frozen q/k, learned age kept"),
                "a2b_dot_no_age": a2b_buckets[b].json("plain dot, frozen q/k, age zeroed"),
                "a3_exact_identity_sieve": a3_buckets[b].json("exact token-identity sieve"),
            }),
        );
    }
    let mut overall = serde_json::Map::new();
    overall.insert("a1_fused".into(), a1.json("the arm's own score"));
    overall.insert(
        "a2a_dot_with_age".into(),
        a2a.json("plain dot, frozen q/k, learned age kept"),
    );
    overall.insert(
        "a2b_dot_no_age".into(),
        a2b.json("plain dot, frozen q/k, age zeroed"),
    );
    overall.insert(
        "a3_exact_identity_sieve".into(),
        a3.json("exact token-identity sieve"),
    );
    Ok(json!({
        "status": "complete",
        "scope": "offline readout swap on frozen vectors; not a serving path",
        "panel": {
            "layout": "synthetic MQAR",
            "sequences": batch,
            "rows": meta.len(),
            "context": context,
            "read_layers": layers,
            "heads": arm.read_heads().len() / layers.max(1),
            "buckets": buckets.iter().map(|b| b.name).collect::<Vec<_>>(),
            "sequences_available": sequences.len(),
            "stage0_sequences_cap": STAGE0_SEQUENCES,
        },
        "arms": {
            "A1": "the model's own fused read score",
            "A2a": "plain dot product on the SAME frozen q/k, learned age kept",
            "A2b": "plain dot product on the SAME frozen q/k, age zeroed",
            "A3": "exact token-identity sieve over the same rows and candidates",
        },
        "frozen_vectors": {
            "saved_score": format!("{saved_read:?}"),
            "swapped_score": "Dot",
            "age_variables_saved_and_restored": saved_age.len(),
            "a1_reproduces_after_restore_bitwise": swap_exact,
        },
        "raw_vector_dump": vector_dump,
        "overall": Value::Object(overall),
        "by_bucket": Value::Object(by_bucket),
        "row_outcomes": row_outcomes,
        "a3_sieve_detail": a3_sieve,
    }))
}

fn cosine_rate(lr: f64, warmup: usize, min_lr: f64, steps: usize, step: usize) -> f64 {
    if step < warmup {
        return lr * (step + 1) as f64 / warmup as f64;
    }
    let span = (steps.saturating_sub(warmup)).max(1) as f64;
    let progress = ((step - warmup) as f64 / span).min(1.0);
    let floor = lr * min_lr;
    floor + (lr - floor) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse() -> Result<Self> {
        let mut map = BTreeMap::new();
        for arg in std::env::args().skip(1) {
            let (key, value) = arg
                .split_once('=')
                .ok_or_else(|| invalid(format!("expected key=value, got {arg}")))?;
            if map.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(invalid(format!("{key} given twice")));
            }
        }
        Ok(Self(map))
    }

    fn take(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    fn parsed<T: std::str::FromStr>(&mut self, key: &str, default: T) -> Result<T> {
        match self.take(key) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={text}"))),
        }
    }

    fn finish(self) -> Result<()> {
        if let Some(key) = self.0.keys().next() {
            return Err(invalid(format!("unknown argument {key}")));
        }
        Ok(())
    }
}

/// Task, training and evaluation settings shared by every arm.
struct Common {
    out: PathBuf,
    device_name: String,
    /// The model vocabulary: `VOCAB` for the synthetic layout, the
    /// tokenizer's for `layout=fact`.
    vocab: usize,
    /// Save the trained weights under `model/` of the report root.
    save_model: bool,
    context: usize,
    pairs_per_bucket: usize,
    batch: usize,
    steps: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    curve_sequences: usize,
    final_sequences: usize,
    seed: u64,
    /// The pointer route, from `pointer=<spec>` (`none`, `prime:<w>`, `prime-ranked:<w>`,
    /// `ngram:<w>`, `ngram-ranked:<w>`). DEFAULTS TO `None`, which is what every prior run
    /// used -- the arm was hardcoded and the geometric `ngram` successor route (admission by
    /// the longest ordered n-let match) has therefore NEVER been exercised on this bench.
    /// That is D20 section 2 condition 3 ("wiring verified reached") in its exact form.
    pointer: Option<PrimeRoute>,
    max_seconds: f64,
    /// Steps at which the read probe runs (0 = before training).
    probe_steps: Vec<usize>,
    probe_sequences: usize,
    /// TEMPORARY (T2): dump the read's weight rows on the held-out set.
    dump_scores: bool,
    /// Stage 0 (offline, no training): after the final evaluation, rank the
    /// gold source with the model's own fused score (A1), with a plain dot
    /// product over the *same frozen q/k* (A2), and with the exact
    /// token-identity sieve (A3). Observation only: it changes no logit and
    /// moves no tally, and the model's own score is restored before it returns.
    stage0: bool,
    /// `init=DIR`: load a saved stack (`StackModel::save`'s directory) instead
    /// of training one. This exists so a *frozen* checkpoint can be re-measured
    /// without repeating the training run that produced it: it is the whole
    /// point of the follow-up, since re-training would make the "frozen
    /// vectors" claim a different object. Pair it with `steps=0` to skip the
    /// loop entirely; the load is not silently a no-op, `main` records whether
    /// it happened.
    init: Option<PathBuf>,
    /// `stage0_dump=DIR`: with `stage0=true`, additionally write the raw frozen
    /// score inputs (per read layer: the query and key projections and the
    /// fused auxiliary vector) and the two library weight dumps to `DIR`, so an
    /// offline readout can evaluate a score the model does not itself implement
    /// (cosine, and the norm/direction split) on exactly the vectors the model
    /// scored with. Observation only: it writes files and changes no logit.
    stage0_dump: Option<PathBuf>,
}

impl Common {
    fn record(&self) -> Value {
        let buckets = buckets_for(self.context);
        json!({
            "device": self.device_name,
            "threads": std::env::var("RAYON_NUM_THREADS").ok(),
            "context": self.context,
            "vocab": self.vocab,
            "save_model": self.save_model,
            "token_ranges": {"filler": FILLER, "keys": KEYS, "values": VALUES},
            "pairing_split": {
                "classes": CLASSES, "held_out_class": HELD_OUT_CLASS,
                "rule": "(key - 64 + value - 288) % 4; training draws classes 1..3, the held-out evaluation class 0",
            },
            "buckets": buckets.iter().map(|b| json!({"name": b.name, "low": b.low, "high": b.high})).collect::<Vec<_>>(),
            "pairs_per_bucket": self.pairs_per_bucket,
            "batch": self.batch, "steps": self.steps, "lr": self.lr, "warmup": self.warmup,
            "min_lr": self.min_lr, "weight_decay": self.weight_decay, "clip": self.clip,
            "eval_every": self.eval_every, "curve_sequences": self.curve_sequences,
            "final_sequences": self.final_sequences, "seed": self.seed,
            "max_seconds": self.max_seconds,
            "probe_steps": self.probe_steps, "probe_sequences": self.probe_sequences,
            "dump_scores": self.dump_scores,
            "stage0": self.stage0,
            "init": self.init.as_ref().map(|p| p.display().to_string()),
            "stage0_dump": self.stage0_dump.as_ref().map(|p| p.display().to_string()),
        })
    }
}

/// What `main` runs.
enum Mode {
    Train,
    TaskBaselines,
}

fn settings(mut args: Args) -> Result<(Common, ArmSpec, Mode, Option<fact::FactTask>)> {
    let mode = match args.take("mode").as_deref() {
        None | Some("train") => Mode::Train,
        Some("task-baselines") => Mode::TaskBaselines,
        Some(other) => return Err(invalid(format!("invalid mode={other}"))),
    };
    let fact_layout = match args.take("layout").as_deref() {
        None | Some("synthetic") => false,
        Some("fact") => true,
        Some(other) => return Err(invalid(format!("invalid layout={other}"))),
    };
    let out = PathBuf::from(
        args.take("out")
            .ok_or_else(|| invalid("out=NEW_REPORT_ROOT is required"))?,
    );
    let common = Common {
        out,
        device_name: args.take("device").unwrap_or_else(|| "cpu".into()),
        vocab: VOCAB,
        save_model: args.parsed("save_model", false)?,
        context: args.parsed(
            "context",
            if fact_layout {
                fact::SERVED_CONTEXT
            } else {
                512usize
            },
        )?,
        pairs_per_bucket: args.parsed("pairs_per_bucket", if fact_layout { 4 } else { 8usize })?,
        batch: args.parsed("batch", 8usize)?,
        steps: args.parsed("steps", 1800usize)?,
        lr: args.parsed("lr", 1e-3f64)?,
        warmup: args.parsed("warmup", 100usize)?,
        min_lr: args.parsed("min_lr", 0.1f64)?,
        weight_decay: args.parsed("weight_decay", 0.1f64)?,
        clip: args.parsed("clip", 1.0f64)?,
        eval_every: args.parsed("eval_every", 100usize)?,
        curve_sequences: args.parsed("curve_sequences", 8usize)?,
        final_sequences: args.parsed("final_sequences", if fact_layout { 256 } else { 64usize })?,
        seed: args.parsed("seed", 1u64)?,
        pointer: match args.take("pointer") {
            None => None,
            Some(text) => parse_pointer_route(&text)?,
        },
        max_seconds: args.parsed("max_seconds", 1200f64)?,
        probe_steps: Vec::new(),
        probe_sequences: args.parsed("probe_sequences", 16usize)?,
        // TEMPORARY (T2): `dump_scores=1` writes the read's own weight rows on
        // the held-out set into the report root; the default path is unchanged.
        dump_scores: match args.take("dump_scores").as_deref() {
            None | Some("0") | Some("false") => false,
            Some("1") | Some("true") => true,
            Some(other) => return Err(invalid(format!("invalid dump_scores={other}"))),
        },
        stage0: args.parsed("stage0", false)?,
        init: args.take("init").map(PathBuf::from),
        stage0_dump: args.take("stage0_dump").map(PathBuf::from),
    };
    let mut common = common;
    let probe_text = args
        .take("probe_steps")
        // The fact layout probes once, after training (or never: `none`).
        .unwrap_or_else(|| if fact_layout { "final" } else { "0,600,final" }.into());
    if fact_layout && !matches!(probe_text.as_str(), "final" | "none") {
        return Err(invalid("layout=fact takes probe_steps=final|none"));
    }
    if probe_text != "none" {
        for part in probe_text.split(',') {
            let step = match part {
                "final" => common.steps,
                number => number
                    .parse()
                    .map_err(|_| invalid(format!("invalid probe step {number}")))?,
            };
            if step > common.steps {
                return Err(invalid(format!("probe step {step} beyond steps")));
            }
            common.probe_steps.push(step);
        }
        common.probe_steps.sort_unstable();
        common.probe_steps.dedup();
    }
    let fact_task = if fact_layout {
        let task = fact::FactTask::parse(&mut args, &common)?;
        common.vocab = task.vocab.vocab;
        Some(task)
    } else {
        None
    };
    let arm = ArmSpec::parse(&mut args, common.seed)?;
    args.finish()?;
    // `steps=0` is refused on the ordinary path because a zero-step run would
    // silently report an untrained model. It is allowed exactly when a
    // checkpoint is loaded (`init=`), where the measurement is defined to run
    // on frozen weights and repeating the training loop would defeat the whole
    // point of the frozen-vector claim. This is the only exemption; every other
    // positivity requirement is unchanged.
    let zero_steps_ok = common.steps == 0 && common.init.is_some();
    if common.batch == 0
        || (common.steps == 0 && !zero_steps_ok)
        || common.eval_every == 0
        || common.curve_sequences == 0
        || common.final_sequences < 2
        || !(common.lr > 0.0)
        || !(common.max_seconds > 0.0)
    {
        return Err(invalid(
            "batch, steps, eval_every, sequences, lr and max_seconds must be positive",
        ));
    }
    if buckets_for(common.context).is_empty() {
        return Err(invalid("the context holds no distance bucket"));
    }
    arm.validate(&common)?;
    Ok((common, arm, mode, fact_task))
}

const TRAIN_DOMAIN: u64 = 0x7472_6169_6E;
const CURVE_DOMAIN: u64 = 0x6375_7276_65;
const CURVE_IN_CLASS_DOMAIN: u64 = 0x6375_7276_63;
const HELD_OUT_DOMAIN: u64 = 0x6865_6C64;
const IN_CLASS_DOMAIN: u64 = 0x636C_6173_73;
const PROBE_DOMAIN: u64 = 0x7072_6F62_65;

fn sequences(
    s: &Common,
    buckets: &[Bucket],
    domain: u64,
    count: usize,
    pairing: Pairing,
) -> Result<Vec<Sequence>> {
    (0..count)
        .map(|i| {
            let mut rng = Rng::new(s.seed, domain, i as u64);
            generate(&mut rng, s.context, buckets, s.pairs_per_bucket, pairing)
        })
        .collect()
}

fn bucket_line(result: &Value, buckets: &[Bucket]) -> String {
    buckets
        .iter()
        .map(|b| {
            format!(
                "{}={:.3}",
                b.name,
                result["by_bucket"][b.name]["accuracy"]
                    .as_f64()
                    .unwrap_or(f64::NAN)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs the read probe at its declared steps and keeps the results.
struct Prober<'a> {
    set: &'a [Sequence],
    steps: &'a [usize],
    context: usize,
    buckets: &'a [Bucket],
    probes: Vec<Value>,
    seconds: f64,
}

impl Prober<'_> {
    fn run(&mut self, arm: &dyn ContextArm, step: usize, log: &mut fs::File) -> Result<()> {
        use std::io::Write;
        if !self.steps.contains(&step) || self.probes.iter().any(|p| p["step"] == step) {
            return Ok(());
        }
        let clock = Instant::now();
        let result = read_probe(arm, self.set, self.context, self.buckets)?;
        let line = format!("probe step {step}: {}", probe_line(&result, self.buckets));
        eprintln!("{line}");
        writeln!(log, "{line}")?;
        self.probes.push(json!({"step": step, "probe": result}));
        self.seconds += clock.elapsed().as_secs_f64();
        Ok(())
    }
}

/// Trains `arm` on the task and scores it; returns the arm-independent
/// results block of the report.
fn run(s: &Common, arm: &mut dyn ContextArm, log: &mut fs::File) -> Result<Value> {
    use std::io::Write;
    let started = Instant::now();
    let buckets = buckets_for(s.context);
    let line = format!(
        "{} {}: {} parameters, context {}, buckets {:?}",
        arm.kind(),
        arm.record(),
        arm.parameters(),
        s.context,
        buckets.iter().map(|b| b.name).collect::<Vec<_>>()
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    let curve_held_out = sequences(
        s,
        &buckets,
        CURVE_DOMAIN,
        s.curve_sequences,
        Pairing::HeldOut,
    )?;
    let curve_in_class = sequences(
        s,
        &buckets,
        CURVE_IN_CLASS_DOMAIN,
        s.curve_sequences,
        Pairing::Train,
    )?;
    let chunk = s.batch;
    let mut curve = Vec::new();
    curve.push(json!({
        "step": 0,
        "in_class": evaluate(&*arm, &curve_in_class, s.context, &buckets, chunk)?,
        "held_out": evaluate(&*arm, &curve_held_out, s.context, &buckets, chunk)?,
    }));
    let probe_set = sequences(s, &buckets, PROBE_DOMAIN, s.probe_sequences, Pairing::Train)?;
    let mut prober = Prober {
        set: &probe_set,
        steps: &s.probe_steps,
        context: s.context,
        buckets: &buckets,
        probes: Vec::new(),
        seconds: 0.0,
    };
    prober.run(&*arm, 0, log)?;
    let (mut window_loss, mut window_steps) = (0f64, 0usize);
    let mut step_seconds = Vec::new();
    let mut stopped_early = false;
    let mut step = 0;
    while step < s.steps {
        // The probe is observation, not training: its time is not charged
        // against the training cap (it is reported separately).
        if started.elapsed().as_secs_f64() - prober.seconds > s.max_seconds {
            stopped_early = true;
            break;
        }
        let clock = Instant::now();
        let lr = cosine_rate(s.lr, s.warmup, s.min_lr, s.steps, step);
        let batch: Vec<Sequence> = (0..s.batch)
            .map(|b| {
                let mut rng = Rng::new(s.seed, TRAIN_DOMAIN, (step * s.batch + b) as u64);
                generate(
                    &mut rng,
                    s.context,
                    &buckets,
                    s.pairs_per_bucket,
                    Pairing::Train,
                )
            })
            .collect::<Result<_>>()?;
        let (ids, targets, weights) = batch_arrays(&batch, s.context);
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
            let in_class = evaluate(&*arm, &curve_in_class, s.context, &buckets, chunk)?;
            let held_out = evaluate(&*arm, &curve_held_out, s.context, &buckets, chunk)?;
            let train_loss = window_loss / window_steps.max(1) as f64;
            let line = format!(
                "step {step} lr {lr:.2e} train query NLL {train_loss:.4} grad {grad_norm:.3} in-class acc {:.3} {} | held-out acc {:.3} ({:.0}s)",
                in_class["accuracy"].as_f64().unwrap_or(f64::NAN),
                bucket_line(&in_class, &buckets),
                held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
                started.elapsed().as_secs_f64()
            );
            eprintln!("{line}");
            writeln!(log, "{line}")?;
            curve.push(json!({
                "step": step, "lr": lr, "train_query_nll": train_loss,
                "grad_norm": grad_norm, "in_class": in_class, "held_out": held_out,
                "elapsed_seconds": started.elapsed().as_secs_f64(),
            }));
            window_loss = 0.0;
            window_steps = 0;
        }
        prober.run(&*arm, step, log)?;
    }
    if stopped_early {
        prober.run(&*arm, s.steps, log)?;
    }
    let train_seconds: f64 = step_seconds.iter().sum();
    let eval_clock = Instant::now();
    let in_class_set = sequences(
        s,
        &buckets,
        IN_CLASS_DOMAIN,
        s.final_sequences,
        Pairing::Train,
    )?;
    let in_class = evaluate(&*arm, &in_class_set, s.context, &buckets, chunk)?;
    let held_out_set = sequences(
        s,
        &buckets,
        HELD_OUT_DOMAIN,
        s.final_sequences / 2,
        Pairing::HeldOut,
    )?;
    let held_out = evaluate(&*arm, &held_out_set, s.context, &buckets, chunk)?;
    let line = format!(
        "final in-class fresh acc {:.4} {}; held-out-class acc {:.4} {}",
        in_class["accuracy"].as_f64().unwrap_or(f64::NAN),
        bucket_line(&in_class, &buckets),
        held_out["accuracy"].as_f64().unwrap_or(f64::NAN),
        bucket_line(&held_out, &buckets),
    );
    eprintln!("{line}");
    writeln!(log, "{line}")?;
    // Save the trained weights BEFORE the stage-0 readout swap: the diagnostic
    // is the memory-heaviest phase here, and a failure in it must not lose the
    // trained model (which `main` otherwise saves only after `run` returns).
    let model_pre_save = if s.save_model {
        match arm.save(&s.out.join("model")) {
            Ok(()) => json!({"requested": true, "saved": true, "path": "model", "when": "before_stage0"}),
            Err(error) => json!({"requested": true, "saved": false, "refusal": error.to_string(), "when": "before_stage0"}),
        }
    } else {
        json!({"requested": false})
    };
    // Stage 0 (offline): hold the frozen q/k vectors, vary only the readout.
    // Observation only: it runs after every tally is final and restores the
    // arm's own score before it returns.
    let stage0 = if s.stage0 {
        let value = stage0_readout(
            arm,
            &in_class_set,
            s.context,
            &buckets,
            s.stage0_dump.as_deref(),
        )?;
        write_json(&s.out.join("stage0_readout.json"), &value)?;
        let line = format!(
            "stage0 frozen-vector readout: A1 top1 {:.4} | A2a(dot+age) {:.4} | A2b(dot,no age) {:.4} | A3(sieve) {:.4} | A1 restored bitwise {}",
            value["overall"]["a1_fused"]["gold_key_top1"].as_f64().unwrap_or(f64::NAN),
            value["overall"]["a2a_dot_with_age"]["gold_key_top1"].as_f64().unwrap_or(f64::NAN),
            value["overall"]["a2b_dot_no_age"]["gold_key_top1"].as_f64().unwrap_or(f64::NAN),
            value["overall"]["a3_exact_identity_sieve"]["gold_key_top1"].as_f64().unwrap_or(f64::NAN),
            value["frozen_vectors"]["a1_reproduces_after_restore_bitwise"],
        );
        eprintln!("{line}");
        writeln!(log, "{line}")?;
        value
    } else {
        json!({"status": "not requested"})
    };
    let mut sorted = step_seconds.clone();
    sorted.sort_by(f64::total_cmp);
    let mean_positions = (0..s.context)
        .map(|t| arm.positions_scored(t) as f64)
        .sum::<f64>()
        / s.context as f64;
    Ok(json!({
        "context_access": {
            "positions_scored_per_query_token_mean": in_class["positions_scored_per_query_token_mean"],
            "positions_scored_per_token_mean_over_window": mean_positions,
            "positions_scored_at_last_position": arm.positions_scored(s.context - 1),
            "counts": arm.access_note(),
        },
        "steps_completed": step,
        "stopped_early_at_max_seconds": stopped_early,
        "read_probe": prober.probes,
        "probe_seconds": prober.seconds,
        "stage0": stage0,
        "model_save_before_stage0": model_pre_save,
        "final_in_class_fresh_pairings": in_class,
        "final_held_out_class_pairings": held_out,
        "curve": curve,
        "train_seconds": train_seconds,
        "median_step_seconds": sorted.get(sorted.len() / 2).copied(),
        "final_eval_seconds": eval_clock.elapsed().as_secs_f64(),
        "wall_seconds": started.elapsed().as_secs_f64(),
        "supervised_queries_seen": step * s.batch * buckets.len() * s.pairs_per_bucket,
    }))
}

/// `save_model=true`: the arm's weights under `model/` of the report root,
/// or the refusal of an arm without a saved form (recorded, not fatal).
fn save_model(common: &Common, arm: &dyn ContextArm) -> Value {
    if !common.save_model {
        return json!({"requested": false});
    }
    match arm.save(&common.out.join("model")) {
        Ok(()) => json!({"requested": true, "saved": true, "path": "model"}),
        Err(error) => json!({"requested": true, "saved": false, "refusal": error.to_string()}),
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

/// The report's fixed per-arm record schema.
const REPORT_SCHEMA: &str = "uor-r4/mqar-bench/arm-record/v1";

/// `mode=task-baselines`: the data-only baselines of the final evaluation
/// sets that the same task settings generate, with no arm and no training.
fn task_baselines_report(common: &Common) -> Result<Value> {
    let buckets = buckets_for(common.context);
    let in_class = sequences(
        common,
        &buckets,
        IN_CLASS_DOMAIN,
        common.final_sequences,
        Pairing::Train,
    )?;
    let held_out = sequences(
        common,
        &buckets,
        HELD_OUT_DOMAIN,
        common.final_sequences / 2,
        Pairing::HeldOut,
    )?;
    Ok(json!({
        "schema": REPORT_SCHEMA, "status": "complete", "mode": "task-baselines",
        "task": common.record(), "argv": std::env::args().collect::<Vec<_>>(),
        "final_in_class_fresh_pairings": task_baselines(&in_class, &buckets),
        "final_held_out_class_pairings": task_baselines(&held_out, &buckets),
    }))
}

fn main() -> Result<()> {
    let mut args = Args::parse()?;
    if args.0.get("mode").map(String::as_str) == Some("decide") {
        args.take("mode");
        return decide::main(args);
    }
    let (common, spec, mode, fact_task) = settings(args)?;
    if let Some(task) = fact_task {
        return fact::main(&common, &spec, matches!(mode, Mode::TaskBaselines), &task);
    }
    if matches!(mode, Mode::TaskBaselines) {
        report_output::claim(&common.out)?;
        let report = task_baselines_report(&common)?;
        write_json(&common.out.join("report.json"), &report)?;
        report_output::seal(&common.out)?;
        report_output::verify(&common.out)?;
        return Ok(());
    }
    // Claimed exclusively after argument validation, before any model work.
    report_output::claim(&common.out)?;
    let task = common.record();
    let argv = std::env::args().collect::<Vec<_>>();
    write_json(
        &common.out.join("config.json"),
        &json!({"schema": REPORT_SCHEMA, "task": task, "arm_label": spec.label(), "argv": argv}),
    )?;
    let mut log = fs::File::create(common.out.join("log.txt"))?;
    let result = (|| -> Result<(Value, Value)> {
        let device = uor_r4_training::baseline_protocol::device(&common.device_name)?;
        let mut arm = spec.build(&common, &device)?;
        // `init=DIR` replaces the freshly built parameters with the saved
        // ones, so the measurement runs on the frozen checkpoint instead of a
        // new training run. The saved `config.json` is loaded with them, and
        // the loaded config is what the arm then reports, so a mismatched
        // config cannot be mistaken for a match.
        let init = match &common.init {
            None => json!({"requested": false}),
            Some(dir) => {
                let model = arm
                    .stack_mut()
                    .ok_or_else(|| invalid("init= needs the stack arm"))?;
                *model = StackModel::load(dir, &device)?;
                json!({
                    "requested": true,
                    "loaded": true,
                    "path": dir.display().to_string(),
                    "loaded_config": model.config.clone(),
                })
            }
        };
        let arm_record = json!({
            "label": spec.label(),
            "kind": arm.kind(),
            "parameters": arm.parameters(),
            "config": arm.record(),
            "init": init,
        });
        let mut results = run(&common, arm.as_mut(), &mut log)?;
        results["model_save"] = save_model(&common, arm.as_ref());
        Ok((arm_record, results))
    })();
    drop(log);
    let report = match &result {
        Ok((arm, results)) => json!({
            "schema": REPORT_SCHEMA, "status": "complete", "task": task, "argv": argv,
            "arm": arm, "results": results,
        }),
        Err(error) => json!({
            "schema": REPORT_SCHEMA, "status": "failed", "task": task, "argv": argv,
            "arm": {"label": spec.label()}, "error": error.to_string(),
        }),
    };
    write_json(&common.out.join("report.json"), &report)?;
    report_output::seal(&common.out)?;
    report_output::verify(&common.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(domain: u64, count: usize, pairing: Pairing, context: usize) -> Vec<Sequence> {
        let buckets = buckets_for(context);
        (0..count)
            .map(|i| {
                let mut rng = Rng::new(7, domain, i as u64);
                generate(&mut rng, context, &buckets, 8, pairing).expect("generation")
            })
            .collect()
    }

    #[test]
    fn the_answer_sits_at_the_stated_distance() {
        for context in [1152, 512] {
            let buckets = buckets_for(context);
            for sequence in draw(TRAIN_DOMAIN, 64, Pairing::Train, context) {
                assert_eq!(sequence.tokens.len(), context);
                assert_eq!(sequence.queries.len(), 8 * buckets.len());
                for query in &sequence.queries {
                    let bucket = buckets[query.bucket];
                    assert!((bucket.low..=bucket.high).contains(&query.distance));
                    let key_position = query.position - query.distance;
                    assert_eq!(sequence.tokens[query.position], query.key);
                    assert_eq!(sequence.tokens[key_position], query.key);
                    assert_eq!(sequence.tokens[key_position + 1], query.value);
                    // The key occurs exactly twice (pair and query); the value
                    // exactly once, before the query.
                    let key_count = sequence.tokens.iter().filter(|&&t| t == query.key).count();
                    let value_positions: Vec<usize> = (0..context)
                        .filter(|&p| sequence.tokens[p] == query.value)
                        .collect();
                    assert_eq!(key_count, 2);
                    assert_eq!(value_positions, vec![key_position + 1]);
                    assert!(query.value >= VALUES.0 && query.value < VALUES.1);
                    assert!(query.key >= KEYS.0 && query.key < KEYS.1);
                }
            }
        }
    }

    #[test]
    fn pairings_are_fresh_and_the_held_out_class_never_trains() {
        let train = draw(TRAIN_DOMAIN, 256, Pairing::Train, 1152);
        let held = draw(HELD_OUT_DOMAIN, 64, Pairing::HeldOut, 1152);
        let mut seen: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
        let mut total = 0usize;
        for sequence in &train {
            for query in &sequence.queries {
                assert!(Pairing::Train.admits(query.key, query.value));
                assert!(!Pairing::HeldOut.admits(query.key, query.value));
                *seen
                    .entry(query.key)
                    .or_default()
                    .entry(query.value)
                    .or_default() += 1;
                total += 1;
            }
        }
        // Fresh pairings: repeats of a (key, value) pairing across sequences
        // occur at the rate of independent uniform draws over the admissible
        // pairings (about total^2 / (2 * pairings)), never as a fixed mapping.
        let repeated: usize = seen
            .values()
            .flat_map(|values| values.values())
            .map(|&n| n - 1)
            .sum();
        let most: usize = seen
            .values()
            .flat_map(|values| values.values())
            .copied()
            .max()
            .unwrap_or(0);
        let admissible = (KEYS.1 - KEYS.0) as f64 * (VALUES.1 - VALUES.0) as f64 * 0.75;
        let chance = (total * total) as f64 / (2.0 * admissible);
        assert!(
            (repeated as f64) < 1.5 * chance,
            "{repeated} repeats of {total}, chance {chance}"
        );
        assert!(most <= 8, "a pairing recurs {most} times");
        let keys_with_one_value = seen.values().filter(|values| values.len() == 1).count();
        assert_eq!(keys_with_one_value, 0);
        for sequence in &held {
            for query in &sequence.queries {
                assert!(Pairing::HeldOut.admits(query.key, query.value));
                assert!(
                    seen.get(&query.key)
                        .is_none_or(|values| !values.contains_key(&query.value)),
                    "a held-out pairing appeared in training"
                );
            }
        }
        // Distinct sequences differ (no reused draws across sequences).
        assert_ne!(train[0].tokens, train[1].tokens);
        let again = draw(TRAIN_DOMAIN, 1, Pairing::Train, 1152);
        assert_eq!(again[0].tokens, train[0].tokens);
    }

    #[test]
    fn weights_fall_on_queries_only() {
        let batch = draw(CURVE_DOMAIN, 2, Pairing::HeldOut, 1152);
        let (ids, targets, weights) = batch_arrays(&batch, 1152);
        assert_eq!(ids.len(), 2 * 1152);
        let weighted: Vec<usize> = (0..weights.len()).filter(|&i| weights[i] > 0.0).collect();
        assert_eq!(weighted.len(), 2 * 8 * buckets_for(1152).len());
        for &i in &weighted {
            assert!(targets[i] >= VALUES.0 && targets[i] < VALUES.1);
            assert!(ids[i] >= KEYS.0 && ids[i] < KEYS.1);
        }
    }

    #[test]
    fn a_stack_arm_trains_scores_and_counts_its_context_access() {
        let common = Common {
            out: PathBuf::from("unused"),
            device_name: "cpu".into(),
            vocab: VOCAB,
            save_model: false,
            context: 64,
            pairs_per_bucket: 2,
            batch: 2,
            steps: 1,
            lr: 1e-3,
            warmup: 1,
            min_lr: 0.1,
            weight_decay: 0.1,
            clip: 1.0,
            eval_every: 1,
            curve_sequences: 1,
            final_sequences: 2,
            seed: 3,
            // `pointer` and `dump_scores` complete the literal; the test target
            // did not compile without them (test-only; `pointer` was already
            // missing on origin/main).
            pointer: None,
            max_seconds: 60.0,
            probe_steps: vec![0],
            probe_sequences: 2,
            dump_scores: false,
            stage0: false,
            // `init` and `stage0_dump` complete the literal: the follow-up
            // added them, and the test target must compile.
            init: None,
            stage0_dump: None,
        };
        let spec = ArmSpec::Stack {
            pattern: "rar".into(),
            read: ReadScore::L2,
            rotation: true,
            width: 16,
            heads: 2,
            mlp_hidden: 16,
            age: AgeInit::Flat,
            lineage: LineageArm::F2,
        };
        spec.validate(&common).expect("valid arm");
        let mut arm = spec.build(&common, &Device::Cpu).expect("arm");
        assert_eq!(arm.positions_scored(0), 2);
        assert_eq!(arm.positions_scored(63), 2 * 64);
        let buckets = buckets_for(common.context);
        let batch = sequences(&common, &buckets, TRAIN_DOMAIN, 2, Pairing::Train).expect("batch");
        let (ids, targets, weights) = batch_arrays(&batch, common.context);
        let loss = arm
            .loss(&ids, &targets, &weights, 2, common.context)
            .expect("loss");
        assert!(arm.update(&loss, 1e-3).expect("update").is_finite());
        let report = evaluate(&*arm, &batch, common.context, &buckets, 2).expect("evaluate");
        assert_eq!(report["total"], 2 * 2 * buckets.len());
        assert_eq!(arm.record()["age_init"], "flat");
        assert_eq!(arm.record()["read_key_shift"], true);
        assert_eq!(arm.record()["lineage"], "f2");
        let probe = read_probe(&*arm, &batch, common.context, &buckets).expect("probe");
        for bucket in &buckets {
            let entry = &probe["by_bucket"][bucket.name];
            assert_eq!(entry["heads"].as_array().map(Vec::len), Some(2));
            let key = entry["best_head_mean_weight_on_key"]
                .as_f64()
                .expect("mass");
            assert!((0.0..=1.0).contains(&key));
        }
    }

    #[test]
    fn the_recency_baseline_reads_the_latest_value_before_the_query() {
        for sequence in draw(TRAIN_DOMAIN, 16, Pairing::Train, 512) {
            for query in &sequence.queries {
                let expected = (0..query.position)
                    .rev()
                    .find(|&p| (VALUES.0..VALUES.1).contains(&sequence.tokens[p]))
                    .map(|p| sequence.tokens[p]);
                assert_eq!(most_recent_value(&sequence, query), expected);
            }
        }
    }

    fn small(pattern: &str, rotation: bool) -> StackConfig {
        StackConfig {
            arch: StackArch::Geometric,
            vocab_size: VOCAB,
            width: 16,
            heads: 2,
            mlp_hidden: 16,
            context: 12,
            pattern: pattern.into(),
            read: ReadScore::L2,
            rotation,
            rotation_group: RotationGroup::Quaternion,
            seed: 5,
            memory: None,
            select: None,
            pointer: None,
        }
    }

    fn last_logits(model: &StackModel, ids: &[u32]) -> Vec<Vec<f32>> {
        model
            .forward(ids, 1, ids.len())
            .and_then(|t| Ok(t.to_vec2::<f32>()?))
            .expect("logits")
    }

    #[test]
    fn the_j_turn_is_the_exact_quaternion_product() {
        let x = Tensor::from_vec(
            vec![1f32, 2., 3., 4., -5., 6., 7., -8.],
            (1, 1, 8),
            &Device::Cpu,
        )
        .expect("tensor");
        let turned = quaternion_j_left(&x).expect("turn");
        let values = turned
            .flatten_all()
            .and_then(|t| t.to_vec1::<f32>())
            .expect("values");
        // j (a + bi + cj + dk) = -c + d i + a j - b k.
        assert_eq!(values, vec![-3., 4., 1., -2., -7., -8., -5., -6.]);
        let mut four = x.clone();
        for _ in 0..4 {
            four = quaternion_j_left(&four).expect("turn");
        }
        assert_eq!(
            four.flatten_all()
                .and_then(|t| t.to_vec1::<f32>())
                .expect("values"),
            x.flatten_all()
                .and_then(|t| t.to_vec1::<f32>())
                .expect("values")
        );
    }

    #[test]
    fn the_key_shift_is_causal_reads_the_previous_token_and_survives_save() {
        let mut model = StackModel::new(small("aa", true), &Device::Cpu).expect("model");
        let ids: Vec<u32> = (0..12).map(|i| 64 + 7 * i).collect();
        let plain = last_logits(&model, &ids);
        model.set_read_key_shift(true).expect("shift");
        let shifted = last_logits(&model, &ids);
        assert_ne!(plain[11], shifted[11], "the shift changes the read");
        // Causal: changing the last token leaves every earlier row unchanged.
        let mut changed = ids.clone();
        changed[11] = 300;
        let after = last_logits(&model, &changed);
        for t in 0..11 {
            assert_eq!(after[t], shifted[t]);
        }
        // Position 0 has no predecessor: its key is unchanged by the shift.
        assert_eq!(plain[0], shifted[0]);
        let dir = std::env::temp_dir().join(format!("mqar-key-shift-{}", std::process::id()));
        model.save(&dir).expect("save");
        let loaded = StackModel::load(&dir, &Device::Cpu).expect("load");
        assert!(loaded.read_key_shift());
        assert_eq!(last_logits(&loaded, &changed), after);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn rotation_has_no_effect_on_an_all_read_stack() {
        // F3 finding: `rotation` parameterizes only the recurrence's transport.
        let on = small("aaa", true);
        let off = small("aaa", false);
        assert_eq!(on.shapes(), off.shapes());
        let a = StackModel::new(on, &Device::Cpu).expect("model");
        let b = StackModel::new(off, &Device::Cpu).expect("model");
        let ids: Vec<u32> = (0..12).map(|i| 64 + 5 * i).collect();
        assert_eq!(last_logits(&a, &ids), last_logits(&b, &ids));
        // ...while a recurrence layer does take rotation rows.
        assert_ne!(small("ra", true).shapes(), small("ra", false).shapes());
    }
}
