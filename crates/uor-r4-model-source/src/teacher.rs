//! Architecture-keyed teacher dispatch (#657 item 1).
//!
//! The compile and serving paths bind a single teacher type. Before this
//! module that type was the concrete [`HuggingFaceLlamaOracle`], so only the
//! Llama family could be compiled. [`Teacher`] is the architecture-neutral
//! carrier: an enum over the registered source oracles that implements the
//! full [`TeacherOracle`] surface by delegating to whichever variant it holds,
//! plus a factory ([`Teacher::load`] / [`Teacher::load_with_sequence_length`])
//! that reads the source `config.json`'s `model_type` and constructs the right
//! oracle — keyed by the #599 family declarations
//! ([`crate::conformance::AdapterFeatures`]).
//!
//! An **enum**, not `Box<dyn TeacherOracle>`, because the two oracles expose
//! genuinely divergent inherent surfaces (e.g. `cfg()` returns `&Config` for
//! Llama and `&Gpt2Config` for GPT-2 — different types that cannot share one
//! trait method): a caller that needs an architecture-specific capability
//! matches on the variant, while the shared [`TeacherOracle`] surface is
//! delegated. The Llama variant delegates to the identical executor, so an
//! existing Llama/SmolLM2 compile is byte-unchanged.
//!
//! Family-specific policy stays confined to `uor-r4-model-source` (the
//! architecture-neutral boundary): this module names families only through
//! the conformance declarations and the two oracle constructors.

use std::path::Path;

use crate::conformance::AdapterFeatures;
use crate::{
    BehaviorSource, HuggingFaceGpt2Oracle, HuggingFaceLlamaOracle, RepresentationSource,
    SourceUnavailable, TeacherOracle, TraceCaptureGeometry, TraceCaptureRequest, TraceCaptureSinks,
};

/// A loaded source teacher of a known architecture. Implements the entire
/// [`TeacherOracle`] surface by delegating to the held oracle; construct with
/// the architecture-keyed [`Teacher::load`] / [`Teacher::load_with_sequence_length`].
pub enum Teacher {
    /// A Llama-family Hugging Face teacher (the pre-#657 default).
    Llama(HuggingFaceLlamaOracle),
    /// A GPT-2-family Hugging Face teacher (#607).
    Gpt2(HuggingFaceGpt2Oracle),
}

/// The registered source families, resolved from `config.json`'s `model_type`.
enum Family {
    Llama,
    Gpt2,
}

/// Resolve a family from a `model_type` label (pure — testable without a
/// snapshot). An **absent** `model_type` resolves to Llama: every caller
/// before #657 loaded [`HuggingFaceLlamaOracle`] unconditionally, so this
/// preserves that exact behavior (and keeps Llama/SmolLM2 compiles unchanged).
/// A known label routes to its family; any other label fails closed rather
/// than being approximated by a family that does not execute it.
fn family_for_model_type(model_type: Option<&str>) -> Result<Family, SourceUnavailable> {
    let Some(label) = model_type else {
        return Ok(Family::Llama);
    };
    if AdapterFeatures::huggingface_gpt2()
        .model_types
        .iter()
        .any(|known| known == label)
    {
        return Ok(Family::Gpt2);
    }
    if AdapterFeatures::huggingface_llama()
        .model_types
        .iter()
        .any(|known| known == label)
    {
        return Ok(Family::Llama);
    }
    Err(SourceUnavailable::new(format!(
        "config.json model_type {label:?} has no registered teacher family \
         (expected one of the #599 families: llama, gpt2)"
    )))
}

/// Read `<source>/config.json` and resolve its family.
fn detect_family(source: &Path) -> Result<Family, SourceUnavailable> {
    let config_path = source.join("config.json");
    let text = std::fs::read_to_string(&config_path)
        .map_err(|error| SourceUnavailable::new(format!("{}: {error}", config_path.display())))?;
    let config: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        SourceUnavailable::new(format!(
            "{}: not valid JSON: {error}",
            config_path.display()
        ))
    })?;
    family_for_model_type(config.get("model_type").and_then(|value| value.as_str()))
}

impl Teacher {
    /// Load the architecture-appropriate teacher from a snapshot directory,
    /// keyed by `config.json`'s `model_type`.
    pub fn load(source: impl AsRef<Path>) -> Result<Self, SourceUnavailable> {
        Self::dispatch(source.as_ref(), None)
    }

    /// Load with a bounded teacher context (the compile path's short
    /// trajectories), keyed by `config.json`'s `model_type`.
    pub fn load_with_sequence_length(
        source: impl AsRef<Path>,
        sequence_length: usize,
    ) -> Result<Self, SourceUnavailable> {
        Self::dispatch(source.as_ref(), Some(sequence_length))
    }

    fn dispatch(source: &Path, sequence_length: Option<usize>) -> Result<Self, SourceUnavailable> {
        match detect_family(source)? {
            Family::Llama => Ok(Self::Llama(match sequence_length {
                Some(len) => HuggingFaceLlamaOracle::load_with_sequence_length(source, len)?,
                None => HuggingFaceLlamaOracle::load(source)?,
            })),
            Family::Gpt2 => Ok(Self::Gpt2(match sequence_length {
                Some(len) => HuggingFaceGpt2Oracle::load_with_sequence_length(source, len)?,
                None => HuggingFaceGpt2Oracle::load(source)?,
            })),
        }
    }

    /// Toggle the experimental #602 R4 route-attention operator. Llama maps
    /// this to its two current registered operators (`standard-source-attention/2`
    /// off, `experimental-r4-source-attention/2` on). GPT-2 has no such
    /// switch — its learned-absolute operator is a separate registry entry
    /// (#657 item 3) — so the toggle is a documented no-op there rather than
    /// silently reinterpreting a Llama switch it does not execute.
    pub fn set_r4_attention(&mut self, enable: bool) {
        match self {
            Self::Llama(oracle) => oracle.set_r4_attention(enable),
            Self::Gpt2(_) => {}
        }
    }

    /// The held teacher as a `&dyn TeacherOracle` (shared delegation target).
    fn as_oracle(&self) -> &dyn TeacherOracle {
        match self {
            Self::Llama(oracle) => oracle,
            Self::Gpt2(oracle) => oracle,
        }
    }

    /// The held teacher as a `&mut dyn TeacherOracle` (mutable delegation).
    fn as_oracle_mut(&mut self) -> &mut dyn TeacherOracle {
        match self {
            Self::Llama(oracle) => oracle,
            Self::Gpt2(oracle) => oracle,
        }
    }
}

impl RepresentationSource for Teacher {
    fn vocab_size(&self) -> usize {
        self.as_oracle().vocab_size()
    }
    fn source_dimension(&self) -> usize {
        self.as_oracle().source_dimension()
    }
    fn tokenizer_address(&self) -> &str {
        self.as_oracle().tokenizer_address()
    }
    fn read_embedding_rows(&self, range: std::ops::Range<usize>, output: &mut [f32]) -> Option<()> {
        self.as_oracle().read_embedding_rows(range, output)
    }
}

impl BehaviorSource for Teacher {
    fn reset(&mut self) {
        self.as_oracle_mut().reset();
    }
    fn step(&mut self, token: usize, pos: usize, logits: &mut [f32]) {
        self.as_oracle_mut().step(token, pos, logits);
    }
}

impl TeacherOracle for Teacher {
    fn vocab(&self) -> usize {
        self.as_oracle().vocab()
    }
    fn dim(&self) -> usize {
        self.as_oracle().dim()
    }
    fn seq_len(&self) -> usize {
        self.as_oracle().seq_len()
    }
    fn bos_token(&self) -> usize {
        self.as_oracle().bos_token()
    }
    fn eos_token(&self) -> usize {
        self.as_oracle().eos_token()
    }
    fn kappa(&self) -> String {
        self.as_oracle().kappa()
    }
    fn source_bytes(&self) -> usize {
        self.as_oracle().source_bytes()
    }
    fn embedding(&self, token: usize, out: &mut [f32]) {
        self.as_oracle().embedding(token, out);
    }
    fn geometry_projection(&self) -> Option<crate::geometry::GeometryProjection> {
        self.as_oracle().geometry_projection()
    }
    fn attention_operator_spec(&self) -> Option<crate::attention::AttentionOperatorSpec> {
        self.as_oracle().attention_operator_spec()
    }
    fn hidden_state(&self) -> Option<&[f32]> {
        self.as_oracle().hidden_state()
    }
    fn top_k(&self, k: usize, out: &mut [(u32, f32)]) -> usize {
        self.as_oracle().top_k(k, out)
    }
    fn trace_capture_geometry(&self) -> Option<TraceCaptureGeometry> {
        self.as_oracle().trace_capture_geometry()
    }
    fn step_with_trace_capture(
        &mut self,
        token: usize,
        pos: usize,
        logits: &mut [f32],
        request: &TraceCaptureRequest<'_>,
        sinks: &mut TraceCaptureSinks<'_, '_>,
    ) -> bool {
        self.as_oracle_mut()
            .step_with_trace_capture(token, pos, logits, request, sinks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #657: the architecture router. An absent model_type stays Llama (the
    // pre-#657 default, so existing compiles are unchanged); the two declared
    // families route to themselves; an unregistered label fails closed rather
    // than being run by a family that does not execute it. Pure — runs in CI
    // without any snapshot.
    #[test]
    fn model_type_routes_to_the_declared_family() {
        assert!(matches!(family_for_model_type(None), Ok(Family::Llama)));
        assert!(matches!(
            family_for_model_type(Some("llama")),
            Ok(Family::Llama)
        ));
        assert!(matches!(
            family_for_model_type(Some("gpt2")),
            Ok(Family::Gpt2)
        ));
        assert!(family_for_model_type(Some("mistral")).is_err());
        assert!(family_for_model_type(Some("")).is_err());
    }

    // #657 regression gate: routing a Llama/SmolLM2 source through `Teacher`
    // must be behaviorally identical to the concrete `HuggingFaceLlamaOracle`
    // — same κ/geometry and bit-identical forward logits — so a compile
    // through the dispatched path is byte-unchanged (the compile is
    // deterministic; identical oracle behavior ⇒ identical artifact bytes).
    // Presence-gated on local weights; `#[ignore]` (loads a 135M checkpoint):
    //   R4_TEACHER_PARITY_SOURCE=/abs/path/to/smollm2-135m-instruct \
    //   cargo test -p uor-r4-model-source --release teacher_llama_matches -- --ignored --nocapture
    #[test]
    #[ignore = "#657 SmolLM2 parity; needs local weights — run with --ignored + R4_TEACHER_PARITY_SOURCE"]
    fn teacher_llama_matches_direct_oracle_on_smollm2() {
        let source = std::env::var("R4_TEACHER_PARITY_SOURCE")
            .unwrap_or_else(|_| ".uor-models/sources/smollm2-135m-instruct".to_owned());
        if !std::path::Path::new(&source).join("config.json").is_file() {
            eprintln!("teacher parity: source absent at {source}, skipping (κ-test convention)");
            return;
        }
        let seq = 8;
        let mut via_enum = Teacher::load_with_sequence_length(&source, seq).expect("Teacher::load");
        let mut direct =
            HuggingFaceLlamaOracle::load_with_sequence_length(&source, seq).expect("direct load");

        assert!(
            matches!(via_enum, Teacher::Llama(_)),
            "a llama-family source must route to Teacher::Llama"
        );
        assert_eq!(via_enum.kappa(), direct.kappa(), "source κ diverged");
        assert_eq!(via_enum.vocab(), direct.vocab());
        assert_eq!(via_enum.dim(), direct.dim());
        assert_eq!(via_enum.seq_len(), direct.seq_len());

        // One forward step from BOS must yield bit-identical logits.
        let vocab = via_enum.vocab();
        let (mut le, mut ld) = (vec![0f32; vocab], vec![0f32; vocab]);
        let bos = via_enum.bos_token();
        via_enum.reset();
        direct.reset();
        via_enum.step(bos, 0, &mut le);
        direct.step(bos, 0, &mut ld);
        let bit_identical = le
            .iter()
            .zip(ld.iter())
            .all(|(a, b)| a.to_bits() == b.to_bits());
        assert!(
            bit_identical,
            "forward logits diverged between Teacher::Llama and the direct oracle"
        );
        eprintln!(
            "#657 parity — Teacher::Llama == HuggingFaceLlamaOracle on {source} \
             (κ {}, vocab {vocab}, {} logits bit-identical)",
            via_enum.kappa(),
            le.len()
        );
    }
}
