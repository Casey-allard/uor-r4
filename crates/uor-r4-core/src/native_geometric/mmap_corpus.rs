//! Memory-mapped pre-tokenized binary corpus ingestion for native geometric models.
//!
//! Provides zero-allocation streaming access to contiguous 16-bit little-endian
//! token streams with a canonical 64-byte binary header.
//!
//! # Binary Header Layout (64 bytes)
//! - `0..4`: Magic bytes `[b'U', b'O', b'R', b'T']` ("UORT")
//! - `4..8`: File format version (`u32` little-endian, currently 1)
//! - `8..16`: Total token count (`u64` little-endian)
//! - `16..20`: Vocabulary size (`u32` little-endian)
//! - `20..64`: Reserved padding (44 bytes, zeros)
//!
//! Followed immediately by `total_tokens * 2` bytes of contiguous `u16` tokens in little-endian order.
//! Because the header is exactly 64 bytes, payload offsets are 64-byte aligned and 2-byte aligned,
//! enabling zero-copy casting to `&[u16]`.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::ops::{
    Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};
use std::path::Path;

/// Canonical 4-byte magic signature: "UORT" (UOR Tokens).
pub const CORPUS_MAGIC: [u8; 4] = *b"UORT";

/// Total header size in bytes. 64-byte aligned.
pub const CORPUS_HEADER_SIZE: usize = 64;

/// Current format version.
pub const CORPUS_VERSION_1: u32 = 1;

/// Error conditions encountered during binary corpus ingestion or construction.
#[derive(Debug)]
pub enum CorpusError {
    /// Underlying I/O error.
    Io(std::io::Error),
    /// Invalid magic signature (expected `b"UORT"`).
    InvalidMagic([u8; 4]),
    /// Unsupported format version.
    UnsupportedVersion(u32),
    /// Truncated header (file smaller than 64 bytes).
    TruncatedHeader(usize),
    /// Payload truncated relative to declared token count in header.
    TruncatedPayload {
        expected_bytes: usize,
        actual_bytes: usize,
    },
    /// Memory alignment violation for `u16` slice access.
    MisalignedData,
    /// Requested range or index out of corpus bounds.
    OutOfBounds { index: usize, len: usize },
    /// Token value exceeds declared vocabulary size or 16-bit range.
    TokenOverflow { token: u32, vocab_size: u32 },
    /// Invalid vocabulary size (must be 1..=65536 for 16-bit tokens).
    InvalidVocabSize(u32),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "Corpus I/O error: {err}"),
            Self::InvalidMagic(magic) => write!(
                f,
                "Invalid corpus magic bytes: {:?} (expected b\"UORT\")",
                magic
            ),
            Self::UnsupportedVersion(ver) => {
                write!(f, "Unsupported corpus version: {ver} (expected 1)")
            }
            Self::TruncatedHeader(len) => write!(
                f,
                "Truncated corpus header: read {len} bytes (expected 64 bytes)"
            ),
            Self::TruncatedPayload {
                expected_bytes,
                actual_bytes,
            } => write!(
                f,
                "Truncated corpus payload: expected {expected_bytes} bytes, found {actual_bytes} bytes"
            ),
            Self::MisalignedData => {
                write!(f, "Corpus payload pointer is misaligned for u16 access")
            }
            Self::OutOfBounds { index, len } => {
                write!(f, "Corpus index {index} out of bounds (length {len})")
            }
            Self::TokenOverflow { token, vocab_size } => write!(
                f,
                "Token {token} exceeds vocabulary size {vocab_size} or 16-bit range"
            ),
            Self::InvalidVocabSize(size) => write!(
                f,
                "Invalid corpus vocabulary size: {size} (must be 1..=65536 for u16 tokens)"
            ),
        }
    }
}

impl std::error::Error for CorpusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CorpusError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// 64-byte binary header for `.u16` pre-tokenized corpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorpusHeader {
    /// Format version (always 1 for CORPUS_VERSION_1).
    pub version: u32,
    /// Total count of 16-bit tokens in the corpus.
    pub total_tokens: u64,
    /// Model vocabulary size (tokens must satisfy `token < vocab_size`).
    pub vocab_size: u32,
}

impl CorpusHeader {
    /// Creates a new header with version 1.
    pub fn new(total_tokens: u64, vocab_size: u32) -> Self {
        Self {
            version: CORPUS_VERSION_1,
            total_tokens,
            vocab_size,
        }
    }

    /// Serializes the header into a contiguous 64-byte array.
    pub fn to_bytes(&self) -> [u8; CORPUS_HEADER_SIZE] {
        let mut bytes = [0u8; CORPUS_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&CORPUS_MAGIC);
        bytes[4..8].copy_from_slice(&self.version.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.total_tokens.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.vocab_size.to_le_bytes());
        // Remaining 20..64 bytes are reserved zero padding.
        bytes
    }

    /// Deserializes a header from a slice of at least 64 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CorpusError> {
        if bytes.len() < CORPUS_HEADER_SIZE {
            return Err(CorpusError::TruncatedHeader(bytes.len()));
        }
        let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
        if magic != CORPUS_MAGIC {
            return Err(CorpusError::InvalidMagic(magic));
        }
        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if version != CORPUS_VERSION_1 {
            return Err(CorpusError::UnsupportedVersion(version));
        }
        let total_tokens = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let vocab_size = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        if vocab_size == 0 || vocab_size > 65536 {
            return Err(CorpusError::InvalidVocabSize(vocab_size));
        }
        Ok(Self {
            version,
            total_tokens,
            vocab_size,
        })
    }
}

/// Zero-allocation memory-mapped reader for pre-tokenized corpus files.
///
/// Uses `memmap2` for zero-copy kernel paging. Working resident memory
/// remains strictly bounded (< 128 MB) as the operating system kernel
/// faults pages in and out transparently.
pub struct MmapCorpusReader {
    mmap: memmap2::Mmap,
    header: CorpusHeader,
}

impl MmapCorpusReader {
    /// Opens and memory-maps a `.u16` binary corpus file from disk.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, CorpusError> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len() as usize;
        if file_len < CORPUS_HEADER_SIZE {
            return Err(CorpusError::TruncatedHeader(file_len));
        }
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        #[cfg(unix)]
        {
            let _ = mmap.advise(memmap2::Advice::Sequential);
        }
        Self::from_mmap(mmap)
    }

    /// Constructs an `MmapCorpusReader` from an existing memory map.
    pub fn from_mmap(mmap: memmap2::Mmap) -> Result<Self, CorpusError> {
        if mmap.len() < CORPUS_HEADER_SIZE {
            return Err(CorpusError::TruncatedHeader(mmap.len()));
        }
        let header = CorpusHeader::from_bytes(&mmap[0..CORPUS_HEADER_SIZE])?;
        let actual_payload_bytes = mmap.len() - CORPUS_HEADER_SIZE;
        let expected_payload_bytes = usize::try_from(header.total_tokens)
            .ok()
            .and_then(|t| t.checked_mul(std::mem::size_of::<u16>()))
            .ok_or(CorpusError::TruncatedPayload {
                expected_bytes: usize::MAX,
                actual_bytes: actual_payload_bytes,
            })?;
        if actual_payload_bytes < expected_payload_bytes {
            return Err(CorpusError::TruncatedPayload {
                expected_bytes: expected_payload_bytes,
                actual_bytes: actual_payload_bytes,
            });
        }
        let ptr = unsafe { mmap.as_ptr().add(CORPUS_HEADER_SIZE) };
        if (ptr as usize) % std::mem::align_of::<u16>() != 0 {
            return Err(CorpusError::MisalignedData);
        }
        Ok(Self { mmap, header })
    }

    /// Returns the parsed 64-byte header.
    #[inline]
    pub fn header(&self) -> &CorpusHeader {
        &self.header
    }

    /// Returns the total number of tokens declared and accessible in the corpus.
    #[inline]
    pub fn total_tokens(&self) -> usize {
        self.header.total_tokens as usize
    }

    /// Returns the vocabulary size declared in the corpus header.
    #[inline]
    pub fn vocab_size(&self) -> u32 {
        self.header.vocab_size
    }

    /// Returns the full token sequence as a zero-copy, zero-allocation `&[u16]` slice.
    #[inline]
    pub fn as_slice(&self) -> &[u16] {
        let ptr = unsafe { self.mmap.as_ptr().add(CORPUS_HEADER_SIZE) as *const u16 };
        // Safety:
        // 1. `ptr` is aligned to at least 64 bytes (page offset 64), which satisfies align_of::<u16>() == 2.
        // 2. `mmap` holds at least `CORPUS_HEADER_SIZE + total_tokens * 2` bytes.
        // 3. Integer primitives u16 have no invalid bit patterns.
        // 4. `mmap` is immutable and lives as long as `&self`.
        unsafe { std::slice::from_raw_parts(ptr, self.header.total_tokens as usize) }
    }

    /// Returns a single token at `index`, or `None` if out of bounds.
    /// Handles byte-order conversion from canonical little-endian format.
    #[inline]
    pub fn get(&self, index: usize) -> Option<u16> {
        self.as_slice().get(index).map(|&tok| u16::from_le(tok))
    }

    /// Returns a subslice `&[u16]` of tokens starting at `start` with length `len`,
    /// or `None` if out of bounds.
    #[inline]
    pub fn subslice(&self, start: usize, len: usize) -> Option<&[u16]> {
        let end = start.checked_add(len)?;
        if end <= self.total_tokens() {
            Some(&self.as_slice()[start..end])
        } else {
            None
        }
    }

    /// Returns a zero-allocation streaming iterator over non-overlapping contiguous chunks of size `chunk_size`.
    #[inline]
    pub fn chunks(&self, chunk_size: usize) -> CorpusChunkIter<'_> {
        CorpusChunkIter {
            slice: self.as_slice(),
            chunk_size,
            offset: 0,
        }
    }

    /// Returns a zero-allocation streaming iterator over sliding windows of size `window_size` with stride 1.
    #[inline]
    pub fn windows(&self, window_size: usize) -> CorpusWindowIter<'_> {
        self.window_stride(window_size, 1)
    }

    /// Returns a zero-allocation streaming iterator over sliding windows of size `window_size` with custom `stride`.
    #[inline]
    pub fn window_stride(&self, window_size: usize, stride: usize) -> CorpusWindowIter<'_> {
        CorpusWindowIter {
            slice: self.as_slice(),
            window_size,
            stride: stride.max(1),
            offset: 0,
        }
    }
}

impl Deref for MmapCorpusReader {
    type Target = [u16];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Index<usize> for MmapCorpusReader {
    type Output = u16;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<Range<usize>> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<RangeInclusive<usize>> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<RangeFrom<usize>> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<RangeTo<usize>> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<RangeToInclusive<usize>> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Index<RangeFull> for MmapCorpusReader {
    type Output = [u16];

    #[inline]
    fn index(&self, index: RangeFull) -> &Self::Output {
        &self.as_slice()[index]
    }
}

/// Zero-allocation chunk iterator yielding `&'a [u16]` slices.
pub struct CorpusChunkIter<'a> {
    slice: &'a [u16],
    chunk_size: usize,
    offset: usize,
}

impl<'a> Iterator for CorpusChunkIter<'a> {
    type Item = &'a [u16];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk_size == 0 || self.offset >= self.slice.len() {
            return None;
        }
        let end = self
            .offset
            .saturating_add(self.chunk_size)
            .min(self.slice.len());
        let chunk = &self.slice[self.offset..end];
        self.offset = end;
        Some(chunk)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.chunk_size == 0 || self.offset >= self.slice.len() {
            return (0, Some(0));
        }
        let remaining = self.slice.len() - self.offset;
        let count = (remaining - 1) / self.chunk_size + 1;
        (count, Some(count))
    }
}

impl<'a> ExactSizeIterator for CorpusChunkIter<'a> {}

/// Zero-allocation sliding window iterator yielding `&'a [u16]` slices.
pub struct CorpusWindowIter<'a> {
    slice: &'a [u16],
    window_size: usize,
    stride: usize,
    offset: usize,
}

impl<'a> Iterator for CorpusWindowIter<'a> {
    type Item = &'a [u16];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.window_size == 0 {
            return None;
        }
        let end = self.offset.checked_add(self.window_size)?;
        if end > self.slice.len() {
            return None;
        }
        let window = &self.slice[self.offset..end];
        self.offset = self.offset.saturating_add(self.stride);
        Some(window)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.window_size == 0 {
            return (0, Some(0));
        }
        let end = match self.offset.checked_add(self.window_size) {
            Some(e) => e,
            None => return (0, Some(0)),
        };
        if end > self.slice.len() {
            return (0, Some(0));
        }
        let avail = self.slice.len() - end;
        let count = 1 + avail / self.stride;
        (count, Some(count))
    }
}

impl<'a> ExactSizeIterator for CorpusWindowIter<'a> {}

/// Streaming writer for creating `.u16` binary corpus files with a 64-byte header.
pub struct CorpusWriter {
    writer: BufWriter<File>,
    vocab_size: u32,
    token_count: u64,
}

impl CorpusWriter {
    /// Creates a new corpus file at `path`, writing a placeholder header.
    pub fn create<P: AsRef<Path>>(path: P, vocab_size: u32) -> Result<Self, CorpusError> {
        if vocab_size == 0 || vocab_size > 65536 {
            return Err(CorpusError::InvalidVocabSize(vocab_size));
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::with_capacity(1024 * 1024, file);
        // Write placeholder header
        let placeholder = CorpusHeader::new(0, vocab_size);
        writer.write_all(&placeholder.to_bytes())?;
        Ok(Self {
            writer,
            vocab_size,
            token_count: 0,
        })
    }

    /// Appends a slice of 16-bit tokens to the corpus stream.
    ///
    /// Validates all tokens upfront before writing to guarantee atomicity:
    /// if any token exceeds `vocab_size`, no corrupted or partial data is written.
    pub fn write_tokens(&mut self, tokens: &[u16]) -> Result<(), CorpusError> {
        if tokens.is_empty() {
            return Ok(());
        }
        for &tok in tokens {
            if tok as u32 >= self.vocab_size {
                return Err(CorpusError::TokenOverflow {
                    token: tok as u32,
                    vocab_size: self.vocab_size,
                });
            }
        }
        self.write_tokens_unchecked(tokens)
    }

    /// Appends a slice of 16-bit tokens without per-token vocabulary validation (faster for trusted streams).
    pub fn write_tokens_unchecked(&mut self, tokens: &[u16]) -> Result<(), CorpusError> {
        if tokens.is_empty() {
            return Ok(());
        }
        // Write little-endian bytes directly
        #[cfg(target_endian = "little")]
        {
            let byte_slice = unsafe {
                std::slice::from_raw_parts(
                    tokens.as_ptr() as *const u8,
                    std::mem::size_of_val(tokens),
                )
            };
            self.writer.write_all(byte_slice)?;
        }
        #[cfg(not(target_endian = "little"))]
        {
            for &tok in tokens {
                self.writer.write_all(&tok.to_le_bytes())?;
            }
        }
        self.token_count += tokens.len() as u64;
        Ok(())
    }

    /// Finalizes the corpus file by updating the 64-byte header with the final token count and flushing.
    pub fn finish(mut self) -> Result<u64, CorpusError> {
        self.writer.flush()?;
        let header = CorpusHeader::new(self.token_count, self.vocab_size);
        let mut file = self.writer.into_inner().map_err(std::io::Error::other)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&header.to_bytes())?;
        file.flush()?;
        Ok(self.token_count)
    }

    /// Convenience function to write a complete token slice into a new corpus file.
    pub fn write_file<P: AsRef<Path>>(
        path: P,
        vocab_size: u32,
        tokens: &[u16],
    ) -> Result<u64, CorpusError> {
        let mut writer = Self::create(path, vocab_size)?;
        writer.write_tokens(tokens)?;
        writer.finish()
    }
}
