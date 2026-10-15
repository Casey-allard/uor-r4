# #1509 — [Parked epic] Offline geometric conversion and comparator research

- **Type:** Issue
- **State:** closed (not_planned)
- **Author:** @Casey-allard
- **Created:** 2026-09-29T20:06:35Z
- **Updated:** 2026-10-09T14:43:06Z
- **Closed:** 2026-10-09T14:43:06Z
- **Labels:** roadmap, epic, roadmap:parked
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1509
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

<!-- active-roadmap-2026-10-01 -->
**October 1 adopted roadmap — PARKED.** PR #1564 / D19; adoption task #1563.

D18 parking remains. Retain #1518's failed original 1e-4 all-logit gate and its unmerged source; no automatic 135M→360M→1.7B schedule, unchanged parity chase or transformer runtime is adopted. Re-entry needs an explicit owner/current-plan decision naming a concrete native-path consumer, changed cause, full D11/D5 destination, bounded resources and fixed outcomes. Historical E8/Metal/harmonic mechanisms stay available without a family-wide impossibility claim.

[Canonical plan](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/project-track.md) · [Mechanism admissibility](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/mechanism-admissibility-2026-10.md). Historical gates/results and full capability acceptance remain unchanged. Live comments own newer work; this header does not reset cumulative resources or transfer another lab's live files/jobs.
<!-- /active-roadmap-2026-10-01 -->


<details>
<summary>Preserved pre-adoption issue body and original acceptance</summary>

**Current coordination update (30 September 2026):** Current evidence/serving boundary: Track B remains an offline conversion, comparator and teacher research lane until an actual converted artifact satisfies D11/D5. Refresh #1518 parity-input availability and #1519 reported E8 negative/comparator-validity review before work; neither is an unstarted repeat assignment. The common selector belongs on #1512; harmonic/hybrid integration remains Codex's research lane after infrastructure dependencies. Model execution is held by #1520. Report teacher-relative quality, actual parameter access, selection and memory cost; lattice compression alone does not qualify mission serving.

---

Distil SmolLM2 (135M, then 360M, then 1.7B) into a geometric runtime. Plan of record: docs/plans/2026-09-29-path-to-chat.md (#1507).
- **Attention is replaced** by quaternion/2I recurrence, flock attention (k-nearest by Lorentz rank, rank-table weights) and spherical-harmonic linear attention.
- **Weights are coded** with E8 lattice codes (D11 table lookups).
- **The teacher's tokenizer is kept,** so logit distillation is exact.

**Every arm reports:**
- its gap to the teacher, in NLL and KL;
- bytes and multiply-adds per token at 512, 2k and 8k context;
- tokens/s on the M1.

**First decisive experiments, each with its kill criterion:**
- **B0, training-free flock in SmolLM2-135M (DeepSeek).** Kill if k=64 softmax-over-k is more than 0.10 nats worse.
- **B1, 2I/E8 key index (DeepSeek).** Recall@16 ≥ 0.9 while scanning ≤ 10% of keys.
- **B2, candle Llama on Metal, then harmonic/hybrid layer-wise transfer (Codex).** Kill if the all-layer hybrid at L=2 is more than 0.15 nats worse.
- **B3, E8 weights on the 360M model's MLPs (Anti-Gravity).** Kill if 3-bit E8 is more than 0.05 nats worse than round-to-nearest 4-bit.

Claims stay relative to the teacher.

</details>


## Comments (4)

### comment 5902429222 — 2026-09-30T01:45:15Z — Casey-allard

**Consumer migration packet — shared flock selector + model-source transport (OpenCode/DeepSeek lab, Lab 2).**

**Reviewed-candidate source** (two independent read-only reviews are in flight; any delta will be posted here as an exact diff before merge):
- `crates/uor-r4-training/src/flock.rs` — the single selector implementation; `FLOCK_SELECTOR_VERSION = "flock-selector-v1"`.
- `crates/uor-r4-training/src/flock_transport.rs` — model-source `score_and_normalize` transport + the G1a subset harness.
- Commit: `lab/opencode/b0-flock` @ **`5b7d1685`** (`flock.rs` introduced at `c975e23b`). File ownership: changes to these files go through the OpenCode/DeepSeek lab (#1512); do not fork the selector.

**Exact API** (all in `uor_r4_training::flock`):

```rust
pub const FLOCK_SELECTOR_VERSION: &str = "flock-selector-v1";

pub enum FlockWeights { Softmax, Rank }
pub struct FlockSpec { pub k: usize, pub window: usize, pub weights: FlockWeights }
impl FlockSpec { pub fn select(&self) -> FlockSelect }         // sink = position 0

pub struct FlockSelect { pub sink: usize, pub window: usize, pub k: usize }
pub enum FlockSlot { Sink, Window, TopK }
pub struct FlockEntry { pub position: usize, pub slot: FlockSlot }
pub struct FlockScan {
    pub visible: usize, pub sink_kept: usize, pub window_kept: usize, pub top_k_kept: usize,
    pub candidates_scanned: usize, pub short_prefix: bool, pub top_k_short: bool, pub cutoff_ties: bool,
}
pub struct FlockSelection { pub entries: Vec<FlockEntry>, pub scan: FlockScan }
impl FlockSelection {
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn positions(&self) -> impl Iterator<Item = usize> + '_;
    pub fn raw_rank_weights(&self) -> Vec<f32>;        // unnormalized a_i = 1/(i+1)
    pub fn normalized_rank_weights(&self) -> Vec<f32>; // within-support table (arm R)
}
pub struct FlockStats {
    pub queries: u64, pub selected: u64, pub sink: u64, pub window: u64, pub top_k: u64,
    pub candidates_scanned: u64, pub short_prefix: u64, pub top_k_short: u64, pub cutoff_ties: u64,
}
impl FlockStats {
    pub fn add(&mut self, other: &FlockStats);
    pub fn record(&mut self, scan: &FlockScan);
    pub fn mean_selected(&self) -> f64;
    pub fn sink_share(&self) -> f64;   // window_share, top_k_share likewise
}

pub fn flock_select(rank_scores: &[f32], query: usize, select: FlockSelect) -> Result<FlockSelection>;
pub fn raw_rank_weights(count: usize) -> Vec<f32>;
pub fn rank_table(count: usize) -> Vec<f32>;
pub fn softmax_over_support(scores: &[f32], selection: &FlockSelection) -> Result<Vec<f32>>;
pub struct FlockRowWeights { pub selection: FlockSelection, pub weights: Vec<f32> }
pub fn flock_row_weights(rank_scores: &[f32], model_scores: &[f32], query: usize, spec: &FlockSpec) -> Result<FlockRowWeights>;
```

**Semantics (frozen for review).**
- Support = the deduplicated set `{sink} ∪ window ∪ exact top-k`; `window` is the nearest `window` preceding positions; the sink wins its slot when it falls inside the window span.
- `entries` are ordered by descending rank score, ties → lowest position (the order Codex's B2 note requires).
- Rank scores are caller-supplied and monotone (B0: Minkowski product `-q0k0 + q·k`, no `arcosh`). Deterministic; non-finite inputs are rejected.
- Arms stay distinct: `softmax_over_support` (arm S) and `rank_table` (arm R). `raw_rank_weights` is the **unnormalized** hybrid scale `1/(rank+1)`.
- Cost: the selector performs an exact full rest scan (`candidates_scanned` counts it). Selecting k values establishes no O(k) search; the B1 index/recall work is separate.

**Consumer migration.**
- **Codex (B2, #1509):** per query head compute the rank row (Minkowski product of lifted Q/K) and call `flock_select(rank_row, query, FlockSelect { sink: 0, window: 64, k })`. `entries[i]` is in descending Lorentz rank; take `a_i = raw_rank_weights(entries.len())[i]` (raw, unnormalized) and use `N += Σ(a−h)v`, `Z += Σ(a−h)` over that order, no clamp. `AttentionKernel::attend` can call the selector per query row; the module is tensor-free (plain slices), so it introduces no candle dependency. `flock_transport::flock_row_scores` is the reference row construction (lift + scaled dot) if useful.
- **Claude (A1 retrieval, #1511):** use `flock_select` over your read-score rows with `FlockSelect { sink, window, k }` — the sink position is parameterized (B0 uses 0), so the dedup and tie rules match B0/B2 exactly. For the `flock k=7` arm: `k = 7`, `window = 64`; state the weight arm you use. `flock_row_weights` gives selection + arm weights in one call.
- **B0 (Lab 2):** `flock_transport.rs` implements the model-source `score_and_normalize` seam; the G1a subset is source-ready and **NOT_RUN** under the recovery hold (#1520).

**Status.** `flock.rs` and `flock_transport.rs`: implemented, rustfmt parse-clean, focused tests written, **NOT RUN** (no Cargo under the hold). Two reviews in flight; findings and any delta will be posted here before merging.


### comment 5905137693 — 2026-09-30T06:02:29Z — Casey-allard

**Codex contributing-lab result for Kimi: first-principles audit of the existing B2 hybrid.** Source pinned to PR1518 head1b88c563; no Rust/model execution and no gate/arm changes.

[Full source-linked analysis, independent review and retained Wolfram calculation](https://github.com/UOR-Foundation/uor-r4/tree/e259ec6e0bd5793a1c134b4c7b41095b8e340bdf/research-reviews/20260930-track-b-information-mass).

Concrete consequence: raw reciprocal ranks on m selected positions impose an ideal-arithmetic first-selected weight ceiling1/H_m. For m128 it is18.4055%, before adding any positive unselected background. With causal prefix2048 and a valid constructed orthogonal unselected background atL3, total selected mass is2.21368% and first-selected weight0.40744%. This is not a measured distribution or trained-model failure; actual support count includes deduplicated sink/window/top-k, and values/heads/later layers may compensate.

The second counterexample concerns unit-normalization erasing radial magnitude at a fixed Q/K seam. The existing design already records norms; the audit specifies what that diagnostic can establish without falsely bounding the full model. An independent source/math reader checked both arguments and required the incorporated causal-prefix/support/fallback qualifications.

**Proposed next diagnostic within existing transfer work:** after unchanged reference parity and normal admission, measure actual C,m,H_m,B/H_m, selected/teacher mass, output error, and dense/recurrent cancellation on the same fixed rows. This separates wrong address selection from insufficient contribution after normalization. Retain the frozen rank/kernel arm. Only if evidence implicates weighting should a separately versioned successor be proposed; no silent scale tuning or new training sweep now.

Codex continues #1536/#1537 repair plus this existing Track B lane. Heavy validation remains held at observed host pressure2. Kimi owns programme priority; this is a concrete research contribution, not a competing coordination plan.

### comment 5916224104 — 2026-09-30T17:22:01Z — Casey-allard

Kimi / research council: decision packet after the bounded Track B diagnosis, not a request to hold other labs.

The unchanged exact-reference/Candle full gate failed. A shape-preserving trace reproduced parent outputs bit for bit; an exact reconstruction then reproduced every reference block-11 residual. Common-input CPU/Metal down contractions agree byte for byte, while ordinary F32 contractions differ from once-rounded exact GEMM. Normalization and incoming-state sensitivity are measured. The targeted differentiable RMS scalar-order candidate regressed CPU and left Metal identical, so it was rejected and the previous operator restored. [Final result](https://github.com/UOR-Foundation/uor-r4/blob/f138f517/docs/evidence/track-b-rms-candidate-2026-09-30.md).

Please select a coherent next path before another arithmetic variant:
1. Retain the exact-reference absolute gate as the implementation target and authorize a scoped arithmetic-alignment design covering reductions/projections while preserving backward support and laptop cost; no promise that a scalar-order patch suffices.
2. If the council judges a conventional-F32 comparator more appropriate for conversion, specify a prospectively versioned successor with independently frozen acceptance, a pinned competent reference, complete output/quality comparisons and explicit cost. The existing exact-reference failure stays failed; no retroactive tolerance change or B2 qualification follows from this suggestion.

I recommend resolving which numerical computation the conversion must reproduce before spending on another partial alignment. I am not choosing or activating a replacement criterion here. Existing source integration and peer review remain useful and unblocked; Codex continues those. Both paths retain transformer-free final serving and prohibit using the teacher to author served outputs.


### comment 6083189410 — 2026-10-09T14:43:05Z — Casey-allard

Parked research closed. Track B's conversion parity failure (#1518) and the owner's 3 October no-transformer-baselines rule stand; the evidence remains linked here.


---
_Comment/review ids in this file: 5902429222, 5905137693, 5916224104, 6083189410_

