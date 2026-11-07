# ORDER: one-model-end-to-end, deepseek-22 (reply-masked SFT data)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `b80933c4b`.

## Goal

Prepare a deterministic, leak-checked reply-masked SFT store for a short fine-tune that
could teach the existing (b64L) trunk to stop. The loss must primarily cover assistant
replies and reply EOS, not full Wiki continuations.

This unit prepares data and a reproducible recipe. It does not launch training or spend
GPU. Generated corpora remain in ignored `local/deepseek/` scratch and are bound by hashes,
not committed.

## Cancel probe

Run first:

1. Locate the authenticated HF store and current D3/OASST2/recall inputs without printing
   credentials.
2. Locate `m2-reply512` inputs, labels and manifest by content hash `b9661606…`; if absent,
   record every checked path and mark it NOT_RUN.
3. Verify the registered corpus writers implement reply/EOS masks at this base. The historical
   `dialogue-recall-corpus` source is not a Cargo target and imports modules excluded from the
   current graph; if it cannot be built without restoring historical code, record that negative
   and use the registered `m-world-recall-corpus` writer as the lawful recall source instead.
4. Check no open claim already owns the same SFT corpus.

## Required preparation

- Generate recall conversations with `train_on=answers`, a fixed seed, context compatible
  with the trainer's 192-token setting, deterministic output and full leak checks.
- Generate an OASST2-only reply-masked continuation store from the raw source.
- Mix recall and OASST2 so assistant content and EOS dominate SFT loss; keep Wiki material
  for pretraining rather than this SFT loss.
- Refuse panel/TRAIN overlap, malformed masks, missing EOS, non-deterministic output and
  context overflow.
- Record commands, input identities, output paths, sizes, hashes, EOS rates, mask rates and
  unavailable items in a machine-readable manifest and REPORT.

## Files in scope

- `crates/uor-r4-tokenizer/**` only if a concrete existing command cannot emit required
  reply masks
- `docs/labs/one-model-end-to-end/instrument/**` only for a missing deterministic mixer or
  verifier
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/deepseek-22/**`
- `docs/research/notebook/float-training.md`

## Done when

1. All available inputs are identified by source, size and hash; missing content is recorded
   without substitution.
2. The reply-masked store is reproduced from committed commands/tools and passes mask,
   EOS-exposure, determinism and leak checks.
3. A short-SFT sheet starts from a sealed (b64L) checkpoint, runs seed 1 only first,
   evaluates at bounded intervals with `served-reply-panel`, includes a held-out
   continuation guard, and has explicit stop/scale rules.
4. The sheet is ready for fresh FIT and D67 pre-run review; no training is launched here.
5. Builds/tests, `git diff --check`, loop report and required local checks pass.
6. REVIEW receives a fresh-context adversarial exact-head review before delivery.

## Stop and ask if

Ask means decide, record and continue. Do not commit corpora, print credentials, touch
Claude's pods, or launch a run. No softmax, matmul, transformer fallback or GPU-side
substitution.
