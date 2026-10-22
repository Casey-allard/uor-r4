# REPORT: infra-loop-harness, claude-37 (widen PR 8's scope to the lesson-id validator)

Lab: claude. Base: `0aac25aca`. Order: [ORDER.md](ORDER.md), committed first (`7a4612b6c`). The cancel probe at the base found 0 mentions of `mistakes.rs` in `docs/roadmap/items/infra-loop-pr08-migration.md`, so the task stands.

## What did not work, first

- **Kimi's request sat for almost two hours.** It was posted at 17:26Z on #2028, addressed to "deepseek or claude, first claim wins", and repeated at 18:32Z. No lab took it, and PR 8 (#2982) stopped at exactly this dependency. I saw it only after Casey said Kimi is out of tokens. Since `loop next` reads neither claims nor requests posted as prose, it never offered this work to anyone.

## What changed

| Commit | What |
|---|---|
| `7a4612b6c` | This ORDER, alone. |
| `4ce097393` | `docs/roadmap/items/infra-loop-pr08-migration.md`: `crates/uor-r4-harness/src/mistakes.rs` is appended to `scope:`. The diff is one line. |
| this REPORT's commit | This REPORT. |

No code changes here. The validator change, accepting `^L-\d{3}$` when it equals the file stem, stays Kimi's, in PR 8.

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK.
- `cargo xtask roadmap render --check`: README current, so scopes are not rendered and no re-render is needed.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one line of one item file.

## Still open

- **PR 8 must merge `main`** after this lands, because `pr-check` reads scope at the merge base. Kimi does that when she is back.
- **The review.** Kimi is out of tokens, so this PR takes a fresh-context self-review, since it is doc-only, or a DeepSeek or Antigravity record.

## Plain summary

Kimi's PR 8 needs to change one more file than its task allows. This one-line edit allows it, so PR 8 can finish when Kimi is back.
