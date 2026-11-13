//! place-asks (claude-359): fit the asks on a split by counting, and count what they find.
//!
//! ```text
//! place-asks fit    --tokens TRAIN/tokens.u16 --kit train-places.json --out asks.json
//! place-asks census --tokens PANEL/tokens.u16 --kit panel-places.json --asks asks.json [--show N]
//! ```
//!
//! A token file is the `UORT` format the trainer reads: a 64-byte header holding the token count
//! and the vocabulary size, then the tokens as little-endian 16-bit integers. A kit is the place
//! kit the turn census writes: the marks and each recall reply's `[reply start, place, value
//! start, value length]`. Nothing here reads a model; every number is a count over tokens.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use uor_r4_integer::fbs_snapshot::PlaceMarks;
use uor_r4_integer::place_asks::{census, fit, AsksIndex, PlaceAsks};

const HEADER: usize = 64;

#[derive(Deserialize)]
struct Kit {
    tokens_sha256: String,
    marks: PlaceMarks,
    places: Vec<[u32; 4]>,
}

/// A token file's tokens, its vocabulary size and the SHA-256 of its bytes; `path` names it in
/// an error.
fn parse_tokens(bytes: &[u8], path: &str) -> Result<(Vec<u32>, u32, String), String> {
    if bytes.len() < HEADER || &bytes[..4] != b"UORT" {
        return Err(format!("{path}: not a UORT token file"));
    }
    let word = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"));
    let (count, vocab) = (word(8), word(16));
    let payload = &bytes[HEADER..];
    if count.checked_mul(2) != Some(payload.len() as u64) {
        return Err(format!("{path}: the header counts {count} tokens and the file holds {} bytes of them", payload.len()));
    }
    let tokens = payload.chunks_exact(2).map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]]))).collect();
    let vocab = u32::try_from(vocab).map_err(|_| format!("{path}: a vocabulary of {vocab}"))?;
    Ok((tokens, vocab, hex::encode(Sha256::digest(bytes))))
}

/// The kit's places are positions in one token file: another file's counts would be printed
/// under this one's name.
fn same_tokens(kit_sha256: &str, tokens_sha256: &str) -> Result<(), String> {
    if kit_sha256 == tokens_sha256 {
        return Ok(());
    }
    Err(format!("the kit was written from tokens {kit_sha256} and --tokens holds {tokens_sha256}"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn flag(args: &[String], name: &str) -> Result<PathBuf, String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(PathBuf::from).ok_or_else(|| format!("{name} PATH is required"))
}

fn run(args: &[String]) -> Result<(), String> {
    let mode = args.first().map(String::as_str).ok_or("usage: place-asks fit|census ...")?;
    let path = flag(args, "--tokens")?;
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (tokens, vocab, tokens_sha256) = parse_tokens(&bytes, &path.display().to_string())?;
    let kit: Kit = read_json(&flag(args, "--kit")?)?;
    same_tokens(&kit.tokens_sha256, &tokens_sha256)?;
    match mode {
        "fit" => {
            let out = flag(args, "--out")?;
            if out.exists() {
                return Err(format!("{}: exists; a fit never writes over a table", out.display()));
            }
            let (asks, lines) = fit(&tokens, &kit.marks, vocab, &kit.places, kit.tokens_sha256.clone())?;
            for line in &lines {
                println!("{line}");
            }
            let index = AsksIndex::new(&asks)?;
            let (counts, _) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, 0);
            for line in counts.lines("fitted") {
                println!("{line}");
            }
            let text = serde_json::to_string(&asks).map_err(|e| e.to_string())?;
            // The file is made here and only if no file is there: a table written since the
            // check above is not written over.
            let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&out).map_err(|e| format!("{}: {e}; a fit never writes over a table", out.display()))?;
            file.write_all(text.as_bytes()).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("asks-fit wrote {} fitted_on={}", out.display(), asks.tokens_sha256);
        }
        "census" => {
            let asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            let show = match args.iter().position(|a| a == "--show") {
                Some(i) => args.get(i + 1).and_then(|n| n.parse().ok()).ok_or("--show N")?,
                None => 0,
            };
            let index = AsksIndex::new(&asks)?;
            let (counts, missed) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, show);
            println!(
                "asks-census tokens={} fitted_on={} margin={} floor={} floor_other={} asking_floor={}",
                kit.tokens_sha256, asks.tokens_sha256, asks.margin, asks.floor, asks.floor_other, asks.asking_floor
            );
            for line in counts.lines("held") {
                println!("{line}");
            }
            for line in missed {
                println!("{line}");
            }
        }
        other => return Err(format!("unknown mode {other}: fit or census")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("place-asks: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A token file of `ids` whose header counts `count` tokens in a vocabulary of 9.
    fn file(count: u64, ids: &[u16]) -> Vec<u8> {
        let mut bytes = vec![0u8; HEADER];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[8..16].copy_from_slice(&count.to_le_bytes());
        bytes[16..24].copy_from_slice(&9u64.to_le_bytes());
        bytes.extend(ids.iter().flat_map(|id| id.to_le_bytes()));
        bytes
    }

    #[test]
    fn a_token_file_is_read_with_the_hash_of_its_own_bytes() {
        let bytes = file(3, &[4, 5, 6]);
        let (tokens, vocab, sha256) = parse_tokens(&bytes, "t").expect("read");
        assert_eq!((tokens, vocab), (vec![4, 5, 6], 9));
        assert_eq!(sha256, hex::encode(Sha256::digest(&bytes)));
        // Another file's kit is refused, and this file's is taken.
        assert!(same_tokens(&sha256, &sha256).is_ok());
        assert!(same_tokens(&hex::encode(Sha256::digest(file(3, &[4, 5, 7]))), &sha256).is_err());
    }

    #[test]
    fn a_malformed_token_file_is_an_error_and_never_a_panic() {
        // A count that does not fit twice in 64 bits, a count that is not the file's, a file
        // shorter than its header, and a file that is not a token file.
        assert!(parse_tokens(&file(u64::MAX, &[4, 5, 6]), "t").is_err());
        assert!(parse_tokens(&file(1 << 63, &[]), "t").is_err());
        assert!(parse_tokens(&file(2, &[4, 5, 6]), "t").is_err());
        assert!(parse_tokens(&file(3, &[4, 5, 6])[..40], "t").is_err());
        let mut other = file(3, &[4, 5, 6]);
        other[0] = b'X';
        assert!(parse_tokens(&other, "t").is_err());
    }
}
