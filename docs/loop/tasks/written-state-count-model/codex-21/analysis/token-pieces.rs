//! Display original vocabulary IDs using the tokenizer bound to the run.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{error::Error, io};
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [path, pin] = args.as_slice() else {
        return Err("clause-veto-token-pieces TOKENIZER SHA256".into());
    };
    let bytes = std::fs::read(path)?;
    if format!("{:x}", Sha256::digest(&bytes)) != *pin {
        return Err("tokenizer differs from pinned run input".into());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("bad tokenizer")?;
    let tokens: Vec<_> = (0..tokenizer.vocab_size() as u32)
        .map(|id| {
            let bytes = tokenizer.decode_bytes(&[id]);
            json!({ "id": id, "piece": String::from_utf8_lossy(&bytes), "bytes": bytes })
        })
        .collect();
    serde_json::to_writer(io::stdout().lock(), &tokens)?;
    Ok(())
}
