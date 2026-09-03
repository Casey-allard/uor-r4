# Retained preparation assembly and launch gate — #1094

**Implementation checkpoint, 2026-09-03.** This step implements the
[preparation contract](r4_text_clause_preparation_1094.md) delivered in protected
PR #1099 at `6008e3527cb119d12af4abd99fe86a3d4ebe5a53`. Its named decision is
whether the exact retained evidence can be assembled and independently admitted
without repeating preparation or resetting its budget. No real comparison,
replay, preparation worker, fit or withheld access is part of this step.

## Implemented boundary

`text_clause_adapter.retained` is a standard-library-only metadata assembler.
It verifies the pinned original #1094/#1096 evidence, original public population
commitments and committed source closures. It keeps the accepted worker source
tree and binding bytes, generates the exact new binding-path profile, and
records the complete profile delta, clean child environment, runtime identities,
sealed-directory metadata and original time/byte debit. It neither imports the
adapter nor hashes/loads runtime or model assets during assembly.

`campaign run-retained` requires the distinct assembly and exact independent
`release.json` from its exclusive output. Lightweight path/provenance admission
precedes an exclusive `admission-started.json`; fresh source/runtime checks then
run under the execution clock. An interruption after admission consumes that
envelope. An exclusive `execution-started.json` must exist before the first
withheld payload hash/read. Reuse, legacy approval, path aliases and changed
identities fail closed.

Every snapshot carries the policy debit of **120 seconds** and original
**3,465,401 bytes**, counting the corpus once. Execution and replay retain their
120-second ceilings and the unchanged 360-second cumulative cap, 3-GiB combined
RSS bound, 128-MiB byte ceiling and 6,400 logical forwards. New output and active
oracle streams count in addition to history. A fresh output cannot reset it.

The actual worker receives the exact clean environment and unchanged accepted
worker source. Parent checks bind the profile, release, sources, eighteen
runtime files, both interpreter links, hardware and five assets before/after
worker attempts. The child retains its existing single harmless denied-read
receipt; this is not a new four-probe #1096 readiness result. If a resource stop
prevents a post-worker identity check, it is recorded as `NOT_VERIFIED`, and
post-worker diagnostic write errors do not replace the original cause. Any
`run-stopped.json` overrides completion. No automatic retry is admitted.

The accepted reader/core, adapter, worker, curator, vocabulary/query, four-fact
context, populations, full soft outputs/logits/decisions, refusal rules and
comparison criteria remain unchanged. Source additions are restricted to the
retained protocol; only the existing coordinator may change.

## Verification and evidence

The [independent source audit](r4_retained_assembly_1094_source_audit.md) follows
the knowledge index to the original contract and records the actual
NEMESIS/W33/UOR source pins. Their relevance is explicit admission, mapping and
identity. They supply no proof or model-preservation result.

Named synthetic checks exercise admission failure before payload/launch,
metadata/source/profile/approval drift, unsafe output and reuse rejection,
historical time/byte carry into replay, resource caps, clean environment and
pre/post-worker cleanup. They use temporary synthetic text and in-memory worker
stubs. They are code checks, not preparation, runtime readiness or empirical
model comparisons. Broad QA remains dormant; protected queue acknowledgements
are transport only.

This record will append the committed source, exact assembled envelope,
independent review and delivery evidence when each exists. The original
#1094 `UNAVAILABLE_REFERENCE_REPLAY`, #1096 `ISOLATED_RUNTIME_READY`, #1079
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK` and #1082 descriptive limits are preserved.
Supplied segmentation remains the qualified model entry. #1094/#973 remain
open and #954 blocked until the separately defined scientific terminals change.

## Direct implementation check record

The root coordinator directly ran the two named synthetic check files: **7/7**
retained-metadata checks and **11/11** campaign admission/accounting checks passed.
The [complete transcript](r4_retained_assembly_1094_checks.txt) has SHA256
`85cfa49159f653e08fe8eecd716d31c80249359dada001df585da4c76597f91e`.
Claim-wording and diff-whitespace checks passed. No broader QA or real model
operation was used. The [independent implementation review](r4_retained_assembly_1094_review.md)
binds the four source/check files and accepts metadata assembly from committed bytes.

## Committed metadata assembly

The reviewed implementation was committed at
`07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`. Its standard-library-only assembly
command completed successfully and created the exclusive output
`/Users/casey.allard/.codex/uor/issue-1094-retained-assembly01`.
The 115,900-byte `retained-preparation.json` has SHA256
`48fae2d391e347e89a290b12a8af97cf8266c5913a21e71f21c1bef74ef54c62`
and status `PREPARATION_ASSEMBLED_FROM_RETAINED_EVIDENCE`.
Its embedded `release: NOT_ADMITTED` is immutable. A separate independently
reviewed `release.json` is the only comparison admission receipt.

The envelope binds the executing coordinator at
`/Users/casey.allard/.codex/worktrees/r4-retained-assembly/uor-r4` and the unchanged
accepted worker at `/Users/casey.allard/.codex/worktrees/r4-runtime-readiness/uor-r4`,
source `79c674c8f6179a68878a12ee86e664f1435c3ebf`. Later documentation commits do
not change either source closure. These paths and the existing Python runtime
must be retained for the bound handoff; creating a new checkout does not silently
rebind it. The exact profile only relocates the bindings literal/ancestor metadata
and removes the old readiness manifest/profile literals. Runtime/asset identities
are recorded from accepted evidence; fresh verification remains inside the
future execution clock.

The assembler performed no `prepare`, authoring rows, readiness worker,
runtime/asset verification, model construction, fit, forward, withheld payload
read/hash/traversal, comparison or replay. It inspected the sealed directory's
own metadata; its mode remains `000`. This is observed metadata construction,
not model-output preservation or a mathematical proof.

## Source navigation refresh

The scoped GitNexus index `uor-r4-07ec3f0d-1094-retained` was rebuilt from
`07ec3f0d39d08ac5bf9c2ba7a6b864229e007867`. Its 399 selected source files
(15,789,771 bytes) were read back against committed Git bytes. The source manifest
has SHA256 `f99d9be820d697ed5e296f3480af1bee59286631a5aa53a9dc79a4e1059f2c9b`.
GitNexus 1.6.10 indexed 395 files, 32,760 nodes and 87,004 edges, without embeddings.
The index omits oversized CLI/server files and has candidate/process/depth caps
and unresolved cross-language links; missing edges are not evidence of no
call path. Original source and history indexes remain preserved. The local
`INDEX_RECEIPT.json` has SHA256
`6133bcab279be0cf50ad0553825913f3144e471173f42c6decf3efbd44450e88`.
The graph is navigation support, not a code, proof or model-behavior verdict.

## Exact independent release and delivery boundary

The independent reviewer accepted the exact envelope and issued
[`release.json`](r4_retained_assembly_1094_evidence/release.json), status
**`ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON`**, 3,305 bytes, SHA256
`5787e4a64113800c5fc82cd1d32d564d9c6e3a344e74ca102a754fe82dccee23`.
The [appended review](r4_retained_assembly_1094_review.md) records independent
inspection of 170 coordinator files, 169 worker files and 19 historical
references, plus exact reconstruction of the profile/environment and accepted
identity records. The root coordinator then directly validated the metadata
and release binding, confirmed mode `000`, and made byte-identical public copies.
Neither reviewer nor root ran the future runtime/asset verification.

The [receipt index](r4_retained_assembly_1094_evidence/receipt-index.json) binds
all five original files to their public copies. Original assembly and release
occupy **185,873 bytes**; the future campaign starts with **3,651,274 retained
bytes** including history, before any new admission/execution/oracle receipts.
The separate public copies and index occupy 188,259 bytes. This publishing
storage is recorded separately from the immutable original campaign ledger.

This step is complete at implementation, metadata assembly and conditional
independent release. It does not complete #1094's scientific DoD. The one next
action is to separately activate the already frozen comparison and fresh-process
replay through `run-retained`, using this exact output, release, coordinator,
worker and interpreter. Fresh identity checks must run under its unchanged
execution/replay clocks before payload access and around worker attempts. No
new `prepare`, fresh budget, automatic retry, input repair, policy widening or
tolerance change is admitted. Any admitted start consumes the envelope; a stop
overrides completion. Supplied segmentation remains the qualified model entry.
#1094 and #973 stay open; #954 remains blocked. This task stops before execution.

## Storage review

The prior September 2 storage audit was reread. No source, artifact, model,
sealed input, user material, old worktree or index was deleted or unsealed.
The main checkout retains its two pre-existing deleted-file entries and all
untracked material. The coordinator worktree uses about 1.11 GiB of allocated
space; the accepted worker worktree uses about 1.11 GiB and remains required by
the envelope. The new scoped graph occupies about 0.42 GiB allocated. These
namespace sizes are not guaranteed recoverable physical space on APFS.
The existing graph's `.gitnexus` database is a regenerable candidate once its
source manifest and current replacement are preserved; it was not deleted.
Both bound worktrees, the accepted interpreter and unique research evidence
must remain available for the next action. The observed free-space snapshot
before the final graph write was about 45.42 GiB; local OS activity also changes it.
