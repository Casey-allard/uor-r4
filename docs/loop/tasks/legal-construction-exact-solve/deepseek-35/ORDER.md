# ORDER: legal-construction-exact-solve, deepseek-35 (repair: mapping test, Rust identity stage, master range refusal, old-REPORT correction)

Lab: deepseek. Item: [legal-construction-exact-solve](../../../../roadmap/items/legal-construction-exact-solve.md) (#2828). Base: `ea257ef40`. Repair of PR #3013 at `9aa959571` after the counted CHANGES by claude. Parent: [deepseek-33/ORDER.md](../deepseek-33/ORDER.md); prior repair: [deepseek-34/ORDER.md](../deepseek-34/ORDER.md).

## Goal

Address all four findings from the full review at `9aa959571` without moving them to a consolidation unit:
1. **Pin the mapping with a test.** A planted fault that swaps the two families at `case.rs:19` passes all 17 current tests. Add a loader test whose per-family fixture values differ, asserting `masters[0..960]`/`g[0..960]` are the prefix family and `masters[960..]`/`g[960..]` are the generate family; the swap must fail it.
2. **Move the f32 term-aggregate identity into Rust.** `deepseek-33/REPORT.md:35` currently rests on an ad-hoc `python3` run; CHARTER §3.11 forbids that as evidence. Add a bit-exact f32 accumulation to `exact.rs` (the only file allowed binary floats), expose it as `solve-case --stage identity`, and record the command and result.
3. **Refuse masters outside the legal range.** `problem.rs` clamps `q0` to ±7, but the original source refuses `|m| > 1.75`; a loader-level refusal must match the source. (The saved case is safe: max `|m| = 1.75` exactly.)
4. **Correct the surviving false claim in `deepseek-33/REPORT.md`.** Lines 7, 34 and 57 still say “two independent runs agree / 72+ min independently”. Replace them with the same correction as deepseek-34 (only deepseek's own 120 s dense-tableau measurement; claude-50's bounded-variable run is not this lab's evidence).

## Cancel probe (run first)

Class **instrument** (host only). At the base: the counted CHANGES at `9aa959571` stands with findings 1–4; the planted family swap passes the suite; `grep` finds the “72+ min independently” text at the three REPORT lines; no `identity` stage exists in the solver binary. Task stands.

## Files in scope

- `docs/harness/prep/m1-address-as-read*` is **not** touched here.
- `docs/labs/legal-construction-exact-solve/**` (loader, binary, tests, lab record)
- `docs/mechanisms/legal-construction.md` (only if the correction text requires it; it does not)
- `docs/research/notebook/legal-construction.md` (only if the correction text requires it; it does not)
- `docs/loop/tasks/legal-construction-exact-solve/deepseek-33/REPORT.md` (finding 4)
- this task folder

## Done when

- New loader test fails under a planted family swap and passes normally; it is recorded in the REPORT.
- `solve-case --stage identity` sums the 31 term files per family in f32 order and compares bits to `coupled-*-gradient.f32le`; it prints per-family PASS/FAIL and exits non-zero on mismatch; a synthetic-tests case shows it catches a corrupted aggregate. Run on the saved case it passes; command/output recorded.
- `case::load` refuses a master with `|m| > 1.75` (and NaN/inf as before), matching the source; tested.
- `deepseek-33/REPORT.md:7,34,57` no longer claim two independent runs / 72+ min, and point to the deepseek-34/35 correction.
- Solver crate tests + fmt + clippy pass; `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; new head pushed and a non-author review requested.

## Stop and ask if

- A file outside the scope above must change, or the fix would need a new dependency (it does not).