# REPORT: worker-state comparison repair, deepseek-102

> **Superseded by [deepseek-103](../deepseek-103/REPORT.md).** A third fresh
> review found three stale CPU-utilization ranges in the earlier deepseek-100
> summary. The timing implementation and all prior repair points remained
> accepted.

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## What did not work, first

- **The second fresh review found the comparison command still included
  worker-specific state.** It compared the full served-forward audit object,
  including `.path`, so a directory-name difference produced four records. The
  command returned `4` rather than the claimed `1`.
- **The repaired timing summary used only the forward-order audit range.** All
  eight runs cover 34.7-35.6 ms; the narrower 34.9-35.3 ms range was not labeled
  as forward-only.
- **The two findings were record defects, not measurement defects.** The
  implementation, timing boundaries, test and raw worker evidence were accepted
  by the same review.

## What changed

The documented comparison now removes only `served_forward_audit.path` and
retains the decision hash, parity counters, workload counts and all other stable
fields. Executing that exact command now returns `1`.

Every timing summary now labels the all-eight-run audit range
`34.7-35.6 ms`. The deepseek-100 and deepseek-101 reports point to this active
repair record, while the accepted timing implementation is unchanged.

The exact PR diff is 52 files: 5,464 insertions and 17 deletions, including the
complete worker evidence and this repair record.

## Measurements

The state comparison was run from the evidence directory:

```sh
for workers in 1 8 16 32; do
  jq -cS '{panel_baseline,panel_final,accepted:.run.accepted_updates,
    artifact:.final_artifact.sha256,checkpoint:.final_checkpoint.sha256,
    audit:(.served_forward_audit | del(.path))}' "w${workers}.json"
done | sort -u | wc -l
```

Result: `1`.

The maintained worker readings remain:

- 32 workers averaged 3.528 s versus 62.147 s at one worker, a **17.62x**
  reduction;
- training stayed at 283-288 ms;
- the recurrent audit stayed at 34.7-35.6 ms across all eight runs;
- publication/reload stayed at 7.0-16.6 ms;
- two concurrent 16-worker artifact jobs took 6.60 s versus 7.06 s for two
  sequential 32-worker jobs, a **6.5%** wall reduction.

## Checks run at the head

- Documented semantic comparison: returned exactly `1`.
- `git diff --check origin/main...HEAD`: passed.
- Evidence `SHA256SUMS`: passed.
- Focused release test, release package tests, strict Clippy and formatting:
  passed on the accepted implementation head; this repair changes records only.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: passed.

## Skills used

No available Codex skill applied.

## Still open

- A training configuration that clears a registered capability or fresh-reply
  bar.
- Unseen-key transfer, trained float shadow and credentialed complete-reply and
  fresh-phrasing evidence under #3506.
- An automatic independent-job scheduler. This unit records the measured
  16+16 pattern but does not add launcher automation.
- CPU training remains D84 development research and does not replace GPU
  qualification.

## Plain summary

The worker results were already valid, but the command meant to prove that the
four worker counts had identical state accidentally included a directory path.
That made the command report four records even though all mathematical and
artifact fields agreed. The command now excludes only that path and returns one
record. Timing summaries also now state that audit time spans all eight
forward/reverse runs instead of only the forward-order subset.
