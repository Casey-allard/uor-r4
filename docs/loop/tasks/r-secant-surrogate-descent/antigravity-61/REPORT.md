# REPORT: r-secant-surrogate-descent, antigravity-61

Lab: antigravity. Item: [r-secant-surrogate-descent](../../../../roadmap/items/r-secant-surrogate-descent.md). Order: [ORDER.md](ORDER.md).
References #2886.

## What did not work

1. **No pre-registration sheet or lab protocol existed for `r-secant-surrogate-descent` (`instrument`):** Probing `docs/harness/prep/r-secant-surrogate-descent*` and `docs/labs/r-secant-surrogate-descent` returned no files, so no GPU run could be launched under `CHARTER.md` §4.5.
2. **The required D37 offline-float execution crate (`float-training-module`, #2823) is not built yet (`instrument`):** Under `CHARTER.md` §3.4 and §3.7 (D37) and `float-training-module` (#2823), offline float forward, gradients, and optimizers are legal only inside the separately scanned crate `crates/uor-r4-float-train` (#2823), never by executing `crates/uor-r4-training`. Because `#2823` is still `status: ready` and both secant surrogates (`rate_select_bwd` in `crates/uor-r4-training/src/cuda_stack.cu:259-291` and `spiralcore_fbs_bwd` in `crates/uor-r4-training/src/spiralcore_fbs.cu:124-183`) currently live only inside `crates/uor-r4-training`, the GPU run cannot be dispatched until `#2823` lands on `main`.

## What changed

- **Files changed:** 6 files (`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`, `docs/labs/r-secant-surrogate-descent/README.md`, `docs/roadmap/items/r-secant-surrogate-descent.md`, `docs/roadmap/README.md`, `docs/loop/tasks/r-secant-surrogate-descent/antigravity-61/ORDER.md`, `docs/loop/tasks/r-secant-surrogate-descent/antigravity-61/REPORT.md`).
- **Size vs. estimate:** ~170 changed documentation/prep lines across 6 files (1.0x against a 1-page prep sheet + lab protocol + task record).
  1. [`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`](../../../../harness/prep/r-secant-surrogate-descent-2026-10-25.md): Wrote the one-page pre-registration sheet defining the two-mechanism secant-surrogate descent test (`rate_select_bwd` for `RecGates::Geometric` and `spiralcore_fbs_bwd` for `QuaternionWeights::SpiralcoreFbs`) against zero-credit (`RecZeroCredit_Null`, `FbsZeroCredit_Null`) and matched-norm random-sign (`RecRandomRefit_Null`, `FbsRandomRefit_Null`) controls across two seeds (`seed = 1`, `seed = 2`), three instrument gates (`I-Finite-CUDA`, `I-Boundary-Crossings`, `I-Keep-Plateau`), two-seed noise-band decision bars (`G-Rec-Descent`, `G-Fbs-Descent`), data/tokenizer SHA-256 pins, and pod cost/stop rules; obtained non-author `TEST FITNESS: FIT` ([`#2886#issuecomment-6395161953`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6395161953)).
  2. [`docs/labs/r-secant-surrogate-descent/README.md`](../../../../labs/r-secant-surrogate-descent/README.md): Documented the exact secant equations in `cuda_stack.cu` and `spiralcore_fbs.cu`, the instrument vs. mechanism classification rules (`I-Boundary-Crossings` for step size vs. cell width and `I-Keep-Plateau` for the `f32` `KEEP_TABLE[12..15] == 1.0f` plateau), and the dependency hold on `float-training-module` (#2823).
  3. [`docs/roadmap/items/r-secant-surrogate-descent.md`](../../../../roadmap/items/r-secant-surrogate-descent.md) and [`docs/roadmap/README.md`](../../../../roadmap/README.md): Transitioned `r-secant-surrogate-descent` (#2886) from `status: ready` to `status: held` under a fresh-context D68 decision review ([`#2886#issuecomment-6395161953`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6395161953)) until `float-training-module` (#2823) lands, and re-rendered `docs/roadmap/README.md`.

## Measurements

- **Prep-sheet fitness (`uor-harness check`):** `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` is verified `TEST FITNESS: FIT` (`0` errors, `0` `prep.fit_before_run` warnings on `r-secant-surrogate-descent`).
- **GPU execution:** Not run (`status: held` until `float-training-module` #2823 merges `crates/uor-r4-float-train`).

## Checks run at the head

- `~/.cargo/bin/cargo run -p xtask -- roadmap check` -> PASS (`92 items OK`)
- `~/.cargo/bin/cargo run -p xtask -- docs-catalog --check` -> PASS (`0 paths not found`)
- `~/.cargo/bin/cargo run -p xtask -- links --json local/antigravity/links.json` -> PASS
- `uor-harness check` -> PASS (`0 error`)
- `uor-harness loop gates --full` -> PASS
- `uor-harness loop report --check --order docs/loop/tasks/r-secant-surrogate-descent/antigravity-61/ORDER.md` -> PASS
- `uor-harness loop pr-check --local` -> PASS

## Skills used

- `uor-research-architect`: verified exact kernel equations (`rate_select_bwd` and `spiralcore_fbs_bwd`) and separated step-size / `f32` table-underflow instrument checks from mechanism descent bars.
- `scope-guard` and `solution-integrity`: kept all changes inside `r-secant-surrogate-descent`'s declared scope and held GPU execution cleanly on `#2823`.

## Still open

- Once `float-training-module` (#2823, `crates/uor-r4-float-train`) merges to `main`, run the 12 micro-probe arms (`6` arms $\times$ `2` seeds $\times$ `256` steps) in `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` on a leased `uor-pod` RTX 5090 under a fresh-context D67 pre-run review and record the descent/no-descent verdict for both mechanisms.

## Plain summary

Staged and FIT-reviewed the one-page pre-registration sheet (`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`) and lab protocol (`docs/labs/r-secant-surrogate-descent/README.md`) to test whether the declared geometric secant surrogates for recurrence gates (`rate_select_bwd` on the 16-entry dyadic fade/keep table) and SpiralCore/FBS (`spiralcore_fbs_bwd` on the 16-root $E_8$ orbit and base-3 binder tree) produce loss descent against zero-credit and random-sign controls across two seeds. Because offline float training is legal only inside the separate scanned module `crates/uor-r4-float-train` (`float-training-module`, #2823, which is not yet built), `r-secant-surrogate-descent` (#2886) is placed on `status: held` under a logged D68 decision review until `#2823` merges.
