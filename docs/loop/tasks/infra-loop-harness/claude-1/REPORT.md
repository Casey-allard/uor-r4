# REPORT: infra-loop-harness, claude-1 (PR A, the loop charter)

Lab: claude. Base: `3429ae024`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **A subagent could not edit the rule files.** The first builder subagent was blocked by the client's permission check when it tried to edit `CHARTER.md`, which is an owner-only rule file, so it stopped without changing anything. The owner then approved, in chat, that the main session make the edits to `CHARTER.md`, `AGENTS.md`, `docs/rules/changes.md` and `docs/integration/DECISIONS.md`. A second subagent wrote the loop page, the roadmap item and the ORDER.
- **Planned commands looked runnable.** The harness rule `harness.documented_commands` failed on the loop page, because it named planned `loop …` subcommands of `uor-harness` as if they already existed. Planned commands are now written as `loop …`, and one sentence says they are future `uor-harness` subcommands. The rule was right, and it was not loosened.
- **Three phrases matched the private kit.** The local copyright check compares every 8-word run in this PR's added lines against the Loop Kit. It matched two table headers and one column list: the decision-table header in `DECISIONS.md` and the HANDOFF column list on the loop page. All three were reworded. The final check finds 0 matches.

## What changed

| File | Lines (+/−) | Class | What and why |
|---|---|---|---|
| `CHARTER.md` | +15 / −5 | rules | §5.1 releases approved in the lab's own chat (D56); §5.3 freeze lifted (D51) and identity by tags (D55); §5.4 labs cannot lift GPU caps (D52); §5.5 cross-vendor review and per-PR ORDER and REPORT (D46); new §5.10 "The loop" (D43–D58); §6 retired items |
| `AGENTS.md` | +7 / −5 | rules | the release and review rules, `antigravity` in the lab list, pointers to the loop and to D0–D58 (70 lines, under the 80-line limit) |
| `docs/rules/changes.md` | +36 | rules | the 20 October entry: 9 interview decisions and 7 design answers, with the owner's words verbatim |
| `docs/integration/DECISIONS.md` | +24 / −1 | rules | the new table "Decisions after the charter (from D43)", rows D43–D58 |
| `docs/loop/README.md` | +233 | docs | the public loop page: what exists and what will be built |
| `docs/roadmap/items/infra-loop-harness.md` | +43 | docs | the umbrella item for the build |
| `docs/roadmap/README.md` | +3 / −2 | generated | re-rendered |
| `docs/loop/tasks/infra-loop-harness/claude-1/` | new | docs | this ORDER and REPORT |

Size is reported, not capped (D48). Everything stayed inside the ORDER's scope. `docs/harness/claims.json` holds the lab's local claim and is not in the PR.

## Measurements

None. This PR changes rules and documentation only, so there is no measurement to report.

## Checks run at the head (output tails)

- `cargo xtask roadmap check`: `roadmap check: 74 items OK`
- `cargo xtask roadmap render --check`: README current
- claim-wording gate: `Claim-wording gate passed`
- `cargo xtask archive-verify --base 3429ae024`: `10802 files, 0 modified, 0 deleted`
- `cargo xtask docs-catalog --check`: `1173 records, 20 lines, 0 paths not found`
- `cargo xtask links`: 0 broken links in the changed files (the repository total is unchanged at about 930)
- `wc -l AGENTS.md`: 70
- local copyright check, 8-word windows against the Loop Kit: 0 matches
- `cargo test -p uor-r4-harness`: run after this REPORT; its result is in the PR body

## Skills used

None of the private kit's skills were invoked. The text was drafted from the owner-approved design and the owner's decisions.

## Still open

- **Review.** A cross-vendor exact-head review of this PR. Kimi, the PM, is the suggested reviewer.
- **The build.** PRs 0 to 12 of the build plan in [docs/loop/README.md](../../../README.md#build-plan), assigned by the D52 team split.
- **Onboarding.** The four other clients, and Ron's five working skills installed locally by the owner.

## Plain summary

- **The charter now holds the loop.** That means one unit of work per session, an order before work, negatives first, a soft 2-hour check-in, subagents inheriting the order, and review by another vendor's lab.
- **Releases have no typed phrase.** A lab asks the owner in its own chat and records the answer word for word.
- **No tooling exists yet.** The loop page says plainly what is still to be built and by which PR.
