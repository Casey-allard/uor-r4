# Explicit learned-parameter transfer for the radial reader comparison

The fourth lab now implements a new adaptation lineage from a retained learned
Dot checkpoint into continuous Lorentz and LorentzAffine readers. This preserves
learned language parameters while changing the reader law explicitly. It is not
an ordinary resume, a new accepted model, or evidence of language advantage.
The [work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5852076941)
owns this implementation and its zero-update observation.

## Parent and initialization

The selected local parent is
`.uor-models/investigations/language-continuation-20260925/fit-quaternion-6/checkpoint-final`:
continuous F32 Dot, quaternion state256/read64/context256/vocabulary4096,
model seed240924, optimizer step15672 and64,192,512 historical target visits.
Its prose/retention result remains negative. This is distinct from the accepted
quantized/learned-rounding lineage; no quantized artifact is stripped into a
continuous source.

The transfer declaration binds the parent checkpoint and campaign SHA-256,
clock and historical exposure. Application verifies the complete sealed parent,
actual model configuration, evaluator and tokenizer bytes, optimizer recipe,
Full admission and every shared array's shape, dtype and finite values. All21
shared tensors (1,678,466 scalars) are copied exactly, including query/key biases,
NoRead, age, value, copy, transport and output parameters. Only log-beta0 and the
constructor radial offset are newly initialized. There is no optimizer migration.

Both children use fresh Adam moments/clocks, local step0 and data seed240927.
Their model seed and historical exposure remain recorded separately. The new
sampling seed changes the stream; it does not turn previously exposed training
data into unseen data or promise nonoverlapping windows. The initial campaign
retains B16/T256 and two gradient shards. Model capacity, training length,
evaluation length, memory access and vector widths remain distinct fields.

## Persistence and scope

`Campaign.shared_parameter_transfer` is optional and omitted for legacy campaigns.
`fresh_model_with_provenance` applies it before optimizer construction and returns
an immutable receipt with each copied array hash, parent identities, actual
initial radial scalars and zero clocks. Fitting carries that receipt through
checkpoint, evaluation, calibration and export metadata. Resume loads its own
evolved weights and Adam state without reopening or recopying the original
parent. Historical hashes describe initialization, not current trained arrays.
Later valid window transitions preserve targets per update and retain the
original transfer history; each resume still binds its immediate parent.

The old transport comparator does not define matching for transferred parents
and rejects this study rather than silently adopting a new comparison. Existing
Dot and Lorentz numerical contracts are unchanged. LorentzAffine remains a
continuous offline control with no adopted quantized or integer serving path.
The optional third argument to `radial-startup` accepts a complete Lorentz
transfer campaign and constructs the matching affine child using the same path.
It saves both actual receipts and separates B16 training configuration from the
four-window zero-update observation. Supplementary prompt IDs and their observed
encode/decode round trip are recorded explicitly.

## Validation and research decision

Ten focused checks pass on source `50c4e3a4`: four transfer checks, four retained
initializer checks and two calibrated-checkpoint checks. Build/check commands
took822.46seconds, including60.22seconds of functional checks. Maximum command
RSS was1,734,459,392bytes. Source review corrected the historical-window/current-
window mismatch and an inaccurate initializer description. The persisted-resume
fixture checks identical next updates after removing its synthetic original
parent. These small real gradient/Adam fixture updates are separate from the
retained candidate, which received zero updates.

The actual learned-parent startup completed in183.70seconds (184.73seconds under
the supervisor), with two Rayon threads and maximum child RSS846,594,048bytes.
It ran under shared machine load using the test profile; no optimized serving,
throughput or energy claim follows. The complete report is sealed and passed the
native verifier. An independent byte-level audit confirms all21 array hashes
against the real parent safetensors file; the two receipts differ only in target
geometry. The [evidence receipt](../evidence/radial-parameter-transfer-validation-2026-09-27.json)
binds source, executable, parent, actual outputs and resource records.

| Four fixed full256 windows, zero updates | Lorentz | LorentzAffine |
| --- | ---: | ---: |
| Initial next-token NLL, nats | 2.389537 | 2.400457 |
| Mean causal read mass | 0.120842 | 0.117484 |
| Mean conditional read entropy, nats | 4.203201 | 4.206362 |
| Finite nonzero measured gradient arrays | 6/6 | 6/6 |
| Exactly zero causal read positions /1,020 | 0 | 0 |

All shared arrays match before observation and remain unchanged afterward.
Both eight-token continuations are `toys. One day, he found a`; both hit the
recorded eight-token cap. The prompt has one leading BOS and its observed
content round trip matches the decoded source IDs. This tiny continuation is
neither a prose qualification nor evidence of an advantage over the Dot parent.
Connected finite gradients establish a signal on these inputs, not useful
learnability or an adequate adaptation dose. No immediate numerical failure
supports a calibration repair or sweep.

At the radial-startup record, no adaptation dose had been selected. The later
[matched study plan](radial-adaptation-study-2026-09-27.md) selects a fixed
adaptation screen, pending its complete resource projection. A startup can reveal lost
activation, finite gradients and immediate disturbance of a learned model;
it cannot establish learnability at a useful dose, final prose or efficiency.
Both radial paths have connected finite credit here; select one matched adaptation study
with useful generated behavior as its outcome. If the transferred scale destroys
credit, diagnose that specific representation/calibration incompatibility before
spending a long dose; do not sweep arbitrary thresholds.

Lorentz versus affine isolates nonlinear distance weighting within the same
radial representation. A later claim of improvement over the retained language
path additionally needs a Dot arm with the same optimizer reset, new data stream,
objective and dose. The unchanged old parent supplies a no-update reference,
not that fitted control. Mainline termination work retains its own optimizer and
stream and changes the objective; it must remain separately owned and is not a
matched control for this study. No broad suite, repeated startup or decoder sweep
is required after the declared implementation risks are resolved.


## Same-geometry Dot reset control

The follow-up [work card](https://github.com/UOR-Foundation/uor-r 4/issues/973#issuecomment-5852324240)
extends the same explicit transfer protocol to Dot. Its target has the same 21
shared arrays and no radial scalars or initializer. Radial receipts retain their
existing JSON scalar object; Dot omits it. Resume still loads the evolved child,
without recopying the source or resetting Adam. Four focused checks pass on
source `998ff0b3`, including actual tiny updates and persisted resume for all
three reader geometries. The previous initializer and calibration results retain
their scope; no unrelated broad suite was required.

One actual zero-update Dot witness loaded the retained parent independently of
the transferred child. All 21 copied arrays, the complete prediction tensor on
the same four full256 windows, the NLL bits and all eight generated decisions
match exactly. Both models' arrays remain unchanged afterward. A newly
constructed optimizer has step 0 and all per-parameter clocks 0; parent Adam was
not loaded. The receipt's copied-array manifest also matches both previously
observed radial children. Source, actual executable, inputs, generation records,
fresh-optimizer fingerprint and complete sealed file inventory are bound in the
[Dot evidence](../evidence/dot-reset-validation-2026-09-27.json).

Dot's initial next-token NLL is 1.996717 on these exposed windows, versus the
previous Lorentz 2.389537 and Affine 2.400457 observations. The radial reader change
therefore starts with a measurable disturbance to this learned parent. This is
startup evidence, not an adapted ranking. The Dot continuation is
`toys and read them all day.`; its eight-token cap and newline are retained in the
raw record. One short continuation does not qualify prose or chat.

Build/check commands took 116.58 seconds, including 55.75 seconds of focused tests;
maximum command RSS was 1,172,553,728 bytes. The supervised actual witness took
35.73 seconds with peak child RSS 185,073,664 bytes, two Rayon threads, test profile
and no `cpu-accelerate` feature. It performed zero candidate updates. This cost is
not comparable to the earlier radial forward/backward workload and is not an
optimized throughput result. All execution occurred under shared machine load.
The declared implementation checks are complete; the next work is the costed
matched adaptation and useful-output comparison, not another startup gate.

After integrating Google's PR #1410 (`5d3a9932`), a separate offline training-package
compile passes on `c4d1837d` in 82.35 seconds. That upstream change leaves the
continuous training/model/tokenizer/configuration sources untouched. The actual
Dot witness and transfer checks retain source `998ff0b3`; their scope is not
extended to the changed integer serving paths. Existing unrelated warnings are
recorded in the build log, not repaired by this deliverable.
