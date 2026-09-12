# Shared geometric core: byte-emission diagnostic

September 12, 2026. References #973 and #964. **DIAGNOSTIC_COMPLETE_NO_REFIT** on preserved experimental `5979f2e2` and initialized `89461806`. Retain `15baec48`; no model is changed or promoted. The [tracked evidence](evidence/native_geometric_shared_core_diagnostic_973.json) binds source, binary, artifact, reports and resources. [Current state](integration/current-state.md) owns the next action.

## Question and method

The first shared-core fit reduced construction loss but failed contextual transfer and produced unusable continuations. This diagnostic asks whether byte decisions fail because useful distinctions disappear at emission, because the existing branch classifier was poorly fitted, or because greedy traversal disagrees with the likelihood objective.

Use the original 672 construction and 249 opened held-out positions, under Full, for both saved artifacts. No new data, fit, candidate or generation campaign. The held-out population is now development evidence. A test-only observer invokes the actual unchanged runtime branch-score function, records each target-path decision and first divergence, and checks that prediction leaves session state unchanged. It reproduces the original position counts, byte/EOS accuracy and NLL (tolerance 1e-10), and verifies artifact bytes and original report seals before and after analysis.

The source first composes two ordered H4 roots and a learned mixing root into one of 120 roots. A learned landmark then supplies a signed angular score; the branch goes right only for a positive score. The diagnostic counts conflicting answers for equal four-root emission states and equal branch codes. It exhausts all 120 legal landmarks on each node's frozen observed inputs to obtain separate minimum error and minimum loss. These are descriptive representability bounds, not trained parameters or transferable results. They hold only with recurrent states and output mixes fixed, and cannot be summed into a byte-accuracy prediction.

## Measured bottleneck

| Candidate branch / population | Decisions | Current errors | Minimum errors over legal landmarks | Equal-code conflict floor |
| --- | ---: | ---: | ---: | ---: |
| First branch, construction | 672 | 206 | 189 | 11 |
| Next ASCII branch, construction | 660 | 292 | 292 | 168 |
| First branch, opened holdout | 249 | 95 | 78 | 4 |
| Next ASCII branch, opened holdout | 245 | 132 | 117 | 49 |

The first branch divides bytes 0–127 from bytes 128–255 and EOS. Construction has 660 targets on the first side and 12 on the second. Even the best permitted landmark makes 189 errors with these frozen states. An unconstrained constant-majority comparator would make 12 errors; it is an analytical comparison, not an implemented answer rule. The current root loss is 448.7246, against a minimum 447.7921 within its landmark class. A focused finite-domain test confirms that the signed-landmark class cannot express constant-left over all 120 input codes.

At the next ASCII branch, the current landmark already minimizes both observed classification error and logistic loss over every permitted landmark. It still makes 292/660 errors. Equal composed codes carry conflicting branch targets on enough observations to force at least 168 errors even under an arbitrary classifier of that code. More fitting of this landmark cannot recover the distinctions already lost at this interface.

In contrast, the four-root emission-state census has a construction lookup error floor of 15/672, falling to 6/660 after excluding document starts. The opened holdout has no conflicting four-root states after input (245 positions). This rules out pervasive exact equality of the complete emission-root tuple as the sole explanation for these populations. It does **not** show that the states have useful semantic organization, are learnable by a compact classifier, or preserve everything in the full memory/phase state.

The first two divergences account for 409/647 wrong construction bytes and 171/249 wrong held-out bytes. Complete leaf-probability mode disagrees with greedy traversal frequently, but only changes correct predictions from 25 to 33 of 672 and from 0 to 2 of 249. A traversal change alone does not produce useful prediction; exhaustive leaf scoring remains an offline diagnostic, not a proposed serving path.

## Decision

The restricted emission interface is a demonstrated bottleneck. Reject an unchanged landmark-only refit and do not add attention or memory mechanisms on the strength of this result. Recurrent semantic quality and useful context learning remain unresolved.

The next causal change is a shared calibrated nonlinear geometric byte-branch operator that retains the ordered state components until the decision and can express class priors and constant decisions. It must remain a bounded finite operation, with no feature-weight contraction, dense vocabulary projection, request-specific answer head or Cartesian table over complete model state. Establish its representability on the existing development diagnostic before one separately frozen joint-learning experiment. This is a proposed implementation, not a claim that calibration alone repairs the 168 conflicting-code errors or establishes language quality.

## Verification and preservation

Three focused checks pass: conflicting-target count arithmetic, the signed-landmark class limitation and sealed-input path preservation including symlink aliases. The actual saved-artifact diagnostic passes and reproduces all four original metric sets. A review caught an output-descendant hazard before any diagnostic execution; the guard was corrected and tested before loading the saved artifacts. Both release builds and their charges are preserved.

Runtime and training implementation files remain byte-identical. Only a test hook and test-only diagnostic source are added. Source/binary pins, all position-level branch traces, the full node summaries and complete-file manifests remain under the established handoff's `shared-core-first-step/diagnostic-1/`. No sealed report was extended or reused. Protected source/evidence delivery does not close #973 or #964 and does not promote a model.
