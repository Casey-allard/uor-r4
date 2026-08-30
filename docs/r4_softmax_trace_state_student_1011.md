# R4 softmax trace state student (#1011)

- **Status:** `COMPLETE`; authoritative result is negative at the frozen gate.
  Preserve the implementation and evidence, but do not promote or scale this
  state representation.
- **Owner:** #1011 under attention issue #973 and programme root #820.
- **Decision:**
  [ADR-0005](adr/0005-predictive-geometric-connection-memory.md).
- **Base revision:** `02b6a08a3dfd8c485d5d22353e0f37ad0cfd9926`.
- **Frozen construction bundle:**
  `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`,
  45,205,493 bytes, four construction documents.
- **Predecessor:** `R4SoftmaxTraceStudentV1`, 39,648 bytes,
  `blake3:e3b48b8bd113bf71be2fe9ecb64257b4eb1516303966d9d6c2c5cbe9d46adfac`.
- **Phase-2 causal input:**
  [`r4_softmax_trace_state_causal_input_1011.json`](r4_softmax_trace_state_causal_input_1011.json),
  containing observed token IDs and the continuation prompt only; it contains
  no actual-next labels or teacher distributions.

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
metadata; the canonical artifact is exactly 40,692 bytes including its header.
No arm may exceed these state, fitted-parameter, or readout budgets.

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
stable-softmax cross-entropy, with each teacher row renormalized over the
unchanged suffix support. There is no RNG, early stopping, or anti-repeat
decoding rule.

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
- Validate the exact artifact CID against the Phase-1 construction-only
  provenance freeze, then read only causal token IDs, canonical R4/Spin
  addresses/frames, and artifact bytes during prediction.
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
- frozen artifact bytes, bound to the Phase-1 provenance freeze.

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
| Implement canonical document and bundle reload | `PASS` | 10 focused loader tests; real 45,205,493-byte byte/CID-identical reload |
| Run fail-closed intake and deterministic reload tests | `PASS` | malformed/truncated/trailing/tampered input rejection; exact nested CIDs |
| Implement matched recurrent state arms | `PASS` | six core tests; exact 278-byte state and equal 120-value arm budgets |
| Harden three-phase seal/reveal orchestration | `PASS` | five orchestration tests; strict CLI; recursive unknown-field, derived-cycle, matched-support, byte-count, and work-ledger checks |
| Seal/reveal the bounded result | `PASS_INTEGRITY / FAIL_PROMOTION` | authoritative CIDs and metrics below |

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

## Authoritative result — 2026-08-30

Implementation revision
`25569057f2d770dd2ffb0f10b6d2af0a985a6bd4` ran through three separate
invocations. Phase 1 was posted to #1011 before Phase 2; the Phase-2 seal was
posted before the frozen judge opened.

- State artifact: 40,692 bytes,
  `blake3:b617fc38e7bef1cdea76991f6e5e7cc653118451d63bcbd595f8ffd7e247ae7b`.
- Construction freeze:
  [`r4_softmax_trace_state_freeze_1011.json`](r4_softmax_trace_state_freeze_1011.json),
  `blake3:67cf67bb46b94cf5644b8dde286e89adb7e49159b3749790dffb500d8047fedb`.
- Source-free seal:
  `blake3:64587526f7883ab046e884a28b6af7e9e89818c9ead2039f8c995de7fb483060`.
- Structured result:
  [`r4_softmax_trace_state_student_1011_raw.json`](r4_softmax_trace_state_student_1011_raw.json),
  `blake3:dc04a8a8b21750799db2d451c8237d1e62cf90ffa74561fb54272b1e9704c824`.
- Measured phase times: 9.663 seconds compile, 1.461 seconds seal, and
  0.998 seconds reveal on this host.

Every arm covered the identical `422,875` Q16 teacher mass on the same nine
context positions:

| Arm | Covered CE (nats) | Teacher top-1 | Actual-next top-1 |
|---|---:|---:|---:|
| Frozen suffix | 2.660721032 | 3/9 | 2/9 |
| Plain recurrent | 2.660770919 | 3/9 | 2/9 |
| Geometric recurrent | 2.660705367 | 3/9 | 2/9 |
| Transport-permuted | 2.660729215 | 3/9 | 2/9 |

The geometric arm's CE is `0.000015665` nats below the suffix and
`0.000023848` below the permuted control. That is not a material effect: the
frozen control threshold was `0.10` nats, no teacher or actual-next top-1
decision changed, and the permuted control lost no top-1 decision. Although
all nine context-bearing states and distributions differed from their
permuted counterparts, those differences were not selection-bearing.

All four autonomous continuations emitted the same token IDs:

```text
[281, 216, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291, 28, 9291]
```

With the pinned tokenizer this decodes to:

```text
 in , Scotland, Scotland, Scotland, Scotland, Scotland, Scotland, Scotland
```

The period-two cycle therefore remains. Exact replay passed; every forbidden
runtime counter was zero; the geometric held-out ledger was 57 observations
and 56 transports; the permuted ledger added exactly 56 permutations. These
are integrity prerequisites, not quality promotion.

The terminal is
`STOP_R4_SOFTMAX_TRACE_STATE_STUDENT_REPAIR_OR_RETIRE_REPRESENTATION`.
This result falsifies the current 4D signed-reduction/token-derived state cell
at the declared gate. It does not falsify ordinary R4/Spin softmax attention,
which remains the established teacher/reference mechanism.

## Next decision

Do not add corpus scale, state dimensions, exact lowering, resonance, WASM, or
release work. Run
[#1012](https://github.com/UOR-Foundation/uor-r4/issues/1012), one
construction-only, leave-one-document-out observability audit that measures
the same teacher-relative candidate loss at four explicit boundaries:

1. the full ordered final-layer Q/K/V trace blocks;
2. the fixed 576-to-4 signed reduction;
3. the token-derived role maps and recurrent state features; and
4. the fitted residual readout/logit scale.

If full trace features transfer but the 4D reduction does not, replace only the
reduction with a structured per-head/multiscale geometric representation. If
the reduction transfers but state features do not, repair context-conditioned
key/value induction. If state features transfer but logits remain inert,
repair readout calibration. If even full traces do not transfer, stop trace
distillation and train the recurrent cell end-to-end under a new independently
frozen holdout. The already-revealed document 13 cannot promote a repaired
mechanism again.
