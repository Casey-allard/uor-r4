//! TLA3/TLA4/TLA5 + TLS1 → R4G1 migration converter (plan §5 Phase 1:
//! "TLA3/TLS1 → R4G1 migration converter (compiler-side tool) so existing
//! fixtures and the κ-reproduction test keep working").
//!
//! **Status (issue #280): container-format demo, not a prediction
//! path.** The converted artifact wires no per-node emission ranges and
//! emits only `kind=0` refinement edges, so a runtime walking it has no
//! readable emission structure — the 2026-07-29 held-out measurement
//! recorded 0.0% top1/agreement, and the row was retired from the
//! certify measurement matrix (the honest C row now measures the
//! serving surface via `uor-r4-api::serving_eval`). This module remains
//! as the format-validation and κ-migration exercise it was built as;
//! real emission/transition wiring is future work specified against the
//! #243/#248 architecture.
//!
//! The converter flattens the legacy transformerless artifact pair — a TLA
//! container ([`compiler::parse_artifacts`]) plus a TLS1 graded store
//! ([`runtime::parse_store`] or, for the pre-u32 era on-disk stores,
//! [`runtime::parse_store_legacy_u16`]) — into a single R4G1 container that
//! passes both validation stages of `uor-r4-graph-format`. The mapping is
//! fixed (no options beyond an optional radius calibration):
//!
//! - **Graph**: a synthetic root region (index 0, depth 0, **all ranges
//!   empty** — no child/forward/emission/prototype/mask wiring; its
//!   prototype and mask words in ROUT are zeros) plus the 1024 class
//!   regions — stage `k ∈ 0..4`, class `c ∈ 0..256` map to node index
//!   `1 + k*256 + c` at depth `k+1`. `depth_count = 5`. The root's
//!   `radius` is 0: it is the synthetic backoff floor, not a calibrated
//!   region. Class-node radii come from the calibration JSON
//!   (`acceptance_radius` per `(stage, class)`) when provided, else
//!   default to 288 (the full 288-bit signature width — uncalibrated
//!   means maximally permissive).
//! - **HEAD** (`W=5`, `signature_bytes=36`): the 36-byte class signatures
//!   ride in word-aligned 5-word (40-byte) storage per RFC §4.1.
//!   `A=32`, `C=16`, `E=64`, `K=8`, `D=64` are the RFC §4 starting
//!   defaults; `A` doubles as the honesty floor — RFC §6 item 7 requires
//!   HEAD to declare honest per-artifact maxima, so `A` rises to the
//!   observed max `child_len` when deeper prefix nodes fan out wider
//!   than 32. `E` stays 64: no PackedNode carries an emission list in
//!   v0 (see EMIT below). `vocab_size` comes from the artifact's token
//!   table.
//! - **CIDs**: `teacher_cid` is blake3 of the source TLA container bytes —
//!   the only teacher-derived identity recoverable at convert time (the
//!   container does not carry the teacher checkpoint CID).
//!   `compiler_version_cid` is blake3 of the fixed label
//!   [`COMPILER_VERSION_LABEL`]. The tokenizer and corpus CIDs and the
//!   `hf_revision` field are zeroed: TLA/TLS1 containers carry none of
//!   them, so there is nothing to migrate.
//! - **Fallback policy** bytes are all 0 = "unset": the D4 per-status
//!   policy is a deployment decision, not derivable from the store.
//! - **ROUT** section layout: `[program][padding to 8B][prototype
//!   words][mask words]` — a single `0x00 HALT` op zero-padded to 8 bytes
//!   (the v0 routing program is degenerate: routing is exhaustive nearest
//!   class in the migrated semantics), then 1025 × 5 u64 prototype words
//!   (root: zeros; class node: the class's 36-byte signature zero-padded
//!   to 40 bytes), then 1025 × 5 u64 mask words (root: zeros; class node:
//!   `0xFF` over the first 36 bytes, then zero padding).
//!   `PackedNode.prototype_word_start`/`mask_word_start` are u64-word
//!   offsets from the section start.
//! - **EDGE (E_r refinement)**: derived from store prefix adjacency. For
//!   every store level `d ∈ 1..=4` key `[s_0..s_{d-1}]` with a non-empty
//!   distribution: parent = the node for prefix `[s_0..s_{d-2}]` (for
//!   `d ≥ 2`; the prefix is identified by its deepest class node), else
//!   the root; child = the class node `(stage d-1, class s_{d-1})`; emit
//!   `(parent → child, kind=E_r, score_q=0)`, deduplicated. Every class
//!   node with no observed parent gets an edge from the root, so every
//!   class node keeps a refinement path from the root. The canonical edge
//!   array is sorted by `(src, dst)`, making each node's child range
//!   contiguous; the reverse index lists edge IDs sorted by `(dst, src)`
//!   with per-node ranges wired into the PackedNode forward fields (the
//!   v0 simplification: refinement reuses the forward range fields and
//!   overlap ranges stay empty; the reverse index itself satisfies the
//!   format's v0 Theorem-7 existence check — it is a permutation of all
//!   edge IDs).
//! - **EDGE (E_f forward transitions): intentionally empty.** The only
//!   transition compiler available (`transitions::
//!   compile_transitions_from_corpus`) is corpus-driven and keyed by a
//!   token→region assigner: it needs the token-ordered corpus stream and
//!   per-position region assignments. The migration inputs are the
//!   artifact container and the store — no corpus, and the store carries
//!   only aggregated `prefix → distribution` evidence, from which
//!   token-ordered transitions between regions cannot be recovered.
//!   Rather than fabricate edges, v0 ships none; a corpus-carrying
//!   converter revision can add them.
//! - **EMIT**: the root prior from `store[0]`'s empty-key distribution,
//!   carried as the section-level root prior block (RFC §5 "root prior
//!   block B(v)"): `(token u32 → count u32)` pairs encoded linearly as
//!   two i32 values per entry under the storage descriptor `{width: 2
//!   (i32), shift: 0, zero_point: 0}`. This is the **v0 linear-count
//!   migration encoding**: counts are stored verbatim, not log-domain
//!   scores; Phase 4 will residualize priors/emissions into the
//!   ScoreQ/log-domain EMIT table layout. No PackedNode emission ranges
//!   are wired in v0 — the root record's ranges stay empty and no other
//!   region carries an emission list.
//! - **EXCT**: storage descriptor `{width: 2, shift: 0, zero_point: 0}`
//!   followed by the **raw original TLS1 container bytes** as the opaque
//!   remainder — the migration carryover of the exact-context evidence,
//!   readable by the legacy parsers ([`runtime::parse_store`] /
//!   [`runtime::parse_store_legacy_u16`]) over the remainder slice.
//!
//! Sections not listed (CODE, PROV) are absent: the v0 draft-line stage-2
//! slice does not require them (RFC §9.4; mandatory-section completeness
//! is a later Phase-1 slice).
//!
//! Output is deterministic: identical inputs produce identical bytes
//! (B-tree iteration order everywhere, fixed section layout, the
//! canonical serializer). The CLI is
//! `r4 transformerless convert-r4g1 --artifacts <TLA> --store <TLS1>
//! [--calibration <hamming_calibration.json>] --out <R4G1>`.

use std::collections::BTreeSet;

use uor_r4_graph_format::{build_segment_lane, ArtifactBuilder, SectionId, SegmentLaneDescriptor};

use super::compiler::{Compiled, HammingCalibrationReport, D, K, SIG_BYTES, SIG_WORDS, STAGES};
use super::runtime::Store;

/// Node index of the synthetic root region.
pub const ROOT_NODE: u32 = 0;
/// Total node count: the root plus `STAGES × K` class regions (1025).
pub const NODE_COUNT: u32 = 1 + (STAGES * K) as u32;
/// HEAD `depth_count`: the root at depth 0 plus four class stages.
pub const DEPTH_COUNT: u8 = (STAGES + 1) as u8;
/// Default class-node acceptance radius: the full 288-bit signature
/// width (uncalibrated ⇒ maximally permissive).
pub const DEFAULT_RADIUS: u16 = D as u16;
/// blake3 input labeling this converter as the compiler of record.
const COMPILER_VERSION_LABEL: &[u8] = b"uor-r4-core convert-r4g1 v0";

/// Edge kind of refinement edges (matches `transitions::EdgeKind`).
const EDGE_KIND_REFINEMENT: u8 = 0;

/// HEAD starting defaults from RFC §4, applied as floors (see module
/// docs): honest observed maxima replace them when larger.
const DEFAULT_MAX_FRONTIER_WIDTH: u16 = 32;
const MAX_CANDIDATES: u16 = 16;
const DEFAULT_MAX_EMISSION_ENTRIES: u32 = 64;
const SHORTLIST_SIZE: u16 = 8;
const MAX_PROGRAM_STEPS: u32 = 64;

/// Class-region node index for `(stage, class)`: `1 + stage * 256 +
/// class`. The root ([`ROOT_NODE`]) is not a class node.
pub fn class_node_index(stage: usize, class: usize) -> u32 {
    debug_assert!(stage < STAGES && class < K);
    1 + (stage * K + class) as u32
}

/// What a conversion produced, for the CLI report and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionReport {
    /// Always [`NODE_COUNT`] (1025).
    pub node_count: u32,
    /// Canonical edges written (all E_r in v0).
    pub edge_count: u32,
    /// Store-derived refinement edges before root fallbacks.
    pub observed_refinement_edges: u32,
    /// Root fallback edges added for parentless class nodes.
    pub root_fallback_edges: u32,
    /// Store keys with non-empty distributions (levels 1..=4).
    pub observed_prefix_keys: u32,
    /// Entries in the EMIT root prior (store[0] empty-key distribution).
    pub root_prior_entries: u32,
    /// Class nodes whose radius came from the calibration report.
    pub calibrated_radii: u32,
    /// HEAD `A` actually declared (floor 32, raised to the observed max).
    pub max_frontier_width: u16,
    /// HEAD `E` declared (always 64 in v0: no node emission lists are
    /// wired; the root prior is a section-level EMIT block).
    pub max_emission_entries: u32,
    /// Length of the produced R4G1 container in bytes.
    pub artifact_bytes: usize,
}

/// Convert a parsed TLA artifact + TLS1 store into R4G1 container bytes.
///
/// `artifact_container` and `store_container` are the raw source files:
/// the former feeds `teacher_cid`, the latter is carried verbatim as the
/// EXCT remainder. `calibration`, when given, supplies per-class
/// acceptance radii. Errors are plain strings (CLI-facing tool).
pub fn convert(
    artifact_container: &[u8],
    artifacts: &Compiled,
    store: &Store,
    store_container: &[u8],
    calibration: Option<&HammingCalibrationReport>,
) -> Option<(Vec<u8>, ConversionReport)> {
    convert_inner(
        artifact_container,
        artifacts,
        store,
        store_container,
        calibration,
        None,
    )
}

/// Like [`convert`], but additionally emits the optional #835 segment-lane
/// `PSTATE` section (#836) carrying `descriptor`. The bytes stay deterministic
/// and every other section is unchanged, so a reader that ignores PSTATE sees
/// exactly the [`convert`] output (absent-section identity). Emitting the
/// descriptor is what makes the deployed segment lane non-inert on the produced
/// bundle.
pub fn convert_with_segment_lane(
    artifact_container: &[u8],
    artifacts: &Compiled,
    store: &Store,
    store_container: &[u8],
    calibration: Option<&HammingCalibrationReport>,
    descriptor: &SegmentLaneDescriptor,
) -> Option<(Vec<u8>, ConversionReport)> {
    convert_inner(
        artifact_container,
        artifacts,
        store,
        store_container,
        calibration,
        Some(descriptor),
    )
}

fn convert_inner(
    artifact_container: &[u8],
    artifacts: &Compiled,
    store: &Store,
    store_container: &[u8],
    calibration: Option<&HammingCalibrationReport>,
    segment_lane: Option<&SegmentLaneDescriptor>,
) -> Option<(Vec<u8>, ConversionReport)> {
    // Input shape honesty: the artifact must carry the 4 × 256 × 36-byte
    // class signature books the ROUT section is built from.
    if !artifacts.token_codes.len().is_multiple_of(STAGES) {
        return None;
    }
    let vocab = u32::try_from(artifacts.token_codes.len() / STAGES).ok()?;
    if artifacts.class_sigs.len() != STAGES
        || artifacts
            .class_sigs
            .iter()
            .any(|sigs| sigs.len() != K * SIG_BYTES)
    {
        return None;
    }

    // Radii: calibration where present, the 288 default elsewhere.
    let mut radii = [[DEFAULT_RADIUS; K]; STAGES];
    let mut calibrated_radii = 0u32;
    if let Some(report) = calibration {
        for region in &report.regions {
            let stage = region.stage as usize;
            let class = region.class as usize;
            if stage < STAGES && class < K {
                radii[stage][class] = region.acceptance_radius;
                calibrated_radii += 1;
            }
        }
    }

    // E_r refinement edges from store prefix adjacency (deduplicated;
    // the BTreeSet orders them by (src, dst) for free).
    let mut edge_set: BTreeSet<(u32, u32)> = BTreeSet::new();
    let mut observed_prefix_keys = 0u32;
    for (d, level) in store.iter().enumerate().take(STAGES + 1).skip(1) {
        for (key, dist) in level {
            if dist.is_empty() || key.len() != d {
                continue;
            }
            observed_prefix_keys += 1;
            let child = class_node_index(d - 1, key[d - 1] as usize);
            let parent = if d >= 2 {
                class_node_index(d - 2, key[d - 2] as usize)
            } else {
                ROOT_NODE
            };
            edge_set.insert((parent, child));
        }
    }
    // Every class node with no observed parent refines from the root.
    let mut has_parent = vec![false; NODE_COUNT as usize];
    for &(_, dst) in &edge_set {
        has_parent[dst as usize] = true;
    }
    let mut root_fallback_edges = 0u32;
    for stage in 0..STAGES {
        for class in 0..K {
            let node = class_node_index(stage, class);
            if !has_parent[node as usize] {
                edge_set.insert((ROOT_NODE, node));
                root_fallback_edges += 1;
            }
        }
    }
    let observed_refinement_edges = (edge_set.len() as u32) - root_fallback_edges;
    let edges: Vec<(u32, u32)> = edge_set.into_iter().collect();
    let edge_count = edges.len() as u32;

    // Per-node child ranges over the canonical array (sorted by
    // (src, dst), so each node's children are contiguous), and the
    // honest frontier width. The root record keeps all ranges empty, so
    // the observed max runs over class nodes only.
    let node_total = NODE_COUNT as usize;
    let mut child_start = vec![0u32; node_total];
    let mut child_len = vec![0u16; node_total];
    for (i, &(src, _)) in edges.iter().enumerate() {
        if child_len[src as usize] == 0 {
            child_start[src as usize] = i as u32;
        }
        child_len[src as usize] += 1;
    }
    let max_child_len = child_len[1..].iter().copied().max().unwrap_or(0);
    let max_frontier_width = DEFAULT_MAX_FRONTIER_WIDTH.max(max_child_len);

    // Reverse index: edge IDs sorted by (dst, src); per-node forward
    // ranges wire the contiguous per-dst runs into the PackedNode
    // forward fields (v0 simplification, see module docs).
    let mut reverse: Vec<u32> = (0..edge_count).collect();
    reverse.sort_by_key(|&id| {
        let (src, dst) = edges[id as usize];
        (dst, src)
    });
    let mut forward_start = vec![0u32; node_total];
    let mut forward_len = vec![0u16; node_total];
    for (i, &id) in reverse.iter().enumerate() {
        let dst = edges[id as usize].1;
        if forward_len[dst as usize] == 0 {
            forward_start[dst as usize] = i as u32;
        }
        forward_len[dst as usize] += 1;
    }

    // EMIT: descriptor + the v0 linear-count root prior, carried as the
    // section-level block; no PackedNode emission ranges are wired in v0.
    let mut emit = vec![2u8, 0, 0, 0]; // {width: i32, shift: 0, zero_point: 0}
    let mut root_prior_entries = 0u32;
    if let Some(dist) = store.first().and_then(|level| level.get(&[][..])) {
        for (&token, &count) in dist {
            let token = i32::try_from(token).ok()?;
            let count = i32::try_from(count).ok()?;
            emit.extend_from_slice(&token.to_le_bytes());
            emit.extend_from_slice(&count.to_le_bytes());
            root_prior_entries += 1;
        }
    }

    // ROUT: [HALT + padding][1025 × W prototype words][1025 × W mask
    // words]. Word 0 is the padded program.
    let mut rout = Vec::with_capacity(8 + node_total * SIG_WORDS * 8 * 2);
    rout.push(0x00); // HALT
    rout.extend_from_slice(&[0u8; 7]); // program padding to 8-byte alignment
    for i in 0..node_total {
        let mut words = [0u8; SIG_WORDS * 8];
        if i > 0 {
            let stage = (i - 1) / K;
            let class = (i - 1) % K;
            words[..SIG_BYTES]
                .copy_from_slice(&artifacts.class_sigs[stage][class * SIG_BYTES..][..SIG_BYTES]);
        }
        rout.extend_from_slice(&words);
    }
    for i in 0..node_total {
        let mut words = [0u8; SIG_WORDS * 8];
        if i > 0 {
            words[..SIG_BYTES].fill(0xFF);
        }
        rout.extend_from_slice(&words);
    }

    // NODE: 1025 packed 30-byte records. The root record is all zeros:
    // depth 0, radius 0, every range empty (see module docs).
    let mut node_section = Vec::with_capacity(node_total * 30);
    for i in 0..node_total {
        let (depth, radius) = if i == 0 {
            (0u8, 0u16)
        } else {
            let stage = (i - 1) / K;
            let class = (i - 1) % K;
            ((stage + 1) as u8, radii[stage][class])
        };
        let (child_start, child_len, forward_start, forward_len, prototype, mask) = if i == 0 {
            (0u32, 0u16, 0u32, 0u16, 0u32, 0u32)
        } else {
            (
                child_start[i],
                child_len[i],
                forward_start[i],
                forward_len[i],
                1 + (i as u32) * (SIG_WORDS as u32),
                1 + (NODE_COUNT + i as u32) * (SIG_WORDS as u32),
            )
        };
        node_section.extend_from_slice(&child_start.to_le_bytes());
        node_section.extend_from_slice(&child_len.to_le_bytes());
        node_section.extend_from_slice(&forward_start.to_le_bytes());
        node_section.extend_from_slice(&forward_len.to_le_bytes());
        node_section.extend_from_slice(&0u32.to_le_bytes()); // emission_start
        node_section.extend_from_slice(&0u16.to_le_bytes()); // emission_len
        node_section.extend_from_slice(&prototype.to_le_bytes());
        node_section.extend_from_slice(&mask.to_le_bytes());
        node_section.extend_from_slice(&radius.to_le_bytes());
        node_section.push(depth);
        node_section.push(0); // flags
    }

    // EDGE: the canonical array (16-byte records, kind E_r, score_q 0)
    // followed by the reverse index.
    let mut edge_section = Vec::with_capacity(edges.len() * 20);
    for &(src, dst) in &edges {
        edge_section.extend_from_slice(&src.to_le_bytes());
        edge_section.extend_from_slice(&dst.to_le_bytes());
        edge_section.extend_from_slice(&0i32.to_le_bytes()); // score_q
        edge_section.push(EDGE_KIND_REFINEMENT);
        edge_section.push(0); // flags
        edge_section.extend_from_slice(&0u16.to_le_bytes()); // reserved
    }
    for &id in &reverse {
        edge_section.extend_from_slice(&id.to_le_bytes());
    }

    // EXCT: descriptor + the raw TLS1 carryover.
    let mut exct = Vec::with_capacity(4 + store_container.len());
    exct.extend_from_slice(&[2, 0, 0, 0]);
    exct.extend_from_slice(store_container);

    // HEAD: the fixed 224-byte v0 prefix.
    let head = head_payload(
        artifact_container,
        vocab,
        edge_count,
        max_frontier_width,
        DEFAULT_MAX_EMISSION_ENTRIES,
    );

    // Canonical container: 64-byte section alignment (RFC §2
    // cache-line-sensitive sections).
    let mut builder = ArtifactBuilder::new(6);
    builder.add_section(SectionId::HEAD, 0, &head);
    builder.add_section(SectionId::NODE, 0, &node_section);
    builder.add_section(SectionId::EDGE, 0, &edge_section);
    builder.add_section(SectionId::ROUT, 0, &rout);
    builder.add_section(SectionId::EMIT, 0, &emit);
    builder.add_section(SectionId::EXCT, 0, &exct);
    // #836: optional segment-lane PSTATE section. The prompt-derived segment
    // lane ships a config-only descriptor (empty residual table), so this adds
    // the lane's capacity/decay/score constants without any learned table.
    // Emitted only when a descriptor is supplied; otherwise the container is
    // byte-identical to `convert` (absent-section identity).
    if let Some(descriptor) = segment_lane {
        let pstate = build_segment_lane(descriptor, &[]).ok()?;
        builder.add_section(SectionId::PSTATE, 0, &pstate);
    }
    let bytes = builder.build().ok()?;

    let report = ConversionReport {
        node_count: NODE_COUNT,
        edge_count,
        observed_refinement_edges,
        root_fallback_edges,
        observed_prefix_keys,
        root_prior_entries,
        calibrated_radii,
        max_frontier_width,
        max_emission_entries: DEFAULT_MAX_EMISSION_ENTRIES,
        artifact_bytes: bytes.len(),
    };
    Some((bytes, report))
}

/// Serialize the fixed 224-byte HEAD prefix (v0 draft line, RFC §4.1).
/// `artifact_container` feeds `teacher_cid` (see module docs); the
/// tokenizer/corpus CIDs and `hf_revision` are zeroed.
fn head_payload(
    artifact_container: &[u8],
    vocab_size: u32,
    edge_count: u32,
    max_frontier_width: u16,
    max_emission_entries: u32,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(224);
    out.extend_from_slice(blake3::hash(artifact_container).as_bytes()); // teacher_cid
    out.extend_from_slice(&[0u8; 32]); // tokenizer_cid: not carried by TLA/TLS1
    out.extend_from_slice(&[0u8; 32]); // corpus_construction_cid: zeroed
    out.extend_from_slice(&[0u8; 32]); // corpus_certification_cid: zeroed
    out.extend_from_slice(&[0u8; 20]); // hf_revision: zeroed
    out.extend_from_slice(blake3::hash(COMPILER_VERSION_LABEL).as_bytes());
    out.extend_from_slice(&max_frontier_width.to_le_bytes()); // A
    out.extend_from_slice(&MAX_CANDIDATES.to_le_bytes()); // C
    out.extend_from_slice(&(SIG_WORDS as u16).to_le_bytes()); // W
    out.extend_from_slice(&SHORTLIST_SIZE.to_le_bytes()); // K
    out.extend_from_slice(&max_emission_entries.to_le_bytes()); // E
    out.extend_from_slice(&MAX_PROGRAM_STEPS.to_le_bytes()); // D
    out.extend_from_slice(&NODE_COUNT.to_le_bytes());
    out.extend_from_slice(&edge_count.to_le_bytes());
    out.push(DEPTH_COUNT);
    out.extend_from_slice(&[0u8; 5]); // fallback policy: unset (module docs)
    out.extend_from_slice(&[0u8; 2]); // reserved
    out.extend_from_slice(&(SIG_BYTES as u16).to_le_bytes()); // signature_bytes
    out.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_major (draft line)
    out.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_minor
    out.extend_from_slice(&0u16.to_le_bytes()); // feature_bits_required
    out.extend_from_slice(&vocab_size.to_le_bytes());
    debug_assert_eq!(out.len(), 224);
    out
}
