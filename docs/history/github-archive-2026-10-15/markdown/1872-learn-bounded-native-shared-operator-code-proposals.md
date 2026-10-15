# #1872 — Learn bounded native shared-operator code proposals

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T03:07:49Z
- **Updated:** 2026-10-08T03:52:21Z
- **Closed:** 2026-10-08T03:52:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1872
- **Merged:** 2026-10-08T03:52:21Z
- **Merge commit:** 34b2dfe0463a6bff64576639e02dc649695eafeb
- **Base / Head:** main / UOR-Foundation:codex/astra-reference-replay-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The replay-supervised Adam step in #1867 increased its native objective and lost every retained complete reply. This adds an opt-in finite learner that proposes legal changes to shared Context/Potential/Generate/U codes, independently exports and scores each against the same frozen native objective, and accepts at most one strict improvement. Every candidate starts from the original parent; untouched floating masters remain bit-identical. No Adam update or adaptive search occurs in this mode.

The declared18-slot pilot evaluated17 candidates, with one saturated Generate no-op. It selected a single Potential content-presence code change (1→2): combined native CE fell9.8275465103→9.6070656275, all398 reference positions and all8 complete replies survived, but completion remained8/512 with no new successes. This is an optimization result, not a useful-model promotion. All12 Context proposals were scored; two descended, neither won, and their own-prefix utility is unmeasured.

Saved traces locate the limitation:92 reached canonical-prefix occurrences slightly worsen (+0.0014208194 weighted CE), while425 unreached occurrences supply the improvement (−0.2219017021). The same504 cases remain wrong at entry, although42 entry token IDs and226 complete output sequences change. Current state records a prospective canonical-frontier/successful-trajectory objective as the next causal step, NOT_RUN and requiring its own fixed work card.

Validation at executed source `929b214e01fa1bcc285230b01ce58e75467c6630`: changed formatting,26 fitter tests (zero failed/ignored), CUDA-cap89 release build; actual one-round pilot, early original-eight and full512 own-prefix evaluation; independent bank/packing/objective/selection/endpoint readers; separate actual Rust complete-file seal verification; mathematical and adversarial source/outcome reviews. An earlier pair of fixture dtype failures is retained and repaired with explicit F32 test literals. Final head `dd911cae7e80a5e8b4f1e0e1e5e00932abe0b660` changes documentation only and passes changed formatting, all26 fitter tests (zero failed/ignored;121.02s including compilation), release CUDA build (20.64s), claim-wording and diff checks. Independent adversarial review of the final documentation finds no concrete claim error. CI's five historical status names are transport acknowledgements, not test execution.

Complete audited evidence is preserved in private HF `caseyallard/uor-r4-store`, revision `55dc5eaedf8b2ef90d94f383bd3c34447cc86c69`, prefix `codex/astra-native-proposal/proposal-0001-929b214e01`. Archive SHA256 `cef0f95db784976c5f82d7fd80578a891e143c458854e66c4ebf303e698a8361` (424,665,546bytes); archive and manifest were freshly downloaded and verified. Runtime/source/checks are at revision `9601e19012819acece5eea98f6b94956c9df661f`, prefix `codex/astra-native-proposal/runtime-929b214e01-attempt2`. The owned4090 was released and deleted after verified preservation.

References #820. Prospective work card: https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051223155


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6051801473 — 2026-10-08T03:51:27Z — Casey-allard

Exact-head validation and review: `dd911cae7e80a5e8b4f1e0e1e5e00932abe0b660`. Changed-file format PASS; actual scoped release CUDA fitter tests26 PASS/0FAIL/0IGNORED,121.02s; release build PASS20.64s, GPU masked for both. Claim-wording and diff checks PASS. Both production source blobs equal executed source929b214e01; final delta is current-state documentation. Independent mathematical/adversarial implementation and result reviews plus independent final-documentation review report no unresolved blocker. Full evidence and fresh verification receipts are in the PR body. Five GitHub compatibility statuses are acknowledgements only. Native objective descent with8→8 complete replies is optimization evidence, not useful-model promotion.



---
_Comment/review ids in this file: 6051801473_

