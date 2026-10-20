# #2475 — Transport publish: bounded retry, and fail safe so a queue entry ejects instead of wedging (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T15:08:24Z
- **Updated:** 2026-10-14T15:12:11Z
- **Closed:** 2026-10-14T15:11:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2475
- **Merged:** 2026-10-14T15:11:53Z
- **Merge commit:** f8cd60b44592a9c084987336c8ad570e4a572549
- **Base / Head:** main / UOR-Foundation:deepseek/transport-retry
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (tooling).

## The wedge, mechanically

`required-transport` publishes the ruleset's five required status names. The step ran under `set -euo pipefail`,
so **one** transient `gh api` failure — the `unexpected end of JSON input` seen twice in production — aborted
the loop and the remaining names were never published. **A required context that is never published does not
fail an entry; it leaves it `AWAITING_CHECKS` for ever.** That is why `--auto` would not move it, why
`--disable-auto` did not dequeue it, and why only `dequeuePullRequest` plus re-arming recovered it. I hit this
twice on my own PRs today before it was mine to fix.

## The change

* Bounded retry with backoff on every publish: three attempts at 3s, 6s, 12s.
* **Fail safe**: if the success publish still cannot be delivered, publish an explicit **failure** check run for
  the same name — the entry is **ejected** and a re-run is visibly needed, instead of waiting on a status that
  will never arrive.
* `set -e` removed (kept `set -uo pipefail`) so one failure cannot abort the loop; the job exits non-zero if any
  name could not be delivered. Both signals: the queue ejects, and the workflow list shows red.

## Verification: the changed path is exercised, not read

The workflow cannot run locally, so the step's `run:` block is **extracted verbatim from the YAML** and driven
through three scenarios with a stubbed `gh` (and a stubbed `sleep`, so backoff costs no wall time):

```
PASS  A publishes 5 successes, exit 0 (5:0)                      <- normal case unchanged
PASS  B recovers: 5 successes + 2 transient errors, exit 0       <- the production incident
PASS  C: 15 failed attempts, exit 1 (15:1)                       <- permanent failure
PASS  C: failure context published for all five (5)              <- the entry ejects
PASS  C: no success context published (0)
```

**My first harness asserted `success=7` for B and `success=15` for C. Both were test bugs, not code bugs** — the
stub logged every attempt, including the ones it then failed. Logging the outcome rather than the arguments
fixed it. Worth stating because a test that counts attempts as successes would have declared the failure path
verified without ever exercising it.

## Checklist

- [x] own worktree, fix with a scenario harness, record at `docs/labs/transport-retry-2026-10-14/README.md` plus a newest-first current-state entry, protected PR
- [x] Checks run at this exact head: `bash local/test-transport.sh` -> **5 passed, 0 failed** (scenarios A/B/C above); `ruby -ryaml` parse of the edited `.github/workflows/ci.yml` -> ok, `required-transport` present; `python3 scripts/harness/cli.py lint check --body-file local/pr-body.md` -> OK, 0 errors
- [x] the changed path is exercised rather than read back
- [x] limitations stated below

### Stated rather than hidden

* **A total API outage still cannot publish anything**, so the entry would still wedge. That is unavoidable from
  inside the job; what this guarantees is a **red job with the reason named**, while the documented transient
  case either recovers or ejects.
* **No Python comparison exists for this unit** — CI YAML with one shell step, no second implementation to diff
  against. That is why the evidence is the scenario harness rather than a byte comparison.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6171531261 — 2026-10-14T15:08:34Z — Casey-allard

## Verification at the exact head (local, this worktree)

```
bash local/test-transport.sh                                   -> 5 passed, 0 failed
ruby -ryaml  (parse the edited .github/workflows/ci.yml)      -> ok, required-transport present
python3 scripts/harness/cli.py lint check --body-file ...     -> OK, 0 errors 0 warnings
```

The scenario harness extracts the step's `run:` block **verbatim from the YAML** and drives it with a stubbed
`gh` and a stubbed `sleep`:

```
PASS  A publishes 5 successes, exit 0 (5:0)                   <- normal case unchanged
PASS  B recovers: 5 successes + 2 transient errors, exit 0    <- the production incident
PASS  C: 15 failed attempts, exit 1 (15:1)                    <- permanent failure
PASS  C: failure context published for all five (5)           <- the entry ejects, it does not wedge
PASS  C: no success context published (0)
```

**One thing a reviewer should know about the evidence:** my first harness reported `success=7` for B and
`success=15` for C, and **both were test bugs** — the stub logged every attempt including the ones it went on to
fail, so failed calls were counted as published successes. Logging the outcome instead of the arguments fixed
it. A test that counts attempts as successes would have declared the failure path verified without ever
exercising it, so I would rather flag it than let the green line stand unexamined.

**And the honest limit:** a total API outage still cannot publish anything, so an entry would still wedge — that
is unavoidable from inside the job. What changes is that the documented transient case now recovers (retry) or
ejects (failure context), and that either way the job is **red with the reason named** rather than quietly
waiting.


---
_Comment/review ids in this file: 6171531261_

