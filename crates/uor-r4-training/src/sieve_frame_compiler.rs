//! A lawful turn compiler for the world=v2 schema: exact frames routed by a
//! prime sieve.
//!
//! Every distinct word of the world=v2 *training* phrasings is one prime. A
//! word set is the squarefree product of its primes, held in its exact bitset
//! form: a squarefree integer is the set of its prime factors, so gcd is AND
//! and divisibility is subset. Each training phrasing is one frame (the words
//! before its `{v}` slot and the words after it), and its routing expert is
//! the semiprime `p_relation · q_act`.
//!
//! A turn compiles only when every frame that divides its signature and
//! aligns exactly with its words names one expert and one value span.
//! Anything else is `Unresolved`. The uniqueness test is the abstention gate;
//! there is no score, probability, softmax or matrix product anywhere.
//!
//! This module prepares data (tokenizing, the frame table, flat device
//! arrays) and decodes stored decisions. The compile decision itself (sieve,
//! align, agree) runs on the GPU in `sieve_frame.cu`. There is no host
//! implementation of it (CHARTER §3.5).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::milestone_world::{Rng, Split};
use crate::milestone_world_v2::relations;
use crate::turn_action::{
    CompiledAction, CompilerIdentity, GroundedSessionError, RelationLabel, SourceSpan, TurnCompiler,
};
use crate::{invalid, sha256_bytes, Result};

/// The compiler artifact's schema.
pub const SIEVE_COMPILER_SCHEMA: &str = "uor-r4.sieve-frame-compiler/1";
/// The world=v2 relation and act labels, the same schema the saved compiler bound.
pub const SIEVE_LABEL_SCHEMA: &str = "m-world-v2.relations-acts/1";
/// The word rule, hashed as the compiler's tokenizer identity.
pub const WORD_RULE: &str = "uor-r4.sieve-words/1: a word is a maximal run of alphanumeric \
characters and apostrophes (' and U+2019, the latter read as '), lowercased; everything else \
separates words";
/// The longest value a slot accepts, in words.
pub const MAX_VALUE_WORDS: u32 = 4;
/// The registered control draw (prep sheet §3): the seed and the development
/// values per phrasing.
pub const REGISTERED_SEED: u64 = 2856;
pub const REGISTERED_VALUES_PER_PHRASING: usize = 8;
/// The SHA-256 of [`FrameTable::world_v2`]'s bytes, registered in the prep
/// sheet before the panel ran. A change to the phrasings, the word rule or the
/// table layout moves it, and its test fails.
pub const WORLD_V2_FRAME_TABLE_SHA256: &str =
    "379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f";
/// The slot marker in a world=v2 phrasing.
const SLOT: &str = "{v}";

/// Decision status: no frame survives the sieve and alignment.
pub const STATUS_NONE: u32 = 0;
/// Decision status: every surviving frame names one expert and one value span.
pub const STATUS_COMPILED: u32 = 1;
/// Decision status: surviving frames disagree on the expert or the span.
pub const STATUS_DISAGREE: u32 = 2;

/// What a frame does to the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Act {
    Assert,
    Update,
    Query,
}

impl Act {
    const ALL: [Act; 3] = [Act::Assert, Act::Update, Act::Query];

    fn index(self) -> usize {
        match self {
            Act::Assert => 0,
            Act::Update => 1,
            Act::Query => 2,
        }
    }
}

/// One word of a turn and its half-open byte span in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// Split `source` into words by [`WORD_RULE`].
pub fn words(source: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (at, c) in source.char_indices() {
        if c.is_alphanumeric() || c == '\'' || c == '\u{2019}' {
            let text = &mut current.get_or_insert_with(|| (at, String::new())).1;
            if c == '\u{2019}' {
                text.push('\'');
            } else {
                text.extend(c.to_lowercase());
            }
        } else if let Some((start, text)) = current.take() {
            out.push(Word {
                text,
                start,
                end: at,
            });
        }
    }
    if let Some((start, text)) = current {
        out.push(Word {
            text,
            start,
            end: source.len(),
        });
    }
    out
}

/// The first `count` primes, by trial division over the primes found so far.
fn primes(count: usize) -> Vec<u64> {
    let mut found: Vec<u64> = Vec::with_capacity(count);
    let mut candidate = 2u64;
    while found.len() < count {
        if found
            .iter()
            .take_while(|&&p| p * p <= candidate)
            .all(|&p| candidate % p != 0)
        {
            found.push(candidate);
        }
        candidate += 1;
    }
    found
}

/// One training phrasing as exact memory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    /// The world=v2 relation ID, 1-based in table order.
    pub relation: u32,
    pub act: Act,
    /// The training phrasing the frame was read from.
    pub template: String,
    /// Lexicon word IDs (1-based) before the slot; the whole phrasing for a query.
    pub prefix: Vec<u32>,
    /// Lexicon word IDs after the slot; empty for a query.
    pub suffix: Vec<u32>,
    pub slot: bool,
    /// The routing expert `p_relation · q_act`.
    pub expert: u64,
}

/// The compiler artifact: the prime lexicon and every training frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameTable {
    pub schema: String,
    pub label_schema: String,
    pub word_rule: String,
    pub max_value_words: u32,
    /// Sorted. Word ID `k + 1` is `lexicon[k]`, whose prime is `word_primes[k]`.
    pub lexicon: Vec<String>,
    pub word_primes: Vec<u64>,
    pub act_primes: [u64; 3],
    pub relations: Vec<RelationLabel>,
    pub relation_primes: Vec<u64>,
    pub frames: Vec<Frame>,
}

impl FrameTable {
    /// The frame table of the world=v2 training phrasings. Development
    /// phrasings and values never enter it.
    pub fn world_v2() -> Result<Self> {
        let table = relations();
        let mut raw = Vec::new();
        for (index, relation) in table.iter().enumerate() {
            let id = u32::try_from(index + 1).map_err(|_| invalid("too many relations"))?;
            let phrasings = [relation.assert, relation.update, relation.query];
            for (act, phrasing) in Act::ALL.into_iter().zip(phrasings) {
                for &template in phrasing.train {
                    raw.push((id, act, template, split_template(template, act)?));
                }
            }
        }
        let lexicon: Vec<String> = raw
            .iter()
            .flat_map(|(.., (before, after))| before.iter().chain(after))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let experts = primes(Act::ALL.len() + table.len());
        let act_primes = [experts[0], experts[1], experts[2]];
        let relation_primes = experts[Act::ALL.len()..].to_vec();
        let id_of = |word: &String| -> Result<u32> {
            let at = lexicon
                .binary_search(word)
                .map_err(|_| invalid("a frame word is missing from the lexicon"))?;
            u32::try_from(at + 1).map_err(|_| invalid("lexicon too large"))
        };
        let mut frames = Vec::with_capacity(raw.len());
        for (relation, act, template, (before, after)) in raw {
            frames.push(Frame {
                relation,
                act,
                template: template.to_string(),
                prefix: before.iter().map(id_of).collect::<Result<_>>()?,
                suffix: after.iter().map(id_of).collect::<Result<_>>()?,
                slot: act != Act::Query,
                expert: relation_primes[relation as usize - 1] * act_primes[act.index()],
            });
        }
        Ok(Self {
            schema: SIEVE_COMPILER_SCHEMA.into(),
            label_schema: SIEVE_LABEL_SCHEMA.into(),
            word_rule: WORD_RULE.into(),
            max_value_words: MAX_VALUE_WORDS,
            word_primes: primes(lexicon.len()),
            lexicon,
            act_primes,
            relations: table
                .iter()
                .enumerate()
                .map(|(index, relation)| RelationLabel {
                    id: index as u32 + 1,
                    name: relation.name.to_string(),
                })
                .collect(),
            relation_primes,
            frames,
        })
    }

    /// The artifact's bytes: its JSON, in field and table order.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| invalid(format!("frame table JSON: {e}")))
    }

    pub fn identity(&self) -> Result<CompilerIdentity> {
        Ok(CompilerIdentity {
            schema: self.schema.clone(),
            artifact_sha256: sha256_bytes(&self.canonical_bytes()?),
            tokenizer_sha256: sha256_bytes(self.word_rule.as_bytes()),
            label_schema: self.label_schema.clone(),
            relations: self.relations.clone(),
            encoder: None,
        })
    }

    /// 64-bit words per signature.
    pub fn sig_words(&self) -> usize {
        self.lexicon.len().div_ceil(64)
    }

    /// A word's lexicon ID, or 0 for a word no training phrasing contains.
    pub fn word_id(&self, word: &str) -> u32 {
        self.lexicon
            .binary_search_by(|entry| entry.as_str().cmp(word))
            .map_or(0, |at| at as u32 + 1)
    }

    /// The squarefree prime product of `ids`, as its prime-factor bitset.
    /// Unknown words (ID 0) carry no prime.
    pub fn signature(&self, ids: &[u32]) -> Vec<u64> {
        let mut bits = vec![0u64; self.sig_words()];
        for &id in ids.iter().filter(|&&id| id != 0) {
            let k = id as usize - 1;
            bits[k / 64] |= 1u64 << (k % 64);
        }
        bits
    }

    /// The frames in the flat layout the kernel reads.
    pub fn device_frames(&self) -> Result<DeviceFrames> {
        let mut out = DeviceFrames {
            sig_words: u32::try_from(self.sig_words()).map_err(|_| invalid("signature width"))?,
            ..DeviceFrames::default()
        };
        for frame in &self.frames {
            let ids: Vec<u32> = frame.prefix.iter().chain(&frame.suffix).copied().collect();
            out.signature.extend(self.signature(&ids));
            out.offset.push(len_u32(out.tokens.len())?);
            out.tokens.extend(&ids);
            out.prefix.push(len_u32(frame.prefix.len())?);
            out.suffix.push(len_u32(frame.suffix.len())?);
            out.slot.push(u32::from(frame.slot));
            out.expert.push(frame.expert);
        }
        Ok(out)
    }

    /// Tokenize turns into the flat layout the kernel reads (data preparation).
    pub fn encode_turns(&self, sources: &[String]) -> Result<(Vec<Vec<Word>>, DeviceTurns)> {
        let mut split = Vec::with_capacity(sources.len());
        let mut out = DeviceTurns::default();
        for source in sources {
            let turn = words(source);
            let ids: Vec<u32> = turn.iter().map(|w| self.word_id(&w.text)).collect();
            out.signature.extend(self.signature(&ids));
            out.offset.push(len_u32(out.tokens.len())?);
            out.length.push(len_u32(ids.len())?);
            out.tokens.extend(ids);
            split.push(turn);
        }
        Ok((split, out))
    }

    /// The action a stored device decision stands for.
    pub fn decode(&self, turn: &[Word], decision: Decision) -> Result<CompiledAction> {
        match decision.status {
            STATUS_NONE => {
                return Ok(CompiledAction::Unresolved {
                    reason: "no frame survives the sieve and alignment".into(),
                })
            }
            STATUS_DISAGREE => {
                return Ok(CompiledAction::Unresolved {
                    reason: "the surviving frames disagree".into(),
                })
            }
            STATUS_COMPILED => {}
            other => return Err(invalid(format!("unknown decision status {other}"))),
        }
        let frame = self
            .frames
            .get(decision.frame as usize)
            .ok_or_else(|| invalid("decision names a frame outside the table"))?;
        let relation = frame.relation;
        if frame.act == Act::Query {
            return Ok(CompiledAction::QueryCurrent { relation });
        }
        let first = decision.hole_start as usize;
        let last = (first + decision.hole_len as usize)
            .checked_sub(1)
            .filter(|_| decision.hole_len > 0)
            .ok_or_else(|| invalid("a compiled slot decision has an empty value"))?;
        let (Some(head), Some(tail)) = (turn.get(first), turn.get(last)) else {
            return Err(invalid("decision value lies outside the turn"));
        };
        let span = SourceSpan {
            start: head.start,
            end: tail.end,
        };
        Ok(match frame.act {
            Act::Assert => CompiledAction::Assert { relation, span },
            Act::Update => CompiledAction::Correct { relation, span },
            Act::Query => unreachable!("queries return above"),
        })
    }
}

/// A training phrasing's words before and after its slot.
fn split_template(template: &str, act: Act) -> Result<(Vec<String>, Vec<String>)> {
    let texts = |s: &str| words(s).into_iter().map(|w| w.text).collect::<Vec<_>>();
    match (act, template.matches(SLOT).count(), template.find(SLOT)) {
        (Act::Query, 0, _) => Ok((texts(template), Vec::new())),
        (Act::Assert | Act::Update, 1, Some(at)) => {
            Ok((texts(&template[..at]), texts(&template[at + SLOT.len()..])))
        }
        _ => Err(invalid(format!(
            "phrasing {template:?} has the wrong slots for {act:?}"
        ))),
    }
}

fn len_u32(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid("device array too large"))
}

/// Frames in the kernel's flat layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceFrames {
    pub sig_words: u32,
    pub signature: Vec<u64>,
    pub tokens: Vec<u32>,
    pub offset: Vec<u32>,
    pub prefix: Vec<u32>,
    pub suffix: Vec<u32>,
    pub slot: Vec<u32>,
    pub expert: Vec<u64>,
}

/// Turns in the kernel's flat layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceTurns {
    pub signature: Vec<u64>,
    pub tokens: Vec<u32>,
    pub offset: Vec<u32>,
    pub length: Vec<u32>,
}

/// One turn's stored device decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub status: u32,
    pub frame: u32,
    pub hole_start: u32,
    pub hole_len: u32,
    pub survivors: u32,
}

/// A [`TurnCompiler`] that answers from decisions the GPU already made.
pub struct DecidedCompiler {
    identity: CompilerIdentity,
    artifact: Vec<u8>,
    actions: BTreeMap<String, CompiledAction>,
}

impl DecidedCompiler {
    pub fn new(table: &FrameTable, actions: BTreeMap<String, CompiledAction>) -> Result<Self> {
        Ok(Self {
            identity: table.identity()?,
            artifact: table.canonical_bytes()?,
            actions,
        })
    }
}

impl TurnCompiler for DecidedCompiler {
    fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }

    fn artifact_bytes(&self) -> &[u8] {
        &self.artifact
    }

    fn compile(&self, source: &str) -> std::result::Result<CompiledAction, GroundedSessionError> {
        self.actions.get(source).cloned().ok_or_else(|| {
            GroundedSessionError::Compiler("this turn was not compiled on the device".into())
        })
    }
}

/// A true relation turn and the action a correct compiler returns for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlTurn {
    pub source: String,
    pub expected: CompiledAction,
    pub template: String,
}

/// World=v2 relation turns from the `phrasings` split's phrasings, filled with
/// `values_per_phrasing` development values drawn from `seed`. A query
/// phrasing has no value and appears once.
pub fn world_v2_controls(
    phrasings: Split,
    values_per_phrasing: usize,
    seed: u64,
) -> Result<Vec<ControlTurn>> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::new();
    for (index, relation) in relations().iter().enumerate() {
        let id = index as u32 + 1;
        let sets = [relation.assert, relation.update, relation.query];
        for (act, set) in Act::ALL.into_iter().zip(sets) {
            let templates = match phrasings {
                Split::Train => set.train,
                Split::Development => set.development,
            };
            for &template in templates {
                if act == Act::Query {
                    out.push(ControlTurn {
                        source: template.to_string(),
                        expected: CompiledAction::QueryCurrent { relation: id },
                        template: template.to_string(),
                    });
                    continue;
                }
                let at = template
                    .find(SLOT)
                    .ok_or_else(|| invalid(format!("phrasing {template:?} has no slot")))?;
                for _ in 0..values_per_phrasing {
                    let value = relation.draw(&mut rng, Split::Development)?;
                    let span = SourceSpan {
                        start: at,
                        end: at + value.len(),
                    };
                    let expected = match act {
                        Act::Assert => CompiledAction::Assert { relation: id, span },
                        _ => CompiledAction::Correct { relation: id, span },
                    };
                    out.push(ControlTurn {
                        source: template.replacen(SLOT, &value, 1),
                        expected,
                        template: template.to_string(),
                    });
                }
            }
        }
    }
    Ok(out)
}

#[cfg(feature = "sieve-cuda")]
pub mod device {
    //! The compile kernel's launcher.

    use cudarc::driver::{CudaContext, CudaFunction, CudaStream, LaunchConfig, Ptx, PushKernelArg};
    use std::sync::Arc;
    use std::time::Instant;

    use super::{Decision, DeviceFrames, DeviceTurns};
    use crate::{invalid, sha256_bytes, Result};

    /// The ahead-of-time PTX module (build.rs, nvcc).
    pub const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/sieve_frame.ptx"));

    pub fn ptx_sha256() -> String {
        sha256_bytes(PTX.as_bytes())
    }

    fn cuda(error: impl std::fmt::Display) -> crate::TrainingError {
        invalid(format!("CUDA: {error}"))
    }

    pub struct SieveDevice {
        stream: Arc<CudaStream>,
        kernel: CudaFunction,
    }

    impl SieveDevice {
        pub fn new(ordinal: usize) -> Result<Self> {
            let context = CudaContext::new(ordinal).map_err(cuda)?;
            let stream = context.default_stream();
            let module = context.load_module(Ptx::from_src(PTX)).map_err(cuda)?;
            let kernel = module.load_function("sieve_frame_compile").map_err(cuda)?;
            Ok(Self { stream, kernel })
        }

        /// Compile every turn in one launch. Returns the decisions and the
        /// launch's wall time in microseconds (upload to download).
        #[allow(unsafe_code)]
        pub fn compile(
            &self,
            frames: &DeviceFrames,
            turns: &DeviceTurns,
            max_value_words: u32,
        ) -> Result<(Vec<Decision>, u64)> {
            let n_turns = super::len_u32(turns.length.len())?;
            let n_frames = super::len_u32(frames.prefix.len())?;
            if n_turns == 0 || n_frames == 0 {
                return Err(invalid("the compile launch needs turns and frames"));
            }
            let s = &self.stream;
            let started = Instant::now();
            let turn_sig = s.clone_htod(&turns.signature).map_err(cuda)?;
            let turn_tokens = s.clone_htod(&nonempty(&turns.tokens)).map_err(cuda)?;
            let turn_off = s.clone_htod(&turns.offset).map_err(cuda)?;
            let turn_len = s.clone_htod(&turns.length).map_err(cuda)?;
            let frame_sig = s.clone_htod(&frames.signature).map_err(cuda)?;
            let frame_tokens = s.clone_htod(&frames.tokens).map_err(cuda)?;
            let frame_off = s.clone_htod(&frames.offset).map_err(cuda)?;
            let frame_prefix = s.clone_htod(&frames.prefix).map_err(cuda)?;
            let frame_suffix = s.clone_htod(&frames.suffix).map_err(cuda)?;
            let frame_slot = s.clone_htod(&frames.slot).map_err(cuda)?;
            let frame_expert = s.clone_htod(&frames.expert).map_err(cuda)?;
            let n = n_turns as usize;
            let mut status = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut frame = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut hole_start = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut hole_len = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut survivors = s.alloc_zeros::<u32>(n).map_err(cuda)?;
            let mut launch = s.launch_builder(&self.kernel);
            launch
                .arg(&turn_sig)
                .arg(&turn_tokens)
                .arg(&turn_off)
                .arg(&turn_len)
                .arg(&n_turns)
                .arg(&frame_sig)
                .arg(&frame_tokens)
                .arg(&frame_off)
                .arg(&frame_prefix)
                .arg(&frame_suffix)
                .arg(&frame_slot)
                .arg(&frame_expert)
                .arg(&n_frames)
                .arg(&frames.sig_words)
                .arg(&max_value_words)
                .arg(&mut status)
                .arg(&mut frame)
                .arg(&mut hole_start)
                .arg(&mut hole_len)
                .arg(&mut survivors);
            let threads = n_turns.clamp(1, 128);
            let config = LaunchConfig {
                grid_dim: (n_turns.div_ceil(threads), 1, 1),
                block_dim: (threads, 1, 1),
                shared_mem_bytes: 0,
            };
            // SAFETY: the argument list is exactly sieve_frame_compile's ABI;
            // every input is a device copy of a checked host array, and every
            // output holds one u32 per turn, the kernel's only writes.
            unsafe { launch.launch(config) }.map_err(cuda)?;
            let status = s.clone_dtoh(&status).map_err(cuda)?;
            let frame = s.clone_dtoh(&frame).map_err(cuda)?;
            let hole_start = s.clone_dtoh(&hole_start).map_err(cuda)?;
            let hole_len = s.clone_dtoh(&hole_len).map_err(cuda)?;
            let survivors = s.clone_dtoh(&survivors).map_err(cuda)?;
            let micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
            let decisions = (0..n)
                .map(|t| Decision {
                    status: status[t],
                    frame: frame[t],
                    hole_start: hole_start[t],
                    hole_len: hole_len[t],
                    survivors: survivors[t],
                })
                .collect();
            Ok((decisions, micros))
        }
    }

    /// A device allocation cannot be empty; a batch of empty turns still
    /// uploads one unread word.
    fn nonempty(tokens: &[u32]) -> Vec<u32> {
        if tokens.is_empty() {
            vec![0]
        } else {
            tokens.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_split_lowercase_and_keep_byte_spans() {
        let got = words("Hi, I'm Sam O\u{2019}Neil!");
        let texts: Vec<&str> = got.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(texts, ["hi", "i'm", "sam", "o'neil"]);
        assert_eq!((got[2].start, got[2].end), (8, 11));
        assert_eq!(got[3].end, "Hi, I'm Sam O\u{2019}Neil".len());
        assert!(words(" ...?! ").is_empty());
    }

    #[test]
    fn primes_are_the_first_primes() {
        assert_eq!(primes(10), [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn world_v2_frame_table_is_deterministic_and_complete() -> Result<()> {
        let table = FrameTable::world_v2()?;
        assert_eq!(
            table.canonical_bytes()?,
            FrameTable::world_v2()?.canonical_bytes()?
        );
        let names: Vec<&str> = table.relations.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "user_name",
                "pet_name",
                "friend_name",
                "hometown",
                "lucky_number",
                "code_word",
                "job",
                "home",
                "favorite_food",
                "favorite_color"
            ]
        );
        assert_eq!(
            table.relations,
            crate::nonrelation_prose_panel::world_v2_relation_labels()
        );
        let expected: usize = relations()
            .iter()
            .map(|r| r.assert.train.len() + r.update.train.len() + r.query.train.len())
            .sum();
        assert_eq!(table.frames.len(), expected);
        let mut experts = BTreeMap::new();
        for frame in &table.frames {
            assert_eq!(frame.slot, frame.act != Act::Query);
            assert!(
                frame.prefix.len() + frame.suffix.len() > 0,
                "{}",
                frame.template
            );
            let key = (frame.relation, frame.act);
            assert_eq!(*experts.entry(key).or_insert(frame.expert), frame.expert);
        }
        // One expert per (relation, act), and every expert is a semiprime of
        // one relation prime and one act prime.
        let distinct: BTreeSet<u64> = experts.values().copied().collect();
        assert_eq!(distinct.len(), experts.len());
        for (&(relation, act), &expert) in &experts {
            let p = table.relation_primes[relation as usize - 1];
            let q = table.act_primes[act.index()];
            assert_eq!(expert, p * q);
            assert!(!table.relation_primes.contains(&q) && !table.act_primes.contains(&p));
        }
        Ok(())
    }

    #[test]
    fn the_world_v2_frame_table_matches_its_registered_digest() -> Result<()> {
        let table = FrameTable::world_v2()?;
        assert_eq!(
            table.identity()?.artifact_sha256,
            WORLD_V2_FRAME_TABLE_SHA256
        );
        Ok(())
    }

    #[test]
    fn the_lexicon_holds_training_phrasing_words_only() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let training: BTreeSet<String> = relations()
            .iter()
            .flat_map(|r| [r.assert.train, r.update.train, r.query.train])
            .flatten()
            .flat_map(|t| words(&t.replace(SLOT, " ")))
            .map(|w| w.text)
            .collect();
        assert_eq!(table.lexicon, training.into_iter().collect::<Vec<_>>());
        let development_only: Vec<String> = relations()
            .iter()
            .flat_map(|r| {
                [
                    r.assert.development,
                    r.update.development,
                    r.query.development,
                ]
            })
            .flatten()
            .flat_map(|t| words(&t.replace(SLOT, " ")))
            .map(|w| w.text)
            .filter(|w| table.word_id(w) == 0)
            .collect();
        assert!(
            !development_only.is_empty(),
            "development phrasings add new words"
        );
        Ok(())
    }

    #[test]
    fn the_signature_is_the_squarefree_prime_product() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let turn = ["my", "name", "is", "zzzunknown", "name"];
        let ids: Vec<u32> = turn.iter().map(|w| table.word_id(w)).collect();
        assert_eq!(ids[3], 0);
        let mut product: u128 = 1;
        for &id in ids.iter().filter(|&&id| id != 0).collect::<BTreeSet<_>>() {
            product *= u128::from(table.word_primes[id as usize - 1]);
        }
        let bits = table.signature(&ids);
        for (k, &prime) in table.word_primes.iter().enumerate() {
            let set = bits[k / 64] >> (k % 64) & 1 == 1;
            assert_eq!(
                set,
                product % u128::from(prime) == 0,
                "word {}",
                table.lexicon[k]
            );
        }
        Ok(())
    }

    #[test]
    fn device_layout_matches_the_frames() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let flat = table.device_frames()?;
        let n = table.frames.len();
        let w = table.sig_words();
        assert_eq!(flat.signature.len(), n * w);
        for (f, frame) in table.frames.iter().enumerate() {
            let at = flat.offset[f] as usize;
            let len = (flat.prefix[f] + flat.suffix[f]) as usize;
            let ids: Vec<u32> = frame.prefix.iter().chain(&frame.suffix).copied().collect();
            assert_eq!(&flat.tokens[at..at + len], ids.as_slice());
            assert_eq!(
                flat.prefix[f] as usize,
                frame.prefix.len(),
                "{}",
                frame.template
            );
            assert_eq!(
                flat.suffix[f] as usize,
                frame.suffix.len(),
                "{}",
                frame.template
            );
            assert_eq!(flat.slot[f], u32::from(frame.slot), "{}", frame.template);
            assert_eq!(
                &flat.signature[f * w..(f + 1) * w],
                table.signature(&ids).as_slice()
            );
            assert_eq!(flat.expert[f], frame.expert);
        }
        let sources = vec!["My name is Ruby.".to_string(), String::new()];
        let (split, turns) = table.encode_turns(&sources)?;
        assert_eq!(turns.length, [4, 0]);
        assert_eq!(turns.offset, [0, 4]);
        assert_eq!(split[0][3].text, "ruby");
        assert_eq!(turns.tokens[3], 0);
        Ok(())
    }

    #[test]
    fn decode_turns_stored_decisions_into_actions() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let find = |relation: u32, act: Act| {
            table
                .frames
                .iter()
                .position(|f| f.relation == relation && f.act == act)
                .map(|at| at as u32)
                .expect("frame")
        };
        let turn = words("Call me Ruby Owen.");
        let compiled = |frame, hole_start, hole_len| Decision {
            status: STATUS_COMPILED,
            frame,
            hole_start,
            hole_len,
            survivors: 1,
        };
        let span = SourceSpan { start: 8, end: 17 };
        assert_eq!(
            table.decode(&turn, compiled(find(1, Act::Assert), 2, 2))?,
            CompiledAction::Assert { relation: 1, span }
        );
        assert_eq!(
            table.decode(&turn, compiled(find(4, Act::Update), 2, 2))?,
            CompiledAction::Correct { relation: 4, span }
        );
        assert_eq!(
            table.decode(&turn, compiled(find(7, Act::Query), 0, 0))?,
            CompiledAction::QueryCurrent { relation: 7 }
        );
        for status in [STATUS_NONE, STATUS_DISAGREE] {
            let d = Decision {
                status,
                ..compiled(0, 0, 0)
            };
            assert!(matches!(
                table.decode(&turn, d)?,
                CompiledAction::Unresolved { .. }
            ));
        }
        let bad = [
            compiled(find(1, Act::Assert), 2, 0),
            compiled(find(1, Act::Assert), 3, 2),
            compiled(u32::MAX, 0, 1),
            Decision {
                status: 9,
                ..compiled(0, 0, 1)
            },
        ];
        for d in bad {
            assert!(table.decode(&turn, d).is_err(), "{d:?}");
        }
        Ok(())
    }

    #[test]
    fn the_registered_controls_carry_exact_spans_and_fit_the_slot() -> Result<()> {
        for split in [Split::Train, Split::Development] {
            let controls =
                world_v2_controls(split, REGISTERED_VALUES_PER_PHRASING, REGISTERED_SEED)?;
            assert!(!controls.is_empty());
            for control in &controls {
                if let CompiledAction::Assert { span, .. } | CompiledAction::Correct { span, .. } =
                    control.expected
                {
                    let value = &control.source[span.start..span.end];
                    let n = words(value).len() as u32;
                    assert!((1..=MAX_VALUE_WORDS).contains(&n), "{}", control.source);
                    let first = words(value).first().map(|w| w.start);
                    assert_eq!(first, Some(0), "value must start on a word: {value:?}");
                    let last = words(value).last().map(|w| w.end);
                    assert_eq!(
                        last,
                        Some(value.len()),
                        "value must end on a word: {value:?}"
                    );
                }
            }
            assert_eq!(
                world_v2_controls(split, REGISTERED_VALUES_PER_PHRASING, REGISTERED_SEED)?,
                controls
            );
        }
        Ok(())
    }

    #[test]
    fn the_decided_compiler_answers_only_from_device_decisions() -> Result<()> {
        let table = FrameTable::world_v2()?;
        let mut actions = BTreeMap::new();
        actions.insert(
            "Hello there.".to_string(),
            CompiledAction::Unresolved { reason: "x".into() },
        );
        let compiler = DecidedCompiler::new(&table, actions)?;
        assert!(compiler.compile("Hello there.").is_ok());
        assert!(compiler.compile("Something else.").is_err());
        assert_eq!(compiler.identity().label_schema, SIEVE_LABEL_SCHEMA);
        assert_eq!(
            compiler.artifact_bytes(),
            table.canonical_bytes()?.as_slice()
        );
        Ok(())
    }
}
