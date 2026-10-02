//! Admission adapter for persistent native geometric attention.
//!
//! This adapter joins independently loaded, source-bound artifacts. It does not
//! introduce another artifact format, reconstruct source shadows on each token,
//! or repeat geometry proofs. The supplied wrappers have private numerical
//! fields and can be constructed only by their existing compiler/loaders. Here
//! their complete dependency identities, token registry, exact algebra, and
//! supported dimensions must agree before any session can be made.
//!
//! Context, potential, value, and NoRead use their strict q4 source policies;
//! the composition bank uses signed geometric selector IDs and q4 gains. Event
//! coefficients and age biases retain their admitted wider source policies.
//! Thus a native integer session is not a claim that every learned coefficient
//! in the attention component already satisfies the four-bit source limit.
//!
//! `session` allocates bounded history/scratch once. Its returned integer
//! session consumes token IDs and returns i64 Q16 residuals with no Tensor,
//! source-model call or floating conversion in `push`. The surrounding stack
//! and its final language head remain a separate floating replay boundary.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uor_r4_integer::geometric_attention::{
    NativeAttentionComponents, NativeAttentionSession, HEADS, LANES, LANES_PER_HEAD, MAX_CONTEXT,
    OUTPUT_WIDTH,
};
use uor_r4_integer::{geometric_context_q4, geometric_potential_q4, geometric_value_q4};

use crate::geometric_composition_native::CompiledComposition;
use crate::geometric_context::{CompiledContext, GeometricContextConfig};
use crate::geometric_event::CompiledEvents;
use crate::geometric_no_read_native::CompiledNoRead;
use crate::geometric_potential_native::CompiledGeometricPotentials;
use crate::geometric_read_native::CompiledGeometricRead;
use crate::geometric_span_native::CompiledSpanActions;
use crate::geometric_stack::{StackArch, StackModel};
use crate::geometric_value_producer_native::CompiledValueProducer;
use crate::{invalid, sha256_bytes, Result};

/// Receipt schema, not a new on-disk numerical artifact or a loader authority.
pub const SCHEMA: &str = "uor-r4.geometric-attention-admission/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionBoundMetadata {
    pub bytes: usize,
    pub sha256: String,
}
fn bound(bytes: &[u8]) -> AttentionBoundMetadata {
    AttentionBoundMetadata {
        bytes: bytes.len(),
        sha256: sha256_bytes(bytes),
    }
}

/// Derived from the actual admitted wrappers; never accepted as an input that
/// could authorize replacement numerical tables.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionAdmissionMetadata {
    pub schema: String,
    pub numerical_session_schema: String,
    pub vocabulary: usize,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub event_lanes: usize,
    pub layer: usize,
    pub context: usize,
    pub output_width: usize,
    pub output_fractional_bits: u32,
    pub output_storage_bits: u32,
    /// Compact serde JSON of the complete metadata of each immutable object.
    pub components: BTreeMap<String, AttentionBoundMetadata>,
    pub tokenizer: AttentionBoundMetadata,
    pub algebra_payload_sha256: String,
    pub algebra_mathematical_sha256: String,
    pub observation_basis_q25_sha256: String,
    pub strict_q4_source_components: Vec<String>,
    pub wider_source_components: Vec<String>,
    pub composition_source_policy: String,
    pub causal_contract: String,
}

/// Borrowed immutable assembly. Construction performs provenance and policy
/// checks once; sessions borrow the admitted kernels for their whole lifetime.
pub struct CompiledGeometricAttention<'a> {
    context: &'a CompiledContext,
    events: &'a CompiledEvents,
    span: &'a CompiledSpanActions,
    potential: &'a CompiledGeometricPotentials,
    reducer: &'a CompiledGeometricRead,
    values: &'a CompiledValueProducer,
    no_read: &'a CompiledNoRead,
    composition: &'a CompiledComposition,
    metadata: AttentionAdmissionMetadata,
}

fn require_metadata(bytes: &[u8], expected_bytes: usize, expected_sha256: &str) -> Result<()> {
    if bytes.len() != expected_bytes || sha256_bytes(bytes) != expected_sha256 {
        return Err(invalid("attention component metadata binding differs"));
    }
    Ok(())
}

fn require_identity(
    actual_bytes: usize,
    actual_sha256: &str,
    expected_bytes: usize,
    expected_sha256: &str,
) -> Result<()> {
    if actual_bytes != expected_bytes || actual_sha256 != expected_sha256 {
        return Err(invalid("attention tokenizer or observation basis differs"));
    }
    Ok(())
}

impl<'a> CompiledGeometricAttention<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: &'a CompiledContext,
        events: &'a CompiledEvents,
        span: &'a CompiledSpanActions,
        potential: &'a CompiledGeometricPotentials,
        reducer: &'a CompiledGeometricRead,
        values: &'a CompiledValueProducer,
        no_read: &'a CompiledNoRead,
        composition: &'a CompiledComposition,
    ) -> Result<Self> {
        let c = context.config();
        c.validate()?;
        let cm = context.metadata();
        let pm = potential.metadata();
        let vm = values.metadata();
        let nm = no_read.metadata();
        let rm = reducer.metadata();
        let sm = span.metadata();
        let em = events.metadata();
        let bm = composition.metadata();
        let cq4 = cm
            .source
            .q4
            .as_ref()
            .ok_or_else(|| invalid("attention requires strict q4 context source"))?;
        let pq4 = pm
            .q4
            .as_ref()
            .ok_or_else(|| invalid("attention requires strict q4 potential source"))?;
        let vq4 = vm
            .q4
            .as_ref()
            .ok_or_else(|| invalid("attention requires strict q4 value source"))?;
        if cm.schema != "uor-r4.geometric-context-native/2"
            || c.schema != crate::geometric_context::Q4_SCHEMA
            || cq4.schema != geometric_context_q4::SCHEMA
            || cq4.policy != geometric_context_q4::POLICY
            || cm.coefficient_policy != geometric_context_q4::POLICY
            || !potential.is_q4()
            || pq4.schema != geometric_potential_q4::SCHEMA
            || pq4.policy != geometric_potential_q4::POLICY
            || pm.policy != geometric_potential_q4::POLICY
            || vm.schema != crate::geometric_value_producer_native::Q4_SCHEMA
            || vm.config.schema != crate::geometric_value_producer::Q4_SCHEMA
            || vq4.schema != geometric_value_q4::SCHEMA
            || vq4.policy != geometric_value_q4::POLICY
            || vm.coefficient_policy != geometric_value_q4::POLICY
            || nm.schema != crate::geometric_no_read_native::SCHEMA
        {
            return Err(invalid("attention source coefficient policy differs"));
        }
        events.config().validate()?;
        pm.address.validate(OUTPUT_WIDTH, HEADS)?;
        sm.span.validate(OUTPUT_WIDTH)?;
        no_read
            .config()
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        bm.config.validate()?;
        if c.width != OUTPUT_WIDTH
            || c.heads != HEADS
            || c.lanes_per_head != LANES_PER_HEAD
            || events.vocab_size() != c.vocab_size
            || span.vocab_size() != c.vocab_size
            || span.lanes() != LANES
            || sm.width != OUTPUT_WIDTH
            || values.config().vocab_size != c.vocab_size
            || values.config().heads != HEADS
            || values.config().latent_lanes_per_head != LANES_PER_HEAD
            || no_read.config().vocabulary != c.vocab_size
            || no_read.config().heads != HEADS
            || no_read.config().latent_lanes_per_head != LANES_PER_HEAD
            || pm.address.heads != HEADS
            || pm.address.lanes_per_head != LANES_PER_HEAD
            || context.metadata().source.frozen.address != pm.address
            || rm.schema != crate::geometric_read_native::SCHEMA
            || rm.layer != 2
            || rm.heads != HEADS
            || rm.context == 0
            || rm.context > MAX_CONTEXT
            || rm.value_width != 16
            || rm.score_fractional_bits != 24
            || rm.value_fractional_bits != 16
            || rm.weight_fractional_bits != 31
            || nm.layer != rm.layer
            || bm.schema != crate::geometric_composition_native::SCHEMA
            || bm.output_width != OUTPUT_WIDTH
            || bm.payload_fractional_bits != 16
            || bm.payload_storage_bits != 64
            || bm.numerical_composition_schema != uor_r4_integer::geometric_composition::SCHEMA
            || bm.numerical_reducer_schema != uor_r4_integer::geometric_composed_read::SCHEMA
        {
            return Err(invalid(
                "attention components do not share the admitted rra H2/L4 shape",
            ));
        }

        // This is a typed join of admitted objects, not acceptance of a caller's
        // replacement hashes. The original compilers independently recompiled
        // each table payload from its actual source before creating an object.
        for (key, bytes) in [
            ("event_native/metadata.json", serde_json::to_vec_pretty(em)?),
            ("span_native/metadata.json", serde_json::to_vec_pretty(sm)?),
            (
                "potential_native/metadata.json",
                serde_json::to_vec_pretty(pm)?,
            ),
        ] {
            let expected = cm
                .source
                .frozen
                .files
                .get(key)
                .ok_or_else(|| invalid("attention context dependency identity absent"))?;
            require_metadata(&bytes, expected.bytes, &expected.sha256)?;
        }
        values.validate_native_context(context)?;
        no_read.validate_dependencies(context, potential, reducer, values, events, span)?;
        composition
            .validate_dependencies(context, potential, reducer, values, events, span, no_read)?;
        require_metadata(
            &serde_json::to_vec_pretty(pm)?,
            rm.potential_metadata.bytes,
            &rm.potential_metadata.sha256,
        )?;
        require_metadata(
            &serde_json::to_vec(rm)?,
            bm.reducer_metadata.bytes,
            &bm.reducer_metadata.sha256,
        )?;

        let tokenizer_bytes = cm.source.tokenizer_bytes;
        let tokenizer_sha256 = &cm.source.tokenizer_sha256;
        if tokenizer_bytes == 0 {
            return Err(invalid("attention token registry is empty"));
        }
        for (bytes, sha) in [
            (
                em.source.tokenizer_bytes,
                em.source.tokenizer_sha256.as_str(),
            ),
            (sm.tokenizer_bytes, sm.tokenizer_sha256.as_str()),
            (pm.tokenizer_bytes, pm.tokenizer_sha256.as_str()),
            (rm.tokenizer.bytes, rm.tokenizer.sha256.as_str()),
            (vm.tokenizer.bytes, vm.tokenizer.sha256.as_str()),
            (nm.tokenizer.bytes, nm.tokenizer.sha256.as_str()),
            (bm.tokenizer.bytes, bm.tokenizer.sha256.as_str()),
        ] {
            require_identity(bytes, sha, tokenizer_bytes, tokenizer_sha256)?;
        }
        let algebra = &cm.source.algebra_payload_sha256;
        let math = &cm.source.algebra_mathematical_sha256;
        for (payload, mathematical) in [
            (
                &em.source.algebra_payload_sha256,
                &em.source.algebra_mathematical_sha256,
            ),
            (&sm.table_payload_sha256, &sm.table_mathematical_sha256),
            (&pm.algebra_payload_sha256, &pm.algebra_mathematical_sha256),
        ] {
            if payload != algebra || mathematical != math {
                return Err(invalid("attention exact algebra ordering differs"));
            }
        }
        if &bm.config.geometry_sha256 != algebra {
            return Err(invalid("attention composition algebra identity differs"));
        }
        let basis_sha = &cq4.basis_sha256;
        for (bytes, sha) in [
            (pq4.basis_q25.bytes, pq4.basis_q25.sha256.as_str()),
            (vq4.basis_q25.bytes, vq4.basis_q25.sha256.as_str()),
            (
                nm.observation_basis.bytes,
                nm.observation_basis.sha256.as_str(),
            ),
        ] {
            require_identity(bytes, sha, 120 * 4 * 4, basis_sha)?;
        }
        // Immutable base lineage is also shared by the dictionaries and the
        // separately trained modules; a token ID alone is not this identity.
        if em.source.base.base_model_sha256 != pm.model_sha256
            || em.source.base.base_config_sha256 != pm.config_sha256
            || sm.model_sha256 != pm.model_sha256
            || sm.config_sha256 != pm.config_sha256
            || em.source.base.base_address_sha256 != pm.address_sidecar_sha256
        {
            return Err(invalid("attention immutable base lineage differs"));
        }
        let components = [
            ("context", serde_json::to_vec(cm)?),
            ("events", serde_json::to_vec(em)?),
            ("span", serde_json::to_vec(sm)?),
            ("potential", serde_json::to_vec(pm)?),
            ("reducer", serde_json::to_vec(rm)?),
            ("values", serde_json::to_vec(vm)?),
            ("no_read", serde_json::to_vec(nm)?),
            ("composition", serde_json::to_vec(bm)?),
        ]
        .into_iter()
        .map(|(name, bytes)| (name.to_owned(), bound(&bytes)))
        .collect();
        let metadata = AttentionAdmissionMetadata {
            schema: SCHEMA.into(),
            numerical_session_schema: uor_r4_integer::geometric_attention::SCHEMA.into(),
            vocabulary: c.vocab_size,
            heads: HEADS,
            lanes_per_head: LANES_PER_HEAD,
            event_lanes: events.config().lanes,
            layer: rm.layer,
            context: rm.context,
            output_width: OUTPUT_WIDTH,
            output_fractional_bits: 16,
            output_storage_bits: 64,
            components,
            tokenizer: AttentionBoundMetadata { bytes: tokenizer_bytes, sha256: tokenizer_sha256.clone() },
            algebra_payload_sha256: algebra.clone(),
            algebra_mathematical_sha256: math.clone(),
            observation_basis_q25_sha256: basis_sha.clone(),
            strict_q4_source_components: ["context", "potential", "values", "no_read"].map(str::to_owned).to_vec(),
            wider_source_components: ["event learned coefficients", "age learned biases"].map(str::to_owned).to_vec(),
            composition_source_policy: "signed-H4 categorical left/right selectors;dimensionless-q4-quarter-gains;exact-group-transport;integer-Q16-decoding".into(),
            causal_contract: "token-only;NEW synchronous context/event;OLD held span;cache each occurrence once;current included;age=query-source;separate head NoRead denominators;span action visible next token;no eviction;reset all history".into(),
        };
        Ok(Self {
            context,
            events,
            span,
            potential,
            reducer,
            values,
            no_read,
            composition,
            metadata,
        })
    }

    pub fn metadata(&self) -> &AttentionAdmissionMetadata {
        &self.metadata
    }
    pub fn config(&self) -> &GeometricContextConfig {
        self.context.config()
    }
    pub fn context(&self) -> &'a CompiledContext {
        self.context
    }
    pub fn events(&self) -> &'a CompiledEvents {
        self.events
    }
    pub fn span(&self) -> &'a CompiledSpanActions {
        self.span
    }
    pub fn potential(&self) -> &'a CompiledGeometricPotentials {
        self.potential
    }
    pub fn reducer(&self) -> &'a CompiledGeometricRead {
        self.reducer
    }
    pub fn values(&self) -> &'a CompiledValueProducer {
        self.values
    }
    pub fn no_read(&self) -> &'a CompiledNoRead {
        self.no_read
    }
    pub fn composition(&self) -> &'a CompiledComposition {
        self.composition
    }

    /// Allocate an empty bounded session. No further metadata serialization,
    /// source access, parameter hashing or float conversion occurs on push.
    pub fn session(&self, capacity: usize) -> Result<NativeAttentionSession<'a>> {
        let components = NativeAttentionComponents {
            events: self.events.native_tables(),
            context: self.context.native_tables(),
            span: self.span.dictionary(),
            potential: self.potential.native_kernel(),
            values: self.values.native_kernel(),
            no_read: self.no_read.native_kernel(),
            composition: self.composition.native_kernel(),
            geometry: self.context.geometry(),
            age_q24: self.reducer.age_q24(),
            age_context: self.reducer.metadata().context,
            exp_q31: self.reducer.exp_q31(),
        };
        NativeAttentionSession::new(components, capacity).map_err(|e| invalid(e.to_string()))
    }

    /// Validate the existing float replay host's dimensions and the original
    /// embedding/age/address lineage it supplies. This is not an assertion that
    /// all live trunk/head parameters equal the saved model bytes. The caller
    /// retains full-model provenance; deliberate unused-map poison controls are
    /// permitted because those maps do not enter the admitted integer reader.
    pub fn validate_stack(&self, model: &StackModel) -> Result<()> {
        model.config.validate()?;
        let c = &model.config;
        if !model.device().is_cpu()
            || c.arch != StackArch::Geometric
            || c.pattern != "rra"
            || c.width != OUTPUT_WIDTH
            || c.heads != HEADS
            || c.vocab_size != self.metadata.vocabulary
            || c.context != self.metadata.context
            || c.select.is_some()
            || c.pointer.is_some()
            || model.served_codec().is_some()
            || model.read_identity_carry()
            || model.read_identity_latch().is_some()
        {
            return Err(invalid(
                "attention session float replay host differs from admitted rra shape",
            ));
        }
        let address = model
            .geometric_address()
            .ok_or_else(|| invalid("attention replay host has no geometric address"))?;
        let span = model
            .geometric_span()
            .ok_or_else(|| invalid("attention replay host has no geometric span"))?;
        self.potential
            .validate_stack_parent(&model.geometric_address_potentials()?, address)?;
        self.span.validate_for(
            model
                .variables()
                .get("embedding.weight")
                .ok_or_else(|| invalid("attention replay embedding absent"))?
                .as_tensor(),
            span,
        )?;
        self.reducer.validate_for(
            model
                .variables()
                .get(&self.reducer.metadata().age_parameter)
                .ok_or_else(|| invalid("attention replay age parameter absent"))?
                .as_tensor(),
            self.potential,
        )?;
        Ok(())
    }
}
