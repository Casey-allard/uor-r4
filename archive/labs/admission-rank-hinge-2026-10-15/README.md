# D-R admission-rank objective — source and fixture scaffolding

**Status: NOT YET PROMOTED. Source implementation, not a model result or CUDA qualification.**
Codex/Sol authors this diff from `origin/main` `18b5ea475`. The contract is
[#2515 at `ab047f4c6`](https://github.com/UOR-Foundation/uor-r4/pull/2515/commits/ab047f4c605e13379e3e5ed0c245585c8449fb6f),
with [Codex co-design](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6196365768)
and [Kimi's six-fact audit](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6197008827).
Kimi subsequently recorded design FIT at sheet head `8a4388424`; that does not
qualify this source or release execution. The
[interface agreement](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6199294252)
explicitly ratifies the metadata compatibility boundary and NOT_RUN scaffolding.

**Review ownership:** Codex is now the author. D-R exact-head review belongs to
**Kimi PM's dispatched non-author reviewer**, superseding the sheet's earlier
Codex reviewer label. AGY owns the integrated sheet; this diff does not edit it.

## Question and implemented boundary

Does training an occupied-cell admission-boundary hinge prevent the admission
erosion measured in Stage A? This PR supplies source for that future contrast;
it neither answers the question nor runs the contrast.

- `ObjectivePolicy::NearestAddressMarginV1` keeps `8T + width*A`;
  `OccupiedCellRankHingeV1` selects `8T + width*R`. At the registered `B=4`,
  `width=32`, giving the matched `8T+32A` and `8T+32R` arms.
- The shared CUDA writer uses one explicit lexicographic
  `(address_distance, cell_id)` comparator for admission and target-cell rank.
  `r` is one plus the number of occupied cells preceding the target cell;
  `R=max(0,r-K)`. A crowded cell contributes once, regardless of token count.
  Virtual proposals recompute cells from virtual codebook moves using that same
  writer. Baseline token-rival eligibility and the wholly-ineligible status-8
  refusal are unchanged.
- Policy is bound into initialization, device bindings, snapshots, driver
  configuration/lineage, checkpoint validation, and evaluation metadata.
  Missing historical policy means nearest-margin only; the default is omitted
  on serialization to retain historical configuration bytes. Rank policy is
  explicit, and cross-policy checkpoint admission is refused.
- GPU evaluation emits raw T/A/R, both loss numerators, target-cell rank,
  occupied-cell count, admitted-token count, admission and admitted strict wins.
  CUDA reductions emit component sums, histograms, zero-row counts and
  nearest-rank quantiles. Means and shares are exact numerator/denominator
  records; denominator zero means undefined, not a floating fallback.
- `completion_baseline.rs` accepts stored `admitted_cells` in `1..=16` rather
  than `1..=8`. Existing pilot/completion release constraints are **not** widened;
  pilot admission now explicitly requires the historical objective.

## Receipt interface

The objective evaluation sub-schema is `uor-r4.discrete-objective-evaluation/1`.
The legacy `total_loss_numerator` and its aggregate remain **TA diagnostics**,
even on the R arm; `objective_loss_numerator` names the selected objective.
`loss_ta_numerator[_sum]` and `loss_tr_numerator[_sum]` are always explicit.
Driver receipts label the legacy semantics and attach target position/token
identity to each row. The existing source/config/data lineage is retained.

Scale disclosure consists of `token_hinge_sum`, `address_hinge_sum`,
`cell_rank_hinge_sum`, `weighted_token_hinge_sum`, `weighted_address_hinge_sum`,
`weighted_cell_rank_hinge_sum`, and exact `address_loss_share_ta` /
`rank_loss_share_tr`. Each hinge distribution contains
`min,p25,p50,p75,p90,p99,max,mean,zero_rows,histogram`.
The nearest-rank rule is `ceil(percent*rows/100)`; minimum is the first occupied
bin. Bounds are T≤640, A≤72, R≤255 over the supported widths/K; at K16, R≤240.
Arithmetic overflow refuses with status 12. Batch-summary histograms are reduced
on CUDA and quantiles are recomputed from the full histogram, not averaged.

G-Occ inputs are `occupied_cells`, `admitted_token_count_sum`, `scored_rows`,
and `mean_admitted_token_count`. Occupancy is invariant within a checkpoint,
not summed across rows/batches. The future paired-arm disposition reader must
apply the sheet's G-Occ thresholds to these inputs; **this PR does not implement
the paired experiment controller, disposition, or new release packet**.

The internal row ABI is 14 words; aggregate ABI is 1020 words, including three
exact histograms (641/73/256 bins) and 21 quantile words. Rust only decodes and
serializes these completed device values; no host model reducer is introduced.

## C-R1..C-R6 scaffolding, not qualification

`discrete-stack list-admission-rank-controls` is a metadata-only command. It
prints all six controls with `NOT_RUN`, authored tie/rank expectations, K1..16
and crowding axes, virtual/applied mutation cases, overflow and schema cases,
and the seed/K matrix for G-Pre. This is **not an executable GPU fixture suite**.
In particular, independent virtual-versus-applied fixture execution and the
checkpoint-0 corpus qualification remain owed. No control gets a PASS receipt
from the host metadata tests or from listing the scaffold.

## Validation and identities

Source/build checks only, on October 15; no model forward, evaluation, update,
GPU qualification, pod, rental, or training command was run.

| Check | Result and scope |
| --- | --- |
| `cargo test --locked --offline -p uor-r4-discrete --no-default-features` | 37 passed, 1 ignored (authenticated retained-baseline artifact required); metadata, static construction and source-contract tests, not model qualification |
| Metadata-only `list-admission-rank-controls` | Six NOT_RUN controls emitted; no CUDA feature or model invocation |
| `rustfmt --check` on changed Rust files; `git diff --check` | PASS |
| `uor-harness ratchet d23` / `d25` | PASS: no added prohibited softmax/dense-matrix sites; historical baselines unchanged |
| `uor-harness check` | 0 errors, 14 warnings: peer merged claims/presence, a historical unidentified receipt, pending sheets and unrelated unfrozen panels; not an all-green result |
| `python3 scripts/check_claim_wording.py` | PASS |
| `clang++ -std=c++17 -x c++ -fsyntax-only` with declarations-only CUDA keyword/index/atomic shims | PASS C++ syntax only; no object, PTX, CUDA qualification or host model execution |
| Isolated `cargo check --features cuda --all-targets` against repository-vendored cudarc | PASS Rust interfaces only, using generated static geometry and an explicitly non-executable PTX placeholder in an ignored local check package; **not a production CUDA build** |
| Production nvcc/PTX build, PTX admission, old finite controls, C-R1..C-R6, G-Pre | NOT_RUN; qualification/release gates remain open |

The old driver source hash
`e1022068eb45febf9d54204cbf003ae40fde8e040e2fc259cf57d2fd090f546d`
is intentionally replaced by
`3487506d0d6b9197e8b99963dfe92e91b5c3e33e0801d9116bf525cdc9b17fb7`.
The source-pin test records that change; it is not a continuation of the old
qualification. `discrete.cu` SHA-256 is
`f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`;
`objective.rs` SHA-256 is
`b813e855be6c4ea460c8711493527d52f1caf1a02bb09b046145f88c7f4ee4d0`.
The device source digest now includes `objective.rs`; a real kernel/PTX digest
and executable identity are **UNAVAILABLE pending an admitted production build**.

Preparation used an owned worktree and owned Cargo targets, four build jobs,
zero paid/model compute and zero pods. The two validation target directories
totaled about 499 MiB at the check (includes reused compilation artifacts);
40 GiB free disk remained. Peak RAM was not measured. No model-time ledger was
created or charged for these source/metadata checks.

Line: exact addressed memory (discrete rung0 admission redirect) · count 0/3 unchanged · M1 headline
37/232 → 37/232 unchanged. **Next:** non-author exact-head review, then separately
authorized fixture-executor completion/qualification and Kimi's D-Packet;
training/M2/Stage B remain HELD for the owner's release. Retain this branch and
worktree until protected delivery; after merge verify fresh main before cleanup.
