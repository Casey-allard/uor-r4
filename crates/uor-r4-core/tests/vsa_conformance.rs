use uor_r4_core::native_geometric::vsa::{
    encode_prime_stride_ngram, BipolarAccumulator4096, BipolarAccumulator8192, Codebook,
    Hypervector4096, Hypervector8192, RollingBipolarContext, WORDS_4096,
};
use uor_r4_core::semantic::{encode_event, encode_graph_edge, encode_statement, expand_atom};

#[test]
fn test_vsa_algebra_identities() {
    let space = "test_space";
    let a = expand_atom("entity", "a", space);
    let b = expand_atom("entity", "b", space);

    // 1. Self-Inverse (Hamming similarity of bind(A, bind(A, B)) with B is 1.0)
    let bound = a.bind(&b);
    let unbound = a.unbind(&bound);
    assert_eq!(
        unbound.similarity(&b),
        1.0,
        "unbind should recover original hypervector"
    );

    // 2. Commutativity of Bind: bind(A, B) == bind(B, A)
    let bound_ba = b.bind(&a);
    assert_eq!(
        bound.similarity(&bound_ba),
        1.0,
        "bind should be commutative"
    );

    // 3. Permutation circular shifts: permute(A, 1024) == A
    let perm_zero = a.permute(0);
    assert_eq!(perm_zero.similarity(&a), 1.0);
    let perm_full = a.permute(1024);
    assert_eq!(perm_full.similarity(&a), 1.0);
}

#[test]
fn test_vsa_grounding_conformance_signatures() {
    let space = "test_space";

    // Statements
    let stmt = encode_statement("Paris", "capital_of", "France", space);
    // Events
    let event = encode_event("Alice", "visited", "2026-07-20", "Paris", space);
    // Graph edge
    let edge = encode_graph_edge("NodeA", "links_to", "NodeB", space);

    // Verify non-zero and stable dimension size
    assert_eq!(stmt.0.len(), 16);
    assert_eq!(event.0.len(), 16);
    assert_eq!(edge.0.len(), 16);

    // Asserts deterministic hash expansion - bit conformance
    let paris_entity = expand_atom("entity", "Paris", space);
    let paris_entity_again = expand_atom("entity", "Paris", space);
    assert_eq!(paris_entity.similarity(&paris_entity_again), 1.0);
}

#[test]
fn bipolar_accumulator_superposition_and_exact_algebraic_revert() {
    let mut acc = BipolarAccumulator8192::new();
    let mut items = Vec::with_capacity(15);

    for i in 0..15u64 {
        let hv = Hypervector8192::from_seed(0x2026_1027_u64, i.wrapping_mul(17).wrapping_add(1));
        acc.accumulate(&hv, 1);
        items.push(hv);
    }

    let bundled = acc.project();

    // Every stored item must have a strong positive integer bit margin (> +4 sigma = +180 bits)
    for (idx, item) in items.iter().enumerate() {
        let excess = bundled.excess_overlap_bits(item);
        assert!(
            excess >= 180,
            "stored item {idx} excess bits {excess} fell below +4 sigma (180 bits)"
        );
    }

    // Orthogonal random probe stays near 0 sigma (within +/- 3 sigma = 135 bits)
    let random_probe = Hypervector8192::from_seed(0xDEAD_BEEF_CAFE_BABEu64, 0x9999);
    let noise_excess = bundled.excess_overlap_bits(&random_probe);
    assert!(
        noise_excess.abs() < 135,
        "orthogonal probe excess bits {noise_excess} exceeded 3 sigma noise bound"
    );

    // Snapshot before transient perturbation
    let snapshot_before = acc;
    let transient = Hypervector8192::from_seed(0xBAD0_FA11_ED00_0001u64, 42);

    acc.accumulate(&transient, 3);
    let dist_during = acc.project().hamming_distance(&bundled);
    assert!(
        dist_during > 500,
        "adding weight +3 transient vector must shift projected bits (shifted {dist_during})"
    );

    // Subtract transient perturbation (-3) -> exact integer state restored
    acc.accumulate(&transient, -3);
    let dist_after = acc.project().hamming_distance(&bundled);
    assert_eq!(
        dist_after, 0,
        "Hamming distance after algebraic revert must be 0"
    );
    assert_eq!(
        acc.counts, snapshot_before.counts,
        "signed i16 counter array must be restored identically"
    );
}

#[test]
fn weighted_accumulator_preserves_salient_signal_under_stopword_flood() {
    let salient = Hypervector4096::from_seed(0xABCD_0001, 1);
    let mut stopwords = Vec::with_capacity(12);
    for i in 0..12u64 {
        stopwords.push(Hypervector4096::from_seed(0xABCD_0002, i + 100));
    }

    // Control arm: equal-weight bundling (1 salient + 12 stopwords, all weight = 1)
    let mut equal_list = Vec::with_capacity(13);
    equal_list.push(salient);
    equal_list.extend_from_slice(&stopwords);
    let equal_bundle = Hypervector4096::bundle(&equal_list);
    let equal_excess = equal_bundle.excess_overlap_bits(&salient);

    // Treatment arm: weighted BipolarAccumulator (salient weight = +6, stopwords weight = +1)
    let mut weighted_acc = BipolarAccumulator4096::new();
    weighted_acc.accumulate(&salient, 6);
    for sw in &stopwords {
        weighted_acc.accumulate(sw, 1);
    }
    let weighted_bundle = weighted_acc.project();
    let weighted_excess = weighted_bundle.excess_overlap_bits(&salient);

    assert!(
        weighted_excess > equal_excess + 256,
        "weighted accumulator excess ({weighted_excess}) must beat equal-weight bundle ({equal_excess}) by > 8 sigma"
    );
    assert!(
        weighted_excess >= 600,
        "salient vector must retain strong signal ({weighted_excess} excess bits)"
    );
}

#[test]
fn rolling_bipolar_context_matches_exact_window_reconstruction() {
    let codebook = Codebook::<WORDS_4096>::new(256, 0x7777_8888);
    let mut rolling = RollingBipolarContext::<WORDS_4096, 16>::new();

    // Push 48 tokens (wrapping the 16-entry ring buffer 3 times) with varying weights
    for step in 0..48u32 {
        let token = (step * 7 + 3) % 256;
        let weight = if step % 5 == 0 { 4i16 } else { 1i16 };
        rolling.push_token(&codebook, token, weight);
    }

    // Build a fresh accumulator from only the final 16 tokens (steps 32..48)
    let mut fresh = BipolarAccumulator4096::new();
    for step in 32..48u32 {
        let token = (step * 7 + 3) % 256;
        let weight = if step % 5 == 0 { 4i16 } else { 1i16 };
        fresh.accumulate(&codebook.get(token), weight);
    }

    assert_eq!(
        rolling.accumulator.counts, fresh.counts,
        "rolling O(1) eviction accumulator must equal fresh reconstruction of active window"
    );
    assert_eq!(rolling.project(), fresh.project());

    // Also verify prime-stride n-gram order sensitivity
    let seq_ab = encode_prime_stride_ngram(&[10, 20, 30], &codebook);
    let seq_ba = encode_prime_stride_ngram(&[30, 20, 10], &codebook);
    assert!(
        seq_ab.excess_overlap_bits(&seq_ba).abs() < 128,
        "reversed prime-stride n-gram must be quasi-orthogonal to forward order"
    );
}
