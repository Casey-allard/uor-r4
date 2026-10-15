//! Exact local byte-tokenizer ingestion and separately versioned occurrence bindings.
//! Storage/data preparation only. No model execution or tokenizer replacement.
use crate::{
    reserve, Error, Limits, Manifest, ObjectVersion, RangeVerifiedAgainstRoot, Result, Store,
};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

const MAGIC: &[u8; 8] = b"UORBND01";
const HEADER: usize = 88;
const OCCURRENCE_BYTES: usize = 28;
pub type TokenizerIdentity = [u8; 32];

/// A borrowed, caller-constructed shared tokenizer with exact-byte policy.
/// Tokenizer loading and its persistent merge cache belong to caller scope.
/// Identity binds its source CID and this adapter policy version.
pub struct PinnedTokenizer<'a> {
    tokenizer: &'a ByteBpeTokenizer,
    identity: TokenizerIdentity,
}
impl<'a> PinnedTokenizer<'a> {
    pub fn new(tokenizer: &'a ByteBpeTokenizer) -> Result<Self> {
        if tokenizer.adds_prefix_space() {
            return Err(Error::BindingMismatch);
        }
        let mut hash = Sha256::new();
        hash.update(b"uor-r4-object-store/lossless-byte-bpe/1\0");
        hash.update(tokenizer.address().as_bytes());
        Ok(Self {
            tokenizer,
            identity: hash.finalize().into(),
        })
    }
    pub fn identity(&self) -> TokenizerIdentity {
        self.identity
    }
    fn token_bytes(&self, id: u32, limit: u64) -> Result<Vec<u8>> {
        let length = self
            .tokenizer
            .token_byte_len(id)
            .ok_or(Error::BindingMismatch)?;
        if length == 0 {
            return Err(Error::BindingMismatch);
        }
        if length as u64 > limit {
            return Err(Error::LimitExceeded("decoded token bytes"));
        }
        let bytes = self.tokenizer.decode_bytes(&[id]);
        if bytes.len() != length {
            return Err(Error::BindingMismatch);
        }
        Ok(bytes)
    }
}

/// Equal token IDs still have distinct ordered occurrences and byte spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    index: u64,
    token_id: u32,
    start: u64,
    end: u64,
}
impl Occurrence {
    pub fn index(&self) -> u64 {
        self.index
    }
    pub fn token_id(&self) -> u32 {
        self.token_id
    }
    pub fn byte_range(&self) -> std::ops::Range<u64> {
        self.start..self.end
    }
}

/// Serialized metadata requires its own trusted provenance. Its identity fields
/// bind the target object/tokenizer; they are not signatures or authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bindings {
    version: ObjectVersion,
    tokenizer: TokenizerIdentity,
    byte_len: u64,
    occurrences: Vec<Occurrence>,
}
impl Bindings {
    pub fn object_version(&self) -> ObjectVersion {
        self.version
    }
    pub fn tokenizer_identity(&self) -> TokenizerIdentity {
        self.tokenizer
    }
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }

    fn validate(&self, limits: &Limits) -> Result<()> {
        checked_wire_size(self.occurrences.len(), limits)?;
        if self.byte_len > limits.max_object_bytes {
            return Err(Error::LimitExceeded("object bytes"));
        }
        let mut end = 0;
        for (index, occurrence) in self.occurrences.iter().enumerate() {
            if occurrence.index != index as u64
                || occurrence.start != end
                || occurrence.end <= occurrence.start
                || occurrence.end > self.byte_len
            {
                return Err(Error::BindingMismatch);
            }
            end = occurrence.end;
        }
        if end != self.byte_len {
            return Err(Error::BindingMismatch);
        }
        Ok(())
    }

    pub fn encode(&self, limits: &Limits) -> Result<Vec<u8>> {
        self.validate(limits)?;
        let size = checked_wire_size(self.occurrences.len(), limits)?;
        let mut out = Vec::new();
        reserve(&mut out, size)?;
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.version);
        out.extend_from_slice(&self.tokenizer);
        out.extend_from_slice(&self.byte_len.to_le_bytes());
        out.extend_from_slice(&(self.occurrences.len() as u64).to_le_bytes());
        for item in &self.occurrences {
            out.extend_from_slice(&item.index.to_le_bytes());
            out.extend_from_slice(&item.token_id.to_le_bytes());
            out.extend_from_slice(&item.start.to_le_bytes());
            out.extend_from_slice(&item.end.to_le_bytes());
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Self> {
        if bytes.len() > limits.max_record_bytes {
            return Err(Error::LimitExceeded("binding bytes"));
        }
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err(Error::SchemaMismatch);
        }
        let byte_len = read_u64(&bytes[72..80])?;
        if byte_len > limits.max_object_bytes {
            return Err(Error::LimitExceeded("object bytes"));
        }
        let count = usize::try_from(read_u64(&bytes[80..88])?)
            .map_err(|_| Error::LimitExceeded("occurrences"))?;
        if bytes.len() != checked_wire_size(count, limits)? {
            return Err(Error::SchemaMismatch);
        }
        let mut occurrences = Vec::new();
        reserve(&mut occurrences, count)?;
        for item in bytes[HEADER..].chunks_exact(OCCURRENCE_BYTES) {
            occurrences.push(Occurrence {
                index: read_u64(&item[..8])?,
                token_id: u32::from_le_bytes(
                    item[8..12].try_into().map_err(|_| Error::SchemaMismatch)?,
                ),
                start: read_u64(&item[12..20])?,
                end: read_u64(&item[20..28])?,
            });
        }
        let result = Self {
            version: bytes[8..40].try_into().map_err(|_| Error::SchemaMismatch)?,
            tokenizer: bytes[40..72]
                .try_into()
                .map_err(|_| Error::SchemaMismatch)?,
            byte_len,
            occurrences,
        };
        result.validate(limits)?;
        Ok(result)
    }

    /// Authenticate the requested path, object version and exact token bytes.
    /// Does not re-tokenize or assert whole-object SHA verification.
    /// A reader's `max_returned_bytes` bounds each decoded token and result,
    /// independently of the ingestion object's `max_object_bytes` budget.
    pub fn read_occurrence(
        &self,
        store: &Store,
        manifest: &Manifest,
        tokenizer: &PinnedTokenizer,
        index: u64,
    ) -> Result<RangeVerifiedAgainstRoot> {
        self.validate(store.limits())?;
        if self.version != manifest.version()
            || self.tokenizer != tokenizer.identity()
            || self.byte_len != manifest.byte_len()
        {
            return Err(Error::BindingMismatch);
        }
        let item = usize::try_from(index)
            .ok()
            .and_then(|v| self.occurrences.get(v))
            .ok_or(Error::InvalidRange)?;
        let token_bytes =
            tokenizer.token_bytes(item.token_id, store.limits().max_returned_bytes)?;
        if token_bytes.len() as u64 != item.end - item.start {
            return Err(Error::BindingMismatch);
        }
        let output = store.range(manifest, item.start..item.end)?;
        if output.bytes() != token_bytes {
            return Err(Error::BindingMismatch);
        }
        Ok(output)
    }
}

fn read_u64(bytes: &[u8]) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes.try_into().map_err(|_| Error::SchemaMismatch)?,
    ))
}
fn checked_wire_size(count: usize, limits: &Limits) -> Result<usize> {
    if count > limits.max_occurrences {
        return Err(Error::LimitExceeded("occurrences"));
    }
    let size = count
        .checked_mul(OCCURRENCE_BYTES)
        .and_then(|v| v.checked_add(HEADER))
        .ok_or(Error::LimitExceeded("binding bytes"))?;
    if size > limits.max_record_bytes {
        return Err(Error::LimitExceeded("binding bytes"));
    }
    Ok(size)
}

pub struct Ingested {
    pub manifest: Manifest,
    pub bindings: Bindings,
}

/// Tokenize source UTF-8 once, prove concatenated decoded token bytes equal the
/// original, then fold. Tokenizer changes cannot silently alter original bytes.
/// The source byte count is a conservative token-count bound before encode;
/// larger objects may use raw Store::fold without token occurrence metadata.
/// `max_object_bytes` bounds ingestion and token decoding. A later quotation
/// uses the reading store's `max_returned_bytes` instead: storing a large token
/// is allowed even if that reader cannot return it in one bounded response.
pub fn ingest(store: &mut Store, tokenizer: &PinnedTokenizer, bytes: &[u8]) -> Result<Ingested> {
    let limits = store.limits();
    if bytes.len() as u64 > limits.max_object_bytes {
        return Err(Error::LimitExceeded("object bytes"));
    }
    checked_wire_size(bytes.len(), limits)?;
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Utf8)?;
    let ids = tokenizer.tokenizer.encode(text);
    checked_wire_size(ids.len(), limits)?;
    let mut occurrences = Vec::new();
    reserve(&mut occurrences, ids.len())?;
    let mut offset = 0usize;
    for (index, &id) in ids.iter().enumerate() {
        let token = tokenizer.token_bytes(id, limits.max_object_bytes)?;
        let end = offset
            .checked_add(token.len())
            .ok_or(Error::BindingMismatch)?;
        if bytes.get(offset..end) != Some(token.as_slice()) {
            return Err(Error::BindingMismatch);
        }
        occurrences.push(Occurrence {
            index: index as u64,
            token_id: id,
            start: offset as u64,
            end: end as u64,
        });
        offset = end;
    }
    if offset != bytes.len() {
        return Err(Error::BindingMismatch);
    }
    let folded = store.fold(bytes)?;
    let bindings = Bindings {
        version: folded.manifest.version(),
        tokenizer: tokenizer.identity(),
        byte_len: bytes.len() as u64,
        occurrences,
    };
    bindings.validate(store.limits())?;
    Ok(Ingested {
        manifest: folded.manifest,
        bindings,
    })
}
