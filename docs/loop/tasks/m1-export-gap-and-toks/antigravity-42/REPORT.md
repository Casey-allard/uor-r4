# REPORT: m1-export-gap-and-toks, antigravity-42

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Negative / open:** No saved trained discrete model checkpoint (`uor-r4.discrete-spiralcore-fbs/2`) has cleared Stage 3 (`m1-close-saved-model`) yet (`m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, and `m1-trunk-assembly-and-training` have not produced a passing checkpoint, and training remains **HELD** under D40). Per step 1 of [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md) ("Check the progress of `m1-d11-exporter-discrete` and of a saved trained model; do not start a run without both"), no GPU DEV scoring or laptop timing run is executed in this unit.
- **Check finding repaired before review:** `uor-harness loop pr-check --local` flagged `## Stop rule` in [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) under `rule-sources` (headings ending in `rule` outside `docs/harness/rule-sources.json`); renamed the heading to `## Stop`, which passes both `prep.sheet_shape` and `rule-sources`.

## What changed

- [`docs/loop/tasks/m1-export-gap-and-toks/antigravity-42/ORDER.md`](ORDER.md) (`1-37`): task order and cancel probe for `antigravity-42`.
- [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (`1-118`): Stage 4 one-page draft pre-registration sheet (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING`).
- [`docs/labs/m1-export-gap-and-toks/README.md`](../../../../labs/m1-export-gap-and-toks/README.md) (`1-53`): Stage 4 design note, dependency audit (`m1-d11-exporter-discrete` is `done` from `#2998`; trained model checkpoint still pending), and compute boundary summary.
- [`docs/mechanisms/lut-serving.md`](../../../../mechanisms/lut-serving.md) (`28`): linked the Stage 4 pre-registration sheet and lab record under the mechanism history's `Next` bullet.
- [`docs/loop/tasks/m1-export-gap-and-toks/antigravity-42/REPORT.md`](REPORT.md) (`1-62`): this task report.
- **Size by class:** tests `0` files (`+0/-0`), code `0` files (`+0/-0`), docs `5` files (`+239/-1`), generated `0` files (`+0/-0`), other `0` files (`+0/-0`).

## Measurements

- Nothing was measured in this unit (`0` GPU hours, `$0.00` spend); this unit delivers the Stage 4 design note and draft pre-registration sheet (`How to start` steps 1–2 of [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md)), citing the existing K16 two-seed DEV receipts (`14,287` rows, `\Delta_{\text{noise,top1}} = 398`, `\Delta_{\text{noise,top8}} = 748`, `\Delta_{\text{noise,adm}} = 930` from [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../../../../labs/k16-outcome-call-2026-10-19/README.md)).

## Checks run at the head

- `uor-harness loop gates --quick` (cancel probe, `SIGINT` after `150 ms`): `rc = -2`.
- `uor-harness check --only prep`: `0 error` (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` passes `prep.sheet_shape` across all 8 required blocks; expected `warn prep.fit_before_run` on `PENDING`).
- `uor-harness check --only harness.documented`: `1 rules, 0 failed (0 error, 0 warn)`.
- `~/.cargo/bin/cargo run -p xtask -- links --baseline /tmp/antigravity-base-links.json --renames-from origin/main`: `new broken: 0`.
- `uor-harness loop gates --quick`: `fmt-claim-wording GREEN`, `fmt-harness GREEN`, `claim-wording GREEN`.
- `uor-harness loop gates --full`: `11 / 11 GREEN` (`fmt-claim-wording`, `fmt-harness`, `claim-wording`, `loop-gates-watch`, `loop-drive`, `loop-next-sources`, `loop-next-wait`, `loop-next-owner-direction`, `git-hooks`, `loop-migrate`, `loop-review-changes-blocks`).
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `verdict: PASS`.

## Skills used

- `uor-research-architect` (`.agents/skills/uor-research-architect/SKILL.md`).

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (CHARTER §4.5), owner/PM registration of the Stage 4 bar and two-seed band on `#2839` / `#2029` (CHARTER §5.2, D39 item 5), and a saved trained discrete checkpoint (`uor-r4.discrete-spiralcore-fbs/2`) from `m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, or `m1-trunk-assembly-and-training` before executing the GPU DEV export-gap evaluation and laptop throughput timing. [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../../../roadmap/items/m1-export-gap-and-toks.md) remains `status: ready`. References #2839, #2029.

## Plain summary

Stage 4 of Milestone M1 (`m1-export-gap-and-toks`, #2839) asks how much accuracy is lost when a trained model is exported into the multiplier-free integer serving file format (`UORLUT01`) and how fast that exported integer file reads prompts and writes replies on a laptop. One of the two required pieces—the discrete exporter and integer evaluator (`m1-d11-exporter-discrete`, #2838)—is already built and verified, while the other piece—a saved trained model checkpoint—is still waiting on training (currently held). This task writes the Stage 4 design note and one-page pre-registration sheet (`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`) so the exact evaluation panel (`14,287` development rows), the export-gap rule (exact zero gap for lossless discrete exports, or within the two-seed `398`-row top-1 noise band for offline float candidates), the separate prompt-reading (`192`-token prefill) and reply-writing (`64`-token decode) laptop timing protocol, and the GPU-versus-laptop compute boundary are ready for a non-author fitness review before any run starts.
