# #1039 — P0: expose frozen #1017 generator as a bounded loopback reference surface

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T01:27:23Z
- **Updated:** 2026-09-02T02:09:51Z
- **Closed:** 2026-09-02T02:09:31Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1039
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Turn the already-qualified frozen #1017 checkpoint into the smallest useful CPU-native local reference surface beyond the existing raw r4 generate CLI.

This is a bounded, source-backed product/reference lane for #973 and programme root #820. It does **not** implement or close the source-free route-native chat stage #962 and does not unblock #954.

## First decision gate

Run one real fresh-prompt generation check from current main using the immutable local #1017 export and the opt-in Apple Accelerate CPU backend.

- If output is valid and useful enough to exercise: add a loopback-only, single-flight HTTP generation seam with honest readiness/provenance.
- If a concrete CLI, artifact-discovery, or sampling defect appears: repair only that defect first; do not add the HTTP surface until the same check passes.

## Run contract

    metric to move:       usable local access to the frozen #1017 generator beyond a process-local CLI
    reachability ceiling: 100% of this reference lane already reaches the #1017 generator; no model-training or geometry mechanism is changed
    instrument + verdict: one fresh narrative prompt, bounded output, nonempty valid UTF-8, no immediate short cycle, exact fixed-seed replay
    exit rule:            proceed only when the existing generator passes that bounded behavior check
    if positive:          ship the loopback-only source-backed HTTP reference seam
    if negative:          repair the demonstrated CLI/model-discovery/sampling defect, then repeat only this check
    cost estimate:        one release build plus an 8-32 token CPU smoke; no corpus-scale run

## Definition of done

- [ ] Use the exact existing #1017 export; record its manifest/tree/weights/tokenizer identities without modifying it.
- [ ] Build CPU-native release code; on Apple Silicon use the existing opt-in Accelerate path. No CUDA or external GPU path.
- [ ] Exercise one small, predeclared fresh narrative prompt and record JSON/audit output.
- [ ] Require nonempty valid UTF-8, no immediate short cycle, and exact fixed-seed replay.
- [ ] On a positive gate, expose the same generator through a loopback-only, single-flight HTTP endpoint with bounded input/output and fail-closed model readiness.
- [ ] Label the endpoint/output explicitly as the source-backed #1017 bounded reference model.
- [ ] Reuse existing generation code; do not introduce a second inference implementation.
- [ ] Add only focused tests that change this delivery decision.
- [ ] Document the local-export prerequisite, Apple Accelerate option, API example, and limitations.
- [ ] Open a PR and deliver through the protected merge queue.

## Nonclaims

This issue does not establish geometry advantage, transformerlessness, source-free/table-native/multiply-free execution, broad coherence, correctness, reasoning, instruction following, chat quality, browser/WASM readiness, or frontier/release quality.


## Comments (5)

### comment 5502977836 — 2026-09-02T01:30:34Z — Casey-allard

## Pre-run freeze

Fresh prompt (not drawn from the #1017 five-prompt qualification set):

> Mira found a small brass key beneath the old pear tree.

Bounded run: 24 new tokens, stable seed 1039, attention enabled, four CPU workers, Apple Accelerate release backend. The exact invocation will be repeated twice. Positive requires successful execution, nonempty valid UTF-8, no immediate short cycle, exact output/token/audit replay, all-layer attention audit enabled, and zero Ollama/provider reads.


### comment 5502985425 — 2026-09-02T01:31:33Z — Casey-allard

## First decision gate: POSITIVE

Current-main Apple Accelerate release build completed in 2m18s using all available Cargo workers. The immutable local export was passed explicitly.

Prompt:

> Mira found a small brass key beneath the old pear tree.

Output (24 tokens, seed 1039):

> She picked it up and said, “I believe it can be a big tree!”
> Mira and her mommy

Measured wall times were 0.22s and 0.17s. The two complete JSON reports were byte-identical after removing timing.

Evidence:

- decision CID: blake3:86ce45d07684f8abead1e4faca2346024ef7b01e99f9b0e3b51188deafcde61b
- output CID: blake3:87150ebc68ed1e3902f6e0c9937f7a642f234ac2a3644cc33262cb081715ae49
- audit CID: blake3:3e7d642eb20c9e1d05c385df2802d120becb91ba88df16a9b61cf2569caec010
- persistent-state CID: blake3:ff69922ed27437e308852b053f3b3b15baebeb1e5a51c27c704f18f1ba423793
- weights CID: blake3:c5bf31aa97a567b3aaad4461ce2fac9cebc12b0a38becb6d02d21b43b493bf5d
- tokenizer CID: blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc
- valid UTF-8: true
- short cycle: none
- all six layers selected: true
- causal/projection/R4/policy audits exact: true
- future reads: 0
- provider calls: 0
- Ollama calls: 0
- selected backend: Apple Accelerate
- effective workers: 4

Binding action: implement the dedicated loopback-only, single-flight #1017 HTTP reference seam. No model, sampler, training, geometry, or #962 work is authorized by this result.


### comment 5503133505 — 2026-09-02T01:50:45Z — Casey-allard

## Implemented endpoint qualification: PASS

The #1039 release binary was rebuilt with the Apple Accelerate CPU feature and started on loopback against the immutable #1017 export.

GET /api/sysinfo reported:

- enabled: true
- checkpoint_preflight_ready: true
- attention_on: true
- greedy: true
- workers: 4
- default/maximum tokens: 8/32
- static_wasm: false

A real POST to /uor/v1/r4-softmax-local/generate returned HTTP 200 in 0.16s wall time with the eight-token continuation:

> She was so excited to see what it

Evidence:

- decision CID: blake3:87d8e9689132584a1437a244bab4e97e03af9d2609905ae6c74db0fa792c2522
- output CID: blake3:aef50423770f79714247e2e496d7f49381423893cbb8c636a31276ff9a5aa885
- audit CID: blake3:6ba5f8d8073748f57692e92a10291d507ef6963251e42069fa42256f56072ffc
- persistent-state CID: blake3:0a837266df1c81d2cc2d54e32c41d6b4e6f0476dfe2bcf04103576278e995390
- loader checkpoint-tree CID: blake3:66ee347b23e818f1816682f0b942737c88f1eca831cd6d4f00b3d14fc00aaa37
- weights/tokenizer identities unchanged
- selected backend: Apple Accelerate
- causal/projection/R4/output-policy/all-layer audits: exact
- future/provider/Ollama reads: zero
- generated UTF-8: valid; short cycle: none
- requested/effective workers: 4/4

A second HTTP request was report-identical after removing timing. The compact response contained neither the prompt nor prompt-token/model-path/file fields.

Focused verification passed: 4 endpoint tests, 1 CLI ownership/default test, 1 static/WASM guard, release Accelerate build, formatting, claim wording, JavaScript syntax, and diff whitespace. The full integration ladder remains delegated to the protected merge queue per repository policy.


### comment 5503255538 — 2026-09-02T02:06:32Z — Casey-allard

## Independent review and post-hardening rerun

The pre-commit review found and resolved two blocking boundary defects:

1. An uncommitted model.safetensors.index.json could have switched the loader to alternate shards while the frozen single-file artifact remained present. Preflight now admits exactly the five frozen export files, rejects every additional loader input, and the successful generation report must reproduce the frozen loader-tree/config/tokenizer/weights identities before HTTP 200.
2. The process-wide reservation subject used the private checkpoint path, which a conflicting legacy route could have echoed. The subject is now non-sensitive, with a cross-operation path-scrub assertion.

Worker language was also corrected: the startup flag bounds exact-executor workers; Apple Accelerate owns its own internal CPU scheduling.

After these fixes, the Accelerate release server repeated the real endpoint check successfully. It returned the same text and decision/output/audit/state CIDs in 0.20s wall time, all attention/decode/source-read predicates remained true, and the response remained prompt/path/file-field scrubbed. Independent re-review found no remaining blocking issues.


### comment 5503283076 — 2026-09-02T02:09:51Z — Casey-allard

## Protected delivery complete

PR #1040 merged through the protected queue as c6e3d95ee39fbb4797b0c8d51507eb5fef978a4a.

The merge-group CI run completed successfully, including the real queue-side fmt/clippy/tests/no_std/kappa ladder, cargo audit, fuzz smoke, WASM build, and Gate C trend check.

Definition of done is complete:

- exact immutable #1017 export and identities preserved
- CPU-native Apple Accelerate behavior run and replay recorded
- dedicated loopback-only single-flight endpoint delivered
- strict request bounds and source-backed/raw-completion labeling delivered
- native dashboard gated on checkpoint readiness
- static/WASM and chat surfaces remain fail-closed/separate
- exact five-file loader input and post-generation identity enforcement delivered
- prompt/path/file response scrubbing and cross-route reservation scrubbing delivered
- documentation and nonclaims updated
- no CUDA, retraining, geometry ladder, #954 unblock, or #962 completion claimed

Issue closes completed.



---
_Comment/review ids in this file: 5502977836, 5502985425, 5503133505, 5503255538, 5503283076_

