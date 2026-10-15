# #2258 — Research harness: make the plan, skills, team and lessons queryable (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T00:54:11Z
- **Updated:** 2026-10-13T00:58:35Z
- **Closed:** 2026-10-13T00:55:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2258
- **Merged:** 2026-10-13T00:55:59Z
- **Merge commit:** 0b9977323513bca80ad36d7640a15d585fc5cfef
- **Base / Head:** main / UOR-Foundation:harness/v1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

No model line is advanced and no count changes: this is tooling, and it runs no model
compute. Milestone reference: #2028 (this serves the #2029-#2032 lab loop).

## What and why

The repo's process layer is strong on paper (the session-goal contract, D21's negative count,
D22's TEST FITNESS gate, the delivery cadence) and almost nothing instruments it. A session
could not say which skill it used, which agents it dispatched, what the milestone headline was
right now, what a run cost, or which past failure applied to the file it was about to edit.

This adds `scripts/harness/` - a standard-library CLI plus the registries it reads in
`docs/harness/` - and wires it into `AGENTS.md`, `CONTRIBUTING.md` and the PR template. It is a
front door to the authorities that already exist, not a new authority: the milestone issues,
`DECISIONS.md`, `docs/mechanisms/` and the dated lab records stay authoritative.

## Result

Decision: **KEEP** (tooling; no model result is claimed).

```
$ python3 -m unittest discover -s tests/harness -t .
Ran 227 tests in 25.049s
OK

$ python3 scripts/harness/cli.py check
harness check: 28 rules, 7 failed (0 error, 6 warn)
# 0 errors; the warnings are the harness reporting real repo state (no heartbeat receipt
# on record for the four in-progress milestones; two orphan branches, below)

$ python3 scripts/harness/cli.py budget scan
running pods: 0 of 4 allowed · cost: $0.00/h · cap: $8.00/h across all labs
# read-only, via scripts/pod/uor-pod status|gpus; degrades to UNAVAILABLE, never fabricates
```

Four findings, none of which were known before the tooling ran:

1. **The registry had drifted from the live prompts.** M2's reopened `protected/discrete
   constructor` line reads `0/3` in the prompt the lab actually pastes, `5/3` on the GitHub
   card and `6/3` in a later comment; the Kimi lab's `evaluation integrity` line existed in no
   registry. `briefing.lines_agree_with_prompts` now computes exactly that comparison, and it
   caught one instance this author introduced and then corrected.
2. **A PR body could satisfy the D22 gate without pre-registering anything**, because the gate
   read the template's own unticked "pre-registration" checkbox as a claim. Markers are now
   read from prose only, so an unticked instruction is not evidence; a body reporting model
   numbers with no citation still fails.
3. **Two merged branches are still standing**, against the 9 October rule:
   `deepseek/invariant-router` (PR #2247 merged) and `bpb-split` (PR #2081 merged). Not touched:
   they belong to other labs and one is checked out in an active worktree.
4. **The owner's rule set is wider than `DECISIONS.md` records.** The 2026-10-12 21:55 freeze
   also bans softplus/sigmoid/tanh and matmul in training or serving, and **contradicts
   `agent-execution-policy.json`, which still states training may use matrix multiplication**.
   That conflict is recorded in `docs/harness/milestones.json` as `OWNER-2026-10-12-NO-MATMUL`
   and needs an owner ruling or a policy update - it is flagged, not silently resolved.

## Checklist

- [x] Checks run at this exact head (commands and outcomes posted on this PR)
- [x] No model compute: nothing here trains, evaluates or scores a model
- [x] Research record: docs/labs/harness-2026-10-12/README.md
- [x] New entry at the top of docs/integration/current-state.md, with **Next:**
- [x] Every document this change makes stale is updated (AGENTS.md, CONTRIBUTING.md and the PR template now point at it)
- [x] STATUS.md / ROADMAP.md / #2028 table: state unchanged - no capability, headline or milestone status moved
- [x] README unchanged: nothing a reader relies on changed
- [x] `python3 scripts/check_claim_wording.py` (run below)
- [ ] After merge: verify on fresh `origin/main`, delete the branch and worktree, `scripts/storage/uor-hygiene --apply`

### Not done in this PR, stated rather than hidden

- `git install-hooks` was **not** run: it resolves to the shared `/Users/casey.allard/uor-r4/.git/hooks`
  and would change commit behaviour for every lab and the owner's checkout. It is opt-in and documented.
- The hook blocks only on `error`; `warn`/`info` print and pass.
- `mistakes/` holds 13 lessons with real evidence paths, but only 4 `block` lessons have a live guard;
  the rest say `none: <reason>` and remain notes.
- `docs/harness/milestones.json` is a **cache** of the GitHub issues and the live prompts, not a sync.
  Nothing here writes to GitHub.
- M5-M8 are absent from the registry; issues #2033-#2036 exist and are untouched since 9 October.
- Line names and counts disagree across four sources for M2, and M3's line name differs between the
  registry and the newest card. The registry follows the live prompt and records the conflict in a
  `note`; reconciling the *authoritative* value needs a lab or owner decision, not a tooling guess.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

