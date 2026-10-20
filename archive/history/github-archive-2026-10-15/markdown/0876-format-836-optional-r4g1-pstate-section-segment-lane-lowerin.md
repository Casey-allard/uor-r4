# #876 — format(#836): optional R4G1 PSTATE section (segment-lane lowering foundation)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T20:09:35Z
- **Updated:** 2026-08-20T20:30:57Z
- **Closed:** 2026-08-20T20:30:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/876
- **Merged:** 2026-08-20T20:30:57Z
- **Merge commit:** 6c47783f83e2c1f68a6951a465b79dc04e68c930
- **Base / Head:** main / UOR-Foundation:issue-836-pstate-section
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

First increment of #836. #836 was activated by the #834/#875 positive: the deployed model is suffix-local (#874) yet whole-prompt content **is** predictive — the #835 segment-lane reference arm beats the suffix baseline +17.5‰ (CI [15.9, 19.0]) and follows 10/4,722 minimal pairs where the suffix baseline follows 0 (#875, `docs/prompt_arms_bakeoff_834.md` §6.2). So there is real signal to lower.

This increment lands the **format foundation**: the optional versioned R4G1 `PSTATE` section that carries the compiler-learned segment lane — a bounded, canonical residual table mapping a quantized whole-prompt content key to signed `ScoreQ` candidate-support contributions — plus its two-stage validation and a borrowed accessor.

## Scope

- `crates/uor-r4-graph-format/src/pstate.rs` — borrowed **no_std** parser (magic/version header + sorted, contiguous, exact-coverage rows→entries), binary-search `find`, an alloc-gated canonical `build_segment_lane`, and inline tests.
- `types.rs` — `SectionId::PSTATE` (`OPTIONAL_BIT | 0x10`) + `is_known`/`mandatory`.
- `error.rs` — typed `Pstate*` `FormatError` variants + `Display`.
- `view.rs` — `GraphView::pstate_table()` accessor (`None` when absent).
- `lib.rs` — module + exports.

## Non-goals (this increment)

- The deployed scorer does **not** yet consume PSTATE (a later #836 increment wires the hot-path transition + contributions + witnesses). This is additive format only.
- No `model/ids.toml` row yet: this is format infrastructure binding RF-21/22; the new **serving** capability and its row land when the scorer consumes PSTATE.

## Compatibility and migration

Purely additive and **non-serving**. The section is optional (`OPTIONAL_BIT`), so a pre-PSTATE artifact and any reader that does not consume it behave exactly as before — **absent-section identity**. No format break; every historical bundle stays valid.

## Tests and verification (all green)

- `cargo test -p uor-r4-graph-format` — including the 4 PSTATE tests: round-trip/determinism (order-independent canonicalization → byte-identical), corrupt magic/version/reserved/offset + truncation rejection, non-canonical builder rejection (empty entries, duplicate key, duplicate token), empty-section validity.
- `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`) — the parser is no_std/no-alloc; the builder is alloc-gated. `forbid(unsafe_code)` preserved.
- `cargo clippy -p uor-r4-graph-format --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo check --workspace`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib` — clean.
- `cargo run -q -p xtask -- validate` — every gate passed; CONFORMANCE unchanged (no capability added), R5 clean (uses the sanctioned `FormatError`/`NotAProduct`).

## Conformance / claim boundary

No capability added; `CONFORMANCE.md` unchanged. Binds existing RF-21/22 (format quality/pathology). No serving/claim change.

## Closure

**References #836** — increment 1 of the #836 lowering (format section). Next increments: compiler packing (write PSTATE from the fitted segment lane), the hot-path scorer + witnesses (P-4/no_std/allocation), and the causal-gate re-run on the packed path with the PROMOTE/REVISE/RETIRE verdict. Untracked `obs-text/`, `tf1-pkg/` untouched.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

