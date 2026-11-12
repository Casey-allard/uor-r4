# REPORT: infra-loop-harness, claude-292 (bootstrap Codex registration)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904.

## What did not work, first

- **The combined branch could not bootstrap itself through policy.** Exact head
  `796d935ee` contained both the first-registration checker and the Codex registry entry.
  GitHub policy builds the checker from the base commit, which still required the ORDER lab
  to be registered at the base. The pre-change harness rejected the combined branch with
  `lab codex ... is not registered ... at the merge base`.
- **A code-first split also has a test dependency.** The current four-lab tests deliberately
  require Codex to remain unregistered. Moving the checker first without the registry would
  require temporarily weakening or removing that refusal test before immediately restoring
  it in the second PR.
- **The attempted single-branch delivery accumulated unrelated follow-up work.** It reached 36
  commits and mixed registration plumbing with owner-approval parser hardening. This PR keeps
  only the bootstrap state needed to make Codex a registered base-roster lab.

## What changed

- **Registry.** `docs/harness/labs.json` now registers Codex with vendor `OpenAI`, client
  `Codex`, headless command `codex exec`, and open-PR limit 3. The note moves from four to five.
- **Registration plumbing.** Onboarding and receipt hints include Codex, receipt verification
  now treats every registered lab uniformly, and the Codex onboarding lane reflects its current
  role.
- **Tests and records.** Registry tests now expect five labs while retaining the refusal of a
  genuinely unregistered lab. The harness and loop pages state five labs, and the umbrella item
  records Codex's departure and re-registration.

## Measurements

- Final branch size, excluding this task folder: 9 files, +34/−38 = 72 changed lines.
  Additions are source 15, tests 5, and documentation/state 14.
- `jq` reports exactly `antigravity, claude, codex, deepseek, kimi`; Codex fields are
  `OpenAI|Codex|codex exec|3`.

## Checks run at the head

- Full `uor-r4-harness` package test target: pass; library **677 passed, 0 failed, 1 ignored**,
  followed by every integration target and doc test passing.
- `cargo test --release --locked -p uor-r4-harness --test lab_registry_cli`: 5 passed.
- `cargo test --release --locked -p uor-r4-harness --test loop_prcheck`: 50 passed under the
  base checker.
- `rustfmt --edition 2021 --check` on changed Rust files and `git diff --check`: pass.
- `UOR_LAB=claude local/harness-target/release/uor-harness loop pr-check --local`: **PASS**.
  The only advisory is the pre-existing missing FILEMAP entries.

## Skills used

- **uor-research-architect:** identified the base-policy ordering constraint and kept the
  registration work isolated from the scientific checkpoint.
- **working-discipline:** checked the repository state before acting and separated the
  bootstrap from the checker-hardening follow-up.
- **solution-integrity:** treated the policy-order failure as a dependency cycle and chose a
  two-PR sequence that does not change the base-code policy guarantee.

## Still open

- The code-only first-registration validator, owner-approval hardening, remaining parser
  regressions, and documentation corrections follow from fresh `main` after this PR merges.
- The combined candidate branch `codex/infra-loop-harness-1` remains preserved at
  `796d935ee` for selective reuse; it is not part of this PR.

## Plain summary

The project needs Codex in the base roster before a new checker can validate a Codex ORDER.
This PR performs that bootstrap using the already registered Claude lab and the current base
checker. A later PR can then improve first-registration validation without changing CI to run
pull-request code.
