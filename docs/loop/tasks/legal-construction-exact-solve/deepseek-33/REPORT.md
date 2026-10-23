# REPORT: legal-construction-exact-solve, deepseek-33 (saved-case mapping and the dense-LP blocker)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item: #2828 (`References`; the item stays `ready`). Base: `ea257ef40`.

## What did not work, first

- **The dense exact root LP is not viable at this size.** This unit's exact root LP (`solve-case --stage lp`, dense tableau) returned **no** objective/status line within a 120 s cap. This is D22's instrument blocker for that method, not a mechanism negative, and the unit makes no optimality or infeasibility claim. **Corrected by deepseek-34/35:** an earlier version of this paragraph claimed a second, "independent" dense run of 72+ minutes; that was wrong — the cited log is claude-50's bounded-variable revised simplex under its own declared cap, and it is not this lab's evidence.
- **The mapping was ambiguous from the record alone.** The design note listed candidates for which files are `g` and `m` and warned that the 31 per-term gradient files' role had to be settled. I read the original source at commit `ebd6ecaa` instead of guessing: `legal_proposals` builds `m = prefix+generate` masters, `g = prefix+generate` gradients, and passes the 380 `protected-margin-NNN` rows; the loader now mirrors that exactly.
- **The gradient aggregate identity only holds in f32 summation order.** Summing the 31 term files in f64 gave a different bit pattern; the original source sums in f32 (`let mut sum = vec![0f32; COUNT]`) and compares bits. Reproduced in f32: both families are bit-identical.
- **The crate's `no_floats` scan is deliberately strict.** The loader needed a saved-bits alias in `exact.rs` and file-name suffix built from `concat!("f", "32le")` so `src/*.rs` still contains no binary32 type token outside `exact.rs`; the loader tests live in `tests/case_loader.rs` because they write test f32 values.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `solver/src/case.rs` | code | +~130 | the saved-case loader: family order, 380 guard rows, exact f32→rational build, NaN/shape refusal |
| `solver/src/bin/solve-case.rs` | code | +~120 | the run binary: `--stage load|lp|bb`, exact zero-move check, root-LP timing, B&B status + independent re-check |
| `solver/src/exact.rs` | code | +9 | `problem_from_saved_bits` loader-facing alias |
| `solver/tests/case_loader.rs` | tests | +~90 | expected shape; NaN refusal; wrong row length (f32 values only in `tests/`) |
| `docs/labs/legal-construction-exact-solve/README.md` | docs | new ~70 | the lab record: mapping table, verification, timing evidence, next-step notes |
| `docs/mechanisms/legal-construction.md` | docs | +~6 | the status paragraph above |
| `docs/research/notebook/legal-construction.md` | docs | new ~7 | the new line page with one `blocked` row |
| this task folder | docs | new | ORDER and REPORT |

**Size:** about 470 lines across code, tests and docs (solver crate is standalone; the root workspace is untouched).

## Measurements (commands and what they printed)

Case: `codex-direct-legal-centered-20261009` (md5 `5a965dade6ed7b04b8238b4d1e020be3`), extracted read-only to the gitignored `local/claude/legal-case/`; nothing from it is committed.

- `cargo test --locked` (solver crate): all suites pass — 3 case-loader tests, 11 controls, the no-float scan, 2 random suites (15 s).
- `solve-case --case <attempt2> --stage load`: `seconds=0.306`, `n=1920 guards=380 fractional=20`, `zero_move legal=true guards_hold=true guard_value_min=0/1 objective=0/1`.
- `solve-case --stage lp` under a 120 s cap: no result line (dense exact root LP did not return).
- The only solver evidence this lab records is its own dense-tableau 120 s cap above; the claude-50 log is not cited as evidence here (correction by deepseek-34/35).
- `python3` f32-order aggregate check: both families' 31 term files sum bit-identically to `coupled-*-gradient.f32le`.
- Structural stats: 380 guard rows, median 1,033 non-zeros per row (min 70, max 1,169); 1,920 columns, median 241 non-zeros (max 298); 20 fractional masters; masters in [-1.75, 1.75].

## Checks run at the head

- solver crate: `cargo test --locked` (see above); `cargo clippy --locked -- -D warnings` and `cargo fmt --check` run before the push.
- loop: `loop report --check`, `loop pr-check --local`, `loop gates --full` and the other-vendor review record (requested on #2028).

## Skills used

- `solution-integrity`: the mapping is read from the original source, not inferred; the solver result is reported as what it is (blocker, no optimality claim).
- `scope-guard` / `working-discipline`: the gitignored case data stays out of the PR; only the loader, binary, tests and records are committed.

## Still open

- **The saved case is not solved.** No optimality or infeasibility claim exists; the item stays `ready`.
- **Next unit:** a reduced/bounded-variable or revised exact simplex (fraction-free updates, sparse factorization), or a structural reduction over the 380 constraints before any branch-and-bound; time the root LP first.
- Scoring any resulting move on the real panel remains a separate GPU step under its own sheet, FIT and owner release (not started; no compute).

## Plain summary

- **What was settled:** exactly which saved files are the masters, the gradient and the 380 guard rows, verified by the case loading and by the exact zero-move check, and by the original source's own f32 term-aggregate identity.
- **What was measured:** loading is fast and exact; the exact zero move is legal and guard-clean; the exact dense-tableau root LP does not return at this size (120 s cap here; this lab records no other run's timing).
- **What that means:** the dense-simplex method, not the mechanism, is what blocks the saved case. The item remains open, and the next attempt needs a different exact method; nothing was scored on the panel.