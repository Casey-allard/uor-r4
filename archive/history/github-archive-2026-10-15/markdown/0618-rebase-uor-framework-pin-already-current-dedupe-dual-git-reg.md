# #618 — rebase(uor-framework): pin already current — dedupe dual git+registry uor-foundation builds, move to crates.io 0.5.2

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T21:42:27Z
- **Updated:** 2026-08-11T23:36:26Z
- **Closed:** 2026-08-11T23:11:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/618
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase tracking). First sub-issue — smallest delta.

## Audit result (measured 2026-08-11)

r4 pins `uor-foundation`, `uor-foundation-sdk` (deps) and `uor-foundation-verify` (dev-dep) at **0.5.2** via git rev `51c0138`. That rev **is UOR-Framework `main` HEAD today** (2026-05-23, latest commit upstream). The pin sits 3 commits past the v0.5.2 tag; `git diff v0.5.2..51c0138 -- foundation uor-foundation-sdk uor-foundation-verify` touches **READMEs only** — zero Rust code delta vs the released 0.5.2. The v6.x/v7.0.0 tags upstream are the pre-reset legacy line (2026-03) and are *older* than 0.5.x — not an upgrade target.

**So there is no version rebase to do. The real finding is build-graph hygiene:**

`Cargo.lock` carries **two copies each** of `uor-foundation 0.5.2` and `uor-foundation-sdk 0.5.2` — our git-pinned copies AND the crates.io registry copies pulled transitively by `uor-addr` → `uor-prism`. Duplicate compile units, plus a standing type-identity hazard: a type from uor-addr's `uor_foundation` is a *different type* from ours. Today no types cross that boundary; nothing guards it.

## Work

1. **Switch the three framework deps from git-rev pins to crates.io `"0.5.2"`** (workspace root + `crates/uor-r4-router` + graph-format if touched). Expected: lockfile drops the git copies; single build of each; type identity unified with the uor-addr/uor-prism chain.
2. **License check:** registry 0.5.2 was published just *before* the Apache-2.0→MIT relicense commit; confirm `cargo deny check licenses` stays green (deny.toml allowlist is exact-by-design).
3. **No-regression gate:** API surface we consume — `uor_foundation::pipeline::{PrismModel, carrier_inline_bytes, FoundationClosed, __sdk_seal}`, `enforcement::{GroundedShape, Grounded, Term, Hasher, ShapeViolation}`, `{DefaultHostTypes, HostBounds, ViolationKind}`, `uor_foundation_sdk::axis!` — must compile unchanged. Full `cargo test --workspace`, BDD (`bdd` harness), conformance gates, `cargo deny`.
4. **Optional adoption survey (file follow-ups, don't do here):** `uor-foundation-verify` in CI beyond dev-dep, `cargo-uor`, `uor-ontology` spec crate, `uor-foundation-test-helpers`.

## Safeguards

- Router runtime stays **zero-matmul**; this change is dependency-source-only, no runtime code paths touched.
- blake3 stays pinned `=1.5.0`; toolchain stays 1.97.1; no `Cargo.lock` drift beyond the dedupe.

## Verification steps (for the PR)

1. `grep -c 'name = "uor-foundation"' Cargo.lock` → expect 1 (was 2); same for `-sdk`.
2. `cargo build --workspace && cargo test --workspace` green.
3. `cargo deny check` green.
4. `cargo xtask` conformance gates green.

## Definition of done

Git pins replaced by registry 0.5.2, lockfile deduped, all gates green, docs (README/AGENTS.md dependency notes) updated if they name the git pin.


## Comments (2)

### comment 5259688502 — 2026-08-11T22:40:24Z — Casey-allard

**Implementation note — direction flip, same goal.** The issue as filed said "move the three framework deps to crates.io 0.5.2." That plan fails on a fact the audit missed: `uor-foundation-verify` is **not published on crates.io** (git workspace only), it is a live dev-dep (`src/tless_uor.rs::grounded_mints_and_replays`), and `verify_trace` consumes types minted by our `uor_foundation` — registry-foundation + git-verify would split type identity and break the test build.

So the unification goes the other direction: **keep the git pins, add `[patch.crates-io]`** mapping `uor-foundation` and `uor-foundation-sdk` onto the same pinned rev `51c0138`. Effect is identical to the issue's intent — the registry copies pulled in via uor-addr → uor-prism collapse onto our pin: **Cargo.lock drops from 2 copies to 1 copy of each**, one build per crate, one type identity across uor-addr, uor-prism, and r4. Safe because the audit showed the rev's foundation crates are code-identical to published 0.5.2 (README-only delta).

Measured after the change: `grep -c 'name = "uor-foundation"' Cargo.lock` = 1 (was 2), same for `-sdk`; lock diff −37/+17 lines; fmt / no_std ladder / clippy `-D warnings` all exit 0; full workspace suite running now, PR to follow.


### comment 5260103602 — 2026-08-11T23:36:25Z — Casey-allard

## Delivered — closing with record

**PR #619 merged 2026-08-11 23:11Z (merge queue), main = `eab191e`.**

What shipped: `[patch.crates-io]` mapping `uor-foundation` and `uor-foundation-sdk` onto the pinned Framework rev `51c0138`. Measured outcome: `Cargo.lock` **2 → 1 copies of each** (git+registry collapsed onto the git pin); one build per crate; one type identity across uor-addr, uor-prism, and r4. Two files touched (Cargo.toml, Cargo.lock).

Direction flip from the issue's original plan (registry switch) is recorded in the comment above and in the PR body — root cause: `uor-foundation-verify` is git-only and `verify_trace` consumes our `uor_foundation` types, so the git source is load-bearing; the patch achieves the same dedupe from the other side.

Verification record: fmt / no_std ladder / clippy `-D warnings` / full workspace suite (120 ok blocks, includes the verify replay test and allocation census) all exit 0 locally; full CI ladder green on the PR and on the queue branch (Kani, cargo audit, register conformance R1–R6, Gate C trend, fuzz smoke; D2 differential skip is the normal PR behavior). No runtime code touched; deployed-kernel P-4 invariant and zero-matmul principle unaffected.

Maintenance rule now encoded at the patch site in Cargo.toml: the `[patch.crates-io]` revs move together with the `[dependencies]` git pins, never separately; if Framework ever publishes `uor-foundation-verify` to crates.io, replace the whole section with plain registry deps (re-evaluate at the next framework bump).

Follow-on surface for this dependency lives in the remaining #589 sub-issues (#620–#625); optional-adoption survey items from this issue (verify-in-CI, cargo-uor, uor-ontology, test-helpers) fold into #589's backlog rather than holding this issue open.



---
_Comment/review ids in this file: 5259688502, 5260103602_

