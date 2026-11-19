//! CUDA tests of the raw count lookups (claude-433): the GPU counts and totals
//! equal `RawTables::lookups` word for word, on hand-built tables at N = 2 and
//! 3, on tables counted by the GPU counter at N = 4, and on empty and BOS-only
//! queries. Run on wick's 4090 with `--release`.

use crate::kn_ceiling::RawTables;
use crate::kn_lookup_device::KnLookup;
use crate::ngram_count::{
    ngram_admitted, NgramAssembler, NgramContext, NgramOrderDigest, NgramTriple, PrimeRouteKey,
    NGRAM_BOS,
};
use crate::ngram_count_device::NgramCounter;
use std::collections::BTreeMap;

fn digest(order: u32) -> NgramOrderDigest {
    NgramOrderDigest {
        order,
        targets: 0,
        grams: 0,
        contexts: 0,
        count_of_counts: [0; 4],
        triple_sha256: String::new(),
        context_sha256: String::new(),
    }
}

/// Raw tables counted on the host from `train` with the admission rule.
fn hand_tables(train: &[u16], highest: u32) -> RawTables {
    let mut orders = Vec::new();
    for k in 0..=highest as usize {
        let mut grams: BTreeMap<(u128, u16), u32> = BTreeMap::new();
        let mut ctx: BTreeMap<u128, u64> = BTreeMap::new();
        for j in k..train.len() {
            if !ngram_admitted(train, j, k) {
                continue;
            }
            let key = PrimeRouteKey::from_context(&train[j - k..j])
                .expect("key")
                .word();
            *grams.entry((key, train[j])).or_default() += 1;
            *ctx.entry(key).or_default() += 1;
        }
        let triples = grams
            .into_iter()
            .map(|((key, next), count)| NgramTriple { key, next, count })
            .collect();
        let contexts = ctx
            .into_iter()
            .map(|(key, total)| NgramContext {
                key,
                total,
                followers: 0,
                n1: 0,
                n2: 0,
                n3_plus: 0,
            })
            .collect();
        orders.push((digest(k as u32), triples, contexts));
    }
    RawTables::new(highest, orders).expect("tables")
}

/// A pseudo-random stream: a heavy head, one draw in 61 a BOS and one in 9 a
/// copy of an earlier span.
fn stream(seed: u64, len: usize) -> Vec<u16> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 16) as u32
    };
    let mut out: Vec<u16> = vec![NGRAM_BOS];
    while out.len() < len {
        let r = next();
        if (r >> 16) % 9 == 0 && out.len() > 16 {
            let from = next() as usize % (out.len() - 9);
            let run = 2 + (r >> 20) as usize % 8;
            for at in from..from + run {
                out.push(out[at]);
            }
            continue;
        }
        let product = (r % 16) * ((r >> 4) % 16) * ((r >> 8) % 16);
        out.push(if (r >> 24) % 61 == 0 {
            NGRAM_BOS
        } else {
            (5 + product / 16) as u16
        });
    }
    out.truncate(len);
    out[0] = NGRAM_BOS;
    out
}

/// Device lookups of `query` equal the host's; returns the host's.
fn assert_device_equals_host(
    device: &KnLookup,
    tables: &RawTables,
    query: &[u16],
) -> (Vec<u32>, Vec<u64>) {
    let (host_counts, host_totals) = tables.lookups(query).expect("host lookups");
    let (gpu_counts, gpu_totals) = device.lookups(query).expect("device lookups");
    assert_eq!(gpu_counts.len(), host_counts.len(), "count words");
    assert_eq!(gpu_totals.len(), host_totals.len(), "total words");
    for (at, (g, h)) in gpu_counts.iter().zip(&host_counts).enumerate() {
        assert_eq!(g, h, "count at word {at}");
    }
    for (at, (g, h)) in gpu_totals.iter().zip(&host_totals).enumerate() {
        assert_eq!(g, h, "total at word {at}");
    }
    (host_counts, host_totals)
}

#[test]
fn kn_lookup_cuda_matches_host_on_hand_built_tables() {
    // Three BOS-separated records of training text.
    let train: Vec<u16> = vec![
        NGRAM_BOS, 5, 6, 7, 5, 6, 8, 5, 6, 7, NGRAM_BOS, 9, 5, 6, 7, 7, 9, NGRAM_BOS, 6, 7, 5, 6,
    ];
    // Seen spans, unseen tokens (40, 41), a BOS inside a context, and
    // targets at j < highest.
    let query: Vec<u16> = vec![
        5, 6, 7, 40, 6, 7, 5, NGRAM_BOS, 6, 7, 9, 41, 5, 6, 8, NGRAM_BOS, 7, 5, 6, 7, 7,
    ];
    for highest in [2u32, 3] {
        let tables = hand_tables(&train, highest);
        let device = KnLookup::new(0, &tables).expect("device");
        assert_eq!(device.highest(), highest);
        let (counts, totals) = assert_device_equals_host(&device, &tables, &query);
        let width = highest as usize + 1;
        let targets = query.iter().filter(|&&t| t != NGRAM_BOS).count();
        assert_eq!(totals.len(), targets * width);
        // Not vacuous: hits, seen contexts without the target, unseen
        // contexts, and k > j at the first target.
        assert!(counts.iter().any(|&a| a > 1));
        assert!(counts.iter().zip(&totals).any(|(&a, &t)| a == 0 && t > 0));
        assert!(totals[1..width].iter().all(|&t| t == 0));
        assert!(totals.chunks(width).any(|row| row[0] > 0 && row[2] == 0));
    }
}

#[test]
fn kn_lookup_cuda_matches_host_on_counted_tables() {
    // Orders above 4 put context tokens in the key's low word (bits below
    // 72), so N = 10 covers the word split and the low-word comparisons.
    let highest = 10u32;
    let train = stream(0x433a, 4_000);
    let other = stream(0x433b, 3_000);
    let counter = NgramCounter::new(0, &train).expect("counter");
    let mut orders = Vec::new();
    for k in 0..=highest {
        let mut assembler = NgramAssembler::new(k, true).expect("assembler");
        counter
            .count_order(k, 1 << 24, &mut assembler)
            .expect("count");
        orders.push(assembler.finish());
    }
    let tables = RawTables::new(highest, orders).expect("tables");
    let device = KnLookup::new(0, &tables).expect("device");
    for query in [&train, &other] {
        let (counts, totals) = assert_device_equals_host(&device, &tables, query);
        let width = highest as usize + 1;
        // Every order is reached with a nonzero count somewhere.
        for k in 0..width {
            assert!(
                counts.iter().skip(k).step_by(width).any(|&a| a > 0),
                "order {k} has no hit"
            );
        }
        assert!(totals.iter().any(|&t| t == 0));
    }
}

#[test]
fn kn_lookup_cuda_empty_and_bos_only_queries_return_nothing() {
    let train = stream(0x433c, 2_000);
    let tables = hand_tables(&train, 3);
    let device = KnLookup::new(0, &tables).expect("device");
    for query in [vec![], vec![NGRAM_BOS], vec![NGRAM_BOS; 5]] {
        let (counts, totals) = assert_device_equals_host(&device, &tables, &query);
        assert!(counts.is_empty() && totals.is_empty());
    }
}
