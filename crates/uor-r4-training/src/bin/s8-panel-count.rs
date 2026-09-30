//! Count-only length check of the sealed §8 panel (D18 section 9). The owner
//! runs it; a lab never runs it on the panel.
//!
//! ```text
//! s8-panel-count panel=PANEL.json tokenizer=TOKENIZER.json
//! ```
//!
//! The tokenizer must be #1017's. The output is one JSON object of token
//! counts and identities, with no panel text: see
//! [`uor_r4_training::s8_panel_count`].

#![forbid(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use sha2::{Digest, Sha256};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::s8_panel_count::{count_panel, REPLY_BUDGET};

/// The #1017 tokenizer, bound by the chat-v0 corpus and every stack artifact
/// trained on it.
const TOKENIZER_1017_CID: &str =
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc";

const USAGE: &str = "usage: s8-panel-count panel=PANEL.json tokenizer=TOKENIZER.json";

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("s8-panel-count: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut panel = None;
    let mut tokenizer = None;
    for argument in std::env::args().skip(1) {
        match argument.split_once('=') {
            Some(("panel", path)) if panel.is_none() => panel = Some(PathBuf::from(path)),
            Some(("tokenizer", path)) if tokenizer.is_none() => {
                tokenizer = Some(PathBuf::from(path))
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    let (Some(panel), Some(tokenizer)) = (panel, tokenizer) else {
        return Err(USAGE.to_owned());
    };
    let tokenizer_bytes = fs::read(&tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer: not a supported byte-level BPE tokenizer.json")?;
    let cid = tokenizer.address();
    if cid != TOKENIZER_1017_CID {
        return Err(format!(
            "tokenizer: {cid} is not the #1017 tokenizer ({TOKENIZER_1017_CID})"
        ));
    }
    let protocol =
        DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| format!("protocol: {e}"))?;
    let identity = protocol.identity().map_err(|e| format!("protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("protocol: {e}"))?;
    let bytes = fs::read(&panel).map_err(|e| format!("panel: {e}"))?;
    let digest = hex::encode(Sha256::digest(&bytes));
    let counts = count_panel(&encoder, &bytes, REPLY_BUDGET).map_err(|e| e.to_string())?;
    Ok(counts.report(&digest, &cid, &identity).to_string())
}
