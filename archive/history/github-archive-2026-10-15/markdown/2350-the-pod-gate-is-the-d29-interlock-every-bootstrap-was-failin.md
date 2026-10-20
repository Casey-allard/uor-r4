# #2350 — The pod gate is the D29 interlock; every bootstrap was failing (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T17:07:16Z
- **Updated:** 2026-10-13T17:09:54Z
- **Closed:** 2026-10-13T17:09:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2350
- **Merged:** 2026-10-13T17:09:38Z
- **Merge commit:** 0fdb2385eb9c719bd7d72cf56b20c3db577f1448
- **Base / Head:** main / UOR-Foundation:deepseek/nvrtc-gates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Claude's item 5 (#2028 comment 6150176928); D24, D29.

## Every bootstrap was failing, for two independent reasons

`scripts/pod/uor-pod-bootstrap.sh` ran `--test cuda_stack_ops_parity` on every `uor-pod up` and **refused the pod unless it passed.** Both of these were true at once:

1. **It is a CPU-reference test.** Its own header: *"Numerical Parity Tests: CPU Reference vs Native CUDA Stack Kernels."* D24 forbids CPU model compute, so **every pod admission depended on a prohibited execution.**
2. **It does not compile.** `StackConfig` gained `rec_gates` (#2293) and this file's initializers were never updated — **zero** occurrences against a now-required field. So the gate failed for a second reason that had nothing to do with compliance.

## The fix

**The gate is now `--test d29_compiled_modules`,** the D29 interlock. It is both the compliant gate and the stronger one: it verifies the sources `build.rs` compiles carry no banned family, and with `--features cuda` it checks the built PTX itself. **It needs no CPU reference to be meaningful.** A failure **refuses** the pod (exit 4) rather than warning, because under D29 **presence counts** — a pod whose compiled modules contain a banned mechanism cannot be used for D29 work.

`BUILD.json` now records `d29` / `d29_summary` / `d29_seconds`; `cache_ok` and `store_build_ok` accept **both the new key and the old `parity`**, so builds cached before this change stay valid rather than being silently rebuilt.

## The parity test is preserved, not deleted

**3,402 lines marked `#![cfg(any())]`** — always false, so it is excluded from every build while its text stays readable. The same treatment D29 gives the historical kernels. Its header states both reasons and what a future owner decision would need to revive it: a **device-side** reference and a `rec_gates` value — **not a revert of that line.**

## A doccheck gap this exposed

`uor-harness ratchet` (#2346) is documented in `AGENTS.md`, `DECISIONS.md` and `agent-execution-policy.md`, and `doccheck` reported it as **`unknown subcommand 'ratchet'` in all three** — reddening `test_doccheck` **on main**. The Rust-only command set has to **grow with the port**; keeping it current is part of porting a command, not an afterthought. `ratchet` and `version` added.

```
bash -n scripts/pod/uor-pod-bootstrap.sh            -> syntax OK
cargo test -p uor-r4-training (compile only)        -> no errors
python3 -m unittest discover -s tests/harness       -> 319 passed (was 3 failing on main)
uor-harness check                                   -> 0 errors
harness.documented_commands                         -> all pass
```

Record: `docs/labs/pod-gate-d29-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **I could not execute the D29 gate.** It needs `--features cuda` and a device; this laptop has neither. **What is verified is that the gate is wired to the right test and that the script parses — not that the test passes on a pod.** The first real `uor-pod up` is the actual test of this change.
- **The parity test is now unreachable by design.** If some workflow depended on it beyond the bootstrap, that workflow is broken here and **I have not searched for one.**
- **`rec_gates` in the parity file is still missing.** I did not repair it, because repairing a never-compiled file means maintaining a prohibited test.
- **The exit code changed from 3 to 4.** A wrapper keying on 3 would no longer recognise the failure.
- **Per R11 the `.cu` module sources are outside my scope and unchanged.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

