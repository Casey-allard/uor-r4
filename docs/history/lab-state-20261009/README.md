# Retired coordination branch `codex/lab-state` (archived 9 October 2026)

Owner direction: `origin/main` is the single source of truth and there are no
standing branches. Claims live on M2 #2030 or M3 #2031; GPU leases use `uor-pod`
and compute board #2037. Do not invoke `lab-runner coord` or recreate the branch.

## Complete, independently restored archive

`lab-state.bundle` contains the complete **1,976-commit** history from genesis
`4a4e47944f320250a4b6aa41513761c68484f166` through final tip
`d9eca2ab1d8240bd854e2ae92bf29f8f1781c765`. Restore into an archive ref:

```sh
git fetch docs/history/lab-state-20261009/lab-state.bundle refs/archive/live:refs/archive/lab-state-final
```

[Verification](final-heartbeats-verification.json) records restoration into a
fresh empty bare repository, complete object verification, commit count, and
exact final commit/tree equality. The bundle requires no prerequisite objects.

The original #2048 bundle was generated from a shallow checkout. Its visible
1,006-commit range did not include all ancestors and a fresh standalone restore
failed. The final retirement check recovered the complete remote history and
replaced that incomplete bundle; its original version remains in Git history.

Two heartbeat commits followed the original archived tip
`669fc6dafb578a338560c446028d00e2e81a5692`. They add the October 9 15:52:32Z and
16:02:37Z events for `opencode-deepseek-20260930` and refresh `state.json`.
The [final patch](../branch-archive/codex_lab-state.final-heartbeats.patch)
preserves their exact tree delta. They contain no source or research changes.

[Plain coordination records](../../../archive/lab-state-20261009/records/)
retain every original directory except heartbeat events, which are in the
bundle. Their last substantive writes were on October 2. The advisory
`Lab delivery evidence` workflow was retired with this branch; the five
required protected-merge checks remain unchanged.

Delete the remote branch only after this archive reaches main, with the final
tip as an exact deletion lease so an unseen new commit cannot be discarded.
