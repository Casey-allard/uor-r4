//! Memory-Mapped Pre-Tokenization Pipeline & Chunked Binary Ingestion (Milestone 9).
//!
//! Ingests the 2.12 GB raw TinyStories text corpus and tokenizes it into
//! a contiguous 16-bit little-endian binary token stream with a canonical
//! 64-byte binary header.
//!
//! # Invariants:
//! - Parallel chunked BPE tokenization using Rayon across CPU cores.
//! - Input: `TinyStoriesV2-GPT4-train.txt` and `tokenizer.json`.
//! - Output: Contiguous `u16` tokens in `tinystories_train.u16` (~1.18 GB).
//! - 64-byte binary header: magic bytes `b"UORT"`, version 1, total token count (`u64`),
//!   vocabulary size (`u32`), and reserved padding.
//! - Working RAM stays strictly bounded (< 128 MB) via streaming chunked ingestion.

use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DELIMITER: &[u8] = b"<|endoftext|>";
const DEFAULT_INPUT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/raw/TinyStoriesV2-GPT4-train.txt";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/export/tokenizer.json";
const DEFAULT_OUTPUT: &str = "tinystories_train.u16";
const DEFAULT_BATCH_STORIES: usize = 4096;

thread_local! {
    static THREAD_TOKENIZER: RefCell<Option<HfBpeTokenizer>> = const { RefCell::new(None) };
}

fn tokenize_story(story: &str, tok_bytes: &[u8]) -> Vec<u16> {
    THREAD_TOKENIZER.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(
                HfBpeTokenizer::from_tokenizer_json_bytes(tok_bytes)
                    .expect("Failed to initialize thread-local BPE tokenizer"),
            );
        }
        let tokenizer = guard.as_ref().unwrap();
        let content_ids = tokenizer.encode(story);
        // Story framed with BOS (0) and EOS (1)
        let mut tokens = Vec::with_capacity(content_ids.len() + 2);
        tokens.push(0u16);
        for id in content_ids {
            debug_assert!(id <= 65535, "Token ID {id} exceeds 16-bit range");
            tokens.push(id as u16);
        }
        tokens.push(1u16);
        tokens
    })
}

fn print_usage(program: &str) {
    eprintln!(
        "Usage: {program} [OPTIONS]\n\
         Options:\n\
         --input <path>         Input text corpus file (default: {DEFAULT_INPUT})\n\
         --tokenizer <path>     Input tokenizer.json path (default: {DEFAULT_TOKENIZER})\n\
         --output <path>        Output .u16 binary corpus file (default: {DEFAULT_OUTPUT})\n\
         --max-bytes <bytes>    Optional byte limit to ingest from input file\n\
         --max-stories <count>  Optional maximum number of stories to process\n\
         --batch-size <stories> Stories per parallel Rayon chunk (default: {DEFAULT_BATCH_STORIES})\n\
         --threads <n>          Number of Rayon worker threads (default: CPU cores)\n\
         --verify               Verify output file after tokenization via MmapCorpusReader\n\
         --help                 Display this help message"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut input_path = PathBuf::from(DEFAULT_INPUT);
    let mut tokenizer_path = PathBuf::from(DEFAULT_TOKENIZER);
    let mut output_path = PathBuf::from(DEFAULT_OUTPUT);
    let mut max_bytes: Option<usize> = None;
    let mut max_stories: Option<usize> = None;
    let mut batch_stories = DEFAULT_BATCH_STORIES;
    let mut verify_after = true;
    let mut num_threads: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                if i < args.len() {
                    input_path = PathBuf::from(&args[i]);
                }
            }
            "--tokenizer" => {
                i += 1;
                if i < args.len() {
                    tokenizer_path = PathBuf::from(&args[i]);
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = PathBuf::from(&args[i]);
                }
            }
            "--max-bytes" => {
                i += 1;
                if i < args.len() {
                    max_bytes = Some(args[i].parse()?);
                }
            }
            "--max-stories" => {
                i += 1;
                if i < args.len() {
                    max_stories = Some(args[i].parse()?);
                }
            }
            "--batch-size" => {
                i += 1;
                if i < args.len() {
                    batch_stories = args[i].parse()?;
                }
            }
            "--threads" => {
                i += 1;
                if i < args.len() {
                    num_threads = Some(args[i].parse()?);
                }
            }
            "--verify" => {
                verify_after = true;
            }
            "--no-verify" => {
                verify_after = false;
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                return Ok(());
            }
            other => {
                eprintln!("Unknown option: {other}");
                print_usage(&args[0]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if let Some(t) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()
            .ok();
    }

    println!("================================================================================");
    println!("UOR-R4 Milestone 9: Memory-Mapped Pre-Tokenization Pipeline");
    println!("================================================================================");
    println!("Input Corpus:      {}", input_path.display());
    println!("Tokenizer:         {}", tokenizer_path.display());
    println!("Output File:       {}", output_path.display());
    println!("Batch Size:        {batch_stories} stories / chunk");
    println!("Rayon Workers:     {}", rayon::current_num_threads());
    if let Some(mb) = max_bytes {
        println!(
            "Byte Limit:        {mb} bytes ({:.2} MB)",
            mb as f64 / (1024.0 * 1024.0)
        );
    }
    if let Some(ms) = max_stories {
        println!("Story Limit:       {ms} stories");
    }
    println!("--------------------------------------------------------------------------------");

    // 1. Ingest Tokenizer and Determine Vocabulary Size
    println!("Loading BPE tokenizer...");
    let tok_bytes = std::fs::read(&tokenizer_path)?;
    let base_tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .ok_or("Failed to parse BPE tokenizer from json")?;
    let vocab_size = base_tokenizer.vocab_size() as u32;
    if vocab_size == 0 || vocab_size > 65536 {
        return Err(format!(
            "Tokenizer vocabulary size {vocab_size} is invalid (must be 1..=65536 for .u16 format)"
        )
        .into());
    }
    println!("Tokenizer loaded: vocabulary size = {vocab_size} tokens");

    // 2. Initialize Corpus Writer with 64-byte header
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut writer = CorpusWriter::create(&output_path, vocab_size)?;

    // 3. Streaming Ingestion Loop
    println!("\nStreaming raw corpus and tokenizing in parallel chunks...");
    let input_file = File::open(&input_path)?;
    let total_file_bytes = input_file.metadata()?.len() as usize;
    let mut reader = BufReader::with_capacity(8 * 1024 * 1024, input_file);

    let start_time = Instant::now();
    let mut last_log_time = Instant::now();
    let mut total_stories: usize = 0;
    let mut total_tokens: u64 = 0;
    let mut bytes_ingested: usize = 0;

    let mut current_batch: Vec<String> = Vec::with_capacity(batch_stories);
    let mut delimiter_search_buf: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let mut chunk = vec![0u8; 1024 * 1024];

    loop {
        if let Some(mb) = max_bytes {
            if bytes_ingested >= mb {
                break;
            }
        }

        let read_len = reader.read(&mut chunk)?;
        if read_len == 0 {
            break;
        }

        bytes_ingested += read_len;
        delimiter_search_buf.extend_from_slice(&chunk[..read_len]);

        // Scan for <|endoftext|> delimiter in delimiter_search_buf
        let mut search_start = 0;
        while let Some(pos) = delimiter_search_buf[search_start..]
            .windows(DELIMITER.len())
            .position(|window| window == DELIMITER)
        {
            let abs_pos = search_start + pos;
            let story_slice = &delimiter_search_buf[search_start..abs_pos];
            let story_str = match std::str::from_utf8(story_slice) {
                Ok(s) => s.trim(),
                Err(_) => {
                    let lossy = String::from_utf8_lossy(story_slice);
                    let trimmed = lossy.trim();
                    if !trimmed.is_empty() {
                        current_batch.push(trimmed.to_string());
                    }
                    search_start = abs_pos + DELIMITER.len();
                    continue;
                }
            };
            if !story_str.is_empty() {
                current_batch.push(story_str.to_string());
            }
            search_start = abs_pos + DELIMITER.len();

            let reached_limit = if let Some(ms) = max_stories {
                if total_stories + current_batch.len() >= ms {
                    let needed = ms.saturating_sub(total_stories);
                    current_batch.truncate(needed);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if current_batch.len() >= batch_stories || (reached_limit && !current_batch.is_empty())
            {
                // Parallel chunked tokenization across Rayon workers
                let batch_results: Vec<Vec<u16>> = current_batch
                    .par_iter()
                    .map(|story_str| tokenize_story(story_str, &tok_bytes))
                    .collect();

                for story_toks in &batch_results {
                    writer.write_tokens_unchecked(story_toks)?;
                    total_tokens += story_toks.len() as u64;
                }
                total_stories += current_batch.len();
                current_batch.clear();

                if last_log_time.elapsed().as_secs_f64() >= 5.0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let tok_per_sec = total_tokens as f64 / elapsed;
                    let mb_ingested = bytes_ingested as f64 / (1024.0 * 1024.0);
                    let pct = if total_file_bytes > 0 {
                        (bytes_ingested as f64 / total_file_bytes as f64) * 100.0
                    } else {
                        0.0
                    };
                    println!(
                        "[{elapsed:6.1}s] Ingested: {mb_ingested:7.2} MB ({pct:5.1}%) | Stories: {total_stories:7} | Tokens: {total_tokens:10} ({tok_per_sec:9.0} tok/s)"
                    );
                    last_log_time = Instant::now();
                }

                if reached_limit {
                    break;
                }
            }
        }

        // Retain unconsumed tail for next delimiter search
        delimiter_search_buf.drain(..search_start);

        if let Some(ms) = max_stories {
            if total_stories >= ms {
                break;
            }
        }
    }

    // Process any remaining tail in delimiter_search_buf on EOF
    if !delimiter_search_buf.is_empty() {
        let story_str = match std::str::from_utf8(&delimiter_search_buf) {
            Ok(s) => s.trim().to_string(),
            Err(_) => String::from_utf8_lossy(&delimiter_search_buf)
                .trim()
                .to_string(),
        };
        if !story_str.is_empty() {
            current_batch.push(story_str);
        }
        delimiter_search_buf.clear();
    }

    // Flush remaining batch
    if !current_batch.is_empty() {
        let batch_results: Vec<Vec<u16>> = current_batch
            .par_iter()
            .map(|story_str| tokenize_story(story_str, &tok_bytes))
            .collect();

        for story_toks in &batch_results {
            writer.write_tokens_unchecked(story_toks)?;
            total_tokens += story_toks.len() as u64;
        }
        total_stories += current_batch.len();
        current_batch.clear();
    }

    // Finalize binary corpus file with finalized 64-byte header
    let written_tokens = writer.finish()?;
    assert_eq!(written_tokens, total_tokens);
    let total_elapsed = start_time.elapsed();
    let elapsed_secs = total_elapsed.as_secs_f64();
    let final_tok_per_sec = total_tokens as f64 / elapsed_secs.max(1e-6);
    let output_bytes = std::fs::metadata(&output_path)?.len();

    println!("\n================================================================================");
    println!("Pre-Tokenization Ingestion Complete!");
    println!("================================================================================");
    println!("Processed Stories:   {total_stories}");
    println!("Total Tokens:        {total_tokens}");
    println!(
        "Output File:         {} ({output_bytes} bytes, {:.2} MB)",
        output_path.display(),
        output_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("Wall Time Elapsed:   {total_elapsed:.2?}");
    println!("Ingestion Rate:      {final_tok_per_sec:.0} tokens/sec");
    println!("================================================================================");

    // 4. Verification Check
    if verify_after {
        println!("\nVerifying output corpus artifact via MmapCorpusReader...");
        let verify_start = Instant::now();
        let reader = MmapCorpusReader::open(&output_path)?;
        let header = reader.header();
        println!(
            "Header verification: magic = OK, version = {}, total_tokens = {}, vocab_size = {}",
            header.version, header.total_tokens, header.vocab_size
        );
        assert_eq!(header.total_tokens, total_tokens);
        assert_eq!(header.vocab_size, vocab_size);
        assert_eq!(reader.len(), total_tokens as usize);

        // Spot-check first and last tokens
        if reader.len() >= 2 {
            println!("First tokens: {:?}", &reader[..reader.len().min(8)]);
            println!(
                "Last tokens:  {:?}",
                &reader[reader.len().saturating_sub(8)..]
            );
            assert_eq!(reader[0], 0, "First token in corpus must be BOS (0)");
            assert_eq!(
                reader[reader.len() - 1],
                1,
                "Last token in corpus must be EOS (1)"
            );
        }

        // Fast throughput scan (zero copy)
        let mut check_sum: u64 = 0;
        for chunk in reader.chunks(65536) {
            for &tok in chunk {
                check_sum = check_sum.wrapping_add(tok as u64);
            }
        }
        let verify_elapsed = verify_start.elapsed();
        let verify_rate = total_tokens as f64 / verify_elapsed.as_secs_f64().max(1e-6);
        println!(
            "Memory-mapped scan passed! Checksum = {check_sum:#x} | Scan Throughput = {verify_rate:.0} tokens/sec ({:.2} MB/s)",
            (verify_rate * 2.0) / (1024.0 * 1024.0)
        );
    }

    Ok(())
}
