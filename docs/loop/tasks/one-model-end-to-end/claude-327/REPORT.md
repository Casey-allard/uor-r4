# REPORT: one-model-end-to-end, claude-327

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Part B's sheet: [one-model-end-to-end-query-recall-2026-11-08.md](../../../harness/prep/one-model-end-to-end-query-recall-2026-11-08.md).

## What did not work, first

- **The first-target rank cannot read choice on claude's served models either.** On every artifact read here (claude-322, claude-323 and claude-325, two seeds each), teacher-forced:
  - the reference's first token is top-1 on **0 of 256** rows, under both renderings;
  - the model's own top-1 at that position is a frame word on every row: " Your", " G" (the start of "Got it, …"), " I" or " It".

  The frame word does not follow the asked relation. So, as on DeepSeek's artifacts, the first-target rank here measures reply format, not which value is chosen. Verdict: **diagnostic stop** for the first-target rank as a choice instrument on these artifacts.
- **No claude checkpoint read here shows a choice of the asked value.** When a reply's first line holds a prompt value at all, it is the other relation's value about as often as the asked one: 75 asked against 79 other, pooled over six seeds (A2). That reading rests on about 10% of rows.
- **A2 deviates from the ORDER.** The ORDER's A2 is the teacher-forced rank of the other value's first token beside the asked one's, at the same position. Where the scorer cannot read the other value, the ORDER asks for the instrument to be extended, with a test. The scorer ranks only the reference reply, and this unit did not extend it. A2 is instead read from the free-generation first-line counts (`firstline.jq`, unchanged).
  - **The substitute is weaker.** It sees only the rows whose first line holds a prompt value at all: 154 of 1,536 rows over six seeds, about 10%. The paired rank would read every row, and a frame word at top-1 would push both ranks down alike.
  - **Done-when 2 is met only under the changed measure.** The paired rank is named under Still open for claude-328.
- **Part B's sheet is not FIT yet.** Two fresh FIT rounds both read NOT FIT. Round 2's blocker: the read runs at two depths (`HAMMING_DEPTHS = [2, 6]`), so a copy head at the query token can fetch the value and the trunk can carry it to the scored position. The first sheet would have misread that as an instrument fault. All findings are applied; round 3 is the first step of the unit that builds Part B's code.
- **The Hamming trace at the first reply position was not taken.** It needs new code, so under the ORDER it is named here as still open.

## What changed

- Part B's prep sheet (new), after two FIT rounds, with every finding applied.
- This REPORT.
- One notebook row in `docs/research/notebook/float-training.md`, above claude-325's.
- No code, data, bar, status, scope or cap changed.

## Measurements

All laptop CPU. `served-reply-panel --teacher-forced 1` was built from `main` at the ORDER's base, and run on each stored served artifact under both renderings. Outputs are in `local/claude/claude-327/a1/`. The free-generation rows of claude-322's seed 1 under this build equal its earlier `score/s1.json` (`jq -S -c .rows`), the build's control.

**A1, the first target** (top-1 / top-8 / top-32 of 256; rank min / median / max of 4,096):

| Artifact | one-user-turn/v1 | per-turn/v1 |
|---|---|---|
| claude-322 s1 | 0 / 1 / 8; 7 / 1,282 / 4,076 | 0 / 0 / 8; 17 / 1,005 / 4,075 |
| claude-322 s2 | 0 / 0 / 0; 65 / 2,029 / 4,048 | 0 / 2 / 14; 7 / 2,314 / 4,050 |
| claude-323 s1 | 0 / 0 / 0; 40 / 1,324 / 4,062 | 0 / 0 / 2; 26 / 1,286 / 4,064 |
| claude-323 s2 | 0 / 0 / 0; 34 / 2,396 / 4,069 | 0 / 0 / 0; 36 / 2,036 / 4,063 |
| claude-325 s1 | 0 / 4 / 12; 6 / 2,335 / 4,080 | 0 / 2 / 4; 5 / 1,940 / 4,070 |
| claude-325 s2 | 0 / 3 / 8; 5 / 1,486 / 4,066 | 0 / 0 / 14; 17 / 1,070 / 4,064 |

**The model's own top-1 at the first reply position** (one-user-turn/v1; per-turn/v1 uses words from the same four, with different counts, for example claude-325 s1 " Your" 128, " G" 84, " It" 44):

| Artifact | Top-1 first token, rows of 256 |
|---|---|
| claude-322 s1 | " Your" 172, " G" 84 |
| claude-322 s2 | " Your" 109, " G" 84, " It" 63 |
| claude-323 s1 | " It" 154, " G" 84, " Your" 18 |
| claude-323 s2 | " Your" 172, " G" 84 |
| claude-325 s1 | " Your" 102, " G" 86, " I" 54, " It" 14 |
| claude-325 s2 | " Your" 172, " G" 84 |

- **The frame does not follow the relation.** On claude-322's seed 1, home questions split " G" 44 / " Your" 84, and job questions split " G" 40 / " Your" 88.
- **It does not follow the stratum either:** each stratum shows both words.
- **It follows the question's template.** The panel phrases each question in one of six forms (`q0`–`q5` in the row id). On claude-322 s1, claude-323 s2 and claude-325 s2:
  - " G" appears only on forms q3–q5;
  - form q4 gives " G" on all 40 of its rows;
  - forms q0–q2 give " Your" on all of theirs.

  On claude-325 s1, " I" and " It" appear on forms q0–q3. This is descriptive.
- **Later targets are mostly right under teacher forcing.** Over all 2,832 reference targets of claude-325's seed 2 (one-user-turn), 1,805 are top-1, while the 256 first targets give 0 of those.

**A2, the first line** (`firstline.jq`, sha `b15c778d…`; asked / other / both, of 256):

| Artifact | First line | Whole reply |
|---|---|---|
| claude-322 s1 | 14 / 11 / 0 | 14 / 11 / 0 |
| claude-322 s2 | 2 / 6 / 0 | 3 / 6 / 0 |
| claude-323 s1 | 0 / 0 / 0 | 0 / 0 / 0 |
| claude-323 s2 | 32 / 30 / 0 | 32 / 30 / 0 |
| claude-325 s1 | 27 / 29 / 0 | 27 / 29 / 0 |
| claude-325 s2 | 0 / 3 / 0 | 0 / 3 / 0 |

- **Pooled first line: 75 asked, 79 other.** A binomial reading at one half cannot tell that from a coin.
- The one record that leans to the asked value is claude-321's seed 2, at 76 / 36 (notebook row 15).
- The whole-reply column uses the same file with its first-line cut removed, and is descriptive only.

## Checks run at the head

- `uor-harness loop report --check --order docs/loop/tasks/one-model-end-to-end/claude-327/ORDER.md`: PASS at the head.
- **Build control:** claude-322 s1's free-generation rows under the teacher-forced build equal its stored rows.
- **Part B FIT, round 1** (fresh, non-author): NOT FIT, with 2 blockers, 4 majors and 4 minors, applied at `a61f7e590`:
  - table alignment lets copy heads solve B1;
  - 16× fewer targets than row 109;
  - recency averaging;
  - the payload confound;
  - the B0 bar was too tight;
  - placement was not specified.
- **Part B FIT, round 2** (fresh): NOT FIT, with 1 blocker, 3 majors and 3 minors, applied at `fd24d51d8`:
  - the depth-2 copy route (i-b), so the trace now runs at both positions and the ablations are split by depth;
  - the other fact's distance is now drawn from the asked fact's bucket;
  - "kept" now means weight above 0, with a pre-registered rate;
  - B0 ablation controls;
  - the minors: B0's held-out count, a bit-exact step-0 check, and the distance's endpoints.
- The fresh exact-head review of this PR is recorded with `loop review`.

## Skills used

- None beyond the loop itself (ORDER, REPORT, two FIT rounds, the fresh review).

## Still open

- **Part B, the next unit (claude-328):** FIT round 3 of the sheet first, then the code it names, then the run under a D67 pre-run review. claude-326's lab decision stands: no one-knob language run on this line until Part B has run.
- **The Hamming trace at the first reply position** on the served artifacts. It needs new code in `fbs-toks` or the scorer. It matters less now that the first position is a frame word.
- **A2 as the ORDER framed it,** the paired teacher-forced rank of the asked value's first token against the other value's at the first reply position, on every row. It needs the scorer to rank a second reference, with a test, and goes to claude-328.
- **Why the question's template picks the frame word** (" G" on forms q3–q5) is described here, not explained.

## Plain summary

We checked what claude's last six trained models say first when they answer a question about a fact in the chat. Every one opens with a stock word ("Your…", "Got it…", "It…"), never the answer, whichever fact is asked. The stock word depends on how the question is phrased, not on which fact it asks for. In the roughly one reply in ten that names a value, it is the wrong fact about as often as the right one. So nothing here shows the models picking the asked-for fact. The test that asks whether the memory read can pick a fact by a question that does not repeat the fact's own word is designed and has had two outside reviews. It is the next unit's to build and run.
