# #1334 — Add confidence continuations; correct contextual-emission experiment

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T16:22:48Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-09-21T16:40:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1334
- **Merged:** 2026-09-21T16:40:40Z
- **Merge commit:** cfc5f7b6f1dfba9e44767f83ff123fbac41f4d8a
- **Base / Head:** main / UOR-Foundation:codex/read-conditioned-state-step
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained confidence reader now executes short autoregressive continuations through the shared causal predictor. The new read-conditioned path is also implemented, but principal review finds that its authored target is determined by the query suffix, the diagnostic used the preceding prediction position, and both fitted parameter sets equal their initializations. This PR preserves the useful rollout and negative evidence, repairs the interfaces, and corrects the stronger readout-only diagnosis.

Verified confidence completion: seven prompts, six continuation tokens per arm; H4 23/42 reads, categorical 22/42; all 45 exact source/payload references verify; ReadDisabled equals local on all seven prompts. Outputs remain degenerate. Old confidence panels/artifacts are exposed replays and their own-parent CE/absence failures remain unchanged.

The reported zero frozen-row ceiling is scoped to 60 incorrectly indexed historical positions; it does not isolate selection/update/readout on a context-required task. Strict accuracy-only search at that zero ceiling cannot accept changes. The claimed categorical comparator is the same H4 update with a different seed. The last of four generated update tokens was reported as the first. The three source hashes verify, but the new module was omitted from the original binding. Original sealed reports remain intact.

Principal repairs preserve the complete observed query, retain empty candidate positions, share causal read dispatch, score the actual first continuation at equal horizons, truthfully label the alternate H4 seed, use fail-closed returned reload objects, include the new module in future source inventory, and bound/canonicalize artifact parsing. These interface checks are not a model rerun.

The new principal review, independent audit, synchronized active documents and comprehensive next prompt select one genuinely older-context-dependent transformation plus a learned shared low-bit emitter, with NLL/margin learning, matched alternate update algebra and actual changed-source/disabled controls. Existing signed H4/Hopf and conditional S7/E8/harmonic directions remain available; dependent scheduling follows a useful one-read primitive.

Validation: focused runner/library boundary tests, package formatting, offline touched-runner compilation, claim-wording, JSON/local links and diff checks. Local executed checks carry validation; queue status names are compatibility acknowledgements. No new model fit/inference or paid compute from the principal repair. Shared resource JSON and safe cache-only cleanup receipts are reconciled.

References #973, #820, #1139, #962, #963, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

