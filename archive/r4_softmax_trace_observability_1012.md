# R4 softmax trace observability ladder (#1012)

- **Status:** `MEASURED / INSUFFICIENT_SUPPORT_COVERAGE`; attempt 02 completed
  the frozen four-fold measurement and exact replay. The result cannot assign
  signal loss to a boundary because one fold covered less than the required
  50% of recorded primary teacher mass.
- **Owner:** #1012 under attention issue #973 and programme root #820.
- **Predecessor:** #1011, merged through the protected queue at
  `f193939bbb40d0fa8c6a3d4a5015ea6439ad7bf5`.
- **Frozen construction bundle:**
  `blake3:2de2affeff0be3dee3cc8fcd88bd83c5f049f81390870a3c78eea485c0fd62eb`,
  45,205,493 bytes and 38 recorded positions in four documents.
- **Scope boundary:** document `13` and every #1011 reveal field are burned.
  They are forbidden from fitting, calibration, model selection, or promotion.
- **Implementation revision:**
  `3ef7af9860baf55fbe43a9b5207c60fd3a737564`.
- **Result CID:**
  `blake3:11f890def300cdafd689ef7cbfcf28e46b693e669a46edfa7431c672582db4a2`.

The contract below was frozen before attempt 02 and includes the P1 hardening
completed after independent review. Attempt 01 remains recorded in the ledger:
it stopped before measurement when its original control construction failed.
The single bounded repair replaced that invalid donor schedule with the fixed
within-support rotation described below; attempt 02 is the authoritative run.

## Authoritative result and direction

Attempt 02 completed in 45.56 seconds with exact two-pass replay. Aggregate
primary-support coverage was `0.6202622204224402`, but held document `5121`
covered only `0.3469116829611222`; the predeclared per-fold 50% floor therefore
forces `INSUFFICIENT_SUPPORT_COVERAGE` and forbids boundary attribution.

On the 26 measured exact-prefix-novel events, the full current-step Q/K/V probe
returned covered teacher CE `2.215410922655504`, versus
`2.215064603216862` for the train-only suffix baseline. The signed improvement
`suffix - full` was `-0.0003463194386417179` nats with the required direction
in `0/4` folds. Both had `14/26` teacher-top-1 agreements and `6/26`
actual-next top-1 correct. The fixed label-rotation control was materially
destroyed: `control - full = 1.3807454322642605` nats in `4/4` folds. That
establishes instrument sensitivity, not useful predictive gain over the suffix
baseline.

The input and execution audits passed: exact replay, zero source-model reads or
forwards, zero document-13 reads, canonical bundle reload, and held-label
fit-identity invariance. Those integrity results make the terminal usable, but
they do not repair its insufficient support or promote the probe.

The project will not expand this support or build another observability ladder.
This bounded current-step final-layer Q/K/V trace-distillation path stops here.
The next deliverable is direct end-to-end causal softmax attention training in
R4 coordinates on a fresh untouched split, followed in the same issue by
autonomous decoded generation under
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014). Comparison tests are acceptance checks for that
implemented mechanism; they are not a new research campaign. This decision
does not claim that all trace distillation or all attention is impossible.

## Outcome sought

Localize the earliest boundary at which held-document next-token signal is lost
between the established ordinary causal R4/Spin Q/K/V softmax trace and the
negative `R4SoftmaxTraceStateStudentV1` result. This is an observability audit,
not a new attention architecture and not a model-promotion run.

#1011 passed its artifact, source-free execution, causal-input, and replay
contracts, but the geometric recurrent arm was only `0.000015665` nats better
than the suffix baseline and only `0.000023848` nats better than the
transport-permuted control. Every arm retained the same top-1 counts and the
same period-two decoded loop. The immediate question is therefore where the
teacher signal disappears, not whether a larger version of that cell works.

## Frozen folds and score slices

Fold order is the canonical construction-bundle order. Every document is
evaluated once and is excluded from every fit performed for its fold.

| Held document | Training events | Test events | Non-BOS | Exact-prefix-novel |
|---|---:|---:|---:|---:|
| `14` | 30 | 8 | 7 | 7 |
| `657` | 30 | 8 | 7 | 3 |
| `4579` | 28 | 10 | 9 | 9 |
| `5121` | 26 | 12 | 11 | 7 |
| **Total** | -- | **38** | **34** | **26** |

The 26 exact-prefix-novel non-BOS events are the binding transfer metric. The
34 non-BOS events are reported as a secondary diagnostic. All documents share
the BOS row, while documents `657` and `5121` share positions `0..=4`; reporting
only the unstratified set would overstate transfer.

Each fold compiles its suffix support from its three training documents only.
One ordered support of at most 32 candidates is then reused at every boundary
within the fold. Teacher targets come from the frozen recorded top-32 logits
and are renormalized over the measured support overlap. Zero-overlap rows and
covered Q16 mass remain explicit.

## Four nested boundaries

### Boundary 1: full ordered current-step trace

Use final layer 29 and flatten the complete recorded current-step trace as:

```text
X[t] = (role Q/K/V, query head, R4 block, lane)
shape = 3 x 9 x 16 x 4 = 1,728 finite f32 values
```

Current K and V are already transported into each query head's gauge by the
qualified teacher trace. “Full” in this record means only these current-step,
final-layer Q/K/V values. It does not include the recorded weighted attention
aggregate or the decoded head output. No source checkpoint or new forward pass
is allowed.

### Boundary 2: frozen signed reduction

Apply `signed_reduce_final_layer_r4` independently to Q, K, and V:

```text
Z[t] = (Q[4], K[4], V[4])
shape = 12 finite f32 values
```

This is the exact reduction used by #1011. The intake census found 31 unique
full rows and 31 unique reduced rows; every fold's centered reduced training
matrix has rank 12. Therefore the reduction is live and introduces no new
exact collision on this corpus, but a loss between boundaries 1 and 2 cannot
be assigned to reduction alone: their distinct candidate-conditioned sketches
also leave projection variance as a possible cause.

### Boundary 3: token maps and recurrent state

Replay fold-fitted plain and geometric #1011 cells, resetting state at each
document boundary. For each candidate expose the natural readout tensor:

```text
Phi[c] = response[4 banks, 4 lanes] outer candidate_query[4 lanes]
shape = 64 finite f32 values
```

The diagnostic snapshot is read-only: it must not change the recurrent state,
prediction, checksum, or runtime counters.

### Boundary 4: current residual logits

Measure the currently deployed experimental readout exactly:

```text
logit[c] = base_suffix_logit[c] + dot64(readout_weights, Phi[c])
```

Report base, residual, total logit, stable-softmax probability, and a fixed
train-only residual-scale diagnostic over exactly:

```text
alpha = 0, 1/16, 1/8, 1/4, 1/2, 1, 2, 4, 8, 16
```

Alpha `1` is the #1011 mechanism. The lowest train-only covered CE selects the
diagnostic alpha, with deterministic preference for the smaller alpha on a
tie. The sweep diagnoses scale but cannot select a promoted model.

## Matched diagnostic probes

Boundaries 1 through 3 receive exactly 64 learned residual weights and no
bias. Boundary 3 already supplies 64 natural features. Boundaries 1 and 2 use
a deterministic candidate-conditioned signed sketch:

```text
mask[i] = next_u64(BLAKE3_XOF(domain, boundary, candidate_token, width))
phi[j]  = sum_i sign(bit_j(mask[i])) * x[i] / sqrt(width)
```

One BLAKE3 XOF supplies the canonical sequence of 64-bit sign masks for each
boundary/candidate/width tuple. The masks are cached and reused across folds;
this preserves all 64 dense projection lanes without constructing one hash per
lane and scalar. The boundary-specific domain, canonical scalar order,
normalization, candidate support, optimizer, regularization, fit rows, and
reduction order are fixed.

Every probe boundary is scaled by train-only per-lane uncentered RMS with a
fixed `1e-12` floor; the same target-blind scales are applied to the held
document. There is no RNG, early stopping, or held-fold hyperparameter
selection. The probe uses zero initialization, `lambda = 2^-10`, 512
full-batch steps, and a fixed backtracking learning-rate ladder. Its objective
weights rows by globally normalized raw covered Q16 mass; within each row, the
covered teacher targets are renormalized over that row's unchanged candidate
support. The reported covered CE uses the same raw-mass aggregation.

The matched destructive control is one fixed left rotation of the teacher Q16
labels across each training row's existing ordered candidate slots. It changes
no candidate token, feature, base logit, support membership, or covered Q16
mass. The row identities, rotation policy, and shift of one are
content-addressed once per fold and reused at every boundary. There is no donor
selection, support alignment, label search, or retry. A support with fewer than
two slots, zero mass, an unchanged rotated label vector, or any mass difference
fails closed before a result can be issued.

Upstream capacity is reported separately from the 64-weight probe:

- boundaries 1 and 2: parameter-free features plus 64 readout weights;
- boundary 3: the existing 48 role-map values, fixed four-bank state, and a
  matched 64-weight diagnostic readout;
- boundary 4: the same role/state machinery plus its actual 64 fitted readout
  weights.

## Causal and leakage contract

Allowed inputs are the canonical frozen construction trace bundle, its exact
predecessor freeze, and the implementation revision. The command accepts no
source checkpoint, teacher service, judge, document-13 input, #1011 seal or
result, fold selection, optimizer choice, or tuning value.

Every fold records fit-document identities, fit artifact/support CIDs, its
label-control mapping CID, a composite training-fit identity CID, held-document
identity, and read counters. The training-fit identity covers the training-only
suffix/state artifacts, support, preprocessing, diagnostic features and
real/control weights, and train-only alpha trials/selections. A deterministic
held-label substitution mutates the held teacher distributions and actual-next
labels, reruns the fold path, and must leave that composite fit identity exact.
The result records the original and substituted held-label CIDs, the number of
mutated held events, and the fit-identity invariance verdict; any changed
training fit fails closed.

The executable performs two physical four-fold passes. Pass 1 produces the
candidate result; pass 2 independently repeats all fold compilation, fitting,
held-label audits, scoring, and state replay. The pass ledgers and structured
fold evidence must agree exactly before the result is serialized. The final
work ledger reports the summed physical work rather than a logical estimate.

## Cheap gates

All gates below bind before the decision run:

1. expected bundle and nested document CIDs, canonical byte reserialization,
   order, shape, and `8 + 8 + 10 + 12 = 38` census;
2. absence of document `13`, #1011 seal, result, and source-model access;
3. finite/nonconstant full, reduced, state, and logit features;
4. feature variance, rank, collision, support-overlap, and zero-overlap census;
5. identical ordered candidate support and exactly 64 weights at each matched
   probe boundary;
6. target-blind fit-CID invariance under held-label substitution;
7. real labels versus the fixed, CID-bound within-support label rotation,
   including exact covered-Q16 mass preservation and fail-closed unchanged rows;
   and
8. exact fold evidence and physical-work ledger replay across two complete
   execution passes.

The decision layer additionally requires at least 50% of recorded primary Q16
teacher mass in aggregate and in every fold. Below that floor the only terminal
is `INSUFFICIENT_SUPPORT_COVERAGE`; no boundary attribution is allowed.

## Frozen pre-run contract

- **Metric to move:** exact-prefix-novel held-document covered teacher
  cross-entropy at each nested boundary; teacher top-1 is secondary. Report the
  34-row non-BOS slice separately.
- **Reachability ceiling:** 26 binding and 34 secondary events. One binding
  top-1 decision is `1/26 = 3.846154` percentage points. No conclusion is
  reachable for unrecorded full-vocabulary logits or a new population.
- **Instrument and launch verdict:** the eight cheap gates above must pass.
  The liveness census already passes exact-collision and reduced-rank checks;
  at freeze time, real-bundle intake, control, replay, and liveness checks were
  `NOT_RUN` pending the release-mode invocation.
- **Materiality:** a downstream boundary is materially worse when its covered
  CE loses at least `0.10` nats against the immediately upstream live boundary
  and the direction appears in at least three of four folds. Exact paired rows,
  per-fold deltas, and sign uncertainty remain visible even when materiality
  is not crossed.
- **Full-trace liveness:** the current-step final-layer Q/K/V probe must improve
  covered CE by at least `0.10` nats against **both** the train-only exact suffix
  baseline and its fixed within-support label-rotation control, with each improvement pointing in
  the required direction in at least three of four folds. Passing only one
  comparison does not establish the boundary.
- **Positive branch:** open exactly one repair issue for the earliest boundary
  that loses a signal present immediately upstream.
- **Negative branch:** if the current-step final-layer Q/K/V trace cannot beat
  the train-only suffix/base and fixed label-rotation control, stop this bounded
  Q/K/V trace-distillation path and write an end-to-end cell-training
  specification with a new untouched holdout.
- **Cost estimate:** minutes on CPU. No hours-long run is authorized.

## Binding interpretation

- Current-step final-layer Q/K/V fails: stop this bounded Q/K/V
  trace-distillation path; specify end-to-end training and a new independently
  frozen holdout.
- Full trace works but the reduced/sketched boundary fails: open one
  reduction-or-projection disambiguation issue. Preserve the full-trace probe
  as the upstream witness and do not assign causal blame to the reduction
  alone.
- Reduction works but state features fail: repair context-conditioned token
  key/value/query induction or the recurrent transition.
- State features work but current logits fail: repair readout calibration;
  use the fixed alpha sweep only to distinguish scale from feature failure.
- Current logits transfer: advance only to a new independent holdout. Revealed
  document `13` cannot promote the repaired mechanism again.

## Immediate action ledger

| Action | Status | Evidence |
|---|---|---|
| Deliver #1011 and refresh live parent/dependency state | `PASS` | PR #1013, merge `f193939b`; #1011 closed; #1012 activated under #973 |
| Isolate a clean #1012 worktree from live `main` | `PASS` | branch `issue-1012-r4-softmax-trace-observability` |
| Freeze folds, boundaries, capacity, controls, and decisions | `PASS` | this record and #1012 architecture comment |
| Canonical input/liveness census | `PASS_PARTIAL` | 38 rows; 31 full/reduced unique rows; centered reduced rank 12 in every fold |
| Implement read-only state diagnostic snapshot | `PASS` | exact 64-feature/base/residual/total-logit snapshot; plain, geometric, and transport-permuted mutation-invariance test |
| Implement strict four-fold observability command | `PASS` | compiler-side harness, strict CLI, fixed CID-bound label rotation, held-label fit-identity audit, structured result, aggregate replay, and two physical-pass work ledger |
| Run focused pre-hardening cheap gates | `PASS` | seven harness tests, core snapshot test, strict CLI test, formatting, clippy, claim wording, and diff integrity |
| Rerun focused gates after P1 hardening | `PASS` | eleven harness tests, seven core state-student tests, strict CLI test, formatting, library/binary clippy with warnings denied, claim wording, and diff integrity |
| Attempt 01 real-bundle control construction | `STOPPED_PRE_MEASUREMENT` | the fixed cross-document donor schedule produced a zero-overlap or unchanged row; command exited before any boundary score or result file |
| Replace invalid donor control with one fixed within-support rotation | `PASS` | one bounded construction repair; no adaptive donor search and no mechanism tuning |
| Run real-bundle cheap intake and liveness gates | `PASS` | canonical input, leakage, control-liveness, fit-identity, and replay audits passed |
| Run decision measurement and select one next action | `INSUFFICIENT_SUPPORT_COVERAGE` | aggregate primary coverage `0.6202622204224402`; minimum fold `0.3469116829611222`; exact replay; result CID `blake3:11f890def300cdafd689ef7cbfcf28e46b693e669a46edfa7431c672582db4a2` |
| Continue the observability ladder or expand its support | `STOPPED` | no boundary attribution is licensed; the measured full probe also failed to improve on suffix on the covered rows (`-0.0003463194386417179` nats, `0/4`) |
| Begin direct end-to-end causal softmax attention training in R4 coordinates | `NEXT` | [#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014); use a fresh untouched split and require autonomous decoded generation in the same deliverable |

## Nonclaims

This record does not establish a geometric advantage, coherent source-free
generation, reasoning, transformerless deployment, exact table lowering,
runtime efficiency, hosted-chat readiness, or release readiness. The
established ordinary R4/Spin causal-softmax reference remains the attention
teacher; #1012 did not localize a failed student boundary because its per-fold
support gate failed. The full
boundary includes current-step final-layer Q/K/V, not the recorded weighted
attention aggregate or decoded head output, and one predeclared signed
projection per boundary does not eliminate cross-boundary sketch variance.
Stopping this bounded path does not establish that all trace distillation, all
geometric representations, or attention itself is impossible.
