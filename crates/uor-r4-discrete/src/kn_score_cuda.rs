//! CUDA tests of the modified Kneser–Ney scorer (claude-421): the GPU masses
//! equal `KnTables::scored_masses` element by element, for N = 0..=4, on the
//! training stream, on another seed's stream and on a stream holding BOS in
//! every context slot. Run on wick's 4090 with `--release`.

use crate::kn_score::KnTables;
use crate::kn_score_device::KnScorer;
use crate::ngram_count::NGRAM_BOS;
use crate::ngram_count_oracle::oracle_adjusted;

/// The stream generator of kn_score.rs's tests, copied: a heavy head, a long
/// tail, one draw in 61 a BOS and one in 9 a copy of an earlier span.
fn kn_score_stream(seed: u64, len: usize) -> Vec<u16> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 16) as u32
    };
    let mut stream: Vec<u16> = vec![NGRAM_BOS];
    while stream.len() < len {
        let r = next();
        if (r >> 16) % 9 == 0 && stream.len() > 16 {
            let from = next() as usize % (stream.len() - 9);
            let run = 2 + (r >> 20) as usize % 8;
            for at in from..from + run {
                stream.push(stream[at]);
            }
            continue;
        }
        let product = (r % 16) * ((r >> 4) % 16) * ((r >> 8) % 16) * ((r >> 12) % 16);
        stream.push(if (r >> 24) % 61 == 0 {
            NGRAM_BOS
        } else {
            (5 + product / 16) as u16
        });
    }
    stream.truncate(len);
    stream[0] = NGRAM_BOS;
    stream
}

fn tables(tokens: &[u16], highest: u32) -> KnTables {
    let orders = (0..=highest)
        .map(|k| oracle_adjusted(tokens, k, highest).expect("adjusted counts"))
        .collect();
    KnTables::new(highest, orders).expect("tables")
}

/// A query with BOS at every distance 1..=5 before some target: a run of
/// training tokens, then BOS followed by 0..=5 training tokens, repeated.
fn bos_query(train: &[u16]) -> Vec<u16> {
    let mut query = vec![NGRAM_BOS];
    for (round, gap) in (0..60usize).zip((0..=5usize).cycle()) {
        let from = 1 + (round * 37) % (train.len() - 16);
        query.extend(train[from..from + 6].iter().filter(|&&t| t != NGRAM_BOS));
        query.push(NGRAM_BOS);
        query.extend(
            train[from + 6..from + 6 + gap]
                .iter()
                .filter(|&&t| t != NGRAM_BOS),
        );
    }
    query.push(7);
    query
}

#[test]
fn kn_score_cuda_matches_the_host_scorer() {
    let train = kn_score_stream(0x421a, 20_000);
    let other = kn_score_stream(0x421b, 20_000);
    let bos = bos_query(&train);
    for k in 1..=5usize {
        assert!(
            bos.windows(k + 1)
                .any(|w| w[0] == NGRAM_BOS && w[1..].iter().all(|&t| t != NGRAM_BOS)),
            "the BOS query lacks BOS at distance {k}"
        );
    }
    let mut by_order = Vec::new();
    for highest in 0..=4u32 {
        let host = tables(&train, highest);
        let scorer = KnScorer::new(0, &host).expect("scorer");
        for (name, query) in [("train", &train), ("other seed", &other), ("bos", &bos)] {
            let want = host.scored_masses(query).expect("host masses");
            let got = scorer.score(query).expect("gpu masses");
            assert_eq!(got.len(), want.len(), "N {highest} {name}: length");
            for (i, (g, w)) in got.iter().zip(&want).enumerate() {
                assert_eq!(g, w, "N {highest} {name}: scored target {i}");
            }
            eprintln!("N {highest} {name}: {} masses equal", want.len());
        }
        by_order.push(scorer.score(&other).expect("gpu masses"));
    }
    let (order_zero, order_two) = (&by_order[0], &by_order[2]);
    let differing = order_zero
        .iter()
        .zip(order_two)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing * 2 > order_zero.len(),
        "N = 2 differs from N = 0 on only {differing} of {} targets",
        order_zero.len()
    );
}

/// A stream whose copies are long (4..=27 tokens, one draw in 4), so grams of
/// orders up to 10 repeat with a falling spectrum n1 > n2 > n3 > n4 > 0.
fn long_copy_stream(seed: u64, len: usize) -> Vec<u16> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 16) as u32
    };
    let mut stream: Vec<u16> = vec![NGRAM_BOS];
    while stream.len() < len {
        let r = next();
        if (r >> 16) % 4 == 0 && stream.len() > 64 {
            let run = 4 + (r >> 20) as usize % 24;
            let from = next() as usize % (stream.len() - run);
            for at in from..from + run {
                stream.push(stream[at]);
            }
            continue;
        }
        let product = (r % 16) * ((r >> 4) % 16) * ((r >> 8) % 16) * ((r >> 12) % 16);
        stream.push(if (r >> 24) % 97 == 0 {
            NGRAM_BOS
        } else {
            (5 + product / 16) as u16
        });
    }
    stream.truncate(len);
    stream
}

/// Orders 5..=10 put context slots below bit 64: the 5th-latest token
/// straddles the key's two words and the 6th..10th sit in the low word.
#[test]
fn kn_score_cuda_matches_the_host_scorer_at_high_orders() {
    let train = long_copy_stream(0x421d, 40_000);
    let other = long_copy_stream(0x421e, 20_000);
    for highest in 5..=10u32 {
        let host = tables(&train, highest);
        let scorer = KnScorer::new(0, &host).expect("scorer");
        for (name, query) in [("train", &train), ("other seed", &other)] {
            let want = host.scored_masses(query).expect("host masses");
            let got = scorer.score(query).expect("gpu masses");
            assert_eq!(got.len(), want.len(), "N {highest} {name}: length");
            let first = (0..want.len()).find(|&i| got[i] != want[i]);
            assert_eq!(first, None, "N {highest} {name}: first differing target");
            eprintln!("N {highest} {name}: {} masses equal", want.len());
        }
        let (_, top, _) = host.order_tables(highest).expect("top order");
        assert!(
            top.iter().any(|t| t.count >= 2),
            "N {highest}: the top order holds no repeated gram"
        );
    }
}

/// kn_score.rs's small-vocabulary stream, copied: 40 ids, one draw in 300 a
/// BOS, so orders 0 and 1 take the fallback discounts.
fn small_vocabulary_stream(seed: u64, len: usize) -> Vec<u16> {
    let mut state = seed | 1;
    let mut stream = vec![NGRAM_BOS];
    while stream.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let r = (state >> 16) as u32;
        stream.push(if r % 300 == 0 {
            NGRAM_BOS
        } else {
            (5 + (r >> 9) % 40) as u16
        });
    }
    stream
}

/// claude-421: the GPU reads the fallback discounts as uploaded.
#[test]
fn kn_score_cuda_matches_the_host_scorer_with_fallback_discounts() {
    let train = small_vocabulary_stream(0x421f, 30_000);
    let other = small_vocabulary_stream(0x4220, 10_000);
    let host = tables(&train, 2);
    assert_eq!(host.discount_fallback(0), Some(true));
    assert_eq!(host.discount_fallback(2), Some(false));
    let scorer = KnScorer::new(0, &host).expect("scorer");
    for (name, query) in [("train", &train), ("other seed", &other)] {
        let want = host.scored_masses(query).expect("host masses");
        assert_eq!(scorer.score(query).expect("gpu masses"), want, "{name}");
    }
}

#[test]
fn kn_score_cuda_empty_and_bos_only_queries_score_nothing() {
    let train = kn_score_stream(0x421c, 6_000);
    let scorer = KnScorer::new(0, &tables(&train, 2)).expect("scorer");
    assert!(scorer
        .score(&[NGRAM_BOS, NGRAM_BOS])
        .expect("bos only")
        .is_empty());
    assert!(scorer.score(&[4096]).is_err(), "a 13-bit id is refused");
}

/// claude-421's 1% real-stream oracle: TRAIN's first 1% is counted on the GPU
/// through the filter and DEV-tune's first 1% is scored on the GPU; every
/// mass equals the independent CPU oracle `KnOracle` at N = 2. Each slice is
/// cut at a record start. DEV-test is sealed and is never read here.
/// Run: `UOR_KN_TRAIN=train.tokens.u16 UOR_KN_QUERY=dev-tune.tokens.u16
/// cargo test --release --features cuda --lib -- --ignored kn_score_cuda_real_slice`.
#[test]
#[ignore = "needs UOR_KN_TRAIN and UOR_KN_QUERY, claude-420's UORT streams"]
fn kn_score_cuda_real_slice_equals_the_cpu_oracle() {
    use crate::kn_filter::KnQuery;
    use crate::kn_score_oracle::KnOracle;
    use crate::ngram_count::{read_uort_tokens, NgramAssembler};
    use crate::ngram_count_device::NgramCounter;
    const N: u32 = 2;
    let read = |var: &str| {
        let path = std::env::var(var).unwrap_or_else(|_| panic!("{var} names a UORT stream"));
        read_uort_tokens(&std::fs::read(&path).expect("read stream")).expect("UORT stream")
    };
    let cut = |tokens: &[u16]| {
        let mark = tokens.len() / 100;
        tokens[..=mark]
            .iter()
            .rposition(|&t| t == NGRAM_BOS)
            .filter(|&at| at > 0)
            .expect("the first 1% holds a whole record")
    };
    let train_all = read("UOR_KN_TRAIN");
    let train = &train_all[..cut(&train_all)];
    let query_all = read("UOR_KN_QUERY");
    let query = &query_all[..cut(&query_all)];
    let wanted = KnQuery::from_stream(query, N).expect("query");
    let counter = NgramCounter::new(0, train).expect("counter");
    let mut orders = Vec::new();
    for k in 0..=N {
        let mut assembler = NgramAssembler::new_adjusted(k, false).expect("assembler");
        let mut filter = wanted.filter(k).expect("filter");
        counter
            .count_order_adjusted_with(k, N, 1 << 24, &mut |partition| {
                assembler.push(partition)?;
                filter.keep(partition)
            })
            .expect("count");
        let (digest, _, _) = assembler.finish();
        let (triples, contexts) = filter.finish();
        eprintln!(
            "real slice order {k}: grams {} contexts {} kept triples {} contexts {} n1..n4 {:?}",
            digest.grams,
            digest.contexts,
            triples.len(),
            contexts.len(),
            digest.count_of_counts
        );
        orders.push((digest, triples, contexts));
    }
    drop(counter);
    let tables = KnTables::new(N, orders).expect("tables");
    let gpu = KnScorer::new(0, &tables)
        .expect("scorer")
        .score(query)
        .expect("score");
    let oracle = KnOracle::new(train, N as usize);
    let expected: Vec<u64> = (0..query.len())
        .filter(|&j| query[j] != NGRAM_BOS)
        .map(|j| u64::try_from(oracle.mass(query, j)).expect("an oracle mass fits u64"))
        .collect();
    assert_eq!(gpu.len(), expected.len());
    let first = (0..gpu.len()).find(|&i| gpu[i] != expected[i]);
    assert_eq!(first, None, "the GPU path differs from the CPU oracle");
    let distinct: std::collections::BTreeSet<u64> = gpu.iter().copied().collect();
    assert!(
        distinct.len() > 1000,
        "the slice's masses are not degenerate"
    );
    eprintln!(
        "real slice: train {} tokens, query {} tokens, {} targets, gpu = oracle",
        train.len(),
        query.len(),
        gpu.len()
    );
}
