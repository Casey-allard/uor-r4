//! Prime-route addressing for the relation store: an evaluation instrument
//! for the memory port's address (ROADMAP §2b), after the prime router of
//! ADR-0003 (`docs/adr/0003-fixed-zeta-prime-route-attention.md`).
//!
//! G v1 ([`crate::stack_aerm`]) keys a fact by the ordered, typed pair
//! (entity token, relation token). A tag head that swaps the two roles on an
//! unseen phrasing reads and writes a different key. The prime router keys
//! data by prime atoms: every token has a registered prime, and a transition
//! expert is the square-free semiprime of two atoms. Here the store is keyed
//! by the semiprime expert of the clause's two key atoms (the last two
//! distinct tokens tagged entity or relation), so the key is the same
//! whichever role the head gave each atom: by unique factorization, `p_a p_b`
//! identifies the unordered pair `{a, b}` exactly. The value is routed
//! through the expert, not multiplied into it, so a relation whose values are
//! names keeps its direction: "Alex's friend is Sam" does not answer "Sam's
//! friend".
//!
//! Two policies for which tokens form a clause's key atoms. They govern
//! **both reads and writes**, since both use the clause's key atoms:
//! - [`ReadPolicy::Tagged`]: only tokens the head tagged entity or relation.
//!   Whenever a key tag completes an expert, both registers (current and
//!   previous distinct value) read, and reads never close the clause. This
//!   is G v1's policy when a clause has at most one entity tag and one
//!   relation tag; with two same-role tags the expert keys the last two
//!   distinct atoms where G v1 keys the latest of each role.
//! - [`ReadPolicy::Sieve`]: also counts, inside a user turn, a token the head
//!   tagged other as a key atom when it is already a registered atom of the
//!   session: a key atom of an expert the store has written (from the model's
//!   own write-time tags, never gold). The rescued atom joins the clause's
//!   key atoms, so it changes **writes as well as reads**. A rescued entity
//!   lets a later assertion about it be written, not only read. The one
//!   read-only part is the preference: a read prefers a written expert both
//!   of whose atoms are among the clause's key atoms, latest atom first. That
//!   is a pairwise membership scan, equivalent to a divisibility filter on the
//!   clause's product of primes, though no product or gcd is formed. The
//!   read-side and write-side contributions are not separated here.
//!
//! Role-agnostic keys would alias `(X, Y)` with `(Y, X)` if a token could play
//! both roles; D2's names and relation words are disjoint, so this is not
//! exercised.
//!
//! Two write policies:
//! - [`WritePolicy::Trigger`]: the model's learned write trigger, as G v1;
//! - [`WritePolicy::TurnEnd`]: a write at the last input position before the
//!   assistant's reply (after the assistant marker), if the clause then has an
//!   expert and a value atom. A question whose tokens the head tags no value
//!   never writes; a value-tagged question would be written, the mood case the
//!   ROADMAP's memory port still lists as open. A session knows where a user
//!   turn ends, so this uses no gold label; the learned trigger is ignored.
//!
//! The store's version semantics are the probe's ([`crate::stack_aerm::RelationStore`]):
//! a different value becomes current and keeps the previous distinct value; a
//! same-value write increments the version and keeps it.
//!
//! Evaluation only: nothing here trains or serves, and the store has no
//! capacity bound (D2's episodes never evict).

use std::collections::{BTreeMap, BTreeSet};

use candle_core::{Tensor, D};
use serde::{Deserialize, Serialize};

use crate::stack_aerm::{
    model_registers, AermModel, DialogueBatch, QueryClass, Registers, RelationWorld, STATUS_ABSENT,
    STATUS_HIT, STATUS_NONE, TAG_ENTITY, TAG_OTHER, TAG_RELATION, TAG_VALUE, TRIGGERS,
    TRIGGER_WRITE,
};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// Token id `t` is the atom of the `(t + 1)`-th prime.
#[derive(Clone, Debug)]
pub struct PrimeRegistry {
    primes: Vec<u64>,
}

impl PrimeRegistry {
    /// The first `atoms` primes, by a sieve bounded with Rosser's theorem:
    /// the `n`-th prime is below `n (ln n + ln ln n)` for `n >= 6`.
    pub fn new(atoms: usize) -> Result<Self> {
        if atoms == 0 {
            return Err(invalid("the registry needs at least one atom"));
        }
        let n = atoms as f64;
        let bound = if atoms < 6 {
            13
        } else {
            (n * (n.ln() + n.ln().ln())).ceil() as usize
        };
        let mut composite = vec![false; bound + 1];
        let mut primes = Vec::with_capacity(atoms);
        for i in 2..=bound {
            if composite[i] {
                continue;
            }
            primes.push(i as u64);
            if primes.len() == atoms {
                break;
            }
            let mut multiple = i * i;
            while multiple <= bound {
                composite[multiple] = true;
                multiple += i;
            }
        }
        if primes.len() < atoms {
            return Err(invalid("the sieve bound produced too few primes"));
        }
        Ok(Self { primes })
    }

    /// The prime registered for `token`.
    pub fn prime(&self, token: u32) -> Result<u64> {
        self.primes
            .get(token as usize)
            .copied()
            .ok_or_else(|| invalid(format!("token {token} is outside the prime registry")))
    }

    pub fn len(&self) -> usize {
        self.primes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primes.is_empty()
    }

    /// The semiprime expert `p_a p_b` of two atoms.
    pub fn semiprime(&self, a: u32, b: u32) -> Result<u64> {
        self.prime(a)?
            .checked_mul(self.prime(b)?)
            .ok_or_else(|| invalid("a semiprime expert overflowed u64"))
    }

    /// The semiprime expert of the last two distinct atoms of `keys`, or
    /// `None` when there are fewer than two.
    pub fn expert(&self, keys: &[u32]) -> Result<Option<u64>> {
        last_two_distinct(keys)
            .map(|(a, b)| self.semiprime(a, b))
            .transpose()
    }
}

/// The last two distinct atoms of `keys`, latest first.
fn last_two_distinct(keys: &[u32]) -> Option<(u32, u32)> {
    let mut atoms = keys.iter().rev();
    let &last = atoms.next()?;
    let &other = atoms.find(|&&atom| atom != last)?;
    Some((last, other))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpertRecord {
    value: u32,
    previous: Option<u32>,
    version: u32,
}

/// An exact store keyed by semiprime experts, with the probe's version
/// semantics and no capacity bound.
#[derive(Clone, Debug, Default)]
pub struct ExpertStore {
    records: BTreeMap<u64, ExpertRecord>,
}

impl ExpertStore {
    /// A different value becomes current and keeps the previous distinct
    /// value; a same-value write increments the version and keeps it.
    pub fn write(&mut self, expert: u64, value: u32) {
        self.records
            .entry(expert)
            .and_modify(|record| {
                if record.value != value {
                    record.previous = Some(record.value);
                    record.value = value;
                }
                record.version += 1;
            })
            .or_insert(ExpertRecord {
                value,
                previous: None,
                version: 1,
            });
    }

    /// `(status, value)`: the current value, or the previous distinct value
    /// when `previous`; Absent when there is none.
    pub fn read(&self, expert: u64, previous: bool) -> (u32, u32) {
        match self.records.get(&expert) {
            Some(record) => match (previous, record.previous) {
                (false, _) => (STATUS_HIT, record.value),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None => (STATUS_ABSENT, 0),
        }
    }

    /// Whether `expert` has been written.
    pub fn contains(&self, expert: u64) -> bool {
        self.records.contains_key(&expert)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Which tokens form a clause's key atoms, for its reads **and** its writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadPolicy {
    /// Only tokens the head tagged entity or relation (G v1).
    Tagged,
    /// Also, inside a user turn, other-tagged tokens that are registered key
    /// atoms of a written expert; reads prefer a written expert.
    Sieve,
}

/// When the prime-route store writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritePolicy {
    /// The model's learned write trigger, which also closes the clause (G v1).
    Trigger,
    /// The last input position before each assistant reply (after the
    /// assistant marker), which also closes the clause.
    TurnEnd,
}

/// Per-position turn marks of one sequence: `turn_end[t]` is the last
/// position before an assistant reply starts, and `reply[t]` lies inside an
/// assistant reply.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TurnMarks {
    pub turn_end: Vec<bool>,
    pub reply: Vec<bool>,
}

impl TurnMarks {
    /// Marks for one padded row from its episode's response mask. Padding is
    /// neither a turn end nor a reply.
    pub fn from_response_mask(response_mask: &[u8], time: usize) -> Self {
        let inputs = response_mask.len().saturating_sub(1);
        Self {
            turn_end: (0..time)
                .map(|t| {
                    t + 1 < response_mask.len()
                        && response_mask[t] == 0
                        && response_mask[t + 1] == 1
                })
                .collect(),
            reply: (0..time)
                .map(|t| t < inputs && response_mask[t] == 1)
                .collect(),
        }
    }
}

/// The written expert a sieve read selects: the first pair of distinct key
/// atoms, latest atom first, whose semiprime the store has written.
fn sieve_pair(
    keys: &[u32],
    registry: &PrimeRegistry,
    store: &ExpertStore,
) -> Result<Option<(u32, u32)>> {
    for (i, &a) in keys.iter().enumerate().rev() {
        for &b in keys[..i].iter().rev() {
            if a != b && store.contains(registry.semiprime(a, b)?) {
                return Ok(Some((a, b)));
            }
        }
    }
    Ok(None)
}

/// One read by the prime-route store: the position, the two atoms of the
/// expert it read (latest first) and whether that expert had been written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteRead {
    pub position: usize,
    pub pair: (u32, u32),
    pub stored: bool,
}

/// One sequence's registers, its reads, and which positions were key atoms.
#[derive(Clone, Debug, Default)]
pub struct PrimeRouteRun {
    pub registers: Registers,
    pub reads: Vec<RouteRead>,
    pub key_atom: Vec<bool>,
}

/// Registers over one sequence from its per-token tags and triggers, keyed
/// by semiprime experts. `eos` also closes the open clause.
#[allow(clippy::too_many_arguments)]
pub fn simulate_prime_route(
    tokens: &[u32],
    tags: &[u32],
    triggers: &[u32],
    marks: &TurnMarks,
    eos: u32,
    registry: &PrimeRegistry,
    write: WritePolicy,
    read: ReadPolicy,
) -> Result<Registers> {
    Ok(trace_prime_route(tokens, tags, triggers, marks, eos, registry, write, read)?.registers)
}

/// [`simulate_prime_route`] with its reads and key-atom positions.
#[allow(clippy::too_many_arguments)]
pub fn trace_prime_route(
    tokens: &[u32],
    tags: &[u32],
    triggers: &[u32],
    marks: &TurnMarks,
    eos: u32,
    registry: &PrimeRegistry,
    write: WritePolicy,
    read: ReadPolicy,
) -> Result<PrimeRouteRun> {
    let n = tokens.len();
    if tags.len() != n || triggers.len() != n || marks.turn_end.len() != n || marks.reply.len() != n
    {
        return Err(invalid("one tag, trigger and turn mark per token"));
    }
    let mut store = ExpertStore::default();
    let mut atoms: BTreeSet<u32> = BTreeSet::new();
    let mut keys: Vec<u32> = Vec::new();
    let mut value: Option<u32> = None;
    let mut register = (STATUS_NONE, 0u32);
    let mut previous = (STATUS_NONE, 0u32);
    let mut out = Registers {
        status: Vec::with_capacity(n),
        value: Vec::with_capacity(n),
        status_previous: Vec::with_capacity(n),
        value_previous: Vec::with_capacity(n),
        read_events: 0,
    };
    let mut reads = Vec::new();
    let mut key_atoms = Vec::with_capacity(n);
    for position in 0..n {
        let (token, tag, trigger) = (tokens[position], tags[position], triggers[position]);
        if trigger as usize >= TRIGGERS {
            return Err(invalid("trigger outside the trigger set"));
        }
        let key_atom = match tag {
            TAG_OTHER => {
                read == ReadPolicy::Sieve && !marks.reply[position] && atoms.contains(&token)
            }
            TAG_ENTITY | TAG_RELATION => true,
            TAG_VALUE => {
                value = Some(token);
                false
            }
            _ => return Err(invalid("tag outside the tag set")),
        };
        key_atoms.push(key_atom);
        if key_atom {
            keys.push(token);
            let pair = match read {
                ReadPolicy::Tagged => last_two_distinct(&keys),
                ReadPolicy::Sieve => match sieve_pair(&keys, registry, &store)? {
                    Some(pair) => Some(pair),
                    None => last_two_distinct(&keys),
                },
            };
            if let Some(pair) = pair {
                let expert = registry.semiprime(pair.0, pair.1)?;
                register = store.read(expert, false);
                previous = store.read(expert, true);
                out.read_events += 2;
                reads.push(RouteRead {
                    position,
                    pair,
                    stored: store.contains(expert),
                });
            }
        }
        let mut close = token == eos;
        let write_here = match write {
            WritePolicy::Trigger => trigger == TRIGGER_WRITE,
            WritePolicy::TurnEnd => marks.turn_end[position],
        };
        if write_here {
            if let (Some((a, b)), Some(value)) = (last_two_distinct(&keys), value) {
                store.write(registry.semiprime(a, b)?, value);
                atoms.insert(a);
                atoms.insert(b);
            }
            close = true;
        }
        if close {
            keys.clear();
            value = None;
        }
        out.status.push(register.0);
        out.value.push(register.1);
        out.status_previous.push(previous.0);
        out.value_previous.push(previous.1);
    }
    Ok(PrimeRouteRun {
        registers: out,
        reads,
        key_atom: key_atoms,
    })
}

/// Prime-route registers for every row of a padded batch, from `tags` and
/// `triggers` (row-major, one per input position).
#[allow(clippy::too_many_arguments)]
pub fn prime_route_registers(
    data: &DialogueBatch,
    tags: &[u32],
    triggers: &[u32],
    eos: u32,
    registry: &PrimeRegistry,
    write: WritePolicy,
    read: ReadPolicy,
) -> Result<Registers> {
    let runs = prime_route_runs(data, tags, triggers, eos, registry, write, read)?;
    Ok(concat_registers(&runs))
}

/// The rows' registers, concatenated row-major.
fn concat_registers(runs: &[PrimeRouteRun]) -> Registers {
    let mut out = Registers::default();
    for run in runs {
        let row = &run.registers;
        out.read_events += row.read_events;
        out.status.extend_from_slice(&row.status);
        out.value.extend_from_slice(&row.value);
        out.status_previous.extend_from_slice(&row.status_previous);
        out.value_previous.extend_from_slice(&row.value_previous);
    }
    out
}

/// [`trace_prime_route`] for every row of a padded batch.
#[allow(clippy::too_many_arguments)]
pub fn prime_route_runs(
    data: &DialogueBatch,
    tags: &[u32],
    triggers: &[u32],
    eos: u32,
    registry: &PrimeRegistry,
    write: WritePolicy,
    read: ReadPolicy,
) -> Result<Vec<PrimeRouteRun>> {
    let (batch, time) = (data.batch, data.time);
    if tags.len() != batch * time || triggers.len() != batch * time {
        return Err(invalid("one tag and trigger per batch position"));
    }
    data.episodes
        .iter()
        .enumerate()
        .map(|(b, episode)| {
            let range = b * time..(b + 1) * time;
            trace_prime_route(
                &data.ids[range.clone()],
                &tags[range.clone()],
                &triggers[range],
                &TurnMarks::from_response_mask(&episode.response_mask, time),
                eos,
                registry,
                write,
                read,
            )
        })
        .collect()
}

/// The model's tags at the gold entity and relation slots of the last gold
/// write of `key` before `end`, and whether the model fired the write there.
#[allow(clippy::too_many_arguments)]
fn write_side(
    ids: &[u32],
    gold_tags: &[u32],
    gold_triggers: &[u32],
    model_tags: &[u32],
    model_triggers: &[u32],
    key: (u32, u32),
    end: usize,
    eos: u32,
) -> String {
    let (mut entity, mut relation, mut found) = (None, None, None);
    for p in 0..end.min(ids.len()) {
        match gold_tags[p] {
            TAG_ENTITY => entity = Some(p),
            TAG_RELATION => relation = Some(p),
            _ => {}
        }
        if gold_triggers[p] == TRIGGER_WRITE {
            if let (Some(e), Some(r)) = (entity, relation) {
                if (ids[e], ids[r]) == key {
                    found = Some((e, r, p));
                }
            }
            (entity, relation) = (None, None);
        } else if ids[p] == eos {
            (entity, relation) = (None, None);
        }
    }
    let name = |tag: u32| match tag {
        TAG_OTHER => "other",
        TAG_ENTITY => "entity",
        TAG_RELATION => "relation",
        TAG_VALUE => "value",
        _ => "invalid",
    };
    match found {
        None => "write=none".into(),
        Some((e, r, w)) => format!(
            "write_entity={},write_relation={},write_fired={}",
            name(model_tags[e]),
            name(model_tags[r]),
            model_triggers[w] == TRIGGER_WRITE
        ),
    }
}

/// Why a query's sieve registers differ from gold, or that they agree.
fn sieve_cause(
    run: &PrimeRouteRun,
    ids: &[u32],
    query: &crate::stack_aerm::Query,
    check: usize,
    equal: bool,
    eos: u32,
) -> String {
    if equal {
        return "RegistersMatch".into();
    }
    let turn_start = ids[..query.answer_start]
        .iter()
        .rposition(|&t| t == eos)
        .map_or(1, |p| p + 1);
    let (entity, relation) = query.key;
    match run
        .reads
        .iter()
        .rev()
        .find(|r| r.position >= turn_start && r.position <= check)
    {
        None => {
            let atom =
                |token: u32| (turn_start..=check).any(|p| ids[p] == token && run.key_atom[p]);
            format!(
                "NoRead/entity_atom={},relation_atom={}",
                atom(entity),
                atom(relation)
            )
        }
        Some(read) => {
            let (a, b) = read.pair;
            if (a, b) == (entity, relation) || (b, a) == (entity, relation) {
                "RightPairWrongRecord".into()
            } else {
                format!("WrongPair/stored={}", read.stored)
            }
        }
    }
}

/// The register sources compared on the same episodes and the same model.
/// Serialized under the same names reports use (`GV1`, `ExpertSieve`, ...).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegisterArm {
    /// G v1: typed `(entity, relation)` keys, the learned write trigger.
    GV1,
    /// Semiprime experts, the learned write trigger.
    ExpertTrigger,
    /// Semiprime experts, a write at each user turn's end.
    ExpertTurnEnd,
    /// Semiprime experts, the learned write trigger, and sieve clause
    /// formation (reads and writes) over the session's registered atoms.
    ExpertSieve,
    /// Gold registers: a perfect parser (the probe's oracle).
    Gold,
}

impl RegisterArm {
    pub const ALL: [RegisterArm; 5] = [
        RegisterArm::GV1,
        RegisterArm::ExpertTrigger,
        RegisterArm::ExpertTurnEnd,
        RegisterArm::ExpertSieve,
        RegisterArm::Gold,
    ];
}

/// One arm's score on one query class.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmClassScore {
    pub queries: usize,
    /// Value classes: the value slot; abstaining classes: the whole answer
    /// (the probe's definition).
    pub correct: usize,
    /// The arm's registers at the answer's check row equal the gold registers.
    pub registers_equal_gold: usize,
}

/// One arm's scores on every query class.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmEvaluation {
    pub classes: BTreeMap<String, ArmClassScore>,
    pub read_events: usize,
}

/// All arms on the same episodes: the probe's dialogue batches for `seed`,
/// generated in `batch`-sized chunks exactly as
/// [`crate::stack_aerm::evaluate_dialogues`] generates them, so the G v1 arm
/// reproduces that function's per-class `correct` counts.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PrimeRouteEvaluation {
    pub episodes: usize,
    pub held: bool,
    pub seed: u64,
    /// The model's tag accuracy on real positions.
    pub tag_accuracy: f64,
    pub arms: BTreeMap<String, ArmEvaluation>,
    /// `ExpertSieve` failures by class and cause (`RegistersMatch` means the
    /// registers equal gold, so the failure is emission).
    pub sieve_failure_causes: BTreeMap<String, BTreeMap<String, usize>>,
    /// Queries `GV1` answered and `ExpertSieve` did not, by class and cause.
    pub sieve_regression_causes: BTreeMap<String, BTreeMap<String, usize>>,
}

fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

/// Scores every [`RegisterArm`] with `model` on `episodes` fresh dialogues.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_prime_route(
    model: &AermModel,
    world: &RelationWorld,
    registry: &PrimeRegistry,
    episodes: usize,
    seed: u64,
    held: bool,
    context: usize,
    batch: usize,
) -> Result<PrimeRouteEvaluation> {
    if !model.has_memory() {
        return Err(invalid("the prime-route arms need the memory arm"));
    }
    if batch == 0 {
        return Err(invalid("the batch must be positive"));
    }
    let vocabulary = world.vocabulary();
    let held_names = vocabulary["held_out_names"]
        .as_array()
        .ok_or_else(|| invalid("the world's vocabulary lists no held-out names"))?
        .iter()
        .map(|pair| {
            pair.get(1)
                .and_then(|id| id.as_u64())
                .and_then(|id| u32::try_from(id).ok())
                .ok_or_else(|| invalid("a held-out name has no token id"))
        })
        .collect::<Result<BTreeSet<u32>>>()?;
    if held_names.is_empty() {
        return Err(invalid("the world's vocabulary lists no held-out names"));
    }
    let mut rng = Rng::new(seed);
    let mut result = PrimeRouteEvaluation {
        episodes,
        held,
        seed,
        ..Default::default()
    };
    let (mut tags_right, mut tags_total) = (0usize, 0usize);
    let mut remaining = episodes;
    while remaining > 0 {
        let size = remaining.min(batch);
        remaining -= size;
        let data = world.batch(&mut rng, size, context, held)?;
        let bottom = model.bottom(&data.ids, size, context)?;
        let tags = argmax_rows(&bottom.tags)?;
        let triggers = argmax_rows(&bottom.triggers)?;
        for row in 0..size * context {
            if data.real[row] > 0.0 {
                tags_total += 1;
                tags_right += usize::from(tags[row] == data.tags[row]);
            }
        }
        let mut gv1_correct: Vec<bool> = Vec::new();
        let mut sieve_runs: Vec<PrimeRouteRun> = Vec::new();
        for arm in RegisterArm::ALL {
            let registers = match arm {
                RegisterArm::GV1 => {
                    model_registers(&data.ids, &tags, &triggers, size, context, world.eos)?
                }
                RegisterArm::ExpertTrigger => prime_route_registers(
                    &data,
                    &tags,
                    &triggers,
                    world.eos,
                    registry,
                    WritePolicy::Trigger,
                    ReadPolicy::Tagged,
                )?,
                RegisterArm::ExpertTurnEnd => prime_route_registers(
                    &data,
                    &tags,
                    &triggers,
                    world.eos,
                    registry,
                    WritePolicy::TurnEnd,
                    ReadPolicy::Tagged,
                )?,
                RegisterArm::ExpertSieve => {
                    sieve_runs = prime_route_runs(
                        &data,
                        &tags,
                        &triggers,
                        world.eos,
                        registry,
                        WritePolicy::Trigger,
                        ReadPolicy::Sieve,
                    )?;
                    concat_registers(&sieve_runs)
                }
                RegisterArm::Gold => Registers {
                    status: data.status.clone(),
                    value: data.value.clone(),
                    status_previous: data.status_previous.clone(),
                    value_previous: data.value_previous.clone(),
                    read_events: 0,
                },
            };
            let predicted = argmax_rows(&model.top(
                &bottom.hidden,
                &registers.status,
                &registers.value,
                &registers.status_previous,
                &registers.value_previous,
            )?)?;
            let evaluation = result.arms.entry(format!("{arm:?}")).or_default();
            evaluation.read_events += registers.read_events;
            let mut query_index = 0usize;
            for (b, episode) in data.episodes.iter().enumerate() {
                let base = b * context;
                for query in &episode.queries {
                    let rows = (query.answer_start - 1)..query.answer_end;
                    let slot_row = query.value_position.map(|p| p - 1);
                    let correct = match (query.class.abstains(), slot_row) {
                        (false, Some(r)) => predicted[base + r] == data.targets[base + r],
                        _ => rows
                            .clone()
                            .all(|r| predicted[base + r] == data.targets[base + r]),
                    };
                    let check = base + slot_row.unwrap_or(query.answer_start - 1);
                    let equal = registers.status[check] == data.status[check]
                        && registers.value[check] == data.value[check]
                        && registers.status_previous[check] == data.status_previous[check]
                        && registers.value_previous[check] == data.value_previous[check];
                    let score = evaluation
                        .classes
                        .entry(format!("{:?}", query.class))
                        .or_default();
                    score.queries += 1;
                    score.correct += usize::from(correct);
                    score.registers_equal_gold += usize::from(equal);
                    match arm {
                        RegisterArm::GV1 => gv1_correct.push(correct),
                        RegisterArm::ExpertSieve if !correct => {
                            let row = base..base + context;
                            let mut cause = sieve_cause(
                                &sieve_runs[b],
                                &data.ids[row.clone()],
                                query,
                                check - base,
                                equal,
                                world.eos,
                            );
                            if cause.starts_with("NoRead") {
                                cause = format!(
                                    "{cause}/held_name={}/{}",
                                    held_names.contains(&query.key.0),
                                    write_side(
                                        &data.ids[row.clone()],
                                        &data.tags[row.clone()],
                                        &data.triggers[row.clone()],
                                        &tags[row.clone()],
                                        &triggers[row],
                                        query.key,
                                        query.answer_start,
                                        world.eos,
                                    )
                                );
                            }
                            let class = format!("{:?}", query.class);
                            *result
                                .sieve_failure_causes
                                .entry(class.clone())
                                .or_default()
                                .entry(cause.clone())
                                .or_default() += 1;
                            if gv1_correct.get(query_index).copied().unwrap_or(false) {
                                *result
                                    .sieve_regression_causes
                                    .entry(class)
                                    .or_default()
                                    .entry(cause)
                                    .or_default() += 1;
                            }
                        }
                        _ => {}
                    }
                    query_index += 1;
                }
            }
        }
    }
    result.tag_accuracy = tags_right as f64 / tags_total.max(1) as f64;
    Ok(result)
}

/// Accuracy of `class` under `arm`, if any query of that class was scored.
pub fn arm_accuracy(
    evaluation: &PrimeRouteEvaluation,
    arm: RegisterArm,
    class: QueryClass,
) -> Option<f64> {
    let score = evaluation
        .arms
        .get(&format!("{arm:?}"))?
        .classes
        .get(&format!("{class:?}"))?;
    (score.queries > 0).then(|| score.correct as f64 / score.queries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_aerm::{simulate, TRIGGER_NONE};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// The probe's toy encoder: pretokens (a word with its leading space, an
    /// apostrophe-suffix or one punctuation mark) each get their own id.
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
                .map(|p| {
                    let next = vocab.len() as u32 + 3;
                    *vocab.entry(p).or_insert(next)
                })
                .collect()
        }
    }

    fn world() -> Result<RelationWorld> {
        RelationWorld::new(&toy_encoder(), 0, 1)
    }

    #[test]
    fn the_registry_holds_the_first_primes_and_experts_are_unordered() -> Result<()> {
        let registry = PrimeRegistry::new(4096)?;
        assert_eq!(registry.len(), 4096);
        assert_eq!(
            (0..10)
                .map(|t| registry.prime(t))
                .collect::<Result<Vec<_>>>()?,
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        );
        // The 4,096th prime.
        assert_eq!(registry.prime(4095)?, 38_873);
        assert!(registry.prime(4096).is_err());
        let ab = registry.expert(&[7, 11])?;
        let ba = registry.expert(&[11, 7])?;
        assert_eq!(ab, Some(19 * 37));
        assert_eq!(ab, ba);
        // The last two distinct atoms; a repeat is not a second atom.
        assert_eq!(registry.expert(&[3, 7, 7])?, Some(7 * 19));
        assert_eq!(registry.expert(&[7, 7])?, None);
        assert_eq!(registry.expert(&[])?, None);
        Ok(())
    }

    /// Unique factorization: two semiprime experts are equal exactly when
    /// their unordered atom pairs are. The store only compares keys for
    /// equality, so an expert key carries the same information as a sorted
    /// pair `(min, max)` and yields identical registers (the sorted-pair
    /// control requested on #973, settled for every pair of a 64-atom
    /// registry).
    #[test]
    fn semiprime_experts_are_exactly_unordered_pairs() -> Result<()> {
        let atoms = 64u32;
        let registry = PrimeRegistry::new(atoms as usize)?;
        let mut seen: BTreeMap<u64, (u32, u32)> = BTreeMap::new();
        for a in 0..atoms {
            for b in 0..atoms {
                if a == b {
                    continue;
                }
                let sorted = (a.min(b), a.max(b));
                let key = registry.semiprime(a, b)?;
                assert_eq!(*seen.entry(key).or_insert(sorted), sorted);
            }
        }
        assert_eq!(seen.len() as u32, atoms * (atoms - 1) / 2);
        Ok(())
    }

    #[test]
    fn the_expert_store_keeps_the_probe_version_semantics() {
        let mut store = ExpertStore::default();
        assert_eq!(store.read(6, false), (STATUS_ABSENT, 0));
        store.write(6, 40);
        assert_eq!(store.read(6, false), (STATUS_HIT, 40));
        assert_eq!(store.read(6, true), (STATUS_ABSENT, 0));
        store.write(6, 41);
        assert_eq!(store.read(6, false), (STATUS_HIT, 41));
        assert_eq!(store.read(6, true), (STATUS_HIT, 40));
        // A same-value write keeps the previous distinct value.
        store.write(6, 41);
        assert_eq!(store.read(6, true), (STATUS_HIT, 40));
        assert_eq!(store.len(), 1);
    }

    /// With gold tags, the expert arms reproduce the gold registers at every
    /// input position for three of the four (write, read) pairs: the trigger
    /// policy with tagged and with sieve reads, and the turn-end policy with
    /// every learned trigger removed and tagged reads.
    #[test]
    fn gold_tags_reproduce_the_gold_registers() -> Result<()> {
        let world = world()?;
        let registry = PrimeRegistry::new(4096)?;
        for held in [false, true] {
            let mut rng = Rng::new(if held { 91 } else { 90 });
            let data = world.batch(&mut rng, 24, 256, held)?;
            let silent = vec![TRIGGER_NONE; data.triggers.len()];
            for (write, read, triggers) in [
                (WritePolicy::Trigger, ReadPolicy::Tagged, &data.triggers),
                (WritePolicy::TurnEnd, ReadPolicy::Tagged, &silent),
                (WritePolicy::Trigger, ReadPolicy::Sieve, &data.triggers),
            ] {
                let registers = prime_route_registers(
                    &data, &data.tags, triggers, world.eos, &registry, write, read,
                )?;
                for row in 0..data.batch * data.time {
                    if data.real[row] == 0.0 {
                        continue;
                    }
                    assert_eq!(
                        (
                            registers.status[row],
                            registers.value[row],
                            registers.status_previous[row],
                            registers.value_previous[row]
                        ),
                        (
                            data.status[row],
                            data.value[row],
                            data.status_previous[row],
                            data.value_previous[row]
                        ),
                        "{write:?}/{read:?} held={held}: row {row}"
                    );
                }
            }
        }
        Ok(())
    }

    /// Swapping the entity and relation tags of every query turn (a turn with
    /// no write) makes a query's roles disagree with its fact's. That breaks
    /// G v1's typed keys but leaves the semiprime experts, and so the
    /// registers, unchanged. (A swap applied to every turn alike would keep
    /// the typed keys consistent.)
    #[test]
    fn swapped_query_roles_break_typed_keys_but_not_experts() -> Result<()> {
        let world = world()?;
        let registry = PrimeRegistry::new(4096)?;
        let mut rng = Rng::new(92);
        let data = world.batch(&mut rng, 24, 256, true)?;
        let mut swapped = data.tags.clone();
        for b in 0..data.batch {
            let row = b * data.time..(b + 1) * data.time;
            let mut turn_start = row.start;
            for position in row.clone() {
                if data.ids[position] != world.eos {
                    continue;
                }
                let turn = turn_start..=position;
                if !turn.clone().any(|p| data.triggers[p] == TRIGGER_WRITE) {
                    for p in turn {
                        swapped[p] = match data.tags[p] {
                            TAG_ENTITY => TAG_RELATION,
                            TAG_RELATION => TAG_ENTITY,
                            other => other,
                        };
                    }
                }
                turn_start = position + 1;
            }
        }
        let experts = prime_route_registers(
            &data,
            &swapped,
            &data.triggers,
            world.eos,
            &registry,
            WritePolicy::Trigger,
            ReadPolicy::Tagged,
        )?;
        let typed = model_registers(
            &data.ids,
            &swapped,
            &data.triggers,
            data.batch,
            data.time,
            world.eos,
        )?;
        let (mut hits, mut typed_differs) = (0usize, 0usize);
        for row in 0..data.batch * data.time {
            if data.real[row] == 0.0 {
                continue;
            }
            assert_eq!(experts.status[row], data.status[row], "row {row}");
            assert_eq!(experts.value[row], data.value[row], "row {row}");
            hits += usize::from(data.status[row] == STATUS_HIT);
            typed_differs += usize::from(
                typed.status[row] != data.status[row] || typed.value[row] != data.value[row],
            );
        }
        assert!(hits > 0, "the batch must contain hits");
        assert!(typed_differs > 0, "swapped roles must break the typed keys");
        Ok(())
    }

    /// The value is routed through the expert, not multiplied into it: a
    /// relation whose value is a name keeps its direction.
    #[test]
    fn a_name_valued_relation_keeps_its_direction() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (alex, friend, sam, eos) = (10u32, 11u32, 12u32, 2u32);
        // "Alex's friend is Sam." then "Who is Sam's friend?"
        let tokens = [alex, friend, sam, eos, sam, friend, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
        ];
        let marks = TurnMarks {
            turn_end: vec![false; 7],
            reply: vec![false; 7],
        };
        for read in [ReadPolicy::Tagged, ReadPolicy::Sieve] {
            let registers = simulate_prime_route(
                &tokens,
                &tags,
                &triggers,
                &marks,
                eos,
                &registry,
                WritePolicy::Trigger,
                read,
            )?;
            assert_eq!(
                (registers.status[5], registers.value[5]),
                (STATUS_ABSENT, 0),
                "{read:?}"
            );
        }
        // The same query about Alex finds Sam.
        let tokens = [alex, friend, sam, eos, friend, alex, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_RELATION,
            TAG_ENTITY,
            TAG_OTHER,
        ];
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &marks,
            eos,
            &registry,
            WritePolicy::Trigger,
            ReadPolicy::Tagged,
        )?;
        assert_eq!((registers.status[5], registers.value[5]), (STATUS_HIT, sam));
        // G v1 agrees on both when the tags are right.
        let typed = simulate(&tokens, &tags, &triggers, eos)?;
        assert_eq!((typed.status[5], typed.value[5]), (STATUS_HIT, sam));
        Ok(())
    }

    #[test]
    fn a_question_never_writes_at_its_turn_end() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (alex, color, blue, eos) = (10u32, 11u32, 12u32, 2u32);
        // A question with no value atom, then the same question again.
        let tokens = [alex, color, eos, alex, color, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [TRIGGER_NONE; 6];
        let marks = TurnMarks {
            turn_end: vec![false, true, false, false, true, false],
            reply: vec![false; 6],
        };
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &marks,
            eos,
            &registry,
            WritePolicy::TurnEnd,
            ReadPolicy::Tagged,
        )?;
        assert_eq!(registers.status[4], STATUS_ABSENT);
        // An assertion writes at its turn end and the next question reads it.
        let tokens = [alex, color, blue, eos, alex, color, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let triggers = [TRIGGER_NONE; 7];
        let marks = TurnMarks {
            turn_end: vec![false, false, true, false, false, true, false],
            reply: vec![false; 7],
        };
        let registers = simulate_prime_route(
            &tokens,
            &tags,
            &triggers,
            &marks,
            eos,
            &registry,
            WritePolicy::TurnEnd,
            ReadPolicy::Tagged,
        )?;
        assert_eq!(
            (registers.status[5], registers.value[5]),
            (STATUS_HIT, blue)
        );
        Ok(())
    }

    /// The failure G v1's decomposition measured: in "Which color does Momo
    /// have now?" the head tags the query's entity other. Tagged reads never
    /// complete an expert; the sieve counts Momo, a registered atom of the
    /// written expert, and finds the record. Inside a reply the sieve never
    /// rescues a token.
    #[test]
    fn the_sieve_finds_a_record_whose_query_entity_was_not_tagged() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (momo, color, blue, which, eos) = (10u32, 11u32, 12u32, 13u32, 2u32);
        // "Momo's color is blue." then "Which color does Momo have now?"
        let tokens = [momo, color, blue, eos, which, color, momo, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_RELATION,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let mut triggers = [TRIGGER_NONE; 8];
        triggers[2] = TRIGGER_WRITE;
        let user = TurnMarks {
            turn_end: vec![false; 8],
            reply: vec![false; 8],
        };
        let run = |read, marks: &TurnMarks| {
            simulate_prime_route(
                &tokens,
                &tags,
                &triggers,
                marks,
                eos,
                &registry,
                WritePolicy::Trigger,
                read,
            )
        };
        let tagged = run(ReadPolicy::Tagged, &user)?;
        assert_ne!((tagged.status[6], tagged.value[6]), (STATUS_HIT, blue));
        let sieve = run(ReadPolicy::Sieve, &user)?;
        assert_eq!((sieve.status[6], sieve.value[6]), (STATUS_HIT, blue));
        // The same untagged mention inside a reply is not rescued.
        let replying = TurnMarks {
            turn_end: vec![false; 8],
            reply: vec![false, false, false, false, false, false, true, false],
        };
        let sieve = run(ReadPolicy::Sieve, &replying)?;
        assert_ne!((sieve.status[6], sieve.value[6]), (STATUS_HIT, blue));
        Ok(())
    }

    /// The sieve changes writes too: an entity tagged other in a later
    /// assertion, but registered by an earlier write, joins that assertion's
    /// key atoms, so the new fact is written and a later query finds it. Under
    /// tagged key atoms the assertion has one key atom and nothing is written.
    #[test]
    fn the_sieve_rescues_a_write_whose_entity_was_not_tagged() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (molly, color, red, pet, cat, eos) = (10u32, 11u32, 12u32, 13u32, 14u32, 2u32);
        // "Molly's color is red." / "Now Molly's pet is a cat." (Molly tagged
        // other) / "Molly's pet?"
        let tokens = [
            molly, color, red, eos, molly, pet, cat, eos, molly, pet, eos,
        ];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
        ];
        let mut triggers = [TRIGGER_NONE; 11];
        triggers[2] = TRIGGER_WRITE;
        triggers[6] = TRIGGER_WRITE;
        let marks = TurnMarks {
            turn_end: vec![false; 11],
            reply: vec![false; 11],
        };
        let run = |read| {
            simulate_prime_route(
                &tokens,
                &tags,
                &triggers,
                &marks,
                eos,
                &registry,
                WritePolicy::Trigger,
                read,
            )
        };
        let tagged = run(ReadPolicy::Tagged)?;
        assert_eq!((tagged.status[9], tagged.value[9]), (STATUS_ABSENT, 0));
        let sieve = run(ReadPolicy::Sieve)?;
        assert_eq!((sieve.status[9], sieve.value[9]), (STATUS_HIT, cat));
        Ok(())
    }

    /// The trace names why a query's registers differ from gold.
    #[test]
    fn the_sieve_cause_names_a_missing_atom_and_a_right_pair() -> Result<()> {
        let registry = PrimeRegistry::new(64)?;
        let (momo, color, blue, which, eos) = (10u32, 11u32, 12u32, 13u32, 2u32);
        let ids = [momo, color, blue, eos, which, color, momo, eos];
        let tags = [
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_RELATION,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let mut triggers = [TRIGGER_NONE; 8];
        triggers[2] = TRIGGER_WRITE;
        let marks = TurnMarks {
            turn_end: vec![false; 8],
            reply: vec![false; 8],
        };
        let query = crate::stack_aerm::Query {
            class: QueryClass::First,
            recency_trap: false,
            key: (momo, color),
            answer_start: 7,
            answer_end: 7,
            value_position: None,
            value: Some(blue),
        };
        let trace = |read| {
            trace_prime_route(
                &ids,
                &tags,
                &triggers,
                &marks,
                eos,
                &registry,
                WritePolicy::Trigger,
                read,
            )
        };
        // Tagged reads: Momo is not an atom in the query turn, so nothing reads.
        let tagged = trace(ReadPolicy::Tagged)?;
        assert_eq!(
            sieve_cause(&tagged, &ids, &query, 6, false, eos),
            "NoRead/entity_atom=false,relation_atom=true"
        );
        // Sieve reads: the right pair is read.
        let sieve = trace(ReadPolicy::Sieve)?;
        assert_eq!(
            sieve_cause(&sieve, &ids, &query, 6, false, eos),
            "RightPairWrongRecord"
        );
        assert_eq!(
            sieve_cause(&sieve, &ids, &query, 6, true, eos),
            "RegistersMatch"
        );
        Ok(())
    }
}
