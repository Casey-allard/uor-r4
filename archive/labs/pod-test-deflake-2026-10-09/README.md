# Two timing-dependent checks in the pod dry-run suite (9 October 2026, claude)

Test-only fix in `scripts/pod/tests/uor-pod-dryrun.sh`; `uor-pod` and its bootstrap are unchanged. References #2037.

## Causes
- **`reap: deletes idle unleased B`** expects the exact text `idle 30 min`. Pod B's fake GPU was marked idle since `NOW - 1800`, with `NOW` read once at the start of the suite. The reaper reports `31 min` or more once the suite takes over a minute to reach the check. Under CPU load a full run takes about 150 s, so the check failed every time.
- **`lock held, fresh heartbeat without progress`** waits for the "no visible build progress" exit, with a stale limit of 2 s. The holder file was written once and never touched again. Two exits race: no progress fires 2 s after the waiter's first poll; a stale heartbeat fires once the file is more than 2 s old. If the waiter took more than about a second to start, the stale exit won.

## Fix
- Just before the reap check, the test restarts B's idle clock at `now - 1800`.
- The lock test dates the holder file an hour ahead with an explicit UTC time (`touch -d …Z`). Only the no-progress exit can fire. Checked in UTC and with `TZ=Asia/Tokyo`: the mtime is 3,599 s ahead either way.

## Evidence
On the owner's laptop, with all 8 cores pinned by `yes` processes:
- the fixed file passed 246/246 in **10 of 10** runs (1,518 s in total);
- the main file failed the reap check in **6 of 6** runs.

The lock race did not appear in those 6 runs. That fix rests on the code path (`wait_for_other_build` in `uor-pod-bootstrap.sh`).
