//! Dialogue serving of a `UORLUT01` stack artifact (`uor-r4.lut-stack/1`)
//! through the D11 integer engine, under the literal-role dialogue protocol.
//!
//! This is the chat CLI's path to the geometric chat stack: the artifact
//! `geometric-stack export` writes is executed directly by
//! [`IntegerStackModel`], position by position, and each reply takes the
//! highest served score (ties to the lower id) until EOS, the historical
//! short terminal cycle (a cycle of length 1..4 repeated three times) or the
//! caller's token cap — the float stack's `greedy_reply` rule. Nothing here is
//! converted into the retained recurrent bundle: the two containers describe
//! different models, so a repackaging of stack bytes as an
//! [`crate::model::IntegerModel`] cannot exist (see [`crate::bundle`]).
//!
//! The served arithmetic is the engine's; this module only encodes and decodes
//! text, tracks the token history, and copies score slices. A model whose
//! artifact carries a pointer head is served through its mixture, because
//! [`IntegerStackSession::next_token_scores`] is that mixture.
//!
//! No floating point and no multiplier instruction is introduced: selection is
//! [`super::stack_argmax`] over the engine's `i32` scores.

use std::fmt;
use std::path::Path;

use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;

use super::{stack_argmax, IntegerStackModel, IntegerStackSession, StackError};

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
    /// A supplied history was empty: the engine has no scores to continue.
    EmptyHistory,
    /// The engine's score vector is not one score per vocabulary entry.
    ScoreWidth { found: usize, vocab: usize },
    /// The message had no user turn, or carried literal special tokens.
    NotAUserTurn,
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
            Self::EmptyHistory => formatter.write_str("an empty token history has no scores"),
            Self::ScoreWidth { found, vocab } => write!(
                formatter,
                "the engine returned {found} scores for a vocabulary of {vocab}"
            ),
            Self::NotAUserTurn => formatter.write_str(
                "the message is not one plain user turn (empty, or carrying role markers or special tokens)",
            ),
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
    /// `history[..fed]` has been stepped into `session`; `scores` are the
    /// scores at position `fed`.
    fed: usize,
    /// The served scores (`next_token_scores`: the pointer mixture when the
    /// artifact has a pointer head).
    scores: Vec<i32>,
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
            scores: Vec::new(),
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

    /// Forget the conversation: a fresh engine session and an empty history.
    pub fn reset(&mut self) {
        self.session.reset();
        self.history.clear();
        self.fed = 0;
        self.scores.clear();
    }

    /// Start from an explicit history (BOS, optionally a persona turn). The
    /// tokens are stepped now, so the first [`Self::reply`] continues them.
    pub fn seed(&mut self, history: &[u32]) -> Result<(), StackChatError> {
        if history.is_empty() {
            return Err(StackChatError::EmptyHistory);
        }
        self.reset();
        self.feed(history)
    }

    /// One greedy reply to a user message, under the literal-role protocol.
    pub fn reply(
        &mut self,
        user: &str,
        max_new_tokens: usize,
    ) -> Result<StackReply, StackChatError> {
        let has_history = self.history.len() > 1;
        let prefix = self
            .protocol
            .bind(&self.tokenizer)
            .map_err(|error| StackChatError::Protocol(error.to_string()))?
            .encode_user_prefix(user, has_history);
        if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
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
        self.feed(history)?;
        let vocab = self.model.shape().vocab;
        let mut ids = Vec::with_capacity(max_new_tokens);
        let mut stop = StackStop::MaximumNewTokens;
        for _ in 0..max_new_tokens {
            let next = stack_argmax(&self.scores) as u32;
            ids.push(next);
            self.history.push(next);
            if next == self.protocol.eos_id {
                stop = StackStop::Eos;
                break;
            }
            if let Some(period) = crate::generation::short_cycle(&ids) {
                stop = StackStop::ShortCycle { period };
                break;
            }
            let scores = self.session.step(next)?;
            if scores.len() != vocab {
                return Err(StackChatError::ScoreWidth {
                    found: scores.len(),
                    vocab,
                });
            }
            self.scores = scores.to_vec();
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
    fn feed(&mut self, history: &[u32]) -> Result<(), StackChatError> {
        if !history.starts_with(&self.history[..]) {
            self.reset();
        }
        let vocab = self.model.shape().vocab;
        for index in self.fed..history.len() {
            let token = history[index];
            let scores = self.session.step(token)?;
            if scores.len() != vocab {
                return Err(StackChatError::ScoreWidth {
                    found: scores.len(),
                    vocab,
                });
            }
            self.scores = scores.to_vec();
            self.history.push(token);
            self.fed = self.history.len();
        }
        if self.scores.is_empty() {
            return Err(StackChatError::EmptyHistory);
        }
        Ok(())
    }
}
