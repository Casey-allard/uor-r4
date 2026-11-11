# REPORT: infra-loop-harness, deepseek-119 (resolve contract decision locators)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md)
(#2904). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, pod or spend.

**In one line:** stack contracts now refuse a decision or legacy-grandfather
locator that is missing or escapes the repository; all four existing
contracts still resolve.

## What did not work, first

- **The base accepted both broken lifecycle locators.** A synthetic contract
  with `decision = "docs/no-such-decision.md#accepted"` passed, and one with
  `grandfathered_by = "../../outside.md#legacy"` passed. The first mutation
  failed the new test before the repair exactly as the cancel probe required.
- **The existing checks stopped at non-empty text.** Evidence paths, mechanism
  implementations and probe fixtures already resolved, but the two fields that
  prove why a contract is current or grandfathered did not.
- **No model, artifact, serving path or geometric arithmetic changed.** This
  is repository-model validation only.

## What worked

- A present `decision` is resolved through the same repository-relative path
  checker as evidence and mechanism files. A trailing Markdown fragment is
  ignored for file lookup and remains part of the recorded locator.
- A present `artifact.grandfathered_by` receives the same resolution check.
- Missing, absolute and parent-escaping lifecycle paths now fail with the
  field name in the error.
- The four registered contracts still pass because every existing decision
  and grandfather path resolves at the head.

## What changed

- `crates/repo-model/src/stack.rs` adds the two resolution calls.
- Its stack test module adds `contract_lifecycle_locators_must_resolve`, which
  proves a missing decision and a parent-escaping grandfather are refused.
- This ORDER and REPORT are the remaining changes. Generated contract views,
  contract contents, artifact schemas and runtime code are unchanged.

## Measurements

- Base mutation: missing decision and parent-escaping grandfather both accepted
  by `StackContract::check`; the new test failed on `unwrap_err()`.
- Repaired head: both mutations return errors naming `decision` and
  `grandfathered_by`.
- Existing contracts: 4 checked, 0 changed.
- Model identity: 33 IDs checked, unchanged.

## Checks run at the head

- `cargo test --offline -p repo-model stack`: 14 passed, 0 failed.
- `cargo run --offline -q -p xtask -- stack-contract --check`: four contracts,
  generated files current.
- `cargo run --offline -q -p xtask -- check-model`: pass, 33 IDs.
- `cargo clippy --offline -p repo-model --lib --tests -- -D warnings`: pass.
- `rustfmt --edition 2021 crates/repo-model/src/stack.rs`: clean.
- `git diff --check`: pass.
- `uor-harness loop report --check --order .../ORDER.md`: pass.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: `PASS`. Its
  commit-order advisory records that the first ORDER commit wrapped the item
  link before the parser-required same-line form was restored; scope,
  report, owner paths, rule sources and rendered views pass. Its file-map
  advisory is pre-existing and names no file in this unit.

## Skills used

`working-discipline` kept the audit read-only until a failing premise was
measured. `solution-integrity` required the base mutation to pass before the
repair and the changed check to fail on the missing and escaping inputs.

## Still open

- Command input-flow references are outside this ORDER. The audit did not add
  a second lifecycle grammar without a measured failure and an existing
  contract migration.
- The current artifact and serving work remains Claude's track and was not
  touched.

## Plain summary

A stack contract can say which review accepted it. Before this repair, it could
name a file that did not exist and still pass. The validator now checks that
the named decision or grandfather record is a real file inside the repository.
All current contracts pass, and two broken examples fail as they should.
