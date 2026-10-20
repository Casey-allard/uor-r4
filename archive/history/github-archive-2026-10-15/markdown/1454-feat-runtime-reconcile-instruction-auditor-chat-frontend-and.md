# #1454 — feat(runtime): reconcile instruction auditor, chat frontend, and H4 table fixtures (#964, #973, #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T06:13:45Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-28T07:15:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1454
- **Merged:** 2026-09-28T07:15:21Z
- **Merge commit:** 8c4e45de8816ade4eba4936effac291de95c7fc0
- **Base / Head:** main / UOR-Foundation:lab/anti-gravity/reconciled-serving-runtime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR addresses all outstanding runtime/frontend audit items identified in the oversight audit (`oversight-audit-20260928-0343/OVERSIGHT_HANDOFF.md`) and review items from #1434 and #1446, rebased cleanly on `origin/main` (incorporating #1453):

1. **Instruction Auditor Hardening (`scripts/audit_zero_matmul_serving.py`):**
   - Fixed regex matcher coverage for 6 missed sentinels identified in oversight audit: numbered registers (`fneg d0, d1`, `fneg d16, d31`, `fabs s0, s1`, `fabs s15, s31`), vector floating arrangements (`fneg.2d v6, v6`, `fabs.4s`, etc.), floating min/max (`fmax`, `fmin`, `fmaxnm`, `fminnm`, `fmaxp`, `fminp`, `fmaxv`, `fminv`, `fmax.2d`, `fmin.4s`), and vector floating/widening multipliers (`fmla`, `fmls`, `smull2`, `umull2`, etc.).
   - Expanded regression suite with both failing sentinels and a benign integer instruction suite (`add`, `sub`, `neg`, `neg.2d`, `lsl`, `lsr`, `asr`, `ldr`, `str`, `cmp`, `csel`, `tbl`, `mvn`, `and`, `orr`, `eor`) confirming 0 false positives.
   - Enforced strict failure exit if mandatory symbols are missing from the inspected binary.
   - Bound metadata in report and TAP v13 output: `artifact_sha256`, `git_commit`, `compiler` (`rustc --version`), and `flags`, with optional CLI flag overrides (`--git-commit`, `--compiler`, `--flags`).
   - Covered actual numerical entry points and callees in transitive call graph analysis including sampling roots (`Sampler::select`, `select_ticket`, `exact_min_p_threshold`, `validate_distribution`, `heapsort_ranked`, `step_zeta_phase_scalar`) while filtering compiler `drop_glue` and `closure` helpers and allowlisting frontend tokenization.

2. **CLI Chat Frontend Hardening (`crates/uor-r4-integer/src/bin/uor-chat.rs`):**
   - Replaced heuristic `+ 3` prompt budgeting with exact prefix token calculation using `protocol.bind(bundle.tokenizer())?.encode_user_prefix(trimmed, has_history)`, initial unobserved tokens, and caller EOS insertion.
   - Handled stream step errors cleanly with session rollback and non-terminating warning instead of abruptly exiting the process.
   - Retained transactional persona and read-mode settings with verified selection recording.

3. **H4 Group Table Fixture & Digest Pinning (`crates/uor-r4-integer/src/h4_tables.rs`):**
   - Pinned mathematical sha256 (`70cb0b96e551b4c6db21931e7e5ab49a3d8ab55b493ed66760cf6b4207c4e968`) compiled via `export-h4-tables`.
   - Committed the canonical `crates/uor-r4-integer/fixtures/historical-h4-tables-v1.bin` (15,601 bytes), resolving unit test failures in `h4_tables.rs` (78/78 lib unit tests pass).

4. **Published Energy Evidence & Limitations (`docs/evidence/energy-qualification-20260928.md`):**
   - Published formal forensic evaluation of the Track T3 Apple Silicon M1 marginal energy measurements across {\text{low}} = 10,240$ and {\text{high}} = 32,768$ ($\Delta N = 22,528$ steps).
   - Documented explicit scientific qualifications: checkpoint divergence (`bundle-quaternion-1` vs `fit256-quaternion-3`), whole-system marginal host power via non-root `macmon` `sys_power`, lack of isolated SoC powermetrics rail validation, and memory traffic attribution.
   - Documented grouped-LUT (T-MAC style) memory traffic reduction and per-token bandwidth requirements (width 256 touches ~0.98 MB/token, width 576 touches ~2.6 MB/token).

## Verification

- `cargo test -p uor-r4-integer --release -j 3`: 78 lib tests PASS, 16 integration suites PASS (including `dialogue576_turn_parity` 58 turns, 1,433 decisions bit-for-bit exact).
- `python3 scripts/audit_zero_matmul_serving.py /Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/uor-chat --tap`: 5/5 TAP tests OK, 0 Class I (multipliers), 0 Class II (dividers), 0 Class III (floats) across 55 symbol ranges and 113 reachable functions.
- `cargo fmt --check`: PASS
- `python3 scripts/check_claim_wording.py`: PASS

References #964, #973, #820.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5865231464 — 2026-09-28T07:14:28Z — Casey-allard

**Director review (Lab 1), 07:24 UTC: approved for the queue** (head `4493d220`).
- **Auditor:** every pattern #1451 completed is still present, and several are expanded (`fmls`, `usdot`, `fmaxnm`, `sqdmull`, `smmla`, `fneg`). I ran `run_sentinel_tests()` from this branch's script: it passes, with both failing and benign sentinels. Binding the compiler, flags and commit, and failing on missing mandatory symbols, are improvements.
- **Rust:** `uor-chat` exact prefix budgeting and rollback, and the pinned H4 fixture (`70cb0b96…`), are covered by your reported 78 library tests, 16 integration suites and bit-exact 58-turn parity.
- **Energy note:** scoped as whole-system marginal with the checkpoint divergence disclosed, and the claim-wording gate passes. It establishes no lower-energy claim, which is correct.
- **Next:** #1452 still needs `main` merged. Then D4 (fidelity objective), pre-registered on #973.


---
_Comment/review ids in this file: 5865231464_

