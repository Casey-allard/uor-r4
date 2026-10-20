# #238 — Pin the Rust toolchain (rust-toolchain.toml) so local gates match CI byte-for-byte

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:04:20Z
- **Updated:** 2026-07-28T03:26:15Z
- **Closed:** 2026-07-28T03:26:15Z
- **Labels:** enhancement
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/238
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary

PR #236 failed CI on `cargo fmt --check` after all four local gates passed on macOS: the local stable rustfmt disagreed with CI's latest-stable rustfmt on two call sites. Updating the local toolchain and reformatting fixed it (d026794), but the root cause remains: AGENTS.md's four local gates and CI's `dtolnay/rust-toolchain@stable` can silently run different toolchain versions.

### Proposal

Add a `rust-toolchain.toml` at the workspace root pinning the exact stable version CI currently resolves, with `rustfmt` and `clippy` components. rustup honors it for every `cargo` invocation, so local gates and CI become byte-identical by construction. Nightly-only jobs (fuzz smoke via `cargo +nightly`) and Kani's own toolchain are unaffected — explicit `+toolchain` overrides bypass the pin.

Maintenance: bumping the pin is a deliberate one-line PR (and pairs naturally with re-pinning κ baselines when a toolchain bump shifts libm-sensitive teacher logprobs, per AGENTS.md's Gate E notes).

### DoD

- `rust-toolchain.toml` pinning current stable with rustfmt+clippy components
- AGENTS.md note in the Commands section that the pin exists and how to bump it
- All five required checks green with the pin in place


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

