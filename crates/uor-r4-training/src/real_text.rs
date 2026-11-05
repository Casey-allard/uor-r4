//! Real human text for P5: OASST2 conversation trees and Wikipedia articles
//! become dialogue-protocol token documents, split by source id and screened
//! against the frozen test panels. CPU data preparation only; the bin
//! `prepare-real-text` streams the inputs and writes the corpora.
//!
//! The panel exclusion rule (ported from the compiled-out `knowledge_corpus`)
//! splits text into words with [`crate::milestone_world`]'s word split:
//! lowercased, punctuation removed, apostrophes kept inside words. A document
//! is dropped whole when any of its [`PANEL_NGRAM`]-word windows equals a
//! window of any panel user turn. Panel turns shorter than [`PANEL_NGRAM`]
//! words cannot produce a window; for those with at least
//! [`MIN_SHORT_REQUEST_WORDS`] words the whole turn's word sequence is
//! excluded instead, so short requests such as "Why do leaves change color in
//! the fall" are not silently uncovered. Turns with fewer words are listed as
//! uncovered rather than matched, because a two- or three-word sequence
//! ("hi dr smith") is not evidence of contamination.
//!
//! Matching is exact: sequences are hashed to `u64` for speed and every
//! candidate hit is confirmed against the stored word sequence, so a hash
//! collision can never drop a document.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueError, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::milestone_world::words;

/// SHA-256 of the pinned tokenizer.json, copied from `PINNED_TOKENIZER_SHA256`
/// in `crates/uor-r4-float-train/src/hamilton_sheet.rs` (this crate does not
/// depend on the float crate).
pub const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";

/// Word count of the panel exclusion window.
pub const PANEL_NGRAM: usize = 8;

/// Fewest words a short panel turn needs to be matched as a whole sequence.
pub const MIN_SHORT_REQUEST_WORDS: usize = 4;

/// The only OASST2 tree state that is read.
pub const OASST_READY_STATE: &str = "ready_for_export";

/// The split rule, as recorded in every manifest.
pub const SPLIT_RULE: &str = "a document is held out iff \
    u64::from_be_bytes(sha256(utf8(source_id))[..8]) % 1000 < heldout_per_mille; the side \
    depends only on the source id (OASST2 message_tree_id, Wikipedia article id)";

/// How a Wikipedia article becomes a document, as recorded in its manifest.
pub const WIKI_CONTENT_CONVENTION: &str = "<|bos|> (mask 0), then the tokens of \
    clean_article(title, text) with \\r\\n/\\r normalised to \\n and both ends trimmed; after \
    <|bos|>: one space then the cleaned title and body under dialogue protocol v2 (dialogue.rs \
    SCHEMA_V2, content carries its leading space), none under v1; then <|eos|>; the content \
    tokens and <|eos|> have mask 1; the round-trip check compares that segment, space included";

/// How an OASST2 thread becomes a document, as recorded in its manifest.
pub const OASST_CONTENT_CONVENTION: &str = "DialogueEncoder::encode_document over the thread: \
    <|bos|> (mask 0), then turns joined by a single '\\n' separator placed before every turn \
    after the first, so the first marker follows <|bos|> with no space and no separator; a turn \
    is the role marker then its content (\\r\\n/\\r normalised to \\n, ends trimmed; protocol 2 \
    puts the separating space in the content); prompter maps to 'user', assistant to \
    'assistant'; each assistant content and its closing <|eos|> have mask 1, everything else \
    mask 0; a thread ending on a user turn gets an unmasked terminal <|eos|>";

/// The OASST2 node filter, in the order the first failing reason is taken.
pub const NODE_FILTER_RULE: &str = "a message is dropped when deleted == true, else when \
    review_result != true (false or null), else when synthetic == true, else when lang != \"en\"";

/// The reply walk, as recorded in the manifest.
pub const WALK_RULE: &str = "from the root, take the best remaining child at each node: \
    lowest rank first, a null rank after every ranked child, ties by message_id ascending; \
    stop when no child passes the node filter; prompter maps to user; the thread is kept only \
    if its first assistant turn has a user turn before it";

#[derive(Debug)]
pub enum RealTextError {
    Json(serde_json::Error),
    Panel(String),
    TokenizerSha {
        expected: String,
        actual: String,
    },
    Dialogue(DialogueError),
    Input(String),
    /// A token id above u16 or a token/mask length mismatch: a defect that
    /// stops the run instead of being counted as a drop.
    Defect(String),
}

impl fmt::Display for RealTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Panel(m) => write!(f, "panel: {m}"),
            Self::TokenizerSha { expected, actual } => write!(
                f,
                "tokenizer sha256 {actual} is not the pinned {expected}; refusing to write"
            ),
            Self::Dialogue(e) => write!(f, "dialogue protocol: {e}"),
            Self::Input(m) => write!(f, "input: {m}"),
            Self::Defect(m) => write!(f, "encoder defect: {m}"),
        }
    }
}

impl std::error::Error for RealTextError {}

impl From<serde_json::Error> for RealTextError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<DialogueError> for RealTextError {
    fn from(e: DialogueError) -> Self {
        Self::Dialogue(e)
    }
}

/// Refuse tokenizer bytes whose SHA-256 is not `expected_hex`.
pub fn check_tokenizer_sha(bytes: &[u8], expected_hex: &str) -> Result<(), RealTextError> {
    let actual = hex::encode(Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(RealTextError::TokenizerSha {
            expected: expected_hex.to_owned(),
            actual,
        })
    }
}

/// `u64::from_be_bytes(sha256(source_id)[..8]) % 1000`.
pub fn split_residue(source_id: &str) -> u64 {
    let digest = Sha256::digest(source_id.as_bytes());
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(head) % 1000
}

/// Whether `source_id` is on the held-out side ([`SPLIT_RULE`]).
pub fn heldout(source_id: &str, heldout_per_mille: u64) -> bool {
    split_residue(source_id) < heldout_per_mille
}

/// Content normalised as `DialogueEncoder` does: `\r\n`/`\r` to `\n`, ends trimmed.
pub fn normalize_content(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_owned()
}

// ---------------------------------------------------------------- panels

/// One panel request: its id and user turns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelRequest {
    pub id: String,
    pub user_turns: Vec<String>,
}

/// The requests of one panel file (a JSON list of `{id, user_turns: [string]}`).
pub fn parse_panel(bytes: &[u8]) -> Result<Vec<PanelRequest>, RealTextError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let rows = value
        .as_array()
        .ok_or_else(|| RealTextError::Panel("not a JSON list".into()))?;
    if rows.is_empty() {
        return Err(RealTextError::Panel("panel has no requests".into()));
    }
    let requests = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let id = row["id"]
                .as_str()
                .ok_or_else(|| RealTextError::Panel(format!("row {index} has no string id")))?;
            let turns = row["user_turns"]
                .as_array()
                .ok_or_else(|| RealTextError::Panel(format!("row {id} has no user_turns list")))?;
            let user_turns = turns
                .iter()
                .map(|t| {
                    t.as_str().map(str::to_owned).ok_or_else(|| {
                        RealTextError::Panel(format!("row {id} has a non-string user turn"))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            if user_turns.is_empty() {
                return Err(RealTextError::Panel(format!("row {id} has no user turn")));
            }
            Ok(PanelRequest {
                id: id.to_owned(),
                user_turns,
            })
        })
        .collect::<Result<Vec<_>, RealTextError>>()?;
    let mut seen = std::collections::HashSet::new();
    for request in &requests {
        if !seen.insert(request.id.as_str()) {
            return Err(RealTextError::Panel(format!(
                "duplicate request id {}",
                request.id
            )));
        }
    }
    Ok(requests)
}

fn window_key(window: &[String]) -> u64 {
    // `DefaultHasher::new()` uses fixed keys, so keys are stable within a run;
    // they are never persisted, and every hit is confirmed exactly.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    window.hash(&mut hasher);
    hasher.finish()
}

/// Which exclusion rule matched a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    /// A [`PANEL_NGRAM`]-word window shared with a panel turn.
    Ngram,
    /// The whole word sequence of a short (4-7 word) panel turn.
    ShortRequest,
}

impl HitKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ngram => "panel_8gram",
            Self::ShortRequest => "panel_short_request",
        }
    }
}

/// The first exclusion match in a text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub kind: HitKind,
    /// The matched normalized words, joined by single spaces.
    pub words: String,
    /// The panel request id whose turn supplied the matched sequence.
    pub request_id: String,
}

/// A panel turn too short to be matched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncoveredTurn {
    pub request_id: String,
    pub text: String,
}

type SequenceTable = HashMap<u64, Vec<(Vec<String>, String)>>;

/// Exact word-sequence exclusion built from panel user turns.
#[derive(Clone, Debug)]
pub struct PanelExclusion {
    /// Sequence length -> key -> (sequence, request id); one entry per length
    /// in use: [`PANEL_NGRAM`] and every short-turn length.
    by_len: BTreeMap<usize, SequenceTable>,
    key: fn(&[String]) -> u64,
    /// Panel requests read.
    pub requests: usize,
    /// User turns read.
    pub turns: usize,
    /// Turns with at least [`PANEL_NGRAM`] words (covered by windows).
    pub ngram_turns: usize,
    /// Turns covered by whole-sequence containment.
    pub short_turns: usize,
    /// Turns with fewer than [`MIN_SHORT_REQUEST_WORDS`] words: never matched.
    pub uncovered: Vec<UncoveredTurn>,
    /// Distinct [`PANEL_NGRAM`]-word windows.
    pub ngrams: usize,
}

impl PanelExclusion {
    /// The exclusion of every user turn of `requests`.
    pub fn new(requests: &[PanelRequest]) -> Self {
        Self::with_key(requests, window_key)
    }

    fn with_key(requests: &[PanelRequest], key: fn(&[String]) -> u64) -> Self {
        let mut exclusion = Self {
            by_len: BTreeMap::new(),
            key,
            requests: 0,
            turns: 0,
            ngram_turns: 0,
            short_turns: 0,
            uncovered: Vec::new(),
            ngrams: 0,
        };
        for request in requests {
            exclusion.requests += 1;
            for turn in &request.user_turns {
                exclusion.turns += 1;
                let all = words(turn);
                if all.len() >= PANEL_NGRAM {
                    exclusion.ngram_turns += 1;
                    for window in all.windows(PANEL_NGRAM) {
                        if exclusion.insert(window, &request.id) {
                            exclusion.ngrams += 1;
                        }
                    }
                } else if all.len() >= MIN_SHORT_REQUEST_WORDS {
                    exclusion.short_turns += 1;
                    exclusion.insert(&all, &request.id);
                } else {
                    exclusion.uncovered.push(UncoveredTurn {
                        request_id: request.id.clone(),
                        text: turn.clone(),
                    });
                }
            }
        }
        exclusion
    }

    /// Insert one sequence; false when it was already present.
    fn insert(&mut self, sequence: &[String], request_id: &str) -> bool {
        let key = (self.key)(sequence);
        let bucket = self
            .by_len
            .entry(sequence.len())
            .or_default()
            .entry(key)
            .or_default();
        if bucket.iter().any(|(seq, _)| seq.as_slice() == sequence) {
            return false;
        }
        bucket.push((sequence.to_vec(), request_id.to_owned()));
        true
    }

    /// The first excluded sequence `text` contains, scanning lengths in
    /// increasing order and positions left to right.
    pub fn first_hit(&self, text: &str) -> Option<Hit> {
        if self.by_len.is_empty() {
            return None;
        }
        let all = words(text);
        for (&len, table) in &self.by_len {
            if all.len() < len {
                continue;
            }
            for window in all.windows(len) {
                let Some(bucket) = table.get(&(self.key)(window)) else {
                    continue;
                };
                if let Some((seq, id)) = bucket.iter().find(|(seq, _)| seq.as_slice() == window) {
                    return Some(Hit {
                        kind: if len == PANEL_NGRAM {
                            HitKind::Ngram
                        } else {
                            HitKind::ShortRequest
                        },
                        words: seq.join(" "),
                        request_id: id.clone(),
                    });
                }
            }
        }
        None
    }
}

/// An article's training text: optional title line, then the body with
/// `\r\n`/`\r` normalised, trailing spaces trimmed per line, runs of blank
/// lines collapsed to one, and the whole trimmed.
pub fn clean_article(title: Option<&str>, body: &str) -> String {
    let body = body.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(body.len() + 64);
    if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
        out.push_str(title);
        out.push_str("\n\n");
    }
    let mut blank = false;
    let mut any = false;
    for line in body.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank = any;
            continue;
        }
        if blank {
            out.push('\n');
        }
        if any {
            out.push('\n');
        }
        out.push_str(line);
        any = true;
        blank = false;
    }
    out.trim().to_owned()
}

/// The number of normalized words in `text`.
pub fn word_count(text: &str) -> usize {
    words(text).len()
}

// ---------------------------------------------------------------- OASST2

/// One OASST2 tree (one line of the `trees.jsonl` export).
#[derive(Clone, Debug, Deserialize)]
pub struct OasstTree {
    pub message_tree_id: String,
    pub tree_state: String,
    pub prompt: OasstNode,
}

/// One OASST2 message and its replies. Other export fields are ignored.
#[derive(Clone, Debug, Deserialize)]
pub struct OasstNode {
    pub message_id: String,
    pub text: String,
    pub role: String,
    pub lang: String,
    pub review_result: Option<bool>,
    pub deleted: bool,
    pub rank: Option<i64>,
    pub synthetic: bool,
    #[serde(default)]
    pub replies: Vec<OasstNode>,
}

/// Why the node filter drops a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeDrop {
    Deleted,
    ReviewNotTrue,
    Synthetic,
    NotEnglish,
}

impl NodeDrop {
    pub const ALL: [Self; 4] = [
        Self::Deleted,
        Self::ReviewNotTrue,
        Self::Synthetic,
        Self::NotEnglish,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Deleted => "deleted",
            Self::ReviewNotTrue => "review_not_true",
            Self::Synthetic => "synthetic",
            Self::NotEnglish => "lang_not_en",
        }
    }
}

/// The node filter ([`NODE_FILTER_RULE`]).
pub fn node_drop(node: &OasstNode) -> Option<NodeDrop> {
    if node.deleted {
        return Some(NodeDrop::Deleted);
    }
    if node.review_result != Some(true) {
        return Some(NodeDrop::ReviewNotTrue);
    }
    if node.synthetic {
        return Some(NodeDrop::Synthetic);
    }
    if node.lang != "en" {
        return Some(NodeDrop::NotEnglish);
    }
    None
}

fn reply_order(node: &OasstNode) -> (bool, i64, &str) {
    (
        node.rank.is_none(),
        node.rank.unwrap_or(0),
        node.message_id.as_str(),
    )
}

/// The best child passing the node filter ([`WALK_RULE`]); every dropped
/// child is counted in `drops` by its [`NodeDrop::label`].
pub fn best_child<'a>(
    children: &'a [OasstNode],
    drops: &mut BTreeMap<&'static str, usize>,
) -> Option<&'a OasstNode> {
    let mut best: Option<&OasstNode> = None;
    for child in children {
        if let Some(reason) = node_drop(child) {
            *drops.entry(reason.label()).or_default() += 1;
            continue;
        }
        if best.is_none_or(|b| reply_order(child) < reply_order(b)) {
            best = Some(child);
        }
    }
    best
}

/// One message of a thread, with its dialogue role.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadMessage {
    pub role: &'static str,
    pub text: String,
}

/// The best-ranked path through one tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thread {
    pub tree_id: String,
    pub messages: Vec<ThreadMessage>,
}

impl Thread {
    /// The first user turn's text, for the exclusion record.
    pub fn first_user_turn(&self) -> &str {
        self.messages
            .iter()
            .find(|m| m.role == "user")
            .map_or("", |m| m.text.as_str())
    }
}

/// What one tree yields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeOutcome {
    Thread(Thread),
    NotReady,
    RootDropped(NodeDrop),
    NoAssistantReply,
}

fn dialogue_role(role: &str) -> Result<&'static str, RealTextError> {
    match role {
        "prompter" => Ok("user"),
        "assistant" => Ok("assistant"),
        other => Err(RealTextError::Input(format!(
            "unknown OASST2 role {other:?}"
        ))),
    }
}

/// Walk one tree; dropped children along the path are counted in `child_drops`.
pub fn walk_tree(
    tree: &OasstTree,
    child_drops: &mut BTreeMap<&'static str, usize>,
) -> Result<TreeOutcome, RealTextError> {
    if tree.tree_state != OASST_READY_STATE {
        return Ok(TreeOutcome::NotReady);
    }
    if let Some(reason) = node_drop(&tree.prompt) {
        return Ok(TreeOutcome::RootDropped(reason));
    }
    let mut path = vec![&tree.prompt];
    let mut node = &tree.prompt;
    while let Some(child) = best_child(&node.replies, child_drops) {
        path.push(child);
        node = child;
    }
    let messages = path
        .iter()
        .map(|node| {
            Ok(ThreadMessage {
                role: dialogue_role(&node.role)?,
                text: node.text.clone(),
            })
        })
        .collect::<Result<Vec<_>, RealTextError>>()?;
    // The builder applies the same rule again to the normalized contents.
    let replied = messages
        .iter()
        .position(|m| m.role == "assistant")
        .is_some_and(|a| messages[..a].iter().any(|m| m.role == "user"));
    if !replied {
        return Ok(TreeOutcome::NoAssistantReply);
    }
    Ok(TreeOutcome::Thread(Thread {
        tree_id: tree.message_tree_id.clone(),
        messages,
    }))
}

// ---------------------------------------------------------------- documents

/// Why an otherwise readable document is not written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocDrop {
    /// No non-empty content after normalisation.
    Empty,
    /// No assistant content survived normalisation, or the first one has no
    /// non-empty user turn before it.
    NoAssistantReply,
    /// Content holds literal `<|bos|>`/`<|eos|>`/`<|unk|>` text.
    SpecialText,
    /// A content's tokens do not decode back to exactly that content.
    RoundTrip,
    /// More tokens than `max_tokens`.
    TooLong,
}

impl DocDrop {
    pub const ALL: [Self; 5] = [
        Self::Empty,
        Self::NoAssistantReply,
        Self::SpecialText,
        Self::RoundTrip,
        Self::TooLong,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::NoAssistantReply => "no_assistant_reply",
            Self::SpecialText => "special_text",
            Self::RoundTrip => "round_trip",
            Self::TooLong => "too_long",
        }
    }
}

/// One written document: parallel tokens and mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub tokens: Vec<u16>,
    pub mask: Vec<u8>,
    /// UTF-8 bytes of the normalised content (markers and specials excluded).
    pub content_bytes: usize,
}

/// What a thread or article becomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Built {
    Kept(Document),
    Excluded(Hit),
    Dropped(DocDrop),
}

/// Encodes threads and articles and applies the exclusion, round-trip and
/// length rules. Checks run in this order. Article: empty, exclusion, special
/// text, round trip, length. Thread: exclusion, empty, no assistant reply
/// (including a first assistant reply with no non-empty user turn before it),
/// special text, round trip, length.
pub struct DocumentBuilder<'a> {
    tokenizer: &'a ByteBpeTokenizer,
    protocol: &'a DialogueProtocol,
    encoder: DialogueEncoder<'a>,
    exclusion: &'a PanelExclusion,
    max_tokens: Option<usize>,
}

impl<'a> DocumentBuilder<'a> {
    pub fn new(
        tokenizer: &'a ByteBpeTokenizer,
        protocol: &'a DialogueProtocol,
        exclusion: &'a PanelExclusion,
        max_tokens: Option<usize>,
    ) -> Result<Self, RealTextError> {
        Ok(Self {
            tokenizer,
            protocol,
            encoder: protocol.bind(tokenizer)?,
            exclusion,
            max_tokens,
        })
    }

    fn round_trips(&self, segment: &str) -> bool {
        self.tokenizer
            .decode_bytes(&self.tokenizer.encode(segment))
            .as_slice()
            == segment.as_bytes()
    }

    fn finish(
        &self,
        tokens: &[u32],
        mask: Vec<u8>,
        content_bytes: usize,
    ) -> Result<Built, RealTextError> {
        let tokens = tokens
            .iter()
            .map(|&t| u16::try_from(t))
            .collect::<Result<Vec<u16>, _>>()
            .map_err(|_| RealTextError::Defect("a token id above u16".into()))?;
        if self.max_tokens.is_some_and(|limit| tokens.len() > limit) {
            return Ok(Built::Dropped(DocDrop::TooLong));
        }
        if tokens.len() != mask.len() {
            return Err(RealTextError::Defect("a token/mask length mismatch".into()));
        }
        Ok(Built::Kept(Document {
            tokens,
            mask,
            content_bytes,
        }))
    }

    /// One OASST2 thread. Each message's content is checked separately
    /// against the panel; any hit drops the whole thread.
    pub fn thread(&self, thread: &Thread) -> Result<Built, RealTextError> {
        let contents: Vec<(&str, String)> = thread
            .messages
            .iter()
            .map(|m| (m.role, normalize_content(&m.text)))
            .collect();
        for (_, content) in &contents {
            if let Some(hit) = self.exclusion.first_hit(content) {
                return Ok(Built::Excluded(hit));
            }
        }
        // The encoder skips blank turns, so admission holds on the normalized
        // contents: the first non-empty assistant reply needs a non-empty user
        // turn before it.
        if let Some(first) = contents
            .iter()
            .position(|(role, content)| *role == "assistant" && !content.is_empty())
        {
            let asked = contents[..first]
                .iter()
                .any(|(role, content)| *role == "user" && !content.is_empty());
            if !asked {
                return Ok(Built::Dropped(DocDrop::NoAssistantReply));
            }
        }
        let messages: Vec<Message<'_>> = contents
            .iter()
            .map(|(role, content)| Message {
                role,
                content: content.as_str(),
            })
            .collect();
        let encoded = self.encoder.encode_document(&messages);
        if encoded.emitted_turns == 0 {
            return Ok(Built::Dropped(DocDrop::Empty));
        }
        if !encoded.response_mask.contains(&1) {
            return Ok(Built::Dropped(DocDrop::NoAssistantReply));
        }
        if encoded.special_token_occurrences > 0 {
            return Ok(Built::Dropped(DocDrop::SpecialText));
        }
        let spaced = self.protocol.spaced_content();
        for (_, content) in contents.iter().filter(|(_, c)| !c.is_empty()) {
            // The encoder encodes each content as its own segment; protocol 2
            // prefixes the separating space.
            let segment = if spaced {
                format!(" {content}")
            } else {
                content.clone()
            };
            if !self.round_trips(&segment) {
                return Ok(Built::Dropped(DocDrop::RoundTrip));
            }
        }
        let content_bytes = contents.iter().map(|(_, c)| c.len()).sum();
        self.finish(&encoded.tokens, encoded.response_mask, content_bytes)
    }

    /// One Wikipedia article ([`WIKI_CONTENT_CONVENTION`]).
    pub fn article(&self, title: Option<&str>, body: &str) -> Result<Built, RealTextError> {
        let text = normalize_content(&clean_article(title, body));
        if text.is_empty() {
            return Ok(Built::Dropped(DocDrop::Empty));
        }
        if let Some(hit) = self.exclusion.first_hit(&text) {
            return Ok(Built::Excluded(hit));
        }
        // Under protocol 2 content carries the space after its marker, so the
        // article's first word is encoded as it is mid-sentence.
        let segment = if self.protocol.spaced_content() {
            format!(" {text}")
        } else {
            text.clone()
        };
        let ids = self.tokenizer.encode(&segment);
        let specials = [
            self.protocol.bos_id,
            self.protocol.eos_id,
            self.protocol.unk_id,
        ];
        if ids.iter().any(|id| specials.contains(id)) {
            return Ok(Built::Dropped(DocDrop::SpecialText));
        }
        if self.tokenizer.decode_bytes(&ids).as_slice() != segment.as_bytes() {
            return Ok(Built::Dropped(DocDrop::RoundTrip));
        }
        let mut tokens = Vec::with_capacity(ids.len() + 2);
        tokens.push(self.protocol.bos_id);
        tokens.extend_from_slice(&ids);
        tokens.push(self.protocol.eos_id);
        let mut mask = vec![1u8; tokens.len()];
        mask[0] = 0;
        self.finish(&tokens, mask, text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: &str, turns: &[&str]) -> PanelRequest {
        PanelRequest {
            id: id.into(),
            user_turns: turns.iter().map(|t| (*t).to_owned()).collect(),
        }
    }

    // ------------------------------------------------ tokenizer fixture

    /// GPT-2's byte-to-unicode table (the tokenizer crate keeps its own private).
    fn byte_alphabet() -> Vec<char> {
        let printable = |b: u32| {
            (0x21..=0x7E).contains(&b) || (0xA1..=0xAC).contains(&b) || (0xAE..=0xFF).contains(&b)
        };
        let mut extra = 0;
        (0..256u32)
            .map(|b| {
                if printable(b) {
                    char::from_u32(b).unwrap()
                } else {
                    extra += 1;
                    char::from_u32(255 + extra).unwrap()
                }
            })
            .collect()
    }

    /// Specials at ids 0-2 and every byte at 3.., except `omit`. Ids stay a
    /// dense prefix (the tokenizer refuses gaps), so bytes after `omit` shift
    /// down by one; the encoder skips the missing byte, so text holding it
    /// cannot round-trip.
    fn tokenizer_json(omit: Option<u8>) -> String {
        let mut vocab = serde_json::Map::new();
        for (id, s) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*s).to_owned(), serde_json::json!(id));
        }
        let mut next = 3usize;
        for (byte, ch) in byte_alphabet().iter().enumerate() {
            if omit != Some(byte as u8) {
                vocab.insert(ch.to_string(), serde_json::json!(next));
                next += 1;
            }
        }
        serde_json::json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ],
            "model": {"type": "BPE", "vocab": vocab, "merges": []}
        })
        .to_string()
    }

    fn tokenizer(omit: Option<u8>) -> ByteBpeTokenizer {
        ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_json(omit).as_bytes()).unwrap()
    }

    // ------------------------------------------------ OASST fixtures

    fn node(id: &str, role: &str, rank: Option<i64>, replies: Vec<OasstNode>) -> OasstNode {
        OasstNode {
            message_id: id.into(),
            text: format!("text of {id}"),
            role: role.into(),
            lang: "en".into(),
            review_result: Some(true),
            deleted: false,
            rank,
            synthetic: false,
            replies,
        }
    }

    fn tree(prompt: OasstNode) -> OasstTree {
        OasstTree {
            message_tree_id: "tree-1".into(),
            tree_state: OASST_READY_STATE.into(),
            prompt,
        }
    }

    fn walk(t: &OasstTree) -> (TreeOutcome, BTreeMap<&'static str, usize>) {
        let mut drops = BTreeMap::new();
        (walk_tree(t, &mut drops).unwrap(), drops)
    }

    fn ids(outcome: &TreeOutcome) -> Vec<String> {
        let TreeOutcome::Thread(thread) = outcome else {
            panic!("expected a thread, got {outcome:?}");
        };
        thread
            .messages
            .iter()
            .map(|m| m.text.trim_start_matches("text of ").to_owned())
            .collect()
    }

    /// A root with a flagged rank-0 reply and a clean rank-1 reply.
    fn flagged_child_tree(flag: impl Fn(&mut OasstNode)) -> OasstTree {
        let mut flagged = node("bad", "assistant", Some(0), vec![]);
        flag(&mut flagged);
        tree(node(
            "root",
            "prompter",
            None,
            vec![flagged, node("good", "assistant", Some(1), vec![])],
        ))
    }

    fn assert_child_dropped(flag: impl Fn(&mut OasstNode), reason: NodeDrop) {
        let (outcome, drops) = walk(&flagged_child_tree(&flag));
        assert_eq!(ids(&outcome), ["root", "good"]);
        assert_eq!(drops.get(reason.label()), Some(&1), "{drops:?}");
        let mut root = flagged_child_tree(|_| {}).prompt;
        flag(&mut root);
        assert_eq!(walk(&tree(root)).0, TreeOutcome::RootDropped(reason));
    }

    #[test]
    fn oasst_filter_drops_deleted() {
        assert_child_dropped(|n| n.deleted = true, NodeDrop::Deleted);
    }

    #[test]
    fn oasst_filter_drops_review_false() {
        assert_child_dropped(|n| n.review_result = Some(false), NodeDrop::ReviewNotTrue);
    }

    #[test]
    fn oasst_filter_drops_review_null() {
        assert_child_dropped(|n| n.review_result = None, NodeDrop::ReviewNotTrue);
    }

    #[test]
    fn oasst_filter_drops_synthetic() {
        assert_child_dropped(|n| n.synthetic = true, NodeDrop::Synthetic);
    }

    #[test]
    fn oasst_filter_drops_non_english() {
        assert_child_dropped(|n| n.lang = "de".into(), NodeDrop::NotEnglish);
    }

    #[test]
    fn oasst_filter_drops_trees_not_ready_for_export() {
        let mut t = flagged_child_tree(|_| {});
        t.tree_state = "growing".into();
        assert_eq!(walk(&t).0, TreeOutcome::NotReady);
    }

    #[test]
    fn oasst_rank_null_comes_after_every_ranked_child() {
        let t = tree(node(
            "root",
            "prompter",
            None,
            vec![
                node("a-null", "assistant", None, vec![]),
                node("c-two", "assistant", Some(2), vec![]),
                node("b-one", "assistant", Some(1), vec![]),
            ],
        ));
        assert_eq!(ids(&walk(&t).0), ["root", "b-one"]);
    }

    #[test]
    fn oasst_rank_tie_breaks_by_message_id() {
        let t = tree(node(
            "root",
            "prompter",
            None,
            vec![
                node("m2", "assistant", Some(0), vec![]),
                node("m1", "assistant", Some(0), vec![]),
            ],
        ));
        assert_eq!(ids(&walk(&t).0), ["root", "m1"]);
    }

    #[test]
    fn oasst_walk_follows_best_path_and_needs_an_assistant_reply() {
        let deep = node(
            "root",
            "prompter",
            None,
            vec![node(
                "a1",
                "assistant",
                Some(0),
                vec![node(
                    "u2",
                    "prompter",
                    None,
                    vec![node("a2", "assistant", Some(0), vec![])],
                )],
            )],
        );
        let (outcome, _) = walk(&tree(deep));
        assert_eq!(ids(&outcome), ["root", "a1", "u2", "a2"]);
        let TreeOutcome::Thread(thread) = outcome else {
            unreachable!()
        };
        assert_eq!(thread.messages[0].role, "user");
        assert_eq!(thread.messages[1].role, "assistant");
        assert_eq!(thread.first_user_turn(), "text of root");

        let mut lone = node("root", "prompter", None, vec![]);
        lone.replies
            .push(node("gone", "assistant", Some(0), vec![]));
        lone.replies[0].deleted = true;
        let (outcome, drops) = walk(&tree(lone));
        assert_eq!(outcome, TreeOutcome::NoAssistantReply);
        assert_eq!(drops.get("deleted"), Some(&1));
    }

    #[test]
    fn oasst_trees_parse_from_export_json() {
        let line = r#"{"message_tree_id":"t","tree_state":"ready_for_export","prompt":{
            "message_id":"m","parent_id":null,"text":"Hi","role":"prompter","lang":"en",
            "review_result":true,"deleted":false,"rank":null,"synthetic":false,"emojis":{},
            "replies":[{"message_id":"r","text":"Hello","role":"assistant","lang":"en",
            "review_result":null,"deleted":false,"rank":0,"synthetic":false,"replies":[]}]}}"#;
        let t: OasstTree = serde_json::from_str(line).unwrap();
        assert_eq!(t.prompt.replies[0].review_result, None);
        assert_eq!(t.prompt.replies[0].rank, Some(0));
        let mut bad = t.clone();
        bad.prompt.role = "moderator".into();
        assert!(walk_tree(&bad, &mut BTreeMap::new()).is_err());
    }

    // ------------------------------------------------ split

    #[test]
    fn split_is_stable_across_input_orders_and_pinned() {
        // Residues computed outside Rust: `printf %s ID | shasum -a 256`,
        // first 16 hex digits, `% 1000` in bc.
        let pinned = [
            ("a", 250),
            ("b", 946),
            ("c", 930),
            ("d", 329),
            ("e", 442),
            ("f", 682),
            ("g", 749),
            ("h", 687),
        ];
        for (id, residue) in pinned {
            assert_eq!(split_residue(id), residue, "{id}");
        }
        let sides = |order: &[&'static str]| {
            let mut held: Vec<&'static str> =
                order.iter().copied().filter(|i| heldout(i, 500)).collect();
            let mut train: Vec<&'static str> =
                order.iter().copied().filter(|i| !heldout(i, 500)).collect();
            held.sort_unstable();
            train.sort_unstable();
            (held, train)
        };
        let forward: Vec<&'static str> = pinned.iter().map(|(id, _)| *id).collect();
        let mut backward = forward.clone();
        backward.reverse();
        backward.rotate_left(3);
        assert_eq!(sides(&forward), sides(&backward));
        assert_eq!(sides(&forward).0, ["a", "d", "e"]);
        assert!(forward.iter().all(|id| !heldout(id, 0)));
        assert!(forward.iter().all(|id| heldout(id, 1000)));
    }

    // ------------------------------------------------ tokenizer pin

    #[test]
    fn tokenizer_sha_is_checked() {
        let bytes = tokenizer_json(None);
        let sha = hex::encode(Sha256::digest(bytes.as_bytes()));
        assert!(check_tokenizer_sha(bytes.as_bytes(), &sha).is_ok());
        let err = check_tokenizer_sha(bytes.as_bytes(), PINNED_TOKENIZER_SHA256).unwrap_err();
        assert!(matches!(err, RealTextError::TokenizerSha { .. }), "{err}");
    }

    // ------------------------------------------------ exclusion

    #[test]
    fn exclusion_finds_a_shared_8_gram_regardless_of_case_and_punctuation() {
        let exclusion = PanelExclusion::new(&[request(
            "heldout-020",
            &["Can you explain the process of pollination and seed dispersal in flowering plants?"],
        )]);
        assert_eq!(exclusion.ngram_turns, 1);
        assert_eq!(exclusion.ngrams, 13 - PANEL_NGRAM + 1);
        let hit = exclusion
            .first_hit("Biology. THE PROCESS of Pollination -- and seed dispersal, in flowering!")
            .expect("an 8-gram hit");
        assert_eq!(hit.kind, HitKind::Ngram);
        assert_eq!(hit.request_id, "heldout-020");
        assert_eq!(
            hit.words,
            "the process of pollination and seed dispersal in"
        );
        // Seven shared words are not a hit.
        assert!(exclusion
            .first_hit("the process of pollination and seed dispersal")
            .is_none());
    }

    #[test]
    fn exclusion_matches_a_short_turn_whole() {
        let exclusion = PanelExclusion::new(&[
            request("ask-02", &["What do bees make?"]),
            request("ask-07", &["Why do leaves change color in fall"]),
        ]);
        assert_eq!(exclusion.short_turns, 2);
        let hit = exclusion
            .first_hit("Children often ask: what do bees make? Honey.")
            .expect("short request hit");
        assert_eq!(hit.kind, HitKind::ShortRequest);
        assert_eq!(hit.request_id, "ask-02");
        assert_eq!(hit.words, "what do bees make");
        let hit = exclusion
            .first_hit("so why do leaves change color in fall, then")
            .expect("seven-word turn hit");
        assert_eq!(hit.request_id, "ask-07");
        // A partial short request is not a hit.
        assert!(exclusion.first_hit("what do bees eat").is_none());
    }

    #[test]
    fn exclusion_lists_a_three_word_turn_as_uncovered_and_never_matches_it() {
        let exclusion = PanelExclusion::new(&[request("heldout-006", &["Hi Dr. Smith,"])]);
        assert_eq!(exclusion.turns, 1);
        assert_eq!(
            exclusion.uncovered,
            [UncoveredTurn {
                request_id: "heldout-006".into(),
                text: "Hi Dr. Smith,".into(),
            }]
        );
        assert_eq!(exclusion.short_turns + exclusion.ngram_turns, 0);
        assert!(exclusion.first_hit("hi dr smith wrote a paper").is_none());
        assert!(exclusion.first_hit("Hi Dr. Smith,").is_none());
    }

    #[test]
    fn exclusion_confirms_a_forced_hash_collision_against_the_words() {
        let exclusion = PanelExclusion::with_key(
            &[request("p", &["one two three four five six seven eight"])],
            |_| 7,
        );
        // Every window has the same key; only the exact words may match.
        assert!(exclusion
            .first_hit("alpha beta gamma delta epsilon zeta eta theta")
            .is_none());
        assert!(exclusion
            .first_hit("x one two three four five six seven eight")
            .is_some());
    }

    #[test]
    fn exclusion_counts_duplicate_windows_once() {
        let turn = "a b c d e f g h";
        let exclusion = PanelExclusion::new(&[request("p", &[turn]), request("q", &[turn])]);
        assert_eq!(exclusion.ngrams, 1);
        assert_eq!(
            exclusion.first_hit(turn).map(|h| h.request_id),
            Some("p".into())
        );
    }

    #[test]
    fn panel_files_parse_and_malformed_ones_fail() {
        let rows = parse_panel(
            br#"[{"id":"talk-01","category":"smalltalk","user_turns":["Hi! How are you today?"]}]"#,
        )
        .unwrap();
        assert_eq!(rows, vec![request("talk-01", &["Hi! How are you today?"])]);
        assert!(parse_panel(br#"{"id":"x"}"#).is_err());
        assert!(parse_panel(br#"[{"user_turns":["a"]}]"#).is_err());
        assert!(parse_panel(br#"[{"id":"x","user_turns":[]}]"#).is_err());
        assert!(parse_panel(br#"[{"id":"x","user_turns":[3]}]"#).is_err());
    }

    #[test]
    fn an_empty_panel_list_is_refused() {
        let err = parse_panel(b"[]").unwrap_err();
        assert!(
            matches!(&err, RealTextError::Panel(m) if m == "panel has no requests"),
            "{err}"
        );
    }

    #[test]
    fn duplicate_request_ids_across_rows_are_refused() {
        let err = parse_panel(
            br#"[{"id":"x","user_turns":["a"]},{"id":"y","user_turns":["b"]},
                {"id":"x","user_turns":["c"]}]"#,
        )
        .unwrap_err();
        assert!(
            matches!(&err, RealTextError::Panel(m) if m.contains("duplicate") && m.contains('x')),
            "{err}"
        );
    }

    #[test]
    fn a_short_document_still_hits_a_short_panel_turn() {
        // Five words, fewer than PANEL_NGRAM, holding a four-word panel turn.
        let exclusion = PanelExclusion::new(&[request("ask-02", &["what do bees make"])]);
        let hit = exclusion.first_hit("so what do bees make").expect("hit");
        assert_eq!(hit.kind, HitKind::ShortRequest);
        assert_eq!(hit.request_id, "ask-02");
        assert_eq!(hit.words, "what do bees make");
    }

    #[test]
    fn the_pinned_tokenizer_sha_is_the_one_in_hamilton_sheet() {
        let source = include_str!("../../uor-r4-float-train/src/hamilton_sheet.rs");
        assert!(
            source.contains(PINNED_TOKENIZER_SHA256),
            "PINNED_TOKENIZER_SHA256 is not in hamilton_sheet.rs"
        );
    }

    #[test]
    fn oasst_walk_drops_a_thread_that_opens_on_an_assistant_turn() {
        // The builder drops the same thread (its first assistant reply has no
        // user turn before it), so the walk and the builder agree.
        let t = tree(node(
            "root",
            "assistant",
            None,
            vec![node(
                "u1",
                "prompter",
                Some(0),
                vec![node("a1", "assistant", Some(0), vec![])],
            )],
        ));
        let (outcome, _) = walk(&t);
        assert!(
            matches!(outcome, TreeOutcome::NoAssistantReply),
            "{outcome:?}"
        );
    }

    #[test]
    fn oasst_negative_rank_sorts_before_rank_zero() {
        let t = tree(node(
            "root",
            "prompter",
            None,
            vec![
                node("zero", "assistant", Some(0), vec![]),
                node("null", "assistant", None, vec![]),
                node("neg", "assistant", Some(-3), vec![]),
            ],
        ));
        assert_eq!(ids(&walk(&t).0), ["root", "neg"]);
    }

    #[test]
    fn articles_are_cleaned() {
        assert_eq!(
            clean_article(Some(" Title "), "\r\nline one  \r\n\r\n\r\n\nline two\n"),
            "Title\n\nline one\n\nline two"
        );
        assert_eq!(clean_article(None, "  \n "), "");
        assert_eq!(word_count("It's a dog's life."), 4);
    }

    // ------------------------------------------------ documents

    fn thread(turns: &[(&'static str, &str)]) -> Thread {
        Thread {
            tree_id: "t".into(),
            messages: turns
                .iter()
                .map(|(role, text)| ThreadMessage {
                    role,
                    text: (*text).into(),
                })
                .collect(),
        }
    }

    #[test]
    fn documents_open_with_bos_and_mask_the_right_tokens() {
        let tok = tokenizer(None);
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, None).unwrap();
        let Built::Kept(doc) = builder.article(Some("Cats"), "Cats purr.\r\n").unwrap() else {
            panic!("article dropped");
        };
        // Protocol 2: the content carries one leading space after <|bos|>.
        let bytes_of = |s: &str| -> Vec<u16> { s.bytes().map(|b| u16::from(b) + 3).collect() };
        let body = bytes_of(" Cats\n\nCats purr.");
        assert_eq!(doc.tokens[0], 0);
        assert_eq!(&doc.tokens[1..doc.tokens.len() - 1], body.as_slice());
        assert_eq!(doc.tokens[doc.tokens.len() - 1], 1);
        assert_eq!(doc.mask[0], 0);
        assert!(doc.mask[1..].iter().all(|&m| m == 1));
        assert_eq!(doc.content_bytes, "Cats\n\nCats purr.".len());

        // Protocol 1: no leading space.
        let v1 = DialogueProtocol::literal_roles_version(&tok, 1).unwrap();
        let v1_builder = DocumentBuilder::new(&tok, &v1, &exclusion, None).unwrap();
        let Built::Kept(doc) = v1_builder.article(Some("Cats"), "Cats purr.").unwrap() else {
            panic!("v1 article dropped");
        };
        let body = bytes_of("Cats\n\nCats purr.");
        assert_eq!(&doc.tokens[1..doc.tokens.len() - 1], body.as_slice());

        let Built::Kept(doc) = builder
            .thread(&thread(&[("user", " Hi\r\n"), ("assistant", "Hello")]))
            .unwrap()
        else {
            panic!("thread dropped");
        };
        assert_eq!(doc.tokens[0], 0);
        assert_eq!(doc.mask[0], 0);
        assert_eq!(doc.tokens.last(), Some(&1));
        // <|bos|> "User:" " Hi" "\n" "Assistant:" are unmasked; " Hello" and
        // its closing <|eos|> are masked in.
        let unmasked = 1 + "User:".len() + " Hi".len() + "\n".len() + "Assistant:".len();
        let expected = [vec![0u8; unmasked], vec![1u8; " Hello".len() + 1]].concat();
        assert_eq!(doc.mask, expected);
        assert_eq!(doc.tokens.len(), expected.len());
        assert_eq!(doc.content_bytes, "Hi".len() + "Hello".len());

        // A thread ending on an unanswered user turn: the assistant's eos has
        // mask 1, the terminal eos has mask 0.
        let Built::Kept(doc) = builder
            .thread(&thread(&[
                ("user", "Hi"),
                ("assistant", "Yo"),
                ("user", "More"),
            ]))
            .unwrap()
        else {
            panic!("open thread dropped");
        };
        let head = 1 + "User:".len() + " Hi".len() + "\n".len() + "Assistant:".len();
        let tail = "\n".len() + "User:".len() + " More".len() + 1;
        let expected = [vec![0u8; head], vec![1u8; " Yo".len() + 1], vec![0u8; tail]].concat();
        assert_eq!(doc.mask, expected);
        assert_eq!(doc.tokens.len(), expected.len());
        let assistant_eos = head + " Yo".len();
        assert_eq!(doc.tokens[assistant_eos], 1);
        assert_eq!(doc.mask[assistant_eos], 1);
        assert_eq!(doc.tokens.last(), Some(&1));
        assert_eq!(doc.mask.last(), Some(&0));
    }

    #[test]
    fn a_token_id_above_u16_is_a_defect_even_in_a_too_long_document() {
        let tok = tokenizer(None);
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, Some(0)).unwrap();
        let err = builder.finish(&[0, 70_000], vec![0, 1], 0).unwrap_err();
        assert!(matches!(err, RealTextError::Defect(_)), "{err}");
    }

    #[test]
    fn a_blank_user_turn_before_the_first_reply_drops_the_thread() {
        let tok = tokenizer(None);
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, None).unwrap();
        assert_eq!(
            builder
                .thread(&thread(&[("user", "   "), ("assistant", "Hi")]))
                .unwrap(),
            Built::Dropped(DocDrop::NoAssistantReply)
        );
    }

    #[test]
    fn special_text_in_a_thread_reply_drops_the_thread() {
        let tok = tokenizer(None);
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, None).unwrap();
        assert_eq!(
            builder
                .thread(&thread(&[("user", "Hi"), ("assistant", "say <|eos|> now")]))
                .unwrap(),
            Built::Dropped(DocDrop::SpecialText)
        );
    }

    #[test]
    fn a_content_that_does_not_round_trip_is_dropped() {
        let tok = tokenizer(Some(b'q'));
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, None).unwrap();
        assert_eq!(
            builder
                .thread(&thread(&[("user", "a quiet one"), ("assistant", "yes")]))
                .unwrap(),
            Built::Dropped(DocDrop::RoundTrip)
        );
        assert_eq!(
            builder.article(None, "quiet").unwrap(),
            Built::Dropped(DocDrop::RoundTrip)
        );
        assert!(matches!(
            builder.article(None, "loud").unwrap(),
            Built::Kept(_)
        ));
    }

    #[test]
    fn exclusion_special_text_and_length_drop_documents() {
        let tok = tokenizer(None);
        let protocol = DialogueProtocol::literal_roles_version(&tok, 2).unwrap();
        let exclusion = PanelExclusion::new(&[request("p", &["What do bees make?"])]);
        let builder = DocumentBuilder::new(&tok, &protocol, &exclusion, Some(12)).unwrap();
        // The assistant turn alone carries the hit: it still drops the thread.
        let Built::Excluded(hit) = builder
            .thread(&thread(&[
                ("user", "Hi"),
                ("assistant", "What do bees make? Honey."),
            ]))
            .unwrap()
        else {
            panic!("not excluded");
        };
        assert_eq!(hit.request_id, "p");
        assert_eq!(
            builder.article(None, "say <|eos|> now").unwrap(),
            Built::Dropped(DocDrop::SpecialText)
        );
        assert_eq!(
            builder
                .article(None, "a long article of many bytes")
                .unwrap(),
            Built::Dropped(DocDrop::TooLong)
        );
        assert_eq!(
            builder.article(Some(" "), " \n").unwrap(),
            Built::Dropped(DocDrop::Empty)
        );
        assert_eq!(
            builder
                .thread(&thread(&[("user", "Hi"), ("assistant", "  ")]))
                .unwrap(),
            Built::Dropped(DocDrop::NoAssistantReply)
        );
    }
}
