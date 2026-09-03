# Frozen raw-text adapter comparison — #1094

Current handoff: the separate [#1096 readiness decision](r4_isolated_runtime_readiness_1096.md)
returned `ISOLATED_RUNTIME_READY` with zero model loads/forwards. The subsequent
[preparation contract](r4_text_clause_preparation_1094.md) is frozen, with release
`NOT_ADMITTED`. #1094 remains open for implementation of retained-evidence
assembly and a launch gate that preserves the original budget. Comparison and
replay remain `NOT_RUN`. The original attempt below is unchanged.

**Original preparation result (2026-09-03): `UNAVAILABLE_REFERENCE_REPLAY`.** The sole authoring
preparation recovered all 320 valid inputs exactly and matched all 16 typed
refusals, with zero failures. The OS then denied execution of the pinned Python
interpreter before it started. Model loads/forwards are zero; worker readiness,
effective isolation, withheld comparison and replay are `NOT_RUN`. Supplied
segmentation remains the qualified model entry. #1094 remains open, blocked by
the separate [#1096 readiness repair](https://github.com/UOR-Foundation/uor-r4/issues/1096).
There was no retry, withheld release or post-stop source/profile modification.

The [retained evidence manifest](r4_text_clause_adapter_1094_evidence/manifest.json)
binds exact original receipts, and the [independent terminal review](r4_text_clause_adapter_1094_review.md)
confirms this boundary. The source implementation is committed at
`ff925481fcb290e8f91442a28a1b43b51b28dd26`. No raw-text model preservation,
general-language result or mathematical proof is claimed.

## Earlier preparation checkpoint

**2026-09-03 — preparation and independent source review in progress. Model
execution and withheld evaluation are `NOT_RUN`.** This record appends to the
accepted [#1085 specification](integration/clause-segmentation-1085.md); it does
not change its syntax, populations, thresholds, resource caps or outcome actions.
The active issue is [#1094](https://github.com/UOR-Foundation/uor-r4/issues/1094),
based on `3e894820c520f3b7803a48c6a2eeeb5b7d7021c5` in an isolated worktree.

## Decision and unchanged model boundary

**Empirical Criterion.** Determine whether the sole deterministic raw-text
adapter recovers independently annotated clauses and preserves the accepted
R4 reader/core computation. Positive evidence admits only bounded raw-text entry.
It cannot establish new semantic-world transfer, general language, reasoning,
coding, geometry advantage or the final source-free integer/table kernel.

Keep the #1077 reader, #1073 core, #1079 coherent R4 path, all soft mixtures,
full vocabulary, known query forms and four-fact context. No new fit or model
parameter is permitted. Preserve #1079 `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`,
including its token-control miss in three of six views, and #1082's descriptive
exposure/displacement result. The adapter is not a causal explanation for that
miss. #973's higher-context terminal remains unmet and #954 remains blocked.

The implementation lives in
[text_clause_adapter](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/adapter.py).
It receives an exact request schema plus original bytes, retains punctuation and
ordered spans, and emits only IDs/lengths to the unchanged reader. A boolean
grammar recognizer cannot forward captures, roles, semantic worlds or answers.
Typed refusals perform zero model forwards. Model output is the actual full-head
token, decoded through the core vocabulary; reader aliases are not output words.

## Independent preparation and commitments

The production adapter author and independent curator are separate agents.
The curator never imports the adapter or historical rendering/parsing/decoding
helpers and never loads a model. It verifies the original #1073 construction
semantics and #1077 construction distractor assignments, then authors text and
annotations independently. Text preparation is independent; semantic worlds are
historically observed. Five related rows and all their surface variants remain
in one partition and are not counted as independent statistical trials.

The policy bytes were fixed before population preparation at SHA256
`91cce30a0b78c48130595369d3ea2a47c4de89cab5db1d4219d1874198cf52d0`.
The independent selection receipt fixes canonical group serialization, four
authoring/sixteen withheld groups, their hashes, exact four surface-profile
rules, refusal families and boundary-control choices. It was written exclusively,
made read-only and [published before text generation](https://github.com/UOR-Foundation/uor-r4/issues/1094#issuecomment-5520856972)
at SHA256 `892e3239773e8a14e72ee650dc12c98ee4e1a5b432b69365a60cef8b15c9b5fa`.

| Partition | Valid rows | Refusals | Boundary controls |
|---|---:|---:|---:|
| Authoring | 320 | 16 | 0 |
| Withheld | 1280 | 64 | 16 |

The curator's population receipt is SHA256
`ad5bf0fdecb66b0de9e28c98941cf0fb2c6f737c7e1be3cbf48570822c65ba30`.
The [public curation record](r4_text_clause_adapter_1094_curation.json) preserves
the complete selection/population metadata and both preparation attempts,
including their original receipt identities and exact serialization.
Corpus, selection and policy total 3,397,265 bytes. Successful independent
preparation took 0.271631 seconds with 79,626,240 bytes peak RSS and zero model
forwards/optimizer updates. Its resources are reported separately from the
frozen comparison budget, as specified in #1085.

An initial launch used system Python 3.9.6 and stopped on unsupported
`zip(strict=...)` before any payload file write or withheld-text generation.
The exact unchanged curator source and selection then succeeded under the
existing Python 3.12.14 environment. Both immutable attempt receipts remain:
`0bccffabe574f8d79a9fef80b487acccb7e77a12e1b4a0f1395235972fbf4330`
and `ceb9582ad6cf63e8001cb8dd39505c130f68b47c0e1543e3dc90b3c3584255bd`.
This is an authoring-environment failure followed by execution of the same
frozen preparation, not a parser/model result or population revision.

Withheld raw/reference files remain under a mode-000 curator directory until
the complete source and independent review freeze. Authoring files alone may
guide parser preflight. No revealed withheld failure can be repaired and rerun
on these inputs.

## Execution and evidence accounting

The compact model binding carries only verified artifact, codec, frame, source
and runtime identities. Historical fit/preparation reports and their answer
arrays do not enter inference. Startup verifies all identities before request
admission or model construction. The two artifacts are loaded once per worker;
every parameter stays frozen and model state is compared before/after execution.

Reuse Python 3.12.14, Torch 2.7.1 and CPU Apple Accelerate with four intra-op
threads, one inter-op thread, one model worker at a time, deterministic
algorithms and batch size 128. The coordinator runs the oracle and adapter
workers sequentially. It sends only supplied IDs/lengths to O and raw requests
to A; row metadata, reference roles and targets stay in the coordinator.

The macOS file-access profile makes corpus, reference and historical result
paths unavailable to model workers while allowing the exact model assets,
source package and existing runtime. A harmless sentinel inside the denied
curator root must raise `PermissionError` before any model is constructed.
This is the declared oracle-access control, not a general sandbox-security
claim about hostile native code.

The empirical criteria remain 100% acceptance/input/span fidelity in all sixteen
form/profile cells; byte-identical full reader attention, all fifteen computed
role vectors, binding attention, full 4096-head logits and decisions; agreement
of the fourteen consumed diagnostic roles; exact typed refusals with zero
forwards; and independent fresh-process replay. Reference errors remain in the
denominators. Gold roles never select model roles or change model outputs.

The hard caps are 120 seconds preparation integrity, 120 seconds execution and
120 seconds replay, at most 360 seconds cumulative, 3 GiB peak RSS and 128 MiB
new corpus/results. The upper bound is 6400 logical row forwards, not an
obligation to execute invalid rows. Complete tensors are compared while
streaming; retained evidence uses domain-tagged digests, exact decision rows,
counts, model/work identities and bounded mismatch details. Only one temporary
oracle stream is needed for sequential comparison; it is discarded after its
bytes are compared and their receipts retained. No model downloads, fitting,
new geometry, accelerator search or broad QA is activated.

The named decision branches remain those of #1085: `CLAUSE_ADAPTER_PRESERVED`,
`CLAUSE_ADAPTER_PREFLIGHT_MISS`, `CLAUSE_ADAPTER_MISS`, the explicit
provenance/reference unavailable terminals, or `INCOMPLETE_RESOURCE`. A failure
retains its evidence and does not silently relax a threshold or renew a budget.

## Source and integration review

The original [NEMESIS/W33/UOR source audit](integration/clause-segmentation-1085-sources.md)
continues to support typed mapping, raw-byte identity and ordered provenance.
It supplies no parsing or learned-capability proof. The user-requested
[afflom ecosystem follow-up](integration/afflom-ecosystem-followup.md) inspects
Prism, both Atlas sources, LexLean, lean4-prod, GNAF and both matmul repositories.
Their concrete integration opportunities belong to #1083/#1087/#1089; the frozen
comparison's arithmetic and reader are unchanged.

The [independent review](r4_text_clause_adapter_1094_review.md) must bind final
source identities before withheld release. Only the declared preparation,
comparison/refusal/replay, claim-wording and evidence-integrity checks are
active. Protected-queue acknowledgements are transport, not scientific tests.
Storage review preserves the original mixed checkout, unique model/corpus and
diagnostic evidence, sealed files and user material.

## Observed preparation and terminal

The named command used the existing qualified interpreter and this committed
package, with `PYTHONDONTWRITEBYTECODE=1`, `PYTHONUNBUFFERED=1` and an explicit
source `PYTHONPATH`:

```sh
/Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python \
  -m r4_softmax_trainer.text_clause_adapter prepare \
  --repo /Users/casey.allard/.codex/worktrees/r4-text-clause-adapter/uor-r4 \
  --corpus /Users/casey.allard/.codex/uor/issue-1094-curator \
  --output /Users/casey.allard/.codex/uor/issue-1094-comparison \
  --python /Users/casey.allard/.codex/worktrees/r4-language-r4/uor-r4/tools/r4-softmax-trainer/.venv/bin/python
```

| Named evidence | Observed result |
|---|---|
| Authoring acceptance, IDs/lengths, raw/derived identities and token/clause spans | 320/320 exact; 20 per each of sixteen form/profile cells |
| Authoring refusals | 16/16 exact; one per frozen refusal family |
| Byte-buffer and external-role schema probes | Both refused with the exact required schema/tag |
| Independent authoring reference integrity | Four original semantic groups, expanded to 64 group/surface combinations; all five rows retained |
| Isolated worker startup | `sandbox-exec: execvp() ... failed: Operation not permitted`; no Python readiness event |
| Model loads, logical forwards, optimizer updates | 0 / 0 / 0 |
| Withheld valid/refusal/boundary rows, soft tensors, answer/role accuracy, fresh-process replay | `NOT_RUN` |

The [authoring receipt](r4_text_clause_adapter_1094_evidence/authoring-input-preflight.json)
was persisted before the failed launch. Its SHA256 is
`f79df8623038961d899ba727d99bb69b39754b1878d10bbed9da0bfe03e5ee82`.
The [stop receipt](r4_text_clause_adapter_1094_evidence/prepare-stopped.json), SHA256
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`,
records `UNAVAILABLE_REFERENCE_REPLAY`. No successful preparation envelope,
execution-start, replay, result or completion receipt exists.

The binding names Apple M1 / MacBookPro17,1, eight logical CPUs and 16 GiB memory.
The coordinator recorded approximately 0.2603 seconds and 65,601,536 bytes peak
RSS. Python/Torch/Accelerate settings were declared but were not verified in the
worker because execution was denied. The recorded worker RSS zero means no
worker measurement arrived; it is not successful worker resource evidence.
Final budgeted corpus/receipt bytes are 3,465,401, including the last resource
receipt. The seven retained original preparation files total 66,523 bytes.

**Decision.** Keep supplied segmentation and preserve the unobserved withheld
population. #1096 must identify the actual launch denial and independently
qualify the minimal runtime access policy with zero forwards and harmless denied
path probes. Do not infer the cause from the single execvp message or broaden
the home-directory allowlist. Only a separately reviewed preparation can later
reopen #1094's comparison. #1086 may specify export for the supplied-clause
reference, but cannot advertise raw-text qualification. #973 remains open and
#954 blocked; #1079/#1082 findings remain unchanged.

The only executed decision checks were static Python syntax/source integrity,
claim wording, independent receipt verification and this authoring/readiness
preflight. Broad QA and model evaluation remain dormant. All model assets,
corpus files, sealed withheld inputs, original receipts and mixed-checkout user
changes are retained. No storage cleanup was performed.

## Subsequent isolated-runtime handoff — #1096 (2026-09-03)

The separate [readiness record](r4_isolated_runtime_readiness_1096.md) qualifies
one zero-forward worker startup at source
`79c674c8f6179a68878a12ee86e664f1435c3ebf`, manifest SHA256
`4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644` and
result SHA256 `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930`.
All four harmless probe reads were denied; runtime/source/asset identities were
verified, states remained null, and loads/forwards/updates were zero. No corpus
preparation, withheld access, model-output comparison or replay occurred.

The original unavailable preparation and its exact receipts remain historical
evidence, not a success retroactively. After #1096's protected delivery, #1094
must separately freeze preparation against the accepted readiness bindings,
account for its consumed budget and obtain independent release review before
withheld access/comparison. It remains open and parked/unassigned until active.


## Subsequent preparation contract and release review (2026-09-03)

The separately requested [contract](r4_text_clause_preparation_1094.md),
[machine-readable freeze](r4_text_clause_preparation_1094_contract.json),
[independent budget audit](r4_text_clause_preparation_1094_budget_audit.md),
[source audit](r4_text_clause_preparation_1094_source_audit.md) and
[contract review](r4_text_clause_preparation_1094_review.md) preserve the
original unavailable attempt and accepted #1096 startup without calling either
a new successful preparation or a model result. No preparation worker, fit,
withheld access, comparison or replay ran in this contract step.

The last original resource snapshot predates its own serialization/write and
exit, so arithmetic remaining-time figures are upper bounds. The chosen
resumption policy quarantines the full original 120-second preparation
allocation. This is a policy debit, not a claim that 120 seconds was measured.
Execution and replay retain at most 120 seconds each under the unchanged
360-second total. Original counted bytes are 3,465,401; the remaining allowance
is 130,752,327 bytes before new receipts. #1096 and independent authoring retain
their separate ledgers. No allocation or original result is reset.

A new exclusive assembly schema/status must link the original authoring and
readiness receipts and bind the exact implementation, clean child environment,
profile delta, runtime/source/asset identities and carried budget. The current
coordinator implements neither that assembly nor its release consumer. Release
is therefore `NOT_ADMITTED`. The next action is to implement this bounded
coordinator path under #1094, then obtain independent review of the exact
implemented envelope before any withheld execution. Adapter, worker, curator,
model computation, populations and empirical criteria remain unchanged.

Storage was reviewed against the September 2 audit. At this checkpoint the
volume had 50,227,488 KiB (47.90 GiB) available, and the isolated contract
worktree occupied 1,165,544 KiB (1.11 GiB) of directory allocation. These are
snapshots, not a reclaimed-space claim. This step adds documents and metadata;
no model/corpus/runtime copy, build output or download was created. Original
model/evidence stores and sealed input directories are retained. The primary
checkout's two pre-existing tracked deletions and untracked user material are
unchanged. Nothing was deleted, pruned or unsealed.


## Retained-evidence implementation and release — 2026-09-03

The [retained assembly and carried-budget gate](r4_retained_assembly_1094.md)
are implemented at source `07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`.
The exact metadata assembly has separate status
`PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE`; its immutable embedded
`NOT_ADMITTED` is accompanied by an independent
`ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON` release receipt. The evidence index
and independent review bind the actual source/path/profile/population envelope.
All 18 named synthetic code checks passed. No new preparation, readiness worker,
runtime/asset verification, model work or withheld access ran in this step.
The original `UNAVAILABLE_REFERENCE_REPLAY` and authoring observations above
remain unchanged. Original 120 seconds are quarantined as a policy debit and
3,465,401 historical bytes remain counted; the new output cannot reset them.
The separately activated frozen comparison/replay is the next action. #1094's
scientific DoD and #973 remain open; #954 remains blocked.
