# The pod gate is the D29 interlock, not a CPU-reference parity test

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** Claude's item 5 (#2028 comment 6150176928); D24, D29

## Every bootstrap was failing, for two independent reasons

`scripts/pod/uor-pod-bootstrap.sh` ran `--test cuda_stack_ops_parity` on every `uor-pod up` and
**refused the pod unless it passed.** Both of these were true at once:

1. **It is a CPU-reference test.** Its own header: *"Numerical Parity Tests: CPU Reference vs Native
   CUDA Stack Kernels."* D24 forbids CPU model compute, so **every pod admission depended on a
   prohibited execution.**
2. **It does not compile.** `StackConfig` gained `rec_gates` (#2293) and this file's initializers were
   not updated — **zero** occurrences of `rec_gates` against a struct field that is now required. So
   the gate failed for a second reason that had nothing to do with compliance.

**Both are fixed by changing what the gate is, not by repairing the test.**

## The fix

**The gate is now `--test d29_compiled_modules`,** the D29 interlock. It is the stronger check and the
compliant one: it verifies the sources `build.rs` compiles carry no banned family, and with
`--features cuda` it checks the built PTX itself. **It needs no CPU reference to be meaningful.** A
failure **refuses** the pod (exit 4) rather than warning, because under D29 presence counts — a pod
whose compiled modules contain a banned mechanism cannot be used for D29 work.

`BUILD.json` now records `d29` / `d29_summary` / `d29_seconds`; `cache_ok` and `store_build_ok` accept
**both the new key and the old `parity`**, so builds cached before this change stay valid instead of
being silently rebuilt, and the result line reads `d29` with a `parity` fallback for an old file.

## The parity test is preserved, not deleted

`crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` — **3,402 lines — is kept and marked
`#![cfg(any())]`**, which is always false. It is excluded from every build while its text stays
readable, the same treatment D29 gives the historical kernels. The header states both reasons and says
what a future owner decision would need to do to revive it: a **device-side** reference and a
`rec_gates` value — **not a revert of that line.**

## A doccheck gap this exposed

`uor-harness ratchet` (#2346) is documented in `AGENTS.md`, `DECISIONS.md` and
`agent-execution-policy.md`, and `doccheck` reported it as **`unknown subcommand 'ratchet'` in all
three** — reddening `test_doccheck` on main. The Rust-only command set has to **grow with the port**;
keeping it current is part of porting a command, not an afterthought. `ratchet` and `version` added.

## Verification

| check | result |
|---|---|
| `bash -n scripts/pod/uor-pod-bootstrap.sh` | syntax OK |
| `cargo test -p uor-r4-training --no-run` | 0 errors (the parity file no longer compiles in) |
| `python3 -m unittest discover -s tests/harness -t .` | **319 passed** (was 3 failing on main) |
| `uor-harness check` | **0 errors** |
| `harness.documented_commands` | all pass |

## Limitations, stated

- **I could not run the D29 gate.** It needs `--features cuda` and a device; this laptop has neither.
  **What is verified is that the gate is wired to the right test and that the script parses** — not that
  the test passes on a pod. The first real `uor-pod up` is the actual test of this change.
- **The parity test is now unreachable by design.** If some workflow depended on it beyond the
  bootstrap, that workflow is broken by this change and I have not searched for one.
- **`rec_gates` in the parity file is still missing.** I did not repair it, because repairing a
  never-compiled file would be maintaining a prohibited test. If the owner wants numerical parity back,
  that is the work.
- **The exit code changed** from 3 (parity) to 4 (D29). A wrapper or runner keying on 3 would no longer
  recognise the failure.
