# Independent preparation source audit — #1094

**2026-09-03 — `SOURCE_INSPECTED`; `RELEASE_NOT_ADMITTED`.** The independent
`/root/preparation_source_review` agent inspected source at
`6f21fc5f4c40b9620c9fec5e95a39097f812ae73`, the protected #1085 specification,
the original #1094 preparation/review and the delivered #1096 handoff. This
audit ran no candidate package, preparation, readiness worker, model, corpus
generation, comparison or replay. It did not list or read sealed withheld
contents. The present deliverable is a preparation contract and source review;
no `prepare` or `run` command is admitted by it.

## Preserved source and evidence

The [#1085 specification](integration/clause-segmentation-1085.md) still has
SHA256 `85f928fec94fa0f6793cff4c35e1fc8c9cba691739d34db272465766c7c9dab1`.
The [original #1094 terminal](r4_text_clause_adapter_1094.md) remains
`UNAVAILABLE_REFERENCE_REPLAY`, with 320/320 authoring inputs and 16/16 refusals
exact before the denied interpreter launch. Its stop is SHA256
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`; its
authoring receipt is
`f79df8623038961d899ba727d99bb69b39754b1878d10bbed9da0bfe03e5ee82`.

The [accepted #1096 readiness](r4_isolated_runtime_readiness_1096.md) binds
executed source `79c674c8f6179a68878a12ee86e664f1435c3ebf` and the following
exact original objects:

| Object | SHA256 |
|---|---|
| Readiness manifest | `4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644` |
| Model/source bindings | `23977918c1685148f6cbc61ebab87507e2ce153e40d9b15c2634a4d68e0c2353` |
| Sandbox profile | `d508d3ec50b78e3bbd3cb1fe9468788f457f4e47f713f9a793f03079df48416e` |
| Independent admission | `e092e4bc3d3a989d37868c5530ff3483d80f4a64e53b3cf609aa1774a5c43759` |
| Observed readiness result | `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930` |

That manifest names the `r4-runtime-readiness` worktree and the
`issue-1096-readiness-freeze02` binding/profile directory. It verifies 169 source
files, five assets, the interpreter aliases and 18 installed runtime files.
The source diff from original #1094 to the inspected revision changes only
`contract.py`, `worker.py` and the new `readiness.py` within the trainer package.
The adapter, curator and comparison coordinator are unchanged. This static
observation does not turn the unavailable preparation into a successful one.

## Actual preparation and release boundaries

The inspected [campaign coordinator](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/campaign.py)
has these concrete operations and boundaries:

- `prepare`, lines 427–520, starts a new budget and exclusive preparation-start
  receipt; reads selection/population metadata; verifies only authoring payload
  hashes; creates new model/source bindings and profile; and reads authoring raw
  and reference rows. It runs parser/reference integrity and schema checks with
  zero forwards, retains their receipt, then calls the adapter worker with
  `--readiness-only`. It writes `COMPARISON_PREPARED` only after that worker
  completes and the authoring checks pass. It does not read withheld payloads.
- `Budget.snapshot`, lines 282–301, counts sealed corpus bytes from the public
  population receipt. It traverses only the chosen output directory. A future
  envelope must bind an exclusive output outside the sealed corpus tree so that
  accounting cannot traverse withheld inputs.
- `arm_process`, lines 344–424, executes the actual worker, verifies its source/
  asset binding and runtime receipts, and requires the existing denied sentinel.
  It currently inherits the entire parent environment and does not pass the
  #1096 manifest or its hash. The worker's `--readiness-only` branch skips model
  construction and batches; its ordinary campaign receipt does not contain the
  four-probe/runtime-file identity receipt used by #1096.
- `run`, lines 837–939, requires `COMPARISON_PREPARED` and a separate review
  status `ACCEPTED_FOR_FROZEN_COMPARISON` bound to preparation, bindings,
  population and selection hashes. It creates `execution-started.json` before
  its first withheld read, including hashing. It then runs both model arms and
  automatically runs fresh-process replay. There is no separate prepare-only
  stopping point inside `run` and no release approval from this audit.

The [worker](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/worker.py)
checks executing source paths and exact asset bytes in `_verify_bindings`,
lines 166–249. Its separately bound runtime-file/probe check lives in
`_readiness_identity`, lines 134–163, and is selected only by the optional
#1096 arguments in `main`, lines 557–599. Exact artifact byte hashing is distinct
from model deserialization or a model forward. Historical report reads in
`contract.make_bindings` remain coordinator-side; the compact model binding
does not transmit historical answer arrays to inference.

## Required receipt assembly and later execution admission

These are implementation clauses for the subsequent bounded step, not changes
made or behavior tested by this contract-only delivery.

1. Implement receipt-bound preparation assembly with an explicit new schema and
   status. It links existing authoring and readiness evidence; it must not emit
   `COMPARISON_PREPARED`, manufacture a successful `prepare` receipt, or invoke a
   new preparation worker. Bind the original stop/resources/authoring receipts, accepted
   #1096 objects above, newly committed coordinator closure, actual absolute
   source/output/binding/profile paths, unchanged policy/curator/selection/
   population commitments, five assets, runtime files, launcher/aliases and
   hardware. A source/path change requires a new explicit identity; it cannot
   inherit the old measured profile hash by name.
2. Quarantine the full original 120-second preparation allowance inside the
   360-second cumulative cap. The original
   last resource receipt records cumulative `0.2602929580025375` seconds,
   zero forwards and 3,465,122 bytes before its own 279-byte write; the retained
   byte total is 3,465,401. The recorded time is a snapshot, not a measurement
   of the subsequent receipt-writing/exit tail. Charging the full original
   allowance is a conservative reservation, not a claim that 120 seconds was
   measured. The preparation remainder is zero, so no new preparation invocation
   is admitted. The execution and replay allowances remain 120 seconds each,
   subject to later independent release. Count the original retained receipts and new output
   while counting the corpus once; report the separately budgeted #1096 work
   explicitly. Current `Budget(args, "preparation")` resets carry-in, its phase
   check considers only new elapsed time, and `main` arms a fresh 120 seconds.
   Those behaviors cannot implement the selected resumption. Later `run` currently reloads only
   `preparation-closed.phase_elapsed_seconds`; that transfer must preserve total
   preparation consumption rather than losing the inherited portion again.
   The new path must recognize the distinct reviewed assembly schema/status,
   carry 120 seconds explicitly and charge all new runtime/source/profile
   verification to execution before its first withheld payload read or model
   work. Missing admission or failed identity checks stop before access.
3. Freeze an exact profile delta before observing execution. The generator in
   [contract.py](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/contract.py),
   lines 138–169, embeds absolute paths. Replacing the old source tree with a
   new worktree grants access to a different tree; replacing binding/output
   literals changes access identity. These are explicit grants/removals, not
   byte-identical policy reuse. Ordinary `prepare` also omits the two extra
   manifest/profile literal permissions present in #1096; removing those is a
   narrowing, but still changes the bytes. Preserve exact interpreter aliases,
   ancestor metadata rules, five assets and denial of network/home writes;
   admit no corpus/reference/history/results contents or broader home tree.
4. Replace inherited `os.environ` with a frozen clean environment equivalent to
   #1096's declared PATH/HOME/PYTHONPATH and fixed CPU/thread settings, including
   `PYTHONNOUSERSITE=1`. Exclude inherited `PYTHONHOME`, loader injection and
   unrelated import settings. Bind and verify the accepted 18 runtime files,
   interpreter aliases, assets and hardware under the later execution/replay
   clocks before model work; bind their post-process checks as well.
   Keep adapter, curator, worker and model computation unchanged. The child must
   still verify the exact source/assets, fixed runtime and denied harmless
   sentinel before model construction, followed by the existing frozen model
   state and actual forward accounting. Parent runtime-file verification plus
   those child receipts must be described accurately; they are not a new
   zero-forward readiness observation or #1096's four-probe receipt.
5. Make assembly immutable and exclusive, with no execution side effect. Keep
   the irreversible execution/replay start receipts, deadlines reduced by
   carry-in, bounded receipt storage and retained partial evidence. A stop
   admits no retry, policy widening or renewed budget. Only independent review
   of the exact assembled evidence, implemented consumption path and final
   committed bindings can later issue the comparison release receipt. Changing the sealed
   directory's permissions is a separate release action after that review;
   the current source does not perform it.

These requirements can be implemented in the coordinator without changing the
accepted adapter, worker, curator or model computation. Receipt assembly is
feasible as a new explicitly typed path; it does not recover an unmeasured
preparation tail or establish a new empirical result. Until that path and its
execution consumer are implemented, frozen and independently reviewed, the
existing `prepare` or `run` commands cannot implement the selected resumption.
The present contract does not authorize invoking `run`, changing sealed access,
issuing `ACCEPTED_FOR_FROZEN_COMPARISON`, or treating readiness as raw-text
preservation. #1094 remains open. #1079's weak-control result, #1082's descriptive
limits, #973 open and #954 blocked remain unchanged. This is static source
evidence and a future implementation boundary, with no new mathematical proof
or empirical model result.
