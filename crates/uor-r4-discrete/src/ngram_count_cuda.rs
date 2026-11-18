//! CUDA tests of the exact n-gram counter (claude-412): GPU records equal the
//! CPU oracle's byte for byte, the SHA does not move across partition budgets
//! or runs, and a consistent planted single-count change moves it. Run on wick's 4090
//! with `--release` (the ten-million-token oracle is slow in a debug build).

// The seeds are written `0x<unit>_<case>` (`0x412_1` is claude-412's first case), so their
// digit groups name the unit and the case rather than bytes.
#![allow(clippy::unusual_byte_groupings)]

use crate::ngram_count::{
    NgramAssembler, NgramContext, NgramOrderDigest, NgramPartition, NgramTriple, PartitionRange,
    PrimeRouteKey, NGRAM_BOS,
};
use crate::ngram_count_device::{CountPhases, NgramCounter};
use crate::ngram_count_oracle::{oracle_adjusted, oracle_count, plant_one_count};

/// A deterministic stream: one in four ids uniform over 4,096, the rest over
/// `narrow` ids, so high orders repeat; `narrow = 0` gives all-uniform ids.
/// Every 61st id is BOS (0), a planted record start, so the device's
/// document-boundary rule (claude-415) meets BOS in every slot.
fn stream(len: usize, seed: u64, narrow: u32) -> Vec<u16> {
    stream_bos_every(len, seed, narrow, 61)
}

/// [`stream`] with BOS every `every` ids (claude-429: every 7 makes the
/// document starts and the BOS-interior rule of the left digit dense).
fn stream_bos_every(len: usize, seed: u64, narrow: u32, every: usize) -> Vec<u16> {
    let mut state = seed | 1;
    (0..len)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let r = (state >> 20) as u32;
            if i % every == 0 {
                return 0;
            }
            (if narrow == 0 || r.is_multiple_of(4) { r % 4096 } else { r % narrow }) as u16
        })
        .collect()
}

fn gpu(
    counter: &NgramCounter,
    order: u32,
    budget: u64,
) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<crate::ngram_count::NgramContext>, usize) {
    let mut assembler = NgramAssembler::new(order, true).expect("assembler");
    let plan = counter.count_order(order, budget, &mut assembler).expect("count");
    let (digest, triples, contexts) = assembler.finish();
    (digest, triples, contexts, plan.len())
}

#[test]
fn ngram_count_cuda_matches_the_oracle() {
    for (len, seed, narrow) in [
        (100_000usize, 0x412_1u64, 0u32),
        (100_000, 0x412_2, 13),
        (1_000_000, 0x412_3, 13),
        (10_000_000, 0x412_4, 29),
    ] {
        let tokens = stream(len, seed, narrow);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for order in 0..=8u32 {
            let (digest, triples, contexts, _) = gpu(&counter, order, u64::MAX);
            let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
            assert_eq!(digest, want, "digest, len {len} narrow {narrow} order {order}");
            assert!(triples == want_triples, "triples, len {len} order {order}");
            assert!(contexts == want_contexts, "contexts, len {len} order {order}");
            eprintln!(
                "match len {len} narrow {narrow} order {order}: grams {} contexts {} n1..n4 {:?} sha {}",
                digest.grams, digest.contexts, digest.count_of_counts, digest.triple_sha256
            );
        }
    }
}

#[test]
fn ngram_count_cuda_max_order_and_short_streams() {
    let tokens = stream(5_000, 0x412_5, 3);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in [9u32, 10] {
        let (digest, triples, contexts, _) = gpu(&counter, order, 777);
        let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
        assert_eq!(digest, want);
        assert!(triples == want_triples && contexts == want_contexts);
    }
    let short = [4095u16, 0, 4095];
    let counter = NgramCounter::new(0, &short).expect("counter");
    for order in 0..=4u32 {
        let (digest, _, _, _) = gpu(&counter, order, u64::MAX);
        let (oracle_digest, ..) = oracle_count(&short, order).expect("oracle");
        assert_eq!(digest, oracle_digest, "order {order}");
    }
    assert!(NgramCounter::new(0, &[1, 4096]).is_err());
}

#[test]
fn ngram_count_cuda_sha_is_stable_across_budgets_and_runs() {
    let tokens = stream(1_000_000, 0x412_6, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=8u32 {
        let (whole, _, _, one) = gpu(&counter, order, u64::MAX);
        let (again, _, _, _) = gpu(&counter, order, u64::MAX);
        assert_eq!(one, 1);
        assert_eq!(whole, again, "two runs, order {order}");
        for budget in [60_000u64, 9_000] {
            let (split, _, _, partitions) = gpu(&counter, order, budget);
            assert!(partitions > 1, "budget {budget} gave one partition");
            assert_eq!(split, whole, "order {order} budget {budget}");
            eprintln!(
                "stable order {order} budget {budget}: {partitions} partitions, sha {}",
                split.triple_sha256
            );
        }
    }
}

#[test]
fn ngram_count_cuda_sha_covers_a_planted_single_count_change() {
    let tokens = stream(100_000, 0x412_7, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    let (want, want_triples, _) = oracle_count(&tokens, 3).expect("oracle");
    let plan = counter.plan(3, u64::MAX).expect("plan");
    let mut partition = counter.count_partition(3, &plan[0]).expect("count");
    let at = partition.triples.len() / 2;
    // A consistent plant: the assembler refuses a count whose context record
    // and n1..n4 do not match; this shows the SHA covers the count field.
    plant_one_count(&mut partition, at);
    let mut assembler = NgramAssembler::new(3, true).expect("assembler");
    assembler.push(&partition).expect("push");
    let (digest, triples, _) = assembler.finish();
    assert_ne!(digest.triple_sha256, want.triple_sha256);
    let differing = triples.iter().zip(&want_triples).filter(|(a, b)| a != b).count();
    assert_eq!(differing, 1);
}

/// GPU = oracle on a real token stream, orders 0..5, plus two GPU runs equal.
/// Run explicitly: `UOR_NGRAM_STREAM=path.u16 cargo test ... -- --ignored
/// ngram_count_cuda_real_stream`. Fails when the variable is unset.
#[test]
#[ignore = "needs UOR_NGRAM_STREAM, a UORT token stream of at least 1M tokens"]
fn ngram_count_cuda_real_stream_matches_the_oracle() {
    let path = std::env::var("UOR_NGRAM_STREAM").expect("UOR_NGRAM_STREAM names a UORT stream");
    let tokens = crate::ngram_count::read_uort_tokens(&std::fs::read(&path).expect("read"))
        .expect("UORT tokens");
    assert!(tokens.len() >= 1_000_000, "stream holds {} tokens", tokens.len());
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=5u32 {
        let (digest, triples, contexts, partitions) = gpu(&counter, order, 4_000_000);
        let (again, _, _, _) = gpu(&counter, order, 4_000_000);
        let (want, want_triples, want_contexts) = oracle_count(&tokens, order).expect("oracle");
        assert_eq!(digest, want, "digest, order {order}");
        assert_eq!(again, digest, "second GPU run, order {order}");
        assert!(triples == want_triples && contexts == want_contexts, "records, order {order}");
        eprintln!(
            "real tokens {} order {order}: partitions {partitions} grams {} contexts {} n1..n4 {:?} gpu=oracle sha {} ctx {}",
            tokens.len(),
            digest.grams,
            digest.contexts,
            digest.count_of_counts,
            digest.triple_sha256,
            digest.context_sha256
        );
    }
}

/// The adjusted counts of `order` under highest order `highest` (claude-418).
fn gpu_adjusted(
    counter: &NgramCounter,
    order: u32,
    highest: u32,
    budget: u64,
) -> (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>, usize) {
    let mut assembler = NgramAssembler::new_adjusted(order, true).expect("assembler");
    let plan = counter.count_order_adjusted(order, highest, budget, &mut assembler).expect("count");
    let (digest, triples, contexts) = assembler.finish();
    (digest, triples, contexts, plan.len())
}

/// Device adjusted counts = the host oracle's, which builds a(g) from sets of
/// left tokens without backing off any triple. Orders 0..3 under N = 3, at one
/// partition and at a budget that forces several. Order 0 sums every order-1
/// partition; orders 1 and 2 pair each range with order k+1 over the same tokens.
#[test]
fn ngram_count_cuda_adjusted_matches_the_oracle() {
    for (len, seed, narrow) in [(100_000usize, 0x418_1u64, 13u32), (1_000_000, 0x418_2, 29)] {
        let tokens = stream(len, seed, narrow);
        assert_eq!(tokens[0], NGRAM_BOS, "rule 4 needs a stream that starts with BOS");
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for order in 0..=3u32 {
            let (want, want_triples, want_contexts) = oracle_adjusted(&tokens, order, 3).expect("oracle");
            let (_, raw_triples, _) = oracle_count(&tokens, order).expect("raw oracle");
            // Not vacuous: below N the adjusted counts differ from the raw
            // ones; at N they are the raw counts. (Compare the triples: the
            // adjusted SHA sits in its own domain, so it differs even at N.)
            assert_eq!(order < 3, want_triples != raw_triples, "order {order}");
            for budget in [u64::MAX, 9_000] {
                let (digest, triples, contexts, partitions) = gpu_adjusted(&counter, order, 3, budget);
                if budget != u64::MAX {
                    assert!(partitions > 1, "budget {budget} gave one partition, order {order}");
                }
                assert_eq!(digest, want, "digest, len {len} order {order} budget {budget}");
                assert!(triples == want_triples, "triples, len {len} order {order} budget {budget}");
                assert!(contexts == want_contexts, "contexts, len {len} order {order} budget {budget}");
                eprintln!(
                    "adjusted len {len} order {order} budget {budget}: {partitions} partitions, grams {} n1..n4 {:?} sha {}",
                    digest.grams, digest.count_of_counts, digest.triple_sha256
                );
            }
        }
    }
}

#[test]
fn ngram_count_cuda_adjusted_sha_is_stable_across_budgets_and_runs() {
    let tokens = stream(1_000_000, 0x418_3, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for highest in [3u32, 8] {
        for order in 0..=highest {
            let (whole, ..) = gpu_adjusted(&counter, order, highest, u64::MAX);
            let (again, ..) = gpu_adjusted(&counter, order, highest, u64::MAX);
            assert_eq!(whole, again, "two runs, order {order} N {highest}");
            for budget in [60_000u64, 9_000] {
                let (split, _, _, partitions) = gpu_adjusted(&counter, order, highest, budget);
                assert!(partitions > 1, "budget {budget} gave one partition");
                assert_eq!(split, whole, "order {order} N {highest} budget {budget}");
            }
        }
    }
}

/// Device adjusted = oracle on a real token stream: orders 0..3 under N = 3 on
/// its first 2M tokens, then the whole stream's adjusted SHA across two budgets
/// and two runs. Run explicitly: `UOR_NGRAM_STREAM=path.u16 cargo test ...
/// -- --ignored ngram_count_cuda_real_stream_adjusted`.
#[test]
#[ignore = "needs UOR_NGRAM_STREAM, a UORT token stream of at least 2M tokens"]
fn ngram_count_cuda_real_stream_adjusted_matches_the_oracle() {
    let path = std::env::var("UOR_NGRAM_STREAM").expect("UOR_NGRAM_STREAM names a UORT stream");
    let tokens = crate::ngram_count::read_uort_tokens(&std::fs::read(&path).expect("read"))
        .expect("UORT tokens");
    assert!(tokens.len() >= 2_000_000, "stream holds {} tokens", tokens.len());
    assert_eq!(tokens[0], NGRAM_BOS, "a UORT stream starts with BOS");
    let slice = &tokens[..2_000_000];
    let counter = NgramCounter::new(0, slice).expect("counter");
    for order in 0..=3u32 {
        let (want, want_triples, want_contexts) = oracle_adjusted(slice, order, 3).expect("oracle");
        let (digest, triples, contexts, partitions) = gpu_adjusted(&counter, order, 3, 1_000_000);
        assert_eq!(digest, want, "slice digest, order {order}");
        assert!(triples == want_triples && contexts == want_contexts, "slice records, order {order}");
        eprintln!(
            "real slice order {order}: partitions {partitions} grams {} n1..n4 {:?} gpu=oracle sha {}",
            digest.grams, digest.count_of_counts, digest.triple_sha256
        );
    }
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for order in 0..=3u32 {
        let (first, _, _, partitions) = gpu_adjusted(&counter, order, 3, 4_000_000);
        let (again, ..) = gpu_adjusted(&counter, order, 3, 4_000_000);
        let (split, ..) = gpu_adjusted(&counter, order, 3, 1_000_000);
        assert_eq!(again, first, "second GPU run, order {order}");
        assert_eq!(split, first, "budget 1M vs 4M, order {order}");
        eprintln!(
            "real tokens {} order {order}: partitions {partitions} grams {} n1..n4 {:?} sha {} ctx {}",
            tokens.len(),
            first.grams,
            first.count_of_counts,
            first.triple_sha256,
            first.context_sha256
        );
    }
}

/// claude-423: the adjusted counter's phase totals are recorded and reset.
/// At order 1 under N = 3 every planned range is counted for order 1 itself
/// and again for order 2's extension runs, so both split totals and the GPU
/// phases are nonzero, at least one partition is counted per range, and a
/// second `take_phases` reads all zeros.
#[test]
fn ngram_count_cuda_phase_totals_are_recorded_and_reset() {
    let tokens = stream(1_000_000, 0x423_1, 29);
    let mut counter = NgramCounter::new(0, &tokens).expect("counter");
    let (ordinary, ..) = gpu_adjusted(&counter, 1, 3, 9_000);
    assert_eq!(counter.take_phases(), CountPhases::default(), "default counting has no timing");
    counter.enable_phase_timing();
    assert_eq!(counter.take_phases(), CountPhases::default(), "enabling starts at zero");
    let (profiled, _, _, planned) = gpu_adjusted(&counter, 1, 3, 9_000);
    assert_eq!(ordinary, profiled, "profiling preserves the order digest");
    let phases = counter.take_phases();
    assert!(planned > 1, "budget 9,000 gave one partition");
    assert!(phases.partitions >= planned as u64, "{phases:?}");
    for (name, ns) in [
        ("own count", phases.own_count_ns),
        ("extension count", phases.extension_count_ns),
        ("histogram", phases.histogram_ns),
        ("select", phases.select_ns),
        ("sort", phases.sort_ns),
        ("emit", phases.emit_ns),
        ("download", phases.download_ns),
    ] {
        assert!(ns > 0, "{name} recorded no time: {phases:?}");
    }
    assert_eq!(counter.take_phases(), CountPhases::default(), "take_phases resets the totals");
    eprintln!("phase totals at order 1 under N = 3: {phases:?}");
}

/// A BOS-dense stream (claude-425): every id is BOS with chance 1/4, else one
/// of `narrow` ids, so order 10 is admitted only on long BOS-free runs.
fn bos_dense_stream(len: usize, seed: u64, narrow: u32) -> Vec<u16> {
    let mut state = seed | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let r = (state >> 20) as u32;
            if r.is_multiple_of(4) { 0 } else { (1 + (r >> 2) % narrow) as u16 }
        })
        .collect()
}

/// Each partition's bucket-slice positions are the whole-stream positions as
/// a set, and the gram sort makes them the identical sorted array.
fn assert_slices_match_stream(counter: &NgramCounter, order: u32, budget: u64, what: &str) -> usize {
    let plan = counter.plan(order, budget).expect("plan");
    for range in &plan {
        let stream_pos = counter.selected_positions(order, range, true).expect("stream select");
        let slice_pos = counter.selected_positions(order, range, false).expect("slice select");
        assert!(stream_pos.windows(2).all(|w| w[0] < w[1]), "{what}: stream select ascends");
        let mut as_set = slice_pos.clone();
        as_set.sort_unstable();
        assert!(as_set == stream_pos, "{what} order {order} range {range:?}: slice set differs");
        if stream_pos.is_empty() {
            continue;
        }
        let from_stream = counter.sorted_positions(order, &stream_pos).expect("sort stream");
        let from_slice = counter.sorted_positions(order, &slice_pos).expect("sort slice");
        assert!(from_stream == from_slice, "{what} order {order} range {range:?}: sorted arrays differ");
    }
    plan.len()
}

#[test]
fn ngram_count_cuda_bucket_slices_equal_the_stream_select() {
    for (len, seed, narrow) in [(200_000usize, 0x425_1u64, 0u32), (1_000_000, 0x425_2, 29)] {
        let tokens = stream(len, seed, narrow);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for budget in [9_000u64, 60_000] {
            for order in 1..=10u32 {
                let what = format!("len {len} narrow {narrow} budget {budget}");
                let partitions = assert_slices_match_stream(&counter, order, budget, &what);
                if budget == 9_000 && len == 1_000_000 {
                    assert!(partitions > 1, "{what} order {order}: one partition");
                }
            }
        }
    }
    let tokens = bos_dense_stream(500_000, 0x425_3, 5);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    for budget in [9_000u64, 60_000] {
        assert_slices_match_stream(&counter, 10, budget, &format!("BOS-dense budget {budget}"));
    }
    let admitted: u64 = counter.plan(10, 9_000).expect("plan").iter().map(|r| r.targets).sum();
    assert!(admitted > 0, "the BOS-dense stream admits no order-10 target");
}

#[test]
fn ngram_count_cuda_bucket_offset_miscount_is_refused() {
    let tokens = stream(100_000, 0x425_4, 13);
    let mut counter = NgramCounter::new(0, &tokens).expect("counter");
    let histogram = counter.partition_histogram(1).expect("histogram");
    let range = PartitionRange { lo: 5, hi: 4096, targets: histogram[5..].iter().sum() };
    let control = counter.selected_positions(1, &range, false).expect("unplanted slice select");
    assert_eq!(control.len() as u64, range.targets);
    // Bucket 5 now starts 50 positions inside bucket 4: the slice gains 50
    // positions whose slot-1 token is 4, and the exact count refuses it.
    counter.plant_bucket_start_shift(5, 50).expect("plant");
    let planted = counter.selected_positions(1, &range, false);
    assert!(planted.is_err(), "a planted bucket-offset miscount was accepted");
    let message = format!("{:?}", planted.err());
    assert!(message.contains("planned"), "refused for another reason: {message}");
}

/// One order's assembled adjusted counts.
type Assembled = (NgramOrderDigest, Vec<NgramTriple>, Vec<NgramContext>);

/// One sweep's sink.
type SweepSink<'a> = dyn FnMut(u32, NgramPartition) -> crate::Result<()> + 'a;

/// claude-425: every order 0..=N from one sweep, one assembler per order;
/// partitions of orders outside `keep` are not assembled. Returns the
/// assembled orders and the shared plan's length.
fn assemble_sweep(
    highest: u32,
    keep: &[u32],
    sweep: impl FnOnce(&mut SweepSink) -> crate::Result<Vec<PartitionRange>>,
) -> crate::Result<(Vec<Assembled>, usize)> {
    let mut assemblers = (0..=highest)
        .map(|order| NgramAssembler::new_adjusted(order, true))
        .collect::<crate::Result<Vec<_>>>()?;
    let plan = sweep(&mut |order, partition| match keep.contains(&order) {
        true => assemblers[order as usize].push_owned(partition),
        false => Ok(()),
    })?;
    Ok((assemblers.into_iter().map(NgramAssembler::finish).collect(), plan.len()))
}

/// The device sweep (claude-429), assembled.
fn gpu_sweep(
    counter: &NgramCounter,
    highest: u32,
    budget: u64,
    keep: &[u32],
) -> crate::Result<(Vec<Assembled>, usize)> {
    assemble_sweep(highest, keep, |sink| {
        counter.count_adjusted_sweep(highest, budget, sink)
    })
}

/// The host-runs sweep with `tamper` on each raw partition, assembled.
fn gpu_sweep_host(
    counter: &NgramCounter,
    highest: u32,
    budget: u64,
    tamper: &mut dyn FnMut(u32, &mut NgramPartition),
    keep: &[u32],
) -> crate::Result<(Vec<Assembled>, usize)> {
    assemble_sweep(highest, keep, |sink| {
        counter.count_adjusted_sweep_tampered(highest, budget, tamper, sink)
    })
}

fn every_order(highest: u32) -> Vec<u32> {
    (0..=highest).collect()
}

/// The sweep's adjusted counts of every order equal the host oracle's, under
/// N = 1 and N = 3, at one partition and at budgets that force several.
#[test]
fn ngram_count_cuda_sweep_matches_the_oracle() {
    for (len, seed, narrow) in [(100_000usize, 0x425_1u64, 13u32), (1_000_000, 0x425_2, 29)] {
        let tokens = stream(len, seed, narrow);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for highest in [1u32, 3] {
            let want = (0..=highest)
                .map(|order| oracle_adjusted(&tokens, order, highest).expect("oracle"))
                .collect::<Vec<_>>();
            for budget in [u64::MAX, 60_000, 9_000] {
                let (orders, partitions) =
                    gpu_sweep(&counter, highest, budget, &every_order(highest)).expect("sweep");
                if budget != u64::MAX {
                    assert!(partitions > 1, "budget {budget} gave one partition, len {len}");
                }
                for (order, (got, want)) in orders.iter().zip(&want).enumerate() {
                    assert_eq!(got.0, want.0, "digest, len {len} N {highest} order {order} budget {budget}");
                    assert!(got.1 == want.1 && got.2 == want.2, "records, len {len} N {highest} order {order} budget {budget}");
                }
            }
        }
    }
}

/// The sweep counts each order once: the extension phase stays zero, and it
/// counts no more nonempty partitions than N per shared range plus order 0's
/// plan. Profiling leaves the records unchanged. The device sweep
/// (claude-429) records no runs and no adjust time; the host-runs sweep
/// records both and assembles the same records.
#[test]
fn ngram_count_cuda_sweep_counts_no_extension() {
    let tokens = stream(1_000_000, 0x425_3, 29);
    let mut counter = NgramCounter::new(0, &tokens).expect("counter");
    let (ordinary, _) = gpu_sweep(&counter, 3, 9_000, &every_order(3)).expect("sweep");
    let unigram_partitions = counter.plan(0, 9_000).expect("plan").len() as u64;
    counter.enable_phase_timing();
    let (profiled, shared) = gpu_sweep(&counter, 3, 9_000, &every_order(3)).expect("sweep");
    let phases = counter.take_phases();
    assert!(ordinary == profiled, "profiling preserves the sweep's records");
    assert_eq!(phases.extension_count_ns, 0, "{phases:?}");
    assert!(
        phases.own_count_ns > 0,
        "own count recorded no time: {phases:?}"
    );
    assert_eq!((phases.runs_ns, phases.adjust_ns), (0, 0), "{phases:?}");
    assert!(shared > 1, "budget 9,000 gave one shared partition");
    assert!(
        phases.partitions <= 3 * shared as u64 + unigram_partitions,
        "{phases:?}, {shared} shared ranges"
    );
    eprintln!("sweep phase totals under N = 3: {phases:?}");
    let (host, _) =
        gpu_sweep_host(&counter, 3, 9_000, &mut |_, _| {}, &every_order(3)).expect("host sweep");
    let phases = counter.take_phases();
    assert!(
        host == ordinary,
        "the host-runs sweep assembles the device sweep's records"
    );
    assert_eq!(phases.extension_count_ns, 0, "{phases:?}");
    for (name, ns) in [
        ("own count", phases.own_count_ns),
        ("runs", phases.runs_ns),
        ("adjust", phases.adjust_ns),
    ] {
        assert!(ns > 0, "{name} recorded no time: {phases:?}");
    }
    eprintln!("host-runs sweep phase totals under N = 3: {phases:?}");
}

/// claude-429: the device sweep's adjusted counts of every order equal the
/// host oracle's at N in {1, 3, 10}, at one partition and at budgets that force
/// several, on a stream with BOS every 61 ids and on one with BOS every 7.
/// N = 10 runs on 200,000 tokens, since its oracle is slow on a million.
#[test]
fn ngram_count_cuda_device_sweep_matches_the_oracle() {
    for (len, seed, every, highest) in [
        (1_000_000usize, 0x429_1u64, 61usize, 1u32),
        (1_000_000, 0x429_1, 61, 3),
        (200_000, 0x429_2, 61, 10),
        (1_000_000, 0x429_3, 7, 1),
        (1_000_000, 0x429_3, 7, 3),
        (200_000, 0x429_4, 7, 10),
    ] {
        let tokens = stream_bos_every(len, seed, 29, every);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        let want = (0..=highest)
            .map(|order| oracle_adjusted(&tokens, order, highest).expect("oracle"))
            .collect::<Vec<_>>();
        for budget in [u64::MAX, 60_000, 9_000] {
            let (orders, partitions) =
                gpu_sweep(&counter, highest, budget, &every_order(highest)).expect("sweep");
            if budget != u64::MAX {
                assert!(
                    partitions > 1,
                    "budget {budget} gave one partition, len {len}"
                );
            }
            for (order, (got, want)) in orders.iter().zip(&want).enumerate() {
                let case = format!(
                    "len {len} BOS every {every} N {highest} order {order} budget {budget}"
                );
                assert_eq!(got.0, want.0, "digest, {case}");
                assert!(got.1 == want.1 && got.2 == want.2, "records, {case}");
            }
        }
    }
}

/// claude-429: the device sweep equals the host-runs sweep byte for byte: the
/// same shared plan, and per order the same digest and records.
#[test]
fn ngram_count_cuda_device_sweep_equals_the_host_sweep() {
    for (len, seed, every, highest) in [
        (1_000_000usize, 0x429_5u64, 61usize, 3u32),
        (200_000, 0x429_6, 61, 10),
        (1_000_000, 0x429_7, 7, 3),
        (200_000, 0x429_8, 7, 10),
    ] {
        let tokens = stream_bos_every(len, seed, 29, every);
        let counter = NgramCounter::new(0, &tokens).expect("counter");
        for budget in [u64::MAX, 60_000, 9_000] {
            let case = format!("len {len} BOS every {every} N {highest} budget {budget}");
            let keep = every_order(highest);
            let (device, shared) =
                gpu_sweep(&counter, highest, budget, &keep).expect("device sweep");
            let (host, host_shared) =
                gpu_sweep_host(&counter, highest, budget, &mut |_, _| {}, &keep)
                    .expect("host sweep");
            assert_eq!(shared, host_shared, "shared plan, {case}");
            for (order, (got, want)) in device.iter().zip(&host).enumerate() {
                assert_eq!(got.0, want.0, "digest, {case} order {order}");
                assert!(
                    got.1 == want.1 && got.2 == want.2,
                    "records, {case} order {order}"
                );
            }
        }
    }
}

/// claude-429: a planted rule-4 violation (every left flag of one adjusted
/// partition left zero) is refused with the host path's message, naming the
/// partition's first gram whose oldest slot is not BOS; the plant is spent by
/// that refusal, and the device sweep refuses it as well.
#[test]
fn ngram_count_cuda_planted_rule4_is_refused() {
    let tokens = stream(100_000, 0x429_9, 13);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    let range = counter.plan(2, u64::MAX).expect("plan").remove(0);
    let raw = counter.count_partition(2, &range).expect("raw");
    let first = raw
        .triples
        .iter()
        .find(|triple| {
            PrimeRouteKey::from_word(triple.key).expect("key").slot(2) != Some(NGRAM_BOS)
        })
        .expect("a gram that is not a document start");
    let clean = counter
        .count_partition_adjusted(2, 3, &range)
        .expect("unplanted adjust");
    assert_eq!(clean.triples.len(), raw.triples.len());
    counter.plant_rule4_violation();
    let message = format!("{:?}", counter.count_partition_adjusted(2, 3, &range).err());
    let named = format!(
        "order-2 gram (key {:#x}, next {}) has no counted left extension",
        first.key, first.next
    );
    assert!(
        message.contains("(rule 4)") && message.contains(&named),
        "refused otherwise: {message}"
    );
    let again = counter
        .count_partition_adjusted(2, 3, &range)
        .expect("the plant is spent");
    assert!(
        again.triples == clean.triples && again.contexts == clean.contexts,
        "an unplanted adjust after the refusal"
    );
    counter.plant_rule4_violation();
    let message = format!(
        "{:?}",
        gpu_sweep(&counter, 3, 60_000, &every_order(3)).err()
    );
    assert!(
        message.contains("(rule 4)"),
        "the sweep refused otherwise: {message}"
    );
}

/// Order k reads only the (key, next) of order k+1's raw triples. A planted
/// +1 on one raw order-3 count (N = 3, so order 3 keeps its raw counts) moves
/// order 3's digest and leaves orders 0..2 equal; dropping a raw order-3
/// triple that is its suffix's only left type moves order 2's digest or is
/// refused. Only order 2 is assembled in the drop run, so a refusal comes
/// from the sweep, not from order 3's assembler. The tamper hook acts on the
/// raw partitions of the host-runs sweep (claude-429).
#[test]
fn ngram_count_cuda_sweep_orders_read_only_their_extension_types() {
    let tokens = stream(1_000_000, 0x425_4, 29);
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    let (base, _) =
        gpu_sweep_host(&counter, 3, 60_000, &mut |_, _| {}, &every_order(3)).expect("sweep");
    let mut planted = false;
    let mut plant = |order: u32, raw: &mut NgramPartition| {
        if order == 3 && !planted && !raw.triples.is_empty() {
            plant_one_count(raw, 0);
            planted = true;
        }
    };
    let (moved, _) =
        gpu_sweep_host(&counter, 3, 60_000, &mut plant, &every_order(3)).expect("planted sweep");
    assert!(planted, "no order-3 triple was planted");
    let (moved_top, base_top) = (&moved[3], &base[3]);
    assert_ne!(moved_top.0, base_top.0, "a raw order-3 count change must move order 3");
    for order in 0..3 {
        let (moved_k, base_k) = (&moved[order], &base[order]);
        assert_eq!(moved_k.0, base_k.0, "order {order} must not read order-3 counts");
    }
    let mut dropped = false;
    let mut drop_one = |order: u32, raw: &mut NgramPartition| {
        if order != 3 || dropped {
            return;
        }
        let suffix = |triple: &NgramTriple| {
            let key = PrimeRouteKey::from_word(triple.key).expect("key").back_off().expect("order 3");
            (key.word(), triple.next)
        };
        let mut types = std::collections::HashMap::new();
        for triple in &raw.triples {
            *types.entry(suffix(triple)).or_insert(0u32) += 1;
        }
        if let Some(at) = raw.triples.iter().position(|triple| types[&suffix(triple)] == 1) {
            raw.triples.remove(at);
            dropped = true;
        }
    };
    match gpu_sweep_host(&counter, 3, 60_000, &mut drop_one, &[2]) {
        Ok((orders, _)) => {
            let (dropped_two, base_two) = (&orders[2], &base[2]);
            assert_ne!(dropped_two.0, base_two.0, "dropping a lone left type must move order 2");
        }
        Err(error) => eprintln!("dropped left type refused: {error}"),
    }
    assert!(dropped, "no lone order-3 left type was dropped");
}

/// Sweep = oracle on a real stream's first 2M tokens, orders 0..3 under N = 3;
/// on the whole stream the sweep's assembled records equal
/// `count_order_adjusted`'s per order. The partition seams differ (orders >= 2
/// now use order 1's plan), so the assembled digests and records are compared.
/// Run explicitly: `UOR_NGRAM_STREAM=path.u16 cargo test ... -- --ignored
/// ngram_count_cuda_real_stream_sweep`.
#[test]
#[ignore = "needs UOR_NGRAM_STREAM, a UORT token stream of at least 2M tokens"]
fn ngram_count_cuda_real_stream_sweep_matches_the_adjusted_counter() {
    let path = std::env::var("UOR_NGRAM_STREAM").expect("UOR_NGRAM_STREAM names a UORT stream");
    let tokens = crate::ngram_count::read_uort_tokens(&std::fs::read(&path).expect("read"))
        .expect("UORT tokens");
    assert!(tokens.len() >= 2_000_000, "stream holds {} tokens", tokens.len());
    let slice = &tokens[..2_000_000];
    let counter = NgramCounter::new(0, slice).expect("counter");
    let (orders, _) = gpu_sweep(&counter, 3, 1_000_000, &every_order(3)).expect("sweep");
    for (order, got) in orders.iter().enumerate() {
        let want = oracle_adjusted(slice, order as u32, 3).expect("oracle");
        assert_eq!(got.0, want.0, "slice digest, order {order}");
        assert!(got.1 == want.1 && got.2 == want.2, "slice records, order {order}");
    }
    let counter = NgramCounter::new(0, &tokens).expect("counter");
    let (orders, shared) = gpu_sweep(&counter, 3, 1_000_000, &every_order(3)).expect("sweep");
    for (order, got) in orders.iter().enumerate() {
        let (digest, triples, contexts, partitions) = gpu_adjusted(&counter, order as u32, 3, 1_000_000);
        assert_eq!(got.0, digest, "whole-stream digest, order {order}");
        assert!(got.1 == triples && got.2 == contexts, "whole-stream records, order {order}");
        eprintln!(
            "real tokens {} order {order}: sweep {shared} shared ranges, adjusted {partitions} partitions, sha {}",
            tokens.len(),
            digest.triple_sha256
        );
    }
}
