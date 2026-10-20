# Shared learned transitions: original result and principal corrections

September 21, 2026. Original PR #1339 submitted head `832c1c33`, based on merged PR #1338 `86623138`. Original isolated checkout `.worktrees/shared-transition`, preserved with all report roots. This record retains the original numerical results with their independently reviewed scope. Read the [principal review](shared-transition-review-2026-09-21.md), [saved-data audit](../evidence/shared-transition-principal-review-2026-09-21.json), [corrected executed checks](../evidence/shared-transition-principal-checks-2026-09-21.json), and [next complete brief](deepseek-grounded-dependent-session-step-2026-09-21.md).

## Implemented mechanism

`s0 = E[selected payload]; s_j = A[observed primitive_j] * s_{j-1}; output_j = D(s_j)`.

One learned action per primitive is reused at every occurrence. The result variable persists inside one serving call; emitted tokens are not fed back as new instructions. Learned maps and decoder are exported, independently reloaded and used. The supervised remaining-indexed stop table encodes explicit input exhaustion. This is supplied-program completion, not learned semantic continuation or answer length. There is no externally resumable result frame, durable source lease or dependent second read yet.

## Original numerical results, retained

| Arm | Development /288 | Length four /128 | Reversal /64 |
| --- | ---: | ---: | ---: |
| Local/NoRead ablation | 0 | 0 | 0 |
| Value-only lexical ablation | 4 | 0 | 0 |
| Whole-program cache, unseen fallback one token | 288 | 0 | 0 |
| Additive C120, same fitting method | 153 | 24 | 15 |
| H4 shared transitions | 186 | 48 | 10 |

H4 saved events independently reconstruct all three complete-response counts, including 556/672 development token hits. Length four contains 32 programs × four values, with the first two primitive tokens fixed by the enumeration stride. Four showcased generations are four values on one program; three complete correctly. The fit/probe partition is value-index parity with all 72 development programs in both halves, and both halves guide optimization. Final decoder fitting uses all development observations. These are now exposed regression panels.

The whole-program cache cannot complete an unseen length-four program by construction. Therefore H4's win over that cache does not establish an advantage over ordinary shared transition lookup. Development labels supply every needed initial and recurrent transition; the principal correction adds a competent finite recurrent table using the same supervision and its own previous predicted outcome at serving. Its cost and artifact size differ from H4 and must be reported.

## Corrections to the original causal interpretation

All 64 reversal cases select the correct source. H4's 10/64 reversal result is consequently a learning/computation limitation in that population; reader confounding is not the observed explanation. Length four selects correctly in 124/128, including all 48 complete successes. Persistent evidence ownership still matters for future multi-call and dependent-read behavior, but cannot by itself repair the current arithmetic failures.

The original payload intervention bypassed actual selection; the supposed absence test disabled the reader without removing the source; state independence compared identical calls. The noncommuting witness showed different final states without checking both expected answers. Additive final-state invariance does not imply identical intermediate output sequences. Those reported controls are preserved as historical fields in the evidence JSON, not promoted as causal checks.

The corrected Rust path derives instructions from the actual declared prefix span, eliminates gold-operand fallback, logs actual all-arm response/source/terminal events, independently reloads a shared finite transition comparator and performs actual changed-source/source-removal/decoder-perturbation controls. RLST v2 distinguishes `Exhausted` from a policy decision; v1 retains its historical behavior. A supplied instruction span is not learned parsing. The [corrected checks receipt](../evidence/shared-transition-principal-checks-2026-09-21.json) owns the executed replay and remaining limitations.

## Corrected exposed replay

**Corrected exposed replay:** H4 remains **186/288, 48/128 and 10/64**; the independently loaded finite shared-transition table reaches **288/288, 124/128 and 64/64** on the same development/length-four/reversal populations. The table uses its own preceding prediction at serving, not gold intermediate answers. Actual payload and order interventions produce correct changed responses; decoder-label perturbation changes emissions without changing states. The noncommuting latent witness still misses the reversed final answer. All 2,880 actual arm/item records and 8,316 step/terminal events are saved. This is a retained partial geometric component, not a superiority or general-continuation result.

The [independent corrected audit](../evidence/shared-transition-corrected-replay-audit-2026-09-21.json) owns source/artifact/seal and event reconstruction. Runtime is 49.754 seconds for the debug experiment, not optimized serving performance.

## What the evidence directs next

The [development-only mathematical audit](../evidence/shared-transition-observed-action-audit-2026-09-21.json) identifies the eight observed primitive permutations as a regular Q8 action. This supports constructive learning from the observed transition graph into a verified quaternion subgroup, with consistent typed outcome and initial-state grounding. It does not establish that a new model has learned those coordinates yet.

Advance that grounding inside a complete owned read → compute → dependent read → answer/stop session. Keep source occurrence/version/payload separate from compressed result geometry, retain state across calls and snapshot/resume, and make the first result causally change the next query and selected evidence. All original roots `shared-transition-{1..6}` remain sealed and preserved. No general language, fresh-final qualification, whole-path D0-b or physical energy advantage is established.
