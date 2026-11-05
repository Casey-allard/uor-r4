# REPORT: cpu-training-pilot, deepseek-169

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `97c4c5a4a`.

## Status

Complete and merged as **INSTRUMENT_FAULT / CONTROL_INVALID**. The primary
lookahead arm also failed strongly, but the registered target-shuffled control
was vacuous.

## Question

Can candidate-conditioned one-step model lookahead improve exact-copy
arbitration after the no-lookahead learned read failed?

## What did not work

Deepseek-167's row-level learned readout tied longest-32 on one seed and gained
one body position on the other. This unit keeps the readout but scores the
state reached after each exact-copy candidate, directly isolating the value of
model lookahead.

The target-shuffled table produced exactly the same lookahead decisions as the
true table. Lookahead ranks only nearest-neighbor distance, which does not
change when targets are shuffled. The control therefore cannot test learned
target identity. The run correctly stopped as an instrument/control fault.

The primary arm was also much worse even before that control issue: it selected
depth four on most decisions and lost about half the correct body positions of
longest-32.

## What changed

- Added learned and shuffled candidate-lookahead arms.
- Added cloned-forward candidate rollouts for unique depths one through four.
- Added candidate-conditioned next-state readout scoring and threshold
  fallback.
- Added independent tests for lookahead ordering, threshold fallback and
  decision identity.
- Extended the driver to compare lookahead against the previous no-lookahead
  learned read.

## Measurements

- implementation pin: `4b05143cbcd9f9a7e8959fb1610acfc0bf14c841`;
- fresh panel: seed 2764, `dev,dev`, 128 conversations and **488** reachable
  replies per artifact seed;
- decisions: **31,232** per arm and seed;
- lookahead body positions: **194/4,799** and **191/4,799**;
- no-lookahead learned body positions: **393** and **360**;
- longest-32 body positions: **393** and **393**;
- lookahead candidate arbitrations: **27,719** and **27,181**;
- lookahead changed **4,577** and **4,643** decisions versus longest-32;
- target-shuffled lookahead changed **0** decisions;
- training fit remained weak: leave-one-out top-1 **116/125**, coverage
  **929/925 of 1,024**, radius **957/2,048**;
- lookahead throughput was **12.5k-12.9k** generated tokens/s versus
  **61.9k-64.4k** for longest-32;
- all arms had zero exact, BOS-complete and premature-BOS replies;
- fresh-process repeats were byte-identical after timing fields.
- compact evidence:
  [`lookahead-2764/`](../../../labs/cpu-training-pilot-copy-key-reply-2026-11-05/evidence/lookahead-2764/);
  the sealed `outcome.txt` is `INSTRUMENT_FAULT`.
- reviewed head: `8b0089c57297777b14b964285804bf331af383cf`;
- protected merge: `4ec22a118a30365fa1c9f6ce6b2180cd3017f53c`;
- fresh-main CPU library: **305 passed**, 4 expected ignores;
- fresh-main CPU binaries: **83 passed**.

## Checks run at the head

- CPU library: **305 passed**, 4 expected ignores.
- All CPU-training binary test targets passed.
- Focused learned/lookahead tests: **10 passed**.
- Scoped Clippy with `-D warnings`: passed.
- rustfmt and `git diff --check`: passed.
- Fresh output root and seed 2764: absent before compute.
- Local sealed evidence manifest and copied evidence: verified.
- Published evidence sums and no-`.u16`: verified.

## What remains

No delivery work remains for this bounded experiment. Do not retune this
distance-only lookahead on seed 2764. A later unit may use predicted target
identity or multi-token future content with a new unopened panel.

## Size

Against base `97c4c5a4a`, the merged result changes 32 paths (**+1,172 / -10
textual lines**) plus gzipped evidence blobs. No `.u16` corpus is committed.

## Skills used

No system-provided Codex skill applies. Project delivery and research
procedures were used.

## Still open

Useful local chat remains open. This report makes no complete-reply, language
or serving claim.

## Plain summary

Distance-only model lookahead is control-invalid and far worse than
longest-copy. It rewards familiar-looking next states rather than the correct
continuation. This exact scoring rule is closed; future lookahead must use
target content or predicted token identity.
