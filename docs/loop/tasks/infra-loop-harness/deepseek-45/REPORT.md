# REPORT: infra-loop-harness, deepseek-45 (make the hardening item claimable)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item: `infra-loop-harness` (#2904), Setup row 6. Base: `7f0f0710c`.

## What did not work, first

- **The hardening item was unclaimable.** `loop claim loop-render-review-hardening` refused with the shared-file rule: its scope listed `crates/uor-r4-harness/src/loopprcheck.rs`, one of the nine shared registration files that no item scope may list. The item was carded in #3040 with that scope, so the first lab to try the work hit a correction loop.
- **Fixing the item could not be claimed under it either**, because the claim tool reads the item at `main`; so the scope fix rides the claimable umbrella item `infra-loop-harness`, whose scope includes `docs/roadmap/items/*.md`.

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/loop-render-review-hardening.md` | removed `loopprcheck.rs` from `scope:`; the queue-mirror paragraph now says it is a shared registration file, edited as an always-allowed path, with its test in the PR's own test file; everything else unchanged (`status: ready`, same done-when) |
| this folder | ORDER and this REPORT |

## Measurements

- `roadmap check` PASS (90 items; ready 34, held 26, decision 4, done 26; infra 21); `roadmap render --check` current (scope lines are not rendered).
- `loop report --check`, `loop pr-check --local`, `loop gates --full` run at the head; results recorded after the runs.

## Checks run at the head

`roadmap check`, `roadmap render --check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`, and a fresh-context record at this head (docs-only, D46). After merge, `loop claim loop-render-review-hardening` is demonstrated to succeed.

## Skills used

- `scope-guard`: one roadmap metadata line plus one paragraph; no shared file, no code.
- `solution-integrity`: recorded the second-order claim problem instead of silently editing a claimed item.

## Still open

- The hardening work itself (five follow-ups) becomes claimable once this merges.

## Plain summary

- Removed the shared file from the hardening item's scope and documented the shared-file route, so the item can be claimed.