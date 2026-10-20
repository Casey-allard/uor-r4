# #1018 — A1Q-H4: freeze #1017 negative NLL-only continuation result

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T06:27:30Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-31T06:28:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1018
- **Merged:** 2026-08-31T06:28:56Z
- **Merge commit:** 967310318bc8cce337dfc1a212a71a8bfb15a2ca
- **Base / Head:** main / UOR-Foundation:issue-1017-quality-capacity-continuation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- implement and bind the one-way #1017 continuation, enabled-only Rust qualification, one-time reveal, five seeded Rust generations/replays, independent rubric, and create-once terminal finalization;
- record the frozen negative result across the research, lifecycle, roadmap, glossary, ADR, and programme documentation;
- retain ordinary coherent R4/Spin causal softmax as the accepted attention baseline while stopping further training or tuning of this exact 7.15M-parameter checkpoint.

## Frozen result

The authorized continuation completed 7,324 optimizer steps and reached exactly `149,995,520` cumulative tokens. The final checkpoint was selected at development NLL `1.580241072373312`.

Enabled-only Python/Rust parity passed: identical top-1 token `448`; maximum absolute logit delta `0.0000057220458984375` against `0.005`; all six learned layers selected; exact causal, projection, R4/Spin, and output-policy audits; zero future/provider/Ollama/prior-trace reads; and no attention-off execution.

The fresh confirmation was revealed once. Enabled NLL was `1.5727521962806827`, an improvement of `0.5546550809359942` nats/token (`26.071880399960784%`) from #1014, but it remained `0.0727521962806827` above the strict `1.50` ceiling.

All five fixed Rust continuations passed subject-or-scene retention; all were valid UTF-8 and free of period-1 through period-4 loops; all five normalized reload replays were exact. The complete Definition of Done therefore fails solely on `ENABLED_NLL`.

## Binding evidence

- selected checkpoint: `blake3:a712911974ea9229943d2d6651cee0c72461479e28a4c45fb93b55d06f031998`
- parity decision: `blake3:5bc79e85fb8427152bcc87a2dd63ff08a0f9a453e5d0ae63161a0b32792312d6`
- reveal result: `blake3:84eb315adab2dbcc4c8811d4ede96ab531b66f8720b7c299b16bbe9e04bfd0b5`
- human rubric: `blake3:22de86b5cf04191934df16cd3d5e24f24f3777c3d438bd2e9e9d4722f129c13e`
- final result: `blake3:0dbd94a279b6cd898e0d69667dcc1457c9dddba6061e6ff1d8c222c93793d46c`
- final manifest: `blake3:a53870f17837592cad7ce87d83abe7315c12181bccf0026e8035424fda783bfa`
- final tree: `blake3:19e2abf0ae4448c61ca093de341f6d4f137094491134be957d003f095cb9d448`

The human-readable record is `docs/r4_softmax_quality_capacity_continuation_1017.md`; the exact machine-readable aggregate is `docs/r4_softmax_quality_capacity_continuation_1017_raw.json`.

## Validation

- `python3 scripts/check_claim_wording.py`
- `cargo fmt --check`
- `cargo test --offline -p uor-r4-wasm-router --lib r4_softmax_local_generation::tests` — 5 passed
- `cargo test --offline -p uor-r4-wasm-router --bin r4 parses_r4_softmax_local_qualify_with_explicit_evidence_only` — 1 passed
- `cargo check --offline -p uor-r4-wasm-router --all-targets`
- `cargo clippy --offline -p uor-r4-wasm-router --all-targets --all-features -- -D warnings`
- `cargo check --offline --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`
- frozen trainer unit suite — 43 passed
- Python compile check, JSON parse, and `git diff --check`

## Decision and claim boundary

This closes the exact #1017 capacity campaign negative. It does not authorize another same-capacity exposure extension, learning-rate adjustment, alternate seed, sweep, or reveal-informed retry.

The result retains #1014's bounded conclusion that ordinary causal attention is load-bearing in coherent R4/Spin frames. It does not establish geometry advantage, transformerlessness, multiplication-free/table-native execution, dependable general-purpose generation, correctness, reasoning, chat, browser/WASM readiness, release readiness, or frontier capability.

The next research action is one predeclared parameter-capacity increase over the same qualified attention and Rust evidence path, using external training hardware only if that capacity cannot be trained efficiently on the current host.

Closes #1017
References #973
References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

