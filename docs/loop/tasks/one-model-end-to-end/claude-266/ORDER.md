# ORDER: one-model-end-to-end, claude-266 (the one model trained as served at full length, on the unsnapped contract)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `59f7770ae` (main with claude-265 merged). This ORDER is hand-written under the umbrella reading (cancel probe 2).

## Goal

Train the one model as served at full length, export it raw, serve it on the laptop, and score it end to end, with tokens per second.

claude-263's part B sheet (`one-model-end-to-end-full-served-2026-11-03.md`, FIT round 5, PRE-RUN PASS) is the plan: (b) and (k2) for 8,192 steps at seeds 1–3, one 5090 pod per seed, arms in turn. It never ran, because its door, I-3 at 256 steps, failed.

claude-265 (#3600) changed what "trained as served" means. The served bridge now trains and scores on the unsnapped D11 contract, and the export is raw. At 256 steps, I-3 then missed on 32 and 14 of 310 (bar 8), down from 100 and 131. Scored twice on one host, the float scorer gave identical output.

**This unit:**
- re-pins part B's sheet to `59f7770ae2d936cfb3ac7337cd041cb3f25ead0f` and to the raw export (`--raw-gates --raw-transition`), in a new sheet, `docs/harness/prep/one-model-end-to-end-full-unsnapped-2026-11-03.md`;
- changes what an I-3 miss holds (the decision card below);
- trains, scores and snapshots both arms at three seeds;
- exports each raw, reads I-3, I-5 and I-6 on the laptop, grades R1–R5, and reads R6 (speed) and R7 (transcripts).

**Plan step (the claim check):** order step 4, one model trained, exported, served and scored end to end, with tok/s.

**What it changes in the model:** the model itself. This unit trains the full-length model that step 4 names, on the contract it is served on.

**§4.4.** This is the fourth unit on the I-3 line, after three results that did not pass it (claude-263, claude-264, claude-265). Under §4.4's pivot note (stop, or one decisive run), it is the one decisive run for step 4. No further 256-step I-3 unit runs until the served-contract readout is settled.

## The decision card (my decision review, logged on #2028)

**The change.** Part B's Bar holds every reading unread if any export misses I-3. claude-266's sheet keeps the rule that nothing closes, nothing is named and no status moves while I-3 fails (§4.1, §4.2, L-007). It drops the rest of the hold:
- R1–R5 are graded and reported, with numbers and per-seed verdicts, as results of the float model, marked "float; served parity open". Both arms go through the same float forward, so (b) against (k2) is a fair float comparison. It says nothing yet about the served models, and no served ranking is inferred from it.
- R6 and R7 are read on each seed-1 raw export, labelled with its I-3 count, as the sheet words them.
- Part B's float-side and integer-side branches for the next unit stay as they were.

This returns R1–R5 toward claude-261's own rule (its I-3 made R6 and R7 instrument faults while "R1 to R5 stand"), with the float label added.

**Why.**
- Casey, 3 Nov, answering my summary of the night: "Yes, please proceed on your recommended path, trying to fix these problems". That path ran the full-length served model next, with I-3 reported beside it, and parity hunted in parallel.
- The remaining parity sites need a per-layer readout on the served contract. `fbs-snapshot`'s D11 readout runs snapped outside the served bridge, so it would compare raw against snapped. A served-contract mode is a new instrument. The trained-as-served track's bound, written when Casey stopped Q5 (2 Nov), brings any new instrument to Casey. That question is with Casey now, and only it waits: the readout parks, and this run, which needs no new instrument, goes ahead (§5.1).
- Holding the full-length run behind exact parity would leave step 4's model untrained for as many units as the parity hunt takes.
- The run gives I-3 at 8,192 steps on all six exports, and sealed full-length checkpoints and snapshots that a later readout can read.

**What could be lost.** If the remaining gap is on the float side, the float readings describe a model the laptop does not serve exactly. That is why nothing is closed, named or re-statused on them, and why the float-side branch voids them as served results.

**Reversal.** Casey can say "hold": no further pod launches, and nothing is closed or named on these readings. The checkpoints, snapshots and float samples stay sealed under `results/claude/one-model-end-to-end-full-unsnapped-2026-11-03/`. The next parity unit re-reads them: the integer-side branch re-exports from the snapshots, and a served-contract readout, once settled, reads the same checkpoints.

**Decision review.** A fresh-context review read the charter, PLACE's order, part B's sheet, claude-263 to claude-265's REPORTs and this card. ALLOW WITH CONDITIONS. Every condition is folded in: the Why above (the readout question goes to Casey and parks alone), I-3 per arm with no served ranking from the float one, part B's branches kept, the L-007 guard, R6's and R7's labels, I-0 naming claude-264's two and claude-265's four tests, the re-estimated cost and watcher limits, the §4.4 statement, and who keeps and re-reads the sealed outputs.

## Cancel probe

1. **Already done?** No. No full-length served run exists on either contract.
2. **The lock.** #3583 (deepseek-9, draft, unchanged since 09:52Z) still holds the item. The basis is the umbrella reading, as in claude-263 to claude-265.
3. **Inputs.** The pins of part B's sheet (TRAIN, DEV, tokenizer, p-w, p-v) are in the store; claude-261's grade, serve and timing scripts are local.

## Files in scope

- `docs/harness/prep/one-model-end-to-end-full-unsnapped*`
- `docs/loop/tasks/one-model-end-to-end/claude-266/**`
- `docs/research/notebook/float-training.md` (added rows only)
- `docs/roadmap/items/one-model-end-to-end.md`

No code.

## Done when

1. The new sheet is FIT, its pre-run review is PASS, and the decision card is logged.
2. On three 5090 pods at the pin, each seed's I-0 passes. Both arms train, are scored and snapshotted in turn. Outputs are sealed and read back, and the pods are down.
3. Every export is raw, and I-3, I-5 and I-6 are read on all six.
4. R1–R5 are graded, R6 and R7 are read, and each is marked by the sheet's rules.
5. A REPORT with tokens per second first and negatives first, a notebook row, and the item line.

## Stop and ask if

Each is decided in the unit, never waited on:
- **A smoke, build or I-0 fails:** stop that pod; it is an instrument result.
- **An arm diverges or runs out of memory:** classify it by the stop rule.
- **Casey says hold:** stop launching, keep what is sealed.
