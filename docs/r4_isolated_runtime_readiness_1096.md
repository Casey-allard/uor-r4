# Isolated runtime readiness — #1096

**2026-09-03: `ISOLATED_RUNTIME_READY` in one independently admitted attempt.**
The readiness attempt completed in 2.058100333 seconds; the actual worker
verified bound runtime and source/artifact identities, refused four harmless
probes and reported zero model loads/forwards/updates. Model behavior, comparison and replay remain `NOT_RUN`.
Independent result review passed; closure requires protected delivery.

The following preserves the pre-execution contract. This is the separate child of open #1094, based on
`df2c4cb8ef47e35b6d4083d4b8da135c7676fc19`. It does not execute the adapter
comparison, prepare another population, or release withheld inputs.

## Preserved evidence and the named decision

The original [#1094 unavailable preparation](r4_text_clause_adapter_1094.md)
remains immutable: stop SHA256
`87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5`,
binding SHA256 `baeb56a632c0e75c0574d9bad503e7849cf434039490e1fe92e3b8f0114b1f0b`.
Its authoring 320/320 and refusal 16/16 results did not qualify worker startup,
isolation, model preservation or withheld replay. The OS denied `execvp` before
Python startup. Supplied segmentation is still the qualified model entry.

**Empirical Criterion.** Can the actual comparison worker start with the fixed
Python 3.12.14 / Torch 2.7.1 CPU runtime, verify the exact source and artifact
identities, and demonstrate denied harmless file access while constructing no
model and performing zero forwards?

**Definition — candidate correction.** Keep the same source, virtual-environment
and fully resolved Python read trees and the same five exact model/codec/frame
files. Add literal read access to the interpreter symlinks hidden by `resolve()`.
Allow metadata on the exact ancestors of permitted paths, without granting
directory enumeration or other data reads there. Keep network denial and all
home-directory writes denied, including model assets. The old editable package
path stays denied; explicit `PYTHONPATH` selects the new source closure.

The static [source audit](r4_isolated_runtime_readiness_1096_sources.md) found the
omitted `cpython-3.12-macos-aarch64-none` alias and CPython's ancestor `lstat`
requirements. Neither identifies the exact cause of the previous `execvp`
denial. This single candidate tests the combined correction; success would not
separately attribute the original error to one rule.

## Frozen procedure, boundaries and budget

The implementation consists of the existing
[profile generator](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/contract.py),
the actual [worker](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/worker.py),
and the separate [readiness coordinator](../tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/readiness.py).
The coordinator never invokes `campaign.prepare`, imports a model definition,
or opens an actual corpus/reference payload. The worker receives no input rows
and takes its existing `--readiness-only` branch, skipping model deserialization,
construction and every batch forward. Reading accepted model files as bytes to
verify their identities is required; it is not a model load into an executable
model. Model state receipts remain null.

1. Freeze reviewed source in Git. Use the installed qualified interpreter only
   as a coordinator to prepare a new exclusive directory under
   `/Users/casey.allard/.codex/uor/issue-1096-readiness`. This identity-authoring
   step imports no Torch/model and makes no sandbox or readiness attempt.
2. Bind the source closure, five accepted assets, interpreter launcher/resolved
   file/symlinks, `pyvenv.cfg`, relevant installed Python/Torch files, runtime
   versions/settings, hardware, profile bytes and original stop receipt. No
   environment download, new venv or library change is permitted.
3. Create four harmless sentinel files in separate `probes/corpus`, `reference`,
   `history`, and `results` directories inside the new task output. They contain
   only a fixed non-research sentence. These are stand-ins subject to the same
   denied-home read rules, not accesses to the real sealed population or history.
   The parent verifies that each exists, is readable and has the frozen bytes.
4. An independent reviewer binds the exact manifest SHA256 in a separate review
   receipt with `APPROVED_FOR_SINGLE_READINESS_ATTEMPT`. Freeze, source review
   and resource authoring are reported separately from execution.
5. Run exactly one `/usr/bin/sandbox-exec` invocation of the actual worker,
   with `--readiness-only`, the bound manifest/hash and stdin closed. An exclusive
   `started.json` is the irreversible attempt lock; failure cannot reset it.
   The parent enforces 60 seconds across identity rechecks, launch and execution,
   killing/reaping the process group on timeout. No automatic retry is allowed.
6. Retain raw stdout/stderr, both worker events or the failure, exact command,
   bindings/profile/source identities, elapsed time, combined parent/child peak
   RSS bound, observed work counts and resource receipt. Preserve partial output
   on timeout. Hashes and raw evidence precede any later interpretation.

Hard limits: **one attempt, 60 seconds, 16 MiB new receipts/probes**, with a
1 MiB combined worker-output sublimit and the inherited conservative 3 GiB
combined peak-RSS ceiling. No model construction/load, forwards, optimizer
updates, downloads, corpus regeneration, withheld reads, replay, fitting,
accelerator/thread search or broad QA is admitted. Reuse Apple M1 / 16 GiB,
four intra-op threads, one inter-op thread, deterministic CPU/Accelerate and one
worker. The #1094 budget and its prior stop are not reset by this readiness task.

**Definition — reachability and cost.** This checks an executable/input-isolation
prerequisite, not a model metric. There are no language decisions to improve.
The source path contains zero model construction/forward calls on this branch.
Expected cost is seconds for file hashing and Torch import; the independent
60-second kill limit is binding. No timing calibration precedes this attempt.

## Admission, outcomes and next action

Require exit zero and exactly two events, `ready`/`done`. The initial event must
say `ARTIFACTS_READY`; both must bind the same source/artifact manifest and exact
profile, verified interpreter chain, installed runtime files and actual CPU
runtime settings. All four probes must raise `PermissionError`; successful
open, missing file or a different I/O error fails. Both events must show zero
model loads and row/batch forwards, initial/final model states must be null,
and the final audit must show zero optimizer updates and denied access. The
worker verifies source/asset bytes and the parent separately rechecks them
before launch. Exact evidence is retained; no tolerance or criterion changes
follow observation.

| Terminal | Action |
|---|---|
| `ISOLATED_RUNTIME_READY` | Close #1096 with its exact evidence and bindings after protected delivery; return the readiness handoff to still-open #1094 for separately frozen preparation and independent release review. No comparison executes here. |
| `UNAVAILABLE_ISOLATED_RUNTIME` | Preserve the specific failure and all original evidence; keep #1094 blocked and its comparison `NOT_RUN`. A separate repair decision is required; no revised profile or retry under this contract. |
| `INCOMPLETE_RESOURCE` | Preserve partial output/counts and stop; do not renew the budget or qualify unobserved steps. Keep #1094 comparison `NOT_RUN`. |

## Claims, delivery and storage

This is an empirical runtime-readiness check. It supplies no mathematical proof,
parser/model-output result, general sandbox-security guarantee, general language,
new-world transfer, geometry advantage or final-kernel qualification. The
[#1085 source audit](integration/clause-segmentation-1085-sources.md) remains
relevant to typed identity and ordered provenance; NEMESIS/W33/UOR research is
not evidence that this operating-system policy works. Preserve #1079
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`, #1082's descriptive findings, #973 open and
#954 blocked.

Named checks: independent static source/contract review before execution;
the single declared readiness attempt; independent result/source/digest review;
changed-Python syntax, claim wording, document/evidence-reference integrity and
`git diff --check`. Protected queue statuses are transport acknowledgements.
Broad Cargo, model, replay and certification suites stay `NOT_RUN`.

Initial storage review found about 51 GiB free. The new isolated worktree shares
Git objects and uses existing runtime/model files. No new Cargo target, runtime
installation, model download or code-graph rebuild is needed before this decision.
Preserve the mixed original checkout, original #1094 receipts and corpus,
mode-000 withheld directory, source caches, all unique evidence and user files.
No deletion is authorized by this task.

## Observed readiness and retained evidence

The [independent pre-execution review](r4_isolated_runtime_readiness_1096_preexecution_review.md)
approved source `79c674c8f6179a68878a12ee86e664f1435c3ebf` and only the second
identity freeze at `/Users/casey.allard/.codex/uor/issue-1096-readiness-freeze02`.
Its manifest SHA256 is
`4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644`;
profile SHA256 is `d508d3ec50b78e3bbd3cb1fe9468788f457f4e47f713f9a793f03079df48416e`.
The first freeze was retained unattempted after static review found unstable
asset ordering on binding reload. The source was corrected before admission.
This was not a runtime retry. The [published amendment](https://github.com/UOR-Foundation/uor-r4/issues/1096#issuecomment-5525355337)
bound the replacement and review before the sole invocation.

[Exact original receipts](r4_isolated_runtime_readiness_1096_evidence/receipt-index.json)
retain both freezes, all four harmless sentinels, the exclusive start receipt,
review, raw stdout/stderr, result and resource receipt. No model/corpus payload
is copied. The [result](r4_isolated_runtime_readiness_1096_evidence/admitted-freeze02/result.json)
has SHA256 `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930`
and CID `blake3:9e62fa112632d3fac636008000086bed0827ea933f0df49b367f86c99461f147`.
The [raw worker stream](r4_isolated_runtime_readiness_1096_evidence/admitted-freeze02/worker.stdout.jsonl)
has SHA256 `cbe9436ab15bf207c362b7863685c524ec95f8f20b2402f3294920289af6561c`;
stderr is empty. The actual command is retained in the result, including the
sandbox profile, pinned interpreter, `--readiness-only`, bindings and manifest.

| Frozen criterion | Observation |
|---|---|
| Actual worker starts and finishes | Exactly `ready` / `done`; `ARTIFACTS_READY`; exit zero |
| Manifest, profile, source and accepted assets | Exact verified identities; 169 bound source files and five assets |
| Interpreter/runtime | Both symlinks, 18 installed runtime files, Python 3.12.14 / Torch 2.7.1 verified |
| Compute settings | Apple M1 CPU/Accelerate, four intra-op / one inter-op thread, deterministic algorithms |
| Harmless corpus/reference/history/results stand-ins | All four raise `PermissionError` |
| Model states | Initial and final states null |
| Model loads / row forwards / batch forwards / optimizer updates | 0 / 0 / 0 / 0 |
| Parent elapsed / fixed ceiling | 2.058100333 seconds / 60 seconds |
| Conservative combined peak RSS / fixed ceiling | 704,806,912 bytes (672.15625 MiB) / 3 GiB |
| Worker peak RSS | 426,901,504 bytes; process receipt, not model memory |
| Withheld access, corpus preparation, fit, model comparison and replay | `NOT_RUN` |

This is measured readiness for one pinned worker, policy and machine. It is not
mathematical proof or a universal isolation guarantee. The tested denied paths
contain harmless stand-ins; real withheld contents were never opened. Success
of the combined symlink/ancestor-metadata correction does not identify which
omitted permission caused the original failure. The empirical comparison must
still establish any raw-text model-output preservation.

The original #1094 stop, binding and profile hashes remain unchanged. Its
`UNAVAILABLE_REFERENCE_REPLAY` terminal is preserved, together with #1079's
weak-control and #1082's descriptive evidence. The mode-000 withheld directory
remains sealed. No original or unique evidence, source cache, runtime, model or
user material was deleted.

## Handoff and delivery

The [independent result review](r4_isolated_runtime_readiness_1096_result_review.md)
checks these raw receipts against the frozen criterion without replay. Only
changed-Python syntax, claim wording, evidence/document references and diff
whitespace supplement the declared readiness decision. No broad QA is activated.
Protected queue statuses establish transport only.

After protected delivery, close #1096's readiness scope. The one next action is
for **#1094** to freeze a separate preparation contract using these exact
source/profile/runtime bindings, account for its prior consumed budget and
obtain independent release review before any withheld access or comparison.
No preparation or release is authorized by this readiness result. #1094 remains
open and parked/unassigned until active; #973 remains open and #954 blocked.

Final local storage review counted 165,740 bytes in the two original freeze
folders and 173,274 bytes in the public evidence copy/index: 339,014 bytes
combined, including the superseded authoring freeze. The source-audit,
contract and independent-review documents are small additional delivery files;
the complete retained task record remains below 1 MiB and the 16 MiB cap.
Available storage was 51,987,124 KiB (about 49.6 GiB). The isolated checkout
is retained for exact-source review; its Git objects are shared. No new build
cache, model installation or download was created, and nothing was deleted.
