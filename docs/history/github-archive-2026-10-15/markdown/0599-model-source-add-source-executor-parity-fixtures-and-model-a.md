# #599 — model-source: add source-executor parity fixtures and model-adapter conformance gates

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-10T21:58:08Z
- **Updated:** 2026-08-12T09:20:54Z
- **Closed:** 2026-08-12T09:20:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/599
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

The repository has a `TeacherOracle` boundary and a runtime-vs-teacher benchmark, but open-weight compilation still lacks a source-executor conformance suite that proves a model adapter is interpreting its own tokenizer/configuration/tensors correctly before observations are trusted.

# Current evidence

- `crates/uor-r4-model-source/src/lib.rs:802-840` exposes only the two-surface `TeacherOracle` contract plus optional final hidden-state and top-k hooks.
- `crates/uor-r4-core/tests/smollm2_adapter.rs` has ignored smoke tests for loading the pinned source and checking finite embeddings/logits; it does not compare reference tokenizer/forward outputs or per-layer behavior.
- `tests/bdd.rs` and merged PR #307 cover teacher-forced parity of deployed legacy/R4G1 artifacts, not source-adapter conformance.
- The source loader currently assumes Llama-family tensor names and configuration fields; `HuggingFaceLlamaOracle` is the only HF adapter and `SmolLm2Oracle` is an alias.
- Tokenizer correctness was materially fixed by #242/#253, and chat-template work landed through #202/#207; the remaining need is a reusable adapter gate that catches source/config mismatches before compilation.

# Problem

An adapter can load and compile while silently misinterpreting tokenizer policy, RoPE/norm/activation/GQA/bias configuration, or tensor layout. Existing smoke and runtime-parity tests do not localize that failure at the source boundary.

# Goal

Add pinned, source-side conformance fixtures and an adapter declaration that make supported and rejected model features explicit.

# Scope

- Define a conformance fixture format containing token sequences, tokenizer IDs/bytes, selected layer outputs, final hidden state, logits, and top-k results.
- Compare the Rust source executor against a pinned reference execution for the currently supported Llama-family fixture.
- Record checks for norm epsilon, RoPE mode/theta, activation, GQA/MQA dimensions, QKV/output biases, tied embeddings, BOS/EOS, and chat-template policy.
- Require each adapter to declare supported/rejected configuration features and fail closed on unsupported combinations.
- Keep source parity separate from compiled-runtime quality; feed its CID into later observation provenance.

# Non-goals

- Do not add a new model family in this issue.
- Do not change the runtime operation contract or claim exact transformer equivalence.
- Do not use the existing vacuous-skip parity convention as the sole acceptance path; missing pinned source fixtures must be reported as unavailable evidence.

# Dependencies and blockers

- Depends on #597 for full source identity and #598 for complete tensor/dtype ingestion.
- It should reuse, not replace, the runtime parity surfaces from #307.
- #515 remains the owner of dormant mechanism gating; #531 owns corpus-scale quality saturation; #589 blocks upstream integration.

# Proposed implementation shape

Add host-side adapter metadata and deterministic fixture runners beside `uor-r4-model-source`; expose only stable, architecture-neutral results to the compiler. Keep per-layer/QKV capture compiler-side and bounded.

# Acceptance criteria

- [ ] A pinned Llama-family fixture passes tokenizer, selected-layer, final-hidden, logits, and top-k comparisons with documented tolerances and source CIDs.
- [ ] Every currently assumed configuration feature has a positive fixture or an explicit rejection test.
- [ ] Unsupported bias/GQA/activation/RoPE/tie configurations fail before observation generation.
- [ ] A failed or absent conformance fixture is distinguishable from a passing result in reports.
- [ ] The fixture runner is deterministic and records adapter/compiler/source/tokenizer identities.

# Verification

- Run the fixture runner twice and compare canonical JSON/byte outputs.
- Run the real pinned SmolLM2 adapter test when the source snapshot is present; otherwise report the missing prerequisite explicitly.
- Run `cargo test --workspace --offline`, the full clippy/fmt gates, and `python3 scripts/check_claim_wording.py`.

# Compatibility and migration

This adds host-side test/report schemas only. Existing artifacts remain valid; any changed source execution or tokenizer identity must produce a new source/observation CID and an era-noted recompile rather than mutating historical fixtures.

# Claim status

Definition, implementation, empirical criterion, and proof-obligation inventory. Results are measurements, not proofs of equivalence.

# Experimental run contract

metric to move: source-reference agreement for tokenizer IDs, selected-layer outputs, final hidden state, logits, and top-k.
current baseline: no source-reference parity numbers are recorded; the current fixture is an ignored finite-output smoke test.
reachability ceiling: awaiting the pinned reference fixture; no baseline number invented.
cheap instrument and required verdict: short deterministic fixture with per-field max error/top-k agreement; any unsupported feature must reject explicitly.
exit rule: stop when all declared supported features have passing fixtures and all undeclared features fail closed.
if positive: use the conformance result as the prerequisite for observation generation and new-family adapters.
if negative: fix or reject the adapter before collecting traces; do not fit a target operator from contaminated observations.
cost estimate: small compiler-side fixture implementation plus one short source execution per supported configuration.
pinned source CID: supplied by #597.
pinned corpus CID: not applicable to source conformance.
target operator ID: not applicable; this is source execution.


## Comments (3)

### comment 5262469894 — 2026-08-12T04:58:24Z — Casey-allard

## Plan (queued behind #598's merge — PR #634; implementation starts when it lands, since the conformance gate sits on that ingestion boundary)

1. **Adapter feature declaration, fail-closed**: a typed `AdapterFeatures` declaration beside the oracle (norm epsilon, RoPE mode/theta, activation, GQA/MQA head geometry, QKV/output biases, tied embeddings, BOS/EOS, chat-template policy). `config.json` values outside the declared support set fail BEFORE observation generation with a focused error naming the feature — no silent misinterpretation.
2. **Conformance fixture format** (schema-versioned canonical JSON): token sequences + tokenizer IDs/bytes, selected layer outputs, final hidden state, logits, top-k, with documented tolerances and the #597 source CID + adapter/compiler/tokenizer identities bound in.
3. **Deterministic fixture runner** host-side beside model-source: runs the source executor against a pinned reference fixture; canonical output bytes (double-run identical); result is a three-state report — PASS / FAIL(with per-check deltas) / **UNAVAILABLE(missing pinned fixture — explicitly NOT the vacuous-skip convention**, per the issue's non-goal).
4. **Positive-or-rejection coverage** for every currently-assumed Llama-family config feature; unsupported combinations get explicit rejection tests.
5. Reuse #307's runtime-parity surfaces untouched; source parity stays separate from compiled-runtime quality. Per-layer/QKV capture stays compiler-side and bounded.

Real-weights caveat up front (honesty): the pinned 257MiB SmolLM2 snapshot is absent from the build environment, so the pinned-fixture arm will run when the snapshot is present and report UNAVAILABLE otherwise — the runner and its determinism are fully testable with synthetic fixtures either way. Gates + full suite + claim-wording before the PR.

### comment 5264430186 — 2026-08-12T08:43:01Z — Casey-allard

## Delivery record (PR #635 in queue)

All five acceptance criteria addressed: **(1)** fixture format + runner delivered (`uor-r4-adapter-fixture/1`, canonical JSON; runner at conformance.rs:954 with per-check tolerances and identity block binding the #597 manifest + adapter/compiler/tokenizer identities); the real pinned-SmolLM2 PASS arm is necessarily unexecuted in the build env (snapshot absent) and is asserted **UNAVAILABLE by a non-ignored test** — never vacuous-skip — with the `#[ignore]`d real arm wired for when the snapshot is present. **(2)** Every assumed config feature enumerated with positive-or-rejection coverage — 12 rows in the delivery table, including a genuine catch: **the executor silently ignored `rms_norm_eps` and hardcoded 1e-5; now any undeclared epsilon fails closed** (the exact silent-misinterpretation class this issue exists for). **(3)** Unsupported bias/GQA/activation/RoPE/tie configs fail at oracle construction — before typed parse, before snapshot ingestion, before observation (proven by `rejection_happens_before_snapshot_ingestion`). **(4)** PASS / FAIL(per-check deltas) / UNAVAILABLE(named prerequisite) are distinct report states. **(5)** Runner deterministic (double-run byte-identical test) and records adapter/compiler/source/tokenizer identities.

R5-conformant (`UnsupportedConfigFeature{AdapterFeature,…}` on the sanctioned SourceUnavailable); wasm per-item cfg gates; `Llama::forward` factoring preserved arithmetic order (bit-exact batch parity test green); #307 surfaces + runtime + deps untouched; all six gates green. Honest note: the fixture binds the manifest's canonical **file bytes** κ (model-source can't compute the uor-addr root κ without a new dep — documented on FixtureIdentity). Closes on #635 merge; #600 next.

### comment 5264809950 — 2026-08-12T09:20:53Z — Casey-allard

PR #635 merged — delivered per the record (comment 5264430186), including the rms_norm_eps fail-closed catch. Closing; #600 begins now.


---
_Comment/review ids in this file: 5262469894, 5264430186, 5264809950_

