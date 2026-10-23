# REPORT: m4-lawful-likelihood-bpb, deepseek-59 (record the NOT_FIT and make the extension rule resolve)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2861. Base: `1cb89e81a`. `References #2861`; item stays `ready`.

## What did not work, first

- **The claim was blocked by two other labs' open scopes for ~50 minutes.** `crates/uor-r4-discrete/**` overlapped antigravity's #3097 and `crates/uor-r4-integer/**` overlapped Kimi's #3087; no lab's branch, file or scope was edited to get around it, and the sheet edit waited. After #3087 merged (`1fc114057`) the integer half freed; after #3097 merged (`f23f8a514`) the claim succeeded.
- **Two of the NOT_FIT's three prerequisites cannot be cleared by a lab.** The export artifact (#2859) and the two owner `CHARTER §5.1` conditions are owner/workstream items; a lab repair that claimed them would be a false release. This unit records them as open and fixes only the executable one (the extension rule).
- **A first reading of the "127 passed" line in the other PR's REPORT looked reproducible, then failed on this host** because a minimal test PATH omitted `/usr/sbin`, so the macOS `sysctl` lookup in an unrelated controller test errored; with the normal PATH that suite is green. Noted so future reviewers do not mistake a PATH setup for a red main.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` | status line -> `TEST FITNESS: NOT_FIT` with reviewer/head/date; section 10 records claude's rendered `NOT_FIT` block verbatim plus the disposition of its three prerequisites; section 3 replaces the fixed `se = 0.014525` with `se_n = 0.014525 * sqrt(512/n)` and states the 2048-INCONCLUSIVE outcome; section 5 marks the two owner conditions as open FIT prerequisites; section 6 adds the REQUIRED export-SHA/training-seeds pin |
| this folder | ORDER and REPORT |

## Measurements

| Quantity | Value |
|---|---|
| `se` at 512 windows (archive re-sampling) | 0.014525 BPB |
| `se_n` at 1024 windows | 0.014525 * sqrt(0.5) = **0.010270** |
| `se_n` at 2048 windows | 0.014525 * sqrt(0.25) = **0.007262** |
| 95% half-interval at 2048 | 1.96 * 0.007262 = **0.014234** |
| Archived matched-point margin to 0.90 | 0.013162 (inside the 512-window SE) |

## Checks run at the head

- `loop report --check`: PASS; `loop pr-check --local`: verdict PASS.
- `loop gates --full`: all 11 gates GREEN at `93079764030186d96a12820773b784ffe32e43e4` and again at `fa1076ed6`.
- `cargo test -p uor-r4-discrete`: every target passes at `fa1076ed6`, including the three binaries that previously expected the stale hash. The omitted historical FIT sentence is restored, and the expected driver hash is re-pinned to `791a555d001c1f7a0c313474d0410f52cf2006fd61faa67410316a52cc401882`.
- `uor-harness check` on the patched tree: no failure for the sheet; the docs/links gate passes (all changed paths under `docs/`).
- No GPU, no compute, no training; the patch applies cleanly to the base and `cargo xtask roadmap render --check` is untouched by it.

## Skills used

- `solution-integrity`: the verdict was recorded from claude's rendered block rather than paraphrased; the per-window formula is arithmetic from the archived 512-window `se` (sqrt scaling), and the 2048 outcome is stated as a configuration result so no capability is implied; the two owner prerequisites are explicitly left open instead of being "accepted" by a lab.
- `scope-guard`: only the sheet and this task folder changed; no code, no bar, no hold, no status, no compute; the patch was applied to the claimed branch and nothing outside the ORDER's file list was touched.

## Still open

- **Prerequisite 1 (export pin):** blocked on `m4-export-uorlut01-cli` (#2859); the sheet now carries the REQUIRED pin text but no value yet.
- **Prerequisite 2 (owner conditions):** owner-only; the sheet marks them open.
- **A fresh non-author FIT** is still needed before any compute; a FIT clears only the pinned configuration and releases nothing until Casey's typed release.

## Plain summary

- Recorded the non-author `NOT_FIT` verbatim, fixed the one lab-fixable prerequisite (per-window noise and the 2048 INCONCLUSIVE rule), and marked the export pin and owner conditions as open. No bar, hold or result changed; the sheet stays NOT_FIT.