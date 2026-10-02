//! Standalone integer numerical runtime for retained UOR-R4 packed models.
//!
//! Model values use integer/additive arithmetic and artifact-bound tables.
//! Loading parses legacy JSON metadata; tokenization, reports, and offline
//! floating table construction are outside this numerical runtime. Dense
//! parameter access, finite context and allocations remain explicit limitations.

#![forbid(unsafe_code)]

pub mod bundle;
pub mod capability_api;
pub mod codec;
pub mod config;
pub mod format;
pub mod generation;
pub mod geometric_context;
pub mod geometric_event;
pub mod geometric_potential;
pub mod geometric_read;
pub mod geometric_span;
pub mod geometric_value;
pub mod h4_classifier;
pub mod h4_tables;
pub mod identity_latch;
pub mod lorentz;
pub mod math;
pub mod model;
mod packed_rows;
pub mod paired_projection;
pub mod report_output;
pub mod sampling;
pub mod session;
pub mod stack;
pub mod tables;
pub mod wasm;

pub use bundle::{create_test_bundle_with_byte_vocab, Bundle};
pub use capability_api::{
    CapabilityTruthMatrix, IntegerCapabilityApi, ModelCapabilityMetadata, SessionConfig,
    SessionTelemetry, CAPABILITY_API_SCHEMA,
};
pub use config::{JointConfig, ReadGeometry, ReadMode, Transport};
pub use model::{
    atan2_q30, compress_barycenter_key_fibonacci, compute_turn_prime_signature,
    extract_salient_tokens, galois_lfsr_absorb, galois_lfsr_step, score_token_salience,
    HopfFiberPointQ30, IntegerModel, IntegerSession, IntegerStep, L2PrimePage, SessionState,
    SlotTarget, T8ZetaState, UnitS3Q30, AGE_HORIZON_CLAMP, DIALOGUE_CAPACITY, FIBONACCI_WEIGHTS,
    GALOIS_POLY_64, GOLDEN_RATIO_IV_64, KEY_DIM, L2_PAGE_CAPACITY, MAX_SCORES_CAPACITY,
    PERSISTENT_CAPACITY, PROBABILITY_TOTAL, SLOT_PRIMES_224, TOTAL_MEMORY_CANDIDATES,
    TOTAL_MEMORY_CAPACITY, VAL_DIM, ZETA_FREQUENCIES_Q30,
};
pub use sampling::{SamplePolicy, Sampler, SamplingError, PROBABILITY_ONE};
pub use session::{
    ChatMemoryTelemetry, ChatSession, ChatTokenStream, IncrementalUtf8Decoder, RoleToken,
    RoleTokens, SerializedChatSession, SerializedSamplerState, SerializedSessionState,
    StreamStopReason, TokenStream, Utf8StreamBuffer, SESSION_SCHEMA_V1,
};

pub type IntegerModelError = IntegerError;

use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum IntegerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
    /// A model declares an offline read operator with no retained integer path.
    UnsupportedReadGeometry(ReadGeometry),
}
impl fmt::Display for IntegerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "integer runtime I/O: {error}"),
            Self::Json(error) => write!(f, "integer runtime JSON: {error}"),
            Self::Invalid(error) => write!(f, "invalid integer runtime input: {error}"),
            Self::UnsupportedReadGeometry(geometry) => write!(
                f,
                "{} read is offline-only; quantization, packed export and integer serving support dot and lorentz reads",
                geometry.name()
            ),
        }
    }
}
impl std::error::Error for IntegerError {}
impl From<std::io::Error> for IntegerError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for IntegerError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
pub type Result<T> = std::result::Result<T, IntegerError>;
pub(crate) fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut digest = Sha256::new();
    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        digest.update(&buffer[..bytes]);
    }
    Ok(hex::encode(digest.finalize()))
}
