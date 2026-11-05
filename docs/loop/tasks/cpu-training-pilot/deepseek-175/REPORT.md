# REPORT: cpu-training-pilot, deepseek-175

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `1f3b25cd6`.

## Status

Complete as **INSUFFICIENT_TRAINING_DATA**. The candidate ranker remains
untested and no capability conclusion follows.

## Question

Does reference-prefix supervision supply enough exact-copy candidate groups to
train the pairwise ranker rejected for insufficient data by deepseek-173?

## What did not work

Teacher forcing raised the supervised group count from **74 to 85**, still far
below the pre-registered **640** floor. The fit stopped before the perceptron,
seed 2 or reply evaluation.

This rules out generated-trajectory drift as the primary reason this candidate
definition has so little supervision. The exact-copy candidate set itself rarely
contains the reference next token under the fixed depth-one-through-eight
boundary. Seed 2767 is closed for this construction and the floor was not
lowered.

## What changed

- Added explicit longest-32 and reference-prefix training modes.
- Added teacher-forced candidate-group construction from reference prefixes.
- Kept generated inference independent of references and unchanged.
- Added a deterministic reference-path test and stable trajectory names.
- Extended the driver with an explicit `--ranker-trajectory reference` switch.

No serving, CUDA, float inference, termination or Claude-owned source changed.

## Measurements

- implementation pin: `f72d9bc035dc6556cf92562538dda323e539c784`;
- longest-32 supervised groups: **74**;
- reference-prefix supervised groups: **85**;
- required groups: **640**;
- eligibility-attempt wall: **0.71 s**, peak RSS **209,620 KiB**;
- fresh panel: seed 2767, `dev,dev`, 128 conversations, **4,946** response
  tokens and **15,201** total tokens;
- fresh tokens SHA-256:
  `3c09726fd65954361f7ddc4c93a5a03789e95c60aaf990acfc974fb20e58440c`;
- raw local manifest SHA-256:
  `4e02d0dd8deecd5b6315566165a10ebc278eb7b360dbbfb420ef6a3434b6d27c`;
- sealed outcome: `INSUFFICIENT_TRAINING_DATA`.

No reply evaluation ran. The empty seed-1 report and process record are retained
as fail-closed stop evidence.

## Checks run at the head

- CPU library: **315 passed**, 4 expected ignores.
- Focused candidate-ranker tests: **7 passed**.
- `cpu-learned-copy-read` binary test: passed.
- Scoped Clippy with `--no-deps -D warnings`: passed.
- Rustfmt on changed Rust files and `git diff --check`: passed.
- Independent TEST FITNESS and GO: passed before seed 2767.
- Local run manifest and copied evidence: verified.
- Published evidence and no-corpus boundary: verified.

## What remains

Exact-copy depth one through eight does not expose enough supervised choices on
this train panel, whether generated or teacher-forced. A later unit must broaden
the candidate construction or derive labels from a denser objective. Do not
lower this floor or reuse seed 2767.

## Size

The implementation extends the candidate-ranker trajectory mode and driver. No
`.u16` corpus is committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This report makes no complete-reply, language
or serving claim.

## Plain summary

Teacher forcing did not solve the ranker's supervision shortage. It increased
candidate-labelled decisions only from 74 to 85, so the bottleneck is the exact
candidate definition rather than generated drift. The next candidate experiment
needs a denser or broader candidate objective.
