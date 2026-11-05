# REPORT: cpu-training-pilot, deepseek-153

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `e06635e3a`.

## Status

Decision: **selection-surrogate positive but termination-limited**. On two
trained artifact seeds and two new m-world-v2 panels, longest-first exact copy
beat `k2>k1`, fixed depth 1, random depth and the unmodified writer on correct
reference body positions. It did not produce a BOS-complete or exact reply.
This selects depth preservation for a trained copy read only when it is
combined with the writer's termination path. It is not a complete-reply,
language or serving result.

## What did not work

- Attempt 1 stopped before producing any comparison result. Its new panels
  contained assistant runs of 49 and 46 tokens under the registered 32-token
  cap. The first result file is empty; this is an input/config instrument
  fault, not a mechanism negative.
- No arm produced an exact-reference reply in any of the four artifact/panel
  cells.
- No arm produced a BOS-complete or non-premature BOS stop. Every one of the
  453 train-panel replies and 449 dev-panel replies hit the 64-token cap in
  every arm.
- The copy selector used a copy decision at every generated position and
  never fell back to the writer. Since copy candidates can emit only tokens
  already in the prompt/generated stream and prompt extraction excludes BOS,
  the override cannot itself emit the stop sentinel.
- Copy-arm body and reply trajectories were identical across the two trained
  artifact seeds. Only the writer control and writer-dependent decision hashes
  changed. This policy bypasses the learned writer rather than combining copy
  content with learned termination.

## What changed

Added an isolated CPU complete-reply evaluator with five fixed arms:

- longest exact suffix from depth four to one;
- `k2>k1`;
- fixed depth one;
- deterministic random available depth;
- the unmodified artifact writer.

The selector sees at most the last 192 prompt/generated tokens, chooses the
latest exact candidate inside that slice, replaces only the next writer token,
and feeds the selected token through the existing exact CPU forward, memory and
point-key path. The shared stopping contract remains unchanged.

The evaluator records exact/BOS completion, opening correctness, correct body
positions, copy/fallback decisions, selected depth, per-reply hashes, decision
hashes and throughput. Paired comparisons use only reference-reachable rows.
The run wrapper requires at least 200 reachable rows, non-vacuous random depth
and the expected artifact hash for every artifact/panel cell.

## Measurements

Both panels reached every recovered reply:

| Panel, seed | Replies | Longest body | `k2>k1` | Depth 1 | Random | Writer |
|---|---:|---:|---:|---:|---:|---:|
| train,train 2711 | 453 | **409 / 4,661** | 376 | 158 | 216 | 60 / 59 |
| dev,dev 2712 | 449 | **556 / 4,614** | 458 | 192 | 307 | 55 / 49 |

Each row was run against both public artifact seeds. The two writer values in
each row order seed 1 / seed 2. Copy arms were identical across artifact seeds.

Longest-first versus controls:

| Panel | `k2>k1` | Depth 1 | Random | Writer |
|---|---:|---:|---:|---:|
| train | 4:2 / threshold 3 | 14:7 / 6 | 12:3 / 4 | 66:54 / 15 and 63:52 / 15 |
| dev | 6:1 / 2 | 18:8 / 6 | 14:3 / 4 | 89:47 / 14 and 83:34 / 12 |

Every body-position comparison passed on both artifact seeds and both panels.
Longest-first also beat the writer on opening correctness, **20** versus
**0/15** on train and **25** versus **0/17** on dev. Copy-depth controls tied
longest on opening correctness, so those paired opening comparisons failed.

All arms still had:

- **0** exact-reference replies;
- **0** BOS-complete replies;
- **0** non-premature BOS stops;
- **0** writer fallbacks in the copy arms;
- exactly 64 generated tokens per reply.

Random depth changed **3,382** steps across **70** train replies and **4,104**
steps across **86** dev replies. The fresh-process seed-1 dev repeat matched
all fields after timing removal.

Resources:

- attempt 1 measured **17.47 s** before stopping;
- attempt 2 measured **12.20 s**;
- release build **0.06 s**, peak **54,132 KiB**;
- generator build **0.06 s**, peak **44,180 KiB**;
- panel generation **0.51 s + 0.59 s**;
- four artifact/panel runs and the repeat **2.19-2.20 s each**, peak
  **370,728 KiB**, about 2,822-2,859% reported CPU utilization;
- longest-first generation throughput **69,487-70,403 tokens/s**;
- driver binary **2,135,808 bytes**, SHA-256
  `fe68aaa9b723477a0848177d11bfb6fb7794703832b6d7fc2df2a2ff0d7945f1`;
- fresh target storage **1,500,952,330 bytes**;
- raw attempt-2 evidence **3,834,190 bytes**; published evidence **69,764
  bytes**;
- admitted cumulative projection: **77,327.677487699 s**.

Attempt-1 raw `RAW-SHA256SUMS` SHA-256 is
`4d926efe870914e77655891c606ea372f8891a841aa7ae42daf1a27be75dc4a1`.
Attempt-2 raw `RAW-SHA256SUMS` SHA-256 is
`716d6f96900b71ebb67d14e1c5f15d10b40864543757e6183dcc2fdc952f4d58`.
Published evidence verifies with `PUBLISHED-SHA256SUMS`, SHA-256
`510582112b39fa0b383ddf64108c5240798b7724addc2c17b3dc1af598553755`.

## Checks run at the head

- Attempt 1 is sealed and classified only as an instrument/config stop.
- Fresh FIT and independent GO preceded attempt 2 on unopened seeds 2711 and
  2712.
- All input, artifact and generator hashes matched; all 902 recovered examples
  across the two panels were reference-reachable.
- Random non-vacuity, source pins, RSS caps, storage caps and executable
  fail-closed checks passed.
- The fresh-process repeat matched every arm, reading, comparison, reply hash
  and decision hash after timing fields were removed.
- **286** CPU library tests passed with 4 expected ignores; **71** binary tests
  and **11** focused copy-reply tests passed; strict scoped Clippy, rustfmt,
  recipe `bash -n`, `git diff --check` and evidence manifests passed.
- No float-training, integer-serving, CUDA or Claude-owned source changed.

## Size

Against base `e06635e3a`, this unit changes **4** code/manifest files
(**+1,036 / -0**), **11** ORDER, preregistration, report, ledger, notebook,
roadmap, current-state and lab-record files (**+1,194 / -1**), and **29**
published evidence files (**+1,859 / -0**): **44 files, +4,089 / -1**. No
generated corpus or binary is tracked.

## Independent result audit

A fresh-context result audit reproduced the checksum manifests, all paired
comparisons and the repeat. It confirmed that all decisions were copy decisions
with zero writer fallbacks, that the prompt-derived copy domain excludes BOS,
and that the registered result is termination-limited rather than an
instrument failure. It also confirmed identical copy trajectories across
artifact seeds and the 69.5k-70.4k tokens/s range.

## Skills used

No system-provided Codex skill applies. Project FIT, GO and independent result
audit procedures were used.

## Still open

- Preserve longest-first depth-three/four candidates in the next trained copy
  read, but combine them with a writer or explicit geometric BOS/termination
  path rather than replacing the writer at every step.
- Test whether a termination gate can fire after a copy-selected body without
  regressing the replicated body-position gain.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

Longest-first exact copying chose markedly more correct body tokens than every
control, but it never stopped: it copied forever from the prompt because BOS
was not in the copyable stream. Keep the depth-four copy signal and pair it
with termination; do not use exact copy as the whole decoder.
