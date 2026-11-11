# ORDER: infra-loop-harness, deepseek-119 (resolve contract decision locators)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md)
(#2904). Base: `c0f7db014`.

## Goal

Make the lifecycle records already named by a stack contract resolve. A
contract's `decision` and `grandfathered_by` locators point to the review or
decision that accepted its state; a missing or escaping path currently passes
schema validation and lets the contract cite a decision that does not exist.

## Cancel probe

At the base:

- `cargo run --offline -q -p xtask -- check-model` passes 33 IDs.
- `cargo run --offline -q -p xtask -- stack-contract --check` passes four
  contracts.
- `cargo test --offline -p repo-model stack` passes 13 tests.
- A valid parsed contract with `decision = "docs/missing.md#decision"` reaches
  no path check.
- A legacy contract with
  `grandfathered_by = "../../outside.md#decision"` reaches no path check.

The unit is cancelled if either mutation is already refused at the base.

## Files in scope

- `crates/repo-model/src/stack.rs`
- `docs/loop/tasks/infra-loop-harness/deepseek-119/**`

## Done when

1. A present `decision` locator resolves to a repository file; fragment
   suffixes remain allowed.
2. A present `grandfathered_by` locator resolves to a repository file.
3. Missing and parent-escaping mutations fail with the field name in the
   error.
4. The four existing contracts and all repo-model stack tests pass.
5. `check-model`, generated stack views, formatting, Clippy on the changed
   crate, report check and local PR check pass, or any pre-existing warning is
   named explicitly.

## Stop and report

Stop if an existing contract names a record that does not resolve, or if the
repair needs a file outside this scope.
