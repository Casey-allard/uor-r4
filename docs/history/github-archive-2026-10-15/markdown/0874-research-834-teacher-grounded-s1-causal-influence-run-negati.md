# #874 — research(#834): teacher-grounded S1 causal-influence run (negative verdict)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T18:55:41Z
- **Updated:** 2026-10-09T13:08:55Z
- **Closed:** 2026-08-20T19:14:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/874
- **Merged:** 2026-08-20T19:14:20Z
- **Merge commit:** 2fc9425aa316ae80e30720021395696ac5de71e5
- **Base / Head:** main / UOR-Foundation:issue-834-causal-run
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Item B (#834) asks whether prompt meaning has causal relevance beyond suffix / exact-context memory. Building on the merged bake-off instrument (#873), this PR runs the **real, teacher-grounded measurement** on the #833 canonical bundle and records a decisive verdict.

Key realization that made it fast and real: when #833 was compiled, the SmolLM2 teacher was already run and its per-position argmax is recorded in the bundle's corpus (`Corpus::t_argmax`). So the harness reuses those teacher targets + the deployed **EXCT-disabled** engine over the real Simple-Wiki held-out set — **no live teacher forward**.

## Result (n = 24,044 held-out real-wiki positions; windows mean 7.8 tokens, 98% longer than the 2-token suffix)

- full-context top-1 vs teacher argmax **26.9%** (CI [26.4, 27.5]); suffix-only **27.1%** → **causal-influence-delta −1.6‰ (paired 95% CI [−2.4, −0.8])** — more context does *not* help; the interval excludes a positive effect.
- **context-saturation sweep** (top-1 vs teacher): k1 25.9%, k2 27.1%, k3 27.0%, k4 27.0%, k6 27.0%, full(8) 26.9% — **flat**; context beyond ~2 tokens adds no predictive signal.
- nulls non-degenerate: prompt-swap 2.2%, trivial-prior 5.0% (both far below full-context); context-helped only **27/24,044**.
- **1,460 natural minimal pairs** (same 2-token suffix, different story, different teacher answer): the model **follows 0**; the suffix-only control is degenerate-by-construction (1,460/1,460 identical), so a zero reading is "no effect", not a broken harness.

## Verdict

**`NO PROMPT-CONDITIONING ARM ESTABLISHED`** for the arms fittable against the current artifact (current-scoring, longer-local-context): the deployed model is **suffix-local** — corroborating #784, now teacher-grounded and quantified. Per #834's run-contract *if-negative* branch, current serving semantics are kept and S1 returns to representation/compiler redesign.

## Scope (do not over-read)

Three arms remain **UNAVAILABLE**: `persistent-state`, `conditional-residuals`, `candidate-support-expansion` are the #835 Ψ-family mechanisms that are **not lowered** into the artifact (the engine's public surface returns the selected token, not a candidate score vector). They can be fitted only after **#836** builds them. The negative establishes that the *current* mechanism has no prompt-conditioning beyond a short suffix — making #836 the prerequisite for any positive S1 conditioning claim — and does **not** assert the unbuilt mechanisms would also fail.

## Tests and verification

- `cargo test -p uor-r4-api --release --test causal_prompt_run_834 -- --ignored` — the run (ignored heavy harness; vacuously skips if the bundle is absent, κ-convention).
- Full api suite green; the harness shows as `1 ignored` in normal `cargo test`.
- `cargo clippy -p uor-r4-api --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `python3 scripts/check_claim_wording.py`, `cargo run -q -p xtask -- validate` (every gate passed) — all clean.

## Evidence artifacts

`docs/causal_run_834_result.json` (CID-bound: artifact_cid `blake3:bc2366f1…`, corpus_meta_cid `blake3:aa9d1767…`, result_cid `blake3:aad1511d…`); `docs/prompt_arms_bakeoff_834.md` §6.1.

## Compatibility / conformance / claim boundary

Reference-only / off-serving. No format/ABI/serving change; no `model/ids.toml` row and no `CONFORMANCE.md` regeneration (spec-leaf precedent; RF-01/21/22/27/28). Adds two dev-dependencies (`uor-r4-core`, `uor-r4-graph-compiler`) for the harness. The negative narrows the S1 conditioning claim for the current mechanism; it does not claim the unbuilt arms fail.

## Negative result

The verdict is negative — #834's explicitly legitimate scientific output ("a negative result is the required scientific output, not an implementation backlog").

## Closure

**References #834.** The measured verdict is a `NO PROMPT-CONDITIONING ARM ESTABLISHED` for the fittable arms; whether that closes #834 (accept the negative → route to #836/redesign) or #834 stays open pending the Ψ-family arms after #836 is a maintainer call — left to @Casey-allard. Untracked working dirs (`obs-text/`, `tf1-pkg/`) untouched and unstaged.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

