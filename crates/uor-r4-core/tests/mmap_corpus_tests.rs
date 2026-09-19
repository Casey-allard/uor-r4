//! Integration tests for memory-mapped binary corpus ingestion and zero-allocation streaming.

use std::fs::File;
use std::io::Write;
use std::time::Instant;

use uor_r4_core::native_geometric::mmap_corpus::{
    CorpusError, CorpusHeader, CorpusWriter, MmapCorpusReader, CORPUS_HEADER_SIZE, CORPUS_MAGIC,
    CORPUS_VERSION_1,
};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const TOKENIZER_PATH: &str =
    "/Users/casey.allard/uor-r4/.uor-models/research/issue-1014/export/tokenizer.json";

#[test]
fn test_corpus_header_roundtrip() {
    let header = CorpusHeader::new(123456789, 4096);
    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), CORPUS_HEADER_SIZE);
    assert_eq!(&bytes[0..4], &CORPUS_MAGIC);

    let parsed = CorpusHeader::from_bytes(&bytes).expect("Valid header must parse");
    assert_eq!(parsed.version, CORPUS_VERSION_1);
    assert_eq!(parsed.total_tokens, 123456789);
    assert_eq!(parsed.vocab_size, 4096);
}

#[test]
fn test_corpus_header_error_handling() {
    // Truncated header (< 64 bytes)
    let short = [0u8; 32];
    assert!(matches!(
        CorpusHeader::from_bytes(&short),
        Err(CorpusError::TruncatedHeader(32))
    ));

    // Invalid magic
    let mut bad_magic = CorpusHeader::new(100, 4096).to_bytes();
    bad_magic[0] = b'X';
    assert!(matches!(
        CorpusHeader::from_bytes(&bad_magic),
        Err(CorpusError::InvalidMagic(_))
    ));

    // Unsupported version
    let mut bad_version = CorpusHeader::new(100, 4096).to_bytes();
    bad_version[4] = 2; // version 2
    assert!(matches!(
        CorpusHeader::from_bytes(&bad_version),
        Err(CorpusError::UnsupportedVersion(2))
    ));
}

#[test]
fn test_corpus_writer_and_reader_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_mmap_corpus_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    let sample_tokens: Vec<u16> = (0..5000).map(|i| (i % 4096) as u16).collect();

    // Write file
    let written = CorpusWriter::write_file(&temp_file, 4096, &sample_tokens)
        .expect("write_file must succeed");
    assert_eq!(written, 5000);

    // Read and verify
    let reader = MmapCorpusReader::open(&temp_file).expect("MmapCorpusReader::open must succeed");
    assert_eq!(reader.total_tokens(), 5000);
    assert_eq!(reader.vocab_size(), 4096);
    assert_eq!(reader.len(), 5000);
    assert_eq!(reader.as_slice(), &sample_tokens[..]);

    // Test indexing
    for i in 0..sample_tokens.len() {
        assert_eq!(reader[i], sample_tokens[i]);
        assert_eq!(reader.get(i), Some(sample_tokens[i]));
    }
    assert_eq!(reader.get(5000), None);

    // Test subslice
    assert_eq!(reader.subslice(100, 50), Some(&sample_tokens[100..150]));
    assert_eq!(reader.subslice(4990, 20), None);

    // Test chunks iterator
    let chunk_size = 512;
    let mut chunk_count = 0;
    let mut token_count = 0;
    for chunk in reader.chunks(chunk_size) {
        chunk_count += 1;
        token_count += chunk.len();
        assert!(chunk.len() <= chunk_size);
    }
    assert_eq!(token_count, 5000);
    assert_eq!(chunk_count, (5000 + chunk_size - 1) / chunk_size);

    // Test sliding windows iterator
    let window_size = 128;
    let stride = 64;
    let mut window_count = 0;
    for window in reader.window_stride(window_size, stride) {
        assert_eq!(window.len(), window_size);
        window_count += 1;
    }
    let expected_windows = (5000 - window_size) / stride + 1;
    assert_eq!(window_count, expected_windows);

    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_corpus_reader_truncated_payload() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("test_truncated_payload_{}.u16", std::process::id()));

    // Write a header declaring 1,000 tokens (needs 2000 bytes payload), but only write 100 bytes
    let header = CorpusHeader::new(1000, 4096);
    let mut file = File::create(&temp_file).unwrap();
    file.write_all(&header.to_bytes()).unwrap();
    file.write_all(&[0u8; 100]).unwrap();
    file.flush().unwrap();
    drop(file);

    let result = MmapCorpusReader::open(&temp_file);
    assert!(matches!(
        result,
        Err(CorpusError::TruncatedPayload {
            expected_bytes: 2000,
            actual_bytes: 100
        })
    ));

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_bpe_roundtrip_tokenize_decode() {
    let tok_bytes = std::fs::read(TOKENIZER_PATH).expect("tokenizer.json must exist");
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .expect("HfBpeTokenizer must parse from json bytes");

    let test_sequences = [
        "Once upon a time there was a little boy named Ben.",
        "The girl found a shiny red apple in the quiet garden.",
        "A friendly puppy wagged its tail and ran through the sunny park.",
        "\"Look at that big mountain!\" shouted Tim happily.",
        "Every single animal in the forest lived in peace and harmony.",
    ];

    for &seq in &test_sequences {
        let token_ids = tokenizer.encode(seq);
        assert!(!token_ids.is_empty(), "Token sequence must not be empty");
        let decoded = tokenizer.decode(&token_ids);
        assert_eq!(
            seq, decoded,
            "Decoded text must match input sequence bit-exact"
        );
    }
}

#[test]
fn test_streaming_loader_throughput() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_throughput_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    // Create a corpus with 2,500,000 tokens (5 MB of tokens)
    let count = 2_500_000;
    let tokens: Vec<u16> = (0..count).map(|i| (i % 4096) as u16).collect();
    CorpusWriter::write_file(&temp_file, 4096, &tokens).expect("write_file must succeed");

    let reader = MmapCorpusReader::open(&temp_file).expect("open must succeed");
    assert_eq!(reader.len(), count);

    // Benchmark reading throughput over multiple passes
    let start = Instant::now();
    let mut sum: u64 = 0;
    let passes = 5;
    for _ in 0..passes {
        for chunk in reader.chunks(65536) {
            for &tok in chunk {
                sum = sum.wrapping_add(tok as u64);
            }
        }
    }
    let elapsed = start.elapsed();
    std::hint::black_box(sum);

    let total_tokens_scanned = (count as f64) * (passes as f64);
    let throughput_tokens_sec = total_tokens_scanned / elapsed.as_secs_f64();
    let mb_sec = (throughput_tokens_sec * 2.0) / (1024.0 * 1024.0);

    println!(
        "Throughput test: scanned {total_tokens_scanned:.0} tokens in {elapsed:.4?} -> {throughput_tokens_sec:.0} tokens/sec ({mb_sec:.2} MB/s)"
    );

    // Requirement: Verify streaming loader throughput >= 50,000,000 tokens/sec
    assert!(
        throughput_tokens_sec >= 50_000_000.0,
        "Streaming loader throughput {throughput_tokens_sec:.0} tok/s was below 50,000,000 tok/s requirement"
    );

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_window_stride_larger_than_window_no_underflow() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_underflow_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    // 10 tokens: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    let tokens: Vec<u16> = (0..10).collect();
    CorpusWriter::write_file(&temp_file, 4096, &tokens).expect("write_file must succeed");

    let reader = MmapCorpusReader::open(&temp_file).expect("open must succeed");
    assert_eq!(reader.len(), 10);

    // window_size = 2, stride = 5:
    // Windows should be [0, 1] (offset 0), [5, 6] (offset 5). Next offset 10 > 10 - 2, done.
    // Total 2 windows.
    let mut iter = reader.window_stride(2, 5);
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.size_hint(), (2, Some(2)));

    let w1 = iter.next().unwrap();
    assert_eq!(w1, &[0, 1]);
    assert_eq!(iter.len(), 1);
    assert_eq!(iter.size_hint(), (1, Some(1)));

    let w2 = iter.next().unwrap();
    assert_eq!(w2, &[5, 6]);
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));

    assert!(iter.next().is_none());
    assert_eq!(iter.len(), 0);

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_exact_size_iterator_consistency() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_exact_iter_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    let count = 100;
    let tokens: Vec<u16> = (0..count).map(|i| (i % 500) as u16).collect();
    CorpusWriter::write_file(&temp_file, 500, &tokens).expect("write_file must succeed");
    let reader = MmapCorpusReader::open(&temp_file).expect("open must succeed");

    // Test chunks iterator ExactSizeIterator
    for chunk_size in [1, 3, 7, 10, 25, 33, 50, 100, 150] {
        let mut iter = reader.chunks(chunk_size);
        let mut expected_len = iter.len();
        while let Some(chunk) = iter.next() {
            assert!(chunk.len() <= chunk_size);
            expected_len -= 1;
            assert_eq!(iter.len(), expected_len);
        }
        assert_eq!(expected_len, 0);
        assert_eq!(iter.len(), 0);
    }

    // Test windows iterator ExactSizeIterator across multiple stride/window configurations
    for (w_size, stride) in [
        (5, 1),
        (10, 5),
        (10, 10),
        (10, 20),
        (50, 25),
        (100, 1),
        (101, 1),
    ] {
        let mut iter = reader.window_stride(w_size, stride);
        let mut expected_len = iter.len();
        while let Some(window) = iter.next() {
            assert_eq!(window.len(), w_size);
            expected_len -= 1;
            assert_eq!(iter.len(), expected_len);
        }
        assert_eq!(expected_len, 0);
        assert_eq!(iter.len(), 0);
    }

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_range_inclusive_indexing() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_inclusive_idx_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    let tokens: Vec<u16> = (0..50).collect();
    CorpusWriter::write_file(&temp_file, 100, &tokens).expect("write_file must succeed");
    let reader = MmapCorpusReader::open(&temp_file).expect("open must succeed");

    // Test Index<RangeInclusive<usize>>
    assert_eq!(&reader[10..=15], &[10, 11, 12, 13, 14, 15]);
    // Test Index<RangeToInclusive<usize>>
    assert_eq!(&reader[..=4], &[0, 1, 2, 3, 4]);

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_writer_atomicity_on_token_overflow() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_writer_atomic_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    let mut writer = CorpusWriter::create(&temp_file, 100).expect("create must succeed");
    // Write valid tokens
    writer.write_tokens(&[1, 2, 3, 4, 5]).unwrap();

    // Attempt to write slice containing invalid token 150 (>= vocab_size 100)
    let bad_batch = [10, 20, 150, 30];
    let err = writer.write_tokens(&bad_batch);
    assert!(matches!(
        err,
        Err(CorpusError::TokenOverflow {
            token: 150,
            vocab_size: 100
        })
    ));

    // Finish writer and verify file contents were not corrupted by partial batch write
    let final_count = writer.finish().unwrap();
    assert_eq!(final_count, 5);

    let reader = MmapCorpusReader::open(&temp_file).unwrap();
    assert_eq!(reader.len(), 5);
    assert_eq!(reader.as_slice(), &[1, 2, 3, 4, 5]);

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_invalid_vocab_size_rejected() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_invalid_vocab_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    // vocab_size == 0
    assert!(matches!(
        CorpusWriter::create(&temp_file, 0),
        Err(CorpusError::InvalidVocabSize(0))
    ));

    // vocab_size > 65536
    assert!(matches!(
        CorpusWriter::create(&temp_file, 70000),
        Err(CorpusError::InvalidVocabSize(70000))
    ));

    // Header validation
    let mut header_bytes = CorpusHeader::new(10, 4096).to_bytes();
    header_bytes[16..20].copy_from_slice(&0u32.to_le_bytes());
    assert!(matches!(
        CorpusHeader::from_bytes(&header_bytes),
        Err(CorpusError::InvalidVocabSize(0))
    ));

    let mut header_bytes_large = CorpusHeader::new(10, 4096).to_bytes();
    header_bytes_large[16..20].copy_from_slice(&100_000u32.to_le_bytes());
    assert!(matches!(
        CorpusHeader::from_bytes(&header_bytes_large),
        Err(CorpusError::InvalidVocabSize(100_000))
    ));
}

#[test]
fn test_empty_corpus_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!(
        "test_empty_corpus_{}_{}.u16",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));

    let written = CorpusWriter::write_file(&temp_file, 4096, &[]).unwrap();
    assert_eq!(written, 0);

    let reader = MmapCorpusReader::open(&temp_file).unwrap();
    assert_eq!(reader.len(), 0);
    assert!(reader.is_empty());
    assert_eq!(reader.as_slice(), &[] as &[u16]);
    assert_eq!(reader.chunks(10).len(), 0);
    assert_eq!(reader.windows(5).len(), 0);

    let _ = std::fs::remove_file(temp_file);
}
