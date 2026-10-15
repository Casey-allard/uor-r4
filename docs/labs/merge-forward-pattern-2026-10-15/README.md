# Merge forward, do not rebase: the pattern that needs no force

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: [#2028 comment 6200456329](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6200456329),
with the hook gap filed as [#2519](https://github.com/UOR-Foundation/uor-r4/issues/2519). No model compute.

## What happened

A routine rebase of a delivery branch was blocked: the rebase rewrites the branch, so the push needs
`--force-with-lease`, and the agent-side pre-command hook rejects that flag outright with no approval path.
The lab's options were "abandon the rebase" or "work around the hook" — and working around a safety hook is
the habit that turns a guard into decoration.

The PM's direction was better than the force push would have been: **merge forward**.

```sh
git fetch origin && git merge origin/main      # shared append files: claims.json union, current-state newest-first
git push                                       # no force, no lease, nothing for a hook to approve
```

The merge queue **squashes**, so the merge commit never reaches `main`'s history, and exact-head review stays
exact: `git diff origin/main..<new-head> -- <delivered paths>` must equal the diff that was reviewed. That is
how the PR this pattern was invented for stayed verifiable.

## What changed

| file | change |
|---|---|
| `docs/harness/pm-rules.md` | a cadence bullet: **base moved under your PR? merge forward, do not rebase**, with the two append-file resolution rules and the verification line |
| `AGENTS.md` | the same rule in the post-merge section of the harness chapter, where a lab reads the delivery sequence |

Both point at [#2519](https://github.com/UOR-Foundation/uor-r4/issues/2519) for the one case merge-forward
does not cover — a stacked child branch whose parent was rewritten — where a lease-protected force push is
the safe operation and the hook has no approval path.

## Why document rather than only code

The hook lives in the agent-side command screening layer, not in this repository: `.git/hooks/pre-push` in
the shared checkout is git-lfs's, and the harness's `git install-hooks` installs `post-checkout`,
`post-commit`, `post-merge` and that same LFS hook. Nothing here refuses a force push, so nothing here can
grant the exception either. What this repository *can* do is make the pattern that avoids the situation the
default one — which is what this PR does — and file the gap where it can be fixed, which #2519 does.

## Verification

- Docs only: no code, no schema, no claims store, no ledger. `uor-harness check` reports no new findings.
- The rule was applied in production before it was written down: Codex's `codex/admission-rank-hinge` was
  carried forward this way, with the shared append files resolved by hand and the reviewed diff re-checked
  against the new head.

## Cost

Laptop CPU only: two document edits. No pod, no GPU, no dependency, no compute.