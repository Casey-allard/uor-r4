//! EOS exposure of a token split under the trainer's scoring rule (claude-310).
//!
//! `corpus-eos-exposure --tokens T --mask M [--manifest J] --eos 1 --bos 0 --out OUT` reads a
//! `tokens.u16` (a 64-byte `UORT` header and little-endian u16 payload, or a bare payload) and its
//! `response_mask.u8` (one 0/1 byte per token), and counts which targets the trainer scores.
//!
//! The rule is `build_document_windows` in the trainer crate's `src/hamilton_sheet.rs` at
//! ff5156bcc (lines 1867-1905): documents start at each BOS; position `p` is a scored target, its
//! target `tokens[p]` read from the document's tokens before `p`, when `p` lies after a document's
//! BOS, `tokens[p] != bos` and `mask[p] == 1`. Tokens before the first BOS are in no document.
//!
//! A reply is the run of scored targets after the previous scored EOS of the same document (or
//! after its BOS) up to and including a scored EOS; scored targets after a document's last scored
//! EOS are counted as unterminated. With a manifest, roots are contiguous in file order, their
//! `tokens_taken` must sum to the token count, each non-empty root must start at a BOS, and each
//! root's `scored_taken` must equal the trainer-scored count.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "uor-r4.corpus-eos-exposure/1";
const CONVENTION: &str = "trainer-target-at-p: scored iff p after a document BOS, tokens[p] != bos, mask[p] == 1; target tokens[p]";
const CONVENTION_SOURCE: &str = "trainer crate src/hamilton_sheet.rs:1903 at ff5156bcc";
const UORT_HEADER_BYTES: usize = 64;

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("corpus-eos-exposure: {error}");
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Manifest {
    roots: Vec<ManifestRoot>,
}

#[derive(Deserialize)]
struct ManifestRoot {
    name: String,
    tokens_taken: usize,
    scored_taken: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
struct Counts {
    positions: usize,
    scored_targets: usize,
    mask_ones: usize,
    eos_tokens: usize,
    eos_scored: usize,
    bos_tokens: usize,
    /// EOS among scored targets, parts per million of scored targets.
    eos_scored_ppm: u64,
    replies: usize,
    reply_scored_total: usize,
    /// Lower median of reply lengths in scored targets; 0 with no replies.
    reply_median: usize,
    reply_max: usize,
    unterminated_scored: usize,
}

#[derive(Debug, Serialize)]
struct RootCounts {
    name: String,
    tokens_taken: usize,
    scored_taken: usize,
    scored_taken_matches: &'static str,
    counts: Counts,
}

#[derive(Default)]
struct Tally {
    counts: Counts,
    reply_lengths: Vec<usize>,
}

impl Tally {
    fn finish(mut self) -> Counts {
        self.reply_lengths.sort_unstable();
        let n = self.reply_lengths.len();
        self.counts.replies = n;
        self.counts.reply_scored_total = self.reply_lengths.iter().sum();
        self.counts.reply_median = if n == 0 { 0 } else { self.reply_lengths[(n - 1) / 2] };
        self.counts.reply_max = self.reply_lengths.last().copied().unwrap_or(0);
        self.counts.eos_scored_ppm = if self.counts.scored_targets == 0 {
            0
        } else {
            self.counts.eos_scored as u64 * 1_000_000 / self.counts.scored_targets as u64
        };
        self.counts
    }
}

/// The trainer's per-position scored flag (hamilton_sheet.rs:1903 at ff5156bcc).
fn scored_flags(tokens: &[u16], mask: &[u8], bos: u16) -> Vec<bool> {
    let mut in_document = false;
    let mut flags = Vec::with_capacity(tokens.len());
    for (p, &token) in tokens.iter().enumerate() {
        if token == bos {
            in_document = true;
        }
        flags.push(in_document && token != bos && mask[p] == 1);
    }
    flags
}

/// Counts over `tokens`, split at `root_lengths` (one root covering everything when empty).
fn count(tokens: &[u16], mask: &[u8], root_lengths: &[usize], eos: u16, bos: u16) -> (Vec<Counts>, Counts) {
    let flags = scored_flags(tokens, mask, bos);
    let mut ends = Vec::new();
    let mut end = 0;
    for &len in root_lengths {
        end += len;
        ends.push(end);
    }
    let mut roots: Vec<Tally> = (0..ends.len()).map(|_| Tally::default()).collect();
    let mut total = Tally::default();
    let mut root = 0;
    let mut run = 0;
    let mut run_root = 0;
    for (p, (&token, &scored)) in tokens.iter().zip(&flags).enumerate() {
        while root < ends.len() && p >= ends[root] {
            root += 1;
        }
        if token == bos && run > 0 {
            total.counts.unterminated_scored += run;
            if let Some(tally) = roots.get_mut(run_root) {
                tally.counts.unterminated_scored += run;
            }
            run = 0;
        }
        if scored {
            run += 1;
            run_root = root;
        }
        let reply = (scored && token == eos).then_some(run);
        if reply.is_some() {
            run = 0;
        }
        for tally in std::iter::once(&mut total).chain(roots.get_mut(root)) {
            let c = &mut tally.counts;
            c.positions += 1;
            c.scored_targets += usize::from(scored);
            c.mask_ones += usize::from(mask[p] == 1);
            c.eos_tokens += usize::from(token == eos);
            c.eos_scored += usize::from(scored && token == eos);
            c.bos_tokens += usize::from(token == bos);
            if let Some(len) = reply {
                tally.reply_lengths.push(len);
            }
        }
    }
    if run > 0 {
        total.counts.unterminated_scored += run;
        if let Some(tally) = roots.get_mut(run_root) {
            tally.counts.unterminated_scored += run;
        }
    }
    (roots.into_iter().map(Tally::finish).collect(), total.finish())
}

/// Decodes a `tokens.u16` against its mask: a `UORT` header whose count is the mask length, or a
/// bare payload of two bytes per mask byte. Returns the tokens and the format name.
fn decode_tokens(bytes: &[u8], mask: &[u8]) -> Result<(Vec<u16>, &'static str)> {
    let (payload, format) = if bytes.len() >= UORT_HEADER_BYTES && &bytes[..4] == b"UORT" {
        let mut count = [0u8; 8];
        count.copy_from_slice(&bytes[8..16]);
        let count = u64::from_le_bytes(count);
        if count != mask.len() as u64 {
            return Err(format!("the UORT count {count} differs from the mask length {}", mask.len()));
        }
        (&bytes[UORT_HEADER_BYTES..], "uort")
    } else {
        (bytes, "raw-u16le")
    };
    if payload.len() != mask.len() * 2 {
        return Err(format!(
            "the token payload holds {} bytes, the mask {} tokens",
            payload.len(),
            mask.len()
        ));
    }
    if let Some(p) = mask.iter().position(|&m| m > 1) {
        return Err(format!("mask byte {} at position {p} is not 0 or 1", mask[p]));
    }
    Ok((payload.chunks_exact(2).map(|w| u16::from_le_bytes([w[0], w[1]])).collect(), format))
}

/// Checks the manifest's roots against the file and the counts: the roots sum to the file, each
/// starts at a BOS (so no document spans two roots), and each root's `scored_taken` equals the
/// trainer-scored count. Returns each root's match name.
fn check_roots(
    roots: &[ManifestRoot],
    counts: &[Counts],
    tokens: &[u16],
    bos: u16,
) -> Result<Vec<&'static str>> {
    let sum: usize = roots.iter().map(|r| r.tokens_taken).sum();
    if sum != tokens.len() {
        return Err(format!("the roots' tokens_taken sum to {sum}, the file holds {}", tokens.len()));
    }
    let mut start = 0usize;
    for r in roots {
        if r.tokens_taken > 0 && tokens[start] != bos {
            return Err(format!("root {} does not start at a BOS (position {start})", r.name));
        }
        start += r.tokens_taken;
    }
    roots
        .iter()
        .zip(counts)
        .map(|(r, c)| {
            if r.scored_taken == c.scored_targets {
                Ok("trainer-scored")
            } else {
                Err(format!(
                    "root {}: scored_taken {} differs from the trainer-scored count {} (mask ones {})",
                    r.name, r.scored_taken, c.scored_targets, c.mask_ones
                ))
            }
        })
        .collect()
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let get = |name: &str| args.get(name).ok_or(format!("--{name} is required"));
    let eos: u16 = get("eos")?.parse().map_err(|e| format!("--eos: {e}"))?;
    let bos: u16 = get("bos")?.parse().map_err(|e| format!("--bos: {e}"))?;
    let out = Path::new(get("out")?);
    let token_bytes = read(get("tokens")?)?;
    let mask = read(get("mask")?)?;
    let (tokens, format) = decode_tokens(&token_bytes, &mask)?;
    let manifest_bytes = args.get("manifest").map(|path| read(path)).transpose()?;
    let manifest: Option<Manifest> = manifest_bytes
        .as_deref()
        .map(|bytes| serde_json::from_slice(bytes).map_err(|e| format!("--manifest: {e}")))
        .transpose()?;
    let lengths: Vec<usize> = manifest.iter().flat_map(|m| m.roots.iter().map(|r| r.tokens_taken)).collect();
    let (root_counts, total) = count(&tokens, &mask, &lengths, eos, bos);
    let mut roots = Vec::new();
    if let Some(manifest) = &manifest {
        let matches = check_roots(&manifest.roots, &root_counts, &tokens, bos)?;
        for ((r, counts), matched) in manifest.roots.iter().zip(root_counts).zip(matches) {
            roots.push(RootCounts {
                name: r.name.clone(),
                tokens_taken: r.tokens_taken,
                scored_taken: r.scored_taken,
                scored_taken_matches: matched,
                counts,
            });
        }
    }
    let report = serde_json::json!({
        "schema": SCHEMA,
        "convention": CONVENTION,
        "convention_source": CONVENTION_SOURCE,
        "reply_rule": "scored targets after the previous scored EOS of the document (or its BOS) through a scored EOS",
        "tokens_path": get("tokens")?,
        "tokens_sha256": sha256(&token_bytes),
        "tokens_format": format,
        "mask_path": get("mask")?,
        "mask_sha256": sha256(&mask),
        "manifest_path": args.get("manifest"),
        "manifest_sha256": manifest_bytes.as_deref().map(sha256),
        "eos": eos,
        "bos": bos,
        "roots": roots,
        "total": total,
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    write_new(out, &bytes)?;
    let summary = serde_json::json!({
        "out": out.display().to_string(),
        "out_sha256": sha256(&bytes),
        "positions": total.positions,
        "scored_targets": total.scored_targets,
        "eos_scored": total.eos_scored,
        "replies": total.replies,
        "reply_median": total.reply_median,
    });
    println!("{summary}");
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 6] = ["tokens", "mask", "manifest", "eos", "bos", "out"];
    let mut values = BTreeMap::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        let name = arg.strip_prefix("--").filter(|name| FLAGS.contains(name)).ok_or(format!("unknown argument {arg}"))?;
        let value = args.next().ok_or(format!("{arg} needs a value"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("{arg} given twice"));
        }
    }
    Ok(values)
}

fn read(path: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOS: u16 = 0;
    const EOS: u16 = 1;

    fn raw(tokens: &[u16]) -> Vec<u8> {
        tokens.iter().flat_map(|t| t.to_le_bytes()).collect()
    }

    #[test]
    fn the_target_is_the_masked_position_not_the_next() {
        // mask[2] == 1 and tokens[3] == EOS: a next-token rule would score the EOS, the trainer
        // does not, because mask[3] == 0.
        let tokens = [BOS, 7, 8, EOS];
        let mask = [0, 1, 1, 0];
        let (_, total) = count(&tokens, &mask, &[], EOS, BOS);
        assert_eq!(total.scored_targets, 2);
        assert_eq!(total.eos_tokens, 1);
        assert_eq!(total.eos_scored, 0);
        assert_eq!(total.unterminated_scored, 2);
        // And with mask[3] == 1 the EOS is scored at its own position.
        let (_, total) = count(&tokens, &[0, 0, 0, 1], &[], EOS, BOS);
        assert_eq!((total.scored_targets, total.eos_scored, total.replies), (1, 1, 1));
    }

    #[test]
    fn bos_and_tokens_before_the_first_bos_are_never_scored() {
        let tokens = [7, EOS, BOS, 8, EOS];
        let mask = [1, 1, 1, 1, 1];
        let (_, total) = count(&tokens, &mask, &[], EOS, BOS);
        assert_eq!(total.mask_ones, 5);
        assert_eq!(total.scored_targets, 2);
        assert_eq!((total.eos_tokens, total.eos_scored, total.bos_tokens), (2, 1, 1));
    }

    #[test]
    fn roots_split_in_file_order_and_must_sum_to_the_file() {
        // Root a: BOS 5 EOS (scored 5, EOS); root b: BOS 6 6 EOS BOS 4 (two documents).
        let tokens = [BOS, 5, EOS, BOS, 6, 6, EOS, BOS, 4];
        let mask = [0, 1, 1, 0, 1, 1, 1, 0, 1];
        let (roots, total) = count(&tokens, &mask, &[3, 6], EOS, BOS);
        assert_eq!(roots.len(), 2);
        assert_eq!((roots[0].positions, roots[0].scored_targets, roots[0].eos_scored), (3, 2, 1));
        assert_eq!((roots[1].positions, roots[1].scored_targets, roots[1].eos_scored), (6, 4, 1));
        assert_eq!((roots[1].bos_tokens, roots[1].unterminated_scored), (2, 1));
        assert_eq!(total.scored_targets, roots[0].scored_targets + roots[1].scored_targets);
        let manifest = |a: usize, b: usize, sa: usize, sb: usize| {
            vec![
                ManifestRoot { name: "a".into(), tokens_taken: a, scored_taken: sa },
                ManifestRoot { name: "b".into(), tokens_taken: b, scored_taken: sb },
            ]
        };
        assert_eq!(
            check_roots(&manifest(3, 6, 2, 4), &roots, &tokens, BOS).unwrap(),
            ["trainer-scored", "trainer-scored"]
        );
        assert!(check_roots(&manifest(3, 6, 2, 9), &roots, &tokens, BOS).unwrap_err().contains("root b"));
        assert!(check_roots(&manifest(3, 5, 2, 4), &roots, &tokens, BOS).unwrap_err().contains("sum to 8"));
        // A split inside a document: root b would start at position 2, an EOS, not a BOS.
        let (split, _) = count(&tokens, &mask, &[2, 7], EOS, BOS);
        assert!(check_roots(&manifest(2, 7, 1, 5), &split, &tokens, BOS).unwrap_err().contains("root b does not start"));
    }

    #[test]
    fn scored_taken_must_equal_the_trainer_count_not_the_mask_ones() {
        let tokens = [7, BOS, 5, EOS];
        let mask = [1, 0, 1, 1];
        let (roots, _) = count(&tokens, &mask, &[4], EOS, BOS);
        let root = [ManifestRoot { name: "r".into(), tokens_taken: 4, scored_taken: 3 }];
        // 7 precedes the first BOS, so the root does not start at a BOS.
        assert!(check_roots(&root, &roots, &tokens, BOS).unwrap_err().contains("does not start"));
        let tokens = [BOS, 5, EOS, 9];
        let mask = [1, 1, 1, 0];
        let (roots, _) = count(&tokens, &mask, &[4], EOS, BOS);
        // Mask ones are 3 (the BOS included), the trainer scores 2.
        let root = [ManifestRoot { name: "r".into(), tokens_taken: 4, scored_taken: 3 }];
        assert!(check_roots(&root, &roots, &tokens, BOS).unwrap_err().contains("differs from the trainer-scored count 2"));
    }

    #[test]
    fn rejects_a_mask_byte_other_than_0_or_1_and_a_length_mismatch() {
        let bytes = raw(&[BOS, 5, EOS]);
        assert!(decode_tokens(&bytes, &[0, 2, 1]).unwrap_err().contains("mask byte 2 at position 1"));
        assert!(decode_tokens(&bytes, &[0, 1]).unwrap_err().contains("payload holds 6 bytes"));
        let mut uort = b"UORT".to_vec();
        uort.extend([0u8; 4]);
        uort.extend(3u64.to_le_bytes());
        uort.resize(UORT_HEADER_BYTES, 0);
        uort.extend(&bytes);
        assert_eq!(decode_tokens(&uort, &[0, 1, 1]).unwrap(), (vec![BOS, 5, EOS], "uort"));
        assert!(decode_tokens(&uort, &[0, 1, 1, 1]).unwrap_err().contains("UORT count 3"));
        assert_eq!(decode_tokens(&bytes, &[0, 1, 1]).unwrap(), (vec![BOS, 5, EOS], "raw-u16le"));
    }

    #[test]
    fn reply_runs_count_scored_targets_between_scored_eos_within_a_document() {
        // Document 1: user turn (unscored) 9 9, reply 5 5 EOS (3), user 9, reply 6 EOS (2),
        // then 4 4 4 with no EOS before the next BOS (unterminated 3).
        // Document 2: reply 7 7 7 7 EOS (5).
        let tokens = [BOS, 9, 9, 5, 5, EOS, 9, 6, EOS, 4, 4, 4, BOS, 7, 7, 7, 7, EOS];
        let mask = [0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1];
        let (_, total) = count(&tokens, &mask, &[], EOS, BOS);
        assert_eq!(total.replies, 3);
        assert_eq!(total.reply_scored_total, 10);
        assert_eq!(total.reply_median, 3);
        assert_eq!(total.reply_max, 5);
        assert_eq!(total.unterminated_scored, 3);
        assert_eq!(total.eos_scored, 3);
        assert_eq!(total.eos_scored_ppm, 3 * 1_000_000 / 13);
        // Even count: the lower median.
        let (_, two) = count(&[BOS, 5, EOS, 6, 6, 6, EOS], &[0, 1, 1, 1, 1, 1, 1], &[], EOS, BOS);
        assert_eq!((two.replies, two.reply_median, two.reply_max), (2, 2, 4));
    }
}
