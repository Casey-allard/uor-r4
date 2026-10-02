//! Re-encode a prepared literal-role dialogue split from protocol 1 into
//! protocol 2 (`uor_r4_tokenizer::dialogue::SCHEMA_V2`), document for
//! document, so a corpus whose raw preparation tool is gone (chat-v0) keeps
//! exactly its documents, sources and split.
//!
//! ```text
//! reencode-dialogue in=SPLIT_DIR out=NEW_DIR tokenizer=TOKENIZER.json
//! ```
//!
//! `SPLIT_DIR` holds `tokens.u16`, `response_mask.u8` and `manifest.json`
//! (schema `uor-r4-chat-corpus/v1`). Each document (from its BOS) is decoded
//! back into messages by the protocol 1 template; it is kept only if
//! re-encoding those messages in protocol 1 reproduces its tokens and its
//! response mask exactly, and is then encoded in protocol 2. Documents that
//! do not round-trip are dropped and counted per source. The new manifest is
//! the old one with the token counts, digests and per-source counts
//! recomputed, `dialogue_protocol` set, and the source split recorded under
//! `reencoded_from`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const MARKERS: [(&str, &str); 3] = [
    ("System: ", "system"),
    ("User: ", "user"),
    ("Assistant: ", "assistant"),
];
/// Stands for an EOS token inside a decoded document.
const EOS_MARK: char = '\u{0}';

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

/// The messages of one protocol 1 document's text (BOS removed, each EOS
/// as [`EOS_MARK`]), by the template: turns joined by "\n", each
/// `<marker><content>`, an assistant turn ending at its EOS, and a
/// document-terminal EOS after a final non-assistant turn. `None` when the
/// text does not parse.
fn parse_document(text: &str) -> Option<Vec<(String, String)>> {
    let mut messages = Vec::new();
    let mut rest = text;
    let mut first = true;
    while !rest.is_empty() {
        if rest == EOS_MARK.to_string() {
            // The document-terminal EOS after a non-assistant final turn.
            return (messages
                .last()
                .is_some_and(|(r, _): &(String, String)| r != "assistant"))
            .then_some(messages);
        }
        if !first {
            rest = rest.strip_prefix('\n')?;
        }
        first = false;
        let (marker, role) = MARKERS.iter().find(|(m, _)| rest.starts_with(m))?;
        rest = &rest[marker.len()..];
        if *role == "assistant" {
            let end = rest.find(EOS_MARK)?;
            messages.push(((*role).to_owned(), rest[..end].to_owned()));
            rest = &rest[end + EOS_MARK.len_utf8()..];
        } else {
            // Up to the next "\n<marker>" or the terminal EOS, whichever is
            // first; the round trip rejects a wrong choice.
            let next = MARKERS
                .iter()
                .filter_map(|(m, _)| rest.find(&format!("\n{m}")))
                .min();
            let terminal = rest.strip_suffix(EOS_MARK).map(str::len);
            let end = match (next, terminal) {
                (Some(n), Some(t)) => n.min(t),
                (Some(n), None) => n,
                (None, Some(t)) => t,
                (None, None) => return None,
            };
            messages.push(((*role).to_owned(), rest[..end].to_owned()));
            rest = &rest[end..];
        }
    }
    (!messages.is_empty()).then_some(messages)
}

fn run(args: &[String]) -> Result<()> {
    let arg = |key: &str| -> Result<PathBuf> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(PathBuf::from)
            .ok_or_else(|| format!("missing {key}="))
    };
    let (input, out, tokenizer_path) = (arg("in")?, arg("out")?, arg("tokenizer")?);
    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let v1 = DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| e.to_string())?;
    let v2 = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let (one, two) = (
        v1.bind(&tokenizer).map_err(|e| e.to_string())?,
        v2.bind(&tokenizer).map_err(|e| e.to_string())?,
    );
    let manifest_path = input.join("manifest.json");
    let mut manifest: Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| format!("{}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| e.to_string())?;
    if manifest.get("dialogue_protocol").is_some() {
        return Err("the split already declares a dialogue protocol".into());
    }
    let reader = MmapCorpusReader::open(input.join("tokens.u16")).map_err(|e| e.to_string())?;
    let tokens = reader.as_slice();
    let mask = fs::read(input.join("response_mask.u8")).map_err(|e| e.to_string())?;
    if tokens.len() != mask.len() {
        return Err("tokens and mask differ in length".into());
    }
    fs::create_dir(&out).map_err(|e| format!("{} must be new: {e}", out.display()))?;
    let out_tokens = out.join("tokens.u16");
    let mut writer =
        CorpusWriter::create(&out_tokens, reader.vocab_size()).map_err(|e| e.to_string())?;
    let mut out_mask: Vec<u8> = Vec::new();
    let files = manifest["files"]
        .as_array()
        .cloned()
        .ok_or("the manifest has no files")?;
    let (bos, eos) = (v1.bos_id, v1.eos_id);
    let mut start = 0usize;
    let mut new_files = Vec::new();
    let (mut kept_total, mut dropped_total) = (0usize, 0usize);
    for mut file in files {
        let count = file["tokens"].as_u64().ok_or("a source without tokens")? as usize;
        let span = &tokens[start..start + count];
        let span_mask = &mask[start..start + count];
        start += count;
        let mut doc_starts: Vec<usize> = span
            .iter()
            .enumerate()
            .filter(|(_, &t)| u32::from(t) == bos)
            .map(|(i, _)| i)
            .collect();
        if doc_starts.first() != Some(&0) {
            return Err(format!("source {} does not start with BOS", file["label"]));
        }
        doc_starts.push(count);
        let (mut source_tokens, mut source_response, mut kept, mut dropped) = (0, 0, 0, 0);
        for window in doc_starts.windows(2) {
            let doc: Vec<u32> = span[window[0]..window[1]]
                .iter()
                .map(|&t| u32::from(t))
                .collect();
            let doc_mask = &span_mask[window[0]..window[1]];
            let mut text = String::new();
            let mut piece = Vec::new();
            for &id in &doc[1..] {
                if id == eos {
                    text.push_str(&String::from_utf8_lossy(&tokenizer.decode_bytes(&piece)));
                    piece.clear();
                    text.push(EOS_MARK);
                } else {
                    piece.push(id);
                }
            }
            text.push_str(&String::from_utf8_lossy(&tokenizer.decode_bytes(&piece)));
            let parsed = parse_document(&text);
            let round_trip = parsed.as_ref().and_then(|messages| {
                let view: Vec<Message<'_>> = messages
                    .iter()
                    .map(|(role, content)| Message { role, content })
                    .collect();
                let again = one.encode_document(&view);
                (again.tokens == doc && again.response_mask == doc_mask)
                    .then(|| two.encode_document(&view))
            });
            match round_trip {
                Some(encoded) => {
                    let ids: Vec<u16> = encoded
                        .tokens
                        .iter()
                        .map(|&t| u16::try_from(t).map_err(|_| "token id above u16".to_string()))
                        .collect::<Result<_>>()?;
                    writer.write_tokens(&ids).map_err(|e| e.to_string())?;
                    source_tokens += ids.len();
                    source_response += encoded.response_mask.iter().filter(|&&m| m == 1).count();
                    out_mask.extend(&encoded.response_mask);
                    kept += 1;
                }
                None => dropped += 1,
            }
        }
        file["tokens"] = json!(source_tokens);
        file["response_tokens"] = json!(source_response);
        file["reencode_kept_documents"] = json!(kept);
        file["reencode_dropped_documents"] = json!(dropped);
        if let Some(used) = file["rows_used"].as_u64() {
            file["rows_used"] = json!(used - dropped as u64);
        }
        eprintln!("{}: kept {kept}, dropped {dropped}", file["label"]);
        kept_total += kept;
        dropped_total += dropped;
        new_files.push(file);
    }
    let total = writer.finish().map_err(|e| e.to_string())?;
    let out_mask_path = out.join("response_mask.u8");
    fs::write(&out_mask_path, &out_mask).map_err(|e| e.to_string())?;
    let source = json!({
        "manifest_sha256": sha256_file(&manifest_path)?,
        "tokens_sha256": manifest["tokens_sha256"],
        "protocol": v1.schema,
        "kept_documents": kept_total,
        "dropped_documents": dropped_total,
        "rule": "each protocol 1 document decoded to messages and kept only if re-encoding them in protocol 1 reproduces its tokens and response mask exactly",
    });
    manifest["files"] = json!(new_files);
    manifest["tokens"] = json!(total);
    manifest["response_tokens"] = json!(out_mask.iter().filter(|&&m| m == 1).count());
    manifest["tokens_bytes"] = json!(fs::metadata(&out_tokens).map_err(|e| e.to_string())?.len());
    manifest["mask_bytes"] = json!(out_mask.len());
    manifest["tokens_sha256"] = json!(sha256_file(&out_tokens)?);
    manifest["mask_sha256"] = json!(sha256_file(&out_mask_path)?);
    manifest["rows_used"] = json!(kept_total);
    manifest["dialogue_protocol"] = json!(v2.schema);
    manifest["reencoded_from"] = source;
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!("kept {kept_total} documents, dropped {dropped_total}; {total} tokens");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("reencode-dialogue: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_parse_by_the_protocol_one_template() {
        let e = EOS_MARK;
        let doc = format!("System: be brief\nUser: hi\nAssistant: hello{e}\nUser: and?{e}");
        assert_eq!(
            parse_document(&doc),
            Some(vec![
                ("system".into(), "be brief".into()),
                ("user".into(), "hi".into()),
                ("assistant".into(), "hello".into()),
                ("user".into(), "and?".into()),
            ])
        );
        // Multi-line content stays whole; a final assistant turn needs no
        // terminal EOS beyond its own.
        let doc = format!("User: a\nb\nAssistant: c\nd{e}");
        assert_eq!(
            parse_document(&doc),
            Some(vec![
                ("user".into(), "a\nb".into()),
                ("assistant".into(), "c\nd".into())
            ])
        );
        assert_eq!(parse_document("Hello there"), None);
        assert_eq!(parse_document(&format!("Assistant: open")), None);
    }
}
