# A check-in surfaces unread notes (2026-10-15)

The durable fix for the missed-CONTEST problem. The per-lab last-seen marker already existed in the
notes module; what did not exist was a reason for a lab to **look**. A lab checks in every five minutes
and never thinks to run the notes command, so a peer note posted mid-task went unseen until somebody
happened to mention it. `heartbeat` now surfaces it at the moment the lab is already reading output.

## What a check-in does now

- Reads the lab notes **cache** only: a check-in must never hang on GitHub, so this adds no network call
  and no latency to the command that runs every five minutes.
- Prints one line per issue with unread notes, following the existing renderer, and names the reader
  (`uor-harness notes`).
- Reports a **floor, never a fact**: a cached read cannot know what it missed, so the line says
  "at least N unread note(s)".
- **Does not advance the marker.** Showing a floor is honest; marking notes seen that the lab may not
  have read is not. The notes command stays the reader that advances, after showing them in full.
- Prints nothing when there is no cache and nothing when nothing is unread: a check-in that always
  printed "0 unread" trains the reader to skip the line it exists to deliver.
- `--no-notes` silences it for a lab mid-run that knows what it is doing.
- JSON output is unchanged on purpose (machine contract); only the human check-in gains the nudge.

## Verification

Two integration tests run the real binary against a scratch live tier with a seeded JSONL note cache:
the check-in surfaces the note, a second check-in still reports it unread (the marker did not move),
`--no-notes` suppresses it, and an empty cache prints nothing extra while still succeeding.

```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Claims housekeeping folded in (PM-authorized)

Released `deepseek/k16-timing-instrument` (merged `f8af41057`) and removed
`kimi/stage-a-sweep-record-20261015` (merged long ago as `c17271e82`). Both branches are gone, so no
proof of merge can be re-derived for them; the PM authorized the removals explicitly rather than leave
the store naming work that is already in main.

## Cost

Laptop CPU only: source, two focused tests, one crate build. No pod, no model compute, no network call
added to the check-in path.
