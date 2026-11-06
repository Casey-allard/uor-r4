# ORDER: one-model-end-to-end, claude-301

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.

## Goal

Casey's one track (#2028 6675163438, "Yes, one track"): the trunk's no-memory top-1 must beat its TRAIN's unigram and its T2 on the human held-out panel, beyond the paired band, at every seed.

claude-298 (#3796) asked whether batch 64 on D3 narrows the trunk's seed spread and passes D3's T2. Every cell ran out of the card's memory. claude-300 (#3799) added gradient accumulation (`--micro-batch`). This unit runs claude-298's arms again at batch 64 as four micro-batches of 16, under the sheet [`one-model-end-to-end-d3-accum-2026-11-06`](../../../../harness/prep/one-model-end-to-end-d3-accum-2026-11-06.md). The sheet adds three gates: I-e (equivalence on the real recipe), I-m (memory below 90%) and I-f (memory flat across steps).

## Claim check

- **(a) The step.** It is claude-298's question, unanswered, on the "data moves it" branch of claude-297's sheet. claude-300 merged as `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`.
- **(b) What it changes in the model.** It tests whether a less noisy gradient makes the trunk's seeds agree and lifts it past D3's T2.

## Cancel probe (run first)

Run on 6 Nov, before any work. Class: instrument.
1. **No hold.** There is no `local/loop/STOP`, and #2028 has no `loop:pause`.
2. **The pin.** `git log 1f2bd76c..ff5156bc` over the trainer, core, tokenizer, integer and scale-probe paths and `Cargo.toml`/`Cargo.lock` lists only #3799. So claude-297's sealed (n3L) stays the batch-16 control.
3. **The inputs** are where claude-298 read them: D3 and the panels in the store, the seen set and claude-297's grades under the main checkout's `local/claude`.
4. **The caps:** `uor-pod status` shows 0 pods running.

Result: GO.

## Done when

1. The sheet is FIT by a fresh non-author review, and its FIT is posted on #3407.
2. The D67 pre-run review of the job and watch scripts passes in a fresh context, and is posted on #3407 before a pod goes up.
3. The runs, or the gates' NOT_RUNs, are sealed under `results/claude/one-model-end-to-end-d3-accum-2026-11-06/`, each with a `SHA256SUMS` that was read back. The pods are down.
4. The REPORT gives the negatives first, then: each arm against D3's T2 and unigram by claude-298's Bar; the levers; the spreads; I-0 to I-5, I-e, I-m, I-f, I-t, I-s and I-r; and one row in `docs/research/notebook/float-training.md`. It ends `References #3407`.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **The sheet's stop rule.**
- **The cap:** $14, and 6 hours a one-card pod. When a watcher's stop fires on a healthy run, extend it and renew the lease; never cancel it.
- **Casey says "hold".**
- **The laptop's free disk falls under 5 GiB.**

## Files in scope

- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
