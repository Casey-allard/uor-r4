# #239 — build: pin Rust toolchain to 1.97.1 so local gates match CI (#238)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:08:41Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-07-28T03:26:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/239
- **Merged:** 2026-07-28T03:26:14Z
- **Merge commit:** 54d0d0ed755b4c559224b7c06fa9fedf8d555c2d
- **Base / Head:** main / UOR-Foundation:issue-238-rust-toolchain-pin
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #238.

## What

Pins the workspace toolchain via `rust-toolchain.toml` (stable **1.97.1**, `rustfmt` + `clippy` components) and documents it in AGENTS.md's Commands section.

## Why

PR #236 failed CI `cargo fmt --check` after all four local gates passed: the dev machine had Homebrew rust 1.96.1 ahead of rustup in `PATH` while CI's `dtolnay/rust-toolchain@stable` resolved 1.97.1 — two different rustfmts, silent drift. With the pin, every rustup-managed `cargo` invocation resolves the same version by construction.

Nightly-only jobs (`cargo +nightly fuzz`) and Kani's own toolchain use explicit overrides and are unaffected. The AGENTS.md note covers the one hole the pin can't close (a non-rustup Rust earlier in `PATH` ignores it) and ties bumps to Gate E κ re-pinning.

## Gates (macOS arm64, rustup 1.97.1, offline)

- `cargo test --workspace --offline` ✅
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` ✅
- `cargo fmt --check` ✅
- `cargo check -p uor-r4-graph-format --no-default-features` (+ `--features alloc`) ✅

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

