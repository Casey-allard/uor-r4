# ORDER: infra-loop-harness, kimi-30 (hotfix: antigravity-41 ORDER probe line reds main)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item records, as with kimi-22, kimi-27 and kimi-29).
Base: `e8706c545`.

## Goal

The #3079 merge left `docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/ORDER.md:27` documenting a `--cancel-probe` flag on `uor-harness loop gates --quick` — a flag that does not exist. `harness.documented_commands` warns on it and `harnessrules::tests::the_real_repo_passes_the_ported_rules` escalates that to a failure, so every PR containing `e8706c545` is red (found by deepseek on #3083's CI). This unit rewrites the probe line to the command actually run — `uor-harness loop gates --quick`, cancelled with SIGINT — so main is green again. No other line of the ORDER changes; the record's results stay antigravity's.

## Cancel probe (run first)

Class docs/PM (host-only). At `e8706c545`: `uor-harness check --only harness.documented` reports exactly one warn, `antigravity-41/ORDER.md:27` unknown flag `--cancel-probe`. Task stands.

## Files in scope

- `docs/loop/tasks/m3-memory-turn-discriminator-gate/antigravity-41/ORDER.md` (line 27 only)
- this task folder

## Done when

- The probe line names only real commands (`uor-harness loop gates --quick`, SIGINT).
- `uor-harness check --only harness.documented` passes with 0 warns; `loop report --check`, `loop pr-check --local` and `loop gates --full` pass.
- The PR body carries `References #2904`.

## Stop and ask if

- The fix would touch anything beyond the one probe line, or the finding turns out to be a harness false positive (a real `--cancel-probe` flag) rather than a doc error.
