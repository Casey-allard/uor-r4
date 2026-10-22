# REPORT: m1-address-as-read, deepseek-27 (repair: the FIT review link and the verdict gaps)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `2d24cf1bb`. Item PR; exact-head review at the head.

## What did not work, first

- **The deepseek-26 ORDER sheet link was wrong by one level and failed the docs gate.** `../../../harness/prep/...` from `docs/loop/tasks/m1-address-as-read/deepseek-26/` resolved to `docs/loop/harness/prep/...`; it is now `../../../../harness/prep/...`, and `xtask links` on the branch reports no new broken link (944 to 943 against the same source set).
- **The verdict was too narrow on the measurement side.** Claude was right that the review missed: the held-out BPB stream is never named or hashed and its `0.01` tolerance is unsourced; `>= 54/80` has no derivation; and the stop rules cover integrity failures only, with no bar-miss stop and no statement of which configuration a negative closes (§4.1). Those are now prerequisite 5 — still `NOT_FIT`, but for a reason the run cannot accidentally satisfy.
- **The recorded block was not in the shape `prep::render_fit` emits.** It is now a plain rendered block (verdict line, `sheet:`, `reviewer:`, `note:`, `date:`, the D22 closing line) under a short why-not-fit list; the ORDER no longer calls it by a name it did not match.
- **The deepseek-26 REPORT line count was stale.** Corrected to the measured sheet diff.
- **Prerequisite 1 now credits the 40/20/20 composition** and asks only for the panel SHA-256 and the 80-vs-200 reconciliation, as the LOW finding asked.
- **No GPU, no training, no bar/hold/spend decision.** The verdict clears no compute (D22).

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | docs | +~18/−12 | Verdict rewritten: five prerequisites in priority order, then the `prep::render_fit`-shaped record under a plain block |
| `docs/loop/tasks/m1-address-as-read/deepseek-26/ORDER.md` | docs | 1 line | the sheet link gains the missing `../` |
| `docs/loop/tasks/m1-address-as-read/deepseek-26/REPORT.md` | docs | 1 line | line count corrected |
| `docs/loop/tasks/m1-address-as-read/deepseek-27/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 55 changed lines (ORDER estimate about 50, 1.1).

## Measurements

- **Link:** `xtask links` = 943 broken on the branch after the fix (944 before, same source set); the new-broken gate compares against the merge-base, where this link is absent, so the docs job should be clean.
- **Sheet:** `prep-check` reads `not FIT NOT_FIT yes` and `may_compute: false`; all 8 blocks still present.
- **Gates:** nine GREEN; `--break all` nine CAUGHT.

## Checks run at the head

- `uor-harness prep-check`: sheet `NOT_FIT`, `may_compute: false`.
- `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`, `loop gates --break all`: before the review request.
- `xtask links`: no new broken link against the base.

## Skills used

- **solution-integrity**: the verdict was re-read against the sheet own metrics and stop rules, not widened by guesswork; each added gap names the exact missing artifact or derivation.
- **scope-guard**: only the item sheet and the two task folders changed.

## Still open

- **Pinning prerequisites 1-5** is the antigravity side; re-request FIT once they are in the sheet.
- **The GPU evaluation** stays a separate unit behind the owner bar, PM dispatch and the typed release.

## Plain summary

- **The docs blocker is fixed** — the ORDER link resolves and `xtask links` shows no new broken link.
- **The NOT_FIT verdict now also names the measurement gaps** (BPB stream/hash and tolerance source, the 54/80 derivation, the bar-miss stop and negative-closes rule), so the sheet cannot reach FIT by pinning only artifacts.
- **The recorded block now matches the harness own FIT shape**, and the deepseek-26 report count is corrected.