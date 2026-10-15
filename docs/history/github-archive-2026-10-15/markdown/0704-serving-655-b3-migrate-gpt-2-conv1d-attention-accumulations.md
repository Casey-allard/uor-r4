# #704 — serving/#655-B3: migrate GPT-2 conv1d + attention accumulations to uor-matmul

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T13:14:28Z
- **Updated:** 2026-08-16T08:08:06Z
- **Closed:** 2026-08-16T08:07:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/704
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Remaining hand-rolled f32 matrix ops after #655-B2 (which migrated the Llama teacher weight matmuls). Not library-BLAS, so the census guard does not flag them; tracked in docs/matrix_operation_census.md.

## Scope
- `uor-r4-model-source/src/gpt2.rs`: `conv1d` / `conv1d_batched` (GPT-2 Conv1D `x@W`) -> `uor_matmul::slice::gemm_float` (mind the Conv1D `[in,out]` orientation vs the Llama `[out,in]` layout when shaping the GEMM).
- Attention scoring accumulations (`Q·K` per-head dot, `·V`) in both the Llama (`lib.rs`) and GPT-2 (`gpt2.rs`) executors -> exact accumulation via uor-matmul where it maps cleanly (these are small per-head dots, not weight GEMMs, so evaluate whether gemm_float or a dot entry fits).

## Discipline (same as B2)
Dry-run first (wire + run the full κ-sensitive suite to confirm zero fixture breakage + acceptable speed), correctly-rounded exact parity within numpy tolerance, teacher-arithmetic era recorded by the teacher κ (no fixture re-pin expected), full gates, PR assigned @me + auto-merge. After this, the census doc's 'conventional-to-migrate' section is empty and the teacher is fully uor-matmul-owned.

## Comments (26)

### comment 5298001556 — 2026-08-14T20:44:50Z — Casey-allard

## B3 dry-run verdict — blocked on the acceptable-speed gate (2026-08-14)

I wired the full #704 migration and ran the correctness/provenance/κ-sensitive dry-run before opening a PR. Correctness is green, but the real GPT-2 performance result is a hard negative, so I am **not** submitting or queueing the current implementation.

### Corrected provenance premise

The source teacher κ is a hash of the pinned snapshot bytes; it does **not** encode the arithmetic implementation. It therefore remains unchanged across B2/B3. The arithmetic era must instead be carried by the registry-validated attention-operator record plus the derived corpus/artifact identities. The dry-run branch implements explicit immutable v1/v2 operator records and fail-closed resume/sidecar handling rather than relabeling old rows.

Non-vacuous evidence:

- Real GPT-2 124M numpy canary: all 3 cases / 11 tokens pass; worst hidden delta `7.6293945e-5`.
- Live SmolLM2-135M serial-vs-batched parity: all 49,152 logits are bit-identical; source κ remains `blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5`.
- Gate E with `/tmp/ref/out/model.bin`: 3/3 pass; source κ remains `blake3:0ae7339518cb124bb9a8fcef88e5dfb615d9e56c55a118e55817dd077c77455d`; derived container κ remains `blake3:8fbf3f68b71a0615deacac9b9ab6e6b51de0765689f27c59abb44565881054d9`; no fixture re-pin is needed.
- Focused current-tree suites: graph-compiler 104/104, graph-CLI 45/45, server façade 60/60, plus claim-wording and diff checks.

### Performance result

Apple M1, release build, warm local source:

| Measurement | `origin/main` | B3 dry-run | Delta |
|---|---:|---:|---:|
| Real GPT-2 canary, 11 tokens | 1.11 s | 244.60 s | ~220x slower |
| Per-token rate | ~0.101 s | ~22.24 s | ~220x slower |
| Default 20,000-token serial corpus, straight-line estimate | ~34 min | ~124 h | ~5.2 days |

One-token instrumentation attributes the 22.527 s total as follows:

| Site | Time | Share |
|---|---:|---:|
| 12x `c_attn` | 3.864 s | 17.15% |
| 12x `c_proj` | 1.289 s | 5.72% |
| 12x `fc` | 5.063 s | 22.47% |
| 12x MLP projection | 5.222 s | 23.18% |
| tied LM head | 7.083 s | 31.44% |
| all exact GEMMs | 22.521 s | 99.97% |
| attention QK/value | 5.0 ms | 0.022% |

A 20-second stack sample places ~96% in Atlas contraction / complete-accumulator placement / lookup-MAC work, ~1.9% in outer GEMM and cache processing, and ~0.03% in allocation. Cache-offer variants save only low single-digit percentages and are bit-identical.

The pinned `uor-matmul` API has no persistent immutable prepared-weight surface: `PackedCode` initial bytes are ignored and every call reprojects. More importantly, its own high-reuse exact-f32 measurements already show an approximately 49–52x contraction floor versus handwritten f32 loops. Workspace reuse, strided views, attention packing, `std`, explicit NEON, and NEON-dotprod therefore cannot plausibly close this gap; even ideal parallelism leaves a large regression.

### Pre-declared decision outcome

The dry-run failed the issue's acceptable-speed condition. Per the B2/B3 discipline, the negative branch is:

1. do not open/queue the B3 PR;
2. preserve the verified dry-run branch and evidence;
3. require either a materially faster upstream exact-float contraction design (not merely prepacking/allocation cleanup), or an explicit maintainer decision accepting the measured latency before resuming publication.

No change has been pushed to `main`, no PR is open, and #704 remains assigned while this blocker is actively evaluated.


### comment 5298214047 — 2026-08-14T21:11:37Z — Casey-allard

Pre-#712 local-gate finding on Apple aarch64: `cargo test --workspace --offline` at current `main` (`765d3a72`) fails to compile `uor-r4-model-source` because `neon_dot_tracks_scalar_result` still calls the removed `dot_neon` function (`crates/uor-r4-model-source/src/lib.rs:2691`). Linux CI does not select this target-gated test, so it survived #702. Treat removal/replacement of this stale backend test as part of #704's arithmetic census cleanup; do not reintroduce the deleted NEON owner. #712 itself does not touch model-source.

### comment 5298261590 — 2026-08-14T21:17:30Z — Casey-allard

A second current-main Apple fixture consequence surfaced in the #712 broad gate: with `/tmp/ref/out/model.bin` present, `uor-r4-graph-compiler --test observe::observe_cli_writes_shards_and_resumes_without_rewriting` now processes only 1/64 tokens in its hard-coded one-second budget (about 107s for the first exact-teacher observation), then fails because no completed manifest exists. CI skips vacuously when the checkpoint is absent. #704 should replace this elapsed-time assumption with a bounded deterministic test seam while resolving the measured exact-GPT2/teacher performance boundary.

### comment 5300679114 — 2026-08-15T05:11:52Z — Casey-allard

A1 provenance slice is now PR #724 (assigned Casey): https://github.com/UOR-Foundation/uor-r4/pull/724

This PR deliberately **does not close #704** and changes no teacher arithmetic. It binds the actual registered source-attention operator at every observation/source-compile boundary, carries it through API/server/cover/evaluation/recorded compile, and refuses incompatible resume before mutation.

One premise correction is now explicit in code/docs: source and snapshot κ are hashes of source bytes. They do not encode the executor arithmetic and therefore do not move for an arithmetic-only change. The versioned attention-operator record is the arithmetic-era identity; corpus/artifact CIDs move only when produced bytes move.

Local evidence for #724 is complete: workspace tests/clippy/fmt/no_std/wasm, R1-R6 audits, `cargo deny check`, independent CLEAN review, and a non-vacuous canonical Gate E against the 60,816,028-byte llama2.c checkpoint (3/3 ignored release tests passed).

The broad dense GPT-2 exact-GEMM prototype remains a no-ship result at roughly 220x slower. After A1 merges, the next active slice is attention-only QK/value migration plus `/2` operator records under the preserved speed gate. Dense Conv1D/MLP/tied-lm-head ownership stays open pending a materially faster upstream fixed-weight exact GEMV/skinny-GEMM path.


### comment 5300741238 — 2026-08-15T05:30:31Z — Casey-allard

The remaining dense fixed-weight portion is now tracked upstream by UOR-Foundation/uor-matmul#37: https://github.com/UOR-Foundation/uor-matmul/issues/37

That issue carries the exact prepared-weight API, contraction-factorization, and <=1.10x paired integration gate. The measured profile rules out prepacking alone: its theoretical ceiling is about 1.02x against an approximately 220x gap.

This is a dependency/reference, not completion of #704. Keep #704 open and assigned while active. #704 may close only after an upstream merge SHA is pinned consistently, the dense Conv1D/MLP/tied-lm-head sites migrate, provenance and correctness gates pass, non-vacuous wasm/Gate E evidence is recorded, and the integrated canary satisfies the speed gate.

Assignment note: GitHub allowed filing uor-matmul#37 but refused Casey self-assignment because this account has pull-only permission in that repository; the issue records that limitation explicitly.

### comment 5300752843 — 2026-08-15T05:34:08Z — Casey-allard

A1 is merged via PR #724 at merge commit 1cd9c1dc5b122d157ca75d3e578da19e45b49155.

Binding merge-group evidence: https://github.com/UOR-Foundation/uor-r4/actions/runs/31866536624 — full workspace tests, clippy/fmt/no_std, deterministic rebuild, non-vacuous canonical kappa reproduction, Gate C, wasm-pack, fuzz, all-features dormant tests, R1-R6 conformance, and audit all passed.

The local pre-queue evidence also included the 60,816,028-byte llama2.c checkpoint with 3/3 ignored release Gate E tests and an independent CLEAN whole-diff review.

This completes only the A1 provenance/resume boundary. #704 stays OPEN and assigned. Next is A2: exact QK/value attention plus immutable source operator v2 records under a paired real-GPT2 <=1.10x latency gate. Dense Conv1D/MLP/tied-lm-head remains tracked through uor-matmul#37.

### comment 5300759803 — 2026-08-15T05:36:24Z — Casey-allard

## A2 paired speed-gate run contract

metric to move: median real GPT-2 124M source-forward nanoseconds per token, model load excluded
reachability ceiling: the broad profile measured exact QK/value at about 5 ms of an approximately 101 ms conventional token, predicting roughly 1.05x; dense exact sites remain untouched
instrument + verdict: one warmup plus 5 release runs of 32 deterministic tokens, state reset between runs, same harness/host/model for baseline and candidate
exit rule: candidate median / baseline median <= 1.10
if positive: run correctness, real numpy, non-vacuous Gate E, wasm and full workspace gates; open an assigned partial PR that References #704
if negative: preserve the measurement, do not queue A2 arithmetic, and keep #704 open
cost estimate: about 20 seconds per measured commit after compilation; no corpus-scale run

The harness will be committed first and measured on conventional main-derived arithmetic. Exact QK/value plus immutable v2 operator records will be a later commit on the same branch and measured with the recorded baseline supplied to the harness. Dense Conv1D/MLP/tied-lm-head is explicitly outside this A2 run and remains owned by uor-matmul#37.

### comment 5300828228 — 2026-08-15T05:56:44Z — Casey-allard

A2 cheap-gate outcome (2026-08-15): **NEGATIVE / no-ship**.

The paired release canary used the exact same real GPT-2 snapshot, source weight kappa, config kappa, canonical path, 32-token story, and five-run median on both commits.

- baseline: `39,898,821.594 ns/token`
- exact QK/value candidate: `105,707,975.250 ns/token`
- ratio: `2.649401x`
- preregistered maximum: `1.10x`

Per the posted run contract, I am not queueing the A2 arithmetic/provenance diff and am not re-pinning Gate E. The branch is retained only as a measured prototype. #704 remains open and assigned.

Next action is the consumer-driven upstream performance track in UOR-Foundation/uor-matmul#37: build the reusable exact execution factorization against this real uor-r4 harness, upstream it, update the pinned dependency only after upstream acceptance, and then rerun A2. The suffixed server-root restart/source-pairing issue found during review will be fixed when A2 is revived.

### comment 5300861283 — 2026-08-15T06:06:16Z — Casey-allard

Upstream scope is now split by the two distinct measured bottlenecks:

- fixed GPT-2/Smol dense weights: UOR-Foundation/uor-matmul#37
- dynamic small/strided QK and value aggregation: UOR-Foundation/uor-matmul#38

A single upstream PR may reference both if one coherent factorization covers them, but #704 will require both binding consumer gates before the final uor-r4 pin/closure.

### comment 5300910543 — 2026-08-15T06:19:50Z — Casey-allard

Upstream #37 cheap-gate update (2026-08-15):

The first immutable prepared-weight implementation is exact but decisively fails the consumer speed gate.

- real GPT-2 layer-0 `c_attn` `2304x768`, `n=1`, 5 alternating pairs
- conventional median: `150,708 ns`
- prepared exact median: `204,631,917 ns`
- ratio: `1357.803945x` against the `1.10x` limit
- full exact-output parity: PASS
- model-wide preparation/parity covered all 49 fixed dense matrices, but no whole-forward timing was launched

The paired upstream `768x768` exact-vs-exact gate improved only 1.94%, and profiling places 91.1% of hot samples in the shared Atlas contraction/lookup kernel. Therefore prepacking is rejected, not shipped. Work has moved to a depth-fused exact contraction; #704 remains open and no uor-matmul pin or arithmetic-era re-pin will occur unless the cheap consumer gate first becomes plausible.


### comment 5300953449 — 2026-08-15T06:31:56Z — Casey-allard

Final #37 reachability result (2026-08-15):

- exact depth/grade aggregation improved the old exact route `6.212x` with full parity
- replacing its lookup coefficient product with an intentionally non-shippable exact native product improved it only another `4.79x`
- even that optimistic upper bound remains approximately `45.6-57x` slower than the conventional GPT-2 consumer, versus the required `<=1.10x`
- the native-product experiment was restored; no forbidden arithmetic remains in either worktree

This is now a contract incompatibility, not an allocation/prepack/placement defect: lookup-only exact binary32 product work plus exact grade accumulation cannot fit the hardware-FMA budget on the binding M1. #704 remains open pending an explicit project decision about offline teacher arithmetic; deployed R4 runtime arithmetic is unaffected by that decision.


### comment 5300982789 — 2026-08-15T06:40:09Z — Casey-allard

Maintainer decision (2026-08-15): **Choice A — separate compiler and deployed-runtime arithmetic.**

The deployed R4 runtime keeps its existing allocation-free, multiply-free opcode contract unchanged. The offline teacher/compiler may use a hardware-native arithmetic lane only when a mechanical witness proves that every returned binary32 bit pattern is identical to the pinned exact `uor-matmul` result; ambiguous cases must fall back to the exact route. This is an arithmetic implementation choice, not a relaxation of output semantics.

The lookup-only prepared/grade prototypes remain measured negative evidence and will not ship. The next local uor-r4 experiment is a certified-native dot/GEMV lane:

- binary32 operands are represented exactly in binary64;
- each binary32 product is exact in binary64;
- a conservative, upward-rounded error enclosure must fit wholly inside one binary32 rounding cell before the native candidate is accepted;
- non-finite, boundary, cancellation, or otherwise ambiguous rows use the pinned exact fallback;
- all outputs are compared bit-for-bit with the current exact route, including adversarial inputs;
- no deployed runtime file or opcode changes.

Run contract before any full model timing:

- metric: real GPT-2 layer-0 `c_attn`, 2304x768x1, paired candidate/conventional time;
- cheap gate: complete-output exact parity, zero hot allocation, and candidate <=4.0x conventional on the pinned source; otherwise stop;
- if positive: run the paired whole-model 11-token consumer gate, initially capped at <=3.0x the same-run conventional baseline (turning the prior ~220x regression into a bounded offline cost), then upstream the independently useful API;
- if negative: keep #704 open with the negative record and do not publish another unused API.

#704 remains open and assigned while this is implemented. The already-merged provenance increment in PR #724 remains complete and unchanged.


### comment 5301130490 — 2026-08-15T07:21:59Z — Casey-allard

## Choice-A certified-native cheap gate — negative / parked (2026-08-15)

The mechanically witnessed offline native lane is bit-correct, but it decisively fails the preregistered real-consumer reachability gate. No full-model run or additional clock was launched.

Pinned identity:

- repository/revision: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model: 548,105,171 bytes, `blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06`
- config: 665 bytes, `blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c`
- host/toolchain: Apple aarch64 macOS, rustc 1.97.1
- shape/protocol: real layer-0 `c_attn`, `2304x768x1`, nine alternating pairs, model load and exact preflight outside timing

Correctness/proof evidence:

- 1,160 adversarial/random proof lanes matched the pinned exact route bit-for-bit; 1,107 certified and all conservative fallback classes were exercised
- real preflight and every timed output matched the exact route bit-for-bit
- fixed weights remained immutable and every timed candidate call allocated zero bytes
- accepted dots use exact binary32 products in binary64 plus an outward-rounded error enclosure strictly contained inside one binary32 rounding cell
- uncertain/tie/zero/nonfinite/overflow-adjacent lanes use the exact `uor_matmul` fallback
- bias is added exactly once in binary32 after the unbiased dot, preserving the target epilogue order
- independent read-only review found no remaining P0-P2 defects

Measured result:

- candidate median: `2,969,958 ns`
- conventional median: `147,333 ns`
- paired median ratio: `16.903285271x`
- preregistered gate: strict `<4.0x`
- certification: `20,676 / 20,736` lanes (`0.997106481`); 60 exact fallbacks, all cell-boundary/uncertain

The negative is structural enough for this design: even 99.71% certification remains roughly 4.2 times slower than the maximum allowed ratio. Per the posted run contract, the POC remains uncommitted, no unused API is submitted upstream, and no whole-model timing is run.

#704 remains OPEN. The merged A1 provenance work remains valid; dense and dynamic arithmetic ownership still require a materially different factorization or a separately approved issue contract.

### comment 5301267027 — 2026-08-15T07:58:22Z — Casey-allard

## Dynamic-attention Choice-A run contract (2026-08-15)

This is the exact-QK/weighted-value feasibility gate that follows the negative dense `c_attn` experiment. It changes no deployed-runtime arithmetic and no production default before the verdict.

- **metric to move:** same-binary real GPT-2 32-token wall time, certified-native QK/value candidate divided by the matching conventional control, paired median across five alternating pairs.
- **reachability ceiling:** the incumbent exact dynamic-attention route measured `2.649401x` conventional and the best direct-panel experiment measured `2.094496x`; only replacing the per-dot exact contraction/setup with a mechanically certified native result (plus exact fallback) can plausibly reach the binding `<=1.10x` ceiling.
- **cheap instrument + current verdict:** frozen candidate `blake3:0dc47d23c81e0f1837e6bcd46ec6d87ade1900c6bd5554bce74289a67cd3198f`; focused proof is green under default test parallelism (`5 passed, 1 ignored`). Independent gathered `uor_matmul` oracles, full recurrent exact-bit parity, all four fallback classes, QK/value non-vacuity, and zero hot allocations pass. Two independent read-only reviews report no P0-P2 findings.
- **exit rule:** PASS only when the paired median candidate/conventional ratio is `<=1.10`, every candidate output matches the pinned exact route bit-for-bit, and both QK and value arms certify real lanes with exact verdict accounting.
- **if positive:** integrate the arithmetic as a new source-operator era in uor-r4, preserve v1 bytes, run the full gates, then amend/supersede upstream #38 for the approved Choice-A semantics and submit the reusable uor-matmul PR requested by the maintainer.
- **if negative:** record the exact evidence, ship no POC/API/pin, keep #704 open, and continue only with a materially different factorization.
- **cost estimate:** one release build plus one exact preflight, two warmups, and five paired 32-token samples on the recorded Apple M1; expected under a few minutes and blocks the shared release target/CPU only for that interval.

Command (single-threaded; no other clocks):
`CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target cargo test --release -p uor-r4-model-source --test certified_native_attention real_gpt2_certified_attention_32_token_gate --offline -- --ignored --exact --nocapture --test-threads=1`

### comment 5301276074 — 2026-08-15T08:01:12Z — Casey-allard

## Choice-A dynamic-attention gate — PASS (2026-08-15)

The frozen certified-native QK/value candidate passed the preregistered real GPT-2 consumer gate. I stopped immediately after this canary; no follow-on/full-model clock ran.

Bound candidate and input:

- candidate algorithm: `uor-r4-certified-native-attention-cell-v1`
- candidate κ: `blake3:0dc47d23c81e0f1837e6bcd46ec6d87ade1900c6bd5554bce74289a67cd3198f`
- uor-r4 base: `cf7b8988e56b4baac5427d213bd7c1a5026769d4`
- exact fallback owner: uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`
- pinned source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model: 548,105,171 bytes, `blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06`
- config: 665 bytes, `blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c`
- host/toolchain: Apple M1, aarch64 macOS, rustc 1.97.1; no external release flags

Five alternating 32-token pairs:

| pair | candidate ns | conventional ns | ratio |
|---:|---:|---:|---:|
| 0 | 1,267,099,584 | 1,258,622,209 | 1.006735440 |
| 1 | 1,267,146,250 | 1,257,545,083 | 1.007634849 |
| 2 | 1,263,566,250 | 1,261,027,208 | 1.002013471 |
| 3 | 1,268,688,542 | 1,253,622,834 | 1.012017736 |
| 4 | 1,268,128,875 | 1,256,443,667 | 1.009300224 |

Verdict:

- paired median ratio: **`1.007634849x`**
- binding ceiling: **`<=1.10x`**
- candidate median: `1,267,146,250 ns`
- conventional median: `1,257,545,083 ns`
- QK: `380,150 / 380,160` lanes certified; 10 exact cell-boundary fallbacks
- value: `1,474,560 / 1,474,560` lanes certified
- exact recurrent output parity: PASS at every preflight and timed story
- hot-path allocations: zero
- focused default-parallel suite: 5 passed / 1 ignored
- two independent final P0-P2 audits: CLEAN

The positive branch is now active: integrate this as a new, explicit source-attention arithmetic era in uor-r4 while keeping every v1 canonical byte and the deployed R4 runtime unchanged; then run the full gates. This result does **not** claim compliance with upstream uor-matmul #38’s current prohibition on native accumulation/fallback. Upstream governance/API work follows only after the consumer integration is complete and reviewed.

### comment 5302900756 — 2026-08-15T15:25:58Z — Casey-allard

Attention slice is ready in #727 (`9c01f928`).

Committed-SHA real GPT-2 evidence:
- paired median certified-native/conventional ratio: **1.010964087** (acceptance limit 1.10)
- QK certification: 0.999973695
- value certification: 1.0
- exact recurrent symbol-bit parity and zero-allocation guards passed
- full workspace tests, clippy, fmt, no_std/alloc, wasm, claim-wording, and independent P0-P2 audits are clean

This PR advances the immutable source-attention aliases to v2, preserves all v1 records, and hardens the provenance/serving/checkpoint boundaries needed to deploy it.

It intentionally **does not close #704**: GPT-2 Conv1D, MLP, and tied `lm_head` remain conventional and are the remaining dense-migration scope.

### comment 5303124220 — 2026-08-15T16:20:52Z — Casey-allard

Attention slice merged via #727 at `6fbf718b4115859df6544545a5c43d7638a6ad0a`.

Completed in this slice:

- immutable source attention v1 plus current certified-native v2 provenance records
- Llama serial/batched and GPT-2 serial/trace/batched QK/value ownership
- exact pinned `uor-matmul` fallback, bit parity, zero hot allocations, v1 compatibility
- crash-atomic source/corpus/cover/score publication and v1/v2 serving/reload resolution
- final real GPT-2 gate on PR head: paired median 1.007121087x <= 1.10x; QK 99.9973695% certified, value 100%
- binding merge-group run 31894549511: all required jobs green

This is deliberately partial and the issue remains open/assigned. Remaining scope is the dense GPT-2 Conv1D/MLP/tied-lm-head arithmetic. The next action is a fresh-main, measurement-gated dense-owner pass; no upstream prepared API will be proposed unless that consumer gate is positive.

### comment 5303170762 — 2026-08-15T16:31:55Z — Casey-allard

_edited 2026-08-15T16:43:59Z_

## Dense Choice-A f64-fold reachability contract — corrected (2026-08-15)

Correction: the previous rendering of this comment accidentally reused the dynamic-attention 1.10x ceiling and emitted literal newline escapes. The maintainer-approved dense Choice-A contract is unchanged: the real layer-0 c_attn cheap gate is <4.0x, followed by a same-run whole-model ceiling of <=3.0x only after an exact certified candidate exists. This comment replaces the malformed contract before the confirmatory clock.

- **metric to move:** paired ratio for one real GPT-2 layer-0 fused c_attn Conv1D (768 inputs x 2304 outputs, n=1), optimistic f64 accumulation floor divided by the existing conventional f32 control.
- **reachability ceiling:** the timed floor contains caller-owned f64 zeroing, exact f32-to-f64 products, a vectorized f64 accumulation, f64-to-f32 conversion, and the one historical f32 bias add. It omits the certificate, census, exact fallback, and parity oracle. It is a necessary-condition screen for this loop family, not a shippable candidate or a universal lower bound.
- **instrument:** hard-bind openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e, model blake3:3bca1b7f...ce06 (548,105,171 bytes), config blake3:23e4471d...310c (665 bytes), rustc 1.97.1/aarch64 macOS; validate the generated widening-vector loop in disassembly. Run 20 warmups and exactly nine alternating same-input pairs, then stop.
- **estimator and uncertainty:** primary statistic is the median of the nine per-pair ratios, not the ratio of arm medians. The positive reachability branch additionally requires the exact one-sided 95% percentile-bootstrap upper bound for that paired median to be <4.0x.
- **if positive:** implement a rigorous outward witness with near-zero real exact fallback, prove bit parity/zero hot allocation/verdict accounting, and rerun the actual certified c_attn candidate against the original strict <4.0x gate. Only a positive exact candidate authorizes the <=3.0x whole-model run.
- **if negative:** preserve the measurement, ship no POC/API/pin, run no full model, and keep #704 open for a materially different factorization.
- **cost estimate:** milliseconds of timed work using the already-loaded local snapshot; one isolated process and no shared cargo build.

This does not alter the multiplication-free deployed R4 runtime and does not claim compliance with upstream uor-matmul #38.

### comment 5303229535 — 2026-08-15T16:46:21Z — Casey-allard

## Dense Choice-A f64-fold reachability — PASS (2026-08-15)

The corrected, replayable necessary-condition screen passed the maintainer-approved dense cheap threshold. This is not an exact candidate and does not authorize production integration or whole-model timing; it authorizes only the rigorous certificate/fallback implementation.

Bound identity:

- source: openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e
- model: 548,105,171 bytes, blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06
- config: 665 bytes, blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c
- rustc 1.97.1 / LLVM 22.1.6 / aarch64-apple-darwin
- exact source SHA-256: 55b3199bae8efd3460b4c585e77d7b753711825c669e4102ba064ca988bf3645
- binary rebuilt from that source SHA-256: 85e6836c4ade8bfea8680da18a5794ecbd0a13f45d5c9a6e36e38a2a207a80da
- disassembly: vector fcvtl/fcvtl2, fmul.2d, fadd.2d, fcvtn; no scalarized inner fold

Nine alternating pairs after 20 warmups:

| pair | first | f64 floor ns | control ns | ratio |
|---:|---|---:|---:|---:|
| 0 | f64 | 622,625 | 354,125 | 1.758206848 |
| 1 | control | 571,791 | 356,666 | 1.603155333 |
| 2 | f64 | 566,625 | 358,375 | 1.581095221 |
| 3 | control | 570,666 | 356,041 | 1.602809789 |
| 4 | f64 | 571,875 | 372,084 | 1.536951334 |
| 5 | control | 565,458 | 673,375 | 0.839737145 |
| 6 | f64 | 578,083 | 369,042 | 1.566442302 |
| 7 | control | 565,583 | 351,667 | 1.608291367 |
| 8 | f64 | 600,167 | 355,417 | 1.688627725 |

- primary paired median: 1.602809789x
- exact one-sided 95% empirical-bootstrap upper percentile bound: 1.608291367x
- approved reachability gate: upper bound <4.0x
- verdict: PASS

The separate 1.10x attention threshold does not apply to this dense Choice-A stage. The dense contract remains strict <4.0x for the completed exact c_attn candidate, then <=3.0x for the same-run whole model.

Next action: add an outward actual-sum witness and lane-sparse error-free refinement so real exact fallback approaches zero; prove exact-bit parity, zero hot allocation, and complete verdict accounting before any further clock. #704 remains open and assigned.

### comment 5303351269 — 2026-08-15T17:16:52Z — Casey-allard

## Choice-A dense `c_attn` certified-candidate run contract (2026-08-15)

This gate is evidence-only. Production Conv1D/MLP/lm-head dispatch remains conventional until the separate full dense owner and whole-model gate pass.

### Candidate identity

- base: `6fbf718b4115859df6544545a5c43d7638a6ad0a`
- `crates/uor-r4-model-source/src/gpt2.rs` SHA-256: `9c3435636638c63d2baac3545480dfb694db923c14b0c5ad8fde7c8c7dcff167`
- `crates/uor-r4-model-source/tests/certified_native_dense.rs` SHA-256: `ea823d27a74ac4cd4168f0ab6d7d1da4c92678ecb178489c0be90d67f3082685`
- candidate algorithm: `gpt2-c-attn-exact-real-dot-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback/1`
- exact fallback owner: uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`
- source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model: 548,105,171 bytes, `blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06`
- config: 665 bytes, `blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c`
- host/toolchain: Apple M1, aarch64 macOS, rustc 1.97.1; release build; no external codegen/profile overrides
- two independent hash-pinned read-only audits: CLEAN; focused local reproduction: 4 passed / 1 ignored

### Run contract

- **metric to move:** real GPT-2 layer-0 fused `c_attn`, `768 x 2304 x 1`. Twenty fixed warmups per arm, then nine matched pairs in alternating candidate-first/control-first order.
- **reachability ceiling:** the prior certified candidate measured `16.903285271x`. The corrected f64-only necessary-condition screen passed with an exact one-sided 95% bootstrap upper of `1.608291367x`, and the current real-input structural probe reports 20,673 fast-certified + 63 TwoSum-refined + 0 exact-fallback lanes out of 20,736.
- **estimator and uncertainty:** primary descriptive statistic is the median of the nine paired ratios, distinct from the ratio of arm medians. The binding statistic is the exact empirical-bootstrap one-sided 95% upper percentile for that paired median. For n=9 it is computed without RNG from the first ordered sample whose `P(Binomial(9, j/9) >= 5) >= 0.95`; distinct samples select order statistic 7.
- **hard preconditions:** exact base, source/config/manifest/dependency/toolchain identity; exactly the two hashed candidate paths dirty; complete exact-bit preflight; every warmup and timed candidate output equals its stored pinned-uor-matmul result bit-for-bit; output poison fully overwritten; every lane receives exactly one verdict; fast and refined arms are non-vacuous; zero exact fallback on these nine real inputs; zero allocation in every timed candidate and control call.
- **exit rule:** PASS only if every precondition holds and the one-sided 95% bootstrap upper is strictly `<4.0x`. Equality fails.
- **if positive:** stop after this canary. Implement the full serial/batched Conv1D, MLP, and tied-lm-head owner; add a separate immutable dense-arithmetic provenance record while preserving source κ; then run the previously declared same-run whole-model gate at `<=3.0x`. No upstream API is proposed before that consumer result.
- **if negative:** record the exact result and hashes; run no dense sweep or whole-model clock; ship no production dispatch, provenance, dependency-pin, or upstream change; keep #704 open and park this factorization.
- **cost estimate:** one warm release build, source load, exact preflight, 40 warmup calls, and nine paired samples; no corpus-scale work.

Binding command:

```bash
UOR_GPT2_SOURCE=/Users/casey.allard/uor-r4/.worktrees/i704/.uor-models/sources/gpt2-124m \
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target \
cargo test --release -p uor-r4-model-source \
  --test certified_native_dense \
  real_layer0_c_attn_certified_native_consumer_gate \
  --offline -- --ignored --exact --nocapture --test-threads=1
```

The deployed R4 runtime remains allocation-free and multiplication-free. This does not claim compliance with upstream uor-matmul #38.

### comment 5303355300 — 2026-08-15T17:17:54Z — Casey-allard

## Choice-A dense `c_attn` certified-candidate gate — PASS (2026-08-15)

The frozen evidence-only candidate passed the preregistered strict uncertainty gate. I stopped immediately after this canary; no dense sweep, whole-model clock, production dispatch, provenance, dependency-pin, or upstream change ran.

### Bound identity

- base before/after: `6fbf718b4115859df6544545a5c43d7638a6ad0a`
- `gpt2.rs` SHA-256 before/after: `9c3435636638c63d2baac3545480dfb694db923c14b0c5ad8fde7c8c7dcff167`
- harness SHA-256 before/after: `ea823d27a74ac4cd4168f0ab6d7d1da4c92678ecb178489c0be90d67f3082685`
- candidate algorithm: `gpt2-c-attn-exact-real-dot-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback/1`
- bootstrap algorithm: `empirical-bootstrap-median-exact-order-statistic-n9-one-sided-upper-95/1`
- exact fallback: uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`
- source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model/config: 548,105,171 B `blake3:3bca1b7f...ce06`; 665 B `blake3:23e4471d...310c`
- host/toolchain: Apple M1; aarch64 macOS; rustc 1.97.1 / LLVM 22.1.6; release; no external overrides
- dirty set before/after: exactly the two reviewed candidate paths
- independent hash-pinned numerical and measurement audits: CLEAN

### Raw nine-pair result

Twenty fixed warmups per arm preceded the alternating pairs. Preparation, exact preflight, output poisoning, and checks were outside each timer.

| pair | first | candidate ns | conventional ns | paired ratio |
|---:|---|---:|---:|---:|
| 0 | candidate | 265,375 | 148,833 | 1.783038708 |
| 1 | conventional | 275,500 | 161,250 | 1.708527132 |
| 2 | candidate | 288,500 | 144,083 | 2.002318108 |
| 3 | conventional | 282,417 | 153,584 | 1.838843890 |
| 4 | candidate | 271,375 | 146,959 | 1.846603474 |
| 5 | conventional | 266,750 | 153,500 | 1.737785016 |
| 6 | candidate | 267,208 | 144,667 | 1.847055652 |
| 7 | conventional | 267,833 | 153,334 | 1.746729362 |
| 8 | candidate | 290,125 | 140,625 | 2.063111111 |

### Verdict

- paired median ratio: **`1.838843890x`**
- exact empirical-bootstrap one-sided 95% upper: **`1.847055652x`**
- binding rule: upper strictly **`<4.0x`**
- candidate median: `271,375 ns`
- conventional median: `148,833 ns`
- ratio of separate medians: `1.823352348x` (reported separately; not the gate)
- preflight census: 20,673 fast-certified + 63 TwoSum-refined + 0 fallback = 20,736 lanes
- timed census: 20,673 fast-certified + 63 TwoSum-refined + 0 fallback = 20,736 lanes
- warmup census: 45,941 fast + 139 refined + 0 fallback = 46,080 lanes
- every preflight, warmup, and timed candidate output matched the stored pinned exact result bit-for-bit
- every output poison was overwritten; every lane had one verdict
- every timed candidate and conventional call allocated zero bytes
- exact command exited 0; release compile 4.61 s; test 3.50 s

**Decision:** PASS. The positive branch is active: implement the complete serial/batched GPT-2 Conv1D, MLP, and tied-lm-head arithmetic owner on this fresh-main track, keep source κ unchanged, add a separate immutable dense-arithmetic record, and only then run the preregistered same-run whole-model `<=3.0x` gate. Production and upstream remain unchanged until that result and review are positive.

#704 remains OPEN and assigned.

### comment 5303588868 — 2026-08-15T18:17:15Z — Casey-allard

## Choice-A full-dense whole-model run contract (2026-08-15)

This gate is evidence-only. The normal GPT-2 serial, trace, and batched entry points remain byte-for-byte unchanged and conventional at every dense site. The candidate is reachable only through checked canary controls; no dense provenance record, production dispatch, dependency pin, or upstream API has been added.

### Frozen candidate identity

- base: `6fbf718b4115859df6544545a5c43d7638a6ad0a`
- `crates/uor-r4-model-source/src/gpt2.rs` SHA-256: `1a945487fb9ec350fd8f670b8c04dacaf6b66e2339ee96b6b3883082e4de4bf8`
- `crates/uor-r4-model-source/tests/certified_native_dense.rs` SHA-256: `144fd32d35292a6fd3e949bc7040b7e14c58563bdc31924b66d38bc1db547d89`
- candidate algorithm: `gpt2-all-dense-exact-dot-plus-f32-bias-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback-tied-head-transpose/1`
- bootstrap algorithm: `whole-gpt2-11-step-3-story-paired-empirical-bootstrap-median-exact-order-statistic-n9-one-sided-upper-95/1`
- exact fallback owner: uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`
- both arms retain the merged certified-native attention owner and differ only in dense mode
- source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model: 548,105,171 bytes, `blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06`
- config: 665 bytes, `blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c`
- host/toolchain: Apple M1, aarch64 macOS, rustc 1.97.1; release build; no external codegen/profile overrides
- exact dirty set: only the two hashed candidate paths above
- focused harness: 9 passed / 2 ignored; dense library tests: 3 passed; package/full-model-source gates, all-target/all-feature Clippy, fmt, diff-check, claim wording, and wasm check are green
- two independent frozen-hash reviews, including a final pre-clock numerical/measurement audit: CLEAN with no P0-P2 findings

### Run contract

- **metric to move:** same-run candidate/conventional elapsed time for three independent real GPT-2 stories: `[464,3290,373,257]`, `[15496,995]`, and `[50256,464,968,1971,318]` (11 recurrent steps total). State resets between stories. Both arms use one loaded immutable model, one prepared caller-owned dense workspace, and the same merged certified-native attention; the only arithmetic difference is dense `CertifiedNative` versus historical `Conventional`.
- **reachability ceiling:** the original broad exact owner was about `220x` conventional and 99.97% of its time was dense arithmetic. The exact layer-0 `c_attn` candidate then passed its strict cheap gate with a one-sided 95% upper of `1.847055652x <4.0x`, 20,673 fast + 63 refined + 0 fallback lanes. This whole-owner run may proceed past its cheap instrument only if all 49 fixed matrices (12 layers x 4 Conv1D sites plus tied `lm_head`) are exercised across the real stories with exactly 539 calls / 1,465,211 output lanes, nonzero fast and refined arms, and zero exact fallback.
- **baseline and correctness preconditions:** before candidate timing, normal production and explicit Conventional must match in full recurrent state bits; the checked-in independent golden must retain hidden error `<5e-2`, argmax, and top-5. Candidate must then match the pinned exact route bit-for-bit after every one of the 11 recurrent steps, including hidden/logits and K/V cache state. Every poison must be overwritten and every lane must receive exactly one verdict.
- **timing protocol:** prepare the model-bound workspace and exact expected states outside clocks. Run exactly two complete three-story warmup suites per arm, candidate then conventional. Then run exactly nine adjacent matched pairs, candidate first for even pairs and conventional first for odd pairs. A sample is the sum of only the 11 `forward_dense_control` call durations; model load, workspace construction, story resets, poisoning, census, allocation accounting, and parity checks are outside the timers. Every timed call must allocate zero bytes.
- **estimator and uncertainty:** report all raw arm times, per-pair order, per-pair ratios, arm medians, median of the nine paired ratios, and ratio of arm medians separately. The binding statistic is the deterministic exact empirical-bootstrap one-sided 95% upper percentile for the paired median; for nine distinct samples this is the seventh ordered paired ratio.
- **exit rule:** PASS only if every identity, baseline, golden, structural, exact-parity, overwrite, verdict-accounting, and zero-allocation precondition holds and the one-sided 95% bootstrap upper is `<=3.0x`. Equality passes. A structural fallback or incomplete owner census stops before the exact preflight or any clock.
- **if positive:** stop after this gate; record all raw evidence and unchanged hashes; then implement the separate immutable dense-arithmetic provenance era, production serial/trace/batched dispatch, resume/server resolution, documentation, and full gates on this branch. Open an assigned PR that **References #704**; do not close #704 until the dense migration is genuinely complete. Only after the real consumer integration succeeds is a reusable upstream API eligible for proposal.
- **if negative:** record the exact stopping point and evidence; run no additional dense clock or corpus-scale job; ship no production dispatch, provenance, dependency pin, or upstream change; keep #704 open and park this factorization.
- **cost estimate:** one warm release build, source/workspace preparation, an exact 11-step preflight (the prior broad exact route was about 244.6 seconds), four warmup suites, and nine paired suites. Expected wall time is roughly 5-7 minutes on the recorded host; it blocks the shared release target and CPU only for that interval.

Binding command (single process; no competing cargo/rustc or timing workload):

```bash
UOR_GPT2_SOURCE=/Users/casey.allard/uor-r4/.worktrees/i704/.uor-models/sources/gpt2-124m \
UOR_DENSE_EXPECTED_GPT2_SHA256=1a945487fb9ec350fd8f670b8c04dacaf6b66e2339ee96b6b3883082e4de4bf8 \
UOR_DENSE_EXPECTED_HARNESS_SHA256=144fd32d35292a6fd3e949bc7040b7e14c58563bdc31924b66d38bc1db547d89 \
CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target \
cargo test --release -p uor-r4-model-source \
  --test certified_native_dense \
  real_whole_gpt2_certified_dense_consumer_gate \
  --offline -- --ignored --exact --nocapture --test-threads=1
```

The deployed R4 runtime remains allocation-free and multiplication-free. This gate does not change source kappa and does not claim compliance with upstream uor-matmul #38.


### comment 5303616033 — 2026-08-15T18:24:07Z — Casey-allard

## Choice-A full-dense whole-model gate — PASS (2026-08-15)

The frozen evidence-only whole dense owner passed the [preregistered contract](https://github.com/UOR-Foundation/uor-r4/issues/704#issuecomment-5303588868). I stopped after this gate; no additional clock, corpus-scale job, production dispatch, provenance edit, dependency pin, or upstream change ran.

### Bound identity

- base before/after: `6fbf718b4115859df6544545a5c43d7638a6ad0a`
- `gpt2.rs` SHA-256 before/after: `1a945487fb9ec350fd8f670b8c04dacaf6b66e2339ee96b6b3883082e4de4bf8`
- harness SHA-256 before/after: `144fd32d35292a6fd3e949bc7040b7e14c58563bdc31924b66d38bc1db547d89`
- dirty set before/after: exactly those two reviewed candidate paths
- candidate algorithm: `gpt2-all-dense-exact-dot-plus-f32-bias-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback-tied-head-transpose/1`
- bootstrap algorithm: `whole-gpt2-11-step-3-story-paired-empirical-bootstrap-median-exact-order-statistic-n9-one-sided-upper-95/1`
- exact fallback: uor-matmul `b13c98449948174f590e337c4dc25dfc394a07d0`
- source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- model/config: 548,105,171 B `blake3:3bca1b7f...ce06`; 665 B `blake3:23e4471d...310c`
- host/toolchain: Apple M1; aarch64 macOS; rustc 1.97.1 / LLVM 22.1.6; release; no external overrides
- source/model/config/manifest/dependency/toolchain/host/build-override bindings: PASS

### Preconditions

- production/default and explicit Conventional full recurrent state bits: PASS after every real token
- independent checked-in golden: hidden `<5e-2`, argmax, and top-5 PASS for all three reset stories
- structural hard gate: 539 calls / 1,465,211 lanes; 1,450,907 fast + 14,304 refined + 0 fallback; PASS
- exact preflight: candidate equals pinned exact full recurrent state bit-for-bit after all 11 steps; PASS
- two complete three-story warmup suites per arm: PASS
- output overwrite, one verdict per lane, nonvacuous fast/refined arms: PASS
- every timed candidate and Conventional call allocated zero bytes
- prepared caller-owned workspace: 157,203,831 bytes total; preparation 113,811,000 ns, outside timing

### Raw nine-pair result

Each arm is the sum of only the 11 forward-call durations across the three independently reset stories.

| pair | first | candidate ns | Conventional ns | paired ratio |
|---:|---|---:|---:|---:|
| 0 | candidate | 310,361,916 | 438,655,001 | 0.707530782 |
| 1 | Conventional | 305,372,418 | 439,392,877 | 0.694987183 |
| 2 | candidate | 306,299,583 | 441,852,623 | 0.693216623 |
| 3 | Conventional | 303,427,415 | 439,566,250 | 0.690288244 |
| 4 | candidate | 309,243,207 | 439,152,917 | 0.704181152 |
| 5 | Conventional | 302,627,915 | 439,511,791 | 0.688554713 |
| 6 | candidate | 306,728,040 | 440,322,877 | 0.696598010 |
| 7 | Conventional | 303,182,542 | 439,220,832 | 0.690273593 |
| 8 | candidate | 308,994,624 | 438,990,000 | 0.703876225 |

### Binding verdict

- paired median ratio: **`0.694987183x`**
- exact empirical-bootstrap one-sided 95% upper: **`0.703876225x`**
- binding rule: upper **`<=3.0x`**
- candidate median: `306,299,583 ns/suite` = `27,845,417 ns/token`
- Conventional median: `439,392,877 ns/suite` = `39,944,807 ns/token`
- ratio of separate medians: `0.697097288x` (reported separately; not the gate)
- descriptive paired-median latency reduction: `30.50%`; ratio-of-medians speedup: `1.43452x`
- timed candidate census: 4,851 calls / 13,186,899 lanes; 13,058,163 fast + 128,736 refined + 0 fallback
- structural and exact-preflight candidate censuses were identical per suite: 539 calls / 1,465,211 lanes; 1,450,907 fast + 14,304 refined + 0 fallback
- exact command exited 0: 1 passed / 0 failed; release compile 6.41 s; test 240.39 s

**Decision: PASS.** This corrects the sequencing implied by the earlier layer-0 comments: the complete owner remained evidence-only through the whole-model decision, so no production arithmetic era was minted for an unaccepted candidate. The positive branch now authorizes the separate immutable dense-arithmetic provenance record, production serial/trace/batched dispatch, resume/server resolution, documentation, and full repository gates. Source kappa remains the pinned source-byte identity. A reusable upstream API becomes eligible only after this real consumer integration is complete and reviewed.

#704 remains OPEN and assigned until the dense migration itself is implemented, reviewed, and merged.


### comment 5304696953 — 2026-08-15T23:17:50Z — Casey-allard

## Integration work is now explicitly tracked (2026-08-15)

The accepted GPT-2 dense arithmetic owner and whole-model gate remain the #704 core. No upstream `uor-matmul` PR has been opened or is required: all implementation stays in `uor-r4`, using the already pinned `uor-matmul` exact owner only as fallback/reference.

The production-integration work discovered after the positive whole-model gate is now split into assigned, auditable repository issues rather than an unnamed local phase:

- #728 — canonical recorded-corpus generation binding, shared producer/reader coordination, crash/retry semantics, bounded hidden-stream verification, and managed server publication.
- #729 — fixed-partition, bounded-memory Rust corpus derivation plus the scale-script and historical-claim correction.

#704 remains the parent arithmetic/provenance issue. The integration branch may implement the three together because their changes overlap, but the PR and closeout evidence will map each changed surface and test to its owning issue. No additional ignored timing or corpus-scale measurement is authorized by this tracking change.

### comment 5305399010 — 2026-08-16T02:48:48Z — Casey-allard

Handoff: implementation is now visible in assigned draft PR #730 at pushed checkpoint `a902dbc2`. The certified GPT-2 dense model-source lane and whole-model evidence are complete; #728 transaction/server acceptance and #729 scale workflow remain explicitly open. No upstream `uor-matmul` PR exists or is required. Consolidated resume state: https://github.com/UOR-Foundation/uor-r4/pull/730#issuecomment-5305398275

### comment 5306462532 — 2026-08-16T08:08:06Z — Casey-allard

Merged via PR #730 in merge commit 830aafde4c571e6356ee920ef4cc1108eb493997 after the full merge-queue ladder passed. The certified GPT-2 dense v2 implementation, provenance plumbing, tests, and documentation now live entirely in uor-r4; this work did not require or create a separate uor-matmul PR.


---
_Comment/review ids in this file: 5298001556, 5298214047, 5298261590, 5300679114, 5300741238, 5300752843, 5300759803, 5300828228, 5300861283, 5300910543, 5300953449, 5300982789, 5301130490, 5301267027, 5301276074, 5302900756, 5303124220, 5303170762, 5303229535, 5303351269, 5303355300, 5303588868, 5303616033, 5304696953, 5305399010, 5306462532_

