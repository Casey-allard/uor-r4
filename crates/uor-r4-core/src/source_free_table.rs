//! Source-free lexical transition baseline (issue #989).
//!
//! This module is intentionally geometry-free. It compiles raw UTF-8
//! construction documents into a reversible lexical-piece codec plus integer
//! unigram, bigram, and trigram count tables. Held-out documents are never
//! consulted during fitting. Runtime selection is the fixed precedence
//! trigram -> bigram -> unigram with highest-count/lowest-token tie breaking.

use std::collections::{BTreeMap, BTreeSet};

use crate::canonical_lexical_ingestion::{canonical_lexical_piece_bytes, CanonicalLexicalError};

const ARTIFACT_MAGIC: [u8; 8] = *b"SFTBL001";
const ARTIFACT_VERSION: u32 = 1;
const HEADER_LEN: usize = 40;
const BYTE_TOKEN_BASE: u32 = 2;
const LEXICAL_TOKEN_BASE: u32 = BYTE_TOKEN_BASE + 256;
pub const BOS_TOKEN: u32 = 0;
pub const EOS_TOKEN: u32 = 1;
pub const MAX_CONTINUATION_UNITS: usize = 64;

type Distribution = BTreeMap<u32, u64>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceFreeTableError {
    Invalid(String),
    Lexical(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for SourceFreeTableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::Lexical(reason) => write!(formatter, "lexical input: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("source-free table arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for SourceFreeTableError {}

impl From<CanonicalLexicalError> for SourceFreeTableError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::Lexical(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocument {
    pub id: String,
    pub text: Vec<u8>,
}

impl SourceDocument {
    pub fn new(id: impl Into<String>, text: impl Into<Vec<u8>>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }

    pub fn text_cid(&self) -> [u8; 32] {
        *blake3::hash(&self.text).as_bytes()
    }
}

/// The repository's canonical D3 document partition.
pub fn d3_is_held_out(document_id: &str) -> bool {
    blake3::hash(document_id.as_bytes()).as_bytes()[0].is_multiple_of(5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffOrder {
    Unigram,
    Bigram,
    Trigram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TablePrediction {
    pub token: u32,
    pub count: u64,
    pub order: BackoffOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeldOutEvaluation {
    pub documents: u64,
    /// Every scored stream target, including held-out byte-fallback targets.
    pub positions: u64,
    /// Targets represented by a construction-fitted lexical payload. Accuracy,
    /// changed-choice, and order counters below use only this denominator.
    pub known_target_positions: u64,
    pub table_correct: u64,
    pub unigram_correct: u64,
    pub changed_choices: u64,
    pub changed_choice_correct: u64,
    pub trigram_choices: u64,
    pub bigram_choices: u64,
    pub unigram_choices: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationStop {
    EndOfDocument,
    PeriodOneCycle,
    PeriodTwoCycle,
    Bound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Continuation {
    pub tokens: Vec<u32>,
    pub decoded: Vec<u8>,
    pub stop: ContinuationStop,
}

/// Deterministic construction-only lexical count model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFreeTable {
    /// Lexical payloads in ascending byte order. Token id is
    /// `LEXICAL_TOKEN_BASE + index`.
    lexical_pieces: Vec<Vec<u8>>,
    piece_tokens: BTreeMap<Vec<u8>, u32>,
    unigram: Distribution,
    bigram: BTreeMap<u32, Distribution>,
    trigram: BTreeMap<(u32, u32), Distribution>,
    construction_document_ids: BTreeSet<String>,
    construction_text_cids: BTreeSet<[u8; 32]>,
}

impl SourceFreeTable {
    /// Fit the codec and every count table from D3 construction documents.
    /// A held-out id is rejected rather than silently admitted.
    pub fn compile(construction: &[SourceDocument]) -> Result<Self, SourceFreeTableError> {
        if construction.is_empty() {
            return Err(SourceFreeTableError::Invalid(
                "construction document set is empty".to_owned(),
            ));
        }

        let mut construction_document_ids = BTreeSet::new();
        let mut construction_text_cids = BTreeSet::new();
        let mut lexical_piece_set = BTreeSet::new();
        for document in construction {
            validate_document_id(&document.id)?;
            if d3_is_held_out(&document.id) {
                return Err(SourceFreeTableError::Invalid(format!(
                    "D3 held-out document {} cannot enter construction",
                    document.id
                )));
            }
            if !construction_document_ids.insert(document.id.clone()) {
                return Err(SourceFreeTableError::Invalid(format!(
                    "duplicate construction document id {}",
                    document.id
                )));
            }
            construction_text_cids.insert(document.text_cid());
            lexical_piece_set.extend(canonical_lexical_piece_bytes(&document.text)?);
        }
        if lexical_piece_set.is_empty() {
            return Err(SourceFreeTableError::Invalid(
                "construction corpus has no lexical pieces".to_owned(),
            ));
        }

        let lexical_pieces = lexical_piece_set.into_iter().collect::<Vec<_>>();
        let piece_tokens = build_piece_tokens(&lexical_pieces)?;
        let mut table = Self {
            lexical_pieces,
            piece_tokens,
            unigram: BTreeMap::new(),
            bigram: BTreeMap::new(),
            trigram: BTreeMap::new(),
            construction_document_ids,
            construction_text_cids,
        };

        for document in construction {
            let mut stream = Vec::new();
            stream.push(BOS_TOKEN);
            stream.extend(table.encode_text(&document.text)?);
            stream.push(EOS_TOKEN);
            for target_index in 1..stream.len() {
                let target = stream[target_index];
                add_count(&mut table.unigram, target)?;
                add_count(
                    table.bigram.entry(stream[target_index - 1]).or_default(),
                    target,
                )?;
                if target_index >= 2 {
                    add_count(
                        table
                            .trigram
                            .entry((stream[target_index - 2], stream[target_index - 1]))
                            .or_default(),
                        target,
                    )?;
                }
            }
        }
        table.validate()?;
        Ok(table)
    }

    pub fn construction_document_count(&self) -> usize {
        self.construction_document_ids.len()
    }

    pub fn lexical_piece_count(&self) -> usize {
        self.lexical_pieces.len()
    }

    pub fn artifact_cid(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.to_bytes()).to_hex())
    }

    /// Encode with construction-fitted lexical pieces and a total raw-byte
    /// fallback. The fallback makes every valid UTF-8 held-out payload exactly
    /// representable without admitting held-out vocabulary into fitting.
    pub fn encode_text(&self, bytes: &[u8]) -> Result<Vec<u32>, SourceFreeTableError> {
        let pieces = canonical_lexical_piece_bytes(bytes)?;
        let mut tokens = Vec::new();
        for piece in pieces {
            if let Some(&token) = self.piece_tokens.get(&piece) {
                tokens.push(token);
            } else {
                tokens.extend(piece.into_iter().map(byte_token));
            }
        }
        Ok(tokens)
    }

    /// Exact inverse of [`Self::encode_text`] for every valid token id.
    pub fn decode_tokens(&self, tokens: &[u32]) -> Result<Vec<u8>, SourceFreeTableError> {
        let mut bytes = Vec::new();
        for &token in tokens {
            match token {
                BOS_TOKEN | EOS_TOKEN => {}
                BYTE_TOKEN_BASE..=257 => bytes.push((token - BYTE_TOKEN_BASE) as u8),
                _ => {
                    let index = token
                        .checked_sub(LEXICAL_TOKEN_BASE)
                        .and_then(|value| usize::try_from(value).ok())
                        .ok_or_else(|| invalid_token(token))?;
                    let piece = self
                        .lexical_pieces
                        .get(index)
                        .ok_or_else(|| invalid_token(token))?;
                    bytes.extend_from_slice(piece);
                }
            }
        }
        Ok(bytes)
    }

    /// Integer-only trigram -> bigram -> unigram selection. Distributions are
    /// B-trees, so retaining the first maximum implements the canonical
    /// lowest-token tie break.
    pub fn predict(&self, context: &[u32]) -> TablePrediction {
        if context.len() >= 2 {
            let key = (context[context.len() - 2], context[context.len() - 1]);
            if let Some(distribution) = self.trigram.get(&key) {
                let (token, count) = distribution_winner(distribution);
                return TablePrediction {
                    token,
                    count,
                    order: BackoffOrder::Trigram,
                };
            }
        }
        if let Some(&last) = context.last() {
            if let Some(distribution) = self.bigram.get(&last) {
                let (token, count) = distribution_winner(distribution);
                return TablePrediction {
                    token,
                    count,
                    order: BackoffOrder::Bigram,
                };
            }
        }
        let (token, count) = distribution_winner(&self.unigram);
        TablePrediction {
            token,
            count,
            order: BackoffOrder::Unigram,
        }
    }

    pub fn unigram_prediction(&self) -> TablePrediction {
        let (token, count) = distribution_winner(&self.unigram);
        TablePrediction {
            token,
            count,
            order: BackoffOrder::Unigram,
        }
    }

    /// Score pristine D3 held-out documents. Evaluation fails closed on
    /// document-id overlap, exact text-CID overlap, duplicates, or a document
    /// whose D3 partition is construction.
    pub fn evaluate_held_out(
        &self,
        held_out: &[SourceDocument],
    ) -> Result<HeldOutEvaluation, SourceFreeTableError> {
        if held_out.is_empty() {
            return Err(SourceFreeTableError::Invalid(
                "held-out document set is empty".to_owned(),
            ));
        }
        let mut seen_ids = BTreeSet::new();
        let unigram = self.unigram_prediction();
        let mut evaluation = HeldOutEvaluation::default();
        for document in held_out {
            validate_document_id(&document.id)?;
            if !d3_is_held_out(&document.id) {
                return Err(SourceFreeTableError::Invalid(format!(
                    "D3 construction document {} cannot enter held-out evaluation",
                    document.id
                )));
            }
            let text_cid = document.text_cid();
            if self.construction_document_ids.contains(&document.id) {
                return Err(SourceFreeTableError::Invalid(format!(
                    "held-out document id {} overlaps construction",
                    document.id
                )));
            }
            if self.construction_text_cids.contains(&text_cid) {
                return Err(SourceFreeTableError::Invalid(format!(
                    "held-out document {} has construction text CID",
                    document.id
                )));
            }
            if !seen_ids.insert(document.id.clone()) {
                return Err(SourceFreeTableError::Invalid(
                    "held-out documents contain a duplicate id".to_owned(),
                ));
            }

            let mut stream = Vec::new();
            stream.push(BOS_TOKEN);
            stream.extend(self.encode_text(&document.text)?);
            stream.push(EOS_TOKEN);
            evaluation.documents = checked_increment(evaluation.documents)?;
            for target_index in 1..stream.len() {
                let target = stream[target_index];
                evaluation.positions = checked_increment(evaluation.positions)?;
                if !self.is_fitted_lexical_token(target) {
                    continue;
                }
                evaluation.known_target_positions =
                    checked_increment(evaluation.known_target_positions)?;
                let prediction = self.predict(&stream[..target_index]);
                match prediction.order {
                    BackoffOrder::Trigram => {
                        evaluation.trigram_choices = checked_increment(evaluation.trigram_choices)?
                    }
                    BackoffOrder::Bigram => {
                        evaluation.bigram_choices = checked_increment(evaluation.bigram_choices)?
                    }
                    BackoffOrder::Unigram => {
                        evaluation.unigram_choices = checked_increment(evaluation.unigram_choices)?
                    }
                }
                if prediction.token == target {
                    evaluation.table_correct = checked_increment(evaluation.table_correct)?;
                }
                if unigram.token == target {
                    evaluation.unigram_correct = checked_increment(evaluation.unigram_correct)?;
                }
                if prediction.token != unigram.token {
                    evaluation.changed_choices = checked_increment(evaluation.changed_choices)?;
                    if prediction.token == target {
                        evaluation.changed_choice_correct =
                            checked_increment(evaluation.changed_choice_correct)?;
                    }
                }
            }
        }
        Ok(evaluation)
    }

    /// Canonical, human-readable evidence bytes for the smallest vertical
    /// slice. The zero counters are claim-bearing: this path has no API by
    /// which a teacher, provider, source weights, or geometry can be read.
    pub fn canonical_transcript_bytes(
        &self,
        evaluation: &HeldOutEvaluation,
        seed: &[u8],
        continuation: &Continuation,
    ) -> Vec<u8> {
        format!(
            "source_free_table_schema=1\nartifact_cid={}\nconstruction_documents={}\nlexical_pieces={}\nheld_out_documents={}\npositions={}\nknown_target_positions={}\ntable_correct={}\nunigram_correct={}\nchanged_choices={}\nchanged_choice_correct={}\ntrigram_choices={}\nbigram_choices={}\nunigram_choices={}\nteacher_calls=0\nprovider_calls=0\nsource_weight_reads=0\ngeometry_calls=0\nseed_hex={}\ncontinuation_tokens={}\ncontinuation_hex={}\ncontinuation_stop={}\n",
            self.artifact_cid(),
            self.construction_document_count(),
            self.lexical_piece_count(),
            evaluation.documents,
            evaluation.positions,
            evaluation.known_target_positions,
            evaluation.table_correct,
            evaluation.unigram_correct,
            evaluation.changed_choices,
            evaluation.changed_choice_correct,
            evaluation.trigram_choices,
            evaluation.bigram_choices,
            evaluation.unigram_choices,
            hex::encode(seed),
            continuation.tokens.len(),
            hex::encode(&continuation.decoded),
            continuation_stop_name(continuation.stop),
        )
        .into_bytes()
    }

    /// Deterministically continue a prompt for at most `max_units`. EOS and
    /// period-one/two cycles stop before their sentinel token is appended.
    pub fn continue_text(
        &self,
        seed: &[u8],
        max_units: usize,
    ) -> Result<Continuation, SourceFreeTableError> {
        if max_units == 0 || max_units > MAX_CONTINUATION_UNITS {
            return Err(SourceFreeTableError::Invalid(format!(
                "continuation bound must be 1..={MAX_CONTINUATION_UNITS}"
            )));
        }
        let mut context = Vec::new();
        context.push(BOS_TOKEN);
        context.extend(self.encode_text(seed)?);
        let mut generated = Vec::new();
        let mut stop = ContinuationStop::Bound;
        while generated.len() < max_units {
            let token = self.predict(&context).token;
            if token == EOS_TOKEN {
                stop = ContinuationStop::EndOfDocument;
                break;
            }
            if generated.last() == Some(&token) {
                stop = ContinuationStop::PeriodOneCycle;
                break;
            }
            if generated.len() >= 3
                && generated[generated.len() - 2] == token
                && generated[generated.len() - 3] == generated[generated.len() - 1]
            {
                stop = ContinuationStop::PeriodTwoCycle;
                break;
            }
            generated.push(token);
            context.push(token);
        }
        let decoded = self.decode_tokens(&generated)?;
        Ok(Continuation {
            tokens: generated,
            decoded,
            stop,
        })
    }

    /// Canonical packed bytes. Maps and sets serialize in their B-tree order.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ARTIFACT_MAGIC);
        push_u32(&mut bytes, ARTIFACT_VERSION);
        push_u32(&mut bytes, usize_u32(self.lexical_pieces.len()));
        push_u32(&mut bytes, usize_u32(self.construction_document_ids.len()));
        push_u32(&mut bytes, usize_u32(self.construction_text_cids.len()));
        push_u32(&mut bytes, usize_u32(self.unigram.len()));
        push_u32(&mut bytes, usize_u32(self.bigram.len()));
        push_u32(&mut bytes, usize_u32(self.trigram.len()));
        push_u32(&mut bytes, 0);
        debug_assert_eq!(bytes.len(), HEADER_LEN);

        for piece in &self.lexical_pieces {
            push_len_bytes(&mut bytes, piece);
        }
        for id in &self.construction_document_ids {
            push_len_bytes(&mut bytes, id.as_bytes());
        }
        for cid in &self.construction_text_cids {
            bytes.extend_from_slice(cid);
        }
        push_distribution(&mut bytes, &self.unigram);
        for (&key, distribution) in &self.bigram {
            push_u32(&mut bytes, key);
            push_u32(&mut bytes, usize_u32(distribution.len()));
            push_distribution(&mut bytes, distribution);
        }
        for (&(key0, key1), distribution) in &self.trigram {
            push_u32(&mut bytes, key0);
            push_u32(&mut bytes, key1);
            push_u32(&mut bytes, usize_u32(distribution.len()));
            push_distribution(&mut bytes, distribution);
        }
        bytes
    }

    /// Validate and reload canonical packed bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SourceFreeTableError> {
        if bytes.len() < HEADER_LEN || bytes[..8] != ARTIFACT_MAGIC {
            return Err(SourceFreeTableError::Invalid(
                "source-free table magic/header is invalid".to_owned(),
            ));
        }
        let mut cursor = Cursor::new(bytes);
        cursor.take(8)?;
        if cursor.u32()? != ARTIFACT_VERSION {
            return Err(SourceFreeTableError::Invalid(
                "source-free table version is unsupported".to_owned(),
            ));
        }
        let piece_count = cursor.count("lexical piece")?;
        let document_id_count = cursor.count("construction document id")?;
        let text_cid_count = cursor.count("construction text CID")?;
        let unigram_count = cursor.count("unigram entry")?;
        let bigram_count = cursor.count("bigram row")?;
        let trigram_count = cursor.count("trigram row")?;
        if cursor.u32()? != 0 {
            return Err(SourceFreeTableError::Invalid(
                "source-free table reserved field is nonzero".to_owned(),
            ));
        }

        let mut lexical_pieces = Vec::with_capacity(piece_count);
        for _ in 0..piece_count {
            lexical_pieces.push(cursor.length_prefixed("lexical piece")?.to_vec());
        }
        ensure_strictly_sorted_nonempty(&lexical_pieces, "lexical pieces")?;
        let piece_tokens = build_piece_tokens(&lexical_pieces)?;

        let mut construction_document_ids = BTreeSet::new();
        let mut previous_id: Option<String> = None;
        for _ in 0..document_id_count {
            let raw = cursor.length_prefixed("construction document id")?;
            let id = std::str::from_utf8(raw)
                .map_err(|_| {
                    SourceFreeTableError::Invalid(
                        "construction document id is not UTF-8".to_owned(),
                    )
                })?
                .to_owned();
            validate_document_id(&id)?;
            if previous_id.as_ref().is_some_and(|previous| previous >= &id) {
                return Err(SourceFreeTableError::Invalid(
                    "construction document ids are not strictly sorted".to_owned(),
                ));
            }
            previous_id = Some(id.clone());
            construction_document_ids.insert(id);
        }

        let mut construction_text_cids = BTreeSet::new();
        let mut previous_cid = None;
        for _ in 0..text_cid_count {
            let cid: [u8; 32] = cursor
                .take(32)?
                .try_into()
                .map_err(|_| SourceFreeTableError::ArithmeticOverflow)?;
            if previous_cid.is_some_and(|previous| previous >= cid) {
                return Err(SourceFreeTableError::Invalid(
                    "construction text CIDs are not strictly sorted".to_owned(),
                ));
            }
            previous_cid = Some(cid);
            construction_text_cids.insert(cid);
        }

        let unigram = cursor.distribution(unigram_count, lexical_pieces.len())?;
        let mut bigram = BTreeMap::new();
        let mut previous_bigram = None;
        for _ in 0..bigram_count {
            let key = cursor.u32()?;
            if previous_bigram.is_some_and(|previous| previous >= key) {
                return Err(SourceFreeTableError::Invalid(
                    "bigram rows are not strictly sorted".to_owned(),
                ));
            }
            validate_token(key, lexical_pieces.len())?;
            previous_bigram = Some(key);
            let entries = cursor.count("bigram entry")?;
            if entries == 0 {
                return Err(SourceFreeTableError::Invalid(
                    "bigram row is empty".to_owned(),
                ));
            }
            bigram.insert(key, cursor.distribution(entries, lexical_pieces.len())?);
        }
        let mut trigram = BTreeMap::new();
        let mut previous_trigram = None;
        for _ in 0..trigram_count {
            let key = (cursor.u32()?, cursor.u32()?);
            if previous_trigram.is_some_and(|previous| previous >= key) {
                return Err(SourceFreeTableError::Invalid(
                    "trigram rows are not strictly sorted".to_owned(),
                ));
            }
            validate_token(key.0, lexical_pieces.len())?;
            validate_token(key.1, lexical_pieces.len())?;
            previous_trigram = Some(key);
            let entries = cursor.count("trigram entry")?;
            if entries == 0 {
                return Err(SourceFreeTableError::Invalid(
                    "trigram row is empty".to_owned(),
                ));
            }
            trigram.insert(key, cursor.distribution(entries, lexical_pieces.len())?);
        }
        if !cursor.is_finished() {
            return Err(SourceFreeTableError::Invalid(
                "source-free table has trailing bytes".to_owned(),
            ));
        }

        let table = Self {
            lexical_pieces,
            piece_tokens,
            unigram,
            bigram,
            trigram,
            construction_document_ids,
            construction_text_cids,
        };
        table.validate()?;
        if table.to_bytes() != bytes {
            return Err(SourceFreeTableError::Invalid(
                "source-free table is not canonical".to_owned(),
            ));
        }
        Ok(table)
    }

    fn validate(&self) -> Result<(), SourceFreeTableError> {
        if self.lexical_pieces.is_empty()
            || self.unigram.is_empty()
            || self.construction_document_ids.is_empty()
            || self.construction_text_cids.is_empty()
        {
            return Err(SourceFreeTableError::Invalid(
                "source-free table has an empty required section".to_owned(),
            ));
        }
        ensure_strictly_sorted_nonempty(&self.lexical_pieces, "lexical pieces")?;
        if build_piece_tokens(&self.lexical_pieces)? != self.piece_tokens {
            return Err(SourceFreeTableError::Invalid(
                "lexical piece index does not reproduce".to_owned(),
            ));
        }
        validate_distribution(&self.unigram, self.lexical_pieces.len())?;
        for (&key, distribution) in &self.bigram {
            validate_token(key, self.lexical_pieces.len())?;
            validate_distribution(distribution, self.lexical_pieces.len())?;
        }
        for (&(key0, key1), distribution) in &self.trigram {
            validate_token(key0, self.lexical_pieces.len())?;
            validate_token(key1, self.lexical_pieces.len())?;
            validate_distribution(distribution, self.lexical_pieces.len())?;
        }
        Ok(())
    }

    fn is_fitted_lexical_token(&self, token: u32) -> bool {
        token.checked_sub(LEXICAL_TOKEN_BASE).is_some_and(|offset| {
            usize::try_from(offset)
                .ok()
                .is_some_and(|index| index < self.lexical_pieces.len())
        })
    }
}

fn continuation_stop_name(stop: ContinuationStop) -> &'static str {
    match stop {
        ContinuationStop::EndOfDocument => "end_of_document",
        ContinuationStop::PeriodOneCycle => "period_one_cycle",
        ContinuationStop::PeriodTwoCycle => "period_two_cycle",
        ContinuationStop::Bound => "bound",
    }
}

fn validate_document_id(id: &str) -> Result<(), SourceFreeTableError> {
    if id.is_empty() || id.len() > u32::MAX as usize {
        return Err(SourceFreeTableError::Invalid(
            "document id is empty or too long".to_owned(),
        ));
    }
    Ok(())
}

fn build_piece_tokens(
    lexical_pieces: &[Vec<u8>],
) -> Result<BTreeMap<Vec<u8>, u32>, SourceFreeTableError> {
    lexical_pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            let offset =
                u32::try_from(index).map_err(|_| SourceFreeTableError::ArithmeticOverflow)?;
            let token = LEXICAL_TOKEN_BASE
                .checked_add(offset)
                .ok_or(SourceFreeTableError::ArithmeticOverflow)?;
            Ok((piece.clone(), token))
        })
        .collect()
}

fn byte_token(byte: u8) -> u32 {
    BYTE_TOKEN_BASE + u32::from(byte)
}

fn invalid_token(token: u32) -> SourceFreeTableError {
    SourceFreeTableError::Invalid(format!("unknown source-free token id {token}"))
}

fn validate_token(token: u32, piece_count: usize) -> Result<(), SourceFreeTableError> {
    if token <= 257 {
        return Ok(());
    }
    let index = token
        .checked_sub(LEXICAL_TOKEN_BASE)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid_token(token))?;
    if index >= piece_count {
        return Err(invalid_token(token));
    }
    Ok(())
}

fn add_count(distribution: &mut Distribution, token: u32) -> Result<(), SourceFreeTableError> {
    let count = distribution.entry(token).or_insert(0);
    *count = count
        .checked_add(1)
        .ok_or(SourceFreeTableError::ArithmeticOverflow)?;
    Ok(())
}

fn checked_increment(value: u64) -> Result<u64, SourceFreeTableError> {
    value
        .checked_add(1)
        .ok_or(SourceFreeTableError::ArithmeticOverflow)
}

fn distribution_winner(distribution: &Distribution) -> (u32, u64) {
    let mut winner = (0, 0);
    for (&token, &count) in distribution {
        if count > winner.1 {
            winner = (token, count);
        }
    }
    winner
}

fn validate_distribution(
    distribution: &Distribution,
    piece_count: usize,
) -> Result<(), SourceFreeTableError> {
    if distribution.is_empty() {
        return Err(SourceFreeTableError::Invalid(
            "source-free distribution is empty".to_owned(),
        ));
    }
    for (&token, &count) in distribution {
        validate_token(token, piece_count)?;
        if count == 0 {
            return Err(SourceFreeTableError::Invalid(
                "source-free distribution contains a zero count".to_owned(),
            ));
        }
    }
    Ok(())
}

fn ensure_strictly_sorted_nonempty(
    values: &[Vec<u8>],
    label: &str,
) -> Result<(), SourceFreeTableError> {
    if values.is_empty()
        || values.iter().any(Vec::is_empty)
        || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(SourceFreeTableError::Invalid(format!(
            "{label} are empty, duplicated, or not strictly sorted"
        )));
    }
    Ok(())
}

fn usize_u32(value: usize) -> u32 {
    u32::try_from(value).expect("validated source-free table length exceeds u32")
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_len_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    push_u32(output, usize_u32(bytes.len()));
    output.extend_from_slice(bytes);
}

fn push_distribution(output: &mut Vec<u8>, distribution: &Distribution) {
    for (&token, &count) in distribution {
        push_u32(output, token);
        push_u64(output, count);
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], SourceFreeTableError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SourceFreeTableError::ArithmeticOverflow)?;
        let value = self.bytes.get(self.offset..end).ok_or_else(|| {
            SourceFreeTableError::Invalid("source-free table is truncated".to_owned())
        })?;
        self.offset = end;
        Ok(value)
    }

    fn u32(&mut self) -> Result<u32, SourceFreeTableError> {
        Ok(u32::from_le_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| SourceFreeTableError::ArithmeticOverflow)?,
        ))
    }

    fn u64(&mut self) -> Result<u64, SourceFreeTableError> {
        Ok(u64::from_le_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| SourceFreeTableError::ArithmeticOverflow)?,
        ))
    }

    fn count(&mut self, label: &str) -> Result<usize, SourceFreeTableError> {
        let count =
            usize::try_from(self.u32()?).map_err(|_| SourceFreeTableError::ArithmeticOverflow)?;
        if count > self.bytes.len().saturating_sub(self.offset) {
            return Err(SourceFreeTableError::Invalid(format!(
                "{label} count exceeds remaining artifact bytes"
            )));
        }
        Ok(count)
    }

    fn length_prefixed(&mut self, label: &str) -> Result<&'a [u8], SourceFreeTableError> {
        let length =
            usize::try_from(self.u32()?).map_err(|_| SourceFreeTableError::ArithmeticOverflow)?;
        if length == 0 {
            return Err(SourceFreeTableError::Invalid(format!("{label} is empty")));
        }
        self.take(length)
    }

    fn distribution(
        &mut self,
        entries: usize,
        piece_count: usize,
    ) -> Result<Distribution, SourceFreeTableError> {
        if entries == 0 {
            return Err(SourceFreeTableError::Invalid(
                "source-free distribution is empty".to_owned(),
            ));
        }
        let mut distribution = BTreeMap::new();
        let mut previous = None;
        for _ in 0..entries {
            let token = self.u32()?;
            let count = self.u64()?;
            validate_token(token, piece_count)?;
            if count == 0 || previous.is_some_and(|prior| prior >= token) {
                return Err(SourceFreeTableError::Invalid(
                    "distribution entries are zero or not strictly sorted".to_owned(),
                ));
            }
            previous = Some(token);
            distribution.insert(token, count);
        }
        Ok(distribution)
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
