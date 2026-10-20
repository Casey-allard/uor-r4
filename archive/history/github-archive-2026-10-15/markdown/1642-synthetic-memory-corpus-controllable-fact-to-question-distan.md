# #1642 — Synthetic memory corpus: controllable fact-to-question distance (distance curriculum)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T03:31:53Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-10-03T12:15:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1642
- **Merged:** 2026-10-03T12:15:51Z
- **Merge commit:** 49829d05b01710a2bbdc22f6a7b2f7ff7fd89d53
- **Base / Head:** main / UOR-Foundation:codex/distance-curriculum
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Draft. References #1512, #820. One file changed; the experiment it enables is reported below.

Base note: this branch is based on `origin/main` 38377c87 and all checks and byte-identity comparisons were run there. `origin/main` advanced to b482a161 while the experiment ran; `crates/uor-r4-training/src/bin/synthetic-memory-corpus.rs` is unchanged upstream, `gh pr diff` shows one file, and the PR is MERGEABLE with no conflict. If the owner prefers a rebase it is content-neutral for this file.

## What this changes

`crates/uor-r4-training/src/bin/synthetic-memory-corpus.rs` (only file changed, +320/-22).

The generator emitted exactly four turns — statement1, statement2, question, answer — so the asked statement always sat 0 or 1 turns from the question, inside the reach of the width-4 causal convolution. A fit on that corpus can score the asked binding without ever reading the fact out of state, which is why the corpus cannot separate the "state capacity" and "interference under sparse supervision" explanations for the measured value-selection failure.

New arguments:

- `distance=N` — N filler user turns between the statement pair and the question.
- `curriculum=1 [max_distance=N]` — per-document filler count drawn uniformly from `0..=max_distance` (default 8). A mixture rather than a corpus-position ramp, because `dialogue-train` samples episodes by `data_seed`, so position in the corpus is not a curriculum to the optimizer.
- `token_budget=N` — stop after the last document keeping the total at or below N tokens, so a ramped arm can be generated at the same token budget as a flat one.

Filler turns are neutral conversational lines, checked by unit test to carry no relation id and no relation value as a word (the test caught a real leak in the first draft: "soup" is a value of `food`).

## distance=0 is byte-identical to the previous generator

| comparison | result |
| --- | --- |
| changed vs pristine source, rows=64 seed=17 | `diff -r -x attempt.json` empty |
| changed vs pristine source, rows=30000 seed=17 | `diff -r -x attempt.json` empty |
| 76-relation variant, changed vs the pre-change binary, rows=64 seed=17 | `diff -r -x attempt.json` empty |

`tokens.u16`, `response_mask.u8` and `manifest.json` are byte-identical (small-corpus tokens sha `53bc78de…` in both the changed and pristine runs). Only `attempt.json` differs, and only because it records the binary path, pid and claim time. The `distance` manifest block is written only for a non-legacy mode, and `rows_used` only changes when `token_budget` truncates.

## Checks run

- `cargo fmt -p uor-r4-training -- --check` clean.
- `cargo build --release -p uor-r4-training --bin synthetic-memory-corpus`, plus the 76-relation variant used by the experiment.
- `cargo test --release -p uor-r4-training --bin synthetic-memory-corpus` — 4 passed (also on the 76-relation variant): fillers deterministic and exact, fillers carry no table vocabulary, legacy path adds no turns, `distance > 16` / `curriculum=2` / `distance=` with `curriculum=1` / `max_distance=17` refused before any I/O.
- Regenerated corpora at rows=64 and rows=30000, both diffed against the unmodified build.
- Distance probes (rows=512 seed=17): 71 / 86 / 114 / 171 / 285 tokens per document at distance 1/2/4/8/16; `documents_over_context` 0 up to 8 and 512/512 at 16, which is why the manifest reports the count and the run warns.

## Pre-existing gap, reported not patched

This generator does not write the `drops` and `files` metadata the trainer requires. `dialogue_learning.rs:217` refuses any source whose `special_token_occurrences` is not 0, and `mix-chat-corpus.rs:158` copies that key out of the input manifest, so a missing key becomes `null`. Executed on this branch's own 10-relation output:

```
$ mix-chat-corpus out=... inputs=<this generator's output>,<another> labels=...
mix files: [('main-d0', 1547087, 'None'), ...]
$ geometric-stack dialogue-train ... train_manifest=<that mix>/manifest.json
Error: Invalid("a source holds literal special-token text")
```

So `main`'s generator cannot currently produce a trainable store. The 76-relation variant used for the experiment carries an uncommitted `drops`/`files` addition from the wide-relation lane's worktree (`.worktrees/d19-emitter-eval`); this PR deliberately does not duplicate it — it belongs with that work — and the experiment is therefore run on that variant, where the distance=0 arm regenerates the wide arm's corpus byte-for-byte (`tokens_sha256 9bae7202…`, rows=30000 seed=17).

## The one experiment

Both arms: `dialogue-train` from init `chat-dose-continue-1/dialogue/model`, `policy=full_prefix`, `data_seed=20260929`, `steps=2500`, `batch=16`, `lr=0.0002`, `eval_every=250`, `dev_seed=20260930`, `dev_per_source=32`, `checkpoint_every=250`, `max_new_tokens=32`, `RAYON_NUM_THREADS=2`, the surviving binary `geometric-stack-9a80e47a`. Only the training store differs. Both arms completed all 2500 steps (`stopped_early=false`).

| | distance-0 arm | distance-ramped arm |
| --- | --- | --- |
| generator | rows=30000 seed=17 | rows=60000 seed=17 curriculum=1 max_distance=8 token_budget=1717822 |
| documents | 30,000 | 15,058 |
| tokens | 1,717,822 | 1,717,772 |
| fillers/doc | 0 | mean 3.99 (uniform 0..=8) |
| asked→question turns | 0 or 1 | 0..=9 |
| max document | 57 tokens | 198 tokens, 0 over the 256 context |
| mixed store | 84,247,512 tokens | 84,247,462 tokens |
| synthetic share | 2.039% | 2.039% |
| eligible binding episodes | 30,000 synthetic (44,826 total) | 15,058 synthetic (29,884 total) |
| train seconds (2 threads) | 9274 | 6358 |

The experiment's operator is a temporary bin built in the lane worktree: the 76-relation variant above with exactly this patch applied (`crates/uor-r4-training/src/bin/synthetic-memory-corpus-wide.rs`, sha256 `bc9a2be7a533acca1504a89ce5eddc0d…`; binary sha256 `33e124796e7eca2409a2979284160f2d…`). It is not part of this diff; the diff is `main`'s file with the distance feature only.

The control is the reference arm, not merely a similar one:

- distance-0 corpus `tokens_sha256 9bae7202…` = the wide arm's `wide-memory-corpus`;
- mixed store tokens sha `13b6270b…` and mask sha `ce5127c9…` = the 14/120 arm's `mixed-wide-2`;
- `dialogue-census` episode digest `96a1704e…` and population identical to `mixed-wide-2`;
- the distance-0 arm's curve is bit-identical at every 250-step checkpoint, and its final `model.safetensors` sha `dc0951d3bdc89a3c883fdc44…` is identical to `wide-bind-1/model`.

Evaluation: `export model=… out=… calibration=chat-v0/train/tokens.u16 calibration_windows=64 calibration_time=256 damp=0.01`, then `lut-chat artifact=… tokenizer=… requests=<panel>/requests.json max_new_tokens=32`, each run scored against **its own** panel with the lane rule (expected value as a word in the last reply). `lut-chat` is deterministic here (three runs on one LUT produced identical replies); re-exporting a fixed model is deterministic too, and two LUTs whose headers differ only in the embedded source path produced identical panel hits.

### Held-out results

| arm | panel-40 | panel-s5 | panel-s11 | total |
| --- | --- | --- | --- | --- |
| distance-0 (control, = the 14/120 reference arm) | 5/40 | 4/40 | 5/40 | **14/120** |
| distance-ramped | 3/40 | 0/40 | 0/40 | **3/120** |
| baseline dose arm (context) | 0/40 | 0/40 | 0/40 | 0/120 |

Exact two-sided Fisher test on 3/120 vs 14/120: **p = 0.0099**.

Supporting measurements:

- matched near-final pair from retained checkpoints at step 2250: distance-0 **10/120**, distance-ramped **3/120**;
- mid-run read of the ramped arm at step 1250: **7/120** — it peaked mid-training and fell back by 2500;
- looser diagnostic rule (expected value anywhere in any turn): 17/120 vs 11/120, same direction;
- secondary matched probe at 5 filler turns with `max_new_tokens=8`: 3/120 vs 0/120 (this cap cannot separate "cannot retrieve at distance" from "reply truncated before the value").

Training curves: the ramped arm's `train_response_nll` is higher throughout (1.20 vs 1.08 at step 2500, as expected for longer, harder documents carrying half the supervised answer tokens), while its `dev_response_nll` on chat-v0 is slightly *lower* (2.0942 vs 2.1119) — the harder corpus did not degrade the chat development distribution, yet the held-out panel skill got worse.

## Check status

`Validate delivery evidence (status publisher)` fails with `PR must be open, ready and target main` — that gate refuses drafts by design; the PR is intentionally a draft, so it is not marked ready and the gate is expected to stay red. `fmt / clippy / tests / no_std / κ` and `fuzz smoke` report pass (the historical names are queue acknowledgements that execute no tests), the rest skip. Local executed checks are the ones listed above.

## Verdict

**The distance curriculum does not dominate the capacity explanation — it goes the wrong way.** At a matched token budget and identical steps, init, seed and schedule, training on documents that separate the fact from the question produced 3/120 on the disjoint held-out panels against the matched control's 14/120 (p = 0.0099), and the deficit is already present at step 2250, so it is not an end-of-schedule artifact. Forcing the binding to be carried across filler turns did not teach a transferable "answer the asked relation" routine; it reduced the near-distance skill the flat corpus teaches.

This is one negative result in one configuration and it does not refute the published curriculum claim at its own setting (masked multi-query recall, reaching 1.000). Two limits must be stated with it:

1. **The token-budget match is not an episode match.** The ramped arm had 15,058 binding episodes against 30,000, and 148,563 supervised answer tokens against 296,423. Its deficit could be partly "half the practice" rather than "distance". A same-episode-count arm would need a 4.08% synthetic share, which the token-budget instruction excluded — so this experiment cannot fully decompose the two.
2. **The metric is volatile near convergence.** The control itself moved 10/120 (step 2250) to 14/120 (step 2500), and the ramped arm moved 7/120 (1250) to 3/120 (2500). The −11 gap exceeds that observed movement and is significant under an exact test, but a smaller gap from a future arm would not be interpretable without matched steps.

What would change the reading: a same-episode-count ramped arm (4.08% share) to remove the practice confound, or a mixture weighted to the near distances rather than uniform 0..=8.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

