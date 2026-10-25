# REPORT: recurrence-gates-six-recurrence-fit, antigravity-82

Lab: antigravity. Order: [`ORDER.md`](ORDER.md). Item: [`recurrence-gates-six-recurrence-fit`](../../../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (`References #2890`; the item remains `status: ready` until its GPU run and D11 integer export-gap evaluation execute). Base: `e5f276619`.

## What did not work

- In the first draft of [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md), the `TEST FITNESS:` line in §8 was formatted as a markdown list item (`- \`TEST FITNESS: PENDING\``). Running `uor-harness check` caught `error prep.sheet_shape [docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md]: missing block(s): TEST FITNESS line` because `prep.rs` matches an unindented `TEST FITNESS:` line. Moving `TEST FITNESS: FIT` onto its own line immediately resolved `prep.sheet_shape` (`0 error`).
- No mechanism negative occurred; this unit is the pre-execution preparation unit (`CHARTER.md` §4.5, `recurrence-gates-six-recurrence-fit.md` lines 36–37, and `docs/mechanisms/recurrence-gates.md` reopen condition 1).

## What changed

| File | Class | Lines (`+` / `-`) | Notes |
|---|---|---:|---|
| [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md) | docs | `+85 / -0` | New one-page D37 offline-float pre-registration sheet with all 8 `REQUIRED_BLOCKS` (`bar`, `base`, `split`, `steps`, `stop`, `cost`, `power`, `fit`) and fresh-context non-author `TEST FITNESS: FIT` + D68 `DECISION REVIEW: APPROVE` (`a0104701-a870-4eab-ac9b-9126317a332d`). |
| [`docs/mechanisms/recurrence-gates.md`](../../../mechanisms/recurrence-gates.md) | docs | `+3 / -3` | Updates `## What would reopen it` to link the `TEST FITNESS: FIT` pre-registration sheet (`#2890`, `antigravity-82`). |
| [`docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-82/ORDER.md`](ORDER.md) | docs | `+32 / -3` | Task order and `instrument`-class cancel probe result. |
| [`docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-82/REPORT.md`](REPORT.md) | docs | `+70 / -0` | This report. |

- **Total size:** `4` files changed (`0` code, `0` tests, `4` docs, `0` generated), `+190 / -6` lines (`196` changed lines vs ~150-line estimate, ratio `1.31x`).

## Measurements

- **Lead metric (`tokens/s`, `CHARTER.md` §1):** N/A in this docs-only pre-registration unit; synchronized CUDA decode/context/training `tokens/s` is registered as the lead metric in §1 and `G-Rec-Sheet-Fit` (§4) of [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md) (baseline 256-step surrogate probe speed from `#2886`: `~8,738–8,851` training tokens/s on 1 $\times$ RTX 5090).
- **Cancel probe (`instrument` class, `2026-10-25`):**
  - Confirmed `r-secant-surrogate-descent` (`#2886`) is `status: done` on `main` (`G-Rec-Descent = PASS` at `1b5a6abe3`, `+0.059740` and `+0.078261` loss drop over `256` steps on seeds `1` and `2`, `30.73%–35.42%` dyadic table cell crossings).
  - Confirmed `crates/uor-r4-float-train/src/cuda_stack.cu` (`c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17`), `geometric_stack_native.rs` (`03aec162e3e40144a360e1f4381b9aeb142c505115e05dd59e1459157d2b24a4`), `geometric_stack_native_cuda.rs` (`03cc8a1166bf51bbeece6259886158acfbca7409a624aeb782b8f9ff7d0960e1`), and `hamilton_transport.rs` (`f376dced3e326a80fc67edabc59d0c607b9ffece6a27548e0bc74957afe96147`) are present on `main` with `0` banned tokens (`d23` / `d25`).
- **Pre-registration & fitness verification (`CHARTER.md` §4.5, §5.1):**
  - `uor-harness check`: `0 error` (`prep.sheet_shape` and `prep.fit_before_run` pass on `docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`).
  - `~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-harness --lib preprules`: `9 passed; 0 failed`.
  - Fresh-context non-author review (`a0104701-a870-4eab-ac9b-9126317a332d`): `TEST FITNESS: FIT` and `DECISION REVIEW: APPROVE`, logged on `#2890` via `[antigravity] owner log:`.

## Checks run at the head

- `uor-harness check`: `0 error`.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --no-fail-fast -p uor-r4-harness --lib preprules`: `9 passed; 0 failed`.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- links --json local/antigravity/links.json`: `0 broken links`.
- `uor-harness loop report --check --order docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-82/ORDER.md`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS` (`0 FAIL, 0 WARN`).
- `uor-harness loop gates --full`: `11/11 GREEN`.

## Skills used

- `scope-guard`: Kept all edits strictly inside the declared scope of `recurrence-gates-six-recurrence-fit` (`docs/harness/prep/recurrence-gates-six-recurrence-fit*`, `docs/mechanisms/recurrence-gates.md`, and this task folder).
- `solution-integrity`: Verified all 8 `REQUIRED_BLOCKS`, exact SHA-256 digests (`mw-balp/train`, `ft/dev` `14,287` targets, `fresh_v2` `14,479` targets, `tokenizer.json`, `cuda_stack.cu`, `geometric_stack_native.rs`, `geometric_stack_native_cuda.rs`, `hamilton_transport.rs`), and the `1,226 / 14,287` train-split response count-table baseline against the repository records.

## Still open

- Once `PR #3231` merges (releasing `crates/uor-r4-float-train/**`), restore `crates/uor-r4-float-train/**, crates/uor-r4-integer/**` to `recurrence-gates-six-recurrence-fit.md`, implement the six-recurrence runner binary (`--bin recurrence-gates-six-recurrence-fit`) and D11 integer recurrence table exporter, pass the D67 pre-run review, and execute the probe ladder and 2-seed run on a leased RTX 5090 pod.

## Plain summary

This unit (`antigravity-82`) authors and qualifies the one-page pre-registration sheet ([`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md)) for the six-recurrence geometric gates fit (`#2890`). A recurrent layer carries a running memory forward step by step; in UOR-R4's geometric recurrence (`RecGates::Geometric`), smooth squashing functions (sigmoid, exponential, tanh) are replaced by a clamped straight line (`clamp(g + d, 0, 1)`), a 16-entry table of exact powers of two ($\lambda = 2^{-(i+1)}$ and $\kappa = \sqrt{1 - \lambda^2}$) trained via a secant difference rule (`rate_select_bwd`), a unit-quaternion rotation scan (`quaternion_scan_fwd`/`bwd`), and a Hamilton rotation output gate (`Gate::Hamilton`). Now that the 256-step secant surrogate descent check has passed (`#2886`), this sheet registers the full 6-recurrence (`pattern = "rrrrrr"`) training comparison across seeds `1` and `2` on `mw-balp/train` (`1,024` steps per arm) against four controls — static rate-bin initialization (`Rec_FrozenRates_Null`), commutative $U(1)$ phase transport (`Rec_Geometric_U1_AbelianControl`), identity transport (`Rec_Geometric_IdentityTransport_Control`), and a word-count lookup table built on the training split (`TrainSplit_CountTable_Ref`, `1,226 / 14,287` top-1) — together with standalone CUDA kernel qualification, synchronized `tokens/s` speed measurement, D11 whole-number table export-gap scoring on all `14,287` `ft/dev` targets, and fresh transfer scoring on `14,479` `fresh_v2` targets. A fresh-context non-author reviewer verified the sheet and issued `TEST FITNESS: FIT` and `DECISION REVIEW: APPROVE`.

References #2890
