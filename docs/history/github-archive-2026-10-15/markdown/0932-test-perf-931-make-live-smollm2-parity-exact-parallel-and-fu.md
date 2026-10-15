# #932 — test/perf(#931): make live SmolLM2 parity exact-parallel and fully observable

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-24T14:06:47Z
- **Updated:** 2026-08-25T17:35:55Z
- **Closed:** 2026-08-25T17:35:20Z
- **Milestone:** R4 Intelligence / S2 — Selective prediction
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/932
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #931
Roadmap context: #823 / milestone 14

## Problem

The live SmolLM2 parity suite in `tests/bdd.rs` is a required non-vacuous local verification input when the pinned source and compiled bundle exist. On 2026-08-23/24, `cargo test --workspace --no-fail-fast --offline` entered its exact teacher path and remained there for more than 34 hours. The harness exposed no forward counter, current stream, rolling rate, ETA basis, worker count, CPU/RSS reading, or durable partial report. A one-second read-only sample ultimately located it in S4 `timed_teacher_generate`, still executing the exact `uor-matmul` path rather than hung.

The old execution shape is unacceptable: prompt replays are serial, S2/S3/S7 repeat equivalent live-teacher work, and S4 runs independent benchmark repetitions sequentially. Autoregressive dependence inside one trajectory does not require the harness to be single-stream.

## Goal

Make the fixture-present parity harness exact-parallel, bounded, and fully observable. By default it must use all logical CPUs available to the process across independent prompts, trajectories, and shared-weight exact batches/workers, while preserving bit-identical logits, top-k results, generated tokens, metric reductions, and verdicts. A run must continuously report enough state to distinguish slow progress, saturation, a stall, UNAVAILABLE evidence, and failure.

## Execution contract

### Scope

- Replace the single-stream live-teacher orchestration with a bounded scheduler.
- Run independent teacher-forced prompts and independent generation trajectories concurrently.
- Use shared immutable teacher weights; do not load one 260 MiB weight copy per worker.
- Use the existing exact batched teacher primitive and/or exact output-row partitions where measured useful. Bound combined lanes, batches, and intra-forward work so nested oversubscription cannot occur.
- Default the effective worker budget to `available_parallelism()`; expose explicit reproducible overrides.
- Bound each worker's sequence state to the actual prompt/generation horizon rather than the model's maximum context.
- Build one content-bound teacher transcript for the pinned prompt set and reuse it for S2/S3/S7 instead of repeating teacher forwards. Reduce results in canonical prompt/position order.
- Execute S4 as multiple concurrent generation streams and report aggregate throughput. Apply the same stream count to compiled-runtime timing so ratios compare the same workload shape.
- Add a hard, configurable wall-clock ceiling checked between forwards; crossing it produces ABORTED/FAIL evidence, never an open-ended multi-day run.
- Preserve the deployed runtime, artifact bytes, inference kernel, κ identities, thresholds, and teacher arithmetic.

### Required instrumentation

Emit periodic human-readable status plus JSONL events during the run, and a final machine-readable JSON report under an untracked target directory (or caller-selected path). Flush periodic events so an interrupted run still has evidence.

The report/event schema must include:

- suite/run identity; source, artifact, store, graph, tokenizer, and corpus presence/CIDs where available;
- availability state per conditional fixture: AVAILABLE, UNAVAILABLE, FAILED, or NOT_RUN;
- actual configured budgets and the actual tokenized work derived from them;
- phase/scenario, stream/run id, queue depth, active/completed/failed work, exact teacher forwards completed/total, and per-stream progress;
- requested/effective worker count, batch width, intra-forward worker count, logical/physical cores, architecture/OS, model geometry, exact backend/kernel/ISA identity where observable;
- phase and total elapsed time, rolling and cumulative forwards/sec, seconds/forward, completed-token throughput, ETA with an explicit basis/status, and longest active-forward age;
- process CPU percentage/time, current and peak RSS, and virtual memory where the host exposes them;
- warm-up versus measured work, per-run token counts/durations/throughput, ordered reduction result, and final speedup;
- final PASS, FAIL, UNAVAILABLE, ABORTED, or NOT_RUN state with the exact reason;
- the path of the durable report and a versioned schema id.

Telemetry is empirical evidence and must not enter deterministic artifact bytes or alter the number/order/arithmetic of normative teacher evaluations.

### Configuration

Document at least:

- `R4_PARITY_WORKERS` (default all available logical CPUs);
- `R4_PARITY_BATCH_PER_WORKER` or the selected bounded batching control;
- `R4_PARITY_STREAMS` (default sufficient to occupy all workers);
- `R4_PARITY_PROGRESS_EVERY_SECS`;
- `R4_PARITY_MAX_WALL_SECS`;
- `R4_PARITY_REPORT`;
- existing position/generation/run budgets, with actual rather than stale example counts.

Invalid zero/out-of-range values must fail clearly rather than silently falling back.

### Non-goals

- Replace exact `uor-matmul` outputs with Accelerate or approximate arithmetic.
- Relax parity floors, κ pins, fixture requirements, or required repository gates.
- Claim semantic/runtime performance from a synthetic microbenchmark.
- Change deployed serving behavior, artifact formats, no_std/runtime allocation guarantees, or conformance claims.
- Preserve single-stream latency as the S4 measurement. This issue intentionally measures the requested concurrent harness workload.

## Acceptance criteria

- [ ] A deterministic unit fixture proves 1/2/4/8-worker and supported batch schedules produce bit-identical raw logits, top-k rows, generated tokens, transcript bytes, metrics, and ordered reductions.
- [ ] Worker forks share the immutable weight allocation while retaining isolated mutable KV/state.
- [ ] The live fixture cheap probe reports exact equality at every tested configuration and samples 1/2/4/8 workers without launching the full suite.
- [ ] The best exact configuration achieves at least 4.0x aggregate forward throughput over one worker on the same live probe; target is at least 6.0x.
- [ ] Reachability arithmetic projects the default fixture-present live suite below eight hours before it may launch. If it does not, the full run is forbidden and this issue remains a truthful negative/blocker.
- [ ] During any live run, a durable progress event appears at the configured cadence even when no forward has completed.
- [ ] A planted slow/stalled worker, missing fixture, invalid budget, worker failure, and wall-clock abort each produce distinct non-PASS evidence.
- [ ] S2/S3/S7 consume one shared content-bound prompt transcript; tests prove no hidden duplicate teacher evaluation.
- [ ] S4 uses concurrent streams and reports aggregate teacher and compiled-runtime throughput over identical stream counts.
- [ ] Fixture-present end-to-end parity completes inside the predeclared bound with non-vacuous verdicts, or records an allowed evidence-backed negative result without claiming PASS.
- [ ] Docs and current suite comments describe actual counts/concurrency and no longer call missing fixtures a pass.
- [ ] Existing exactness, model-source, BDD, conformance, deterministic-build, wasm, and repository gates remain green.

## Falsifiers and negative controls

- One-worker reference versus reordered completion and 2/4/8-worker execution.
- Worker count greater than work count and batch remainder handling.
- One stream forced to sleep while heartbeat telemetry continues.
- Worker panic/error propagation; no partial metrics accepted.
- Missing/corrupt source, graph, corpus, or report path.
- Zero/invalid budgets and a wall-clock ceiling shorter than one work unit.
- Counter total intentionally misdeclared in a fixture; finalization must refuse.
- Report write failure must be visible and cannot be labeled PASS.

## Pre-registered performance run contract

- **Metric/current value:** exact live-teacher forwards/sec and projected wall time for the fixture-present default parity suite. The aborted serial run exceeded 34 hours; S2 observed 36 exact forwards in about 4h29m, approximately 448 s/forward. S4 alone declares hundreds of forwards under the old sequential 3-run shape.
- **Reachability ceiling:** with `W` independent exact workers, compute-only speedup is at most `W`; shared memory bandwidth and scheduling lower it. Shared-weight batching can reduce weight traffic, so worker×batch configurations are measured rather than assumed. The machine exposes 8 logical CPUs, giving an 8x worker ceiling before batching benefits.
- **Binding cheap instrument:** load once, run the same real-model forward workload at 1/2/4/8 effective workers and selected bounded batch widths, compare exact output bits at every point, and record CPU/RSS. Cost ceiling 60 minutes; no default full suite is launched by this probe.
- **Required verdict:** exact equality throughout, at least 4.0x measured aggregate throughput, all configured workers observed active, and a projected full default runtime below 8 hours. The target is at least 6.0x and materially below the 8-hour ceiling.
- **Exit rule:** stop the probe after all predeclared configurations or immediately on any exactness/resource failure. Do not retune the acceptance floor after results.
- **If positive:** select the fastest exact bounded configuration, publish the report, then run the instrumented fixture-present parity suite under the hard wall-clock ceiling.
- **If negative:** do not launch the full suite; retain the measurements and continue in-scope scheduler/batching/row-partition work until the gate passes or close with an explicit performance limitation requiring a maintainer decision.
- **Cost:** probe at most 60 minutes, one host, at most all local CPUs and a pre-measured bounded state allocation. Full run only after the projection qualifies, with an absolute 8-hour wall-clock limit and continuous progress evidence.

## Compatibility, conformance, and claim boundary

This is host-side verification infrastructure and exact source-teacher execution only. It changes no deployed-serving path, graph/runtime format, migration policy, or normative multiplication-free kernel. Parallel completion order is not a mathematical claim: every output element/trajectory remains independently exact, and all observable reductions return to canonical input order. RF-29 remains the existing parity evidence mapping; this issue adds no new runtime capability id and never edits generated `CONFORMANCE.md` directly.

## Expected files

Likely scope includes `crates/uor-r4-model-source/src/lib.rs`, its focused tests, `tests/bdd.rs` and/or a dedicated parity support module, the teacher-parity Gherkin text, `docs/CONFIGURATION.md`, the relevant testing/measurement record, and current guidance where the old serial/count assumptions are stated. Exact paths are subject to overlap review before staging.

## Verification

Issue-specific unit/integration tests first; planted negatives; live 1/2/4/8 probe; fixture-present bounded parity; then repository-required fmt, clippy, workspace tests/no-fail-fast, no_std ladders, claim wording, conformance/generated-file determinism, doc tests, deterministic rebuild, κ with fixture check, wasm build when model-source/core routing is touched, and applicable operation/allocation scans. Missing fixtures are UNAVAILABLE, never PASS.


## Comments (7)

### comment 5397311918 — 2026-08-24T15:16:34Z — Casey-allard

Design clarification from the active implementation review:

The qualified path MUST NOT be a single generation stream, even if one stream's matrix work is internally multithreaded.

The normal host configuration has two independently enforced dimensions:

- S = R4_PARITY_STREAMS: independent private-state teacher/generation trajectories simultaneously in flight (default: all available logical CPUs; on the binding M1 host, S=8).
- W = R4_PARITY_WORKERS: persistent exact output-row CPU workers (default: all available logical CPUs; on the binding M1 host, W=8).

A teacher forward_batch is one physical call only in the accounting sense; it must advance all S independent states together through shared immutable weights. This is S-stream execution, not a serial trajectory. Compiled candidates must execute the same S-wide logical workload with no backend overlap or oversubscription beyond W.

Admission and final evidence must separately prove:

1. configured/effective/current/peak active streams;
2. configured/effective/current/peak exact row workers;
3. S=8 stream occupancy in every 1/2/4/8-worker probe point;
4. bit-identical per-position logits, greedy choices, top-k order, state, and transcript identity across worker counts;
5. complete physical-batch/logical-forward/matrix/tile/cell/scalar-term accounting;
6. CPU/RSS/elapsed/rate/ETA/stall/deadline events at an independent durable heartbeat cadence;
7. >=4x configured-worker speedup and <=8h safety-adjusted full-suite projection.

Any collapse to one active stream, incomplete worker participation, lost/failed stream, missing fixture, telemetry failure, equality failure, inadequate speedup, or excessive projection is a fail-closed NOT_RUN / REFUSED verdict. No full parity run starts before this gate qualifies.


### comment 5397456711 — 2026-08-24T15:27:08Z — Casey-allard

Pre-measurement addendum to the binding #932 run contract:

- Full-run qualification requires W == S == available_parallelism. On the binding M1 host this is eight exact workers and eight independent streams. Smaller/larger overrides remain useful diagnostics but cannot authorize the full suite.
- The probe must reconcile observed batched-matrix width and exact matrix/tile/cell/scalar-term totals against the owner exact_forward_plan(S). Declaring states.len() is not sufficient proof of multistream execution.
- Trace evidence is shape-bound: every configured position, stream, raw-logit word, top-k row, greedy choice, and persistent-state record must be present.
- Sustained process use is binding, not only a momentary high-water mark: for W > 1 on the binding macOS host, mean CPU core-equivalents divided by W must be at least 0.75 (target 0.90), with measured CPU-time evidence available.
- S4 uses S distinct deterministic seeds derived from the pinned prompt set, normalized to one common nonzero length so the exact teacher batch remains full-width. The teacher, legacy runtime, and graph runtime receive the same seed for each lane. Per-lane output CIDs and distinct private-state/completion evidence are retained; identical fan-out cannot satisfy the multistream contract.
- Compiled-engine stream-worker occupancy and teacher exact-row-worker occupancy are separate fields. Planned occupancy cannot be reported as observed occupancy.

These conditions are fixed before the live probe. Any mismatch is NOT_RUN / REFUSED; thresholds will not be weakened after seeing results.


### comment 5398098348 — 2026-08-24T16:19:35Z — Casey-allard

### Bounded live-probe run contract — 2026-08-24

- **Metric and current value:** exact aggregate logical forwards per second for one full-width `S=8` teacher batch; the aborted old path observed about 448 seconds per serial logical forward and exceeded 34 hours without a verdict. No new-scheduler empirical rate is established yet.
- **Reachability ceiling:** the host exposes 8 logical and 8 physical CPUs, split 4 performance plus 4 efficiency cores. Row-only worker parallelism has an 8x worker ceiling; shared-weight `S=8` batching may additionally amortize weight traffic, but no benefit is assumed before measurement.
- **Binding cheap instrument:** one pinned 269,060,552-byte SmolLM2 source load; the same `S=8` private-state, maximum-prefix forward at `W=1/2/4/8`; complete logits/top-k/greedy/persistent-state trace and exact owner-plan counter reconciliation; CPU-time/RSS sampling and independent 10-second durable heartbeats.
- **Required verdict:** bit-identical traces at every point; all 8 streams at every point; every configured row worker observed; configured `W=8` speedup at least 4.0x over `W=1`; mean CPU core-equivalents divided by 8 at least 0.75; safety-adjusted full default-suite projection strictly below 28,800 seconds.
- **Exit rule:** stop after the predeclared sweep, on the first exactness/resource/telemetry failure, or at the fixed 3,600-second probe ceiling. No threshold retuning after results.
- **If positive:** publish the content-bound report, then run the fixture-present instrumented suite at exactly `W=S=8` under its 28,800-second hard ceiling.
- **If negative:** do not launch the full suite; retain the typed report/events and continue only bounded in-scope scheduler/kernel optimization or record the exact limitation for maintainer decision.
- **Cost:** at most 60 minutes, one model load, at most all 8 local CPUs, bounded private state at the conservative maximum context used by the configured parity budget. The probe does not touch deployed serving or artifact bytes.

Command:

```text
R4_PARITY_SOURCE=/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct cargo test -p uor-r4-model-source --lib --offline live_exact_multicore_probe_emits_json_and_preserves_bits -- --ignored --nocapture
```

No full parity run is admitted before a `QUALIFIED` report validates against the current source, host, executor, build, budgets, and strict gates.

### comment 5398824740 — 2026-08-24T17:23:00Z — Casey-allard

### Superseding pre-measurement optimization amendment — 2026-08-24

No #932 live probe or fixture-present parity measurement has run. The earlier fixed-shape admission conditions are therefore superseded before empirical evidence exists; their comments remain as historical rationale.

The objective is now the shortest exact end-to-end path to a decision that can unlock #931—not a prescribed worker count, stream count, utilization percentage, speedup multiple, or sample count.

#### Binding work and claim boundary

- Preserve eight canonical prompt/lane identities, shared immutable weights, private mutable state, exact raw-bit outputs, canonical reductions, complete counters, durable heartbeats/events, hard deadlines, and fail-closed fixture/report handling.
- S2/S3/S7 still consume one 36-logical-forward content-bound transcript. S4 will also reuse a retained canonical prefix state from that transcript instead of paying a new teacher prefill.
- Remove the independent eight-token teacher warmup: the transcript has already exercised six full exact physical batches. If S4 is run without that transcript, building the same transcript is its preparation and is reported separately.
- Replace three 128-token S4 waves with one causal measured continuation that starts at one decode step per lane and may extend deterministically through 2, 4, then 8 cumulative steps only while more work can change the compiled-vs-teacher verdict. Eight is the hard live ceiling for this issue. Exact determinism/repeatability remains established by focused structural tests rather than repeated live-model waves.
- Report preparation, decode, and one-shot elapsed time separately. The empirical claim is limited to the actually executed concurrent interval; no three-run median or broader sustained-performance claim is made.
- All planned/executed logical forwards, physical batches, matrix calls, row tiles, output cells, scalar terms, lane CIDs, state CIDs, output CIDs, elapsed/rate/ETA basis, CPU/RSS, workspace allocations, and stop/extend reason are evidence fields.

#### Adaptive exact tuner

- Do not run the old W=1/2/4/8 full-model sweep and do not perform a full-model warmup per candidate.
- Hold scientific work fixed at eight canonical logical lanes and S=8 shared-weight batching for this issue. Compare W=4 and W=8, the two plausible M1 scheduling shapes, over identical live-model work; choose the faster exact result with a deterministic tie-break. W=1/2 exactness remains covered by deterministic planted tests, not repeated expensive live work.
- Prestart worker pools/backends with bounded cheap work outside measurement. Record this cost and exclusion.
- Speedup versus W=1 and mean CPU utilization remain diagnostics, not qualification thresholds. W and S are independent; full-run admission no longer requires W=S=available_parallelism.
- A future candidate is measured only when its plausible saving on the remaining declared work exceeds its measurement cost. This break-even rule prevents tuning from costing more than it can save.
- Full-run admission requires current source/host/build/backend/event identities, complete exact equality, eight-lane coverage, no serial fallback, complete resource/counter evidence, selection of the faster measured exact W=4/W=8 point, and a safety-adjusted projection below the configured hard wall ceiling (never above 8 hours). The 8-hour value is a safety ceiling, not a performance target.

#### Reachability and decision arithmetic

The current code asks for 3,364 logical live-teacher forwards including the probe; S4 contributes about 97% and makes the old gate arithmetically impossible. The old plan represents 452,390,289,408 exact scalar terms.

The replacement upper bound is:

- tuner: 16 logical forwards (two S8 points);
- shared transcript: 36;
- S4 continuation: initially 8 and at most 64 logical forwards;
- no duplicate teacher prefill or S4 teacher warmup.

That is 60 initial or 116 maximum logical forwards, before any further allocation/workspace efficiency gain: a 98.2% initial and 96.6% maximum reduction from the 3,364-forward plan.

If the smallest continuation establishes both compiled/runtime ratios above the pinned 1.0 floor, stop. If not, extend only to the next predeclared cumulative bound. If the eight-step ceiling remains inconclusive, record NOT ESTABLISHED and stop; do not turn uncertainty into another multi-day run.

#### Outcome branches

- **Positive:** select the faster exact W=4/W=8 point, publish content-bound tuner evidence, execute the bounded instrumented BDD work, and complete #932/#931 through the protected workflow.
- **Negative exactness/resource/projection:** refuse the full run and retain the precise blocker evidence.
- **Inconclusive S4 at eight steps:** record NOT ESTABLISHED and request the smallest decision that can alter the next action; do not silently increase the budget.

This amendment replaces the fixed 4x, 0.75 utilization, W=S=8, mandatory 1/2/4/8 live sweep, 128-token, and three-wave requirements. It does not weaken exactness, fixture availability, parity floors, evidence durability, or the hard wall-clock abort.


### comment 5400234027 — 2026-08-24T19:27:29Z — Casey-allard

### Teacher-free admission outcome — 2026-08-24

The optimized #932 implementation reached its binding cheap gate without opening the teacher.

#### Explicit preflight

```text
R4_PARITY_SOURCE=/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct
R4_PARITY_BUNDLE=/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct
R4_PARITY_PREFLIGHT_ONLY=1
R4_PARITY_PREFLIGHT_REPORT=target/teacher-parity/teacher-free-preflight.json
cargo test --test bdd --offline
```

- wall time: 4.05 s
- process exit: 101, predeclared refusal
- schema/status: `uor-r4.teacher-parity-preflight/1` / `FAILED`
- `teacher_source_opened=false`
- `teacher_forwards=0`
- graph: present, 30,486,100 bytes, `blake3:10715ebc2df0f885163d80c8afd1ef22406caa7add1571179b5082ba652ddee0`
- graph report: present, 24,534 bytes, `blake3:29c64e28e89d2ba21d766abd57c2c9511f9dcc6072de72c19f2a3c27dae7b14f`
- exact refusal: graph runtime top-1 25.65% is below its required 30.12% TLA baseline

The refusal artifact also binds the tokenizer, TLA, and TLS CIDs. Teacher model/config were inspected by metadata only and not opened.

#### Ordinary BDD path

The same canonical inputs were then exercised through the ordinary BDD entry point. It completed with 120/120 scenarios and 402/402 steps structurally green. The finalized machine-readable parity verdict is truthfully `FAIL`, with:

- `uor-r4.teacher-parity-report/2`
- `uor-r4.teacher-parity-evidence/2`
- five flushed `uor-r4.teacher-parity-progress/2` events
- preflight report CID `blake3:3e9caca6c4cb4605e60d83bd3c2237cbec14dde7048faae5efff2b1314ff3933`
- zero logical forwards, physical batches, matrix calls, output cells, scalar terms, and teacher tokens
- graph `FAILED` with CID/reason; parsed graph report `AVAILABLE` with CID
- tokenizer/TLA/TLS `AVAILABLE` with CIDs; teacher source rows metadata-only `NOT_RUN`

A zero-work final-report bug found by this exercise was fixed: zero forwards/sec now yields seconds/forward `UNAVAILABLE`; explicit non-finite telemetry remains rejected by strict atomic readback.

#### Binding decision

The live exact tuner and full fixture-present parity suite are **NOT_RUN / REFUSED**. No different-CID historical bundle was substituted, no quality threshold was relaxed, and `load_accepting_quality` was not used. The current outcome is an evidence-backed failed prerequisite, not parity PASS and not a speed measurement.

This is also the intended time-saving result for #931 verification: the ordinary workspace path now refuses the inadmissible graph before opening the live teacher instead of entering the former multi-day black box. A future canonical graph must legitimately clear its own TLA-relative gate before live multicore throughput can be established.


### comment 5401406811 — 2026-08-24T21:08:52Z — Casey-allard

PARKED — foundational prerequisite #933 now natively blocks this issue. The completed #932 implementation remains preserved, unstaged and uncommitted, on local branch issue-932-exact-parity-multicore in /Users/casey.allard/uor-r4-issue-932; no PR exists and no threshold was weakened. Its teacher-free refusal reports remain retained under target/teacher-parity with verified unchanged hashes. Resume only after #933 restores one normative selector, binds production admission to that exact path, and produces a truthful canonical verdict. I am releasing the active-work signal now; #933 is the only assigned implementation issue.

### comment 5414274924 — 2026-08-25T17:35:53Z — Casey-allard

Definition of Done evidence:

- Merged through the protected queue in PR #937.
- Merge commit: 7c37e2d6642d1975038253109b9d56b7907db23e.
- Binding merge-group run: 32876360527, SUCCESS.
- Required queue jobs passed: full workspace nextest/BDD/doc/no_std/deterministic rebuild/kappa/Gate C, all-features, WASM, fuzz, register conformance R1-R6, and cargo audit.
- Focused local evidence: model-source 155 passed and 4 fixture-gated ignored; parity determinism 4 passed; parity observability 49 passed.
- Claim boundary remains truthful: the current canonical local fixture is UNAVAILABLE, so the live tuner/full parity run is NOT_RUN; no threshold was weakened and no performance PASS is claimed.
- Exact negative evidence and CIDs are recorded in docs/teacher_parity_parallelism_932.md.

Issue #932 is complete.


---
_Comment/review ids in this file: 5397311918, 5397456711, 5398098348, 5398824740, 5400234027, 5401406811, 5414274924_

