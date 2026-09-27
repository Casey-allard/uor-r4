# Current UOR-R4 research state

Updated September 27, 2026. **Pre-alpha; no useful general-language, coding,
frontier, geometric-advantage or full-path energy qualification.**

## Fourth lab: shared research and integration

The owner has authorized the [fourth Codex research lab](../../.codex-lab/README.md)
and shared GitHub/worktree coordination with Google, OpenCode/DeepSeek/Kimi and
Claude. **Owner clarification, September 27:** “Keep the native,
multiplier-free serving target.” This lab does not adopt D10's converted
transformer backbone or hardware serving-multiplier exceptions. Offline Rust
learning remains permitted under D0-b. Preserve the other session's historical
record and source as research/comparators.
[Shared clarification](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5852795279).
The [canonical roadmap](project-track.md#four-lab-research-programme--owner-direction-september-26)
orders geometric attention, inference, prose, chat and reasoning, with discovery
branches allowed to revise the mechanism from evidence. The initial independent
source review identifies a concrete training/chat role-token mismatch and
overstated synthetic conversational evidence; see the
[integration review](fourth-lab-integration-review-2026-09-26.md).
The new [shared dialogue protocol](dialogue-protocol-contract-2026-09-26.md)
is an explicit integration seam; adoption by the active training and chatbot
branches is separate from providing it. All seven tokenizer checks pass. A
loaded-tokenizer witness confirms both the special-role fallback mismatch and
the subtler difference between segmented training text and concatenated panel
text. [Source bindings, witness and resource receipt](../evidence/fourth-lab-establishment-2026-09-26.json)
record the executed scope. No model fit or capability promotion
follows from the lab setup. Accepted models and the continuation negative below
remain unchanged. Other labs' active source and model jobs are preserved.

The retained integer `TextSession` now has an opt-in exact-ID dialogue adapter
and consistent optional stop-token handling. Six focused checks pass. A loaded
accepted-bundle witness matches exact prefix IDs and all 16 prediction records
across two selection policies against direct integer stepping; both short outputs
remain incoherent for chat. The
[adapter receipt](../evidence/dialogue-token-session-validation-2026-09-27.json)
records actual text, source/artifact identity and cost. This is verified input
and prediction transport, not trained chat. Existing chatbot CLI, corpus builder
and panel adoption remain open. The separate width-576 dialogue artifact recovery
is recorded below. No accepted bundle was changed.

The [persistent conversation adapter](dialogue-continuity-2026-09-27.md) now
retains generated IDs, pending tokens and categorical state across turns. It
preflights each complete request and records model EOS separately from explicit
caller closure; execution failure poisons the session. Five tokenizer and ten
integer-generation checks pass. On the unchanged accepted bundle, a two-turn
witness matches all eight prediction records and exact input history against
direct integer stepping, with 36 total conversation steps and no prefix replay.
The actual capped outputs are `“Is there the` and `1 asked the people`; this is
verified token/state transport, not useful chat. The
[source-bound result and complete cost](../evidence/dialogue-continuity-validation-2026-09-27.json)
record 112.04 seconds of build/check work and 2.576 seconds of actual inference.
Google retains CLI wiring ownership; this resolves its follow-up's
[display-history reconstruction seam](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5852795123)
at the reusable library boundary, with adoption still pending.

The [retained R1d dialogue artifact](dialogue-artifact-replay-2026-09-27.md) now
reloads through a strict offline continuous Full/Dot/Quaternion importer. All
four focused checks pass; an initial malformed tokenizer fixture and its repair
remain in the cost record. A loaded 15-position full/incremental comparison has
maximum probability delta 9.69e-7 and zero argmax disagreements. The fixed 38
open development requests produced 58 actual responses with exact generated
multi-turn history: 11 model EOS, 4 cycle and 43 token-cap stops. They contain
conversational phrasing and some topic/fact reuse, but are repetitive, often
irrelevant, and none of the eight factual replies supplies the requested fact.
This is a preserved weak-output baseline, not useful chat. The
[complete outputs and source/artifact/resource bindings](../evidence/dialogue-r1d-replay-2026-09-27.json)
record 18.132 seconds of actual inference and 58,572,800 bytes peak child RSS.
The historical fit remains 9,162,752 sampled positions; summing its per-update
records gives 6,280,627 response-supervised positions, correcting a duplicated
report field. Its bounded no-longer-fit decision remains; unstable extrapolated
floors do not establish capacity as the cause. No training extension or integer
loader widening follows. The frozen reader study and other labs' mechanisms
remain independent.

The [R1d training-context audit](dialogue-context-audit-2026-09-27.md) now
reconstructs all 2,237 update denominators exactly. Of 6,280,627 response-target
visits, 74.6033% start their input window inside the same response, with all
pre-response context outside the window. This does not establish the cause of
weak generated replies. Only 14,826/129,486 complete response prefixes fit
within 256 total IDs; 89.4847% of that eligible response-run population comes
from Everyday Conversations and Smol Constraints. A prefix-preserving option
therefore requires explicit population/weighting choices and a matched control,
coordinated with the separate dialogue trainer owner. No fit or filtering of
retained artifacts follows. Three focused Rust checks pass; the data-only pass
took 3.210 seconds with no model execution. The
[receipt](../evidence/dialogue-context-audit-2026-09-27.json) binds source,
inputs, rowwise reconstruction, result and cost.

**Concurrent discovery:** the [fourth-lab native-read review](fourth-lab-native-read-review-2026-09-26.md)
inspects Claude PR#1401's full-context Lorentz implementation and reported
development results. Claude fixed the two reported implementation defects and
guarded the conversational integration risks; PR #1401 is now merged. PR #1406
supplies its [native packet](../evidence/native-lorentz-packet-2026-09-26/README.md),
whose 104 listed file identities were verified by the fourth lab. Continuous
checkpoints and several replay inputs remain external. The
[radial read comparison](radial-read-control-2026-09-26.md) extends the merged
reader with an offline affine control while preserving existing upstream paths.
All six focused checks pass on source `48fd43c1`, covering the affine control
and retained Dot/Lorentz and checkpoint paths; the
[source-bound receipt](../evidence/offline-radial-read-validation-2026-09-27.json)
records executed checks, prior attempts and complete elapsed accounting.
Its implementation and a future language comparison have distinct scope.
The optional unit-scale campaign initializer is now implemented and all four
initializer plus two retained checkpoint checks pass. A source-bound startup
observation executed by Antigravity was independently verified and preserved by
the fourth lab, avoiding a duplicate run. Both readers have six finite nonzero
language-gradient families and nonzero causal read mass on all four fixed
full256 windows; no calibration correction is indicated. Supplementary
untrained continuations remain incoherent and include a disclosed duplicate-BOS
prompt condition. [Startup evidence and exact scope](../evidence/radial-unit-start-validation-2026-09-27.json).
The [explicit shared-parameter transfer](radial-parameter-transfer-2026-09-27.md)
is now implemented for both radial arms. It copies the retained negative
step15,672 Dot candidate's21 shared arrays, records its64,192,512 historical
target visits, and starts a new lineage with identical Adam resets and sampling
seed240927. Saved children retain their own evolved weights, optimizer and
historical transfer receipt. Ten source-bound focused checks pass. A single
183.70-second zero-update observation at the learned query/key scale completed:
both readers have finite nonzero measured gradient arrays and no zero causal
read positions across four fixed full256 windows. Initial NLL is2.389537 versus
2.400457; both eight-token outputs are `toys. One day, he found a`. These are
startup observations, not fitted advantage or prose qualification. The
[source, output and resource receipt](../evidence/radial-parameter-transfer-validation-2026-09-27.json)
binds the actual copied arrays and sealed report. No calibration sweep is
indicated. The same-geometry Dot parameter-only reset is now implemented, with
four focused checks passing for transfer, receipt compatibility and persisted
resume across all three geometries. One actual zero-update witness preserves
all 21 arrays, complete predictions on the same four full256 windows, and eight
generation decisions exactly against the independently loaded parent. Its fresh
Adam and per-parameter clocks start at zero. Initial Dot NLL is 1.996717, so the
radial startup losses above show an initial disturbance rather than a fitted
advantage. The Dot continuation is `toys and read them all day.` at the eight-token
cap; this remains a tiny observation, not prose qualification.
[Source, actual output and cost](../evidence/dot-reset-validation-2026-09-27.json)
are retained. The [matched adaptation plan](radial-adaptation-study-2026-09-27.md)
selects one 1,024-update screen per arm, with a descriptive checkpoint at 512 and
the final endpoint as the sole comparison. The complete twelve-hour local
process/evaluation projection is recorded. Source `488e3976` supplies the scoped
seven-root comparison and opt-in greedy prose supplement; three focused release
checks and three CLI rejection cases pass. The existing `joint-compare` contract
is unchanged. The source-bound optimized CPU-Accelerate executable has now
completed both the Dot reset and Lorentz fits: each reached 1,024 updates /
4,194,304 new visits, exit0 and a sealed final root. Affine started at 07:06 UTC
with the same executable and fixed conditions.
Full evaluation, successful greedy supplementation, comparative output review
and the adaptation result remain pending. The
[implementation, launch and resource receipt](../evidence/reader-study-tools-validation-2026-09-27.json)
records the exact scope and live process pointer. The mainline termination-objective
study remains independently owned and cannot supply that matched control.
General language/M1 qualification remains unmeasured here. The owner has now
confirmed this lab's native, multiplier-free serving target; the control
introduces no serving exception or converted backbone.

The fourth lab's [source review of Google PR #1410 at `9722200c`](https://github.com/UOR-Foundation/uor-r4/pull/1410#issuecomment-5851958824)
identifies useful serving optimization alongside unresolved integration and
evidence issues: conversational Lorentz remains guarded, the CLI retains role
fallbacks, BPE salience interprets token IDs as characters, and synthetic phase
and selected-byte checks do not qualify generated reasoning/recall. Its claimed
MinP prose repair and complete-path memory measurements are not established by
the cited source. Those findings do not change accepted artifacts or the native model's current
capability status. PR #1410 subsequently merged as `5d3a9932`; a merge and its
compatibility acknowledgements alone do not resolve the evidence boundaries.
Its merged files match the reviewed `9722200c` files, so the recorded findings
remain. The merge changes integer execution, not the continuous transfer/model
path measured in the Dot witness. Future integer measurements must bind this
changed source separately.


Claude's [draft stack experiment, PR #1414](https://github.com/UOR-Foundation/uor-r4/pull/1414)
now has a reviewed six-pilot packet at `dc721f0a`: all 31 listed file identities
match. Each pilot used 1,000 updates / 4,096,000 visits at one seed, with final
evaluation on 131,072 exposed development targets. The selected geometric
lr0.004 NLL is 2.609729 versus the selected transformer lr0.002 control's 2.768041.
This compares tuned architecture packages at similar parameter counts, not
curvature alone. Independent review of all 36 sampled/greedy continuations finds
repetition, malformed code and no useful coding/prose result. The packet records
12,066 seconds elapsed over three pilot rounds; the sum of process durations is
22,989 seconds and is not CPU time or complete preparation/build cost.
[Packet review and limitations](https://github.com/UOR-Foundation/uor-r4/pull/1414#issuecomment-5853505407).
Current head `35d8da2f` corrects the high-rate control wording to underperformance.
Source fixes for resume and scan/drive-bound claims remain separate from older
executed binaries. Six raw model weights and two executables are still external;
the fourth lab has designated an owner-SSD intake and requested an immutable
transfer, with payload verification pending.
[Preservation coordination](https://github.com/UOR-Foundation/uor-r4/pull/1414#issuecomment-5853595061).
This complementary Claude-owned track has no integrated integer/session path
and neither supplies the matched Dot control nor changes its dose or serving
policy. Its package-level likelihood result keeps deeper architecture as a
candidate; useful generated behavior remains required.

The separately owned termination-objective study has a
[source-verified weighted-shard normalization/reporting defect](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5852801075)
at `c54d7801`: shard-normalized weighted losses are combined by equal batch mass,
so unequal boundary counts differ from a global weighted mean; its training
curve also records weighted loss as `batch_mean_nll`. Standard development
metrics remain unweighted. The four fits and eight full evaluations are now
complete. The [fourth-lab result review](termination-objective-review-2026-09-27.md)
classifies this implemented dose as **INERT: no automatic extension**. Weighted
full Read NLL is worse than plain by 0.009439 and 0.012453. All 20 sampled Read
stories fail complete prose criteria; both plain and weighted groups select
EOS in 2/10 outputs. The original 20 source failures yield only three weighted
resolutions beyond plain, below the declared four; only two are closure fixes.
All 128 new source outputs use the caller's first-period stop, not model EOS.
The declared numerical, source-retention and terminal-cycle guardrails hold at
this saved-packet scope. Preserve the modest source gains and remaining errors,
without attributing this result to the known objective defect. Future adoption
needs the correction, but no repair-fit follows automatically. The
[derived output/identity packet](../evidence/termination-objective-review-2026-09-27.json)
keeps the parent/control row join and recorded cost components. Owning-lab full
resource reconciliation remains open; overlapping shared elapsed must not be
charged again. The frozen unweighted reader study is unaffected.

After the optional weighted objective entered mainline, the current-source
fixed reader comparator now explicitly rejects any `end_weight` override.
Three focused optimized Rust checks pass, including actual campaign
deserialization for Dot, Lorentz and Affine: historical omission and null stay
accepted, while explicit weights are rejected. The
[source-bound guard receipt](../evidence/reader-objective-guard-2026-09-27.json)
records 46.57 seconds of build/check work and the scope. Training arithmetic and
the frozen running study are unchanged. Source/result descriptions now distinguish
weighted training loss from ordinary evaluation and non-timing witness agreement
from identical files. An
[overlapping termination delivery charge was reconciled](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5853741962)
against 18 linked receipts, removing 17,348,658 ms of duplicate shared wall time
while preserving original records and unchanged resource limits.

## Decision and active work

The owner supplied a whole-project stuck-point review and directed warranted
corrections before the pending merge. The [independent assessment](stuck-point-review-response-2026-09-24.md)
agrees with its central learning-method criticism, corrects outdated and
mathematical claims, and establishes [D8](DECISIONS.md#d8--correct-the-training-method-and-reference-ladder).
The [canonical plan](project-track.md) owns the persistent training/reference
ladder. #973 remains active under programme #820; their full acceptance is open.

**Park further A1–A4 local selector tuning.** Preserve its native serving and
exact-memory scaffolds. The September25 joint recurrent-memory learner and its
learned neighboring-code successor retain their original scoped engineering
acceptance; the two older quantized negatives remain preserved. The completed
September26 language continuation is a distinct candidate with failed prose
and retention criteria, as reported below. The shared bounded-admission successor now completes
its paired comparison but fails source retention in both arms. Keep the accepted
learned-code parents and full 256-token access. The recent64 training follow-up
is withdrawn under the owner-directed [D9 correction](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract).
The full-context standalone integer serving session is now implemented and retained
at the scope below. An exact arithmetic optimization makes actual generation
4.5–6.6 times faster on the measured workloads. The subsequent fixed exposure
completed without meeting coherent-language criteria. Preserve the accepted
parents and close exposure-only fitting. The recommended same-checkpoint
emission/selection diagnostic is now executed read-only at the same step-15,672
artifacts; its witnessed malformed decisions are predominantly low-probability
draws from the model's own ranking. The follow-on selection-policy diagnostic is
now executed too: with the draw removed the sampled malformed clauses do not
recur, but greedy prose still misses the frozen acceptability bar (quaternion
2/5, ordinary pair 0/5) and the greedy source panel still fails 11/32 and 9/32
rows, so ranking/emission was the supported next target. The bounded
termination-weighted objective experiment is now executed and returns INERT: at
2.5× on sentence-final targets over 1,024 updates (4.19M targets per arm) it adds
only 3 unique source rows beyond a dose-matched plain continuation (0 unique
losses) and leaves sampled prose at0/5, while the plain dose already recovers12
of20 failing rows; guardrails hold and standard development NLL is slightly
worse for the treatment. Termination weighting is a weak lever at this dose; the
next rung is the state/read path. Admission pruning remains deferred.
The transformerless integer/table serving goal and D0-b/D4–D6 remain unchanged.

## Latest result: completed continuation, useful-language target unmet

The [completed result](language-continuation-result-2026-09-26.md) records both
continuous finals at step15,672 and all frozen downstream outputs completed at
23:04:43UTC. The final SSD snapshot is verified, including67 copied report/checkpoint
roots; the [closeout](language-continuation-result-2026-09-26.md#delivery-and-cost-closeout)
records storage, cost and protected delivery. Continuous prose is **0/5 quaternion and0/5 ordinary**; integer prose
is **0/5 and1/5**, against0/5 same-path parents. All four groups miss the fixed
3/5 and+2 improvement rule. Natural likelihood and Read/NoRead utility improve
without qualifying coherent language. Ordinary numerical/source-retention gates
also fail; quaternion has source regressions against the accepted parent and an
actual integer short-cycle. No candidate is promoted. The [next single work card](#next-single-work-card--failure-localization)
localizes these failures from existing records; it authorizes no fit or sweep.

## Parallel lab track: integer chat vehicle and hyperbolic reads, September26

A separate lab on branch `claude/blissful-wozniak-girwwq` followed the owner's
decisions of 2026-09-26 ([D10](DECISIONS.md)). An open instruct model, converted
and served without floating point or multiplier-based weight maps, is the interim
chat vehicle, reported as dense; hyperbolic geometry is the lead mechanism. This
track does not change the work card below.

- **Integer chat engine** (`crates/uor-r4-lut`, the audited SIMD crate
  `crates/uor-r4-simd`, the exporter `lut_export` and `lut-chat`). It has 4-bit
  table GEMV (AVX2/NEON), integer attention and integer sampling.
  - A 4.2M-parameter stand-in Llama exported with GPTQ scores +0.0034 nats with
    97.95% top-1 agreement against bf16.
  - A SmolLM2-135M-shaped model decodes 36.5–54.8 tokens/s on the 4-core x86
    sandbox.
  - M1 speed, energy and real SmolLM2 fidelity are unmeasured
    ([lab note §7](geometric-lab-phase2-2026-09-26.md#7-m3-the-integer-serving-engine)).
- **Hyperbolic cache memory.** A learned Lorentz cache over a frozen backbone
  beats equal Euclidean and dot caches. Its integer form keeps 99.3% of the float
  gain on the stand-in
  ([§8](geometric-lab-phase2-2026-09-26.md#8-m4a-work-card-a-learned-cache-memory-over-a-frozen-backbone)).
- **The native model's Lorentz read** ([cycle 3](hyperbolic-cycle3-2026-09-26.md)).
  At reduced scale (code corpus, width 128), the flat-start Lorentz read beats the
  retained Dot read.
  - At context 128 it wins in 3 of 4 seeds, by 0.018 nats on average; at context
    256 it wins by 0.051–0.061.
  - `uor-r4-integer` now serves the Lorentz read. The Dot contract, tables and
    arithmetic are unchanged.
  - The Lorentz read adds two learned signed 16-bit scalars (log β and δ) beside
    the ≤4-bit weight maps, plus a Q32 scale derived at load. They are not 4-bit
    parameters. This lab's native/multiplier-free clarification does not silently
    qualify their precision under D0-b; any serving adoption must state the
    exact parameter and arithmetic contract.
  - After 300 quantization-aware updates, 4-bit integer serving is within 0.035
    nats of equal float fine-tunes and keeps the Lorentz advantage (−0.062 and
    −0.059 nats)
    ([§11](hyperbolic-cycle3-2026-09-26.md#11-addendum-the-integer-lorentz-read)).
- **Owner-runnable M1 scripts.**
  - `scripts/lut-m1-chat.sh`: integer SmolLM2 chat, with an optional cache stage
    and joules per token.
  - `scripts/kappa-m1-pilot.sh`: the curvature drive test on real SmolLM2 heads.
  - `scripts/native-lorentz-m1.sh`: the full-scale native Dot/Lorentz comparison
    through integer serving.
- **Scope.** Development measurements on stand-ins and at reduced scale. No
  chat-quality, M1-energy or general-language qualification.

## Retained result: standalone integer serving and exact speedup, September25

The [standalone result](integer-serving-result-2026-09-25.md) delivers a Rust
CLI and stateful library session with bound tokenizer/model/tables, integer
categorical selection, and full256 attention. The serving dependency graph
excludes training, Candle and external model providers. The numerical source is
shared with training evaluation; no fitting or model change occurred.

All4,096 retained target probability hashes,128 source-output rows and341 source
decisions match exactly. Source answers remain28/32 quaternion and24/32 ordinary,
NoRead0/32 each, with zero lost correct rows. Both actual multi-call session
checks pass. Oversized appends/generation fail without mutating session state.

One profile-directed signed4 product-table optimization preserves all ten new
integer-sampled continuations exactly. Source batches improve52.97/53.15s to
8.02/8.01s; five-continuation batches improve11.93/10.76s to2.62/2.14s. These are
single before/after observations with two arms concurrent; timings include load,
prompt ingest, model, selection, hashes and JSONL, excluding final sealing.
Full256 Read model calls average3.695/3.698ms. Dense access/allocation persist.

The compiled numerical model, sampler and generation ranges contain no floating
arithmetic/conversion or numerical-value multiply/divide. Shape/address and clock
multiplications remain outside that narrower claim. No full-process multiplier,
D5 sparsity, geometric advantage, Hamiltonian dynamics, useful general language
or energy qualification is promoted. Actual prose still confuses entities and
roles. Session capacity remains256, with independently tokenized text appends.

**Completed continuation:** both arms reached the fixed continuous target
15,672 and completed the frozen evaluation, calibration, learned-code and integer
packet. The [final result](language-continuation-result-2026-09-26.md) misses all
four prose improvement targets and retains specific numerical/source failures;
neither candidate is promoted. The standalone-serving delivery above itself
performed no new fit.

[Bound result](../evidence/integer-serving-result-2026-09-25.json),
[exact comparison](../evidence/integer-serving-comparison-2026-09-25.json),
[all text](integer-serving-outputs-2026-09-25.md),
[instruction audit](../evidence/integer-serving-instruction-audit-2026-09-25.json),
[resources](../evidence/integer-serving-closeout-2026-09-25.json).
Optimized source `be223fc5`; executableSHA and full commit are bound in the result.
RDC ran DeepSeek review and concurrent local Rust workers. Focused unit/arithmetic,
training-compatibility and loaded-artifact checks executed; the canceled broad
implicit core build's integration tests remain NOT_RUN.

## Retained predecessor: integer attention and transport with full context

The [executed integer bridge](integer-execution-result-2026-09-25.md) loads the
unchanged accepted learned-code parents. Learned attention, R4/ordinary transport,
recurrent state and normalized vocabulary/copy output now execute with integer
model-value arithmetic. Context/access remain **256/full**; every tested step
checks all causal occurrences, reaching255 previous events at the window end.
No new training, admission change or fresh holdout occurs.

| Matched-feature execution | Quaternion | Householder pair |
|---|---:|---:|
| Integer / parent complete source answers | 28/32 /28/32 | 24/32 /24/32 |
| Previously correct answers lost | 0 | 0 |
| Integer NoRead complete answers | 0/32 | 0/32 |
| Maximum Read state drift; bound0.01 | 0.007324219 | 0.005859375 |
| Maximum Read probability drift; bound0.01 | 0.006691748 | 0.008505233 |
| Integer step, measured batch1 | 16.14 ms | 16.09 ms |

Read and NoRead both meet the prospective numerical bounds on **four full256
exposed calibration windows**,1,024 targets per mode. This is not the full976-window
natural-language gate. The32 source variants are the existing authored panel.
First attempt used the wrong F32 backend and retains its ordinary state-drift
failure. Rebuilding only with the original parent features restores historical
F32 source outputs; all4,096 integer target distribution hashes and all source/
continuation values remain identical between builds (excluding elapsed time).
Both comparisons took about112 seconds. No threshold changed.

The audited compiled numerical-value ranges contain no floating arithmetic or
model-value hardware multiply/divide. Five shape/address multiply/divide
instructions remain; whole-step compliance is not claimed. The host seeded
sampler remains floating, the prototype remains in the training crate, and
parameter access is dense with allocation. Measured integer calls are about
28–29 times slower than matched F32. Generated prose remains unreliable.
No useful language, geometric advantage, Hamiltonian dynamics, D5 sparsity or
energy claim is promoted.

**Historical next, now executed above:** integrate the retained computation into a standalone Rust serving
session with integer token selection, complete-generation cost and the same
full256 contract. Use exact-output optimizations only after locating actual
cost. Keep the ordinary arm and existing artifacts; no new model fit or admission
sweep is needed for that implementation. See the work card below.

Executed Rust source: `45da66d4a2e77a908411c7084e109f6a17472114`.
[Reports/bindings](../evidence/integer-execution-result-2026-09-25.json),
[all text](integer-execution-outputs-2026-09-25.md),
[independent row comparison](../evidence/integer-execution-comparison-2026-09-25.json),
[instruction inspection](../evidence/integer-execution-instruction-audit-2026-09-25.json),
[prospective plan](integer-execution-plan-2026-09-25.md),
[resources](../evidence/integer-execution-closeout-2026-09-25.json).
RDC ran DeepSeek review and concurrent Rust workers locally; independent agents
implemented arithmetic/evaluation and reviewed the exact compiled path. Ten new
focused checks plus two codec checks pass. No Kimi model or external training.

## Retained result: bounded admission complete; source retention fails

The [paired result](bounded-admission-result-2026-09-25.md) completes one shared
bounded-admission implementation and256 updates /1,048,576 fitted target visits
per arm at B16/T256. The original stricter physical-reserve guard checkpointed
at166; both arms resumed the same optimizer/data state for90 more updates.
Actual data hashes match in both segments. No learning or artifact was discarded.

| Comparison-tail NLL / complete source answers | Quaternion | Householder pair |
|---|---:|---:|
| Accepted learned-code parent | 2.114226 /28 of32 | 2.110880 /24 of32 |
| **Final orthant64** | **2.129595 /17 of32** | **2.124890 /17 of32** |
| Same final weights, full | 2.080041 /27 of32 | 2.069663 /25 of32 |
| Same final weights, recent64 | 2.119169 /27 of32 | 2.108628 /25 of32 |
| Same final weights, exact_cache64 | 2.146802 /23 of32 | 2.136447 /18 of32 |

Both frozen retention vectors are **PASS/PASS/PASS/FAIL/PASS**. First-noun losses
against the immediate parents are11/8 and against rung1 are11/9, above maximum2.
NLL, combined NoRead effect, limited noncollapse and mechanical/export checks
pass. Indexed complement utility also passes versus recent32 (17 additional
complete answers; late-position NLL gains0.110267/0.105152), but cannot override
retention failure. Same-weights full/recent64 recovery identifies the access
policy as a material cause. Neither a diagnostic override nor a lower average
loss promotes a failed candidate. The accepted learned-code parents remain current.

**Historical next, now executed above:** implement quantized R4 transport and the integer execution bridge
from the accepted learned-code parents, with full 256-token access and the
matched ordinary arm. Reuse the existing evaluator and retained evidence.
The proposed recent64 fit is withdrawn before execution; its same-weights
results remain diagnostics. Sparse admission is parked until new causal evidence
and an implementation need justify reopening it. Neither bounded-candidate
retention nor another selection sweep blocks full-access numerical implementation.
See the active contract below for the required deliverable and validation scope.

Executed source: `78fde5711ea01f80783eb9911e01197a61406d1a`.
Rejected but retained artifacts: `fit-{quaternion,householder_pair}-bounded-2/packed-model`
under `/Users/casey.allard/uor-r4-investigations/bounded-admission-20260925`.
[Arithmetic and sealed bindings](../evidence/bounded-admission-result-2026-09-25.json),
[principal decision](../evidence/bounded-admission-decision-2026-09-25.json),
[all outputs](bounded-admission-outputs-2026-09-25.md),
[plan/review](bounded-admission-plan-2026-09-25.md),
[resources](../evidence/bounded-admission-closeout-2026-09-25.json).
Three DeepSeek sessions and paired Rust jobs ran through RDC on this M1; no Kimi.
Training supervision totals24.67minutes;62 optimized focused checks pass in4.79seconds.
F32 dense parameter computation, allocations, unreliable prose and the256-token
ceiling remain. No geometric advantage, Hamiltonian dynamics, integer-serving,
terminal sparsity or energy qualification is claimed.

## Active execution contract

Owner-directed correction, September 25; governed by
[D9](DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)
and the [progress-control rules](agent-execution-policy.md#progress-control--owner-correction-september-25).

| Quantity | Retained execution baseline |
|---|---|
| Training / evaluation / session context | 256 / 256 / 256 tokens |
| Direct context access | Every causally available event within that window; `full` admission |
| Candidate cap | Up to the complete causal window, not an imposed 64-event subset |
| Read representation width | 64 coordinates; a vector dimension, not a token horizon |
| State width | 256 coordinates; separate from context and admission |
| Accepted starting artifacts | `learned-rounding-20260925/fit-{quaternion,householder_pair}-rounding-3/packed-model` |
| Evaluator | Existing `reference-evaluator-v2.json`; no new panel or holdout during implementation |
| Deferred branch | recent64 fitting and sign-index/width/hash sweeps |

**Completed exposure work card (#973 under #820):**

Owner resumed this milestone after storage consolidation. The
[prospective continuation plan](language-continuation-plan-2026-09-25.md),
[parent output judgments](../evidence/language-continuation-parent-quality-2026-09-25.json)
and [resource projection](../evidence/language-continuation-budget-2026-09-25.json)
freeze one further29,999,104-target dose per arm, steps8,348→15,672, with unchanged
full256 conditions. Two configured DeepSeek reviews completed through RDC;
[principal adjudication](../evidence/language-continuation-review-2026-09-25.json)
retains actual prose criteria and declines additional heuristic panels.
Preparation found a continuous-to-rounding interface prerequisite: explicit
fresh-calibration shadow parents with truthful zero-update provenance.
The adapter is implemented and [verified](../evidence/language-continuation-adapter-2026-09-25.json):
2 focused release checks pass; both actual step8,348 parents complete calibration,
packed export, integer bundle loading and an8-token interface exercise in about
3 seconds each, with zero optimizer/data advances. This is interface evidence,
not language retention. **Both arms completed the original continuous target
15,672 and the frozen downstream packet. The fixed prose targets remain unmet;
no candidate replaces the accepted parents.**
The first RDC-attached controller [failed](../evidence/language-continuation-interruption-2026-09-25.json)
at 23:35 UTC with a broken output pipe; its workers exited on signal 2 before
their first checkpoint. They logged 2,031/2,032 updates but saved no recoverable
weights or optimizer state. That attempt is retained as an execution failure,
not a model-quality result. A detached controller restarted the same full256
step8,348→15,672 dose from the preserved parents at 23:46 UTC. Three intermediate
checkpoints were scheduled to reduce recovery loss; only the final step is eligible for
evaluation. The necessary local time extension was recorded before restart.
At02:08UTC on September26, that pair then stopped cleanly at its shared-storage
allocation guard, saving verified checkpoints at10,480/10,488. The
[exact-state recovery](../evidence/language-continuation-recovery-2026-09-26.json)
retains both saved states. A detached resume started at02:37:55UTC; new updates
were observed in both arms. Only5,192/5,184 remaining updates lead to15,672.
A prospectively recorded local storage/time extension preserves the original
allocation baseline and machine reserves. At that recovery point, final
evaluation, rounding and candidate quality were NOT_RUN.
The next shared-cache stop at05:36UTC also exited cleanly and preserved all
fit3 updates at12,707/12,715. Four complete Rust seal checks passed.
[Fit4 recovery](../evidence/language-continuation-recovery-4-2026-09-26.json)
started at06:01:36UTC from those saved states for only2,965/2,957 remaining
updates. Internal allocation20GiB was recorded prospectively; the14:41UTC
cycle deadline, cumulative time ceiling and physical/RSS guards were retained
for fit4. Both remaining-fit processes enforced the projected six-hour limit.
At 07:22 UTC, fit4 hit the physical-space reserve and exited cleanly,
saving 13,939 / 13,949. All four Rust report/checkpoint checks passed. The
[sequential recovery](../evidence/language-continuation-recovery-5-2026-09-26.json)
started at 07:31:51 UTC and also stopped at the physical reserve at 07:35:46.
Quaternion saved all 61 new updates at step 14,000; both new Rust full-set
checks passed. Householder did not start and remained at 13,949. At the fit5
stop, **1,672 / 1,723 updates remained** to the original sole final 15,672.
B16/T256, two gradient shards, full causal access, Adam and sampler continuity,
and every acceptance criterion remain unchanged. Serial execution reduced
campaign RSS but did not resolve host physical-space instability. At the fit5
stop, automatic retries were paused pending owner resolution of competing
workloads or a new storage arrangement, without authorizing reserve reduction
or deletion. The prospectively recorded fit5 deadline 16:41 UTC and cumulative
limit 621,600,000 ms were budget caps, not a completion estimate. Final evaluation,
integer conversion and prose adjudication were NOT_RUN at that stop.

### Latest recovery6 status

The owner has authorized SSD cache offload and completion of the existing dose.
The [recovery6 record](../evidence/language-continuation-recovery-6-2026-09-26.json)
bound quaternion fit5 at **14,000** and Householder-pair fit4 at **13,949**;
Householder never started fit5. Fit6 has now saved both **TARGET_COMPLETE** finals
at **15,672**, retaining its **1,672 / 1,723** remaining updates respectively.
The combined actual segments complete exactly **7,324 updates / 29,999,104 target
visits per arm from 8,348**. Both optimizer and next-data clocks are15,672, with
64,192,512 cumulative target visits. The unsaved fit1 updates remain a charged
execution loss and are excluded from saved learning. B16/T256, full causal access,
two gradient shards, data, optimizer and every acceptance criterion remain fixed.
One model process ran at a time throughout downstream work; no extra continuous
dose or checkpoint selection was introduced.

The recorded prospective complete-work projection was **33,300 seconds**, including
cleanup/verification, sequential fit, conversion, evaluation/review/delivery and
stop margin. The recorded whole-cycle deadline is **September 27 at 02:41 UTC**, with
cumulative time limit **657,600,000 ms**. These are caps, not an ETA. The tracked
allocation ceiling is **32 GiB**, retaining the original whole-cycle baseline
and conservatively charging the relocated shared cache after its device/inode
change; this is not a measurement of internal-drive consumption. The separate
**32 GiB cache-offload allowance** and original **4 GiB snapshot allowance**
remain separate. Physical reserve **24 GiB + 128 MiB stop margin + 64 MiB
checkpoint headroom**, and existing RSS guards remain unchanged. Current model
parents and frozen executables remain at their existing internal paths.

**Cleanup/cutover: complete. Continuous fit6 and frozen postprocessing: COMPLETE.
Prose targets: UNMET in both arms; neither candidate is promoted.** The migration verified
45,329 files and their hardlink groups before removing the redundant internal
Cargo cache. Its immediate free-space increase was 18,462,396,416 bytes; later
host free-space changes are reported separately. The cache now resides on the
approved SSD behind the original paths. One regenerable compiler-probe metadata
file was snapshotted separately because Cargo refreshes it outside build locks;
source, models, original checkpoints and Docker data were preserved. Original
Cargo lock inodes remain in a tiny internal directory referenced from the SSD.

The detached controller launched at **17:11:38 UTC** and ran the arms sequentially.
Launch physical free space was 48,023,339,008 bytes and tracked allocation,
including headroom, was 21,699,153,920 bytes. These are retained launch readings;
physical/RSS/allocation guards remained active through both completed fits.
The 29,700-second remaining launch projection includes fit, conversion,
evaluation/review/delivery and stop margin; preparation is already charged.
The [conditional principal direction](language-continuation-direction-2026-09-26.md)
and [completed result](language-continuation-result-2026-09-26.md) guide the
existing-evidence localization decision. No new panel or automatic fit follows
from a monitor or the completed packet.

The [completed-fit and downstream record](../evidence/language-continuation-postprocess-6-2026-09-26.json)
binds both actual finals: quaternion completed at **18:45 UTC**, and the
Householder-pair control at **21:11 UTC**, each exiting0 without a resource stop.
The sequential supervisor phases took **5,609.99s / 8,779.09s** respectively;
these include child supervision and are separate from cumulative preparation,
prior attempts and delivery charges. The controller wrote its complete-final
index at21:11:37UTC and exited. Read-only downstream readiness accepted the
actual asymmetric lineage and fixed exposure. Postprocessing launched at
**21:16:54UTC**. Both full fit reports and both final checkpoint sets passed the
existing Rust verifier by21:17:35UTC (four exit0 records); the quaternion continuous
Read evaluation started at21:17:37UTC. The unchanged sequential Read/NoRead,
calibration, alpha512, integer/source and prose packet completed at23:04:43UTC.
Its [final adjudication](../evidence/language-continuation-result-2026-09-26.json)
retains the negative quality and numerical/source findings. Training completion
and seal verification do not establish language quality or geometric advantage.

### Completed continuous and integer prose result

The [actual continuous output review](../evidence/language-continuation-continuous-quality-2026-09-26.json)
finds **0/5 fully acceptable stories in each arm**, compared with0/5 for each
same-path parent. Principal and independent reviewers agree on the overall
failure to meet the fixed3/5 and+2 criteria; dimension-level disagreements are
retained. Some individual passages have understandable characters/events, but
unresolved roles, disconnected events and incomplete endings still prevent the
required conjunction. New characters and personification remain allowed; the
128-token cap is not automatically a failure when a complete stable clause exists.

Continuous comparison-tail Read NLL is **1.996490474 quaternion /
1.974724189 ordinary**; NoRead is **2.492064889 /2.472758698**. The corresponding
penalties,0.495574415 /0.498034509 nats on233,472 targets per arm, show context
utility at this evaluator's scope. They do not qualify coherent prose or
geometric advantage. This dose has not met its continuous prose objective.

The [integer output review](../evidence/language-continuation-integer-quality-2026-09-26.json)
finds **0/5 quaternion and1/5 ordinary**, again below the frozen threshold. Both
reviewers accept the ordinary lion story; disagreements over quaternion seed2017
progression and ordinary seed2014 completion are preserved and change neither
row acceptance nor the outcome. The [integrated result](language-continuation-result-2026-09-26.md)
separates prose failure from measured conversion/source failures and the quaternion
integer goose short-cycle. Different continuous/integer sampling policies prevent
attributing sample differences alone to conversion. Final delivery items are
listed only in the result's [delivery section](language-continuation-result-2026-09-26.md#delivery-and-cost-closeout).

### Executed: same-checkpoint emission/selection diagnostic (September 27)

- **Executed read-only** from the [predeclared plan](emission-selection-diagnostic-plan-2026-09-27.md)
  and [result](emission-selection-diagnostic-result-2026-09-27.md) at the same
  step-15,672 checkpoints, frozen prompts/seeds/policy and Read mode. The new
  additive `joint-emission-trace` instrument reproduced the retained packet
  exactly (`PARITY_EXACT`, 0 mismatches, maximum float delta0.0, all five
  stories in both arms); [evidence](../evidence/emission-selection-diagnostic-2026-09-27.json).
- **Witnessed result:** 8 of 12 witnessed decisions are low-probability draws
  that departed from a materially better-ranked alternative (2 near-ties,
  2 top choices, 0 copy-dominated). The witnessed malformed tokens were
  vocabulary-side, not copy-side.
- **Supported interface:** selection policy. The next work card, not started, is
  one separately authorized bounded same-checkpoint selection-policy diagnostic
  with no weight change; it must also explain the retained greedy source-panel
  regressions. No decoding/weight change and no candidate promotion follows; a
  better-ranked token is not a coherent alternate trajectory.
- **Stop:** this packet is closed. #973 and #820 remain open at their wider
  acceptance scope.

### Executed: same-checkpoint selection-policy diagnostic (September 27)

- **Executed read-only** from the [predeclared plan](selection-policy-diagnostic-plan-2026-09-27.md)
  and [result](selection-policy-diagnostic-result-2026-09-27.md): deterministic
  greedy replay at the same step-15,672 checkpoints. The greedy source panel
  reproduced the retained `story-probes.json` exactly (`PARITY_EXACT`, 0
  mismatches; complete answers21/32 quaternion,23/32 ordinary pair), and the five
  frozen prompts were replayed with the draw removed.
- **Witnessed result:** all five greedy trajectories diverge within the first
  three tokens, so the sampled malformed clauses do not recur; nevertheless
  greedy prose stays below the frozen acceptability bar in both arms (quaternion
  2/5, ordinary pair0/5) and the greedy source panel keeps failing11/32 and9/32
  rows. The independent evidence auditor returned identical verdicts.
- **Supported next target:** ranking/emission, on the concrete greedy regression
  rows (lost versus the accepted parent:9 and6 complete answers) and the two
  dominant modes (correct-noun-then-extra phrase13/20; wrong first noun7/20). No
  promotion and no weight/serving-decoding change; selection policy did not reach
  the positive branch.
- **Correction:** the plan's `5/2`/`7/4` citation is the new hard artifact's row,
  not this continuous panel's (`9/2` and `6/5`); the plan carries a corrigendum.
- **Stop:** this packet is closed. #973 and #820 remain open at their wider scope.

### Executed: termination-weighted objective experiment (September 27)

- **Executed** from the [predeclared plan](termination-objective-plan-2026-09-27.md)
  and [result](termination-objective-result-2026-09-27.md): four sequential fits from the
  step-15,672 parents (1,024 updates /4,194,304 targets each) comparing an additive
  `end_weight=2.5` on sentence-final targets against a dose-matched plain continuation.
- **Witnessed result:** `INERT`. The treatment uniquely resolves3 source rows (2
  non-termination +1 wrong-noun) with zero unique losses, but the plain dose alone
  recovers12 of20 failing rows, sampled five-prompt prose stays0/5 in all four finals
  (principal and independent audit agree), and the standard development NLL is slightly
  worse for the treatment. Guardrails hold.
- **Decision:** stop the termination-weighting branch; the residual entity/role collapse
  and cap-truncation failures place the next rung in the state/read path (cross-lab
  native radial reader comparison, or a conditional depth hypothesis). No promotion; the
  plan's INERT band is recorded as treatment-minus-control and the judgment sensitivity
  is disclosed.
- **Stop:** this packet is closed. #973 and #820 remain open at their wider scope.

The full256 baseline is finite; terminal D5 parameter sparsity remains open.
Standalone integer generation is **executed at the scoped numerical boundary**.
General language quality, complete-path arithmetic compliance and energy remain
separate obligations. One paired seed remains exploratory for geometry claims.

## Retained result: paired learned-code retention accepted

The [completed fixed-recipe learning](learned-rounding-result-2026-09-25.md)
adds512 alpha-only updates /2,097,152 fitted target visits per arm, B16/T256,
plus32,768 training-only normalization visits. Both projected parents and their
scales, bit widths, interfaces and architecture remain fixed. Resumes preserve
all learning; actual sample hashes agree across both arms in every segment.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Retained continuous reference | 2.090518499 | 2.064403221 |
| Prior projected packed | 2.150319798 | 2.147928712 |
| **New learned hard codes** | **2.114226169** | **2.110879800** |
| Hard minus continuous; allowance0.05 | **+0.023707670** | **+0.046476579** |
| Whole-prefix NoRead | 2.609608832 | 2.607989949 |

Both five-gate vectors are **PASS/PASS/PASS/PASS/PASS** at the original limited
scope. Combined NoRead penalties are+0.495383/+0.497110. All five real
read-enabled generations per arm avoid constant/short-cycle collapse. First-noun
losses against rung1 are2/1, within the frozen maximum2; full-context packed
reload probability delta is zero and seeded outputs agree. Principal arithmetic
verifies32 sealed roots,44 binding records, complete target identities and all
source-row changes. No fresh final holdout or checkpoint selection occurs.

Exact source completions are28/32 and24/32. **Both still lose3 previously correct
complete answers against rung1**, with gains reported separately. All ten free
generations remain semantically unreliable. The continuous references are the
original step8,348 anchors; new code candidates receive extra learning. This
one-paired-seed result does not establish equal-compute superiority, geometric
advantage, Hamiltonian dynamics, useful conversation/code, integer execution,
terminal D5 parameter sparsity or physical energy savings. Serving of these
packed artifacts is still F32 numerical emulation.

**Historical successor, now executed above:** implement the actual bounded serving admission rule in the common
training/session graph. Preserve exact occurrence identity, causality and NoRead;
keep full-context and matched ordinary/exact-cache controls. Count the complete
admission cost, then integrate observable quantized group transport and measure
accumulated state/output error. Integer execution and useful complete outputs
follow retained behavior under those constraints. No new diagnostic sweep.

Executed Rust source: `036eabcc86e453f581aa0563e2967cc1f7db96d5`.
Final artifacts: `fit-{quaternion,householder_pair}-rounding-3/packed-model` under
`/Users/casey.allard/uor-r4-investigations/learned-rounding-20260925`.
[Full result/bindings](../evidence/learned-rounding-result-2026-09-25.json),
[all responses](learned-rounding-outputs-2026-09-25.md),
[resolved recipes](../evidence/learned-rounding-resolved-recipes-2026-09-25.json),
[plan/research/review](learned-rounding-plan-2026-09-25.md),
[resource closeout](../evidence/learned-rounding-closeout-2026-09-25.json).

RDC ran two concurrent DeepSeek expert sessions and both local Rust fits.
No Kimi model or external training hardware was used. All42 focused optimized
Rust checks pass in3.59seconds; paired fit supervision totals48.16minutes.
A physical-reserve stop and a configured process-time boundary checkpointed and
resumed without repeated updates. No artifact/worktree was deleted. The ledger
charges the whole preparation/build/fit/evaluation/review/delivery cycle.

## Retained result: parameter/interface precision diagnosis complete

The [fixed-parent comparison](precision-factorial-result-2026-09-25.md) evaluates
all four precision combinations in both retained step-8,348 parents. Both new
FF/QQ endpoint pairs reproduce every retained target record and response field
apart from elapsed time. No training, scale search or new holdout occurs.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Floating parameters / floating interfaces (FF) | 2.113220567 | 2.098025647 |
| Quantized parameters / floating interfaces (QF) | 2.150291052 | 2.147912004 |
| Floating parameters / quantized interfaces (FQ) | 2.113217828 | 2.098039459 |
| Quantized parameters / quantized interfaces (QQ) | 2.150319798 | 2.147928712 |

At floating interfaces, parameter quantization costs **+0.037070/+0.049886**
nats/target. Interface quantization costs **−0.000002739/+0.000013812** at
floating parameters; interaction is **+0.000031484/+0.000002896**. Parameter
cost dominates in both arms, the tune prefix and every comparison quarter.
Even QF misses the original +0.05 allowance against continuous. This diagnoses
these fixed forward paths; it does not isolate individual tensors, predict
retraining recovery or establish geometric advantage.

Small mean interface loss is not behavioral identity: FF→FQ changes nine of ten
free generations, and QF→QQ changes eight of ten. All 40 free generations and
256 source responses are retained in the [complete output record](precision-factorial-outputs-2026-09-25.md).
Exact source completions, in FF/QF/FQ/QQ order, are **25/22/25/23** of 32 for
quaternion and **28/22/28/22** for ordinary; row gains and losses stay separate.
Prose remains semantically unreliable. Both prior quantized recipes remain
rejected and all computation here remains F32 emulation.

**Historical successor, now executed above:** [one paired learning of neighboring integer-code choices](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
with fixed parents/scales/bit widths/interfaces, the full 256-step recurrent
language objective and training-only calibration. Freeze one recipe and complete
resource projection before fitting; decide once on the final hard export/reload
using the original five gates. No scale/per-tensor sweep or automatic fallback
is adopted. Success would permit trained bounded admission/transport, followed
by integer execution and useful complete outputs; it would not itself qualify
them.

Executed Rust source: `49dfd0b15486ba3a7a763c598257f651e857279f`.
Artifact container: `/Users/casey.allard/uor-r4-investigations/precision-factorial-20260925`.
[Full arithmetic and bindings](../evidence/precision-factorial-result-2026-09-25.json),
[endpoint receipt](../evidence/precision-factorial-endpoints-2026-09-25.json),
[resource closeout](../evidence/precision-factorial-closeout-2026-09-25.json),
[independent review and primary research](precision-factorial-review-2026-09-25.md).
Eight model evaluations took about three minutes of concurrent pair wall time;
25 focused tests took 2.16 seconds, with about seven minutes of compilation.
Whole-cycle accounting includes preparation, analysis, review and delivery.
The owner's September 25 cost direction makes DeepSeek the default for all
specialists, including architecture; Kimi requires a new explicit owner request.

## Retained result: D8 projected continuation completed and rejected

[PR #1391](https://github.com/UOR-Foundation/uor-r4/pull/1391) delivers the
[completed paired result](projected-recurrent-result-2026-09-25.md) under the
unchanged [prospective plan](projected-recurrent-plan-2026-09-25.md).
Each arm continues its preserved fully quantized midpoint **7,836 → 8,348**,
adding **512 updates / 2,097,152 target visits** at matched B16/T256. Projection
is applied at entry and after every AdamW update, preserving moments, clocks,
fixed scales and all five original gates. There is no new checkpoint selection.

| Comparison-tail NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Matched continuous continuation | 2.090518499 | 2.064403221 |
| Original unprojected packed QAT | 2.149631373 | 2.144230654 |
| New projected packed model | 2.150319798 | 2.147928712 |
| Same projected shadows, quantizers disabled | 2.113220567 | 2.098025647 |
| Projected packed whole-prefix NoRead | 2.637288928 | 2.618333390 |

Both packed-minus-continuous gaps (**+0.059801/+0.083525**) exceed **+0.05**,
so both candidates are rejected for retention. The other four frozen gates pass
at their declared scope: combined NoRead penalties **+0.486969/+0.470405**,
five noncollapsed generations per arm, two/one lost rung-1-correct first nouns,
and loaded parity/finite normalization/provenance. Ordinary's former short cycle
is absent on this panel. Exact completions are **23/32 and 22/32**; source gains
never cancel losses. Prose remains semantically unreliable.

Entry projection brings **41,184/43,770** out-of-range shadows to zero while
preserving all hard codes, all actual full-context probabilities and optimizer/
sampler state. Both final models still have zero clipped parameter coordinates.
Shadow likelihood improves and packed-versus-shadow state RMS falls, yet packed
likelihood does not improve. The [full evidence](../evidence/projected-recurrent-result-2026-09-25.json)
retains every response, row comparison, quarter and artifact binding. This is
one paired seed on exposed development, with no fresh holdout opened.

The [previous unprojected result](quantized-recurrent-result-2026-09-25.md),
[PR #1390](https://github.com/UOR-Foundation/uor-r4/pull/1390), remains rejected
at its original gates, including the ordinary period-two cycle. Its final
continuous and packed artifacts are preserved without rewriting that outcome.
All model paths here remain F32 emulators: integer serving, D5 parameter
sparsity, Hamiltonian dynamics, general language and energy are unqualified.

### Successor diagnosis completed

The recommended fixed-checkpoint precision comparison is complete above. Its
[one code-choice successor](precision-factorial-result-2026-09-25.md#recommended-next-learn-rounding-decisions-within-the-existing-parameter-grids)
is the current NOT_RUN recommendation. This projected experiment's artifacts
and rejection remain unchanged.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/projected-recurrent-20260925`.
Final roots: `fit-{quaternion|householder_pair}-projected-1/checkpoint-final`
and `final-packed-{quaternion|householder_pair}-1`.
All Rust model/diagnostic runs use source
`66349cdb69883dd7d438c76394c22f4e19000c73`; report roots are sealed.
[Resources and executed checks](../evidence/projected-recurrent-closeout-2026-09-25.json),
[independent RDC review](projected-recurrent-review-2026-09-25.md).
The two CPU fits complete concurrently in about36 minutes on the same M1;
22 focused optimized Rust tests passed. Queue acknowledgements execute no tests.

## Retained integer-path result: A4

Four matched eight-epoch continuations consume 4,499,104 new token updates and
185.945 seconds of joint fitting. A4 selects the correct source on 2/24 new first
decisions in each arm; both controls select 0/24. All four produce **0/12 complete
correct read-enabled answers**. Loaded A4 retains only 28/167 C120 and 24/167 2I
admitted fit sources despite each eligible local update succeeding immediately.
Development bits/token is C120 A4/control **6.839477/6.799839**, and 2I
**6.798995/6.872785**. There is no model promotion.

The dedicated address coefficients stay fixed, but shared State updates and
causal inputs change actual fine codes. This corrects the original A4 report's
freeze wording. All 112 generation rows and eight full files per arm reproduce
in independent replay. 33 focused checks passed. See the [A4 result](integrated-attention-a4-result-2026-09-24.md)
and [compact evidence](../evidence/integrated-attention-a4-result-2026-09-24.json).

## Retained reference and shared evaluator

**D8 rung 0 is complete at its declared reference/comparator scope.** The shared
offline Rust tool reproduces #1017's full 249,856-target development NLL at
**1.580241190 nats**, within `1.173e-7` of the historical value. All five actual
seeded continuations reproduce all **582 generated IDs**, decoded text and stop
reasons. Batch isolation and future-input prefix checks have zero measured error.
The shared forward path retains its CPU language-gradient/parity check; the
previous CPU/Metal integrity results remain preserved.

On the 233,472-target comparison tail, mean NLL is **1.574024** for #1017,
**2.405627** for the new normalized, count-pruned interpolated 5-gram, and
**2.391786** for that 5-gram plus causal cache. Lower is better. Count fitting uses
both inherited training stores, totaling 149,996,416 raw IDs. Selected discount
is 0.9 and cache mixture 0.05. Four report roots are sealed and independently
verified; every scored row matches the actual input/target population.
[Result and interpretation](reference-baselines-result-2026-09-24.md),
[compact evidence](../evidence/reference-baselines-result-2026-09-24.json),
[evaluator v2](reference-evaluator-v2.json).

This is previously exposed development: the comparison tail is separate from
current count calibration but was used in historical neural checkpoint selection.
Five exact replays are not five correct answers; outputs retain repetition,
semantic drift and three capped continuations. #1017 is an ordinary floating-point
transformer used offline, never target serving. No neural optimizer step or
native model promotion occurs. The reference advantage does not isolate attention
or geometric causality. #1014 retains the historical attention-off evidence at
its original scope.

## Retained result: D8 rung 1 complete at its engineering scope

The [joint recurrent-memory campaign](joint-recurrent-result-2026-09-25.md)
completed **29,999,104 target visits and 7,324 updates per arm**. This includes a
matched 8,617,984-visit warmup at context 64 and 21,381,120 visits with training
and evaluation both at context 256. The owner correctly challenged the mismatch;
weights, AdamW moments and clocks were preserved across its declared correction
and two disk-guard checkpoints. Both final save/reload loss differences are zero.

Both arms selected final step 7,324 from the prescribed midpoint/final choices
using recorded tune-prefix scores **before** population evaluation. The Rust
comparison joins all 249,856 targets and reproduces all 21 partition means.
[Selection](../evidence/joint-recurrent-selection-2026-09-25.json),
[full result and actual responses](../evidence/joint-recurrent-result-2026-09-25.json).

| Comparison-tail NLL, nats/token | Read enabled | Whole-prefix NoRead |
|---|---:|---:|
| Quaternion recurrent learner | 2.110368 | 2.592991 |
| Matched ordinary recurrent learner | 2.085241 | 2.561712 |
| Count/cache baseline | 2.391786 | — |
| Historical offline transformer reference | 1.574024 | — |

Both learners improve their retained loss, beat count/cache, and generate varied
loaded text without constant or short-cycle collapse. They pass the **frozen
rung 1 engineering gate**. Text still shows semantic drift, role confusion,
malformed words and repetition; sustained coherent prose and useful general
conversation are not qualified. The read intervention removes both recurrent
value feedback and pointer-copy probability; the 0.482623/0.476471 loss penalties
establish their combined contribution, not isolated geometric or distant access.

On 16 frozen source-edit pairs, quaternion produces 28/32 exact completions and
ordinary 23/32; first-noun correctness is 28/32 versus 31/32. Both NoRead arms
produce 0/32. The strict-answer difference includes ordinary over-continuation;
it is not a geometric retrieval win. Prompts are only 56–62 tokens. Neither the
probe nor later-position likelihood isolates retrieval beyond 64 positions.
This is one paired seed on exposed development, with no fresh final holdout.

The learner executes quaternion transport in its prediction graph, with shared
language credit through state, contextual Q/K/V reads/writes and normalized
vocabulary/copy output. It remains **floating-point offline Rust**, with dense
affine maps and full soft context access. Integer export, bounded prime/zeta
admission, exact H4 serving, Hamiltonian dynamics and energy are unqualified.

## Retained rung 1 identities

These parents and their original acceptance remain preserved. The latest rung 2
result and next comparison above supersede their earlier scheduling statement.
The optional 600-cell diagnostic remains NOT_RUN.

Artifact container:
`/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924`.
Selected roots: `fit256-quaternion-3/checkpoint-final` and
`fit256-householder_pair-3/checkpoint-final`; all final fit/evaluation/comparison
roots are sealed. Executed source: `ad4e639fedecf9d490035f9986be736117bd3e29`.
The result receipt binds the exact executable, source, checkpoints and data.

## Retained rung 1 delivery/resources and programme limits

The preceding correction was protected [PR #1387](https://github.com/UOR-Foundation/uor-r4/pull/1387),
merged as `942645b264f73ec49507cffd7c2f4cbd95de3aa2`. Rung0 was delivered through
protected [PR #1388](https://github.com/UOR-Foundation/uor-r4/pull/1388), merged as
`c84b198df3b47bc8326307dd9964dc9c87cb830c`. The completed learner campaign is delivered through
[protected PR #1389](https://github.com/UOR-Foundation/uor-r4/pull/1389). Queue compatibility acknowledgements
execute no tests; local executed checks carry validation.

The [campaign closeout](../evidence/joint-recurrent-closeout-2026-09-25.json)
charges measured complete-cycle wall time once; overlapping training-process
seconds are reported separately. A measured delivery tail is charged after the
committed cutoff. Eight focused release checks and all actual population/generation
runs passed their stated execution checks; queue labels are not those results.

RDC used the same local 8-core, 16-GiB M1. Measured CPU/GPU profiles selected two
sequence-gradient workers per arm, both arms concurrent, with nested backend
threads limited to one. Sustained final fitting achieved about 1,362/1,365 target
visits per second; Metal and four workers per arm were slower. Final-phase peak
sampled combined RSS was 6.01 GiB. No energy or serving-throughput claim follows.

The excessive 26.1-GB disk guard was corrected prospectively to **20 GiB reserve
plus 128 MiB stop margin and 64 MiB checkpoint headroom**. The final phase's
minimum measured free space was 25,032,024,064 bytes, above that stop. Inspected
inactive compiler intermediates reclaimed 3,652,976,640 physical bytes in the
main cleanup; all models, source, binaries, reports and worktrees remain. Exact
gross new compiler allocation is **UNRESOLVED**; retained size and free-space
measurements do not certify the gross-storage ceiling. No paid compute was used.

The old #1017 revealed test remains a regression set. Persistent sessions still
need exact posting membership preserved through saturated-page eviction/restore.
Final integer export, geometry attribution, useful complete outputs, terminal D5
parameter access and physical energy remain gates.

## History and authority

The complete previous 4,324-line state record is preserved in the
[dated archive](current-state-archive-through-2026-09-24.md), with all original
relative evidence links. Read scoped history as needed. Start routine work from
this page, the [plan](project-track.md), [decisions](DECISIONS.md), and the exact
source/artifacts for the active rung; do not restart a whole-project survey.
