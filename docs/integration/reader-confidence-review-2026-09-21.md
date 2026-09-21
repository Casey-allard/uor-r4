# Principal review: retain confidence access, advance read-conditioned generation

September 21, 2026. Review of PR #1333 submitted at `51470ea99fb32ba7060e2657c1b661e95fd7844d`, on PR #1332 merge `e0d6296c9d787874ade8d2b705ccf6da85d6cf4b`. Independent source, evidence and mathematical reviews were combined with live resource/knowledge recovery. [Independent audit](../evidence/reader-confidence-principal-review-2026-09-21.json); [next execution prompt](deepseek-read-conditioned-state-step-2026-09-21.md).

## Decision and retained result

**Retain the confidence interface as a useful component. End automatic scalar-copy feature expansion. Next, close the missing frozen-artifact generation boundary and implement one learned read-conditioned geometric state update feeding shared emission.** The copied-token boost cannot perform that operation, irrespective of how accurately its dose is selected. This advances the existing attention/composition roadmap while keeping current negatives visible.

The extra sign bit makes a construction-preserving policy expressible and a feasible fit was measured. H4 fit selects 148 correct answers and 7 absent reads, categorical 158 and 5, with text deltas +0.0228/+0.0036 bits/token. The old 32-address class was independently infeasible. These are useful gains in the fitted policy class, not an integrated language qualification. The original fitted tables/artifacts are preserved even though final acceptance fails.

## Correct the outcome before choosing a mechanism

The original summary compares categorical absence to the H4 parent and emphasizes emitted counts while omitting present CE. Each arm must be compared to the parent used in its own fitting constraints.

| New exposed acceptance population | H4 confidence | Categorical confidence |
| --- | ---: | ---: |
| Present emitted correct, own parent | 71/117, parent 73 | 79/117, parent 81 |
| Present loss increase over own parent | **+0.159790 bits/query** | **+0.152981 bits/query** |
| Allowed present loss increase | +0.05 | +0.05 |
| Absent reads, own parent | **11/23, parent 10** | **7/23, parent 6** |
| Text loss delta versus local | +0.037882 | +0.020116 |
| Tune text delta versus local | **+0.061316** | **+0.061381** |

Both meet the fresh emitted-count tolerance and text harm screen, but both fail own-parent present-loss and absent-read preservation. Both tune text values exceed the +0.05 screen. A one-read miss can be a useful near miss rather than grounds for discarding a mechanism; it is not the only miss here. Preserve exact practical criteria and report all outcomes before proposing prospective changes.

On the whole fresh construction stream, H4 loses +0.837488 bits/position relative to its parent, with correct emissions 178 versus 364; categorical loses +0.833452 with 184 versus 378. Those broader populations were not the declared narrow present-query criterion, but are material retained behavior, not interchangeable with the final-query counts. Do not call full relational behavior preserved.

The parent comparison table must use the same population: H4 parent-rule development text is +0.5724, not the old held-out +0.4532 figure. Exact fit/tune/final quantities and arm identities are in the audit. The saved interval panel concerns old arms/populations. The independent audit additionally computes a post-hoc empirical document bootstrap over all 256 ordered resamples of four documents: H4 +0.037882 [+0.010568,+0.074770], categorical +0.020116 [+0.002479,+0.043939] bits/token. These are descriptive small-sample calculations, not new preregistered acceptance evidence or broad-population precision.

## What the controls actually establish

The recorded witness compares the scored parent's selected index/strength with a hand-built D-sign/eight-nat rule at 2,969 nonempty construction-fit positions. It reports zero mismatches for each arm. Empty pools are skipped; this is not an independently reloaded `predict_next`/integer-logit/generation parity result, nor does the count prove that all tie cases were exercised. The principal repair makes witness failure and required confidence-loader failures stop execution; two focused boundary tests, fmt and the offline touched-runner check pass; they can no longer silently fall through to a local baseline.

The confidence artifacts are evaluated in fresh teacher-forced panels. Their generated responses, changed-source/read-disabled interventions and timings are **NOT_RUN**. The corresponding saved panels run older policy arms. Their repaired neighbor accounting and control successes retain that old-operator scope. The next substantive run must execute the actual confidence artifacts through the shared target-free inference path before assigning them behavior claims.

All three declared source hashes now match submitted source, including `policy_feasibility.rs`, and the actual base checkout is recorded. This repairs the preceding source-correspondence problem for the current submission. All three attempts repeat the same selected tables, populations, panel values and generated outputs. The four remaining Dev documents and the new construction population were first exposed in attempt 1; attempts 2/3 are repaired replays of that one draw, not additional independent evidence. The local prior's exposure is distinct from reader-held-out status.

The new confidence input digest still omits relevant text-fit inputs, sequence/occurrence boundaries and causal dependencies. A loader that checks that digest is real byte/config integrity, but not complete fit-input binding. The 64-address fit/tune/action arrays and per-position traces are not retained. This prevents independent reconstruction of the optimizer's complete objective/constraints or a causal attribution of all text harm to particular addresses. Report claimed exhaustive optimum at its implemented support/fallback problem scope; no universal 64-state impossibility follows.

## Why a new stream-role bit is not the selected successor

Natural-text target availability is only 37/976 positions in the new text panel. H4 contributes +49.1178 loss bits at the 939 positions where the next target is absent from admitted payloads, versus a gain of 12.1449 bits at the 37 available positions. Thirty-seven of 38 eight-nat reads occur in the unavailable stratum. This locates a copy-access limitation; saved dose/address data are insufficient to establish the exclusive cause as eight-nat stream mixing or prove which additional causal feature would solve it.

The selected tables also permit some one-nat reads when D<=0. Restoring the parent's NoRead veto is a useful frozen diagnostic and guarantees a subset of parent reads on identical prefixes; it does not guarantee text, present CE or rollout improvement. A stream/domain flag obtained from the evaluator would leak supervision. A genuinely learned contextual role feature remains possible, but the current evidence does not select it uniquely, and another bit does not address the expressivity limit below.

## A sharper mathematical bottleneck and a constructive bridge

The served operator in `competitive-reader::predict_next` changes exactly one selected payload logit:

`z'_c = z_c + a`, with every other `z'_v = z_v`.

With a nonnegative effective boost, for two tokens v,w distinct from c,

`p'(v)/p'(w) = p(v)/p(w)`.

If the actual next token y differs from c, its loss changes by

`delta CE = log(1 + p(c)*(exp(a)-1)) >= 0`, strictly positive for a>0 and p(c)>0.

Integer saturation can reduce a to zero but cannot make this operation favor a different uncopied token. If the target is absent from every admitted payload, any positive single-copy boost harms its teacher-forced CE. Better scalar gating can abstain, but cannot use a retrieved subject to choose among uncopied verbs or use selected evidence to derive a new output. This is an operator expressivity fact, not an explanation of every measured regression or a promise that a replacement will learn.

The smallest relevant new operation is therefore **Read -> learned state update -> shared emission**, initially one read. A finite sketch is `h' = U(h, directed_relation, learned_payload_code)` with a shared bounded residual `R(h') - R(h_NoRead)` affecting more than the copied payload logit. NoRead must preserve the exact local baseline. Avoid a joint identity-transport/flat-readout initialization with no learning signal; verify a learnable escape once. Keep a separately learned bounded value-code map available if frozen selector codes erase a needed distinction, and do not call an isomorphic relabeling of H4 a nongeometric control. Learn the update and, if necessary, one small shared low-bit output residual under final-token/output loss. Do not add a vocabulary-sized expert per source or a hidden dense transformer.

Concrete donors are `learner/query_read.rs` (`row_scores`, `residual_scores` and the shared low-bit emitter), `dependent_attention/runtime.rs::update_query`, and the retained shared Read/Emit/Stop/owned-reference mechanisms in the mechanism synthesis. Their existence does not establish transfer to this BPE artifact. Inspect actual types, arithmetic, state capacity and parameter traffic before reuse. `QueryHard` row materialization can touch a full vocabulary; count startup work, cached-row bytes and per-token accesses explicitly.

Use a controlled noncopy target whose answer is absent from all source payloads, plus matched recent suffixes with changed older evidence, and require actual changed-source/ReadDisabled/update-disabled effects. Ordinary text controls remain essential. Only after this single read changes appropriate nonpayload predictions should we add a learned dependent-read schedule. This sequence moves beyond copy calibration without demanding perfect text gating as a prerequisite.

## Research connections and broader sequence

[Pointer Sentinel Mixture Models](https://arxiv.org/abs/1609.07843) explicitly separate reproducing a context word from general generation. [CoCoLex, ACL 2025](https://aclanthology.org/2025.acl-long.931/) uses confidence to mix copying and generation, useful as an influence comparison, but its dense decoder does not supply our missing geometric update. [Fast Weight Attention for Continual Learning, August 2026](https://arxiv.org/abs/2608.27763) distinguishes temporal alignment, plasticity and forgetting in recurrent memory; causal read/write alignment is a transferable design question, while its matrix-valued update is not adopted as a multiplier-free kernel. These sources guide a concrete interface distinction, not a claim of transferred quality.

The roadmap now prioritizes actual frozen-reader rollout and one read-conditioned geometric emission update; then dependent attention/derived composition, contextual structural persistence where older-scope aliases require it, broader prose/conversation/executed Rust, and quality-matched whole-path cost/energy. Persistence is a supporting responsibility, not a compulsory stage before every composition experiment. Signed H4/Spin, exact occurrence/version ownership, retained Hopf fiber and typed paired-H4/icosian operators remain primary reusable tools. S7/E8/harmonic coefficient banks remain conditional on a witnessed capacity/interference need with equal-bit controls. No new dimensional or physics mechanism is required to change nonpayload odds.

## Resources and delivery scope

The live JSON had not charged this run or applied its reported extension. Its reported components sum to 3739800 ms, not 3440000 ms; the principal reconciliation retains the full components and 2000000 ms allowance increment, yielding 199623905/200900000 ms before principal checks. Initial compilation preceded the recorded projection, which preceded main inference; do not call the entire run prospectively accounted. The subsequent reserve breach is recorded, not normalized by a paper allowance.

Read-only inventory found 35.42 GB free. Removing only 32 inactive incremental-cache directories within the older geometric-query-read worktree recovered 4.358 GB of immediately observed free space, reaching 39.78 GB. `du` measured 5.802 GB allocated, distinct from actual free gain because of filesystem accounting/sharing. The worktree, source, compiled executables/dependencies, models, all sealed evidence/research and downloads remain. The reserve 36766079385 bytes plus 128 MiB stop margin is unchanged. The ledger records the bounded principal repair checks before execution and their measured charges afterward.

This review completes PR #1333 through protected delivery, corrects the active docs and six owning issues, and supplies a revision-pinned knowledge handoff. Original reports are retained unchanged. New model rollout and the read-conditioned state experiment remain next-run work; physical energy and whole-path serving qualification remain UNAVAILABLE.
