#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, error::Error, fs, path::Path};
use uor_r4_discrete::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use uor_r4_discrete::training::DriverConfig;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const CONTEXT: usize = 192;
const TARGETS: usize = 2000;
const CHAT_START: usize = 7_512_797 * 2 + 2_048_204;
const TOKENIZER: &str = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const BASE: &str = include_str!("../../../rung0-completion-release-2026-10-14/seed1-config.json");

struct Corpus {
    bytes: Vec<u8>,
    mask: Vec<u8>,
}

impl Corpus {
    fn read(root: &Path, name: &str, tokens_sha: &str, mask_sha: &str) -> Result<Self> {
        let bytes = FilePin {
            path: root.join(name).join("tokens.u16"),
            sha256: tokens_sha.into(),
        }
        .read()?;
        let mask = FilePin {
            path: root.join(name).join("response_mask.u8"),
            sha256: mask_sha.into(),
        }
        .read()?;
        let corpus = Self { bytes, mask };
        corpus.validate()?;
        Ok(corpus)
    }

    fn validate(&self) -> Result<()> {
        if self.bytes.len() < 64 || &self.bytes[..8] != b"UORT\x01\0\0\0" {
            return Err("invalid UORT version/header".into());
        }
        let count = u64::from_le_bytes(self.bytes[8..16].try_into()?);
        if self.bytes[16..20] != 4096u32.to_le_bytes()
            || self.bytes[20..64].iter().any(|byte| *byte != 0)
            || count.checked_mul(2).and_then(|size| size.checked_add(64))
                != Some(self.bytes.len() as u64)
            || count != self.mask.len() as u64
            || self.mask.is_empty()
            || self.mask.iter().any(|mask| *mask > 1)
            || self.token(0) != 0
            || (0..self.mask.len()).any(|position| self.token(position) >= 4096)
        {
            return Err("invalid UORT payload/mask/vocabulary".into());
        }
        Ok(())
    }

    fn token(&self, position: usize) -> u16 {
        let offset = 64 + 2 * position;
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }

    fn payload(&self, start: usize, end: usize) -> &[u8] {
        &self.bytes[64 + 2 * start..64 + 2 * end]
    }

    fn row_key(&self, document_start: usize, position: usize) -> [u8; 32] {
        let start = document_start.max(position.saturating_sub(CONTEXT));
        let mut hash = Sha256::new();
        hash.update(b"uor-r4.c192-context-plus-target/u16le/1\0");
        hash.update(((position - start) as u64).to_le_bytes());
        hash.update(self.payload(start, position + 1));
        hash.finalize().into()
    }

    fn row_keys(&self) -> HashSet<[u8; 32]> {
        let mut keys = HashSet::new();
        let mut document_start = 0;
        for position in 0..self.mask.len() {
            if self.token(position) == 0 {
                document_start = position;
            } else if self.mask[position] == 1 {
                keys.insert(self.row_key(document_start, position));
            }
        }
        keys
    }

    fn document_keys(&self) -> HashSet<String> {
        let mut starts = (0..self.mask.len())
            .filter(|position| self.token(*position) == 0)
            .collect::<Vec<_>>();
        starts.push(self.mask.len());
        starts
            .windows(2)
            .map(|range| sha256(self.payload(range[0], range[1])))
            .collect()
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn spread_positions(eligible: &[usize]) -> Vec<usize> {
    let count = eligible.len().min(10);
    (0..count)
        .map(|ordinal| eligible[ordinal * eligible.len() / count])
        .collect()
}

fn prepare(root: &Path, out: &Path) -> Result<()> {
    if out.exists() {
        return Err("output must be a new directory; existing receipts are immutable".into());
    }
    let tokenizer = FilePin {
        path: root.join("tokenizer.json"),
        sha256: TOKENIZER.into(),
    }
    .read()?;
    let bos = BosIdentity {
        token: "<|bos|>".into(),
        id: 0,
    };
    bos.validate_tokenizer(&tokenizer, 4096)?;
    let base: DriverConfig = serde_json::from_str(BASE)?;
    let train = Corpus::read(
        root,
        "mw-balp/train",
        &base.train.tokens.sha256,
        &base.train.mask.sha256,
    )?;
    let dev = Corpus::read(
        root,
        "ft/dev",
        &base.dev.tokens.sha256,
        &base.dev.mask.sha256,
    )?;
    let source = Corpus::read(
        root,
        "balp",
        "13261ba81180d8c59086787d80e34cd8ff2a144d6a818018725777d5a8d6bf62",
        "19c1a93f504e4de83af73fe8380c8dde1dec3cb4bef52ba0c7c6148fc73aaf67",
    )?;
    let manifest_bytes = FilePin {
        path: root.join("balp/manifest.json"),
        sha256: "28e6908122865168ad84664039bccadee6932e0bdc9a5cf7a6b52abedb0c912a".into(),
    }
    .read()?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    if manifest["files"][3]["label"] != "chat-v0-p2"
        || manifest["files"][0]["tokens"] != 7_512_797u64
        || manifest["files"][1]["tokens"] != 7_512_797u64
        || manifest["files"][2]["tokens"] != 2_048_204u64
        || source.token(CHAT_START) != 0
    {
        return Err("pinned chat-v0-p2 concatenation boundary differs".into());
    }
    let train_keys = train.row_keys();
    let dev_keys = dev.row_keys();
    let mut forbidden_documents = train.document_keys();
    forbidden_documents.extend(dev.document_keys());
    let mut selected_keys = HashSet::new();
    let mut token_payload = Vec::new();
    let mut mask = Vec::new();
    let mut rows = Vec::new();
    let mut documents = Vec::new();
    let mut duplicate_rows = 0usize;
    let mut rejected_documents = 0usize;
    let mut start = CHAT_START;
    while rows.len() < TARGETS && start < source.mask.len() {
        let end = (start + 1..source.mask.len())
            .find(|position| source.token(*position) == 0)
            .unwrap_or(source.mask.len());
        let document_sha = sha256(source.payload(start, end));
        if forbidden_documents.contains(&document_sha) {
            rejected_documents += 1;
            start = end;
            continue;
        }
        let mut selected = Vec::new();
        let eligible = (start + 1..end)
            .filter(|position| source.mask[*position] == 1)
            .collect::<Vec<_>>();
        for position in spread_positions(&eligible) {
            if rows.len() + selected.len() == TARGETS {
                continue;
            }
            let key = source.row_key(start, position);
            if train_keys.contains(&key) || dev_keys.contains(&key) || !selected_keys.insert(key) {
                duplicate_rows += 1;
                continue;
            }
            selected.push((position, key));
        }
        if !selected.is_empty() {
            let output_start = mask.len();
            token_payload.extend_from_slice(source.payload(start, end));
            mask.resize(output_start + end - start, 0);
            for (position, key) in selected {
                let output_position = output_start + position - start;
                mask[output_position] = 1;
                rows.push(json!({"source_target_position":position,"output_target_position":output_position,
                    "source_document_start":start,"target_token":source.token(position),
                    "causal_row_sha256":key.iter().map(|byte| format!("{byte:02x}")).collect::<String>()}));
            }
            documents.push(json!({"source_start":start,"source_end_exclusive":end,"output_start":output_start,"tokens_sha256":document_sha}));
        }
        start = end;
    }
    if rows.len() != TARGETS {
        return Err("not enough unique held-out rows".into());
    }
    let mut tokens = vec![0u8; 64];
    tokens[..8].copy_from_slice(b"UORT\x01\0\0\0");
    tokens[8..16].copy_from_slice(&(mask.len() as u64).to_le_bytes());
    tokens[16..20].copy_from_slice(&4096u32.to_le_bytes());
    tokens.extend(token_payload);
    let fresh = CorpusSource {
        split: Split::Fresh,
        tokens: FilePin {
            path: "fresh/tokens.u16".into(),
            sha256: sha256(&tokens),
        },
        mask: FilePin {
            path: "fresh/response_mask.u8".into(),
            sha256: sha256(&mask),
        },
        expected_payload_tokens: mask.len() as u64,
        expected_eligible_targets: TARGETS as u64,
    };
    let prepared = PreparedCorpus::from_bytes(fresh.clone(), &tokens, &mask, 4096, bos)?;
    for first in (0..TARGETS).step_by(64) {
        let batch = prepared.batch(first as u64, 192, 64)?;
        for (offset, position) in batch.target_positions.iter().enumerate() {
            let mut hash = Sha256::new();
            hash.update(b"uor-r4.c192-context-plus-target/u16le/1\0");
            hash.update((batch.windows[offset].len() as u64).to_le_bytes());
            for identity in &batch.windows[offset] {
                hash.update((identity.token as u16).to_le_bytes());
            }
            hash.update((batch.targets[offset] as u16).to_le_bytes());
            if rows[first + offset]["output_target_position"] != *position
                || rows[first + offset]["target_token"] != batch.targets[offset]
                || rows[first + offset]["causal_row_sha256"] != format!("{:x}", hash.finalize())
            {
                return Err("PreparedCorpus row correspondence differs".into());
            }
        }
    }
    fs::create_dir_all(out.join("fresh"))?;
    fs::create_dir(out.join("configs"))?;
    fs::write(out.join("fresh/tokens.u16"), &tokens)?;
    fs::write(out.join("fresh/response_mask.u8"), &mask)?;
    write_json(&out.join("fresh/rows.json"), &json!(rows))?;
    write_json(&out.join("fresh/documents.json"), &json!(documents))?;
    let mut configs = Vec::new();
    for seed in [1, 2] {
        for (arm, policy) in [
            ("A", "nearest_address_margin_v1"),
            ("R", "occupied_cell_rank_hinge_v1"),
        ] {
            let mut config: Value = serde_json::from_str(BASE)?;
            config["seed"] = json!(seed);
            config["spec"]["kept_cells"] = json!(16);
            config["objective_policy"] = json!(policy);
            let parsed: DriverConfig = serde_json::from_value(config.clone())?;
            parsed.validate()?;
            let name = format!("k16-{arm}-seed{seed}.json");
            write_json(&out.join("configs").join(&name), &config)?;
            configs.push(
                json!({"file":name,"sha256":sha256(&fs::read(out.join("configs").join(name))?)}),
            );
        }
    }
    write_json(
        &out.join("binding.json"),
        &json!({
            "schema":"uor-r4.admission-redirect-packet-data/1",
            "status":"DATA_BOUND_NOT_MODEL_EVALUATED",
            "fresh":prepared.binding(), "context":CONTEXT,
            "source":{"hf_dataset":"caseyallard/uor-r4-store","revision":"7e0a52a70165d01ee5f37d1b28740f53c49ca3f2",
                "archive":"data/ft-balp.tar","archive_sha256":"b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f",
                "member":"balp/tokens.u16","tokens_sha256":sha256(&source.bytes),"mask_sha256":sha256(&source.mask),
                "manifest_sha256":sha256(&manifest_bytes),"component":"chat-v0-p2","component_payload_start":CHAT_START},
        "selection":"ascending complete BOS documents in chat-v0-p2; at most 10 response positions per document at floor(i*n/min(10,n)), i=0..min(10,n)-1 in eligible source order; retain first 2000 unique C192 context+target keys absent from all masked train/dev keys; exclude whole-document duplicates; other mask bits zero",
            "novelty_scope":"held out from this D32 mw-balp/train and ft/dev contrast; historically used by other pre-D32 models, not globally unseen data; different chat-source distribution, not a matched M-world generator draw",
            "disjointness":{"train_tokens_sha256":base.train.tokens.sha256,"train_mask_sha256":base.train.mask.sha256,
                "dev_tokens_sha256":base.dev.tokens.sha256,"dev_mask_sha256":base.dev.mask.sha256,
                "train_unique_row_keys":train_keys.len(),"dev_unique_row_keys":dev_keys.len(),
                "row_key_policy":"SHA256(domain || u64le(context_length) || u16le(context_tokens_plus_target)); BOS-local prefix, max192",
                "selected_duplicate_row_keys":0,"train_overlap":0,"dev_overlap":0,
                "excluded_overlapping_or_duplicate_rows":duplicate_rows,"excluded_duplicate_documents":rejected_documents},
            "rows_sha256":sha256(&fs::read(out.join("fresh/rows.json"))?),
            "documents_sha256":sha256(&fs::read(out.join("fresh/documents.json"))?),
            "configs":configs,"tokenizer_sha256":TOKENIZER,
        "source_sha256":sha256(include_bytes!("main.rs")),"binary_sha256":sha256(&fs::read(std::env::current_exe()?)?),
        "driver_source_sha256":uor_r4_discrete::training::driver_source_sha256(),
            "model_operations":0,"corpus_apply_calls":0
        }),
    )?;
    println!("{}", out.join("binding.json").display());
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [root, out] => prepare(Path::new(root), Path::new(out)),
        _ => Err(
            "usage: admission-redirect-packet EXTRACTED_INPUT_ROOT NEW_OUTPUT_DIR (data only)"
                .into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(tokens: &[u16], mask: &[u8]) -> Corpus {
        let mut bytes = vec![0; 64];
        bytes[..8].copy_from_slice(b"UORT\x01\0\0\0");
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&4096u32.to_le_bytes());
        bytes.extend(tokens.iter().flat_map(|token| token.to_le_bytes()));
        Corpus {
            bytes,
            mask: mask.to_vec(),
        }
    }

    #[test]
    fn rejects_bad_mask_and_vocabulary() {
        assert!(fixture(&[0, 4096], &[0, 1]).validate().is_err());
        assert!(fixture(&[0, 2], &[0, 2]).validate().is_err());
        assert!(fixture(&[0, 2], &[0]).validate().is_err());
        assert!(fixture(&[0, 2], &[0, 1]).validate().is_ok());
    }

    #[test]
    fn content_key_ignores_split_offsets_but_binds_target_and_context() {
        let left = fixture(&[0, 5, 6], &[0, 0, 1]);
        let right = fixture(&[0, 2, 0, 5, 6, 7], &[0, 0, 0, 0, 1, 1]);
        assert_eq!(left.row_key(0, 2), right.row_key(2, 4));
        assert_ne!(left.row_key(0, 2), right.row_key(2, 5));
        assert_eq!(left.row_keys().intersection(&right.row_keys()).count(), 1);
    }

    #[test]
    fn c192_discards_only_prefix_outside_window() {
        let mut tokens = vec![0];
        tokens.extend(vec![9; 194]);
        let left = fixture(&tokens, &vec![1; tokens.len()]);
        tokens[1] = 8;
        let right = fixture(&tokens, &vec![1; tokens.len()]);
        assert_eq!(left.row_key(0, 194), right.row_key(0, 194));
        assert_ne!(left.row_key(0, 192), right.row_key(0, 192));
    }

    #[test]
    fn response_positions_are_spread_and_capped_per_document() {
        assert!(spread_positions(&[]).is_empty());
        assert_eq!(spread_positions(&[3, 5, 7]), vec![3, 5, 7]);
        assert_eq!(
            spread_positions(&(0..20).collect::<Vec<_>>()),
            (0..20).step_by(2).collect::<Vec<_>>()
        );
    }
}
