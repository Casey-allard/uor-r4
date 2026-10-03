//! Label-free protocol-2 lexical view of one exact selected source.
//!
//! The store's original token sequence stays unchanged. The derived sequence
//! encodes one literal protocol separator followed by the source's exact bytes.
//! Its positions are **emission-view positions**, not original token offsets:
//! a view token may merge several original tokens or split an original token.
//! Tokenization/admission allocates; this module is outside the numerical reader.

use std::ops::Range;

use serde::Serialize;
use uor_r4_integer::geometric_occurrence_read::SelectedRecordFrame;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::{invalid, sha256_bytes, Result};

pub const POLICY: &str = "literal-role-dialogue2-source-emission-view/1;one-literal-space+exact-UTF8-source-bytes;bound-byte-BPE;no-trim-normalization-labels-or-appended-punctuation/EOS;half-open-byte-provenance;derived-token-positions-not-original-occurrences";

/// Byte provenance of one derived token. Every interval is half-open.
///
/// `source_byte_*` indexes the unmodified original byte string. The original
/// token range contains every original piece overlapping that byte interval.
/// A standalone protocol separator has empty source/token ranges `0..0`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EmissionProvenance {
    emitted_token_offset: usize,
    emitted_byte_start: usize,
    emitted_byte_end: usize,
    source_byte_start: usize,
    source_byte_end: usize,
    original_token_start: usize,
    original_token_end: usize,
    includes_protocol_separator: bool,
}

impl EmissionProvenance {
    pub fn emitted_token_offset(&self) -> usize {
        self.emitted_token_offset
    }
    pub fn emitted_byte_range(&self) -> Range<usize> {
        self.emitted_byte_start..self.emitted_byte_end
    }
    pub fn source_byte_range(&self) -> Range<usize> {
        self.source_byte_start..self.source_byte_end
    }
    pub fn original_token_range(&self) -> Range<usize> {
        self.original_token_start..self.original_token_end
    }
    pub fn includes_protocol_separator(&self) -> bool {
        self.includes_protocol_separator
    }
}

/// Admitted immutable view. Deserialize by recompiling the original IDs with
/// the bound tokenizer, rather than trusting externally supplied provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEmissionView {
    policy: &'static str,
    tokenizer_sha256: String,
    original_token_ids: Vec<u32>,
    emitted_token_ids: Vec<u32>,
    original_bytes: Vec<u8>,
    rendered_bytes: Vec<u8>,
    provenance: Vec<EmissionProvenance>,
}

impl SourceEmissionView {
    pub fn policy(&self) -> &'static str {
        self.policy
    }
    pub fn emitted_token_ids(&self) -> &[u32] {
        &self.emitted_token_ids
    }
    pub fn original_token_ids(&self) -> &[u32] {
        &self.original_token_ids
    }
    pub fn original_bytes(&self) -> &[u8] {
        &self.original_bytes
    }
    pub fn rendered_bytes(&self) -> &[u8] {
        &self.rendered_bytes
    }
    pub fn tokenizer_sha256(&self) -> &str {
        &self.tokenizer_sha256
    }
    pub fn provenance(&self) -> &[EmissionProvenance] {
        &self.provenance
    }

    /// Preserve the supplied exact identity/metadata and substitute only this
    /// admitted lexical view. The caller must wrap the reader's token offsets
    /// with this provenance; they no longer denote original token occurrences.
    ///
    /// The compiler receives token IDs, not a record identity. It verifies those
    /// original IDs here and preserves (rather than invents) the caller's record.
    pub fn derived_frame<'a>(
        &'a self,
        original: SelectedRecordFrame<'a>,
    ) -> Result<SelectedRecordFrame<'a>> {
        if original.token_ids != self.original_token_ids.as_slice() {
            return Err(invalid(
                "source emission view original token identity differs",
            ));
        }
        Ok(SelectedRecordFrame {
            identity: original.identity,
            metadata: original.metadata,
            token_ids: &self.emitted_token_ids,
        })
    }
}

/// Parse the tokenizer once. This adapter takes no answer, label, desired token,
/// relation or target prefix; its sole added byte is the protocol separator.
pub struct SourceEmissionCompiler {
    tokenizer: ByteBpeTokenizer,
    tokenizer_sha256: String,
}

impl SourceEmissionCompiler {
    pub fn new(tokenizer_bytes: &[u8]) -> Result<Self> {
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_bytes)
            .ok_or_else(|| invalid("source emission view requires a valid byte-BPE tokenizer"))?;
        Ok(Self {
            tokenizer,
            tokenizer_sha256: sha256_bytes(tokenizer_bytes),
        })
    }

    pub fn tokenizer_sha256(&self) -> &str {
        &self.tokenizer_sha256
    }

    pub fn compile(&self, original_ids: &[u32]) -> Result<SourceEmissionView> {
        if original_ids.is_empty() {
            return Err(invalid(
                "source emission view requires a nonempty selected source",
            ));
        }
        let original_pieces = self.pieces(original_ids)?;
        let original_bytes = join_pieces(&original_pieces)?;
        let original_text = std::str::from_utf8(&original_bytes)
            .map_err(|_| invalid("source emission view source is not exact UTF8"))?;
        let capacity = original_bytes
            .len()
            .checked_add(1)
            .ok_or_else(|| invalid("source emission view byte length overflow"))?;
        let mut rendered = String::with_capacity(capacity);
        rendered.push(' ');
        rendered.push_str(original_text);
        let emitted_token_ids = self.tokenizer.encode(&rendered);
        let emitted_pieces = self.pieces(&emitted_token_ids)?;
        let provenance = map_provenance(&original_pieces, &emitted_pieces)?;
        Ok(SourceEmissionView {
            policy: POLICY,
            tokenizer_sha256: self.tokenizer_sha256.clone(),
            original_token_ids: original_ids.to_vec(),
            emitted_token_ids,
            original_bytes,
            rendered_bytes: rendered.into_bytes(),
            provenance,
        })
    }

    fn pieces(&self, ids: &[u32]) -> Result<Vec<Vec<u8>>> {
        ids.iter()
            .map(|&id| {
                if id as usize >= self.tokenizer.vocab_size() {
                    return Err(invalid("source emission view token outside tokenizer"));
                }
                // decode_bytes preserves partial UTF8, while decode would replace it.
                let piece = self.tokenizer.decode_bytes(&[id]);
                if piece.is_empty() {
                    return Err(invalid("source emission view token has no decoded bytes"));
                }
                Ok(piece)
            })
            .collect()
    }
}

fn join_pieces(pieces: &[Vec<u8>]) -> Result<Vec<u8>> {
    let size = pieces.iter().try_fold(0usize, |size, piece| {
        if piece.is_empty() {
            return Err(invalid(
                "source emission view contains an empty token piece",
            ));
        }
        size.checked_add(piece.len())
            .ok_or_else(|| invalid("source emission view byte length overflow"))
    })?;
    let mut bytes = Vec::with_capacity(size);
    for piece in pieces {
        bytes.extend_from_slice(piece);
    }
    Ok(bytes)
}

/// Map raw pieces, not individually decoded UTF8 strings. Exact roundtrip is
/// checked before deriving intervals, so no byte alignment is guessed.
fn map_provenance(original: &[Vec<u8>], emitted: &[Vec<u8>]) -> Result<Vec<EmissionProvenance>> {
    if original.is_empty() || emitted.is_empty() {
        return Err(invalid("source emission view has an empty token sequence"));
    }
    let source = join_pieces(original)?;
    let rendered = join_pieces(emitted)?;
    if rendered.first() != Some(&b' ') || rendered.get(1..) != Some(source.as_slice()) {
        return Err(invalid("source emission view failed exact byte roundtrip"));
    }
    let boundary_count = original
        .len()
        .checked_add(1)
        .ok_or_else(|| invalid("source emission view token count overflow"))?;
    let mut boundaries = Vec::with_capacity(boundary_count);
    boundaries.push(0usize);
    let mut end = 0usize;
    for piece in original {
        end = end
            .checked_add(piece.len())
            .ok_or_else(|| invalid("source emission view byte offset overflow"))?;
        boundaries.push(end);
    }
    let mut cursor = 0usize;
    let mut result = Vec::with_capacity(emitted.len());
    for (emitted_token_offset, piece) in emitted.iter().enumerate() {
        let next = cursor
            .checked_add(piece.len())
            .ok_or_else(|| invalid("source emission view byte offset overflow"))?;
        let source_byte_start = cursor.saturating_sub(1);
        let source_byte_end = next.saturating_sub(1);
        let (original_token_start, original_token_end) = if source_byte_start == source_byte_end {
            (0, 0)
        } else {
            (
                boundaries[1..].partition_point(|&b| b <= source_byte_start),
                boundaries.partition_point(|&b| b < source_byte_end),
            )
        };
        result.push(EmissionProvenance {
            emitted_token_offset,
            emitted_byte_start: cursor,
            emitted_byte_end: next,
            source_byte_start,
            source_byte_end,
            original_token_start,
            original_token_end,
            includes_protocol_separator: cursor == 0,
        });
        cursor = next;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_occurrence_read::{FrameMetadata, FrameStatus, SourceIdentity};

    const FUSED: &str = r#"{
        "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
        "model":{"type":"BPE",
            "vocab":{"a":0,"b":1,"ab":2,"Ġ":3,"Ġa":4,"Ġab":5,"Ã":6,"©":7},
            "merges":["a b","Ġ ab"]}
    }"#;

    #[test]
    fn source_emission_view_fuses_original_pieces_without_rewriting_identity() -> Result<()> {
        let compiler = SourceEmissionCompiler::new(FUSED.as_bytes())?;
        let original = [0, 1];
        let view = compiler.compile(&original)?;
        assert_eq!(view.original_token_ids(), &original);
        assert_eq!(view.emitted_token_ids(), &[5]);
        assert_eq!(view.original_bytes(), b"ab");
        assert_eq!(view.rendered_bytes(), b" ab");
        assert_eq!(view.tokenizer_sha256(), sha256_bytes(FUSED.as_bytes()));
        let p = &view.provenance()[0];
        assert_eq!(p.emitted_byte_range(), 0..3);
        assert_eq!(p.source_byte_range(), 0..2);
        assert_eq!(p.original_token_range(), 0..2);
        assert!(p.includes_protocol_separator());
        let entity = [1, 0];
        let frame = SelectedRecordFrame {
            identity: SourceIdentity {
                record: 9,
                commit: 12,
            },
            metadata: FrameMetadata {
                scope: b"fixture",
                entity: &entity,
                relation: 4,
                view: 2,
                status: FrameStatus::Found,
            },
            token_ids: &original,
        };
        let derived = view.derived_frame(frame)?;
        assert_eq!(derived.identity, frame.identity);
        assert_eq!(derived.metadata.scope, frame.metadata.scope);
        assert_eq!(derived.metadata.entity, frame.metadata.entity);
        assert_eq!(derived.metadata.relation, 4);
        assert_eq!(derived.metadata.view, 2);
        assert_eq!(derived.metadata.status, FrameStatus::Found);
        assert_eq!(derived.token_ids, &[5]);
        let other_record = SourceIdentity {
            record: 10,
            commit: 13,
        };
        assert_eq!(
            view.derived_frame(SelectedRecordFrame {
                identity: other_record,
                ..frame
            })?
            .identity,
            other_record
        );
        let different = [2]; // Same decoded bytes do not equal original token identity.
        assert!(view
            .derived_frame(SelectedRecordFrame {
                token_ids: &different,
                ..frame
            })
            .is_err());
        assert!(view
            .derived_frame(SelectedRecordFrame {
                token_ids: &[1, 0],
                ..frame
            })
            .is_err());
        assert_eq!(serde_json::to_value(&view)?["policy"], POLICY);
        Ok(())
    }

    #[test]
    fn source_emission_view_repeated_pieces_keep_distinct_byte_occurrences() -> Result<()> {
        let compiler = SourceEmissionCompiler::new(FUSED.as_bytes())?;
        let view = compiler.compile(&[0, 0])?;
        assert_eq!(view.emitted_token_ids(), &[3, 0, 0]);
        let p = view.provenance();
        assert_eq!(p[0].source_byte_range(), 0..0);
        assert_eq!(p[0].original_token_range(), 0..0);
        assert!(p[0].includes_protocol_separator());
        assert_eq!(p[1].source_byte_range(), 0..1);
        assert_eq!(p[1].original_token_range(), 0..1);
        assert_eq!(p[2].source_byte_range(), 1..2);
        assert_eq!(p[2].original_token_range(), 1..2);
        assert!(!p[2].includes_protocol_separator());
        // The view may also split one original piece: both fragments retain it.
        let split = map_provenance(&[b"ab".to_vec()], &[b" a".to_vec(), b"b".to_vec()])?;
        assert_eq!(split[0].original_token_range(), 0..1);
        assert_eq!(split[1].original_token_range(), 0..1);
        assert_eq!(split[1].source_byte_range(), 1..2);
        Ok(())
    }

    #[test]
    fn source_emission_view_preserves_partial_utf8_and_standalone_separator() -> Result<()> {
        let compiler = SourceEmissionCompiler::new(FUSED.as_bytes())?;
        let view = compiler.compile(&[6, 7])?;
        assert_eq!(view.emitted_token_ids(), &[3, 6, 7]);
        assert_eq!(view.provenance()[1].source_byte_range(), 0..1);
        assert_eq!(view.provenance()[2].source_byte_range(), 1..2);
        let fused = map_provenance(&[vec![0xc3], vec![0xa9]], &[vec![b' ', 0xc3, 0xa9]])?;
        assert_eq!(fused[0].source_byte_range(), 0..2);
        assert_eq!(fused[0].original_token_range(), 0..2);
        assert!(compiler.compile(&[6]).is_err());
        assert!(compiler.compile(&[]).is_err());
        assert!(compiler.compile(&[99]).is_err());
        Ok(())
    }

    #[test]
    fn source_emission_view_does_not_trim_or_accept_changed_rendered_bytes() -> Result<()> {
        let compiler = SourceEmissionCompiler::new(FUSED.as_bytes())?;
        let view = compiler.compile(&[3, 0, 3])?;
        assert_eq!(
            compiler.tokenizer.decode_bytes(view.emitted_token_ids()),
            b"  a "
        );
        assert!(map_provenance(&[b"ab".to_vec()], &[b" ab.".to_vec()]).is_err());
        assert!(map_provenance(&[b"ab".to_vec()], &[b"ab".to_vec()]).is_err());
        assert!(map_provenance(&[b"ab".to_vec()], &[b" ac".to_vec()]).is_err());
        assert!(map_provenance(&[Vec::new()], &[b" ".to_vec()]).is_err());
        assert!(SourceEmissionCompiler::new(b"not-json").is_err());
        Ok(())
    }
}
