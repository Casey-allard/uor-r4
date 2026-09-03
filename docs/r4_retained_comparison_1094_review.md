# Independent comparison review — #1094

## Source launch handoff — 2026-09-03

**`SOURCE_LAUNCH_HANDOFF_CONFIRMED`; result review pending.** The independent
`/root/retained_admission_audit` reviewer read the frozen preparation contract,
implemented handoff, actual bound coordinator CLI, sealed-directory admission
code and original exact release. This checks the one-shot launch arrangement;
it does not repeat the completed implementation/envelope approval or perform
fresh runtime/source-closure/asset verification outside the campaign clock.

The evidence worktree starts at protected
`1ff8b81cb060ca2e8ced409ec78ae566b5b98891`. Execution remains bound to the existing
coordinator source `07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`, actual path
`/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4`. The separate
new evidence worktree must not replace that path. The existing worker remains
at `/Users/casey.allard/.codex/worktrees/r4-runtime-readiness/uor-r4` under its
accepted source `79c674c8f6179a68878a12ee86e664f1435c3ebf`.

The original output is
`/Users/casey.allard/.codex/uor/issue-1094-retained-assembly01`.
Its unchanged assembly SHA256 is
`48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`,
and the separately accepted release SHA256 is
`5787e4a64113800c5fc82cd1d32d564d9c6e3a344e74ca102a754fe82dccee23`.
The reviewer inspected those named metadata objects without repeating their
previous complete hash/source review. The already accepted release admits one
future comparison plus fresh-process replay; the current user activation
selects that one attempt.

### Required operator handoff

1. Preserve the exact original output and all existing receipts. The consumer
   refuses a prior admission/start/progress/stop/completion marker. Do not make
   a fresh output or repeat preparation to get another attempt.
2. Immediately before the single invocation, the operator changes only the
   original `/Users/casey.allard/.codex/uor/issue-1094-curator/withheld`
   directory from mode `000` to owner-read/execute mode `0500`. No recursive
   permission change, payload preview, copy, filtering or regeneration belongs
   in this handoff. `_sealed(require_sealed=False)` accepts `000` or `0500`,
   but leaving it at `000` would cause the admitted attempt to fail when the
   coordinator reaches withheld reading.
3. Invoke `campaign run-retained` exactly once. It owns both sequential model
   arms and the subsequent fresh-process replay; there is no second replay
   command or new preparation invocation. Its initial clock covers admission,
   fresh source/runtime/release checks and execution. It writes a durable
   admission marker before fresh checks and an execution-start receipt before
   the first withheld hash/read.
4. The coordinator, retained module and worker contain no chmod or reseal
   operation. The operator must restore the same withheld directory to `000`
   in an outer `finally` after the coordinator and its workers have exited,
   including nonzero/exceptional termination. Workers use separate process
   sessions; an external emergency termination must not assume killing only
   the coordinator's process group also kills a worker. Preserve partial
   evidence and report any exceptional cleanup rather than retrying.

The reviewed command arguments are below. The clean coordinator environment
selects the bound package and existing interpreter; the consumer independently
supplies the exact already-reviewed worker environment. Run from the existing
coordinator worktree, with operator permission/reseal handling around this one
invocation and output capture that does not overwrite a retained receipt:

```sh
/usr/bin/env -i \
  PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  HOME=/Users/casey.allard \
  PYTHONPATH=/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4/tools/r4-softmax-trainer/src \
  PYTHONNOUSERSITE=1 PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1 \
  OMP_NUM_THREADS=4 VECLIB_MAXIMUM_THREADS=4 \
  /Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python -B \
  -m r4_softmax_trainer.text_clause_adapter.campaign run-retained \
  --repo /Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4 \
  --corpus /Users/casey.allard/.codex/uor/issue-1094-curator \
  --output /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01 \
  --python /Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python \
  --assembly /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01/retained-preparation.json \
  --review /Users/casey.allard/.codex/uor/issue-1094-retained-assembly01/release.json
```

The original 120-second preparation allocation remains a policy debit, with
3,465,401 historical bytes. Fresh execution and replay each have at most 120
seconds, within 360 seconds cumulative; the 3-GiB combined RSS, 128-MiB retained
byte and 6,400 logical-forward caps remain unchanged. Fresh identities must be
checked inside these clocks. Any admission marker consumes the envelope and a
`run-stopped.json` takes precedence over completion. No automatic retry, new
budget, input repair, policy widening or tolerance change is admitted.

No source-based launch blocker was found. This reviewer did not invoke the
command, hash runtime/model assets, traverse the source closure or withheld
directory, read a payload, launch a worker or change permissions. The eventual
independent result review must inspect the original terminal, execution/replay,
resource, identity, fidelity and completion/stop receipts. Until those exist,
model behavior is unmeasured. The original unavailable preparation, #1096
runtime-only readiness, #1079 weak control and #1082 descriptive limits remain
unchanged; this handoff supplies no new mathematical proof.

## Independent original-result review — 2026-09-03

**`INDEPENDENT_RESULT_ACCEPTED: CLAUSE_ADAPTER_PRESERVED`.** The independent
`/root/retained_admission_audit` reviewer accepts the sole retained-evidence
comparison and its fresh-process replay under the frozen #1085/#1094 contract.
The original completion receipt binds the positive result and final resource
receipt, and the original output contains no stop marker. The bounded empirical
DoD for #1094 is met; issue closure is appropriate after protected delivery of
these records. This decision does not authorize another invocation.

The reviewer read the original 17 retained files in
`/Users/casey.allard/.codex/uor/issue-1094-retained-assembly01`, rather than relying
on the operator's summary or copied report. A standard-library, read-only
assertion pass verified the receipt SHA256/byte chains, reconstructed canonical
deterministic and oracle digests, every recorded decision/check/cell/tensor
layout, worker/progress identities, complete group counts, resource arithmetic
and terminal precedence. It returned `PASS_ORIGINAL_RECEIPT_ASSERTIONS`.
The reviewer launched no model or runtime probe, performed no fresh source
closure or model-asset hashing, and read no withheld payload. The only withheld
filesystem operation was `lstat` of the directory itself to confirm resealing.

### Bound evidence and source

The assembly, release, bindings and worker profile still have the exact hashes
listed in the launch review. The durable admission and execution-start receipts
bind those objects, carry the 120-second policy debit and 3,465,401 historical
bytes, and record zero withheld reads before the execution-start boundary.
Replay starts with 3,200 carried forwards and 128.1820302089909 charged seconds,
including the preceding phase and its receipt tail.

| Original receipt | Bytes | SHA256 |
| --- | ---: | --- |
| `execution.json` | 1,894,358 | `c803b0daa1769f26af82bc093d330e59a8fdd390fdc760ed501a73618580bd91` |
| `replay.json` | 1,894,351 | `a803cdbc361adc2ccb9b6fefaec24f785982f86faa3ae296da08f573fa0915e2` |
| `result.json` | 1,598 | `c50b354f8da5ae170b97eabbc3b887bf065efb8e1861353cb485d8515c558171` |
| `final-resources.json` | 344 | `3bdb020aad7b29e56c1eaff1028f02ab8bf44b110715a46d5019517c84516361` |
| `completion.json` | 720 | `ea14a44f6077fd9af2ef13bd1d3acb7976f2ac8fbb95aa76b320da14ae79f36c` |

Each phase contains five fresh identity events: the coordinator before withheld
access or replay, and before/after each of the two workers. All ten events bind
the accepted coordinator and worker source digests and hardware, and each
reports verification of five assets, 18 runtime files and two interpreter
aliases with zero model forwards during the identity check. These are file and
hardware identity receipts; they are not additional #1096 readiness trials.
Scoped Git comparisons of the actual coordinator and worker adapter package
sources against `07ec3f0d39d08ac5bf9c2ba7a6b864229e007867` and
`79c674c8f6179a68878a12ee86e664f1435c3ebf`, respectively, returned no changes.
The prior complete source approval remains the source-closure authority.

### Empirical fidelity and replay decision

Each arm in each phase consumed 1,600 valid rows: 320 authoring and 1,280 withheld,
with all 16 form/profile cells in each partition. The adapter also refused all
80 ordinary refusal rows across the 16 frozen families and all 16 boundary
controls. The full 96 refusals produced zero model forwards and matched the
frozen refusal receipts; no row repair, selection change or tolerance is used.
The partition integrity records preserve independent selection. Their `groups`
counts are keyed by `(base group_id, form, profile)`: four authoring base semantic
groups become 64 form/profile group instances, and sixteen withheld base semantic
groups become 256 instances. Each of the twenty already-observed base groups
appears in all sixteen form/profile cells with five query variants; these 1,600
rendered rows are not 1,600 independent semantic trials. Independent inspection
of the retained group IDs confirms those four/sixteen distinct base counts.

All 166 deterministic checks passed, with zero recorded failures. Every valid
row records exact adapter input fidelity and exact equality for all eight
compared tensors: inputs, lengths, role attention, role vectors, binding
attention, logits, predictions and role positions. The unchanged consumer
compared the actual bytes during the attempt; the retained evidence records the
per-row equality and the corresponding tensor identities. The independent
review verified those receipts and their layout/denominators without rerunning
inference or reconstructing removed raw tensor streams.

There are 208 tensor receipts per phase: two arms, 13 valid batches and eight
tensors. Batches 0–11 contain 128 rows and batch 12 contains 64. The int64
`inputs[B,5,13]` and `lengths[B,5]` layouts and all remaining frozen tensor shapes,
dtypes, byte counts and valid indices agree. Tensor receipt hashes include the
arm in their domain; cross-arm byte equality is established by the comparison
receipts, not by requiring the arm-specific hashes to be equal.

For each arm/phase, all 1,600 answer decisions equal both the oracle and target,
and all 22,400 consumed role decisions are correct. Every all/supported/unknown
stratum has its prescribed denominator in all 32 partition/form/profile cells;
all 320 form/profile group instances per arm are complete for both the
supported-four and five-row criteria. Those instances represent the same twenty
base semantic groups, not 320 distinct worlds. These are counts on the frozen
known-world family. The 15
role slots materialized per row are distinct from the 14 roles consumed by the
criterion; no double counting of the all and supported/unknown strata is used.

Execution and replay contain equal deterministic objects. Independently
reconstructed canonical SHA256 values agree with both receipts:

- Complete deterministic record:
  `b28336e8c0413b277c5655d2841b7aef4a0e254618aa1910c50146e7dfcea1d4`.
- Oracle-only replay identity:
  `a3420cd5b865c0f55d08bce02d3292bb7349c0679e3acad2616da9cb31e49470`.

The four sequential workers each report two model loads, 13 valid batch
forwards and 1,600 row forwards, totaling exactly 6,400 logical forwards. Each
starts from and ends at the accepted reader/core state identities. Every audit
reports zero optimizer updates, future-input reads, oracle/label-file reads,
refusal forwards, padding transport and frame-matrix/position changes. The
ordinary worker isolation probe is denied as required. All workers report the
fixed Python 3.12.14/Torch 2.7.1 CPU runtime, four computation threads, one interop
thread, one worker and Accelerate BLAS.

### Resources, terminal tail and storage

Execution closes at 8.181647041987162 measured phase seconds, with 128.18164704198716
charged cumulative seconds. Replay closes at 7.509532583004329 phase seconds and
135.69156279199524 charged cumulative seconds. The final resource receipt records
135.69782133398985 charged seconds and a combined peak-RSS bound of 471,531,520
bytes. Each phase is within 120 seconds, cumulative time within 360 seconds,
and RSS within 3 GiB. The historical 120 seconds remains a conservative policy
debit, not a recovered measurement of the unavailable preparation.

The operator receipt
[`r4_retained_comparison_1094_operator.json`](r4_retained_comparison_1094_operator.json),
SHA256 `2743d42b9e4a8051dd1d39b8a0aa55677af2508cda57c9d59f2c63d159a6472d`,
records one invocation, return code zero, empty stderr, no outer failure and no
emergency descendant termination. Its 15.821630625-second outer wall measurement
includes permission handoff, interpreter startup, command completion and
resealing. It covers the final receipt-write/exit tail that the earlier final
resource snapshot cannot itself measure. Its activation hash matches the
retained activation record. Independent directory `lstat` confirms device
16777232, inode 64741475 and mode `000`, matching the operator's reseal receipt.

The original 17 files total 4,033,394 bytes. Adding the 3,465,401 historical bytes
gives 7,498,795 charged retained bytes. This exactly equals the final resource
snapshot's 7,497,731 bytes plus its own 344 bytes and the 720-byte completion
receipt. The final byte count therefore includes the terminal write tail.

Two temporary oracle streams were deleted by the frozen consumer: one after
each phase, each 47,281,715 bytes with SHA256
`2cd1412bc8866be60ab63a294377ae0f6c3ae71ac89a34f02bb089975b248f64`.
Their progress events bind cleanup to the already persisted corresponding
execution/replay evidence. The source confirms that persistence identity is
checked before `unlink`; both temporary paths are now absent. This was the
registered sequential-arm spool cleanup, and must not be described as “nothing
was deleted.” Adding one maximum-size active stream to the final charged
retained footprint gives a conservative reconstructed bound of 54,780,510 bytes,
below 128 MiB. This is a reconstructed upper bound, not a sampled peak byte
measurement. No cleanup was performed by this reviewer; the complete retained
receipts and prior evidence remain available.

### Closure scope and next action

**Empirical result:** externally supplied clause segmentation can be replaced by
the frozen bounded text adapter while preserving the accepted reader/core,
known vocabulary, query form and four-fact context on this exact family and
runtime. Fresh-process replay preserves the recorded deterministic evidence.
This admits the bounded text entry to the research reference under the frozen
contract. It supplies no new mathematical proof, fit, generation result,
semantic-world novelty, general parsing/transfer result or geometry-superiority
claim. The earlier `UNAVAILABLE_REFERENCE_REPLAY` preparation remains a negative
record; #1096 readiness, #1079 weak control and #1082 descriptive evidence retain
their original meanings.

After protected delivery, close #1094 with this bounded result. Keep #973 open
and #954 blocked. The concrete next action is to hand the accepted bounded
entry's schema and evidence to the existing #1086 native-export specification
task, following its live eligibility and frozen scope; it is not started by
this review.
