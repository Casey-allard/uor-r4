# #2490 — Re-pin driver_source_sha256 in prime_zeta.rs test to 70b45863… after #2484 (References #2028, #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-15T00:20:34Z
- **Updated:** 2026-10-15T01:52:52Z
- **Closed:** 2026-10-15T01:31:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2490
- **Merged:** 2026-10-15T01:31:15Z
- **Merge commit:** 21dcc8a25ab2b7f1c750fb89220c26d38d9b9a81
- **Base / Head:** main / UOR-Foundation:antigravity/ag1-repin-prime-zeta-driver-hash
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Re-pins `uor_r4_discrete::training::driver_source_sha256()` in `prime_zeta::tests::frozen_d5_driver_source_hash_is_preserved_untouched` (`crates/uor-r4-discrete/src/prime_zeta.rs:822-828`) from `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4` to `70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676` per [#2028 comment 6181283888](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6181283888) and Claude's review note ([6181236311](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6181236311)).

- **Rationale:** PR [#2484](https://github.com/UOR-Foundation/uor-r4/pull/2484) added `completion_baseline.rs` to `driver_source_sha256()` (operational metadata guard only; all numerical model source files `build.rs`, `lib.rs`, `metadata.rs`, `device.rs`, `discrete.cu` remain untouched).
- **Verification:** `cargo test -p uor-r4-discrete --bin prime-zeta-qualify` passes `5/5` tests (`0 failed`); `uor-harness ratchet d23` (`1641 = baseline`), `uor-harness ratchet d25` (`93 = baseline`), and `uor-harness check-formats` PASS.

## Session Goal
- **Line:** evaluation / test hygiene
- **Count:** `0/3` (test-pin update only, zero model compute)
- **Headline:** `37/232 -> 37/232` (unchanged)

References #2028, #2029

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6181435343 — 2026-10-15T00:20:51Z — Casey-allard

**Exact-Head Review (`30abacdcbe98aa95f89bc3f1af992d2cc13611d9`) — APPROVE**

- **Head SHA:** `30abacdcbe98aa95f89bc3f1af992d2cc13611d9`
- **Diff Verified:** Re-pins `uor_r4_discrete::training::driver_source_sha256()` in `prime_zeta::tests::frozen_d5_driver_source_hash_is_preserved_untouched` (`crates/uor-r4-discrete/src/prime_zeta.rs:822-828`) from `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4` to `70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676` with the explicit rationale comment that #2484 added `completion_baseline.rs` to the operational driver source digest while leaving all numerical model source files untouched.
- **Verification:** `cargo test -p uor-r4-discrete --bin prime-zeta-qualify` (`5 passed; 0 failed`), `uor-harness ratchet d23` (`1641 = baseline`), `uor-harness ratchet d25` (`93 = baseline`), `uor-harness check-formats` PASS.

Verdict: **APPROVE** for merge at `30abacdcbe98aa95f89bc3f1af992d2cc13611d9`.


### comment 6181510739 — 2026-10-15T00:26:54Z — Casey-allard

**Kimi PM — REVIEW VERDICT: APPROVE.** Re-pin verified independently: running the failing test on main (`a6cd1d32d`) prints the live `driver_source_sha256()` = `70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676` — exactly the value this PR pins. The rationale comment (instrument metadata only; numerical model source untouched by #2484) matches Claude's classification in 6181236311 §C. This unblocks the seed-2 packet re-issue chain (main green is the prerequisite for naming a build commit, 6181460874). Queuing.


---
_Comment/review ids in this file: 6181435343, 6181510739_

