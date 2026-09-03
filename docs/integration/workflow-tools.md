# Installed skill and daily-workflow assessment

Read-only inspection; no installations or configuration changes. The architecture skill was used to organize the proposed boundary. Other skills below were inspected for future use, not activated to run tests, create issues or make changes.

## Useful installed roles

| Skill | Appropriate use | Fit for this project |
|---|---|---|
| `engineering:architecture` | One architecture decision record: context, options, decision, consequences | Good default for the web/Rust/provider boundary. Keep it a short decision, not a parallel backlog. Path: `/Users/casey.allard/.codex/plugins/cache/claude-cowork/engineering/1.2.0/skills/architecture/SKILL.md`. |
| `design:design-critique` | Review an actual screenshot or live flow for hierarchy, usability and consistency | Useful for conversation/composer/editor flow. Its accessibility prompts are checks to perform, not evidence already obtained. Don't claim contrast or keyboard access from a screenshot. Path: `/Users/casey.allard/.codex/plugins/cache/claude-cowork/design/1.2.0/skills/design-critique/SKILL.md`. |
| `product-management:write-spec` | Convert the chosen flow into goals, requirements and acceptance criteria | Useful for one product epic and its small delivery slices. Existing user direction is enough to draft; generic question prompts should not force repeat approvals. Path: `/Users/casey.allard/.codex/plugins/cache/claude-cowork/product-management/1.2.0/skills/write-spec/SKILL.md`. |
| `engineering:debug` | Reproduce, isolate and fix a particular behavior mismatch | Good for model selection, cache status, dropped context, save failures and request cancellation. Freeze the short reproducer and stop expanding when the causal defect is found. Path: `/Users/casey.allard/.codex/plugins/cache/claude-cowork/engineering/1.2.0/skills/debug/SKILL.md`. |
| `engineering:code-review` | Review the exact changed paths for correctness and meaningful risk | Good independent review for provider boundaries, credentials, renderer/preview isolation, state transitions and file/Git mutations. Do not turn each UI tweak into a broad security audit. Path: `/Users/casey.allard/.codex/plugins/cache/claude-cowork/engineering/1.2.0/skills/code-review/SKILL.md`. |
| `browser:control-in-app-browser` | Observe the actual UI, invoke controls, capture visible state | Use only for named product behavior checks; use GitHub API/CLI for semantic tracker operations. The installed browser skill provides the correct connected browser runtime; do not install another automation stack merely to drive the same browser. Path: `/Users/casey.allard/.codex/plugins/cache/openai-bundled/browser/26.831.21537/skills/control-in-app-browser/SKILL.md`. |
| Remembered `uor-r4-studio-evaluation` | Distinguish model answers from ready/cache/editor/preview surface checks | Useful tailored checklist: exact revision/model/cache state, one factual and one unknown-answer probe for a chat provider, stop on a reproducible basic failure. Those probes must be adapted to the operation actually claimed; they are not appropriate gates for #1079's constrained binding task. Path: `/Users/casey.allard/.codex/memories/skills/uor-r4-studio-evaluation/SKILL.md`. |
| Remembered `uor-r4-live-issue-execution` | Native eligibility, one issue, isolated worktree, named staging, protected delivery, truthful closure | Strong process skeleton, but its historical local-testing instructions are subordinate to current AGENTS and user scope. It explicitly says not to invoke for read-only work. Path: `/Users/casey.allard/.codex/memories/skills/uor-r4-live-issue-execution/SKILL.md`. |

## Superpowers assessment

The installed `superpowers@openai-curated-remote` package has useful planning, worktree, debugging and review patterns. Its `using-superpowers` entry explicitly tells dispatched subagents to ignore that global auto-invocation skill. This audit is a dispatched read-only task, so it was inspected, not applied as an implementation workflow.

The following details matter before using it as a daily executor:

- `brainstorming` requires an explicit design approval before every implementation path. Apply the real session's existing authorization; do not repeatedly ask for an already-approved bounded issue. This user wants concrete reviewable work rather than recurrent permission gates.
- `writing-plans` defaults to TDD and frequent commits and saves to `docs/superpowers/plans/`. Its helpful part is exact file/behavior/acceptance steps. Use the user's selected plan location and test policy instead of proliferating competing docs or test cycles.
- `using-git-worktrees` correctly starts by detecting existing isolation and preferring native workspace tools, but defaults to baseline suites and says to continue in the current directory after a sandbox failure. That fallback must not override this repo's preservation/isolation requirement or silently use a dirty research checkout.
- `systematic-debugging` is useful for tracing one failure before changing code; it should not create broad unrelated testing after a sufficient named reproducer.
- `requesting-code-review` can dispatch a narrowly scoped independent reviewer with exact revision/diff/acceptance criteria. This is preferable to having the coordinator repeatedly inspect unchanged history.
- `verification-before-completion` reinforces evidence-based claims. For this repository, verification can be the authorized scoped product check and observed result; dormant broad QA or research experiments should not be activated merely to satisfy a generic template.
- `executing-plans` is suitable for a frozen accepted plan, but the repository's actual issue status, parent/blocker edges and user authorization remain authoritative over a stale plan/checklist.

Paths share `/Users/casey.allard/.codex/plugins/cache/openai-curated-remote/superpowers/6.3.0/skills/<name>/SKILL.md`. No additional architecture/spec/debug/review/browser skill install is needed for the proposed workflow. Root owns any requested installation decisions.

## Proposed daily GitHub issue/worktree loop

1. Read current origin/main and AGENTS; refresh native issue parent/blocker edges, assignees, branch/PR/queue and the exact accepted plan slice. Do not use old memory or a checkbox alone as eligibility.
2. Select one concrete deliverable with a user-visible result. Give it an owner and an issue branch/worktree; preserve existing research artifacts and unrelated work.
3. Freeze a small contract: behavior to change, exact provider/artifact or data seam, named acceptance question, negative outcome, and scope boundary. For scientific execution keep the separately declared run/reveal rules; UI work must not open new research populations or silently fit a model.
4. Implement the bounded change. Delegate independent files or a short source review when it helps; do not have multiple agents broaden testing or retread the whole repo.
5. Activate only the named product/release verification. Capture actual UI/API outputs when that behavior changed. Report `NOT_RUN` and `UNAVAILABLE` accurately. Current AGENTS says automatic QA is disabled; the historical five queue statuses are no-QA acknowledgements, not quality evidence.
6. Update the concrete record and claims, stage named files, create the focused PR and use the required protected delivery mechanism. Verify the actual merged revision/tree and native issue closure; preserve parent blockers until their own goal is fulfilled.
7. Stop the issue at its DoD. The next action comes from the measured outcome and eligibility, not an automatic cascade into broader features, training or QA.

## Future automation shape

Start with a read-only heartbeat on the current work thread only when the user requests recurring work: watch current main, the selected issue/PR/queue and named blockers. It should stay quiet on unchanged state and notify only for a completed merge, failing named gate, actionable blocker change or needed user decision. It should not create new issues, auto-select broad work, rerun scientific inference, install tools, or start model fits without the corresponding authorization.

If the user explicitly authorizes repeated implementation, the automation prompt should name the accepted plan/issue range, eligibility rule, one-issue limit, allowed verification, release/merge boundary and stop conditions. Use Codex automation tools rather than raw cron or an invented external scheduler. Keep notification preferences in the automation configuration, not repeated inside the task prose.

No automation was created by this audit.
