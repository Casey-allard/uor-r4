# Fixed-checkpoint attention continuation (#1057)

- **Current status:** completed with retained near-target learning progress:
  `98.52294921875%` final development recall, up `30.23681640625` points.
  The strict `>99%` target was not met; this is not failed attention or a
  discarded artifact. Fresh-process replay passed. The frozen contract,
  launch history, and terminal evidence remain below.
- **Authority:** [issue #1057](https://github.com/UOR-Foundation/uor-r4/issues/1057), under #973.
- **Project value:** test remaining learning in the existing attention cell
  before coherent-R4 integration, without restarting a fit or changing its
  mechanism. The user approved this one bounded continuation.

The [#1055 result](r4_zoology_optimizer_clock_1055.md) improved development
from 12.0117% to 68.2861%, with a rising late curve, but missed its strict
target. It remains an immutable `CLOCK_MATCHED_TRANSFER_MISS`; this continuation
does not retroactively pass that result or its unrun control. #1050,
HELM-D-R4, and #1014 attention positives remain within their existing scope.
#1055 merged through [PR #1056](https://github.com/UOR-Foundation/uor-r4/pull/1056)
at `f2f2396bedeafaa8377eb67232f02e82ca8b45d9` before this successor.

## Frozen source state and continuation

| Inherited evidence | CID |
| --- | --- |
| #1055 result | `blake3:3cb810f09a118cfb70752643f5d9e60d0e42780dc6e47dc4f99224cbd69af0ee` |
| #1055 model artifact | `blake3:2a225b691ffde7b40afd41ac888c5a4449a6b3ba48c2773a2108e1e407d6f8b4` |
| Exact 3,920-update checkpoint | `blake3:e41064d74f31d45b51ecf07590ef1ae12e3f1efddce535e92b8b1b1eabd15386` |

Restore the exact model, AdamW moments/counters, cosine scheduler at block 20
with `T_max=64`, current sampler permutation/cursor, and global Torch RNG.
Preserve the first 20 history entries unchanged. Use a new evidence root and
a small sibling runner bound to the parent checkpoint, parent binding, and
new contract; never rewrite the predecessor or reset from initialization.

Keep the same 8,192 unique training rows and 1,024 development rows, exact
primary/control data, width 64, two layers, one head, V4096/T120/eight K/V,
batch 512, dropout, and optimizer policy. This is more repeated exposure,
not an experiment that increases unique data. Continue the existing cosine
schedule and evaluate after each 196-update block. Strict development top-1
`>99%` stops immediately, with no further optimizer or scheduler step.

| Work | Inherited | Maximum additional | Maximum total |
| --- | ---: | ---: | ---: |
| Source-clock blocks | 20 | 20 | 40 |
| Optimizer updates | 3,920 | 3,920 | 7,840 |
| Training queries | 16,056,320 | 16,056,320 | 32,112,640 |
| Primary-development queries | 163,840 | 163,840 | 327,680 |

Only after a primary pass, score the frozen physical binding-permuted
development input once, without fitting, and require at least a 50-percentage-
point drop. This adds at most 8,192 control queries; report artifact replay
work separately. No new data, LR/width change, parameter grid, new control,
geometry change, or automatic further extension is authorized.

## Minimal implementation and resource boundary

Reuse the unchanged cell, loaders, scorer, sampler, step, checkpoint writer,
and serializer. Do not alter historically CID-bound code or use module-global
monkeypatching. Activate only new continuation-equality, inherited/additional
accounting, restart-budget, strict-pass/control-exclusion, and predecessor-
tamper checks, plus scoped Python format/lint, claim wording, and diff
integrity. Existing #1055 checks remain valid; do not rerun old QA, C0, source
reproduction, or a CPU timing matrix.

Reuse eight intra-op Apple Accelerate CPU threads, one inter-op thread, and
one training process. The measured same-shape #1055 primary took
`1,078.926785 s`; multiplying by the declared 1.25 safety factor projects
`1,348.658482 s`, within the `1,800 s` additional-execution ceiling. Peak RSS
must remain at most `8 GiB`. CUDA and MPS are forbidden.

Keep inherited, additional, and total time/work separate. Checkpoint at least
every 16 updates and each block boundary, persisting sampler/RNG state and
accumulated additional elapsed time. Recovery must neither renew the budget
nor regenerate an unconsumed permutation. Emit flushed progress and measured
ETA. A resource/interruption stop before a complete decision is incomplete,
not a model failure.

## Decision and pre-run ledger

The metric is development top-1, currently `5,594/8,192 = 68.2861328125%`
with NLL `2.6761019229888916`, against strict `>99%`. All targets have causal
binding evidence, giving a structural ceiling of 100%, not a model guarantee.

- Primary pass plus binding drop `≥50pp`: qualify the continued artifact for
  a separately scoped inference-only coherent-R4 integration.
- Miss by total update 7,840: retain the complete curve/artifact, do not infer
  a capacity ceiling or extend/sweep again, and recommend a distinct project
  action; qualified #1050 R4 integration remains available.
- Primary pass but insufficient binding drop: a nonassociative-shortcut
  result, with no geometric promotion.
- Invalid inherited state: stop before fitting as invalid continuation
  mechanics. A resource/interruption stop remains incomplete.

Continuation binding, new focused checks, execution, conditional control,
artifact/state/result CIDs, exact work/read ledger, and fresh-process replay
are `NOT_RUN`. Completed evidence will be appended. No English generation,
reasoning, exact lowering, product promotion, or release runs in this issue.
Completion requires current-summary updates, this append-only record,
protected PR delivery, and child/#973 outcome comments. Queue acknowledgements
are transport only, not test evidence.

## Continuation launch (2026-09-02)

Implementation commit `94b75a66` froze before launch. Seven new focused
continuation checks passed in `2.167 s`, including exact uninterrupted-versus-
resumed trajectory equality. Existing source-cell, C0, and #1055 evidence was
reused; no old QA or CPU timing matrix was rerun.

- Preparation CID:
  `blake3:4a01fada99551fa7360520fdda18890bdd3d4f3b8c5ed03cfb41aee20150eeaa`.
- Implementation CID:
  `blake3:39530199c1f0cfec2d2d85be7684c6c66688b74b22cbd86b91b6a672eb0010fd`.
- Evidence root: `.uor-models/research/issue-1057-zoology-checkpoint-continuation`.
- [Frozen launch comment](https://github.com/UOR-Foundation/uor-r4/issues/1057#issuecomment-5513160718).

Admission reused eight Apple Accelerate CPU threads, one inter-op thread, and
one training process at batch 512. Its safety-adjusted additional projection
was `1,348.6584815112292 s`, within the unchanged `1,800 s`/`8 GiB` limits.
The exact inherited checkpoint is now continuing, not restarting a fresh fit.
Terminal development metrics, any conditional binding-control result, final
artifact, and fresh-process replay remain pending; launch is not a pass.

## Retained near-target learning result (2026-09-02)

The approved continuation improved development from `5,594/8,192 =
68.2861328125%` to `8,071/8,192 = 98.52294921875%`, a gain of
`30.23681640625` percentage points. NLL fell from `2.6761019229888916` to
`0.09141154401004314` nats. The final block-40 artifact and checkpoint are
retained as meaningful learning evidence and available research assets.
The best observed development score was block 36 at `8,082/8,192 =
98.6572265625%`, NLL `0.0898820087313652`; the exported artifact is the final
block-40 model, not the block-36 weights.

The raw frozen result is `CONTINUATION_MISS`: precisely, no evaluation
exceeded the predeclared strict `>99%` target. It does not mean false
attention, mechanism falsification, a capacity ceiling, convergence, discarded
work, or a project stop. This interpretation follows the
[user's explicit clarification](https://github.com/UOR-Foundation/uor-r4/issues/1057#issuecomment-5513353757)
without changing the threshold or control rule. The physical binding control
remains `NOT_RUN_PRIMARY_MISS`, with zero control decisions; a near-target
score is not retrospectively relabeled as a passed control.

The first 20 inherited history entries are preserved. The 20 additional
blocks below used the same 8,192 unique training rows with repeated exposure,
not a larger unique-data experiment. Each development evaluation has 8,192
queries; displayed percentages and NLL are rounded to six decimals.

| Block | Total updates | Correct | Top-1 (%) | NLL |
| ---: | ---: | ---: | ---: | ---: |
| 21 | 4,116 | 6,381 | 77.893066 | 1.707101 |
| 22 | 4,312 | 7,316 | 89.306641 | 0.701348 |
| 23 | 4,508 | 7,666 | 93.579102 | 0.397354 |
| 24 | 4,704 | 7,785 | 95.031738 | 0.287774 |
| 25 | 4,900 | 7,881 | 96.203613 | 0.223562 |
| 26 | 5,096 | 7,901 | 96.447754 | 0.206298 |
| 27 | 5,292 | 7,953 | 97.082520 | 0.178457 |
| 28 | 5,488 | 7,981 | 97.424316 | 0.161003 |
| 29 | 5,684 | 7,966 | 97.241211 | 0.163364 |
| 30 | 5,880 | 7,997 | 97.619629 | 0.157803 |
| 31 | 6,076 | 8,015 | 97.839355 | 0.133334 |
| 32 | 6,272 | 8,030 | 98.022461 | 0.121824 |
| 33 | 6,468 | 8,031 | 98.034668 | 0.118529 |
| 34 | 6,664 | 8,048 | 98.242188 | 0.104654 |
| 35 | 6,860 | 8,045 | 98.205566 | 0.110593 |
| 36 | 7,056 | 8,082 | 98.657227 | 0.089882 |
| 37 | 7,252 | 8,062 | 98.413086 | 0.100748 |
| 38 | 7,448 | 8,065 | 98.449707 | 0.091828 |
| 39 | 7,644 | 8,069 | 98.498535 | 0.092466 |
| 40 | 7,840 | 8,071 | 98.522949 | 0.091412 |

### Exact work, resources, and replay

| Primary work | Inherited | Additional | Total |
| --- | ---: | ---: | ---: |
| Blocks | 20 | 20 | 40 |
| Optimizer updates | 3,920 | 3,920 | 7,840 |
| Training queries | 16,056,320 | 16,056,320 | 32,112,640 |
| Development queries | 163,840 | 163,840 | 327,680 |
| Recorded elapsed seconds | 1,078.9107200000435 | 1,096.986829750007 | 2,175.8975497500505 |

Inherited elapsed time is the persisted parent checkpoint clock, not a fresh
measurement or the parent's slightly later artifact-finalization timestamp.
Additional execution took about 18.3 minutes and stayed within its `1,800 s`
budget. Peak RSS was `1,192,968,192` bytes, below `8 GiB`. Execution reused
eight Apple Accelerate CPU threads, one inter-op thread, and one process.
No CUDA/MPS, new timing matrix, C0, source reproduction, old QA, or fresh fit
ran. The seven focused continuation checks recorded at launch remain the
new implementation evidence; no repeated test campaign was added.

Preparation read the parent artifact, checkpoint, and evaluation RNG payload
once each and restored the exact checkpoint. New initializations, new/copied
corpus payloads, parent model inference, C0 updates, timing-matrix runs,
source-teacher weight reads, provider/teacher calls, sealed reads, model
role/geometry inputs, future-value reads, and control decisions were zero in
their respective ledgers.

Fresh-process verification passed, reproducing final development logits,
top-1/NLL and result identity. It added 8,192 development inference decisions
and zero optimizer updates or control decisions, separate from primary work.
The independent final-checkpoint audit also passed: all 20 exported tensors,
tied embedding/head, and evaluation RNG matched; the original 20 parent
history entries were unchanged. All AdamW counters were 7,840, scheduler
block 40 retained `T_max=64`, and its learning rate
`0.00014326648435691823` matched the declared analytic schedule. The sampler
recorded 490 cycles and cursor 8,192; work, time, and CIDs reconciled. This
read-only audit added no inference, tests, or control reads.
Broad QA remains dormant;
protected-queue acknowledgements are transport, not verification evidence.

### Evidence identities

The exact parent result/artifact/checkpoint identities and launch commitment
above remain bound. Additional identities are:

| Evidence | CID |
| --- | --- |
| Parent history | `blake3:cbb71c64faa2464d849ddcdef3d4d4044d4c61f5f7e6d94b9e05e56688c35b9f` |
| Parent binding | `blake3:9546417eaf6f0f6b1df20bc0f4ca4dc6dcd04ee8474980340d70dc3d6835cd75` |
| Primary data file | `blake3:96f154042f0fd920c7f6f3b1b650a6ce20f11c401f9ae0c81734f47ae231b7f1` |
| Primary tensor bytes | `blake3:baef4fd29bedddc5c9cd826b78101c5c412db0e883241e04e478e4e3baf1d8b1` |
| Bound control file; not scored | `blake3:34dad69bfbeda87ea7e4d5d7af2fd8434dc5ab33cc055df4a0965f0b7b96b693` |
| Reused preflight | `blake3:9f184ae09fea32d6a797303a3770ad10f5603ec4ac86bd2010bfc063ce39cdf6` |
| Implementation tree | `blake3:6d4548667d63d680ceb26a41a7c258abf0536f978909c2a90e26199ecebfac50` |
| Continuation binding | `blake3:fa2eb06da63e3cca8e1862a18b910a608d5fbc53720f116132bc4e592f7afcf5` |
| Primary result | `blake3:f5f7e5ca70bd4c3f9cad60ebe13886b452e82736c947a83cc048485e0dd55182` |
| Final model, 1,217,024 bytes | `blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445` |
| Final checkpoint, 3,759,925 bytes | `blake3:fd24e6b84af9891c1dad1eb2a13c6d86b8e8833ac02ab40f0e4e95d4530d140a` |
| Final model state | `blake3:f2a67ec0cc7ac44f586b815da43efabcc81d444b1bab9954b5536c37cb96ff90` |
| Final development logits | `blake3:3b50fe99e35e55602585cbba637eee3b3b6574393319bc96eddffacd65003e37` |
| Result | `blake3:35b1cedfd51385bf98277a4527b1ce05f5dd3b93fffe125a5ea28c2a34b6387c` |

### Retain the learned artifact and move toward R4 integration

Keep this near-target exact-data artifact and the complete curve; do not
discard, restart, or automatically add a third training window. The next
recommendation is a separately scoped inference-only coherent-R4 integration
using the already-qualified #1050 reference, while retaining #1057 as a
valuable exact-data companion for later inference work. #1050's `99.1667%`
comes from a different configuration and population, not a head-to-head
comparison with #1057. Its positive, HELM-D-R4, and #1014 remain intact.

The integration option substitutes only inner Q/K/V transport/aggregation,
preserving learned weights, positions, norms/residuals and softmax. It neither
depends on discarding this result nor promotes its unrun control. No such
integration, geometric-advantage claim, generation, reasoning, or lowering
was executed here. Benefit from additional unique data and model convergence
remain unmeasured.
