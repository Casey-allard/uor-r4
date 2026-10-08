//! Exact byte accounting for a u16 token stream under a declared tokenizer.
//!
//! ```text
//! token-stream-bytes <tokens.u16> <tokenizer.json> [--skip-header 64]
//! ```
//!
//! Sealed evaluations record nats per token and a token-stream path but leave `bits_per_byte`
//! null, so the byte denominator has to be recoverable exactly. It is: each token's byte string
//! is fixed by the tokenizer, so the stream's byte total is the sum of the per-token lengths.
//! The default skips the 64-byte UORT header these streams carry.

use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.len() < 2 {
        eprintln!("usage: token-stream-bytes <tokens.u16> <tokenizer.json> [--skip-header 64]");
        std::process::exit(1);
    }
    let skip = argv
        .iter()
        .position(|a| a == "--skip-header")
        .and_then(|i| argv.get(i + 1))
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(64);
    let raw = match std::fs::read(&argv[0]) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("read {}: {e}", argv[0]);
            std::process::exit(1);
        }
    };
    let bytes = match std::fs::read(&argv[1]) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("read {}: {e}", argv[1]);
            std::process::exit(1);
        }
    };
    let tokenizer = match HfBpeTokenizer::from_tokenizer_json_bytes(&bytes) {
        Some(tokenizer) => tokenizer,
        None => {
            eprintln!("failed to parse {}", argv[1]);
            std::process::exit(1);
        }
    };
    let lens = tokenizer.token_byte_lengths();
    let payload = if raw.len() > skip {
        &raw[skip..]
    } else {
        &raw[..]
    };
    let mut total = 0u64;
    let mut out_of_vocab = 0u64;
    let mut max_id = 0u16;
    let mut tokens = 0u64;
    for chunk in payload.chunks_exact(2) {
        let id = u16::from_le_bytes([chunk[0], chunk[1]]);
        tokens += 1;
        max_id = max_id.max(id);
        match lens.get(id as usize) {
            Some(len) => total += *len as u64,
            None => out_of_vocab += 1,
        }
    }
    // With --eval, convert a sealed evaluation's nats-per-token into bits per byte using this
    // stream's exact byte total, so `bits_per_byte: null` in the record can be filled in.
    if let Some(index) = argv.iter().position(|a| a == "--eval") {
        if let Some(path) = argv.get(index + 1) {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            let value: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let nll = value["full"]["nll"].as_f64().unwrap_or(f64::NAN);
            let targets = value["full"]["targets"].as_f64().unwrap_or(f64::NAN);
            let model = value["model"]["path"].as_str().unwrap_or("?");
            let bytes_per_token = total as f64 / tokens.max(1) as f64;
            let bits_per_token = nll / std::f64::consts::LN_2;
            let bpb = bits_per_token / bytes_per_token;
            println!(
                "{{\"eval\":\"{}\",\"model\":\"{}\",\"nll\":{:.7},\"targets\":{:.0},\"bits_per_token\":{:.6},\"bytes_per_token\":{:.6},\"bits_per_byte\":{:.6}}}",
                path, model, nll, targets, bits_per_token, bytes_per_token, bpb
            );
            return;
        }
    }
    println!(
        "{{\"path\":\"{}\",\"tokens\":{},\"bytes\":{},\"bytes_per_token\":{:.6},\"out_of_vocab\":{},\"max_id\":{}}}",
        argv[0],
        tokens,
        total,
        total as f64 / tokens.max(1) as f64,
        out_of_vocab,
        max_id
    );
}
