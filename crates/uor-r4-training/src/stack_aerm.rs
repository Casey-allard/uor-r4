//! D2: architectural exact relational memory (AERM) inside the
//! recurrence-primary geometric stack (whole-project synthesis §3 M1, §4 D2).
//!
//! The stack is split after `split` layers. Two learned heads read the
//! RMS-normalised residual stream there: a per-token role tag (entity,
//! relation, value or other) and a per-token trigger (none, write, read the
//! current value, read the previous value). An exact integer store sits
//! between the two halves:
//!
//! - a trigger closes the open clause: the latest tagged entity, relation and
//!   value tokens since the previous trigger or end of turn;
//! - a write stores `(entity, relation) -> value`. A hit on an existing key
//!   overwrites it, keeps the previous distinct value and increments the
//!   version: version order, never a score, decides the current value;
//! - a read looks the key up and sets a register `(status, value)` that holds
//!   until the next read. The statuses are None, Hit, Absent and Evicted.
//!
//! The register re-enters the residual stream through a zero-initialised status
//! embedding and value projection, and adds a zero-initialised copy boost to
//! the value token's logit, so the branch starts silent. Training drives the
//! store from the gold tags and triggers (teacher forcing) and trains the heads
//! with auxiliary cross-entropies: no gradient passes through the discrete
//! store. Evaluation drives the store from the model's own tags and triggers
//! only.
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

/// Registers per position and the store after the last position.
#[derive(Clone, Debug)]
pub struct Simulation {
    pub status: Vec<u32>,
    pub value: Vec<u32>,
    pub reads: Vec<ReadEvent>,
    pub store: RelationStore,
}

/// Runs the store over one sequence from its per-token tags and triggers.
/// `eos` also closes the open clause (a turn boundary).
pub fn simulate(tokens: &[u32], tags: &[u32], triggers: &[u32], eos: u32) -> Result<Simulation> {
    if tags.len() != tokens.len() || triggers.len() != tokens.len() {
        return Err(invalid("one tag and one trigger per token"));
    }
    let mut store = RelationStore::new();
    let (mut entity, mut relation, mut value) = (None, None, None);
    let mut register = (STATUS_NONE, 0u32);
    let mut status = Vec::with_capacity(tokens.len());
    let mut values = Vec::with_capacity(tokens.len());
    let mut reads = Vec::new();
    for (position, ((&token, &tag), &trigger)) in tokens.iter().zip(tags).zip(triggers).enumerate()
    {
        match tag {
            TAG_OTHER => {}
            TAG_ENTITY => entity = Some(token),
            TAG_RELATION => relation = Some(token),
            TAG_VALUE => value = Some(token),
            _ => return Err(invalid("tag outside the tag set")),
        }
        let mut close = token == eos;
        match trigger {
            TRIGGER_NONE => {}
            TRIGGER_WRITE => {
                if let (Some(e), Some(r), Some(v)) = (entity, relation, value) {
                    store.write(e, r, v);
                }
                close = true;
            }
            TRIGGER_READ | TRIGGER_READ_PREVIOUS => {
                let previous = trigger == TRIGGER_READ_PREVIOUS;
                let key = entity.zip(relation);
                register = match key {
                    Some((e, r)) => store.read(e, r, previous),
                    None => (STATUS_ABSENT, 0),
                };
                reads.push(ReadEvent {
                    position,
                    key,
                    previous,
                    status: register.0,
                    value: register.1,
                });
                close = true;
            }
            _ => return Err(invalid("trigger outside the trigger set")),
        }
        if close {
            (entity, relation, value) = (None, None, None);
        }
        status.push(register.0);
        values.push(register.1);
    }
    Ok(Simulation {
        status,
        value: values,
        reads,
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
    text: &'static str,
    id: u32,
}

#[derive(Clone, Debug)]
struct RelationSpec {
    word: Word,
    article: Option<&'static str>,
    values: Vec<Word>,
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
    pub queries: Vec<Query>,
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
                [id] => Ok(Word { text, id: *id }),
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
            let values = match values {
                Some(values) => values
                    .iter()
                    .map(|&v| single(v))
                    .collect::<Result<Vec<_>>>()?,
                None => names.clone(),
            };
            relations.push(RelationSpec {
                word: single(word)?,
                article,
                values,
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
        })
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
                "word": (r.word.text, r.word.id),
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

    /// Samples one episode that fits `limit` tokens (`context + 1`).
    pub fn episode(&self, rng: &mut Rng, held: bool, limit: usize) -> Result<Episode> {
        EpisodeBuilder::new(self, held, limit).build(rng)
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
                    out.text.push_str(entity.text);
                    out.tokens.push(entity.id);
                    out.tags.push(TAG_ENTITY);
                }
                Relation => {
                    out.text.push_str(relation.word.text);
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
                    out.text.push_str(value.text);
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
        out.text.push_str(value.text);
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
        self.tokens.push(self.world.eos);
        self.mask.push(1);
        self.tags.push(TAG_OTHER);
        self.triggers.push(TRIGGER_NONE);
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
        Ok(Episode {
            messages: self.messages,
            tokens: self.tokens,
            response_mask: self.mask,
            tags: self.tags,
            triggers: self.triggers,
            queries: self.queries,
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
    /// Gold registers (teacher forcing).
    pub status: Vec<u32>,
    pub value: Vec<u32>,
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
            status: vec![STATUS_NONE; rows],
            value: vec![0; rows],
        };
        for (b, episode) in episodes.iter().enumerate() {
            let n = episode.tokens.len();
            if n < 2 || n > context + 1 {
                return Err(invalid("an episode must fit the context"));
            }
            let inputs = n - 1;
            let gold = simulate(
                &episode.tokens[..inputs],
                &episode.tags[..inputs],
                &episode.triggers[..inputs],
                self.eos,
            )?;
            for t in 0..inputs {
                let row = b * context + t;
                out.ids[row] = episode.tokens[t];
                out.targets[row] = episode.tokens[t + 1];
                out.response[row] = f32::from(episode.response_mask[t + 1]);
                out.real[row] = 1.0;
                out.tags[row] = episode.tags[t];
                out.triggers[row] = episode.triggers[t];
                out.status[row] = gold.status[t];
                out.value[row] = gold.value[t];
            }
        }
        out.episodes = episodes;
        Ok(out)
    }
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

fn zeros_var(shape: &[usize], device: &Device) -> Result<Var> {
    Ok(Var::from_tensor(&Tensor::zeros(
        shape,
        DType::F32,
        device,
    )?)?)
}

/// The memory branch: status embedding, value projection and copy boost.
struct MemoryBranch {
    status: Var,
    projection: Var,
    copy_weight: Var,
    copy_bias: Var,
    copy_scale: Var,
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
            Some(MemoryBranch {
                status: zeros_var(&[STATUSES, width], device)?,
                projection: zeros_var(&[width, width], device)?,
                copy_weight: zeros_var(&[width, 1], device)?,
                copy_bias: zeros_var(&[1], device)?,
                copy_scale: zeros_var(&[1], device)?,
            })
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
    /// stream and the per-position registers (ignored without a memory branch).
    pub fn top(&self, hidden: &Tensor, status: &[u32], value: &[u32]) -> Result<Tensor> {
        let (batch, time, width) = hidden.dims3()?;
        let rows = batch * time;
        let device = self.stack.device();
        let layers = self.stack.config.layers();
        let Some(m) = &self.memory else {
            let x = self.stack.run_layers(hidden.clone(), self.split..layers)?;
            return self.stack.head(&self.stack.finish(x)?);
        };
        if status.len() != rows || value.len() != rows {
            return Err(invalid("one register per position"));
        }
        if status.iter().any(|&s| s as usize >= STATUSES)
            || value
                .iter()
                .any(|&v| v as usize >= self.stack.config.vocab_size)
        {
            return Err(invalid("register outside the status set or vocabulary"));
        }
        let hit: Vec<f32> = status
            .iter()
            .map(|&s| if s == STATUS_HIT { 1.0 } else { 0.0 })
            .collect();
        let hit = Tensor::from_vec(hit, (rows, 1), device)?;
        let status_index = Tensor::from_vec(status.to_vec(), rows, device)?;
        let value_index = Tensor::from_vec(value.to_vec(), rows, device)?;
        let embedding = self
            .stack
            .variables()
            .get("embedding.weight")
            .ok_or_else(|| invalid("the stack has no embedding"))?
            .as_tensor();
        let value_rows = embedding
            .index_select(&value_index, 0)?
            .broadcast_mul(&hit)?;
        let side = m
            .status
            .as_tensor()
            .index_select(&status_index, 0)?
            .add(&value_rows.matmul(&m.projection.as_tensor().t()?)?)?
            .reshape((batch, time, width))?;
        let x = self
            .stack
            .run_layers(hidden.add(&side)?, self.split..layers)?;
        let out = self.stack.finish(x)?;
        let logits = self.stack.head(&out)?;
        let gate = candle_nn::ops::sigmoid(
            &out.matmul(m.copy_weight.as_tensor())?
                .broadcast_add(m.copy_bias.as_tensor())?,
        )?;
        let boost = gate
            .broadcast_mul(m.copy_scale.as_tensor())?
            .broadcast_mul(&hit)?;
        let vocab = self.stack.config.vocab_size as u32;
        let onehot = value_index
            .unsqueeze(1)?
            .broadcast_eq(&Tensor::arange(0u32, vocab, device)?.unsqueeze(0)?)?
            .to_dtype(DType::F32)?;
        Ok(logits.add(&onehot.broadcast_mul(&boost)?)?)
    }
}

/// Argmax of each row of `[rows, classes]` logits.
fn argmax_rows(logits: &Tensor) -> Result<Vec<u32>> {
    Ok(logits.detach().argmax(D::Minus1)?.to_vec1::<u32>()?)
}

/// Registers from the model's own tags and triggers, one sequence at a time.
pub fn model_registers(
    ids: &[u32],
    tags: &[u32],
    triggers: &[u32],
    batch: usize,
    time: usize,
    eos: u32,
) -> Result<(Vec<u32>, Vec<u32>)> {
    let mut status = Vec::with_capacity(batch * time);
    let mut value = Vec::with_capacity(batch * time);
    for b in 0..batch {
        let range = b * time..(b + 1) * time;
        let simulation = simulate(
            &ids[range.clone()],
            &tags[range.clone()],
            &triggers[range],
            eos,
        )?;
        status.extend(simulation.status);
        value.extend(simulation.value);
    }
    Ok((status, value))
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
        let logits = model.top(&bottom.hidden, &silent_status, &silent_value)?;
        let text_loss = logits_cross_entropy(&logits, &targets, None)?;
        let text_tag = head_loss(&bottom.tags, &text_tags, &text_ones)?;
        let text_trigger = head_loss(&bottom.triggers, &text_triggers, &text_ones)?;
        // Dialogues.
        let dialogues = world.batch(&mut rng, batch, context, false)?;
        let bottom = model.bottom(&dialogues.ids, batch, context)?;
        let logits = model.top(&bottom.hidden, &dialogues.status, &dialogues.value)?;
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
        let aux_tag = ((&text_tag + &tag_loss)? * (0.5 * config.tag_weight))?;
        let aux_trigger = ((&text_trigger + &trigger_loss)? * (0.5 * config.trigger_weight))?;
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
    pub trace: BTreeMap<String, usize>,
    /// The last query of the first episodes: gold and teacher-forced predictions.
    pub examples: Vec<QueryExample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryExample {
    pub class: QueryClass,
    pub gold_answer: Vec<u32>,
    pub predicted_answer: Vec<u32>,
    pub register: (u32, u32),
    pub gold_register: (u32, u32),
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
        let (status, value) =
            model_registers(&data.ids, &tags, &triggers, size, context, world.eos)?;
        let predicted = argmax_rows(&model.top(&bottom.hidden, &status, &value)?)?;
        let oracle = if model.has_memory() {
            Some(argmax_rows(&model.top(
                &bottom.hidden,
                &data.status,
                &data.value,
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
                let register = (status[base + check], value[base + check]);
                let gold_register = (data.status[base + check], data.value[base + check]);
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
    register: (u32, u32),
    gold_register: (u32, u32),
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
        let (status, value) = model_registers(&ids, &tags, &triggers, chunk.len(), context, eos)?;
        let logits = model.top(&bottom.hidden, &status, &value)?.detach();
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
            let (status, value) = model_registers(&ids, &tags, &triggers, 1, time, world.eos)?;
            let logits = model.top(&bottom.hidden, &status, &value)?;
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
    use crate::geometric_stack::{ReadScore, StackArch};
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
                let register = (
                    gold.status[query.answer_start - 1],
                    gold.value[query.answer_start - 1],
                );
                match query.value {
                    Some(value) => {
                        assert_eq!(register, (STATUS_HIT, value));
                        let slot = query.value_position.expect("value slot");
                        assert_eq!(episode.tokens[slot], value);
                        assert_eq!(episode.response_mask[slot], 1);
                    }
                    None => assert_ne!(register.0, STATUS_HIT),
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
            .top(&bottom.hidden, &status, &value)?
            .to_vec2::<f32>()?;
        for (a, b) in reference.iter().flatten().zip(split.iter().flatten()) {
            assert!((a - b).abs() < 1e-5, "{a} vs {b}");
        }
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
        let logits = model.top(&bottom.hidden, &data.status, &data.value)?;
        let loss = logits_cross_entropy(&logits, &data.targets, Some(&data.response))?;
        let grads = loss.backward()?;
        let m = model.memory.as_ref().expect("memory branch");
        for (name, var) in [
            ("status", &m.status),
            ("projection", &m.projection),
            ("scale", &m.copy_scale),
        ] {
            let grad = grads.get(var.as_tensor()).expect("gradient");
            let norm = grad.sqr()?.sum_all()?.to_scalar::<f32>()?;
            assert!(norm > 0.0, "{name} receives no gradient");
        }
        Ok(())
    }
}
