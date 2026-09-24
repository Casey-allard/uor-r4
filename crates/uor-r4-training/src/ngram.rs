//! A bounded, offline count-pruned interpolated Kneser–Ney comparator.
//!
//! This is a floating-point statistical baseline, not native geometric serving.
//! The highest order uses raw five-gram counts. Each lower order uses the number
//! of distinct left extensions in the **unpruned raw** (n + 1)-gram types:
//! `c_n(h,w) = N_1+(.,h,w)`. This is the continuation construction described by
//! Kneser and Ney (ICASSP 1995) and the interpolated recursion in Chen and Goodman
//! (Computer Speech & Language 1999, section 3):
//! <https://u.cs.biu.ac.il/~yogo/courses/mt2014/papers/chen-goodman-99.pdf>.
//!
//! We use one fixed discount, not modified Kneser–Ney's count-dependent discounts.
//! With `C(h) = sum_w c_n(h,w)`, retained rows `R(h)`, and `0 < D < 1`,
//!
//! ```text
//! P_n(w|h) = 1[(h,w) retained] (c_n(h,w) - D) / C(h)
//!            + [C(h) - sum_(h,u in R) c_n(h,u) + D |R(h)|] / C(h)
//!              * P_(n-1)(w|suffix(h)).
//! ```
//!
//! Thus every discarded count backs off; a context with no retained rows backs
//! off completely. The unigram distribution is the normalized additive floor
//! `(continuation_count(w) + alpha) / (sum continuation_counts + alpha * V)`.
//! Neither a post-hoc probability floor nor a fabricated continuation is used.
//! Pruning is deterministic by descending count, then ascending packed key.
//!
//! Fit resets at every supplied sequence boundary, but token IDs (including any
//! EOS/BOS IDs) have no special meaning. Evaluation must reset history and cache
//! at the declared block boundary. Observe the current input, predict its next
//! token, then observe that next token only when it becomes an input.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const ALGORITHM: &str =
    "count-pruned fixed-discount interpolated Kneser-Ney with additive unigram floor";
pub const MAX_TRAIN_TOKENS: usize = 150_000_000;
pub const MAX_MODEL_BYTES: u64 = 700 * 1024 * 1024;
pub const MAX_FIT_PEAK_BYTES: u64 = 6 * 1024 * 1024 * 1024;
const MAX_SEQUENCES: usize = 65_536;
const MAX_CACHE_CAPACITY: usize = 65_536;
const TOKEN_BITS: usize = 12;
const MAGIC: &[u8; 8] = b"R4KN5\0\0\0";
const FORMAT_VERSION: u32 = 1;
const FIXED_FILE_BYTES: u64 = 8 + 4 + 4 + 8 + 8 + 8 + 32 + 8 + 4 + 5 * 53 + 8 + 4 * 16;

#[derive(Debug)]
pub enum NgramError {
    Io(std::io::Error),
    InvalidConfig(&'static str),
    InvalidToken { token: u16, vocab_size: usize },
    TooManyTokens { count: usize, limit: usize },
    Allocation,
    InvalidArtifact(&'static str),
}

impl fmt::Display for NgramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "n-gram I/O: {error}"),
            Self::InvalidConfig(reason) => write!(f, "invalid n-gram configuration: {reason}"),
            Self::InvalidToken { token, vocab_size } => {
                write!(f, "token {token} is outside vocabulary 0..{vocab_size}")
            }
            Self::TooManyTokens { count, limit } => {
                write!(
                    f,
                    "n-gram training has {count} tokens, exceeding limit {limit}"
                )
            }
            Self::Allocation => write!(f, "unable to reserve bounded n-gram storage"),
            Self::InvalidArtifact(reason) => write!(f, "invalid n-gram artifact: {reason}"),
        }
    }
}

impl std::error::Error for NgramError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NgramError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, NgramError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NgramConfig {
    pub vocab_size: usize,
    pub discount: f64,
    pub unigram_alpha: f64,
    pub max_train_tokens: usize,
    /// Maximum retained row counts for orders two, three, four, and five.
    pub row_caps: [usize; 4],
}

impl Default for NgramConfig {
    fn default() -> Self {
        Self {
            vocab_size: 4096,
            discount: 0.75,
            unigram_alpha: 1e-4,
            max_train_tokens: MAX_TRAIN_TOKENS,
            row_caps: [4_000_000, 4_000_000, 8_000_000, 8_000_000],
        }
    }
}

impl NgramConfig {
    pub fn validate(&self) -> Result<()> {
        if self.vocab_size == 0 || self.vocab_size > 4096 {
            return Err(NgramError::InvalidConfig("vocabulary must be in 1..=4096"));
        }
        validate_discount(self.discount)?;
        if !self.unigram_alpha.is_finite() || self.unigram_alpha <= 0.0 {
            return Err(NgramError::InvalidConfig(
                "unigram alpha must be finite and positive",
            ));
        }
        if !(self.unigram_alpha * self.vocab_size as f64).is_finite() {
            return Err(NgramError::InvalidConfig(
                "unigram floor denominator overflows",
            ));
        }
        if self.max_train_tokens == 0 || self.max_train_tokens > MAX_TRAIN_TOKENS {
            return Err(NgramError::InvalidConfig(
                "training token limit must be in 1..=150000000",
            ));
        }
        if self.row_caps.iter().any(|&cap| cap > self.max_train_tokens) {
            return Err(NgramError::InvalidConfig(
                "a row cap exceeds the token limit",
            ));
        }
        let cap_bytes = self.retained_cap_bytes();
        if cap_bytes > MAX_MODEL_BYTES {
            return Err(NgramError::InvalidConfig(
                "retained model cap exceeds 700 MiB",
            ));
        }
        if self.projected_peak_bytes(self.max_train_tokens) > MAX_FIT_PEAK_BYTES {
            return Err(NgramError::InvalidConfig("fit projection exceeds 6 GiB"));
        }
        Ok(())
    }

    /// Packed rows use 12 bytes and context metadata at most 18 bytes per row.
    /// Includes the largest permitted sequence-boundary list and format header.
    pub fn retained_cap_bytes(&self) -> u64 {
        self.row_caps
            .iter()
            .fold(0u64, |sum, &n| sum.saturating_add(n as u64))
            .saturating_mul(30)
            .saturating_add((self.vocab_size as u64).saturating_mul(4))
            .saturating_add(MAX_SEQUENCES as u64 * 8 + FIXED_FILE_BYTES)
    }

    /// Allocation projection, not a measured process RSS. Includes borrowed
    /// input (2 bytes/token), raw key scratch (8), count scratch (4), retained
    /// arrays, and 64 MiB for sorting stack, histogram, vector headers and I/O.
    pub fn projected_peak_bytes(&self, train_tokens: usize) -> u64 {
        (train_tokens as u64)
            .saturating_mul(14)
            .saturating_add(self.retained_cap_bytes())
            .saturating_add(64 * 1024 * 1024)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NgramOrderReport {
    pub order: u8,
    /// Raw order-five windows, or raw (order + 1) windows for continuation counts.
    pub counted_windows: u64,
    pub total_count: u64,
    pub raw_types: u64,
    pub retained_types: u64,
    pub retained_count: u64,
    /// Zero if no pruning; otherwise the count of the last retained rank.
    pub pruning_threshold: u32,
    pub threshold_types_kept: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NgramFitReport {
    pub algorithm: String,
    pub train_tokens: u64,
    pub sequence_lengths: Vec<u64>,
    /// Ordered from unigram through five-gram.
    pub orders: Vec<NgramOrderReport>,
    pub model_bytes: u64,
    pub projected_fit_peak_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct NgramFitProgress {
    pub completed_order: u8,
    pub retained_types: u64,
    pub raw_types: u64,
}

/// Structure-of-arrays avoids alignment padding in tens of millions of rows.
#[derive(Debug, Default)]
struct OrderTable {
    keys: Vec<u64>,
    counts: Vec<u32>,
    context_keys: Vec<u64>,
    context_totals: Vec<u32>,
    retained_sums: Vec<u32>,
    retained_types: Vec<u16>,
}

impl OrderTable {
    fn with_capacity(rows: usize) -> Result<Self> {
        Ok(Self {
            keys: reserved(rows)?,
            counts: reserved(rows)?,
            context_keys: reserved(rows)?,
            context_totals: reserved(rows)?,
            retained_sums: reserved(rows)?,
            retained_types: reserved(rows)?,
        })
    }

    fn terms(&self, context: u64, next: u16) -> ProbabilityLevel {
        let Ok(index) = self.context_keys.binary_search(&context) else {
            return ProbabilityLevel::default();
        };
        let key = (context << TOKEN_BITS) | u64::from(next);
        let count = self.keys.binary_search(&key).map_or(0, |i| self.counts[i]);
        ProbabilityLevel {
            count,
            total: self.context_totals[index],
            retained_sum: self.retained_sums[index],
            retained_types: self.retained_types[index],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ProbabilityLevel {
    count: u32,
    total: u32,
    retained_sum: u32,
    retained_types: u16,
}

/// A single set of binary lookups reusable across candidate discount values.
#[derive(Clone, Copy, Debug)]
pub struct NgramProbabilityTerms {
    unigram: f64,
    levels: [ProbabilityLevel; 4],
    levels_len: usize,
}

impl NgramProbabilityTerms {
    pub fn probability(&self, discount: f64) -> Result<f64> {
        validate_discount(discount)?;
        let mut probability = self.unigram;
        for level in &self.levels[..self.levels_len] {
            if level.total == 0 {
                continue;
            }
            let total = f64::from(level.total);
            let direct = if level.count == 0 {
                0.0
            } else {
                (f64::from(level.count) - discount) / total
            };
            let backoff = (f64::from(level.total - level.retained_sum)
                + discount * f64::from(level.retained_types))
                / total;
            probability = direct + backoff * probability;
        }
        if !probability.is_finite() || probability <= 0.0 {
            return Err(NgramError::InvalidConfig(
                "probability underflow or overflow; use a representable floor and discount",
            ));
        }
        Ok(probability)
    }
}

#[derive(Debug)]
pub struct KneserNey5Gram {
    config: NgramConfig,
    unigrams: Vec<u32>,
    unigram_total: u64,
    tables: [OrderTable; 4],
    report: NgramFitReport,
}

impl KneserNey5Gram {
    pub fn fit(sequences: &[&[u16]], config: NgramConfig) -> Result<(Self, NgramFitReport)> {
        Self::fit_with_progress(sequences, config, |_| {})
    }

    pub fn fit_with_progress(
        sequences: &[&[u16]],
        config: NgramConfig,
        mut progress: impl FnMut(NgramFitProgress),
    ) -> Result<(Self, NgramFitReport)> {
        config.validate()?;
        if sequences.is_empty() || sequences.len() > MAX_SEQUENCES {
            return Err(NgramError::InvalidConfig(
                "fit requires 1..=65536 sequences",
            ));
        }
        let mut train_tokens = 0usize;
        for sequence in sequences {
            train_tokens =
                train_tokens
                    .checked_add(sequence.len())
                    .ok_or(NgramError::TooManyTokens {
                        count: usize::MAX,
                        limit: config.max_train_tokens,
                    })?;
            if train_tokens > config.max_train_tokens {
                return Err(NgramError::TooManyTokens {
                    count: train_tokens,
                    limit: config.max_train_tokens,
                });
            }
            for &token in *sequence {
                validate_token(token, config.vocab_size)?;
            }
        }
        if train_tokens == 0 {
            return Err(NgramError::InvalidConfig("fit requires at least one token"));
        }
        let mut tables = std::array::from_fn(|_| OrderTable::default());
        let mut orders = vec![NgramOrderReport::default(); 5];
        // Each continuation pass starts from raw corpus types. Deriving from a
        // pruned table, or from another continuation table, loses left contexts.
        for order in (2..=5).rev() {
            let (keys, counts, windows) = collect_counts(sequences, order)?;
            let (table, report) =
                prune_counts(keys, counts, order, windows, config.row_caps[order - 2])?;
            progress(NgramFitProgress {
                completed_order: order as u8,
                retained_types: report.retained_types,
                raw_types: report.raw_types,
            });
            tables[order - 2] = table;
            orders[order - 1] = report;
        }
        let (keys, counts, windows) = collect_counts(sequences, 1)?;
        let mut unigrams = reserved(config.vocab_size)?;
        unigrams.resize(config.vocab_size, 0u32);
        let unigram_total = counts.iter().map(|&count| u64::from(count)).sum();
        for (&key, &count) in keys.iter().zip(&counts) {
            unigrams[key as usize] = count;
        }
        orders[0] = NgramOrderReport {
            order: 1,
            counted_windows: windows as u64,
            total_count: unigram_total,
            raw_types: keys.len() as u64,
            retained_types: keys.len() as u64,
            retained_count: unigram_total,
            pruning_threshold: 0,
            threshold_types_kept: 0,
        };
        progress(NgramFitProgress {
            completed_order: 1,
            retained_types: keys.len() as u64,
            raw_types: keys.len() as u64,
        });
        let mut model = Self {
            report: NgramFitReport {
                algorithm: ALGORITHM.to_owned(),
                train_tokens: train_tokens as u64,
                sequence_lengths: sequences
                    .iter()
                    .map(|sequence| sequence.len() as u64)
                    .collect(),
                orders,
                model_bytes: 0,
                projected_fit_peak_bytes: config.projected_peak_bytes(train_tokens),
            },
            config,
            unigrams,
            unigram_total,
            tables,
        };
        model.report.model_bytes = model.model_bytes();
        let report = model.report.clone();
        Ok((model, report))
    }

    pub fn config(&self) -> &NgramConfig {
        &self.config
    }

    pub fn fit_report(&self) -> &NgramFitReport {
        &self.report
    }

    /// Exact format length; excludes process allocator overhead.
    pub fn model_bytes(&self) -> u64 {
        FIXED_FILE_BYTES
            + self.report.sequence_lengths.len() as u64 * 8
            + self.unigrams.len() as u64 * 4
            + self
                .tables
                .iter()
                .map(|table| table.keys.len() as u64 * 12 + table.context_keys.len() as u64 * 18)
                .sum::<u64>()
    }

    pub fn probability(&self, context: &[u16], next: u16) -> Result<f64> {
        self.probability_with_discount(context, next, self.config.discount)
    }

    pub fn probability_with_discount(
        &self,
        context: &[u16],
        next: u16,
        discount: f64,
    ) -> Result<f64> {
        self.probability_terms(context, next)?.probability(discount)
    }

    /// Only the last four context tokens affect the score. No token is observed
    /// or persisted by this method, and no BOS token is inserted.
    pub fn probability_terms(&self, context: &[u16], next: u16) -> Result<NgramProbabilityTerms> {
        validate_token(next, self.config.vocab_size)?;
        for &token in context {
            validate_token(token, self.config.vocab_size)?;
        }
        let alpha = self.config.unigram_alpha;
        let unigram = (f64::from(self.unigrams[usize::from(next)]) + alpha)
            / (self.unigram_total as f64 + alpha * self.config.vocab_size as f64);
        let mut terms = NgramProbabilityTerms {
            unigram,
            levels: [ProbabilityLevel::default(); 4],
            levels_len: context.len().min(4),
        };
        let mut context_key = 0u64;
        for length in 1..=terms.levels_len {
            context_key |=
                u64::from(context[context.len() - length]) << ((length - 1) * TOKEN_BITS);
            terms.levels[length - 1] = self.tables[length - 1].terms(context_key, next);
        }
        Ok(terms)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut writer = BufWriter::with_capacity(1024 * 1024, File::create(path)?);
        self.write_to(&mut writer)?;
        writer.flush()?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::read_from(BufReader::with_capacity(1024 * 1024, File::open(path)?))
    }

    /// Stable little-endian representation without platform-sized integers.
    pub fn write_to(&self, mut writer: impl Write) -> Result<()> {
        writer.write_all(MAGIC)?;
        writer.write_all(&FORMAT_VERSION.to_le_bytes())?;
        writer.write_all(&(self.config.vocab_size as u32).to_le_bytes())?;
        write_u64(&mut writer, self.config.discount.to_bits())?;
        write_u64(&mut writer, self.config.unigram_alpha.to_bits())?;
        write_u64(&mut writer, self.config.max_train_tokens as u64)?;
        for cap in self.config.row_caps {
            write_u64(&mut writer, cap as u64)?;
        }
        write_u64(&mut writer, self.report.train_tokens)?;
        writer.write_all(&(self.report.sequence_lengths.len() as u32).to_le_bytes())?;
        for &length in &self.report.sequence_lengths {
            write_u64(&mut writer, length)?;
        }
        for report in &self.report.orders {
            writer.write_all(&[report.order])?;
            for value in [
                report.counted_windows,
                report.total_count,
                report.raw_types,
                report.retained_types,
                report.retained_count,
            ] {
                write_u64(&mut writer, value)?;
            }
            writer.write_all(&report.pruning_threshold.to_le_bytes())?;
            write_u64(&mut writer, report.threshold_types_kept)?;
        }
        write_u64(&mut writer, self.unigram_total)?;
        for &count in &self.unigrams {
            writer.write_all(&count.to_le_bytes())?;
        }
        for table in &self.tables {
            write_u64(&mut writer, table.keys.len() as u64)?;
            write_u64(&mut writer, table.context_keys.len() as u64)?;
            for (&key, &count) in table.keys.iter().zip(&table.counts) {
                write_u64(&mut writer, key)?;
                writer.write_all(&count.to_le_bytes())?;
            }
            for i in 0..table.context_keys.len() {
                write_u64(&mut writer, table.context_keys[i])?;
                writer.write_all(&table.context_totals[i].to_le_bytes())?;
                writer.write_all(&table.retained_sums[i].to_le_bytes())?;
                writer.write_all(&table.retained_types[i].to_le_bytes())?;
            }
        }
        Ok(())
    }

    pub fn read_from(mut reader: impl Read) -> Result<Self> {
        if &read_array::<8>(&mut reader)? != MAGIC || read_u32(&mut reader)? != FORMAT_VERSION {
            return Err(NgramError::InvalidArtifact(
                "magic or format version differs",
            ));
        }
        let vocab_size = read_u32(&mut reader)? as usize;
        let discount = f64::from_bits(read_u64(&mut reader)?);
        let unigram_alpha = f64::from_bits(read_u64(&mut reader)?);
        let max_train_tokens = read_usize(&mut reader, MAX_TRAIN_TOKENS)?;
        let mut row_caps = [0; 4];
        for cap in &mut row_caps {
            *cap = read_usize(&mut reader, MAX_TRAIN_TOKENS)?;
        }
        let config = NgramConfig {
            vocab_size,
            discount,
            unigram_alpha,
            max_train_tokens,
            row_caps,
        };
        config.validate()?;
        let train_tokens = read_u64(&mut reader)?;
        if train_tokens == 0 || train_tokens > config.max_train_tokens as u64 {
            return Err(NgramError::InvalidArtifact(
                "training token count is outside configured bounds",
            ));
        }
        let sequence_count = read_u32(&mut reader)? as usize;
        if sequence_count == 0 || sequence_count > MAX_SEQUENCES {
            return Err(NgramError::InvalidArtifact(
                "sequence count is outside bounds",
            ));
        }
        let mut sequence_lengths = reserved(sequence_count)?;
        let mut sequence_sum = 0u64;
        for _ in 0..sequence_count {
            let length = read_u64(&mut reader)?;
            if length > train_tokens {
                return Err(NgramError::InvalidArtifact(
                    "sequence length exceeds training token count",
                ));
            }
            sequence_sum += length;
            sequence_lengths.push(length);
        }
        if sequence_sum != train_tokens {
            return Err(NgramError::InvalidArtifact(
                "sequence lengths do not sum to training tokens",
            ));
        }
        let mut orders = Vec::with_capacity(5);
        for order in 1..=5 {
            let report = NgramOrderReport {
                order: read_array::<1>(&mut reader)?[0],
                counted_windows: read_u64(&mut reader)?,
                total_count: read_u64(&mut reader)?,
                raw_types: read_u64(&mut reader)?,
                retained_types: read_u64(&mut reader)?,
                retained_count: read_u64(&mut reader)?,
                pruning_threshold: read_u32(&mut reader)?,
                threshold_types_kept: read_u64(&mut reader)?,
            };
            let raw_order = if order == 5 { 5 } else { order + 1 };
            let expected_windows: u64 = sequence_lengths
                .iter()
                .map(|&length| length.saturating_sub(raw_order - 1))
                .sum();
            if report.order != order as u8
                || report.counted_windows != expected_windows
                || report.total_count > expected_windows
                || report.raw_types > report.total_count
                || report.retained_types > report.raw_types
                || report.retained_count > report.total_count
                || report.retained_types > report.retained_count
                || report.threshold_types_kept > report.retained_types
                || report.pruning_threshold as u64 > report.total_count
            {
                return Err(NgramError::InvalidArtifact(
                    "invalid per-order count report",
                ));
            }
            if order == 5 && report.total_count != expected_windows {
                return Err(NgramError::InvalidArtifact(
                    "raw five-gram counts do not sum to windows",
                ));
            }
            orders.push(report);
        }
        let unigram_total = read_u64(&mut reader)?;
        let mut unigrams = reserved(vocab_size)?;
        for _ in 0..vocab_size {
            unigrams.push(read_u32(&mut reader)?);
        }
        let count_sum: u64 = unigrams.iter().map(|&count| u64::from(count)).sum();
        let types = unigrams.iter().filter(|&&count| count > 0).count() as u64;
        if count_sum != unigram_total
            || orders[0].total_count != unigram_total
            || orders[0].retained_count != unigram_total
            || orders[0].raw_types != types
            || orders[0].retained_types != types
            || orders[0].pruning_threshold != 0
            || orders[0].threshold_types_kept != 0
        {
            return Err(NgramError::InvalidArtifact(
                "inconsistent unigram continuation counts",
            ));
        }
        let mut tables = std::array::from_fn(|_| OrderTable::default());
        for (i, table) in tables.iter_mut().enumerate() {
            let rows = read_usize(&mut reader, config.row_caps[i])?;
            let contexts = read_usize(&mut reader, rows)?;
            table.keys = reserved(rows)?;
            table.counts = reserved(rows)?;
            table.context_keys = reserved(contexts)?;
            table.context_totals = reserved(contexts)?;
            table.retained_sums = reserved(contexts)?;
            table.retained_types = reserved(contexts)?;
            for _ in 0..rows {
                table.keys.push(read_u64(&mut reader)?);
                table.counts.push(read_u32(&mut reader)?);
            }
            for _ in 0..contexts {
                table.context_keys.push(read_u64(&mut reader)?);
                table.context_totals.push(read_u32(&mut reader)?);
                table.retained_sums.push(read_u32(&mut reader)?);
                table
                    .retained_types
                    .push(u16::from_le_bytes(read_array(&mut reader)?));
            }
            validate_table(table, i + 2, vocab_size, train_tokens, &orders[i + 1])?;
        }
        if reader.read(&mut [0u8; 1])? != 0 {
            return Err(NgramError::InvalidArtifact("trailing bytes"));
        }
        let mut model = Self {
            report: NgramFitReport {
                algorithm: ALGORITHM.to_owned(),
                train_tokens,
                sequence_lengths,
                orders,
                model_bytes: 0,
                projected_fit_peak_bytes: config.projected_peak_bytes(train_tokens as usize),
            },
            config,
            unigrams,
            unigram_total,
            tables,
        };
        model.report.model_bytes = model.model_bytes();
        if model.model_bytes() > MAX_MODEL_BYTES {
            return Err(NgramError::InvalidArtifact(
                "model exceeds retained byte cap",
            ));
        }
        Ok(model)
    }
}

/// A causal empirical token cache with an explicit observation operation.
/// Empty caches use the base distribution without reducing its mass.
#[derive(Debug)]
pub struct TokenCache {
    counts: Vec<u32>,
    ring: Vec<u16>,
    next_slot: usize,
    len: usize,
}

impl TokenCache {
    pub fn new(vocab_size: usize, capacity: usize) -> Result<Self> {
        if vocab_size == 0 || vocab_size > 4096 || capacity == 0 || capacity > MAX_CACHE_CAPACITY {
            return Err(NgramError::InvalidConfig(
                "cache requires V in 1..=4096 and capacity in 1..=65536",
            ));
        }
        let mut counts = reserved(vocab_size)?;
        counts.resize(vocab_size, 0);
        let mut ring = reserved(capacity)?;
        ring.resize(capacity, 0);
        Ok(Self {
            counts,
            ring,
            next_slot: 0,
            len: 0,
        })
    }

    pub fn reset(&mut self) {
        self.counts.fill(0);
        self.next_slot = 0;
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.ring.len()
    }

    pub fn observe(&mut self, token: u16) -> Result<()> {
        validate_token(token, self.counts.len())?;
        if self.len == self.ring.len() {
            self.counts[usize::from(self.ring[self.next_slot])] -= 1;
        } else {
            self.len += 1;
        }
        self.ring[self.next_slot] = token;
        self.counts[usize::from(token)] += 1;
        self.next_slot = (self.next_slot + 1) % self.ring.len();
        Ok(())
    }

    pub fn probability(&self, next: u16) -> Result<Option<f64>> {
        validate_token(next, self.counts.len())?;
        Ok(if self.len == 0 {
            None
        } else {
            Some(f64::from(self.counts[usize::from(next)]) / self.len as f64)
        })
    }

    pub fn mixed_probability(&self, base_probability: f64, next: u16, lambda: f64) -> Result<f64> {
        if !base_probability.is_finite()
            || base_probability <= 0.0
            || base_probability > 1.0 + 1e-12
        {
            return Err(NgramError::InvalidConfig(
                "base probability must be finite and in (0,1]",
            ));
        }
        if !lambda.is_finite() || !(0.0..1.0).contains(&lambda) {
            return Err(NgramError::InvalidConfig(
                "cache lambda must be finite and in [0,1)",
            ));
        }
        Ok(match self.probability(next)? {
            Some(cache) => (1.0 - lambda) * base_probability + lambda * cache,
            None => base_probability,
        })
    }
}

fn validate_discount(discount: f64) -> Result<()> {
    if !discount.is_finite() || discount <= 0.0 || discount >= 1.0 {
        return Err(NgramError::InvalidConfig(
            "discount must be finite and in (0,1)",
        ));
    }
    Ok(())
}

fn validate_token(token: u16, vocab_size: usize) -> Result<()> {
    if usize::from(token) >= vocab_size {
        return Err(NgramError::InvalidToken { token, vocab_size });
    }
    Ok(())
}

fn reserved<T>(capacity: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| NgramError::Allocation)?;
    Ok(values)
}

fn key_mask(order: usize) -> u64 {
    (1u64 << (order * TOKEN_BITS)) - 1
}

fn collect_counts(sequences: &[&[u16]], order: usize) -> Result<(Vec<u64>, Vec<u32>, usize)> {
    let raw_order = if order == 5 { 5 } else { order + 1 };
    let windows: usize = sequences
        .iter()
        .map(|sequence| sequence.len().saturating_sub(raw_order - 1))
        .sum();
    let mut keys = reserved(windows)?;
    let mask = key_mask(raw_order);
    for sequence in sequences {
        let mut key = 0u64;
        for (i, &token) in sequence.iter().enumerate() {
            key = ((key << TOKEN_BITS) | u64::from(token)) & mask;
            if i + 1 >= raw_order {
                keys.push(key);
            }
        }
    }
    keys.sort_unstable();
    if order < 5 {
        // Deduplicate BEFORE dropping the predecessor. Each remaining raw type
        // contributes exactly one distinct left extension to its suffix.
        keys.dedup();
        let suffix_mask = key_mask(order);
        for key in &mut keys {
            *key &= suffix_mask;
        }
        keys.sort_unstable();
    }
    let mut counts = reserved(keys.len())?;
    let mut read = 0;
    let mut write = 0;
    while read < keys.len() {
        let key = keys[read];
        let mut end = read + 1;
        while end < keys.len() && keys[end] == key {
            end += 1;
        }
        keys[write] = key;
        counts.push((end - read) as u32);
        write += 1;
        read = end;
    }
    keys.truncate(write);
    Ok((keys, counts, windows))
}

fn prune_counts(
    keys: Vec<u64>,
    counts: Vec<u32>,
    order: usize,
    windows: usize,
    cap: usize,
) -> Result<(OrderTable, NgramOrderReport)> {
    let total_count = counts.iter().map(|&count| u64::from(count)).sum();
    let mut threshold = 0u32;
    let mut tie_quota = 0usize;
    if keys.len() > cap {
        // A histogram of count VALUES, never a hash table of n-gram keys. At
        // 150M total count its distinct positive entries are bounded by ~17K.
        let mut histogram = BTreeMap::<u32, usize>::new();
        for &count in &counts {
            *histogram.entry(count).or_default() += 1;
        }
        let mut remaining = cap;
        for (&count, &types) in histogram.iter().rev() {
            if remaining <= types {
                threshold = count;
                tie_quota = remaining;
                break;
            }
            remaining -= types;
        }
    }
    let mut table = OrderTable::with_capacity(keys.len().min(cap))?;
    let mut ties_left = tie_quota;
    let mut i = 0;
    while i < keys.len() {
        let context = keys[i] >> TOKEN_BITS;
        let mut total = 0u32;
        let mut retained_sum = 0u32;
        let mut retained_types = 0u16;
        while i < keys.len() && keys[i] >> TOKEN_BITS == context {
            let count = counts[i];
            total += count;
            let keep =
                keys.len() <= cap || count > threshold || (count == threshold && ties_left > 0);
            if keep {
                if keys.len() > cap && count == threshold {
                    ties_left -= 1;
                }
                table.keys.push(keys[i]);
                table.counts.push(count);
                retained_sum += count;
                retained_types += 1;
            }
            i += 1;
        }
        if retained_types > 0 {
            table.context_keys.push(context);
            table.context_totals.push(total);
            table.retained_sums.push(retained_sum);
            table.retained_types.push(retained_types);
        }
    }
    let report = NgramOrderReport {
        order: order as u8,
        counted_windows: windows as u64,
        total_count,
        raw_types: keys.len() as u64,
        retained_types: table.keys.len() as u64,
        retained_count: table.counts.iter().map(|&count| u64::from(count)).sum(),
        pruning_threshold: threshold,
        threshold_types_kept: tie_quota as u64,
    };
    Ok((table, report))
}

fn validate_table(
    table: &OrderTable,
    order: usize,
    vocab_size: usize,
    train_tokens: u64,
    report: &NgramOrderReport,
) -> Result<()> {
    if table.keys.len() as u64 != report.retained_types
        || table.keys.windows(2).any(|pair| pair[0] >= pair[1])
        || table.context_keys.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(NgramError::InvalidArtifact(
            "unordered or duplicate table keys",
        ));
    }
    for (&key, &count) in table.keys.iter().zip(&table.counts) {
        if key > key_mask(order) || count == 0 || u64::from(count) > train_tokens {
            return Err(NgramError::InvalidArtifact("invalid packed key or count"));
        }
        for digit in 0..order {
            if ((key >> (digit * TOKEN_BITS)) & 4095) as usize >= vocab_size {
                return Err(NgramError::InvalidArtifact(
                    "packed token outside vocabulary",
                ));
            }
        }
    }
    let mut row = 0;
    let mut total_retained = 0u64;
    let mut original_context_mass = 0u64;
    for i in 0..table.context_keys.len() {
        let context = table.context_keys[i];
        let mut sum = 0u64;
        let mut types = 0usize;
        while row < table.keys.len() && table.keys[row] >> TOKEN_BITS == context {
            sum += u64::from(table.counts[row]);
            types += 1;
            row += 1;
        }
        if types == 0
            || types != usize::from(table.retained_types[i])
            || sum != u64::from(table.retained_sums[i])
            || sum > u64::from(table.context_totals[i])
            || u64::from(table.context_totals[i]) > train_tokens
        {
            return Err(NgramError::InvalidArtifact(
                "inconsistent context mass or row count",
            ));
        }
        original_context_mass += u64::from(table.context_totals[i]);
        total_retained += sum;
    }
    if row != table.keys.len()
        || total_retained != report.retained_count
        || original_context_mass > report.total_count
    {
        return Err(NgramError::InvalidArtifact(
            "context metadata does not cover retained rows",
        ));
    }
    Ok(())
}

fn write_u64(writer: &mut impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn read_array<const N: usize>(reader: &mut impl Read) -> Result<[u8; N]> {
    let mut bytes = [0; N];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn read_u32(reader: &mut impl Read) -> Result<u32> {
    Ok(u32::from_le_bytes(read_array(reader)?))
}

fn read_u64(reader: &mut impl Read) -> Result<u64> {
    Ok(u64::from_le_bytes(read_array(reader)?))
}

fn read_usize(reader: &mut impl Read, limit: usize) -> Result<usize> {
    let value = read_u64(reader)?;
    if value > limit as u64 {
        return Err(NgramError::InvalidArtifact(
            "array length exceeds configured limit",
        ));
    }
    usize::try_from(value)
        .map_err(|_| NgramError::InvalidArtifact("array length does not fit platform"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(caps: [usize; 4]) -> NgramConfig {
        NgramConfig {
            vocab_size: 5,
            max_train_tokens: 100,
            row_caps: caps,
            ..NgramConfig::default()
        }
    }

    #[test]
    fn continuation_counts_boundaries_and_pruning_preserve_normalization() -> Result<()> {
        let sequences: &[&[u16]] = &[&[0, 2, 3, 0, 2, 3, 0, 2, 3], &[1, 2, 4]];
        let (full, _) = KneserNey5Gram::fit(sequences, config([100; 4]))?;
        // Raw frequencies favor (2,3) three to one, but each has exactly one
        // distinct predecessor and each target has one unigram predecessor.
        assert_eq!(full.probability(&[2], 3)?, full.probability(&[2], 4)?);
        for caps in [[100; 4], [1; 4], [0; 4]] {
            let (model, report) = KneserNey5Gram::fit(sequences, config(caps))?;
            for context in [
                &[][..],
                &[2],
                &[0, 2],
                &[1, 2],
                &[4, 4, 4, 4],
                &[0, 2, 3, 0],
            ] {
                for discount in [0.5, 0.75, 0.9] {
                    let mut total = 0.0;
                    for token in 0..5 {
                        let probability =
                            model.probability_with_discount(context, token, discount)?;
                        assert!(probability > 0.0 && probability <= 1.0);
                        total += probability;
                    }
                    assert!((total - 1.0).abs() < 1e-12);
                }
            }
            if caps[0] == 1 {
                assert!(report.orders[1].retained_count < report.orders[1].total_count);
            }
        }
        let (separate, _) = KneserNey5Gram::fit(&[&[0, 1], &[2, 3]], config([100; 4]))?;
        let alpha = separate.config().unigram_alpha;
        assert_eq!(separate.probability(&[], 2)?, alpha / (2.0 + 5.0 * alpha));
        assert!(full.probability(&[], 5).is_err());
        assert!(KneserNey5Gram::fit(&[&[5]], config([100; 4])).is_err());
        Ok(())
    }

    #[test]
    fn cache_scores_before_observation_and_resets_without_future_information() -> Result<()> {
        let mut cache = TokenCache::new(5, 2)?;
        assert_eq!(cache.mixed_probability(0.2, 3, 0.4)?, 0.2);
        cache.observe(2)?;
        let before = cache.mixed_probability(0.2, 3, 0.4)?;
        assert!((before - 0.12).abs() < 1e-12);
        assert_eq!(cache.len(), 1);
        cache.observe(3)?;
        assert!((cache.mixed_probability(0.2, 3, 0.4)? - 0.32).abs() < 1e-12);
        cache.observe(4)?;
        assert_eq!(cache.probability(2)?, Some(0.0));
        let total: f64 = (0..5)
            .map(|token| cache.mixed_probability(0.2, token, 0.4))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .sum();
        assert!((total - 1.0).abs() < 1e-12);
        cache.reset();
        assert!(cache.is_empty());
        assert_eq!(cache.mixed_probability(0.2, 3, 0.4)?, 0.2);
        assert!(cache.observe(5).is_err());
        assert!(cache.mixed_probability(0.2, 3, 1.0).is_err());
        Ok(())
    }

    #[test]
    fn deterministic_serialization_rejects_damage_and_preserves_probabilities() -> Result<()> {
        let sequences: &[&[u16]] = &[&[0, 2, 3, 0, 2, 3, 0, 2, 3], &[1, 2, 4]];
        let (model, _) = KneserNey5Gram::fit(sequences, config([2; 4]))?;
        let (again, _) = KneserNey5Gram::fit(sequences, config([2; 4]))?;
        let mut bytes = Vec::new();
        model.write_to(&mut bytes)?;
        let mut second = Vec::new();
        again.write_to(&mut second)?;
        assert_eq!(bytes, second);
        assert_eq!(bytes.len() as u64, model.model_bytes());
        let loaded = KneserNey5Gram::read_from(bytes.as_slice())?;
        for context in [&[][..], &[2], &[0, 2, 3, 0], &[4, 4, 4, 4]] {
            for token in 0..5 {
                for discount in [0.5, 0.75, 0.9] {
                    assert_eq!(
                        model.probability_with_discount(context, token, discount)?,
                        loaded.probability_with_discount(context, token, discount)?
                    );
                }
            }
        }
        assert!(KneserNey5Gram::read_from(&bytes[..bytes.len() - 1]).is_err());
        bytes.push(0);
        assert!(KneserNey5Gram::read_from(bytes.as_slice()).is_err());
        second[8] = 255;
        assert!(KneserNey5Gram::read_from(second.as_slice()).is_err());
        Ok(())
    }
}
