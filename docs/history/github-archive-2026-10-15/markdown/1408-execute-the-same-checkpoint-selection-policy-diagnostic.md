# #1408 — Execute the same-checkpoint selection-policy diagnostic

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T01:46:09Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-27T02:04:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1408
- **Merged:** 2026-09-27T02:04:30Z
- **Merge commit:** 591cb2b503b15c2d701ee6ec4048c60cbf24a206
- **Base / Head:** main / UOR-Foundation:codex/selection-policy-diagnostic-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Executes the supported next work card of the emission/selection diagnostic: the **same-checkpoint selection-policy diagnostic** (deterministic greedy replay), read-only, with a new additive `joint-selection-replay` instrument.

References #973 and #820. No candidate is promoted; no weights, serving decoding, gates or panels change; the frozen prose verdict stands.

## Instrument and identity

- `joint-selection-replay CAMPAIGN_JSON SEALED_CHECKPOINT RETAINED_EVALUATE_ROOT NEW_REPORT_ROOT cpu` — additive; `run_story_probes` and `generate` are unchanged. Executed source `18d5df51` (rebased over the concurrent Lorentz-attention merge `e3191a83`); binary sha256 `f13d463c…`.
- `PARITY_EXACT` against the retained `story-probes.json` in both arms: 0 mismatches over every compared field (exact equality, including f64 values). Counts: 21/32 and 23/32 complete, 26/32 and 31/32 first-noun, 8/16 and 10/16 pairs.
- Three runs across two `main` advances (`-1` at `4f71ea71`, `-2` at `5d40e751`, `-3` at `18d5df51`) produced identical results (timing fields excluded), and all three report `PARITY_EXACT`.
- All five greedy trajectories diverge from the retained sampled ones at generation indices `[0,0,2,1,0]`; the sampled witnessed spans ("called out in.", "so exciting!") do not recur.

## Result

- Greedy prose against the frozen four criteria (principal read; the independent evidence auditor returned identical verdicts): **quaternion 2/5, householder pair 0/5**. Six of ten outputs stop at the 128-token cap mid-clause; a lenient reading of the cap clause gives 4/5 each, but malformed constructs persist, so the positive branch's condition is not met.
- Greedy source panel failures: 11/32 (quaternion) and 9/32 (householder pair). Modes: correct-noun-then-extra phrase 13/20; wrong first noun 7/20. Versus the accepted September25 parent, complete rows lost/gained **9/2** and **6/5** (first-noun 5/2 and 1/2); versus rung1, 7/0 and 3/3.
- **Decision: negative/mixed → the supported next target is ranking/emission** on the concrete greedy regression rows and modes above, not a fresh selection-policy experiment. No promotion.

## Correction

The plan's cited `5/2`/`7/4` lost/gained are the new F32-hard artifact's complete-row figures, not this continuous panel's (`9/2` and `6/5`). Corrected in the result document and a plan corrigendum; the predeclared rule, criteria and cost are unchanged.

## Files

- `docs/integration/selection-policy-diagnostic-plan-2026-09-27.md` (predeclared rule + corrigendum)
- `docs/integration/selection-policy-diagnostic-result-2026-09-27.md`
- `docs/evidence/selection-policy-diagnostic-2026-09-27.json`
- `crates/uor-r4-training/src/joint_campaign.rs`, `crates/uor-r4-training/README.md`
- `docs/integration/current-state.md`, `README.md`

## Checks (executed locally)

- `cargo fmt -p uor-r4-training -- --check` clean; `cargo check --offline -p uor-r4-training --features metal,cpu-accelerate,reference-accelerate` 0 errors.
- Focused tests `selection_replay_*`: 3 passed.
- Both witness runs completed with `PARITY_EXACT`; post-rebase and post-Lorentz-merge re-runs identical.
- `python3 scripts/check_claim_wording.py` clean.
- Report roots `selection-replay-{quaternion,householder_pair}-{1,2,3}` (≈0.4 MiB each); no fit; no external compute.

## Notes

The repository's historical CI status names are queue compatibility acknowledgements and execute no formatting, Clippy or tests; the local checks above are the executed evidence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

