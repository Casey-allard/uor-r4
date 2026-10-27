---
id: m1-fresh-panel-transfer
title: Evaluate the gain on the fresh sealed panel
milestone: M1
area: evaluation
status: done
open_to: lab
needs_gpu: true
depends_on: [m1-export-gap-and-toks]
notebook: docs/research/notebook/m1-discrete-attention.md
scope: [docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-fresh-panel-transfer*, docs/labs/m1-fresh-panel-transfer/**, docs/loop/tasks/m1-fresh-panel-transfer/**, docs/research/notebook/m1-discrete-attention.md, docs/research/open-questions.md, docs/start/faq.md]
---

## Why it matters

M1 bar (e) stops a result that only fits its development panel from closing M1. The M2 reply model failed this way: 437 of 512 replies, later judged panel fitting, then 0 of 128 on opened transfer rows (`docs/start/faq.md`, question 21; a genuinely fresh test has not run). It cannot run before the earlier bars pass. See [where we're going](../../start/where-were-going.md).

## What is missing

M1 bar (e) needs the gain to hold on a fresh, sealed test panel registered in advance. The fresh slice remains sealed: the 19 October outcome call records fresh transfer as `NOT_TRIGGERED` (binding `4bd108ad…`, unopened), and the A8 sheet names the fresh V2 tokens, mask and seal as `0b7e754c…`, `a1cda529…` and `df5d074a…`. The `trunk-joint-pointer-v1-d11` model has now passed bars (a)–(d) on `ft/dev` across both seeds, so Step 4 can score that model once. Frozen `ft/dev` is not reported as fresh.

The fresh panel's size, noise floor and thresholds are registered in `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`: 14,479 eligible targets, 3,594 `P_succ` rows, and the paired `max(floor, ceil(2 * sqrt(b + c)), two-seed spread)` rules. CHARTER §4.3 requires decisive runs to use at least 2 seeds or at least 200 fresh rows, with bars no narrower than measured noise and a fresh transfer number with every development gain.

**All 5 steps are complete on `main` (`PASS_BAR_E_FRESH_TRANSFER`, `bar_e_pass = true`; `antigravity-99` #3292, `antigravity-100` #3293, `antigravity-109` #3311, `antigravity-110` #3312, `antigravity-111` #3318).** `antigravity-99` (#3292) and `antigravity-109` (#3311) landed `inst::fresh_transfer` (`refuse_sealed_fresh_source`, `validate_fresh_transfer_gate`, `fresh_rows`, `fresh_rows_from_corpus`, `build_train_count_tables`, `score_fresh_against_count_tables`, `evaluate_fresh_transfer_two_seeds`, and `d11-scorer fresh-score` in `docs/labs/m1-d11-artifact-scorer/instrument/`), posted the Step 2 design note (`#issuecomment-6423580146`), stream-verified the sealed archives (`codex-attention-dose1-fresh-v2-preservation.tar` `2cd13ef6…`: `0b7e754c…` / `a1cda529…` / `df5d074a…` / `c2d4c7e5…`; `admission-redirect-transfer-20261015-codex-v2.tar` `004523df…`: `4bd108ad…` / `82ead8db…` / `8287d20d…`) without opening or scoring any row, and registered [`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md) with non-author `TEST FITNESS: FIT` and D67 pre-run review (`#issuecomment-6423732020`, `#issuecomment-6432026082`). In `antigravity-110` (#3312) and `antigravity-111` (#3318), `d11-scorer fresh-score` scored the two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `04652a3d…` and `1b49e8bd…`) artifacts across two ahead-of-time CUDA builds (`Build A` `ce5843b0…` and `Build B` `0357abf8…`) on `1x RTX 5090` (`vast-57903382`): `7,215` and `7,228 / 14,479` (`+5,936` / `+5,949` over `Unigram = 1,279` vs floor `156`; `+633` / `+646` over `T2 = 6,582` vs floor `82`), `1,713` and `1,711 / 3,594` on `rposition` `P_succ` (`+473` / `+471` over `T2_psucc = 1,240` vs floor `52`; `2,333` / `2,329` vs `1,863` on `4,900` `windows(2)` rows), `0` refused/mismatched rows, and `B4` pass (`PASS_BAR_E_FRESH_TRANSFER`, recorded in [`docs/labs/m1-fresh-panel-transfer/README.md`](../../labs/m1-fresh-panel-transfer/README.md) and [`evidence/`](../../labs/m1-fresh-panel-transfer/evidence/)).

## Done when

'(e) Fresh transfer. The gain holds on a fresh, sealed test panel registered in advance.' Read with bar (d), "the gain" is integer top-1 through the D11 export beating the unigram table (1,226 of 14,287) and the next-word table (6,300 of 14,287) on each of two seeds. Step 3 has registered the fresh-panel thresholds and noise floor in `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`; Step 4 applies them on the single fresh opening.

## How to start

1. Read the current state. Open `docs/labs/k16-outcome-call-2026-10-19/README.md` (line 53, fresh transfer `NOT_TRIGGERED`), `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md` (the fresh order, lines 73, 87-89 and 120), `crates/uor-r4-discrete/src/attention_transfer.rs` (the deferred paired fresh evaluation) and `docs/research/open-questions.md`. Check whether the scoring path already refuses to open the slice before a sealed DEV decision. If it does not, write a failing test that shows it.
2. Post a design note on the issue as your first deliverable. It must say what "the gain" is on the fresh slice (read with bar (d): integer top-1 through the D11 export against the same two count tables), what is frozen before the slice opens (row count, the seal hashes, the tables, the seeds), and who opens it. If any of this changes the bar, it is a decision card approved by the lab's own fresh-context decision review and logged for the owner (CHARTER §5.1, §5.2, D68).
3. Draft the one-page pre-registration sheet in `docs/harness/prep/` (use the A8 sheet as the format model). Register the fresh-panel threshold and noise floor, bound to the sealed bytes by their hashes. Verify the seal against the stored bytes without opening or scoring the slice. A non-author must give TEST FITNESS: FIT before it runs (CHARTER §4.5).
4. Do not run it until `m1-export-gap-and-toks` has produced a saved model that passes the earlier bars on two seeds, and a fresh-context pre-run review has passed after its FIT (CHARTER §5.1, D67, D77). Score once. Do not re-draw or re-open the slice; use DEV for any earlier look.
5. Open a PR that says `References #<this issue>` for the note, the test and the sheet, and `Closes #<this issue>` only when the fresh number is recorded. Add one row to `docs/research/notebook/m1-discrete-attention.md`.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:14`](../../../ROADMAP.md)
- [`docs/integration/NOW.md:85`](../../../docs/integration/NOW.md)
- [`docs/start/where-were-going.md:34`](../../../docs/start/where-were-going.md)
- [`docs/start/faq.md:111`](../../../docs/start/faq.md)

