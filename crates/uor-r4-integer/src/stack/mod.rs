//! Multiplier-free integer serving of the lab's geometric stack under owner
//! decision D11 (`docs/integration/DECISIONS.md`, ROADMAP §0 R1–R2).
//!
//! The engine executes the `UORLUT01` stack artifact that
//! `uor-r4-training`'s `stack_export` writes (schema `uor-r4.lut-stack/1`).
//! It reproduces every operation of the frozen D10 comparator
//! (`uor-r4-lut`'s `stack`), including its rounding, truncation and
//! saturation, so that the logits are the same integers. That equality is an
//! empirical criterion: `uor-r4-training`'s `stack_d11_oracle` tests check it
//! position by position on exported random stacks, and
//! `geometric-stack d11-evaluate` measures it on a trained artifact. Each
//! layer is a temporal mixer and then a SwiGLU MLP, both pre-norm residual
//! blocks:
//!
//! - **quaternion transport recurrence** (`r`): per lane of four channels,
//!   `h_t = lambda_t (u_t (x) h_{t-1}) + sqrt(1 - lambda_t^2) c_t` with the
//!   Hamilton product `(x)`, a unit quaternion `u_t` (the identity without
//!   learned rotations), a decay `lambda_t` read from the exp table and a
//!   width-4 causal convolution `c_t` of the drive; the output is
//!   `h_t gelu(g_t)`;
//! - **read** (`a`): heads scored by `<q, k> / sqrt(d)` (Dot) or
//!   `-beta (d(q, k) - offset)` (Lorentz, the hyperboloid distance; L2, its
//!   flat control, the Euclidean distance `|q - k|` from digit-table squares
//!   and a digit-by-digit square root), plus a learned age bias, with a NoRead
//!   slot whose value is zero.
//!
//! Arithmetic (R1–R2):
//!
//! - there is no floating point anywhere in a step;
//! - learned 4-bit weights contribute through table reads and additions:
//!   per-activation tables of the sixteen nibble products, and for the maps
//!   that read the normalized state, pair tables indexed by a whole weight
//!   byte (one read per two weights); a group scale `(16 + m) 2^(e - 4)` is
//!   applied by shifts and additions;
//! - a product of two runtime values (transport, gating, scores, value
//!   mixing, normalization) is read from a table of the sixteen multiples of
//!   one operand at the radix-16 digits of the other; a quotient is exact
//!   long division and a square root is digit by digit;
//! - learned scalars (convolution taps, decay rates, Lorentz and L2 scales)
//!   are grid codes applied by shifts and adds; exp, sigmoid, SiLU, GELU and
//!   arcosh are the artifact's sealed tables.
//!
//! The engine is dense: every weight map is read in full per token
//! ([`IntegerStackModel::weights_per_token`]), the labelled interim stepping
//! stone of R3, and it is single-threaded. The instruction audit of the
//! `uor-r4-stack` binary (`scripts/audit_zero_matmul_serving.py --stack`)
//! checks the step and every kernel for multiply, divide and floating-point
//! instructions.

pub mod flock;
mod format;
pub(crate) mod kernels;
mod session;
#[cfg(test)]
mod tests;

use std::fmt;

pub use flock::{
    flock_select_integer, rank_table_q31, raw_rank_weights_q16, top_k_select_integer, FlockEntry,
    FlockScan, FlockScratch, FlockSelect, FlockSlot, FLOCK_INTEGER_SELECTOR_VERSION,
    MAX_FLOCK_CONTEXT, RAW_RANK_WEIGHTS_Q16, RECIPROCAL_Q32,
};
pub use format::{
    Fixed, StackNumerics, StackPointer, StackShape, StackTransportSnap, GROUP, MAGIC,
    MAX_POINTER_DIM, STACK_POINTER_SCHEMA, STACK_SCHEMA,
};
pub use kernels::{stack_argmax, stack_snap_select};
pub use session::{
    IntegerStackModel, IntegerStackSession, SerializedStackLayerState, SerializedStackSession,
    SnapTraceEntry, CONVOLUTION_WIDTH, STACK_SESSION_SCHEMA,
};

/// SHA-256 of the 120 unit icosians of 2I (the transport snap's roots) as
/// little-endian f32 bytes row by row, in `canonical_h4_roots` order. Equals
/// `TransportSnap::Icosian.roots_sha256()` of the training crate (a
/// cross-crate test there checks it); the loader refuses any other roots.
pub const ICOSIAN_ROOTS_SHA256: &str =
    "69c969ebc9cfcf8c56109eb9d6fc853d446361f1d0c80a7b805ecfa65bbbaa2d";

/// `round(phi * 2^32)`, the golden ratio as a Q32 constant for the snapped
/// transition's `lambda phi` (a `stack_mul_u128` table product).
pub const PHI_Q32: u128 = 6_949_403_065;

/// Why an artifact was rejected or a step refused.
#[derive(Debug)]
pub enum StackError {
    /// The artifact file could not be read.
    Io(std::io::Error),
    /// The first eight bytes are not `UORLUT01`.
    Magic,
    /// The declared header length is too large or reaches past the file.
    HeaderLength { declared: u64, available: usize },
    /// The JSON header does not parse as a stack header.
    Header(serde_json::Error),
    /// The header's schema is not [`STACK_SCHEMA`].
    Schema(String),
    /// The header's group size is not [`GROUP`].
    Group(usize),
    /// The shape breaks a container rule.
    Shape(&'static str),
    /// The data sections start past the end of the file.
    MissingData,
    /// Two sections share a name.
    DuplicateSection(String),
    /// A matrix's sections are out of bounds or of the wrong size.
    MatrixSection(String),
    /// A table's section is out of bounds or of the wrong size.
    TableSection(String),
    /// A matrix or table the shape requires is absent.
    MissingSection(String),
    /// A matrix has the wrong dimensions.
    MatrixShape {
        name: String,
        rows: usize,
        cols: usize,
        expected_rows: usize,
        expected_cols: usize,
    },
    /// A table has the wrong element type.
    TableKind {
        name: String,
        found: &'static str,
        expected: &'static str,
    },
    /// A table has the wrong number of entries.
    TableLength {
        name: String,
        len: usize,
        expected: usize,
    },
    /// A table of grid codes holds an invalid (or, for a positive scalar, a
    /// non-positive) code.
    GridCodes(String),
    /// The numerics or a sealed table are out of range.
    Numerics(String),
    /// The header records a transport snap or roots this engine does not
    /// know, or a snap on a stack without learned rotations.
    TransportSnap(String),
    /// A token outside the vocabulary.
    Token { token: u32, vocab: usize },
    /// The session has served its whole context.
    ContextFull { context: usize },
    /// A session's layer state does not match its model's layer.
    SessionState,
}

impl fmt::Display for StackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "stack artifact I/O: {error}"),
            Self::Magic => write!(f, "stack artifact: missing UORLUT01 magic"),
            Self::HeaderLength {
                declared,
                available,
            } => write!(
                f,
                "stack artifact: header length {declared} does not fit {available} bytes"
            ),
            Self::Header(error) => write!(f, "stack artifact header: {error}"),
            Self::Schema(schema) => write!(
                f,
                "stack artifact: schema {schema:?} is not the schema for its shape \
                 ({STACK_SCHEMA:?} plain, {STACK_POINTER_SCHEMA:?} with a pointer head)"
            ),
            Self::Group(group) => write!(f, "stack artifact: group size {group} is not {GROUP}"),
            Self::Shape(reason) => write!(f, "stack artifact shape: {reason}"),
            Self::MissingData => write!(f, "stack artifact: data sections missing"),
            Self::DuplicateSection(name) => write!(f, "stack artifact: duplicate section {name}"),
            Self::MatrixSection(name) => write!(f, "stack artifact: matrix {name} is malformed"),
            Self::TableSection(name) => write!(f, "stack artifact: table {name} is malformed"),
            Self::MissingSection(name) => write!(f, "stack artifact: missing section {name}"),
            Self::MatrixShape {
                name,
                rows,
                cols,
                expected_rows,
                expected_cols,
            } => write!(
                f,
                "stack artifact: matrix {name} is {rows}x{cols}, expected {expected_rows}x{expected_cols}"
            ),
            Self::TableKind {
                name,
                found,
                expected,
            } => write!(
                f,
                "stack artifact: table {name} holds {found}, expected {expected}"
            ),
            Self::TableLength {
                name,
                len,
                expected,
            } => write!(
                f,
                "stack artifact: table {name} has {len} entries, expected {expected}"
            ),
            Self::GridCodes(name) => {
                write!(f, "stack artifact: table {name} holds invalid grid codes")
            }
            Self::Numerics(reason) => write!(f, "stack artifact numerics: {reason}"),
            Self::TransportSnap(reason) => write!(
                f,
                "the stack engine refuses a transport snap or roots it does not know: {reason}"
            ),
            Self::Token { token, vocab } => {
                write!(f, "token {token} is outside the vocabulary of {vocab}")
            }
            Self::ContextFull { context } => {
                write!(f, "the session is full ({context} positions)")
            }
            Self::SessionState => write!(f, "session state does not match the model"),
        }
    }
}

impl std::error::Error for StackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Header(error) => Some(error),
            _ => None,
        }
    }
}
