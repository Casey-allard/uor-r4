# ORDER: m4-flock-coupling-prereg, antigravity-25

Lab: antigravity. Item: [m4-flock-coupling-prereg](../../../../roadmap/items/m4-flock-coupling-prereg.md). Base: `2987514aa`.

## Goal

The item's done-when reads:

> A registered C1 flock-coupling sheet with a non-author FIT, or a recorded decision that it is superseded.

For this CPU-only task (`antigravity-25`, Issue `#2860`), we:

1. Verify whether C1 (`FlockCoupled`) is still wanted on the mechanism board (`docs/mechanisms/README.md:90`) and `docs/mechanisms/flock-read.md`, and post the design note plus the 5 open PM/owner decision questions on `#2860`.
2. Re-derive every control literal (`X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`, including the exact writer losses for `X7` and the literals for `X8` and `X9`) against current `main` (`crates/uor-r4-discrete/build.rs`, `src/discrete.cu`, and `src/flock_qualification.rs`), and add a deterministic host-side verification test in `crates/uor-r4-discrete` checking all hand-derived literals and the banned-token scanner.
3. Author `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` with all 8 required `prep.rs::REQUIRED_BLOCKS` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop`, `Cost`, `Minimum test power`, `TEST FITNESS: PENDING` until the PM/owner registers the bar, `k_max > 1` ruling, and base checkpoint) and `docs/labs/m4-flock-coupling-prereg/README.md` (with the staged six-column research notebook row for `m1-discrete-attention.md`), and update `docs/mechanisms/flock-read.md`.

## Cancel probe (run first)

Command: `sh -c 'trap "exit 130" INT TERM; sleep 30' & pid=$!; sleep 0.2; kill -INT $pid; wait $pid; echo exit=$?`
Observed: `exit=130` in ~0.2 s, zero orphan processes.

## Files in scope

- `docs/harness/prep/m4-flock-coupling-prereg*`
- `crates/uor-r4-discrete/**`
- `docs/mechanisms/flock-read.md`
- `docs/labs/m4-flock-coupling-prereg/**`

## Done when

- Design note and the 5 open PM/owner registration questions are posted on `#2860`.
- All finite control literals (`X1`–`X12`, `F1`–`F7`, `F5b`, `F5c`, including `X7` writer losses `[448, 448, 448, 256, 64, 256, 64]` / `X8` / `X9`) are re-derived from current `build.rs` `ROOTS`/`BFS` and `discrete.cu`, and verified by a finite host test in `crates/uor-r4-discrete` (`cargo test -p uor-r4-discrete`).
- `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` passes `uor-harness check` (`prep.sheet_shape`) and `docs/labs/m4-flock-coupling-prereg/README.md` and `docs/mechanisms/flock-read.md` are updated.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` are green.

## Stop and ask if

- Any file outside `scope:` needs modification.
- GPU execution is needed (this task is $0 host-only design, literal derivation, and pre-registration staging).
- Any PM/owner decision (bar registration, `k_max > 1` D11 ruling, D37 item-3 confirmation, or base checkpoint binding) is required before flipping `TEST FITNESS: PENDING` to registered.
