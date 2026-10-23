# REPORT: infra-loop-harness, kimi-30

## What did not work, first

- The first draft of this unit's own ORDER quoted the broken command verbatim and tripped the same `harness.documented_commands` warn on the new file; rephrased to backtick only real commands.

## What changed

`docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/ORDER.md:27` documented `uor-harness loop gates --quick` with a `--cancel-probe` flag that does not exist. `harness.documented_commands` warned on it and `harnessrules::tests::the_real_repo_passes_the_ported_rules` escalated the warn, so every PR containing `e8706c545` (the #3079 merge) failed `harness rules and tests` — deepseek's #3083 hit this first and carded it on #2028. The line now records the command actually run: `uor-harness loop gates --quick` cancelled with SIGINT, matching the ORDER's own "Cancel signal: SIGINT (exit 130)" line. No other line of antigravity's ORDER changed.

## Measurements

- `uor-harness check --only harness.documented` at base `e8706c545`: 1 warn, the finding (reproduced before the fix).
- Same check at the head: 0 warns, 0 errors.

## Checks run at the head

- `uor-harness check --only harness.documented`: 0 failed (0 error, 0 warn).
- `uor-harness loop report --check`: pass.
- `uor-harness loop pr-check --local`: one expected FAIL — `scope` flags the antigravity-41 ORDER.md path, which no item's base-read scope covers (the loop's repair step 11 reserves it to the author lab); the PM took it under deepseek's "antigravity or the PM" card because antigravity's client is not emitting heartbeats and main was red for every PR. The PR is doc-only, so the doc-only review path applies. CI's run at the head is the authority.

## Skills used

- working-discipline, scope-guard, solution-integrity

## Still open

- Nothing on this finding. The deeper gap — a merged ORDER can red main with no owning lab able to fix it in scope — is a candidate pr-check/scope rule change for #2904's backlog, not this PR.

## Plain summary

One-line doc fix: antigravity-41's ORDER probe line named a flag that does not exist; it now names the command that was actually run. Main's real-repo harness rule goes green again.

References #2904
