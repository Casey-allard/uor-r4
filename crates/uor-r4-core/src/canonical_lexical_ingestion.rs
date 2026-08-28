//! Canonical GI-1 lexical ingestion and incremental route-hierarchy state.
//!
//! This module is the additive S0 boundary from issue #961. It preserves the
//! frozen [`CompiledSpinManifest`] schema-2 artifact and binds its exact bytes
//! as a transitively validated child of a new canonical envelope. The lexical
//! codec owns segmentation, boundaries, vocabulary identity, unknown-unit
//! behavior, and inverse decoding. Kappa remains exact object identity only.
//!
//! The public artifact is deliberately not an attention or generation API. It
//! exposes the seven immutable views consumed by #952: current, previous,
//! last-two, sentence, paragraph, conversation, and bounded global state.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU16, NonZeroUsize};

use serde::{Deserialize, Serialize};

use crate::prime_route_attention::{
    compile_spin_manifest, zeta_phase_delta, CompiledSpinManifest, GeometricAddress,
    ManifestProvenance, PhaseQ29, PrimeAtom, PrimeRegistry, PrimeRouteError, RouteSentence,
    SemanticAtom, SpinTorsionState, UnitS3Q30, ZPhi, ZeroPowerBridge,
    TINY_CANARY_MAX_ROUTES_PER_SENTENCE, TINY_CANARY_MAX_SENTENCES,
};

pub const CANONICAL_ROUTE_ARTIFACT_SCHEMA: u32 = 1;
pub const CANONICAL_ROUTE_ARTIFACT_DOMAIN: &str = "uor-r4.canonical-lexical-route-manifest/1";
pub const LEXICAL_CODEC_SCHEMA: u32 = 1;
pub const LEXICAL_CODEC_DOMAIN: &str = "uor-r4.unicode-lexical-runs/1";
pub const HIERARCHY_NODE_SCHEMA: u32 = 1;
pub const HIERARCHY_NODE_DOMAIN: &str = "uor-r4.incremental-route-hierarchy-node/1";
pub const ROUTE_RECORD_SCHEMA: u32 = 1;
pub const ROUTE_RECORD_DOMAIN: &str = "uor-r4.lexical-route-occurrence/1";
pub const CHART_PROFILE_SCHEMA: u32 = 1;
pub const CHART_PROFILE_DOMAIN: &str = "uor-r4.typed-chart-adapter-profile/1";
pub const ICOSIAN_PROFILE_SCHEMA: u32 = 1;
pub const ICOSIAN_PROFILE_DOMAIN: &str = "uor-r4.icosian-h4-phi-h4-profile/1";
pub const H4_BINARY_ICOSAHEDRAL_TABLE_SCHEMA: u32 = 1;
pub const H4_BINARY_ICOSAHEDRAL_TABLE_DOMAIN: &str =
    "uor-r4.scaled-h4-binary-icosahedral-multiplication/1";
pub const H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE: &str =
    "blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76";
pub const H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE: &str =
    "blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759";
pub const ORDERED_H4_FOLD_SCHEMA: u32 = 1;
pub const ORDERED_H4_FOLD_DOMAIN: &str = "uor-r4.associative-ordered-h4-fold/1";
pub const ATTENTION_ORDERED_H4_FOLD_TRACE_SCHEMA: u32 = 1;
pub const ATTENTION_ORDERED_H4_FOLD_TRACE_DOMAIN: &str =
    "uor-r4.attention-associative-ordered-h4-fold/1";
pub const TRAJECTORY_PROFILE_SCHEMA: u32 = 1;
pub const TRAJECTORY_PROFILE_DOMAIN: &str = "uor-r4.route-trajectory-summary/1";
pub const PROBE_WITNESS_SCHEMA: u32 = 1;
pub const PROBE_WITNESS_DOMAIN: &str = "uor-r4.canonical-lexical-ingestion-witness/1";
pub const CANONICAL_ROUTE_ARTIFACT_MAX_BYTES: usize = 128 * 1024 * 1024;

const MAX_TURNS: usize = 8;
const MAX_PARAGRAPHS: usize = 32;
// One child-manifest sentence is reserved for the bounded global snapshot.
const MAX_SENTENCES: usize = TINY_CANARY_MAX_SENTENCES - 1;
const MAX_LEXICAL_UNITS_PER_SENTENCE: usize = TINY_CANARY_MAX_ROUTES_PER_SENTENCE;
const MAX_LEXICAL_UNITS: usize = 512;
const MAX_VOCABULARY: usize = 256;
const MAX_SOURCE_BYTES: usize = 64 * 1024;
const MAX_GLOBAL_SNAPSHOT_UNITS: usize = 64;
const MAX_IDENTITY_SCOPE_BYTES: usize = 256;
const MAX_GLOBAL_EPOCH_BYTES: usize = 256;
const MAX_TURN_ID_BYTES: usize = 256;
const CHILD_MAX_CANDIDATES: u16 = 8;
const COMPILER_NAME: &str = "uor-r4-core::canonical_lexical_ingestion/1";
const COMPILER_IDENTITY_BYTES: &[u8] = b"uor-r4 canonical lexical ingestion compiler/1";
const ZETA_SUMMARY_CHANNELS: [u16; 8] = [0, 1, 2, 3, 5, 8, 13, 21];
const QUARTER_TURN_Q29: i32 = 843_314_857;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalLexicalError {
    Invalid(String),
    UnknownUnit {
        turn: usize,
        paragraph: usize,
        offset: usize,
        surface_hex: String,
    },
    Serialization(String),
    Addressing(String),
    ArithmeticOverflow,
    PrimeRoute(String),
}

impl std::fmt::Display for CanonicalLexicalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::UnknownUnit {
                turn,
                paragraph,
                offset,
                surface_hex,
            } => write!(
                formatter,
                "unknown lexical unit at turn {turn}, paragraph {paragraph}, byte {offset}: {surface_hex}"
            ),
            Self::Serialization(reason) => write!(formatter, "canonical serialization: {reason}"),
            Self::Addressing(reason) => write!(formatter, "canonical addressing: {reason}"),
            Self::ArithmeticOverflow => formatter.write_str("canonical ingestion arithmetic overflow"),
            Self::PrimeRoute(reason) => write!(formatter, "prime-route child artifact: {reason}"),
        }
    }
}

impl std::error::Error for CanonicalLexicalError {}

impl From<PrimeRouteError> for CanonicalLexicalError {
    fn from(error: PrimeRouteError) -> Self {
        Self::PrimeRoute(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationInput {
    pub identity_scope: String,
    pub global_epoch: String,
    pub global_snapshot_units: Vec<Vec<u8>>,
    pub turns: Vec<TurnInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnInput {
    pub turn_id: String,
    pub paragraphs: Vec<ParagraphInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphInput {
    /// Ordered sentence byte slices. Concatenating them reconstructs the exact
    /// paragraph bytes; each slice owns any boundary whitespace it contains.
    pub sentences: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedLexicalUnit {
    pub unit_id: u32,
    pub leading_bytes: Vec<u8>,
    pub span_start: u32,
    pub span_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedParagraph {
    pub units: Vec<EncodedLexicalUnit>,
    pub trailing_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalLexicalCodec {
    profile: LexicalCodecProfile,
    vocabulary: Vec<VocabularyBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LexicalCodecProfile {
    schema: u32,
    domain: String,
    family: String,
    version: u32,
    normalization: String,
    unit_boundary: String,
    boundary_bytes: String,
    sentence_boundary: String,
    paragraph_boundary: String,
    turn_boundary: String,
    unknown_unit_policy: String,
    special_unit_policy: String,
    unicode_version: (u8, u8, u8),
    vocabulary_kappa: String,
    codec_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct VocabularyBinding {
    unit_id: u32,
    surface_hex: String,
    payload_cid: String,
}

/// The additive schema-1 registry for every codec unit. Its stable unit-ID
/// order is deliberately independent of the frozen schema-2 child's observed
/// address vector, whose indexes may change as a causal prefix grows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LexicalRouteAddressBinding {
    lexical_unit_id: u32,
    prime: u32,
    payload_cid: String,
    s3_spin_q30: [i32; 4],
    hopf_observation_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    radial_zphi: ZPhiWire,
    address_kappa: String,
}

#[derive(Serialize)]
struct CodecIdentityWire<'a> {
    schema: u32,
    domain: &'a str,
    family: &'a str,
    version: u32,
    normalization: &'a str,
    unit_boundary: &'a str,
    boundary_bytes: &'a str,
    sentence_boundary: &'a str,
    paragraph_boundary: &'a str,
    turn_boundary: &'a str,
    unknown_unit_policy: &'a str,
    special_unit_policy: &'a str,
    unicode_version: (u8, u8, u8),
    vocabulary_kappa: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawSegment {
    leading: Vec<u8>,
    surface: Vec<u8>,
    start: usize,
    end: usize,
}

impl CanonicalLexicalCodec {
    pub fn compile(input: &ConversationInput) -> Result<Self, CanonicalLexicalError> {
        validate_input_shape(input)?;
        let mut surfaces = BTreeSet::<Vec<u8>>::new();
        for global_unit in &input.global_snapshot_units {
            let (segments, _) = segment_paragraph(global_unit)?;
            for segment in segments {
                surfaces.insert(segment.surface);
            }
        }
        for turn in &input.turns {
            for paragraph in &turn.paragraphs {
                for sentence in &paragraph.sentences {
                    let (segments, _) = segment_paragraph(sentence)?;
                    for segment in segments {
                        surfaces.insert(segment.surface);
                    }
                }
            }
        }
        if surfaces.is_empty() || surfaces.len() > MAX_VOCABULARY {
            return Err(CanonicalLexicalError::Invalid(format!(
                "lexical vocabulary must contain 1..={MAX_VOCABULARY} units"
            )));
        }
        let vocabulary = surfaces
            .into_iter()
            .enumerate()
            .map(|(unit_id, surface)| {
                Ok(VocabularyBinding {
                    unit_id: u32::try_from(unit_id)
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                    surface_hex: hex::encode(&surface),
                    payload_cid: canonical_kappa(&surface)?,
                })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        let vocabulary_kappa = canonical_kappa(&canonical_json(&vocabulary)?)?;
        let mut profile = LexicalCodecProfile {
            schema: LEXICAL_CODEC_SCHEMA,
            domain: LEXICAL_CODEC_DOMAIN.to_owned(),
            family: "unicode-scalar lexical runs".to_owned(),
            version: 1,
            normalization: "none; input UTF-8 bytes are identity-preserved".to_owned(),
            unit_boundary:
                "maximal Unicode alphanumeric-or-underscore run; otherwise one scalar".to_owned(),
            boundary_bytes: "leading Unicode whitespace retained per occurrence; trailing retained per declared sentence slice".to_owned(),
            sentence_boundary: "caller-declared ordered sentence byte slices; punctuation remains a lexical unit".to_owned(),
            paragraph_boundary: "caller-declared ordered groups of sentence byte slices".to_owned(),
            turn_boundary: "caller-declared ordered identity-scoped turns".to_owned(),
            unknown_unit_policy: "reject transaction before hierarchy mutation".to_owned(),
            special_unit_policy: "none".to_owned(),
            unicode_version: std::char::UNICODE_VERSION,
            vocabulary_kappa,
            codec_kappa: String::new(),
        };
        profile.codec_kappa = codec_identity_kappa(&profile)?;
        let codec = Self {
            profile,
            vocabulary,
        };
        codec.validate()?;
        Ok(codec)
    }

    pub fn codec_kappa(&self) -> &str {
        &self.profile.codec_kappa
    }

    pub fn vocabulary_kappa(&self) -> &str {
        &self.profile.vocabulary_kappa
    }

    pub fn encode(
        &self,
        turn: usize,
        paragraph: usize,
        bytes: &[u8],
    ) -> Result<EncodedParagraph, CanonicalLexicalError> {
        self.validate()?;
        let (segments, trailing_bytes) = segment_paragraph(bytes)?;
        let by_surface = self
            .vocabulary
            .iter()
            .map(|entry| (entry.surface_hex.as_str(), entry.unit_id))
            .collect::<BTreeMap<_, _>>();
        let mut units = Vec::with_capacity(segments.len());
        for segment in segments {
            let surface_hex = hex::encode(&segment.surface);
            let unit_id = by_surface
                .get(surface_hex.as_str())
                .copied()
                .ok_or_else(|| CanonicalLexicalError::UnknownUnit {
                    turn,
                    paragraph,
                    offset: segment.start,
                    surface_hex: surface_hex.clone(),
                })?;
            units.push(EncodedLexicalUnit {
                unit_id,
                leading_bytes: segment.leading,
                span_start: u32::try_from(segment.start)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                span_end: u32::try_from(segment.end)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            });
        }
        if units.is_empty() {
            return Err(CanonicalLexicalError::Invalid(
                "each S0 paragraph requires at least one lexical unit".to_owned(),
            ));
        }
        Ok(EncodedParagraph {
            units,
            trailing_bytes,
        })
    }

    pub fn decode(&self, encoded: &EncodedParagraph) -> Result<Vec<u8>, CanonicalLexicalError> {
        self.validate()?;
        let by_id = self
            .vocabulary
            .iter()
            .map(|entry| (entry.unit_id, entry))
            .collect::<BTreeMap<_, _>>();
        let mut decoded = Vec::new();
        for unit in &encoded.units {
            let binding = by_id.get(&unit.unit_id).ok_or_else(|| {
                CanonicalLexicalError::Invalid(format!(
                    "lexical inverse refused unknown unit ID {}",
                    unit.unit_id
                ))
            })?;
            decoded.extend_from_slice(&unit.leading_bytes);
            decoded.extend_from_slice(&decode_hex(&binding.surface_hex, "vocabulary surface")?);
        }
        decoded.extend_from_slice(&encoded.trailing_bytes);
        let reencoded = self.encode(usize::MAX, usize::MAX, &decoded)?;
        if &reencoded != encoded {
            return Err(CanonicalLexicalError::Invalid(
                "lexical inverse refused non-canonical IDs, boundary bytes, or spans".to_owned(),
            ));
        }
        Ok(decoded)
    }

    fn binding(&self, unit_id: u32) -> Option<&VocabularyBinding> {
        self.vocabulary
            .binary_search_by_key(&unit_id, |entry| entry.unit_id)
            .ok()
            .and_then(|index| self.vocabulary.get(index))
    }

    fn validate(&self) -> Result<(), CanonicalLexicalError> {
        if self.profile.schema != LEXICAL_CODEC_SCHEMA
            || self.profile.domain != LEXICAL_CODEC_DOMAIN
            || self.profile.family != "unicode-scalar lexical runs"
            || self.profile.version != 1
            || self.profile.normalization != "none; input UTF-8 bytes are identity-preserved"
            || self.profile.unit_boundary
                != "maximal Unicode alphanumeric-or-underscore run; otherwise one scalar"
            || self.profile.boundary_bytes
                != "leading Unicode whitespace retained per occurrence; trailing retained per declared sentence slice"
            || self.profile.sentence_boundary
                != "caller-declared ordered sentence byte slices; punctuation remains a lexical unit"
            || self.profile.paragraph_boundary
                != "caller-declared ordered groups of sentence byte slices"
            || self.profile.turn_boundary
                != "caller-declared ordered identity-scoped turns"
            || self.profile.unknown_unit_policy != "reject transaction before hierarchy mutation"
            || self.profile.special_unit_policy != "none"
            || self.profile.unicode_version != std::char::UNICODE_VERSION
        {
            return Err(CanonicalLexicalError::Invalid(
                "lexical codec profile is unsupported".to_owned(),
            ));
        }
        if self.vocabulary.is_empty() || self.vocabulary.len() > MAX_VOCABULARY {
            return Err(CanonicalLexicalError::Invalid(
                "lexical vocabulary is empty or exceeds its bound".to_owned(),
            ));
        }
        let mut prior_surface: Option<Vec<u8>> = None;
        for (index, entry) in self.vocabulary.iter().enumerate() {
            if entry.unit_id
                != u32::try_from(index).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
            {
                return Err(CanonicalLexicalError::Invalid(
                    "lexical vocabulary IDs are not contiguous canonical order".to_owned(),
                ));
            }
            let surface = decode_hex(&entry.surface_hex, "vocabulary surface")?;
            if surface.is_empty() || canonical_kappa(&surface)? != entry.payload_cid {
                return Err(CanonicalLexicalError::Invalid(
                    "lexical vocabulary payload CID does not reproduce".to_owned(),
                ));
            }
            let (segments, trailing) = segment_paragraph(&surface)?;
            if segments.len() != 1
                || !segments[0].leading.is_empty()
                || segments[0].surface != surface
                || segments[0].start != 0
                || segments[0].end != surface.len()
                || !trailing.is_empty()
            {
                return Err(CanonicalLexicalError::Invalid(
                    "lexical vocabulary surface is not one boundary-free codec unit".to_owned(),
                ));
            }
            if prior_surface
                .as_ref()
                .is_some_and(|prior| prior >= &surface)
            {
                return Err(CanonicalLexicalError::Invalid(
                    "lexical vocabulary is not in strict surface-byte order".to_owned(),
                ));
            }
            prior_surface = Some(surface);
        }
        let vocabulary_kappa = canonical_kappa(&canonical_json(&self.vocabulary)?)?;
        if vocabulary_kappa != self.profile.vocabulary_kappa
            || codec_identity_kappa(&self.profile)? != self.profile.codec_kappa
        {
            return Err(CanonicalLexicalError::Invalid(
                "lexical codec identity does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }
}

impl CanonicalRouteArtifact {
    fn validate_hierarchy(
        &self,
        route_by_kappa: &BTreeMap<&str, &RouteRecord>,
        provenance_kappa: &str,
    ) -> Result<(), CanonicalLexicalError> {
        if self.body.hierarchy_nodes.is_empty()
            || self
                .body
                .hierarchy_nodes
                .windows(2)
                .any(|pair| pair[0].node_kappa >= pair[1].node_kappa)
        {
            return Err(CanonicalLexicalError::Invalid(
                "hierarchy nodes are not in strict identity order".to_owned(),
            ));
        }
        let node_by_kappa = self
            .body
            .hierarchy_nodes
            .iter()
            .map(|node| (node.node_kappa.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        for node in &self.body.hierarchy_nodes {
            let body = &node.body;
            validate_kappa_label(&node.node_kappa, "hierarchy node kappa")?;
            if body.schema != HIERARCHY_NODE_SCHEMA
                || body.domain != HIERARCHY_NODE_DOMAIN
                || body.identity_scope != self.body.provenance.identity_scope
                || body.provenance_kappa != provenance_kappa
                || body.span_start >= body.span_end
                || body.boundary_identity.is_empty()
                || body.chain_identity.is_empty()
            {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy node header, provenance, or span is invalid".to_owned(),
                ));
            }
            let (expected_child_kind, child_scope, previous_allowed, expected_boundary) =
                match body.scope.as_str() {
                    "local" => ("route", None, true, "rolling-last-two"),
                    "sentence" => ("route", None, true, "codec-declared-sentence"),
                    "paragraph" => (
                        "sentence-node",
                        Some("sentence"),
                        true,
                        "declared-paragraph",
                    ),
                    "conversation" => (
                        "paragraph-node",
                        Some("paragraph"),
                        true,
                        "observed-turn-paragraph",
                    ),
                    "global" => (
                        "global-snapshot",
                        None,
                        false,
                        "immutable-bounded-global-epoch",
                    ),
                    _ => {
                        return Err(CanonicalLexicalError::Invalid(
                            "hierarchy node scope is unsupported".to_owned(),
                        ));
                    }
                };
            if body.ordered_child_kind != expected_child_kind
                || body.boundary_kind != expected_boundary
                || (!previous_allowed && body.previous_chain_kappa.is_some())
                || (matches!(body.scope.as_str(), "local" | "conversation")
                    && body.chain_identity != body.identity_scope)
                || (body.scope == "global"
                    && (body.chain_identity != self.body.global_snapshot.snapshot_kappa
                        || body.boundary_identity != self.body.global_snapshot.snapshot_kappa))
            {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy node child, boundary, or chain mode is invalid".to_owned(),
                ));
            }
            let previous = body
                .previous_chain_kappa
                .as_deref()
                .map(|kappa| {
                    node_by_kappa.get(kappa).copied().ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "hierarchy previous-chain reference is absent".to_owned(),
                        )
                    })
                })
                .transpose()?;
            if previous.is_some_and(|prior| {
                prior.body.scope != body.scope
                    || prior.body.identity_scope != body.identity_scope
                    || prior.body.chain_identity != body.chain_identity
                    || prior.body.child_count.checked_add(1) != Some(body.child_count)
            }) {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy previous-chain scope or count is invalid".to_owned(),
                ));
            }
            let (child_summary, child_span_start, child_span_end) = if expected_child_kind
                == "route"
            {
                let route = route_by_kappa
                    .get(body.ordered_child_kappa.as_str())
                    .copied()
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "hierarchy route child reference is absent".to_owned(),
                        )
                    })?;
                (
                    route_summary(route)?,
                    route.body.occurrence,
                    route
                        .body
                        .occurrence
                        .checked_add(1)
                        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
                )
            } else if expected_child_kind == "global-snapshot" {
                if body.ordered_child_kappa != self.body.global_snapshot.snapshot_kappa {
                    return Err(CanonicalLexicalError::Invalid(
                        "global root does not reference the bound immutable snapshot".to_owned(),
                    ));
                }
                (
                    self.body.global_snapshot.summary.clone(),
                    0,
                    u32::try_from(self.body.global_snapshot.ordered_units.len())
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                )
            } else {
                let child = node_by_kappa
                    .get(body.ordered_child_kappa.as_str())
                    .copied()
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "hierarchy child-node reference is absent".to_owned(),
                        )
                    })?;
                if Some(child.body.scope.as_str()) != child_scope
                    || child.body.identity_scope != body.identity_scope
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "hierarchy child-node scope or identity is invalid".to_owned(),
                    ));
                }
                (
                    child.body.summary.clone(),
                    child.body.span_start,
                    child.body.span_end,
                )
            };
            let expected_count = previous
                .map_or(0, |prior| prior.body.child_count)
                .checked_add(1)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            let expected_span_start =
                previous.map_or(child_span_start, |prior| prior.body.span_start);
            if body.child_count != expected_count
                || body.span_start != expected_span_start
                || body.span_end != child_span_end
            {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy count or monotonic span does not reproduce".to_owned(),
                ));
            }
            let expected_exact = canonical_kappa(&canonical_json(&ExactChainWire {
                schema: HIERARCHY_NODE_SCHEMA,
                domain: HIERARCHY_NODE_DOMAIN,
                scope: &body.scope,
                identity_scope: &body.identity_scope,
                previous_chain_kappa: body.previous_chain_kappa.as_deref(),
                ordered_child_kappa: &body.ordered_child_kappa,
                ordered_child_kind: &body.ordered_child_kind,
                child_count: body.child_count,
                boundary_kind: &body.boundary_kind,
                boundary_identity: &body.boundary_identity,
                chain_identity: &body.chain_identity,
            })?)?;
            let expected_summary = combine_summary(
                previous.map(|prior| &prior.body.summary),
                &child_summary,
                &body.ordered_child_kappa,
            )?;
            let expected_summary_kappa = canonical_kappa(&canonical_json(&expected_summary)?)?;
            if body.exact_chain_kappa != expected_exact
                || body.summary != expected_summary
                || body.summary_kappa != expected_summary_kappa
                || body.payload_or_summary_cid != expected_summary_kappa
                || body.bridge_mode != body.summary.state.trig_chart.bridge_mode
                || body.exact_chain_kappa == body.summary_kappa
                || canonical_kappa(&canonical_json(body)?)? != node.node_kappa
            {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy exact identity or geometric summary does not reproduce".to_owned(),
                ));
            }
        }

        let routes = &self.body.route_records;
        if routes.len() < 2
            || self.body.hierarchy_roots.local.current
                != routes
                    .last()
                    .map(|record| record.route_kappa.as_str())
                    .unwrap_or_default()
            || self.body.hierarchy_roots.local.previous
                != routes
                    .get(routes.len() - 2)
                    .map(|record| record.route_kappa.as_str())
                    .unwrap_or_default()
        {
            return Err(CanonicalLexicalError::Invalid(
                "local current/previous roots are not the final ordered routes".to_owned(),
            ));
        }
        let root_specs = [
            (self.body.hierarchy_roots.local.last_two.as_str(), "local"),
            (self.body.hierarchy_roots.sentence.as_str(), "sentence"),
            (self.body.hierarchy_roots.paragraph.as_str(), "paragraph"),
            (
                self.body.hierarchy_roots.conversation.as_str(),
                "conversation",
            ),
            (self.body.hierarchy_roots.global.as_str(), "global"),
        ];
        for (kappa, scope) in root_specs {
            let node = node_by_kappa.get(kappa).copied().ok_or_else(|| {
                CanonicalLexicalError::Invalid("hierarchy root reference is absent".to_owned())
            })?;
            if node.body.scope != scope {
                return Err(CanonicalLexicalError::Invalid(
                    "hierarchy root resolves to the wrong scope".to_owned(),
                ));
            }
        }
        let local = node_by_kappa[&self.body.hierarchy_roots.local.last_two.as_str()];
        let local_previous = local
            .body
            .previous_chain_kappa
            .as_deref()
            .and_then(|kappa| node_by_kappa.get(kappa).copied())
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid("last-two root lacks its previous route".to_owned())
            })?;
        if local.body.child_count != 2
            || local.body.ordered_child_kappa != self.body.hierarchy_roots.local.current
            || local_previous.body.child_count != 1
            || local_previous.body.previous_chain_kappa.is_some()
            || local_previous.body.ordered_child_kappa != self.body.hierarchy_roots.local.previous
        {
            return Err(CanonicalLexicalError::Invalid(
                "last-two root does not bind exactly current and previous".to_owned(),
            ));
        }
        let sentence = node_by_kappa[&self.body.hierarchy_roots.sentence.as_str()];
        let paragraph = node_by_kappa[&self.body.hierarchy_roots.paragraph.as_str()];
        let conversation = node_by_kappa[&self.body.hierarchy_roots.conversation.as_str()];
        let global = node_by_kappa[&self.body.hierarchy_roots.global.as_str()];
        if paragraph.body.ordered_child_kappa != sentence.node_kappa
            || conversation.body.ordered_child_kappa != paragraph.node_kappa
            || global.body.ordered_child_kappa != self.body.global_snapshot.snapshot_kappa
            || global.body.previous_chain_kappa.is_some()
        {
            return Err(CanonicalLexicalError::Invalid(
                "hierarchy terminal roots are not transitively linked".to_owned(),
            ));
        }

        let mut pending = root_specs
            .into_iter()
            .map(|(kappa, _)| kappa)
            .collect::<Vec<_>>();
        let mut reached_nodes = BTreeSet::new();
        let mut reached_routes = BTreeSet::new();
        while let Some(kappa) = pending.pop() {
            if !reached_nodes.insert(kappa) {
                continue;
            }
            let node = node_by_kappa.get(kappa).copied().ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "hierarchy traversal found an absent node".to_owned(),
                )
            })?;
            if let Some(previous) = node.body.previous_chain_kappa.as_deref() {
                pending.push(previous);
            }
            if node.body.ordered_child_kind == "route" {
                reached_routes.insert(node.body.ordered_child_kappa.as_str());
            } else if node.body.ordered_child_kind != "global-snapshot" {
                pending.push(node.body.ordered_child_kappa.as_str());
            }
        }
        if reached_nodes.len() != self.body.hierarchy_nodes.len()
            || reached_routes.len() != self.body.route_records.len()
            || reached_routes
                .iter()
                .any(|kappa| !route_by_kappa.contains_key(*kappa))
        {
            return Err(CanonicalLexicalError::Invalid(
                "hierarchy has an orphan node or does not reach every route".to_owned(),
            ));
        }
        Ok(())
    }
}

fn codec_identity_kappa(profile: &LexicalCodecProfile) -> Result<String, CanonicalLexicalError> {
    canonical_kappa(&canonical_json(&CodecIdentityWire {
        schema: profile.schema,
        domain: &profile.domain,
        family: &profile.family,
        version: profile.version,
        normalization: &profile.normalization,
        unit_boundary: &profile.unit_boundary,
        boundary_bytes: &profile.boundary_bytes,
        sentence_boundary: &profile.sentence_boundary,
        paragraph_boundary: &profile.paragraph_boundary,
        turn_boundary: &profile.turn_boundary,
        unknown_unit_policy: &profile.unknown_unit_policy,
        special_unit_policy: &profile.special_unit_policy,
        unicode_version: profile.unicode_version,
        vocabulary_kappa: &profile.vocabulary_kappa,
    })?)
}

fn segment_paragraph(bytes: &[u8]) -> Result<(Vec<RawSegment>, Vec<u8>), CanonicalLexicalError> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        CanonicalLexicalError::Invalid(format!("lexical input is not valid UTF-8: {error}"))
    })?;
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let mut boundary_start = 0usize;
    while cursor < bytes.len() {
        let character = text[cursor..]
            .chars()
            .next()
            .ok_or_else(|| CanonicalLexicalError::Invalid("UTF-8 cursor is invalid".to_owned()))?;
        if character.is_whitespace() {
            cursor = cursor
                .checked_add(character.len_utf8())
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            continue;
        }
        let leading = bytes[boundary_start..cursor].to_vec();
        let start = cursor;
        let word = character.is_alphanumeric() || character == '_';
        cursor = cursor
            .checked_add(character.len_utf8())
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        if word {
            while cursor < bytes.len() {
                let next = text[cursor..].chars().next().ok_or_else(|| {
                    CanonicalLexicalError::Invalid("UTF-8 word cursor is invalid".to_owned())
                })?;
                if !(next.is_alphanumeric() || next == '_') {
                    break;
                }
                cursor = cursor
                    .checked_add(next.len_utf8())
                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            }
        }
        segments.push(RawSegment {
            leading,
            surface: bytes[start..cursor].to_vec(),
            start,
            end: cursor,
        });
        boundary_start = cursor;
    }
    Ok((segments, bytes[boundary_start..].to_vec()))
}

/// Split UTF-8 text with the canonical lexical-run rule while making every
/// returned piece independently decodable.
///
/// Each non-whitespace run owns the whitespace immediately before it. A
/// trailing whitespace-only suffix is its own final piece. Concatenating the
/// returned pieces therefore reproduces `bytes` exactly. This is the small,
/// geometry-free segmentation surface used by the source-free table baseline;
/// it deliberately reuses the codec's established Unicode boundary rule.
pub fn canonical_lexical_piece_bytes(bytes: &[u8]) -> Result<Vec<Vec<u8>>, CanonicalLexicalError> {
    let (segments, trailing) = segment_paragraph(bytes)?;
    let mut pieces = Vec::with_capacity(segments.len() + usize::from(!trailing.is_empty()));
    for segment in segments {
        let mut piece = segment.leading;
        piece.extend_from_slice(&segment.surface);
        pieces.push(piece);
    }
    if !trailing.is_empty() {
        pieces.push(trailing);
    }
    Ok(pieces)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ContentBlob {
    cid: String,
    kind: String,
    bytes_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SpinManifestBinding {
    schema: u32,
    domain: String,
    blob_cid: String,
    manifest_kappa: String,
    base_bridge_tag: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ChartAdapterSpec {
    adapter: String,
    marker: String,
    units: String,
    direction: String,
    exactness: String,
    maximum_error_q30: u32,
    conversion_cost: u16,
    inverse_fidelity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TypedChartProfile {
    schema: u32,
    domain: String,
    deterministic_selection: String,
    tangent_pole_rule: String,
    null_sentinel_rule: String,
    bridge_expression: String,
    bridge_modes: [String; 2],
    bridge_assignment_rule: String,
    adapters: Vec<ChartAdapterSpec>,
    profile_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ChartTransitionWitness {
    profile_kappa: String,
    bridge_mode: String,
    bridge_output: u8,
    sin_q30: i32,
    cos_q30: i32,
    activation_q30: u32,
    chirality: i8,
    cosine_polarity: i8,
    source_chart: String,
    active_chart: String,
    tangent_evaluated: bool,
    quarter_turn_orientation: i8,
    phase_shift_q29: i32,
    torsion_shift_q29: i32,
    transported_fiber_q29: i32,
    transported_torsion_q29: i32,
    inverse_fiber_q29: i32,
    inverse_torsion_q29: i32,
    selected_adapter: String,
    exact_inverse: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ZPhiWire {
    a: i64,
    b: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct H4RootTable {
    schema: u32,
    domain: String,
    coordinate_scale: String,
    basis_order: [String; 4],
    roots: Vec<[ZPhiWire; 4]>,
    table_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IcosianOperatorRow {
    operator: String,
    coefficient_matrix: [[i64; 2]; 2],
    inverse_operator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IcosianOperatorTable {
    schema: u32,
    domain: String,
    coefficient_basis: [String; 2],
    rows: Vec<IcosianOperatorRow>,
    table_kappa: String,
}

impl From<ZPhi> for ZPhiWire {
    fn from(value: ZPhi) -> Self {
        Self {
            a: value.a,
            b: value.b,
        }
    }
}

impl From<ZPhiWire> for ZPhi {
    fn from(value: ZPhiWire) -> Self {
        Self::new(value.a, value.b)
    }
}

fn lexical_route_address_binding(
    lexical_unit_id: u32,
    address: &GeometricAddress,
) -> Result<LexicalRouteAddressBinding, CanonicalLexicalError> {
    Ok(LexicalRouteAddressBinding {
        lexical_unit_id,
        prime: address.atom.value(),
        payload_cid: address.payload_cid.clone(),
        s3_spin_q30: address.spin.s3.raw(),
        hopf_observation_q30: address.spin.hopf.raw(),
        fiber_q29: address.spin.fiber.raw(),
        torsion_q29: address.spin.torsion.raw(),
        radial_zphi: address.radial.into(),
        address_kappa: address.canonical_kappa()?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IcosianProfile {
    schema: u32,
    domain: String,
    project_shorthand: String,
    normative_construction: String,
    module_identity: String,
    quaternion_basis: [String; 4],
    z_basis_order: [String; 8],
    glue_parity_rule: String,
    golden_conjugation: String,
    turyn_norm: String,
    turyn_to_e8_norm: String,
    scale: String,
    shell_membership: String,
    orientation: String,
    coset: String,
    root_order: String,
    h4_root_table: H4RootTable,
    operator_table: IcosianOperatorTable,
    collision_policy: String,
    forward_map: String,
    inverse_map: String,
    profile_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IcosianCoordinateWitness {
    profile_kappa: String,
    selected_h4_root_index: u16,
    e8_basis_coordinates: [i64; 8],
    h4: [i64; 4],
    phi_h4: [i64; 4],
    zphi_quaternion: [ZPhiWire; 4],
    galois_companion: [ZPhiWire; 4],
    phi_galois_companion: [ZPhiWire; 4],
    turyn_norm_zphi: ZPhiWire,
    reconstructed_e8_basis_coordinates: [i64; 8],
    coordinate_kappa: String,
    inverse_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RouteRecordBody {
    schema: u32,
    domain: String,
    occurrence: u32,
    turn: u16,
    paragraph: u16,
    sentence: u16,
    ordinal_in_sentence: u16,
    span_start: u32,
    span_end: u32,
    leading_bytes_hex: String,
    lexical_unit_id: u32,
    payload_cid: String,
    prime: u32,
    address_index: u16,
    address_kappa: String,
    zeta_phase_signature_q29: [i32; 8],
    s3_spin_q30: [i32; 4],
    hopf_observation_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    radial_zphi: ZPhiWire,
    chart: ChartTransitionWitness,
    icosian: IcosianCoordinateWitness,
    shared_class_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RouteRecord {
    route_kappa: String,
    body: RouteRecordBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SentenceRebuildWitness {
    sentence_index: u16,
    route_kappas: Vec<String>,
    trailing_bytes_hex: String,
    source_cid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ParagraphRebuildWitness {
    turn_id: String,
    paragraph_index: u16,
    sentences: Vec<SentenceRebuildWitness>,
    source_cid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct PrimeCount {
    prime: u32,
    count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GeometricStateSnapshot {
    prime_factors: Vec<PrimeCount>,
    zeta_phase_signature_q29: [i32; 8],
    s3_spin_q30: [i32; 4],
    s2_hopf_observation_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    radial_zphi: ZPhiWire,
    trig_chart: ChartTransitionWitness,
    cross_domain_cost_profile_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TrajectorySummary {
    schema: u32,
    domain: String,
    observed_children: u32,
    session_hypersphere_q30: [i64; 4],
    winding_turns: i64,
    window_start: u32,
    window_end: u32,
    projection_energy_q30: u64,
    shared_prime_factors: Vec<PrimeCount>,
    cosine_resonance_q30: [i64; 8],
    accumulated_hopf_phase_q29: i32,
    paired_h4_e8_coordinate_sum: [i64; 8],
    paired_h4_e8_coordinate_kappa: String,
    transported_trajectory_kappa: String,
    state: GeometricStateSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HierarchyNodeBody {
    schema: u32,
    domain: String,
    scope: String,
    identity_scope: String,
    bridge_mode: String,
    previous_chain_kappa: Option<String>,
    ordered_child_kappa: String,
    ordered_child_kind: String,
    child_count: u32,
    span_start: u32,
    span_end: u32,
    boundary_kind: String,
    boundary_identity: String,
    chain_identity: String,
    exact_chain_kappa: String,
    summary_kappa: String,
    summary: TrajectorySummary,
    payload_or_summary_cid: String,
    provenance_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HierarchyNode {
    node_kappa: String,
    body: HierarchyNodeBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LocalRoots {
    current: String,
    previous: String,
    last_two: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HierarchyRoots {
    local: LocalRoots,
    sentence: String,
    paragraph: String,
    conversation: String,
    global: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SharedSpinTorsionClass {
    class_kappa: String,
    s3_spin_q30: [i32; 4],
    hopf_observation_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    ordered_route_members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GlobalSnapshotUnitBinding {
    ordinal: u16,
    lexical_unit_id: u32,
    address_index: u16,
    address_kappa: String,
    entry_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GlobalSnapshotBinding {
    schema: u32,
    domain: String,
    snapshot_kappa: String,
    ordered_units: Vec<GlobalSnapshotUnitBinding>,
    summary: TrajectorySummary,
    summary_kappa: String,
}

#[derive(Serialize)]
struct GlobalSnapshotEntryWire<'a> {
    schema: u32,
    domain: &'static str,
    snapshot_kappa: &'a str,
    ordinal: u16,
    lexical_unit_id: u32,
    address_kappa: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactBounds {
    maximum_turns: u16,
    maximum_paragraphs: u16,
    maximum_sentences: u16,
    maximum_lexical_units_per_sentence: u16,
    maximum_lexical_units: u16,
    maximum_vocabulary: u16,
    maximum_source_bytes: u32,
    maximum_artifact_bytes: u32,
    maximum_identity_scope_bytes: u16,
    maximum_global_epoch_bytes: u16,
    maximum_turn_id_bytes: u16,
    maximum_global_snapshot_units: u16,
    global_epoch_policy: String,
}

/// Fixed state and query-work ceilings for one hierarchy scope. S0 carries no
/// continuation rows yet, so every candidate ceiling is explicitly zero until
/// #952 supplies the separately measured recursive-attention operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeCeiling {
    pub scope: String,
    pub maximum_open_children: u16,
    pub maximum_rows_per_query: u16,
    pub maximum_candidates_per_row: u16,
    pub maximum_retained_candidates: u16,
    pub maximum_patch_or_epoch_depth: u16,
    pub overflow_policy: String,
    pub query_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactProvenance {
    compiler: String,
    compiler_cid: String,
    source_cid: String,
    codec_kappa: String,
    cost_profile_cid: String,
    identity_scope: String,
    global_epoch: String,
    source_weights_opened: bool,
    teacher_forwards: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct HierarchyProvenance {
    schema: u32,
    domain: String,
    compiler_cid: String,
    codec_kappa: String,
    chart_profile_kappa: String,
    icosian_profile_kappa: String,
    identity_scope: String,
    global_epoch: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactBody {
    schema: u32,
    domain: String,
    codec: LexicalCodecProfile,
    vocabulary: Vec<VocabularyBinding>,
    lexical_route_addresses: Vec<LexicalRouteAddressBinding>,
    content_blobs: Vec<ContentBlob>,
    spin_manifest: SpinManifestBinding,
    chart_profile: TypedChartProfile,
    icosian_profile: IcosianProfile,
    route_records: Vec<RouteRecord>,
    paragraph_witnesses: Vec<ParagraphRebuildWitness>,
    global_snapshot: GlobalSnapshotBinding,
    hierarchy_nodes: Vec<HierarchyNode>,
    hierarchy_roots: HierarchyRoots,
    shared_classes: Vec<SharedSpinTorsionClass>,
    bounds: ArtifactBounds,
    scope_ceilings: Vec<ScopeCeiling>,
    hierarchy_provenance: HierarchyProvenance,
    provenance: ArtifactProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArtifactEnvelope {
    schema: u32,
    domain: String,
    manifest_kappa: String,
    body: ArtifactBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalRouteArtifact {
    manifest_kappa: String,
    body: ArtifactBody,
}

/// Exact lexical value recovered from one registered geometric address.
///
/// The payload bytes are the codec unit itself; occurrence boundary bytes
/// remain owned by route records. This is the narrow selected-address value
/// interface required by A1 and the later source-free inference stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LexicalRouteValueView {
    pub lexical_unit_id: u32,
    /// Index in the complete parent codec registry. This is not the index of
    /// the same address in the embedded schema-2 manifest's observed subset.
    pub registry_address_index: u16,
    pub prime: u32,
    pub address_kappa: String,
    pub payload_cid: String,
    pub payload_bytes: Vec<u8>,
}

/// Exact multiplication table derived from the concrete scaled 120-root H4
/// table embedded in S0. Products and inverses are indexes into that fixed
/// root order; no floating point or approximate nearest-root lookup is used.
///
/// This key is only an opaque address into the exact finite table. Its numeric
/// value and numeric differences between keys have no geometric meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct OpaqueH4TableIndex(u16);

impl OpaqueH4TableIndex {
    /// Construct one checked lookup key. The offset is an addressing detail,
    /// not a scalar coordinate or a distance.
    pub fn from_table_offset(offset: u16, table: &H4BinaryIcosahedralClosure) -> Option<Self> {
        (usize::from(offset) < table.root_count).then_some(Self(offset))
    }

    /// Return the row/column offset needed for bounded table lookup only.
    /// Arithmetic on this value is not an admissible geometric operation.
    pub const fn table_offset(self) -> u16 {
        self.0
    }
}

/// One exact scaled quaternion in the fixed H4 root order. Each pair is the
/// `(1, phi)` coefficient and the quaternion basis order is `(1, i, j, k)`;
/// all coordinates are multiplied by two as bound by the root-table kappa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct H4RootCoordinate {
    pub scaled_zphi_quaternion: [[i64; 2]; 4],
}

/// Smallest exact group element used by the associative ordered fold. Route
/// counts belong to hierarchy/cursor diagnostics, not to this algebraic value,
/// so inverse and commutator construction cannot manufacture false counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct OrderedH4FoldState {
    opaque_table_index: OpaqueH4TableIndex,
}

impl OrderedH4FoldState {
    pub(crate) fn from_table_index(
        index: OpaqueH4TableIndex,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CanonicalLexicalError> {
        validate_ordered_h4_table_shape(table)?;
        if usize::from(index.table_offset()) >= table.root_count {
            return Err(CanonicalLexicalError::Invalid(
                "ordered H4 state index is outside the bound table".to_owned(),
            ));
        }
        Ok(Self {
            opaque_table_index: index,
        })
    }

    pub(crate) fn identity(
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CanonicalLexicalError> {
        let index = OpaqueH4TableIndex::from_table_offset(table.identity_index, table).ok_or_else(
            || {
                CanonicalLexicalError::Invalid(
                    "ordered H4 identity is outside the bound table".to_owned(),
                )
            },
        )?;
        Self::from_table_index(index, table)
    }

    /// Exact `self * right` in the declared left-to-right operand order.
    pub(crate) fn compose(
        self,
        right: Self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CanonicalLexicalError> {
        let product = table
            .product_state_index(self.opaque_table_index, right.opaque_table_index)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "ordered H4 composition addressed outside the bound table".to_owned(),
                )
            })?;
        Self::from_table_index(product, table)
    }

    pub(crate) fn inverse(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CanonicalLexicalError> {
        let inverse = table
            .inverse_state_index(self.opaque_table_index)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "ordered H4 inverse addressed outside the bound table".to_owned(),
                )
            })?;
        Self::from_table_index(inverse, table)
    }

    pub(crate) const fn table_index(self) -> OpaqueH4TableIndex {
        self.opaque_table_index
    }

    pub(crate) fn root_coordinate(
        self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<H4RootCoordinate, CanonicalLexicalError> {
        h4_root_coordinate(self.opaque_table_index, table)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct H4BinaryIcosahedralClosure {
    pub schema: u32,
    pub domain: String,
    pub h4_root_table_kappa: String,
    pub root_count: usize,
    pub product_count: usize,
    pub identity_index: u16,
    pub inverse_indices: Vec<u16>,
    /// Row-major products: `left * root_count + right` names the index of the
    /// exact quaternion product in the fixed canonical root order.
    pub multiplication_indices: Vec<u16>,
    pub multiplication_table_kappa: String,
    pub unique_closure_exact: bool,
    pub identity_exact: bool,
    pub inverses_exact: bool,
    pub associativity_exact: bool,
    pub integer_only_no_rounding: bool,
}

impl H4BinaryIcosahedralClosure {
    /// Read one exact row-major product without constructing a quaternion at
    /// runtime. Out-of-range indexes return `None`.
    pub fn product_index(&self, left: u16, right: u16) -> Option<u16> {
        let left = usize::from(left);
        let right = usize::from(right);
        if left >= self.root_count || right >= self.root_count {
            return None;
        }
        left.checked_mul(self.root_count)
            .and_then(|base| base.checked_add(right))
            .and_then(|offset| self.multiplication_indices.get(offset))
            .copied()
    }

    /// Typed counterpart to [`Self::product_index`]. Keys remain opaque table
    /// addresses and are never interpreted as distances.
    pub fn product_state_index(
        &self,
        left: OpaqueH4TableIndex,
        right: OpaqueH4TableIndex,
    ) -> Option<OpaqueH4TableIndex> {
        self.product_index(left.table_offset(), right.table_offset())
            .and_then(|offset| OpaqueH4TableIndex::from_table_offset(offset, self))
    }

    pub fn inverse_state_index(&self, state: OpaqueH4TableIndex) -> Option<OpaqueH4TableIndex> {
        self.inverse_indices
            .get(usize::from(state.table_offset()))
            .copied()
            .and_then(|offset| OpaqueH4TableIndex::from_table_offset(offset, self))
    }

    /// Reproduce the table identity using the canonical self-cleared seed.
    pub fn reproduce_multiplication_table_kappa(&self) -> Result<String, CanonicalLexicalError> {
        h4_multiplication_table_kappa(self)
    }
}

/// Map one already-registered lexical address through S0's frozen
/// `prime mod 120` H4-root assignment. No payload, digest, or table-key
/// distance participates.
pub(crate) fn h4_leaf_state_for_address(
    address: &GeometricAddress,
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, CanonicalLexicalError> {
    h4_leaf_state_for_prime(address.atom.value(), table)
}

/// One ordered fold state attached to a fixed attention hierarchy level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionOrderedFoldLevel {
    pub level: String,
    pub observed_routes: u32,
    pub state: OrderedH4FoldState,
    pub root_coordinate: H4RootCoordinate,
    /// The complete consumer level with the same ordered algebraic state
    /// attached directly. This avoids a caller-side join between identity /
    /// additive fields and the A1R ordered-state overlay.
    pub consumer_level: AttentionLevelTrace,
}

/// Versioned non-digest state attached to one [`AttentionLevelTrace`] only by
/// the A1R overlay. Legacy S0/#952 traces leave this absent, preserving their
/// canonical bytes and identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionOrderedH4StateTrace {
    pub schema: u32,
    pub domain: String,
    pub observed_routes: u32,
    pub state: OrderedH4FoldState,
    pub root_coordinate: H4RootCoordinate,
}

/// Versioned overlay over a frozen S0 artifact. This is intentionally not a
/// field on [`CanonicalRouteArtifact`] or [`AttentionConsumerTrace`], so their
/// canonical bytes and identities remain unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionOrderedFoldTrace {
    pub schema: u32,
    pub domain: String,
    pub overlay_kappa: String,
    pub source_artifact_manifest_kappa: String,
    pub h4_root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub leaf_assignment: String,
    pub composition_order: String,
    pub ordered_levels: Vec<AttentionOrderedFoldLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionHierarchyView {
    pub current: String,
    pub previous: String,
    pub last_two: String,
    pub sentence: String,
    pub paragraph: String,
    pub conversation: String,
    pub global: String,
}

pub const ATTENTION_CONSUMER_TRACE_SCHEMA: u32 = 1;
pub const ATTENTION_CONSUMER_TRACE_DOMAIN: &str = "uor-r4.canonical-attention-consumer-trace/1";

/// The exact, ordered S0 handoff consumed by recursive attention in #952.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionConsumerTrace {
    pub schema: u32,
    pub domain: String,
    pub artifact_manifest_kappa: String,
    pub embedded_spin_manifest_kappa: String,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub chart_profile_kappa: String,
    pub icosian_profile_kappa: String,
    pub h4_root_table_kappa: String,
    pub icosian_operator_table_kappa: String,
    pub global_snapshot_kappa: String,
    pub scope_ceilings: Vec<ScopeCeiling>,
    pub ordered_levels: Vec<AttentionLevelTrace>,
}

/// One immutable route or hierarchy input to the seven-level attention order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionLevelTrace {
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_h4: Option<AttentionOrderedH4StateTrace>,
    pub identity_kind: String,
    pub identity_kappa: String,
    pub occurrence: Option<u32>,
    pub turn: Option<u16>,
    pub paragraph: Option<u16>,
    pub sentence: Option<u16>,
    pub ordinal_in_sentence: Option<u16>,
    pub lexical_unit_id: Option<u32>,
    pub prime: Option<u32>,
    pub address_index: Option<u16>,
    pub boundary_kind: Option<String>,
    pub boundary_identity: Option<String>,
    pub chain_identity: Option<String>,
    pub exact_chain_kappa: String,
    pub geometric_summary_kappa: String,
    pub previous_identity_kappa: Option<String>,
    pub ordered_child_kappa: String,
    pub direct_child_count: u32,
    pub observed_descendant_routes: u32,
    pub window_start: u32,
    pub window_end: u32,
    pub session_hypersphere_q30: [i64; 4],
    pub winding_turns: i64,
    pub projection_energy_q30: u64,
    pub shared_prime_factors: Vec<AttentionPrimeFactorTrace>,
    pub cosine_resonance_q30: [i64; 8],
    pub accumulated_hopf_phase_q29: i32,
    pub zeta_phase_signature_q29: [i32; 8],
    pub s3_spin_q30: [i32; 4],
    pub s2_hopf_observation_q30: [i32; 3],
    pub fiber_q29: i32,
    pub torsion_q29: i32,
    pub radial_zphi: [i64; 2],
    pub bridge_mode: String,
    pub active_chart: String,
    pub selected_adapter: String,
    pub chart_sin_q30: i32,
    pub chart_cos_q30: i32,
    pub chart_activation_q30: u32,
    pub chart_chirality: i8,
    pub chart_cosine_polarity: i8,
    pub quarter_turn_orientation: i8,
    pub phase_shift_q29: i32,
    pub torsion_shift_q29: i32,
    pub transported_fiber_q29: i32,
    pub transported_torsion_q29: i32,
    pub inverse_fiber_q29: i32,
    pub inverse_torsion_q29: i32,
    pub chart_inverse_exact: bool,
    pub payload_cid: Option<String>,
    pub address_kappa: Option<String>,
    pub shared_class_kappa: Option<String>,
    pub paired_h4_e8_coordinate_sum: [i64; 8],
    pub paired_h4_e8_coordinate_kappa: String,
    pub transported_trajectory_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionPrimeFactorTrace {
    pub prime: u32,
    pub count: u32,
}

pub const INCREMENTAL_ATTENTION_CONSUMER_TRACE_SCHEMA: u32 = 1;
pub const INCREMENTAL_ATTENTION_CONSUMER_TRACE_DOMAIN: &str =
    "uor-r4.incremental-attention-consumer-trace/1";

/// A causal, possibly partial seven-level view. Every slot is present in the
/// fixed consumer order; `None` means that boundary has not yet established
/// the corresponding state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalAttentionConsumerTrace {
    pub schema: u32,
    pub domain: String,
    pub artifact_manifest_kappa: String,
    pub embedded_spin_manifest_kappa: String,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub chart_profile_kappa: String,
    pub icosian_profile_kappa: String,
    pub h4_root_table_kappa: String,
    pub icosian_operator_table_kappa: String,
    pub global_snapshot_kappa: String,
    pub scope_ceilings: Vec<ScopeCeiling>,
    pub ordered_levels: Vec<IncrementalAttentionLevelSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalAttentionLevelSlot {
    pub level: String,
    pub trace: Option<AttentionLevelTrace>,
}

pub const INCREMENTAL_UPDATE_TRACE_SCHEMA: u32 = 1;
pub const INCREMENTAL_UPDATE_TRACE_DOMAIN: &str = "uor-r4.incremental-hierarchy-update-trace/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalHierarchyTrace {
    pub schema: u32,
    pub domain: String,
    pub events: Vec<IncrementalHierarchyEvent>,
    pub maximum_changed_states_per_event: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalHierarchyEvent {
    pub event_index: u32,
    pub event_kind: String,
    pub source_identity_kappa: String,
    pub changed_scopes: Vec<String>,
    pub resulting_identity_kappas: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalHierarchyState {
    pub current_route: Option<String>,
    pub previous_route: Option<String>,
    pub last_two_identity_kappa: Option<String>,
    pub sentence_root: Option<String>,
    pub paragraph_root: Option<String>,
    pub conversation_root: Option<String>,
    pub global_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalHierarchyChange {
    pub scope: String,
    pub before_identity_kappa: Option<String>,
    pub after_identity_kappa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncrementalHierarchyDelta {
    pub event_index: u32,
    pub event_kind: String,
    pub source_identity_kappa: String,
    pub changed_nodes: Vec<IncrementalHierarchyChange>,
}

#[derive(Debug, Clone)]
pub struct IncrementalHierarchyCursor {
    artifact_manifest_kappa: String,
    events: Vec<IncrementalHierarchyEvent>,
    next_event: usize,
    state: IncrementalHierarchyState,
}

impl IncrementalHierarchyCursor {
    pub fn state(&self) -> &IncrementalHierarchyState {
        &self.state
    }

    pub fn remaining_events(&self) -> usize {
        self.events.len().saturating_sub(self.next_event)
    }

    /// Apply one causal source event. The returned delta contains one or two
    /// changed state nodes; all other roots remain byte-for-byte unchanged.
    pub fn apply_next(
        &mut self,
    ) -> Result<Option<IncrementalHierarchyDelta>, CanonicalLexicalError> {
        let Some(event) = self.events.get(self.next_event).cloned() else {
            return Ok(None);
        };
        if usize::try_from(event.event_index)
            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
            != self.next_event
        {
            return Err(CanonicalLexicalError::Invalid(
                "incremental event indexes are not contiguous".to_owned(),
            ));
        }
        let next_event = self
            .next_event
            .checked_add(1)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        let mut changed_nodes = Vec::new();
        match event.event_kind.as_str() {
            "observe-lexical-unit" => {
                if event.changed_scopes != ["local", "sentence"]
                    || event.resulting_identity_kappas.len() != 2
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "lexical event has an unsupported state-delta shape".to_owned(),
                    ));
                }
                let prior_local = self.state.last_two_identity_kappa.clone();
                let prior_sentence = self.state.sentence_root.clone();
                self.state.previous_route = self.state.current_route.clone();
                self.state.current_route = Some(event.source_identity_kappa.clone());
                let local_identity = event.resulting_identity_kappas[0].clone();
                self.state.last_two_identity_kappa = Some(local_identity.clone());
                self.state.sentence_root = Some(event.resulting_identity_kappas[1].clone());
                changed_nodes.push(IncrementalHierarchyChange {
                    scope: "local".to_owned(),
                    before_identity_kappa: prior_local,
                    after_identity_kappa: local_identity,
                });
                changed_nodes.push(IncrementalHierarchyChange {
                    scope: "sentence".to_owned(),
                    before_identity_kappa: prior_sentence,
                    after_identity_kappa: event.resulting_identity_kappas[1].clone(),
                });
            }
            "close-sentence" => {
                if event.changed_scopes != ["paragraph"]
                    || event.resulting_identity_kappas.len() != 1
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "sentence-close event has an unsupported state-delta shape".to_owned(),
                    ));
                }
                let before = self.state.paragraph_root.clone();
                self.state.paragraph_root = Some(event.resulting_identity_kappas[0].clone());
                changed_nodes.push(IncrementalHierarchyChange {
                    scope: "paragraph".to_owned(),
                    before_identity_kappa: before,
                    after_identity_kappa: event.resulting_identity_kappas[0].clone(),
                });
            }
            "close-paragraph" => {
                if event.changed_scopes != ["conversation"]
                    || event.resulting_identity_kappas.len() != 1
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "paragraph-close event has an unsupported state-delta shape".to_owned(),
                    ));
                }
                let before = self.state.conversation_root.clone();
                self.state.conversation_root = Some(event.resulting_identity_kappas[0].clone());
                changed_nodes.push(IncrementalHierarchyChange {
                    scope: "conversation".to_owned(),
                    before_identity_kappa: before,
                    after_identity_kappa: event.resulting_identity_kappas[0].clone(),
                });
            }
            "publish-global-epoch" => {
                if event.changed_scopes != ["global"]
                    || event.resulting_identity_kappas.len() != 1
                    || self.state.global_root.is_some()
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "global event has an unsupported immutable-epoch delta".to_owned(),
                    ));
                }
                self.state.global_root = Some(event.resulting_identity_kappas[0].clone());
                changed_nodes.push(IncrementalHierarchyChange {
                    scope: "global".to_owned(),
                    before_identity_kappa: None,
                    after_identity_kappa: event.resulting_identity_kappas[0].clone(),
                });
            }
            _ => {
                return Err(CanonicalLexicalError::Invalid(
                    "incremental event kind is unsupported".to_owned(),
                ));
            }
        }
        if changed_nodes.is_empty() || changed_nodes.len() > 2 {
            return Err(CanonicalLexicalError::Invalid(
                "incremental cursor exceeded its one-or-two-node delta contract".to_owned(),
            ));
        }
        self.next_event = next_event;
        Ok(Some(IncrementalHierarchyDelta {
            event_index: event.event_index,
            event_kind: event.event_kind,
            source_identity_kappa: event.source_identity_kappa,
            changed_nodes,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeServingBoundary {
    pub source_model_weights_opened: bool,
    pub teacher_forwards: u64,
    pub transformer_calls: u64,
    pub source_attention_calls: u64,
    pub dense_intelligence_matrix_calls: u64,
    pub moe_calls: u64,
    pub ollama_calls: u64,
    pub hosted_provider_calls: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeWitness {
    pub schema: u32,
    pub domain: String,
    pub verdict: String,
    pub artifact_schema: u32,
    pub artifact_domain: String,
    pub manifest_kappa_before_reload: String,
    pub manifest_kappa_after_reload: String,
    pub canonical_bytes_cid: String,
    pub canonical_bytes_len: usize,
    pub canonical_bytes_identical: bool,
    pub referenced_kappas_identical: bool,
    pub codec_kappa: String,
    pub vocabulary_kappa: String,
    pub lexical_units: usize,
    pub unique_payload_blobs: usize,
    pub reconstructed_paragraphs_hex: Vec<String>,
    pub lexical_reconstruction_exact: bool,
    pub unknown_unit_status: String,
    pub unknown_unit_surface_hex: String,
    pub p_squared_semiprime_present: bool,
    pub embedded_spin_manifest_kappa: String,
    pub chart_profile_kappa: String,
    pub icosian_profile_kappa: String,
    pub h4_root_table_kappa: String,
    pub icosian_operator_table_kappa: String,
    pub global_snapshot_kappa: String,
    pub scope_ceilings: Vec<ScopeCeiling>,
    pub attention_consumer_order: Vec<String>,
    pub attention_consumer_contract_exact: bool,
    pub hierarchy_views: AttentionHierarchyView,
    pub all_hierarchy_roots_present: bool,
    pub transitive_references_present: bool,
    pub maximum_nodes_changed_per_event: u8,
    pub incremental_cursor_maximum_changed_nodes: u8,
    pub incremental_cursor_final_state_exact: bool,
    pub incremental_attention_states_exact: bool,
    pub incremental_prefix_routes_stable: bool,
    pub incremental_prefix_closed_hierarchy_stable: bool,
    pub exact_identity_kappa: String,
    pub geometric_summary_kappa: String,
    pub exact_identity_distinct_from_summary: bool,
    pub shared_class_kappa: String,
    pub shared_class_expected_members: usize,
    pub shared_class_lookup_members: usize,
    pub shared_class_lookup_reaches_all: bool,
    pub bridge_modes_present: Vec<String>,
    pub tangent_pole_quarter_turn_present: bool,
    pub icosian_inverse_witnesses_exact: bool,
    pub serving_boundary: ProbeServingBoundary,
}

#[derive(Serialize)]
struct SpinClassKeyWire {
    schema: u32,
    domain: &'static str,
    s3_spin_q30: [i32; 4],
    hopf_observation_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
}

#[derive(Serialize)]
struct SourceWire<'a> {
    schema: u32,
    domain: &'static str,
    identity_scope: &'a str,
    global_epoch: &'a str,
    global_snapshot_units_hex: Vec<String>,
    turns: Vec<SourceTurnWire<'a>>,
}

#[derive(Serialize)]
struct GlobalEpochWire {
    schema: u32,
    domain: &'static str,
    ordered_lexical_units_hex: Vec<String>,
}

#[derive(Serialize)]
struct SourceTurnWire<'a> {
    turn_id: &'a str,
    paragraphs: Vec<SourceParagraphWire>,
}

#[derive(Serialize)]
struct SourceParagraphWire {
    sentences_hex: Vec<String>,
}

#[derive(Serialize)]
struct ExactChainWire<'a> {
    schema: u32,
    domain: &'static str,
    scope: &'a str,
    identity_scope: &'a str,
    previous_chain_kappa: Option<&'a str>,
    ordered_child_kappa: &'a str,
    ordered_child_kind: &'a str,
    child_count: u32,
    boundary_kind: &'a str,
    boundary_identity: &'a str,
    chain_identity: &'a str,
}

fn fixed_scope_ceilings() -> Result<Vec<ScopeCeiling>, CanonicalLexicalError> {
    let state_only = "NOT_IMPLEMENTED_S0_STATE_ONLY".to_owned();
    Ok(vec![
        ScopeCeiling {
            scope: "local".to_owned(),
            maximum_open_children: 2,
            maximum_rows_per_query: 0,
            maximum_candidates_per_row: 0,
            maximum_retained_candidates: 0,
            maximum_patch_or_epoch_depth: 2,
            overflow_policy: "evict oldest route after current-plus-previous".to_owned(),
            query_status: state_only.clone(),
        },
        ScopeCeiling {
            scope: "sentence".to_owned(),
            maximum_open_children: u16::try_from(MAX_LEXICAL_UNITS_PER_SENTENCE)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            maximum_rows_per_query: 0,
            maximum_candidates_per_row: 0,
            maximum_retained_candidates: 0,
            maximum_patch_or_epoch_depth: u16::try_from(MAX_LEXICAL_UNITS_PER_SENTENCE)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            overflow_policy: "reject before mutation or close at declared sentence boundary"
                .to_owned(),
            query_status: state_only.clone(),
        },
        ScopeCeiling {
            scope: "paragraph".to_owned(),
            maximum_open_children: u16::try_from(MAX_SENTENCES)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            maximum_rows_per_query: 0,
            maximum_candidates_per_row: 0,
            maximum_retained_candidates: 0,
            maximum_patch_or_epoch_depth: u16::try_from(MAX_SENTENCES)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            overflow_policy: "reject before mutation or close at declared paragraph boundary"
                .to_owned(),
            query_status: state_only.clone(),
        },
        ScopeCeiling {
            scope: "conversation".to_owned(),
            maximum_open_children: u16::try_from(MAX_PARAGRAPHS)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            maximum_rows_per_query: 0,
            maximum_candidates_per_row: 0,
            maximum_retained_candidates: 0,
            maximum_patch_or_epoch_depth: u16::try_from(MAX_PARAGRAPHS)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            overflow_policy: "reject input above bounded turn/paragraph population".to_owned(),
            query_status: state_only.clone(),
        },
        ScopeCeiling {
            scope: "global".to_owned(),
            maximum_open_children: u16::try_from(MAX_GLOBAL_SNAPSHOT_UNITS)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            maximum_rows_per_query: 0,
            maximum_candidates_per_row: 0,
            maximum_retained_candidates: 0,
            maximum_patch_or_epoch_depth: 1,
            overflow_policy: "replace only with a new bounded content-addressed epoch".to_owned(),
            query_status: state_only,
        },
    ])
}

fn fixed_chart_profile() -> Result<TypedChartProfile, CanonicalLexicalError> {
    let mut profile = TypedChartProfile {
        schema: CHART_PROFILE_SCHEMA,
        domain: CHART_PROFILE_DOMAIN.to_owned(),
        deterministic_selection:
            "minimum conversion_cost among exact adapters, then adapter name".to_owned(),
        tangent_pole_rule:
            "cos_q30=0 selects complementary cotangent chart and signed quarter turn; tangent is never divided"
                .to_owned(),
        null_sentinel_rule:
            "(sin_q30,cos_q30)=(0,0) is typed null and requires prior orientation; it is not an angle"
                .to_owned(),
        bridge_expression: "e^(i*pi) + pi^0 =_bridge 0^0".to_owned(),
        bridge_modes: [
            "continuous-null:0".to_owned(),
            "discrete-empty-product:1".to_owned(),
        ],
        bridge_assignment_rule:
            "schema-2 base manifest uses continuous-null; schema-1 lexical occurrences use lexical-unit-ID parity and bind the selected typed mode"
                .to_owned(),
        adapters: vec![
            ChartAdapterSpec {
                adapter: "complex-discrete".to_owned(),
                marker: "2i".to_owned(),
                units: "oriented complex displacement; magnitude 2".to_owned(),
                direction: "signed".to_owned(),
                exactness: "exact typed landmark".to_owned(),
                maximum_error_q30: 0,
                conversion_cost: 1,
                inverse_fidelity: "exact sign and phase recovery".to_owned(),
            },
            ChartAdapterSpec {
                adapter: "euclidean-chord".to_owned(),
                marker: "sqrt(2)".to_owned(),
                units: "unit-sphere chord".to_owned(),
                direction: "orientation retained separately".to_owned(),
                exactness: "declared orthogonal-unit marker".to_owned(),
                maximum_error_q30: 1,
                conversion_cost: 2,
                inverse_fidelity: "Q1.30 bounded reconstruction".to_owned(),
            },
            ChartAdapterSpec {
                adapter: "riemannian-chord-interval".to_owned(),
                marker: "[0,2]".to_owned(),
                units: "normalized chord distance, not raw geodesic angle".to_owned(),
                direction: "orientation retained in route state".to_owned(),
                exactness: "Q1.30 bounded interval".to_owned(),
                maximum_error_q30: 1,
                conversion_cost: 3,
                inverse_fidelity: "bounded by one Q1.30 unit".to_owned(),
            },
        ],
        profile_kappa: String::new(),
    };
    profile.profile_kappa = profile_kappa(&profile)?;
    Ok(profile)
}

fn profile_kappa<T>(profile: &T) -> Result<String, CanonicalLexicalError>
where
    T: Serialize + Clone + ClearProfileKappa,
{
    let mut seed = profile.clone();
    seed.clear_profile_kappa();
    canonical_kappa(&canonical_json(&seed)?)
}

trait ClearProfileKappa {
    fn clear_profile_kappa(&mut self);
}

impl ClearProfileKappa for TypedChartProfile {
    fn clear_profile_kappa(&mut self) {
        self.profile_kappa.clear();
    }
}

impl ClearProfileKappa for IcosianProfile {
    fn clear_profile_kappa(&mut self) {
        self.profile_kappa.clear();
    }
}

fn fixed_h4_root_table() -> Result<H4RootTable, CanonicalLexicalError> {
    let zero = ZPhiWire { a: 0, b: 0 };
    let mut roots = BTreeSet::<[ZPhiWire; 4]>::new();
    for axis in 0..4 {
        for sign in [-1i64, 1] {
            let mut root = [zero; 4];
            root[axis] = ZPhiWire { a: 2 * sign, b: 0 };
            roots.insert(root);
        }
    }
    for signs in 0u8..16 {
        let mut root = [zero; 4];
        for (axis, coordinate) in root.iter_mut().enumerate() {
            let sign = if signs & (1 << axis) == 0 { -1 } else { 1 };
            *coordinate = ZPhiWire { a: sign, b: 0 };
        }
        roots.insert(root);
    }
    let base = [
        zero,
        ZPhiWire { a: 1, b: 0 },
        ZPhiWire { a: 0, b: 1 },
        ZPhiWire { a: -1, b: 1 },
    ];
    for first in 0..4 {
        for second in 0..4 {
            for third in 0..4 {
                for fourth in 0..4 {
                    let permutation = [first, second, third, fourth];
                    if permutation.iter().copied().collect::<BTreeSet<_>>().len() != 4 {
                        continue;
                    }
                    let inversions = (0..4)
                        .flat_map(|left| ((left + 1)..4).map(move |right| (left, right)))
                        .filter(|(left, right)| permutation[*left] > permutation[*right])
                        .count();
                    if !inversions.is_multiple_of(2) {
                        continue;
                    }
                    for signs in 0u8..8 {
                        let mut signed = base;
                        for (source, coordinate) in signed.iter_mut().enumerate().skip(1) {
                            if signs & (1 << (source - 1)) == 0 {
                                coordinate.a = coordinate
                                    .a
                                    .checked_neg()
                                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
                                coordinate.b = coordinate
                                    .b
                                    .checked_neg()
                                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
                            }
                        }
                        roots.insert(permutation.map(|source| signed[source]));
                    }
                }
            }
        }
    }
    let roots = roots.into_iter().collect::<Vec<_>>();
    if roots.len() != 120 {
        return Err(CanonicalLexicalError::Invalid(format!(
            "fixed H4 root table produced {} roots instead of 120",
            roots.len()
        )));
    }
    let mut table = H4RootTable {
        schema: 1,
        domain: "uor-r4.icosian-h4-root-table/1".to_owned(),
        coordinate_scale: "coordinates are multiplied by two in Z[phi]".to_owned(),
        basis_order: ["1", "i", "j", "k"].map(str::to_owned),
        roots,
        table_kappa: String::new(),
    };
    let mut seed = table.clone();
    seed.table_kappa.clear();
    table.table_kappa = canonical_kappa(&canonical_json(&seed)?)?;
    Ok(table)
}

fn validate_ordered_h4_table_shape(
    table: &H4BinaryIcosahedralClosure,
) -> Result<(), CanonicalLexicalError> {
    if table.schema != H4_BINARY_ICOSAHEDRAL_TABLE_SCHEMA
        || table.domain != H4_BINARY_ICOSAHEDRAL_TABLE_DOMAIN
        || table.root_count != 120
        || table.product_count != table.root_count * table.root_count
        || table.multiplication_indices.len() != table.product_count
        || table.inverse_indices.len() != table.root_count
        || usize::from(table.identity_index) >= table.root_count
        || table.h4_root_table_kappa != H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE
        || table.multiplication_table_kappa
            != H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE
        || !table.unique_closure_exact
        || !table.identity_exact
        || !table.inverses_exact
        || !table.associativity_exact
        || !table.integer_only_no_rounding
    {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 fold requires the validated exact 120-root table".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_ordered_h4_table_exact(
    table: &H4BinaryIcosahedralClosure,
) -> Result<(), CanonicalLexicalError> {
    validate_ordered_h4_table_shape(table)?;
    if h4_multiplication_table_kappa(table)?
        != H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE
    {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 fold table bytes do not reproduce the frozen exact table identity"
                .to_owned(),
        ));
    }
    Ok(())
}

fn h4_root_coordinate(
    index: OpaqueH4TableIndex,
    table: &H4BinaryIcosahedralClosure,
) -> Result<H4RootCoordinate, CanonicalLexicalError> {
    validate_ordered_h4_table_shape(table)?;
    let roots = fixed_h4_root_table()?;
    if roots.table_kappa != table.h4_root_table_kappa {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 fold table does not bind the canonical root order".to_owned(),
        ));
    }
    let root = roots
        .roots
        .get(usize::from(index.table_offset()))
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid(
                "ordered H4 root coordinate index is out of range".to_owned(),
            )
        })?;
    Ok(H4RootCoordinate {
        scaled_zphi_quaternion: root.map(|coordinate| [coordinate.a, coordinate.b]),
    })
}

fn h4_leaf_state_for_prime(
    prime: u32,
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldState, CanonicalLexicalError> {
    validate_ordered_h4_table_shape(table)?;
    let root_count =
        u32::try_from(table.root_count).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    let offset =
        u16::try_from(prime % root_count).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    let index = OpaqueH4TableIndex::from_table_offset(offset, table).ok_or_else(|| {
        CanonicalLexicalError::Invalid(
            "ordered H4 leaf assignment addressed outside the bound table".to_owned(),
        )
    })?;
    OrderedH4FoldState::from_table_index(index, table)
}

fn zphi_checked_neg(value: ZPhi) -> Result<ZPhi, CanonicalLexicalError> {
    Ok(ZPhi::new(
        value
            .a
            .checked_neg()
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
        value
            .b
            .checked_neg()
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
    ))
}

fn h4_product_coordinate(
    left: &[ZPhi; 4],
    right: &[ZPhi; 4],
    terms: [(usize, usize, i8); 4],
) -> Result<ZPhi, CanonicalLexicalError> {
    terms
        .into_iter()
        .try_fold(ZPhi::new(0, 0), |total, (left_index, right_index, sign)| {
            let product = left[left_index].checked_mul(right[right_index])?;
            let signed = match sign {
                1 => product,
                -1 => zphi_checked_neg(product)?,
                _ => {
                    return Err(CanonicalLexicalError::Invalid(
                        "H4 multiplication contains a non-unit sign".to_owned(),
                    ));
                }
            };
            Ok(total.checked_add(signed)?)
        })
}

/// Multiply two H4 roots whose stored coordinates are scaled by two.
///
/// If `q` and `r` are the stored quaternions, their represented unit elements
/// are `q/2` and `r/2`; the stored product is therefore `(q*r)/2`. Exact
/// coefficient divisibility is required in both `1` and `phi` components.
fn multiply_scaled_h4_roots(
    left: [ZPhiWire; 4],
    right: [ZPhiWire; 4],
) -> Result<[ZPhiWire; 4], CanonicalLexicalError> {
    let left = left.map(ZPhi::from);
    let right = right.map(ZPhi::from);
    let raw = [
        h4_product_coordinate(
            &left,
            &right,
            [(0, 0, 1), (1, 1, -1), (2, 2, -1), (3, 3, -1)],
        )?,
        h4_product_coordinate(&left, &right, [(0, 1, 1), (1, 0, 1), (2, 3, 1), (3, 2, -1)])?,
        h4_product_coordinate(&left, &right, [(0, 2, 1), (1, 3, -1), (2, 0, 1), (3, 1, 1)])?,
        h4_product_coordinate(&left, &right, [(0, 3, 1), (1, 2, 1), (2, 1, -1), (3, 0, 1)])?,
    ];
    let mut scaled = [ZPhiWire { a: 0, b: 0 }; 4];
    for (target, coordinate) in scaled.iter_mut().zip(raw) {
        if coordinate.a % 2 != 0 || coordinate.b % 2 != 0 {
            return Err(CanonicalLexicalError::Invalid(
                "scaled H4 product is not exactly divisible by two in Z[phi]".to_owned(),
            ));
        }
        *target = ZPhiWire {
            a: coordinate.a / 2,
            b: coordinate.b / 2,
        };
    }
    Ok(scaled)
}

/// Exhaustively derive and validate the exact binary-icosahedral group table
/// for S0's concrete scaled H4 root order.
pub fn validate_h4_binary_icosahedral_closure(
) -> Result<H4BinaryIcosahedralClosure, CanonicalLexicalError> {
    let roots = fixed_h4_root_table()?;
    let root_count = roots.roots.len();
    if root_count != 120 || roots.roots.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(CanonicalLexicalError::Invalid(
            "fixed H4 roots are not 120 unique canonical elements".to_owned(),
        ));
    }

    let identity = [
        ZPhiWire { a: 2, b: 0 },
        ZPhiWire { a: 0, b: 0 },
        ZPhiWire { a: 0, b: 0 },
        ZPhiWire { a: 0, b: 0 },
    ];
    let identity_index = roots.roots.binary_search(&identity).map_err(|_| {
        CanonicalLexicalError::Invalid("fixed H4 table has no exact identity root".to_owned())
    })?;
    let product_count = root_count
        .checked_mul(root_count)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let mut multiplication_indices = Vec::with_capacity(product_count);
    for left in &roots.roots {
        for right in &roots.roots {
            let product = multiply_scaled_h4_roots(*left, *right)?;
            let index = roots.roots.binary_search(&product).map_err(|_| {
                CanonicalLexicalError::Invalid(
                    "concrete scaled H4 table is not closed under exact quaternion multiplication"
                        .to_owned(),
                )
            })?;
            multiplication_indices
                .push(u16::try_from(index).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?);
        }
    }
    let product_index = |left: usize, right: usize| -> Result<usize, CanonicalLexicalError> {
        let offset = left
            .checked_mul(root_count)
            .and_then(|base| base.checked_add(right))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        multiplication_indices
            .get(offset)
            .copied()
            .map(usize::from)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "H4 multiplication-table index is out of range".to_owned(),
                )
            })
    };

    for index in 0..root_count {
        if product_index(identity_index, index)? != index
            || product_index(index, identity_index)? != index
        {
            return Err(CanonicalLexicalError::Invalid(
                "concrete scaled H4 identity law failed".to_owned(),
            ));
        }
    }

    let mut inverse_indices = Vec::with_capacity(root_count);
    for index in 0..root_count {
        let mut inverses = Vec::new();
        for candidate in 0..root_count {
            if product_index(index, candidate)? == identity_index
                && product_index(candidate, index)? == identity_index
            {
                inverses.push(candidate);
            }
        }
        if inverses.len() != 1 {
            return Err(CanonicalLexicalError::Invalid(format!(
                "concrete scaled H4 root {index} has {} exact two-sided inverses",
                inverses.len()
            )));
        }
        inverse_indices.push(
            u16::try_from(inverses[0]).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
        );
    }

    for left in 0..root_count {
        for middle in 0..root_count {
            for right in 0..root_count {
                let left_associated = product_index(product_index(left, middle)?, right)?;
                let right_associated = product_index(left, product_index(middle, right)?)?;
                if left_associated != right_associated {
                    return Err(CanonicalLexicalError::Invalid(format!(
                        "concrete scaled H4 associativity failed at ({left},{middle},{right})"
                    )));
                }
            }
        }
    }

    let mut report = H4BinaryIcosahedralClosure {
        schema: H4_BINARY_ICOSAHEDRAL_TABLE_SCHEMA,
        domain: H4_BINARY_ICOSAHEDRAL_TABLE_DOMAIN.to_owned(),
        h4_root_table_kappa: roots.table_kappa,
        root_count,
        product_count,
        identity_index: u16::try_from(identity_index)
            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
        inverse_indices,
        multiplication_indices,
        multiplication_table_kappa: String::new(),
        unique_closure_exact: true,
        identity_exact: true,
        inverses_exact: true,
        associativity_exact: true,
        integer_only_no_rounding: true,
    };
    report.multiplication_table_kappa = h4_multiplication_table_kappa(&report)?;
    Ok(report)
}

fn h4_multiplication_table_kappa(
    report: &H4BinaryIcosahedralClosure,
) -> Result<String, CanonicalLexicalError> {
    let mut seed = report.clone();
    seed.multiplication_table_kappa.clear();
    canonical_kappa(&canonical_json(&seed)?)
}

fn fixed_icosian_operator_table() -> Result<IcosianOperatorTable, CanonicalLexicalError> {
    let mut table = IcosianOperatorTable {
        schema: 1,
        domain: "uor-r4.icosian-coefficient-operators/1".to_owned(),
        coefficient_basis: ["1", "phi"].map(str::to_owned),
        rows: vec![
            IcosianOperatorRow {
                operator: "identity".to_owned(),
                coefficient_matrix: [[1, 0], [0, 1]],
                inverse_operator: "identity".to_owned(),
            },
            IcosianOperatorRow {
                operator: "golden-conjugation".to_owned(),
                coefficient_matrix: [[1, 1], [0, -1]],
                inverse_operator: "golden-conjugation".to_owned(),
            },
            IcosianOperatorRow {
                operator: "multiply-phi".to_owned(),
                coefficient_matrix: [[0, 1], [1, 1]],
                inverse_operator: "multiply-phi-inverse".to_owned(),
            },
            IcosianOperatorRow {
                operator: "multiply-phi-inverse".to_owned(),
                coefficient_matrix: [[-1, 1], [1, 0]],
                inverse_operator: "multiply-phi".to_owned(),
            },
            IcosianOperatorRow {
                operator: "phi-galois-companion".to_owned(),
                coefficient_matrix: [[0, -1], [1, 0]],
                inverse_operator: "inverse-phi-galois-companion".to_owned(),
            },
            IcosianOperatorRow {
                operator: "inverse-phi-galois-companion".to_owned(),
                coefficient_matrix: [[0, 1], [-1, 0]],
                inverse_operator: "phi-galois-companion".to_owned(),
            },
        ],
        table_kappa: String::new(),
    };
    let mut seed = table.clone();
    seed.table_kappa.clear();
    table.table_kappa = canonical_kappa(&canonical_json(&seed)?)?;
    Ok(table)
}

fn fixed_icosian_profile() -> Result<IcosianProfile, CanonicalLexicalError> {
    let h4_root_table = fixed_h4_root_table()?;
    let operator_table = fixed_icosian_operator_table()?;
    let mut profile = IcosianProfile {
        schema: ICOSIAN_PROFILE_SCHEMA,
        domain: ICOSIAN_PROFILE_DOMAIN.to_owned(),
        project_shorthand: "E8 = H4 x H4".to_owned(),
        normative_construction: "icosian golden-coupled H4 ⊕ phiH4".to_owned(),
        module_identity: "Lambda_E8 ~=_Z I".to_owned(),
        quaternion_basis: ["1", "i", "j", "k"].map(str::to_owned),
        z_basis_order: [
            "h4.1", "h4.i", "h4.j", "h4.k", "phi_h4.1", "phi_h4.i", "phi_h4.j", "phi_h4.k",
        ]
        .map(str::to_owned),
        glue_parity_rule: "icosian membership is exact membership in the bound 120-root H4 table; E8 Z-basis coefficients are the ordered (a_i,b_i) pairs"
            .to_owned(),
        golden_conjugation: "(a,b)->(a+b,-b), phi->1-phi".to_owned(),
        turyn_norm: "quaternion norm q*q_bar = a+b*phi in Z[phi]".to_owned(),
        turyn_to_e8_norm: "E8 shell norm is bound by the declared integral coefficient basis and exact inverse; no scalar-domain equality is inferred"
            .to_owned(),
        scale: "H4 coordinates multiplied by two; E8 coordinates are integral coefficients in the declared Z-basis"
            .to_owned(),
        shell_membership: "exact scaled H4 root with Turyn quaternion norm (4,0)"
            .to_owned(),
        orientation: "ordered (1,i,j,k), right-handed".to_owned(),
        coset: "icosian H4-root coefficient coset under the bound root table".to_owned(),
        root_order: "600-cell roots by coefficient tuple then sign, lexicographic revision 1"
            .to_owned(),
        h4_root_table,
        operator_table,
        collision_policy: "exact coefficient identity; no lossy collision".to_owned(),
        forward_map: "[a0..a3,b0..b3] -> q_i=a_i+b_i*phi; companion=phi*conj(q)"
            .to_owned(),
        inverse_map: "q_i -> [a_i,b_i]; conj(phi^-1*companion)=q".to_owned(),
        profile_kappa: String::new(),
    };
    profile.profile_kappa = profile_kappa(&profile)?;
    Ok(profile)
}

fn spin_for_binding(
    unit_id: u32,
    atom: PrimeAtom,
) -> Result<SpinTorsionState, CanonicalLexicalError> {
    let r4 = match unit_id % 4 {
        0 => [1.0, 0.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0, 0.0],
        2 => [0.5, 0.5, 0.5, 0.5],
        _ => [0.5, -0.5, 0.5, -0.5],
    };
    let fiber_raw = i32::try_from(
        i64::from(atom.value())
            .checked_mul(1_000_003)
            .and_then(|value| value.checked_add(i64::from(unit_id) * 17_071))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
    )
    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    let torsion_raw = i32::try_from(
        i64::from(atom.value())
            .checked_mul(-97_409)
            .and_then(|value| value.checked_add(i64::from(unit_id) * 7_919))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
    )
    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    Ok(SpinTorsionState::new(
        UnitS3Q30::from_r4(r4)?,
        PhaseQ29::from_raw(fiber_raw)?,
        PhaseQ29::from_raw(torsion_raw)?,
    )?)
}

fn chart_witness(
    unit_id: u32,
    spin: SpinTorsionState,
    profile: &TypedChartProfile,
) -> Result<ChartTransitionWitness, CanonicalLexicalError> {
    let raw = spin.s3.raw();
    let sin_q30 = raw[0];
    let cos_q30 = raw[1];
    if sin_q30 == 0 && cos_q30 == 0 {
        return Err(CanonicalLexicalError::Invalid(
            "ordinary route state cannot use the typed-null trigonometric sentinel".to_owned(),
        ));
    }
    let activation_q30 = u32::try_from((i64::from(sin_q30) * i64::from(sin_q30)) >> 30)
        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    let chirality = sign_i32(sin_q30);
    let cosine_polarity = sign_i32(cos_q30);
    let pole = cos_q30 == 0;
    let orientation = if pole { chirality } else { 0 };
    let bridge_mode = if unit_id.is_multiple_of(2) {
        "continuous-null"
    } else {
        "discrete-empty-product"
    };
    let phase_shift_q29 = i32::from(orientation)
        .checked_mul(QUARTER_TURN_Q29)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let torsion_shift_q29 = phase_shift_q29;
    let forward_phase = PhaseQ29::from_raw(phase_shift_q29)?;
    let inverse_phase = PhaseQ29::from_raw(
        phase_shift_q29
            .checked_neg()
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
    )?;
    let transported_fiber = spin.fiber.wrapping_add(forward_phase)?;
    let transported_torsion = spin.torsion.wrapping_add(forward_phase)?;
    let inverse_fiber = transported_fiber.wrapping_add(inverse_phase)?;
    let inverse_torsion = transported_torsion.wrapping_add(inverse_phase)?;
    let exact_inverse = inverse_fiber == spin.fiber && inverse_torsion == spin.torsion;
    if !exact_inverse {
        return Err(CanonicalLexicalError::Invalid(
            "typed quarter-turn chart transition did not invert exactly".to_owned(),
        ));
    }
    Ok(ChartTransitionWitness {
        profile_kappa: profile.profile_kappa.clone(),
        bridge_mode: bridge_mode.to_owned(),
        bridge_output: u8::from(bridge_mode == "discrete-empty-product"),
        sin_q30,
        cos_q30,
        activation_q30,
        chirality,
        cosine_polarity,
        source_chart: "tangent".to_owned(),
        active_chart: if pole {
            "cotangent-complement"
        } else {
            "atan2-angle-table"
        }
        .to_owned(),
        tangent_evaluated: false,
        quarter_turn_orientation: orientation,
        phase_shift_q29,
        torsion_shift_q29,
        transported_fiber_q29: transported_fiber.raw(),
        transported_torsion_q29: transported_torsion.raw(),
        inverse_fiber_q29: inverse_fiber.raw(),
        inverse_torsion_q29: inverse_torsion.raw(),
        selected_adapter: "complex-discrete".to_owned(),
        exact_inverse,
    })
}

fn sign_i32(value: i32) -> i8 {
    match value.cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn apply_icosian_operator(
    profile: &IcosianProfile,
    operator: &str,
    value: ZPhi,
) -> Result<ZPhi, CanonicalLexicalError> {
    let row = profile
        .operator_table
        .rows
        .iter()
        .find(|row| row.operator == operator)
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid(format!("icosian operator table lacks {operator}"))
        })?;
    let input = [i128::from(value.a), i128::from(value.b)];
    let mut output = [0i128; 2];
    for (target, coefficients) in output.iter_mut().zip(row.coefficient_matrix) {
        *target = i128::from(coefficients[0])
            .checked_mul(input[0])
            .and_then(|left| {
                i128::from(coefficients[1])
                    .checked_mul(input[1])
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    }
    Ok(ZPhi::new(
        i64::try_from(output[0]).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
        i64::try_from(output[1]).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
    ))
}

fn icosian_coordinate(
    atom: PrimeAtom,
    profile: &IcosianProfile,
) -> Result<IcosianCoordinateWitness, CanonicalLexicalError> {
    let root_count = profile.h4_root_table.roots.len();
    if root_count != 120 {
        return Err(CanonicalLexicalError::Invalid(
            "icosian profile does not bind the 120 H4 roots".to_owned(),
        ));
    }
    let selected_h4_root_index = usize::try_from(atom.value())
        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
        % root_count;
    let root = profile.h4_root_table.roots[selected_h4_root_index];
    let mut e8_basis_coordinates = [0i64; 8];
    let mut h4 = [0i64; 4];
    let mut phi_h4 = [0i64; 4];
    let mut zphi_quaternion = [ZPhiWire { a: 0, b: 0 }; 4];
    let mut galois_companion = [ZPhiWire { a: 0, b: 0 }; 4];
    let mut phi_galois_companion = [ZPhiWire { a: 0, b: 0 }; 4];
    let mut reconstructed = [0i64; 8];
    let mut norm = ZPhi::new(0, 0);
    for index in 0..4 {
        h4[index] = root[index].a;
        phi_h4[index] = root[index].b;
        let coordinate = ZPhi::new(h4[index], phi_h4[index]);
        let conjugate = apply_icosian_operator(profile, "golden-conjugation", coordinate)?;
        let coupled = apply_icosian_operator(profile, "phi-galois-companion", coordinate)?;
        let inverse = apply_icosian_operator(profile, "inverse-phi-galois-companion", coupled)?;
        let phi = apply_icosian_operator(profile, "multiply-phi", coordinate)?;
        let phi_inverse = apply_icosian_operator(profile, "multiply-phi-inverse", phi)?;
        let conjugate_inverse = apply_icosian_operator(profile, "golden-conjugation", conjugate)?;
        if inverse != coordinate
            || phi_inverse != coordinate
            || conjugate_inverse != coordinate
            || conjugate != coordinate.golden_conjugate()?
            || phi != coordinate.times_phi()?
            || coupled != conjugate.times_phi()?
        {
            return Err(CanonicalLexicalError::Invalid(
                "bound icosian operator rows or inverses do not reconstruct".to_owned(),
            ));
        }
        zphi_quaternion[index] = coordinate.into();
        galois_companion[index] = conjugate.into();
        phi_galois_companion[index] = coupled.into();
        e8_basis_coordinates[index] = coordinate.a;
        e8_basis_coordinates[index + 4] = coordinate.b;
        reconstructed[index] = coordinate.a;
        reconstructed[index + 4] = coordinate.b;
        norm = norm.checked_add(coordinate.checked_mul(coordinate)?)?;
    }
    if reconstructed != e8_basis_coordinates || norm != ZPhi::new(4, 0) {
        return Err(CanonicalLexicalError::Invalid(
            "icosian root shell or coefficient inverse does not reproduce".to_owned(),
        ));
    }
    let mut witness = IcosianCoordinateWitness {
        profile_kappa: profile.profile_kappa.clone(),
        selected_h4_root_index: u16::try_from(selected_h4_root_index)
            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
        e8_basis_coordinates,
        h4,
        phi_h4,
        zphi_quaternion,
        galois_companion,
        phi_galois_companion,
        turyn_norm_zphi: norm.into(),
        reconstructed_e8_basis_coordinates: reconstructed,
        coordinate_kappa: String::new(),
        inverse_exact: true,
    };
    witness.coordinate_kappa = icosian_coordinate_kappa(&witness)?;
    Ok(witness)
}

fn icosian_coordinate_kappa(
    coordinate: &IcosianCoordinateWitness,
) -> Result<String, CanonicalLexicalError> {
    let mut seed = coordinate.clone();
    seed.coordinate_kappa.clear();
    canonical_kappa(&canonical_json(&seed)?)
}

fn shared_class_kappa(spin: SpinTorsionState) -> Result<String, CanonicalLexicalError> {
    canonical_kappa(&canonical_json(&SpinClassKeyWire {
        schema: 1,
        domain: "uor-r4.exact-spin-torsion-class/1",
        s3_spin_q30: spin.s3.raw(),
        hopf_observation_q30: spin.hopf.raw(),
        fiber_q29: spin.fiber.raw(),
        torsion_q29: spin.torsion.raw(),
    })?)
}

#[derive(Serialize)]
struct TrajectoryStepWire<'a> {
    schema: u32,
    domain: &'static str,
    previous_trajectory_kappa: Option<&'a str>,
    child_trajectory_kappa: &'a str,
    child_identity_kappa: &'a str,
    observed_children: u32,
}

#[derive(Serialize)]
struct PairedH4StepWire<'a> {
    schema: u32,
    domain: &'static str,
    previous_commitment_kappa: &'a str,
    child_commitment_kappa: &'a str,
    child_identity_kappa: &'a str,
    observed_children: u32,
}

fn route_summary(record: &RouteRecord) -> Result<TrajectorySummary, CanonicalLexicalError> {
    geometric_leaf_summary(
        &record.route_kappa,
        record.body.occurrence,
        record.body.prime,
        record.body.zeta_phase_signature_q29,
        record.body.s3_spin_q30,
        record.body.hopf_observation_q30,
        record.body.fiber_q29,
        record.body.torsion_q29,
        record.body.radial_zphi,
        &record.body.chart,
        &record.body.icosian,
    )
}

#[allow(clippy::too_many_arguments)]
fn geometric_leaf_summary(
    identity_kappa: &str,
    occurrence: u32,
    prime: u32,
    zeta_phase_signature_q29: [i32; 8],
    s3_spin_q30: [i32; 4],
    hopf: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    radial_zphi: ZPhiWire,
    chart: &ChartTransitionWitness,
    icosian: &IcosianCoordinateWitness,
) -> Result<TrajectorySummary, CanonicalLexicalError> {
    let projection_energy_q30 = hopf.iter().try_fold(0u64, |total, value| {
        let square = i128::from(*value)
            .checked_mul(i128::from(*value))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        let scaled =
            u64::try_from(square >> 30).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
        total
            .checked_add(scaled)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)
    })?;
    let mut cosine_resonance_q30 = [0i64; 8];
    for (target, phase) in cosine_resonance_q30
        .iter_mut()
        .zip(zeta_phase_signature_q29)
    {
        let cosine = libm::cos(f64::from(phase) / f64::from(1u32 << 29));
        *target = libm::round(cosine * f64::from(1u32 << 30)) as i64;
    }
    let accumulated_hopf_phase_q29 = wrap_phase_raw(
        i64::from(fiber_q29)
            .checked_add(i64::from(torsion_q29))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
    )?;
    Ok(TrajectorySummary {
        schema: TRAJECTORY_PROFILE_SCHEMA,
        domain: TRAJECTORY_PROFILE_DOMAIN.to_owned(),
        observed_children: 1,
        session_hypersphere_q30: s3_spin_q30.map(i64::from),
        winding_turns: 0,
        window_start: occurrence,
        window_end: occurrence
            .checked_add(1)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
        projection_energy_q30,
        shared_prime_factors: vec![PrimeCount { prime, count: 1 }],
        cosine_resonance_q30,
        accumulated_hopf_phase_q29,
        paired_h4_e8_coordinate_sum: icosian.e8_basis_coordinates,
        paired_h4_e8_coordinate_kappa: icosian.coordinate_kappa.clone(),
        transported_trajectory_kappa: canonical_kappa(identity_kappa.as_bytes())?,
        state: GeometricStateSnapshot {
            prime_factors: vec![PrimeCount { prime, count: 1 }],
            zeta_phase_signature_q29,
            s3_spin_q30,
            s2_hopf_observation_q30: hopf,
            fiber_q29,
            torsion_q29,
            radial_zphi,
            trig_chart: chart.clone(),
            cross_domain_cost_profile_kappa: chart.profile_kappa.clone(),
        },
    })
}

fn attention_level_from_route(
    level: &str,
    record: &RouteRecord,
    previous_identity_kappa: Option<&str>,
) -> Result<AttentionLevelTrace, CanonicalLexicalError> {
    let summary = route_summary(record)?;
    let summary_kappa = canonical_kappa(&canonical_json(&summary)?)?;
    Ok(attention_level_from_summary(
        level,
        "route",
        &record.route_kappa,
        &record.route_kappa,
        &summary_kappa,
        previous_identity_kappa.map(str::to_owned),
        &record.route_kappa,
        1,
        &summary,
        Some(record),
        None,
        None,
        None,
    ))
}

fn attention_level_from_node(level: &str, node: &HierarchyNode) -> AttentionLevelTrace {
    attention_level_from_summary(
        level,
        "hierarchy-node",
        &node.node_kappa,
        &node.body.exact_chain_kappa,
        &node.body.summary_kappa,
        node.body.previous_chain_kappa.clone(),
        &node.body.ordered_child_kappa,
        node.body.child_count,
        &node.body.summary,
        None,
        Some(node.body.boundary_kind.clone()),
        Some(node.body.boundary_identity.clone()),
        Some(node.body.chain_identity.clone()),
    )
}

#[allow(clippy::too_many_arguments)]
fn attention_level_from_summary(
    level: &str,
    identity_kind: &str,
    identity_kappa: &str,
    exact_chain_kappa: &str,
    summary_kappa: &str,
    previous_identity_kappa: Option<String>,
    ordered_child_kappa: &str,
    direct_child_count: u32,
    summary: &TrajectorySummary,
    route: Option<&RouteRecord>,
    boundary_kind: Option<String>,
    boundary_identity: Option<String>,
    chain_identity: Option<String>,
) -> AttentionLevelTrace {
    AttentionLevelTrace {
        level: level.to_owned(),
        ordered_h4: None,
        identity_kind: identity_kind.to_owned(),
        identity_kappa: identity_kappa.to_owned(),
        occurrence: route.map(|record| record.body.occurrence),
        turn: route.map(|record| record.body.turn),
        paragraph: route.map(|record| record.body.paragraph),
        sentence: route.map(|record| record.body.sentence),
        ordinal_in_sentence: route.map(|record| record.body.ordinal_in_sentence),
        lexical_unit_id: route.map(|record| record.body.lexical_unit_id),
        prime: route.map(|record| record.body.prime),
        address_index: route.map(|record| record.body.address_index),
        boundary_kind,
        boundary_identity,
        chain_identity,
        exact_chain_kappa: exact_chain_kappa.to_owned(),
        geometric_summary_kappa: summary_kappa.to_owned(),
        previous_identity_kappa,
        ordered_child_kappa: ordered_child_kappa.to_owned(),
        direct_child_count,
        observed_descendant_routes: summary.observed_children,
        window_start: summary.window_start,
        window_end: summary.window_end,
        session_hypersphere_q30: summary.session_hypersphere_q30,
        winding_turns: summary.winding_turns,
        projection_energy_q30: summary.projection_energy_q30,
        shared_prime_factors: summary
            .shared_prime_factors
            .iter()
            .map(|factor| AttentionPrimeFactorTrace {
                prime: factor.prime,
                count: factor.count,
            })
            .collect(),
        cosine_resonance_q30: summary.cosine_resonance_q30,
        accumulated_hopf_phase_q29: summary.accumulated_hopf_phase_q29,
        zeta_phase_signature_q29: summary.state.zeta_phase_signature_q29,
        s3_spin_q30: summary.state.s3_spin_q30,
        s2_hopf_observation_q30: summary.state.s2_hopf_observation_q30,
        fiber_q29: summary.state.fiber_q29,
        torsion_q29: summary.state.torsion_q29,
        radial_zphi: [summary.state.radial_zphi.a, summary.state.radial_zphi.b],
        bridge_mode: summary.state.trig_chart.bridge_mode.clone(),
        active_chart: summary.state.trig_chart.active_chart.clone(),
        selected_adapter: summary.state.trig_chart.selected_adapter.clone(),
        chart_sin_q30: summary.state.trig_chart.sin_q30,
        chart_cos_q30: summary.state.trig_chart.cos_q30,
        chart_activation_q30: summary.state.trig_chart.activation_q30,
        chart_chirality: summary.state.trig_chart.chirality,
        chart_cosine_polarity: summary.state.trig_chart.cosine_polarity,
        quarter_turn_orientation: summary.state.trig_chart.quarter_turn_orientation,
        phase_shift_q29: summary.state.trig_chart.phase_shift_q29,
        torsion_shift_q29: summary.state.trig_chart.torsion_shift_q29,
        transported_fiber_q29: summary.state.trig_chart.transported_fiber_q29,
        transported_torsion_q29: summary.state.trig_chart.transported_torsion_q29,
        inverse_fiber_q29: summary.state.trig_chart.inverse_fiber_q29,
        inverse_torsion_q29: summary.state.trig_chart.inverse_torsion_q29,
        chart_inverse_exact: summary.state.trig_chart.exact_inverse,
        payload_cid: route.map(|record| record.body.payload_cid.clone()),
        address_kappa: route.map(|record| record.body.address_kappa.clone()),
        shared_class_kappa: route.map(|record| record.body.shared_class_kappa.clone()),
        paired_h4_e8_coordinate_sum: summary.paired_h4_e8_coordinate_sum,
        paired_h4_e8_coordinate_kappa: summary.paired_h4_e8_coordinate_kappa.clone(),
        transported_trajectory_kappa: summary.transported_trajectory_kappa.clone(),
    }
}

fn combine_summary(
    previous: Option<&TrajectorySummary>,
    child: &TrajectorySummary,
    child_identity_kappa: &str,
) -> Result<TrajectorySummary, CanonicalLexicalError> {
    let observed_children = previous
        .map_or(0, |summary| summary.observed_children)
        .checked_add(child.observed_children)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let projection_energy_q30 = previous
        .map_or(0, |summary| summary.projection_energy_q30)
        .checked_add(child.projection_energy_q30)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let mut factors = BTreeMap::<u32, u32>::new();
    for factor in previous
        .into_iter()
        .flat_map(|summary| summary.shared_prime_factors.iter())
        .chain(child.shared_prime_factors.iter())
    {
        let count = factors.entry(factor.prime).or_default();
        *count = count
            .checked_add(factor.count)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    }
    let shared_prime_factors = factors
        .into_iter()
        .map(|(prime, count)| PrimeCount { prime, count })
        .collect::<Vec<_>>();
    let mut cosine_resonance_q30 = child.cosine_resonance_q30;
    if let Some(previous) = previous {
        for (target, prior) in cosine_resonance_q30
            .iter_mut()
            .zip(previous.cosine_resonance_q30)
        {
            *target = target
                .checked_add(prior)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        }
    }
    let prior_phase = previous.map_or(0, |summary| summary.accumulated_hopf_phase_q29);
    let raw_phase = i64::from(prior_phase)
        .checked_add(i64::from(child.accumulated_hopf_phase_q29))
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let (accumulated_hopf_phase_q29, phase_wraps) = wrap_phase_with_turns(raw_phase)?;
    let prior_trajectory = previous.map(|summary| summary.transported_trajectory_kappa.as_str());
    let transported_trajectory_kappa = canonical_kappa(&canonical_json(&TrajectoryStepWire {
        schema: TRAJECTORY_PROFILE_SCHEMA,
        domain: TRAJECTORY_PROFILE_DOMAIN,
        previous_trajectory_kappa: prior_trajectory,
        child_trajectory_kappa: &child.transported_trajectory_kappa,
        child_identity_kappa,
        observed_children,
    })?)?;
    let mut session_hypersphere_q30 = child.session_hypersphere_q30;
    if let Some(previous) = previous {
        for (target, prior) in session_hypersphere_q30
            .iter_mut()
            .zip(previous.session_hypersphere_q30)
        {
            *target = target
                .checked_add(prior)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        }
    }
    let paired_h4_e8_coordinate_kappa = if let Some(previous) = previous {
        canonical_kappa(&canonical_json(&PairedH4StepWire {
            schema: TRAJECTORY_PROFILE_SCHEMA,
            domain: "uor-r4.paired-h4-trajectory-step/1",
            previous_commitment_kappa: &previous.paired_h4_e8_coordinate_kappa,
            child_commitment_kappa: &child.paired_h4_e8_coordinate_kappa,
            child_identity_kappa,
            observed_children,
        })?)?
    } else {
        child.paired_h4_e8_coordinate_kappa.clone()
    };
    let mut paired_h4_e8_coordinate_sum = child.paired_h4_e8_coordinate_sum;
    if let Some(previous) = previous {
        for (target, prior) in paired_h4_e8_coordinate_sum
            .iter_mut()
            .zip(previous.paired_h4_e8_coordinate_sum)
        {
            *target = target
                .checked_add(prior)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        }
    }
    let winding_turns = previous
        .map_or(0, |summary| summary.winding_turns)
        .checked_add(child.winding_turns)
        .and_then(|turns| turns.checked_add(phase_wraps))
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    Ok(TrajectorySummary {
        schema: TRAJECTORY_PROFILE_SCHEMA,
        domain: TRAJECTORY_PROFILE_DOMAIN.to_owned(),
        observed_children,
        session_hypersphere_q30,
        winding_turns,
        window_start: previous.map_or(child.window_start, |summary| summary.window_start),
        window_end: child.window_end,
        projection_energy_q30,
        shared_prime_factors: shared_prime_factors.clone(),
        cosine_resonance_q30,
        accumulated_hopf_phase_q29,
        paired_h4_e8_coordinate_sum,
        paired_h4_e8_coordinate_kappa,
        transported_trajectory_kappa,
        state: GeometricStateSnapshot {
            prime_factors: shared_prime_factors,
            ..child.state.clone()
        },
    })
}

fn wrap_phase_raw(value: i64) -> Result<i32, CanonicalLexicalError> {
    wrap_phase_with_turns(value).map(|(phase, _)| phase)
}

fn wrap_phase_with_turns(value: i64) -> Result<(i32, i64), CanonicalLexicalError> {
    const MODULUS: i64 = 3_373_259_426;
    const HALF: i64 = 1_686_629_713;
    let shifted = value
        .checked_add(HALF)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let turns = shifted.div_euclid(MODULUS);
    let phase = i32::try_from(shifted.rem_euclid(MODULUS) - HALF)
        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
    Ok((phase, turns))
}

#[allow(clippy::too_many_arguments)]
fn append_hierarchy_node(
    nodes: &mut Vec<HierarchyNode>,
    scope: &str,
    identity_scope: &str,
    previous: Option<&HierarchyNode>,
    child_kappa: &str,
    child_kind: &str,
    child_summary: &TrajectorySummary,
    span_start: u32,
    span_end: u32,
    boundary_kind: &str,
    boundary_identity: &str,
    chain_identity: &str,
    provenance_kappa: &str,
) -> Result<HierarchyNode, CanonicalLexicalError> {
    if boundary_kind.is_empty() || boundary_identity.is_empty() || chain_identity.is_empty() {
        return Err(CanonicalLexicalError::Invalid(
            "hierarchy append requires an exact boundary kind and identity".to_owned(),
        ));
    }
    let child_count = previous
        .map_or(0, |node| node.body.child_count)
        .checked_add(1)
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let previous_kappa = previous.map(|node| node.node_kappa.as_str());
    let exact_chain_kappa = canonical_kappa(&canonical_json(&ExactChainWire {
        schema: HIERARCHY_NODE_SCHEMA,
        domain: HIERARCHY_NODE_DOMAIN,
        scope,
        identity_scope,
        previous_chain_kappa: previous_kappa,
        ordered_child_kappa: child_kappa,
        ordered_child_kind: child_kind,
        child_count,
        boundary_kind,
        boundary_identity,
        chain_identity,
    })?)?;
    let summary = combine_summary(
        previous.map(|node| &node.body.summary),
        child_summary,
        child_kappa,
    )?;
    let summary_kappa = canonical_kappa(&canonical_json(&summary)?)?;
    if exact_chain_kappa == summary_kappa {
        return Err(CanonicalLexicalError::Invalid(
            "exact hierarchy identity collided with geometric summary identity".to_owned(),
        ));
    }
    let effective_span_start = previous.map_or(span_start, |node| node.body.span_start);
    let body = HierarchyNodeBody {
        schema: HIERARCHY_NODE_SCHEMA,
        domain: HIERARCHY_NODE_DOMAIN.to_owned(),
        scope: scope.to_owned(),
        identity_scope: identity_scope.to_owned(),
        bridge_mode: summary.state.trig_chart.bridge_mode.clone(),
        previous_chain_kappa: previous.map(|node| node.node_kappa.clone()),
        ordered_child_kappa: child_kappa.to_owned(),
        ordered_child_kind: child_kind.to_owned(),
        child_count,
        span_start: effective_span_start,
        span_end,
        boundary_kind: boundary_kind.to_owned(),
        boundary_identity: boundary_identity.to_owned(),
        chain_identity: chain_identity.to_owned(),
        exact_chain_kappa,
        summary_kappa: summary_kappa.clone(),
        summary,
        payload_or_summary_cid: summary_kappa,
        provenance_kappa: provenance_kappa.to_owned(),
    };
    let node = HierarchyNode {
        node_kappa: canonical_kappa(&canonical_json(&body)?)?,
        body,
    };
    nodes.push(node.clone());
    Ok(node)
}

fn append_local_window_node(
    nodes: &mut Vec<HierarchyNode>,
    previous: Option<&RouteRecord>,
    current: &RouteRecord,
    identity_scope: &str,
    provenance_kappa: &str,
) -> Result<HierarchyNode, CanonicalLexicalError> {
    let previous_node = previous
        .map(|record| {
            append_hierarchy_node(
                nodes,
                "local",
                identity_scope,
                None,
                &record.route_kappa,
                "route",
                &route_summary(record)?,
                record.body.occurrence,
                record
                    .body
                    .occurrence
                    .checked_add(1)
                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
                "rolling-last-two",
                &record.route_kappa,
                identity_scope,
                provenance_kappa,
            )
        })
        .transpose()?;
    append_hierarchy_node(
        nodes,
        "local",
        identity_scope,
        previous_node.as_ref(),
        &current.route_kappa,
        "route",
        &route_summary(current)?,
        current.body.occurrence,
        current
            .body
            .occurrence
            .checked_add(1)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
        "rolling-last-two",
        &current.route_kappa,
        identity_scope,
        provenance_kappa,
    )
}

#[derive(Debug, Clone)]
struct PreparedParagraph {
    turn_index: usize,
    paragraph_index: usize,
    turn_id: String,
    source_bytes: Vec<u8>,
    sentences: Vec<PreparedSentence>,
}

#[derive(Debug, Clone)]
struct PreparedSentence {
    sentence_index: usize,
    source_bytes: Vec<u8>,
    encoded: EncodedParagraph,
}

#[allow(clippy::too_many_arguments)]
fn build_global_snapshot(
    codec: &CanonicalLexicalCodec,
    input: &ConversationInput,
    addresses_by_unit: &BTreeMap<u32, GeometricAddress>,
    registered_addresses: &[GeometricAddress],
    address_kappas: &[String],
    chart_profile: &TypedChartProfile,
    icosian_profile: &IcosianProfile,
) -> Result<GlobalSnapshotBinding, CanonicalLexicalError> {
    let phase_origin = PrimeAtom::new(2)?;
    let mut ordered_units = Vec::with_capacity(input.global_snapshot_units.len());
    let mut accumulated: Option<TrajectorySummary> = None;
    for (ordinal, bytes) in input.global_snapshot_units.iter().enumerate() {
        let encoded = codec.encode(MAX_TURNS, ordinal, bytes)?;
        let unit = encoded.units.first().ok_or_else(|| {
            CanonicalLexicalError::Invalid("global snapshot unit did not encode".to_owned())
        })?;
        let address = addresses_by_unit.get(&unit.unit_id).ok_or_else(|| {
            CanonicalLexicalError::Invalid(
                "global snapshot unit has no exact route address".to_owned(),
            )
        })?;
        let address_index =
            usize::try_from(unit.unit_id).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
        if registered_addresses.get(address_index) != Some(address) {
            return Err(CanonicalLexicalError::Invalid(
                "global snapshot route is absent from the codec address registry".to_owned(),
            ));
        }
        let address_kappa = address_kappas.get(address_index).ok_or_else(|| {
            CanonicalLexicalError::Invalid(
                "global snapshot address kappa index is out of range".to_owned(),
            )
        })?;
        let ordinal_u16 =
            u16::try_from(ordinal).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
        let entry_kappa = canonical_kappa(&canonical_json(&GlobalSnapshotEntryWire {
            schema: 1,
            domain: "uor-r4.bounded-global-route-entry/1",
            snapshot_kappa: &input.global_epoch,
            ordinal: ordinal_u16,
            lexical_unit_id: unit.unit_id,
            address_kappa,
        })?)?;
        let mut zeta = [0i32; 8];
        for (target, channel) in zeta.iter_mut().zip(ZETA_SUMMARY_CHANNELS) {
            *target = zeta_phase_delta(channel, phase_origin, address.atom)?.raw();
        }
        let chart = chart_witness(unit.unit_id, address.spin, chart_profile)?;
        let icosian = icosian_coordinate(address.atom, icosian_profile)?;
        let leaf = geometric_leaf_summary(
            &entry_kappa,
            u32::try_from(ordinal).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            address.atom.value(),
            zeta,
            address.spin.s3.raw(),
            address.spin.hopf.raw(),
            address.spin.fiber.raw(),
            address.spin.torsion.raw(),
            address.radial.into(),
            &chart,
            &icosian,
        )?;
        accumulated = Some(combine_summary(accumulated.as_ref(), &leaf, &entry_kappa)?);
        ordered_units.push(GlobalSnapshotUnitBinding {
            ordinal: ordinal_u16,
            lexical_unit_id: unit.unit_id,
            address_index: u16::try_from(address_index)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            address_kappa: address_kappa.clone(),
            entry_kappa,
        });
    }
    let summary = accumulated.ok_or_else(|| {
        CanonicalLexicalError::Invalid("global snapshot produced no summary".to_owned())
    })?;
    let summary_kappa = canonical_kappa(&canonical_json(&summary)?)?;
    Ok(GlobalSnapshotBinding {
        schema: 1,
        domain: "uor-r4.bounded-global-route-snapshot/1".to_owned(),
        snapshot_kappa: input.global_epoch.clone(),
        ordered_units,
        summary,
        summary_kappa,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderedH4FoldAggregate {
    state: OrderedH4FoldState,
    observed_routes: u32,
}

impl OrderedH4FoldAggregate {
    fn identity(table: &H4BinaryIcosahedralClosure) -> Result<Self, CanonicalLexicalError> {
        Ok(Self {
            state: OrderedH4FoldState::identity(table)?,
            observed_routes: 0,
        })
    }

    fn compose(
        self,
        right: Self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<Self, CanonicalLexicalError> {
        Ok(Self {
            state: self.state.compose(right.state, table)?,
            observed_routes: self
                .observed_routes
                .checked_add(right.observed_routes)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
        })
    }
}

fn ordered_h4_route_aggregate(
    record: &RouteRecord,
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldAggregate, CanonicalLexicalError> {
    let state = h4_leaf_state_for_prime(record.body.prime, table)?;
    if state.table_index().table_offset() != record.body.icosian.selected_h4_root_index {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 leaf does not reproduce the S0 icosian root assignment".to_owned(),
        ));
    }
    Ok(OrderedH4FoldAggregate {
        state,
        observed_routes: 1,
    })
}

fn ordered_h4_global_aggregate(
    global: &GlobalSnapshotBinding,
    lexical_addresses: &[LexicalRouteAddressBinding],
    table: &H4BinaryIcosahedralClosure,
) -> Result<OrderedH4FoldAggregate, CanonicalLexicalError> {
    let mut aggregate = OrderedH4FoldAggregate::identity(table)?;
    for unit in &global.ordered_units {
        let address = lexical_addresses
            .get(usize::from(unit.address_index))
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "ordered H4 global unit address index is out of range".to_owned(),
                )
            })?;
        if address.lexical_unit_id != unit.lexical_unit_id
            || address.address_kappa != unit.address_kappa
        {
            return Err(CanonicalLexicalError::Invalid(
                "ordered H4 global unit does not reproduce its lexical address".to_owned(),
            ));
        }
        aggregate = aggregate.compose(
            OrderedH4FoldAggregate {
                state: h4_leaf_state_for_prime(address.prime, table)?,
                observed_routes: 1,
            },
            table,
        )?;
    }
    if aggregate.observed_routes != global.summary.observed_children {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 global route count disagrees with the S0 summary".to_owned(),
        ));
    }
    Ok(aggregate)
}

#[allow(clippy::too_many_arguments)]
fn ordered_h4_node_aggregate(
    node_kappa: &str,
    table: &H4BinaryIcosahedralClosure,
    node_by_kappa: &BTreeMap<&str, &HierarchyNode>,
    route_by_kappa: &BTreeMap<&str, &RouteRecord>,
    global: &GlobalSnapshotBinding,
    lexical_addresses: &[LexicalRouteAddressBinding],
    memo: &mut BTreeMap<String, OrderedH4FoldAggregate>,
    visiting: &mut BTreeSet<String>,
) -> Result<OrderedH4FoldAggregate, CanonicalLexicalError> {
    if let Some(aggregate) = memo.get(node_kappa) {
        return Ok(*aggregate);
    }
    if !visiting.insert(node_kappa.to_owned()) {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 hierarchy links contain a cycle".to_owned(),
        ));
    }
    let node = node_by_kappa.get(node_kappa).copied().ok_or_else(|| {
        CanonicalLexicalError::Invalid(
            "ordered H4 hierarchy link references an absent node".to_owned(),
        )
    })?;
    let previous = node
        .body
        .previous_chain_kappa
        .as_deref()
        .map(|previous| {
            ordered_h4_node_aggregate(
                previous,
                table,
                node_by_kappa,
                route_by_kappa,
                global,
                lexical_addresses,
                memo,
                visiting,
            )
        })
        .transpose()?
        .unwrap_or(OrderedH4FoldAggregate::identity(table)?);
    let child = match node.body.ordered_child_kind.as_str() {
        "route" => {
            let route = route_by_kappa
                .get(node.body.ordered_child_kappa.as_str())
                .copied()
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "ordered H4 hierarchy route child is absent".to_owned(),
                    )
                })?;
            ordered_h4_route_aggregate(route, table)?
        }
        "sentence-node" | "paragraph-node" => ordered_h4_node_aggregate(
            &node.body.ordered_child_kappa,
            table,
            node_by_kappa,
            route_by_kappa,
            global,
            lexical_addresses,
            memo,
            visiting,
        )?,
        "global-snapshot" => {
            if node.body.ordered_child_kappa != global.snapshot_kappa {
                return Err(CanonicalLexicalError::Invalid(
                    "ordered H4 global hierarchy link names a different snapshot".to_owned(),
                ));
            }
            ordered_h4_global_aggregate(global, lexical_addresses, table)?
        }
        other => {
            return Err(CanonicalLexicalError::Invalid(format!(
                "ordered H4 hierarchy has unsupported child kind {other}"
            )));
        }
    };
    let aggregate = previous.compose(child, table)?;
    if aggregate.observed_routes != node.body.summary.observed_children {
        return Err(CanonicalLexicalError::Invalid(
            "ordered H4 hierarchy route count disagrees with the S0 summary".to_owned(),
        ));
    }
    visiting.remove(node_kappa);
    memo.insert(node_kappa.to_owned(), aggregate);
    Ok(aggregate)
}

fn ordered_h4_trace_level(
    level: &str,
    aggregate: OrderedH4FoldAggregate,
    mut consumer_level: AttentionLevelTrace,
    table: &H4BinaryIcosahedralClosure,
) -> Result<AttentionOrderedFoldLevel, CanonicalLexicalError> {
    if consumer_level.level != level
        || consumer_level.observed_descendant_routes != aggregate.observed_routes
        || consumer_level.ordered_h4.is_some()
    {
        return Err(CanonicalLexicalError::Invalid(format!(
            "ordered H4 {level} state does not align with its attention consumer level"
        )));
    }
    let root_coordinate = aggregate.state.root_coordinate(table)?;
    consumer_level.ordered_h4 = Some(AttentionOrderedH4StateTrace {
        schema: ORDERED_H4_FOLD_SCHEMA,
        domain: ORDERED_H4_FOLD_DOMAIN.to_owned(),
        observed_routes: aggregate.observed_routes,
        state: aggregate.state,
        root_coordinate,
    });
    Ok(AttentionOrderedFoldLevel {
        level: level.to_owned(),
        observed_routes: aggregate.observed_routes,
        state: aggregate.state,
        root_coordinate,
        consumer_level,
    })
}

impl CanonicalRouteArtifact {
    pub fn ingest(
        codec: &CanonicalLexicalCodec,
        input: &ConversationInput,
    ) -> Result<Self, CanonicalLexicalError> {
        validate_input_shape(input)?;
        codec.validate()?;

        // The entire lexical input is encoded before any hierarchy object is
        // created. An unknown unit therefore refuses the transaction without a
        // partial route or parent state.
        let mut prepared = Vec::new();
        for (turn_index, turn) in input.turns.iter().enumerate() {
            for (paragraph_index, paragraph) in turn.paragraphs.iter().enumerate() {
                let sentences = paragraph
                    .sentences
                    .iter()
                    .enumerate()
                    .map(|(sentence_index, sentence)| {
                        Ok(PreparedSentence {
                            sentence_index,
                            source_bytes: sentence.clone(),
                            encoded: codec.encode(turn_index, paragraph_index, sentence)?,
                        })
                    })
                    .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
                prepared.push(PreparedParagraph {
                    turn_index,
                    paragraph_index,
                    turn_id: turn.turn_id.clone(),
                    source_bytes: paragraph.sentences.concat(),
                    sentences,
                });
            }
        }
        let lexical_units = prepared.iter().try_fold(0usize, |total, paragraph| {
            paragraph
                .sentences
                .iter()
                .try_fold(total, |subtotal, sentence| {
                    subtotal
                        .checked_add(sentence.encoded.units.len())
                        .ok_or(CanonicalLexicalError::ArithmeticOverflow)
                })
        })?;
        if lexical_units == 0 || lexical_units > MAX_LEXICAL_UNITS {
            return Err(CanonicalLexicalError::Invalid(format!(
                "S0 input must contain 1..={MAX_LEXICAL_UNITS} lexical units"
            )));
        }

        let source_cid = source_cid(input)?;
        let chart_profile = fixed_chart_profile()?;
        let icosian_profile = fixed_icosian_profile()?;
        let compiler_cid = canonical_kappa(COMPILER_IDENTITY_BYTES)?;
        let cost_profile_cid = chart_profile.profile_kappa.clone();
        let hierarchy_provenance = HierarchyProvenance {
            schema: 1,
            domain: "uor-r4.incremental-hierarchy-provenance/1".to_owned(),
            compiler_cid: compiler_cid.clone(),
            codec_kappa: codec.profile.codec_kappa.clone(),
            chart_profile_kappa: chart_profile.profile_kappa.clone(),
            icosian_profile_kappa: icosian_profile.profile_kappa.clone(),
            identity_scope: input.identity_scope.clone(),
            global_epoch: input.global_epoch.clone(),
        };
        let provenance_kappa = canonical_kappa(&canonical_json(&hierarchy_provenance)?)?;
        let provenance = ArtifactProvenance {
            compiler: COMPILER_NAME.to_owned(),
            compiler_cid: compiler_cid.clone(),
            source_cid: source_cid.clone(),
            codec_kappa: codec.profile.codec_kappa.clone(),
            cost_profile_cid: cost_profile_cid.clone(),
            identity_scope: input.identity_scope.clone(),
            global_epoch: input.global_epoch.clone(),
            source_weights_opened: false,
            teacher_forwards: 0,
        };

        let semantic_atoms = codec
            .vocabulary
            .iter()
            .map(|binding| SemanticAtom {
                semantic_atom_id: format!("lexical-unit-{:08}", binding.unit_id),
                payload_cid: binding.payload_cid.clone(),
            })
            .collect::<Vec<_>>();
        let prime_registry = PrimeRegistry::compile(&semantic_atoms)?;
        let mut addresses_by_unit = BTreeMap::<u32, GeometricAddress>::new();
        for vocabulary in &codec.vocabulary {
            let semantic_id = format!("lexical-unit-{:08}", vocabulary.unit_id);
            let binding = prime_registry.binding_for_id(&semantic_id).ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "compiled prime registry omitted a lexical vocabulary entry".to_owned(),
                )
            })?;
            let spin = spin_for_binding(vocabulary.unit_id, binding.atom)?;
            addresses_by_unit.insert(
                vocabulary.unit_id,
                GeometricAddress {
                    atom: binding.atom,
                    spin,
                    radial: ZPhi::new(
                        i64::from(vocabulary.unit_id)
                            .checked_add(1)
                            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
                        i64::from(binding.atom.value().rem_euclid(5)),
                    ),
                    payload_cid: binding.payload_cid.clone(),
                },
            );
        }
        let lexical_route_addresses = addresses_by_unit
            .iter()
            .map(|(unit_id, address)| lexical_route_address_binding(*unit_id, address))
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        let registered_addresses = addresses_by_unit.values().cloned().collect::<Vec<_>>();
        let registered_address_kappas = lexical_route_addresses
            .iter()
            .map(|binding| binding.address_kappa.clone())
            .collect::<Vec<_>>();

        let mut spin_sentences = Vec::new();
        for paragraph in &prepared {
            for sentence in &paragraph.sentences {
                let routes = sentence
                    .encoded
                    .units
                    .iter()
                    .map(|unit| {
                        addresses_by_unit
                            .get(&unit.unit_id)
                            .cloned()
                            .ok_or_else(|| {
                                CanonicalLexicalError::Invalid(
                                    "encoded lexical unit has no registered route address"
                                        .to_owned(),
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
                spin_sentences.push(RouteSentence {
                    sentence_id: format!(
                        "turn-{:04}/paragraph-{:04}/sentence-{:04}",
                        paragraph.turn_index, paragraph.paragraph_index, sentence.sentence_index
                    ),
                    routes,
                });
            }
        }
        let global_routes = input
            .global_snapshot_units
            .iter()
            .enumerate()
            .map(|(ordinal, bytes)| {
                let encoded = codec.encode(MAX_TURNS, ordinal, bytes)?;
                let unit = encoded.units.first().ok_or_else(|| {
                    CanonicalLexicalError::Invalid("global snapshot unit did not encode".to_owned())
                })?;
                addresses_by_unit
                    .get(&unit.unit_id)
                    .cloned()
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "global snapshot unit has no registered address".to_owned(),
                        )
                    })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        spin_sentences.push(RouteSentence {
            sentence_id: format!("global-snapshot/{}", input.global_epoch),
            routes: global_routes,
        });
        let compiled = compile_spin_manifest(
            &spin_sentences,
            prime_registry,
            ZeroPowerBridge::ContinuousNull,
            ManifestProvenance {
                tokenizer_cid: codec.profile.codec_kappa.clone(),
                corpus_cid: source_cid.clone(),
                compiler_cid,
                cost_profile_cid,
            },
            NonZeroU16::new(CHILD_MAX_CANDIDATES)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
            NonZeroUsize::new(1).ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
        )?;
        let spin_manifest_bytes = compiled.manifest.canonical_bytes()?;
        let spin_manifest_blob = content_blob("prime-route-spin-manifest", &spin_manifest_bytes)?;
        let spin_manifest = SpinManifestBinding {
            schema: compiled.manifest.schema,
            domain: "uor-r4.prime-route-spin-manifest/2".to_owned(),
            blob_cid: spin_manifest_blob.cid.clone(),
            manifest_kappa: compiled.manifest.manifest_kappa.clone(),
            base_bridge_tag: compiled.manifest.bridge.value(),
        };

        let mut content_blobs = codec
            .vocabulary
            .iter()
            .map(|binding| {
                let bytes = decode_hex(&binding.surface_hex, "vocabulary surface")?;
                content_blob("lexical-payload", &bytes)
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        content_blobs.push(spin_manifest_blob);
        content_blobs.sort();
        if content_blobs
            .windows(2)
            .any(|pair| pair[0].cid == pair[1].cid)
        {
            return Err(CanonicalLexicalError::Invalid(
                "content blob CIDs are not unique".to_owned(),
            ));
        }

        let global_snapshot = build_global_snapshot(
            codec,
            input,
            &addresses_by_unit,
            &registered_addresses,
            &registered_address_kappas,
            &chart_profile,
            &icosian_profile,
        )?;
        let phase_origin = PrimeAtom::new(2)?;
        let mut route_records = Vec::with_capacity(lexical_units);
        let mut paragraph_witnesses = Vec::with_capacity(prepared.len());
        let mut class_rows = BTreeMap::<String, SharedSpinTorsionClass>::new();
        let mut occurrence = 0u32;
        for paragraph in &prepared {
            let mut sentence_witnesses = Vec::with_capacity(paragraph.sentences.len());
            for sentence in &paragraph.sentences {
                let mut route_kappas = Vec::with_capacity(sentence.encoded.units.len());
                for (ordinal, unit) in sentence.encoded.units.iter().enumerate() {
                    let vocabulary = codec.binding(unit.unit_id).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "encoded lexical unit is absent from codec vocabulary".to_owned(),
                        )
                    })?;
                    let address = addresses_by_unit.get(&unit.unit_id).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "encoded lexical unit is absent from route registry".to_owned(),
                        )
                    })?;
                    if compiled.manifest.addresses.binary_search(address).is_err() {
                        return Err(CanonicalLexicalError::Invalid(
                            "observed lexical route is absent from the child manifest".to_owned(),
                        ));
                    }
                    let address_index = usize::try_from(unit.unit_id)
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
                    let address_kappa = registered_address_kappas
                        .get(address_index)
                        .cloned()
                        .ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "registered lexical address index is out of range".to_owned(),
                            )
                        })?;
                    let mut zeta_phase_signature_q29 = [0i32; 8];
                    for (target, channel) in zeta_phase_signature_q29
                        .iter_mut()
                        .zip(ZETA_SUMMARY_CHANNELS)
                    {
                        *target = zeta_phase_delta(channel, phase_origin, address.atom)?.raw();
                    }
                    let chart = chart_witness(unit.unit_id, address.spin, &chart_profile)?;
                    let icosian = icosian_coordinate(address.atom, &icosian_profile)?;
                    let shared_class_kappa = shared_class_kappa(address.spin)?;
                    let body = RouteRecordBody {
                        schema: ROUTE_RECORD_SCHEMA,
                        domain: ROUTE_RECORD_DOMAIN.to_owned(),
                        occurrence,
                        turn: u16::try_from(paragraph.turn_index)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        paragraph: u16::try_from(paragraph.paragraph_index)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        sentence: u16::try_from(sentence.sentence_index)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        ordinal_in_sentence: u16::try_from(ordinal)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        span_start: unit.span_start,
                        span_end: unit.span_end,
                        leading_bytes_hex: hex::encode(&unit.leading_bytes),
                        lexical_unit_id: unit.unit_id,
                        payload_cid: vocabulary.payload_cid.clone(),
                        prime: address.atom.value(),
                        address_index: u16::try_from(address_index)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        address_kappa,
                        zeta_phase_signature_q29,
                        s3_spin_q30: address.spin.s3.raw(),
                        hopf_observation_q30: address.spin.hopf.raw(),
                        fiber_q29: address.spin.fiber.raw(),
                        torsion_q29: address.spin.torsion.raw(),
                        radial_zphi: address.radial.into(),
                        chart,
                        icosian,
                        shared_class_kappa: shared_class_kappa.clone(),
                    };
                    let route_kappa = canonical_kappa(&canonical_json(&body)?)?;
                    let record = RouteRecord {
                        route_kappa: route_kappa.clone(),
                        body,
                    };
                    let class = class_rows
                        .entry(shared_class_kappa.clone())
                        .or_insert_with(|| SharedSpinTorsionClass {
                            class_kappa: shared_class_kappa,
                            s3_spin_q30: address.spin.s3.raw(),
                            hopf_observation_q30: address.spin.hopf.raw(),
                            fiber_q29: address.spin.fiber.raw(),
                            torsion_q29: address.spin.torsion.raw(),
                            ordered_route_members: Vec::new(),
                        });
                    class.ordered_route_members.push(route_kappa.clone());
                    route_kappas.push(route_kappa);
                    route_records.push(record);
                    occurrence = occurrence
                        .checked_add(1)
                        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
                }
                sentence_witnesses.push(SentenceRebuildWitness {
                    sentence_index: u16::try_from(sentence.sentence_index)
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                    route_kappas,
                    trailing_bytes_hex: hex::encode(&sentence.encoded.trailing_bytes),
                    source_cid: canonical_kappa(&sentence.source_bytes)?,
                });
            }
            paragraph_witnesses.push(ParagraphRebuildWitness {
                turn_id: paragraph.turn_id.clone(),
                paragraph_index: u16::try_from(paragraph.paragraph_index)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                sentences: sentence_witnesses,
                source_cid: canonical_kappa(&paragraph.source_bytes)?,
            });
        }
        let shared_classes = class_rows.into_values().collect::<Vec<_>>();

        let record_by_kappa = route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let mut hierarchy_nodes = Vec::new();
        let mut conversation_previous: Option<HierarchyNode> = None;
        let mut final_sentence: Option<HierarchyNode> = None;
        let mut final_paragraph: Option<HierarchyNode> = None;
        for witness in &paragraph_witnesses {
            let mut paragraph_previous: Option<HierarchyNode> = None;
            for sentence_witness in &witness.sentences {
                let mut sentence_previous: Option<HierarchyNode> = None;
                for route_kappa in &sentence_witness.route_kappas {
                    let record = record_by_kappa.get(route_kappa.as_str()).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "sentence witness references an absent route record".to_owned(),
                        )
                    })?;
                    let summary = route_summary(record)?;
                    sentence_previous = Some(append_hierarchy_node(
                        &mut hierarchy_nodes,
                        "sentence",
                        &input.identity_scope,
                        sentence_previous.as_ref(),
                        route_kappa,
                        "route",
                        &summary,
                        record.body.occurrence,
                        record
                            .body
                            .occurrence
                            .checked_add(1)
                            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
                        "codec-declared-sentence",
                        &format!(
                            "turn={};paragraph={};sentence={}",
                            witness.turn_id,
                            witness.paragraph_index,
                            sentence_witness.sentence_index
                        ),
                        &format!(
                            "turn={};paragraph={};sentence={}",
                            witness.turn_id,
                            witness.paragraph_index,
                            sentence_witness.sentence_index
                        ),
                        &provenance_kappa,
                    )?);
                }
                let sentence = sentence_previous.ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "sentence witness produced no sentence root".to_owned(),
                    )
                })?;
                paragraph_previous = Some(append_hierarchy_node(
                    &mut hierarchy_nodes,
                    "paragraph",
                    &input.identity_scope,
                    paragraph_previous.as_ref(),
                    &sentence.node_kappa,
                    "sentence-node",
                    &sentence.body.summary,
                    sentence.body.span_start,
                    sentence.body.span_end,
                    "declared-paragraph",
                    &format!(
                        "turn={};paragraph={};sentence={}",
                        witness.turn_id, witness.paragraph_index, sentence_witness.sentence_index
                    ),
                    &format!(
                        "turn={};paragraph={}",
                        witness.turn_id, witness.paragraph_index
                    ),
                    &provenance_kappa,
                )?);
                final_sentence = Some(sentence);
            }
            let paragraph = paragraph_previous.ok_or_else(|| {
                CanonicalLexicalError::Invalid("paragraph produced no paragraph root".to_owned())
            })?;
            conversation_previous = Some(append_hierarchy_node(
                &mut hierarchy_nodes,
                "conversation",
                &input.identity_scope,
                conversation_previous.as_ref(),
                &paragraph.node_kappa,
                "paragraph-node",
                &paragraph.body.summary,
                paragraph.body.span_start,
                paragraph.body.span_end,
                "observed-turn-paragraph",
                &format!(
                    "turn={};paragraph={}",
                    witness.turn_id, witness.paragraph_index
                ),
                &input.identity_scope,
                &provenance_kappa,
            )?);
            final_paragraph = Some(paragraph);
        }
        let conversation = conversation_previous.ok_or_else(|| {
            CanonicalLexicalError::Invalid("conversation root was not produced".to_owned())
        })?;
        let global = append_hierarchy_node(
            &mut hierarchy_nodes,
            "global",
            &input.identity_scope,
            None,
            &global_snapshot.snapshot_kappa,
            "global-snapshot",
            &global_snapshot.summary,
            0,
            u32::try_from(global_snapshot.ordered_units.len())
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            "immutable-bounded-global-epoch",
            &global_snapshot.snapshot_kappa,
            &global_snapshot.snapshot_kappa,
            &provenance_kappa,
        )?;
        let current = route_records.last().ok_or_else(|| {
            CanonicalLexicalError::Invalid("route record population is empty".to_owned())
        })?;
        let previous = route_records
            .get(route_records.len().checked_sub(2).ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "local previous/last-two state requires at least two routes".to_owned(),
                )
            })?)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid("local previous route is absent".to_owned())
            })?;
        let last_two = append_local_window_node(
            &mut hierarchy_nodes,
            Some(previous),
            current,
            &input.identity_scope,
            &provenance_kappa,
        )?;
        let hierarchy_roots = HierarchyRoots {
            local: LocalRoots {
                current: current.route_kappa.clone(),
                previous: previous.route_kappa.clone(),
                last_two: last_two.node_kappa,
            },
            sentence: final_sentence
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid("sentence hierarchy root is absent".to_owned())
                })?
                .node_kappa,
            paragraph: final_paragraph
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid("paragraph hierarchy root is absent".to_owned())
                })?
                .node_kappa,
            conversation: conversation.node_kappa,
            global: global.node_kappa,
        };
        hierarchy_nodes.sort_by(|left, right| left.node_kappa.cmp(&right.node_kappa));

        let body = ArtifactBody {
            schema: CANONICAL_ROUTE_ARTIFACT_SCHEMA,
            domain: CANONICAL_ROUTE_ARTIFACT_DOMAIN.to_owned(),
            codec: codec.profile.clone(),
            vocabulary: codec.vocabulary.clone(),
            lexical_route_addresses,
            content_blobs,
            spin_manifest,
            chart_profile,
            icosian_profile,
            route_records,
            paragraph_witnesses,
            global_snapshot,
            hierarchy_nodes,
            hierarchy_roots,
            shared_classes,
            bounds: ArtifactBounds {
                maximum_turns: u16::try_from(MAX_TURNS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_paragraphs: u16::try_from(MAX_PARAGRAPHS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_sentences: u16::try_from(MAX_SENTENCES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_lexical_units_per_sentence: u16::try_from(MAX_LEXICAL_UNITS_PER_SENTENCE)
                    .map_err(|_| {
                    CanonicalLexicalError::ArithmeticOverflow
                })?,
                maximum_lexical_units: u16::try_from(MAX_LEXICAL_UNITS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_vocabulary: u16::try_from(MAX_VOCABULARY)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_source_bytes: u32::try_from(MAX_SOURCE_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_artifact_bytes: u32::try_from(CANONICAL_ROUTE_ARTIFACT_MAX_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_identity_scope_bytes: u16::try_from(MAX_IDENTITY_SCOPE_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_global_epoch_bytes: u16::try_from(MAX_GLOBAL_EPOCH_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_turn_id_bytes: u16::try_from(MAX_TURN_ID_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_global_snapshot_units: u16::try_from(MAX_GLOBAL_SNAPSHOT_UNITS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                global_epoch_policy:
                    "immutable during a session; replace only by new content-addressed epoch"
                        .to_owned(),
            },
            scope_ceilings: fixed_scope_ceilings()?,
            hierarchy_provenance,
            provenance,
        };
        let manifest_kappa = canonical_kappa(&canonical_json(&body)?)?;
        let artifact = Self {
            manifest_kappa,
            body,
        };
        artifact.validate_transitive()?;
        Ok(artifact)
    }

    pub fn manifest_kappa(&self) -> &str {
        &self.manifest_kappa
    }

    pub fn embedded_spin_manifest_kappa(&self) -> &str {
        &self.body.spin_manifest.manifest_kappa
    }

    pub fn codec_kappa(&self) -> &str {
        &self.body.codec.codec_kappa
    }

    pub fn vocabulary_kappa(&self) -> &str {
        &self.body.codec.vocabulary_kappa
    }

    pub fn scope_ceilings(&self) -> &[ScopeCeiling] {
        &self.body.scope_ceilings
    }

    fn lexical_route_address_unvalidated(
        &self,
        lexical_unit_id: u32,
    ) -> Result<Option<GeometricAddress>, CanonicalLexicalError> {
        let index = usize::try_from(lexical_unit_id)
            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
        let Some(stored) = self.body.lexical_route_addresses.get(index) else {
            return Ok(None);
        };
        let vocabulary = self.body.vocabulary.get(index).ok_or_else(|| {
            CanonicalLexicalError::Invalid(
                "lexical address registry has no matching vocabulary row".to_owned(),
            )
        })?;
        if stored.lexical_unit_id != lexical_unit_id
            || vocabulary.unit_id != lexical_unit_id
            || stored.payload_cid != vocabulary.payload_cid
        {
            return Err(CanonicalLexicalError::Invalid(
                "lexical address registry is not aligned with vocabulary order".to_owned(),
            ));
        }
        let atom = PrimeAtom::new(stored.prime)?;
        let address = GeometricAddress {
            atom,
            spin: spin_for_binding(lexical_unit_id, atom)?,
            radial: ZPhi::from(stored.radial_zphi),
            payload_cid: stored.payload_cid.clone(),
        };
        if lexical_route_address_binding(lexical_unit_id, &address)? != *stored {
            return Err(CanonicalLexicalError::Invalid(
                "lexical address does not reproduce from its exact registry row".to_owned(),
            ));
        }
        Ok(Some(address))
    }

    /// Resolve one stable codec unit to its exact registered geometric
    /// address. The parent registry is complete even when the frozen S0 child
    /// observed only a subset of vocabulary addresses.
    pub fn lexical_route_address(
        &self,
        lexical_unit_id: u32,
    ) -> Result<Option<GeometricAddress>, CanonicalLexicalError> {
        self.validate_transitive()?;
        self.lexical_route_address_unvalidated(lexical_unit_id)
    }

    /// Resolve a bounded set of stable codec units after validating the
    /// immutable parent registry once. This is the safe public batch form for
    /// callers that need several exact addresses from the same artifact.
    pub fn lexical_route_addresses(
        &self,
        lexical_unit_ids: &[u32],
    ) -> Result<Vec<Option<GeometricAddress>>, CanonicalLexicalError> {
        self.validate_transitive()?;
        lexical_unit_ids
            .iter()
            .map(|unit_id| self.lexical_route_address_unvalidated(*unit_id))
            .collect()
    }

    /// Internal fixed-probe lookup after the immutable artifact has already
    /// crossed `validate_transitive`. This prevents one bounded candidate
    /// query from revalidating the complete artifact for every registry row.
    pub(crate) fn lexical_route_address_from_validated_artifact(
        &self,
        lexical_unit_id: u32,
    ) -> Result<Option<GeometricAddress>, CanonicalLexicalError> {
        self.lexical_route_address_unvalidated(lexical_unit_id)
    }

    /// Decode the exact schema-2 child used by the real bounded I1/I2/IS,
    /// divisor, and adjacent-spin candidate path.
    pub fn embedded_spin_manifest(&self) -> Result<CompiledSpinManifest, CanonicalLexicalError> {
        self.validate_transitive()?;
        let blob = self
            .body
            .content_blobs
            .iter()
            .find(|blob| blob.cid == self.body.spin_manifest.blob_cid)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "embedded spin-manifest blob reference is absent".to_owned(),
                )
            })?;
        if blob.kind != "prime-route-spin-manifest" {
            return Err(CanonicalLexicalError::Invalid(
                "embedded spin-manifest blob has the wrong kind".to_owned(),
            ));
        }
        let manifest = CompiledSpinManifest::decode_canonical(&decode_hex(
            &blob.bytes_hex,
            "spin manifest blob",
        )?)?;
        if manifest.manifest_kappa != self.body.spin_manifest.manifest_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "embedded spin-manifest kappa does not match its parent binding".to_owned(),
            ));
        }
        Ok(manifest)
    }

    /// Invert one exact selected address to its codec payload bytes without a
    /// corpus-text lookup or a source-model dependency.
    pub fn lexical_route_value_for_address(
        &self,
        address: &GeometricAddress,
    ) -> Result<Option<LexicalRouteValueView>, CanonicalLexicalError> {
        self.validate_transitive()?;
        self.lexical_route_value_for_address_from_validated_artifact(address)
    }

    /// Internal fixed-probe inverse after the immutable artifact has already
    /// crossed `validate_transitive`. Public callers retain the fail-closed
    /// validating entrypoint above.
    pub(crate) fn lexical_route_value_for_address_from_validated_artifact(
        &self,
        address: &GeometricAddress,
    ) -> Result<Option<LexicalRouteValueView>, CanonicalLexicalError> {
        let address_kappa = address.canonical_kappa()?;
        let Some(stored) = self
            .body
            .lexical_route_addresses
            .iter()
            .find(|stored| stored.address_kappa == address_kappa)
        else {
            return Ok(None);
        };
        let expected = self
            .lexical_route_address_unvalidated(stored.lexical_unit_id)?
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "selected lexical address is absent from the complete registry".to_owned(),
                )
            })?;
        if &expected != address {
            return Err(CanonicalLexicalError::Invalid(
                "selected lexical address kappa resolves to different exact fields".to_owned(),
            ));
        }
        let vocabulary = self
            .body
            .vocabulary
            .get(
                usize::try_from(stored.lexical_unit_id)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            )
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "selected lexical value has no vocabulary binding".to_owned(),
                )
            })?;
        let blob = self
            .body
            .content_blobs
            .iter()
            .find(|blob| blob.cid == vocabulary.payload_cid)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "selected lexical value has no payload blob".to_owned(),
                )
            })?;
        if blob.kind != "lexical-payload" || blob.bytes_hex != vocabulary.surface_hex {
            return Err(CanonicalLexicalError::Invalid(
                "selected lexical payload blob does not match its vocabulary binding".to_owned(),
            ));
        }
        Ok(Some(LexicalRouteValueView {
            lexical_unit_id: stored.lexical_unit_id,
            registry_address_index: u16::try_from(stored.lexical_unit_id)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
            prime: stored.prime,
            address_kappa,
            payload_cid: stored.payload_cid.clone(),
            payload_bytes: decode_hex(&blob.bytes_hex, "selected lexical payload")?,
        }))
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CanonicalLexicalError> {
        self.validate_transitive()?;
        let expected = canonical_kappa(&canonical_json(&self.body)?)?;
        if expected != self.manifest_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "canonical lexical-route manifest kappa does not reproduce".to_owned(),
            ));
        }
        let bytes = canonical_json(&ArtifactEnvelope {
            schema: CANONICAL_ROUTE_ARTIFACT_SCHEMA,
            domain: CANONICAL_ROUTE_ARTIFACT_DOMAIN.to_owned(),
            manifest_kappa: self.manifest_kappa.clone(),
            body: self.body.clone(),
        })?;
        if bytes.len() > CANONICAL_ROUTE_ARTIFACT_MAX_BYTES {
            return Err(CanonicalLexicalError::Invalid(
                "canonical lexical-route artifact exceeds its fixed serialization ceiling"
                    .to_owned(),
            ));
        }
        Ok(bytes)
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CanonicalLexicalError> {
        if bytes.len() > CANONICAL_ROUTE_ARTIFACT_MAX_BYTES {
            return Err(CanonicalLexicalError::Invalid(
                "canonical lexical-route artifact exceeds its decode ceiling".to_owned(),
            ));
        }
        let envelope: ArtifactEnvelope = serde_json::from_slice(bytes)
            .map_err(|error| CanonicalLexicalError::Serialization(error.to_string()))?;
        if envelope.schema != CANONICAL_ROUTE_ARTIFACT_SCHEMA
            || envelope.domain != CANONICAL_ROUTE_ARTIFACT_DOMAIN
            || envelope.body.schema != CANONICAL_ROUTE_ARTIFACT_SCHEMA
            || envelope.body.domain != CANONICAL_ROUTE_ARTIFACT_DOMAIN
        {
            return Err(CanonicalLexicalError::Invalid(
                "canonical lexical-route artifact schema/domain is unsupported".to_owned(),
            ));
        }
        validate_kappa_label(&envelope.manifest_kappa, "manifest kappa")?;
        let expected = canonical_kappa(&canonical_json(&envelope.body)?)?;
        if expected != envelope.manifest_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "decoded lexical-route manifest kappa does not reproduce".to_owned(),
            ));
        }
        let artifact = Self {
            manifest_kappa: envelope.manifest_kappa,
            body: envelope.body,
        };
        artifact.validate_transitive()?;
        if artifact.canonical_bytes()? != bytes {
            return Err(CanonicalLexicalError::Invalid(
                "lexical-route artifact bytes are not canonical".to_owned(),
            ));
        }
        Ok(artifact)
    }

    pub fn reconstruct_conversation(&self) -> Result<Vec<Vec<u8>>, CanonicalLexicalError> {
        self.reconstruct_paragraph_inputs().map(|paragraphs| {
            paragraphs
                .into_iter()
                .map(|paragraph| paragraph.sentences.concat())
                .collect()
        })
    }

    fn reconstruct_paragraph_inputs(&self) -> Result<Vec<ParagraphInput>, CanonicalLexicalError> {
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let blobs = self
            .body
            .content_blobs
            .iter()
            .map(|blob| (blob.cid.as_str(), blob))
            .collect::<BTreeMap<_, _>>();
        self.body
            .paragraph_witnesses
            .iter()
            .map(|witness| {
                let mut sentences = Vec::with_capacity(witness.sentences.len());
                for sentence in &witness.sentences {
                    let mut bytes = Vec::new();
                    for route_kappa in &sentence.route_kappas {
                        let record = route_by_kappa.get(route_kappa.as_str()).ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "lexical inverse references an absent route".to_owned(),
                            )
                        })?;
                        let blob =
                            blobs.get(record.body.payload_cid.as_str()).ok_or_else(|| {
                                CanonicalLexicalError::Invalid(
                                    "lexical inverse references an absent payload blob".to_owned(),
                                )
                            })?;
                        if blob.kind != "lexical-payload" {
                            return Err(CanonicalLexicalError::Invalid(
                                "lexical inverse payload reference has the wrong blob kind"
                                    .to_owned(),
                            ));
                        }
                        bytes.extend_from_slice(&decode_hex(
                            &record.body.leading_bytes_hex,
                            "route leading boundary",
                        )?);
                        bytes.extend_from_slice(&decode_hex(&blob.bytes_hex, "lexical payload")?);
                    }
                    bytes.extend_from_slice(&decode_hex(
                        &sentence.trailing_bytes_hex,
                        "sentence trailing boundary",
                    )?);
                    if canonical_kappa(&bytes)? != sentence.source_cid {
                        return Err(CanonicalLexicalError::Invalid(
                            "lexical inverse sentence CID does not reproduce".to_owned(),
                        ));
                    }
                    sentences.push(bytes);
                }
                if canonical_kappa(&sentences.concat())? != witness.source_cid {
                    return Err(CanonicalLexicalError::Invalid(
                        "lexical inverse paragraph CID does not reproduce".to_owned(),
                    ));
                }
                Ok(ParagraphInput { sentences })
            })
            .collect()
    }

    /// Reconstruct the declared identity/turn/paragraph input, not merely a
    /// flattened byte list. This is the complete inverse on schema-1's domain.
    pub fn reconstruct_input(&self) -> Result<ConversationInput, CanonicalLexicalError> {
        let paragraphs = self.reconstruct_paragraph_inputs()?;
        let mut turns = Vec::<TurnInput>::new();
        let mut seen_turns = BTreeSet::new();
        for (witness, paragraph) in self.body.paragraph_witnesses.iter().zip(paragraphs) {
            if turns
                .last()
                .is_none_or(|turn| turn.turn_id != witness.turn_id)
            {
                if !seen_turns.insert(witness.turn_id.as_str()) {
                    return Err(CanonicalLexicalError::Invalid(
                        "lexical inverse found a non-contiguous repeated turn ID".to_owned(),
                    ));
                }
                turns.push(TurnInput {
                    turn_id: witness.turn_id.clone(),
                    paragraphs: Vec::new(),
                });
            }
            let turn = turns.last_mut().ok_or_else(|| {
                CanonicalLexicalError::Invalid("lexical inverse turn is absent".to_owned())
            })?;
            if usize::from(witness.paragraph_index) != turn.paragraphs.len() {
                return Err(CanonicalLexicalError::Invalid(
                    "lexical inverse paragraph indexes are not contiguous".to_owned(),
                ));
            }
            turn.paragraphs.push(paragraph);
        }
        let input = ConversationInput {
            identity_scope: self.body.provenance.identity_scope.clone(),
            global_epoch: self.body.provenance.global_epoch.clone(),
            global_snapshot_units: self
                .body
                .global_snapshot
                .ordered_units
                .iter()
                .map(|unit| {
                    self.body
                        .vocabulary
                        .get(
                            usize::try_from(unit.lexical_unit_id)
                                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        )
                        .ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "global snapshot lexical unit is absent from vocabulary".to_owned(),
                            )
                        })
                        .and_then(|binding| {
                            decode_hex(&binding.surface_hex, "global snapshot lexical unit")
                        })
                })
                .collect::<Result<Vec<_>, CanonicalLexicalError>>()?,
            turns,
        };
        validate_input_shape(&input)?;
        Ok(input)
    }

    pub fn attention_hierarchy_view(&self) -> AttentionHierarchyView {
        AttentionHierarchyView {
            current: self.body.hierarchy_roots.local.current.clone(),
            previous: self.body.hierarchy_roots.local.previous.clone(),
            last_two: self.body.hierarchy_roots.local.last_two.clone(),
            sentence: self.body.hierarchy_roots.sentence.clone(),
            paragraph: self.body.hierarchy_roots.paragraph.clone(),
            conversation: self.body.hierarchy_roots.conversation.clone(),
            global: self.body.hierarchy_roots.global.clone(),
        }
    }

    /// Derive a versioned associative, noncommutative H4 fold overlay from
    /// the frozen route and hierarchy links. The S0 artifact and the legacy
    /// attention trace are not reserialized or assigned a new identity.
    pub fn attention_consumer_trace_with_ordered_h4(
        &self,
        table: &H4BinaryIcosahedralClosure,
    ) -> Result<AttentionOrderedFoldTrace, CanonicalLexicalError> {
        validate_ordered_h4_table_exact(table)?;
        if self.body.icosian_profile.h4_root_table.table_kappa != table.h4_root_table_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "ordered H4 overlay table is not the source artifact's root table".to_owned(),
            ));
        }
        let base_trace = self.attention_consumer_trace()?;

        let views = self.attention_hierarchy_view();
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let node_by_kappa = self
            .body
            .hierarchy_nodes
            .iter()
            .map(|node| (node.node_kappa.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let current = route_by_kappa
            .get(views.current.as_str())
            .copied()
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid("ordered H4 current-route root is absent".to_owned())
            })?;
        let previous = route_by_kappa
            .get(views.previous.as_str())
            .copied()
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "ordered H4 previous-route root is absent".to_owned(),
                )
            })?;
        let expected_levels = [
            "current",
            "previous",
            "last-two",
            "sentence",
            "paragraph",
            "conversation",
            "global",
        ];
        if base_trace.ordered_levels.len() != expected_levels.len()
            || base_trace
                .ordered_levels
                .iter()
                .zip(expected_levels)
                .any(|(level, expected)| level.level != expected)
        {
            return Err(CanonicalLexicalError::Invalid(
                "ordered H4 overlay requires the fixed seven-level attention order".to_owned(),
            ));
        }
        let mut base_levels = base_trace.ordered_levels.into_iter();
        let mut memo = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        let mut ordered_levels = vec![
            ordered_h4_trace_level(
                "current",
                ordered_h4_route_aggregate(current, table)?,
                base_levels.next().ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "ordered H4 current consumer level is absent".to_owned(),
                    )
                })?,
                table,
            )?,
            ordered_h4_trace_level(
                "previous",
                ordered_h4_route_aggregate(previous, table)?,
                base_levels.next().ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "ordered H4 previous consumer level is absent".to_owned(),
                    )
                })?,
                table,
            )?,
        ];
        for (level, expected_scope, kappa) in [
            ("last-two", "local", views.last_two.as_str()),
            ("sentence", "sentence", views.sentence.as_str()),
            ("paragraph", "paragraph", views.paragraph.as_str()),
            ("conversation", "conversation", views.conversation.as_str()),
            ("global", "global", views.global.as_str()),
        ] {
            let node = node_by_kappa.get(kappa).copied().ok_or_else(|| {
                CanonicalLexicalError::Invalid(format!(
                    "ordered H4 {level} hierarchy root is absent"
                ))
            })?;
            if node.body.scope != expected_scope {
                return Err(CanonicalLexicalError::Invalid(format!(
                    "ordered H4 {level} hierarchy root has scope {}",
                    node.body.scope
                )));
            }
            let aggregate = ordered_h4_node_aggregate(
                kappa,
                table,
                &node_by_kappa,
                &route_by_kappa,
                &self.body.global_snapshot,
                &self.body.lexical_route_addresses,
                &mut memo,
                &mut visiting,
            )?;
            let consumer_level = base_levels.next().ok_or_else(|| {
                CanonicalLexicalError::Invalid(format!(
                    "ordered H4 {level} consumer level is absent"
                ))
            })?;
            ordered_levels.push(ordered_h4_trace_level(
                level,
                aggregate,
                consumer_level,
                table,
            )?);
        }
        if base_levels.next().is_some() {
            return Err(CanonicalLexicalError::Invalid(
                "ordered H4 overlay found an unexpected attention consumer level".to_owned(),
            ));
        }

        let mut trace = AttentionOrderedFoldTrace {
            schema: ATTENTION_ORDERED_H4_FOLD_TRACE_SCHEMA,
            domain: ATTENTION_ORDERED_H4_FOLD_TRACE_DOMAIN.to_owned(),
            overlay_kappa: String::new(),
            source_artifact_manifest_kappa: self.manifest_kappa.clone(),
            h4_root_table_kappa: table.h4_root_table_kappa.clone(),
            multiplication_table_kappa: table.multiplication_table_kappa.clone(),
            leaf_assignment: "S(route) = H4[prime mod 120] in the canonical root order".to_owned(),
            composition_order: "left-to-right S(A || B) = S(A) * S(B)".to_owned(),
            ordered_levels,
        };
        let mut seed = trace.clone();
        seed.overlay_kappa.clear();
        trace.overlay_kappa = canonical_kappa(&canonical_json(&seed)?)?;
        Ok(trace)
    }

    /// Return the fixed current -> global consumer order and every geometric
    /// state field #952 is allowed to score. Exact identities remain separate
    /// from bounded summaries and digest values are never used as distances.
    pub fn attention_consumer_trace(
        &self,
    ) -> Result<AttentionConsumerTrace, CanonicalLexicalError> {
        self.validate_transitive()?;
        let views = self.attention_hierarchy_view();
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let current = route_by_kappa
            .get(views.current.as_str())
            .copied()
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid("attention current-route root is absent".to_owned())
            })?;
        let previous = route_by_kappa
            .get(views.previous.as_str())
            .copied()
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid("attention previous-route root is absent".to_owned())
            })?;
        let mut ordered_levels = vec![
            attention_level_from_route("current", current, Some(&views.previous))?,
            attention_level_from_route("previous", previous, None)?,
        ];
        for (level, kappa) in [
            ("last-two", views.last_two.as_str()),
            ("sentence", views.sentence.as_str()),
            ("paragraph", views.paragraph.as_str()),
            ("conversation", views.conversation.as_str()),
            ("global", views.global.as_str()),
        ] {
            let node = self.node(kappa).ok_or_else(|| {
                CanonicalLexicalError::Invalid(format!(
                    "attention {level} hierarchy root is absent"
                ))
            })?;
            ordered_levels.push(attention_level_from_node(level, node));
        }
        Ok(AttentionConsumerTrace {
            schema: ATTENTION_CONSUMER_TRACE_SCHEMA,
            domain: ATTENTION_CONSUMER_TRACE_DOMAIN.to_owned(),
            artifact_manifest_kappa: self.manifest_kappa.clone(),
            embedded_spin_manifest_kappa: self.body.spin_manifest.manifest_kappa.clone(),
            codec_kappa: self.body.codec.codec_kappa.clone(),
            vocabulary_kappa: self.body.codec.vocabulary_kappa.clone(),
            chart_profile_kappa: self.body.chart_profile.profile_kappa.clone(),
            icosian_profile_kappa: self.body.icosian_profile.profile_kappa.clone(),
            h4_root_table_kappa: self.body.icosian_profile.h4_root_table.table_kappa.clone(),
            icosian_operator_table_kappa: self
                .body
                .icosian_profile
                .operator_table
                .table_kappa
                .clone(),
            global_snapshot_kappa: self.body.global_snapshot.snapshot_kappa.clone(),
            scope_ceilings: self.body.scope_ceilings.clone(),
            ordered_levels,
        })
    }

    /// Resolve the cursor's causal state into the same numeric route and
    /// hierarchy fields used by terminal attention. Missing slots are explicit
    /// until their source boundary has been observed.
    pub fn attention_consumer_trace_for_cursor(
        &self,
        cursor: &IncrementalHierarchyCursor,
    ) -> Result<IncrementalAttentionConsumerTrace, CanonicalLexicalError> {
        if cursor.artifact_manifest_kappa != self.manifest_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "incremental cursor belongs to a different artifact".to_owned(),
            ));
        }
        let state = cursor.state();
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        if state.current_route.is_none()
            && (state.previous_route.is_some()
                || state.last_two_identity_kappa.is_some()
                || state.sentence_root.is_some()
                || state.paragraph_root.is_some()
                || state.conversation_root.is_some())
        {
            return Err(CanonicalLexicalError::Invalid(
                "incremental hierarchy contains descendants without a current route".to_owned(),
            ));
        }
        let current = state
            .current_route
            .as_deref()
            .map(|kappa| {
                route_by_kappa.get(kappa).copied().ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "incremental current route is absent from the artifact".to_owned(),
                    )
                })
            })
            .transpose()?;
        let previous = state
            .previous_route
            .as_deref()
            .map(|kappa| {
                route_by_kappa.get(kappa).copied().ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "incremental previous route is absent from the artifact".to_owned(),
                    )
                })
            })
            .transpose()?;
        if previous.is_some_and(|record| {
            current.is_none_or(|current_record| {
                record.body.occurrence >= current_record.body.occurrence
            })
        }) {
            return Err(CanonicalLexicalError::Invalid(
                "incremental local route order is noncausal".to_owned(),
            ));
        }
        let current_trace = current
            .map(|record| {
                attention_level_from_route("current", record, state.previous_route.as_deref())
            })
            .transpose()?;
        let previous_trace = previous
            .map(|record| attention_level_from_route("previous", record, None))
            .transpose()?;
        let last_two_trace = match (current, state.last_two_identity_kappa.as_deref()) {
            (Some(current), Some(expected_kappa)) => {
                let provenance_kappa =
                    canonical_kappa(&canonical_json(&self.body.hierarchy_provenance)?)?;
                let mut nodes = Vec::with_capacity(2);
                let node = append_local_window_node(
                    &mut nodes,
                    previous,
                    current,
                    &self.body.provenance.identity_scope,
                    &provenance_kappa,
                )?;
                let expected_children = if previous.is_some() { 2 } else { 1 };
                if node.node_kappa != expected_kappa
                    || node.body.scope != "local"
                    || node.body.ordered_child_kappa != current.route_kappa
                    || node.body.child_count != expected_children
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "incremental last-two numeric state does not reproduce".to_owned(),
                    ));
                }
                Some(attention_level_from_node("last-two", &node))
            }
            (None, None) => None,
            _ => {
                return Err(CanonicalLexicalError::Invalid(
                    "incremental current and last-two roots disagree".to_owned(),
                ));
            }
        };
        let hierarchy_trace =
            |level: &str,
             root: Option<&str>|
             -> Result<Option<AttentionLevelTrace>, CanonicalLexicalError> {
                root.map(|kappa| {
                    let node = self.node(kappa).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(format!(
                            "incremental {level} root is absent from the artifact"
                        ))
                    })?;
                    if node.body.scope != level {
                        return Err(CanonicalLexicalError::Invalid(format!(
                            "incremental {level} root has the wrong hierarchy scope"
                        )));
                    }
                    Ok(attention_level_from_node(level, node))
                })
                .transpose()
            };
        let sentence_trace = hierarchy_trace("sentence", state.sentence_root.as_deref())?;
        let paragraph_trace = hierarchy_trace("paragraph", state.paragraph_root.as_deref())?;
        let conversation_trace =
            hierarchy_trace("conversation", state.conversation_root.as_deref())?;
        let global_trace = hierarchy_trace("global", state.global_root.as_deref())?;
        let ordered_levels = [
            ("current", current_trace),
            ("previous", previous_trace),
            ("last-two", last_two_trace),
            ("sentence", sentence_trace),
            ("paragraph", paragraph_trace),
            ("conversation", conversation_trace),
            ("global", global_trace),
        ]
        .into_iter()
        .map(|(level, trace)| IncrementalAttentionLevelSlot {
            level: level.to_owned(),
            trace,
        })
        .collect();
        Ok(IncrementalAttentionConsumerTrace {
            schema: INCREMENTAL_ATTENTION_CONSUMER_TRACE_SCHEMA,
            domain: INCREMENTAL_ATTENTION_CONSUMER_TRACE_DOMAIN.to_owned(),
            artifact_manifest_kappa: self.manifest_kappa.clone(),
            embedded_spin_manifest_kappa: self.body.spin_manifest.manifest_kappa.clone(),
            codec_kappa: self.body.codec.codec_kappa.clone(),
            vocabulary_kappa: self.body.codec.vocabulary_kappa.clone(),
            chart_profile_kappa: self.body.chart_profile.profile_kappa.clone(),
            icosian_profile_kappa: self.body.icosian_profile.profile_kappa.clone(),
            h4_root_table_kappa: self.body.icosian_profile.h4_root_table.table_kappa.clone(),
            icosian_operator_table_kappa: self
                .body
                .icosian_profile
                .operator_table
                .table_kappa
                .clone(),
            global_snapshot_kappa: self.body.global_snapshot.snapshot_kappa.clone(),
            scope_ceilings: self.body.scope_ceilings.clone(),
            ordered_levels,
        })
    }

    /// Replay the fixed-shape state mutations encoded by the append chains.
    /// Lexical observations change local + sentence state; declared sentence
    /// and paragraph boundaries then change paragraph and conversation state
    /// in causal source order. The fixed global epoch is published first and
    /// then remains immutable for every lexical observation.
    pub fn incremental_update_trace(
        &self,
    ) -> Result<IncrementalHierarchyTrace, CanonicalLexicalError> {
        self.validate_transitive()?;
        let mut events = vec![IncrementalHierarchyEvent {
            event_index: 0,
            event_kind: "publish-global-epoch".to_owned(),
            source_identity_kappa: self.body.provenance.global_epoch.clone(),
            changed_scopes: vec!["global".to_owned()],
            resulting_identity_kappas: vec![self.body.hierarchy_roots.global.clone()],
        }];
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let provenance_kappa = canonical_kappa(&canonical_json(&self.body.hierarchy_provenance)?)?;
        let mut local_previous: Option<&RouteRecord> = None;
        for witness in &self.body.paragraph_witnesses {
            let mut final_paragraph_node = None;
            for sentence in &witness.sentences {
                let boundary_identity = format!(
                    "turn={};paragraph={};sentence={}",
                    witness.turn_id, witness.paragraph_index, sentence.sentence_index
                );
                for route_kappa in &sentence.route_kappas {
                    let record = route_by_kappa.get(route_kappa.as_str()).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "incremental trace cannot resolve a lexical route".to_owned(),
                        )
                    })?;
                    let sentence_node = self
                        .body
                        .hierarchy_nodes
                        .iter()
                        .find(|node| {
                            node.body.scope == "sentence"
                                && node.body.ordered_child_kind == "route"
                                && node.body.ordered_child_kappa == record.route_kappa
                                && node.body.boundary_identity == boundary_identity
                        })
                        .ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "incremental trace cannot resolve a sentence append".to_owned(),
                            )
                        })?;
                    let mut local_nodes = Vec::new();
                    let local_node = append_local_window_node(
                        &mut local_nodes,
                        local_previous,
                        record,
                        &self.body.provenance.identity_scope,
                        &provenance_kappa,
                    )?;
                    events.push(IncrementalHierarchyEvent {
                        event_index: u32::try_from(events.len())
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                        event_kind: "observe-lexical-unit".to_owned(),
                        source_identity_kappa: record.route_kappa.clone(),
                        changed_scopes: vec!["local".to_owned(), "sentence".to_owned()],
                        resulting_identity_kappas: vec![
                            local_node.node_kappa,
                            sentence_node.node_kappa.clone(),
                        ],
                    });
                    local_previous = Some(record);
                }
                let final_route = sentence.route_kappas.last().ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "incremental trace found an empty sentence witness".to_owned(),
                    )
                })?;
                let sentence_node = self
                    .body
                    .hierarchy_nodes
                    .iter()
                    .find(|node| {
                        node.body.scope == "sentence"
                            && node.body.ordered_child_kappa == *final_route
                            && node.body.boundary_identity == boundary_identity
                    })
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "incremental trace cannot resolve a sentence root".to_owned(),
                        )
                    })?;
                let paragraph_node = self
                    .body
                    .hierarchy_nodes
                    .iter()
                    .find(|node| {
                        node.body.scope == "paragraph"
                            && node.body.ordered_child_kappa == sentence_node.node_kappa
                    })
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "incremental trace cannot resolve a sentence close".to_owned(),
                        )
                    })?;
                events.push(IncrementalHierarchyEvent {
                    event_index: u32::try_from(events.len())
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                    event_kind: "close-sentence".to_owned(),
                    source_identity_kappa: sentence.source_cid.clone(),
                    changed_scopes: vec!["paragraph".to_owned()],
                    resulting_identity_kappas: vec![paragraph_node.node_kappa.clone()],
                });
                final_paragraph_node = Some(paragraph_node);
            }
            let paragraph_node = final_paragraph_node.ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "incremental trace found an empty paragraph witness".to_owned(),
                )
            })?;
            let conversation_node = self
                .body
                .hierarchy_nodes
                .iter()
                .find(|node| {
                    node.body.scope == "conversation"
                        && node.body.ordered_child_kappa == paragraph_node.node_kappa
                })
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "incremental trace cannot resolve a conversation append".to_owned(),
                    )
                })?;
            events.push(IncrementalHierarchyEvent {
                event_index: u32::try_from(events.len())
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                event_kind: "close-paragraph".to_owned(),
                source_identity_kappa: witness.source_cid.clone(),
                changed_scopes: vec!["conversation".to_owned()],
                resulting_identity_kappas: vec![conversation_node.node_kappa.clone()],
            });
        }
        let maximum_changed_states_per_event = events
            .iter()
            .map(|event| event.changed_scopes.len())
            .max()
            .map(u8::try_from)
            .transpose()
            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
            .unwrap_or(0);
        if maximum_changed_states_per_event > 2 {
            return Err(CanonicalLexicalError::Invalid(
                "incremental hierarchy update exceeded its two-state mutation bound".to_owned(),
            ));
        }
        Ok(IncrementalHierarchyTrace {
            schema: INCREMENTAL_UPDATE_TRACE_SCHEMA,
            domain: INCREMENTAL_UPDATE_TRACE_DOMAIN.to_owned(),
            events,
            maximum_changed_states_per_event,
        })
    }

    pub fn incremental_cursor(&self) -> Result<IncrementalHierarchyCursor, CanonicalLexicalError> {
        let trace = self.incremental_update_trace()?;
        Ok(IncrementalHierarchyCursor {
            artifact_manifest_kappa: self.manifest_kappa.clone(),
            events: trace.events,
            next_event: 0,
            state: IncrementalHierarchyState {
                current_route: None,
                previous_route: None,
                last_two_identity_kappa: None,
                sentence_root: None,
                paragraph_root: None,
                conversation_root: None,
                global_root: None,
            },
        })
    }

    pub fn lookup_shared_class(&self, class_kappa: &str) -> Option<&[String]> {
        self.body
            .shared_classes
            .binary_search_by(|row| row.class_kappa.as_str().cmp(class_kappa))
            .ok()
            .and_then(|index| self.body.shared_classes.get(index))
            .map(|row| row.ordered_route_members.as_slice())
    }

    /// Resolve every ordered occurrence in an exact shared spin/torsion class
    /// to the same numeric route-state shape consumed by #952. Payload bytes
    /// remain deduplicated and are referenced by CID.
    pub fn lookup_shared_class_trace(
        &self,
        class_kappa: &str,
    ) -> Result<Option<Vec<AttentionLevelTrace>>, CanonicalLexicalError> {
        self.validate_transitive()?;
        let Some(members) = self.lookup_shared_class(class_kappa) else {
            return Ok(None);
        };
        let route_by_kappa = self
            .body
            .route_records
            .iter()
            .map(|record| (record.route_kappa.as_str(), record))
            .collect::<BTreeMap<_, _>>();
        let traces = members
            .iter()
            .map(|kappa| {
                route_by_kappa
                    .get(kappa.as_str())
                    .copied()
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "shared class references an absent route occurrence".to_owned(),
                        )
                    })
                    .and_then(|record| {
                        attention_level_from_route("shared-class-member", record, None)
                    })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        Ok(Some(traces))
    }

    fn node(&self, kappa: &str) -> Option<&HierarchyNode> {
        self.body
            .hierarchy_nodes
            .binary_search_by(|node| node.node_kappa.as_str().cmp(kappa))
            .ok()
            .and_then(|index| self.body.hierarchy_nodes.get(index))
    }

    fn referenced_kappas(&self) -> Vec<String> {
        let mut kappas = BTreeSet::new();
        kappas.insert(self.manifest_kappa.clone());
        kappas.insert(self.body.codec.codec_kappa.clone());
        kappas.insert(self.body.codec.vocabulary_kappa.clone());
        kappas.insert(self.body.spin_manifest.manifest_kappa.clone());
        kappas.insert(self.body.chart_profile.profile_kappa.clone());
        kappas.insert(self.body.icosian_profile.profile_kappa.clone());
        kappas.insert(self.body.icosian_profile.h4_root_table.table_kappa.clone());
        kappas.insert(self.body.icosian_profile.operator_table.table_kappa.clone());
        kappas.insert(self.body.global_snapshot.snapshot_kappa.clone());
        kappas.insert(self.body.global_snapshot.summary_kappa.clone());
        for address in &self.body.lexical_route_addresses {
            kappas.insert(address.address_kappa.clone());
        }
        for unit in &self.body.global_snapshot.ordered_units {
            kappas.insert(unit.address_kappa.clone());
            kappas.insert(unit.entry_kappa.clone());
        }
        for record in &self.body.route_records {
            kappas.insert(record.route_kappa.clone());
            kappas.insert(record.body.address_kappa.clone());
            kappas.insert(record.body.shared_class_kappa.clone());
            kappas.insert(record.body.icosian.coordinate_kappa.clone());
        }
        for node in &self.body.hierarchy_nodes {
            kappas.insert(node.node_kappa.clone());
            kappas.insert(node.body.exact_chain_kappa.clone());
            kappas.insert(node.body.summary_kappa.clone());
            kappas.insert(node.body.summary.transported_trajectory_kappa.clone());
        }
        kappas.into_iter().collect()
    }

    fn validate_transitive(&self) -> Result<(), CanonicalLexicalError> {
        if self.body.schema != CANONICAL_ROUTE_ARTIFACT_SCHEMA
            || self.body.domain != CANONICAL_ROUTE_ARTIFACT_DOMAIN
        {
            return Err(CanonicalLexicalError::Invalid(
                "lexical-route artifact body schema/domain is unsupported".to_owned(),
            ));
        }
        validate_kappa_label(&self.manifest_kappa, "manifest kappa")?;
        let codec = CanonicalLexicalCodec {
            profile: self.body.codec.clone(),
            vocabulary: self.body.vocabulary.clone(),
        };
        codec.validate()?;
        if self.body.chart_profile != fixed_chart_profile()?
            || self.body.icosian_profile != fixed_icosian_profile()?
        {
            return Err(CanonicalLexicalError::Invalid(
                "chart or icosian profile differs from its revisioned fixed contract".to_owned(),
            ));
        }
        if self.body.bounds
            != (ArtifactBounds {
                maximum_turns: u16::try_from(MAX_TURNS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_paragraphs: u16::try_from(MAX_PARAGRAPHS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_sentences: u16::try_from(MAX_SENTENCES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_lexical_units_per_sentence: u16::try_from(MAX_LEXICAL_UNITS_PER_SENTENCE)
                    .map_err(|_| {
                    CanonicalLexicalError::ArithmeticOverflow
                })?,
                maximum_lexical_units: u16::try_from(MAX_LEXICAL_UNITS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_vocabulary: u16::try_from(MAX_VOCABULARY)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_source_bytes: u32::try_from(MAX_SOURCE_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_artifact_bytes: u32::try_from(CANONICAL_ROUTE_ARTIFACT_MAX_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_identity_scope_bytes: u16::try_from(MAX_IDENTITY_SCOPE_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_global_epoch_bytes: u16::try_from(MAX_GLOBAL_EPOCH_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_turn_id_bytes: u16::try_from(MAX_TURN_ID_BYTES)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                maximum_global_snapshot_units: u16::try_from(MAX_GLOBAL_SNAPSHOT_UNITS)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
                global_epoch_policy:
                    "immutable during a session; replace only by new content-addressed epoch"
                        .to_owned(),
            })
        {
            return Err(CanonicalLexicalError::Invalid(
                "artifact hard bounds are not the fixed S0 contract".to_owned(),
            ));
        }
        if self.body.scope_ceilings != fixed_scope_ceilings()? {
            return Err(CanonicalLexicalError::Invalid(
                "hierarchy scope ceilings are not the fixed S0 state-only contract".to_owned(),
            ));
        }
        if self.body.provenance.compiler != COMPILER_NAME
            || self.body.provenance.compiler_cid != canonical_kappa(COMPILER_IDENTITY_BYTES)?
            || self.body.provenance.identity_scope.trim().is_empty()
            || self.body.provenance.global_epoch.trim().is_empty()
            || self.body.provenance.codec_kappa != self.body.codec.codec_kappa
            || self.body.provenance.cost_profile_cid != self.body.chart_profile.profile_kappa
            || self.body.provenance.source_weights_opened
            || self.body.provenance.teacher_forwards != 0
        {
            return Err(CanonicalLexicalError::Invalid(
                "artifact provenance or serving boundary is inconsistent".to_owned(),
            ));
        }
        if self.body.hierarchy_provenance
            != (HierarchyProvenance {
                schema: 1,
                domain: "uor-r4.incremental-hierarchy-provenance/1".to_owned(),
                compiler_cid: self.body.provenance.compiler_cid.clone(),
                codec_kappa: self.body.codec.codec_kappa.clone(),
                chart_profile_kappa: self.body.chart_profile.profile_kappa.clone(),
                icosian_profile_kappa: self.body.icosian_profile.profile_kappa.clone(),
                identity_scope: self.body.provenance.identity_scope.clone(),
                global_epoch: self.body.provenance.global_epoch.clone(),
            })
        {
            return Err(CanonicalLexicalError::Invalid(
                "incremental hierarchy provenance is not the stable profile binding".to_owned(),
            ));
        }
        let provenance_kappa = canonical_kappa(&canonical_json(&self.body.hierarchy_provenance)?)?;

        if self.body.content_blobs.is_empty()
            || self
                .body
                .content_blobs
                .windows(2)
                .any(|pair| pair[0].cid >= pair[1].cid)
        {
            return Err(CanonicalLexicalError::Invalid(
                "content blobs are not in strict CID order".to_owned(),
            ));
        }
        let mut blob_by_cid = BTreeMap::new();
        for blob in &self.body.content_blobs {
            let bytes = decode_hex(&blob.bytes_hex, "content blob")?;
            if canonical_kappa(&bytes)? != blob.cid {
                return Err(CanonicalLexicalError::Invalid(
                    "content blob CID does not reproduce".to_owned(),
                ));
            }
            if blob.kind != "lexical-payload" && blob.kind != "prime-route-spin-manifest" {
                return Err(CanonicalLexicalError::Invalid(
                    "content blob kind is unsupported".to_owned(),
                ));
            }
            blob_by_cid.insert(blob.cid.as_str(), blob);
        }
        let spin_blob = blob_by_cid
            .get(self.body.spin_manifest.blob_cid.as_str())
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "embedded spin-manifest blob reference is absent".to_owned(),
                )
            })?;
        if spin_blob.kind != "prime-route-spin-manifest"
            || self.body.spin_manifest.schema != 2
            || self.body.spin_manifest.domain != "uor-r4.prime-route-spin-manifest/2"
        {
            return Err(CanonicalLexicalError::Invalid(
                "embedded spin-manifest binding is unsupported".to_owned(),
            ));
        }
        let spin_bytes = decode_hex(&spin_blob.bytes_hex, "spin manifest blob")?;
        let spin_manifest = CompiledSpinManifest::decode_canonical(&spin_bytes)?;
        if spin_manifest.manifest_kappa != self.body.spin_manifest.manifest_kappa
            || spin_manifest.bridge.value() != self.body.spin_manifest.base_bridge_tag
            || self.body.spin_manifest.base_bridge_tag != ZeroPowerBridge::ContinuousNull.value()
            || spin_manifest.maximum_candidates.get() != CHILD_MAX_CANDIDATES
            || spin_manifest.provenance.tokenizer_cid != self.body.codec.codec_kappa
            || spin_manifest.provenance.corpus_cid != self.body.provenance.source_cid
            || spin_manifest.provenance.compiler_cid != self.body.provenance.compiler_cid
            || spin_manifest.provenance.cost_profile_cid != self.body.provenance.cost_profile_cid
        {
            return Err(CanonicalLexicalError::Invalid(
                "embedded spin-manifest identity/provenance does not match its parent".to_owned(),
            ));
        }
        let semantic_atoms = self
            .body
            .vocabulary
            .iter()
            .map(|binding| SemanticAtom {
                semantic_atom_id: format!("lexical-unit-{:08}", binding.unit_id),
                payload_cid: binding.payload_cid.clone(),
            })
            .collect::<Vec<_>>();
        let expected_prime_registry = PrimeRegistry::compile(&semantic_atoms)?;
        if spin_manifest.prime_registry != expected_prime_registry {
            return Err(CanonicalLexicalError::Invalid(
                "embedded prime registry is not the canonical codec registry".to_owned(),
            ));
        }
        if self.body.lexical_route_addresses.len() != self.body.vocabulary.len() {
            return Err(CanonicalLexicalError::Invalid(
                "codec route-address registry is incomplete".to_owned(),
            ));
        }
        let mut addresses_by_unit = BTreeMap::new();
        let mut registered_addresses = Vec::with_capacity(self.body.vocabulary.len());
        let mut registered_address_kappas = Vec::with_capacity(self.body.vocabulary.len());
        let mut seen_payloads = BTreeSet::new();
        let mut seen_address_kappas = BTreeSet::new();
        for (index, (vocabulary, stored)) in self
            .body
            .vocabulary
            .iter()
            .zip(&self.body.lexical_route_addresses)
            .enumerate()
        {
            let expected_unit_id =
                u32::try_from(index).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
            let payload_blob = blob_by_cid
                .get(vocabulary.payload_cid.as_str())
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "codec vocabulary payload blob is absent".to_owned(),
                    )
                })?;
            if payload_blob.kind != "lexical-payload"
                || payload_blob.bytes_hex != vocabulary.surface_hex
            {
                return Err(CanonicalLexicalError::Invalid(
                    "codec vocabulary payload blob does not match its lexical unit".to_owned(),
                ));
            }
            let semantic_id = format!("lexical-unit-{expected_unit_id:08}");
            let prime_binding = expected_prime_registry
                .binding_for_id(&semantic_id)
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "canonical codec prime binding is absent".to_owned(),
                    )
                })?;
            let expected_address = GeometricAddress {
                atom: prime_binding.atom,
                spin: spin_for_binding(expected_unit_id, prime_binding.atom)?,
                radial: ZPhi::new(
                    i64::from(expected_unit_id)
                        .checked_add(1)
                        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?,
                    i64::from(prime_binding.atom.value().rem_euclid(5)),
                ),
                payload_cid: vocabulary.payload_cid.clone(),
            };
            let expected_stored =
                lexical_route_address_binding(expected_unit_id, &expected_address)?;
            if vocabulary.unit_id != expected_unit_id
                || stored != &expected_stored
                || !seen_payloads.insert(vocabulary.payload_cid.as_str())
                || !seen_address_kappas.insert(stored.address_kappa.as_str())
            {
                return Err(CanonicalLexicalError::Invalid(
                    "codec route-address registry is noncanonical or duplicated".to_owned(),
                ));
            }
            addresses_by_unit.insert(expected_unit_id, expected_address.clone());
            registered_addresses.push(expected_address);
            registered_address_kappas.push(stored.address_kappa.clone());
        }
        if spin_manifest.addresses.iter().any(|address| {
            !registered_addresses
                .iter()
                .any(|registered| registered == address)
        }) {
            return Err(CanonicalLexicalError::Invalid(
                "embedded manifest carries an address outside the codec registry".to_owned(),
            ));
        }
        let global_units = self
            .body
            .global_snapshot
            .ordered_units
            .iter()
            .map(|unit| {
                codec
                    .binding(unit.lexical_unit_id)
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "global snapshot references an unknown lexical unit".to_owned(),
                        )
                    })
                    .and_then(|binding| {
                        decode_hex(&binding.surface_hex, "global snapshot lexical unit")
                    })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        let global_input = ConversationInput {
            identity_scope: self.body.provenance.identity_scope.clone(),
            global_epoch: self.body.provenance.global_epoch.clone(),
            global_snapshot_units: global_units,
            turns: Vec::new(),
        };
        let expected_global = build_global_snapshot(
            &codec,
            &global_input,
            &addresses_by_unit,
            &registered_addresses,
            &registered_address_kappas,
            &self.body.chart_profile,
            &self.body.icosian_profile,
        )?;
        if expected_global != self.body.global_snapshot
            || canonical_global_epoch(&global_input.global_snapshot_units)?
                != self.body.provenance.global_epoch
        {
            return Err(CanonicalLexicalError::Invalid(
                "bounded global snapshot does not reproduce transitively".to_owned(),
            ));
        }

        if self.body.route_records.is_empty() || self.body.route_records.len() > MAX_LEXICAL_UNITS {
            return Err(CanonicalLexicalError::Invalid(
                "route-record population is empty or exceeds its bound".to_owned(),
            ));
        }
        let phase_origin = PrimeAtom::new(2)?;
        let mut route_by_kappa = BTreeMap::new();
        for (index, record) in self.body.route_records.iter().enumerate() {
            if record.body.schema != ROUTE_RECORD_SCHEMA
                || record.body.domain != ROUTE_RECORD_DOMAIN
                || usize::try_from(record.body.occurrence)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
                    != index
                || record.body.span_start >= record.body.span_end
                || canonical_kappa(&canonical_json(&record.body)?)? != record.route_kappa
            {
                return Err(CanonicalLexicalError::Invalid(
                    "route occurrence identity or span is invalid".to_owned(),
                ));
            }
            validate_kappa_label(&record.route_kappa, "route kappa")?;
            if route_by_kappa
                .insert(record.route_kappa.as_str(), record)
                .is_some()
            {
                return Err(CanonicalLexicalError::Invalid(
                    "route occurrence kappas are not unique".to_owned(),
                ));
            }
            let vocabulary = codec.binding(record.body.lexical_unit_id).ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "route occurrence references an unknown lexical unit".to_owned(),
                )
            })?;
            if vocabulary.payload_cid != record.body.payload_cid {
                return Err(CanonicalLexicalError::Invalid(
                    "route occurrence payload is outside the codec vocabulary".to_owned(),
                ));
            }
            let payload_blob = blob_by_cid
                .get(record.body.payload_cid.as_str())
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "route occurrence payload blob is absent".to_owned(),
                    )
                })?;
            if payload_blob.kind != "lexical-payload"
                || payload_blob.bytes_hex != vocabulary.surface_hex
            {
                return Err(CanonicalLexicalError::Invalid(
                    "route occurrence payload blob does not match its lexical unit".to_owned(),
                ));
            }
            decode_hex(&record.body.leading_bytes_hex, "route leading boundary")?;
            let address_index = usize::from(record.body.address_index);
            let address = registered_addresses.get(address_index).ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "route occurrence codec address index is out of range".to_owned(),
                )
            })?;
            if self
                .body
                .lexical_route_addresses
                .get(address_index)
                .is_none_or(|binding| binding.lexical_unit_id != record.body.lexical_unit_id)
                || address.atom.value() != record.body.prime
                || address.payload_cid != record.body.payload_cid
                || address.spin.s3.raw() != record.body.s3_spin_q30
                || address.spin.hopf.raw() != record.body.hopf_observation_q30
                || address.spin.fiber.raw() != record.body.fiber_q29
                || address.spin.torsion.raw() != record.body.torsion_q29
                || ZPhi::from(record.body.radial_zphi) != address.radial
                || registered_address_kappas.get(address_index) != Some(&record.body.address_kappa)
                || spin_manifest.addresses.binary_search(address).is_err()
            {
                return Err(CanonicalLexicalError::Invalid(
                    "route occurrence does not exactly bind its codec and child-manifest address"
                        .to_owned(),
                ));
            }
            let mut expected_zeta = [0i32; 8];
            for (target, channel) in expected_zeta.iter_mut().zip(ZETA_SUMMARY_CHANNELS) {
                *target = zeta_phase_delta(channel, phase_origin, address.atom)?.raw();
            }
            if expected_zeta != record.body.zeta_phase_signature_q29
                || chart_witness(
                    record.body.lexical_unit_id,
                    address.spin,
                    &self.body.chart_profile,
                )? != record.body.chart
                || icosian_coordinate(address.atom, &self.body.icosian_profile)?
                    != record.body.icosian
                || shared_class_kappa(address.spin)? != record.body.shared_class_kappa
            {
                return Err(CanonicalLexicalError::Invalid(
                    "route geometry/chart/icosian/shared-class witness does not reproduce"
                        .to_owned(),
                ));
            }
        }
        let lexical_blob_count = self
            .body
            .content_blobs
            .iter()
            .filter(|blob| blob.kind == "lexical-payload")
            .count();
        if lexical_blob_count != self.body.vocabulary.len()
            || self.body.content_blobs.len() != self.body.vocabulary.len() + 1
        {
            return Err(CanonicalLexicalError::Invalid(
                "codec vocabulary payload blobs are incomplete or duplicated".to_owned(),
            ));
        }

        let reconstructed = self.reconstruct_paragraph_inputs()?;
        if reconstructed.len() != self.body.paragraph_witnesses.len() {
            return Err(CanonicalLexicalError::Invalid(
                "paragraph reconstruction cardinality changed".to_owned(),
            ));
        }
        let expected_child_sentences = self
            .body
            .paragraph_witnesses
            .iter()
            .try_fold(0usize, |count, witness| {
                count
                    .checked_add(witness.sentences.len())
                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)
            })?
            .checked_add(1)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        if spin_manifest.rebuild_witnesses.len() != expected_child_sentences
            || spin_manifest.nlets.len() != expected_child_sentences
        {
            return Err(CanonicalLexicalError::Invalid(
                "embedded spin-manifest sentence population differs from its parent".to_owned(),
            ));
        }
        let expected_global_sentence_id =
            format!("global-snapshot/{}", self.body.provenance.global_epoch);
        let global_rebuild = spin_manifest
            .rebuild_witnesses
            .iter()
            .find(|child| child.sentence_id == expected_global_sentence_id)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "embedded spin manifest lacks the global rebuild witness".to_owned(),
                )
            })?;
        let global_nlet = spin_manifest
            .nlets
            .iter()
            .find(|child| child.sentence_id == expected_global_sentence_id)
            .ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "embedded spin manifest lacks the global ordered n-let".to_owned(),
                )
            })?;
        let global_addresses = self
            .body
            .global_snapshot
            .ordered_units
            .iter()
            .map(|unit| {
                registered_addresses
                    .get(usize::from(unit.address_index))
                    .cloned()
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "global codec address index is out of range".to_owned(),
                        )
                    })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        let global_child_address_indices = global_addresses
            .iter()
            .map(|address| {
                spin_manifest
                    .addresses
                    .binary_search(address)
                    .map_err(|_| {
                        CanonicalLexicalError::Invalid(
                            "global address is absent from the embedded manifest".to_owned(),
                        )
                    })
                    .and_then(|index| {
                        u16::try_from(index).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)
                    })
            })
            .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
        let global_primes = global_addresses
            .iter()
            .map(|address| address.atom)
            .collect::<Vec<_>>();
        if global_rebuild.sentence_id != expected_global_sentence_id
            || global_rebuild.address_indices != global_child_address_indices
            || global_nlet.sentence_id != expected_global_sentence_id
            || global_nlet.ordered_primes != global_primes
        {
            return Err(CanonicalLexicalError::Invalid(
                "embedded global route order disagrees with its parent snapshot".to_owned(),
            ));
        }
        let mut witnessed_routes = Vec::new();
        let mut rebuilt_turns = Vec::<TurnInput>::new();
        let mut expected_turn_index = 0usize;
        let mut expected_paragraph_index = 0usize;
        let mut prior_turn: Option<&str> = None;
        let mut prior_conversation_node: Option<&HierarchyNode> = None;
        let mut conversation_child_index = 0usize;
        let mut final_witness_sentence_node: Option<&HierarchyNode> = None;
        let mut final_witness_paragraph_node: Option<&HierarchyNode> = None;
        for (witness, paragraph_input) in self.body.paragraph_witnesses.iter().zip(&reconstructed) {
            let paragraph_bytes = paragraph_input.sentences.concat();
            if canonical_kappa(&paragraph_bytes)? != witness.source_cid
                || witness.sentences.is_empty()
                || witness.sentences.len() != paragraph_input.sentences.len()
            {
                return Err(CanonicalLexicalError::Invalid(
                    "paragraph rebuild witness does not reproduce".to_owned(),
                ));
            }
            if prior_turn != Some(witness.turn_id.as_str()) {
                if rebuilt_turns
                    .iter()
                    .any(|turn| turn.turn_id == witness.turn_id)
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "turn identity reappears non-contiguously".to_owned(),
                    ));
                }
                rebuilt_turns.push(TurnInput {
                    turn_id: witness.turn_id.clone(),
                    paragraphs: Vec::new(),
                });
                expected_turn_index = rebuilt_turns.len() - 1;
                expected_paragraph_index = 0;
                prior_turn = Some(witness.turn_id.as_str());
            }
            if usize::from(witness.paragraph_index) != expected_paragraph_index {
                return Err(CanonicalLexicalError::Invalid(
                    "paragraph indexes are not contiguous within a turn".to_owned(),
                ));
            }
            let paragraph_chain_identity = format!(
                "turn={};paragraph={}",
                witness.turn_id, witness.paragraph_index
            );
            let mut prior_paragraph_node: Option<&HierarchyNode> = None;
            for (sentence_index, (sentence_witness, sentence_bytes)) in witness
                .sentences
                .iter()
                .zip(&paragraph_input.sentences)
                .enumerate()
            {
                if usize::from(sentence_witness.sentence_index) != sentence_index
                    || sentence_witness.route_kappas.is_empty()
                    || canonical_kappa(sentence_bytes)? != sentence_witness.source_cid
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "sentence rebuild witness does not reproduce in declared order".to_owned(),
                    ));
                }
                let boundary_identity = format!(
                    "turn={};paragraph={};sentence={}",
                    witness.turn_id, witness.paragraph_index, sentence_witness.sentence_index
                );
                let mut prior_sentence_node: Option<&HierarchyNode> = None;
                for (ordinal, route_kappa) in sentence_witness.route_kappas.iter().enumerate() {
                    let record = route_by_kappa.get(route_kappa.as_str()).ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "sentence rebuild witness references an absent route".to_owned(),
                        )
                    })?;
                    if usize::from(record.body.turn) != expected_turn_index
                        || usize::from(record.body.paragraph) != expected_paragraph_index
                        || usize::from(record.body.sentence) != sentence_index
                        || usize::from(record.body.ordinal_in_sentence) != ordinal
                    {
                        return Err(CanonicalLexicalError::Invalid(
                            "route occurrence metadata disagrees with declared sentence order"
                                .to_owned(),
                        ));
                    }
                    let sentence_node = self
                        .body
                        .hierarchy_nodes
                        .iter()
                        .find(|node| {
                            node.body.scope == "sentence"
                                && node.body.ordered_child_kappa == *route_kappa
                                && node.body.boundary_identity == boundary_identity
                                && node.body.chain_identity == boundary_identity
                        })
                        .ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "sentence chain omits a declared lexical route".to_owned(),
                            )
                        })?;
                    if sentence_node.body.previous_chain_kappa.as_deref()
                        != prior_sentence_node.map(|node| node.node_kappa.as_str())
                        || usize::try_from(sentence_node.body.child_count)
                            .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
                            != ordinal + 1
                    {
                        return Err(CanonicalLexicalError::Invalid(
                            "sentence hierarchy does not reset and append in lexical order"
                                .to_owned(),
                        ));
                    }
                    prior_sentence_node = Some(sentence_node);
                    witnessed_routes.push(route_kappa.as_str());
                }
                let sentence_node = prior_sentence_node.ok_or_else(|| {
                    CanonicalLexicalError::Invalid("sentence hierarchy root is absent".to_owned())
                })?;
                final_witness_sentence_node = Some(sentence_node);
                let reencoded = codec.encode(
                    expected_turn_index,
                    expected_paragraph_index,
                    sentence_bytes,
                )?;
                if reencoded.units.len() != sentence_witness.route_kappas.len()
                    || hex::encode(&reencoded.trailing_bytes) != sentence_witness.trailing_bytes_hex
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "lexical boundary re-encoding changed sentence structure".to_owned(),
                    ));
                }
                for (encoded, route_kappa) in
                    reencoded.units.iter().zip(&sentence_witness.route_kappas)
                {
                    let record = route_by_kappa[route_kappa.as_str()];
                    if encoded.unit_id != record.body.lexical_unit_id
                        || hex::encode(&encoded.leading_bytes) != record.body.leading_bytes_hex
                        || encoded.span_start != record.body.span_start
                        || encoded.span_end != record.body.span_end
                    {
                        return Err(CanonicalLexicalError::Invalid(
                            "lexical unit ID, boundary bytes, or span does not re-encode"
                                .to_owned(),
                        ));
                    }
                }
                let expected_sentence_id = format!(
                    "turn-{expected_turn_index:04}/paragraph-{expected_paragraph_index:04}/sentence-{sentence_index:04}"
                );
                let child_rebuild = spin_manifest
                    .rebuild_witnesses
                    .iter()
                    .find(|child| child.sentence_id == expected_sentence_id)
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "embedded spin manifest lacks a parent sentence witness".to_owned(),
                        )
                    })?;
                let child_nlet = spin_manifest
                    .nlets
                    .iter()
                    .find(|child| child.sentence_id == expected_sentence_id)
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "embedded spin manifest lacks a parent ordered n-let".to_owned(),
                        )
                    })?;
                let expected_address_indices = sentence_witness
                    .route_kappas
                    .iter()
                    .map(|kappa| {
                        let parent_index =
                            usize::from(route_by_kappa[kappa.as_str()].body.address_index);
                        let address = registered_addresses.get(parent_index).ok_or_else(|| {
                            CanonicalLexicalError::Invalid(
                                "parent route address index is out of range".to_owned(),
                            )
                        })?;
                        spin_manifest
                            .addresses
                            .binary_search(address)
                            .map_err(|_| {
                                CanonicalLexicalError::Invalid(
                                    "parent route address is absent from child manifest".to_owned(),
                                )
                            })
                            .and_then(|index| {
                                u16::try_from(index)
                                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)
                            })
                    })
                    .collect::<Result<Vec<_>, CanonicalLexicalError>>()?;
                let expected_primes = sentence_witness
                    .route_kappas
                    .iter()
                    .map(|kappa| PrimeAtom::new(route_by_kappa[kappa.as_str()].body.prime))
                    .collect::<Result<Vec<_>, PrimeRouteError>>()?;
                if child_rebuild.address_indices != expected_address_indices
                    || child_nlet.ordered_primes != expected_primes
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "embedded spin-manifest order disagrees with parent lexical routes"
                            .to_owned(),
                    ));
                }

                let paragraph_node = self
                    .body
                    .hierarchy_nodes
                    .iter()
                    .find(|node| {
                        node.body.scope == "paragraph"
                            && node.body.ordered_child_kappa == sentence_node.node_kappa
                            && node.body.boundary_identity == boundary_identity
                            && node.body.chain_identity == paragraph_chain_identity
                    })
                    .ok_or_else(|| {
                        CanonicalLexicalError::Invalid(
                            "paragraph chain does not bind its declared sentence boundary"
                                .to_owned(),
                        )
                    })?;
                if paragraph_node.body.previous_chain_kappa.as_deref()
                    != prior_paragraph_node.map(|node| node.node_kappa.as_str())
                    || usize::try_from(paragraph_node.body.child_count)
                        .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
                        != sentence_index + 1
                {
                    return Err(CanonicalLexicalError::Invalid(
                        "paragraph hierarchy does not reset and append in sentence order"
                            .to_owned(),
                    ));
                }
                prior_paragraph_node = Some(paragraph_node);
            }
            let final_paragraph_node = prior_paragraph_node.ok_or_else(|| {
                CanonicalLexicalError::Invalid(
                    "paragraph final hierarchy node is absent".to_owned(),
                )
            })?;
            final_witness_paragraph_node = Some(final_paragraph_node);
            let conversation_boundary = format!(
                "turn={};paragraph={}",
                witness.turn_id, witness.paragraph_index
            );
            let conversation_node = self
                .body
                .hierarchy_nodes
                .iter()
                .find(|node| {
                    node.body.scope == "conversation"
                        && node.body.ordered_child_kappa == final_paragraph_node.node_kappa
                        && node.body.boundary_identity == conversation_boundary
                        && node.body.chain_identity == self.body.provenance.identity_scope
                })
                .ok_or_else(|| {
                    CanonicalLexicalError::Invalid(
                        "conversation chain does not bind the exact turn/paragraph identity"
                            .to_owned(),
                    )
                })?;
            if conversation_node.body.previous_chain_kappa.as_deref()
                != prior_conversation_node.map(|node| node.node_kappa.as_str())
                || usize::try_from(conversation_node.body.child_count)
                    .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?
                    != conversation_child_index + 1
            {
                return Err(CanonicalLexicalError::Invalid(
                    "conversation hierarchy does not append paragraphs in exact turn order"
                        .to_owned(),
                ));
            }
            prior_conversation_node = Some(conversation_node);
            conversation_child_index = conversation_child_index
                .checked_add(1)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            rebuilt_turns
                .last_mut()
                .ok_or_else(|| CanonicalLexicalError::Invalid("rebuilt turn is absent".to_owned()))?
                .paragraphs
                .push(paragraph_input.clone());
            expected_paragraph_index = expected_paragraph_index
                .checked_add(1)
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        }
        if final_witness_sentence_node.map(|node| node.node_kappa.as_str())
            != Some(self.body.hierarchy_roots.sentence.as_str())
            || final_witness_paragraph_node.map(|node| node.node_kappa.as_str())
                != Some(self.body.hierarchy_roots.paragraph.as_str())
            || prior_conversation_node.map(|node| node.node_kappa.as_str())
                != Some(self.body.hierarchy_roots.conversation.as_str())
        {
            return Err(CanonicalLexicalError::Invalid(
                "public hierarchy roots do not terminate the exact witnessed chains".to_owned(),
            ));
        }
        if witnessed_routes
            != self
                .body
                .route_records
                .iter()
                .map(|record| record.route_kappa.as_str())
                .collect::<Vec<_>>()
        {
            return Err(CanonicalLexicalError::Invalid(
                "paragraph rebuild witnesses do not cover routes exactly in order".to_owned(),
            ));
        }
        let rebuilt_input = self.reconstruct_input()?;
        if rebuilt_input.turns != rebuilt_turns {
            return Err(CanonicalLexicalError::Invalid(
                "complete lexical inverse disagrees with ordered turn reconstruction".to_owned(),
            ));
        }
        if source_cid(&rebuilt_input)? != self.body.provenance.source_cid {
            return Err(CanonicalLexicalError::Invalid(
                "artifact source CID does not reconstruct from lexical witnesses".to_owned(),
            ));
        }

        self.validate_hierarchy(&route_by_kappa, &provenance_kappa)?;

        let mut recomputed_classes = BTreeMap::<String, SharedSpinTorsionClass>::new();
        for record in &self.body.route_records {
            let row = recomputed_classes
                .entry(record.body.shared_class_kappa.clone())
                .or_insert_with(|| SharedSpinTorsionClass {
                    class_kappa: record.body.shared_class_kappa.clone(),
                    s3_spin_q30: record.body.s3_spin_q30,
                    hopf_observation_q30: record.body.hopf_observation_q30,
                    fiber_q29: record.body.fiber_q29,
                    torsion_q29: record.body.torsion_q29,
                    ordered_route_members: Vec::new(),
                });
            if row.s3_spin_q30 != record.body.s3_spin_q30
                || row.hopf_observation_q30 != record.body.hopf_observation_q30
                || row.fiber_q29 != record.body.fiber_q29
                || row.torsion_q29 != record.body.torsion_q29
            {
                return Err(CanonicalLexicalError::Invalid(
                    "shared exact class combines non-identical spin/torsion state".to_owned(),
                ));
            }
            row.ordered_route_members.push(record.route_kappa.clone());
        }
        if recomputed_classes.into_values().collect::<Vec<_>>() != self.body.shared_classes {
            return Err(CanonicalLexicalError::Invalid(
                "shared spin/torsion index is incomplete or non-canonical".to_owned(),
            ));
        }

        if canonical_kappa(&canonical_json(&self.body)?)? != self.manifest_kappa {
            return Err(CanonicalLexicalError::Invalid(
                "artifact body kappa does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }
}

fn validate_input_shape(input: &ConversationInput) -> Result<(), CanonicalLexicalError> {
    if input.identity_scope.trim().is_empty() || input.global_epoch.trim().is_empty() {
        return Err(CanonicalLexicalError::Invalid(
            "identity scope and global epoch must be non-empty".to_owned(),
        ));
    }
    if input.identity_scope.len() > MAX_IDENTITY_SCOPE_BYTES
        || input.global_epoch.len() > MAX_GLOBAL_EPOCH_BYTES
    {
        return Err(CanonicalLexicalError::Invalid(
            "identity scope or global epoch exceeds its metadata bound".to_owned(),
        ));
    }
    validate_kappa_label(&input.global_epoch, "global epoch")?;
    if input.global_snapshot_units.is_empty()
        || input.global_snapshot_units.len() > MAX_GLOBAL_SNAPSHOT_UNITS
        || canonical_global_epoch(&input.global_snapshot_units)? != input.global_epoch
    {
        return Err(CanonicalLexicalError::Invalid(
            "global epoch must content-address 1..=64 declared lexical snapshot units".to_owned(),
        ));
    }
    if input.turns.is_empty() || input.turns.len() > MAX_TURNS {
        return Err(CanonicalLexicalError::Invalid(format!(
            "S0 input must contain 1..={MAX_TURNS} turns"
        )));
    }
    let mut turn_ids = BTreeSet::new();
    let mut paragraphs = 0usize;
    let mut sentences = 0usize;
    let mut source_bytes = 0usize;
    let mut lexical_units = 0usize;
    for unit in &input.global_snapshot_units {
        source_bytes = source_bytes
            .checked_add(unit.len())
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        let (segments, trailing) = segment_paragraph(unit)?;
        if segments.len() != 1
            || !segments[0].leading.is_empty()
            || segments[0].start != 0
            || segments[0].end != unit.len()
            || !trailing.is_empty()
        {
            return Err(CanonicalLexicalError::Invalid(
                "each global snapshot entry must be exactly one boundary-free lexical unit"
                    .to_owned(),
            ));
        }
        lexical_units = lexical_units
            .checked_add(1)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    }
    for turn in &input.turns {
        if turn.turn_id.trim().is_empty()
            || turn.turn_id.len() > MAX_TURN_ID_BYTES
            || !turn_ids.insert(turn.turn_id.as_str())
        {
            return Err(CanonicalLexicalError::Invalid(
                "turn IDs must be non-empty and unique".to_owned(),
            ));
        }
        if turn.paragraphs.is_empty() {
            return Err(CanonicalLexicalError::Invalid(
                "every turn requires at least one paragraph".to_owned(),
            ));
        }
        paragraphs = paragraphs
            .checked_add(turn.paragraphs.len())
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        for paragraph in &turn.paragraphs {
            if paragraph.sentences.is_empty() {
                return Err(CanonicalLexicalError::Invalid(
                    "paragraphs must contain at least one declared sentence".to_owned(),
                ));
            }
            sentences = sentences
                .checked_add(paragraph.sentences.len())
                .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            for sentence in &paragraph.sentences {
                source_bytes = source_bytes
                    .checked_add(sentence.len())
                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
                let (segments, _) = segment_paragraph(sentence)?;
                if segments.is_empty() {
                    return Err(CanonicalLexicalError::Invalid(
                        "declared sentences must contain at least one lexical unit".to_owned(),
                    ));
                }
                if segments.len() > MAX_LEXICAL_UNITS_PER_SENTENCE {
                    return Err(CanonicalLexicalError::Invalid(format!(
                        "declared sentence exceeds its {MAX_LEXICAL_UNITS_PER_SENTENCE}-unit bound"
                    )));
                }
                lexical_units = lexical_units
                    .checked_add(segments.len())
                    .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
            }
        }
    }
    if paragraphs > MAX_PARAGRAPHS
        || sentences > MAX_SENTENCES
        || source_bytes > MAX_SOURCE_BYTES
        || lexical_units > MAX_LEXICAL_UNITS
    {
        return Err(CanonicalLexicalError::Invalid(
            "S0 input exceeds its paragraph, sentence, byte, or lexical-unit bound".to_owned(),
        ));
    }
    Ok(())
}

fn source_cid(input: &ConversationInput) -> Result<String, CanonicalLexicalError> {
    canonical_kappa(&canonical_json(&SourceWire {
        schema: 1,
        domain: "uor-r4.canonical-lexical-source/1",
        identity_scope: &input.identity_scope,
        global_epoch: &input.global_epoch,
        global_snapshot_units_hex: input
            .global_snapshot_units
            .iter()
            .map(hex::encode)
            .collect(),
        turns: input
            .turns
            .iter()
            .map(|turn| SourceTurnWire {
                turn_id: &turn.turn_id,
                paragraphs: turn
                    .paragraphs
                    .iter()
                    .map(|paragraph| SourceParagraphWire {
                        sentences_hex: paragraph.sentences.iter().map(hex::encode).collect(),
                    })
                    .collect(),
            })
            .collect(),
    })?)
}

pub fn canonical_global_epoch(
    ordered_lexical_units: &[Vec<u8>],
) -> Result<String, CanonicalLexicalError> {
    canonical_kappa(&canonical_json(&GlobalEpochWire {
        schema: 1,
        domain: "uor-r4.bounded-global-lexical-snapshot/1",
        ordered_lexical_units_hex: ordered_lexical_units.iter().map(hex::encode).collect(),
    })?)
}

fn content_blob(kind: &str, bytes: &[u8]) -> Result<ContentBlob, CanonicalLexicalError> {
    Ok(ContentBlob {
        cid: canonical_kappa(bytes)?,
        kind: kind.to_owned(),
        bytes_hex: hex::encode(bytes),
    })
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalLexicalError> {
    serde_json::to_vec(value)
        .map_err(|error| CanonicalLexicalError::Serialization(error.to_string()))
}

#[derive(Serialize)]
struct ByteIdentityWire {
    schema: u32,
    domain: &'static str,
    bytes_hex: String,
}

fn canonical_kappa(bytes: &[u8]) -> Result<String, CanonicalLexicalError> {
    let identity_bytes = canonical_json(&ByteIdentityWire {
        schema: 1,
        domain: "uor-r4.canonical-byte-identity/1",
        bytes_hex: hex::encode(bytes),
    })?;
    let label = uor_addr::json::address_blake3(&identity_bytes)
        .map(|outcome| outcome.address.to_string())
        .map_err(|error| CanonicalLexicalError::Addressing(format!("{error:?}")))?;
    validate_kappa_label(&label, "generated kappa")?;
    Ok(label)
}

fn validate_kappa_label(value: &str, field: &str) -> Result<(), CanonicalLexicalError> {
    let digest = value.strip_prefix("blake3:").ok_or_else(|| {
        CanonicalLexicalError::Invalid(format!(
            "{field} must use canonical lowercase blake3:<64 hex> syntax"
        ))
    })?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CanonicalLexicalError::Invalid(format!(
            "{field} must use canonical lowercase blake3:<64 hex> syntax"
        )));
    }
    Ok(())
}

fn decode_hex(value: &str, field: &str) -> Result<Vec<u8>, CanonicalLexicalError> {
    if value.len().is_multiple_of(2)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return hex::decode(value).map_err(|error| {
            CanonicalLexicalError::Invalid(format!("{field} is not canonical hex: {error}"))
        });
    }
    Err(CanonicalLexicalError::Invalid(format!(
        "{field} is not canonical lowercase even-length hex"
    )))
}

/// Run the single bounded product witness authorized by issue #961.
///
/// This is intentionally a fixed fixture rather than a general evaluation
/// harness. A successful return means every asserted reversible-ingestion
/// invariant held. It does not establish attention, generation, correctness,
/// or reasoning.
pub fn run_authorized_probe() -> Result<ProbeWitness, CanonicalLexicalError> {
    let global_snapshot_units = vec![b"memory".to_vec(), b"route".to_vec()];
    let accepted = ConversationInput {
        identity_scope: "issue-961/fixed-two-turn-session".to_owned(),
        global_epoch: canonical_global_epoch(&global_snapshot_units)?,
        global_snapshot_units,
        turns: vec![
            TurnInput {
                turn_id: "turn-0001".to_owned(),
                paragraphs: vec![ParagraphInput {
                    sentences: vec![b"route route opens.".to_vec(), b" Memory waits.".to_vec()],
                }],
            },
            TurnInput {
                turn_id: "turn-0002".to_owned(),
                paragraphs: vec![ParagraphInput {
                    sentences: vec![b"memory route returns.".to_vec()],
                }],
            },
        ],
    };
    let codec = CanonicalLexicalCodec::compile(&accepted)?;
    let (unknown_unit_status, unknown_unit_surface_hex) = match codec.encode(2, 0, b"unmapped") {
        Err(CanonicalLexicalError::UnknownUnit { surface_hex, .. }) => (
            "REJECTED_UNKNOWN_UNIT_BEFORE_STATE_MUTATION".to_owned(),
            surface_hex,
        ),
        Err(error) => return Err(error),
        Ok(_) => {
            return Err(CanonicalLexicalError::Invalid(
                "fixed probe unknown lexical unit was accepted".to_owned(),
            ));
        }
    };

    let artifact = CanonicalRouteArtifact::ingest(&codec, &accepted)?;
    let prefix_input = ConversationInput {
        identity_scope: accepted.identity_scope.clone(),
        global_epoch: accepted.global_epoch.clone(),
        global_snapshot_units: accepted.global_snapshot_units.clone(),
        turns: accepted.turns[..1].to_vec(),
    };
    let prefix_artifact = CanonicalRouteArtifact::ingest(&codec, &prefix_input)?;
    let bytes_before = artifact.canonical_bytes()?;
    let decoded = CanonicalRouteArtifact::decode_canonical(&bytes_before)?;
    let bytes_after = decoded.canonical_bytes()?;
    let reconstructed = decoded.reconstruct_conversation()?;
    let expected_paragraphs = accepted
        .turns
        .iter()
        .flat_map(|turn| turn.paragraphs.iter())
        .map(|paragraph| paragraph.sentences.concat())
        .collect::<Vec<_>>();

    let spin_blob = decoded
        .body
        .content_blobs
        .iter()
        .find(|blob| blob.cid == decoded.body.spin_manifest.blob_cid)
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid(
                "fixed probe cannot resolve embedded spin manifest".to_owned(),
            )
        })?;
    let spin_manifest = CompiledSpinManifest::decode_canonical(&decode_hex(
        &spin_blob.bytes_hex,
        "fixed-probe spin manifest",
    )?)?;
    let route_unit = decoded
        .body
        .vocabulary
        .iter()
        .find(|binding| binding.surface_hex == hex::encode(b"route"))
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid("fixed probe route unit is absent".to_owned())
        })?;
    let route_prime = spin_manifest
        .prime_registry
        .binding_for_id(&format!("lexical-unit-{:08}", route_unit.unit_id))
        .map(|binding| binding.atom)
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid("fixed probe route prime binding is absent".to_owned())
        })?;
    let p_squared_semiprime_present = spin_manifest
        .experts
        .iter()
        .any(|expert| expert.factors == [route_prime, route_prime]);

    let matching_routes = decoded
        .body
        .route_records
        .iter()
        .filter(|record| record.body.lexical_unit_id == route_unit.unit_id)
        .collect::<Vec<_>>();
    let shared_class_kappa = matching_routes
        .first()
        .map(|record| record.body.shared_class_kappa.clone())
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid("fixed probe shared class is absent".to_owned())
        })?;
    if matching_routes
        .iter()
        .any(|record| record.body.shared_class_kappa != shared_class_kappa)
    {
        return Err(CanonicalLexicalError::Invalid(
            "fixed probe repeated routes did not share exact spin/torsion class".to_owned(),
        ));
    }
    let expected_members = matching_routes
        .iter()
        .map(|record| record.route_kappa.clone())
        .collect::<Vec<_>>();
    let looked_up_traces = decoded
        .lookup_shared_class_trace(&shared_class_kappa)?
        .ok_or_else(|| {
            CanonicalLexicalError::Invalid("fixed probe shared-class lookup missed".to_owned())
        })?;
    let looked_up_members = looked_up_traces
        .iter()
        .map(|trace| trace.identity_kappa.clone())
        .collect::<Vec<_>>();
    let shared_class_lookup_reaches_all = looked_up_members == expected_members;

    let hierarchy_views = decoded.attention_hierarchy_view();
    let exact_node = decoded.node(&hierarchy_views.conversation).ok_or_else(|| {
        CanonicalLexicalError::Invalid("fixed probe conversation root is absent".to_owned())
    })?;
    let exact_identity_kappa = exact_node.body.exact_chain_kappa.clone();
    let geometric_summary_kappa = exact_node.body.summary_kappa.clone();
    let exact_identity_distinct_from_summary = exact_identity_kappa != geometric_summary_kappa;
    let all_hierarchy_roots_present = decoded.node(&hierarchy_views.last_two).is_some()
        && decoded.node(&hierarchy_views.sentence).is_some()
        && decoded.node(&hierarchy_views.paragraph).is_some()
        && decoded.node(&hierarchy_views.conversation).is_some()
        && decoded.node(&hierarchy_views.global).is_some()
        && decoded
            .body
            .route_records
            .iter()
            .any(|record| record.route_kappa == hierarchy_views.current)
        && decoded
            .body
            .route_records
            .iter()
            .any(|record| record.route_kappa == hierarchy_views.previous);
    let bridge_modes_present = decoded
        .body
        .route_records
        .iter()
        .map(|record| record.body.chart.bridge_mode.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let tangent_pole_quarter_turn_present = decoded.body.route_records.iter().any(|record| {
        let chart = &record.body.chart;
        chart.cos_q30 == 0
            && chart.active_chart == "cotangent-complement"
            && !chart.tangent_evaluated
            && chart.quarter_turn_orientation != 0
            && chart.phase_shift_q29 == i32::from(chart.quarter_turn_orientation) * QUARTER_TURN_Q29
            && chart.torsion_shift_q29 == chart.phase_shift_q29
    });
    let icosian_inverse_witnesses_exact = decoded.body.route_records.iter().all(|record| {
        record.body.icosian.inverse_exact
            && record.body.icosian.e8_basis_coordinates
                == record.body.icosian.reconstructed_e8_basis_coordinates
    });
    let referenced_kappas_identical = artifact.referenced_kappas() == decoded.referenced_kappas();
    let attention_consumer = decoded.attention_consumer_trace()?;
    let attention_consumer_order = attention_consumer
        .ordered_levels
        .iter()
        .map(|level| level.level.clone())
        .collect::<Vec<_>>();
    let expected_attention_order = [
        "current",
        "previous",
        "last-two",
        "sentence",
        "paragraph",
        "conversation",
        "global",
    ];
    let attention_consumer_contract_exact = attention_consumer_order
        == expected_attention_order.map(str::to_owned)
        && attention_consumer.scope_ceilings == fixed_scope_ceilings()?
        && attention_consumer
            .ordered_levels
            .iter()
            .all(|level| level.chart_inverse_exact)
        && attention_consumer
            .ordered_levels
            .iter()
            .take(2)
            .all(|level| {
                level.payload_cid.is_some()
                    && level.address_kappa.is_some()
                    && level.shared_class_kappa.is_some()
                    && level.paired_h4_e8_coordinate_sum != [0; 8]
            });
    let incremental_trace = decoded.incremental_update_trace()?;
    let mut incremental_cursor = decoded.incremental_cursor()?;
    let mut incremental_cursor_maximum_changed_nodes = 0u8;
    let mut incremental_attention_states_exact = true;
    while let Some(delta) = incremental_cursor.apply_next()? {
        incremental_cursor_maximum_changed_nodes = incremental_cursor_maximum_changed_nodes.max(
            u8::try_from(delta.changed_nodes.len())
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?,
        );
        let causal_attention = decoded.attention_consumer_trace_for_cursor(&incremental_cursor)?;
        let causal_order = causal_attention
            .ordered_levels
            .iter()
            .map(|slot| slot.level.as_str())
            .collect::<Vec<_>>();
        incremental_attention_states_exact &= causal_order == expected_attention_order;
        for change in &delta.changed_nodes {
            let level = if change.scope == "local" {
                "last-two"
            } else {
                change.scope.as_str()
            };
            incremental_attention_states_exact &= causal_attention
                .ordered_levels
                .iter()
                .find(|slot| slot.level == level)
                .and_then(|slot| slot.trace.as_ref())
                .is_some_and(|trace| trace.identity_kappa == change.after_identity_kappa);
        }
        if delta.event_kind == "observe-lexical-unit" {
            incremental_attention_states_exact &= causal_attention.ordered_levels[2]
                .trace
                .as_ref()
                .is_some_and(|trace| trace.level == "last-two");
        }
    }
    let final_incremental_state = incremental_cursor.state();
    let incremental_cursor_final_state_exact = incremental_cursor.remaining_events() == 0
        && final_incremental_state.current_route.as_deref()
            == Some(decoded.body.hierarchy_roots.local.current.as_str())
        && final_incremental_state.previous_route.as_deref()
            == Some(decoded.body.hierarchy_roots.local.previous.as_str())
        && final_incremental_state.sentence_root.as_deref()
            == Some(decoded.body.hierarchy_roots.sentence.as_str())
        && final_incremental_state.paragraph_root.as_deref()
            == Some(decoded.body.hierarchy_roots.paragraph.as_str())
        && final_incremental_state.conversation_root.as_deref()
            == Some(decoded.body.hierarchy_roots.conversation.as_str())
        && final_incremental_state.global_root.as_deref()
            == Some(decoded.body.hierarchy_roots.global.as_str())
        && final_incremental_state.last_two_identity_kappa.as_deref()
            == Some(decoded.body.hierarchy_roots.local.last_two.as_str());
    let final_incremental_attention =
        decoded.attention_consumer_trace_for_cursor(&incremental_cursor)?;
    incremental_attention_states_exact &= final_incremental_attention
        .ordered_levels
        .iter()
        .map(|slot| slot.trace.clone())
        .collect::<Option<Vec<_>>>()
        .is_some_and(|levels| levels == attention_consumer.ordered_levels);
    let incremental_prefix_routes_stable = prefix_artifact
        .body
        .route_records
        .iter()
        .all(|prefix| decoded.body.route_records.contains(prefix));
    // Local current/previous/last-two are intentionally replaced bounded
    // state. Closed sentence/paragraph/conversation nodes and the immutable
    // global epoch are the reusable prefix commitments.
    let incremental_prefix_closed_hierarchy_stable = prefix_artifact
        .body
        .hierarchy_nodes
        .iter()
        .filter(|node| node.body.scope != "local")
        .all(|prefix| decoded.body.hierarchy_nodes.contains(prefix))
        && decoded
            .body
            .hierarchy_nodes
            .iter()
            .any(|node| node.node_kappa == prefix_artifact.body.hierarchy_roots.conversation)
        && prefix_artifact.body.hierarchy_roots.global == decoded.body.hierarchy_roots.global;
    let canonical_bytes_identical = bytes_before == bytes_after;
    let lexical_reconstruction_exact = reconstructed == expected_paragraphs;
    let unique_payload_blobs = decoded
        .body
        .content_blobs
        .iter()
        .filter(|blob| blob.kind == "lexical-payload")
        .count();

    let required_bridge_modes = [
        "continuous-null".to_owned(),
        "discrete-empty-product".to_owned(),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let serving_boundary = ProbeServingBoundary {
        source_model_weights_opened: false,
        teacher_forwards: 0,
        transformer_calls: 0,
        source_attention_calls: 0,
        dense_intelligence_matrix_calls: 0,
        moe_calls: 0,
        ollama_calls: 0,
        hosted_provider_calls: 0,
    };
    let serving_boundary_closed = !serving_boundary.source_model_weights_opened
        && serving_boundary.teacher_forwards == 0
        && serving_boundary.transformer_calls == 0
        && serving_boundary.source_attention_calls == 0
        && serving_boundary.dense_intelligence_matrix_calls == 0
        && serving_boundary.moe_calls == 0
        && serving_boundary.ollama_calls == 0
        && serving_boundary.hosted_provider_calls == 0;
    if !canonical_bytes_identical
        || !referenced_kappas_identical
        || !lexical_reconstruction_exact
        || !p_squared_semiprime_present
        || !all_hierarchy_roots_present
        || !exact_identity_distinct_from_summary
        || !shared_class_lookup_reaches_all
        || !tangent_pole_quarter_turn_present
        || !icosian_inverse_witnesses_exact
        || !attention_consumer_contract_exact
        || incremental_trace.maximum_changed_states_per_event != 2
        || incremental_cursor_maximum_changed_nodes != 2
        || !incremental_cursor_final_state_exact
        || !incremental_attention_states_exact
        || !incremental_prefix_routes_stable
        || !incremental_prefix_closed_hierarchy_stable
        || !serving_boundary_closed
        || bridge_modes_present
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != required_bridge_modes
    {
        return Err(CanonicalLexicalError::Invalid(
            "fixed issue-961 witness did not satisfy every required invariant".to_owned(),
        ));
    }

    Ok(ProbeWitness {
        schema: PROBE_WITNESS_SCHEMA,
        domain: PROBE_WITNESS_DOMAIN.to_owned(),
        verdict: "PASS_REVERSIBLE_STATE_PLUMBING_ONLY".to_owned(),
        artifact_schema: CANONICAL_ROUTE_ARTIFACT_SCHEMA,
        artifact_domain: CANONICAL_ROUTE_ARTIFACT_DOMAIN.to_owned(),
        manifest_kappa_before_reload: artifact.manifest_kappa.clone(),
        manifest_kappa_after_reload: decoded.manifest_kappa.clone(),
        canonical_bytes_cid: canonical_kappa(&bytes_before)?,
        canonical_bytes_len: bytes_before.len(),
        canonical_bytes_identical,
        referenced_kappas_identical,
        codec_kappa: decoded.codec_kappa().to_owned(),
        vocabulary_kappa: decoded.vocabulary_kappa().to_owned(),
        lexical_units: decoded.body.route_records.len(),
        unique_payload_blobs,
        reconstructed_paragraphs_hex: reconstructed.iter().map(hex::encode).collect(),
        lexical_reconstruction_exact,
        unknown_unit_status,
        unknown_unit_surface_hex,
        p_squared_semiprime_present,
        embedded_spin_manifest_kappa: decoded.embedded_spin_manifest_kappa().to_owned(),
        chart_profile_kappa: attention_consumer.chart_profile_kappa,
        icosian_profile_kappa: attention_consumer.icosian_profile_kappa,
        h4_root_table_kappa: attention_consumer.h4_root_table_kappa,
        icosian_operator_table_kappa: attention_consumer.icosian_operator_table_kappa,
        global_snapshot_kappa: attention_consumer.global_snapshot_kappa,
        scope_ceilings: attention_consumer.scope_ceilings,
        attention_consumer_order,
        attention_consumer_contract_exact,
        hierarchy_views,
        all_hierarchy_roots_present,
        transitive_references_present: true,
        maximum_nodes_changed_per_event: incremental_trace.maximum_changed_states_per_event,
        incremental_cursor_maximum_changed_nodes,
        incremental_cursor_final_state_exact,
        incremental_attention_states_exact,
        incremental_prefix_routes_stable,
        incremental_prefix_closed_hierarchy_stable,
        exact_identity_kappa,
        geometric_summary_kappa,
        exact_identity_distinct_from_summary,
        shared_class_kappa,
        shared_class_expected_members: expected_members.len(),
        shared_class_lookup_members: looked_up_members.len(),
        shared_class_lookup_reaches_all,
        bridge_modes_present,
        tangent_pole_quarter_turn_present,
        icosian_inverse_witnesses_exact,
        serving_boundary,
    })
}
