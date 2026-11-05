# REPORT: cpu-training-pilot, deepseek-173

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `1f56a8e7b`.

## Status

Complete and merged as **INSUFFICIENT_TRAINING_DATA**. The candidate ranker is
not evaluated and no capability conclusion follows.

## Question

Can a deterministic pairwise integer ranker improve exact-copy arbitration
using candidate-specific geometric and occurrence features?

## What did not work

The registered fitter found only **74** decisions on train panel 2711 where at
least two unique exact-copy candidates existed and the reference next token was
among them. The pre-registered floor was **640**. It stopped before fitting a
usable ranker, before any model evaluation, and before seed 2.

The first local policy check found that the new ORDER's item link was wrapped
across two lines and therefore not recognized by the ORDER parser. The link was
joined onto the required `Item:` line. This parser repair changed no source,
configuration, input, threshold or result.

This threshold was not lowered after seeing the result. Seed 2766 is closed for
this construction. The result says that longest-32-generated train trajectories
rarely expose a supervised choice among available copy candidates; it is not a
negative for pairwise ranking over a larger or differently generated candidate
set.

## What changed

- Added a deterministic seven-feature candidate representation.
- Added a bounded integer pairwise perceptron with eight passes.
- Added shuffled-target and fixed depth-only controls.
- Added candidate-only full-reply evaluation and exact replay tests.
- Extended the existing learned-copy driver with ranker training, evaluation,
  comparisons and non-vacuity checks.
- Registered and ran the seed-2766 command until the first hard eligibility
  gate stopped it.

No serving, CUDA, float inference, termination or Claude-owned source changed.

## Measurements

- implementation pin: `f54ab142c30330cc9aa0212b715b5010fd57ef26`;
- supervised candidate groups: **74**;
- required groups: **640**;
- fit-attempt wall: **0.95 s**, peak RSS **209,684 KiB**;
- fresh panel: seed 2766, `dev,dev`, 128 conversations, **5,006** response
  tokens and **15,928** total tokens;
- fresh artifact and tokens SHA-256:
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`
  and `cb84ab9105443b5116ae31855b7d188596f2b914f68e2dd39a3356459444c599`;
- raw local manifest SHA-256:
  `13f2a81e9027e57c7bf14c5fb598bf2217738e2ce43c6410891628d5fea499f1`;
- sealed outcome: `INSUFFICIENT_TRAINING_DATA`.
- reviewed head: `528b169cd948cf821417e940ef9d9aaa83f5cbae`;
- protected merge: `cc1fb4d6b36dbfbe50a56a17311efc8906d90fd1`;
- fresh-main CPU library: **313 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**.

The fresh panel was generated only after FIT and GO. No reply evaluation ran.
The one empty seed-1 report and its process record are retained as the
fail-closed stop evidence rather than discarded.

## Checks run at the head

- CPU library: **313 passed**, 4 expected ignores.
- Focused candidate-ranker tests: **5 passed**.
- `cpu-learned-copy-read` binary test: passed.
- Scoped Clippy with `--no-deps -D warnings`: passed.
- Rustfmt on changed Rust files and `git diff --check`: passed.
- Independent TEST FITNESS and GO: passed before seed 2766.
- Local run manifest and copied evidence: verified.
- Published evidence and no-corpus boundary: verified.

## What remains

This closes the 640-group test on longest-32-generated train states only. A
later unit may increase candidate-eligible supervision with a different
trajectory source, but it needs a new seed and a new registration. Do not lower
this unit's group floor or reuse seed 2766.

## Size

Against base `1f56a8e7b`, the merged result changes 24 paths
(**+1,841 / -26 textual lines**) plus compact evidence. No `.u16` corpus is
committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This report makes no complete-reply, language
or serving claim.

## Plain summary

The pairwise ranker was implementable and exactly testable, but the public
longest-32 training trajectory supplied too few decisions with a correct token
among multiple exact-copy candidates. The run stopped before model evaluation.
The next ranker experiment must first create materially more candidate-labelled
states.
