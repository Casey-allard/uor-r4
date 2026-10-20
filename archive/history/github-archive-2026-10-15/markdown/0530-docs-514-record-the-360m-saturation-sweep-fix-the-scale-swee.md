# #530 — docs(#514): record the 360M saturation sweep; fix the scale_sweep clobber bug

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T07:12:17Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-09T07:31:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/530
- **Merged:** 2026-08-09T07:31:04Z
- **Merge commit:** de7cfd941773dcfc8d7e8dcc3286f9d909b7c6bc
- **Base / Head:** main / UOR-Foundation:issue-514-calibration
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #514.

First end-to-end run of the #514 saturation-sweep harness over the pinned #516 broad corpus (360,924 records, SmolLM2-360M). Sub-sampling that one corpus and re-running compile → cover → score:

| records | held-out | top-1 (Rule 1+2) | EXCT-miss |
|---:|---:|---:|---:|
| 25,000 | 5,008 | 15.83% | 56.65% |
| 50,000 | 10,127 | 14.85% | 48.65% |
| 100,000 | 18,373 | 18.54% | 38.13% |
| 200,000 | 40,149 | 19.66% | 33.53% |
| 360,000 | 72,195 | 23.91% | 26.77% |

**Findings.** The curve densifies and confirms the existing coverage anchors (25k sits on the 21k→62.5% anchor; the monotone decay is continuous with 500k→14.6%). But there is **no knee at 360k** — top-1 is still climbing (+4.3pp over 200k→360k) and EXCT-miss still falling (−6.8pp), so the 360M corpus is coverage-limited, below the ~1–2M knee. It yields no `N_knee` to pin β; β stays provisional **0.5**, bounded `[0.45, 1.0]`. This validates the calculator's headroom rule (#516 recommends ~2–3M records).

**Harness fix.** The first run surfaced a latent bug: `compile-recorded --out $D` re-emits `corpus.meta`/`corpus.records` into `$D`, clobbering the sub-sampled inputs when they share those names (the #516 pipeline dodged it only via `.bin` names). `scale_sweep.sh` now writes the sub-sample as `sub.meta`/`sub.records`. The shipped harness is now verified end-to-end.

Law + calculator (`recommend-scale`, #517) + harness are complete and mutually consistent; pinning β from a real knee is remaining **compute** (a teacher observed past its ~2M knee), filed as a follow-up.

Docs + script only.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

