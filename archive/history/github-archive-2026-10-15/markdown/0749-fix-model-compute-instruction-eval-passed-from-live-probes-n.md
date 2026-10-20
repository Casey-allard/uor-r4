# #749 — fix(model): compute instruction_eval_passed from live probes, not CLI flags

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T01:09:30Z
- **Updated:** 2026-08-17T08:42:08Z
- **Closed:** 2026-08-17T01:31:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/749
- **Merged:** 2026-08-17T01:31:50Z
- **Merge commit:** 20001570f7896ddbbd0e658eeaa11fcd8f59288f
- **Base / Head:** main / UOR-Foundation:issue-744-live-quality-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`instruction_eval_passed`, `grounded_answer_rate`, and `repetition_rate` were never computed by any code path. `r4 import`'s CLI accepted them as manually-typed `--instruction-eval-passed`/`--grounded-answer-rate`/`--repetition-rate` flags, copied verbatim into the manifest. That's why two manifests over byte-identical compiled artifacts could carry opposite recorded quality and both still pass the gate meant to keep unevaluated or known-bad bundles from being served (#744).

## Fix

For `--capability instruction-chat`, `import` now derives the quality attestation itself:

- Loads the exact `--artifacts`/`--store`/`--tokenizer` bytes being imported (new `chat::engine_from_bytes`).
- Runs them through a fixed probe set of ordinary questions on the real `ask` code path.
- Scores each answer's repetition with a new `recent_window_repetition_rate`, reusing Gate C's greedy-repetition-rate precedent (`uor-r4-graph-certify`'s `generate_greedy_repetition_rate`), but over a window equal to the *full generated length* rather than a fixed 32 tokens.
- `--evaluation-report` is still accepted and stored content-addressed as provenance of the offline D3 evaluation from `evaluate-report`, but is no longer consulted for the pass/fail decision — only the live probe run decides that now.

## A false positive caught during verification

The first working version (32-token window, 0.5 repetition bar) reported the exact #745 bundle (`smollm2-1-7b-instruct`, byte-identical to the `smollm2-1-7b-chat` bundle from the original #655 sweep) as **passing**. I didn't take that at face value — I cross-checked against real `r4 ask` output on the identical bytes via both live code paths:

- Directory-fallback + R4G1 beam-search path: `"cut cut cut cut cut ..."` (single-token degenerate repetition).
- Imported-manifest plain-path: the original #745 word-salad transcript.

Both confirmed the bundle is genuinely broken. Root cause: a 32-token window measured only 0.10–0.25 repetition on this transcript (too locally-scoped — a small pool of recurring fragments diluted by many one-off filler subword tokens). Widening to the full generation raised it to 0.31–0.41 — still under the original 0.5 bar. Fixed by widening the window and lowering `LIVE_QUALITY_REPETITION_BAR` to 0.25, with a code comment stating plainly that this bar is calibrated only against known-*bad* evidence — there's no known-good local bundle yet to confirm it doesn't also reject good output.

Verified end-to-end after the fix: both `smollm2-1-7b-instruct` and `smollm2-135m-instruct` are now correctly rejected by the real `import` CLI (`error=model has not passed its instruction/grounding evaluation gate`).

## Known limitation (not fixed here, filed as a follow-up)

The live probe run only exercises the plain TLA/TLS1 generation path — it never loads or probes a bundle's R4G1 graph, even though `ask`/`chat` prefer R4G1 over the plain path at serving time when a `compiled.r4g1` file exists at the manifest-name-keyed convention path. The two paths can disagree sharply, as shown above (word-salad vs. single-token "cut cut cut" repetition, on the exact same underlying bytes). A manifest that passes today's gate is only guaranteed non-degenerate on the plain path. Documented in `src/chat.rs`, `docs/MODEL_LIFECYCLE.md`, and filed as a separate follow-up issue rather than expanding this PR's scope — see linked issue.

## Verification

- `cargo fmt --check` — clean
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — clean
- `cargo run -p xtask -- audit-limits` — clean (R5: no unsanctioned error types)
- `cargo test --workspace --offline` — full suite green, including two new tests (`recent_window_repetition_rate_catches_small_vocabulary_cycling_that_repeated_suffix_misses`, `live_quality_gate_rejects_known_bad_probe_sets_and_accepts_a_healthy_one`)
- Manual end-to-end re-verification against both known-bad local bundles via the real `r4 import` CLI, post-fix

References #744 — leaving open rather than closing, since the R4G1-path gap means acceptance criterion 3 ("gate correlates with live `ask` output quality") is only fully satisfied for the plain-path bundles tested so far, not for bundles that would be served via R4G1 at runtime. See PR description above and the follow-up issue for the remaining scope.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

