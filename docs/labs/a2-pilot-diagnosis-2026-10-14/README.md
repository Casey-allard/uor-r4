# A2 pilot diagnosis and completion admission — 14 October 2026

**Training and M2 remain HELD.** This delivery analyzes the already-preserved
seed-1 pilot and prepares D5 completion admission. It runs no model computation,
changes no numerical kernel, and grants no release. Kimi owns the prospective
[completion sheet](../../harness/prep/m1-discrete-rung0-completion-2026-10-14.md);
Claude owns its independent FIT and the exact-source D5 review.

Line: evaluation integrity; milestone-owned count 1/3 unchanged. M1 reply-quality
headline remains SUSPENDED (R14). The pilot is diagnostic, not a mechanism verdict.

## Why ranks improved while loss rose

The [pilot record](../a2-pilot-2026-10-14/README.md) binds the exact source,
executable, panel, configuration and preserved report. The implemented writer
uses `8 * token_hinge + width * address_hinge`; width is 32 and the per-row
denominator is 256. The recorded changes therefore decompose exactly:

| Quantity | Baseline | Pilot end | Delta |
|---|---:|---:|---:|
| Token hinge sum | 679,708 | 671,712 | −7,996 |
| Weighted token contribution | 5,437,664 | 5,373,696 | −63,968 |
| Address hinge sum | 261,824 | 265,948 | +4,124 |
| Weighted address contribution | 8,378,368 | 8,510,336 | +131,968 |
| Total numerator | 13,816,032 | 13,884,032 | **+68,000** |

This is not an arithmetic inconsistency. Top-1/top-8 count correct identities in
the admitted list, ordered by distance then token ID. The hinge additionally
requires token and address margins. Every accepted update decreases its own
current TRAIN batch loss, not the loss of the complete, different DEV panel.

The source permits an additional exact inference from existing CUDA receipts.
`writer` selects distinct occupied cells, and counts every selected cell except
the target's cell as an address rival. For each row:

```
target_cell_admitted = admitted_cells - address_rivals   // exactly 0 or 1
```

Summing the stored metrics gives **5,680 → 2,180 admitted targets** out of 14,287,
so omission rose **8,607 → 12,107**, even as top-8 hits rose **173 → 1,169**.
Meanwhile token rivals fell **29,254,096 → 3,631,257**, occupied cells grew
**16 → 124**, and address-margin-satisfied rows fell **331 → 3**. The shared
writer separated some token representations while its fixed eight-cell access
budget reached fewer target cells. Reduced competition is consistent with better
within-list ranks. It does not establish which individual parameter change
caused a gain or loss.

The full archived report includes individual GPU row receipts, so paired
admission/rank/loss accounting can be performed without running the model again.
The [metadata audit](metadata-audit/src/main.rs) binds the original report SHA,
checks fixed DEV lineage/order and aggregate consistency, and records the paired
results in [diagnosis.json](diagnosis.json). Its row identity is the ordinal in
the pinned evaluator's ascending, once-only DEV traversal, not a new sampled row.
No host-side distance, routing, forward, predictor or model oracle is executed.
Tie counts and causal attribution to individual moves were not recorded and
remain UNOBSERVED; they must not be inferred from aggregate gains.

Source: `crates/uor-r4-discrete/src/discrete.cu`, `writer`,
`discrete_evaluate`, `discrete_evaluation_aggregate`; `training.rs`,
`PreparedDriver::evaluate` and `transition`. All remain numerically unchanged.

## Initialization: a coupled-index defect, not an E8 capacity result

The initializer chooses each root from the low four bits of a xorshift of
`seed XOR block_index XOR constant`. Symbolic expansion of that function gives:

```
low4(X(x)) = low4(x) XOR low4(x >> 4) XOR low4(x >> 17)
r_b = C_seed XOR ((t & 3) << 2) XOR ((t >> 2) & 15) XOR b
```

Here token `t` is 0..4095 and its block `b` is 0..3; `block_index = 4*t+b`
is below 16384. Thus all four root choices vary together through one four-bit
quantity. There are exactly **16 full-vector patterns, each assigned to 256 token
identities**, for every fixed seed at this shape. The endpoint pairs likewise
number 16. Changing the seed changes the common root labels; it does not remove
the shared collision partition. This is symbolic source analysis, corroborated
by the recorded initial 16 patterns / 16 cells, not a host initializer run.

The root orbit has sixteen members per 8D block. Four independently varied
blocks permit 16^4 combinations, so the sixteen-pattern collapse is imposed by
the initializer's coupling. This capacity count does not establish language
capacity, full E8 coverage, or usable contextual geometry.

**Proposed remedy for a separately registered successor:** construct an injective seeded
assignment into valid four-block root tuples, with a declared, measured endpoint
occupancy distribution. Retain actual SpiralCore transport, FBS composition,
one shared input/carry/output codebook, exact token identity, and geometric cell
selection from the vectors. A token ID may index its stored vector; it must not
become a semantic distance or bypass geometric admission. Pin the construction
and its capacity refusal before qualification. A shuffle/hash substitution alone
is not evidence of collision freedom.

Its exact CUDA construction controls must cover uniqueness over the whole
configured vocabulary, root membership, deterministic repeat/serialization,
seed behavior, shared-writer and transport compatibility, and a typed refusal
when the requested capacity is unsupported. **Full-vector uniqueness is distinct
from cell uniqueness:** 4,096 identities cannot occupy 4,096 unique endpoint
cells when only 256 cells exist. Learning may also legitimately merge vectors;
initial uniqueness is not an unreviewed constraint on every subsequent update.

The current completion sheet deliberately retains the original initialization.
Applying this remedy would start a separately registered configuration; it must
not rewrite or discard the pilot's measured trajectory.

## Coverage: three quantities that must stay separate

1. **Parameter coverage.** The pilot visited 1,640 of 16,384 vector-block
   coordinates: four distinct block proposals for each of 410 token identities,
   one proposal per block. The remaining pass visits tokens 410..4095 and then
   twenty Route coordinates. Already-visited frequent-token blocks receive no
   second proposal in this pass. Route is untested in the pilot, and flock/gates
   remain outside the completion configuration.
2. **Target mass represented by those parameter identities.** The visited IDs
   include 236 zero-count IDs but hold 1,440,407 / 2,425,925 masked-TRAIN target
   counts and seven of the count reference's top eight identities. This is about
   59.4% of target mass, not a rare-token slice. A zero target-count vector can
   still participate as input context or as a writer competitor; it is not
   automatically dead or a wasted update.
3. **Actual supervision exposure.** The pilot consumed 104,960 masked targets
   in ascending corpus order. A complete 16,404-coordinate pass consumes
   **1,049,856 of 2,425,925** eligible TRAIN target positions, with no corpus
   wrap. It leaves **1,376,069** positions unvisited. Both seeds use the same
   corpus order, so their combined exposure count does not double the unique
   positions covered. A full parameter pass is not a full corpus epoch.

The completion's diagnostic tables should report all three quantities plus
accepted/sensitive proposals by parameter kind, zero/nonzero target-count cohort,
and the existing P2 pattern/cell counts. On a shortfall, follow R12's instrument,
fixture, bar and mechanism attribution order; the schedule is a concrete fixture
question, never a rule that automatically declares geometry unsuccessful.
The named schedule remedies are a second coordinate pass or a prospectively
specified revisit of frequent-target blocks after the rest of the codebook has
moved. Neither is silently inserted into the unchanged completion pass.

Collision-free initialization, cell access, parameter revisits and corpus exposure
are distinct interventions. Changing them together would obscure which change
helped. No loss weight, cell budget, bar, target ordering or seed is altered here.

## D5 delivery and review boundary

The completion controller is admission-only. It retains the pilot cap and old
release semantics, requires separate explicit release scopes for seed 1 and
seed 2, imports only the pinned checkpoint 1,640 for seed 1, and records vector
and Route counters separately through the registered 16,404-coordinate pass.
Registered checkpoint evaluations remain bound to their exact saved cursor.

The existing driver hash includes controller and CLI source. Consequently D5
necessarily changes driver lineage. Seed-1 import checks the exact archived
parent digest and permits only the explicit driver-metadata transition; the
model bytes, numerical identities, data/configuration, cursor and original
baseline remain bound to the pilot. The old baseline is retained with its old
lineage, and new evaluations carry the completion lineage. This is recorded
migration, not a claim that old and new executables are identical.

D33 delegated execution only for the completed pilot. Completion release is
owner-personal, with a fresh packet and decision; no lab creates an owner record
in this work. The generic training hold stays in place. Source delivery, metadata
checks and a scientific FIT each remain separate from execution permission.

Validation, exact source review and delivery identifiers are recorded below when
available. No future qualification or review is presumed by this document.

## Resource and evidence scope

This work uses source inspection and Rust analysis of saved CUDA receipts on the
laptop. No model operations or new GPU rental are run. Local builds are isolated
under the claimed worktree with four build jobs; projected additional retained
source/results are under 1 MB, temporary build/artifact storage under 2 GB, and
available disk at task entry was 50 GiB against the 30 GiB floor. Original result
archive and all other labs' worktrees are preserved. The legacy model-time path
named in the research skill was absent; current pilot costs remain in the
[canonical preserved record](../a2-pilot-2026-10-14/README.md), not reset here.

The completion sheet's approximately $29 two-seed projection is an extrapolation
from the pilot's controller time, not measured completion throughput. Changing
occupancy, candidate mix, twenty Route proposals, registered evaluations, recovery
checkpoints, build/setup and preservation all affect it. Keep those components
visible and charge actual work cumulatively; a self-set estimate does not justify
stopping a healthy authorized run. No spending is incurred by this record.

## Paired receipt results

The Rust audit verifies all 14,287 row pairs and both source digests. In particular:

| Target-cell admission transition | Rows | Token contribution delta | Address contribution delta | Total loss numerator delta | Final top-8 hits |
|---|---:|---:|---:|---:|---:|
| Retained | 580 | −7,616 | +896 | −6,720 | 186 |
| Lost | 5,100 | +170,912 | +790,144 | +961,056 | 0 |
| Gained | 1,600 | −123,488 | −465,280 | −588,768 | 983 |
| Never admitted | 7,007 | −103,776 | −193,792 | −297,568 | 0 |
| **Total** | **14,287** | **−63,968** | **+131,968** | **+68,000** | **1,169** |

Top-8 hit transitions are 35 retained, 138 lost, 1,134 gained and 12,980 absent
at both endpoints. Of the 1,134 gained-hit rows, **72 have higher loss**. These
are exact associations in stored measurements, not isolated counterfactual effects.
The zero-TRAIN-target coordinate cohort has 944 attempts / 679 sensitive proposals /
90 accepted; the nonzero cohort has 696 / 500 / 146. Zero target count demonstrably
does not imply objective inactivity.

The existing strict-win guard is a distance-margin measurement, not necessarily
an admitted writer hit. Its GPU predicate is `token_rivals > 0 && token_hinge < 32`;
it does not require target-cell admission. The audit separates these quantities:

| Stored strict-win rows | Baseline | Pilot end |
|---|---:|---:|
| Total strict wins | 105 | 149 |
| Target admitted (also top-1 and top-8 correct) | 0 | 36 |
| Target omitted | 105 | 113 |

All admitted strict-win rows satisfy top-1 correctness in the preserved receipts.
The guard therefore cannot be described as 105 → 149 executed-writer wins.
This diagnostic changes neither its predicate nor the registered bar. Kimi and
Claude have the [written interpretation request](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6164092169)
for the prospective sheet; no revised FIT is inferred here.

Claude independently checked the admission inference and requested B3 on the
completion sheet: top-1/top-8 hits are bounded by target-cell admission. The
pilot-end ceiling 2,180 is below the unchanged top-8 bar 4,775. Completion can
only meet that bar if admission recovers. This is a measured prerequisite, not
a mechanism rejection or a license to lower the bar. The matching separately
registered successor is an admission/address-objective configuration, with
explicit target-admission and within-list rank accounting; its implementation
and test-fitness are still pending. Do not silently change K or objective weights.
[Claude B3](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163704400).

## Reproduction and operational commands

The audit takes the preserved full `report.json`, `steps.jsonl`, and a new output
file. It checks their exact SHA-256 digests before reading any metrics. Reuse of
an existing output is refused. The original files are in the cloud archive named
by the pilot record; their local restore is task-scoped and discarded only after
preservation checks.

```
CARGO_TARGET_DIR=local/analysis-target ~/.cargo/bin/cargo run --offline \
  --manifest-path docs/labs/a2-pilot-diagnosis-2026-10-14/metadata-audit/Cargo.toml \
  -- REPORT.json STEPS.jsonl NEW_DIAGNOSIS.json
```

D5's new CLI surfaces (the words below are usage, not executed releases):

- `completion-build-identity CONFIG` emits executable/compiled/driver/config/lineage
  pins without creating a CUDA context, using a CUDA-enabled build.
- `completion-check-plan PLAN` validates metadata without an owner record or apply
  capability. Kimi prepares one plan per seed; seed 1 includes the exact parent
  FilePin, and seed 2 must have no parent.
- `completion-owner-release PLAN NEW_RELEASE_DIR` is **Casey's personal terminal
  action only**, with `UOR_LAB` unset and a typed phrase binding that seed and dose.
  It is not run by a lab or inherited from D33.
- `completion CONFIG ADMISSION NEW_REPORT_DIR` starts the separately released seed
  in a fresh operational directory. `completion-resume` requires the same
  admission, a new report directory and its validated durable checkpoint.

Both retain source/config/model identities, immutable baseline and checkpoint
records, and independent vector/Route sensitivity counters. Quarter evaluations
are sealed with digest pins and checked before resume applies another update.
A persisted valid score is reused; if a crash occurs before an evaluation is
persisted, that unpersisted evaluation may have to run again and its cost is
charged. No claim of exactly-once GPU execution across arbitrary crashes is made.
Report/state/release directories cannot overlap the archived parent directory.
Fixed-name run-state JSON publishes through a unique pending file, complete write,
file sync, rename and directory sync while holding the run lock. Stray pending
files are preserved and do not block a retry. Immutable final files refuse
replacement; only the latest-checkpoint pointer is replaceable. This repairs
Claude's R1 interruption finding without changing numerical execution.

The measured pilot checkpoint occupies 3,138,466 bytes uncompressed. Retaining
roughly 260 checkpoints per complete seed is on the order of **0.8 GB per seed**
before quarter evaluations, logs and overhead, although repeated baseline data
compresses well in archives. Pod working-storage admission must budget the
uncompressed files, not reuse the pilot's ~6 MB compressed archive size as a
completion-storage estimate. No new pod has been started to measure completion.

Archive completion run-state and all attempt directories together: evaluation
FilePins may refer to checkpoint files in earlier attempts. Keep the original
pilot parent available for every seed-1 resume. No archive operation copies the
owner-release record; its preservation stays with its owner under R15.

## Validation scope

- Scoped Rust metadata suite: **25 tests passed**, including six D5 controls
  for lineage-only migration, seed/pilot isolation, owner-plan binding, rejection
  of the pilot owner format, corrupted evaluation receipts, and atomic run-state
  publication/recovery with preserved pending files. These are
  admission/data-format tests, not model compute or CUDA qualification.
- Standalone receipt auditor: PASS on the preserved full report and steps; all
  14,287 row pairs and 1,640 steps checked. Existing-output and wrong-input-digest
  refusals exercised. No floating-point oracle or host model path.
- Source review independently checked the imported parent hash, cap isolation,
  unchanged numerical files and evaluation recovery. It found and prompted the
  sealed-evaluation repair above. It is internal review, not Claude's FIT.
- [Claude's exact-source review](https://github.com/UOR-Foundation/uor-r4/pull/2454#issuecomment-6164027803)
  found one required atomic-publication repair; it is fixed at `33564b75e` and
  [approved by Claude](https://github.com/UOR-Foundation/uor-r4/pull/2454#issuecomment-6164246420)
  at source `33564b75e`, carried unchanged at `c8bf51909`. Claude reproduced
  25/25 metadata tests; its CUDA-gated Rust type-check used a stub compiler and
  is explicitly not a qualified build. The first-save interruption before a
  latest pointer exists may require a new run-state/plan/personal release; only
  the pre-run baseline is exposed, and Claude records this as non-blocking.
  Its source verdict and the eventual executable-bound admission review are
  separate. The prior `route_apply_reality` CUDA control is PASS in
  [driver-controls.json](../discrete-driver-cost-2026-10-13/receipts/driver-controls.json)
  at source `e7d926fbe`: it applies Route(0), verifies route-only state/epoch and
  freshly carries/forwards at loss 256 after the initial 640. This is a small
  finite fixture, not a corpus-scale Route result or a new D5 build qualification.
- CUDA compilation/runtime of the new controller build: **UNVERIFIED**. This
  machine has no `nvcc`; no GPU was rented or model control run. Unchanged model
  source retains its prior pilot evidence, while new executable/driver identities
  must be built and pinned before release. Do not call local metadata tests a
  CUDA pass.
- The first bare `cargo` resolved to unrelated `cargo-ai` and refused `test`;
  no build or test ran in that invocation. Validation used explicit
  `~/.cargo/bin/cargo` thereafter. The inherited `uor-pod status` also warned
  that its Rust Vast adapter was absent in this fresh worktree: its displayed
  zero inventory is **not a verified provider inventory**. No pod action followed.

Written coordination: [ownership and scope](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163681805),
[D5 migration](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163736585),
[paired findings and sheet corrections](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163817784).
