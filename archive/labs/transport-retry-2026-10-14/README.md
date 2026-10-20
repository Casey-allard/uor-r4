# The transport publish failure wedged the queue; it now ejects it

**Date:** 2026-10-14 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** PM queue item 3 (new, from the merge-queue incidents)

## The wedge, mechanically

`required-transport` publishes the ruleset's five required status names as completed check runs. The step ran
under `set -euo pipefail`, so **one** transient `gh api` failure — the `unexpected end of JSON input` seen
twice in production — aborted the loop and the **remaining names were never published**.

**A required context that is never published does not fail the entry; it leaves it `AWAITING_CHECKS` for
ever.** That is the wedge, and it is why `gh pr merge --auto` would not move it, why `--disable-auto` did not
dequeue it, and why the only recovery was `dequeuePullRequest` followed by re-arming. I hit this twice today
on my own PRs before it was mine to fix.

## The change

* **Bounded retry with backoff** around every publish: three attempts at 3s, 6s, 12s.
* **Fail safe on exhaustion**: if the success publish still cannot be delivered, an explicit **failure** check
  run is published for the same name. The entry is then **ejected** and a re-run is visibly needed, instead of
  waiting on a status that will never arrive.
* `set -e` is gone (kept `set -uo pipefail`) precisely so one failure cannot abort the loop, and the job exits
  non-zero at the end if any name could not be delivered — both signals, so a transport failure shows up in the
  workflow list as well as in the queue.

## Verified by exercising the changed path, not by reading it

The workflow cannot run locally, so the step's `run:` block is **extracted verbatim from the YAML** and driven
through three scenarios with a stubbed `gh` (and a stubbed `sleep`, so the backoff costs no wall time):

| scenario | outcome |
|---|---|
| A — no failures (normal) | 5 successes, exit 0 |
| B — two transient failures | **recovers**, 5 successes, exit 0 |
| C — success publish permanently broken | 15 attempts, **5 failure contexts published**, 0 success contexts, exit 1 |

```
PASS  A publishes 5 successes, exit 0 (5:0)
PASS  B recovers: 5 successes + 2 transient errors, exit 0 (5:2:0)
PASS  C: 15 failed attempts, exit 1 (15:1)
PASS  C: failure context published for all five (5)
PASS  C: no success context published (0)
```

**My first version of this harness asserted `success=7` for scenario B and `success=15` for C — and both were
test bugs, not code bugs.** The stub logged every *attempt* including the ones it then failed, so the counts
included calls that errored. Logging the outcome (`OK`/`ERR`) rather than the arguments made the counts mean
what they say. **A test that counts attempts as successes would have declared the failure path verified without
exercising it.**

## Stated rather than hidden

* **A total API outage still cannot publish anything**, so the entry would still wedge. That case is
  unavoidable from inside the job — what the change guarantees is that the job is **red** and the reason is
  named, and that the documented transient case (partial failure) now either recovers or ejects.
* **No Python comparison exists for this unit.** It is CI YAML with one shell step; there is no second
  implementation to diff against, which is why the verification is the scenario harness above.
* The retry is on the publish calls only. Nothing else in the job was changed.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 1 (PATH install refresh), then item 2 (unread-note surfacing).
