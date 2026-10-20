# #1382 — Bridge exact addressed memory into loaded lexical generation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T14:30:09Z
- **Updated:** 2026-10-09T13:17:06Z
- **Closed:** 2026-09-24T18:15:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1382
- **Merged:** not merged
- **Merge commit:** 939cc4df7648ceadb658169320cd48ece5b6b842
- **Base / Head:** codex/kvar-relative-energy-20260924 / UOR-Foundation:codex/addressed-lexical-bridge-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Scope

References #820. Stacked on #1381. This adds an explicit typed-address bridge from the retained scoped/versioned `Memory` into the loaded native `TlModel` Generate/Copy/Stop path. It does not transplant KVAR's fitted gate or train a raw-text source selector.

## Evidence

- Frozen two-case intervention: ` red` -> ` green` (alternative ` north`) and ` north` -> ` summer` (alternative ` red`). Actual outputs change through Copy: ` green now` vs ` north now`, and ` summer now` vs ` red now`.
- Authoritative sealed `final4`/independent `replay4`: 24 rows identical across separate-process model and binary-memory reload. Current/pinned/previous, read-disabled, unrelated edit, absent, evicted, no-history, wrong-scope, and native/compiled parity gates pass.
- Earlier `final1`/`replay1` expose a premature reload claim and ungrounded missing-read fallback; the correction record preserves those roots and the `final2`/`replay2`, `final3`/`replay3` progressions.
- Offline release build, executed runner/replay, `cargo fmt --check`, `git diff --check`, and claim-wording gate pass. An attempted debug lib test-harness build was interrupted for excessive temporary cache growth before tests ran; it is not counted as a pass.

## Limits and next dependency

All values are familiar single tokens, and the uncopied `now` does not change with the stored value. The request address, writes, and read permission are caller-supplied. The loaded decoder still inspects 153,052 nonzero parameter slots per step. There is no general-language, coding, geometric-advantage, D5 sparse-serving, or energy result. Next: learn observed-text source admission and read/write/address decisions on the same native path, then require a fixed-flags source intervention to change an uncopied word while retaining the negative and lexical/prose controls. Full ordinary `(f)` and geometric `(h)` KVAR arms remain NOT_RUN.

Evidence and provenance: `docs/integration/addressed-lexical-bridge-result-2026-09-24.md`. All eight attempt roots are sealed and retained under `/Users/casey.allard/uor-r4-investigations/addressed-lexical-20260924/`.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5819632705 — 2026-09-24T18:15:53Z — Casey-allard

Delivered as part of owner-authorized consolidated PR #1386, merged through the protected main queue at `b020f34a09816ba235984cedde1c50f4c8e9cae2`. The main tree equals reviewed stack head `d1f5524c1d2b302cf2c07fea6b47309460b85f66` exactly (tree `c68b8b332080d09884b27191731b56296dc4a1f4`). Closing this component PR as incorporated; its experimental limitations remain as recorded. Programme #820 and #973 remain open.



---
_Comment/review ids in this file: 5819632705_

