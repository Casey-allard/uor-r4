//! Dialogue serving of a `UORLUT01` stack artifact through the D11 integer
//! engine, under the literal-role dialogue protocol.
//!
//! This is the chat CLI's path to the geometric chat stack: the artifact is
//! executed directly by [`IntegerStackModel`], position by position, and each
//! reply takes the token [`IntegerStackSession::serve_step`] serves until EOS,
//! the historical short terminal cycle (a cycle of length 1..4 repeated three
//! times) or the caller's token cap — the float stack's `greedy_reply` rule.
//! - **A dense stack** (`uor-r4.lut-stack/1`, `geometric-stack export`): the
//!   highest score of the step's logits, ties to the lower id
//!   ([`super::stack_argmax`]).
//! - **A SpiralCore/FBS model** (`fbs-export`): its native head's own pick, the
//!   whole table's nearest under `head_nearest`. Its step has no logits.
//!
//! A model with a turn memory has its reply start marked before the prompt's
//! last token is stepped, where `fbs-toks` and training's reply rows place it.
//! The mark is cleared while a later turn's prompt is stepped, so no prompt
//! row takes a reply-table move.
//!
//! Nothing here is converted into the retained recurrent bundle: the two
//! containers describe different models, so a repackaging of stack bytes as an
//! [`crate::model::IntegerModel`] cannot exist (see [`crate::bundle`]).
//!
//! The served arithmetic is the engine's; this module only encodes and decodes
//! text, tracks the token history and the served pick, and places the turn
//! memory's mark. No floating point and no multiplier instruction is
//! introduced.

use std::fmt;
use std::path::Path;

use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use super::{IntegerStackModel, IntegerStackSession, StackError};

/// The context the chat profile declares: the sealed bundle contract's 256.
/// An artifact may declare another context; [`StackChat`] serves the artifact's
/// own declaration and never truncates a turn silently.
pub const CHAT_CONTEXT: usize = 256;
/// Terminal cycles of length 1..=4 repeated this many times stop a reply
/// (the float stack's historical rule).
pub const CYCLE_REPEATS: usize = 3;

/// A refused chat operation. Every variant is a boundary condition with a
/// message; nothing here panics on reachable input.
#[derive(Debug)]
pub enum StackChatError {
    /// The engine refused a step, a token or a state.
    Stack(StackError),
    /// `tokenizer.json` did not parse as a byte-level BPE tokenizer.
    Tokenizer,
    /// The literal-role protocol could not be built or bound to the tokenizer.
    Protocol(String),
    /// The tokenizer's vocabulary and the artifact's head disagree.
    Vocabulary { tokenizer: usize, model: usize },
    /// A supplied history was empty: the engine has served no pick to continue.
    EmptyHistory,
    /// The message had no user turn, or carried literal special tokens.
    NotAUserTurn,
    /// A system-memory value was empty, multiline or otherwise not a plain value.
    InvalidMemory(String),
    /// The turn does not fit the artifact's declared context. The caller may
    /// reset the conversation; this path never truncates.
    Context { needed: usize, context: usize },
}

impl fmt::Display for StackChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stack(error) => write!(formatter, "{error}"),
            Self::Tokenizer => formatter.write_str("unreadable tokenizer.json"),
            Self::Protocol(reason) => write!(formatter, "dialogue protocol: {reason}"),
            Self::Vocabulary { tokenizer, model } => write!(
                formatter,
                "tokenizer vocabulary {tokenizer} differs from the artifact vocabulary {model}"
            ),
            Self::EmptyHistory => formatter.write_str("an empty token history has no served pick"),
            Self::NotAUserTurn => formatter.write_str(
                "the message is not one plain user turn (empty, or carrying role markers or special tokens)",
            ),
            Self::InvalidMemory(reason) => {
                write!(formatter, "invalid system-memory value: {reason}")
            }
            Self::Context { needed, context } => write!(
                formatter,
                "the turn needs {needed} positions of a {context}-position context; reset the \
                 conversation or shorten the turn"
            ),
        }
    }
}

impl std::error::Error for StackChatError {}

impl From<StackError> for StackChatError {
    fn from(error: StackError) -> Self {
        Self::Stack(error)
    }
}

/// How a reply ended: the model's EOS, the historical short terminal cycle, or
/// the caller's token cap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackStop {
    Eos,
    ShortCycle { period: usize },
    MaximumNewTokens,
}

impl StackStop {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Eos => "eos",
            Self::ShortCycle { .. } => "short_cycle",
            Self::MaximumNewTokens => "max_new_tokens",
        }
    }
}

/// One greedy reply. `ids` are the generated ids with the terminal token
/// included, exactly as the float stack's reply record keeps them; `text` is
/// the decoded reply without a terminal EOS.
#[derive(Clone, Debug)]
pub struct StackReply {
    pub ids: Vec<u32>,
    pub text: String,
    pub stop: StackStop,
}

/// A persistent conversation over one [`IntegerStackModel`]: the engine session,
/// the exact token history and the literal-role protocol.
pub struct StackChat<'m> {
    model: &'m IntegerStackModel,
    tokenizer: ByteBpeTokenizer,
    protocol: DialogueProtocol,
    session: IntegerStackSession<'m>,
    /// Exact token history, including ids generated but not stepped yet.
    history: Vec<u32>,
    /// `history[..fed]` has been stepped into `session`; `pick` is the token
    /// [`IntegerStackSession::serve_step`] served at position `fed`.
    fed: usize,
    pick: Option<u32>,
}

impl<'m> StackChat<'m> {
    /// Bind an artifact, a parsed tokenizer and a protocol. The protocol is
    /// bound here, so a tokenizer identity or special-id mismatch is refused
    /// before any step.
    pub fn new(
        model: &'m IntegerStackModel,
        tokenizer: ByteBpeTokenizer,
        protocol: DialogueProtocol,
    ) -> Result<Self, StackChatError> {
        let vocab = model.shape().vocab;
        if tokenizer.vocab_size() != vocab {
            return Err(StackChatError::Vocabulary {
                tokenizer: tokenizer.vocab_size(),
                model: vocab,
            });
        }
        protocol
            .bind(&tokenizer)
            .map_err(|error| StackChatError::Protocol(error.to_string()))?;
        Ok(Self {
            model,
            tokenizer,
            protocol,
            session: model.session(),
            history: Vec::new(),
            fed: 0,
            pick: None,
        })
    }

    /// [`Self::new`] from `tokenizer.json` bytes and a literal-role version.
    pub fn from_tokenizer_json(
        model: &'m IntegerStackModel,
        json: &[u8],
        version: u8,
    ) -> Result<Self, StackChatError> {
        let tokenizer =
            ByteBpeTokenizer::from_tokenizer_json_bytes(json).ok_or(StackChatError::Tokenizer)?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)
            .map_err(|error| StackChatError::Protocol(error.to_string()))?;
        Self::new(model, tokenizer, protocol)
    }

    /// [`Self::from_tokenizer_json`] reading the tokenizer from `path`.
    pub fn from_tokenizer_path(
        model: &'m IntegerStackModel,
        path: &Path,
        version: u8,
    ) -> Result<Self, StackChatError> {
        let bytes = std::fs::read(path).map_err(|_| StackChatError::Tokenizer)?;
        Self::from_tokenizer_json(model, &bytes, version)
    }

    pub fn protocol(&self) -> &DialogueProtocol {
        &self.protocol
    }

    pub fn tokenizer(&self) -> &ByteBpeTokenizer {
        &self.tokenizer
    }

    pub fn model(&self) -> &'m IntegerStackModel {
        self.model
    }

    /// The artifact's declared context: the positions a conversation serves.
    pub fn context(&self) -> usize {
        self.model.shape().context
    }

    /// The exact token history fed so far (including a generated id not yet
    /// stepped into the session).
    pub fn tokens(&self) -> &[u32] {
        &self.history
    }

    /// Finished steps of the current session (diagnostic).
    pub fn position(&self) -> usize {
        self.session.position()
    }

    /// Whether the artifact's declared context is the chat profile's 256.
    pub fn declares_chat_context(&self) -> bool {
        self.context() == CHAT_CONTEXT
    }

    /// Forget the conversation: a fresh engine session, no reply mark and an
    /// empty history. (The session's own reset keeps the turn memory's mark.)
    pub fn reset(&mut self) {
        self.session.reset();
        self.session.clear_reply_start();
        self.history.clear();
        self.fed = 0;
        self.pick = None;
    }

    /// Start from an explicit history (BOS, optionally a persona turn). The
    /// tokens are stepped now, so the first [`Self::reply`] continues them.
    pub fn seed(&mut self, history: &[u32]) -> Result<(), StackChatError> {
        if history.is_empty() {
            return Err(StackChatError::EmptyHistory);
        }
        self.reset();
        self.feed(history, false)
    }

    /// The exact system turn used for a recall value.
    pub fn memory_content(value: &str) -> Result<String, StackChatError> {
        if value.is_empty() {
            return Err(StackChatError::InvalidMemory("value is empty".into()));
        }
        if value.contains(['\r', '\n']) {
            return Err(StackChatError::InvalidMemory(
                "value contains a line break".into(),
            ));
        }
        Ok(format!("Memory: {value}."))
    }

    /// One greedy reply to a user message, under the literal-role protocol.
    pub fn reply(
        &mut self,
        user: &str,
        max_new_tokens: usize,
    ) -> Result<StackReply, StackChatError> {
        self.reply_with_memory(user, None, max_new_tokens)
    }

    /// One greedy reply to a user message with an optional system-memory turn.
    /// The memory value is encoded as `System: Memory: {value}.` between the
    /// user turn and the assistant marker.
    pub fn reply_with_memory(
        &mut self,
        user: &str,
        memory: Option<&str>,
        max_new_tokens: usize,
    ) -> Result<StackReply, StackChatError> {
        let has_history = self.history.len() > 1;
        let memory = memory.map(Self::memory_content).transpose()?;
        let mut messages = Vec::with_capacity(2);
        messages.push(Message {
            role: "user",
            content: user,
        });
        if let Some(memory) = memory.as_deref() {
            messages.push(Message {
                role: "system",
                content: memory,
            });
        }
        let prefix = self
            .protocol
            .bind(&self.tokenizer)
            .map_err(|error| StackChatError::Protocol(error.to_string()))?
            .encode_assistant_suffix(&messages, has_history);
        if prefix.emitted_turns != messages.len() || prefix.special_token_occurrences != 0 {
            return Err(StackChatError::NotAUserTurn);
        }
        let mut history = self.history.clone();
        history.extend_from_slice(&prefix.tokens);
        self.reply_history(&history, max_new_tokens)
    }

    /// One greedy reply to a complete token history (BOS first, the assistant
    /// marker last). The history must extend the current conversation, or the
    /// session starts over. The turn is refused, never truncated, when it does
    /// not fit the artifact's declared context.
    pub fn reply_history(
        &mut self,
        history: &[u32],
        max_new_tokens: usize,
    ) -> Result<StackReply, StackChatError> {
        self.reply_history_with(history, max_new_tokens, |_, _| Ok(()))
    }

    /// The same greedy reply, with every generated ID and its tokenizer bytes
    /// emitted after the ID enters history and before another model step. An
    /// emission error aborts the loop and is returned unchanged.
    pub fn reply_history_with<F>(
        &mut self,
        history: &[u32],
        max_new_tokens: usize,
        mut emit: F,
    ) -> Result<StackReply, StackChatError>
    where
        F: FnMut(u32, &[u8]) -> Result<(), StackChatError>,
    {
        if history.is_empty() {
            return Err(StackChatError::EmptyHistory);
        }
        let needed = history
            .len()
            .checked_add(max_new_tokens)
            .ok_or(StackChatError::Context {
                needed: usize::MAX,
                context: self.context(),
            })?;
        if needed > self.context() {
            return Err(StackChatError::Context {
                needed,
                context: self.context(),
            });
        }
        self.feed(history, true)?;
        let mut ids = Vec::with_capacity(max_new_tokens);
        let mut stop = StackStop::MaximumNewTokens;
        for _ in 0..max_new_tokens {
            let next = self.pick.ok_or(StackChatError::EmptyHistory)?;
            ids.push(next);
            self.history.push(next);
            let token_bytes = self.tokenizer.decode_bytes(&[next]);
            emit(next, &token_bytes)?;
            if next == self.protocol.eos_id {
                stop = StackStop::Eos;
                break;
            }
            if let Some(period) = crate::generation::short_cycle(&ids) {
                stop = StackStop::ShortCycle { period };
                break;
            }
            self.pick = Some(self.session.serve_step(next)?.0);
            self.fed = self.history.len();
        }
        let words: Vec<u32> = ids
            .iter()
            .copied()
            .filter(|&id| id != self.protocol.eos_id)
            .collect();
        let decoded = self.tokenizer.decode(&words);
        Ok(StackReply {
            ids,
            text: self.protocol.reply_text(&decoded).to_owned(),
            stop,
        })
    }

    /// Make `history` the conversation's history: start over unless it extends
    /// what is already there, then step every token the session has not seen
    /// (including a generated id left pending by the previous reply).
    ///
    /// With `prompt`, the history's last token is the prompt's last: the turn
    /// memory's mark is cleared while the turn's other new tokens are stepped,
    /// and set before that last token, so the reply's rows are the only reply
    /// rows. A model with a turn memory whose prompt was already stepped
    /// unmarked (a seed) steps the history again, so its opening row is keyed.
    fn feed(&mut self, history: &[u32], prompt: bool) -> Result<(), StackChatError> {
        if !history.starts_with(&self.history[..]) {
            self.reset();
        }
        if prompt && self.fed == history.len() && self.session.has_turn_memory() {
            self.reset();
        }
        if prompt {
            self.session.clear_reply_start();
        }
        for index in self.fed..history.len() {
            let token = history[index];
            if prompt && index + 1 == history.len() {
                self.session.mark_reply_start();
            }
            self.pick = Some(self.session.serve_step(token)?.0);
            // A pending generated id is already in the history: keep one copy.
            if index == self.history.len() {
                self.history.push(token);
            }
            self.fed = index + 1;
        }
        if self.pick.is_none() {
            return Err(StackChatError::EmptyHistory);
        }
        Ok(())
    }
}
