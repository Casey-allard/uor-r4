# #1779 — Connect composed prediction credit to retained geometric states

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T03:26:17Z
- **Updated:** 2026-10-09T13:17:19Z
- **Closed:** 2026-10-07T00:07:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1779
- **Merged:** not merged
- **Merge commit:** a6867844aadb4301d841604ec866d93d4e852a43
- **Base / Head:** codex/geometric-source-bound-actions-20261006 / UOR-Foundation:codex/geometric-composed-state-credit-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Prediction credit in the composed cue/prefix/source-end path previously used frozen geometric encodes or a four-coordinate projection that can erase nonconstant discrete-state utilities. This change exposes all120 conditional post-state scores to offline learning and connects composed next-token loss to the existing geometric state parameters. Native integer hard scores, serving formats and normalizer remain unchanged.

Executed evidence at numerical head ee438488a49d3058f5788c85774129ccda1d30a2:
- 85 scoped release tests pass; both native example binaries build. Full-choice nullspace regression passes.
- Zero-update real-data probe reaches all9 state families with finite nonzero gradients at12/12 fixed sampled canonical positions. Native traces match independent integer runtime.
- Independent export/reload yields identical actual own-prefix generation on4 rows. Full512 canonical/generation files are byte-identical to retained legacy control (74 complete, native CE0.9544226152).
- Linux x86_64 checks304.332s CPU8 peak4.144GB; probe27.427s CPU16 peak2.448GB. No CUDA model execution. Durable reports and exact binary are on canonical /workspace/uor-r4/codex/composed-state-credit; finished pod released/deleted.

Copy/Period/Stop remain the output scope; general Generate is absent. Earlier recurrence uses the existing approximate adjoint, conditional counterfactuals hold other states factual, and source-end selection remains stopped. No fit, fresh predictions, geometric advantage or language improvement is claimed. Independent exact-source/admission/outcome reviews found no concrete blocker. Final documentation commit does not change tested numerical source. Claim-wording and diff checks pass.

Owner compute correction: next fit waits for a genuine device-resident native CUDA forward/backward port; D19 CUDA covers a different model. Plan also retains jointly learned full-vocabulary geometric generation as the next predictive capability dependency, with full candidate admission and actual emitted-token feedback.

References #820. Stacked on #1778; not delivered to main yet.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6008781061 — 2026-10-06T03:33:15Z — Casey-allard

Exact numerical head ee438488a49d3058f5788c85774129ccda1d30a2: scoped release checks PASS on pod6waqxs8dbpj0dt (Linux x86_64, CPU8, no CUDA model claim). 28 context/credit +5 transport +2 composed/rebind +50 example tests =85 passed. Both native example binaries built. Complete checks304.332s, peak process-group RSS4,144,193,536B. Full-choice nullspace regression executed and passed. Real-data zero-update probe is now running under its600s/12GiB cap; learning improvement and general Generate remain unmeasured.

### comment 6027854859 — 2026-10-07T00:07:37Z — Casey-allard

Superseded by #1810, #1812 and #1813, all now merged through the protected queue. The selected active native runtime/CUDA/learning dependency closure is on main. Historical experiment drivers and unused API variants remain on this pushed branch as retained evidence; no branch is deleted and no mechanism-family verdict follows. Exact-head checks and preservation receipts: #1813 and #820 (triage issuecomment-6026806242).


---
_Comment/review ids in this file: 6008781061, 6027854859_

