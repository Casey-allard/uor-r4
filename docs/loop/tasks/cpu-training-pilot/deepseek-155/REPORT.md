# REPORT: cpu-training-pilot, deepseek-155

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `835cfeb66`.

## Status

Decision: **termination-only blocker**. With an oracle BOS inserted at the
reference length, longest-first produced **11 / 453** exact train replies and
**13 / 449** exact dev replies on both artifact seeds. `k2>k1` produced 8/8,
random depth 0/2, and fixed depth 1 and writer produced zero. The longest copy
trajectories contain a small complete-reply signal that the actual decoder
cannot expose without a stop mechanism.

This is a ceiling analysis over sealed traces, not a deployable decoder,
language result or serving claim.

## What did not work

- Attempts 1 and 2 are instrument faults. They computed oracle body
  eligibility and trailing-mismatch histograms but copied the original
  no-stop `exact_reference` field into the oracle reading, leaving the result
  at zero. Both attempts are preserved.
- Attempt 3 had the correct oracle count but omitted report/corpus hash
  binding, per-row full-trace hashes, step-count equality and mean-prefix
  output. Attempt 4 enforced those fields but compared each row against the
  arm-level aggregate hash. Attempt 5 fixed that but omitted report-schema and
  source-arm identity checks. Attempt 6 is the first complete analysis.
- Most copy bodies remain incomplete even with perfect stopping: 442/453 train
  and 436/449 dev longest-first replies are not exact.
- No new model generation ran. This unit cannot determine whether a trainable
  termination signal can recover the 11/13 ceiling.

## What changed

Added `cpu-copy-key-reply-oracle`, a read-only analyzer for the sealed
deepseek-153 reports. For each selected-token trace it:

1. reconstructs every row in generated-step order;
2. rejects missing, duplicate or non-monotonic decisions;
3. truncates at the normalized reference length;
4. inserts the BOS that an oracle terminator would provide;
5. recomputes exact, opening and body-position readings from the truncated
   trace;
6. compares longest-first against `k2>k1`, depth 1, random depth and writer
   using the existing paired helper.

It also reports maximum/common-prefix statistics and the number of replies
with a zero-, one-, two- or three-token tail mismatch.

## Measurements

Both artifact seeds produced identical copy-arm readings because every
deepseek-153 copy decision overrode the writer.

| Panel | Longest exact | `k2>k1` | Depth 1 | Random | Writer |
|---|---:|---:|---:|---:|---:|
| train,train 2711 | **11 / 453** | 8 | 0 | 0 | 0 |
| dev,dev 2712 | **13 / 449** | 8 | 0 | 2 | 0 |

Longest versus `k2>k1` was 3:0 train and 5:0 dev for exact oracle replies.
Against random it was 11:0 train and 11:0 dev; depth 1 and writer were also
0/0 controls. All exact and body-position paired comparisons passed.

Longest-first common-prefix totals were **325 / 4,661** and **451 / 4,614**.
Its maximum common prefix was 41 and 48 tokens. It also led `k2>k1` by 34 and
101 correct body positions.

Resources:

- release build **18.92 s** plus **0.40 s** incremental rebuild;
- four attempt-6 analyses **0.51-0.57 s each**, peak **159,464 KiB**;
- analyzer binary **1,350,696 bytes**, SHA-256
  `a6fffd70b67a220a5b699ca86398ee6caa1ceabf3c2de4b3b1278fa8c937413e`;
- fresh target storage **4,664,680,601 bytes**;
- published evidence **198,944 bytes**;
- admitted cumulative projection: **78,227.677487699 s**.

Attempt-6 raw manifest SHA-256:
`e697083ccd8bcdd7fa7e7bf15ad047bcf04260ce359e57d82c9402bd31104914`.
Published evidence SHA-256:
`a07ac236a5645f98a252ecce0c2b0737263c025203de50e730818f85af520891`.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores; **74** binary tests
  and **3** focused oracle tests passed.
- Scoped Clippy, rustfmt, evidence manifests and `git diff --check` passed.
- Report token/mask identities, each row's full-trace hash, and source
  generated-token counts match before oracle truncation.
- All four sealed reports reconstruct to the expected row counts and trace
  lengths.
- Both artifact seeds and both panels agree on the termination-only outcome.
- No serving, integer, CUDA, float-training or Claude-owned source changed.

## Size

Against base `835cfeb66`, this unit changes **2** code/manifest files
(**+507 / -0**), **8** task, notebook, roadmap, current-state and lab-record
files (**+369 / -1**), and **49** published evidence files
(**+6,466 / -0**): **59 files, +7,342 / -1**. No generated corpus or new model
binary is tracked.

## Skills used

No system-provided Codex skill applies. Project CPU research and evidence
review procedures were used.

## Still open

- Add an actual termination mechanism to the longest-first copy path.
- Measure whether a trainable terminator can recover the 11/13 exact ceiling
  without premature stops.
- Improve the final-token body reconstruction where no exact reply is yet
  reachable.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

If stopping were perfect, longest-first copying would complete 11 train and 13
dev replies, beating its controls. Perfect stopping is therefore the immediate
blocker. The copy body itself still misses most references, so termination and
last-token accuracy are the two next targets.
