//! Freeze the position sheet's order-sensitive DEV slice (`uor_r4_discrete::order_slice`).
//! Host-only: token-identity selection over the pinned DEV split; no model execution.
//!
//! Usage: order-slice-freeze TOKENS MASK OUTPUT
//! TOKENS and MASK are `ft/dev/tokens.u16` and `ft/dev/response_mask.u8` from `data/ft-dev.tar`;
//! they load only under the registered DEV digests. OUTPUT must not exist; it is created
//! exclusively.

use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;
use uor_r4_discrete::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use uor_r4_discrete::order_slice::{render, select, CONTEXT, SELECTION_POLICY};

const DEV_TOKENS_SHA256: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const DEV_MASK_SHA256: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";

fn run(args: &[String]) -> Result<(), String> {
    let [tokens, mask, output] = args else {
        return Err("usage: order-slice-freeze TOKENS MASK OUTPUT".into());
    };
    let source = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: tokens.into(),
            sha256: DEV_TOKENS_SHA256.into(),
        },
        mask: FilePin {
            path: mask.into(),
            sha256: DEV_MASK_SHA256.into(),
        },
        expected_payload_tokens: 42_856,
        expected_eligible_targets: 14_287,
    };
    let bos = BosIdentity {
        token: "<|bos|>".into(),
        id: 0,
    };
    let corpus = PreparedCorpus::load(&source, 4096, &bos).map_err(|e| e.to_string())?;
    let slice = select(&corpus).map_err(|e| e.to_string())?;
    let text = render(&slice);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("{output}: {e}"))?;
    file.write_all(text.as_bytes())
        .map_err(|e| format!("{output}: {e}"))?;
    println!("policy {SELECTION_POLICY}");
    println!("context {CONTEXT}");
    println!("scanned {}", slice.scanned);
    println!("selected {}", slice.rows.len());
    println!("loose_diagnostic {}", slice.loose);
    println!("bytes {}", text.len());
    println!("sha256 {}", sha256(text.as_bytes()));
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("order-slice-freeze: {e}");
            ExitCode::FAILURE
        }
    }
}
