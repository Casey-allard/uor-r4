//! Recall-vs-generation attribution audit (plan item E1.4).
//!
//! Measures what fraction of a *generated* token sequence `G` appears verbatim
//! in a training corpus `C`, and what the longest verbatim match is. This is the
//! leakage/contamination audit that must precede any quality claim: "the fluent
//! spans are >= 13-gram matches" means the output is recall and must be
//! described as such.
//!
//! # Method: the index is inverted on purpose
//!
//! `docs/integration/native-core-transition-plan.md` (E1.4) describes indexing
//! every k-gram of `tinystories_train.u16`. That index does not fit in memory
//! (555M tokens, 9 values of k). This tool inverts it: the **small** side — the
//! generated sequence, typically 64..4096 tokens — is built into a
//! `HashMap<u64, Vec<u32>>` of rolling-hash -> start indices, and the corpus is
//! streamed once per `k`. Streaming maintains a rolling window hash and looks up
//! each corpus position.
//!
//! For a *single* generated sequence the inverted form is equivalent to the full
//! index for the quantities reported here, and strictly bounded in memory:
//!
//! 1. Every corpus start offset `p` is visited once per `k`, so no occurrence can
//!    be missed (no false negatives by construction).
//! 2. On a hash hit, each candidate `i` is confirmed by direct `u16` slice
//!    comparison, so a hash collision costs time but can never produce a false
//!    positive. **Verification is ON and the result is exact.**
//! 3. Memory is `O(max_positions)` distinct generated k-grams plus a fixed-size
//!    rolling window; nothing is allocated per corpus position.
//!
//! # Exactness properties
//!
//! - Hash: rolling 64-bit polynomial (FNV-1a style) over `u16` token values
//!   `token + 1` with base `0x100000001b3`, all arithmetic wrapping mod 2^64, and
//!   a splitmix64 finalizer as the map key. Collisions are harmless (verified).
//! - The rolling recurrence is self-checked at the end of every pass that runs to
//!   the final corpus window: the incrementally rolled hash must equal a direct
//!   recomputation of that window. Any drift aborts the run rather than reporting
//!   a wrong number. A pass that exits early (below) skips this check, because it
//!   changed nothing; the output marks it `early_exit=true`.
//! - Reported witnesses are re-verified against the corpus at report time
//!   (`witness_reverification_failures` must be 0).
//!
//! # Known limits (stated, not hidden)
//!
//! - One corpus pass per `k` in `k_min..=k_max` (9 by default). Elapsed time per
//!   `k` is printed, so the cost is visible before scaling up.
//! - A generated sequence that is one repeated token collapses many start
//!   indices into one hash entry, making verification cost `O(hits * k)`. The
//!   reported `verified_pairs` exposes that volume; no k value is ever skipped
//!   silently.
//! - A pass that exits early reports **partial** `hash_hit_positions` and
//!   `verified_pairs`, because the corpus scan stopped as soon as no start could
//!   change. `starts_ge_k`, the coverage columns and the witnesses are
//!   unaffected. Both the pass line and the table row mark `early_exit=true`.
//!   This short-circuits passes early precisely in the `RECALL_DOMINANT` case,
//!   where every generated start already has a longer match.
//! - `MmapCorpusReader::as_slice()` yields native-endian `u16`, which matches the
//!   little-endian on-disk format on little-endian hosts only (the same
//!   assumption as the reader's own `Deref`/`chunks`/`windows`).
//! - `--text` calls `HfBpeTokenizer::encode`, which adds **no** BOS/EOS
//!   (`pretokenize-corpus` adds those separately; this tool does not).
//! - Deviation from the specified `first_corpus_offset.entry(i).or_insert(p)`: the
//!   recorded corpus offset is the one that *first achieved the current maximum
//!   length* for `i`, not the earliest offset seen for any length. A witness is
//!   therefore verifiable for the length it is printed with.
//! - `VERDICT` is `UNDETERMINED` when `--k-max < 13` (or `G` is shorter than 13),
//!   because the `>= 13`-gram statistic the verdict is defined on does not exist.
//!   Reporting `GENERATION_DOMINANT` from an insufficient `k` range would be a
//!   false negative on long-span recall.
//!
//! # Exit codes
//!
//! `0` success (including `--help`), `1` any error, with a single clean
//! `error: ...` line on stderr.

#![forbid(unsafe_code)]

use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// FNV-1a 64-bit prime; odd, and the rolling base for the polynomial hash.
const HASH_BASE: u64 = 0x0000_0100_0000_01B3;
const DEFAULT_K_MIN: usize = 5;
const DEFAULT_K_MAX: usize = 13;
/// The k at which the recall/generation verdict is defined.
const VERDICT_K: usize = 13;
const DEFAULT_MAX_POSITIONS: usize = 8192;
const MAX_WITNESSES: usize = 20;
const WITNESS_PREVIEW_TOKENS: usize = 16;
/// Upper bound on `--k-max`; keeps the drop-term exponent and reports sane.
const MAX_K: usize = 4096;
const READ_CHUNK_BYTES: usize = 1 << 16;
const DIGEST_CHUNK_BYTES: usize = 1 << 20;

#[inline]
fn token_value(token: u16) -> u64 {
    // +1 keeps token 0 from collapsing the polynomial to a pure shift.
    u64::from(token) + 1
}

/// splitmix64 finalizer. Only the number of collisions is affected; correctness
/// is guaranteed by the slice verification on every hit.
#[inline]
fn mix64(mut x: u64) -> u64 {
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^= x >> 33;
    x
}

/// Direct (non-rolling) polynomial hash of one window; the reference the rolling
/// recurrence must agree with.
#[inline]
fn unmixed_hash(tokens: &[u16]) -> u64 {
    let mut h = 0u64;
    for &token in tokens {
        h = h.wrapping_mul(HASH_BASE).wrapping_add(token_value(token));
    }
    h
}

#[inline]
fn hash_key(unmixed: u64) -> u64 {
    mix64(unmixed)
}

enum GeneratedSource {
    Tokens(PathBuf),
    Text { text: PathBuf, tokenizer: PathBuf },
}

struct Args {
    corpus: PathBuf,
    source: GeneratedSource,
    k_min: usize,
    k_max: usize,
    max_positions: usize,
}

struct LoadedGenerated {
    ids: Vec<u16>,
    source_desc: String,
    source_digest: String,
    truncated: bool,
    extra_lines: Vec<String>,
}

#[derive(Default)]
struct PassOutcome {
    scanned_positions: u64,
    hash_hit_positions: u64,
    verified_positions: u64,
    verified_pairs: u64,
    new_start_marks: u64,
    early_exit: bool,
    skipped_reason: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let argv: Vec<String> = std::env::args().collect();
    let program = argv
        .first()
        .cloned()
        .unwrap_or_else(|| "attribute-recall".to_owned());

    let Some(args) = parse_args(&argv, &program)? else {
        return Ok(());
    };

    let generated = load_generated(&args)?;
    if generated.ids.is_empty() {
        return Err("generated token sequence is empty (nothing to attribute)".into());
    }

    let start_time = Instant::now();
    let corpus_reader = MmapCorpusReader::open(&args.corpus)
        .map_err(|e| format!("failed to open corpus {}: {e}", args.corpus.display()))?;
    let corpus_version = corpus_reader.header().version;
    let corpus_total_tokens = corpus_reader.header().total_tokens;
    let corpus_vocab_size = corpus_reader.header().vocab_size;
    let corpus: &[u16] = corpus_reader.as_slice();

    println!("================================================================================");
    println!("attribute-recall — recall vs generation attribution audit (E1.4)");
    println!("================================================================================");
    println!("corpus: {}", args.corpus.display());
    println!("corpus_header: version={corpus_version} total_tokens={corpus_total_tokens} vocab_size={corpus_vocab_size}");
    println!(
        "corpus_blake3: {}",
        file_digest(&args.corpus).unwrap_or_else(|e| format!("UNAVAILABLE ({e})"))
    );
    println!("generated_source: {}", generated.source_desc);
    println!("generated_source_blake3: {}", generated.source_digest);
    println!("generated_tokens: {}", generated.ids.len());
    println!(
        "generated_tokens_truncated_by_max_positions: {} (max_positions={})",
        generated.truncated, args.max_positions
    );
    for line in &generated.extra_lines {
        println!("{line}");
    }
    let outside_vocab = generated
        .ids
        .iter()
        .filter(|&&t| u32::from(t) >= corpus_vocab_size)
        .count();
    println!(
        "generated_ids_outside_corpus_vocab: {outside_vocab} (corpus vocab_size={corpus_vocab_size}; such ids cannot match)"
    );
    println!("k_range: {}..={}", args.k_min, args.k_max);
    println!(
        "method: exact_verification=ON; k_range={}..={}; hash=rolling_polynomial_u16 base=0x{HASH_BASE:016x} mod=2^64 key=splitmix64_finalizer; \
         index=INVERTED (generated k-grams in HashMap, corpus streamed once per k); verification=direct u16 slice comparison on every hash hit, so collisions cost time, never correctness",
        args.k_min, args.k_max
    );
    println!("--------------------------------------------------------------------------------");

    let n = generated.ids.len();
    let mut start_match = vec![0u32; n];
    let mut witness_offset = vec![0u64; n];
    let mut outcomes: Vec<(usize, PassOutcome, f64)> =
        Vec::with_capacity(args.k_max - args.k_min + 1);

    for k in args.k_min..=args.k_max {
        let pass_start = Instant::now();
        let outcome = scan_corpus_for_k(
            corpus,
            &generated.ids,
            k,
            &mut start_match,
            &mut witness_offset,
        )?;
        let elapsed = pass_start.elapsed().as_secs_f64();
        if let Some(reason) = &outcome.skipped_reason {
            println!("pass k={k}: SKIPPED ({reason})");
        } else {
            println!(
                "pass k={k}: elapsed_s={elapsed:.3} scanned_positions={} hash_hit_positions={} verified_positions={} verified_pairs={} new_start_marks={} early_exit={}",
                outcome.scanned_positions,
                outcome.hash_hit_positions,
                outcome.verified_positions,
                outcome.verified_pairs,
                outcome.new_start_marks,
                outcome.early_exit
            );
        }
        outcomes.push((k, outcome, elapsed));
    }

    let longest_start = start_match.iter().copied().max().unwrap_or(0);
    let covering = longest_covering_lengths(&start_match);
    let longest_covered = covering.iter().copied().max().unwrap_or(0);

    // Per-k coverage of generated tokens by verified matches of length >= k.
    let mut delta = vec![0i32; n + 1];
    let coverage: Vec<(usize, u64)> = outcomes
        .iter()
        .map(|&(k, _, _)| (k, coverage_for_k(&start_match, k, &mut delta)))
        .collect();

    println!("--------------------------------------------------------------------------------");
    println!("longest_verbatim_match_tokens: {longest_start}");
    let limit_note = if longest_start as usize == n {
        "generated_sequence_fully_covered_by_one_verified_match".to_owned()
    } else if longest_start as usize == args.k_max {
        format!(
            "capped_by_k_max({}) — a longer verbatim match may exist; raise --k-max to measure it",
            args.k_max
        )
    } else {
        "not_capped (the longest match ended inside the generated sequence, below k_max)".to_owned()
    };
    println!("longest_match_limit: {limit_note}");
    println!(
        "longest_covered_match_tokens: {longest_covered} (extra: longest match covering any generated position)"
    );
    println!();
    println!("per-k verified-match table:");
    println!(
        "{:>5}  {:>12}  {:>14}  {:>16}  {:>16}  {:>13}  {:>10}",
        "k",
        "starts_ge_k",
        "covered_tokens",
        "covered_fraction",
        "hash_hit_positions",
        "verified_pairs",
        "elapsed_s"
    );
    for (idx, &(k, ref outcome, elapsed)) in outcomes.iter().enumerate() {
        if let Some(reason) = &outcome.skipped_reason {
            println!("{k:>5}  (skipped: {reason})");
            continue;
        }
        let starts_ge_k = start_match.iter().filter(|&&m| m as usize >= k).count();
        let covered = coverage[idx].1;
        let early_exit_note = if outcome.early_exit {
            "  early_exit=true (hits/pairs partial)"
        } else {
            ""
        };
        println!(
            "{k:>5}  {starts_ge_k:>12}  {covered:>14}  {:>16.6}  {:>16}  {:>13}  {elapsed:>10.3}{early_exit_note}",
            covered as f64 / n as f64,
            outcome.hash_hit_positions,
            outcome.verified_pairs
        );
    }

    println!();
    println!("tokens_covered_by_ge_k: count and fraction of generated tokens inside >=1 verified match of length >= k");
    for &(k, covered) in &coverage {
        println!(
            "tokens_covered_by_ge_{k}: {covered} / {n} ({:.6})",
            covered as f64 / n as f64
        );
    }

    println!();
    let mut witnesses: Vec<(usize, u32, u64)> = start_match
        .iter()
        .enumerate()
        .filter(|(_, &len)| len > 0)
        .map(|(i, &len)| (i, len, witness_offset[i]))
        .collect();
    witnesses.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    println!(
        "distinct_match_witnesses: {} (verified matches found; showing up to {MAX_WITNESSES} longest)",
        witnesses.len()
    );
    let mut reverification_failures = 0usize;
    if witnesses.is_empty() {
        println!(
            "  none: no verified k-gram match for any k in {}..={}",
            args.k_min, args.k_max
        );
    }
    for (rank, &(i, len, offset)) in witnesses.iter().take(MAX_WITNESSES).enumerate() {
        let len_usize = len as usize;
        let offset_usize = usize::try_from(offset).unwrap_or(usize::MAX);
        let generated_span = generated.ids.get(i..i + len_usize);
        let corpus_span = corpus.get(offset_usize..offset_usize + len_usize);
        let reverified = matches!(
            (generated_span, corpus_span),
            (Some(g), Some(c)) if g == c
        );
        if !reverified {
            reverification_failures += 1;
        }
        let preview_end = (i + WITNESS_PREVIEW_TOKENS).min(i + len_usize);
        let preview = generated
            .ids
            .get(i..preview_end)
            .map(|s| {
                let mut text = s.iter().map(u16::to_string).collect::<Vec<_>>().join(",");
                if preview_end < i + len_usize {
                    text.push_str(",...");
                }
                text
            })
            .unwrap_or_else(|| "UNAVAILABLE".to_owned());
        println!(
            "  witness[{}]: generated[{i}..{}] len={len} corpus_offset={offset} corpus_span=[{offset},{}) corpus_tokens={corpus_total_tokens} reverified={reverified} tokens=[{preview}]",
            rank + 1,
            i + len_usize,
            offset_usize + len_usize
        );
    }
    println!("witness_reverification_failures: {reverification_failures}");

    println!();
    let verdict = verdict(args.k_max, n, &coverage);
    println!("{verdict}");
    println!("elapsed_total_s: {:.3}", start_time.elapsed().as_secs_f64());
    Ok(())
}

fn verdict(k_max: usize, n: usize, coverage: &[(usize, u64)]) -> String {
    if k_max < VERDICT_K {
        return format!(
            "VERDICT: UNDETERMINED (k_max={k_max} < {VERDICT_K}: the >= {VERDICT_K}-gram statistic the verdict is defined on does not exist in this run; rerun with --k-max >= {VERDICT_K})"
        );
    }
    if n < VERDICT_K {
        return format!(
            "VERDICT: UNDETERMINED (generated sequence has {n} tokens, fewer than {VERDICT_K}; no {VERDICT_K}-gram exists)"
        );
    }
    let covered = coverage
        .iter()
        .find(|&&(k, _)| k == VERDICT_K)
        .map(|&(_, c)| c)
        .unwrap_or(0);
    let fraction = covered as f64 / n as f64;
    let basis = format!(
        "VERDICT_basis: tokens_covered_by_ge_{VERDICT_K} = {covered} / {n} ({fraction:.6})"
    );
    let label = if fraction > 0.5 {
        "RECALL_DOMINANT"
    } else if fraction > 0.1 {
        "MIXED"
    } else {
        "GENERATION_DOMINANT"
    };
    format!("{basis}\nVERDICT: {label}")
}

/// One streaming pass over the corpus for a single `k`.
///
/// Builds the inverted index of generated k-grams, then walks every corpus start
/// offset once, recording for each generated start `i` the largest `k` with a
/// verified occurrence and the corpus offset that first achieved it.
fn scan_corpus_for_k(
    corpus: &[u16],
    generated: &[u16],
    k: usize,
    start_match: &mut [u32],
    witness_offset: &mut [u64],
) -> Result<PassOutcome, Box<dyn std::error::Error>> {
    let mut outcome = PassOutcome::default();
    let n = generated.len();
    if k == 0 || k > n {
        outcome.skipped_reason = Some(format!(
            "k={k} exceeds generated length {n}; no generated k-gram exists"
        ));
        return Ok(outcome);
    }
    if corpus.len() < k {
        outcome.skipped_reason = Some(format!(
            "corpus holds {} tokens, shorter than k={k}",
            corpus.len()
        ));
        return Ok(outcome);
    }

    // Only positions `0..=n-k` can start a generated k-gram, so only those can
    // ever be marked by this pass; counting the untouchable tail would make the
    // early exit unreachable.
    let mut below_k = start_match[..=n - k]
        .iter()
        .filter(|&&matched| (matched as usize) < k)
        .count();
    if below_k == 0 {
        // Every start already holds a verified match of length >= k, so this
        // pass can only re-derive `k` and cannot change any recorded maximum.
        outcome.early_exit = true;
        return Ok(outcome);
    }

    let mut index: HashMap<u64, Vec<u32>> = HashMap::with_capacity(n - k + 1);
    for i in 0..=(n - k) {
        let key = hash_key(unmixed_hash(&generated[i..i + k]));
        index.entry(key).or_default().push(i as u32);
    }

    let drop_term = HASH_BASE.wrapping_pow((k - 1) as u32);
    let mut hash = unmixed_hash(&corpus[..k]);
    let last = corpus.len() - k;
    let mut p = 0usize;
    let mut ran_to_end = false;

    loop {
        outcome.scanned_positions += 1;
        if let Some(entries) = index.get(&hash_key(hash)) {
            outcome.hash_hit_positions += 1;
            let mut verified_here = false;
            for &entry_index in entries {
                let i = entry_index as usize;
                if generated[i..i + k] == corpus[p..p + k] {
                    verified_here = true;
                    outcome.verified_pairs += 1;
                    if start_match[i] < k as u32 {
                        start_match[i] = k as u32;
                        witness_offset[i] = p as u64;
                        outcome.new_start_marks += 1;
                        below_k -= 1;
                    }
                }
            }
            if verified_here {
                outcome.verified_positions += 1;
            }
        }

        if p == last {
            ran_to_end = true;
            break;
        }
        if below_k == 0 {
            // No start can gain from any remaining corpus position in this pass.
            outcome.early_exit = true;
            break;
        }

        hash = hash
            .wrapping_sub(token_value(corpus[p]).wrapping_mul(drop_term))
            .wrapping_mul(HASH_BASE)
            .wrapping_add(token_value(corpus[p + k]));
        p += 1;
    }

    if ran_to_end {
        // The rolled hash must equal a direct recomputation of the final window.
        let expected = unmixed_hash(&corpus[last..last + k]);
        if hash != expected {
            return Err(format!(
                "internal rolling-hash invariant violated at k={k}: rolled={hash:#018x} expected={expected:#018x}. No result reported."
            )
            .into());
        }
    }

    Ok(outcome)
}

/// Counts generated positions lying inside at least one verified match of length
/// `>= k`, via a difference array reused across `k`.
fn coverage_for_k(start_match: &[u32], k: usize, delta: &mut [i32]) -> u64 {
    let n = start_match.len();
    for slot in delta[..=n].iter_mut() {
        *slot = 0;
    }
    for (i, &len) in start_match.iter().enumerate() {
        let len = len as usize;
        if len >= k {
            let end = (i + len).min(n);
            delta[i] += 1;
            delta[end] -= 1;
        }
    }
    let mut running = 0i32;
    let mut covered = 0u64;
    for &step in delta[..n].iter() {
        running += step;
        if running > 0 {
            covered += 1;
        }
    }
    covered
}

/// For every generated position, the longest verified match covering it.
fn longest_covering_lengths(start_match: &[u32]) -> Vec<u32> {
    let n = start_match.len();
    let mut out = vec![0u32; n];
    let mut heap: BinaryHeap<(u32, usize)> = BinaryHeap::new();
    for pos in 0..n {
        let len = start_match[pos];
        if len > 0 {
            heap.push((len, pos + len as usize));
        }
        while let Some(&(_, end)) = heap.peek() {
            if end <= pos {
                heap.pop();
            } else {
                break;
            }
        }
        out[pos] = heap.peek().map(|&(len, _)| len).unwrap_or(0);
    }
    out
}

/// Streams whitespace-separated ASCII decimal `u16` ids from `path`.
///
/// Stops storing once `limit` ids are held, reporting whether more were present,
/// so a large token dump cannot blow up memory. Any non-digit, non-whitespace
/// byte is an error rather than a silent separator.
fn read_token_ids(
    path: &Path,
    limit: usize,
) -> Result<(Vec<u16>, bool), Box<dyn std::error::Error>> {
    let file =
        File::open(path).map_err(|e| format!("failed to open tokens {}: {e}", path.display()))?;
    let mut reader = BufReader::with_capacity(READ_CHUNK_BYTES, file);
    let mut buf = vec![0u8; READ_CHUNK_BYTES];
    let mut out: Vec<u16> = Vec::new();
    let mut value: u64 = 0;
    let mut digits: u32 = 0;
    let mut parsed: u64 = 0;
    let mut truncated = false;

    'outer: loop {
        let read = reader
            .read(&mut buf)
            .map_err(|e| format!("failed to read tokens {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        for &byte in &buf[..read] {
            if byte.is_ascii_digit() {
                digits += 1;
                if digits > 5 {
                    return Err(format!(
                        "{}: token number {parsed} has more than 5 digits; u16 ids only",
                        path.display()
                    )
                    .into());
                }
                value = value * 10 + u64::from(byte - b'0');
                if value > u64::from(u16::MAX) {
                    return Err(format!(
                        "{}: token number {parsed} = {value} exceeds the u16 range",
                        path.display()
                    )
                    .into());
                }
            } else if byte.is_ascii_whitespace() {
                if digits > 0 {
                    if out.len() >= limit {
                        truncated = true;
                        break 'outer;
                    }
                    out.push(value as u16);
                    parsed += 1;
                    value = 0;
                    digits = 0;
                }
            } else {
                return Err(format!(
                    "{}: unexpected byte {byte:#04x} at token number {parsed}; expected ASCII decimal digits or whitespace",
                    path.display()
                )
                .into());
            }
        }
    }

    if !truncated && digits > 0 {
        if out.len() >= limit {
            truncated = true;
        } else {
            out.push(value as u16);
        }
    }

    Ok((out, truncated))
}

/// `blake3:<hex>` over the raw bytes of `path`, for artifact binding.
fn file_digest(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    let mut reader = BufReader::with_capacity(DIGEST_CHUNK_BYTES, file);
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; DIGEST_CHUNK_BYTES];
    loop {
        let read = reader
            .read(&mut buf)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn load_generated(args: &Args) -> Result<LoadedGenerated, Box<dyn std::error::Error>> {
    match &args.source {
        GeneratedSource::Tokens(path) => {
            let (ids, truncated) = read_token_ids(path, args.max_positions)?;
            Ok(LoadedGenerated {
                ids,
                source_desc: format!(
                    "--tokens {} (whitespace-separated u16 ids, verbatim; no BOS/EOS added)",
                    path.display()
                ),
                source_digest: file_digest(path)?,
                truncated,
                extra_lines: Vec::new(),
            })
        }
        GeneratedSource::Text { text, tokenizer } => {
            let tokenizer_bytes = std::fs::read(tokenizer)
                .map_err(|e| format!("failed to read tokenizer {}: {e}", tokenizer.display()))?;
            let bpe = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or_else(
                || -> Box<dyn std::error::Error> {
                    format!(
                        "{}: not a valid byte-level BPE tokenizer (a tokenizer.json with model.type=BPE, vocab and merges is required)",
                        tokenizer.display()
                    )
                    .into()
                },
            )?;
            let text_body = std::fs::read_to_string(text)
                .map_err(|e| format!("failed to read text {}: {e}", text.display()))?;
            let encoded = bpe.encode(&text_body);
            let encoded_total = encoded.len();
            let mut ids: Vec<u16> = Vec::with_capacity(encoded_total);
            for (position, &id) in encoded.iter().enumerate() {
                let id16 = u16::try_from(id).map_err(|_| -> Box<dyn std::error::Error> {
                    format!("encoded token {position} = {id} exceeds the u16 corpus token range")
                        .into()
                })?;
                ids.push(id16);
            }
            let mut truncated = false;
            if ids.len() > args.max_positions {
                ids.truncate(args.max_positions);
                truncated = true;
            }
            Ok(LoadedGenerated {
                ids,
                source_desc: format!(
                    "--text {} via --tokenizer {} (HfBpeTokenizer::encode; no BOS/EOS added)",
                    text.display(),
                    tokenizer.display()
                ),
                source_digest: file_digest(text)?,
                truncated,
                extra_lines: vec![
                    format!("generated_tokens_encoded_total: {encoded_total}"),
                    format!("tokenizer_address: {}", bpe.address()),
                    format!("tokenizer_vocab_size: {}", bpe.vocab_size()),
                ],
            })
        }
    }
}

fn parse_args(args: &[String], program: &str) -> Result<Option<Args>, Box<dyn std::error::Error>> {
    let mut corpus: Option<PathBuf> = None;
    let mut tokens: Option<PathBuf> = None;
    let mut text: Option<PathBuf> = None;
    let mut tokenizer: Option<PathBuf> = None;
    let mut k_min = DEFAULT_K_MIN;
    let mut k_max = DEFAULT_K_MAX;
    let mut max_positions = DEFAULT_MAX_POSITIONS;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                let mut stdout = std::io::stdout();
                let _ = write_usage(program, &mut stdout);
                return Ok(None);
            }
            "--corpus" => corpus = Some(PathBuf::from(require_value(args, &mut i, "--corpus")?)),
            "--tokens" => tokens = Some(PathBuf::from(require_value(args, &mut i, "--tokens")?)),
            "--text" => text = Some(PathBuf::from(require_value(args, &mut i, "--text")?)),
            "--tokenizer" => {
                tokenizer = Some(PathBuf::from(require_value(args, &mut i, "--tokenizer")?))
            }
            "--k-min" => {
                k_min = parse_usize(require_value(args, &mut i, "--k-min")?, "--k-min")?;
            }
            "--k-max" => {
                k_max = parse_usize(require_value(args, &mut i, "--k-max")?, "--k-max")?;
            }
            "--max-positions" => {
                max_positions = parse_usize(
                    require_value(args, &mut i, "--max-positions")?,
                    "--max-positions",
                )?;
            }
            other => {
                let mut stderr = std::io::stderr();
                let _ = write_usage(program, &mut stderr);
                return Err(format!("unknown option: {other}").into());
            }
        }
        i += 1;
    }

    let corpus = corpus.ok_or_else(|| -> Box<dyn std::error::Error> {
        "--corpus <path.u16> is required".into()
    })?;
    let source = match (tokens, text) {
        (Some(_), Some(_)) => {
            return Err("--tokens and --text are mutually exclusive".into());
        }
        (Some(path), None) => {
            if tokenizer.is_some() {
                return Err("--tokenizer is only valid with --text".into());
            }
            GeneratedSource::Tokens(path)
        }
        (None, Some(text)) => {
            let tokenizer = tokenizer.ok_or_else(|| -> Box<dyn std::error::Error> {
                "--text requires --tokenizer <tokenizer.json>".into()
            })?;
            GeneratedSource::Text { text, tokenizer }
        }
        (None, None) => {
            return Err("exactly one of --tokens <path> or --text <path> is required".into());
        }
    };

    if max_positions == 0 {
        return Err("--max-positions must be at least 1".into());
    }
    if k_min == 0 {
        return Err("--k-min must be at least 1".into());
    }
    if k_max < k_min {
        return Err(format!("--k-max ({k_max}) must be >= --k-min ({k_min})").into());
    }
    if k_max > MAX_K {
        return Err(format!("--k-max ({k_max}) exceeds the supported maximum of {MAX_K}").into());
    }

    Ok(Some(Args {
        corpus,
        source,
        k_min,
        k_max,
        max_positions,
    }))
}

fn require_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| -> Box<dyn std::error::Error> { format!("{flag} requires a value").into() })
}

fn parse_usize(value: &str, flag: &str) -> Result<usize, Box<dyn std::error::Error>> {
    value
        .parse::<usize>()
        .map_err(|_| -> Box<dyn std::error::Error> {
            format!("{flag}: '{value}' is not a non-negative integer").into()
        })
}

fn write_usage(program: &str, out: &mut dyn Write) -> std::io::Result<()> {
    writeln!(out, "Usage:")?;
    writeln!(
        out,
        "  {program} --corpus <path.u16> --tokens <path> [--k-min {DEFAULT_K_MIN}] [--k-max {DEFAULT_K_MAX}] [--max-positions {DEFAULT_MAX_POSITIONS}]"
    )?;
    writeln!(
        out,
        "  {program} --corpus <path.u16> --text <path> --tokenizer <tokenizer.json> [--k-min {DEFAULT_K_MIN}] [--k-max {DEFAULT_K_MAX}] [--max-positions {DEFAULT_MAX_POSITIONS}]"
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "Measures what fraction of a generated token sequence appears verbatim in a"
    )?;
    writeln!(
        out,
        "training corpus and the longest verbatim match. The corpus is streamed once"
    )?;
    writeln!(
        out,
        "per k; every 64-bit rolling-hash hit is confirmed by direct u16 slice"
    )?;
    writeln!(out, "comparison, so the reported matches are exact.")?;
    writeln!(out)?;
    writeln!(out, "Options:")?;
    writeln!(
        out,
        "  --corpus <path>        .u16 corpus (64-byte UORT header + u16 tokens)"
    )?;
    writeln!(
        out,
        "  --tokens <path>        whitespace-separated u16 ids; no BOS/EOS added"
    )?;
    writeln!(
        out,
        "  --text <path>          UTF-8 text encoded by --tokenizer; no BOS/EOS added"
    )?;
    writeln!(
        out,
        "  --tokenizer <path>     tokenizer.json, required with --text"
    )?;
    writeln!(
        out,
        "  --k-min <n>            smallest k-gram length (default {DEFAULT_K_MIN})"
    )?;
    writeln!(
        out,
        "  --k-max <n>            largest k-gram length (default {DEFAULT_K_MAX}; verdict needs >= {VERDICT_K})"
    )?;
    writeln!(
        out,
        "  --max-positions <n>    cap on generated tokens considered (default {DEFAULT_MAX_POSITIONS})"
    )?;
    writeln!(out, "  --help, -h             print this usage")?;
    Ok(())
}
