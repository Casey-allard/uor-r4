# ORDER: m4-lawful-likelihood-bpb, deepseek-59 (record the NOT_FIT and make the extension rule resolve)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `1cb89e81a`. Partial unit: `References #2861`; the item stays `ready` (the M4 bar is not met and no compute is cleared).

## Goal

Fold the non-author `TEST FITNESS: NOT_FIT` (claude, sheet read at `main fb6156b46`, #2861 14:06Z) into the pre-registration sheet without changing the bar, a hold, a status or a result: record the verdict verbatim, state the three prerequisites with their disposition, and fix the one prerequisite a lab can fix now - the extension rule - by scaling the measured standard error by window count (`se_n = 0.014525 * sqrt(512 / n)`; 0.010270 at 1024, 0.007262 at 2048) and stating what an INCONCLUSIVE at 2048 windows decides (a configuration result, no bar claim). Add the REQUIRED export-SHA/training-seeds pin to section 6. Also restore the complete historical FIT block, state the REPORT's gate result, and re-pin the stale driver-source hash without weakening its assertion.

## Cancel probe (run first)

At base `1cb89e81a`, `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` still reads `TEST FITNESS: PENDING` (status line and section 10) while Claude's `NOT_FIT` exists on #2861, and the extension rule still fixes `se = 0.014525` at every window count, so an INCONCLUSIVE cannot resolve. Task stands; the prepared patch was applied and validated against `uor-harness check`.

Takeover probe at `930797640`: the historical block omits the final sentence `Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.`, the REPORT has no gate outcome, and three binaries fail because the expected driver hash is stale (`b13d4920…` instead of intentional `791a555d001c1f7a0c313474d0410f52cf2006fd61faa67410316a52cc401882`). Both repairs stand.

## Files in scope

- `docs/harness/prep/m4-lawful-likelihood-bpb*`
- `docs/labs/m4-lawful-likelihood-bpb/**`
- `crates/uor-r4-discrete/src/prime_zeta.rs`
- this task folder

## Done when

- The sheet records claude's complete `NOT_FIT` block verbatim (reviewer, head, date), including the omitted final sentence, and the status line says NOT_FIT; the historical review is not rewritten.
- Sections 3, 5 and 6 state: the per-window `se_n` and 1024/2048 values; the 2048-INCONCLUSIVE rule; the two owner `CHARTER §5.1` prerequisites as open owner-only FIT prerequisites; the REQUIRED export SHA + training-seeds pin (blocked on #2859). The REPORT states the actual full-gate result.
- The expected driver hash is `791a555d001c1f7a0c313474d0410f52cf2006fd61faa67410316a52cc401882`; no assertion is removed, ignored or weakened, and `cargo test -p uor-r4-discrete` passes every target.
- No bar, hold, status, spend or result changes; no FIT or release claimed.
- `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted review records at the exact head.

## Stop and ask if

- A bar/hold/release decision, an actual driver-hash change, or a file outside the sheet/scope is needed.