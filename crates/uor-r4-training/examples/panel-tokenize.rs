//! Reports how each milestone panel value tokenizes with a byte-level BPE
//! tokenizer. A value is usable as a single-token gold slot only when its
//! leading-space form `" <value>"` encodes to exactly one token.
//!
//! ```text
//! cargo run --release -p uor-r4-training --example panel-tokenize -- TOKENIZER.json
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use uor_r4_tokenizer::ByteBpeTokenizer;

/// The ten values of the development memory panel `dev-mem-01..10`.
const PANEL_VALUES: [&str; 10] = [
    "Alex", "Momo", "green", "teacher", "Tokyo", "two", "July", "blue", "piano", "pizza",
];

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: panel-tokenize TOKENIZER.json");
        return ExitCode::from(2);
    };
    let path = PathBuf::from(path);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("read {}: {error}", path.display());
            return ExitCode::from(2);
        }
    };
    let Some(tokenizer) = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes) else {
        eprintln!("{} is not a supported byte-level BPE", path.display());
        return ExitCode::from(2);
    };
    println!("tokenizer: {}", path.display());
    let mut multi = 0usize;
    for value in PANEL_VALUES {
        let text = format!(" {value}");
        let ids = tokenizer.encode(&text);
        let pieces: Vec<String> = ids
            .iter()
            .map(|&id| format!("{:?}", tokenizer.decode(&[id])))
            .collect();
        let single = ids.len() == 1;
        if !single {
            multi += 1;
        }
        println!(
            "{value:<8} {:>2} token(s)  [{}]  {}",
            ids.len(),
            pieces.join(", "),
            if single { "single" } else { "MULTI" }
        );
    }
    println!(
        "{multi} of {} panel values are multi-token with the leading space",
        PANEL_VALUES.len()
    );
    ExitCode::SUCCESS
}
