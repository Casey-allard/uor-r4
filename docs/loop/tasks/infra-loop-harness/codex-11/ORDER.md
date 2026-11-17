# ORDER for the three permanent remote branches

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `11c2901d90af122c53069aff62191bc4f147ff9a`. References #2904.

## Goal

Apply the owner's November 17 branch decision to the project rule, branch protection, cleanup, local Studio publisher and documentation workflow. Retain the normal book build and link checks on main pushes and explicit manual production publication from main. The top session owns archive construction, review, delivery and retirement of old remote refs.

## Cancel probe (run first)

The base registry and built-in protection floor still protect gh-pages-preview, the docs workflow pushes to it on main updates, and the Studio helper supports it as a target. Cancel if the permanent branch names and both publishing entry points already meet the new decision.

## Files in scope

Charter §5.5's long-lived-branches sentence, the dated changes log, docs/harness/branches.json, docs/harness/README.md, docs/labs/workspace.md, current publishing instructions, .github/workflows/docs-site.yml and branch-cleanup.yml, scripts/studio/build-and-publish.sh, its regression coverage under scripts/tests, crates/uor-r4-harness/src/{githelp,statusdelivery,statusrules}.rs, and this ORDER and REPORT. Inspect any additional actual writer or deleter before changing its exact path.

## Done when

On a main push, the docs workflow builds and checks the book with read-only permissions. Only a manual run from main may update the existing gh-pages, preserving Studio. The Studio helper rejects archive and retired preview as destinations. Both cleanup jobs and the harness protect archive. Check workflow conditions, simulate publication with a local fixture, test protection even with a missing registry, and rerun focused existing checks. No workflow dispatch, remote ref mutation or production publication in this unit.

## Owner approval

Casey chose "main, gh-pages, and a new archive branch; retire the preview and update its publishing workflow". This authorizes the branch sentence in CHARTER.md §5.5, its log in docs/rules/changes.md, the registry docs/harness/branches.json, and .github/** for the named docs publishing and branch cleanup workflows. Recorded at 2026-11-17T22:55:45Z.

## Stop and decide if

Escalate any requested charter change beyond §5.5's branch names, change to research rules, conflict with another lab's open implementation, or need for a live publishing action.
