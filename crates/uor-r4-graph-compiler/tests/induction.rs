//! Cover-induction tests (graph-compiler plan §5 Phase 2, issue #60):
//! multiresolution recovery on a planted synthetic corpus, entropy-gated
//! splitting, calibrated radii, reference-classifier agreement,
//! T-invariance/determinism, memory-budget behavior, and R4G1 output
//! validation. The end-to-end fixture-corpus run (150k observations) is a
//! release-only workload and is `#[ignore]`d by default, mirroring the
//! κ-reproduction convention: run it with
//! `cargo test -p uor-r4-core --release --offline --test cover -- --ignored`.

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler::{self, Corpus, D, K, SIG_BYTES, STAGES, WINDOW};
use uor_r4_graph_compiler::induction as cover;
use uor_r4_graph_compiler::induction::{
    Cover, CoverConfig, CoverEdge, CoverRegion, EDGE_KIND_NEIGHBOR, EDGE_KIND_REFINEMENT,
    EDGE_KIND_TRANSITION, Observation,
};
use uor_r4_graph_compiler::observation as observe;
use uor_r4_graph_format::{GraphView, SectionId};

// ------------------------------------------------------- synthetic data --

/// Documented floors/thresholds the assertions below are written against.
const RECOVERY_RECALL_FLOOR: f64 = 0.95;
const REFERENCE_TOP1_FLOOR: f64 = 0.95;

/// Planted group sizes: G0, G1, G2a, G2b, G3 (the G2 region has 120
/// members ≥ min_support 64; its children have 60 < 64 and never split).
const GROUP_SIZES: [usize; 5] = [100, 100, 60, 60, 100];
/// Coarse group of each planted group (G2a/G2b share the G2 geometry).
const COARSE: [usize; 5] = [0, 1, 2, 2, 3];

fn xorshift(s: &mut u64) -> u64 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *s = x;
    x
}

/// Planted cluster centers on the 288-dim unit sphere: four orthogonal
/// 72-dim blocks; the G2 block is shared by G2a/G2b, which differ by
/// reweighting its halves (mutual cosine ≈ 0.92 — tight against the
/// orthogonal inter-group distance 1, cleanly separable at depth 2).
fn planted_center(coarse: usize) -> Vec<f32> {
    let mut center = vec![0f32; D];
    for d in 0..72 {
        center[coarse * 72 + d] = 1.0 / (72f32).sqrt();
    }
    center
}

fn planted_center_g2(sub_b: bool) -> Vec<f32> {
    let mut center = vec![0f32; D];
    for d in 0..36 {
        center[144 + d] = if sub_b { 0.8 } else { 1.2 };
        center[180 + d] = if sub_b { 1.2 } else { 0.8 };
    }
    center
}

fn normalize(v: &mut [f32]) {
    let nn = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    for x in v.iter_mut() {
        *x /= nn;
    }
}

/// The synthetic observation corpus: tight clusters (within-cluster
/// cosine > 0.99) with planted next-token distributions:
/// - G0: deterministic token 30 (entropy 0 — never splits);
/// - G1: uniform over {40..44} with no geometric sub-structure
///   (entropy ≈ 2 bits, gain of any split ≈ 0 — never splits);
/// - G2a/G2b: deterministic tokens 10 and 20 (the G2 region has entropy
///   1.0 bit and pure children — split gain 1.0 > 0.25, splits);
/// - G3: uniform over {50..54} (like G1).
fn synthetic_observations() -> (Vec<Observation>, Vec<usize>) {
    let mut observations = Vec::new();
    let mut labels = Vec::new();
    let mut position = 0u32;
    for (group, &size) in GROUP_SIZES.iter().enumerate() {
        let center = match group {
            2 => planted_center_g2(false),
            3 => planted_center_g2(true),
            _ => planted_center(COARSE[group]),
        };
        for i in 0..size {
            let index = position as usize;
            let mut vector = center.clone();
            // Deterministic per-(member, dim) sign-independent jitter,
            // magnitude < 0.01: sign noise outside the signal block, like
            // real threshold crossings, without correlation across
            // members (a low-entropy phase pattern would inflate the
            // within-cluster Hamming spread and break the radius toy).
            let mut rng = (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for x in vector.iter_mut() {
                let draw = xorshift(&mut rng);
                let magnitude = ((draw >> 9) % 1000) as f32 * 1e-5;
                *x += if draw & 1 == 1 { magnitude } else { -magnitude };
            }
            normalize(&mut vector);
            let mut sig = [0u8; SIG_BYTES];
            for (d, &x) in vector.iter().enumerate() {
                if x > 0.0 {
                    sig[d / 8] |= 1 << (d % 8);
                }
            }
            let next = match group {
                0 => 30,
                1 => 40 + (i % 4) as u32,
                2 => 10,
                3 => 20,
                _ => 50 + (i % 4) as u32,
            };
            observations.push(Observation {
                position,
                sample: blake3::hash(&position.to_le_bytes()).into(),
                vector,
                sig,
                prev: 0,
                next,
            });
            labels.push(COARSE[group]);
            position += 1;
        }
    }
    (observations, labels)
}

fn synthetic_config() -> CoverConfig {
    CoverConfig {
        depths: 3,
        k0: 4,
        regions_budget: 256,
        min_support: 64,
        entropy_gain_bits: 0.25,
        ..CoverConfig::default()
    }
}

const ART_KAPPA: &str = "blake3:synthetic-artifact";
const CORPUS_KAPPA: &str = "blake3:synthetic-corpus";

fn induce_synthetic(observations: &[Observation], config: &CoverConfig) -> cover::InducedCover {
    cover::induce_cover(observations, config, ART_KAPPA, CORPUS_KAPPA).expect("induction succeeds")
}

/// A minimal Compiled for `evaluate_held_out`'s class-code path: the
/// class signatures are deterministic xorshift bytes (the class cover is
/// not under test here; the codes just need to be well-defined).
fn synthetic_compiled() -> compiler::Compiled {
    let vocab = 64usize;
    let mut rng = 0xC0DE42u64;
    let mut rand_bytes =
        |n: usize| -> Vec<u8> { (0..n).map(|_| (xorshift(&mut rng) & 0xff) as u8).collect() };
    compiler::Compiled {
        token_codes: rand_bytes(vocab * STAGES),
        stage_books: (0..STAGES)
            .map(|_| rand_bytes(K * D).iter().map(|&b| b as i8).collect())
            .collect(),
        stage_shifts: vec![0; STAGES],
        thresholds: vec![0; D],
        class_sigs: (0..STAGES).map(|_| rand_bytes(K * SIG_BYTES)).collect(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb: Vec::new(),
    }
}

fn hand_region(id: u32, depth: u8, sig: [u8; SIG_BYTES], radius: u16) -> CoverRegion {
    CoverRegion {
        id,
        depth,
        parent: None,
        children: Vec::new(),
        prototype: vec![0.0; D],
        sig,
        radius,
        support: 0,
        entropy_bits: 0.0,
        split_gain_bits: 0.0,
    }
}

// ------------------------------------------------------------ induction --

#[test]
fn recovers_planted_regions_and_splits_only_on_entropy_gain() {
    let (observations, labels) = synthetic_observations();
    let induced = induce_synthetic(&observations, &synthetic_config());
    let cover = &induced.cover;

    // The broad cover: 4 regions at depth 1; the G2 region splits into 2
    // at depth 2; nothing reaches depth 3. Total 6 regions.
    assert_eq!(cover.regions.len(), 6, "4 depth-1 regions + 2 G2 children");
    assert_eq!(cover.regions_at_depth(1).len(), 4);
    assert_eq!(cover.regions_at_depth(2).len(), 2);
    assert_eq!(cover.max_depth, 2);

    // Depth-1 recovery of the planted coarse groups: every planted group
    // is ≥95% contained in some depth-1 region (expected: exact).
    for group in 0..4 {
        let group_members: Vec<usize> = (0..observations.len())
            .filter(|&i| labels[i] == group)
            .collect();
        let best = cover
            .regions_at_depth(1)
            .iter()
            .map(|&r| {
                group_members
                    .iter()
                    .filter(|i| cover.members[r as usize].contains(i))
                    .count()
            })
            .max()
            .unwrap();
        let recall = best as f64 / group_members.len() as f64;
        assert!(
            recall >= RECOVERY_RECALL_FLOOR,
            "planted coarse group {group} recovered with recall {recall:.3} (floor {RECOVERY_RECALL_FLOOR})"
        );
    }

    // Split discipline: exactly one region split — the G2 region, with
    // the full 1.0-bit gain of separating tokens 10 and 20.
    let splits: Vec<&CoverRegion> = cover
        .regions
        .iter()
        .filter(|r| !r.children.is_empty())
        .collect();
    assert_eq!(splits.len(), 1, "only the G2 region earns a split");
    let g2 = splits[0];
    assert_eq!(g2.depth, 1);
    assert_eq!(g2.support, 120, "G2a ∪ G2b");
    assert!(
        g2.split_gain_bits > 0.99,
        "50/50 deterministic tokens give a 1.0-bit gain, got {}",
        g2.split_gain_bits
    );
    assert_eq!(g2.children.len(), 2);
    for &child in &g2.children {
        let region = &cover.regions[child as usize];
        assert_eq!(region.depth, 2);
        assert_eq!(region.parent, Some(g2.id));
        assert_eq!(region.support, 60);
        assert_eq!(region.entropy_bits, 0.0, "children are token-pure");
    }
    // Every other region is a leaf with zero gain (G0: entropy 0; G1/G3:
    // uniform next-token without geometric sub-structure).
    for region in &cover.regions {
        if region.id != g2.id && region.depth == 1 {
            assert!(region.children.is_empty());
            assert_eq!(region.split_gain_bits, 0.0);
        }
    }
}

#[test]
fn respects_the_region_budget() {
    let (observations, _) = synthetic_observations();
    let mut config = synthetic_config();
    config.regions_budget = 4; // exactly the depth-1 cover: no room to split
    let induced = induce_synthetic(&observations, &config);
    assert_eq!(induced.cover.regions.len(), 4);
    assert!(
        induced.cover.regions.iter().all(|r| r.children.is_empty()),
        "budget exhaustion forbids splits even where entropy justifies them"
    );
}

#[test]
fn respects_the_depth_cap() {
    let (observations, _) = synthetic_observations();
    let mut config = synthetic_config();
    config.depths = 1;
    let induced = induce_synthetic(&observations, &config);
    assert_eq!(induced.cover.regions.len(), 4);
    assert_eq!(induced.cover.max_depth, 1);
}

#[test]
fn memory_budget_derives_batch_size_without_changing_results() {
    // The §4.1 formula shape: budget − reserve − cluster state, divided
    // by the per-observation accounting size, clamped to [1, n].
    let n = 1000usize;
    let big = cover::derive_batch_size(512 * 1024 * 1024, 256, n);
    assert_eq!(big, n, "a large budget admits one full batch");
    let tiny = cover::derive_batch_size(1024 * 1024, 256, n);
    assert_eq!(
        tiny, 1,
        "a budget below the reserve degrades to batches of one"
    );
    let mid = cover::derive_batch_size(65 * 1024 * 1024, 256, n);
    assert!(mid > 1 && mid < n, "mid budgets shard: got {mid}");
    // Batch size is a pure resource knob: the cover is byte-identical
    // under different memory budgets.
    let (observations, _) = synthetic_observations();
    let mut config = synthetic_config();
    config.memory_budget_bytes = 512 * 1024 * 1024;
    let wide = induce_synthetic(&observations, &config);
    config.memory_budget_bytes = 1024 * 1024;
    let narrow = induce_synthetic(&observations, &config);
    assert!(wide.batch_size > narrow.batch_size);
    assert_eq!(wide.cover.kappa(), narrow.cover.kappa());
}

// --------------------------------------------------------- determinism --

/// T-invariance (plan §4.1 / Gate E): identical inputs produce
/// byte-identical outputs regardless of the recorded thread count. Worker
/// extraction is sharded, while k-means reductions remain ordered, so T=1
/// and T=4 agree and a double run reproduces the artifact bytes exactly.
#[test]
fn t_invariance_and_double_run_identity() {
    let (observations, _) = synthetic_observations();
    let mut config = synthetic_config();
    config.threads = 1;
    let one = induce_synthetic(&observations, &config);
    config.threads = 4;
    let four = induce_synthetic(&observations, &config);
    assert_eq!(
        one.cover.kappa(),
        four.cover.kappa(),
        "T=1 and T=4 induce the identical cover"
    );

    let emit = |induced: &cover::InducedCover| -> Vec<u8> {
        let reference = cover::ReferenceClassifier::freeze(&induced.cover);
        let edges = cover::build_edges(&induced.cover, &reference, &observations, &vec![0; 10000]);
        let prior = cover::root_prior(&observations);
        cover::emit_r4g1(
            b"synthetic-artifact-container",
            (b"synthetic-meta", b"synthetic-recs"),
            64,
            &induced.cover,
            &edges,
            &prior,
            &[],
        )
        .expect("emit")
        .0
    };
    let bytes_t1 = emit(&one);
    let bytes_t4 = emit(&four);
    assert_eq!(bytes_t1, bytes_t4, "T-invariant artifact bytes");
    let bytes_again = emit(&induce_synthetic(&observations, &synthetic_config()));
    assert_eq!(bytes_t1, bytes_again, "double-run byte identity");
}

/// Shard completion order never reaches the merged observation stream:
/// observations partitioned by `observe::shard_of` and completed in a
/// shuffled order merge (ascending shard-id order) into one canonical
/// sequence — the §22/§4.1 content-addressed sharding rule.
#[test]
fn shuffled_shard_completion_order_is_invisible() {
    let (observations, _) = synthetic_observations();
    let shard_bits = 2u8;
    let shard_count = 1u32 << shard_bits;
    let mut shards: Vec<Vec<usize>> = vec![Vec::new(); shard_count as usize];
    for (i, observation) in observations.iter().enumerate() {
        shards[observe::shard_of(&observation.sample, shard_bits) as usize].push(i);
    }
    // "Complete" the shards in reverse order, then merge ascending — the
    // merged index sequence must equal the never-shuffled merge.
    let merge = |completion: &[u32]| -> Vec<usize> {
        let mut completed: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
        for &shard in completion {
            completed.insert(shard, shards[shard as usize].clone());
        }
        completed.into_values().flatten().collect()
    };
    let forward: Vec<u32> = (0..shard_count).collect();
    let reverse: Vec<u32> = (0..shard_count).rev().collect();
    let canonical = merge(&forward);
    let shuffled = merge(&reverse);
    assert_eq!(canonical, shuffled);
    // And the induced covers over the merged stream agree.
    let order_a: Vec<Observation> = canonical.iter().map(|&i| observations[i].clone()).collect();
    let order_b: Vec<Observation> = shuffled.iter().map(|&i| observations[i].clone()).collect();
    let a = induce_synthetic(&order_a, &synthetic_config());
    let b = induce_synthetic(&order_b, &synthetic_config());
    assert_eq!(a.cover.kappa(), b.cover.kappa());
}

// ----------------------------------------------------------- calibration --

/// The 95th-percentile radius covers every member distance in-sample for
/// member counts ≤ 19 (the quantile target is the full count there).
#[test]
fn calibrated_radius_covers_members_on_toy_case() {
    let proto = [0u8; SIG_BYTES];
    // 10 members at distances 0..=9 from the prototype.
    let members: Vec<[u8; SIG_BYTES]> = (0..10)
        .map(|dist| {
            let mut sig = [0u8; SIG_BYTES];
            for bit in 0..dist {
                sig[bit / 8] |= 1 << (bit % 8);
            }
            sig
        })
        .collect();
    let radius = cover::calibrate_region_radius(&members, &proto);
    assert_eq!(radius, 9, "radius = the maximum member distance here");
    for (i, sig) in members.iter().enumerate() {
        let dist = sig.iter().map(|b| b.count_ones()).sum::<u32>();
        assert_eq!(dist, i as u32);
        assert!(dist <= u32::from(radius));
    }
}

/// The nearest-region fallback is intact: when no top-M candidate is
/// within its calibrated radius, membership is exactly the nearest region.
#[test]
fn nearest_region_fallback_when_radius_misses() {
    let sig_a = [0u8; SIG_BYTES];
    let mut sig_b = [0xFFu8; SIG_BYTES];
    let mut sig_c = [0u8; SIG_BYTES];
    sig_c[0] = 0b0000_1111;
    sig_b[0] = 0b1111_0000; // make B's distance to the probe unambiguous
    let cover = Cover {
        regions: vec![
            hand_region(0, 1, sig_a, 0),
            hand_region(1, 1, sig_b, 0),
            hand_region(2, 1, sig_c, 0),
        ],
        max_depth: 1,
        paths: Vec::new(),
        members: Vec::new(),
    };
    let reference = cover::ReferenceClassifier::freeze(&cover);
    // Probe: distance 2 to A (bits 0,1), far from B and C; radius 0
    // rejects every candidate, so the fallback returns nearest = A.
    let mut probe = [0u8; SIG_BYTES];
    probe[0] = 0b0000_0011;
    let memberships = reference.binary_memberships(1, &probe);
    assert_eq!(memberships, vec![0], "nearest-region fallback");
    // With A's radius covering distance 2, A is a normal in-range member.
    let cover2 = Cover {
        regions: vec![
            hand_region(0, 1, sig_a, 2),
            hand_region(1, 1, sig_b, 0),
            hand_region(2, 1, sig_c, 0),
        ],
        max_depth: 1,
        paths: Vec::new(),
        members: Vec::new(),
    };
    let reference2 = cover::ReferenceClassifier::freeze(&cover2);
    let memberships2 = reference2.binary_memberships(1, &probe);
    assert_eq!(memberships2, vec![0], "in-range membership, distance order");
}

/// Top-M ordering and the TOP_M bound on a hand-built depth.
#[test]
fn binary_memberships_are_top_m_in_distance_order() {
    let sig_a = [0u8; SIG_BYTES]; // distance 2 to probe
    let mut sig_b = [0u8; SIG_BYTES];
    sig_b[0] = 0b0000_0111; // distance 1 to probe (nearest)
    let mut sig_c = [0u8; SIG_BYTES];
    sig_c[0] = 0b0001_1111; // distance 3 to probe
    let mut sig_d = [0u8; SIG_BYTES];
    sig_d[0] = 0b0011_1111; // distance 4 to probe (dropped: TOP_M = 3)
    let cover = Cover {
        regions: vec![
            hand_region(0, 1, sig_a, 8),
            hand_region(1, 1, sig_b, 8),
            hand_region(2, 1, sig_c, 8),
            hand_region(3, 1, sig_d, 8),
            hand_region(4, 2, [0xAAu8; SIG_BYTES], 288), // other depth: ignored
        ],
        max_depth: 2,
        paths: Vec::new(),
        members: Vec::new(),
    };
    let reference = cover::ReferenceClassifier::freeze(&cover);
    let mut probe = [0u8; SIG_BYTES];
    probe[0] = 0b0000_0011;
    let memberships = reference.binary_memberships(1, &probe);
    assert_eq!(
        memberships,
        vec![1, 0, 2],
        "top-3 by distance, ties to lower id"
    );
    assert!(memberships.len() <= cover::TOP_M);
}

// ------------------------------------------------- reference classifier --

/// Reference-classifier agreement (issue #60's recall number): the
/// shipped binary-Hamming routing reproduces the exact compiler-side
/// membership with top-1 recall ≥ the documented floor on the synthetic
/// corpus (expected ≈ 1.0: well-separated clusters).
#[test]
fn reference_classifier_agreement_floor() {
    let (observations, _) = synthetic_observations();
    let compiled = synthetic_compiled();
    let induced = induce_synthetic(&observations, &synthetic_config());
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let recall = cover::evaluate_held_out(
        &compiled,
        &induced.cover,
        &reference,
        &observations,
        &observations,
    );
    let depth1 = &recall[0];
    assert_eq!(depth1.evaluated, observations.len());
    assert!(
        depth1.reference_top1_recall >= REFERENCE_TOP1_FLOOR,
        "reference top-1 recall {:.3} below the {REFERENCE_TOP1_FLOOR} floor",
        depth1.reference_top1_recall
    );
    assert!(
        depth1.reference_topm_recall >= depth1.reference_top1_recall,
        "top-M recall dominates top-1"
    );
    assert!(depth1.frontier_width_max <= cover::TOP_M as u32);
    // In-sample co-assignment against the (synthetic) class cover is
    // self-consistent: rates are probabilities.
    for rate in [
        depth1.class_coassignment_recall_top1,
        depth1.class_coassignment_recall_topm,
        depth1.class_coassignment_precision_top1,
        depth1.class_coassignment_precision_topm,
    ] {
        assert!((0.0..=1.0).contains(&rate));
    }
}

/// The frozen classifier survives cover mutation: freezing copies the
/// region parameters (the normative semantics cannot drift).
#[test]
fn frozen_classifier_is_stable_under_cover_mutation() {
    let (observations, _) = synthetic_observations();
    let mut induced = induce_synthetic(&observations, &synthetic_config());
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let kappa = reference.kappa().to_owned();
    let probe = observations[0].sig;
    let before = reference.binary_memberships(1, &probe);
    induced.cover.regions[0].radius = 0;
    induced.cover.regions[0].sig = [0xFFu8; SIG_BYTES];
    assert_eq!(reference.binary_memberships(1, &probe), before);
    assert_eq!(reference.kappa(), kappa);
}

// ---------------------------------------------------------------- edges --

#[test]
fn edges_are_canonical_bounded_and_honest() {
    let (observations, _) = synthetic_observations();
    let induced = induce_synthetic(&observations, &synthetic_config());
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let edges = cover::build_edges(&induced.cover, &reference, &observations, &vec![0; 10000]);

    // Refinement: root → each depth-1 region, parent → child below.
    let refinement: Vec<&CoverEdge> = edges
        .iter()
        .filter(|e| e.kind == EDGE_KIND_REFINEMENT)
        .collect();
    assert_eq!(refinement.len(), induced.cover.regions.len());
    let root_children = refinement
        .iter()
        .filter(|e| e.src == cover::ROOT_NODE)
        .count();
    assert_eq!(root_children, 4);
    // Canonical sort: (src, kind, dst) strictly increasing; each node's
    // refinement children contiguous by construction.
    for pair in edges.windows(2) {
        assert!(
            (pair[0].src, pair[0].kind, pair[0].dst) < (pair[1].src, pair[1].kind, pair[1].dst),
            "canonical edge order"
        );
    }
    // Neighbor degree cap and canonical orientation (src < dst).
    let mut degree: BTreeMap<u32, u32> = BTreeMap::new();
    for edge in edges.iter().filter(|e| e.kind == EDGE_KIND_NEIGHBOR) {
        assert!(edge.src < edge.dst);
        *degree.entry(edge.src).or_insert(0) += 1;
        *degree.entry(edge.dst).or_insert(0) += 1;
    }
    for (node, deg) in &degree {
        assert!(
            *deg as usize <= 2 * cover::MAX_NEIGHBOR_EDGES,
            "node {node} degree {deg} beyond the bounded cap"
        );
    }
}

#[test]
fn canonical_fragment_merge_is_sorted_and_deduplicated() {
    let fragments = vec![
        vec![
            CoverEdge {
                src: 3,
                kind: EDGE_KIND_TRANSITION,
                dst: 8,
            },
            CoverEdge {
                src: 1,
                kind: EDGE_KIND_REFINEMENT,
                dst: 2,
            },
        ],
        vec![
            CoverEdge {
                src: 2,
                kind: EDGE_KIND_NEIGHBOR,
                dst: 5,
            },
            CoverEdge {
                src: 1,
                kind: EDGE_KIND_REFINEMENT,
                dst: 2,
            },
        ],
        vec![CoverEdge {
            src: 2,
            kind: EDGE_KIND_NEIGHBOR,
            dst: 5,
        }],
    ];
    let merged = cover::canonical_merge_edge_fragments(&fragments);
    assert_eq!(
        merged,
        vec![
            CoverEdge {
                src: 1,
                kind: EDGE_KIND_REFINEMENT,
                dst: 2,
            },
            CoverEdge {
                src: 2,
                kind: EDGE_KIND_NEIGHBOR,
                dst: 5,
            },
            CoverEdge {
                src: 3,
                kind: EDGE_KIND_TRANSITION,
                dst: 8,
            },
        ],
        "stable canonical sort + deterministic dedup"
    );
}

#[test]
fn canonical_fragment_merge_is_invariant_under_fragment_interleaving() {
    let base = vec![
        vec![
            CoverEdge {
                src: 4,
                kind: EDGE_KIND_NEIGHBOR,
                dst: 7,
            },
            CoverEdge {
                src: 0,
                kind: EDGE_KIND_REFINEMENT,
                dst: 1,
            },
        ],
        vec![
            CoverEdge {
                src: 0,
                kind: EDGE_KIND_REFINEMENT,
                dst: 1,
            },
            CoverEdge {
                src: 5,
                kind: EDGE_KIND_TRANSITION,
                dst: 5,
            },
        ],
        vec![CoverEdge {
            src: 2,
            kind: EDGE_KIND_NEIGHBOR,
            dst: 6,
        }],
    ];
    let canonical = cover::canonical_merge_edge_fragments(&base);
    for &i in &[0usize, 1, 2] {
        for &j in &[0usize, 1, 2] {
            if j == i {
                continue;
            }
            for &k in &[0usize, 1, 2] {
                if k == i || k == j {
                    continue;
                }
                let permuted = vec![base[i].clone(), base[j].clone(), base[k].clone()];
                assert_eq!(
                    cover::canonical_merge_edge_fragments(&permuted),
                    canonical,
                    "fragment permutation ({i},{j},{k}) changes merge output"
                );
            }
        }
    }
}

// ----------------------------------------------------------------- emit --

#[test]
fn r4g1_artifact_validates_and_reproduces() {
    let (observations, _) = synthetic_observations();
    let induced = induce_synthetic(&observations, &synthetic_config());
    let cover = &induced.cover;
    let reference = cover::ReferenceClassifier::freeze(cover);
    let edges = cover::build_edges(cover, &reference, &observations, &vec![0; 10000]);
    let prior = cover::root_prior(&observations);
    let (bytes, info) = cover::emit_r4g1(
        b"synthetic-artifact-container",
        (b"synthetic-meta", b"synthetic-recs"),
        64,
        cover,
        &edges,
        &prior,
        &[],
    )
    .expect("emit succeeds");

    let view = GraphView::parse(&bytes).expect("stage-1+2 validation");
    view.verify_cids().expect("integrity CIDs");
    let head = view.head().expect("HEAD present");
    assert_eq!(head.node_count(), 1 + cover.regions.len() as u32);
    assert_eq!(head.edge_count(), edges.len() as u32);
    assert_eq!(head.depth_count() as usize, cover.max_depth + 1);
    assert_eq!(head.signature_bytes() as usize, SIG_BYTES);
    assert_eq!(info.node_count, head.node_count());
    assert_eq!(info.edge_count, head.edge_count());
    assert_eq!(info.artifact_bytes, bytes.len());
    assert_eq!(
        info.root_prior_entries as usize,
        prior.len(),
        "root prior carries the train next-token distribution"
    );

    // Node records are honest: the root is the all-zero synthetic floor;
    // region nodes carry their depth and calibrated radius.
    let root = view.node(cover::ROOT_NODE).expect("root record");
    assert_eq!(root.depth.0, 0);
    assert_eq!(
        root.child_len, 0,
        "root keeps ranges empty (converter convention)"
    );
    for region in &cover.regions {
        let node = view
            .node(cover::region_node_id(region.id))
            .expect("region record");
        assert_eq!(node.depth.0, region.depth);
        assert_eq!(node.radius.0, region.radius);
        assert_eq!(node.child_len as usize, region.children.len());
    }
    // The reverse index is a permutation of the canonical edge ids
    // (stage 2 checks existence; check honesty directly).
    let mut reverse: Vec<u32> = (0..head.edge_count())
        .map(|i| view.reverse_edge_id(i).expect("reverse entry"))
        .collect();
    reverse.sort_unstable();
    assert_eq!(reverse, (0..head.edge_count()).collect::<Vec<u32>>());

    // Deterministic double-run.
    let (bytes2, _) = cover::emit_r4g1(
        b"synthetic-artifact-container",
        (b"synthetic-meta", b"synthetic-recs"),
        64,
        cover,
        &edges,
        &prior,
        &[],
    )
    .expect("emit succeeds");
    assert_eq!(bytes, bytes2, "canonical serializer reproduces the bytes");

    // The HEAD corpus construction CID pins the corpus material.
    let expected_corpus_cid = {
        let mut h = blake3::Hasher::new();
        h.update(b"synthetic-meta");
        h.update(b"synthetic-recs");
        *h.finalize().as_bytes()
    };
    assert_eq!(head.corpus_construction_cid().0, expected_corpus_cid);
    // EXCT is optional in this slice and omitted.
    assert!(view.section(SectionId::EXCT).is_none());
}

// ------------------------------------------------------------ primitives --

#[test]
fn entropy_reduction_matches_hand_computation() {
    // 50/50 deterministic tokens, token-pure children: gain = 1.0 bit.
    let obs = |next: u32| Observation {
        position: 0,
        sample: [0u8; 32],
        vector: vec![0.0; D],
        sig: [0u8; SIG_BYTES],
        prev: 0,
        next,
    };
    let observations: Vec<Observation> = (0..100)
        .map(|i| obs(if i < 50 { 10 } else { 20 }))
        .collect();
    let members: Vec<usize> = (0..100).collect();
    let children = vec![(0..50).collect::<Vec<usize>>(), (50..100).collect()];
    let gain = cover::entropy_reduction(&observations, &members, &children);
    assert!(
        (gain - 1.0).abs() < 1e-12,
        "pure 50/50 split: gain 1 bit, got {gain}"
    );
    // Same distribution in both children: gain ≈ 0.
    let mixed_children = vec![
        (0..100).step_by(2).collect::<Vec<usize>>(),
        (1..100).step_by(2).collect::<Vec<usize>>(),
    ];
    let gain = cover::entropy_reduction(&observations, &members, &mixed_children);
    assert!(
        gain.abs() < 1e-12,
        "distribution-preserving split: gain 0, got {gain}"
    );
}

#[test]
fn objective_decomposition_and_tie_breaking_are_deterministic() {
    let obs = |position: u32, next: u32, bit: usize| {
        let mut vector = vec![0.0f32; D];
        vector[bit] = 1.0;
        let mut sig = [0u8; SIG_BYTES];
        sig[bit / 8] = 1 << (bit % 8);
        Observation {
            position,
            sample: [position as u8; 32],
            vector,
            sig,
            prev: position.saturating_sub(1),
            next,
        }
    };
    let observations = vec![obs(0, 10, 0), obs(1, 10, 0), obs(2, 20, 1), obs(3, 20, 1)];
    let mut config = CoverConfig {
        depths: 2,
        k0: 1,
        min_support: 2,
        regions_budget: 8,
        entropy_gain_bits: -1.0,
        ..CoverConfig::default()
    };
    config.objective.weights.predictive_entropy = 0.0;
    config.objective.weights.future_state_entropy = 0.0;
    config.objective.weights.teacher_loss = 0.0;
    config.objective.weights.runtime_cost = 0.0;
    config.objective.weights.artifact_size = 0.0;
    config.objective.weights.bytes_read = 0.0;
    config.objective.weights.structural_complexity = 0.0;
    config.objective.weights.ib_term = 0.0;

    let induced =
        cover::induce_cover(&observations, &config, ART_KAPPA, CORPUS_KAPPA).expect("induce");
    assert_eq!(
        induced.cover.regions.len(),
        1,
        "objective ties resolve to keep"
    );
    assert_eq!(
        induced.decision_trace.len(),
        1,
        "one eligible split was audited"
    );
    let decision = &induced.decision_trace[0];
    assert_eq!(decision.decision, "keep:objective_tie");
    assert!(
        decision.entropy_gain_bits > 0.0,
        "entropy would have permitted the split without the objective tie"
    );
    assert!(
        (decision.keep.weighted_score - decision.split.weighted_score).abs() < 1e-12,
        "weights forced an exact objective tie"
    );

    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let story = vec![0u32; observations.len()];
    let edges = cover::build_edges(&induced.cover, &reference, &observations, &story);
    let report = cover::build_report(
        &config,
        &induced,
        cover::ReportData {
            reference: &reference,
            train: &observations,
            held_out: &observations,
            edges: &edges,
            recall: Vec::new(),
            artifact: None,
        },
    );
    assert_eq!(
        report.objective.config.schema,
        cover::OBJECTIVE_CONFIG_SCHEMA
    );
    assert_eq!(report.objective.region_decisions.len(), 1);
    assert!(
        report.objective.train.predictive_entropy_bits >= 0.0
            && report.objective.held_out.predictive_entropy_bits >= 0.0
    );
}

#[test]
fn kmeans_recovers_tight_clusters_at_any_batch_size() {
    let (observations, _) = synthetic_observations();
    let points: Vec<&[f32]> = observations.iter().map(|o| o.vector.as_slice()).collect();
    // Direct k-means recovery check (depth-1 geometry): k=4 over the 5
    // planted groups merges G2a/G2b and recovers the coarse groups.
    let run = |batch: usize| cover::spherical_kmeans(&points, 4, &[7u8; 32], batch);
    let full = run(points.len());
    let single = run(1);
    assert_eq!(
        full.assignment, single.assignment,
        "batch size never changes assignments"
    );
    assert_eq!(full.centroids.len(), 4);
    // Cluster sizes: three ~100s and the merged G2 at 120.
    let mut sizes = [0usize; 4];
    for &a in &full.assignment {
        sizes[a as usize] += 1;
    }
    sizes.sort_unstable();
    assert_eq!(sizes, [100, 100, 100, 120]);
}

#[test]
fn context_window_respects_story_boundaries_and_width() {
    let corpus = Corpus {
        n: 12,
        stories: 2,
        story: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
        input: (0..12).collect(),
        next: vec![0; 12],
        t_argmax: vec![0; 12],
        top_tokens: vec![[0; 8]; 12],
        top_weights: vec![[0; 8]; 12],
        span_start: vec![0; 12],
        span_end: vec![0; 12],
        byte_start: vec![0; 12],
        byte_end: vec![0; 12],
        hidden: None,
    };
    assert_eq!(cover::context_window(&corpus, 0), vec![0]);
    assert_eq!(cover::context_window(&corpus, 3), vec![0, 1, 2, 3]);
    // The window caps at WINDOW tokens.
    let window = cover::context_window(&corpus, 9);
    assert_eq!(window.len(), WINDOW);
    assert_eq!(window, (2..=9).collect::<Vec<u32>>());
    // A story boundary truncates the window.
    assert_eq!(cover::context_window(&corpus, 10), vec![10]);
    assert_eq!(cover::context_window(&corpus, 11), vec![10, 11]);
    // The sample id is the content address of exactly this window.
    assert_eq!(
        observe::sample_id(&cover::context_window(&corpus, 9)),
        observe::sample_id(&(2..=9).collect::<Vec<u32>>())
    );
}

#[test]
fn split_positions_applies_the_80_20_story_cut() {
    let corpus = Corpus {
        n: 10,
        stories: 5,
        story: vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4],
        input: (0..10).collect(),
        next: vec![0; 10],
        t_argmax: vec![0; 10],
        top_tokens: vec![[0; 8]; 10],
        top_weights: vec![[0; 8]; 10],
        span_start: vec![0; 10],
        span_end: vec![0; 10],
        byte_start: vec![0; 10],
        byte_end: vec![0; 10],
        hidden: None,
    };
    let (train, held_out) = cover::split_positions(&corpus);
    assert_eq!(
        train,
        vec![0, 1, 2, 3, 4, 5, 6, 7],
        "stories 0..4 (< cut 4)"
    );
    assert_eq!(held_out, vec![8, 9], "story 4 (≥ cut 4)");
}

// ------------------------------------------------- fixture end-to-end --

/// Full pipeline on the pinned fixture corpus (150k legacy records):
/// observation lane, induction, reference freeze, held-out recall vs the
/// incumbent 4×256 class cover, R4G1 emission. Release-only workload —
/// run with `cargo test -p uor-r4-core --release --offline --test cover
/// -- --ignored --nocapture` to see the recall report.
#[test]
#[ignore = "release-only fixture workload"]
fn fixture_corpus_end_to_end() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let artifact_container = std::fs::read(format!(
        "{dir}/../uor-r4-core/tests/fixtures/tless_artifacts.bin"
    ))
    .expect("fixture TLA5");
    let artifacts = compiler::parse_artifacts(&artifact_container).expect("fixture parses");
    let meta_bytes =
        std::fs::read(format!("{dir}/../uor-r4-core/tests/fixtures/c_meta.bin")).expect("meta");
    let recs_bytes =
        std::fs::read(format!("{dir}/../uor-r4-core/tests/fixtures/c_recs.bin")).expect("recs");
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/../uor-r4-core/tests/fixtures/c_meta.bin"),
        &format!("{dir}/../uor-r4-core/tests/fixtures/c_recs.bin"),
    )
    .expect("fixture corpus loads");
    let artifact_kappa = format!("blake3:{}", blake3::hash(&artifact_container).to_hex());
    let corpus_kappa = {
        let mut h = blake3::Hasher::new();
        h.update(&meta_bytes);
        h.update(&recs_bytes);
        format!("blake3:{}", h.finalize().to_hex())
    };

    let config = CoverConfig::default();
    let (train_pos, held_out_pos) = cover::split_positions(&corpus);
    let train = cover::build_observations(&artifacts, &corpus, &train_pos);
    let held_out = cover::build_observations(&artifacts, &corpus, &held_out_pos);
    assert_eq!(train.len() + held_out.len(), corpus.n);
    let induced = cover::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)
        .expect("fixture induction");
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let recall =
        cover::evaluate_held_out(&artifacts, &induced.cover, &reference, &train, &held_out);
    let edges = cover::build_edges(&induced.cover, &reference, &train, &corpus.story);
    let prior = cover::root_prior(&train);
    let vocab = (artifacts.token_codes.len() / STAGES) as u32;
    let (bytes, info) = cover::emit_r4g1(
        &artifact_container,
        (&meta_bytes, &recs_bytes),
        vocab,
        &induced.cover,
        &edges,
        &prior,
        &[],
    )
    .expect("fixture emit");
    let view = GraphView::parse(&bytes).expect("fixture artifact validates");
    view.verify_cids().expect("fixture CIDs");

    let report = cover::build_report(
        &config,
        &induced,
        cover::ReportData {
            reference: &reference,
            train: &train,
            held_out: &held_out,
            edges: &edges,
            recall: recall.clone(),
            artifact: Some((&bytes, info)),
        },
    );
    println!(
        "fixture recall report:\n{}",
        serde_json::to_string_pretty(&report).expect("report serializes")
    );

    assert!(induced.cover.regions.len() <= config.regions_budget);
    assert_eq!(info.node_count, 1 + induced.cover.regions.len() as u32);
    for depth in &recall {
        assert!(depth.evaluated > 0);
        for rate in [
            depth.reference_top1_recall,
            depth.reference_topm_recall,
            depth.class_coassignment_recall_top1,
            depth.class_coassignment_recall_topm,
            depth.class_coassignment_precision_top1,
            depth.class_coassignment_precision_topm,
        ] {
            assert!((0.0..=1.0).contains(&rate), "rate {rate} is a probability");
        }
        assert!(depth.frontier_width_max <= cover::TOP_M as u32);
    }
    // Double-run identity on the fixture too.
    let induced2 = cover::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)
        .expect("fixture induction");
    assert_eq!(induced.cover.kappa(), induced2.cover.kappa());
}
