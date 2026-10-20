# Construction-only token-frame exposure diagnostic — #1082

## Frozen scope, before diagnostic inference

Issue [#1082](https://github.com/UOR-Foundation/uor-r4/issues/1082) follows
the [#1079 result](r4_zoology_language_r4_1079.md),
`LANGUAGE_R4_PRESERVED_CONTROL_WEAK`. Preserve its two-stage R4 result and
its valid but weaker token-control result. The current diagnostic is descriptive:
it does not fit a mechanism, select a replacement control, promote generation,
or infer a causal explanation from associations.

Reuse the exact #1077 reader, #1073 core, original native frame bundle, and
the two #1079 construction renderings: 10,240 rows each, 20,480 total. Keep
every token and learned soft coefficient. Each row computes fifteen role
vectors; measure the fourteen consumed by binding, excluding only the unused
question-location role. No gold role, target answer, entity mask, or changed-answer
indicator enters the reader or the token computation. Source prediction arrays
are read from the already-published #1079 result after reconstruction; no head,
binding-projection, development, or generation forward is needed.

Source envelopes remain:

- Preparation: `blake3:d9c8ad8448365b2039276fdeda6b70da53ef63fde24e02dd1dd8dea437b546a4`.
- Result: `blake3:dee107190172afcb7637d52469662ecab217847271e4bbdb0721514fcfbdc3a5`.
- Replay: `blake3:eaa17433d5cd150a2a0c52adab6104bda4c4dae26221944fcde112ef841ca597`.

The new preparation checks public/local envelopes, their relationships, all
307 files in the frozen #1079 implementation, reader/core file identities,
existing dataset and frame identities, exact construction view selection and
actual changed source matrices. Historical validation may hash development
files for identity; it does not decode or score development tensors. No
learned-model inference occurs during preparation.

## Definitions and numerical boundaries

For one clause and role, let `a_i` be the unchanged nonnegative f32 reader
weight promoted to f64, and `c_i` indicate that the source matrix actually
changes under the fixed within-clause next-frame permutation. A changed index
alone does not imply a changed matrix.

For each token, keep its true local encoding. Compute its coherent and
controlled transported values, then decode each into the shared embedding
coordinates exactly as specified by #1079. Let `delta_i` be controlled minus
coherent decoded value, a 64-dimensional vector containing all sixteen R4 blocks.

**Definition — raw changed-frame mass:** `M = sum_i a_i c_i`.
Also report `M / sum_i a_i`; the latter removes only softmax sum roundoff from
the descriptive mass fraction. A and D below use the original weights without
renormalization. The reader-weight sum must be within `1e-5` of one.

**Definition — weighted individual displacement:**
`A = sum_i a_i ||delta_i||_2`.

**Definition — net displacement:** `D = ||sum_i a_i delta_i||_2`.
The weighted triangle inequality implies `D <= A` for real nonnegative weights
and any vectors; it does not require orthogonal frames. The instrument checks
the computed values against `D <= A + 1e-12 (1 + A)`. This numerical check is
not a floating-point refinement proof. It separately checks reconstruction of
the f64 pooled-vector difference within `1e-12 (1 + max_abs_pool)`.

**Definition — actual used-role displacement:** the Euclidean norm of the
difference between the two completed f32 role vectors, promoted to f64 for
measurement, before the unchanged downstream normalization/projection. Record
the coherent used-role norm alongside it to show the scale. These norms use
the original embedding-coordinate units; the actual f32 displacement is kept
separate from D, so rounding is not disguised as cancellation.

**Definition — retained fraction after cancellation:** `D/A` when `A > 0`.
At `A=0` this ratio is undefined, represented by a documented zero placeholder
in the binary artifact and excluded only from ratio summaries. All zero
mass/displacement observations remain in their other summaries. `M=0` implies
`A=D=0`; positive changed-frame mass need not imply displacement because a
different matrix can act identically on a particular value. `D=0` with `A>0`
describes exact cancellation in the measured values.

## Measurement, interpretation, and resources

For each rendering and each used role, report mean and fixed quantiles
`0, 0.1, 0.25, 0.5, 0.75, 0.9, 1`, separately for supported and unknown rows,
and for all, recorded-changed, and recorded-retained answers within those groups.
Retained/changed compares historical coherent and token-control prediction IDs;
it is not a new correctness or generalization score. No threshold is fitted and
no subgroup chooses a new candidate. Patterns of small exposure, cancellation,
or nonzero displacement with a retained answer remain descriptive.

Before summarization, require exact agreement with the #1079 reader-attention
CID and both full fifteen-role-vector CIDs for each construction view. The new
reconstruction also compares bit-for-bit with the unchanged `_pool_roles`
helper on each batch. Guard hooks reject any downstream core or head invocation.
Learned state, tying, eval/no-grad mode, implementation and inputs are checked
before/after execution. Identity, scope, numerical-bound, or resource failure
produces a typed refusal without a substantive diagnosis.

Execution remains CPU/Apple Accelerate, four intra-op threads, one inter-op
thread, one process, batch 256, matching the retained runtime. Each phase runs
80 reader batches / 20,480 reader rows. Each arm reconstructs 307,200 role
vectors and the frozen reference helper computes another 307,200: 1,228,800
pool-vector evaluations across both arms, measuring 286,720 used-role rows.
There are zero new answer predictions, fits, parameters, controls, populations,
development evaluations, geometry changes, native exports or generated tokens.

Expected time is below the prior full #1079 run-plus-replay's 57 seconds,
because no 4,096-token head or four development views run; this is a cost
estimate, not a measured diagnostic result. The tightened budget is 120 seconds
per execution/replay, 3 GiB peak RSS, and 256 MiB new output disk. Checks are
cooperative between bindings, batches, and views; use an external process
timeout for the wall limit. No automatic retry or renewed budget follows a stop.
Per-row metrics use deterministic little-endian f64 arrays, shape
`[10240,14,7]`, in each frozen construction view's selection order, with source
input/length/group/variant CIDs and the ordered role/metric schema. One fresh
process must reproduce both raw metric artifacts and complete summary evidence.

If valid, independently review the distributions and freeze a separate successor
contract informed by what they do and do not show. If invalid, preserve the
refusal and repair the declared diagnostic boundary before drawing conclusions.
Neither branch changes the #1079 criterion or licenses a search over controls.
#973 stays open; #954 remains blocked.

## Execution ledger

At source freeze, no retained-model diagnostic outcomes had been read. Source review,
preparation identity, native admission, execution and replay entries will be
appended in that order. Only the issue's focused synthetic measurement/scope/
provenance checks are active; broad QA remains dormant.

### Prepared admission (2026-09-03)

The [exact preparation](r4_token_exposure_1082_preparation.json) completed in
4.731441 seconds without a retained-model diagnostic forward. Its CID is
`blake3:c8a4a56de77767cbbe8fd31edc83251b42a6f729e182d065caa47d981f248741`.
The 316-file implementation closure has CID
`blake3:1451084b66531dbe5eb0cb1d3b3e60e5f0aa3f4bd59fffbf582a18d4dfc9380e`;
the historical 307-file implementation is unchanged.

Each view contains 10,240 rows, including 8,192 supported and 2,048 unknown,
624,640 valid tokens and 143,360 used-role measurements. The original token
control changes 471,638 source matrices in view 0 and 461,246 in view 1,
exactly matching #1079. All 8,192 supported rows per view contain a changed
matrix. That structural opportunity does not yet measure attended exposure.

Four focused synthetic checks passed in 0.115 seconds. They exercise the
distinct cancellation/zero cases, matrix equality and padding, the fourteen-role
mask and forbidden downstream path, recorded-answer grouping and undefined
ratio exclusion, and exclusive preparation/policy binding. Ruff 0.12.11 and
claim wording passed. Independent mathematical/source review found no blocker.
Native admission and diagnostic outcome remain pending at this entry.

### Sole diagnostic and fresh-process replay

The [native admission](https://github.com/UOR-Foundation/uor-r4/issues/1082#issuecomment-5520228872)
froze the preparation and implementation above before outcome access. One run
and one fresh-process replay completed without retry or implementation changes.
Terminal: **`TOKEN_EXPOSURE_DESCRIPTIVE_COMPLETE`**.

All eight historical reconstruction checks passed: the exact reader-attention,
coherent-role and controlled-role CIDs, plus actual changed-matrix counts, in
both construction views. All batch reconstructions matched the unchanged
`_pool_roles` helper bit-for-bit. Maximum f64 displacement-to-pool closure error
was `2.7755575615628914e-16`. No zero-mass or zero-A observation occurred among
the 286,720 measured used-role rows; some masses and displacements were small,
but not exactly zero.

The intervention exposes different roles in the two renderings. This table
averages each fact-role class over the four fact slots and 8,192 supported rows
per view. M is raw attention mass; A and used displacement are embedding-space
Euclidean magnitudes. The D/A column is the mean of the defined per-role ratios,
not the ratio of the means.

| Construction view / fact role | Mean M | Mean A | Mean used f32 displacement | Mean D/A |
|---|---:|---:|---:|---:|
| 0 / owner | 0.999993821 | 0.459996382 | 0.459957289 | 0.999916174 |
| 0 / object | 0.999978704 | 0.457707939 | 0.457707669 | 0.999999412 |
| 0 / location | 0.0000349734 | 0.0000197326 | 0.0000143815 | 0.711855463 |
| 1 / owner | 0.999999272 | 0.650518344 | 0.650470561 | 0.999927260 |
| 1 / object | 0.0000391333 | 0.0000397022 | 0.0000244239 | 0.596841681 |
| 1 / location | 0.999976920 | 1.209482534 | 1.209478715 | 0.999996800 |

Both query roles have almost unit exposure in both views: owner M is
`0.9999959305` with used displacement about `0.46020749`; object M is
`0.9999846884` with displacement about `0.45747903`. Their mean D/A values
are approximately `0.9999734` and `0.9999422` respectively.

Across all fourteen used roles, supported mean M is nearly the same in the
two views (`0.714286472` and `0.714288709`), whereas mean used displacement is
`0.327743134` and `0.596970095`. Thus a row-level changed-frame opportunity or
one average attention-mass number hides the role-specific exposure differences.
The dominant displaced roles have D/A close to one: within-role cancellation
does not substantially reduce their measured perturbation. The partly cancelling
roles in the table already have tiny exposure and displacement.

The following are comparisons of the recorded #1079 prediction arrays, not new
answer scores:

| View | Supported changed / retained | Unknown changed / retained |
|---|---:|---:|
| 0 | 2,353 / 5,839 | 1,964 / 84 |
| 1 | 4,224 / 3,968 | 1,807 / 241 |

Supported changed and retained groups have very similar per-role mean exposure
and displacement. Their all-used-role mean displacements are respectively
`0.327393231` / `0.327884138` in view 0 and
`0.597038700` / `0.596897063` in view 1. Retained answers therefore coexist
with nonzero, often large, pooled-role perturbations. This observation does not
locate tolerance at normalization, projection, binding, or the head, or establish
which protected role caused an answer to survive. Those downstream computations
were not rerun. The exact per-role quantiles, supported/unknown strata and
changed/retained summaries remain in the result envelope.

The findings support role-selective weak exposure and observed answer retention
after substantial perturbation of other roles; they do not identify one causal
explanation. They give no reason to tune the old control until its revealed
50-point criterion passes. Preserve the #1079 terminal and coherent preservation.
The separately scoped next plan step is to specify removal of externally supplied
clause segmentation while retaining the frozen reader/core, known lexicon,
question form and four-fact setting; it needs its own adapter/oracle comparison
contract. No extra downstream experiment, segmentation implementation, fitting,
or generation ran in this issue.

### Resources and immutable evidence

Run process `38020` took `15.617839708` seconds internally and
`16.785676542` seconds under the external wrapper. Fresh replay process `38088`
took `15.541262500` internally and `16.873878208` externally. Totals are
`31.159102208` internal / `33.659554750` external seconds. Both processes exited
zero without reaching their separate 120-second external timeout. Maximum peak
RSS was `1,061,994,496` bytes (`0.9890594482 GiB`), below 3 GiB. The complete
local output directory measured `26,336,012` bytes after replay, below 256 MiB.
The preparation's separate `4.731441` seconds remains recorded above.

Both processes retained four CPU threads, one inter-op thread, one worker and
batch 256. Reader/core states, tying, source/input/frame identities and the
316-file implementation closure remained unchanged. The downstream guards did
not fire. There were zero new predictions, optimizer updates, parameters,
controls, populations, development tensor reads, geometry changes, native
exports or generated tokens.

- [Result](r4_token_exposure_1082_result.json): `blake3:e88501f05d4c58249806b2c9c5dabddd84eecd71256de9d4001c378cf8b9be03`.
- Complete evidence: `blake3:fcd2a0c740dc23d0dc89066fef70683e2eace0e308e05020947c38fde996d4e4`.
- [Fresh-process replay](r4_token_exposure_1082_replay.json): `blake3:3629ea8327a3e3cfc3a35d17da7d78de81ffe95f65b53266f18fac865a8240cc`.
- Construction 0 raw metrics: `blake3:26d2824790a6c2431fd4e16e625528405014dcd3fd25330e240a3ce5c414d82f`.
- Construction 1 raw metrics: `blake3:f8f9bd1ec0460a5a50538b169873b46f3aa12667fb032f3edd15d7087eddb96f`.

Raw metrics are retained locally as `construction-0.f64le` and
`construction-1.f64le` under the declared issue output root. Each file contains
8,028,160 bytes with the frozen `[10240,14,7]` schema. The fresh process reproduced
both byte streams and the entire evidence envelope exactly. The raw files are
not copied into Git. Public result and replay envelopes are exact copies of the
local immutable records. Broad QA remains `NOT_RUN`; protected queue
acknowledgements carry reviewed content, not additional test evidence.

### Independent acceptance and delivery binding

The [independent source/result review](r4_token_exposure_1082_review.md) accepts
the descriptive terminal and recommends #1085's clause-segmentation interface
contract. Its [receipt](r4_token_exposure_1082_review.json) independently checks
the canonical envelope and raw-file identities, all 316 implementation files,
numerical bounds, summary reconciliation and recorded fresh-process replay.
It ran no additional model evaluation. Integrating the already-merged workflow
adoption `11e46611b82702e005165fb0034e1adf7d119a70` changed none of the 316 bound
implementation files. No fitting or new model evaluation followed the result.
