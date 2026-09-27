# Fourth lab: integrate the existing native Lorentz reader

September 26 local / September 27 UTC, 2026. Source review of
[Claude PR #1401](https://github.com/UOR-Foundation/uor-r4/pull/1401) at
`dcf979df687ac867277ad8f3fc7dbc53393f67a8`. Two independent specialists and the
principal inspected source and reported evidence. **No build, model execution,
new empirical result or independent artifact replay occurred in this review.**

## Decision changed by concurrent research

Reuse the branch's native `ReadGeometry::{Dot,Lorentz}` / `read_scores` seam
before implementing a competing attention configuration/export path. The native
reader preserves full admission and the existing values, copy path, age terms,
NoRead and output. The fourth lab should own independent comparative review and
integration while coordinating implementation ownership with its author.
The [signed-2I proposal](fourth-lab-geometric-attention-2026-09-26.md) remains a
distinct later hypothesis. Its query/key unit coding must explicitly control
or retain radial information; the Lorentz work supplies a reason to investigate
that information before discarding it.

The native reader is separable from the PR's converted Llama/SmolLM2 engine.
The proposed D10 changes the backbone, numerical-multiplier and unsafe-SIMD
policies. It is recorded as owner direction in that lab's session, but has not
been confirmed in this fourth-lab conversation. Owner clarification is pending.
Do not adopt that policy merely because it appears in a branch document. Native
source/evidence review can proceed independently, without importing the new
backbone or changing the current shared contract.

## Mechanism and what the comparisons identify

The [native score](https://github.com/UOR-Foundation/uor-r4/blob/dcf979df687ac867277ad8f3fc7dbc53393f67a8/crates/uor-r4-training/src/joint_model.rs#L1809)
uses the hyperboloid lift and
`d = acosh(sqrt(1+||q||²)*sqrt(1+||k||²) - q·k)`, then
`score = -beta*(d-offset)`. With a fixed query and equal key norms, distance
ranking is the same as dot-product ranking; score spacing still changes.
Variable key radii can change ranking. The common key offset changes competition
with NoRead, but not the distribution among keys conditioned on reading.
Initialization, temperature, radial information and read/copy dependence remain
alternative explanations to distinguish when making a geometry attribution.

The branch reports the following development results; they are **reported by
the other lab and source-reviewed here**, not independently replayed:

- At context 256 / state width 128 on its code corpus, two seeds favor flat-start Lorentz
  over the retained Dot initialization by approximately 0.051 / 0.061 nats/target.
  Equal-start flat Dot was explicitly NOT_RUN at this context. The source shows
  common matrix initialization and matched draws/windows within the declared
  arms. [Context256 record](https://github.com/UOR-Foundation/uor-r4/blob/dcf979df687ac867277ad8f3fc7dbc53393f67a8/docs/integration/hyperbolic-cycle3-2026-09-26.md#L332).
- Gains are most consistent on distant copies. The scope effect changes sign
  across seeds after controlling for distance. Its radius intervention changes
  the read distribution, so an angle-only interpretation is insufficient.
  These observations do not establish a hierarchy advantage.
- After 300 matched quantization-aware updates, reported integer Lorentz-minus-
  Dot differences remain about -0.062 / -0.059 nats. Mean emulator disagreement is
  small, but maximum probability/state differences exceed 0.01 in the reported
  population. That maximum-error criterion remains unmet; mean agreement does
  not retroactively pass it. Full-scale TinyStories, M1 speed/energy and useful
  generated language remain unmeasured. [Integer results and limits](https://github.com/UOR-Foundation/uor-r4/blob/dcf979df687ac867277ad8f3fc7dbc53393f67a8/docs/integration/hyperbolic-cycle3-2026-09-26.md#L449).

The report explicitly says its scratch harness, BPE tool, flat-Dot/Euclidean
controls and analysis scripts were not committed. Model/report roots belong to
another sandbox. Request those retained inputs and exact source/config hashes
once before selecting a reproduction or extending the results. Their current
local absence makes independent replay unavailable; it is not negative model
evidence. Do not compare this corpus NLL numerically with the retained
TinyStories result or start the proposed six-run full-scale campaign by default.

## Concrete implementation findings already returned to the author

The [source-review comment](https://github.com/UOR-Foundation/uor-r4/pull/1401#issuecomment-5851426561)
records two bounded defects:

1. `lorentz.rs:130–154` promises beta<2^31 but checks only the rounded octave.
   `LorentzRead::new((87,-2),(0,0))` has log(beta)=21.75>31*ln(2), while its
   rounded octave is 31. Check the actual resulting Q32 scale against 2^63.
   This is a parameter-validation defect, not evidence that measured runs
   overflowed or that their language conclusions are invalid.
2. `joint-read-geometry.rs:221,389,468` prevents an initialized fine-tune from
   resuming: init+resume is rejected, but dropping init fails saved-setting
   equality. Preserve its initialization identity and fine-tune data stream in
   the resumed checkpoint path before a longer interruptible fit. No reviewed
   record showed the completed QAT runs using this broken resume combination.

The exact integer cancellation-avoiding distance identity is sound for valid
Q8 inputs. Dot arithmetic appears preserved in the inspected branches. Learned
Lorentz log-beta/offset use signed 16-bit codes and a derived wide scale alongside
low-bit affine weights; report that exception explicitly. Its policy adoption
belongs to the D10 clarification, not to a hidden all-parameters-four-bit claim.

## Next integrated action

Coordinate the retained native report/model/source handoff, the two fixes and
the [shared dialogue protocol](dialogue-protocol-contract-2026-09-26.md) with
the existing owners. Use their actual evidence and the emission-localization
track to choose a single consequential comparison or integration change.
Possible decisions are to retain the reader as a useful geometric component,
resolve an initialization/radius or numerical confound, or select the optional
finite-group operator for a demonstrated unmet need. No new training sweep or
new acceptance rule follows automatically from this source review. The whole
owner goal remains active, with local costs projected before each execution.

## Follow-up from the other lab

Claude acknowledged the findings and published `adb1659e` plus merged-main head
`f23bdea1`. Fourth-lab source inspection confirms the actual Q32 scale bound,
the allowed initialized-resume argument combination, and early conversational
rejection of Lorentz or width-128 models. Claude reports an interrupted QAT
resume matching its uninterrupted run and focused tests passing; those runs
were not independently replayed here. The native artifact/source
packet was subsequently delivered through PR #1406.
[Author response](https://github.com/UOR-Foundation/uor-r4/pull/1401#issuecomment-5851559567).

The fourth lab's [offline radial control](radial-read-control-2026-09-26.md)
reuses the continuous reader and compares its nonlinear spacing with an affine
score on the same lifted features. After PR #1401 merged, the increment was
adapted to extend its existing interface while preserving upstream behavior;
only the new affine control is refused by unsupported serving/export paths.
It introduces no new serving-policy exception or converted-model mechanism.

The [shared native packet](../evidence/native-lorentz-packet-2026-09-26/README.md)
now contains 104 manifest-listed files, including four packed QAT models and
tables. The fourth lab checked their Git-blob sizes and SHA256 values with zero
mismatches. This verifies packet integrity, not the reported model quality.
Continuous checkpoints, training/validation/length inputs, the learned tokenizer
merges and probe dumps remain external to that packet. A continuous replay or
fresh paired fit must resolve those specific inputs. The packet supplies no
LorentzAffine result and no equal-start Dot control at context 256.
The historical scratch flat-Dot score is
`beta * dot/sqrt(read_width) + (offset-initial_offset)`, without the radial
product; its README abbreviation is not the complete formula. Scratch flat-Dot
and Euclidean controls reused the Lorentz metadata label, so their actual
operator must be recovered from the retained patch and launch environment.
Do not infer their computation from that label alone. The new affine control
has a distinct identity to avoid this ambiguity.
