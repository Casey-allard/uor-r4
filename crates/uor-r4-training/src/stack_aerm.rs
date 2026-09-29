//! Architectural exact relational memory (AERM) inside the recurrence-primary
//! geometric stack (whole-project synthesis §3 M1, §4 D2; G v1 read policy).
//!
//! The stack is split after `split` layers. Two learned heads read the
//! RMS-normalised residual stream there: a per-token role tag (entity,
//! relation, value or other) and a per-token trigger (none, write, read the
//! current value, read the previous value). An exact integer store sits
//! between the two halves:
//!
//! - a write or the end of a turn closes the open clause: the latest tagged
//!   entity, relation and value tokens since the previous close;
//! - a write stores `(entity, relation) -> value`. A hit on an existing key
//!   overwrites it, keeps the previous distinct value and increments the
//!   version: version order, never a score, decides the current value;
//! - a read looks the key up and sets a register `(status, value)` that holds
//!   until the next read of that register. The statuses are None, Hit, Absent
//!   and Evicted.
//!
//! **G v1 read policy (always-on, address-driven).** Reads are no longer a
//! learned per-token fire decision. Whenever the tag stream yields an entity or
//! relation label and both are defined, the store is read twice at that
//! position — the current value and the previous distinct value — and **two**
//! registers update. Reads never close the clause; writes (`TRIGGER_WRITE`,
//! still the learned trigger) and `eos` close it. The trigger head and its gold
//! loss are unchanged; only the simulator's read policy changed from D2's
//! `TRIGGER_READ`/`TRIGGER_READ_PREVIOUS` gating. The failed D2 decision was
//! surface-dependent (a fire at the end of the user turn); address completion
//! occurs at the entity/relation token, which every phrasing contains.
//!
//! Both registers re-enter the residual stream through zero-initialised status
//! embeddings and value projections, and each adds a zero-initialised copy
//! boost to its value token's logit, so the branch starts silent. Training
//! drives the store from the gold tags and write triggers (teacher forcing) and
//! trains the heads with auxiliary cross-entropies: no gradient passes through
//! the discrete store. Evaluation drives the store from the model's own tags
//! and triggers only.
//!
//! The control arm has the same heads and auxiliary losses, a widened MLP of
//! about equal parameter count, and no store.
//!
//! The store keys are exact `(entity, relation)` token pairs placed by two
//! multiplier-free xor-shift mixes in a power-of-two table; each record keeps
//! its exact key, so a collision can evict but never alias.
//!
//! Offline training and evaluation only; nothing here is a serving path.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use candle_core::{backprop::GradStore, DType, Device, Tensor, Var, D};
use candle_nn::{AdamW, Optimizer, ParamsAdamW};
use serde::{Deserialize, Serialize};

use crate::geometric_stack::{logits_cross_entropy, StackConfig, StackModel};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

pub const TAGS: usize = 4;
pub const TAG_OTHER: u32 = 0;
pub const TAG_ENTITY: u32 = 1;
pub const TAG_RELATION: u32 = 2;
pub const TAG_VALUE: u32 = 3;

pub const TRIGGERS: usize = 4;
pub const TRIGGER_NONE: u32 = 0;
pub const TRIGGER_WRITE: u32 = 1;
pub const TRIGGER_READ: u32 = 2;
pub const TRIGGER_READ_PREVIOUS: u32 = 3;

pub const STATUSES: usize = 4;
pub const STATUS_NONE: u32 = 0;
pub const STATUS_HIT: u32 = 1;
pub const STATUS_ABSENT: u32 = 2;
pub const STATUS_EVICTED: u32 = 3;

/// Store slots: a power of two, addressed by masking.
pub const STORE_SLOTS: usize = 64;
/// Overflow entries searched when both of a key's slots are taken.
pub const STORE_STASH: usize = 8;

// ---------------------------------------------------------------------------
// Canonical atoms of the natural relation world (D2-natural v2).
//
// The standard world keys a fact by the exact surface `(entity, relation)`
// tokens, so two synonyms of the same word ("name"/"named"/"called") or the
// pronoun and possessive forms of one person ("I"/"My"/"my") fragment into
// different keys. The natural world gives every entity and relation a small,
// stable canonical atom instead; the gold store and the per-clause labels are
// keyed by those atoms, independent of the surface tokens a phrasing chooses.
//
// Ids are fixed and documented; they are never renumbered. A `0` atom means
// "no canonical label at this position".

/// Canonical entity atoms. `SPEAKER` is the first-person writer (I, my, me).
pub mod entity_atom {
    pub const SPEAKER: u32 = 1;
    pub const CAT: u32 = 2;
    pub const SISTER: u32 = 3;
    pub const BROTHERS: u32 = 4;
    pub const CAR: u32 = 5;
    pub const DOG: u32 = 6;
    pub const BROTHER: u32 = 7;
    pub const BIKE: u32 = 8;
}

/// Canonical relation atoms. `NAME..INSTRUMENT` drive the milestone panel;
/// `PET..CITY` are non-panel relations.
pub mod relation_atom {
    pub const NAME: u32 = 1;
    pub const JOB: u32 = 2;
    pub const HOME: u32 = 3;
    pub const COUNT: u32 = 4;
    pub const BIRTHDAY: u32 = 5;
    pub const CAR_COLOR: u32 = 6;
    pub const FAVORITE_COLOR: u32 = 7;
    pub const FAVORITE_FOOD: u32 = 8;
    pub const INSTRUMENT: u32 = 9;
    pub const PET: u32 = 10;
    pub const TOY: u32 = 11;
    pub const FRIEND: u32 = 12;
    pub const AGE: u32 = 13;
    pub const CITY: u32 = 14;
}

/// One natural-world clause: a write at a statement end, or a read at a
/// question end. `trigger_position` is the document position that closes the
/// clause (`TRIGGER_WRITE` for a statement, `TRIGGER_READ`/`TRIGGER_READ_PREVIOUS`
/// for a question); `value` is the written value for a write and the gold
/// read value for a read (`None` when the read is absent); `previous` marks a
/// read of the previous distinct value; `class` is the fact's history class at
/// the clause. `entity_atom`/`relation_atom` are the canonical key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clause {
    pub trigger_position: usize,
    pub entity_atom: u32,
    pub relation_atom: u32,
    pub value: Option<u32>,
    pub previous: bool,
    pub class: QueryClass,
}

/// The change history of one canonical key in the v2 gold store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomRecord {
    /// The current value token.
    pub value: u32,
    /// The value before the last change; a same-value write keeps it.
    pub previous: Option<u32>,
    pub version: u32,
}

/// The D2-natural v2 gold store, keyed by canonical `(entity_atom,
/// relation_atom)`. It mirrors [`RelationStore`]'s version semantics exactly
/// (current value, previous distinct value, version) but is an exact map, so a
/// stored key is never evicted. It exists beside the token-keyed store, which
/// is unchanged, so a key probe can compare a model's read register against
/// canonical gold without the surface fragmentation of the token store.
#[derive(Clone, Debug, Default)]
pub struct AtomStore {
    records: BTreeMap<(u32, u32), AtomRecord>,
}

impl AtomStore {
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    pub fn record(&self, entity: u32, relation: u32) -> Option<AtomRecord> {
        self.records.get(&(entity, relation)).copied()
    }

    /// Writes `value` under a canonical key: overwrite on a hit (keeping the
    /// previous distinct value and incrementing the version), else insert.
    pub fn write(&mut self, entity: u32, relation: u32, value: u32) {
        match self.records.get_mut(&(entity, relation)) {
            Some(record) => {
                if record.value != value {
                    record.previous = Some(record.value);
                    record.value = value;
                }
                record.version += 1;
            }
            None => {
                self.records.insert(
                    (entity, relation),
                    AtomRecord {
                        value,
                        previous: None,
                        version: 1,
                    },
                );
            }
        }
    }

    /// `(status, value)`: the current value, or the previous distinct value
    /// when `previous`; Absent when there is none.
    pub fn read(&self, entity: u32, relation: u32, previous: bool) -> (u32, u32) {
        match self.record(entity, relation) {
            Some(record) => match (previous, record.previous) {
                (false, _) => (STATUS_HIT, record.value),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None => (STATUS_ABSENT, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// The exact store.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub entity: u32,
    pub relation: u32,
    pub value: u32,
    /// The value before the last change; a same-value write keeps it.
    pub previous: Option<u32>,
    pub version: u32,
    written: u64,
}

/// A fixed-capacity exact relation store: two-choice placement with a small
/// overflow stash, so a record is evicted only when its two slots and the
/// stash are all taken.
#[derive(Clone, Debug)]
pub struct RelationStore {
    /// `STORE_SLOTS` addressed slots followed by `STORE_STASH` stash entries.
    slots: Vec<Option<Record>>,
    evicted: Vec<(u32, u32)>,
    clock: u64,
}

impl Default for RelationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationStore {
    pub fn new() -> Self {
        Self {
            slots: vec![None; STORE_SLOTS + STORE_STASH],
            evicted: Vec::new(),
            clock: 0,
        }
    }

    /// Two candidate slots of an exact key: both ids are spread over the
    /// whole word, mixed by three xor-shift rounds (no multiply), and the two
    /// choices come from different halves of the result.
    pub fn candidates(entity: u32, relation: u32) -> [usize; 2] {
        let (e, r) = (u64::from(entity), u64::from(relation));
        let mut x = e ^ (r << 32) ^ (r << 11) ^ (e << 43) ^ 0x9E37_79B9_7F4A_7C15;
        for _ in 0..3 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
        }
        let mask = (STORE_SLOTS - 1) as u64;
        [(x & mask) as usize, ((x >> 32) & mask) as usize]
    }

    fn find(&self, entity: u32, relation: u32) -> Option<usize> {
        let [a, b] = Self::candidates(entity, relation);
        [a, b]
            .into_iter()
            .chain(STORE_SLOTS..STORE_SLOTS + STORE_STASH)
            .find(|&slot| {
                matches!(self.slots[slot], Some(r) if r.entity == entity && r.relation == relation)
            })
    }

    /// The record of an exact key, if stored.
    pub fn record(&self, entity: u32, relation: u32) -> Option<Record> {
        self.find(entity, relation)
            .and_then(|slot| self.slots[slot])
    }

    /// Writes `value`: overwrite on a hit (keeping the previous distinct value
    /// and incrementing the version), otherwise insert into a free candidate
    /// slot or stash entry, evicting the older of the two candidate records
    /// only when all are taken.
    pub fn write(&mut self, entity: u32, relation: u32, value: u32) {
        self.clock += 1;
        if let Some(slot) = self.find(entity, relation) {
            if let Some(record) = self.slots[slot].as_mut() {
                if record.value != value {
                    record.previous = Some(record.value);
                    record.value = value;
                }
                record.version += 1;
                record.written = self.clock;
            }
            return;
        }
        let [a, b] = Self::candidates(entity, relation);
        let free_stash =
            (STORE_SLOTS..STORE_SLOTS + STORE_STASH).find(|&i| self.slots[i].is_none());
        let slot = match (self.slots[a], self.slots[b], free_stash) {
            (None, _, _) => a,
            (_, None, _) => b,
            (_, _, Some(stash)) => stash,
            (Some(x), Some(y), None) => {
                if x.written <= y.written {
                    a
                } else {
                    b
                }
            }
        };
        if let Some(old) = self.slots[slot] {
            self.evicted.push((old.entity, old.relation));
        }
        self.evicted.retain(|&key| key != (entity, relation));
        self.slots[slot] = Some(Record {
            entity,
            relation,
            value,
            previous: None,
            version: 1,
            written: self.clock,
        });
    }

    /// `(status, value)`: the current value, or the previous distinct value
    /// when `previous`; Absent when there is none; Evicted when the key was
    /// stored and has been overwritten by another key.
    pub fn read(&self, entity: u32, relation: u32, previous: bool) -> (u32, u32) {
        match self.record(entity, relation) {
            Some(record) => match (previous, record.previous) {
                (false, _) => (STATUS_HIT, record.value),
                (true, Some(value)) => (STATUS_HIT, value),
                (true, None) => (STATUS_ABSENT, 0),
            },
            None if self.evicted.contains(&(entity, relation)) => (STATUS_EVICTED, 0),
            None => (STATUS_ABSENT, 0),
        }
    }
}

/// One read performed by the store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadEvent {
    pub position: usize,
    pub key: Option<(u32, u32)>,
    pub previous: bool,
    pub status: u32,
    pub value: u32,
}

/// One completed write: the clause's exact key and value at a write trigger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteEvent {
    pub position: usize,
    pub key: (u32, u32),
    pub value: u32,
}

/// Registers per position and the store after the last position.
#[derive(Clone, Debug)]
pub struct Simulation {
    pub status: Vec<u32>,
    pub value: Vec<u32>,
    pub status_previous: Vec<u32>,
    pub value_previous: Vec<u32>,
    pub reads: Vec<ReadEvent>,
    pub writes: Vec<WriteEvent>,
    pub store: RelationStore,
}

/// The model's own registers for one padded batch, row-major, plus the number
/// of read events its tag stream produced.
#[derive(Clone, Debug, Default)]
pub struct Registers {
    pub status: Vec<u32>,
    pub value: Vec<u32>,
    pub status_previous: Vec<u32>,
    pub value_previous: Vec<u32>,
    pub read_events: usize,
}

/// Runs the store over one sequence from its per-token tags and triggers.
/// `eos` also closes the open clause (a turn boundary).
///
/// G v1 read policy: an entity or relation label with a complete address reads
/// both registers at that position; the read trigger classes no longer decide
/// reads and never close the clause.
pub fn simulate(tokens: &[u32], tags: &[u32], triggers: &[u32], eos: u32) -> Result<Simulation> {
    simulate_inner(tokens, tags, triggers, eos, None)
}

/// [`simulate`] with the structural user-turn gate: at a position whose
/// `user_turn` flag is 0 the tags and triggers are ignored, so no read or
/// write happens outside a user turn. The world's operations all sit inside
/// user turns, so its own episodes are unchanged; prose carries no user turns
/// and its registers stay silent.
pub fn simulate_gated(
    tokens: &[u32],
    tags: &[u32],
    triggers: &[u32],
    eos: u32,
    user_turn: &[u8],
) -> Result<Simulation> {
    if user_turn.len() != tokens.len() {
        return Err(invalid("one user-turn flag per token"));
    }
    simulate_inner(tokens, tags, triggers, eos, Some(user_turn))
}

fn simulate_inner(
    tokens: &[u32],
    tags: &[u32],
    triggers: &[u32],
    eos: u32,
    user_turn: Option<&[u8]>,
) -> Result<Simulation> {
    if tags.len() != tokens.len() || triggers.len() != tokens.len() {
        return Err(invalid("one tag and one trigger per token"));
    }
    let mut store = RelationStore::new();
    let (mut entity, mut relation, mut value) = (None, None, None);
    let mut register = (STATUS_NONE, 0u32);
    let mut previous = (STATUS_NONE, 0u32);
    let mut status = Vec::with_capacity(tokens.len());
    let mut values = Vec::with_capacity(tokens.len());
    let mut status_previous = Vec::with_capacity(tokens.len());
    let mut values_previous = Vec::with_capacity(tokens.len());
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for (position, ((&token, &tag), &trigger)) in tokens.iter().zip(tags).zip(triggers).enumerate()
    {
        let active = user_turn.is_none_or(|flags| flags[position] != 0);
        if active {
            match tag {
                TAG_OTHER => {}
                TAG_ENTITY => entity = Some(token),
                TAG_RELATION => relation = Some(token),
                TAG_VALUE => value = Some(token),
                _ => return Err(invalid("tag outside the tag set")),
            }
            if matches!(tag, TAG_ENTITY | TAG_RELATION) {
                if let (Some(e), Some(r)) = (entity, relation) {
                    register = store.read(e, r, false);
                    previous = store.read(e, r, true);
                    reads.push(ReadEvent {
                        position,
                        key: Some((e, r)),
                        previous: false,
                        status: register.0,
                        value: register.1,
                    });
                    reads.push(ReadEvent {
                        position,
                        key: Some((e, r)),
                        previous: true,
                        status: previous.0,
                        value: previous.1,
                    });
                }
            }
        }
        let mut close = token == eos;
        if active {
            match trigger {
                TRIGGER_NONE | TRIGGER_READ | TRIGGER_READ_PREVIOUS => {}
                TRIGGER_WRITE => {
                    if let (Some(e), Some(r), Some(v)) = (entity, relation, value) {
                        store.write(e, r, v);
                        writes.push(WriteEvent {
                            position,
                            key: (e, r),
                            value: v,
                        });
                    }
                    close = true;
                }
                _ => return Err(invalid("trigger outside the trigger set")),
            }
        }
        if close {
            (entity, relation, value) = (None, None, None);
        }
        status.push(register.0);
        values.push(register.1);
        status_previous.push(previous.0);
        values_previous.push(previous.1);
    }
    Ok(Simulation {
        status,
        value: values,
        status_previous,
        value_previous: values_previous,
        reads,
        writes,
        store,
    })
}

// ---------------------------------------------------------------------------
// Synthetic relation dialogues in the literal-role protocol.

#[derive(Clone, Copy, Debug)]
enum Part {
    Text(&'static str),
    Entity,
    Relation,
    Value,
}
use Part::{Entity, Relation, Text, Value};

type Template = &'static [Part];

const ASSERT_TRAIN: &[Template] = &[
    &[
        Text("Note that"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is"),
        Value,
        Text("."),
    ],
    &[
        Text("Remember that the"),
        Relation,
        Text(" of"),
        Entity,
        Text(" is"),
        Value,
        Text("."),
    ],
    &[
        Text("I want to tell you that"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is"),
        Value,
        Text("."),
    ],
];
const ASSERT_HELD: &[Template] = &[
    &[
        Text("Here is a fact:"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is"),
        Value,
        Text("."),
    ],
    &[
        Text("Please know that the"),
        Relation,
        Text(" of"),
        Entity,
        Text(" is"),
        Value,
        Text("."),
    ],
];
const UPDATE_TRAIN: &[Template] = &[
    &[
        Text("Now"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is"),
        Value,
        Text("."),
    ],
    &[
        Text("Update: the"),
        Relation,
        Text(" of"),
        Entity,
        Text(" is now"),
        Value,
        Text("."),
    ],
    &[
        Text("Things changed, and"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is now"),
        Value,
        Text("."),
    ],
];
const UPDATE_HELD: &[Template] = &[&[
    Text("These days"),
    Entity,
    Text("'s"),
    Relation,
    Text(" is"),
    Value,
    Text("."),
]];
const REASSERT_TRAIN: &[Template] = &[
    &[
        Text("As before,"),
        Entity,
        Text("'s"),
        Relation,
        Text(" is still"),
        Value,
        Text("."),
    ],
    &[
        Text("Yes, the"),
        Relation,
        Text(" of"),
        Entity,
        Text(" is still"),
        Value,
        Text("."),
    ],
];
const REASSERT_HELD: &[Template] = &[&[
    Text("Nothing changed:"),
    Entity,
    Text("'s"),
    Relation,
    Text(" is still"),
    Value,
    Text("."),
]];
const QUERY_TRAIN: &[Template] = &[
    &[Text("What is"), Entity, Text("'s"), Relation, Text("?")],
    &[
        Text("Can you tell me"),
        Entity,
        Text("'s"),
        Relation,
        Text("?"),
    ],
    &[
        Text("Do you know the"),
        Relation,
        Text(" of"),
        Entity,
        Text("?"),
    ],
];
const QUERY_HELD: &[Template] = &[&[
    Text("Which"),
    Relation,
    Text(" does"),
    Entity,
    Text(" have now?"),
]];
const PREVIOUS_TRAIN: &[Template] = &[
    &[
        Text("What was"),
        Entity,
        Text("'s"),
        Relation,
        Text(" before?"),
    ],
    &[
        Text("Before the change, what was the"),
        Relation,
        Text(" of"),
        Entity,
        Text("?"),
    ],
];
const PREVIOUS_HELD: &[Template] = &[&[
    Text("What"),
    Relation,
    Text(" did"),
    Entity,
    Text(" have before?"),
]];
/// Chit-chat, including hard negatives that mention relation or value words.
const CHAT_TRAIN: &[&str] = &[
    "I like sunny days.",
    "The weather is nice today.",
    "I saw a cat in the park.",
    "The sky has a pretty color today.",
    "My friend and I played outside.",
    "I read a book about a dog.",
    "We went to the beach last week.",
    "I think red is a warm color.",
];
const CHAT_HELD: &[&str] = &[
    "The garden looks lovely in the morning.",
    "A bird sang near my window.",
];
const CHAT_REPLIES: &[&str] = &["That sounds nice.", "Okay."];
const ACKS: &[&str] = &["Okay.", "Got it."];
const ABSTAIN: &str = "I do not know.";

const NAMES_TRAIN: &[&str] = &[
    " Bob", " Sam", " Mia", " Tom", " Lily", " Ben", " Max", " Anna", " Jack", " Lucy", " Tim",
    " Sue", " Sara", " Amy", " Jane", " John",
];
const NAMES_HELD: &[&str] = &[" Molly", " Leo", " Emma", " Rose"];
/// Relation word, article before its values, and values. `None` values mean
/// the relation takes names.
const RELATIONS: &[(&str, Option<&str>, Option<&[&str]>)] = &[
    (
        " pet",
        Some(" a"),
        Some(&[
            " cat", " dog", " bird", " fish", " frog", " bunny", " duck", " mouse", " fox",
            " puppy", " kitten",
        ]),
    ),
    (
        " color",
        None,
        Some(&[
            " red", " blue", " green", " yellow", " pink", " purple", " orange", " brown",
            " black", " white",
        ]),
    ),
    (
        " toy",
        Some(" a"),
        Some(&[
            " ball", " doll", " kite", " car", " train", " drum", " boat", " robot", " truck",
            " puzzle", " teddy",
        ]),
    ),
    (
        " job",
        Some(" a"),
        Some(&[
            " doctor", " teacher", " farmer", " cook", " nurse", " king", " queen", " driver",
        ]),
    ),
    (
        " place",
        Some(" the"),
        Some(&[
            " park", " zoo", " beach", " farm", " forest", " garden", " store", " lake", " river",
            " castle", " village",
        ]),
    ),
    (" friend", None, None),
];

#[derive(Clone, Debug)]
struct Word {
    text: String,
    id: u32,
}

#[derive(Clone, Debug)]
struct RelationSpec {
    word: Word,
    article: Option<&'static str>,
    values: Vec<Word>,
    /// The values were the name pool at construction; `extend_names` grows them.
    from_names: bool,
}

/// The class of a query, from the gold store when it is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum QueryClass {
    /// Asserted once, never changed.
    First,
    /// Changed at least once; its last event changed it. The gated class.
    Updated,
    /// Its last event restated the current value.
    Reasserted,
    /// The previous distinct value, after a change.
    Previous,
    /// "Before" of a value that never changed: abstain.
    PreviousAbsent,
    /// Never asserted: abstain.
    Absent,
}

impl QueryClass {
    pub fn abstains(self) -> bool {
        matches!(self, Self::PreviousAbsent | Self::Absent)
    }
}

/// One query and its gold answer inside an episode (document positions).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query {
    pub class: QueryClass,
    /// Another entity's value of the same relation was written after this
    /// fact's last write: the most recent mention of the relation is wrong.
    pub recency_trap: bool,
    pub key: (u32, u32),
    /// First answer-content token and the answer's closing EOS.
    pub answer_start: usize,
    pub answer_end: usize,
    /// The value token inside the answer, for value classes.
    pub value_position: Option<usize>,
    pub value: Option<u32>,
}

/// One protocol document with gold per-token labels.
#[derive(Clone, Debug)]
pub struct Episode {
    /// `(user, assistant)` message texts, for protocol verification.
    pub messages: Vec<(String, String)>,
    pub tokens: Vec<u32>,
    pub response_mask: Vec<u8>,
    pub tags: Vec<u32>,
    pub triggers: Vec<u32>,
    /// 1 on the tokens of a user turn, 0 elsewhere (assistant replies, markers,
    /// EOS, padding). Built from the protocol's role structure, never labels.
    pub user_turn: Vec<u8>,
    pub queries: Vec<Query>,
    /// Canonical entity atom at a tagged entity position, 0 elsewhere. In the
    /// standard world every entry is 0.
    pub entity_atom: Vec<u32>,
    /// Canonical relation atom at the surface token that carries the relation,
    /// 0 elsewhere. In the standard world every entry is 0.
    pub relation_atom: Vec<u32>,
    /// The natural world's per-clause canonical gold; empty in the standard
    /// world.
    pub clauses: Vec<Clause>,
}

/// Vocabulary, templates and piece encodings of the relation dialogues.
pub struct RelationWorld {
    names: Vec<Word>,
    held_names: Vec<Word>,
    relations: Vec<RelationSpec>,
    pieces: BTreeMap<&'static str, Vec<u32>>,
    user_marker: Vec<u32>,
    assistant_marker: Vec<u32>,
    newline: Vec<u32>,
    pub bos: u32,
    pub eos: u32,
    mode: WorldMode,
    natural: Option<NaturalWorld>,
}

/// Which world [`RelationWorld::episode`] renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldMode {
    /// The literal-role surface-keyed world (unchanged).
    Standard,
    /// The natural world with canonical `(entity, relation)` atoms.
    Natural,
}

fn every_template() -> impl Iterator<Item = Template> {
    [
        ASSERT_TRAIN,
        ASSERT_HELD,
        UPDATE_TRAIN,
        UPDATE_HELD,
        REASSERT_TRAIN,
        REASSERT_HELD,
        QUERY_TRAIN,
        QUERY_HELD,
        PREVIOUS_TRAIN,
        PREVIOUS_HELD,
    ]
    .into_iter()
    .flatten()
    .copied()
}

impl RelationWorld {
    /// Builds the world from a text encoder. Every name, relation and value
    /// must encode to exactly one token; `bos` and `eos` are the protocol's.
    pub fn new(encode: &dyn Fn(&str) -> Vec<u32>, bos: u32, eos: u32) -> Result<Self> {
        let single = |text: &'static str| -> Result<Word> {
            match encode(text).as_slice() {
                [id] => Ok(Word {
                    text: text.to_owned(),
                    id: *id,
                }),
                _ => Err(invalid(format!("{text:?} is not a single token"))),
            }
        };
        let names = NAMES_TRAIN
            .iter()
            .map(|&n| single(n))
            .collect::<Result<Vec<_>>>()?;
        let held_names = NAMES_HELD
            .iter()
            .map(|&n| single(n))
            .collect::<Result<Vec<_>>>()?;
        let mut relations = Vec::new();
        for &(word, article, values) in RELATIONS {
            let (values, from_names) = match values {
                Some(values) => (
                    values
                        .iter()
                        .map(|&v| single(v))
                        .collect::<Result<Vec<_>>>()?,
                    false,
                ),
                None => (names.clone(), true),
            };
            relations.push(RelationSpec {
                word: single(word)?,
                article,
                values,
                from_names,
            });
        }
        let mut pieces = BTreeMap::new();
        let mut add = |text: &'static str| {
            pieces.entry(text).or_insert_with(|| encode(text));
        };
        for template in every_template() {
            for part in template {
                if let Text(text) = part {
                    add(text);
                }
            }
        }
        for &text in CHAT_TRAIN
            .iter()
            .chain(CHAT_HELD)
            .chain(CHAT_REPLIES)
            .chain(ACKS)
            .chain([&ABSTAIN, &"It is", &"It was", &".", &" a", &" the"])
        {
            add(text);
        }
        if pieces.values().any(Vec::is_empty) {
            return Err(invalid("every template piece must encode to tokens"));
        }
        Ok(Self {
            names,
            held_names,
            relations,
            pieces,
            user_marker: encode("User: "),
            assistant_marker: encode("Assistant: "),
            newline: encode("\n"),
            bos,
            eos,
            mode: WorldMode::Standard,
            natural: None,
        })
    }

    /// Builds the natural world (D2-natural v2) from a text encoder. Every
    /// entity, relation and value surface must encode to exactly one token in
    /// the exact form the templates use (sentence-initial words carry no
    /// leading space). The standard pools are left empty; only
    /// [`Self::episode`] uses the natural families.
    pub fn natural(encode: &dyn Fn(&str) -> Vec<u32>, bos: u32, eos: u32) -> Result<Self> {
        let mut pieces: BTreeMap<&'static str, Vec<u32>> = BTreeMap::new();
        let natural = NaturalWorld::build(encode, &mut pieces)?;
        if pieces.values().any(Vec::is_empty) {
            return Err(invalid(
                "every natural template piece must encode to tokens",
            ));
        }
        Ok(Self {
            names: Vec::new(),
            held_names: Vec::new(),
            relations: Vec::new(),
            pieces,
            user_marker: encode("User: "),
            assistant_marker: encode("Assistant: "),
            newline: encode("\n"),
            bos,
            eos,
            mode: WorldMode::Natural,
            natural: Some(natural),
        })
    }

    /// The active world mode.
    pub fn mode(&self) -> WorldMode {
        self.mode
    }

    /// Extends the training name pool from a list of first names. A name must
    /// encode together with its leading space to exactly one token; duplicates
    /// and held-out names are skipped. Returns how many names were added.
    pub fn extend_names(
        &mut self,
        encode: &dyn Fn(&str) -> Vec<u32>,
        names: &[String],
    ) -> Result<usize> {
        let mut added = 0usize;
        for name in names {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            let text = format!(" {trimmed}");
            if self
                .held_names
                .iter()
                .chain(&self.names)
                .any(|w| w.text == text)
            {
                continue;
            }
            let word = match encode(&text).as_slice() {
                [id] => Word { text, id: *id },
                _ => return Err(invalid(format!("{text:?} is not a single token"))),
            };
            if self.relations.iter().any(|r| r.word.id == word.id) {
                continue;
            }
            for relation in self.relations.iter_mut() {
                if relation.from_names {
                    relation.values.push(word.clone());
                }
            }
            self.names.push(word);
            added += 1;
        }
        Ok(added)
    }

    /// Every slot word with its token id: names, held-out names and each
    /// relation's word and values.
    pub fn vocabulary(&self) -> serde_json::Value {
        let words = |list: &[Word]| -> Vec<(String, u32)> {
            list.iter().map(|w| (w.text.to_owned(), w.id)).collect()
        };
        serde_json::json!({
            "names": words(&self.names),
            "held_out_names": words(&self.held_names),
            "relations": self.relations.iter().map(|r| serde_json::json!({
                "word": (r.word.text.clone(), r.word.id),
                "article": r.article,
                "values": words(&r.values),
            })).collect::<Vec<_>>(),
        })
    }

    fn piece(&self, text: &str) -> Result<&[u32]> {
        self.pieces
            .get(text)
            .map(Vec::as_slice)
            .ok_or_else(|| invalid(format!("unencoded piece {text:?}")))
    }

    /// Samples one episode that fits `limit` tokens (`context + 1`), in the
    /// active world mode.
    pub fn episode(&self, rng: &mut Rng, held: bool, limit: usize) -> Result<Episode> {
        match self.mode {
            WorldMode::Standard => EpisodeBuilder::new(self, held, limit).build(rng),
            WorldMode::Natural => NaturalBuilder::new(self, held, limit)?.build(rng),
        }
    }
}

/// Gold state of one fact during generation.
#[derive(Clone, Copy, Debug)]
struct Fact {
    value: u32,
    changed: bool,
    last_was_reassert: bool,
    last_write: usize,
}

struct EpisodeBuilder<'a> {
    world: &'a RelationWorld,
    held: bool,
    limit: usize,
    tokens: Vec<u32>,
    mask: Vec<u8>,
    tags: Vec<u32>,
    triggers: Vec<u32>,
    user_turn: Vec<u8>,
    queries: Vec<Query>,
    messages: Vec<(String, String)>,
    turns: usize,
    events: usize,
    /// Relation -> index of the latest write event of any entity.
    latest_relation_write: BTreeMap<u32, (usize, u32)>,
}

/// A rendered message: tokens with tags; the trigger goes on its last token.
struct Rendered {
    text: String,
    tokens: Vec<u32>,
    tags: Vec<u32>,
    value_offset: Option<usize>,
}

impl<'a> EpisodeBuilder<'a> {
    fn new(world: &'a RelationWorld, held: bool, limit: usize) -> Self {
        Self {
            world,
            held,
            limit,
            tokens: vec![world.bos],
            mask: vec![0],
            tags: vec![TAG_OTHER],
            triggers: vec![TRIGGER_NONE],
            user_turn: vec![0],
            queries: Vec::new(),
            messages: Vec::new(),
            turns: 0,
            events: 0,
            latest_relation_write: BTreeMap::new(),
        }
    }

    fn render(
        &self,
        template: Template,
        entity: &Word,
        relation: &RelationSpec,
        value: Option<&Word>,
    ) -> Result<Rendered> {
        let mut out = Rendered {
            text: String::new(),
            tokens: Vec::new(),
            tags: Vec::new(),
            value_offset: None,
        };
        for part in template {
            match part {
                Text(text) => {
                    out.text.push_str(text);
                    let piece = self.world.piece(text)?;
                    out.tokens.extend_from_slice(piece);
                    out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
                }
                Entity => {
                    out.text.push_str(&entity.text);
                    out.tokens.push(entity.id);
                    out.tags.push(TAG_ENTITY);
                }
                Relation => {
                    out.text.push_str(&relation.word.text);
                    out.tokens.push(relation.word.id);
                    out.tags.push(TAG_RELATION);
                }
                Value => {
                    let value = value.ok_or_else(|| invalid("a value slot needs a value"))?;
                    if let Some(article) = relation.article {
                        out.text.push_str(article);
                        let piece = self.world.piece(article)?;
                        out.tokens.extend_from_slice(piece);
                        out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
                    }
                    out.text.push_str(&value.text);
                    out.value_offset = Some(out.tokens.len());
                    out.tokens.push(value.id);
                    out.tags.push(TAG_VALUE);
                }
            }
        }
        Ok(out)
    }

    fn plain(&self, text: &str) -> Result<Rendered> {
        let tokens = self.world.piece(text)?.to_vec();
        let tags = vec![TAG_OTHER; tokens.len()];
        Ok(Rendered {
            text: text.to_owned(),
            tokens,
            tags,
            value_offset: None,
        })
    }

    /// The answer `It is/was {value}.` or the abstention.
    fn answer(
        &self,
        relation: &RelationSpec,
        value: Option<&Word>,
        previous: bool,
    ) -> Result<Rendered> {
        let Some(value) = value else {
            return self.plain(ABSTAIN);
        };
        let mut out = self.plain(if previous { "It was" } else { "It is" })?;
        if let Some(article) = relation.article {
            out.text.push_str(article);
            let piece = self.world.piece(article)?;
            out.tokens.extend_from_slice(piece);
            out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
        }
        out.text.push_str(&value.text);
        out.text.push('.');
        out.value_offset = Some(out.tokens.len());
        out.tokens.push(value.id);
        out.tags.push(TAG_OTHER);
        let period = self.world.piece(".")?;
        out.tokens.extend_from_slice(period);
        out.tags
            .extend(std::iter::repeat_n(TAG_OTHER, period.len()));
        Ok(out)
    }

    fn turn_len(&self, user: &Rendered, reply: &Rendered) -> usize {
        let separator = if self.turns == 0 {
            0
        } else {
            self.world.newline.len()
        };
        separator
            + self.world.user_marker.len()
            + user.tokens.len()
            + self.world.newline.len()
            + self.world.assistant_marker.len()
            + reply.tokens.len()
            + 1
    }

    fn fits(&self, user: &Rendered, reply: &Rendered, reserve: usize) -> bool {
        self.tokens.len() + self.turn_len(user, reply) + reserve <= self.limit
    }

    fn push_plain(&mut self, tokens: &[u32]) {
        self.tokens.extend_from_slice(tokens);
        self.mask.extend(std::iter::repeat_n(0, tokens.len()));
        self.tags
            .extend(std::iter::repeat_n(TAG_OTHER, tokens.len()));
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, tokens.len()));
        self.user_turn.extend(std::iter::repeat_n(0, tokens.len()));
    }

    /// Appends one user turn and its assistant reply; returns the reply's
    /// start position.
    fn push_turn(&mut self, user: &Rendered, trigger: u32, reply: &Rendered) -> usize {
        if self.turns > 0 {
            let newline = self.world.newline.clone();
            self.push_plain(&newline);
        }
        let marker = self.world.user_marker.clone();
        self.push_plain(&marker);
        self.tokens.extend_from_slice(&user.tokens);
        self.mask.extend(std::iter::repeat_n(0, user.tokens.len()));
        self.tags.extend_from_slice(&user.tags);
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, user.tokens.len()));
        self.user_turn
            .extend(std::iter::repeat_n(1, user.tokens.len()));
        if let Some(last) = self.triggers.last_mut() {
            *last = trigger;
        }
        let newline = self.world.newline.clone();
        self.push_plain(&newline);
        let marker = self.world.assistant_marker.clone();
        self.push_plain(&marker);
        let start = self.tokens.len();
        self.tokens.extend_from_slice(&reply.tokens);
        self.mask.extend(std::iter::repeat_n(1, reply.tokens.len()));
        self.tags
            .extend(std::iter::repeat_n(TAG_OTHER, reply.tokens.len()));
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, reply.tokens.len()));
        self.user_turn
            .extend(std::iter::repeat_n(0, reply.tokens.len()));
        self.tokens.push(self.world.eos);
        self.mask.push(1);
        self.tags.push(TAG_OTHER);
        self.triggers.push(TRIGGER_NONE);
        self.user_turn.push(0);
        self.messages.push((user.text.clone(), reply.text.clone()));
        self.turns += 1;
        start
    }

    fn templates(
        &self,
        train: &'static [Template],
        held: &'static [Template],
    ) -> &'static [Template] {
        if self.held {
            held
        } else {
            train
        }
    }

    fn chat(&mut self, rng: &mut Rng, reserve: usize) -> Result<bool> {
        let pool = if self.held { CHAT_HELD } else { CHAT_TRAIN };
        let user = self.plain(pool[rng.below(pool.len())])?;
        let reply = self.plain(CHAT_REPLIES[rng.below(CHAT_REPLIES.len())])?;
        if !self.fits(&user, &reply, reserve) {
            return Ok(false);
        }
        self.push_turn(&user, TRIGGER_NONE, &reply);
        Ok(true)
    }

    /// Writes `value` to a fact with one of `templates`; false when it does not fit.
    fn write(
        &mut self,
        rng: &mut Rng,
        templates: &'static [Template],
        entity: &Word,
        relation: &RelationSpec,
        value: &Word,
        reserve: usize,
    ) -> Result<bool> {
        let template = templates[rng.below(templates.len())];
        let user = self.render(template, entity, relation, Some(value))?;
        let reply = self.plain(ACKS[rng.below(ACKS.len())])?;
        if !self.fits(&user, &reply, reserve) {
            return Ok(false);
        }
        self.push_turn(&user, TRIGGER_WRITE, &reply);
        self.events += 1;
        self.latest_relation_write
            .insert(relation.word.id, (self.events, entity.id));
        Ok(true)
    }

    fn build(mut self, rng: &mut Rng) -> Result<Episode> {
        let world = self.world;
        const QUERY_RESERVE: usize = 34;
        // Cast: two or three people and two or three relations.
        let people_count = 2 + rng.below(2);
        let relation_count = 2 + rng.below(2);
        let mut people: Vec<Word> = Vec::new();
        while people.len() < people_count {
            let pool = if self.held && (people.is_empty() || rng.below(2) == 0) {
                &world.held_names
            } else {
                &world.names
            };
            let candidate = pool[rng.below(pool.len())].clone();
            if people.iter().all(|p| p.id != candidate.id) {
                people.push(candidate);
            }
        }
        let mut relation_ids: Vec<usize> = Vec::new();
        while relation_ids.len() < relation_count {
            let r = rng.below(world.relations.len());
            if !relation_ids.contains(&r) {
                relation_ids.push(r);
            }
        }
        // Facts: two to four distinct (person, relation) pairs.
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for p in 0..people.len() {
            for &r in &relation_ids {
                pairs.push((p, r));
            }
        }
        let fact_count = (2 + rng.below(3)).min(pairs.len());
        let mut chosen = Vec::new();
        while chosen.len() < fact_count {
            let pair = pairs[rng.below(pairs.len())];
            if !chosen.contains(&pair) {
                chosen.push(pair);
            }
        }
        let value_for =
            |rng: &mut Rng, relation: &RelationSpec, person: &Word, avoid: Option<u32>| -> Word {
                loop {
                    let candidate = &relation.values[rng.below(relation.values.len())];
                    if Some(candidate.id) != avoid && candidate.id != person.id {
                        return candidate.clone();
                    }
                }
            };
        let mut facts: BTreeMap<(usize, usize), Fact> = BTreeMap::new();
        // Assertions, with chit-chat between.
        for &(p, r) in &chosen {
            if rng.below(10) < 3 && !self.chat(rng, QUERY_RESERVE * 3)? {
                break;
            }
            let relation = &world.relations[r];
            let value = value_for(rng, relation, &people[p], None);
            if !self.write(
                rng,
                self.templates(ASSERT_TRAIN, ASSERT_HELD),
                &people[p],
                relation,
                &value,
                QUERY_RESERVE * 3,
            )? {
                break;
            }
            facts.insert(
                (p, r),
                Fact {
                    value: value.id,
                    changed: false,
                    last_was_reassert: false,
                    last_write: self.events,
                },
            );
        }
        if facts.is_empty() {
            return Err(invalid("an episode must assert at least one fact"));
        }
        // Updates, reassertions and chit-chat; at least one update is attempted.
        let middle = 2 + rng.below(4);
        let keys: Vec<(usize, usize)> = facts.keys().copied().collect();
        for index in 0..middle {
            let key = keys[rng.below(keys.len())];
            let roll = if index == 0 { 0 } else { rng.below(10) };
            let (p, r) = key;
            let relation = &world.relations[r];
            let Some(fact) = facts.get(&key).copied() else {
                continue;
            };
            let written = if roll < 5 {
                let value = value_for(rng, relation, &people[p], Some(fact.value));
                let ok = self.write(
                    rng,
                    self.templates(UPDATE_TRAIN, UPDATE_HELD),
                    &people[p],
                    relation,
                    &value,
                    QUERY_RESERVE * 2,
                )?;
                if ok {
                    facts.insert(
                        key,
                        Fact {
                            value: value.id,
                            changed: true,
                            last_was_reassert: false,
                            last_write: self.events,
                        },
                    );
                }
                ok
            } else if roll < 7 {
                let value = relation
                    .values
                    .iter()
                    .find(|v| v.id == fact.value)
                    .cloned()
                    .ok_or_else(|| invalid("fact value outside its relation"))?;
                let ok = self.write(
                    rng,
                    self.templates(REASSERT_TRAIN, REASSERT_HELD),
                    &people[p],
                    relation,
                    &value,
                    QUERY_RESERVE * 2,
                )?;
                if ok {
                    facts.insert(
                        key,
                        Fact {
                            last_was_reassert: true,
                            last_write: self.events,
                            ..fact
                        },
                    );
                }
                ok
            } else {
                self.chat(rng, QUERY_RESERVE * 2)?
            };
            if !written {
                break;
            }
        }
        // Queries: two or three, the last drawn with the gated class favoured.
        let query_count = 2 + rng.below(2);
        for index in 0..query_count {
            let last = index + 1 == query_count;
            let mut available: Vec<(QueryClass, (usize, usize))> = Vec::new();
            for (&key, fact) in &facts {
                let class = if fact.last_was_reassert {
                    QueryClass::Reasserted
                } else if fact.changed {
                    QueryClass::Updated
                } else {
                    QueryClass::First
                };
                available.push((class, key));
                available.push((
                    if fact.changed {
                        QueryClass::Previous
                    } else {
                        QueryClass::PreviousAbsent
                    },
                    key,
                ));
            }
            for p in 0..people.len() {
                for &r in &relation_ids {
                    if !facts.contains_key(&(p, r)) {
                        available.push((QueryClass::Absent, (p, r)));
                    }
                }
            }
            let want_updated = last && rng.below(10) < 4;
            let pick = if want_updated {
                available
                    .iter()
                    .filter(|(c, _)| *c == QueryClass::Updated)
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let (class, (p, r)) = if pick.is_empty() {
                available[rng.below(available.len())]
            } else {
                pick[rng.below(pick.len())]
            };
            let relation = &world.relations[r];
            let entity = &people[p];
            let previous_query = matches!(class, QueryClass::Previous | QueryClass::PreviousAbsent);
            let (templates, trigger) = if previous_query {
                (
                    self.templates(PREVIOUS_TRAIN, PREVIOUS_HELD),
                    TRIGGER_READ_PREVIOUS,
                )
            } else {
                (self.templates(QUERY_TRAIN, QUERY_HELD), TRIGGER_READ)
            };
            let template = templates[rng.below(templates.len())];
            let user = self.render(template, entity, relation, None)?;
            // Gold answer from the gold store simulated so far.
            let simulation = simulate(&self.tokens, &self.tags, &self.triggers, world.eos)?;
            let (status, value) =
                simulation
                    .store
                    .read(entity.id, relation.word.id, previous_query);
            let value_word = if status == STATUS_HIT {
                Some(
                    relation
                        .values
                        .iter()
                        .find(|v| v.id == value)
                        .cloned()
                        .ok_or_else(|| invalid("stored value outside its relation"))?,
                )
            } else {
                None
            };
            if value_word.is_some() == class.abstains() {
                return Err(invalid("query class disagrees with the gold store"));
            }
            let reply = self.answer(relation, value_word.as_ref(), previous_query)?;
            if !self.fits(&user, &reply, 0) {
                break;
            }
            let recency_trap = facts.get(&(p, r)).is_some_and(|fact| {
                self.latest_relation_write
                    .get(&relation.word.id)
                    .is_some_and(|&(event, writer)| event > fact.last_write && writer != entity.id)
            });
            let start = self.push_turn(&user, trigger, &reply);
            let end = self.tokens.len() - 1;
            self.queries.push(Query {
                class,
                recency_trap,
                key: (entity.id, relation.word.id),
                answer_start: start,
                answer_end: end,
                value_position: reply.value_offset.map(|o| start + o),
                value: value_word.map(|w| w.id),
            });
        }
        if self.queries.is_empty() {
            return Err(invalid("an episode must end with at least one query"));
        }
        let tokens = self.tokens.len();
        Ok(Episode {
            messages: self.messages,
            tokens: self.tokens,
            response_mask: self.mask,
            tags: self.tags,
            triggers: self.triggers,
            user_turn: self.user_turn,
            queries: self.queries,
            entity_atom: vec![0; tokens],
            relation_atom: vec![0; tokens],
            clauses: Vec::new(),
        })
    }
}

/// A padded batch of episodes, as model inputs and next-token targets.
pub struct DialogueBatch {
    pub episodes: Vec<Episode>,
    pub batch: usize,
    pub time: usize,
    pub ids: Vec<u32>,
    pub targets: Vec<u32>,
    /// Response weights on targets (assistant content and EOS).
    pub response: Vec<f32>,
    /// 1 on real input positions, 0 on padding.
    pub real: Vec<f32>,
    pub tags: Vec<u32>,
    pub triggers: Vec<u32>,
    /// Canonical atoms per input position, from the natural world; 0 for the
    /// standard world and padding.
    pub entity_atom: Vec<u32>,
    pub relation_atom: Vec<u32>,
    /// 1 on user-turn input positions, 0 elsewhere (assistant, padding).
    pub user_turn: Vec<u8>,
    /// Gold registers (teacher forcing) for both the current and previous
    /// registers.
    pub status: Vec<u32>,
    pub value: Vec<u32>,
    pub status_previous: Vec<u32>,
    pub value_previous: Vec<u32>,
}

impl RelationWorld {
    /// `batch` fresh episodes padded to `context` inputs.
    pub fn batch(
        &self,
        rng: &mut Rng,
        batch: usize,
        context: usize,
        held: bool,
    ) -> Result<DialogueBatch> {
        let mut episodes = Vec::with_capacity(batch);
        for _ in 0..batch {
            episodes.push(self.episode(rng, held, context + 1)?);
        }
        self.pad(episodes, context)
    }

    /// Pads `episodes` to `context` inputs with EOS, weight zero.
    pub fn pad(&self, episodes: Vec<Episode>, context: usize) -> Result<DialogueBatch> {
        let batch = episodes.len();
        let rows = batch * context;
        let mut out = DialogueBatch {
            episodes: Vec::new(),
            batch,
            time: context,
            ids: vec![self.eos; rows],
            targets: vec![self.eos; rows],
            response: vec![0.0; rows],
            real: vec![0.0; rows],
            tags: vec![TAG_OTHER; rows],
            triggers: vec![TRIGGER_NONE; rows],
            entity_atom: vec![0; rows],
            relation_atom: vec![0; rows],
            user_turn: vec![0; rows],
            status: vec![STATUS_NONE; rows],
            value: vec![0; rows],
            status_previous: vec![STATUS_NONE; rows],
            value_previous: vec![0; rows],
        };
        for (b, episode) in episodes.iter().enumerate() {
            let n = episode.tokens.len();
            if n < 2 || n > context + 1 {
                return Err(invalid("an episode must fit the context"));
            }
            let inputs = n - 1;
            let gold = simulate_gated(
                &episode.tokens[..inputs],
                &episode.tags[..inputs],
                &episode.triggers[..inputs],
                self.eos,
                &episode.user_turn[..inputs],
            )?;
            for t in 0..inputs {
                let row = b * context + t;
                out.ids[row] = episode.tokens[t];
                out.targets[row] = episode.tokens[t + 1];
                out.response[row] = f32::from(episode.response_mask[t + 1]);
                out.real[row] = 1.0;
                out.tags[row] = episode.tags[t];
                out.triggers[row] = episode.triggers[t];
                out.entity_atom[row] = episode.entity_atom[t];
                out.relation_atom[row] = episode.relation_atom[t];
                out.user_turn[row] = episode.user_turn[t];
                out.status[row] = gold.status[t];
                out.value[row] = gold.value[t];
                out.status_previous[row] = gold.status_previous[t];
                out.value_previous[row] = gold.value_previous[t];
            }
        }
        out.episodes = episodes;
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// D2-natural v2: the canonical-key natural relation world.

/// One part of a natural phrasing template. `Entity`/`Relation` carry the exact
/// surface word used at that slot (each must encode to one token); `Value` is
/// filled from the family's value pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NSlot {
    Text(&'static str),
    Entity(&'static str),
    Relation(&'static str),
    Value,
}

const fn nt(text: &'static str) -> NSlot {
    NSlot::Text(text)
}
const fn ne(entity: &'static str) -> NSlot {
    NSlot::Entity(entity)
}
const fn nr(relation: &'static str) -> NSlot {
    NSlot::Relation(relation)
}
const NY: NSlot = NSlot::Value;

type NTemplate = &'static [NSlot];

/// A natural `(entity, relation)` family: its canonical key, value pools and
/// train/held-out phrasing sets. The two sets and their value pools are
/// disjoint by construction.
struct NaturalFamily {
    entity: u32,
    relation: u32,
    article: Option<&'static str>,
    values_train: &'static [&'static str],
    values_held: &'static [&'static str],
    statements_train: &'static [NTemplate],
    statements_held: &'static [NTemplate],
    queries_train: &'static [NTemplate],
    queries_held: &'static [NTemplate],
    previous_train: &'static [NTemplate],
    previous_held: &'static [NTemplate],
}

/// Update/reassert wrappers reuse a family statement template, so every family
/// gets grammatical update and reassertion phrasings without duplicating them.
/// The lead-ins and endings are distinct between train and held-out.
const UPDATE_LEAD_TRAIN: &str = "Update: ";
const UPDATE_LEAD_HELD: &str = "These days: ";
const REASSERT_LEAD_TRAIN: &str = "Nothing changed: ";
const REASSERT_LEAD_HELD: &str = "As always: ";
const UPDATE_END: &str = " now.";
const REASSERT_END: &str = " still.";

/// Chatter turns and hard-negative distractors with no memory operation.
const NATURAL_CHAT_TRAIN: &[&str] = &[
    "I also like rainy days.",
    "The weather is nice today.",
    "My neighbor has a cat too.",
    "I saw a dog at the park.",
    "My brother rides his bike.",
    "I think red is a warm color.",
    "The nurse is very kind.",
    "We went to the beach last week.",
    "My friend and I play outside.",
    "The garden looks lovely today.",
];
/// Panel sentences (and their near neighbours) held out of training entirely.
const NATURAL_CHAT_HELD: &[&str] = &[
    "I also like apples.",
    "I also like blue.",
    "It is hard but fun.",
    "She likes to sleep.",
];

/// Pronoun follow-ups after a possessed-entity fact: the memory operation
/// concerns the entity, the follow-up turn does not.
const FOLLOW_CAT_TRAIN: &[&str] = &["She purrs loudly.", "She sleeps all day."];
const FOLLOW_CAT_HELD: &[&str] = &["She likes to sleep."];
const FOLLOW_DOG_TRAIN: &[&str] = &["He runs in the yard.", "He is very friendly."];
const FOLLOW_DOG_HELD: &[&str] = &["He likes to play."];
const FOLLOW_SISTER_TRAIN: &[&str] = &["She calls me often.", "She is very kind."];
const FOLLOW_SISTER_HELD: &[&str] = &["She visits often."];
const FOLLOW_BROTHERS_TRAIN: &[&str] = &["They are all taller.", "They make me laugh."];
const FOLLOW_BROTHERS_HELD: &[&str] = &["They are older than me."];
const FOLLOW_BROTHER_TRAIN: &[&str] = &["He lives nearby.", "He is a good friend."];
const FOLLOW_BROTHER_HELD: &[&str] = &["He works nearby."];
const FOLLOW_CAR_TRAIN: &[&str] = &["It is quite old.", "It runs well."];
const FOLLOW_CAR_HELD: &[&str] = &["It is fast."];
const FOLLOW_BIKE_TRAIN: &[&str] = &["It is bright red.", "It goes fast."];
const FOLLOW_BIKE_HELD: &[&str] = &["It is new."];
const FOLLOW_INSTRUMENT_TRAIN: &[&str] = &["It sounds lovely.", "It is a lot of work."];
const FOLLOW_INSTRUMENT_HELD: &[&str] = &["It is hard but fun."];

fn follow_ups(entity: u32, relation: u32, held: bool) -> Option<&'static [&'static str]> {
    match entity {
        entity_atom::CAT => Some(if held {
            FOLLOW_CAT_HELD
        } else {
            FOLLOW_CAT_TRAIN
        }),
        entity_atom::DOG => Some(if held {
            FOLLOW_DOG_HELD
        } else {
            FOLLOW_DOG_TRAIN
        }),
        entity_atom::SISTER => Some(if held {
            FOLLOW_SISTER_HELD
        } else {
            FOLLOW_SISTER_TRAIN
        }),
        entity_atom::BROTHERS => Some(if held {
            FOLLOW_BROTHERS_HELD
        } else {
            FOLLOW_BROTHERS_TRAIN
        }),
        entity_atom::BROTHER => Some(if held {
            FOLLOW_BROTHER_HELD
        } else {
            FOLLOW_BROTHER_TRAIN
        }),
        entity_atom::CAR => Some(if held {
            FOLLOW_CAR_HELD
        } else {
            FOLLOW_CAR_TRAIN
        }),
        entity_atom::BIKE => Some(if held {
            FOLLOW_BIKE_HELD
        } else {
            FOLLOW_BIKE_TRAIN
        }),
        _ if relation == relation_atom::INSTRUMENT => Some(if held {
            FOLLOW_INSTRUMENT_HELD
        } else {
            FOLLOW_INSTRUMENT_TRAIN
        }),
        _ => None,
    }
}

/// The natural families: one `(entity, relation)` pair each, covering all
/// eight entity atoms and all fourteen relation atoms. Panel values and
/// panel-relevant phrasings are held out.
const NATURAL_FAMILIES: &[NaturalFamily] = &[
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::NAME,
        article: None,
        values_train: &[" Sam", " Mia", " Tom", " Anna", " Max"],
        values_held: &[" Molly", " Leo", " Emma", " Rose"],
        statements_train: &[
            &[ne("My"), nr(" name"), nt(" is"), NY, nt(".")],
            &[ne("I"), nt(" am"), nr(" named"), NY, nt(".")],
            &[ne("I"), nt(" am"), nr(" called"), NY, nt(".")],
        ],
        statements_held: &[&[nt("They"), nr(" call"), ne(" me"), NY, nt(".")]],
        queries_train: &[
            &[nt("What am"), ne(" I"), nr(" called"), nt("?")],
            &[nt("Can you tell me"), ne(" my"), nr(" name"), nt("?")],
        ],
        queries_held: &[&[nt("What is"), ne(" my"), nr(" name"), nt("?")]],
        previous_train: &[&[nt("What was"), ne(" my"), nr(" name"), nt(" before?")]],
        previous_held: &[&[
            nt("What"),
            nr(" name"),
            nt(" did"),
            ne(" I"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::CAT,
        relation: relation_atom::NAME,
        article: None,
        values_train: &[" Luna", " Rex", " Buddy"],
        values_held: &[" Daisy", " Bella"],
        statements_train: &[
            &[nt("My"), ne(" cat"), nt(" is"), nr(" named"), NY, nt(".")],
            &[nt("My"), ne(" cat"), nt(" is"), nr(" called"), NY, nt(".")],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" cat"),
            nt(" has the"),
            nr(" name"),
            NY,
            nt("."),
        ]],
        queries_train: &[
            &[nt("What is my"), ne(" cat"), nr(" called"), nt("?")],
            &[nt("What is my"), ne(" cat"), nr(" named"), nt("?")],
        ],
        queries_held: &[&[nt("What is my"), ne(" cat"), nt("'s"), nr(" name"), nt("?")]],
        previous_train: &[&[
            nt("What was my"),
            ne(" cat"),
            nt("'s"),
            nr(" name"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("What"),
            nr(" name"),
            nt(" did my"),
            ne(" cat"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::DOG,
        relation: relation_atom::NAME,
        article: None,
        values_train: &[" Rex", " Buddy"],
        values_held: &[" Daisy", " Bella"],
        statements_train: &[
            &[nt("My"), ne(" dog"), nt(" is"), nr(" named"), NY, nt(".")],
            &[nt("My"), ne(" dog"), nt(" is"), nr(" called"), NY, nt(".")],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" dog"),
            nt(" has the"),
            nr(" name"),
            NY,
            nt("."),
        ]],
        queries_train: &[&[nt("What is my"), ne(" dog"), nr(" called"), nt("?")]],
        queries_held: &[&[nt("What is my"), ne(" dog"), nt("'s"), nr(" name"), nt("?")]],
        previous_train: &[&[
            nt("What was my"),
            ne(" dog"),
            nt("'s"),
            nr(" name"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("What"),
            nr(" name"),
            nt(" did my"),
            ne(" dog"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::JOB,
        article: Some(" a"),
        values_train: &[" doctor", " nurse", " cook", " farmer", " driver"],
        values_held: &[" teacher"],
        statements_train: &[
            &[ne("I"), nr(" work"), nt(" as a"), NY, nt(".")],
            &[ne("My"), nr(" job"), nt(" is a"), NY, nt(".")],
        ],
        statements_held: &[&[ne("My"), nr(" job"), nt(" is being a"), NY, nt(".")]],
        queries_train: &[
            &[nt("What do"), ne(" I"), nr(" work"), nt(" as?")],
            &[nt("What"), nr(" work"), nt(" do"), ne(" I"), nt(" do?")],
        ],
        queries_held: &[&[nt("What is"), ne(" my"), nr(" job"), nt("?")]],
        previous_train: &[&[nt("What was"), ne(" my"), nr(" job"), nt(" before?")]],
        previous_held: &[&[nt("What did"), ne(" I"), nr(" work"), nt(" as before?")]],
    },
    NaturalFamily {
        entity: entity_atom::BROTHER,
        relation: relation_atom::JOB,
        article: Some(" a"),
        values_train: &[" cook", " farmer", " driver"],
        values_held: &[" teacher"],
        statements_train: &[
            &[
                nt("My"),
                ne(" brother"),
                nt(" has a"),
                nr(" job"),
                nt(" as a"),
                NY,
                nt("."),
            ],
            &[
                nt("My"),
                ne(" brother"),
                nt("'s"),
                nr(" job"),
                nt(" is a"),
                NY,
                nt("."),
            ],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" brother"),
            nt(" took a"),
            nr(" job"),
            nt(" as a"),
            NY,
            nt("."),
        ]],
        queries_train: &[&[
            nt("What is my"),
            ne(" brother"),
            nt("'s"),
            nr(" job"),
            nt("?"),
        ]],
        queries_held: &[&[
            nt("What"),
            nr(" job"),
            nt(" does my"),
            ne(" brother"),
            nt(" have?"),
        ]],
        previous_train: &[&[
            nt("What was my"),
            ne(" brother"),
            nt("'s"),
            nr(" job"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("What"),
            nr(" job"),
            nt(" did my"),
            ne(" brother"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SISTER,
        relation: relation_atom::HOME,
        article: Some(" in"),
        values_train: &[" park", " farm", " beach", " town", " village", " island"],
        values_held: &[" woods", " field", " house"],
        statements_train: &[
            &[
                nt("My"),
                ne(" sister"),
                nt("'s"),
                nr(" home"),
                nt(" is"),
                NY,
                nt("."),
            ],
            &[
                nt("My"),
                ne(" sister"),
                nt(" has her"),
                nr(" home"),
                NY,
                nt("."),
            ],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" sister"),
            nt(" will"),
            nr(" stay"),
            NY,
            nt("."),
        ]],
        queries_train: &[
            &[nr("Where"), nt(" does my"), ne(" sister"), nt(" stay?")],
            &[nr("Where"), nt(" is my"), ne(" sister"), nt("'s home?")],
        ],
        queries_held: &[&[nr("Where"), nt(" does my"), ne(" sister"), nt(" live?")]],
        previous_train: &[&[
            nr("Where"),
            nt(" did my"),
            ne(" sister"),
            nt(" stay before?"),
        ]],
        previous_held: &[&[
            nr("Where"),
            nt(" did my"),
            ne(" sister"),
            nt(" live before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::HOME,
        article: Some(" in"),
        values_train: &[" river", " lake", " hill", " forest", " castle", " garden"],
        values_held: &[" zoo", " market", " school"],
        statements_train: &[
            &[ne("I"), nr(" live"), NY, nt(".")],
            &[ne("My"), nr(" home"), nt(" is"), NY, nt(".")],
        ],
        statements_held: &[&[ne("I"), nt(" will"), nr(" stay"), NY, nt(".")]],
        queries_train: &[
            &[nr("Where"), nt(" do"), ne(" I"), nt(" live?")],
            &[nr("Where"), nt(" is"), ne(" my"), nt(" home?")],
        ],
        queries_held: &[&[nr("Where"), nt(" will"), ne(" I"), nt(" stay?")]],
        previous_train: &[&[nr("Where"), nt(" did"), ne(" I"), nt(" live before?")]],
        previous_held: &[&[nr("Where"), nt(" did"), ne(" I"), nt(" stay before?")]],
    },
    NaturalFamily {
        entity: entity_atom::BROTHERS,
        relation: relation_atom::COUNT,
        article: None,
        values_train: &[" one", " three", " four", " five", " ten"],
        values_held: &[" two"],
        statements_train: &[
            &[nt("I"), nr(" have"), NY, ne(" brothers"), nt(".")],
            &[nt("My family"), nr(" has"), NY, ne(" brothers"), nt(".")],
        ],
        statements_held: &[&[nt("My"), ne(" brothers"), nr(" number"), NY, nt(".")]],
        queries_train: &[
            &[nr("How"), nt(" many"), ne(" brothers"), nt(" are there?")],
            &[nr("How"), nt(" many"), ne(" brothers"), nt(" do I count?")],
        ],
        queries_held: &[&[nr("How"), nt(" many"), ne(" brothers"), nt(" do I have?")]],
        previous_train: &[&[
            nr("How"),
            nt(" many"),
            ne(" brothers"),
            nt(" did I have before?"),
        ]],
        previous_held: &[&[
            nr("How"),
            nt(" many"),
            ne(" brothers"),
            nt(" were there before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::BIRTHDAY,
        article: Some(" in"),
        values_train: &[" spring", " fall"],
        values_held: &[" bloom"],
        statements_train: &[
            &[ne("My"), nr(" birthday"), nt(" is"), NY, nt(".")],
            &[ne("My"), nr(" birthday"), nt(" falls"), NY, nt(".")],
        ],
        statements_held: &[&[ne("My"), nr(" birthday"), nt(" comes"), NY, nt(".")]],
        queries_train: &[
            &[nr("When"), nt(" does"), ne(" my"), nt(" birthday come?")],
            &[nr("When"), nt(" does"), ne(" my"), nt(" birthday fall?")],
        ],
        queries_held: &[&[nr("When"), nt(" is"), ne(" my"), nt(" birthday?")]],
        previous_train: &[&[nr("When"), nt(" was"), ne(" my"), nt(" birthday before?")]],
        previous_held: &[&[
            nr("When"),
            nt(" did"),
            ne(" my"),
            nt(" birthday come before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::CAR,
        relation: relation_atom::CAR_COLOR,
        article: None,
        values_train: &[" red", " yellow", " pink", " purple", " orange", " brown"],
        values_held: &[" green", " blue"],
        statements_train: &[
            &[nt("I"), nr(" drive"), nt(" a"), NY, ne(" car"), nt(".")],
            &[nt("My"), ne(" car"), nt(" is"), nr(" painted"), NY, nt(".")],
        ],
        statements_held: &[
            &[
                nt("The"),
                ne(" car"),
                nt(" I"),
                nr(" drive"),
                nt(" is"),
                NY,
                nt("."),
            ],
            &[nt("I"), nr(" have"), nt(" a"), NY, ne(" car"), nt(".")],
        ],
        queries_train: &[&[nr("What"), nt(" color do"), ne(" I"), nt(" drive?")]],
        queries_held: &[&[nr("What"), nt(" color is my"), ne(" car"), nt("?")]],
        previous_train: &[&[nr("What"), nt(" color was my"), ne(" car"), nt(" before?")]],
        previous_held: &[&[
            nr("What"),
            nt(" color did my"),
            ne(" car"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::BIKE,
        relation: relation_atom::CAR_COLOR,
        article: None,
        values_train: &[" black", " white", " brown"],
        values_held: &[" green", " blue"],
        statements_train: &[
            &[nt("I"), nr(" ride"), nt(" a"), NY, ne(" bike"), nt(".")],
            &[
                nt("My"),
                ne(" bike"),
                nt(" is"),
                nr(" painted"),
                NY,
                nt("."),
            ],
        ],
        statements_held: &[&[nt("I"), nr(" have"), nt(" a"), NY, ne(" bike"), nt(".")]],
        queries_train: &[&[nr("What"), nt(" color does my"), ne(" bike"), nt(" have?")]],
        queries_held: &[&[nr("What"), nt(" color is my"), ne(" bike"), nt("?")]],
        previous_train: &[&[nr("What"), nt(" color was my"), ne(" bike"), nt(" before?")]],
        previous_held: &[&[
            nr("What"),
            nt(" color did my"),
            ne(" bike"),
            nt(" have before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::FAVORITE_COLOR,
        article: None,
        values_train: &[" red", " yellow", " pink", " purple", " orange", " brown"],
        values_held: &[" green", " blue"],
        statements_train: &[
            &[
                ne("My"),
                nt(" favorite"),
                nr(" color"),
                nt(" is"),
                NY,
                nt("."),
            ],
            &[ne("I"), nt(" love the"), nr(" color"), NY, nt(".")],
        ],
        statements_held: &[
            &[
                ne("My"),
                nt(" most loved"),
                nr(" color"),
                nt(" is"),
                NY,
                nt("."),
            ],
            &[ne("I"), nt(" like the"), nr(" color"), NY, nt(" best.")],
        ],
        queries_train: &[&[nr("What"), nt(" color do"), ne(" I"), nt(" love most?")]],
        queries_held: &[&[
            nt("What is"),
            ne(" my"),
            nt(" favorite"),
            nr(" color"),
            nt("?"),
        ]],
        previous_train: &[&[
            nt("What was"),
            ne(" my"),
            nt(" favorite"),
            nr(" color"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("What"),
            nr(" color"),
            nt(" did"),
            ne(" I"),
            nt(" love before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::FAVORITE_FOOD,
        article: None,
        values_train: &[" pasta", " rice", " bread", " soup", " salad"],
        values_held: &[" pizza"],
        statements_train: &[
            &[
                ne("My"),
                nt(" favorite"),
                nr(" food"),
                nt(" is"),
                NY,
                nt("."),
            ],
            &[
                ne("My"),
                nt(" favorite"),
                nr(" dish"),
                nt(" is"),
                NY,
                nt("."),
            ],
        ],
        statements_held: &[&[
            ne("My"),
            nt(" preferred"),
            nr(" food"),
            nt(" is"),
            NY,
            nt("."),
        ]],
        queries_train: &[&[nr("What"), nt(" food do"), ne(" I"), nt(" love?")]],
        queries_held: &[&[
            nt("What is"),
            ne(" my"),
            nt(" favorite"),
            nr(" food"),
            nt("?"),
        ]],
        previous_train: &[&[
            nt("What was"),
            ne(" my"),
            nt(" favorite"),
            nr(" food"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("What"),
            nr(" food"),
            nt(" did"),
            ne(" I"),
            nt(" love before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::INSTRUMENT,
        article: Some(" the"),
        values_train: &[" guitar", " violin", " flute"],
        values_held: &[" piano"],
        statements_train: &[
            &[ne("I"), nr(" play"), NY, nt(".")],
            &[ne("I"), nt(" am"), nr(" learning"), NY, nt(".")],
        ],
        statements_held: &[&[ne("I"), nt(" am learning to"), nr(" play"), NY, nt(".")]],
        queries_train: &[&[nr("What"), nt(" instrument do"), ne(" I"), nt(" play?")]],
        queries_held: &[&[nr("What"), nt(" instrument am"), ne(" I"), nt(" learning?")]],
        previous_train: &[&[
            nr("What"),
            nt(" instrument did"),
            ne(" I"),
            nt(" play before?"),
        ]],
        previous_held: &[&[
            nr("What"),
            nt(" instrument did"),
            ne(" I"),
            nt(" learn before?"),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::PET,
        article: Some(" a"),
        values_train: &[" bird", " fish", " frog", " bunny", " duck", " mouse"],
        values_held: &[" horse", " cow", " pig", " sheep", " goat"],
        statements_train: &[
            &[ne("I"), nr(" have"), NY, nt(".")],
            &[ne("My"), nr(" pet"), nt(" is"), NY, nt(".")],
        ],
        statements_held: &[&[ne("I"), nr(" keep"), NY, nt(".")]],
        queries_train: &[&[nr("What"), nt(" pet do"), ne(" I"), nt(" have?")]],
        queries_held: &[&[nt("What is"), ne(" my"), nr(" pet"), nt("?")]],
        previous_train: &[&[nr("What"), nt(" pet did"), ne(" I"), nt(" have before?")]],
        previous_held: &[&[nr("What"), nt(" pet did"), ne(" I"), nt(" keep before?")]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::TOY,
        article: Some(" a"),
        values_train: &[" ball", " doll", " kite", " drum", " boat"],
        values_held: &[" robot", " truck", " puzzle", " teddy"],
        statements_train: &[
            &[ne("My"), nr(" toy"), nt(" is"), NY, nt(".")],
            &[ne("My"), nt(" best"), nr(" toy"), nt(" is"), NY, nt(".")],
        ],
        statements_held: &[&[
            ne("My"),
            nt(" favorite"),
            nr(" toy"),
            nt(" is"),
            NY,
            nt("."),
        ]],
        queries_train: &[&[nr("What"), nt(" toy do"), ne(" I"), nt(" have?")]],
        queries_held: &[&[nt("What is"), ne(" my"), nr(" toy"), nt("?")]],
        previous_train: &[&[nr("What"), nt(" toy did"), ne(" I"), nt(" have before?")]],
        previous_held: &[&[nr("What"), nt(" toy did"), ne(" I"), nt(" own before?")]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::FRIEND,
        article: None,
        values_train: &[" Sam", " Mia", " Tom", " Anna", " Max"],
        values_held: &[" Molly", " Leo", " Emma", " Rose"],
        statements_train: &[
            &[ne("My"), nr(" friend"), nt(" is"), NY, nt(".")],
            &[ne("My"), nt(" best"), nr(" friend"), nt(" is"), NY, nt(".")],
        ],
        statements_held: &[&[
            ne("My"),
            nt(" closest"),
            nr(" friend"),
            nt(" is"),
            NY,
            nt("."),
        ]],
        queries_train: &[&[nt("Who is"), ne(" my"), nr(" friend"), nt("?")]],
        queries_held: &[&[nt("Tell me who"), ne(" my"), nr(" friend"), nt(" is.")]],
        previous_train: &[&[nt("Who was"), ne(" my"), nr(" friend"), nt(" before?")]],
        previous_held: &[&[nt("Who was"), ne(" my"), nr(" friend"), nt(" before that?")]],
    },
    NaturalFamily {
        entity: entity_atom::CAT,
        relation: relation_atom::AGE,
        article: None,
        values_train: &[" one", " three", " four", " five", " ten"],
        values_held: &[" two"],
        statements_train: &[
            &[
                nt("My"),
                ne(" cat"),
                nt(" is"),
                NY,
                nr(" years"),
                nt(" old."),
            ],
            &[nt("My"), ne(" cat"), nr(" turned"), NY, nt(".")],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" cat"),
            nt(" is now"),
            NY,
            nr(" years"),
            nt(" old."),
        ]],
        queries_train: &[&[nt("How"), nr(" old"), nt(" is my"), ne(" cat"), nt("?")]],
        queries_held: &[&[
            nt("Tell me how"),
            nr(" old"),
            nt(" my"),
            ne(" cat"),
            nt(" is."),
        ]],
        previous_train: &[&[
            nt("How"),
            nr(" old"),
            nt(" was my"),
            ne(" cat"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("Tell me how"),
            nr(" old"),
            nt(" my"),
            ne(" cat"),
            nt(" was before."),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SISTER,
        relation: relation_atom::AGE,
        article: None,
        values_train: &[" one", " three", " four", " five", " ten"],
        values_held: &[" two"],
        statements_train: &[
            &[
                nt("My"),
                ne(" sister"),
                nt(" is"),
                NY,
                nr(" years"),
                nt(" old."),
            ],
            &[nt("My"), ne(" sister"), nr(" turned"), NY, nt(".")],
        ],
        statements_held: &[&[
            nt("My"),
            ne(" sister"),
            nt(" is now"),
            NY,
            nr(" years"),
            nt(" old."),
        ]],
        queries_train: &[&[nt("How"), nr(" old"), nt(" is my"), ne(" sister"), nt("?")]],
        queries_held: &[&[
            nt("Tell me how"),
            nr(" old"),
            nt(" my"),
            ne(" sister"),
            nt(" is."),
        ]],
        previous_train: &[&[
            nt("How"),
            nr(" old"),
            nt(" was my"),
            ne(" sister"),
            nt(" before?"),
        ]],
        previous_held: &[&[
            nt("Tell me how"),
            nr(" old"),
            nt(" my"),
            ne(" sister"),
            nt(" was before."),
        ]],
    },
    NaturalFamily {
        entity: entity_atom::SPEAKER,
        relation: relation_atom::CITY,
        article: None,
        values_train: &[" park", " farm", " beach", " town", " village", " island"],
        values_held: &[" woods", " field", " house"],
        statements_train: &[
            &[ne("My"), nr(" city"), nt(" is called"), NY, nt(".")],
            &[ne("I"), nr(" come"), nt(" from"), NY, nt(".")],
        ],
        statements_held: &[&[ne("My"), nr(" city"), nt(" is"), NY, nt(" these days.")]],
        queries_train: &[&[
            nt("What"),
            nr(" city"),
            nt(" do"),
            ne(" I"),
            nt(" come from?"),
        ]],
        queries_held: &[&[nt("What is"), ne(" my"), nr(" city"), nt("?")]],
        previous_train: &[&[
            nr("What"),
            nt(" city did"),
            ne(" I"),
            nt(" come from before?"),
        ]],
        previous_held: &[&[
            nr("What"),
            nt(" city did"),
            ne(" I"),
            nt(" live in before?"),
        ]],
    },
];

/// A family with its surface words and value pools resolved to tokens.
struct ResolvedFamily {
    entity: u32,
    relation: u32,
    article: Option<&'static str>,
    values_train: Vec<Word>,
    values_held: Vec<Word>,
    statements_train: &'static [NTemplate],
    statements_held: &'static [NTemplate],
    queries_train: &'static [NTemplate],
    queries_held: &'static [NTemplate],
    previous_train: &'static [NTemplate],
    previous_held: &'static [NTemplate],
}

/// Encoder-bound vocabulary and phrasing data of the natural world.
struct NaturalWorld {
    families: Vec<ResolvedFamily>,
    words: BTreeMap<String, Word>,
    by_id: BTreeMap<u32, Word>,
}

fn natural_piece(
    encode: &dyn Fn(&str) -> Vec<u32>,
    pieces: &mut BTreeMap<&'static str, Vec<u32>>,
    text: &'static str,
) {
    pieces.entry(text).or_insert_with(|| encode(text));
}

fn natural_word(
    encode: &dyn Fn(&str) -> Vec<u32>,
    words: &mut BTreeMap<String, Word>,
    by_id: &mut BTreeMap<u32, Word>,
    text: &'static str,
) -> Result<()> {
    if words.contains_key(text) {
        return Ok(());
    }
    match encode(text).as_slice() {
        [id] => {
            let word = Word {
                text: text.to_owned(),
                id: *id,
            };
            by_id.entry(*id).or_insert_with(|| word.clone());
            words.insert(text.to_owned(), word);
            Ok(())
        }
        _ => Err(invalid(format!(
            "natural slot {text:?} is not a single token"
        ))),
    }
}

fn natural_template(
    encode: &dyn Fn(&str) -> Vec<u32>,
    words: &mut BTreeMap<String, Word>,
    by_id: &mut BTreeMap<u32, Word>,
    pieces: &mut BTreeMap<&'static str, Vec<u32>>,
    template: NTemplate,
) -> Result<()> {
    for part in template {
        match *part {
            NSlot::Text(text) => natural_piece(encode, pieces, text),
            NSlot::Entity(surface) | NSlot::Relation(surface) => {
                natural_word(encode, words, by_id, surface)?
            }
            NSlot::Value => {}
        }
    }
    Ok(())
}

impl NaturalWorld {
    fn build(
        encode: &dyn Fn(&str) -> Vec<u32>,
        pieces: &mut BTreeMap<&'static str, Vec<u32>>,
    ) -> Result<Self> {
        let mut words: BTreeMap<String, Word> = BTreeMap::new();
        let mut by_id: BTreeMap<u32, Word> = BTreeMap::new();
        for family in NATURAL_FAMILIES {
            for template in family
                .statements_train
                .iter()
                .chain(family.statements_held)
                .chain(family.queries_train)
                .chain(family.queries_held)
                .chain(family.previous_train)
                .chain(family.previous_held)
            {
                natural_template(encode, &mut words, &mut by_id, pieces, template)?;
            }
            if let Some(article) = family.article {
                natural_piece(encode, pieces, article);
            }
            for value in family.values_train.iter().chain(family.values_held) {
                natural_word(encode, &mut words, &mut by_id, value)?;
            }
        }
        for text in [
            UPDATE_LEAD_TRAIN,
            UPDATE_LEAD_HELD,
            REASSERT_LEAD_TRAIN,
            REASSERT_LEAD_HELD,
            UPDATE_END,
            REASSERT_END,
        ] {
            natural_piece(encode, pieces, text);
        }
        for &text in NATURAL_CHAT_TRAIN
            .iter()
            .chain(NATURAL_CHAT_HELD)
            .chain(CHAT_REPLIES)
            .chain(ACKS)
            .chain([&ABSTAIN, &"It is", &"It was"])
        {
            natural_piece(encode, pieces, text);
        }
        for entity in 1..=8u32 {
            for relation in 1..=14u32 {
                for held in [false, true] {
                    if let Some(list) = follow_ups(entity, relation, held) {
                        for &text in list {
                            natural_piece(encode, pieces, text);
                        }
                    }
                }
            }
        }
        let mut families = Vec::with_capacity(NATURAL_FAMILIES.len());
        for family in NATURAL_FAMILIES {
            let resolve = |texts: &'static [&'static str]| -> Result<Vec<Word>> {
                texts
                    .iter()
                    .map(|text| {
                        words
                            .get(*text)
                            .cloned()
                            .ok_or_else(|| invalid(format!("unresolved value {text:?}")))
                    })
                    .collect()
            };
            families.push(ResolvedFamily {
                entity: family.entity,
                relation: family.relation,
                article: family.article,
                values_train: resolve(family.values_train)?,
                values_held: resolve(family.values_held)?,
                statements_train: family.statements_train,
                statements_held: family.statements_held,
                queries_train: family.queries_train,
                queries_held: family.queries_held,
                previous_train: family.previous_train,
                previous_held: family.previous_held,
            });
        }
        Ok(Self {
            families,
            words,
            by_id,
        })
    }
}

/// A rendered natural message with its canonical atom labels.
struct NRendered {
    text: String,
    tokens: Vec<u32>,
    tags: Vec<u32>,
    entity_atom: Vec<u32>,
    relation_atom: Vec<u32>,
    entity_token: u32,
    relation_token: u32,
    value_offset: Option<usize>,
}

struct NaturalBuilder<'a> {
    world: &'a RelationWorld,
    natural: &'a NaturalWorld,
    held: bool,
    limit: usize,
    tokens: Vec<u32>,
    mask: Vec<u8>,
    tags: Vec<u32>,
    triggers: Vec<u32>,
    user_turn: Vec<u8>,
    entity_atom: Vec<u32>,
    relation_atom: Vec<u32>,
    queries: Vec<Query>,
    clauses: Vec<Clause>,
    messages: Vec<(String, String)>,
    turns: usize,
    events: usize,
    store: AtomStore,
    history: BTreeMap<(u32, u32), Fact>,
    latest_relation_write: BTreeMap<u32, (usize, u32)>,
}

impl<'a> NaturalBuilder<'a> {
    fn new(world: &'a RelationWorld, held: bool, limit: usize) -> Result<Self> {
        let natural = world
            .natural
            .as_ref()
            .ok_or_else(|| invalid("the natural builder needs a natural world"))?;
        Ok(Self {
            world,
            natural,
            held,
            limit,
            tokens: vec![world.bos],
            mask: vec![0],
            tags: vec![TAG_OTHER],
            triggers: vec![TRIGGER_NONE],
            user_turn: vec![0],
            entity_atom: vec![0],
            relation_atom: vec![0],
            queries: Vec::new(),
            clauses: Vec::new(),
            messages: Vec::new(),
            turns: 0,
            events: 0,
            store: AtomStore::new(),
            history: BTreeMap::new(),
            latest_relation_write: BTreeMap::new(),
        })
    }

    fn render(
        &self,
        family: &ResolvedFamily,
        template: &[NSlot],
        value: Option<&Word>,
    ) -> Result<NRendered> {
        let mut out = NRendered {
            text: String::new(),
            tokens: Vec::new(),
            tags: Vec::new(),
            entity_atom: Vec::new(),
            relation_atom: Vec::new(),
            entity_token: 0,
            relation_token: 0,
            value_offset: None,
        };
        let mut entities = 0usize;
        let mut relations = 0usize;
        for part in template {
            match *part {
                NSlot::Text(text) => {
                    out.text.push_str(text);
                    let piece = self.world.piece(text)?;
                    out.tokens.extend_from_slice(piece);
                    out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
                    out.entity_atom.extend(std::iter::repeat_n(0, piece.len()));
                    out.relation_atom
                        .extend(std::iter::repeat_n(0, piece.len()));
                }
                NSlot::Entity(surface) => {
                    let word = self
                        .natural
                        .words
                        .get(surface)
                        .ok_or_else(|| invalid(format!("unresolved entity {surface:?}")))?;
                    out.text.push_str(surface);
                    out.tokens.push(word.id);
                    out.tags.push(TAG_ENTITY);
                    out.entity_atom.push(family.entity);
                    out.relation_atom.push(0);
                    out.entity_token = word.id;
                    entities += 1;
                }
                NSlot::Relation(surface) => {
                    let word = self
                        .natural
                        .words
                        .get(surface)
                        .ok_or_else(|| invalid(format!("unresolved relation {surface:?}")))?;
                    out.text.push_str(surface);
                    out.tokens.push(word.id);
                    out.tags.push(TAG_RELATION);
                    out.entity_atom.push(0);
                    out.relation_atom.push(family.relation);
                    out.relation_token = word.id;
                    relations += 1;
                }
                NSlot::Value => {
                    let value = value.ok_or_else(|| invalid("a value slot needs a value"))?;
                    if let Some(article) = family.article {
                        out.text.push_str(article);
                        let piece = self.world.piece(article)?;
                        out.tokens.extend_from_slice(piece);
                        out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
                        out.entity_atom.extend(std::iter::repeat_n(0, piece.len()));
                        out.relation_atom
                            .extend(std::iter::repeat_n(0, piece.len()));
                    }
                    out.text.push_str(&value.text);
                    out.value_offset = Some(out.tokens.len());
                    out.tokens.push(value.id);
                    out.tags.push(TAG_VALUE);
                    out.entity_atom.push(0);
                    out.relation_atom.push(0);
                }
            }
        }
        if entities != 1 || relations != 1 {
            return Err(invalid(
                "a natural template needs one entity and one relation slot",
            ));
        }
        Ok(out)
    }

    fn plain(&self, text: &str) -> Result<NRendered> {
        let tokens = self.world.piece(text)?.to_vec();
        let tags = vec![TAG_OTHER; tokens.len()];
        Ok(NRendered {
            text: text.to_owned(),
            entity_atom: vec![0; tokens.len()],
            relation_atom: vec![0; tokens.len()],
            tokens,
            tags,
            entity_token: 0,
            relation_token: 0,
            value_offset: None,
        })
    }

    fn answer(
        &self,
        family: &ResolvedFamily,
        value: Option<&Word>,
        previous: bool,
    ) -> Result<NRendered> {
        let Some(value) = value else {
            return self.plain(ABSTAIN);
        };
        let mut out = self.plain(if previous { "It was" } else { "It is" })?;
        if let Some(article) = family.article {
            out.text.push_str(article);
            let piece = self.world.piece(article)?;
            out.tokens.extend_from_slice(piece);
            out.tags.extend(std::iter::repeat_n(TAG_OTHER, piece.len()));
        }
        out.text.push_str(&value.text);
        out.text.push('.');
        out.value_offset = Some(out.tokens.len());
        out.tokens.push(value.id);
        out.tags.push(TAG_OTHER);
        let period = self.world.piece(".")?;
        out.tokens.extend_from_slice(period);
        out.tags
            .extend(std::iter::repeat_n(TAG_OTHER, period.len()));
        out.entity_atom = vec![0; out.tokens.len()];
        out.relation_atom = vec![0; out.tokens.len()];
        Ok(out)
    }

    fn turn_len(&self, user: &NRendered, reply: &NRendered) -> usize {
        let separator = if self.turns == 0 {
            0
        } else {
            self.world.newline.len()
        };
        separator
            + self.world.user_marker.len()
            + user.tokens.len()
            + self.world.newline.len()
            + self.world.assistant_marker.len()
            + reply.tokens.len()
            + 1
    }

    fn fits(&self, user: &NRendered, reply: &NRendered, reserve: usize) -> bool {
        self.tokens.len() + self.turn_len(user, reply) + reserve <= self.limit
    }

    fn push_plain(&mut self, tokens: &[u32]) {
        self.tokens.extend_from_slice(tokens);
        self.mask.extend(std::iter::repeat_n(0, tokens.len()));
        self.tags
            .extend(std::iter::repeat_n(TAG_OTHER, tokens.len()));
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, tokens.len()));
        self.user_turn.extend(std::iter::repeat_n(0, tokens.len()));
        self.entity_atom
            .extend(std::iter::repeat_n(0, tokens.len()));
        self.relation_atom
            .extend(std::iter::repeat_n(0, tokens.len()));
    }

    /// Appends one user turn and its reply; returns the reply start and the
    /// position of the trigger on the user turn's last token.
    fn push_turn(&mut self, user: &NRendered, trigger: u32, reply: &NRendered) -> (usize, usize) {
        if self.turns > 0 {
            let newline = self.world.newline.clone();
            self.push_plain(&newline);
        }
        let marker = self.world.user_marker.clone();
        self.push_plain(&marker);
        self.tokens.extend_from_slice(&user.tokens);
        self.mask.extend(std::iter::repeat_n(0, user.tokens.len()));
        self.tags.extend_from_slice(&user.tags);
        self.entity_atom.extend_from_slice(&user.entity_atom);
        self.relation_atom.extend_from_slice(&user.relation_atom);
        self.user_turn
            .extend(std::iter::repeat_n(1, user.tokens.len()));
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, user.tokens.len()));
        let trigger_position = self.tokens.len() - 1;
        if let Some(last) = self.triggers.last_mut() {
            *last = trigger;
        }
        let newline = self.world.newline.clone();
        self.push_plain(&newline);
        let marker = self.world.assistant_marker.clone();
        self.push_plain(&marker);
        let start = self.tokens.len();
        self.tokens.extend_from_slice(&reply.tokens);
        self.mask.extend(std::iter::repeat_n(1, reply.tokens.len()));
        self.tags
            .extend(std::iter::repeat_n(TAG_OTHER, reply.tokens.len()));
        self.triggers
            .extend(std::iter::repeat_n(TRIGGER_NONE, reply.tokens.len()));
        self.entity_atom
            .extend(std::iter::repeat_n(0, reply.tokens.len()));
        self.relation_atom
            .extend(std::iter::repeat_n(0, reply.tokens.len()));
        self.user_turn
            .extend(std::iter::repeat_n(0, reply.tokens.len()));
        self.tokens.push(self.world.eos);
        self.mask.push(1);
        self.tags.push(TAG_OTHER);
        self.triggers.push(TRIGGER_NONE);
        self.entity_atom.push(0);
        self.relation_atom.push(0);
        self.user_turn.push(0);
        self.messages.push((user.text.clone(), reply.text.clone()));
        self.turns += 1;
        (start, trigger_position)
    }

    fn chat(&mut self, rng: &mut Rng, reserve: usize) -> Result<bool> {
        let pool = if self.held {
            NATURAL_CHAT_HELD
        } else {
            NATURAL_CHAT_TRAIN
        };
        let user = self.plain(pool[rng.below(pool.len())])?;
        let reply = self.plain(CHAT_REPLIES[rng.below(CHAT_REPLIES.len())])?;
        if !self.fits(&user, &reply, reserve) {
            return Ok(false);
        }
        self.push_turn(&user, TRIGGER_NONE, &reply);
        Ok(true)
    }

    /// A pronoun follow-up turn after a possessed-entity statement, when the
    /// family has one.
    fn follow_up(
        &mut self,
        family: &ResolvedFamily,
        rng: &mut Rng,
        reserve: usize,
    ) -> Result<bool> {
        let Some(list) = follow_ups(family.entity, family.relation, self.held) else {
            return Ok(true);
        };
        let user = self.plain(list[rng.below(list.len())])?;
        let reply = self.plain(CHAT_REPLIES[rng.below(CHAT_REPLIES.len())])?;
        if !self.fits(&user, &reply, reserve) {
            return Ok(false);
        }
        self.push_turn(&user, TRIGGER_NONE, &reply);
        Ok(true)
    }

    fn emit_statement(
        &mut self,
        family: &ResolvedFamily,
        template: &[NSlot],
        value: &Word,
        reserve: usize,
        rng: &mut Rng,
    ) -> Result<bool> {
        let class = match self.history.get(&(family.entity, family.relation)) {
            None => QueryClass::First,
            Some(fact) if fact.value == value.id => QueryClass::Reasserted,
            Some(_) => QueryClass::Updated,
        };
        let user = self.render(family, template, Some(value))?;
        let reply = self.plain(ACKS[rng.below(ACKS.len())])?;
        if !self.fits(&user, &reply, reserve) {
            return Ok(false);
        }
        let (_, trigger_position) = self.push_turn(&user, TRIGGER_WRITE, &reply);
        self.store.write(family.entity, family.relation, value.id);
        self.events += 1;
        let changed = self
            .history
            .get(&(family.entity, family.relation))
            .is_some_and(|fact| fact.changed);
        self.history.insert(
            (family.entity, family.relation),
            Fact {
                value: value.id,
                changed: changed || class == QueryClass::Updated,
                last_was_reassert: class == QueryClass::Reasserted,
                last_write: self.events,
            },
        );
        self.latest_relation_write
            .insert(family.relation, (self.events, family.entity));
        self.clauses.push(Clause {
            trigger_position,
            entity_atom: family.entity,
            relation_atom: family.relation,
            value: Some(value.id),
            previous: false,
            class,
        });
        Ok(true)
    }

    fn emit_query(
        &mut self,
        family: &ResolvedFamily,
        template: &[NSlot],
        previous: bool,
        class: QueryClass,
    ) -> Result<bool> {
        let user = self.render(family, template, None)?;
        let (status, value) = self.store.read(family.entity, family.relation, previous);
        let value_word = if status == STATUS_HIT {
            self.natural.by_id.get(&value).cloned()
        } else {
            None
        };
        if value_word.is_some() == class.abstains() {
            return Err(invalid("natural query class disagrees with the gold store"));
        }
        let reply = self.answer(family, value_word.as_ref(), previous)?;
        if !self.fits(&user, &reply, 0) {
            return Ok(false);
        }
        let recency_trap = self
            .history
            .get(&(family.entity, family.relation))
            .is_some_and(|fact| {
                self.latest_relation_write
                    .get(&family.relation)
                    .is_some_and(|&(event, writer)| {
                        event > fact.last_write && writer != family.entity
                    })
            });
        let (start, trigger_position) = self.push_turn(
            &user,
            if previous {
                TRIGGER_READ_PREVIOUS
            } else {
                TRIGGER_READ
            },
            &reply,
        );
        let end = self.tokens.len() - 1;
        self.queries.push(Query {
            class,
            recency_trap,
            key: (user.entity_token, user.relation_token),
            answer_start: start,
            answer_end: end,
            value_position: reply.value_offset.map(|o| start + o),
            value: value_word.as_ref().map(|w| w.id),
        });
        self.clauses.push(Clause {
            trigger_position,
            entity_atom: family.entity,
            relation_atom: family.relation,
            value: value_word.map(|w| w.id),
            previous,
            class,
        });
        Ok(true)
    }

    fn build(mut self, rng: &mut Rng) -> Result<Episode> {
        const RESERVE: usize = 40;
        let families = &self.natural.families;
        let panel: Vec<usize> = (0..families.len())
            .filter(|&f| (1..=9).contains(&families[f].relation))
            .collect();
        let count = 2 + rng.below(3);
        let mut chosen: Vec<usize> = vec![panel[rng.below(panel.len())]];
        while chosen.len() < count.min(families.len()) {
            let candidate = rng.below(families.len());
            if !chosen.contains(&candidate) {
                chosen.push(candidate);
            }
        }
        for &index in &chosen {
            let family = &families[index];
            if rng.below(10) < 2 && !self.chat(rng, RESERVE * 3)? {
                break;
            }
            let pool = if self.held {
                &family.values_held
            } else {
                &family.values_train
            };
            let value = pool[rng.below(pool.len())].clone();
            let templates = if self.held {
                family.statements_held
            } else {
                family.statements_train
            };
            let template = templates[rng.below(templates.len())];
            if !self.emit_statement(family, template, &value, RESERVE * 3, rng)? {
                break;
            }
            if !self.follow_up(family, rng, RESERVE * 2)? {
                break;
            }
        }
        if self.history.is_empty() {
            return Err(invalid("a natural episode must assert at least one fact"));
        }
        let middle = 1 + rng.below(3);
        for _ in 0..middle {
            let keys: Vec<(u32, u32)> = self.history.keys().copied().collect();
            let key = keys[rng.below(keys.len())];
            let index = chosen
                .iter()
                .copied()
                .find(|&f| families[f].entity == key.0 && families[f].relation == key.1);
            let Some(index) = index else {
                break;
            };
            let family = &families[index];
            let fact = self.history[&key];
            let roll = rng.below(10);
            if roll < 5 {
                let pool = if self.held {
                    &family.values_held
                } else {
                    &family.values_train
                };
                let options: Vec<Word> = pool
                    .iter()
                    .filter(|value| value.id != fact.value)
                    .cloned()
                    .collect();
                if options.is_empty() {
                    continue;
                }
                let value = options[rng.below(options.len())].clone();
                let templates = if self.held {
                    family.statements_held
                } else {
                    family.statements_train
                };
                let base = templates[rng.below(templates.len())];
                let lead = if self.held {
                    UPDATE_LEAD_HELD
                } else {
                    UPDATE_LEAD_TRAIN
                };
                let template = wrapped(base, lead, UPDATE_END);
                if !self.emit_statement(family, &template, &value, RESERVE * 2, rng)? {
                    break;
                }
            } else if roll < 7 {
                let value = family
                    .values_train
                    .iter()
                    .chain(&family.values_held)
                    .find(|value| value.id == fact.value)
                    .cloned()
                    .ok_or_else(|| invalid("natural fact value outside its pools"))?;
                let templates = if self.held {
                    family.statements_held
                } else {
                    family.statements_train
                };
                let base = templates[rng.below(templates.len())];
                let lead = if self.held {
                    REASSERT_LEAD_HELD
                } else {
                    REASSERT_LEAD_TRAIN
                };
                let template = wrapped(base, lead, REASSERT_END);
                if !self.emit_statement(family, &template, &value, RESERVE * 2, rng)? {
                    break;
                }
            } else if !self.chat(rng, RESERVE * 2)? {
                break;
            }
        }
        let query_count = 2 + rng.below(2);
        for position in 0..query_count {
            let last = position + 1 == query_count;
            let mut available: Vec<(QueryClass, usize, bool)> = Vec::new();
            for &index in &chosen {
                let family = &families[index];
                let Some(fact) = self.history.get(&(family.entity, family.relation)) else {
                    continue;
                };
                let class = if fact.last_was_reassert {
                    QueryClass::Reasserted
                } else if fact.changed {
                    QueryClass::Updated
                } else {
                    QueryClass::First
                };
                available.push((class, index, false));
                available.push((
                    if fact.changed {
                        QueryClass::Previous
                    } else {
                        QueryClass::PreviousAbsent
                    },
                    index,
                    true,
                ));
            }
            for index in 0..families.len() {
                if !chosen.contains(&index) {
                    available.push((QueryClass::Absent, index, false));
                }
            }
            let want_updated = last && rng.below(10) < 4;
            let pick: Vec<(QueryClass, usize, bool)> = if want_updated {
                available
                    .iter()
                    .filter(|(class, _, _)| *class == QueryClass::Updated)
                    .copied()
                    .collect()
            } else {
                Vec::new()
            };
            let (class, index, previous) = if pick.is_empty() {
                available[rng.below(available.len())]
            } else {
                pick[rng.below(pick.len())]
            };
            let family = &families[index];
            let templates = if previous {
                if self.held {
                    family.previous_held
                } else {
                    family.previous_train
                }
            } else if self.held {
                family.queries_held
            } else {
                family.queries_train
            };
            let template = templates[rng.below(templates.len())];
            if !self.emit_query(family, template, previous, class)? {
                break;
            }
        }
        if self.queries.is_empty() {
            return Err(invalid(
                "a natural episode must end with at least one query",
            ));
        }
        Ok(Episode {
            messages: self.messages,
            tokens: self.tokens,
            response_mask: self.mask,
            tags: self.tags,
            triggers: self.triggers,
            user_turn: self.user_turn,
            queries: self.queries,
            entity_atom: self.entity_atom,
            relation_atom: self.relation_atom,
            clauses: self.clauses,
        })
    }
}

/// A statement template with a lead-in and a replaced sentence-final period,
/// so update and reassertion phrasings derive from the family's statements.
fn wrapped(base: &[NSlot], prefix: &'static str, ending: &'static str) -> Vec<NSlot> {
    let mut parts = Vec::with_capacity(base.len() + 1);
    parts.push(NSlot::Text(prefix));
    parts.extend_from_slice(&base[..base.len().saturating_sub(1)]);
    parts.push(NSlot::Text(ending));
    parts
}

// ---------------------------------------------------------------------------
// The split model.

fn normal_var(rng: &mut Rng, shape: &[usize], std: f64, device: &Device) -> Result<Var> {
    let count: usize = shape.iter().product();
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        // Box-Muller from two uniforms in (0, 1].
        let u1 = ((rng.next_u64() >> 11) as f64 + 1.0) / (1u64 << 53) as f64;
        let u2 = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let radius = (-2.0 * u1.ln()).sqrt();
        values.push((radius * (std::f64::consts::TAU * u2).cos() * std) as f32);
    }
    Ok(Var::from_vec(values, shape, device)?)
}

/// The zero-initialised memory branch: the branch contributes nothing until it
/// is trained.
fn zero_memory_branch(width: usize, device: &Device) -> Result<MemoryBranch> {
    Ok(MemoryBranch {
        status: zeros_var(&[STATUSES, width], device)?,
        projection: zeros_var(&[width, width], device)?,
        copy_weight: zeros_var(&[width, 1], device)?,
        copy_bias: zeros_var(&[1], device)?,
        copy_scale: zeros_var(&[1], device)?,
        status_previous: zeros_var(&[STATUSES, width], device)?,
        projection_previous: zeros_var(&[width, width], device)?,
        copy_weight_previous: zeros_var(&[width, 1], device)?,
        copy_bias_previous: zeros_var(&[1], device)?,
        copy_scale_previous: zeros_var(&[1], device)?,
    })
}

fn zeros_var(shape: &[usize], device: &Device) -> Result<Var> {
    Ok(Var::from_tensor(&Tensor::zeros(
        shape,
        DType::F32,
        device,
    )?)?)
}

/// The memory branch: one status embedding, value projection and copy boost per
/// register (current and previous).
struct MemoryBranch {
    status: Var,
    projection: Var,
    copy_weight: Var,
    copy_bias: Var,
    copy_scale: Var,
    status_previous: Var,
    projection_previous: Var,
    copy_weight_previous: Var,
    copy_bias_previous: Var,
    copy_scale_previous: Var,
}

impl MemoryBranch {
    fn registers_side(
        &self,
        embedding: &Tensor,
        batch: usize,
        time: usize,
        width: usize,
        status: &[u32],
        value: &[u32],
        previous: bool,
    ) -> Result<Tensor> {
        let rows = status.len();
        let device = embedding.device();
        let (status_var, projection) = if previous {
            (&self.status_previous, &self.projection_previous)
        } else {
            (&self.status, &self.projection)
        };
        let hit: Vec<f32> = status
            .iter()
            .map(|&s| if s == STATUS_HIT { 1.0 } else { 0.0 })
            .collect();
        let hit = Tensor::from_vec(hit, (rows, 1), device)?;
        let status_index = Tensor::from_vec(status.to_vec(), rows, device)?;
        let value_index = Tensor::from_vec(value.to_vec(), rows, device)?;
        let value_rows = embedding
            .index_select(&value_index, 0)?
            .broadcast_mul(&hit)?;
        Ok(status_var
            .as_tensor()
            .index_select(&status_index, 0)?
            .add(&value_rows.matmul(&projection.as_tensor().t()?)?)?
            .reshape((batch, time, width))?)
    }

    fn copy_boost(
        &self,
        out: &Tensor,
        status: &[u32],
        value: &[u32],
        vocab: u32,
        previous: bool,
    ) -> Result<Tensor> {
        let rows = status.len();
        let device = out.device();
        let (weight, bias, scale) = if previous {
            (
                &self.copy_weight_previous,
                &self.copy_bias_previous,
                &self.copy_scale_previous,
            )
        } else {
            (&self.copy_weight, &self.copy_bias, &self.copy_scale)
        };
        let hit: Vec<f32> = status
            .iter()
            .map(|&s| if s == STATUS_HIT { 1.0 } else { 0.0 })
            .collect();
        let hit = Tensor::from_vec(hit, (rows, 1), device)?;
        let value_index = Tensor::from_vec(value.to_vec(), rows, device)?;
        let gate = candle_nn::ops::sigmoid(
            &out.matmul(weight.as_tensor())?
                .broadcast_add(bias.as_tensor())?,
        )?;
        let boost = gate.broadcast_mul(scale.as_tensor())?.broadcast_mul(&hit)?;
        let onehot = value_index
            .unsqueeze(1)?
            .broadcast_eq(&Tensor::arange(0u32, vocab, device)?.unsqueeze(0)?)?
            .to_dtype(DType::F32)?;
        Ok(onehot.broadcast_mul(&boost)?)
    }
}

/// The stack split after `split` layers, with tag and trigger heads there and,
/// in the memory arm, the store's register re-entering the residual stream.
pub struct AermModel {
    pub stack: StackModel,
    pub split: usize,
    tag_weight: Var,
    tag_bias: Var,
    trigger_weight: Var,
    trigger_bias: Var,
    memory: Option<MemoryBranch>,
}

/// The residual stream after the split and both heads' logits.
pub struct Bottom {
    pub hidden: Tensor,
    pub tags: Tensor,
    pub triggers: Tensor,
}

impl AermModel {
    pub fn new(
        config: StackConfig,
        split: usize,
        memory: bool,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        if split == 0 || split >= config.layers() {
            return Err(invalid("the split must leave layers on both sides"));
        }
        let width = config.width;
        let stack = StackModel::new(config, device)?;
        let mut rng = Rng::new(seed ^ 0x6165_726D_2D64_3221);
        let memory = if memory {
            Some(zero_memory_branch(width, device)?)
        } else {
            None
        };
        Ok(Self {
            tag_weight: normal_var(&mut rng, &[TAGS, width], 0.02, device)?,
            tag_bias: zeros_var(&[TAGS], device)?,
            trigger_weight: normal_var(&mut rng, &[TRIGGERS, width], 0.02, device)?,
            trigger_bias: zeros_var(&[TRIGGERS], device)?,
            stack,
            split,
            memory,
        })
    }

    /// Wraps a trained stack: the heads are initialised exactly as
    /// [`AermModel::new`] initialises them and the memory branch is
    /// zero-initialised, so the wrapped model reproduces the stack's logits
    /// before any training.
    pub fn from_stack(stack: StackModel, split: usize, memory: bool, seed: u64) -> Result<Self> {
        if split == 0 || split >= stack.config.layers() {
            return Err(invalid("the split must leave layers on both sides"));
        }
        let width = stack.config.width;
        let device = stack.device().clone();
        let mut rng = Rng::new(seed ^ 0x6165_726D_2D64_3221);
        let memory = if memory {
            Some(zero_memory_branch(width, &device)?)
        } else {
            None
        };
        Ok(Self {
            tag_weight: normal_var(&mut rng, &[TAGS, width], 0.02, &device)?,
            tag_bias: zeros_var(&[TAGS], &device)?,
            trigger_weight: normal_var(&mut rng, &[TRIGGERS, width], 0.02, &device)?,
            trigger_bias: zeros_var(&[TRIGGERS], &device)?,
            stack,
            split,
            memory,
        })
    }

    pub fn has_memory(&self) -> bool {
        self.memory.is_some()
    }

    /// Stack, head and memory-branch parameter counts.
    pub fn parameter_counts(&self) -> (usize, usize, usize) {
        let heads = self.tag_weight.elem_count()
            + self.tag_bias.elem_count()
            + self.trigger_weight.elem_count()
            + self.trigger_bias.elem_count();
        let memory = self.memory.as_ref().map_or(0, |m| {
            m.status.elem_count()
                + m.projection.elem_count()
                + m.copy_weight.elem_count()
                + m.copy_bias.elem_count()
                + m.copy_scale.elem_count()
                + m.status_previous.elem_count()
                + m.projection_previous.elem_count()
                + m.copy_weight_previous.elem_count()
                + m.copy_bias_previous.elem_count()
                + m.copy_scale_previous.elem_count()
        });
        (self.stack.parameter_count(), heads, memory)
    }

    /// Decayed matrices and other variables.
    pub fn optimizer_groups(&self) -> (Vec<Var>, Vec<Var>) {
        let (mut decayed, mut plain) = self.stack.optimizer_groups();
        decayed.push(self.tag_weight.clone());
        decayed.push(self.trigger_weight.clone());
        plain.push(self.tag_bias.clone());
        plain.push(self.trigger_bias.clone());
        if let Some(m) = &self.memory {
            decayed.push(m.status.clone());
            decayed.push(m.projection.clone());
            plain.push(m.copy_weight.clone());
            plain.push(m.copy_bias.clone());
            plain.push(m.copy_scale.clone());
            decayed.push(m.status_previous.clone());
            decayed.push(m.projection_previous.clone());
            plain.push(m.copy_weight_previous.clone());
            plain.push(m.copy_bias_previous.clone());
            plain.push(m.copy_scale_previous.clone());
        }
        (decayed, plain)
    }

    /// Layers below the split and both heads, which never see the store.
    pub fn bottom(&self, ids: &[u32], batch: usize, time: usize) -> Result<Bottom> {
        let x = self.stack.embed(ids, batch, time)?;
        let hidden = self.stack.run_layers(x, 0..self.split)?;
        let width = self.stack.config.width;
        let flat = hidden.reshape((batch * time, width))?;
        let rms = (flat.sqr()?.mean_keepdim(D::Minus1)? + 1e-5)?.sqrt()?;
        let normed = flat.broadcast_div(&rms)?;
        let tags = normed
            .matmul(&self.tag_weight.as_tensor().t()?)?
            .broadcast_add(self.tag_bias.as_tensor())?;
        let triggers = normed
            .matmul(&self.trigger_weight.as_tensor().t()?)?
            .broadcast_add(self.trigger_bias.as_tensor())?;
        Ok(Bottom {
            hidden,
            tags,
            triggers,
        })
    }

    /// Next-token logits `[batch * time, vocabulary]` from the bottom's residual
    /// stream and the per-position register pairs (ignored without a memory
    /// branch).
    pub fn top(
        &self,
        hidden: &Tensor,
        status: &[u32],
        value: &[u32],
        status_previous: &[u32],
        value_previous: &[u32],
    ) -> Result<Tensor> {
        let (batch, time, width) = hidden.dims3()?;
        let rows = batch * time;
        let layers = self.stack.config.layers();
        let Some(m) = &self.memory else {
            let x = self.stack.run_layers(hidden.clone(), self.split..layers)?;
            return self.stack.head(&self.stack.finish(x)?);
        };
        if status.len() != rows
            || value.len() != rows
            || status_previous.len() != rows
            || value_previous.len() != rows
        {
            return Err(invalid("one register per position"));
        }
        if status
            .iter()
            .chain(status_previous)
            .any(|&s| s as usize >= STATUSES)
            || value
                .iter()
                .chain(value_previous)
                .any(|&v| v as usize >= self.stack.config.vocab_size)
        {
            return Err(invalid("register outside the status set or vocabulary"));
        }
        let embedding = self
            .stack
            .variables()
            .get("embedding.weight")
            .ok_or_else(|| invalid("the stack has no embedding"))?
            .as_tensor();
        let side_current = m.registers_side(embedding, batch, time, width, status, value, false)?;
        let side_previous = m.registers_side(
            embedding,
            batch,
            time,
            width,
            status_previous,
            value_previous,
            true,
        )?;
        let x = self.stack.run_layers(
            hidden.add(&side_current.add(&side_previous)?)?,
            self.split..layers,
        )?;
        let out = self.stack.finish(x)?;
        let logits = self.stack.head(&out)?;
        let vocab = self.stack.config.vocab_size as u32;
        let boost_current = m.copy_boost(&out, status, value, vocab, false)?;
        let boost_previous = m.copy_boost(&out, status_previous, value_previous, vocab, true)?;
        Ok(logits.add(&boost_current)?.add(&boost_previous)?)
    }
}

/// Checkpoint configuration beside the weights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AermCheckpoint {
    pub split: usize,
    pub memory: bool,
}

impl AermModel {
    /// Saves the stack, both heads and, when present, the memory branch.
    pub fn save(&self, directory: &Path) -> Result<()> {
        fs::create_dir_all(directory)?;
        self.stack.save(&directory.join("stack"))?;
        let mut tensors: std::collections::HashMap<String, Tensor> =
            std::collections::HashMap::new();
        tensors.insert("tag_weight".into(), self.tag_weight.as_tensor().clone());
        tensors.insert("tag_bias".into(), self.tag_bias.as_tensor().clone());
        tensors.insert(
            "trigger_weight".into(),
            self.trigger_weight.as_tensor().clone(),
        );
        tensors.insert("trigger_bias".into(), self.trigger_bias.as_tensor().clone());
        if let Some(m) = &self.memory {
            for (name, var) in [
                ("memory.status", &m.status),
                ("memory.projection", &m.projection),
                ("memory.copy_weight", &m.copy_weight),
                ("memory.copy_bias", &m.copy_bias),
                ("memory.copy_scale", &m.copy_scale),
                ("memory.status_previous", &m.status_previous),
                ("memory.projection_previous", &m.projection_previous),
                ("memory.copy_weight_previous", &m.copy_weight_previous),
                ("memory.copy_bias_previous", &m.copy_bias_previous),
                ("memory.copy_scale_previous", &m.copy_scale_previous),
            ] {
                tensors.insert(name.into(), var.as_tensor().clone());
            }
        }
        candle_core::safetensors::save(&tensors, directory.join("heads.safetensors"))?;
        fs::write(
            directory.join("aerm.json"),
            serde_json::to_vec_pretty(&AermCheckpoint {
                split: self.split,
                memory: self.memory.is_some(),
            })?,
        )?;
        Ok(())
    }

    /// Loads a checkpoint written by [`AermModel::save`].
    pub fn load(directory: &Path, device: &Device) -> Result<Self> {
        let checkpoint: AermCheckpoint =
            serde_json::from_slice(&fs::read(directory.join("aerm.json"))?)?;
        let stack_directory = directory.join("stack");
        if StackModel::saved_served_representation(&stack_directory)?.is_some() {
            return Err(invalid(
                "the saved memory model carries a served representation the wrapper cannot reapply",
            ));
        }
        let mut stack = StackModel::load(&stack_directory, device)?;
        if let Some(snap) = StackModel::saved_transport_snap(&stack_directory)? {
            stack.set_transport_snap(Some(snap))?;
        }
        let width = stack.config.width;
        if checkpoint.split == 0 || checkpoint.split >= stack.config.layers() {
            return Err(invalid("saved split must leave layers on both sides"));
        }
        let tensors = candle_core::safetensors::load(directory.join("heads.safetensors"), device)?;
        let expected = 4 + if checkpoint.memory { 10 } else { 0 };
        if tensors.len() != expected {
            return Err(invalid("saved heads tensors differ from the configuration"));
        }
        let var = |name: &str, shape: &[usize]| -> Result<Var> {
            let tensor = tensors
                .get(name)
                .ok_or_else(|| invalid(format!("saved heads lack {name}")))?;
            if tensor.dims() != shape || tensor.dtype() != DType::F32 {
                return Err(invalid(format!("saved heads shape differs for {name}")));
            }
            Ok(Var::from_tensor(tensor)?)
        };
        let tag_weight = var("tag_weight", &[TAGS, width])?;
        let tag_bias = var("tag_bias", &[TAGS])?;
        let trigger_weight = var("trigger_weight", &[TRIGGERS, width])?;
        let trigger_bias = var("trigger_bias", &[TRIGGERS])?;
        let memory = if checkpoint.memory {
            Some(MemoryBranch {
                status: var("memory.status", &[STATUSES, width])?,
                projection: var("memory.projection", &[width, width])?,
                copy_weight: var("memory.copy_weight", &[width, 1])?,
                copy_bias: var("memory.copy_bias", &[1])?,
                copy_scale: var("memory.copy_scale", &[1])?,
                status_previous: var("memory.status_previous", &[STATUSES, width])?,
                projection_previous: var("memory.projection_previous", &[width, width])?,
                copy_weight_previous: var("memory.copy_weight_previous", &[width, 1])?,
                copy_bias_previous: var("memory.copy_bias_previous", &[1])?,
                copy_scale_previous: var("memory.copy_scale_previous", &[1])?,
            })
        } else {
            None
        };
        Ok(Self {
            stack,
            split: checkpoint.split,
            tag_weight,
            tag_bias,
            trigger_weight,
            trigger_bias,
            memory,
        })
    }
}

/// Argmax of each row of `[rows, classes]` logits.
fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

/// Registers from the model's own tags and write triggers, one sequence at a
/// time.
pub fn model_registers(
    ids: &[u32],
    tags: &[u32],
    triggers: &[u32],
    batch: usize,
    time: usize,
    eos: u32,
) -> Result<Registers> {
    model_registers_inner(ids, tags, triggers, batch, time, eos, None)
}

/// [`model_registers`] with the structural user-turn gate: positions whose
/// flag is 0 ignore their tags and triggers.
pub fn model_registers_gated(
    ids: &[u32],
    tags: &[u32],
    triggers: &[u32],
    batch: usize,
    time: usize,
    eos: u32,
    user_turn: &[u8],
) -> Result<Registers> {
    model_registers_inner(ids, tags, triggers, batch, time, eos, Some(user_turn))
}

fn model_registers_inner(
    ids: &[u32],
    tags: &[u32],
    triggers: &[u32],
    batch: usize,
    time: usize,
    eos: u32,
    user_turn: Option<&[u8]>,
) -> Result<Registers> {
    let mut out = Registers {
        status: Vec::with_capacity(batch * time),
        value: Vec::with_capacity(batch * time),
        status_previous: Vec::with_capacity(batch * time),
        value_previous: Vec::with_capacity(batch * time),
        read_events: 0,
    };
    for b in 0..batch {
        let range = b * time..(b + 1) * time;
        let simulation = match user_turn {
            Some(flags) => simulate_gated(
                &ids[range.clone()],
                &tags[range.clone()],
                &triggers[range.clone()],
                eos,
                &flags[range],
            )?,
            None => simulate(
                &ids[range.clone()],
                &tags[range.clone()],
                &triggers[range],
                eos,
            )?,
        };
        out.read_events += simulation.reads.len();
        out.status.extend(simulation.status);
        out.value.extend(simulation.value);
        out.status_previous.extend(simulation.status_previous);
        out.value_previous.extend(simulation.value_previous);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Training.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AermConfig {
    pub steps: usize,
    pub batch: usize,
    pub context: usize,
    pub learning_rate: f64,
    pub warmup: usize,
    pub weight_decay: f64,
    /// Global gradient-norm clip.
    pub clip: f64,
    pub tag_weight: f64,
    pub trigger_weight: f64,
    /// Loss weight of a trigger-positive position relative to a negative one.
    pub trigger_positive_weight: f32,
    pub data_seed: u64,
    /// Drop the tag auxiliary loss on ordinary-text batches.
    #[serde(default)]
    pub mask_text_tags: bool,
    /// Drop the trigger auxiliary loss on ordinary-text batches; independent of
    /// [`Self::mask_text_tags`]. With both set this reproduces the original
    /// single-flag behaviour.
    #[serde(default)]
    pub mask_text_triggers: bool,
    /// Gate reads and writes to user turns, located by the protocol's role
    /// structure. New runs default to it on.
    #[serde(default = "default_true")]
    pub user_turn_gate: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AermRecord {
    pub step: usize,
    pub text_loss: f64,
    pub dialogue_loss: f64,
    pub tag_loss: f64,
    pub trigger_loss: f64,
    pub gradient_norm: f64,
    pub seconds: f64,
}

fn schedule(step: usize, config: &AermConfig) -> f64 {
    if step < config.warmup {
        return (step + 1) as f64 / config.warmup as f64;
    }
    let span = config.steps.saturating_sub(config.warmup).max(1);
    let progress = (step - config.warmup) as f64 / span as f64;
    0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress.min(1.0)).cos())
}

/// `batch` random text windows of `context` inputs and their targets.
pub fn text_batch(
    text: &[u16],
    rng: &mut Rng,
    batch: usize,
    context: usize,
) -> Result<(Vec<u32>, Vec<u32>)> {
    if text.len() <= context + 1 {
        return Err(invalid("text is shorter than one window"));
    }
    let mut ids = Vec::with_capacity(batch * context);
    let mut targets = Vec::with_capacity(batch * context);
    for _ in 0..batch {
        let start = rng.below(text.len() - context - 1);
        ids.extend(text[start..start + context].iter().map(|&t| u32::from(t)));
        targets.extend(
            text[start + 1..start + context + 1]
                .iter()
                .map(|&t| u32::from(t)),
        );
    }
    Ok((ids, targets))
}

/// Scales every gradient so the global L2 norm is at most `max_norm`;
/// returns the norm before clipping.
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

/// Weighted auxiliary cross-entropy over the real positions.
fn head_loss(logits: &Tensor, targets: &[u32], weights: &[f32]) -> Result<Tensor> {
    logits_cross_entropy(logits, targets, Some(weights))
}

/// One text batch and one dialogue batch per update. The loss is text
/// cross-entropy, response cross-entropy on the dialogues, and the tag and
/// trigger cross-entropies on both batches (stories are all Other / None).
/// The memory arm's store is driven by the gold labels.
pub fn train_aerm(
    model: &AermModel,
    world: &RelationWorld,
    text: &[u16],
    config: &AermConfig,
) -> Result<(Vec<AermRecord>, f64)> {
    let started = Instant::now();
    let (decayed, plain) = model.optimizer_groups();
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
    let (batch, context) = (config.batch, config.context);
    let rows = batch * context;
    let text_tags = vec![TAG_OTHER; rows];
    let text_triggers = vec![TRIGGER_NONE; rows];
    let text_ones = vec![1.0f32; rows];
    let silent_status = vec![STATUS_NONE; rows];
    let silent_value = vec![0u32; rows];
    let mut records = Vec::new();
    for step in 0..config.steps {
        let lr = config.learning_rate * schedule(step, config);
        decayed_optimizer.set_learning_rate(lr);
        plain_optimizer.set_learning_rate(lr);
        // Text.
        let (ids, targets) = text_batch(text, &mut rng, batch, context)?;
        let bottom = model.bottom(&ids, batch, context)?;
        let logits = model.top(
            &bottom.hidden,
            &silent_status,
            &silent_value,
            &silent_status,
            &silent_value,
        )?;
        let text_loss = logits_cross_entropy(&logits, &targets, None)?;
        let text_tag = if config.mask_text_tags {
            None
        } else {
            Some(head_loss(&bottom.tags, &text_tags, &text_ones)?)
        };
        let text_trigger = if config.mask_text_triggers {
            None
        } else {
            Some(head_loss(&bottom.triggers, &text_triggers, &text_ones)?)
        };
        // Dialogues.
        let dialogues = world.batch(&mut rng, batch, context, false)?;
        let bottom = model.bottom(&dialogues.ids, batch, context)?;
        let logits = model.top(
            &bottom.hidden,
            &dialogues.status,
            &dialogues.value,
            &dialogues.status_previous,
            &dialogues.value_previous,
        )?;
        let dialogue_loss =
            logits_cross_entropy(&logits, &dialogues.targets, Some(&dialogues.response))?;
        let tag_loss = head_loss(&bottom.tags, &dialogues.tags, &dialogues.real)?;
        let trigger_weights: Vec<f32> = dialogues
            .triggers
            .iter()
            .zip(&dialogues.real)
            .map(|(&t, &r)| {
                if t == TRIGGER_NONE {
                    r
                } else {
                    r * config.trigger_positive_weight
                }
            })
            .collect();
        let trigger_loss = head_loss(&bottom.triggers, &dialogues.triggers, &trigger_weights)?;
        let aux_tag = match &text_tag {
            Some(text_term) => ((text_term + &tag_loss)? * (0.5 * config.tag_weight))?,
            None => (&tag_loss * (0.5 * config.tag_weight))?,
        };
        let aux_trigger = match &text_trigger {
            Some(text_term) => ((text_term + &trigger_loss)? * (0.5 * config.trigger_weight))?,
            None => (&trigger_loss * (0.5 * config.trigger_weight))?,
        };
        let loss = (((&text_loss + &dialogue_loss)? + aux_tag)? + aux_trigger)?;
        let mut grads = loss.backward()?;
        let norm = clip_gradients(&mut grads, &all, config.clip)?;
        decayed_optimizer.step(&grads)?;
        plain_optimizer.step(&grads)?;
        if step % 50 == 0 || step + 1 == config.steps {
            records.push(AermRecord {
                step,
                text_loss: f64::from(text_loss.to_scalar::<f32>()?),
                dialogue_loss: f64::from(dialogue_loss.to_scalar::<f32>()?),
                tag_loss: f64::from(tag_loss.to_scalar::<f32>()?),
                trigger_loss: f64::from(trigger_loss.to_scalar::<f32>()?),
                gradient_norm: norm,
                seconds: started.elapsed().as_secs_f64(),
            });
        }
    }
    Ok((records, started.elapsed().as_secs_f64()))
}

// ---------------------------------------------------------------------------
// Evaluation.

/// Where a memory-arm failure arose (synthesis §7.2's four-class trace).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TraceClass {
    /// The store lacked the gold record or held a wrong value for its key.
    Unavailable,
    /// The store held it, but the read used another key or never fired.
    NotSelected,
    /// The read used the gold key but returned another value (wrong mode).
    WrongValue,
    /// The register matched gold, yet the answer was wrong.
    Emission,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClassScore {
    pub queries: usize,
    /// Value classes: the value slot; abstaining classes: the whole answer.
    pub correct: usize,
    pub whole_answer_correct: usize,
    /// Memory arm: correct with gold registers (a perfect parser).
    pub oracle_correct: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DialogueEvaluation {
    pub episodes: usize,
    pub classes: BTreeMap<String, ClassScore>,
    /// Updated queries whose most recent same-relation write was another entity's.
    pub updated_recency_trap: ClassScore,
    pub tag_accuracy: f64,
    /// `[gold][predicted]` trigger counts over real positions.
    pub trigger_confusion: [[usize; TRIGGERS]; TRIGGERS],
    /// Model read events over the evaluated episodes (two per completed
    /// address).
    pub read_events: usize,
    pub trace: BTreeMap<String, usize>,
    /// Every memory-arm failure decomposed by [`AddressCause`].
    pub failure_cause: BTreeMap<String, usize>,
    /// The last query of the first episodes: gold and teacher-forced predictions.
    pub examples: Vec<QueryExample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryExample {
    pub class: QueryClass,
    pub gold_answer: Vec<u32>,
    pub predicted_answer: Vec<u32>,
    /// `(status, value, status_previous, value_previous)`.
    pub register: (u32, u32, u32, u32),
    pub gold_register: (u32, u32, u32, u32),
}

impl DialogueEvaluation {
    pub fn accuracy(&self, class: QueryClass) -> Option<f64> {
        self.classes
            .get(&format!("{class:?}"))
            .filter(|s| s.queries > 0)
            .map(|s| s.correct as f64 / s.queries as f64)
    }
}

/// Scores `episodes` fresh dialogues (seed `seed`, held-out templates and
/// names when `held`). The memory arm's store follows the model's own tags and
/// triggers; its failures are traced against the gold store.
pub fn evaluate_dialogues(
    model: &AermModel,
    world: &RelationWorld,
    episodes: usize,
    seed: u64,
    held: bool,
    context: usize,
    batch: usize,
    gate: bool,
) -> Result<DialogueEvaluation> {
    let mut rng = Rng::new(seed);
    let mut result = DialogueEvaluation {
        episodes,
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
                result.trigger_confusion[data.triggers[row] as usize][triggers[row] as usize] += 1;
            }
        }
        let registers = if gate {
            model_registers_gated(
                &data.ids,
                &tags,
                &triggers,
                size,
                context,
                world.eos,
                &data.user_turn,
            )?
        } else {
            model_registers(&data.ids, &tags, &triggers, size, context, world.eos)?
        };
        result.read_events += registers.read_events;
        let predicted = argmax_rows(&model.top(
            &bottom.hidden,
            &registers.status,
            &registers.value,
            &registers.status_previous,
            &registers.value_previous,
        )?)?;
        let oracle = if model.has_memory() {
            Some(argmax_rows(&model.top(
                &bottom.hidden,
                &data.status,
                &data.value,
                &data.status_previous,
                &data.value_previous,
            )?)?)
        } else {
            None
        };
        for (b, episode) in data.episodes.iter().enumerate() {
            let base = b * context;
            let last = episode.queries.len() - 1;
            for (index, query) in episode.queries.iter().enumerate() {
                // Target rows predicting the answer tokens.
                let rows = (query.answer_start - 1)..query.answer_end;
                let whole = rows
                    .clone()
                    .all(|r| predicted[base + r] == data.targets[base + r]);
                let slot_row = query.value_position.map(|p| p - 1);
                let correct = match (query.class.abstains(), slot_row) {
                    (false, Some(r)) => predicted[base + r] == data.targets[base + r],
                    _ => whole,
                };
                let oracle_correct =
                    oracle
                        .as_ref()
                        .is_some_and(|o| match (query.class.abstains(), slot_row) {
                            (false, Some(r)) => o[base + r] == data.targets[base + r],
                            _ => rows.clone().all(|r| o[base + r] == data.targets[base + r]),
                        });
                let key = format!("{:?}", query.class);
                let score = result.classes.entry(key).or_default();
                score.queries += 1;
                score.correct += usize::from(correct);
                score.whole_answer_correct += usize::from(whole);
                score.oracle_correct += usize::from(oracle_correct);
                if query.class == QueryClass::Updated && query.recency_trap {
                    let trap = &mut result.updated_recency_trap;
                    trap.queries += 1;
                    trap.correct += usize::from(correct);
                    trap.whole_answer_correct += usize::from(whole);
                    trap.oracle_correct += usize::from(oracle_correct);
                }
                let check = slot_row.unwrap_or(query.answer_start - 1);
                let register = (
                    registers.status[base + check],
                    registers.value[base + check],
                    registers.status_previous[base + check],
                    registers.value_previous[base + check],
                );
                let gold_register = (
                    data.status[base + check],
                    data.value[base + check],
                    data.status_previous[base + check],
                    data.value_previous[base + check],
                );
                if model.has_memory() && !correct {
                    let class = trace(
                        episode,
                        query,
                        &tags[base..base + context],
                        &triggers[base..base + context],
                        register,
                        gold_register,
                        world.eos,
                    )?;
                    *result.trace.entry(format!("{class:?}")).or_default() += 1;
                    let inputs = episode.tokens.len() - 1;
                    if inputs >= 2 {
                        let cause = address_cause(
                            &episode.tokens[..inputs],
                            &episode.tags[..inputs],
                            &episode.triggers[..inputs],
                            &tags[base..base + inputs],
                            &triggers[base..base + inputs],
                            query.answer_start - 1,
                            query.key,
                            world.eos,
                        )?;
                        *result
                            .failure_cause
                            .entry(format!("{cause:?}"))
                            .or_default() += 1;
                    }
                }
                if index == last && result.examples.len() < 32 {
                    result.examples.push(QueryExample {
                        class: query.class,
                        gold_answer: rows.clone().map(|r| data.targets[base + r]).collect(),
                        predicted_answer: rows.clone().map(|r| predicted[base + r]).collect(),
                        register,
                        gold_register,
                    });
                }
            }
        }
    }
    result.tag_accuracy = tags_right as f64 / tags_total.max(1) as f64;
    Ok(result)
}

/// Classifies one memory-arm failure against the gold store.
fn trace(
    episode: &Episode,
    query: &Query,
    tags: &[u32],
    triggers: &[u32],
    register: (u32, u32, u32, u32),
    gold_register: (u32, u32, u32, u32),
    eos: u32,
) -> Result<TraceClass> {
    if register == gold_register {
        return Ok(TraceClass::Emission);
    }
    // The model's store and the gold store just before the answer.
    let end = query.answer_start - 1;
    let tokens = &episode.tokens[..=end];
    let model = simulate(tokens, &tags[..=end], &triggers[..=end], eos)?;
    let gold = simulate(
        tokens,
        &episode.tags[..=end],
        &episode.triggers[..=end],
        eos,
    )?;
    let (entity, relation) = query.key;
    let held = model
        .store
        .record(entity, relation)
        .map(|r| (r.value, r.previous));
    let wanted = gold
        .store
        .record(entity, relation)
        .map(|r| (r.value, r.previous));
    if held != wanted {
        return Ok(TraceClass::Unavailable);
    }
    // The query's own turn starts after the previous EOS.
    let turn_start = episode.tokens[..query.answer_start]
        .iter()
        .rposition(|&t| t == eos)
        .map_or(1, |p| p + 1);
    let read = model
        .reads
        .iter()
        .rev()
        .find(|event| event.position >= turn_start);
    match read {
        Some(event) if event.key == Some(query.key) => Ok(TraceClass::WrongValue),
        _ => Ok(TraceClass::NotSelected),
    }
}

/// Where a memory-arm failure's address went wrong (the G-decomposition
/// diagnostic; it changes no score).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AddressCause {
    /// The query turn's entity token was not tagged as an entity.
    QueryEntityTag,
    /// The query turn's relation token was not tagged as a relation.
    QueryRelationTag,
    /// The last gold write's entity slot was not tagged as an entity.
    WriteEntityKey,
    /// The last gold write's relation slot was not tagged as a relation.
    WriteRelationKey,
    /// The last gold write's value slot was not tagged as a value.
    WriteValueTag,
    /// The model fired no write trigger where gold wrote the key.
    WriteMissed,
    /// The key is present but the model's latest write carries another value.
    WrongValueStored,
    /// The store reports the query key evicted.
    Evicted,
    /// None of the above.
    Other,
}

/// Classifies a failed query by comparing the model's own tags and triggers
/// with the gold stream, on the query turn and on the last gold write of the
/// query key. All five arrays must be the same length and `end` in range.
pub fn address_cause(
    tokens: &[u32],
    gold_tags: &[u32],
    gold_triggers: &[u32],
    model_tags: &[u32],
    model_triggers: &[u32],
    end: usize,
    key: (u32, u32),
    eos: u32,
) -> Result<AddressCause> {
    if tokens.len() != gold_tags.len()
        || tokens.len() != gold_triggers.len()
        || tokens.len() != model_tags.len()
        || tokens.len() != model_triggers.len()
        || end >= tokens.len()
    {
        return Err(invalid("aligned arrays and an in-range end are required"));
    }
    let gold = simulate(
        &tokens[..=end],
        &gold_tags[..=end],
        &gold_triggers[..=end],
        eos,
    )?;
    let model = simulate(
        &tokens[..=end],
        &model_tags[..=end],
        &model_triggers[..=end],
        eos,
    )?;
    let turn_start = tokens[..end]
        .iter()
        .rposition(|&t| t == eos)
        .map_or(1, |p| p + 1);
    let query_slot = |tag: u32| (turn_start..=end).rev().find(|&p| gold_tags[p] == tag);
    let Some(entity) = query_slot(TAG_ENTITY) else {
        return Ok(AddressCause::Other);
    };
    if model_tags[entity] != TAG_ENTITY {
        return Ok(AddressCause::QueryEntityTag);
    }
    let Some(relation) = query_slot(TAG_RELATION) else {
        return Ok(AddressCause::Other);
    };
    if model_tags[relation] != TAG_RELATION {
        return Ok(AddressCause::QueryRelationTag);
    }
    let Some(write) = gold.writes.iter().rev().find(|w| w.key == key) else {
        return Ok(AddressCause::Other);
    };
    if model.store.read(key.0, key.1, false).0 == STATUS_EVICTED {
        return Ok(AddressCause::Evicted);
    }
    let clause_slot = |tag: u32| (0..write.position).rev().find(|&p| gold_tags[p] == tag);
    if let Some(p) = clause_slot(TAG_ENTITY) {
        if model_tags[p] != TAG_ENTITY {
            return Ok(AddressCause::WriteEntityKey);
        }
    }
    if let Some(p) = clause_slot(TAG_RELATION) {
        if model_tags[p] != TAG_RELATION {
            return Ok(AddressCause::WriteRelationKey);
        }
    }
    if let Some(p) = clause_slot(TAG_VALUE) {
        if model_tags[p] != TAG_VALUE {
            return Ok(AddressCause::WriteValueTag);
        }
    }
    if model_triggers[write.position] != TRIGGER_WRITE {
        return Ok(AddressCause::WriteMissed);
    }
    let held = model.store.record(key.0, key.1).map(|r| r.value);
    let wanted = gold.store.record(key.0, key.1).map(|r| r.value);
    if held != wanted {
        return Ok(AddressCause::WrongValueStored);
    }
    Ok(AddressCause::Other)
}

/// Mean next-token NLL over `windows` evenly spaced development windows. The
/// memory arm's store follows the model's own triggers; also returns the
/// number of non-None triggers the model fired per thousand tokens.
pub fn text_nll(
    model: &AermModel,
    dev: &[u16],
    context: usize,
    windows: usize,
    batch: usize,
    eos: u32,
    gate: bool,
) -> Result<(f64, f64)> {
    let span = context + 1;
    if dev.len() < span || windows == 0 || batch == 0 {
        return Err(invalid("development text needs at least one window"));
    }
    let stride = ((dev.len() - span) / windows).max(1);
    let starts: Vec<usize> = (0..windows)
        .map(|w| w * stride)
        .filter(|s| s + span <= dev.len())
        .collect();
    let (mut total, mut count, mut fired) = (0.0f64, 0usize, 0usize);
    for chunk in starts.chunks(batch) {
        let mut ids = Vec::with_capacity(chunk.len() * context);
        let mut targets = Vec::with_capacity(chunk.len() * context);
        for &start in chunk {
            ids.extend(dev[start..start + context].iter().map(|&t| u32::from(t)));
            targets.extend(dev[start + 1..start + span].iter().map(|&t| u32::from(t)));
        }
        let bottom = model.bottom(&ids, chunk.len(), context)?;
        let tags = argmax_rows(&bottom.tags)?;
        let triggers = argmax_rows(&bottom.triggers)?;
        fired += triggers.iter().filter(|&&t| t != TRIGGER_NONE).count();
        // Prose carries no user turns, so with the gate on every position is
        // outside a turn and the registers stay silent.
        let registers = if gate {
            model_registers_gated(
                &ids,
                &tags,
                &triggers,
                chunk.len(),
                context,
                eos,
                &vec![0u8; ids.len()],
            )?
        } else {
            model_registers(&ids, &tags, &triggers, chunk.len(), context, eos)?
        };
        let logits = model
            .top(
                &bottom.hidden,
                &registers.status,
                &registers.value,
                &registers.status_previous,
                &registers.value_previous,
            )?
            .detach();
        let loss = logits_cross_entropy(&logits, &targets, None)?;
        total += f64::from(loss.to_scalar::<f32>()?) * ids.len() as f64;
        count += ids.len();
    }
    Ok((
        total / count.max(1) as f64,
        fired as f64 * 1000.0 / count.max(1) as f64,
    ))
}

/// Greedy free-running answers to the last query of `episodes` fresh episodes,
/// recomputing the window (and the store) at every step, as generation does.
/// Returns `(gold answer, generated answer)` token lists.
pub fn free_running(
    model: &AermModel,
    world: &RelationWorld,
    episodes: usize,
    seed: u64,
    held: bool,
    context: usize,
    max_new: usize,
    gate: bool,
) -> Result<Vec<(Vec<u32>, Vec<u32>)>> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(episodes);
    for _ in 0..episodes {
        let episode = world.episode(&mut rng, held, context + 1)?;
        let query = episode
            .queries
            .last()
            .ok_or_else(|| invalid("an episode without a query"))?;
        let gold = episode.tokens[query.answer_start..=query.answer_end].to_vec();
        let mut ids = episode.tokens[..query.answer_start].to_vec();
        let mut generated = Vec::new();
        for _ in 0..max_new {
            if ids.len() > context {
                break;
            }
            let time = ids.len();
            let bottom = model.bottom(&ids, 1, time)?;
            let tags = argmax_rows(&bottom.tags)?;
            let triggers = argmax_rows(&bottom.triggers)?;
            let registers = if gate {
                let mut flags = vec![0u8; ids.len()];
                let known = ids.len().min(episode.user_turn.len());
                flags[..known].copy_from_slice(&episode.user_turn[..known]);
                model_registers_gated(&ids, &tags, &triggers, 1, time, world.eos, &flags)?
            } else {
                model_registers(&ids, &tags, &triggers, 1, time, world.eos)?
            };
            let logits = model.top(
                &bottom.hidden,
                &registers.status,
                &registers.value,
                &registers.status_previous,
                &registers.value_previous,
            )?;
            let next = argmax_rows(&logits.narrow(0, time - 1, 1)?)?[0];
            generated.push(next);
            ids.push(next);
            if next == world.eos {
                break;
            }
        }
        out.push((gold, generated));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_stack::{ReadScore, StackArch, TransportSnap};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// A toy encoder: pretokens (a word with its leading space, an
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
    fn the_store_overwrites_by_version_and_keeps_the_previous_value() {
        let mut store = RelationStore::new();
        assert_eq!(store.read(5, 7, false), (STATUS_ABSENT, 0));
        store.write(5, 7, 100);
        assert_eq!(store.read(5, 7, false), (STATUS_HIT, 100));
        assert_eq!(store.read(5, 7, true), (STATUS_ABSENT, 0));
        store.write(5, 7, 101);
        assert_eq!(store.read(5, 7, false), (STATUS_HIT, 101));
        assert_eq!(store.read(5, 7, true), (STATUS_HIT, 100));
        // A same-value reassertion keeps the previous distinct value.
        store.write(5, 7, 101);
        assert_eq!(store.read(5, 7, true), (STATUS_HIT, 100));
        assert_eq!(store.record(5, 7).map(|r| r.version), Some(3));
        // Other keys are independent.
        store.write(6, 7, 200);
        assert_eq!(store.read(5, 7, false), (STATUS_HIT, 101));
        assert_eq!(store.read(6, 7, false), (STATUS_HIT, 200));
    }

    #[test]
    fn eviction_is_reported_not_aliased() {
        let mut store = RelationStore::new();
        for key in 0..(4 * STORE_SLOTS) as u32 {
            store.write(key, 9, key + 1000);
        }
        let mut evicted = 0;
        for key in 0..(4 * STORE_SLOTS) as u32 {
            match store.read(key, 9, false) {
                (STATUS_HIT, value) => assert_eq!(value, key + 1000),
                (STATUS_EVICTED, 0) => evicted += 1,
                other => panic!("unexpected read {other:?}"),
            }
        }
        assert!(evicted >= 4 * STORE_SLOTS - STORE_SLOTS - STORE_STASH);
    }

    #[test]
    fn placement_spreads_keys_and_rarely_repeats_a_choice() {
        let mut load = vec![0usize; STORE_SLOTS];
        let (mut keys, mut same) = (0usize, 0usize);
        for entity in (3..4096).step_by(7) {
            for relation in [448u32, 931, 1412, 1541, 1760, 3915] {
                let [a, b] = RelationStore::candidates(entity, relation);
                load[a] += 1;
                keys += 1;
                same += usize::from(a == b);
            }
        }
        let mean = keys / STORE_SLOTS;
        assert!(
            load.iter().all(|&l| l <= 2 * mean && l * 2 >= mean),
            "{load:?}"
        );
        assert!(same * 16 < keys, "{same} of {keys} keys repeat a choice");
    }

    #[test]
    fn episodes_label_their_slots_and_answer_from_the_gold_store() -> Result<()> {
        let encode = toy_encoder();
        let world = RelationWorld::new(&encode, 0, 1)?;
        let mut rng = Rng::new(11);
        let mut classes = BTreeMap::new();
        for index in 0..300 {
            let held = index % 3 == 0;
            let episode = world.episode(&mut rng, held, 257)?;
            assert!(episode.tokens.len() <= 257);
            // The message texts re-encode, segment by segment, to the tokens.
            let mut rebuilt = vec![0u32];
            for (turn, (user, reply)) in episode.messages.iter().enumerate() {
                if turn > 0 {
                    rebuilt.extend(encode("\n"));
                }
                rebuilt.extend(encode("User: "));
                rebuilt.extend(encode(user));
                rebuilt.extend(encode("\n"));
                rebuilt.extend(encode("Assistant: "));
                rebuilt.extend(encode(reply));
                rebuilt.push(1);
            }
            assert_eq!(rebuilt, episode.tokens);
            assert_eq!(episode.tokens.len(), episode.tags.len());
            assert_eq!(episode.tokens.len(), episode.triggers.len());
            // Tagged tokens are the slot words; writes close a tagged clause.
            let gold = simulate(&episode.tokens, &episode.tags, &episode.triggers, 1)?;
            for query in &episode.queries {
                *classes.entry(query.class).or_insert(0) += 1;
                let check = query.answer_start - 1;
                match query.value {
                    Some(value) => {
                        // Previous-value queries are answered by the previous
                        // register; every other class by the current register.
                        let (status, register_value) = if query.class == QueryClass::Previous {
                            (gold.status_previous[check], gold.value_previous[check])
                        } else {
                            (gold.status[check], gold.value[check])
                        };
                        assert_eq!((status, register_value), (STATUS_HIT, value));
                        let slot = query.value_position.expect("value slot");
                        assert_eq!(episode.tokens[slot], value);
                        assert_eq!(episode.response_mask[slot], 1);
                    }
                    None if query.class == QueryClass::PreviousAbsent => {
                        assert_eq!(gold.status[check], STATUS_HIT);
                        assert_ne!(gold.status_previous[check], STATUS_HIT);
                    }
                    None => {
                        assert_ne!(gold.status[check], STATUS_HIT);
                        assert_ne!(gold.status_previous[check], STATUS_HIT);
                    }
                }
                assert_eq!(episode.tokens[query.answer_end], 1);
            }
        }
        for class in [
            QueryClass::First,
            QueryClass::Updated,
            QueryClass::Reasserted,
            QueryClass::Previous,
            QueryClass::PreviousAbsent,
            QueryClass::Absent,
        ] {
            assert!(
                classes.get(&class).copied().unwrap_or(0) > 5,
                "{class:?} is rare: {classes:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn an_address_tag_reads_without_any_read_trigger() -> Result<()> {
        // Turn 1 writes (entity 10, relation 20) = 30. Turn 2 queries the same
        // key. No token carries a read trigger in either turn, and the write
        // still sees a complete clause: the read neither needs a trigger nor
        // closes the clause.
        let eos = 1u32;
        let tokens = vec![0u32, 10, 20, 30, 40, eos, 10, 20, 41, eos];
        let tags = vec![
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let triggers = vec![
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
        ];
        let simulation = simulate(&tokens, &tags, &triggers, eos)?;
        assert_eq!(
            simulation.store.record(10, 20).map(|r| r.value),
            Some(30),
            "the write after the read still saw the open clause"
        );
        // The read fired at the relation tag of turn 2 with no trigger.
        assert_eq!(simulation.reads.len(), 4);
        let read = simulation
            .reads
            .iter()
            .find(|r| r.position == 7 && !r.previous)
            .expect("current read at the held-out address tag");
        assert_eq!(read.key, Some((10, 20)));
        assert_eq!((read.status, read.value), (STATUS_HIT, 30));
        // The register persists to the end of the query turn.
        assert_eq!(
            (simulation.status[9], simulation.value[9]),
            (STATUS_HIT, 30)
        );
        Ok(())
    }

    #[test]
    fn different_trailing_phrases_do_not_change_the_read() -> Result<()> {
        // The D2 failure: a trigger learned at the end of the training query
        // template never fired on a held-out ending. The address tags are the
        // same in both endings, so the always-on read must be identical.
        let eos = 1u32;
        let prefix = vec![0u32, 10, 20, 30, 40, eos];
        let prefix_tags = vec![
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let prefix_triggers = vec![
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
        ];
        let run = |query: &[u32], query_tags: &[u32]| -> Result<Simulation> {
            let mut tokens = prefix.clone();
            tokens.extend_from_slice(query);
            let mut tags = prefix_tags.clone();
            tags.extend_from_slice(query_tags);
            let mut triggers = prefix_triggers.clone();
            triggers.extend(std::iter::repeat_n(TRIGGER_NONE, query.len()));
            simulate(&tokens, &tags, &triggers, eos)
        };
        let train = run(
            &[10, 20, 41, eos],
            &[TAG_ENTITY, TAG_RELATION, TAG_OTHER, TAG_OTHER],
        )?;
        let held = run(
            &[20, 50, 10, 60, 61, 42, eos],
            &[
                TAG_RELATION,
                TAG_OTHER,
                TAG_ENTITY,
                TAG_OTHER,
                TAG_OTHER,
                TAG_OTHER,
                TAG_OTHER,
            ],
        )?;
        let last = |s: &Simulation| (s.status[s.status.len() - 1], s.value[s.status.len() - 1]);
        assert_eq!(last(&train), (STATUS_HIT, 30));
        assert_eq!(last(&held), (STATUS_HIT, 30));
        assert!(held.reads.iter().any(|r| r.key == Some((10, 20))));
        Ok(())
    }

    #[test]
    fn the_previous_register_follows_updates_and_reassertions() -> Result<()> {
        let eos = 1u32;
        let turn = |value: u32| -> (Vec<u32>, Vec<u32>, Vec<u32>) {
            (
                vec![10, 20, value, 40],
                vec![TAG_ENTITY, TAG_RELATION, TAG_VALUE, TAG_OTHER],
                vec![TRIGGER_NONE, TRIGGER_NONE, TRIGGER_NONE, TRIGGER_WRITE],
            )
        };
        let mut tokens = vec![0u32];
        let mut tags = vec![TAG_OTHER];
        let mut triggers = vec![TRIGGER_NONE];
        for value in [30, 31, 31] {
            let (t, g, r) = turn(value);
            tokens.extend(t);
            tags.extend(g);
            triggers.extend(r);
            tokens.push(eos);
            tags.push(TAG_OTHER);
            triggers.push(TRIGGER_NONE);
        }
        tokens.extend([10, 20, eos]);
        tags.extend([TAG_ENTITY, TAG_RELATION, TAG_OTHER]);
        triggers.extend([TRIGGER_NONE, TRIGGER_NONE, TRIGGER_NONE]);
        let simulation = simulate(&tokens, &tags, &triggers, eos)?;
        let last = simulation.status.len() - 1;
        assert_eq!(
            (simulation.status[last], simulation.value[last]),
            (STATUS_HIT, 31)
        );
        assert_eq!(
            (
                simulation.status_previous[last],
                simulation.value_previous[last]
            ),
            (STATUS_HIT, 30),
            "the same-value reassertion kept the previous distinct value"
        );
        Ok(())
    }

    #[test]
    fn untagged_text_is_register_silent() -> Result<()> {
        let tokens: Vec<u32> = (0..64).collect();
        let tags = vec![TAG_OTHER; tokens.len()];
        let triggers = vec![TRIGGER_NONE; tokens.len()];
        let simulation = simulate(&tokens, &tags, &triggers, 1)?;
        assert!(simulation.reads.is_empty());
        assert!(simulation.status.iter().all(|&s| s == STATUS_NONE));
        assert!(simulation.status_previous.iter().all(|&s| s == STATUS_NONE));
        Ok(())
    }

    #[test]
    fn write_events_record_the_exact_clause() -> Result<()> {
        let tokens = vec![0u32, 10, 20, 30, 40, 1, 10, 20, 41, 1];
        let tags = vec![
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let triggers = vec![
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
        ];
        let simulation = simulate(&tokens, &tags, &triggers, 1)?;
        assert_eq!(
            simulation.writes,
            vec![WriteEvent {
                position: 4,
                key: (10, 20),
                value: 30,
            }]
        );
        Ok(())
    }

    #[test]
    fn address_cause_names_the_query_and_write_slots() -> Result<()> {
        let tokens = vec![0u32, 10, 20, 30, 40, 1, 10, 20, 41, 1];
        let tags = vec![
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let triggers = vec![
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
        ];
        let cause = |model_tags: &[u32], model_triggers: &[u32]| -> Result<AddressCause> {
            address_cause(
                &tokens,
                &tags,
                &triggers,
                model_tags,
                model_triggers,
                9,
                (10, 20),
                1,
            )
        };
        assert_eq!(cause(&tags, &triggers)?, AddressCause::Other);
        let flip = |at: usize, value: u32| -> (Vec<u32>, Vec<u32>) {
            let mut model_tags = tags.clone();
            model_tags[at] = value;
            (model_tags, triggers.clone())
        };
        assert_eq!(
            cause(&flip(6, TAG_OTHER).0, &triggers)?,
            AddressCause::QueryEntityTag
        );
        assert_eq!(
            cause(&flip(7, TAG_OTHER).0, &triggers)?,
            AddressCause::QueryRelationTag
        );
        assert_eq!(
            cause(&flip(1, TAG_OTHER).0, &triggers)?,
            AddressCause::WriteEntityKey
        );
        assert_eq!(
            cause(&flip(2, TAG_OTHER).0, &triggers)?,
            AddressCause::WriteRelationKey
        );
        assert_eq!(
            cause(&flip(3, TAG_OTHER).0, &triggers)?,
            AddressCause::WriteValueTag
        );
        let mut missed = triggers.clone();
        missed[4] = TRIGGER_NONE;
        assert_eq!(cause(&tags, &missed)?, AddressCause::WriteMissed);
        Ok(())
    }

    #[test]
    fn gated_prose_registers_are_silent() -> Result<()> {
        let tokens = vec![0u32, 10, 20, 30, 40, 1];
        let tags = vec![
            TAG_OTHER,
            TAG_ENTITY,
            TAG_RELATION,
            TAG_VALUE,
            TAG_OTHER,
            TAG_OTHER,
        ];
        let triggers = vec![
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_NONE,
            TRIGGER_WRITE,
            TRIGGER_NONE,
        ];
        let flags = vec![0u8; tokens.len()];
        let gated = simulate_gated(&tokens, &tags, &triggers, 1, &flags)?;
        assert!(gated.reads.is_empty());
        assert!(gated.writes.is_empty());
        assert!(gated.status.iter().all(|&s| s == STATUS_NONE));
        assert!(gated.status_previous.iter().all(|&s| s == STATUS_NONE));
        assert!(gated.store.record(10, 20).is_none());
        let plain = simulate(&tokens, &tags, &triggers, 1)?;
        assert!(!plain.reads.is_empty());
        assert_eq!(plain.writes.len(), 1);
        Ok(())
    }

    #[test]
    fn the_worlds_user_turn_operations_are_unchanged() -> Result<()> {
        let world = world()?;
        let mut rng = Rng::new(77);
        for index in 0..64 {
            let episode = world.episode(&mut rng, index % 2 == 0, 257)?;
            let plain = simulate(&episode.tokens, &episode.tags, &episode.triggers, 1)?;
            let gated = simulate_gated(
                &episode.tokens,
                &episode.tags,
                &episode.triggers,
                1,
                &episode.user_turn,
            )?;
            assert_eq!(plain.status, gated.status);
            assert_eq!(plain.value, gated.value);
            assert_eq!(plain.reads, gated.reads);
            assert_eq!(plain.writes, gated.writes);
        }
        Ok(())
    }

    #[test]
    fn every_tagged_or_triggered_position_is_inside_a_user_turn() -> Result<()> {
        let world = world()?;
        let mut rng = Rng::new(91);
        for index in 0..64 {
            let episode = world.episode(&mut rng, index % 2 == 0, 257)?;
            assert_eq!(episode.tokens.len(), episode.user_turn.len());
            for (position, ((&tag, &trigger), &flag)) in episode
                .tags
                .iter()
                .zip(&episode.triggers)
                .zip(&episode.user_turn)
                .enumerate()
            {
                if tag != TAG_OTHER || trigger != TRIGGER_NONE {
                    assert_eq!(flag, 1, "position {position} operates outside a user turn");
                }
            }
        }
        Ok(())
    }

    #[test]
    fn many_episodes_never_evict_their_facts() -> Result<()> {
        // D2's first attempt stopped after about 3,000 training episodes when
        // two-choice placement evicted one of an episode's few facts.
        let world = world()?;
        let mut rng = Rng::new(20_260_928);
        for index in 0..30_000 {
            let episode = world.episode(&mut rng, index % 4 == 0, 257)?;
            let gold = simulate(&episode.tokens, &episode.tags, &episode.triggers, 1)?;
            assert!(gold.reads.iter().all(|read| read.status != STATUS_EVICTED));
        }
        Ok(())
    }

    #[test]
    fn the_memory_branch_starts_silent_and_the_split_matches_the_stack() -> Result<()> {
        let device = Device::Cpu;
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 400,
            width: 32,
            heads: 2,
            mlp_hidden: 48,
            context: 16,
            pattern: "rrar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 3,
            memory: None,
        };
        let model = AermModel::new(config, 2, true, 3, &device)?;
        let ids: Vec<u32> = (0..32).map(|i| (i * 7 % 400) as u32).collect();
        let reference = model.stack.forward(&ids, 2, 16)?.to_vec2::<f32>()?;
        let bottom = model.bottom(&ids, 2, 16)?;
        let status: Vec<u32> = (0..32).map(|i| (i % 4) as u32).collect();
        let value: Vec<u32> = (0..32).map(|i| (i * 13 % 400) as u32).collect();
        let split = model
            .top(&bottom.hidden, &status, &value, &status, &value)?
            .to_vec2::<f32>()?;
        for (a, b) in reference.iter().flatten().zip(split.iter().flatten()) {
            assert!((a - b).abs() < 1e-5, "{a} vs {b}");
        }
        Ok(())
    }

    #[test]
    fn from_stack_reproduces_the_stack_before_training() -> Result<()> {
        let device = Device::Cpu;
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 400,
            width: 32,
            heads: 2,
            mlp_hidden: 48,
            context: 16,
            pattern: "rrar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 17,
            memory: None,
        };
        let stack = StackModel::new(config.clone(), &device)?;
        let ids: Vec<u32> = (0..32).map(|i| (i * 11 % 400) as u32).collect();
        let reference = stack.forward(&ids, 2, 16)?.to_vec2::<f32>()?;
        // The same seed must give the same heads as `new`, and the branch must
        // start at zero; the logit comparison below cannot see either.
        let from_new = AermModel::new(config, 2, true, 17, &device)?;
        let model = AermModel::from_stack(stack, 2, true, 17)?;
        let flat = |var: &Var| -> Result<Vec<f32>> {
            Ok(var.as_tensor().flatten_all()?.to_vec1::<f32>()?)
        };
        for (name, a, b) in [
            ("tag_weight", &from_new.tag_weight, &model.tag_weight),
            ("tag_bias", &from_new.tag_bias, &model.tag_bias),
            (
                "trigger_weight",
                &from_new.trigger_weight,
                &model.trigger_weight,
            ),
            ("trigger_bias", &from_new.trigger_bias, &model.trigger_bias),
        ] {
            assert_eq!(flat(a)?, flat(b)?, "{name} differs from new()");
        }
        let new_branch = from_new.memory.as_ref().expect("memory branch");
        let branch = model.memory.as_ref().expect("memory branch");
        for (name, a, b) in [
            ("status", &new_branch.status, &branch.status),
            ("projection", &new_branch.projection, &branch.projection),
            ("copy_weight", &new_branch.copy_weight, &branch.copy_weight),
            ("copy_bias", &new_branch.copy_bias, &branch.copy_bias),
            ("copy_scale", &new_branch.copy_scale, &branch.copy_scale),
            (
                "status_previous",
                &new_branch.status_previous,
                &branch.status_previous,
            ),
            (
                "projection_previous",
                &new_branch.projection_previous,
                &branch.projection_previous,
            ),
            (
                "copy_weight_previous",
                &new_branch.copy_weight_previous,
                &branch.copy_weight_previous,
            ),
            (
                "copy_bias_previous",
                &new_branch.copy_bias_previous,
                &branch.copy_bias_previous,
            ),
            (
                "copy_scale_previous",
                &new_branch.copy_scale_previous,
                &branch.copy_scale_previous,
            ),
        ] {
            assert_eq!(flat(a)?, flat(b)?, "{name} differs from new()");
        }
        let bottom = model.bottom(&ids, 2, 16)?;
        let silent = vec![STATUS_NONE; 32];
        let values = vec![0u32; 32];
        let wrapped = model
            .top(&bottom.hidden, &silent, &values, &silent, &values)?
            .to_vec2::<f32>()?;
        let delta = reference
            .iter()
            .flatten()
            .zip(wrapped.iter().flatten())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            delta <= 1e-5,
            "wrapped model differs from the stack by {delta}"
        );
        Ok(())
    }

    #[test]
    fn a_training_step_reaches_every_memory_parameter() -> Result<()> {
        let device = Device::Cpu;
        let world = world()?;
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 400,
            width: 32,
            heads: 2,
            mlp_hidden: 48,
            context: 256,
            pattern: "rrar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 5,
            memory: None,
        };
        let model = AermModel::new(config, 2, true, 5, &device)?;
        let mut rng = Rng::new(3);
        let data = world.batch(&mut rng, 2, 256, false)?;
        let bottom = model.bottom(&data.ids, 2, 256)?;
        let logits = model.top(
            &bottom.hidden,
            &data.status,
            &data.value,
            &data.status_previous,
            &data.value_previous,
        )?;
        let loss = logits_cross_entropy(&logits, &data.targets, Some(&data.response))?;
        let grads = loss.backward()?;
        let m = model.memory.as_ref().expect("memory branch");
        for (name, var) in [
            ("status", &m.status),
            ("projection", &m.projection),
            ("scale", &m.copy_scale),
            ("status_previous", &m.status_previous),
            ("projection_previous", &m.projection_previous),
            ("scale_previous", &m.copy_scale_previous),
        ] {
            let grad = grads.get(var.as_tensor()).expect("gradient");
            let norm = grad.sqr()?.sum_all()?.to_scalar::<f32>()?;
            assert!(norm > 0.0, "{name} receives no gradient");
        }
        Ok(())
    }

    #[test]
    fn load_restores_the_saved_transport_snap() -> Result<()> {
        let device = Device::Cpu;
        let directory = std::env::temp_dir().join(format!("aerm-snap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 400,
            width: 32,
            heads: 2,
            mlp_hidden: 48,
            context: 16,
            pattern: "rrar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 19,
            memory: None,
        };
        let mut stack = StackModel::new(config.clone(), &device)?;
        stack.set_transport_snap(Some(TransportSnap::Icosian))?;
        let model = AermModel::from_stack(stack, 2, true, 19)?;
        model.save(&directory)?;
        let loaded = AermModel::load(&directory, &device)?;
        assert_eq!(loaded.stack.transport_snap(), Some(TransportSnap::Icosian));
        let ids: Vec<u32> = (0..32).map(|i| (i * 13 % 400) as u32).collect();
        let silent = vec![STATUS_NONE; 32];
        let values = vec![0u32; 32];
        let before = model
            .top(
                &model.bottom(&ids, 2, 16)?.hidden,
                &silent,
                &values,
                &silent,
                &values,
            )?
            .to_vec2::<f32>()?;
        let after = loaded
            .top(
                &loaded.bottom(&ids, 2, 16)?.hidden,
                &silent,
                &values,
                &silent,
                &values,
            )?
            .to_vec2::<f32>()?;
        for (a, b) in before.iter().flatten().zip(after.iter().flatten()) {
            assert_eq!(a, b, "snapped logits differ after reload");
        }
        // A full memory checkpoint whose *stack* carries a served
        // representation: the wrapper cannot reapply it, so load must refuse.
        let served_directory = directory.join("served");
        let mut served = StackModel::new(config, &device)?;
        served.set_served_representation(Some(std::sync::Arc::new(
            crate::geometric_stack::D11Interim,
        )))?;
        let served_model = AermModel::from_stack(served, 2, true, 19)?;
        served_model.save(&served_directory)?;
        match AermModel::load(&served_directory, &device) {
            Ok(_) => panic!("a saved served representation must be refused"),
            Err(error) => assert!(
                format!("{error}").contains("served representation"),
                "unexpected error: {error}"
            ),
        }
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn aerm_save_and_load_reproduce_logits() -> Result<()> {
        let device = Device::Cpu;
        let directory = std::env::temp_dir().join(format!("aerm-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 400,
            width: 32,
            heads: 2,
            mlp_hidden: 48,
            context: 16,
            pattern: "rrar".into(),
            read: ReadScore::Dot,
            rotation: true,
            seed: 11,
            memory: None,
        };
        let mut model = AermModel::new(config.clone(), 2, true, 11, &device)?;
        // Non-zero, deterministic head and memory values: an all-zero branch
        // would leave a parameter dropped from both save and load invisible.
        let fill = |shape: &[usize], seed: usize| -> Result<Var> {
            let count: usize = shape.iter().product();
            let values: Vec<f32> = (0..count)
                .map(|i| (((i * 7 + seed * 3) % 13) as f32 - 6.0) / 9.0)
                .collect();
            Ok(Var::from_vec(values, shape, &device)?)
        };
        model.tag_weight = fill(&[TAGS, 32], 1)?;
        model.tag_bias = fill(&[TAGS], 2)?;
        model.trigger_weight = fill(&[TRIGGERS, 32], 3)?;
        model.trigger_bias = fill(&[TRIGGERS], 4)?;
        let branch = model.memory.as_mut().expect("memory branch");
        branch.status = fill(&[STATUSES, 32], 5)?;
        branch.projection = fill(&[32, 32], 6)?;
        branch.copy_weight = fill(&[32, 1], 7)?;
        branch.copy_bias = fill(&[1], 8)?;
        branch.copy_scale = fill(&[1], 9)?;
        branch.status_previous = fill(&[STATUSES, 32], 10)?;
        branch.projection_previous = fill(&[32, 32], 11)?;
        branch.copy_weight_previous = fill(&[32, 1], 12)?;
        branch.copy_bias_previous = fill(&[1], 13)?;
        branch.copy_scale_previous = fill(&[1], 14)?;
        model.save(&directory)?;
        let loaded = AermModel::load(&directory, &device)?;
        let ids: Vec<u32> = (0..32).map(|i| (i * 5 % 400) as u32).collect();
        let status: Vec<u32> = (0..32).map(|i| (i % 4) as u32).collect();
        let value: Vec<u32> = (0..32).map(|i| (i * 7 % 400) as u32).collect();
        let tags_a = model.bottom(&ids, 2, 16)?.tags.to_vec2::<f32>()?;
        let tags_b = loaded.bottom(&ids, 2, 16)?.tags.to_vec2::<f32>()?;
        for (x, y) in tags_a.iter().flatten().zip(tags_b.iter().flatten()) {
            assert_eq!(x, y, "loaded tag logits differ");
        }
        let a = model
            .top(
                &model.bottom(&ids, 2, 16)?.hidden,
                &status,
                &value,
                &status,
                &value,
            )?
            .to_vec2::<f32>()?;
        let b = loaded
            .top(
                &loaded.bottom(&ids, 2, 16)?.hidden,
                &status,
                &value,
                &status,
                &value,
            )?
            .to_vec2::<f32>()?;
        for (x, y) in a.iter().flatten().zip(b.iter().flatten()) {
            assert_eq!(x, y, "loaded memory-arm logits differ");
        }
        let control = AermModel::new(config, 2, false, 11, &device)?;
        control.save(&directory.join("control"))?;
        let loaded_control = AermModel::load(&directory.join("control"), &device)?;
        let c = control.stack.forward(&ids, 2, 16)?.to_vec2::<f32>()?;
        let d = loaded_control
            .stack
            .forward(&ids, 2, 16)?
            .to_vec2::<f32>()?;
        for (x, y) in c.iter().flatten().zip(d.iter().flatten()) {
            assert_eq!(x, y, "loaded control logits differ");
        }
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    fn natural_world() -> Result<RelationWorld> {
        RelationWorld::natural(&toy_encoder(), 0, 1)
    }

    fn normalize(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim_end_matches(['.', '?'])
            .trim_end()
            .to_owned()
    }

    /// The 30 user turns of development panel `dev-mem-01..10` (three turns
    /// each), frozen before any natural training run.
    const PANEL_SENTENCES: [&str; 30] = [
        "My name is Alex.",
        "I also like apples.",
        "What is my name?",
        "My cat is named Momo.",
        "She likes to sleep.",
        "What is my cat's name?",
        "My favorite color is green.",
        "I also like blue.",
        "What is my favorite color?",
        "I work as a teacher.",
        "My school is nearby.",
        "What is my job?",
        "My sister lives in Tokyo.",
        "She visits often.",
        "Where does my sister live?",
        "I have two brothers.",
        "They are older than me.",
        "How many brothers do I have?",
        "My birthday is in July.",
        "I love summer.",
        "When is my birthday?",
        "I drive a blue car.",
        "It is fast.",
        "What color is my car?",
        "I am learning to play the piano.",
        "It is hard but fun.",
        "What instrument am I learning?",
        "My favorite food is pizza.",
        "I eat it on Fridays.",
        "What is my favorite food?",
    ];

    #[test]
    fn natural_training_never_contains_a_panel_sentence() -> Result<()> {
        let world = natural_world()?;
        let panels: Vec<String> = PANEL_SENTENCES.iter().map(|s| normalize(s)).collect();
        let mut rng = Rng::new(0x0D2_2026_09_29);
        for _ in 0..2_000 {
            let episode = world.episode(&mut rng, false, 257)?;
            for (user, reply) in &episode.messages {
                for text in [user, reply] {
                    let message = normalize(text);
                    for panel in &panels {
                        assert!(
                            !message.contains(panel),
                            "training message {message:?} contains panel sentence {panel:?}"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn natural_tagged_slots_carry_canonical_atoms() -> Result<()> {
        let world = natural_world()?;
        let mut rng = Rng::new(7);
        for index in 0..500 {
            let held = index % 2 == 0;
            let episode = world.episode(&mut rng, held, 257)?;
            assert_eq!(episode.tokens.len(), episode.entity_atom.len());
            assert_eq!(episode.tokens.len(), episode.relation_atom.len());
            for position in 0..episode.tokens.len() {
                assert_eq!(
                    episode.tags[position] == TAG_ENTITY,
                    episode.entity_atom[position] != 0,
                    "entity atom disagrees with the entity tag at {position}"
                );
                assert_eq!(
                    episode.tags[position] == TAG_RELATION,
                    episode.relation_atom[position] != 0,
                    "relation atom disagrees with the relation tag at {position}"
                );
            }
            assert!(!episode.clauses.is_empty());
            for clause in &episode.clauses {
                assert!(clause.entity_atom != 0 && clause.relation_atom != 0);
                assert!(clause.trigger_position < episode.tokens.len());
                assert!(matches!(
                    episode.triggers[clause.trigger_position],
                    TRIGGER_WRITE | TRIGGER_READ | TRIGGER_READ_PREVIOUS
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn natural_held_out_phrasings_never_appear_in_training() -> Result<()> {
        let world = natural_world()?;
        let natural = world.natural.as_ref().expect("natural world");
        let held_values: Vec<u32> = natural
            .families
            .iter()
            .flat_map(|family| family.values_held.iter().map(|word| word.id))
            .collect();
        assert!(!held_values.is_empty());
        let mut rng = Rng::new(99);
        let mut training: Vec<String> = Vec::new();
        let mut training_tokens: Vec<u32> = Vec::new();
        for _ in 0..1_000 {
            let episode = world.episode(&mut rng, false, 257)?;
            for (user, reply) in &episode.messages {
                training.push(normalize(user));
                training.push(normalize(reply));
            }
            training_tokens.extend_from_slice(&episode.tokens);
        }
        for id in &held_values {
            assert!(
                !training_tokens.contains(id),
                "held value token {id} appears in a training episode"
            );
        }
        let mut rng = Rng::new(1_234);
        for _ in 0..400 {
            let episode = world.episode(&mut rng, true, 257)?;
            for (user, _) in &episode.messages {
                let message = normalize(user);
                assert!(
                    !training.iter().any(|text| text.contains(&message)),
                    "held phrasing {message:?} appears in a training episode"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn natural_held_out_templates_are_not_training_templates() {
        for family in NATURAL_FAMILIES {
            for held in [
                family.statements_held,
                family.queries_held,
                family.previous_held,
            ] {
                for template in held {
                    for train in [
                        family.statements_train,
                        family.queries_train,
                        family.previous_train,
                    ] {
                        assert!(
                            !train.contains(template),
                            "a held-out phrasing is also a training phrasing: {template:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn natural_episodes_answer_from_the_atom_gold_store() -> Result<()> {
        let world = natural_world()?;
        let mut rng = Rng::new(4_242);
        let mut classes: BTreeMap<QueryClass, usize> = BTreeMap::new();
        for index in 0..800 {
            let held = index % 3 == 0;
            let episode = world.episode(&mut rng, held, 257)?;
            let reads: Vec<&Clause> = episode
                .clauses
                .iter()
                .filter(|clause| {
                    matches!(
                        episode.triggers[clause.trigger_position],
                        TRIGGER_READ | TRIGGER_READ_PREVIOUS
                    )
                })
                .collect();
            assert_eq!(reads.len(), episode.queries.len());
            let mut store = AtomStore::new();
            for clause in &episode.clauses {
                match episode.triggers[clause.trigger_position] {
                    TRIGGER_WRITE => {
                        assert!(!clause.previous);
                        assert!(matches!(
                            clause.class,
                            QueryClass::First | QueryClass::Updated | QueryClass::Reasserted
                        ));
                        let value = clause.value.expect("a write carries its value");
                        store.write(clause.entity_atom, clause.relation_atom, value);
                    }
                    trigger @ (TRIGGER_READ | TRIGGER_READ_PREVIOUS) => {
                        let previous = trigger == TRIGGER_READ_PREVIOUS;
                        assert_eq!(clause.previous, previous);
                        assert_eq!(clause.value.is_none(), clause.class.abstains());
                        let (status, value) =
                            store.read(clause.entity_atom, clause.relation_atom, previous);
                        match clause.value {
                            Some(expected) => {
                                assert_eq!(status, STATUS_HIT);
                                assert_eq!(value, expected);
                            }
                            None => assert_ne!(status, STATUS_HIT),
                        }
                    }
                    other => panic!("natural clause trigger {other} outside the trigger set"),
                }
            }
            for (query, clause) in episode.queries.iter().zip(&reads) {
                *classes.entry(query.class).or_insert(0) += 1;
                assert_eq!(query.class, clause.class);
                assert_eq!(query.value, clause.value);
                assert_eq!(query.value.is_some(), !query.class.abstains());
                match query.value {
                    Some(value) => {
                        let slot = query.value_position.expect("value slot");
                        assert_eq!(episode.tokens[slot], value);
                        assert_eq!(episode.response_mask[slot], 1);
                    }
                    None => assert!(query.value_position.is_none()),
                }
                assert_eq!(episode.tokens[query.answer_end], 1);
            }
        }
        for class in [
            QueryClass::First,
            QueryClass::Updated,
            QueryClass::Reasserted,
            QueryClass::Previous,
            QueryClass::PreviousAbsent,
            QueryClass::Absent,
        ] {
            assert!(
                classes.get(&class).copied().unwrap_or(0) > 3,
                "{class:?} is rare: {classes:?}"
            );
        }
        Ok(())
    }
}
