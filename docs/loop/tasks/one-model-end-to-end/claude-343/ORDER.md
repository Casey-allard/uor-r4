# ORDER: one-model-end-to-end, claude-343

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `81f97127f`.

## Goal

**Check float-to-integer parity on L-a's sealed `step-016384`, at both seeds, on the panel's own prompts and their decode.**

- **Why now.** claude-342's decision review put this first: L-a's served replies loop on 243 and 255 of 256 rows, and row P of its read table says a disagreement here means a serving fault, to be repaired before any ask reading stands. claude-336's parity traced 8 fixed 192-token sequences, which would miss a fault that shows only on short contexts. deepseek-264 compared the CPU port with the integer engine on L-aC, so the trained float was not compared.

**The work:**
1. **A windows input for the float trace.** `fbs-snapshot --windows FILE` takes token windows from a file, in place of `--dev`'s pinned DEV sample. The integer side, `fbs-toks --hamming-trace`, already reads its sequences from the float's readout, so it needs no change.
2. **The panel's windows.** `served-reply-panel` writes each row's window: the prompt ids and the reply ids the integer engine chose, without the last reply id, which no decision read. The float is fed exactly what the integer read, so it cannot run free past a first flip.
3. **The check, on one GPU pod.** On L-a seeds 1 and 2 at `step-016384`: the float trace with `--served-readout` and arm `d11-tie-rounded` (the contract the run trained on, rounded as exported), the integer trace on the sealed served artifact, and `--hamming-trace-compare`.
4. **Controls that must fail:** the ask block removed from the artifact, and one window's token changed in the integer's input.

**How it reads, fixed before the run.** Disagreement is any Hamming question or key bit that differs from the float trace rounded as exported, row by row. The compare reports it as query flips, key flips or both. Clean means 0 of each, 0 ties, 0 not served and 0 instrument faults, on every read, head kind and window, at both seeds.
- **Clean at both seeds:** row P does not fire, and the ladder goes ahead on its sheet.
- **Any disagreement:** `blocked`, row P. Classify where and why, repair, re-serve. Nothing about the ask heads is read until then.
- **A row the compare calls downstream** (a later read after an earlier difference) is reported and not judged, as in claude-336.

The check says nothing about why the replies loop if it is clean, beyond removing the serving path as a cause.

**Claim check:**
- **(a) Step:** step 4, one model trained, served and scored end to end.
- **(b) What it changes:** no model. It establishes whether the served artifact is the trained model on the panel, which every later ask reading needs, and it unblocks the cause ladder.

**Not in scope:** the ladder's training runs (claude-344, own sheet, FIT and pre-run review), new mechanisms, the CPU port (DeepSeek's lane).

**Why no prep sheet for this step.** It reads sealed checkpoints and trains nothing, as claude-336's parity did: a host check under CHARTER §4 item 5. claude-342 planned parity and ladder under one sheet. The ladder's sheet will record this result, so parity is run first and the sheet can state it as a fact.

## Cancel probe (run first)

1. claude-342 is merged and verified.
2. No open claude PR, and none touching these files.
3. L-a's sealed checkpoints and served artifacts are in the store.

**Result (at claim):** pass.
- claude-342 merged as `81f97127f90c76d226a15af90c39ebe2931a271e` and was verified on fresh `main`.
- Open PRs are Kimi's 3903 and DeepSeek's 3889; neither is claude's or touches these files.
- The store lists `runs-3/L-a/s1` and `s2` (`language-carry-L-a-s*.tar.gz`, `SHA256SUMS`) and `early/L-a-s*/step-016384/served.uorlut`.
- `loop next` offers only DeepSeek's CPU-repair direction, so this ORDER is hand-written under Casey's harness exception, as claude-331 to claude-342 were.

## Files in scope

- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`

## Done when

1. Both new inputs have host tests, each with a negative gate.
2. The sealed `reply_ids` are reproduced by the panel run that writes the windows, or the difference is named.
3. The compare is reported for both seeds, with both controls failing where they should.
4. The REPORT reads the result by the rule above.
5. A fresh exact-head review passes the PR, which merges and is verified on `main`.

## Stop and decide

Nothing waits on Casey (§5.1). A decision that would change a bar goes through its own decision review and is logged.
