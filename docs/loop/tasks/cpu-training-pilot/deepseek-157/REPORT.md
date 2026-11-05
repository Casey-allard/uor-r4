# REPORT: cpu-training-pilot, deepseek-157

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `ed9b5b9aa`.

## Status

Decision: **REJECT** for the fixed generated-trajectory stop gate. The gate
fit under the 1% training false-stop limit, but it produced **0 exact replies
on both artifact seeds and both train/dev panels**. It sometimes fired at the
reference position, yet never on a complete reference body. This closes the
all-reference-length terminal labeling with eight signed Hamming prototypes,
not geometric termination generally.

## What did not work

- Attempt 1 used the wrong sealed-report schema literal and stopped before
  reading an arm.
- Attempt 2 completed, but its output recorded transient `/dev/fd` paths, so a
  fresh-process identity check was not stable.
- Attempt 3 stopped after report replay was moved to buffered bytes because a
  process substitution had already been consumed.
- Attempt 4 reproduced the result but did not publish per-row fired decisions
  or selected-trace hashes.
- Attempt 5 published those rows but undercounted no-fire trajectory cost.
- The final gate learned only **8/453** training terminal rows on seed 1 and
  **13/453** on seed 2 under the 46-row false-stop cap.
- Training at-reference stops were 7 and 11, but none matched an exact body.
  Dev at-reference stops were 0 and 8; again none were exact.
- Training premature stops were 34 and 31, far above the registered 4/453
  limit. Dev premature stops were 3 and 8; seed 2 failed the 4-row limit.
- True and shuffled gates both produced zero exact dev replies. There is no
  complete-reply gain to compare.

## What changed

Added `cpu-copy-generated-stop`, which:

1. validates the sealed longest-first report, artifact identity and all input
   hashes;
2. replays every selected token through the exact CPU FBS forward;
3. captures 256-bit sign signatures from the writer query at each generated
   decision through the reference horizon;
4. labels only the reference-length decision terminal;
5. fits eight terminal and eight nonterminal prototypes with the existing
   Hamming score and 1% training false-stop cap;
6. evaluates the fitted gate, a class-preserving shuffled-label gate and
   no-gate on the dev trajectories.

The first gate firing emits BOS; otherwise the sealed longest-first token is
fed forward. The decoder never consults the reference during inference.

## Measurements

Training fit:

| Artifact seed | Arm | Terminal stops | False stops / limit | Margin |
|---:|---|---:|---:|---:|
| 1 | true | 8/453 | 38/4,661 (limit 46) | 83 |
| 1 | shuffled | 9/453 | 40/4,661 | 102 |
| 2 | true | 13/453 | 43/4,661 | 100 |
| 2 | shuffled | 11/453 | 41/4,661 | 98 |

Complete-reply simulation:

| Artifact seed | Panel | Oracle exact | True exact | True premature | True at reference |
|---:|---|---:|---:|---:|---:|
| 1 | train | 11 | **0** | 34 | 7 |
| 1 | dev | 13 | **0** | 3 | 0 |
| 2 | train | 11 | **0** | 31 | 11 |
| 2 | dev | 13 | **0** | 8 | 8 |

The shuffled gate also produced zero exact replies on both dev panels.
No-gate produced zero exact replies by construction. Both bar evaluations
failed.

`no_fire` means no firing through the reference horizon. Those trajectories are
charged to the complete 64-token trace for generated-token accounting.

Resources:

- release build **19.71 s**, peak unavailable from that compile; final CLI
  processes peaked at **198,460 KiB**;
- final runs **8.78 / 8.71 / 8.77 s**;
- driver binary **1,993,000 bytes**, SHA-256
  `c9a2f9836c6a7250e370e3c8ac9e9f626a76390e1ec700a2d95382cea89f1d05`;
- fresh target **4,944,779,271 bytes**;
- published evidence **4,750,278 bytes**;
- admitted cumulative projection: **79,427.677487699 s**.

Attempt-6 raw manifest SHA-256:
`49aaf5c7608b20e40f5da44d8ba1b72d7860e1572400c49be79c96250e13085d`.
Published evidence SHA-256:
`fd468c7257b0d04f7007133ab76f2f7cbd8f26c6a1945a28ebb157b54327dfee`.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores; **77** binary tests
  and **3** focused generated-stop tests passed.
- Strict scoped Clippy, rustfmt, D23/D25/D32, report/input hashes and evidence
  manifests passed.
- The seed-1 repeat matched every field after timing removal.
- Every true/shuffled/no-gate row records fired step, stop class, exact status
  and the unchanged selected-token trace hash.
- Both artifact seeds fit on train panel 2711 and evaluated unchanged on dev
  panel 2712.
- No serving, integer runtime, CUDA, float-training or Claude-owned source
  changed.

## Size

Against base `ed9b5b9aa`, this unit changes **5** code/API files
(**+665 / -4**), **8** task, notebook, roadmap, current-state and lab-record
files (**+372 / -1**), and **33** published evidence files
(**+148,222 / -0**): **46 files, +149,259 / -5**.

## Skills used

No system-provided Codex skill applies. Project CPU research and evidence
review procedures were used.

## Still open

- Do not retry terminal labels for every reference-length trajectory. The
  negative shows that reference length alone is not a separable generated
  state for this prototype gate.
- Train termination only over trajectories whose bodies are exact, or use a
  continuation/terminal objective that accounts for copy phrase boundaries.
- Improve terminal-token reconstruction before asking the stop gate to recover
  the 11/13 oracle ceiling.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

The generated-state prototype gate learned a little about where replies end,
but never stopped on an exact reply. It overspent premature stops and lost to
the zero-exact controls. This rejects the all-reference-length labeling with
eight sign prototypes; it does not reject geometric termination.
