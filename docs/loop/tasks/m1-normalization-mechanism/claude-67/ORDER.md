# ORDER: m1-normalization-mechanism, claude-67 (DEV scoring for the normalization arms)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). Base: `origin/main`. Casey in Claude's chat, 2026-10-24: "build DEV scoring for C_unscaled and C_raw (let the DEV-B3 analysis accept the normalization entry's modes while the A8 path stays locked)".

## Goal

#3115 gave the normalization test a run entry, but the sheet's §6 step 5 (score paired DEV contrasts) has no tool:
- `attention-dev-b3` refuses non-default modes (`attention_preflight.rs:272`);
- the only in-crate paired counting is for the fresh transfer draw (`attention_transfer.rs`, with floor `max(20, …)`).

This unit adds two commands:
1. **`normalization-dev-b3 CONFIG CHECKPOINT DEV_EVAL OUT`.** It runs the existing DEV-B3 validation for any of the three arms on the A8 trunk config (`validate_normalization_config`) and records the mode and the normalization scope. `attention-dev-b3` keeps its lock.
2. **`normalization-dev-paired`.** It takes two arms' (config, checkpoint, DEV report) for one seed, with configs equal except `normalization_mode`. It pairs rows by identity on `top1_correct` and reports `b`, `c` and `F_I = max(24, ⌈2√(b+c)⌉)` in integers, plus `beats` when `b − c ≥ F_I`, for all rows and for `P_succ`. It also reports each arm's `P_succ` admission against the 1,351 floor. The registered bar terms it cannot score are reported as `UNSCORED`, with the reason: non-inferiority (no margin registered) and structural norm stability (needs per-stage traces).

`driver_source_sha256` moves, so the PR re-pins `prime_zeta.rs` and runs every discrete target.

## Cancel probe (run first)

At the base, can any command produce a DEV B3 or a paired DEV contrast for `C_unscaled` or `C_raw`? If one can, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**`
- This task folder

## Done when

1. **Per-arm B3.** `normalization-dev-b3` accepts all three modes on the A8 trunk config, and `attention-dev-b3` still refuses non-default modes.
2. **Paired scoring.** `normalization-dev-paired` refuses pairs whose configs differ in anything but the mode, or whose DEV row identities differ, and reports the counts and verdicts above.
3. **Pinned by tests.** CPU tests pin:
   - the pairing counts;
   - the integer `F_I`, both at the 24 floor and above it;
   - the mismatch refusals;
   - the A8 lock.

   One test must fail with its fix set aside.
4. **Checks pass:**
   - every discrete target and the `prime_zeta.rs` re-pin;
   - clippy on the changed files;
   - `fmt`;
   - D23/D25/D32 at baseline;
   - `loop report --check` and `loop pr-check --local`.

## Stop and ask if

- The A8 entry's behaviour, a scope or a schema must change.
- Anything outside `crates/uor-r4-discrete` must change.
- Structural norm stability needs a trace-capture path. Name it in the REPORT as the next unit.
