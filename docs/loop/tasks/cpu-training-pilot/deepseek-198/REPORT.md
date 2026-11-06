# REPORT: cpu-training-pilot, deepseek-198

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[full-sequence suffix sheet](../../../harness/prep/cpu-training-pilot-full-sequence-suffix-2026-11-05.md).
Base: `0e3924274`.

## Status

Complete as **REJECT under the combined pre-registered bar**, with a strong
content-positive and termination-limited result.

Prompt-plus-reply suffix keys and same-stream depth arbitration improved both
body positions and exact replies on both artifacts, but the unchanged true
reply-boundary memory stopped too many replies early. The combined result
cannot be promoted. This is not a negative for suffix memory or depth
arbitration.

## What did not work

- Treatment made **85** and **104** premature BOS stops against the registered
  maximum of four. The inherited boundary memory does not follow the changed
  full-sequence content trajectory.
- The first TEST FITNESS review was NOT_FIT. It found that local candidates
  were counted as selected copy even when a global suffix overrode them. The
  implementation now records `used_global` explicitly and counts selected
  sources.
- The first complete-trajectory shuffle test did not prove a global selection.
  The corrected test requires `global_decisions > 0` and a changed shuffled
  trajectory.
- `full_sequence_32_only` was weaker than treatment: **305/330** body positions
  and **10** exact replies. Local priority remains necessary.
- The shallow-depth-4 control performed close to treatment but stopped even
  more prematurely: **108/127** premature stops. Merely replacing shallow
  local context is not the deciding rule.
- The exact oracle was 14 local-body replies. Treatment completed 17 exact
  replies because its global overrides changed the bodies; this is not an
  oracle contradiction, but confirms the new content signal.

## What changed

- Added `derive_full_sequence_suffix_content_memory`, which learns reply-target
  transitions from prompt-plus-reply histories and excludes BOS.
- Added the same-stream depth arbiter:
  `deeper_full_sequence_32`.
- Added shuffled, shallow-local depth-4, global-only, local, writer and oracle
  controls.
- Added explicit selected-source accounting and corpus/range/statistical gates.
- Added focused tests for prompt-conditioned starts, depth priority, tie
  retention, global selection and shuffled trajectory divergence.

No serving, CUDA, float-training, exporter or Claude-owned source changed.

## Measurements

Fresh panel:
`local/deepseek/full-sequence-suffix/2026-11-05/panels/dd-2774`, seed 2774,
`dev,dev`, protocol 2, 128 conversations, 457 reference-reachable replies.

- tokens SHA-256:
  `38e299167f8ee3d6e1e066341cfa73b4b67564d0c0c1e33f52faf3cebb70276f`;
- response-mask SHA-256:
  `01eee77e26d06b58864cc16d6f5600a66df787ef81fceeceffa8f2cbac000c1a`;
- manifest SHA-256:
  `0381a32b430a4911c1f8ebc152390c6c9e207ec6196fea6ef0fbaed4d5f2aec7`.

The table contained **76,157** suffixes, **114,556** transitions, zero BOS
transitions, and **1,848** ambiguous suffixes. The deterministic shuffle
changed **75,011** associations while preserving support and target multisets.

| Seed | Arm | Exact | BOS | Premature | Opening | Body | Global decisions |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | deeper full-sequence | 17 | 218 | 85 | 68 | 847 | 2,439 |
| 1 | shuffled | 0 | 19 | 11 | 45 | 574 | 11 |
| 1 | shallow-local depth-4 | 17 | 270 | 108 | 68 | 810 | 3,216 |
| 1 | global only | 10 | 449 | 33 | 34 | 305 | 11,560 |
| 1 | local longest | 0 | 19 | 11 | 45 | 578 | 0 |
| 1 | writer | 0 | 0 | 0 | 0 | 53 | 0 |
| 2 | deeper full-sequence | 17 | 244 | 104 | 68 | 844 | 2,616 |
| 2 | shuffled | 0 | 19 | 11 | 45 | 574 | 11 |
| 2 | shallow-local depth-4 | 17 | 313 | 127 | 68 | 808 | 3,666 |
| 2 | global only | 10 | 457 | 55 | 46 | 330 | 3,851 |
| 2 | local longest | 0 | 19 | 11 | 34 | 578 | 0 |
| 2 | writer | 0 | 0 | 0 | 0 | 69 | 0 |

All five primary body-position comparisons passed on both artifact seeds.
Treatment also beat local exact replies by the paired rule, **17/17** versus
**0/0**.

Treatment changed **4,862/3,713** reachable steps from local longest and
**4,933/3,756** from the shuffled treatment. The controls were non-vacuous.

Resources:

- TRAIN preflight: **2.76 s**, 658,516 KiB peak RSS;
- seed 1 / seed 2: **2.96 / 2.87 s**;
- seed 1 / seed 2 repeat: **2.96 / 2.89 s**;
- maximum measured process RSS: **693,476 KiB**;
- all non-timing fields and both digests repeated exactly;
- sealed output `SHA256SUMS`:
  `bf8d08a1c65dff4bf3b77ac0068a892d069d04890017d154815738a65ca372b7`.

Compact evidence: [result.json](evidence/result.json).

## Checks run at the head

- first non-author TEST FITNESS: **NOT_FIT**, selected-source accounting
  repaired;
- corrected non-author TEST FITNESS: **FIT**;
- separate D67 pre-run review: **GO**;
- focused arbitration tests: **3 passed**;
- full CPU release suite: **375 passed**, 4 expected ignores (379 selected);
- strict scoped Clippy, formatting and wrapper `bash -n`: passed;
- source/input pins, per-command RSS, preflight activation/non-vacuity and
  fresh per-seed gates: passed;
- all fresh repeats matched after timing removal.

## Reading

This is the first CPU suffix experiment in the sequence with a strong content
result: full prompt-plus-reply memory, gated by same-stream depth and preserving
local ties, increased exact replies from **0 to 17** and body positions from
**578 to 847/844** on both artifacts. The global-only and shallow controls were
worse, so local priority and the deeper-context rule both matter.

Termination is now the blocker. The boundary table was trained on the old local
copy trajectory and was not updated with the full-sequence memory. The next
unit should train or re-key termination on the same full-sequence continuation
state, or apply a reference-independent endpoint action associated with the
selected global source. More content tuning without fixing stopping would
discard this gain.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
D84 CPU research and protected delivery procedures were used.

## Still open

- termination aligned to the new full-sequence content trajectory;
- fresh-phrasing generalization beyond exact-memory recall;
- Claude's independent scale and serving track.

## Plain summary

Full prompt-plus-reply suffix memory materially improved content and exact
replies while retaining local copy at ties, but its stopping policy ended many
replies too early. The content mechanism is worth carrying forward; the
unchanged boundary is not.
