# #1011 — A1Q-H1: compile and qualify R4SoftmaxTraceStateStudentV1 from frozen R4/Spin traces

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-30T11:22:36Z
- **Updated:** 2026-08-30T15:46:23Z
- **Closed:** 2026-08-30T15:46:23Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1011
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Build and qualify the smallest source-free recurrent R4/Spin state student that can improve on the bounded `R4SoftmaxTraceStudentV1` result without consulting source weights or held-out traces at runtime.

Parent: #973. Programme root: #820. #973 remains the native blocker of #954.

## Frozen predecessor

PR #1010 records the predecessor result:

- implementation revision `3cbeab64ba40f4445f878741b87ec6a487fcee00`;
- construction trace bundle CID `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`;
- suffix artifact CID `blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`;
- freeze CID `blake3:bb19fc6f6976aca6dfd8c67c470fd1fb70a1e1e74763800fdcb635f135325df7`;
- result CID `blake3:e48b4172e02fc84eef9e00024ac6602b790d8230026a979d2f71c552ddca0cd4`;
- bounded context result: distilled covered teacher CE `2.660721`, teacher top-1 `3/9`, actual-next top-1 `2/9`;
- negative decoded boundary: `He was born` enters a period-2 `, Scotland` loop.

This predecessor establishes bounded source-free trace distillation only. It does not establish geometric advantage, coherent generation, general-purpose inference, correctness, reasoning, WASM readiness, or release readiness.

## Mandatory prerequisite: fail-closed trace intake

The existing trace and bundle paths serialize but cannot yet reload the already-captured trace as a canonical typed input. Before fitting any state cell:

- implement canonical `from_bytes` readers for `R4SoftmaxTeacherTraceV1` documents and the construction bundle;
- validate magic/version, lengths, counts, dimensions, support caps, ordering, finite values, token ranges, nested CIDs, and exact end-of-input;
- reject truncation, trailing bytes, malformed counts, noncanonical order, shape mismatch, CID mismatch, and one-byte tampering;
- prove byte-identical roundtrip/reload and deterministic bundle CID;
- bind the loader schema and implementation revision into the next freeze.

No held-out run launches until this prerequisite is green.

## Three-phase leak boundary

1. **Compile and seal:** use construction documents only. Trace fields may be fitting targets/codebook-induction data.
2. **Freeze source-free held-out execution:** execute and seal predictions, state transitions, suffix depths, runtime-input provenance, and continuation before any held-out teacher trace is opened.
3. **Reveal and score:** only then execute the held-out teacher trace and compare the sealed predictions.

At held-out/runtime execution the student may read only:

- prior compiled recurrent state;
- observed causal token IDs;
- independently available canonical R4/Spin route addresses and frames; and
- the frozen compiled artifact.

It may not read held-out Q/K/V, attention weights, aggregates, decoded heads, logits, targets, source weights, source traces, future tokens, Ollama, Gemma, or a hosted provider.

## Equal-budget arms

- frozen 39,648-byte suffix student from PR #1010;
- equal-state/readout-budget non-geometric recurrent cell;
- full `R4SoftmaxTraceStateStudentV1`;
- deterministic transport/state-permuted geometric control.

The geometric arm may fit separate key, value, query/readout, retention, and delta-overwrite terms from construction traces. Exact R4/Spin transport must act on recurrent state between route frames. Candidate support and scoring work remain identical across matched arms.

## Promotion gate

On the frozen nine non-BOS context-bearing held-out positions, require all of:

- shared-support covered teacher CE strictly below `2.660721` and below both newly fit controls;
- teacher top-1 strictly greater than `3/9`;
- actual-next top-1 strictly greater than `2/9`;
- material loss under transport/state permutation;
- exact artifact reload and byte-identical causal replay;
- zero runtime source forwards and a field-by-field runtime-input provenance audit;
- no future reads;
- decoded continuation that avoids period-1 and period-2 cycles observed in the predecessor.

Full-vocabulary NLL remains unavailable unless support is expanded under a new leak-free freeze. A positive result remains bounded mechanism evidence, not correctness, reasoning, product, or scale evidence.

## Run discipline

Before any run longer than minutes, post:

- metric and current value;
- reachability ceiling from the frozen nine-position support;
- cheap loader/shape/runtime-input preflight and binding verdict;
- divergent positive and negative actions;
- wall-clock estimate and worker count.

Use the exact multiworker backend. Do not launch corpus scaling, intrinsic/readout alternatives, resonance replacement, E8 expansion, exact lowering, WASM promotion, or release work in this issue.

## Definition of done

- canonical fail-closed trace/bundle loader plus tamper/reload tests;
- frozen construction-only state compiler and artifact;
- all four equal-budget arms;
- sealed source-free held-out execution before teacher reveal;
- complete metrics, controls, runtime-input provenance, causal/work audit, replay, CIDs, and decoded output;
- append-only evidence record and structured raw result;
- explicit terminal that either advances the qualified state cell or repairs/retires this representation before scale.

References #973

## Comments (5)

### comment 5469109496 — 2026-08-30T13:57:22Z — Casey-allard

# R4 softmax trace state student (#1011)

- **Status:** `IN_PROGRESS`; intake and architecture freeze only. No state-cell
  result has been run.
- **Owner:** #1011 under attention issue #973 and programme root #820.
- **Decision:**
  [ADR-0005](adr/0005-predictive-geometric-connection-memory.md).
- **Base revision:** `02b6a08a3dfd8c485d5d22353e0f37ad0cfd9926`.
- **Frozen construction bundle:**
  `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`,
  45,205,493 bytes, four construction documents.
- **Predecessor:** `R4SoftmaxTraceStudentV1`, 39,648 bytes,
  `blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`.

## Context

PR #1010 established a bounded source-free suffix student compiled from the
ordinary R4/Spin Q/K/V plus stable-softmax teacher trace. On the nine frozen
non-BOS held-out positions, its covered teacher cross-entropy was `2.660721`,
teacher top-1 was `3/9`, and actual-next top-1 was `2/9`. Its autonomous
continuation from `He was born` entered the period-two loop `, Scotland`.

That is evidence that the construction trace contains learnable predictive
information. It is not evidence that recurrent geometric state helps, that
generation is coherent, or that the source-free student is a complete model.
The smallest next decision is therefore whether a causal state transition over
the same frozen trace beats the suffix representation and matched recurrent
controls.

## Accepted architecture

`R4SoftmaxTraceStateStudentV1` is a bounded compiler-produced causal state
machine. It keeps one fixed-size state budget for each matched recurrent arm
and a shared candidate/readout budget. At event `t`, runtime inputs are only
the prior state, the observed token ID, the independently reproducible
canonical route/frame identity, and the frozen artifact.

The construction compiler may use floating point and the captured teacher
fields to fit four roles:

1. an observed-token write/key representation;
2. a value/update representation;
3. a retained causal state transition; and
4. a candidate-relative readout over the shared frozen support.

The geometric arm transports prior state from the prior canonical R4/Spin
frame into the current frame before applying its retained update and readout.
The non-geometric arm receives the same state and readout budget but no frame
or transport. The geometric destructive control uses the same fitted artifact
and work budget with a deterministic state/transport permutation. The frozen
suffix artifact remains the fourth arm.

The recurrent state is four fixed `4 x 4` banks. For bank `b`, the geometric
transition is:

```text
Sbar[b] = P(previous -> current) S[b] P(previous -> current)^T
S'[b]   = rho[b] Sbar[b] + eta[b] (v - Sbar[b] k) k^T
r[b]    = S'[b] k_current
```

The four retention/write policies are fixed, not learned from only 38 events:

```text
rho = [0.10, 0.55, 0.90, 0.985]
eta = [1.00, 0.55, 0.20, 0.060]
```

This makes the banks a declared multiscale basis rather than four
underidentified learned timescales. The recurrent numerical state is exactly
64 `f32` values (256 bytes). Canonical metadata is one H4 frame offset (`u16`),
one observation count (`u32`), and a four-token ring (`[u32; 4]`): 22 bytes.
The total serialized runtime state budget is therefore 278 bytes per arm.

Each separately fitted geometric and plain arm has exactly 120 `f32` parameter
values (480 bytes): three `4 x 4` key/value/query maps, four `4 x 4` bilinear
readout maps, and the eight fixed policy values above. Thus 112 values are
fitted and eight are frozen policy constants. The permuted control reuses the
geometric parameters. Both fitted arms plus the shared 39,648-byte predecessor
artifact have a payload budget of 40,608 bytes before headers, CIDs, and schema
metadata. No arm may exceed these state, fitted-parameter, or readout budgets.

Construction features use only the final traced layer. A fixed BLAKE3-signed
reduction folds its nine heads and sixteen R4 blocks per head into one 4D value
for each of query, current key, and current value, in strict
`(head, block, lane)` order with fixed `1/sqrt(144)` scaling. The geometric fit
maps current-gauge features through the canonical H4 frame registry; the plain
arm fits the same targets in a common model frame. Runtime token features are
deterministic BLAKE3-derived unit R4 vectors and therefore require no learned
token table.

The role maps use deterministic ridge normal equations (`lambda = 2^-10`, 64
fixed Gauss-Seidel sweeps, canonical document/event/lane order). Readout maps
use 512 fixed full-batch gradient steps (`lr = 2^-5`, `lambda = 2^-10`) on
row-centered teacher log-probability residuals over the unchanged suffix
support. There is no RNG, early stopping, or anti-repeat decoding rule.

This first state student is a representation experiment, not deployed exact
lowering. Compiler and experimental runtime arithmetic may be ordinary host
arithmetic. H4/Q29/ternary/table lowering remains a later gate after the
stateful representation wins.

## Alternatives considered

- **Scale the suffix table:** rejected for this rung because it increases
  lookup density without testing whether causal state supplies attention.
- **Resume intrinsic distance, tangent readout, or resonance replacement:**
  rejected because their frozen predecessor gates are negative or parked and
  ordinary softmax already supplies the working attention teacher.
- **Compile a full decoder immediately:** rejected because the nine-position
  bounded gate can falsify this representation much more cheaply.
- **Use held-out Q/K/V at runtime:** rejected as leakage; held-out teacher
  fields are reveal-only scoring data.

## Consequences and tradeoffs

- A positive result establishes only bounded recurrent state evidence on the
  frozen support. It does not establish full-vocabulary likelihood, coherent
  general generation, reasoning, transformerless deployment, or geometric
  efficiency.
- A negative geometric-control separation falsifies the claim that R4/Spin
  transport caused the improvement, even if the geometric arm beats the old
  suffix table.
- A plain recurrent win with no geometric win advances recurrent compilation
  but retires this geometric state representation before scale.
- Exact byte reload, construction/held-out separation, and sealed predictions
  are prerequisites, not post-hoc quality checks.

## Three-phase execution boundary

### Phase 1: compile and seal

- Reload the already-frozen construction bundle through a canonical,
  fail-closed typed decoder.
- Fit every arm from construction documents only.
- Serialize the complete artifact, configuration, implementation revision,
  trace CID, arm budgets, and permutation identity.
- This is a separate invocation that cannot accept the held-out result or
  teacher-judge path.

### Phase 2: source-free held-out execution

- Start from the declared initial state.
- Read only causal token IDs, canonical R4/Spin addresses/frames, and artifact
  bytes.
- Seal predictions, transition hashes, provenance counters, suffix depths, and
  the autonomous continuation before opening held-out teacher traces.
- This is a separate invocation that cannot accept source weights, source
  traces, teacher distributions, targets, or a reveal result.

### Phase 3: reveal and score

- Open the frozen held-out teacher rows only after Phase 2 is immutable.
- Score the sealed predictions on the shared top-32 support.
- Report all arms, destructive-control deltas, causal/input audit, exact
  replay, and cycle checks.
- This invocation consumes the already-sealed predictions; it cannot refit or
  rewrite the artifact or state run.

## Runtime input boundary

Allowed:

- prior compiled recurrent state;
- observed causal token IDs;
- independently reproducible canonical R4/Spin route and frame identity; and
- frozen artifact bytes.

Forbidden:

- held-out Q/K/V, attention weights, aggregates, decoded heads, logits, or
  targets;
- source checkpoint weights, source traces, source forwards, future tokens,
  Ollama, Gemma, or a hosted provider.

Every allowed category receives an explicit read counter or provenance digest.
Every forbidden source must remain absent or zero.

## Run contract

- **Metric to move:** shared-support covered teacher cross-entropy from
  `2.660721`; secondary gates are teacher top-1 `3/9`, actual-next top-1 `2/9`,
  and the period-two continuation failure.
- **Reachability ceiling:** exactly nine frozen non-BOS context-bearing
  held-out positions are scoreable under the existing top-32 support. Covered
  teacher mass is `422,875 / 589,815 = 71.696210%`. One changed decision is
  `1/9 = 11.111...` percentage points, so top-1 evidence is coarse. Teacher
  top-1 can improve by at most six decisions (`+66.666667pp`) and actual-next
  top-1 by seven (`+77.777778pp`); covered CE has the theoretical floor zero.
  No claim is reachable for the other 48 held-out positions, uncovered mass,
  or full-vocabulary NLL.
- **Cheap instrument:** canonical document/bundle reload, exact reserialization,
  expected-CID binding, malformed-count/truncation/trailing-byte/one-byte-tamper
  rejection, shape audit, construction-only fit audit, runtime-input audit,
  and one tiny deterministic state/replay fixture.
- **Binding verdict to launch:** every cheap instrument passes and the sealed
  construction artifact is byte-identical on two compiles. Otherwise no
  held-out run launches.
- **Material geometry-control loss:** permuted CE minus geometric CE must be at
  least `0.10` nats per position, geometric teacher top-1 must exceed permuted
  top-1 by at least one of nine decisions, and at least one context-bearing
  position must have a distinct state checksum and output distribution.
- **Positive branch:** if the full geometric arm passes every promotion gate,
  advance it to a larger leak-free natural-language state-cell qualification;
  do not lower it yet.
- **Negative branch:** if the plain recurrent arm wins but geometry does not,
  retain the recurrent compiler and retire/repair the R4 state representation;
  if neither recurrent arm beats the suffix baseline, stop this cell and audit
  trace observability/readout before adding data or dimensions.
- **Cost estimate:** loader/preflight should take minutes on one process. The
  bounded four-arm fit/evaluation must declare measured wall time and worker
  count after the cheap fixture. If fitting is projected above 15 minutes, a
  one-document 1-worker versus 4/8-worker canary must first prove identical
  artifacts/results, useful worker activity, measured speedup, and a finite
  ETA. No multi-hour run is authorized by this record.

## Promotion gate

All are required on the frozen nine-position set:

- geometric covered teacher cross-entropy strictly below `2.660721` and below
  both newly fitted controls;
- teacher top-1 strictly greater than `3/9`;
- actual-next top-1 strictly greater than `2/9`;
- material degradation under deterministic transport/state permutation;
- specifically, at least `0.10` nats worse CE, at least one fewer teacher
  top-1 decision, and a distinct state/output witness under permutation;
- exact artifact reload and byte-identical causal replay;
- zero source forwards, no future reads, and a field-by-field runtime-input
  provenance audit; and
- an autonomous continuation with neither period-one nor period-two cycling.

## Immediate action ledger

| Action | Status | Evidence |
|---|---|---|
| Refresh live programme/issue state and isolate a clean worktree | `PASS` | base revision and issue metadata above |
| Preserve and identify the frozen four-document bundle | `PASS` | CID and byte count above |
| Freeze architecture, leak boundary, controls, and run contract | `PASS` | this record |
| Implement canonical document and bundle reload | `IN_PROGRESS` | result pending |
| Run fail-closed intake and deterministic reload tests | `NOT_RUN` | blocked on loader |
| Implement matched recurrent state arms | `NOT_RUN` | blocked on intake |
| Seal/reveal the bounded result | `NOT_RUN` | blocked on preflight |

## Known risks

- Thirty-eight construction events are a high-variance sample; this is why
  gates are frozen and readout fitting is regularized.
- Gauge equivalence may make the coherent and plain arms indistinguishable. If
  so, geometry has not supplied the measured effect.
- The fixed final-layer signed reduction may discard useful trace information.
  A negative result triggers an observability audit, not automatic dimension or
  corpus expansion.
- The capped suffix support limits what CE can measure and may exclude the
  actual token.
- The prior raw result contains held-out teacher distributions and is forbidden
  as a compiler or Phase-2 input.


### comment 5469172206 — 2026-08-30T14:10:10Z — Casey-allard

### Intake prerequisite checkpoint — PASS locally, fitting still withheld

The new typed loader parsed the preserved 45,205,493-byte `R4STB001` construction bundle in 1.62 seconds without opening source weights. It matched bundle CID `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`, all four ordered nested document CIDs, and byte-identical reserialization.

Focused evidence before concurrent state-kernel edits: document/bundle loader tests 10/10, old trace-experiment tests 3/3, and library clippy with warnings denied passed. The loader rejects truncation, trailing bytes, malformed headers/schema/UTF-8/counts, outer-CID substitutions, nested-CID reordering, and duplicate document identities; freeze validation additionally rechecks construction identities/tokens and independently reconstructed H4 frame offsets.

This is the mandatory intake result only. The final implementation revision is not frozen yet and no recurrent fit, held-out seal, or reveal has launched.

### comment 5469358065 — 2026-08-30T14:45:31Z — Casey-allard

### Authoritative Phase 1 freeze — committed before held-out execution

Implementation revision: `25569057f2d770dd2ffb0f10b6d2af0a985a6bd4`.

Construction-only compile completed in 9.663 seconds with no source-model, held-out-input, or judge reads. The 45,205,493-byte bundle and four nested trace CIDs matched the frozen predecessor. Two compiles were byte/CID identical and canonical reload was exact.

- state artifact: 40,692 bytes, `blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b`
- construction digest: `blake3:c30af1d0e75ad4b9fb6c3df365084479a1ce95614dfbca4744a0b74aa0d87b1c`
- Phase-1 freeze: `blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb`
- exact state/fit budget: 278 bytes state per arm; 120 parameter values (112 fitted) / 480 bytes per fitted arm

This comment commits the Phase-1 identities before Phase 2 opens the causal-input-only fixture. No held-out score or teacher distribution has been opened.

### comment 5469361024 — 2026-08-30T14:46:08Z — Casey-allard

### Authoritative Phase 2 seal — committed before judge reveal

The source-free run accepted only the 40,692-byte artifact, Phase-1 provenance freeze, and fixed causal-input JSON. It produced seal `blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060` with allowed-input digest `blake3:ce6d38bf8647fa36ea7480654a5bc3fc4e5b4d51663dfaf27767f357c6cf9e02`.

Before opening the judge:

- all four arms sealed 57 held-out predictions and one 16-token continuation;
- exact duplicate execution matched byte-for-value;
- source-model reads/forwards, construction-trace reads, judge reads, target reads, and future-token reads were all zero;
- held-out work ledgers were suffix/plain `57 reads, 0 transports`, geometric `57/56`, and permuted `57/56/56`;
- continuation ledgers were 19 observations, with geometric 18 transports and permuted 18 transports + 18 permutations;
- the sealed autonomous IDs are identical across arms and visibly retain a terminal period-2 cycle.

No teacher distribution or held-out score was opened to create this seal. Phase 3 will now compare this immutable seal with frozen predecessor result `blake3:e48b4172e02fc84eef9e00024ac6602b790d8230026a979d2f71c552ddca0cd4`.

### comment 5469410048 — 2026-08-30T14:56:11Z — Casey-allard

## Authoritative reveal result

The three-phase #1011 run completed at implementation revision 25569057f2d770dd2ffb0f10b6d2af0a985a6bd4. Execution integrity passed; promotion failed.

| Arm | Covered teacher CE | Teacher top-1 | Actual-next top-1 |
|---|---:|---:|---:|
| frozen suffix | 2.660721032 | 3/9 | 2/9 |
| plain recurrent | 2.660770919 | 3/9 | 2/9 |
| geometric recurrent | 2.660705367 | 3/9 | 2/9 |
| transport-permuted | 2.660729215 | 3/9 | 2/9 |

The geometric arm improved over the suffix by only 0.000015665 nats. Permuted minus geometric was 0.000023848 nats against the frozen 0.10 materiality floor. No teacher or actual-next top-1 decision changed. Every arm emitted the same period-two continuation:

    in , Scotland, Scotland, Scotland, Scotland, Scotland, Scotland, Scotland

Exact artifact reload/replay passed; all forbidden runtime-read counters were zero; candidate support, fixed teacher mass, derived fields, and work ledgers matched exactly.

Artifacts:

- state artifact CID: blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b
- construction freeze CID: blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb
- source-free seal CID: blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060
- structured-result CID: blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824

Terminal: STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION.

This is a clean negative for the frozen 4D signed-reduction/token-derived state cell, not a failure of ordinary R4/Spin causal softmax attention. Scale, state widening, resonance replacement, exact lowering, WASM promotion, and release remain refused.

The evidence-forced successor is #1012: a construction-only leave-one-document-out observability ladder over the full final-layer Q/K/V trace, fixed 576-to-4 reduction, recurrent features, and readout. Document 13 is revealed and permanently barred from promoting a repair. Protected delivery for the #1011 implementation and evidence is being prepared now.


---
_Comment/review ids in this file: 5469109496, 5469172206, 5469358065, 5469361024, 5469410048_

