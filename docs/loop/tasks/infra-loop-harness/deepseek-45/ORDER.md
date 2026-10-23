# ORDER: infra-loop-harness, deepseek-45 (make the hardening item claimable)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `7f0f0710c`.

## Goal

`loop claim loop-render-review-hardening` refuses because the item's scope lists `crates/uor-r4-harness/src/loopprcheck.rs`, one of the nine shared registration files (loop page, "Shared registration files": no item scope may list them). Remove it from the scope and say in the item that the queue-mirror fix edits that shared file as an always-allowed path, with its test in the PR's own test file. No code changes.

## Cancel probe

Class instrument (roadmap metadata only). At the base: the claim above refuses with the shared-file message. Task stands.

## Files in scope

- `docs/roadmap/items/loop-render-review-hardening.md` (scope + wording)
- this task folder

## Done when

- The hardening item's `scope:` no longer lists any of the nine shared registration files, and its text explains that `loopprcheck.rs` is edited as a shared file.
- `roadmap check`, `roadmap render --check`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; a fresh-context record covers this docs-only PR.
- `loop claim loop-render-review-hardening` succeeds from a fresh `main` worktree (demonstrated after merge).

## Stop and ask if

- Any file outside the item file must change.