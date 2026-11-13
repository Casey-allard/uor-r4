//! place-asks (claude-359): fit the asks on a split by counting, and count what they find.
//!
//! ```text
//! place-asks fit    --tokens TRAIN/tokens.u16 --kit train-places.json --out asks.json
//! place-asks census --tokens PANEL/tokens.u16 --kit panel-places.json --asks asks.json [--show N]
//! place-asks ablate --tokens TRAIN/tokens.u16 --kit train-places.json \
//!                   --held PANEL/tokens.u16 --held-kit panel-places.json [--held .. --held-kit ..] \
//!                   [--slots 1,2,3|none] [--question-last N]
//! ```
//!
//! `ablate` (claude-360) fits under a cut, which keeps the named frame offsets (1 to 5 before a
//! place, nearest first; 6 to 8 the turn's last tokens, the last first) and the last N tokens
//! of a question, and prints the census of each held split under the same cut. It writes no
//! table: a table file never carries a cut.
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
use uor_r4_integer::place_asks::{census, fit, fit_cut, AsksIndex, Cut, PlaceAsks, ASKS_FRAME};

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

/// Every value of a flag that may be given more than once, in order.
fn flags(args: &[String], name: &str) -> Vec<PathBuf> {
    args.windows(2).filter(|pair| pair[0] == name).map(|pair| PathBuf::from(&pair[1])).collect()
}

/// A split: its tokens, its vocabulary size and the kit written from those tokens.
fn split(tokens: &Path, kit: &Path) -> Result<(Vec<u32>, u32, Kit), String> {
    let bytes = std::fs::read(tokens).map_err(|e| format!("{}: {e}", tokens.display()))?;
    let (tokens, vocab, tokens_sha256) = parse_tokens(&bytes, &tokens.display().to_string())?;
    let kit: Kit = read_json(kit)?;
    same_tokens(&kit.tokens_sha256, &tokens_sha256)?;
    Ok((tokens, vocab, kit))
}

/// The cut of `--slots` and `--question-last`: every offset and the whole question when a flag
/// is not given.
fn cut(args: &[String]) -> Result<Cut, String> {
    let value = |name: &str| args.iter().position(|a| a == name).map(|i| args.get(i + 1).map(String::as_str).ok_or_else(|| format!("{name} takes a value")));
    let mut cut = Cut::ALL;
    if let Some(text) = value("--slots") {
        let text = text?;
        cut.slots = 0;
        // `none` is the whole value or an error: `none,6` would run a cut nobody named.
        for part in text.split(',').filter(|_| text != "none") {
            let offset: usize = part.parse().ok().filter(|n| (1..=ASKS_FRAME).contains(n)).ok_or_else(|| format!("--slots {part}: an offset is 1 to {ASKS_FRAME}, or the one word none"))?;
            let bit = 1 << (offset - 1);
            if cut.slots & bit != 0 {
                return Err(format!("--slots names offset {offset} twice"));
            }
            cut.slots |= bit;
        }
    }
    if let Some(text) = value("--question-last") {
        cut.question_last = text?.parse().map_err(|_| "--question-last N".to_owned())?;
    }
    Ok(cut)
}

/// The held splits of `ablate`: each `--held` with the `--held-kit` at the same place.
fn held(args: &[String]) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let (tokens, kits) = (flags(args, "--held"), flags(args, "--held-kit"));
    if tokens.is_empty() || tokens.len() != kits.len() {
        return Err("ablate takes one --held-kit for each --held, and at least one".to_owned());
    }
    Ok(tokens.into_iter().zip(kits).collect())
}

/// A held split is counted with the fitted split's marks and vocabulary, or not at all.
fn same_marks(fitted: (&PlaceMarks, u32), held: (&PlaceMarks, u32), kit: &Path) -> Result<(), String> {
    if fitted == held {
        return Ok(());
    }
    Err(format!("{}: other marks or another vocabulary than the fitted split's", kit.display()))
}

/// The offsets a cut keeps, as `--slots` takes them.
fn slots(cut: Cut) -> String {
    let kept: Vec<String> = (1..=ASKS_FRAME).filter(|j| cut.slots & (1 << (j - 1)) != 0).map(|j| j.to_string()).collect();
    if kept.is_empty() {
        "none".to_owned()
    } else {
        kept.join(",")
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mode = args.first().map(String::as_str).ok_or("usage: place-asks fit|census|ablate ...")?;
    let (tokens, vocab, kit) = split(&flag(args, "--tokens")?, &flag(args, "--kit")?)?;
    match mode {
        "ablate" => {
            let cut = cut(args)?;
            let held = held(args)?;
            let (asks, _) = fit_cut(&tokens, &kit.marks, vocab, &kit.places, kit.tokens_sha256.clone(), cut)?;
            let index = AsksIndex::new(&asks)?.with_cut(cut);
            println!(
                "asks-ablate slots={} question_last={} fitted_on={} shared={} pairs={} margin={} floor={} floor_other={} asking_floor={}",
                slots(cut),
                cut.question_last,
                asks.tokens_sha256,
                asks.shared.len(),
                asks.pairs.len(),
                asks.margin,
                asks.floor,
                asks.floor_other,
                asks.asking_floor
            );
            for (n, (tokens, kit_path)) in held.iter().enumerate() {
                let (tokens, held_vocab, held_kit) = split(tokens, kit_path)?;
                same_marks((&kit.marks, vocab), (&held_kit.marks, held_vocab), kit_path)?;
                let (counts, _) = census(&tokens, &held_kit.marks, &held_kit.places, &index, &asks.forms, 0);
                for line in counts.lines(&format!("held{}", n + 1)) {
                    println!("{line}");
                }
            }
        }
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
        other => return Err(format!("unknown mode {other}: fit, census or ablate")),
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

    #[test]
    fn a_cut_is_read_from_its_flags_and_written_back_the_same() {
        let args = |text: &str| text.split(' ').map(str::to_owned).collect::<Vec<String>>();
        assert_eq!(cut(&args("ablate")), Ok(Cut::ALL));
        assert_eq!(cut(&args("ablate --slots 1,6,8 --question-last 2")), Ok(Cut { slots: 0b1010_0001, question_last: 2 }));
        assert_eq!(cut(&args("ablate --slots none")), Ok(Cut { slots: 0, question_last: 0 }));
        assert_eq!(slots(Cut { slots: 0b1010_0001, question_last: 0 }), "1,6,8");
        assert_eq!(slots(Cut { slots: 0, question_last: 0 }), "none");
        assert_eq!(slots(Cut::ALL), "1,2,3,4,5,6,7,8");
        for refused in ["ablate --slots 0", "ablate --slots 9", "ablate --slots 1,x", "ablate --slots", "ablate --slots none,6", "ablate --slots 1,1", "ablate --question-last x", "ablate --question-last"] {
            assert!(cut(&args(refused)).is_err(), "{refused}");
        }
        assert_eq!(flags(&args("ablate --held a --held-kit k --held b"), "--held"), vec![PathBuf::from("a"), PathBuf::from("b")]);
    }

    #[test]
    fn the_held_splits_come_in_pairs_with_the_marks_of_the_fitted_split() {
        let args = |text: &str| text.split(' ').map(str::to_owned).collect::<Vec<String>>();
        let pair = |tokens: &str, kit: &str| (PathBuf::from(tokens), PathBuf::from(kit));
        assert_eq!(held(&args("ablate --held a --held-kit ka --held b --held-kit kb")), Ok(vec![pair("a", "ka"), pair("b", "kb")]));
        assert!(held(&args("ablate")).is_err());
        assert!(held(&args("ablate --held a --held-kit ka --held b")).is_err());
        let marks = |user: u32| -> PlaceMarks { serde_json::from_str(&format!(r#"{{"bos":0,"eos":1,"sep":[2],"system":[3],"user":[{user}],"assistant":[5]}}"#)).expect("marks") };
        let kit = Path::new("kit.json");
        assert_eq!(same_marks((&marks(4), 9), (&marks(4), 9), kit), Ok(()));
        assert!(same_marks((&marks(4), 9), (&marks(6), 9), kit).is_err());
        assert!(same_marks((&marks(4), 9), (&marks(4), 10), kit).is_err());
    }

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
