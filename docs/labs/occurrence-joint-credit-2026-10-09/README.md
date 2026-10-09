# Complete donor utility for coupled episode learning — 9 October 2026

**Decision: implement an explicit alternative donor-credit rule; NOT YET PROMOTED.** The previous learner already couples Prefix occurrence scoring to Generate. Its indirect credit uses a single-lane state tangent, which can omit the finite interaction when a physical donor changes both endpoints of a Generate pair. The new opt-in rule evaluates the complete alternative donor through the native pooled loss. It adds no parameters, source features or serving cases.

This work follows the fixed-state exclusion in #2063 under the [M2 claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6085926805). That bound rules out another pair-only search with the other terms frozen. It does **not** establish that the donor-credit approximation caused the coupled candidate's 9/15 negative. The accepted original parent still completes **8/512** development replies; the pair candidate's 6/15 and the separate coupled candidate's 9/15 remain unselected conditional results.

## Source finding and causal question

`context_cue_coadapt::selector` attaches a softmax-weighted combination of donor-state one-hot contrasts to the factual state. In `GenerateLearningWeights::forward_prepared_state_choices`, each lane's pair utility varies that lane while holding the other endpoint at its factual relative code. This is a declared local state surrogate, not complete finite donor utility.

For factual endpoints `(a,b)` and alternative endpoints `(a',b')`, summing those two single-lane contrasts omits

```
E(a',b') - E(a',b) - E(a,b') + E(a,b).
```

The term can be nonzero while both individual contrasts vanish. Native construction, by contrast, evaluates the complete donor's transported state and includes the interaction. This establishes a general mismatch between that initial credit approximation and the discrete change it ranks. Its magnitude and causal importance on the retained complete episode are NOT_RUN here.

The prospective model question is: **does complete source-bound donor utility change Prefix/Generate credit and permit complete-episode construction under the same 17-reference and 380-guard protection?** This changes the credit mechanism while retaining the original initializer, objective and native acceptance, rather than widening the failed pair-only search or combining negative candidates.

## Intended and implemented boundary

The coupled learner exposes `donor_credit: "full_pool_utility"`; omitted configuration retains `state_tangent` for historical reproduction. Corrected-mode use of retained gradients or exports is rejected. Conversely, historical recovery requires agreement between the requested mode, authenticated producer config and inherited gradient receipt, so it cannot relabel corrected-mode science as legacy. An absent historical receipt field remains `state_tangent`. Reports, gradient receipts, checkpoint receipts and positive artifact authority must bind the selected mode.

For each physical donor at each of the 31 original gradient frames:

1. Apply the existing source-bound bridge and native Generate scorer to that complete donor state.
2. Add the frozen shared-action U contribution to its Generate scores. Keep all factual physical Copy scores, including their U terms, unchanged.
3. Reduce the full native action pool with its common clip, admitted integer lookup, token aliases and tie rule. Compute the target's CE from the native target and total masses.
4. Subtract the factual-donor CE in f64 before conversion to f32, then detach those scalar donor contrasts and use their softmax-weighted expectation to supply indirect credit to **BASE Copy** selector logits. Add the expectation minus its detached value, so the factual forward loss is unchanged.

This replaces the indirect state-tangent path. The ordinary factual loss retains direct Copy credit and factual Generate coefficient credit. Each physical frame's objective weight is applied once. U is not removed and is not inserted into the donor selector: native donor selection precedes U. Duplicate physical occurrences remain separate alternatives, even if their token IDs or post-states agree.

The result remains a soft selection surrogate, not a derivative of hard argmax. Complete native alternative losses are evaluated in integer mass space and converted to floating loss values for offline credit; no bitwise exact floating gradient or language-to-Context recurrence claim follows. Same-post donors have equal indirect utility; this mechanism alone does not create a missing within-source offset feature.

## Validation, identity and cost

The implementation source is `2f64255db593f375c7a121561c6221e182432051`, based on fresh main `d53d5dff3dcea3e5554facd39f08c043700a0cbb`. The changed coupled child SHA-256 is `0350c3e1ca574741a283cfc3727ccf36522533c89a710ce3717c559cc39d6923`. Exact runtime/test identities and executed checks are retained in [validation.json](validation.json). All sixteen coupled tests pass on the repaired source, including four new mechanism/recovery tests; named Rust formatting and claim-wording checks pass. Both mathematical and adversarial source reviews pass, with their scope and the repaired provenance finding retained in [review.json](review.json).

The decisive test uses an authored native eight-token/eight-lane artifact with one ordered pair `(0,1)`. The factual state is all identity; the alternative changes lanes 0/1 to H4 codes 2/3. Only the joint target pair entry has coefficient 7. Its native target score changes from 0 to `7 << 20`, while the old target-score state tangent gives zero route credit. The complete pooled CE improves and the corrected donor utility gives the improving donor a negative descent gradient. This does **not** assert that the entire old pooled-loss gradient is zero. Other focused checks isolate zero-forward loss, identical factual Generate gradients, direct plus indirect Copy credit with one frame weight, duplicated physical aliases, clipping and rejected recovery mode mismatches. These are synthetic native mechanism fixtures, not development or held-out language data.

No actual-parent backward, constructor, candidate export, autoregressive answer or held-out evaluation is claimed in this implementation record. The tests do not execute the complete 31-frame production run; that run's artifact-bound factual parity and resource gates remain mandatory. The cumulative local projection is in [resource-ledger.json](resource-ledger.json), chained from #2063 without resetting earlier charges. No GPU pod or new spending is admitted for this piece. The cold dependency build took 360.05 s (3,033,808,896 B maximum RSS); the initial sixteen-test check before the recovery repair took 24.28 s (866,516,992 B maximum RSS). The repaired-source sixteen-test build/check took 123.15 s (1,778,941,952 B maximum RSS). The non-test optimized example build took 117.72 s (887,275,520 B maximum RSS). The source review found and repaired the reverse recovery provenance boundary before delivery; this was not a model failure.

## Preservation

The [verified iCloud receipt](preservation.json) retains the production/test binaries, changed source, checks and pre-delivery record as `codex-occurrence-joint-credit-20261009`: 19,009,536 archive bytes, MD5 `6b862a261543a357621db37748b3e8ff`. The fresh remote download checksum and index agree. The package inventory binds sixteen files (18,945,593 logical payload bytes); final delivery receipts are on main and the PR. No retained model artifact was modified or replaced.

## Next

After protected delivery and cleanup, recover the exact original parent and complete-episode authorities. Prospectively bind the new donor-credit mode, inspect its recorded original-frame native donor utilities and fresh credit against the retained legacy result, then perform the bounded native construction only within a separately recorded complete run projection. Retain all negatives and the 17/380 controls. A positive fifteen-position conditional gate must immediately pass the existing actual-artifact whole-answer/EOS check before broader qualification; neither a nonzero gradient nor a fixture result is a language gain.
