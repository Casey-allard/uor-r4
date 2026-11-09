# REPORT: one-model-end-to-end, claude-343

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md).

## What did not work, first

- **This does not explain why the replies loop.** Parity is clean, so the serving path is removed as a cause of the disagreement row P was looking for, and nothing more. The loops themselves stay unexplained. The cause ladder (claude-344) tests the trained model's parts.
- **Parity here means the Hamming reads only, as the compare counts them.** For the copy and type heads at depths 2 and 6, the compare checks that each position's kept set matches, and looks at the keys only inside a set that differs. So "0 key flips" means no read's choice moved. It does not mean every stored key is equal: see the position-0 note under Measurements. The trunk's final hidden values and the nearest head's chosen token are not compared here.
- **The float-against-integer token report compares two different heads, and I should have seen that before adding it.** The float's `next` is the native writer's choice. The panel decodes through the nearest head. They agree on 107 of 16,068 reply rows at seed 1 and 86 of 16,384 at seed 2, differing at reply position 0 on all 256 windows at both seeds. That number says nothing about parity. It is reported only so nobody re-derives it and reads it as a fault. The alignment was checked on one row by hand: reply token 0 sits at window position `prompt_tokens`.
- **The moved-token control as first written proved less than it looked.** The compare refused it (exit 1, `window 0: the traces' ids or reads differ`). That shows the compare checks identity, not that a changed input moves the reads. So the moved trace was also compared with window 0's ids restored, on the laptop, and that form does show flips (below).

## What changed

- `fbs-snapshot --windows FILE` traces the float on token windows from a file, in place of `--dev`'s pinned sample. It refuses a wrong schema, no windows, duplicate ids, a window under 2 or over `context` tokens, and a token at or past the vocabulary.
- `served-reply-panel --windows-out FILE` writes each row's window: the prompt ids and the reply ids the integer engine read. A row that stopped on EOS or BOS keeps every reply id. A row that stopped on the cap or the context leaves out the last one, which no decision read.
- No model, trainer or serving code changed.

## Measurements

**The rule, fixed in the ORDER before the run: clean means 0 query flips, 0 key flips, 0 of both, 0 ties, 0 not served and 0 instrument faults on every read, head kind and window, at both seeds. It reads clean. Row P of claude-342's table does not fire.**

L-a sealed, `step-016384`, float arm `d11-tie-rounded` with `--served-readout`, against the sealed served artifact, on the panel's own windows. The float was fed the ids the integer read.

| Seed | Windows | Positions | Reads judged per row (depth 2 and 6, copy and type) | Query flips | Key flips | Both | Ties | Not served | Instrument faults | Downstream |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 256 | 29,822 | 59,644 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 2 | 256 | 30,132 | 60,264 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

The compare's `faults` list is empty at both seeds.

**A difference the compare does not count, stated so the word "clean" is not read as more than it is.** The ORDER says disagreement is any question or key bit that differs, and that the compare reports it as flips. Read directly from the raw traces, all 2,048 head traces per seed (256 windows × 8 head reads) have identical questions and kept sets, and every key is equal except at position 0. There the float holds a nonzero key and the integer records 0, on all 2,048 at both seeds. No kept set at either seed contains position 0, so no read ever used that word. It cannot change a decision, so I read it as not a disagreement and row P as not firing. The ask-side and copy-side reads are unaffected.

**The controls.** Each must not read clean.

| Control | Seed | Row | Reads equal of judged | Query flips | Key flips | Downstream |
|---|---|---|---|---|---|---|
| Ask block removed | 1 | depth 2, type | 9,277 of 59,644 | 50,367 | 0 | 0 |
| Ask block removed | 1 | depth 6, type | 9,216 of 9,216 | 0 | 0 | 50,428 |
| Ask block removed | 2 | depth 2, type | 9,851 of 60,264 | 50,413 | 0 | 0 |
| Ask block removed | 2 | depth 6, type | 9,216 of 9,216 | 0 | 0 | 51,048 |
| One token moved, window 0's ids restored | 1 | depth 2, copy | 59,537 of 59,644 | 2 | 105 | 0 |
| One token moved, window 0's ids restored | 1 | depth 2, type | 59,587 of 59,644 | 2 | 55 | 0 |
| One token moved, window 0's ids restored | 1 | depth 6, copy | 59,532 of 59,644 | 2 | 110 | 0 |
| One token moved, window 0's ids restored | 1 | depth 6, type | 59,530 of 59,530 | 0 | 0 | 114 |
| One token moved, window 0's ids restored | 2 | depth 2, copy | 60,155 of 60,264 | 2 | 107 | 0 |
| One token moved, window 0's ids restored | 2 | depth 2, type | 60,207 of 60,264 | 2 | 55 | 0 |
| One token moved, window 0's ids restored | 2 | depth 6, copy | 60,152 of 60,264 | 2 | 110 | 0 |
| One token moved, window 0's ids restored | 2 | depth 6, type | 60,150 of 60,150 | 0 | 0 | 114 |

- With the ask block removed, the copy heads stay equal at both depths and seeds (0 flips). That is expected: the block feeds the type heads only.
- The moved-token control, as run on the pod, was refused by the compare at both seeds (exit 1). The table is the same trace with window 0's ids restored, so the compare reads it.

**Recorded and judged by no rule.**
- The float with the reads not rounded (`d11-tie`) gives window traces identical to the rounded arm at both seeds, so rounding the reads moved no decision on these windows. claude-336 saw it move 21 of 3,072 depth-2 type decisions on its 192-token sequences. I did not establish why this differs.
- The float-native-writer report above.

**The sealed replies were reproduced.** `served-reply-panel --windows-out` on the sealed artifacts rewrites the sealed `[id, reply_ids, stop]` rows exactly. Taking the first 16 hex of the sha256 of `jq -c '[.rows[]|[.id,.reply_ids,.stop]]'` on the sealed `score.json` and on the new run gives `9293dbb78201ea59` (seed 1) and `acde5330e301213a` (seed 2) on both sides. The artifacts' sha256 are the sealed `7b817369dbe0420a…` and `df3c8f60999fe116…`.

**Where the served artifact comes from.** The served trunk and base Hamming reads come from `cpu-fbs-export --raw-gates --raw-transition` on a base-only checkpoint copy, and the ask entries are spliced in from the pod's snapshot (`read-339.sh`). That is the artifact the panel scored and the one compared here.

**Evidence, in the store** under `results/claude/one-model-end-to-end-panel-parity-2026-11-09/`:
- `in/`: windows, served artifacts and no-ask artifacts, with `SHA256SUMS`.
- `out/parity-out.tar.gz` (the job's traces and compares), read back from the store on the pod against its hash, and again on the laptop. Compares: `s1.compare.json` `d64b31ebc2d8f356…`, `s2.compare.json` `595b12fee80bc7b1…`. Binaries: `fbs-snapshot` `5bf9558681683e91…`, `fbs-toks` `f8644deb4fdce55c…`.
- `laptop/`: the restored-id compares, the float-native report, `next.jq`, `job-343.sh`, `launch-343.sh`, `gates.sh` and `gates-at-head.log`, with `SHA256SUMS`.

**The pod.** One 5090, `vast-59968193`, at `$0.563/h`, built from `2491203fe47cd2cb83f241734a50240187dbe9dd`. It was up from about 22:34Z to 22:45Z, about $0.10. The job took about 8 minutes. The pod is down.

## Checks run at the head

Code head `2491203fe47cd2cb83f241734a50240187dbe9dd`, the commit the pod built. Later commits on this branch touch only this REPORT and the notebook.

- `cargo test --offline -p uor-r4-float-train --bin fbs-snapshot`: 11 passed.
- `cargo test --offline --features cpu-parity --bin served-reply-panel`, in the instrument crate: 30 passed.
- Eleven negative gates, each a one-line break that must fail its named test, then restored:
  - in `parse_panel_windows`: a 1-token window allowed, a 2-token window refused, a window past `context` allowed, a window at `context` refused, a token equal to the vocabulary allowed, the schema unchecked, no windows allowed, duplicate ids allowed;
  - in `take_windows_flag`: the flag given twice allowed;
  - in `read_window`: an EOS row dropping its last id, a cap row keeping its last id.

  All eleven failed their tests (log in the store). The duplicate-ids gate has to be written `... && false`, because `if false` stops the set's type being inferred and fails to compile, which would prove nothing.

## Skills used

- None beyond the loop itself: the ORDER, the fresh exact-head reviews and the pod tools.

## Still open

- **The cause ladder** (N0 and N1, two seeds each, at pin `721a4784f`) is next: claude-344, under its own sheet, FIT and pre-run review. It goes ahead by this result.
- **Why replies loop:** unknown.
- **The trunk's final hidden values and the nearest head's token** are not compared between float and integer by this unit.
- **Why rounding moved no decision here** but moved some in claude-336.
- **Before reusing these windows on a carry artifact such as L-aC:** the windows file holds no reply boundary, and the panel calls `mark_reply_start`, which only touches turn memory. L-a has no turn memory, so this result is unaffected.
- **Small gaps in the new tool, from the code review, none reaching this result:** `artifact_sha256` in the windows file is printed and not bound to the model, so seed-1 windows would be accepted on seed 2's model; a prompt longer than the context gives a window the parser refuses, loudly; `--windows-out`'s own flag handling and the `--windows`/`--dev` refusal have no test.
- **mw-balp's 35 paraphrase files** are not published, and **`test-watch`** still depends on signal timing.

Closes nothing: #3407 stays open.

## Plain summary

On L-a's sealed checkpoint, at both seeds, on the panel's own prompts and replies, the trained float model and the served integer file make the same Hamming reads at every position: 0 differences in about 240,000 judged reads per seed. Removing the ask block or changing one input token breaks that agreement, so the comparison can fail.

This rules out one thing. The served file's attention reads are the trained model's, so the looping replies are not caused by the integer export getting those reads wrong. It does not say what does cause them. The next unit trains smaller versions of the model to find out which part brings the loops on.
