# #1608 — Add trainable geometric values and a source-bound integer producer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T07:48:32Z
- **Updated:** 2026-10-09T13:17:25Z
- **Closed:** 2026-10-02T08:00:23Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1608
- **Merged:** 2026-10-02T08:00:23Z
- **Merge commit:** 2b598f23f4d286b38cccc79e22c6a1afcf207bd4
- **Base / Head:** main / UOR-Foundation:codex/geometric-finite-credit-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native attention path still used donor value vectors, and tangent-only state credit could vanish for useful finite antipodal actions. Expose actual retained roots differentiably and add an opt-in full-adjoint finite-choice policy. Legacy construction/artifacts keep their original policy; explicit continuation preserves every learned parameter while binding a new training-policy identity.

Add a trainable two-atom geometric value head using token, actual retained own/neighbor state and explicitly valid held-span features. Compile its finite factors to selected Q24 score tables; the integer producer chooses signed-root/radius packets and supplies checked Q16 sums directly to the integer reader. Bind the saved head, exact compiled context, tokenizer and dependencies, with independent source recompilation on load. Neither labels nor donor value vectors enter this producing path.

Validation: 22 unique focused cases cover current/delayed answer credit, reset and synchronous neighbor behavior, actual value-to-state gradients with address credit stopped, donor-value bypass, packet/status/overflow arithmetic, source/native reload, stale/resealed input rejection, and fixture packet/Q16 parity. The revised retained-weight conversion and final repaired integration were rerun. Formatting, diff and claim-wording checks pass. Independent source/results review is recorded on the exact head. Release emitted-assembly inspection covers 695 instructions across four numerical methods with no flagged multiply/divide/floating arithmetic. Two generated MADD failures are retained; safe packet iteration and a private 256-byte descriptor stride repair them. This is not a final-linked/full-model instruction qualification.

Retained-trace analysis finds frozen-input donor imitation conflicts, including a raw-verified witness that requires the same answer. The active plan therefore prioritizes ordinary answer loss and treats packet imitation as auxiliary supervision. No new fit ran in this PR. Hard-choice parity is measured on fixtures, not guaranteed globally; selected zero-category answer gradients stop. NoRead, trunk and output remain float, and no language, complete-serving or energy claim is made.

References #1512, #973. Next: jointly fit transitions and values from the accepted parents under a separate fixed data/controls/cost card, preserving every prior row-level baseline.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5947803029 — 2026-10-02T07:59:31Z — Casey-allard

# Independent review — geometric finite-choice credit and native values

**Decision: APPROVED for protected PR #1608 at exact commit `99a3b653a1c690744bcb5ed104b3bcf418078ccd`.** No unresolved source, mathematical, evidence-binding, or claim-scope blocker remains in the reviewed change. This is approval of the implemented component and its stated evidence, not qualification of a fitted model or the complete serving path.

Reviewer: principal_mathematician, independent of the authors of the changes in this commit. Review performed by reading source, retained outputs, test logs, emitted assembly, and evidence; the reviewer did not run Cargo, inference, training, or modify implementation files. The worktree was clean, the full commit was resolved independently, and its ten changed files match the submitted scope.

The seven implementation-file SHA256 values in `docs/evidence/geometric-value-producer-2026-10-02.json` were independently recomputed and all match the committed files. In particular:

- Integer producer: `29047be3ea6648d8a73c22af97e1f597ecc948545d6bb616c431d305e8fd10cc`.
- Context and continuation policy: `67606dc09c6a33b87f8520873a26023c4c606597491fc5b4781ad2ecdbb0b0ee`.
- Integrated reader: `36902aec1f372a5ce0a3dd1f5faf5456874b48488162218c591905e812dc476d`.
- Offline value producer: `4eaebd600595b9d42b31c277d54cf8f6be2502e50800602529c22749781fcc2c`.
- Source-bound native compiler: `ca0d9c7e497321128a15a216237df4224b05f1c2c98dc83b8ad409a0aa6c11c4`.

The context correction preserves the hard geometric forward operation and gives the declared finite-choice surrogate the full upstream adjoint, including the antipodal component. Synchronous old-state own/neighbor dependencies and reset cuts are preserved. Exposed latent roots share the actual context operation; they are not recomputed through an unrelated observer. The consuming continuation conversion retains parameter values and Var identities and changes only the declared surrogate policy. This is a biased training derivative, not a derivative of hard argmax or an optimization guarantee.

The two-atom value producer retains signed roots, explicit occurrence validity, span presence, checked Q16 addition, and primitive packet identity through cancellation. The offline decoder-prototype surrogate restores antipodal choice credit. The integrated gradient fixture detaches address credit, so its earlier-token gradient is attributable to the value path. Zeroing the donor value map leaves the tested learned-value output unchanged. Native admission binds the actual value and context sources and dependent artifacts, independently recompiles on reload, and checks counterfactual latent/span/validity inputs. Q24 compilation can change close hard decisions; the implementation and report do not assert universal floating/native choice equality.

All six executed-check log hashes in the evidence JSON were independently recomputed and match. The logs contain 8 context, 5 offline-value, 4 final integer-value, 4 compiler, and 1 final integrated passing cases: **22 unique focused cases**. The revised continuation case was also rerun and passed; it is already included in the context count. Repeated executions are not additional independent cases. The final integrated log records the completed run after the final kernel repair. Formatting, diff, and claim-wording results are consistent with the final delivery record. These are root-executed checks read by the reviewer, not reviewer-executed tests or a model fit.

Both earlier assembly failures remain preserved. The final private descriptor alignment changes layout and metadata footprint without altering table artifacts or numerical semantics. The retained full compiler-emitted assembly independently hashes to `243264a871c0bf413a107f8e54a5b22cac93591fa822029b209b3f415f0a27b3`; the selected four-method body file hashes to `c8208544850c20e804956f18c5d82aca5ee3b9f5ce9cdaf1407074bc935d5274`. The scoped 695-instruction inspection contains no detected floating arithmetic, multiply/multiply-add, divide, or dot/matrix variant under the declared scan. This establishes only the named emitted bodies on `aarch64-apple-darwin`. External bounds-panic paths, final linked instructions, other runtime components, and energy are outside that evidence.

The retained-input audit has an important valid limitation for the next fit. I independently joined the seed-one stress pair207 witness: base row28/time22 and reversed row31/time12 share token21, all eight actual context states, the complete old-held span, and required answer21, but have different lane4 residual-radius targets. The raw source identities and row alignment agree. A deterministic head cannot reproduce both frozen donor targets from that identical input tuple, while the witness does not require different task answers. I verified this witness and the detailed report hash `bb0889c15f1e612bce45a4565e7084adb174798ff4227c49282ae2ea5db14b34`; I did not independently recompute its aggregate 3/68 conflicting full-tuple counts for the two seeds. Joint transition learning changes the inputs, so those frozen-input counts are not a lower bound on the proposed learned model. Perfect donor imitation must remain a diagnostic rather than an acceptance gate or a reason to retire the geometric family.

The final current-state, roadmap, and evidence wording correctly report **an executed, unfitted component**. NoRead, the remaining trunk, and output operations still include floating computation. No new fit, natural-language capability, full-serving qualification, general donor recompilation result, or energy advantage is claimed. Zero-category answer credit stops by design; declared auxiliary category credit supplies a possible escape. The next joint fit remains prospective and should compare actual answers and compiled behavior while preserving prior row-level baselines.

No further test or model campaign is requested for this delivery. Protected merge may proceed at the exact approved head; subsequent substantive changes require review of their actual delta.



---
_Comment/review ids in this file: 5947803029_

