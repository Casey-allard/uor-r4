# Exact-parallel live teacher parity — issue #932

## Status

This is the append-only design, run contract, and evidence record for issue
#932, the verification blocker nested under #931. It concerns host-side
SmolLM2-135M parity only. It does not change deployed serving, artifact formats,
the multiplication-free runtime kernel, parity thresholds, or teacher
arithmetic.

Current status (2026-08-25, after #933 merged):

- exact scheduler, deterministic trace, observability, and planted-negative
  implementation: **PASS** on focused structural gates;
- canonical 135M teacher-free production preflight: **UNAVAILABLE** because the
  bundle has no schema-2 `release-bundle.json` production envelope required by
  the sole normative selector after #933;
- exact live tuner: **NOT_RUN / REFUSED by preflight** with zero teacher work;
- fixture-present parity measurement: **NOT_RUN / REFUSED by preflight**;
- throughput, speedup, resource, and projected-runtime claims: **NOT
  ESTABLISHED**;
- ordinary BDD runner: 124/124 scenarios and 414/414 steps structurally green,
  with its separate machine parity verdict correctly finalized as
  **UNAVAILABLE** and every live-work counter zero.

Historical status (2026-08-24):

- prior fixture-present workspace run: **ABORTED**;
- focused exactness, owner-plan, observability, and planted-negative gates:
  **PASS**;
- bounded A-panel cache instrument: **NEGATIVE / no material improvement**;
- canonical teacher-free deployed-path preflight: **FAILED** because the
  present graph scores 25.65% top-1 against its required 30.12% TLA baseline;
- exact live tuner: **NOT_RUN / REFUSED by preflight**;
- throughput, resource, and projected-runtime evidence: **NOT ESTABLISHED**;
- new fixture-present full parity suite: **NOT_RUN / REFUSED by preflight**;
- speedup claim: **NOT ESTABLISHED**.

No placeholder below is a PASS. Measurements are appended after they execute;
historical entries are not rewritten.

The fixed-shape design and launch gate below are historical. They were
superseded before any #932 live measurement by the dated optimization amendment
at the end of this record. Current code and future evidence use that amendment;
the earlier text remains only to preserve the decision history.

## Why this exists

On 2026-08-23/24, `cargo test --workspace --no-fail-fast --offline` entered the
fixture-present exact teacher path and ran for more than 34 hours without a
forward counter, current stream, rolling rate, defensible ETA, worker count,
CPU/RSS reading, or durable partial report. A read-only one-second sample
located it in S4 `timed_teacher_generate`, still executing the pinned exact
`uor-matmul` path rather than hung. It was then stopped deliberately; the cargo
process exited 130 and no complete BDD verdict exists.

The evidence available after the abort is limited. S2's actual tokenized work
required 36 exact teacher forwards and took about 4 h 29 min, approximately
448 s/forward. S4's old three-run sequential shape required hundreds more
forwards. Those observations explain why the earlier estimate was invalid;
they do not measure the new scheduler and do not establish its speedup.

## Exact multi-stream design

Autoregressive dependence is local to one trajectory: token `t + 1` in that
trajectory depends on token `t`. It does not impose a single stream on the
suite. The bounded scheduler exposes two independently enforced dimensions:

- `S = R4_PARITY_STREAMS` is the number of independent private-state
  trajectories advanced in one physical exact batch;
- `W = R4_PARITY_WORKERS` is the size of the one persistent exact output-row
  worker pool.

Model weights must be loaded once and shared immutably. Every stream must own
isolated mutable KV/model state sized to its actual prompt and generation horizon. One
physical `forward_batch` must advance all `S` states; reporting `states.len()`
without observed full-width matrix work is not multistream evidence. The `W`
pool schedules disjoint output rows, and no nested stream/row pool may exceed
that physical CPU-worker budget. Completed work is placed in fixed
prompt/position/run slots and reduced in canonical order.

The only configuration that may authorize a full run is
`W = S = available_parallelism()`. The binding M1 host therefore requires eight
independent states and eight exact row workers. The diagnostic probe holds
`S = 8` while sweeping `W = 1/2/4/8`; smaller/larger overrides remain useful
diagnostics but can never authorize a different full-run shape.

S2 must construct one content-bound teacher transcript for the pinned prompt
set. S3 and S7 must consume that transcript instead of repeating equivalent
live-teacher forwards. S4 must tokenize `S` distinct pinned prompts and truncate
them to their common nonzero length so every teacher batch stays full-width.
Teacher, legacy runtime, and graph runtime must receive the same resulting seed
in each lane. Every timing sample must retain per-lane
seed CID, output CID, private-state identity, and completion status; identical
fan-out cannot satisfy the contract. Repetitions remain separate temporal
samples so the median can measure run-to-run variation. Teacher exact-row
occupancy and each compiled engine's stream-worker occupancy must be separate
observed fields, and planned occupancy may never be reported as observed
occupancy.

### Arithmetic and accuracy boundary

**Guarantee.** Parallel work may partition **output rows only**. Each output row
must retain the complete pinned exact dot-product reduction. The reduction
dimension may not be split, partially accumulated, reassociated, or combined
across workers. Batch or completion order may vary, but it must not alter
raw-logit bits. Status: **Unproven** until the focused #932 gates pass.

The equality gate compares, byte for byte where applicable:

- every configured position/stream/raw-logit word, including signed-zero/NaN
  handling defined by the existing exact owner;
- top-k rows and tie order;
- every greedy choice and generated token sequence;
- persistent-state records and distinct per-lane completion/output CIDs;
- content-bound transcript bytes and CID;
- per-prompt/per-run metrics;
- canonical ordered reductions and final verdict inputs.

The trace shape must contain every expected item; comparing a partial prefix is
`FAIL`. Observed physical batches, logical forwards, matrix calls and widths,
row tiles, output cells, and scalar terms must reconcile exactly with the owner
`exact_forward_plan(S)`. Any mismatch is `FAIL`. The harness must not fall back
to Accelerate, approximate arithmetic, relaxed thresholds, or a different
teacher era.

## Configuration

| Variable | Default | Contract |
|---|---:|---|
| `R4_PARITY_WORKERS` | all available logical CPUs | Persistent exact row-worker pool `W`; diagnostic range `1..=available_parallelism()`; only `W = S = available_parallelism()` may authorize a full run |
| `R4_PARITY_STREAMS` | all available logical CPUs | Independent private-state/batch width `S`; diagnostic configurations require `S >= W`, and the binding probe holds `S` at host width for every worker point |
| `R4_PARITY_BATCH_PER_WORKER` | 4 | Bounded exact output-row task fan-out per scheduled worker; requested and effective fan-out are reported |
| `R4_PARITY_PROGRESS_EVERY_SECS` | 10 | Human status and flushed JSONL heartbeat cadence, even during a long forward |
| `R4_PARITY_MAX_WALL_SECS` | 28,800 | Eight-hour ceiling; crossing it records `ABORTED`, never partial PASS |
| `R4_PARITY_REPORT` | `target/teacher-parity/parity-report.json` | Final JSON; events are the sibling `parity-report.events.jsonl` |
| `R4_PARITY_TELEMETRY` | enabled | Disabling is available only to focused planted/unit controls; fixture-present parity refuses it |
| `R4_PARITY_POSITIONS` | 256 | Teacher-forced position cap |
| `R4_PARITY_GEN_TOKENS` | 128 | Per-run generation-token cap |
| `R4_PARITY_RUNS` | 3 | Independent S4 repetitions |
| `R4_PARITY_CORPUS_POSITIONS` | 1000 | Teacher-free S6 replay cap |
| `R4_EXACT_PROBE_REPORT` | `target/teacher-parity/exact-multicore-probe.json` | Source/host/executor-bound admission record; events are the sibling `.events.jsonl` |
| `R4_EXACT_PROBE_POSITIONS` | 1 | Valid range `1..=8`; every configured position is exercised at every worker point, including the configured maximum-prefix shape |
| `R4_EXACT_PROBE_SEED_TOKENS` | 64 | Conservative projection/maximum-context budget for registered positions and suite-work arithmetic; not the actual live-probe or S4 lane-seed length |
| `R4_PARITY_SOURCE` | `.uor-models/sources/smollm2-135m-instruct` | Probe-only source override; it does not retarget BDD bundle/source identities |

Zero, malformed, impossible, or host-out-of-range values fail explicitly. The
final report records both requested and effective values plus the actual
tokenized work; configured caps are never presented as completed-forward
counts. The probe has a 3,600-second admission deadline including fixture load;
it is not widened by the full-suite wall control. No new exact forward begins
at or after that deadline. A non-cancellable fixture load or exact forward that
is already active may finish, after which the probe records `ABORTED`; a probe
whose total elapsed time reaches the deadline cannot qualify.

The live probe uses deterministic distinct token inputs at its registered
positions. S4's actual common seed length is derived separately by truncating
the distinct tokenized pinned prompts; `R4_EXACT_PROBE_SEED_TOKENS` controls
neither input construction.

## Durable observability contract

The durable artifacts are:

| Artifact | Schema id | Default path | Determinism boundary |
|---|---|---|---|
| Progress events | `uor-r4.teacher-parity-progress/1` | `target/teacher-parity/parity-report.events.jsonl` | Empirical/run-variant |
| Final run report | `uor-r4.teacher-parity-report/1` | `target/teacher-parity/parity-report.json` | Empirical/run-variant |
| Deterministic evidence | `uor-r4.teacher-parity-evidence/1` | `target/teacher-parity/parity-report.evidence.json` | Timing-free exact identities and outputs |
| Exact admission probe | `uor-r4.exact-multicore-probe/1` | `target/teacher-parity/exact-multicore-probe.json` plus `.events.jsonl` | Source/host/executor-bound admission evidence |

The exact admission report binds `probe_deadline_policy` to the registered
finish-current-operation-then-abort semantics above. Its required `events`
object binds the sibling JSONL `file_name`, full-byte `content_cid`, `byte_len`,
`record_count`, and `final_record_number`; the final record must be `FINAL`, its
`final_status` and `final_qualifies_full_run` must match the report verdict, and
its `report_body_cid` must match the canonical report fields with the cyclic
events binding replaced by the registered pending placeholder. The FINAL record
also carries `sequence == record_count`. The producer synchronizes FINAL before
atomically publishing the report. Admission re-reads and validates the current
sibling bytes before loading teacher weights; a missing, truncated, appended,
or tampered event stream is a typed refusal even when the report file remains.

Schema `/1` bytes remain historical evidence. A change to field meaning, type,
requiredness, units, or artifact partition requires a new schema id instead of
reinterpreting an existing record. Human-readable status and JSONL events carry
the same counter snapshot. JSONL is flushed at every event so an interrupted
run retains evidence. The heartbeat runs independently of forward completion.

Event kinds on the wire are:

- `SUITE_STARTED`;
- `FIXTURE_STATUS`;
- `PHASE_STARTED` and `PHASE_COMPLETED`;
- `WORK_STARTED` and `WORK_COMPLETED`;
- `HEARTBEAT`;
- `WORK_FAILED`;
- `SUITE_COMPLETED` or `SUITE_ABORTED`.

Every applicable event/report contains:

- schema id, suite/run id, event sequence, event kind, and timestamp;
- source, artifact, store, graph, tokenizer, and corpus presence plus CID where
  available;
- per-fixture `AVAILABLE`, `UNAVAILABLE`, `FAILED`, or `NOT_RUN` status and
  exact reason;
- requested budgets and actual tokenized prompts, positions, warm-ups,
  replays, generation tokens, and exact teacher forwards;
- phase/scenario, lane seed/output CID, private-state/completion identity,
  queue depth, active/completed/failed work, forwards completed/total, tokens
  completed/total, and per-stream progress;
- configured/effective/current/peak streams and teacher exact-row workers, plus
  separately configured/effective/current/peak compiled-engine stream workers;
- requested/effective batch width, logical/physical cores, architecture, OS,
  model geometry, and exact backend/kernel/ISA identity where observable;
- planned and observed physical batches, logical forwards, matrix calls and
  maximum widths, row tiles, output cells, scalar terms, and their
  `exact_forward_plan(S)` reconciliation verdict;
- expected and observed counts for every position, stream, raw-logit word,
  top-k row/tie, greedy choice, persistent-state record, transcript byte/CID,
  metric, and ordered reduction;
- phase and total elapsed time, rolling and cumulative forwards/s,
  seconds/forward, completed-token throughput, longest active-forward age, and
  ETA value plus status (`WARMING_UP`, `ESTIMATED`, `UNAVAILABLE`, or `STALL`);
- process CPU percentage/time, mean CPU core-equivalents, mean
  core-equivalents divided by `W`, current/peak RSS, and virtual memory when the
  host exposes them; unsupported readings are `UNAVAILABLE`, not zero and
  cannot authorize the binding macOS run;
- warm-up versus measured work, per-run tokens/duration/throughput, canonical
  reduction status, selected configuration, measured speedup, and projection;
- final `PASS`, `FAIL`, `UNAVAILABLE`, `ABORTED`, or `NOT_RUN` state with the
  exact reason and durable report/event paths.

A report create, write, or flush failure is `FAIL`. A worker failure is
`FAIL`, cancels admission of new work, and cannot contribute partial metrics.
A missing conditional fixture is `UNAVAILABLE`; an enclosing test-process exit
code of zero does not promote it to PASS.

Telemetry is empirical evidence. Timing and resource fields do not enter
deterministic transcript/artifact bytes and do not add teacher evaluations.

## Pre-registered launch gate

**Empirical Criterion.** The cheap live probe loads the pinned model once and
runs the same real-model workload at `W = 1/2/4/8` plus supported bounded batch
widths while holding `S = available_parallelism()` (eight on the binding M1) at
every point. It records exact trace/accounting equality, wall time, sustained
CPU, and RSS. Its cost ceiling is 60 minutes. Claim status: **Unproven**;
execution verdict: **NOT_RUN**.

Required verdict before a full live suite:

1. every worker/batch configuration contains the complete expected trace shape
   and matches the one-worker logits, top-k/ties, greedy choices, persistent
   states, output/transcript CIDs, metrics, and ordered reductions exactly;
2. observed physical-batch/logical-forward/matrix/tile/cell/scalar-term totals
   equal the owner `exact_forward_plan(S)` at every point;
3. configured/effective/current/peak evidence proves all `S = 8` independent
   states in flight at every M1 worker point, with no lost/failed lane;
4. observed teacher exact-row occupancy and compiled-engine stream-worker
   occupancy are reported separately and never exceed `W`; every configured
   worker is observed doing useful work;
5. the configured all-core `W = S = available_parallelism()` point—not merely
   the fastest diagnostic point—measures at least 4.0x the `W = 1` aggregate
   forward rate (target at least 6.0x);
6. for `W > 1`, mean process CPU core-equivalents divided by `W` is at least
   0.75 (target 0.90), and measured CPU-time evidence is available;
7. safety-adjusted reachability arithmetic projects the fixture-present default
   suite below 28,800 seconds;
8. every required fixture, heartbeat, report/evidence write, source/host/executor
   identity, and counter reconciliation is available and non-failing.

If any condition fails, the full suite is refused. The measurements remain as
a truthful negative result and scheduler/batch/row-partition work continues
without weakening exactness or a threshold. If all conditions pass, the full
run uses exactly `W = S = available_parallelism()` under the same eight-hour
ceiling and continuous event stream. The fastest diagnostic point is retained
as evidence but cannot select a different full-run shape.

The wall ceiling is checked at work boundaries. Once reached, no new forward is
admitted; finalization records `ABORTED` even if an already-running exact
forward returns after the boundary.

## Evidence ledger

### #932 focused structural and negative-control gates

| Evidence | Required content | Verdict |
|---|---|---|
| `tests/parity_determinism_932.rs` | 1/2/4/8 and supported-batch raw bits, top-k/ties, greedy tokens, persistent state, transcript/CID, metrics, canonical reductions, shuffled completion, workers>work, batch remainder, shared weights/private state | NOT_RUN |
| `tests/parity_observability_932.rs` | invalid/zero/out-of-host budgets, planted slow/stall, worker failure, wall abort, missing fixture, write failure, counter mismatch, typed non-PASS verdicts, heartbeat persistence, schema/path checks | NOT_RUN |
| Model-source exact executor/probe tests | owner-plan reconciliation, full trace shape, stream/worker occupancy, CPU threshold, speed/projection refusal, source/host/executor binding | NOT_RUN |
| RF-29 BDD/conformance gates | tagged steps, explicit UNAVAILABLE, durable artifacts, generated conformance equality | NOT_RUN |

These rows become PASS only from exact command output. The live probe and full
suite remain independently `NOT_RUN`; a focused structural PASS cannot promote
either empirical verdict.

### 2026-08-24 — prior serial run

| Field | Verdict |
|---|---|
| Fixture presence | AVAILABLE during the run |
| Exact backend | pinned `uor-matmul` path |
| Elapsed before intervention | more than 34 h |
| Last located phase | S4 exact teacher generation |
| Progress/resource artifact | UNAVAILABLE — old harness emitted none |
| Cargo verdict | ABORTED, exit 130 |
| Complete BDD verdict | NOT_RUN/ABORTED; no PASS claimed |
| Decision | do not restart the old single-stream shape |

### #932 cheap exact-parallel probe

| W | Required S | Role | Batch schedule | Exact trace | Stream/row occupancy | Owner-plan counters | Aggregate forwards/s | Speedup | Mean CPU/W | Peak RSS | Verdict |
|---:|---:|---|---|---|---|---|---:|---:|---:|---:|---|
| 1 | 8 | diagnostic reference | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | 1.0x reference, NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| 2 | 8 | diagnostic | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| 4 | 8 | diagnostic | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |
| 8 | 8 | binding all-core point | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN | NOT_RUN |

Configured/effective/current/peak stream evidence: **NOT_RUN**.

Configured/effective/current/peak teacher row-worker evidence: **NOT_RUN**.

Separate compiled-engine stream-worker evidence: **NOT_RUN**.

Full trace-shape and per-lane seed/state/output-CID evidence: **NOT_RUN**.

CPU-time/core-equivalent evidence and 0.75 utilization criterion: **NOT_RUN**.

Projection inputs, safety factor, and arithmetic: **NOT_RUN**.

Binding probe verdict: **NOT_RUN**.

### #932 fixture-present full suite

Launch authorization: **REFUSED / NOT_RUN pending the cheap probe**.

Required launch shape: `W = S = available_parallelism()`; binding M1 `8/8`.

Matched teacher/legacy/graph lane-seed and per-lane output evidence: **NOT_RUN**.

Progress/report/deterministic-evidence artifact paths and CIDs: **NOT_RUN**.

Final report path/CID: **NOT_RUN**.

Final parity verdict: **NOT_RUN**.

## 2026-08-24 — superseding pre-measurement optimization amendment

No #932 live probe or fixture-present parity run had executed when this
amendment was adopted. The objective is the least exact end-to-end wall time to
a decision that can unblock #931. Worker count, stream count, utilization,
speedup, and sample count are instruments rather than goals.

### Why the fixed shape was retired

The eight pinned prompts tokenize to lengths `6, 7, 6, 5, 4, 5, 6, 5`. The
shared S2/S3/S7 transcript therefore needs 36 logical teacher forwards in six
physical `S = 8` batches. The superseded S4 shape added:

- 96 logical forwards for a separate seed-plus-eight-token warm-up; and
- 3,168 logical forwards for three seed-plus-128-token measured waves.

Including the old eight-forward W=1/2/4/8 probe, that plan requested 3,364
logical forwards, or 452,390,289,408 owner-counted exact scalar terms. S4 was
97% of the work. At the prior observed 448 seconds per serial logical forward,
even ideal eight-way scaling could not approach the eight-hour safety ceiling.
The old `>= 4x` gate could not make its own workload reachable, so launching it
would have had no decision value.

### Current bounded work

- Preserve eight canonical prompt/lane identities, shared immutable weights,
  private mutable state, complete raw-bit equality, canonical reductions, and
  exact owner-plan counters.
- Retain every lane's distinct final teacher-forced prompt-prefix state and
  next token while building the required 36-forward transcript. Prefix lengths
  may differ; the exact S=8 batch carries each lane's position explicitly. S4
  clones those content-bound states and performs no duplicate teacher prefill.
- The transcript already exercises the complete exact stack, so S4 performs no
  independent live-teacher warm-up.
- Run one causal S4 continuation. Start at one decode step per lane and extend
  cumulatively through 2, 4, then 8 only while more work can change the pinned
  compiled-versus-teacher verdict. Eight steps per lane is the hard ceiling.
  Exact repeatability is established by the focused structural suite, not by
  repeating an expensive live-model wave.
- Report transcript preparation, state-clone preparation, decode, and one-shot
  elapsed time separately. The empirical performance statement is limited to
  the executed interval; it is not a three-run median or a broader sustained
  performance claim.
- Retain the model-wide batch buffers, exact transpose/output buffers, and one
  exact scratch workspace per dedicated worker. Reserve their bounded capacity
  during excluded preparation, record actual capacity growth, reset counters
  without freeing capacity, and require zero growth during every measured
  forward.
- Overlay monotonic exact matrix, tile, output-cell, and scalar-term counters
  into every durable heartbeat while a physical forward is still active.
  Scalar terms are the ETA/liveness unit when planned, worker tasks are the
  fallback, and completed-forward throughput remains separately reported; an
  advancing long forward is never labeled stalled merely because it has not
  returned yet.
- Compile the exact teacher substrate and the measured legacy/graph engine path
  with narrow test-profile opt-level 3 package overrides. An opt-level-0 debug
  engine is neither an efficient harness nor a truthful proxy for shipped CPU
  throughput; unrelated workspace packages keep the normal test profile.

The BDD work is therefore 36 transcript forwards plus 8 initially and at most
64 S4 forwards: 44 initially and 100 at the ceiling. The tuner adds 16 logical
forwards. Combined live work is 60 initially and 116 at the ceiling, reductions
of 98.2% and 96.6% respectively from the superseded 3,364-forward design.

### Current adaptive tuner and admission

The tuner holds the scientific work fixed at eight canonical lanes in one
shared-weight `S = 8` batch. It measures the host's `W = available` and
`W = min(4, available)` candidates over identical work, without a full-model
warm-up per point, and selects the faster exact result with a deterministic
tie-break. These are `W = 8` and `W = 4` on the binding M1; they are a bounded
break-even search, not prescribed utilization targets. A cheap worker-pool/backend
prestart is recorded separately and excluded from forward timing. The
deterministic structural suite retains W=1/2/4/8 exactness and planted-negative
coverage; the live fixture is not spent re-proving every diagnostic width.

This contract is serialized as `uor-r4.exact-multicore-probe/2`; schema `/1`
remains the historical fixed-sweep shape. The `/2` work record requires the
complete registered binding shape: transcript batch widths `8, 8, 8, 7, 4, 1`
for 36 logical forwards in six physical batches; eight continuation tokens on
each of eight lanes for 64 logical forwards in eight physical batches; 100
logical forwards and 14 physical batches in total; zero-based maximum private-
state position 13; and state capacity 14. A smaller configured cap may emit a
diagnostic record but cannot authorize the full suite. Projection uses the
selected S8 elapsed time times exactly 14 physical batches before applying the
safety factor; the 100 logical-forward total is retained as accounting, not
misused as a same-width rate estimate. The report names its first measured
candidate through `reference_workers`, binds every candidate through
`equal_to_reference`, and excludes the recorded cheap worker-pool/backend
prestart from timed forward rates.

The current run artifacts are likewise versioned
`uor-r4.teacher-parity-progress/2`, `uor-r4.teacher-parity-report/2`, and
`uor-r4.teacher-parity-evidence/2` because adaptive cumulative checkpoints,
one-time compiled-cohort preparation, and the expanded work/resource ledger
change the meaning of the historical `/1` fields. The `/1` table above remains
an unmodified record of the superseded design.

Speedup versus W=1 and CPU utilization remain reported diagnostics. They are
not admission thresholds, and `S` need not equal `W`. Full-run admission now
requires:

1. current source, host, executor/build, backend, report, and finalized event
   identities;
2. complete raw-bit trace equality and all eight canonical lane identities;
3. no serial fallback, lost lane, counter mismatch, structurally invalid
   resource record, or incomplete candidate; a truthful platform-reported
   resource status of `UNAVAILABLE` remains diagnostic rather than a gate;
4. selection of the faster measured exact W=available/W=min(4, available)
   candidate; and
5. a safety-adjusted projection for the actual optimized work strictly below
   the configured hard wall ceiling, which is capped at 28,800 seconds.

A later candidate is worth measuring only when its plausible saving on the
remaining declared work exceeds the candidate's own measurement cost. This
break-even rule prevents tuning from costing more time than it can save.

### Cheap exact-kernel optimization decision

Before opening the live fixture, the bounded ignored instrument
`tests::bench_exact_a_panel_cache_rows_on_smollm2_tiles` compared the existing
one-row Atlas A-panel offer with an eight-row offer on the five W=8/S=8 tile
shapes used by SmolLM2 (Q/O, K/V, W1/W3, W2, and vocabulary). It interleaved
samples, took per-shape medians, weighted them by calls per forward, and
required raw-bit equality for every sample. Run command:

```bash
cargo test -p uor-r4-model-source --lib \
  bench_exact_a_panel_cache_rows_on_smollm2_tiles --offline -- \
  --ignored --nocapture --test-threads=1
```

The run completed in 2.96 seconds. Weighted time was 1,601,233,947 ns for the
one-row offer and 1,596,652,561 ns for eight rows, or 1.0029x. Per-shape ratios
were mixed: 0.9497x, 0.9382x, 1.0289x, 0.9687x, and 1.0432x. This is a
non-material/noisy result, so production retains the simpler `pa = k` offer;
no claimed speedup or live-model work rests on it. The benchmark remains as a
reproducible negative decision instrument.

### Predeclared outcomes

- If the smallest causal continuation puts both compiled/runtime ratios above
  the pinned 1.0 floor with the registered early-stop margin, stop and record
  the positive measurement.
- If it is inconclusive, extend only to the next cumulative bound. At eight
  steps, record the exact positive, negative, or `NOT ESTABLISHED` result; do
  not silently increase the budget.
- Any equality, fixture, counter, structurally invalid resource record,
  durability, or projection failure refuses the full run and preserves its
  typed evidence. Truthful CPU/RSS `UNAVAILABLE` evidence is retained without
  being reclassified as a scientific failure.

Before either live-teacher phase, run the teacher-free deployed-path preflight:

```bash
R4_PARITY_PREFLIGHT_ONLY=1 cargo test --test bdd --offline
```

It parses the tokenizer, legacy artifact/store, graph and graph report, then
exercises one typed deployed decision for every canonical legacy and graph lane.
Its `uor-r4.teacher-parity-preflight/1` artifact binds fixture and output
identities plus the current authorizing code contract while proving
`teacher_source_opened=false` and `teacher_forwards=0`.
It is atomically written to
`target/teacher-parity/teacher-free-preflight.json`, or the explicit
`R4_PARITY_PREFLIGHT_REPORT` path. A missing or malformed prerequisite—or an
unwritable evidence path—therefore blocks the expensive tuner or BDD run before
any teacher forward is spent.

Amendment implementation status: **COMPLETE**. Adaptive tuner:
**NOT_RUN / REFUSED**. Fixture-present bounded parity:
**NOT_RUN / REFUSED**. The exact reason and zero-teacher counters are recorded
below; no performance conclusion is inferred from the prerequisite failure.

## 2026-08-24 — canonical teacher-free outcome

The explicit teacher-free preflight used the configured canonical 135M source
and compiled bundle:

```bash
R4_PARITY_SOURCE=/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct \
R4_PARITY_BUNDLE=/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct \
R4_PARITY_PREFLIGHT_ONLY=1 \
R4_PARITY_PREFLIGHT_REPORT=target/teacher-parity/teacher-free-preflight.json \
cargo test --test bdd --offline
```

It completed in 4.05 seconds and exited 101 with the predeclared refusal. The
atomically published `uor-r4.teacher-parity-preflight/1` artifact was 2,745
bytes and recorded:

| Field | Evidence |
|---|---|
| Preflight status | `FAILED` |
| Teacher source opened | `false` |
| Exact teacher forwards | `0` |
| Graph | present, 30,486,100 bytes, `blake3:10715ebc2df0f885163d80c8afd1ef22406caa7add1571179b5082ba652ddee0` |
| Graph report | present, 24,534 bytes, `blake3:29c64e28e89d2ba21d766abd57c2c9511f9dcc6072de72c19f2a3c27dae7b14f` |
| Legacy artifact | present, `blake3:487532dda0b33ac0306d939126b0da54ac802036446dd8514905531cfc8c23df` |
| Legacy store | present, `blake3:a65671b7aa0c7706a632c2f3922928353208b8e8ae4dd0e49a00f1a17e3947fc` |
| Tokenizer | present, `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1` |
| Exact refusal | graph runtime top-1 25.65% is below its 30.12% TLA baseline |

The same inputs were then exercised through the ordinary BDD entry point, not
the special preflight-only path. The command completed in 4.85 seconds with
120/120 scenarios and 402/402 steps structurally green. Its finalized
`uor-r4.teacher-parity-report/2` verdict was nevertheless `FAIL`, accompanied
by `uor-r4.teacher-parity-evidence/2`, five flushed progress events, the same
quality-gate reason, and zero logical forwards, physical batches, matrix calls,
output cells, scalar terms, or teacher tokens. A zero-work telemetry regression
found during this run was fixed: seconds-per-forward is now unavailable when
the measured rate is zero, while explicit non-finite telemetry remains rejected
by strict serialization readback.

The live tuner and full parity suite were therefore not launched. A historical
different-CID bundle was not substituted, the quality floor was not changed,
and `load_accepting_quality` was not used. This is an evidence-backed negative
prerequisite outcome, not a parity PASS and not a speed measurement. The
implemented automatic gate is still the time-saving result needed by #931:
ordinary workspace verification now refuses this invalid graph before opening
the teacher instead of entering the former multi-day path. A future canonical
graph must legitimately clear its own TLA-relative quality gate before the
live tuner can establish multicore throughput or authorize full parity.

### Post-audit hardened rerun

The historical run above exposed the intended blocker, then final review added
a current-code admission binding, exact published-byte CID semantics, and
distinct `FAILED`/`UNAVAILABLE`/`NOT_RUN` projection. The canonical inputs were
rerun after those changes. The explicit preflight completed in 2.91 seconds,
exited 101 with the same predeclared graph-quality refusal, and wrote 2,850
bytes with:

- authorizing contract CID
  `blake3:12eaeeb0bb52f34e9f4ee276a4b4dc7258a175c7e2cbb93396f6f82d20a95160`;
- exact published preflight CID
  `blake3:8126381f4ec713cc0310d3e1e223ee150761b50d8f3086264cd68b1a3eee9494`;
- `teacher_source_opened=false`, `teacher_forwards=0`, and unchanged compiled
  input CIDs;
- the unchanged 25.65% graph versus 30.12% TLA refusal.

The ordinary BDD entry point then completed in 2.58 seconds with 120/120
scenarios and 402/402 steps structurally green. Its machine verdict remained
`FAIL`, with the graph `FAILED`, its parsed report `AVAILABLE`, teacher source
rows `NOT_RUN`, and every live-work counter zero. The direct tuner now validates
the current authorizing contract, report/source/bundle paths, and recomputed
compiled-input CIDs before its teacher loader is reachable; planted stale-code,
stale-input, and non-available artifacts all refuse.

### Final path-normalized refusal and readback rerun

The final code was rerun after relative input/report paths were normalized to
the workspace root and the BDD consumer learned to distinguish a full qualified
probe report from a truthful `EXACT_MULTICORE_PROBE_STATE` refusal. The explicit
preflight completed in 2.52 seconds, exited 101 as predeclared, and atomically
wrote 2,850 bytes. Its current bindings are:

- authorizing contract CID
  `blake3:340299452fb9574a0f76f72850472e2bd495053eaa6c55f0ff7eb1ff702c24a4`;
- exact published preflight CID
  `blake3:8b300e87f5feb62565197867cbb5f2e009a0b92ad3350729b8a536b9efc309f2`;
- `teacher_source_opened=false`, `teacher_forwards=0`, and the unchanged
  tokenizer/TLA/TLS/graph/report CIDs recorded above;
- the unchanged graph-quality refusal: 25.65% graph top-1 versus the required
  30.12% same-corpus TLA baseline.

The direct ignored tuner then revalidated that current artifact and refused in
0.057 seconds of probe time, before opening the teacher. Its
`uor-r4.exact-multicore-probe/2` state is `NOT_RUN / REFUSE_FULL_RUN`, explicitly
sets `qualifies_full_run=false`, and has published-byte CID
`blake3:2a4347c7f25686ace455b8824992c06039ff08068f6aab02779d4f4d3cc9f18a`.
The surrounding one-time test command spent 6.93 seconds compiling the changed
model-source crate; that build time is not probe work.

Finally, the ordinary BDD entry point consumed that present refusal state
without promoting it to qualification. It completed in 2.80 seconds with
120/120 scenarios and 402/402 steps structurally green. The final machine
verdict remained `FAIL`; the preflight is `FAILED`, the exact probe is
`NOT_RUN` with its content CID, teacher model/config remain metadata-only
`NOT_RUN`, and every logical-forward, physical-batch, matrix, tile, cell,
scalar-term, stream, worker-task, and teacher-token counter is zero. Malformed,
unknown, contradictory, or incomplete probe artifacts remain fail-closed.

### Post-gate current-code binding

The full model-source suite subsequently exposed a scheduler-sensitive planted
overlap test. Its test-only observer was hardened to synchronize two genuinely
entered worker tasks; production execution was unchanged. Because the
authorizing contract binds that source file, the complete evidence trio was
rerun once more after 21/21 overlap stress repetitions, the full workspace
suite, strict Clippy, no-std/wasm checks, and non-vacuous κ reproduction passed.

This final-code rerun recorded:

- authorizing contract CID
  `blake3:22e9f35d6333f3c25f526ce0ec2315a2461504cc77c75f26d19ab26251147b94`;
- exact published preflight CID
  `blake3:f09672bce4019e779e93151c15dcc1d6b3094bf9bff49335bc67a6b7e5cd0f9b`;
- exact direct-tuner refusal CID
  `blake3:423ae9b88a343eb5a3f61fe5c1075269c58529c2e8affc56edfb684279ab743a`;
- explicit preflight process time 2.63 seconds;
- direct tuner probe time 0.037 seconds and warm command time 0.16 seconds;
- ordinary BDD process time 2.61 seconds, with 120/120 scenarios and 402/402
  steps structurally green and 2.149 seconds recorded inside the final report.

The machine verdict and decision boundary are unchanged: graph prerequisite
`FAILED`, exact tuner `NOT_RUN / REFUSED`, teacher unopened, and every expensive
work counter zero.

## 2026-08-25 — post-#933 normative-production preflight

The parked branch was fast-forwarded to `origin/main` at
`e346816cb40b089583cbd3fff2cdd84924362c65` after #933 restored
`R4G1Runtime` as the sole production selector and required a schema-2,
CID-bound production envelope. The final #932 code additionally binds the full
local source closure, invokes production envelope semantics before teacher
access, reparses only content-bound `tokenizer.bin` bytes, and excludes
worker/tile/workspace observations from deterministic evidence. Its focused
gates are green:

- `uor-r4-model-source` library: 155 passed, 4 fixture/benchmark tests ignored;
- deterministic schedule integration: 4 passed;
- fail-closed observability integration: 49 passed;
- strict workspace Clippy, format, and diff checks: PASS.

The explicit teacher-free command spent 25.22 seconds compiling the frozen
BDD binary, then exited 101 with the predeclared refusal:

```bash
R4_PARITY_SOURCE=/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct \
R4_PARITY_BUNDLE=/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct \
R4_PARITY_PREFLIGHT_ONLY=1 \
R4_PARITY_PREFLIGHT_REPORT=target/teacher-parity/teacher-free-preflight.json \
cargo test --test bdd --offline
```

The `uor-r4.teacher-parity-preflight/1` artifact records:

| Field | Evidence |
|---|---|
| Status | `UNAVAILABLE` |
| Exact reason | required production component `release-bundle.json` is absent from the selected 135M bundle |
| Authorizing contract | `blake3:c16bdf54615c45db06b1ed2f3655128009b4890d49d03adfed66cf0556e893ab` |
| Preflight artifact CID | `blake3:6978404f3f627abe82c46cfd0bd0434f1a68eef2c2d251c584c8eb0d7d8600bf` |
| Teacher source opened / forwards | `false` / `0` |
| Graph | `blake3:10715ebc2df0f885163d80c8afd1ef22406caa7add1571179b5082ba652ddee0` |
| Graph report | `blake3:29c64e28e89d2ba21d766abd57c2c9511f9dcc6072de72c19f2a3c27dae7b14f` |
| TLA artifact | `blake3:487532dda0b33ac0306d939126b0da54ac802036446dd8514905531cfc8c23df` |
| TLS store | `blake3:a65671b7aa0c7706a632c2f3922928353208b8e8ae4dd0e49a00f1a17e3947fc` |
| Tokenizer | `blake3:70af0cb08bbcd3b323d3387ca1d7d33da39873820604d183711e8e99f9903fc1` |

The complete production inventory also records the absent sections-absent and
label-shuffled graphs, deployed-quality report, cross-surface parity, witness
replay, and tokenizer adapter; none is silently treated as present. The direct
ignored tuner then consumed that current refusal artifact and terminated after
0.14 test seconds before loading teacher weights. Its
`uor-r4.exact-multicore-probe/2` state is `NOT_RUN / REFUSE_FULL_RUN`, with
content CID
`blake3:706395f99a10ee2346e31026edeae51697817600f5d34dc6be49e97b909f38d4`.
The nonzero test-process exit is the required fail-closed signal, not a broken
measurement.

Finally, the ordinary BDD entry point re-ran against the same current inputs.
All 124 scenarios and 414 steps completed structurally green in 2.83 command
seconds, while `uor-r4.teacher-parity-report/2` finalized truthfully as
`UNAVAILABLE` after 2.207 recorded seconds. The preflight is `UNAVAILABLE`; the
exact probe is `NOT_RUN`; teacher model/config are
metadata-only `NOT_RUN`; and logical forwards, physical batches, matrix calls,
row tiles, output cells, scalar terms, streams, worker tasks, and teacher tokens
are all zero. The final artifact CIDs are:

- run report:
  `blake3:065b79a5f427db3f7fefeee165bff75d841d30f22aebecb18e0831ad89bbb123`;
- deterministic evidence:
  `blake3:b99930011ba7f0d1f5c0066a5adf3772ca6a9b4eb0317548cc508af23e7e0116`;
- terminal event log:
  `blake3:3ecd48208d712f32bddc8f3db7cee506168c240a9e9b0b5746e574d92dca206a`.

The deterministic evidence contains no absolute path, elapsed-time field,
selected-worker row, worker/tile/workspace counter, or empirical concurrency
peak. The event log's final record is `SUITE_COMPLETED / UNAVAILABLE`.

### Final issue verdict

The host-side capability is implemented and its structural/falsifier gates
pass, but live exact multicore throughput and fixture-present parity remain
**NOT ESTABLISHED** for the selected canonical 135M bundle. The binding
production prerequisite is unavailable before teacher access because that
bundle predates #933's required envelope and companion evidence. The frozen negative branch therefore
applies: do not launch or retune the live experiment, do not substitute the
different-CID #933 broad bundle, and retain the exact refusal evidence. A
future 135M bundle must be re-emitted, schema-2 quality-bound, and production
admitted before this harness can measure live scaling.
