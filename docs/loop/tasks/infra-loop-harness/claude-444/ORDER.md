# ORDER: infra-loop-harness, claude-444

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md).
Base: `a9d8a35aa` (`origin/main`). Asked for by Claude's top session on 23 Nov, after the CI job
"harness rules and tests" failed on PR #4136; Casey answered "Yes, add them" (23 Nov) when asked
whether to add this ORDER and REPORT to PR #4197 so the loop queue takes it.
Setup row: 5 (the nearest row, the gate workflows: `harness.documented_commands` is a
`uor-harness check` rule that the CI job "harness rules and tests" enforces; no row names it).

## Why

`harnessrules::tests::the_real_repo_passes_the_ported_rules` fails on every PR whose merge ref
holds `main` at `7b74734fc` (PR #4192, antigravity) or later. The rule `harness.documented_commands`
reports `uor-harness claim-wording` as a command that does not exist, at two places:

- `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-11/REPORT.md` line 55
- `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-12/ORDER.md` line 7

The command exists: `crates/uor-r4-harness/src/main.rs` dispatches `claim-wording` to
`cmd_claim_wording` and lists it in the help. The rule resolves against a hand-kept table,
`ACCEPTED` in `crates/uor-r4-harness/src/doccheck.rs`, and that table does not hold it. The harness
workflow does not run on pushes to `main`, so `main` did not show the failure.

## What it does

1. **The fix:** `claim-wording` is added to `ACCEPTED` and to `RUST_ONLY_COMMANDS` in
   `crates/uor-r4-harness/src/doccheck.rs`. Nothing else changes: no command is added to the
   harness, the rule and its test are not weakened or skipped, and the two documents of
   antigravity are not touched.
2. **The test:** `claim_wording_is_a_command_this_binary_runs` in the same file. The bare command
   resolves, and an unknown flag after it is still refused.

## Gates

- **G1, the named test:** `cargo test -p uor-r4-harness the_real_repo_passes_the_ported_rules`
  passes (it fails on the base).
- **G2, the negative:** with the two table lines removed and the new test kept, the new test and
  the real-repo test both fail. Restore afterwards.
- **G3:** `cargo test -p uor-r4-harness` passes, `rustfmt --edition 2021 --check` on the changed
  file is clean, and `uor-harness check` reports no error.

No compute is rented for this unit; every run is on the laptop.
