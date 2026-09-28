//! D5 addressing-contest codebooks, assignment, PQ-ADC scoring and decode accounting.
//!
//! This is the Stage 2b, self-contained, offline module for the addressing contest. It defines
//! codebook families over the 64-coordinate read key, fits the ordinary families on supplied tune
//! keys, encodes keys and queries, builds a per-query product-quantization table and scores a
//! quantized key against it. It loads no model, runs no admission, trains nothing and changes no
//! serving default; the contest harness and the restricted-read evaluation are separate stages.
//!
//! # Families
//!
//! | Family | Layout | Codes/block | Index bits/block | Info bits/block |
//! |---|---|---|---|---|
//! | [`CodebookFamily::H4`] | 16 x 4-D | 120 | 7 | 6.9069 (log2 120) |
//! | [`CodebookFamily::E8`] | 8 x 8-D | 240 | 8 | 7.9069 (log2 240) |
//! | [`CodebookFamily::KMeans`] | 16 x 4-D or 8 x 8-D | 120 / 240 | 7 / 8 | log2 K |
//! | [`CodebookFamily::Sign`] | 16 x 4-D or 8 x 8-D | 2^7 / 2^8 | 7 / 8 | index bits |
//!
//! The plan's "6.9 / 7.9 bits per block" are the information values `log2(K)`; the stored index
//! width is `ceil(log2(K))` (7 / 8 bits). [`AddressingArm::bits_per_block`] reports the stored
//! width and [`AddressingArm::information_bits_per_block`] the information value.
//!
//! # Scoring (PQ-ADC)
//!
//! A key is encoded to one code index per block ([`AddressingArm::encode`]). A query builds a
//! table of block inner products against every code ([`AddressingArm::query_lut`]). The score of
//! the key is the sum of the table entries selected by its codes ([`AddressingArm::score`]). This
//! is asymmetric: the key is quantized, the query is not.
//!
//! # Decode scheme (D11 R1/R2)
//!
//! `encode`/`query_lut`/`score` are the **offline f64 reference** of the declared serving decode.
//! The served decode is multiplier-free for every family:
//!
//! - H4/E8 codes are fixed exact constants, so a quantized block product is a product-table
//!   (quarter-square) read plus adds; no multiplier instruction is required.
//! - K-means centroids are stored as at-most-4-bit integers, so their products use the same
//!   quarter-square table.
//! - Sign codes use a Rademacher `±1` projection (`z = P q` is adds only) and each code
//!   coordinate is `±1`, so each table entry is a sum of `±z_j`.
//!
//! [`AddressingArm::decode_work_per_event`] and [`AddressingArm::lut_build_work`] report the
//! declared table reads/adds/multiplies; [`AddressingArm::decode_note`] states the method.

use std::fmt;
use std::path::Path;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;

use crate::TrainingError;

/// Width of the read key the codebooks are defined over.
pub const KEY_WIDTH: usize = 64;
/// Number of canonical H4 roots (the 600-cell vertex set on S3).
pub const H4_CODE_COUNT: usize = 120;
/// Number of normalized E8 roots.
pub const E8_ROOT_COUNT: usize = 240;
/// Contiguous-block layout of the H4 family: 16 blocks of 4.
pub const H4_LAYOUT: (usize, usize) = (16, 4);
/// Contiguous-block layout of the E8 family: 8 blocks of 8.
pub const E8_LAYOUT: (usize, usize) = (8, 8);
/// Number of k-means fit seeds the contest declares.
pub const KMEANS_SEED_COUNT: usize = 3;
/// Three fixed fit seeds for the k-means family.
pub const DEFAULT_SEEDS: [u64; KMEANS_SEED_COUNT] = [
    0x51D5_0001_0000_0001,
    0x51D5_0002_0000_0002,
    0x51D5_0003_0000_0003,
];
/// Maximum Lloyd iterations for the offline k-means fit.
pub const KMEANS_MAX_ITERS: usize = 32;
/// Lloyd convergence tolerance on the largest centroid shift (L2).
pub const KMEANS_TOLERANCE: f64 = 1e-7;

/// The contest randomized-Hadamard pre-rotation seed.
///
/// Shared by the contest harness and the restricted-read admission policy so
/// both decode exactly the same arm. Changing it changes every `*-rht` arm.
pub const CONTEST_RHT_SEED: u64 = 0x51D5_11A7_0000_0001;
/// The contest 4-D sign-code seed.
pub const CONTEST_SIGN4_SEED: u64 = 0x51D5_0004_0000_0004;
/// The contest 8-D sign-code seed.
pub const CONTEST_SIGN8_SEED: u64 = 0x51D5_0008_0000_0008;

/// Codebook family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodebookFamily {
    /// 120 canonical H4 roots, one 4-D code per block.
    H4,
    /// 240 normalized E8 roots, one 8-D code per block.
    E8,
    /// Learned k-means centroids, fitted offline on tune keys.
    KMeans,
    /// Random-projection sign codes (SimHash) at equal stored index bits.
    Sign,
}

impl CodebookFamily {
    /// Stable identifier used in artifacts and reports.
    pub const fn name(self) -> &'static str {
        match self {
            Self::H4 => "h4",
            Self::E8 => "e8",
            Self::KMeans => "kmeans",
            Self::Sign => "sign",
        }
    }
}

impl fmt::Display for CodebookFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Focused error type for the addressing module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressingError {
    /// The supplied key/query has the wrong width.
    KeyWidth { expected: usize, got: usize },
    /// A fit was requested with no tune keys.
    EmptyTuneKeys,
    /// A codebook would be empty.
    EmptyCodebook,
    /// A layout or family-specific field is inconsistent.
    InvalidLayout(String),
    /// A family-specific table was required but absent.
    NotFitted(&'static str),
    /// A code index is outside the block's codebook.
    CodeOutOfRange { code: usize, codes_per_block: usize },
    /// Two values that must agree do not.
    ShapeMismatch(String),
    /// JSON (de)serialization failed.
    Serde(String),
    /// File I/O failed.
    Io(String),
}

impl fmt::Display for AddressingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyWidth { expected, got } => {
                write!(f, "key width mismatch: expected {expected}, got {got}")
            }
            Self::EmptyTuneKeys => write!(f, "k-means fit requires at least one tune key"),
            Self::EmptyCodebook => write!(f, "codebook must have at least one code per block"),
            Self::InvalidLayout(message) => write!(f, "invalid codebook layout: {message}"),
            Self::NotFitted(field) => write!(f, "codebook is not fitted: missing {field}"),
            Self::CodeOutOfRange {
                code,
                codes_per_block,
            } => write!(f, "code {code} outside codebook of {codes_per_block} codes"),
            Self::ShapeMismatch(message) => write!(f, "shape mismatch: {message}"),
            Self::Serde(message) => write!(f, "serde: {message}"),
            Self::Io(message) => write!(f, "io: {message}"),
        }
    }
}

impl std::error::Error for AddressingError {}

impl From<serde_json::Error> for AddressingError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error.to_string())
    }
}

impl From<std::io::Error> for AddressingError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<AddressingError> for TrainingError {
    fn from(error: AddressingError) -> Self {
        TrainingError::Invalid(error.to_string())
    }
}

/// Result alias local to the addressing module.
pub type Result<T> = std::result::Result<T, AddressingError>;

/// Pre-rotation configuration: a seeded randomized Hadamard transform.
///
/// The transform is `(1/sqrt(n)) * H * D` with `D` a seeded random `±1` diagonal and `H` the
/// unnormalized Walsh-Hadamard butterfly. `D` is sign flips and `H` is adds/subtracts only; the
/// `1/sqrt(64) = 1/8` scale is a right shift by three at serving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhtConfig {
    /// Seed for the random `±1` diagonal.
    pub seed: u64,
}

/// The 240 normalized E8 roots.
///
/// Raw E8 roots have squared norm 2; they are scaled by `1/sqrt(2)` here so every code lies on
/// S7 and a dot product is a cosine. 112 roots are `±e_i ± e_j`; 128 are `(±1/2, ..., ±1/2)` with
/// an even number of minus signs.
pub fn e8_roots() -> &'static [[f64; 8]; E8_ROOT_COUNT] {
    static ROOTS: OnceLock<[[f64; 8]; E8_ROOT_COUNT]> = OnceLock::new();
    ROOTS.get_or_init(|| {
        let mut list: Vec<[f64; 8]> = Vec::with_capacity(E8_ROOT_COUNT);
        let inv_sqrt2 = 1.0 / 2.0f64.sqrt();

        // 28 pairs * 4 sign patterns = 112 roots of the form ±e_i ± e_j.
        for i in 0..8 {
            for j in (i + 1)..8 {
                for si in [-1.0f64, 1.0] {
                    for sj in [-1.0f64, 1.0] {
                        let mut root = [0.0f64; 8];
                        root[i] = si * inv_sqrt2;
                        root[j] = sj * inv_sqrt2;
                        list.push(root);
                    }
                }
            }
        }

        // 2^8 / 2 = 128 roots of the form (±1/2, ..., ±1/2) with an even number of minus signs.
        for bits in 0u32..256 {
            if bits.count_ones() % 2 != 0 {
                continue;
            }
            let mut root = [0.0f64; 8];
            for (dim, slot) in root.iter_mut().enumerate() {
                let sign = if bits & (1 << dim) != 0 { -1.0 } else { 1.0 };
                *slot = sign * 0.5 * inv_sqrt2;
            }
            list.push(root);
        }

        assert_eq!(list.len(), E8_ROOT_COUNT, "E8 root construction");
        let mut arr = [[0.0f64; 8]; E8_ROOT_COUNT];
        for (slot, root) in arr.iter_mut().zip(list.iter()) {
            *slot = *root;
        }
        arr
    })
}

/// `ceil(log2(k))` for `k >= 1`.
pub fn ceil_log2(k: usize) -> u32 {
    debug_assert!(k >= 1);
    (usize::BITS - (k - 1).leading_zeros()).max(1)
}

/// Per-event and per-query decode accounting.
///
/// The counts describe the *declared multiplier-free serving decode*, not the offline f64
/// reference used by this module. `multiplies` is zero for every family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DecodeWork {
    /// Table reads (product-table reads, codebook reads and LUT reads).
    pub table_reads: u64,
    /// Integer additions/subtractions.
    pub adds: u64,
    /// Integer multiply/divide instructions; declared zero.
    pub multiplies: u64,
}

impl DecodeWork {
    /// Sum two work profiles.
    pub const fn combine(self, other: Self) -> Self {
        Self {
            table_reads: self.table_reads + other.table_reads,
            adds: self.adds + other.adds,
            multiplies: self.multiplies + other.multiplies,
        }
    }
}

/// One quantized key: one code index per block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Codes {
    /// Code index for each block, in block order.
    pub blocks: Vec<u8>,
}

impl Codes {
    /// Compare two codes for exact equality.
    pub fn codes_equal(&self, other: &Self) -> bool {
        self.blocks == other.blocks
    }
}

/// A per-query table of block inner products against every code.
#[derive(Debug, Clone, PartialEq)]
pub struct Lut {
    /// Number of blocks.
    pub blocks: usize,
    /// Codes per block.
    pub codes_per_block: usize,
    /// Flattened `blocks * codes_per_block` inner products.
    pub entries: Vec<f64>,
}

/// The block layout of a k-means codebook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KMeansLayout {
    /// Number of blocks.
    pub blocks: usize,
    /// Dimension of each block.
    pub block_dim: usize,
    /// Centroids per block.
    pub codes_per_block: usize,
}

impl KMeansLayout {
    /// 120 centroids per 4-D block (matches the H4 layout).
    pub const fn h4_120() -> Self {
        Self {
            blocks: H4_LAYOUT.0,
            block_dim: H4_LAYOUT.1,
            codes_per_block: H4_CODE_COUNT,
        }
    }

    /// 240 centroids per 8-D block (matches the E8 layout).
    pub const fn e8_240() -> Self {
        Self {
            blocks: E8_LAYOUT.0,
            block_dim: E8_LAYOUT.1,
            codes_per_block: E8_ROOT_COUNT,
        }
    }
}

/// A fitted addressing codebook plus its assignment, scoring and accounting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressingArm {
    family: CodebookFamily,
    blocks: usize,
    block_dim: usize,
    codes_per_block: usize,
    index_bits: u32,
    seed: u64,
    pre_rotation: Option<RhtConfig>,
    /// Flattened `blocks * codes_per_block * block_dim` centroids (k-means only).
    centroids: Option<Vec<f64>>,
    /// Sign bits per block (sign only).
    sign_bits: Option<u32>,
    /// Flattened `blocks * sign_bits * block_dim` Rademacher projections (sign only).
    sign_projections: Option<Vec<i8>>,
}

impl AddressingArm {
    /// The 120-root H4 (600-cell) codebook: 16 x 4-D blocks, 7 stored bits per block.
    pub fn h4() -> Self {
        Self {
            family: CodebookFamily::H4,
            blocks: H4_LAYOUT.0,
            block_dim: H4_LAYOUT.1,
            codes_per_block: H4_CODE_COUNT,
            index_bits: ceil_log2(H4_CODE_COUNT),
            seed: 0,
            pre_rotation: None,
            centroids: None,
            sign_bits: None,
            sign_projections: None,
        }
    }

    /// The 240-root E8 codebook: 8 x 8-D blocks, 8 stored bits per block.
    pub fn e8() -> Self {
        Self {
            family: CodebookFamily::E8,
            blocks: E8_LAYOUT.0,
            block_dim: E8_LAYOUT.1,
            codes_per_block: E8_ROOT_COUNT,
            index_bits: ceil_log2(E8_ROOT_COUNT),
            seed: 0,
            pre_rotation: None,
            centroids: None,
            sign_bits: None,
            sign_projections: None,
        }
    }

    /// Random-projection sign codes on the H4 layout (128 codes, 7 bits/block).
    pub fn sign_codes_4d(seed: u64) -> Self {
        Self::sign(16, 4, ceil_log2(H4_CODE_COUNT), seed)
    }

    /// Random-projection sign codes on the E8 layout (256 codes, 8 bits/block).
    pub fn sign_codes_8d(seed: u64) -> Self {
        Self::sign(8, 8, ceil_log2(E8_ROOT_COUNT), seed)
    }

    /// The contest `h4` arm: 120 canonical H4 roots, no pre-rotation.
    pub fn contest_h4() -> Self {
        Self::h4()
    }

    /// The contest `h4-rht` arm.
    pub fn contest_h4_rht() -> Self {
        Self::h4().with_pre_rotation(CONTEST_RHT_SEED)
    }

    /// The contest `e8` arm: 240 normalized E8 roots, no pre-rotation.
    pub fn contest_e8() -> Self {
        Self::e8()
    }

    /// The contest `e8-rht` arm.
    pub fn contest_e8_rht() -> Self {
        Self::e8().with_pre_rotation(CONTEST_RHT_SEED)
    }

    /// The contest `sign4` arm (4-D sign codes at 7 bits/block).
    pub fn contest_sign4() -> Self {
        Self::sign_codes_4d(CONTEST_SIGN4_SEED)
    }

    /// The contest `sign4-rht` arm.
    pub fn contest_sign4_rht() -> Self {
        Self::sign_codes_4d(CONTEST_SIGN4_SEED).with_pre_rotation(CONTEST_RHT_SEED)
    }

    /// The contest `sign8` arm (8-D sign codes at 8 bits/block).
    pub fn contest_sign8() -> Self {
        Self::sign_codes_8d(CONTEST_SIGN8_SEED)
    }

    /// The contest `sign8-rht` arm.
    pub fn contest_sign8_rht() -> Self {
        Self::sign_codes_8d(CONTEST_SIGN8_SEED).with_pre_rotation(CONTEST_RHT_SEED)
    }

    fn sign(blocks: usize, block_dim: usize, sign_bits: u32, seed: u64) -> Self {
        let mut rng = SplitMix64::new(seed ^ 0x5A5A_1234_5A5A_1234);
        let projected = sign_bits as usize;
        let mut projections = Vec::with_capacity(blocks * projected * block_dim);
        for _ in 0..blocks * projected * block_dim {
            projections.push(if rng.next_u64() & 1 == 0 { 1i8 } else { -1i8 });
        }
        Self {
            family: CodebookFamily::Sign,
            blocks,
            block_dim,
            codes_per_block: 1usize << sign_bits,
            index_bits: sign_bits,
            seed,
            pre_rotation: None,
            centroids: None,
            sign_bits: Some(sign_bits),
            sign_projections: Some(projections),
        }
    }

    /// Enable the seeded randomized Hadamard pre-rotation on keys and queries.
    pub fn with_pre_rotation(mut self, seed: u64) -> Self {
        self.pre_rotation = Some(RhtConfig { seed });
        self
    }

    /// Fit a k-means codebook on tune keys with one seeded initialization.
    ///
    /// Initialization is seeded k-means++ (D^2 sampling); Lloyd iterations follow, with empty
    /// clusters reseeded to the farthest point. Identical keys and seed give identical centroids.
    pub fn fit_kmeans(keys: &[[f32; KEY_WIDTH]], layout: KMeansLayout, seed: u64) -> Result<Self> {
        if keys.is_empty() {
            return Err(AddressingError::EmptyTuneKeys);
        }
        if layout.codes_per_block == 0 {
            return Err(AddressingError::EmptyCodebook);
        }
        if layout.block_dim == 0
            || layout.blocks == 0
            || layout.blocks * layout.block_dim != KEY_WIDTH
        {
            return Err(AddressingError::InvalidLayout(format!(
                "blocks {} * block_dim {} must equal {KEY_WIDTH}",
                layout.blocks, layout.block_dim
            )));
        }

        let mut rng = SplitMix64::new(seed ^ 0x9E3779B97F4A7C15);
        let mut centroids = Vec::with_capacity(KEY_WIDTH * layout.codes_per_block);
        for block in 0..layout.blocks {
            let start = block * layout.block_dim;
            let points: Vec<Vec<f64>> = keys
                .iter()
                .map(|key| {
                    (0..layout.block_dim)
                        .map(|dim| key[start + dim] as f64)
                        .collect()
                })
                .collect();
            let fitted = fit_block_centroids(&points, layout.codes_per_block, &mut rng);
            for centroid in fitted {
                centroids.extend_from_slice(&centroid);
            }
        }

        let arm = Self {
            family: CodebookFamily::KMeans,
            blocks: layout.blocks,
            block_dim: layout.block_dim,
            codes_per_block: layout.codes_per_block,
            index_bits: ceil_log2(layout.codes_per_block),
            seed,
            pre_rotation: None,
            centroids: Some(centroids),
            sign_bits: None,
            sign_projections: None,
        };
        arm.validate()?;
        Ok(arm)
    }

    /// Fit the k-means codebook with each of [`DEFAULT_SEEDS`].
    pub fn fit_kmeans_three_seeds(
        keys: &[[f32; KEY_WIDTH]],
        layout: KMeansLayout,
    ) -> Result<Vec<Self>> {
        DEFAULT_SEEDS
            .iter()
            .map(|seed| Self::fit_kmeans(keys, layout, *seed))
            .collect()
    }

    /// Codebook family.
    pub fn family(&self) -> CodebookFamily {
        self.family
    }

    /// Number of blocks.
    pub fn blocks(&self) -> usize {
        self.blocks
    }

    /// Dimension of each block.
    pub fn block_dim(&self) -> usize {
        self.block_dim
    }

    /// Codes per block.
    pub fn codes_per_block(&self) -> usize {
        self.codes_per_block
    }

    /// Stored index bits per block (`ceil(log2 K)`).
    pub fn bits_per_block(&self) -> u32 {
        self.index_bits
    }

    /// Information bits per block (`log2 K`).
    pub fn information_bits_per_block(&self) -> f64 {
        (self.codes_per_block as f64).log2()
    }

    /// Total stored index bits per event.
    pub fn bits_per_event(&self) -> u64 {
        self.index_bits as u64 * self.blocks as u64
    }

    /// Index bytes per event, packed at the stored index width.
    pub fn bytes_per_event(&self) -> u64 {
        (self.bits_per_event() + 7) / 8
    }

    /// Whether the randomized Hadamard pre-rotation is enabled.
    pub fn pre_rotation_enabled(&self) -> bool {
        self.pre_rotation.is_some()
    }

    /// The fit seed recorded in the artifact.
    pub fn fit_seed(&self) -> u64 {
        self.seed
    }

    /// Statement of the declared multiplier-free decode for this family.
    pub fn decode_note(&self) -> &'static str {
        match self.family {
            CodebookFamily::H4 => {
                "H4 roots are fixed exact constants; a quantized block product is a \
                 quarter-square product-table read plus adds, so no multiplier is required."
            }
            CodebookFamily::E8 => {
                "E8 roots are fixed exact constants with coordinates in {0, ±1/√2, ±1/(2√2)}; \
                 quantized block products use a quarter-square product table, so no multiplier is \
                 required."
            }
            CodebookFamily::KMeans => {
                "Centroids are stored as at-most-4-bit integers; their runtime products use a \
                 quarter-square product table, so no multiplier is required."
            }
            CodebookFamily::Sign => {
                "The Rademacher ±1 projection z = P q is adds/subtracts only, and every code \
                 coordinate is ±1, so each table entry is a sum of ±z_j with no multiplier."
            }
        }
    }

    /// Declared decode work to score one encoded event against a prebuilt query LUT.
    pub fn decode_work_per_event(&self) -> DecodeWork {
        DecodeWork {
            table_reads: self.blocks as u64,
            adds: self.blocks as u64,
            multiplies: 0,
        }
    }

    /// Declared decode work to build the per-query LUT, multiplier-free.
    pub fn lut_build_work(&self) -> DecodeWork {
        let blocks = self.blocks as u64;
        let dim = self.block_dim as u64;
        match self.family {
            CodebookFamily::Sign => {
                let sign_bits = self.sign_bits.unwrap_or(0) as u64;
                let entries = self.codes_per_block as u64;
                DecodeWork {
                    table_reads: blocks * sign_bits * dim,
                    adds: blocks * (sign_bits * dim.saturating_sub(1) + entries * sign_bits),
                    multiplies: 0,
                }
            }
            _ => {
                let entries = self.codes_per_block as u64;
                DecodeWork {
                    table_reads: blocks * entries * dim,
                    adds: blocks * entries * dim.saturating_sub(1),
                    multiplies: 0,
                }
            }
        }
    }

    /// Encode a key to one code index per block.
    pub fn encode(&self, key: &[f32]) -> Result<Codes> {
        let rotated = self.rotate(key)?;
        let mut codes = Vec::with_capacity(self.blocks);
        for block in 0..self.blocks {
            let q = &rotated[block * self.block_dim..(block + 1) * self.block_dim];
            codes.push(self.assign_block(block, q)? as u8);
        }
        Ok(Codes { blocks: codes })
    }

    /// Build the per-query table of block inner products against every code.
    pub fn query_lut(&self, query: &[f32]) -> Result<Lut> {
        let rotated = self.rotate(query)?;
        let mut entries = vec![0.0f64; self.blocks * self.codes_per_block];
        for block in 0..self.blocks {
            let q = &rotated[block * self.block_dim..(block + 1) * self.block_dim];
            for code in 0..self.codes_per_block {
                entries[block * self.codes_per_block + code] =
                    self.block_inner_product(block, code, q)?;
            }
        }
        Ok(Lut {
            blocks: self.blocks,
            codes_per_block: self.codes_per_block,
            entries,
        })
    }

    /// PQ-ADC score: the sum of the query-LUT entries selected by the key's codes.
    pub fn score(&self, codes: &Codes, lut: &Lut) -> Result<f32> {
        if codes.blocks.len() != self.blocks {
            return Err(AddressingError::ShapeMismatch(format!(
                "codes has {} blocks, codebook has {}",
                codes.blocks.len(),
                self.blocks
            )));
        }
        if lut.blocks != self.blocks || lut.codes_per_block != self.codes_per_block {
            return Err(AddressingError::ShapeMismatch(format!(
                "lut is {}x{}, codebook is {}x{}",
                lut.blocks, lut.codes_per_block, self.blocks, self.codes_per_block
            )));
        }
        if lut.entries.len() != self.blocks * self.codes_per_block {
            return Err(AddressingError::ShapeMismatch(
                "lut entry count does not match its declared shape".to_string(),
            ));
        }
        let mut acc = 0.0f64;
        for block in 0..self.blocks {
            let code = codes.blocks[block] as usize;
            if code >= self.codes_per_block {
                return Err(AddressingError::CodeOutOfRange {
                    code,
                    codes_per_block: self.codes_per_block,
                });
            }
            acc += lut.entries[block * self.codes_per_block + code];
        }
        Ok(acc as f32)
    }

    /// Compare two codes (block count and indices).
    pub fn codes_equal(&self, a: &Codes, b: &Codes) -> bool {
        a.blocks.len() == self.blocks && a.codes_equal(b)
    }

    /// The decoded block vector for a code, used by the explicit-inner-product reference.
    pub fn code_coordinates(&self, block: usize, code: usize) -> Result<Vec<f64>> {
        if code >= self.codes_per_block {
            return Err(AddressingError::CodeOutOfRange {
                code,
                codes_per_block: self.codes_per_block,
            });
        }
        if block >= self.blocks {
            return Err(AddressingError::ShapeMismatch(format!(
                "block {block} outside {} blocks",
                self.blocks
            )));
        }
        match self.family {
            CodebookFamily::H4 => {
                let root = canonical_h4_roots()
                    .get(code)
                    .ok_or(AddressingError::NotFitted("H4 roots"))?;
                Ok(root.to_array().to_vec())
            }
            CodebookFamily::E8 => {
                let root = e8_roots()
                    .get(code)
                    .ok_or(AddressingError::NotFitted("E8 roots"))?;
                Ok(root.to_vec())
            }
            CodebookFamily::KMeans => self
                .centroid(block, code)
                .map(|slice| slice.to_vec())
                .ok_or(AddressingError::NotFitted("k-means centroids")),
            CodebookFamily::Sign => {
                let projection = self
                    .projection(block)
                    .ok_or(AddressingError::NotFitted("sign projections"))?;
                let sign_bits = self
                    .sign_bits
                    .ok_or(AddressingError::NotFitted("sign bits"))?
                    as usize;
                let dim = self.block_dim;
                let mut out = vec![0.0f64; dim];
                for bit in 0..sign_bits {
                    let sign = if (code >> bit) & 1 == 1 { -1.0 } else { 1.0 };
                    let row = &projection[bit * dim..(bit + 1) * dim];
                    for dim_idx in 0..dim {
                        out[dim_idx] += sign * row[dim_idx] as f64;
                    }
                }
                Ok(out)
            }
        }
    }

    /// Serialize the fitted arm to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(AddressingError::from)
    }

    /// Parse a fitted arm from JSON, validating its shape.
    pub fn from_json(json: &str) -> Result<Self> {
        let arm: Self = serde_json::from_str(json)?;
        arm.validate()?;
        Ok(arm)
    }

    /// Write the fitted arm to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load and validate a fitted arm from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    fn validate(&self) -> Result<()> {
        if self.blocks == 0 || self.block_dim == 0 || self.codes_per_block == 0 {
            return Err(AddressingError::InvalidLayout(
                "blocks, block_dim and codes_per_block must be positive".to_string(),
            ));
        }
        if self.blocks * self.block_dim != KEY_WIDTH {
            return Err(AddressingError::InvalidLayout(format!(
                "blocks {} * block_dim {} must equal {KEY_WIDTH}",
                self.blocks, self.block_dim
            )));
        }
        if self.index_bits != ceil_log2(self.codes_per_block) {
            return Err(AddressingError::InvalidLayout(format!(
                "index_bits {} does not match ceil(log2 {})",
                self.index_bits, self.codes_per_block
            )));
        }
        match self.family {
            CodebookFamily::H4 => {
                if (self.blocks, self.block_dim, self.codes_per_block)
                    != (H4_LAYOUT.0, H4_LAYOUT.1, H4_CODE_COUNT)
                {
                    return Err(AddressingError::InvalidLayout(
                        "H4 must be 16 x 4-D with 120 codes".to_string(),
                    ));
                }
            }
            CodebookFamily::E8 => {
                if (self.blocks, self.block_dim, self.codes_per_block)
                    != (E8_LAYOUT.0, E8_LAYOUT.1, E8_ROOT_COUNT)
                {
                    return Err(AddressingError::InvalidLayout(
                        "E8 must be 8 x 8-D with 240 codes".to_string(),
                    ));
                }
            }
            CodebookFamily::KMeans => {
                let expected = self.blocks * self.codes_per_block * self.block_dim;
                match &self.centroids {
                    Some(centroids) if centroids.len() == expected => {}
                    Some(centroids) => {
                        return Err(AddressingError::ShapeMismatch(format!(
                            "k-means centroids has {} values, expected {expected}",
                            centroids.len()
                        )))
                    }
                    None => return Err(AddressingError::NotFitted("k-means centroids")),
                }
            }
            CodebookFamily::Sign => {
                let sign_bits = self
                    .sign_bits
                    .ok_or(AddressingError::NotFitted("sign bits"))?;
                if self.codes_per_block != 1usize << sign_bits {
                    return Err(AddressingError::InvalidLayout(
                        "sign codes_per_block must be 2^sign_bits".to_string(),
                    ));
                }
                let expected = self.blocks * sign_bits as usize * self.block_dim;
                match &self.sign_projections {
                    Some(projections) if projections.len() == expected => {}
                    Some(projections) => {
                        return Err(AddressingError::ShapeMismatch(format!(
                            "sign projections has {} values, expected {expected}",
                            projections.len()
                        )))
                    }
                    None => return Err(AddressingError::NotFitted("sign projections")),
                }
            }
        }
        Ok(())
    }

    fn rotate(&self, input: &[f32]) -> Result<[f64; KEY_WIDTH]> {
        if input.len() != KEY_WIDTH {
            return Err(AddressingError::KeyWidth {
                expected: KEY_WIDTH,
                got: input.len(),
            });
        }
        let mut out = [0.0f64; KEY_WIDTH];
        for (slot, value) in out.iter_mut().zip(input.iter()) {
            *slot = *value as f64;
        }
        if let Some(rht) = self.pre_rotation {
            randomized_hadamard(&mut out, rht.seed);
        }
        Ok(out)
    }

    fn centroid(&self, block: usize, code: usize) -> Option<&[f64]> {
        let centroids = self.centroids.as_ref()?;
        let dim = self.block_dim;
        let start = (block * self.codes_per_block + code) * dim;
        centroids.get(start..start + dim)
    }

    fn projection(&self, block: usize) -> Option<&[i8]> {
        let projections = self.sign_projections.as_ref()?;
        let sign_bits = self.sign_bits? as usize;
        let dim = self.block_dim;
        let start = block * sign_bits * dim;
        projections.get(start..start + sign_bits * dim)
    }

    fn assign_block(&self, block: usize, q: &[f64]) -> Result<usize> {
        match self.family {
            CodebookFamily::H4 => {
                let roots = canonical_h4_roots();
                let mut best = 0usize;
                let mut best_dot = f64::NEG_INFINITY;
                for (index, root) in roots.iter().enumerate() {
                    let dot = dot_slices(q, &root.to_array());
                    if dot > best_dot {
                        best_dot = dot;
                        best = index;
                    }
                }
                Ok(best)
            }
            CodebookFamily::E8 => {
                let roots = e8_roots();
                let mut best = 0usize;
                let mut best_dot = f64::NEG_INFINITY;
                for (index, root) in roots.iter().enumerate() {
                    let dot = dot_slices(q, root);
                    if dot > best_dot {
                        best_dot = dot;
                        best = index;
                    }
                }
                Ok(best)
            }
            CodebookFamily::KMeans => {
                let mut best = 0usize;
                let mut best_dist = f64::INFINITY;
                for code in 0..self.codes_per_block {
                    let centroid = self
                        .centroid(block, code)
                        .ok_or(AddressingError::NotFitted("k-means centroids"))?;
                    let dist = squared_distance(q, centroid);
                    if dist < best_dist {
                        best_dist = dist;
                        best = code;
                    }
                }
                Ok(best)
            }
            CodebookFamily::Sign => {
                let projection = self
                    .projection(block)
                    .ok_or(AddressingError::NotFitted("sign projections"))?;
                let sign_bits = self
                    .sign_bits
                    .ok_or(AddressingError::NotFitted("sign bits"))?
                    as usize;
                let dim = self.block_dim;
                let mut index = 0usize;
                for bit in 0..sign_bits {
                    let row = &projection[bit * dim..(bit + 1) * dim];
                    let mut z = 0.0f64;
                    for dim_idx in 0..dim {
                        z += row[dim_idx] as f64 * q[dim_idx];
                    }
                    if z < 0.0 {
                        index |= 1 << bit;
                    }
                }
                Ok(index)
            }
        }
    }

    fn block_inner_product(&self, block: usize, code: usize, q: &[f64]) -> Result<f64> {
        match self.family {
            CodebookFamily::H4 => {
                let root =
                    canonical_h4_roots()
                        .get(code)
                        .ok_or(AddressingError::CodeOutOfRange {
                            code,
                            codes_per_block: self.codes_per_block,
                        })?;
                Ok(dot_slices(q, &root.to_array()))
            }
            CodebookFamily::E8 => {
                let root = e8_roots()
                    .get(code)
                    .ok_or(AddressingError::CodeOutOfRange {
                        code,
                        codes_per_block: self.codes_per_block,
                    })?;
                Ok(dot_slices(q, root))
            }
            CodebookFamily::KMeans => {
                let centroid = self
                    .centroid(block, code)
                    .ok_or(AddressingError::NotFitted("k-means centroids"))?;
                Ok(dot_slices(q, centroid))
            }
            CodebookFamily::Sign => {
                let projection = self
                    .projection(block)
                    .ok_or(AddressingError::NotFitted("sign projections"))?;
                let sign_bits = self
                    .sign_bits
                    .ok_or(AddressingError::NotFitted("sign bits"))?
                    as usize;
                let dim = self.block_dim;
                let mut acc = 0.0f64;
                for bit in 0..sign_bits {
                    let row = &projection[bit * dim..(bit + 1) * dim];
                    let mut z = 0.0f64;
                    for dim_idx in 0..dim {
                        z += row[dim_idx] as f64 * q[dim_idx];
                    }
                    let sign = if (code >> bit) & 1 == 1 { -1.0 } else { 1.0 };
                    acc += sign * z;
                }
                Ok(acc)
            }
        }
    }
}

/// Seeded deterministic PRNG (SplitMix64), avoiding a new dependency.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    fn next_index(&mut self, upper: usize) -> usize {
        if upper == 0 {
            return 0;
        }
        (self.next_u64() % upper as u64) as usize
    }
}

fn dot_slices(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn squared_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum()
}

/// Seeded randomized Hadamard transform over the full key width.
///
/// Applies the random `±1` diagonal, then the unnormalized Walsh-Hadamard butterfly (adds and
/// subtractions only), then the `1/sqrt(n)` scale (a right shift by `log2(n)/2 = 3` at serving).
fn randomized_hadamard(x: &mut [f64; KEY_WIDTH], seed: u64) {
    let mut rng = SplitMix64::new(seed ^ 0xA5A5_5A5A_1234_5678);
    for value in x.iter_mut() {
        if rng.next_u64() & 1 == 1 {
            *value = -*value;
        }
    }

    let mut half = 1;
    while half < KEY_WIDTH {
        let mut start = 0;
        while start < KEY_WIDTH {
            for index in start..start + half {
                let a = x[index];
                let b = x[index + half];
                x[index] = a + b;
                x[index + half] = a - b;
            }
            start += half * 2;
        }
        half *= 2;
    }

    const SHIFT_SCALE: f64 = 1.0 / 8.0;
    for value in x.iter_mut() {
        *value *= SHIFT_SCALE;
    }
}

/// K-means++ initialization plus Lloyd iterations for one block.
///
/// `points` is non-empty with equal dimension. Initialization picks the first centroid
/// uniformly, then each next centroid with probability proportional to its squared distance to
/// the chosen set (D^2 sampling); if every point is already a centroid, a seeded replacement is
/// chosen. Lloyd reassigns, recomputes means and reseeds empty clusters to the farthest point.
fn fit_block_centroids(points: &[Vec<f64>], k: usize, rng: &mut SplitMix64) -> Vec<Vec<f64>> {
    let n = points.len();
    let dim = points[0].len();
    let k = k.max(1);

    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    let first = rng.next_index(n);
    centroids.push(points[first].clone());
    let mut min_dist2 = vec![f64::INFINITY; n];

    while centroids.len() < k {
        let last = centroids.len() - 1;
        for index in 0..n {
            let dist = squared_distance(&points[index], &centroids[last]);
            if dist < min_dist2[index] {
                min_dist2[index] = dist;
            }
        }
        let total: f64 = min_dist2.iter().sum();
        let next = if total <= f64::MIN_POSITIVE {
            rng.next_index(n)
        } else {
            let mut threshold = rng.next_f64() * total;
            let mut chosen = n - 1;
            for index in 0..n {
                threshold -= min_dist2[index];
                if threshold <= 0.0 {
                    chosen = index;
                    break;
                }
            }
            chosen
        };
        centroids.push(points[next].clone());
    }

    let mut assignment = vec![0usize; n];
    for _ in 0..KMEANS_MAX_ITERS {
        let mut changed = false;
        for index in 0..n {
            let mut best = 0usize;
            let mut best_dist = f64::INFINITY;
            for code in 0..k {
                let dist = squared_distance(&points[index], &centroids[code]);
                if dist < best_dist {
                    best_dist = dist;
                    best = code;
                }
            }
            if assignment[index] != best {
                assignment[index] = best;
                changed = true;
            }
        }

        let mut sums = vec![vec![0.0f64; dim]; k];
        let mut counts = vec![0usize; k];
        for index in 0..n {
            let code = assignment[index];
            counts[code] += 1;
            for dim_idx in 0..dim {
                sums[code][dim_idx] += points[index][dim_idx];
            }
        }

        let mut max_shift = 0.0f64;
        for code in 0..k {
            let next = if counts[code] == 0 {
                let mut farthest = 0usize;
                let mut farthest_dist = -1.0f64;
                for index in 0..n {
                    let dist = squared_distance(&points[index], &centroids[code]);
                    if dist > farthest_dist {
                        farthest_dist = dist;
                        farthest = index;
                    }
                }
                points[farthest].clone()
            } else {
                let inv = 1.0 / counts[code] as f64;
                let mut mean = vec![0.0f64; dim];
                for dim_idx in 0..dim {
                    mean[dim_idx] = sums[code][dim_idx] * inv;
                }
                mean
            };
            let shift = squared_distance(&centroids[code], &next).sqrt();
            if shift > max_shift {
                max_shift = shift;
            }
            centroids[code] = next;
        }

        if !changed && max_shift < KMEANS_TOLERANCE {
            break;
        }
    }

    centroids
}

/// Number of 4-D lanes in a read query/key for the D6 information audit.
pub const D6_LANES: usize = H4_LAYOUT.0;
/// Dimension of one D6 lane.
pub const D6_LANE_DIM: usize = H4_LAYOUT.1;
/// Centroids per lane for the D6 ordinary equal-bit control (10 stored bits).
pub const D6_LANE_CENTROIDS: usize = 1024;
/// Gain levels for the D6 3-bit dyadic gain channel.
pub const D6_GAIN_LEVELS: u32 = 8;

fn lane_squared_distance(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let mut total = 0.0;
    for dim in 0..4 {
        let delta = a[dim] - b[dim];
        total += delta * delta;
    }
    total
}

/// Fit one 1,024-centroid k-means codebook on 4-D lane vectors.
///
/// Seeded k-means++ initialization (D^2 sampling), then Lloyd reassignment and
/// mean recomputation with empty-cluster reseeding to the farthest point, using
/// the module's iteration cap and convergence tolerance. `lanes` is row-major;
/// identical lanes and seed give identical centroids. This is the D6 ordinary
/// control's single declared fit: no codebook-size sweep and no seed sweep.
pub fn fit_lane_kmeans_1024(lanes: &[[f64; 4]], seed: u64) -> Result<Vec<[f64; 4]>> {
    if lanes.is_empty() {
        return Err(AddressingError::EmptyTuneKeys);
    }
    let count = lanes.len();
    let k = D6_LANE_CENTROIDS;
    let mut rng = SplitMix64::new(seed ^ 0x9E37_79B9_7F4A_7C15);

    let mut centroids: Vec<[f64; 4]> = Vec::with_capacity(k);
    centroids.push(lanes[rng.next_index(count)]);
    let mut min_dist2 = vec![f64::INFINITY; count];
    while centroids.len() < k {
        let last = centroids[centroids.len() - 1];
        for index in 0..count {
            let distance = lane_squared_distance(&lanes[index], &last);
            if distance < min_dist2[index] {
                min_dist2[index] = distance;
            }
        }
        let total: f64 = min_dist2.iter().sum();
        let next = if total <= f64::MIN_POSITIVE {
            rng.next_index(count)
        } else {
            let mut threshold = rng.next_f64() * total;
            let mut chosen = count - 1;
            for index in 0..count {
                threshold -= min_dist2[index];
                if threshold <= 0.0 {
                    chosen = index;
                    break;
                }
            }
            chosen
        };
        centroids.push(lanes[next]);
    }

    let mut assignment = vec![0u32; count];
    for _ in 0..KMEANS_MAX_ITERS {
        let mut changed = false;
        for index in 0..count {
            let point = lanes[index];
            let mut best = 0usize;
            let mut best_distance = f64::INFINITY;
            for code in 0..k {
                let distance = lane_squared_distance(&point, &centroids[code]);
                if distance < best_distance {
                    best_distance = distance;
                    best = code;
                }
            }
            if assignment[index] as usize != best {
                assignment[index] = best as u32;
                changed = true;
            }
        }

        let mut sums = vec![[0.0f64; 4]; k];
        let mut counts = vec![0u32; k];
        for index in 0..count {
            let code = assignment[index] as usize;
            counts[code] += 1;
            for dim in 0..4 {
                sums[code][dim] += lanes[index][dim];
            }
        }

        let mut max_shift = 0.0f64;
        for code in 0..k {
            let next = if counts[code] == 0 {
                let mut farthest = 0usize;
                let mut farthest_distance = -1.0f64;
                for index in 0..count {
                    let distance = lane_squared_distance(&lanes[index], &centroids[code]);
                    if distance > farthest_distance {
                        farthest_distance = distance;
                        farthest = index;
                    }
                }
                lanes[farthest]
            } else {
                let inverse = 1.0 / f64::from(counts[code]);
                [
                    sums[code][0] * inverse,
                    sums[code][1] * inverse,
                    sums[code][2] * inverse,
                    sums[code][3] * inverse,
                ]
            };
            let shift = lane_squared_distance(&centroids[code], &next).sqrt();
            if shift > max_shift {
                max_shift = shift;
            }
            centroids[code] = next;
        }

        if !changed && max_shift < KMEANS_TOLERANCE {
            break;
        }
    }
    Ok(centroids)
}

/// Nearest centroid of `lane` under squared L2, with the decoded centroid.
pub fn nearest_lane_centroid(centroids: &[[f64; 4]], lane: &[f64; 4]) -> (u16, [f64; 4]) {
    let mut best = 0usize;
    let mut best_distance = f64::INFINITY;
    for (index, centroid) in centroids.iter().enumerate() {
        let distance = lane_squared_distance(lane, centroid);
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    (best as u16, centroids[best])
}

/// Quantize a lane L2 norm to a 3-bit dyadic gain `2^(min_exponent + code)`.
///
/// The exponent is `round(log2(norm))` clamped into the eight consecutive
/// exponents `min_exponent..=min_exponent + 7`; the third value is `-1` when the
/// rounded exponent clipped low, `1` when it clipped high and `0` when it fell
/// inside the declared range. A non-positive or non-finite norm maps to the
/// lowest code and reports low clipping.
pub fn quantize_dyadic_gain(norm: f64, min_exponent: i32) -> (u8, f64, i8) {
    let max_exponent = min_exponent + D6_GAIN_LEVELS as i32 - 1;
    if !(norm > 0.0) || !norm.is_finite() {
        return (0, 2f64.powi(min_exponent), -1);
    }
    let raw = norm.log2().round() as i32;
    let clipped = raw.clamp(min_exponent, max_exponent);
    let flag = if raw < min_exponent {
        -1
    } else if raw > max_exponent {
        1
    } else {
        0
    };
    ((clipped - min_exponent) as u8, 2f64.powi(clipped), flag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn synthetic_key(seed: u64) -> [f32; KEY_WIDTH] {
        let mut rng = SplitMix64::new(seed ^ 0xDEAD_BEEF_CAFE_F00D);
        let mut key = [0.0f32; KEY_WIDTH];
        for value in key.iter_mut() {
            *value = (rng.next_f64() * 2.0 - 1.0) as f32;
        }
        key
    }

    fn synthetic_keys(count: usize, seed: u64) -> Vec<[f32; KEY_WIDTH]> {
        let mut rng = SplitMix64::new(seed);
        (0..count)
            .map(|_| {
                let mut key = [0.0f32; KEY_WIDTH];
                for value in key.iter_mut() {
                    *value = (rng.next_f64() * 2.0 - 1.0) as f32;
                }
                key
            })
            .collect()
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "uor-r4-addressing-{}-{counter}-{name}",
            std::process::id()
        ));
        path
    }

    #[test]
    fn addressing_h4_codebook_has_120_roots() {
        let arm = AddressingArm::h4();
        assert_eq!(arm.family(), CodebookFamily::H4);
        assert_eq!(arm.blocks(), 16);
        assert_eq!(arm.block_dim(), 4);
        assert_eq!(arm.codes_per_block(), 120);
        assert_eq!(arm.bits_per_block(), 7);
        assert!((arm.information_bits_per_block() - 6.906_890_7).abs() < 1e-4);

        let canonical = canonical_h4_roots();
        assert_eq!(canonical.len(), 120);
        let mut distinct = std::collections::BTreeSet::new();
        for code in 0..120 {
            let decoded = arm.code_coordinates(0, code).expect("H4 coordinates");
            assert_eq!(decoded, canonical[code].to_array().to_vec());
            distinct.insert(decoded.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
        }
        assert_eq!(distinct.len(), 120);
    }

    #[test]
    fn addressing_e8_codebook_has_240_roots() {
        let arm = AddressingArm::e8();
        assert_eq!(arm.family(), CodebookFamily::E8);
        assert_eq!(arm.blocks(), 8);
        assert_eq!(arm.block_dim(), 8);
        assert_eq!(arm.codes_per_block(), 240);
        assert_eq!(arm.bits_per_block(), 8);
        assert!((arm.information_bits_per_block() - 7.906_890_7).abs() < 1e-4);

        let roots = e8_roots();
        assert_eq!(roots.len(), 240);
        let mut pair_count = 0usize;
        let mut half_count = 0usize;
        for root in roots {
            let norm_sq: f64 = root.iter().map(|value| value * value).sum();
            assert!((norm_sq - 1.0).abs() < 1e-12, "E8 root must be unit");
            let nonzero = root.iter().filter(|value| value.abs() > 1e-9).count();
            match nonzero {
                2 => pair_count += 1,
                8 => half_count += 1,
                other => panic!("unexpected E8 support size {other}"),
            }
        }
        assert_eq!(pair_count, 112);
        assert_eq!(half_count, 128);
        assert_eq!(
            arm.code_coordinates(0, 0).expect("E8 coordinates"),
            roots[0].to_vec()
        );
    }

    #[test]
    fn addressing_pq_adc_matches_explicit_inner_product() {
        for arm in [
            AddressingArm::h4(),
            AddressingArm::e8(),
            AddressingArm::sign_codes_4d(0x1234),
            AddressingArm::sign_codes_8d(0x5678),
            AddressingArm::h4().with_pre_rotation(0x9ABC),
        ] {
            let key = synthetic_key(3);
            let query = synthetic_key(5);
            let codes = arm.encode(&key).expect("encode");
            let lut = arm.query_lut(&query).expect("query_lut");
            let adc = arm.score(&codes, &lut).expect("score") as f64;

            let rotated_query = arm.rotate(&query).expect("rotate");
            let mut explicit = 0.0f64;
            for block in 0..arm.blocks() {
                let q = &rotated_query[block * arm.block_dim()..(block + 1) * arm.block_dim()];
                let decoded = arm
                    .code_coordinates(block, codes.blocks[block] as usize)
                    .expect("decoded code");
                explicit += dot_slices(q, &decoded);
            }

            assert!(
                (adc - explicit).abs() < 1e-4,
                "{:?} PQ-ADC {adc} vs explicit {explicit}",
                arm.family()
            );
        }
    }

    #[test]
    fn addressing_sign_codes_equal_bits() {
        let arm_4d = AddressingArm::sign_codes_4d(11);
        assert_eq!(arm_4d.bits_per_block(), ceil_log2(H4_CODE_COUNT));
        assert_eq!(arm_4d.bits_per_block(), 7);
        assert_eq!(arm_4d.codes_per_block(), 1 << 7);
        assert_eq!(arm_4d.bytes_per_event(), 14);

        let arm_8d = AddressingArm::sign_codes_8d(22);
        assert_eq!(arm_8d.bits_per_block(), ceil_log2(E8_ROOT_COUNT));
        assert_eq!(arm_8d.bits_per_block(), 8);
        assert_eq!(arm_8d.codes_per_block(), 1 << 8);
        assert_eq!(arm_8d.bytes_per_event(), 8);
    }

    #[test]
    fn addressing_work_counts_are_declared_and_positive() {
        let keys = synthetic_keys(160, 0xBEEF);
        let kmeans = AddressingArm::fit_kmeans(&keys, KMeansLayout::h4_120(), DEFAULT_SEEDS[0])
            .expect("fit kmeans");
        let arms = [
            AddressingArm::h4(),
            AddressingArm::e8(),
            AddressingArm::sign_codes_4d(1),
            AddressingArm::sign_codes_8d(2),
            kmeans,
        ];

        for arm in &arms {
            let per_event = arm.decode_work_per_event();
            let lut = arm.lut_build_work();
            assert_eq!(
                per_event.multiplies,
                0,
                "{} decode must be multiplier-free",
                arm.family()
            );
            assert_eq!(
                lut.multiplies,
                0,
                "{} LUT must be multiplier-free",
                arm.family()
            );
            assert!(per_event.table_reads > 0 && per_event.adds > 0);
            assert!(lut.table_reads > 0 && lut.adds > 0);
            assert_eq!(per_event.table_reads, arm.blocks() as u64);
            assert_eq!(per_event.adds, arm.blocks() as u64);
            assert!(!arm.decode_note().is_empty());
        }

        assert_eq!(
            AddressingArm::h4().decode_work_per_event(),
            DecodeWork {
                table_reads: 16,
                adds: 16,
                multiplies: 0
            }
        );
        assert_eq!(
            AddressingArm::h4().lut_build_work(),
            DecodeWork {
                table_reads: 16 * 120 * 4,
                adds: 16 * 120 * 3,
                multiplies: 0
            }
        );
        assert_eq!(
            AddressingArm::e8().decode_work_per_event(),
            DecodeWork {
                table_reads: 8,
                adds: 8,
                multiplies: 0
            }
        );
        assert_eq!(
            AddressingArm::e8().lut_build_work(),
            DecodeWork {
                table_reads: 8 * 240 * 8,
                adds: 8 * 240 * 7,
                multiplies: 0
            }
        );
        assert_eq!(
            AddressingArm::sign_codes_4d(1).lut_build_work(),
            DecodeWork {
                table_reads: 16 * 7 * 4,
                adds: 16 * (7 * 3 + 128 * 7),
                multiplies: 0
            }
        );
        assert_eq!(
            AddressingArm::sign_codes_8d(2).lut_build_work(),
            DecodeWork {
                table_reads: 8 * 8 * 8,
                adds: 8 * (8 * 7 + 256 * 8),
                multiplies: 0
            }
        );
    }

    #[test]
    fn addressing_roundtrip_serde() {
        let keys = synthetic_keys(160, 0xC0FFEE);
        let arm = AddressingArm::fit_kmeans(&keys, KMeansLayout::h4_120(), DEFAULT_SEEDS[0])
            .expect("fit");
        let from_json = AddressingArm::from_json(&arm.to_json().expect("to_json")).expect("parse");

        let path = unique_temp_path("arm-roundtrip.json");
        arm.save(&path).expect("save");
        let from_file = AddressingArm::load(&path).expect("load");
        std::fs::remove_file(&path).ok();

        for seed in [13u64, 37, 61] {
            let key = synthetic_key(seed);
            let query = synthetic_key(seed + 1);
            let codes = arm.encode(&key).expect("encode");
            let lut = arm.query_lut(&query).expect("lut");
            let score = arm.score(&codes, &lut).expect("score");
            for other in [&from_json, &from_file] {
                let other_codes = other.encode(&key).expect("encode");
                assert!(arm.codes_equal(&codes, &other_codes));
                let other_lut = other.query_lut(&query).expect("lut");
                let other_score = other.score(&other_codes, &other_lut).expect("score");
                assert!((score - other_score).abs() < 1e-6);
            }
        }

        for arm in [
            AddressingArm::h4(),
            AddressingArm::e8(),
            AddressingArm::sign_codes_4d(5),
            AddressingArm::sign_codes_8d(6),
        ] {
            let other = AddressingArm::from_json(&arm.to_json().expect("json")).expect("parse");
            let key = synthetic_key(101);
            let query = synthetic_key(102);
            let codes = arm.encode(&key).expect("encode");
            let other_codes = other.encode(&key).expect("encode");
            assert!(arm.codes_equal(&codes, &other_codes));
            let score = arm
                .score(&codes, &arm.query_lut(&query).expect("lut"))
                .expect("score");
            let other_score = other
                .score(&other_codes, &other.query_lut(&query).expect("lut"))
                .expect("score");
            assert!((score - other_score).abs() < 1e-9);
        }
    }

    #[test]
    fn addressing_pre_rotation_preserves_norm_and_is_deterministic() {
        let key = synthetic_key(77);
        let mut a = [0.0f64; KEY_WIDTH];
        let mut b = [0.0f64; KEY_WIDTH];
        for index in 0..KEY_WIDTH {
            a[index] = key[index] as f64;
            b[index] = key[index] as f64;
        }
        let norm_before: f64 = a.iter().map(|value| value * value).sum();
        randomized_hadamard(&mut a, 4242);
        randomized_hadamard(&mut b, 4242);
        assert_eq!(a, b);
        let norm_after: f64 = a.iter().map(|value| value * value).sum();
        assert!((norm_before - norm_after).abs() < 1e-6);

        let arm = AddressingArm::h4().with_pre_rotation(4242);
        assert!(arm.pre_rotation_enabled());
        let codes = arm.encode(&key).expect("encode");
        let score = arm
            .score(&codes, &arm.query_lut(&synthetic_key(78)).expect("lut"))
            .expect("score");
        assert!(score.is_finite());
    }

    #[test]
    fn contest_arms_reproduce_their_declared_seeds() {
        assert_eq!(CONTEST_RHT_SEED, 0x51D5_11A7_0000_0001);
        assert_eq!(CONTEST_SIGN4_SEED, 0x51D5_0004_0000_0004);
        assert_eq!(CONTEST_SIGN8_SEED, 0x51D5_0008_0000_0008);

        for arm in [
            AddressingArm::contest_h4(),
            AddressingArm::contest_e8(),
            AddressingArm::contest_sign4(),
            AddressingArm::contest_sign8(),
        ] {
            assert!(!arm.pre_rotation_enabled(), "{:?}", arm.family());
        }
        for arm in [
            AddressingArm::contest_h4_rht(),
            AddressingArm::contest_e8_rht(),
            AddressingArm::contest_sign4_rht(),
            AddressingArm::contest_sign8_rht(),
        ] {
            assert!(arm.pre_rotation_enabled(), "{:?}", arm.family());
        }
        assert_eq!(
            AddressingArm::contest_sign4().fit_seed(),
            CONTEST_SIGN4_SEED
        );
        assert_eq!(
            AddressingArm::contest_sign8().fit_seed(),
            CONTEST_SIGN8_SEED
        );
    }

    #[test]
    fn d6_dyadic_gain_is_power_of_two_and_clips() {
        let min_exponent = -4;
        let (code, gain, flag) = quantize_dyadic_gain(1.0, min_exponent);
        assert_eq!((code, flag), (4, 0));
        assert_eq!(gain, 1.0);

        let (code_low, gain_low, flag_low) = quantize_dyadic_gain(1e-6, min_exponent);
        assert_eq!((code_low, flag_low), (0, -1));
        assert_eq!(gain_low, 2f64.powi(min_exponent));

        let (code_high, gain_high, flag_high) = quantize_dyadic_gain(1e6, min_exponent);
        assert_eq!((code_high, flag_high), (7, 1));
        assert_eq!(gain_high, 2f64.powi(3));

        let (code_zero, _, flag_zero) = quantize_dyadic_gain(0.0, min_exponent);
        assert_eq!((code_zero, flag_zero), (0, -1));

        for code in 0..D6_GAIN_LEVELS {
            let exponent = min_exponent + code as i32;
            let (decoded_code, decoded_gain, flag) =
                quantize_dyadic_gain(2f64.powi(exponent), min_exponent);
            assert_eq!(decoded_code as u32, code);
            assert_eq!(flag, 0);
            assert!((decoded_gain.log2() - f64::from(exponent)).abs() < 1e-12);
        }
    }

    #[test]
    fn d6_nearest_lane_centroid_selects_the_closest() {
        let centroids = vec![
            [0.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
        ];
        let (code, decoded) = nearest_lane_centroid(&centroids, &[0.9, 0.0, 0.0, 0.0]);
        assert_eq!(code, 1);
        assert_eq!(decoded, [1.0, 0.0, 0.0, 0.0]);
        let (code, _) = nearest_lane_centroid(&centroids, &[0.0, 1.8, 0.0, 0.0]);
        assert_eq!(code, 2);
    }

    #[test]
    fn d6_lane_kmeans_is_deterministic_and_clusters_are_recovered() {
        let mut lanes: Vec<[f64; 4]> = Vec::with_capacity(512);
        for index in 0..256 {
            let jitter = (index as f64) * 1e-4;
            lanes.push([jitter, jitter, jitter, jitter]);
            lanes.push([1.0 + jitter, -1.0 - jitter, 0.5 + jitter, 0.0]);
        }
        let first = fit_lane_kmeans_1024(&lanes, 0xD6D6).expect("fit");
        let second = fit_lane_kmeans_1024(&lanes, 0xD6D6).expect("refit");
        assert_eq!(first.len(), D6_LANE_CENTROIDS);
        assert_eq!(first, second, "identical lanes and seed must be identical");

        let (_, near_a) = nearest_lane_centroid(&first, &[0.0, 0.0, 0.0, 0.0]);
        let (_, near_b) = nearest_lane_centroid(&first, &[1.0, -1.0, 0.5, 0.0]);
        assert!(lane_squared_distance(&near_a, &[0.0, 0.0, 0.0, 0.0]) < 0.05);
        assert!(lane_squared_distance(&near_b, &[1.0, -1.0, 0.5, 0.0]) < 0.05);
    }
}
