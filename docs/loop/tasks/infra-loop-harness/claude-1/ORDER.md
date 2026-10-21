# ORDER: infra-loop-harness / claude-1

- **Item:** `infra-loop-harness`
- **Lab:** claude
- **Base:** `3429ae024`
- **Task:** PR A. Add CHARTER §5.10 "The loop" and the matching rule, decision and onboarding edits; publish the loop page `docs/loop/README.md`; add the roadmap item `infra-loop-harness` and re-render the roadmap page.

## Scope

- `CHARTER.md`
- `AGENTS.md`
- `docs/rules/changes.md`
- `docs/integration/DECISIONS.md`
- `docs/loop/**`
- `docs/roadmap/items/infra-loop-harness.md`
- `docs/roadmap/README.md`

## Done when

- G1: `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check 2>&1 | head -1` prints `roadmap check: 74 items OK`.
- G2: `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap render --check 2>&1 | tail -1` says the README is current.
- G3: `~/.cargo/bin/cargo run -q --locked -p uor-r4-claim-wording -- . 2>&1 | tail -1` passes.
- G4: the PR's commits touch only paths in the scope above.

## Cancel probe

Does `main` already carry §5.10? Result at base `3429ae024`: it does not; a grep of CHARTER.md finds no §5.10. Class: instrument. The order proceeds.

## Stop list

Stop and ask if any of these happens:

- a change is needed to any file outside the scope;
- a gate still fails after 3 attempts;
- any edit to `archive/` would be needed.
