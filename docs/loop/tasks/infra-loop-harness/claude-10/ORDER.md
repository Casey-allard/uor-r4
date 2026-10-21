# ORDER: infra-loop-harness, claude-10 (open issues for the 14 loop items)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `9aa7ad275`.

## Goal

The owner wrote in Claude's chat on 21 Oct 2026, about 07:50Z: "Yes: run cargo xtask roadmap sync for the 14 loop items (dry run first, then --apply), and record the new issue numbers in docs/roadmap/issues.json through the same delivery path."

The 14 loop items are `infra-loop-harness` and `infra-loop-pr02` … `pr12`, including 7a, 7b and 7c. They are on `main` (#2898, #2901, #2902, #2903) but have no GitHub issues. A lab cannot claim them as issues yet.

## Cancel probe (run first)

Do these items already have issues?

Result at `9aa7ad275`:
- `docs/roadmap/issues.json` has 73 keys and no `infra-loop` key.
- A GitHub search for `roadmap:id=infra-loop` returns four issues: #2874, #2827, #2826 and #2823. Each carries a different item's marker, so these are loose full-text matches.
- No issue carries an `infra-loop` marker.

The task stands. Class: instrument.

## Steps

1. Dry run: `cargo xtask roadmap sync`, without `--apply`. Read the whole plan. It must create 14 issues, one per loop item, and nothing unexpected.
2. If the plan also updates other items' issues or milestone bodies, stop and list them before applying. The sync covers every item, and the owner asked only for the 14.
3. Apply with `cargo xtask roadmap sync --apply`. It creates the issues, writes `docs/roadmap/issues.json` and re-renders `docs/roadmap/README.md`.
4. Check each new issue: its title, its labels, and that its marker matches its item.

## Files in scope

- `docs/roadmap/issues.json`, written by the sync.
- `docs/roadmap/README.md`, re-rendered by the sync.
- `docs/loop/tasks/infra-loop-harness/claude-10/**`.

Outside the repository, the 14 new issues on GitHub, plus any labels the sync creates.

**Scope widened before the apply.** The dry run planned 14 creates, 0 labels, 0 milestone changes and one update: the body of #2839 (`m1-export-gap-and-toks`). The reason is that #2894 edited that item's file at 23:54Z on 20 Oct, after the issue was last synced at 21:40Z. Asked whether to include it, the owner chose "Include #2839 (Recommended)" on 21 Oct 2026, before the apply. This widening commit is dated 07:56:27Z, and the first issue was created at 07:56:41Z. The option was worded by Claude. So the #2839 body update is in scope.

## Done when

- 14 new issues exist, one per loop item, each with its `roadmap:id` marker.
- `issues.json` maps all 14 ids, and no other id changed.
- `roadmap check` and `roadmap render --check` pass.
- A second dry run plans no changes.
- The claim-wording, links and local copyright checks pass.

## Stop and ask if

- The dry run plans writes beyond the 14 creates and their labels.
- Any file outside scope changes.
