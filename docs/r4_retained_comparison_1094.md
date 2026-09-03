# Raw-text entry preservation and fresh-process replay — #1094

**2026-09-03 — `CLAUSE_ADAPTER_PRESERVED`.** The sole frozen `run-retained`
invocation completed both comparison and fresh-process replay. The deterministic
text-to-clause adapter recovered every independently annotated valid input and
preserved the unchanged learned R4 model's complete compared tensors and decisions.
The [independent result review](r4_retained_comparison_1094_review.md) accepted
`CLAUSE_ADAPTER_PRESERVED`; the original receipts remain the evidence authority.

This qualifies removal of externally supplied clause segmentation for the
specified known-vocabulary, fixed-query, four-fact grammar. The reader still
consumes `inputs[B,5,13]` and `lengths[B,5]`; the adapter derives those arrays
from the one raw text buffer. This does not establish unrestricted parsing,
new semantic worlds, variable context, generation, coding or final native serving.

## Frozen source and one-shot execution

The base was protected PR #1100 at
`1ff8b81cb060ca2e8ced409ec78ae566b5b98891`. The [activation receipt](r4_retained_comparison_1094_activation.json)
and [native activation comment](https://github.com/UOR-Foundation/uor-r4/issues/1094#issuecomment-5526352267)
bound one invocation and its divergent terminal actions before access.
The accepted coordinator source remains
`07ec3f0d39d08ac5bf9c2ba7a6b864229e007867` in
`/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4`; the accepted
worker remains at its separately bound source root. The fresh
`issue-1094-retained-comparison` worktree holds delivery records only.
No adapter, worker, curator, reader/core, geometry, policy or comparison source
was edited. No new preparation, readiness attempt, fit, timing calibration,
population regeneration or automatic retry ran.

The immutable retained assembly SHA256 is
`48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`.
The independent release SHA256 is
`5787e4a64113800c5fc82cd1d32d564d9c6e3a344e74ca102a754fe82dccee23`.
The consumer used those original files in
`/Users/casey.allard/.codex/uor/issue-1094-retained-assembly01`.
The [operator receipt](r4_retained_comparison_1094_operator.json) and
[driver snapshot](r4_retained_comparison_1094_operator_driver.txt) record the exact
command/environment, one invocation, exit 0, empty stderr and no emergency abort.
Only the original withheld directory changed `000 -> 0500 -> 000`; its identity
was preserved. No recursive permission change or pre-run payload inspection occurred.

## Measured comparison

The population contains four authoring and sixteen withheld semantic groups
from already-observed worlds, each with five queries, four fact forms and four
surface profiles. The 1,600 rows are related renderings of twenty groups, not
1,600 independent semantic trials. Each partition has sixteen form/profile cells.
Reference integrity and every cell's input and soft-output criterion passed.

| Criterion, per comparison phase | Authoring | Withheld | Total |
|---|---:|---:|---:|
| Valid input/token/length/span fidelity | 320/320 | 1,280/1,280 | 1,600/1,600 |
| Exact adapter/reference answers | 320/320 | 1,280/1,280 | 1,600/1,600 |
| Correct answers against the frozen labels, each arm | 320/320 | 1,280/1,280 | 1,600/1,600 |
| Correct consumed role pointers, each arm | 4,480/4,480 | 17,920/17,920 | 22,400/22,400 |
| Exact typed refusal cases | 16/16 | 64/64 | 80/80 |
| Exact period-removal boundary refusals | — | 16/16 | 16/16 |

All thirteen valid batches matched byte for byte for the eight declared tensor
kinds: inputs, lengths, role attention, soft role vectors, binding attention,
full logits, role positions and predictions. The persisted records retain
208 tensor receipts per phase across the two arms, including dtype, shape,
valid-row indices and digests. They also retain row decisions, per-cell/group
metrics and failures; no reference errors were dropped. Failure count is zero.
The 96 refusal/boundary rows caused zero model forwards in each phase.

Fresh-process replay reproduced the complete deterministic comparison object,
including every decision, tensor receipt, metric, audit and model state.
Execution and replay share deterministic SHA256
`b28336e8c0413b277c5655d2841b7aef4a0e254618aa1910c50146e7dfcea1d4`
and oracle replay SHA256
`a3420cd5b865c0f55d08bce02d3292bb7349c0679e3acad2616da9cb31e49470`.
There were four sequential worker processes: oracle and adapter for execution,
then fresh oracle and adapter for replay. Each loaded the reader and core once,
performed thirteen model batches and 1,600 row forwards, and retained equal
before/after model-state identities. Total work was **52 model batches, 6,400
logical row forwards and zero optimizer updates**. Refusal transport batches
are separate from model-forward batches.

Ten parent identity events cover pre-access/pre-replay plus before/after each
worker, with eighteen runtime files, two interpreter aliases, five assets,
hardware and both source closures verified. Every actual worker retained its
ordinary single denied-probe receipt and fixed runtime. This is not a repeated
four-probe #1096 readiness measurement or proof of universal isolation.
The runtime was Python 3.12.14 / Torch 2.7.1, CPU Accelerate, four intra-op and
one inter-op thread, one worker at a time, on the bound Apple M1 / 16-GiB host.

## Terminal and resource accounting

The [receipt index](r4_retained_comparison_1094_evidence/receipt-index.json) binds
all seventeen original files to byte-identical public copies. The five assembly
files keep their earlier public copies; twelve new consumer receipts are added.
[`completion.json`](r4_retained_comparison_1094_evidence/completion.json) binds the
result and final resources, and no `run-stopped.json` exists. The result CID is
`blake3:cba0ca583ff0b1fd2bc3533b2e82e1b574095e71cad7bd95469510acf4d31e3f`,
SHA256 `c50b354f8da5ae170b97eabbc3b887bf065efb8e1861353cb485d8515c558171`.
The [derived summary](r4_retained_comparison_1094_summary.json) is a readable
extraction from these receipts, not another evaluation.

| Resource | Recorded value | Frozen ceiling / interpretation |
|---|---:|---|
| Execution at phase-close snapshot | 8.181647042 s | 120 s; fresh admission/identity checks included |
| Replay at final-resource snapshot | 7.515791125 s | 120 s; fresh checks and result writes included |
| Final cumulative snapshot | 135.697821334 s | 360 s; includes the historical 120-s policy debit |
| Operator wall time through cleanup/reseal | 15.821630625 s | Separate end-to-end observation, not 135.7 s of new compute |
| Combined peak-RSS bound | 471,531,520 bytes (about 0.439 GiB) | 3 GiB |
| Final original output | 4,033,394 bytes | Seventeen retained files |
| Historical plus final output | 7,498,795 bytes | 128-MiB corpus/results cap, history counted once |
| Reconstructed retained-plus-spool upper bound | 54,780,510 bytes | Conservative bound from final retained sizes plus one largest spool; not sampled peak disk use |

The original preparation's full 120-second allocation remains a conservative
policy debit, not measured elapsed time. The final-resource snapshot reports
7,497,731 bytes before its own 344 bytes and the 720-byte completion receipt;
the final inventory reconciles that 1,064-byte tail. The consumer checks after
completion, but no new timestamp purports to measure its own final exit tail;
the independent operator wall interval covers process completion and resealing.
No ceiling was renewed or criterion relaxed.

## Preservation, storage and next action

The frozen consumer removed its two **47,281,715-byte temporary oracle streams**
only after each complete deterministic phase record was persisted, as the
retained progress events show. The raw temporary tensor bytes are no longer present; the persisted byte-comparison
results, content digests and replay inputs/model identities remain.
There was no other storage cleanup: original results, source, model assets,
corpus, old unavailable/readiness evidence, cited worktrees/indexes and unrelated
user material remain preserved. New public comparison copies and their index
occupy 3,854,366 bytes, separately identified from the campaign ledger.
The new delivery worktree uses about 1.11 GiB; pre-creation free space was about
45.08 GiB. The unchanged code graph is reused; no reindex or model download ran.

The original #1094 `UNAVAILABLE_REFERENCE_REPLAY` remains the historical
preparation outcome. #1096 remains runtime-only readiness. #1079's
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK` and #1082's descriptive limits are unchanged.
The [NEMESIS/W33/UOR source review](r4_retained_comparison_1094_sources.md)
records design/provenance support; it supplies no proof of this model result.
No causal conclusion about the weak token control follows from removing supplied
segmentation, and no semantic novelty or wider language/context claim is added.

Independent result review accepted this positive terminal as satisfying #1094's
bounded adapter-comparison DoD; protected delivery completes its closure. #973 remains open and #954
blocked. The one next action is **#1086's separately frozen native export/loader
contract**: exact artifacts, lexical/frame identities, input/state/output,
operator/dtype/reduction semantics and the smallest matched reference/native
behavior check with its resource envelope. That contract must precede native
implementation; this task does not start it.
