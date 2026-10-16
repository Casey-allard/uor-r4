# Admission redirect D-Packet — data/configuration binding, qualification pending

**DRAFT / NOT EXECUTION-READY. No model result is claimed.** This packet implements
the data/configuration part of step 3 of the [FIT redirect sheet](../../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md).
Codex/Sol owns this packet; Kimi PM and the PM-dispatched reviewer own acceptance.
The D-R implementation landed in #2518 as `2fa663b54ca9dad90c5f5f9e07baaafb0b99a8f3`.

The [owner release](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250)
authorizes qualification first, then the matched training contrast only after
qualification and packet gates are green. **This work does not launch training.**
M2 and Stage B are not released. The earlier sheet's blanket hold text is historical
relative to this scoped release; its technical gates still apply unchanged.

## Qualification blocker and division of work

**16 October execution update:** the reviewed executors were merged, but the
[qualification attempt at `3c651df97`](../k16-qualification-2026-10-16/README.md)
halted at the fixture runner's CUDA build (E0609 identity-field accesses).
Discrete and timing builds passed; clippy, G-Pre and all six controls were
NOT_RUN. The K16 timing slot remains **UNMEASURED**, Q4 **NOT_EXECUTABLE**.
Receipts were cloud-verified before pod deletion; no training gate is satisfied
by this build-only attempt. The dependency account below retains the packet's
original pre-delivery diagnosis; its GPU-executor-pending rows now mean a
reviewed repair and successful qualification are pending, not absent source.

The [readiness report](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201479931)
found that `list-admission-rank-controls` prints metadata, not GPU test results.
`objective.rs` explicitly marks the C-R3 executor and C-R6 corpus qualification
as owed. The existing timing runner is K8 / three repeats, not K16 / 100 batches.
This is a missing executable dependency, **not a failed model gate or a negative**.

The [PM disposition](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519)
assigns AGY the real C-R1..C-R6/G-Pre executor and runbook, DeepSeek the synchronized
timing runner, and Codex this packet. No pod spend until those executors are reviewed
and merged. The timing [interface agreement](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201508191)
requires immutable before/after snapshots and zero corpus apply calls.

| Required receipt | Current disposition |
| --- | --- |
| Existing 25 finite controls, newly compiled PTX admission | NOT_RUN on the future qualification executable |
| C-R1 rank/admission equivalence | NOT_RUN — GPU executor pending |
| C-R2 occupied-cell multiplicity | NOT_RUN — GPU executor pending |
| C-R3 virtual/applied authored-candidate parity | NOT_RUN — GPU executor pending |
| C-R4 absent rivals / occupancy below K | NOT_RUN — GPU executor pending |
| C-R5 bounded integer arithmetic / dual loss / distributions | NOT_RUN — GPU executor pending |
| C-R6 stored guard and untrained ceiling | NOT_RUN — GPU executor pending |
| G-Pre seeds 1/2, K8/K16; K16 admission >= 4,894 | NOT_RUN; no measured ceiling or strict-win baseline `u` |
| 100-batch K16 timing, both policies, checkpoint 0 and Q4 | UNMEASURED |
| Qualified executable / PTX / geometry / execution-lineage pins | UNAVAILABLE until qualified build; never reuse historical K8 executable pins |

Qualification executes finite authored fixtures only where the reviewed fixture
contract calls for them. Corpus evaluation/timing performs **zero `.apply` calls**.
Any fixture, status or receipt failure returns to #2029 with preserved evidence;
no silent patch-and-rerun and no scientific disposition. Q4 is an evaluation-only
fixture rebind with its original SHA retained, never a K8 training resume.

## Bound fresh-transfer source

`data-binding.json` is the reproducible Rust data-preparation receipt. It binds
exactly **2,000 unique eligible masked targets**, source-relative and output-relative
row positions, complete BOS documents, token/mask hashes and four configuration
file hashes. Model operations and corpus apply calls are both zero. The private
archive and its verified round-trip receipt are listed in `archive-receipt.json`.
The bound payload has **505,085 tokens in 200 documents**, with exactly ten masked
targets per document: tokens SHA-256
`82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b`,
mask SHA-256 `8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4`.
Two independent preparations are byte-identical. The private archive is
3,276,288 bytes, MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, verified by streamed
download. Preparation took 4.64 seconds (repeat 4.26 seconds); archive transfer
and round-trip took 28.11 seconds. These are data/metadata timings, not K16 rates.

- Source: private HF dataset `caseyallard/uor-r4-store`, revision
  `7e0a52a70165d01ee5f37d1b28740f53c49ca3f2`, `data/ft-balp.tar`, SHA-256
  `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`.
- Member: `balp/tokens.u16` and its response mask, **only the `chat-v0-p2`
  component beginning at payload offset 17,073,798**. The pinned concatenation
  manifest and the existing Rust `mix-chat-corpus` implementation establish that
  boundary; the two `mw-bal-*` copies and oracle-augmented `mw-orcp` prefix are
  not candidate transfer data.
- Selection is fixed before any model scoring: scan complete documents in
  ascending source order; reject documents identical to train/dev; sample at most
  ten evenly spaced eligible response positions per document (`floor(i*n/min(10,n))`
  in eligible source order); retain the first 2,000 unique C192 context-plus-target rows absent from every
  eligible train and dev row. Retain full document context; mask every other
  target off. No selection by model output, confidence, score or outcome.
- Exact overlap keys bind context length, BOS-local preceding token sequence
  (up to 192 tokens), and target token. They intentionally ignore split names
  and source offsets, so relabeling duplicates cannot pass. This is exact-token
  disjointness, **not a semantic/paraphrase contamination guarantee**.
- The same tokenizer SHA `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
  and explicit `<|bos|>` ID 0 apply. `PreparedCorpus` validates the resulting
  `Split::Fresh` payload/mask and all ordered rows, including reconstructed C192
  causal keys. No CPU model scoring or geometry is performed.

**Freshness scope:** this chat-source slice is held out from the current discrete
`mw-balp/train` / `ft/dev` contrast. The source was historically used by pre-D32
models; it is not globally unseen project data. It is a different chat-source
distribution, not a fresh draw of the M-world generator. The 2,000 targets are
token rows across exactly 200 documents, not 2,000 independent conversations.
This per-document cap avoids satisfying the target count with two long responses.
PM/reviewer acceptance of this
scope is required; its measured result cannot broaden the dev acceptance bars.

The frozen 14,287-row dev panel remains the KEEP/iterate decision panel. Fresh
transfer is scored once, with zero updates, for each qualifying seed/arm before
any capability promotion. Never put it in training, checkpoint selection, the
timing benchmark or the G-Pre panel. The current `DriverConfig.dev` requires
`Split::Dev`; **do not relabel fresh data as dev** to bypass it. A reviewed
evaluation-only path must retain the fresh binding, model SHA and row mapping.

## Exact four-arm configuration and initialization

`configs/k16-{A,R}-seed{1,2}.json` are actual `DriverConfig` files, validated by
the native metadata validator. They retain the frozen train/dev/tokenizer pins
from the completion record and differ only in seed and objective policy.
Their paths resolve from the pod repository root after staging data at
`local/driver/data/`; staging must reproduce hashes, not rewrite configurations.

| Field | Binding |
| --- | --- |
| Shape / batch | V4096, B4 (32 integer components), C192, K16, 64 target rows |
| Reader / schedule | Route / `canonical_vector_then_read_v1` |
| Control / candidate | `nearest_address_margin_v1` (8T+32A) / `occupied_cell_rank_hinge_v1` (8T+32R) |
| Seeds | 1 and 2; A/R share identical initial table/routes/gates per seed |
| Initialization | Fresh `Device::new_with_objective`; epoch 0; no checkpoint resume |
| Vector parameters | 4096 x 4 = 16,384 independently scheduled 8-component root blocks |
| Route parameters | 4 x 5 = 20 route operators; total scheduled coordinates 16,404 |
| Stored parameter arrays | 131,072 i32 table components + 20 u32 routes + 20 inactive i32 gate slots; 524,448 array bytes, excluding metadata/geometry/scratch |
| Dose per arm/seed | 16,404 batches x 64 = 1,049,856 masked target rows; one coordinate pass, not a full 2,425,925-target corpus epoch |
| Total planned dose | 65,616 batches / 4,199,424 target presentations across four runs |
| Dev checkpoints | 0, 4,101, 8,202, 12,303, 16,404; all 14,287 rows each time |
| Transfer | Bound `Split::Fresh`, 2,000 targets; no train/dev/timing exposure |

Initializer source: `discrete_init` seeds each token-block with
`seed XOR block_index XOR 0x9e3779b9`, applies xorshift 13/17/5, and selects
`state & 15` from the exact root table. Routes initialize to the identity action;
gate storage initializes to zero. The root choice is a discrete parameter, not
eight independently learned real weights. The same shared writer, comparator,
geometry, data order and dose are required across A/R. Initial snapshot and array
hash equality per seed must be recorded on GPU before training.

The D-R CUDA SHA is
`f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008`;
`objective.rs` SHA is
`b813e855be6c4ea460c8711493527d52f1caf1a02bb09b046145f88c7f4ee4d0`.
The future qualified binary must pin its own full source, PTX, geometry, compiler,
config and lineage identities. These source hashes alone are not runtime admission.
Historical pilot/completion release controllers remain K8-scoped; this packet does
not widen them or fabricate `owner.json`, review receipts or capability tokens.

The [separate K16 validator source deliverable](../k16-admission-validators-2026-10-15/README.md)
adds explicit fresh-start arm/seed scopes without extending historical K8 release
authority. Its qualification receipts, exact-head review and owner-transport
resolution remain prerequisites, not completed execution. **Codex/Sol owns the
fresh-scoring eval-only path**, separately reviewed before KEEP promotion;
AGY owns qualification fixtures/G-Pre and DeepSeek owns timing. Fresh scoring
is not a prerequisite for starting training, but is mandatory before a KEEP
is promoted to a capability claim.

The [fresh-scoring source deliverable](../fresh-scoring-eval-2026-10-15/README.md)
now supplies a standalone CUDA consumer for the pinned 2,000-row population:
authenticated completion/admission/config/lineage/cursor, ordered row binding,
production T/A/R/rank/occupancy metrics, and zero-apply snapshot/epoch invariance.
Host metadata/data inspection passes; CUDA compilation and model scoring remain
NOT_RUN pending independent exact-head review and authorized execution. Production
training sources are unchanged. This remains a before-KEEP dependency, not a
training-start gate.

## Cost model and actual spending

`cost-model.json` separates planned quantities from actual receipts. All K16 rates
remain **UNMEASURED**, never populated with the old K8 3.393 s/batch estimate.
For each policy and occupancy fixture, report 100 synchronized elapsed-ns samples,
target rows/s, physical padded context tokens/s, actual context lengths, candidate
work, warmup, transfers/allocation, and the zero-apply exclusion. The denominator
for rows/s is not the denominator for context tokens/s. D32 model arithmetic is
integer; host wall-clock and monetary accounting are not model arithmetic.
The [interface follow-up](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201694881)
aligns sampling with AGY's runbook: 80 VectorBlock and 20 Route batches, reporting
per-family rates separately. Both families evaluate 16 ordinals including CURRENT.
This is not 100 consecutive canonical coordinates and its 80:20 mix is not the
training dose's 16,384:20 mix; only the separately weighted rates price that dose.

Full-run budgeting must include vector and route proposals, accepted-update
costs (not measured by zero-apply qualification), dev/transfer evaluation,
initialization, checkpoint/archive I/O, retries and provisioning. A rate measured
at checkpoint 0 alone cannot price Q4 occupancy. A proposed compute-only estimate
is not an all-in budget or release. Retain the sheet's historical estimate only
as historical, not as a current quote.

The local Vast adapter was absent; building the existing `uor-r4-pod-provider`
fixed the environment. A subsequent admitted status check listed **0 pods/$0 per
hour**, no free GPUs, and live 5090 offers starting at $0.96/GPU/h. This was an
unbooked point-in-time quote. No lease was acquired, so this packet has **0 GPU
seconds, $0 GPU rental, no pod storage or teardown**. The owner qualification
stage envelope is approximately 30 minutes and <$2, superseding the sheet's
historical 15-minute/$0.35 estimate; global limits remain <=4 pods and <=$8/hour. Preparation/archive
wall time, local and archived bytes are disclosed with their receipts, not
charged as model execution. Cloud subscription charges are not measured here.

## Reproduction and validation

Download only the pinned HF `data/ft-balp.tar`, `data/ft-dev.tar` and
`data/tokenizer.json.tar` at the revision above, verify archive SHA-256s in
`archive-receipt.json`, and extract into a new owned input directory. No teacher,
model or GPU is involved. Run:

```sh
cargo test --locked --offline --manifest-path docs/labs/admission-redirect-packet-2026-10-15/instrument/Cargo.toml
cargo run --release --locked --offline --manifest-path docs/labs/admission-redirect-packet-2026-10-15/instrument/Cargo.toml -- EXTRACTED_INPUT_ROOT NEW_OUTPUT_DIR
```

Set `CARGO_TARGET_DIR` to an owned local target and `CARGO_BUILD_JOBS=4`.
The output directory must not already exist. Binary/source receipt fields can
differ on a rebuild; payload, mask, row map, document map and config pins must
reproduce exactly. Four focused metadata tests cover malformed headers/masks,
cross-offset duplicate detection, the C192 boundary and capped/spread response
selection. These are not C-R tests.

## Required archive / receipt layout

Use immutable per-attempt names. All generated pod outputs live under `/workspace`.
The final runbook must seal a SHA256SUMS inventory and retain:

```text
qualification-ATTEMPT/
  authority/       owner-release URL/id, FIT sheet, exact-head review references
  source/          commit, changed-path hashes, build log, binary, PTX, geometry
  inputs/          configs, tokenizer/corpus pins, Q4 original/rebind lineage
  controls/        existing-25 and C-R1..C-R6 per-case expected/actual/status
  g-pre/           seed{1,2}-k{8,16} checkpoint0, ordered rows, GPU aggregates, u
  timing/          policy x checkpoint0/Q4, warmup, 100 raw samples, summaries
  integrity/       before/after snapshots, epoch/apply counters, GPU/PTX admission
  resources/       lease, offers, GPU/runtime, wall time, charges, storage bytes
  disposition.json
  SHA256SUMS
training-ARM-SEED-ATTEMPT/  # future, not created by this packet
  authority/ inputs/ checkpoint0/ checkpoints/ transitions/ dev/ fresh/ resources/
  SHA256SUMS
```

Each training checkpoint binds source/executable/config/data/model/lineage,
cursor and split-family counters. Dev/fresh receipts retain ordered target
position/token correspondence, raw T/A/R, sums 8T/32A/32R, both losses,
histograms/quantiles, occupied cells and admitted-token counts. G-Occ is evaluated
only at the registered final cursor against the matched seed's control.

Before teardown, `cloud-store put codex DIR` must succeed and stream back the
same MD5; independently retain SHA256SUMS plus archive URI/size/digest/receipt.
Retain failed and partial attempts too. Verify the archive **before**
`uor-pod release` and `uor-pod down`; verify no owned pod remains and report final
lease/wall/storage cost on #2029. Never destroy an unarchived unique result.

Line: exact addressed memory / admission redirect; count unchanged;
M1 headline 37/232 unchanged. **Next:** exact-head packet review, reviewed executor
merges, qualification, then fill measured timing/runtime pins and final packet
acceptance. This draft is not gate 2 green and does not authorize a training batch.
