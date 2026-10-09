# Retired coordination branch `codex/lab-state` (archived 9 October 2026)

Owner direction, 9 October 2026: `origin/main` is the single source of truth and
there are no standing branches. `codex/lab-state` was an orphan branch written by
`lab-runner coord`: 1,006 commits, mostly 10-minute lab heartbeats, plus the
coordination records below. Claims now live on the milestone issues of tracker
#2028 and GPU leases on compute board #2037.

- `lab-state.bundle`: the complete branch history (tip `669fc6dafb578a338560c446028d00e2e81a5692`). Restore with
  `git fetch docs/history/lab-state-20261009/lab-state.bundle codex/lab-state:refs/heads/lab-state-archive`.
- [`archive/lab-state-20261009/records/`](../../../archive/lab-state-20261009/records/) (outside `docs/`, so verbatim records are not re-scanned by the claim-wording gate): plain copies of every directory except `events/` (the heartbeats,
  which are only in the bundle): activation, delivery, deployment, handoffs,
  integration, maintenance, recovery, research-reviews, reviews, runner1536 and
  validation. Their last substantive writes were on 2 October 2026.

The `Lab delivery evidence` workflow, which read receipts from this branch and
was advisory (not a required check), was retired with it.
