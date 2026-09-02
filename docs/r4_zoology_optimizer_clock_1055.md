# Exact-byte attention transfer: optimizer-clock correction (#1055)

- **Current status:** `CLOCK_MATCHED_TRANSFER_MISS`; development improved from
  `12.01171875%` to `68.2861328125%`, but missed strict `>99%`. Fresh-process
  verification passed. The original pre-run contract and resource history
  remain below; terminal evidence and the next recommendation are appended.
- **Authority:** [issue #1055](https://github.com/UOR-Foundation/uor-r4/issues/1055), under #973.
- **Project value:** give the already-working copied attention cell a
  source-comparable optimizer clock on our exact data before coherent-R4
  transport/replacement. This is not a new attention mechanism.

The [#1053 transfer](r4_zoology_exact_transfer_1053.md) exhausted its cosine
schedule after 1,024 updates; the [#1050 positive](r4_zoology_release_reproduction_1050.md)
needed 3,920. #1050, HELM-D-R4 ordinary attention, and #1014 load-bearing
attention remain established within their existing claim boundaries.

## Frozen single arm

Bind #1053 result
`blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1`,
its preparation, exact primary/control tensors, and source/dependency closure.
Read no fitted predecessor weights. Reuse unchanged credited model, exact data
adapters, serializer, scorer, and source-cell C0 evidence; do not rerun its
source reproduction or overfit campaign.

| Component | Contract |
| --- | --- |
| Initialization/cell | fresh seed 123; width 64; two layers; one head |
| Data/model shape | unchanged #1053: 8,192 train and 1,024 development rows; V4096/T120; eight K/V pairs |
| Batch/optimizer | 512; AdamW weight decay 0.1; unchanged source dropout |
| Initial learning rate | `0.00046415888336127773` |
| Optimizer clock | at most 20 blocks × 196 updates = 3,920 updates |
| Scheduler | cosine `T_max=64`; advance once after a failed block, not per data pass |
| Development | existing split after each 196-update block; strict top-1 `>0.99` stops |
| Maximum primary work | 16,056,320 training queries; 163,840 development queries |

Cycle complete shuffled permutations of the unchanged 8,192 training rows.
Retain the current permutation and cursor across block boundaries and resume;
never discard its unconsumed rows. Training/development shuffles and dropout
share the global Torch RNG. A passing block permits no further optimizer or
scheduler step. A saved pass finalizes immediately on restart.

This matches optimizer opportunities and evaluation/scheduler cadence, not
source ordering, exposure, or arithmetic. #1050 ends each 100,000-row source
epoch with a 160-row batch; every #1055 batch has 512 rows. Eight rather than
four queries per row also increases query work. Equal updates are not equal
query dose, data coverage, or wall time.

Only after a primary pass, score the frozen physical binding-permuted
development input once without fitting; require a top-1 drop of at least
50 percentage points. That conditional evaluation adds 8,192 queries. No
other model controls, probes, sweeps, or same-issue tuning are authorized.

## Minimal implementation and CPU admission

Add only the clocked trainer, lifecycle/implementation bindings, and focused
schedule/order/cycling/RNG/resume/control-boundary checks. Bind `pyproject.toml`
and `uv.lock`; preserve historically CID-bound #1050/#1053 code. Activate only
those focused checks, Python format/baseline lint, claim wording, and diff
integrity. Broad Rust/workspace/BDD/WASM/release QA stays dormant.

Measure stable one-, four-, and eight-intra-op-thread CPU plans on the fixed
batch shape and select the fastest eligible plan. Require safety-adjusted
full-run projection `≤1,800 s` and measured peak RSS `≤8 GiB`. Reuse passed
#1053 deterministic mechanics; do not add a cross-backend campaign. Use one
training process, no multiprocessing of dependent updates, and no CUDA/MPS.
Emit flushed progress, completed/remaining updates, and measured ETA.
Checkpoint at least every 16 updates and every block boundary, including the
current permutation/cursor and global RNG. A resource stop is incomplete,
not model failure.

## Decision

Metric: exact-data development top-1, currently `12.01171875%`, target strict
`>99%`. All 8,192 targets have admitted causal K/V evidence: the structural
ceiling is 100%. The instrument is focused clock/resume checks plus measured
CPU admission; no broader scientific claim is inferred from passing it.

| Outcome | Terminal and next action |
| --- | --- |
| Primary `>99%` and binding drop `≥50pp` | `CLOCK_MATCHED_TRANSFER_PASSES`; scope separate coherent-R4 transport/replacement |
| Primary miss by 3,920 updates | `CLOCK_MATCHED_TRANSFER_MISS`; preserve curve/model/positives and recommend a distinct next action, not another LR/width/epoch sweep |
| Primary passes, binding drop misses | `NONASSOCIATIVE_SHORTCUT` |
| No eligible CPU plan | `NOT_RUN_PREFLIGHT` |
| Invalid clock/lifecycle mechanics | `INVALID_CLOCK_MECHANICS` |
| Resource/interruption without a complete decision | `INCOMPLETE`, never attention failure |

## Pre-run ledger and scope boundary

Input/dependency/preparation bindings, focused clock tests, CPU admission,
primary/control execution, artifact/state/result CIDs, and fresh-process replay
are `NOT_RUN`. Completed evidence and exact work/read ledgers will be appended.
The binding control is conditional; its prepared input is not a model result.
Role/geometry/future-value model inputs, teacher/provider/fitted-predecessor
weights, and hidden/sealed payload reads must remain zero.

No R4 code, new corpus, generation, reasoning, W8/lowering, product, or release
work runs here. Scope ends with one terminal result, concise current-summary
updates, this append-only record, protected PR delivery, and child/#973 closure
comments. Queue acknowledgements are transport only, not test evidence.

## Resource-only pause after preflight (2026-09-02)

The implementation is frozen at `47ebdabc`. The 15 focused clock/lifecycle
tests passed in `0.982 s`; code formatting, scoped baseline Python lint, claim
wording, and diff integrity also passed. The inherited source-cell C0 was
reused without a new reproduction or overfit campaign.

Preparation CID is
`blake3:935363a19038bb9573cda29c29179f98b4b2a80f4d2e0ac9b64b46ae399f5916`;
implementation/dependency CID is
`blake3:bf45a18a2b4ed1aee607220f8d0331c32254bb43e94c2e1d7bece70d33634de3`.
The `46.456571 s` CPU preflight produced CID
`blake3:a0f88c9328505f56860a3e2cfdb947be83fb5d5877d9acac2e9ab3f4ff92ffd5`.
It measured two eight-batch training units per Apple Accelerate/OpenMP plan:

| Threads | Training units (s) | Development units (s) | Stable | Peak RSS (bytes) | Safety-adjusted full projection (s) |
| ---: | --- | --- | :---: | ---: | ---: |
| 1 | 3.376828 / 3.319727 | 0.467915 / 0.457117 | yes | 1,162,067,968 | 2,080.005000 |
| 4 | 2.378479 / 5.017547 | 0.320396 / 0.518029 | no | 1,197,031,424 | 3,086.198205 |
| 8 | 6.401626 / 10.149635 | 0.703424 / 0.637655 | no | 958,251,008 | 6,234.236962 |

All three plans fit memory but exceeded the frozen `1,800 s` projection
ceiling; four and eight threads also failed timing stability. Their repeated
loss sequences were deterministic within each plan. No plan was admitted
(`selected=null`, `passed=false`): this is `NOT_RUN_PREFLIGHT`, not an
attention or model negative.

Primary optimizer updates, primary development evaluations, and binding-control
evaluations are all **zero**. Disposable timing work is not a primary fit.
No fitted #1055 artifact exists, so artifact replay is unavailable. The issue
remains open and execution pending. The next proposed action is to resolve
the active Studio/browser workload and obtain direction for timing-only
remeasurement; no new attempt or model change is recorded here. The existing
#1050, HELM-D-R4, and #1014 attention evidence remains unchanged.

## Authorized timing retry and first primary start (2026-09-02)

After the user stopped the competing workload and authorized timing-only
remeasurement, a separate `issue-1055-zoology-optimizer-clock-attempt-2`
evidence directory retained the retry. The preparation and implementation
CIDs are unchanged from attempt 1. No source, model, data, budget, or test
contract changed, and no code/test rerun was performed; the original 15
focused checks remain the implementation evidence.

The `23.849352 s` retry admitted a stable eight-thread Apple Accelerate CPU
plan. Its preflight CID is
`blake3:9f184ae09fea32d6a797303a3770ad10f5603ec4ac86bd2010bfc063ce39cdf6`.

| Threads | Eight-batch training units (s) | Stable | Peak RSS (bytes) | Full projection (s) |
| ---: | --- | :---: | ---: | ---: |
| 1 | 3.019445 / 3.027039 | yes | 1,211,580,416 | 1,865.098308 — over budget |
| 4 | 2.199384 / 2.100942 | yes | 1,211,432,960 | 1,353.474036 |
| 8 | 2.117389 / 2.122450 | yes | 1,211,580,416 | 1,306.500579 — selected |

The measured-fastest eligible plan uses eight intra-op threads, one inter-op
thread, one training process, and batch 512; each plan's repeated timing-unit
loss sequence was deterministic. The first and only primary fit has started
under the unchanged 3,920-update cap. The directory's “attempt-2” denotes the
timing retry, not a second model fit. Primary verdict, conditional binding
control, final artifact, and fresh-process artifact replay are still pending;
admission is not an attention result. The refused attempt-1 evidence above is
preserved.

## Completed primary result (2026-09-02)

Correcting the optimizer clock retained a substantial learning improvement:
development rose from #1053's `984/8,192 = 12.01171875%` to
`5,594/8,192 = 68.2861328125%`, a gain of `56.2744140625` percentage points.
NLL fell from `6.79664158821106` to `2.6761019229888916` nats. The frozen
strict `>99%` criterion was not met, so the terminal remains
`CLOCK_MATCHED_TRANSFER_MISS`. The binding-permuted control is
`NOT_RUN_PRIMARY_MISS`, with zero control inference decisions; the retained
gain is not a passed binding-control or geometric-attribution result.

This was one primary fit, completing 20 blocks and 3,920 updates without
retraining or selecting a different checkpoint. It consumed 16,056,320
training queries and 163,840 primary-development queries. Final-block online
training accuracy was `754,401/802,816 = 93.96935287786989%`, with NLL
`0.29081812645403704`; these are during-update metrics, not a post-fit
training reevaluation. The complete development curve is preserved here
(NLL rounded to six decimals; each block scores 8,192 queries):

| Block | Updates | Correct | Top-1 (%) | NLL |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 196 | 477 | 5.822754 | 7.200712 |
| 2 | 392 | 898 | 10.961914 | 6.831665 |
| 3 | 588 | 985 | 12.023926 | 6.809559 |
| 4 | 784 | 975 | 11.901855 | 6.898668 |
| 5 | 980 | 998 | 12.182617 | 6.952888 |
| 6 | 1,176 | 974 | 11.889648 | 7.017067 |
| 7 | 1,372 | 972 | 11.865234 | 7.120267 |
| 8 | 1,568 | 976 | 11.914063 | 7.262002 |
| 9 | 1,764 | 964 | 11.767578 | 7.364768 |
| 10 | 1,960 | 972 | 11.865234 | 7.513181 |
| 11 | 2,156 | 978 | 11.938477 | 7.641114 |
| 12 | 2,352 | 951 | 11.608887 | 7.806085 |
| 13 | 2,548 | 964 | 11.767578 | 7.909654 |
| 14 | 2,744 | 1,344 | 16.406250 | 7.409531 |
| 15 | 2,940 | 2,787 | 34.020996 | 5.794369 |
| 16 | 3,136 | 2,930 | 35.766602 | 5.840742 |
| 17 | 3,332 | 2,953 | 36.047363 | 5.898982 |
| 18 | 3,528 | 4,516 | 55.126953 | 3.897963 |
| 19 | 3,724 | 5,229 | 63.830566 | 3.064048 |
| 20 | 3,920 | 5,594 | 68.286133 | 2.676102 |

The primary took `1,078.9267852089833 s`; total run wall was
`1,079.2978823750746 s` (about 18 minutes). Peak primary RSS was
`1,151,451,136` bytes. Execution used the admitted eight-intra-op-thread
Apple Accelerate CPU plan, one inter-op thread and one training process;
CUDA/MPS were not used. The initial refused timing attempt did not train a
primary model. Its authorized timing retry did not change the source,
preparation, model, or training contract.

### Identities, read ledger, and verification

| Evidence | Identity |
| --- | --- |
| Credited Zoology source | `de4e258784224e09909c257ff3ea040f089ed660` |
| Implementation commit | `47ebdabc` |
| Bound #1053 result | `blake3:e2d1deb55a4612015ba924a94051beacd517f3c062c714c4972ba954f57621a1` |
| Primary dataset file | `blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1` |
| Primary tensor bytes | `blake3:baef4fd29bedddc5c9cd826b78101c5c412db0e883241e04e478e4e3baf1d8b1` |
| Bound control file; inference not run | `blake3:34dad69bfbeda87ea7e4d5d7af2fd8434dc5ab33cc055df4a0965f0b7b96b693` |
| Implementation/dependencies | `blake3:bf45a18a2b4ed1aee607220f8d0331c32254bb43e94c2e1d7bece70d33634de3` |
| Implementation tree | `blake3:c32c8f87d14cc0522ff17655ce919bc95dd3a85bdf11f2fec9376f28bfd42f0b` |
| Preparation | `blake3:935363a19038bb9573cda29c29179f98b4b2a80f4d2e0ac9b64b46ae399f5916` |
| Admitted preflight | `blake3:9f184ae09fea32d6a797303a3770ad10f5603ec4ac86bd2010bfc063ce39cdf6` |
| Primary run | `blake3:3cabea73e6388daf7512a4a061f0ef1c239a5b478e33a39178d00001c5ba4d81` |
| Model artifact, 1,217,024 bytes | `blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4` |
| Model state | `blake3:4743114d1fa6a68d8ef856d4e6306415a75187879acc6b479ea6f75cf68c7d80` |
| Final development logits | `blake3:feef01968e9e6a05a5d1cb267119f4a2f2fbf6ded4fafdc4dea9086368e8f4a9` |
| Result | `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee` |

Preparation read three predecessor JSON envelopes and the existing primary
tensor payload once. New/copied corpus payloads, control tensor payloads,
role payloads, fitted predecessor weights, English/natural payloads, sealed
inputs, teacher/provider calls, and future-value/geometry/role model reads
were zero. The unchanged source-cell C0 evidence was reused, with zero new
C0 training updates. The original 15 focused checks and formatting/scoped
lint/claim/diff checks remain valid; they were not rerun for the timing retry.

Fresh-process verification passed: it validated source/preparation/preflight,
artifact and work bindings, reproduced final logits/top-1/NLL and the result
CID, and confirmed no control execution. Verification added 8,192 development
inference decisions and zero optimizer updates, separate from the 163,840
primary-development queries. Independent read-only checkpoint audit also
passed: every AdamW counter is 3,920, the 20-block histories match, scheduler
state is block 20 with `T_max=64`, and all 20 exported tensors match the
checkpoint exactly. The tied embedding/head, evaluation RNG, and valid
245-cycle sampler ending at cursor 8,192 also matched. This audit performed
no inference, tests, or retraining.
Broad workspace/release QA remains `NOT_RUN`; queue acknowledgements are
transport only.

### Interpretation and approved project-directed next action

The correction produced strong transfer learning without meeting this
population's frozen qualification target. It does not falsify ordinary
attention, identify a unique remaining defect, or establish geometric
advantage. The run reused the same 8,192 unique training rows; the increased
quantity was repeated exposure and optimizer updates, not new data. The final
four development scores rose from about 36% to 55%, 64%, and 68%. The preset
cap stopped the run; convergence was not established, so 68% is not a measured
capacity ceiling. Benefit from adding unique data remains unmeasured.
#1050's `99.1666667%` source positive and the existing HELM-D-R4/#1014 attention
results remain intact.

The user approved one **separately contracted continuation of this exact
saved checkpoint** to check whether the observed learning continues: at most
20 additional 196-update blocks, or 3,920 additional updates, retaining the
strict `>99%` stop. Preserve the data, model, learning-rate policy, optimizer,
sampler, and RNG state; do not repeat training from scratch or add a parameter
grid. Freeze the new contract before execution. This does not silently extend
#1055, change its frozen `CLOCK_MATCHED_TRANSFER_MISS`, or imply its unrun
binding control passed. No continuation has launched within #1055.

The project-directed integration goal remains coherent R4 execution. The
already-qualified #1050 artifact/population offers an available inference-only
integration option: substitute inner Q/K/V transport/aggregation while keeping
learned weights, positions, norms/residuals and softmax. It remains a fallback
or next integration option, not a required first step before the approved
continuation. Neither option establishes geometric superiority, softmax
removal, generation, reasoning, or exact lowering without its own evidence.

### Successor continuation retained (#1057, 2026-09-02)

The separately authorized [#1057 continuation](r4_zoology_checkpoint_continuation_1057.md)
resumed this exact checkpoint and reached `8,071/8,192 = 98.52294921875%`
final development recall, with NLL `0.09141154401004314`; its best measured
recall was `98.6572265625%` at block 36. The final saved artifact is block 40,
not that earlier best-scoring state. This is a retained `30.23681640625`-point
gain over #1055 after 3,920 additional updates on the same unique training
rows, not an increased-unique-data experiment.

The user explicitly directed that this near-target result must not be
discarded or labeled false/failed attention. Its frozen `CONTINUATION_MISS`
records only that strict `>99%` was narrowly unmet; the conditional binding
control remains unrun. It does not falsify attention, establish a capacity
ceiling, or stop geometric integration. #1055's original result is unchanged;
both its checkpoint and the #1057 final artifact are preserved, as is #1050's
separate `99.1667%` reference. The next recommendation is inference-only
coherent-R4 integration using that already-qualified reference, retaining
#1057 as a valuable exact-data artifact for subsequent inference work, without
an automatic third training window. See #1057 for the complete curve,
artifact identities, work accounting, and successful fresh-process replay.
