# C1 sheet-shape repair — 2026-10-18

Base: `origin/main` `bc485d1f3c45d690e6a967d23f009a1db291cb43` (#2620).
Branch: `kimi/c1-sheet-shape-repair-2026-10-18`, isolated worktree
`.worktrees/kimi-c1-sheet-shape-repair`. The owner checkout and every other
worktree were left untouched.

## Problem

The merged [C1 DRAFT sheet](../../harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md)
failed `prep.sheet_shape` in CI job 117572368828: no Bar, Base artifact, Data
split, Steps, Stop rule, Cost or Minimum test power block, and no TEST FITNESS
line. PM comment 6245063482 and #2029 comment 6245085269 record this as a
docs-shape blocker, not a mechanism result.

## Change

Added §8 to that sheet only. The seven required blocks are present and each is
truthfully marked `NOT_REGISTERED` or `NOT_RUN`, with `$0` cost, plus
`TEST FITNESS: PENDING` and an explicit DRAFT/no-FIT-requested line. `DRAFT`
itself is not an accepted harness verdict; `PENDING` is the harness's
unregistered/non-FIT state.

No bar, split, control, mechanism, source, artifact or hold changed. No
registration, FIT request, compute, pod, model run, push, PR or merge.

## Checks

- `python3 scripts/harness/cli.py check` — exit 0, 0 errors; `prep.sheet_shape`
  passes. The only new finding is the designed
  `prep.fit_before_run` **warning** for this sheet: a `PENDING` verdict with a
  run plan is not cleared for compute. It is not an error and authorizes
  nothing.
- Rust harness check (`local/harness-target/debug/uor-harness check --repo .`) —
  exit 0, 0 errors; `prep.sheet_shape` passes, and the same
  `prep.fit_before_run` warning is reported.
- `python3.12 scripts/harness/parity.py --build` — exit 0; 8 formats,
  53 rules, ledger/presence/prep/status/brief all agree. Exact line:
  `prep sheets read identically by both: ok`.
- `python3 scripts/check_claim_wording.py` — passed.
- `git diff --check` — clean.

The parity build used `~/.cargo/bin` first on `PATH`: the host default
`/opt/homebrew/bin/cargo` is the cargo-ai shim, not Rust cargo.

## Files

- `docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md` (§8 only)
- `docs/harness/claims.json` (this branch claim)
- `docs/harness/ledger/kimi-2026-10.jsonl` (harness-generated claim receipt)
- `docs/integration/current-state.md` (newest-first entry)
- this record

## Next

The sheet remains a post-Dose-1 registration draft. Registration and FIT wait
on the Dose 1 decision. Training/M2/Stage B remain HELD.
