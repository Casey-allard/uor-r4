# REPORT: written-state-count-model, claude-417 (the D87 dedup keyed on span multiplicity)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The DEV-test cost is five times the 5% line** [claude]. 1,450 of 5,321 DEV-test records (272 per mille) were still contaminated after the TRAIN drop, and all of them would have to go. They carry 9,997,755 of 21,354,897 DEV-test tokens (468 per mille), so the records dropped are the long ones.
- **The TRAIN cost misses the line narrowly** [claude]. The TRAIN drop removes 14,416 of 519,580 records (28 per mille) and 119,749,006 of 2,091,560,001 tokens (57 per mille).
- **So split v3 is not adopted**, by the ORDER's own bar [claude].
- **My prediction of the residue was wrong** [claude]. I expected about 70 DEV-test records left after the TRAIN drop: claude-414's 68 stock-only ones. There were 1,450.
  - claude-414 grouped each contaminated record by its *least* span multiplicity. A record holding one copied span counted as copied, even when it also held stock spans.
  - Dropping the copied holders removes the copied spans, but the stock spans stay.
  - So about 1,382 of claude-414's 1,868 copied-contaminated records also hold a stock span [claude, inference: 1,450 less the 68 stock-only].
  - That share was never measured; the least-multiplicity table could not show it.
- **The fresh review requested changes** [claude]. Its fixes are in `537f0c0af`:
  - MEDIUM: `holds_copied` read a tuple field as `self.uniq[j].0`. The float lexer counts `].0` as a float, so the PR would not have queued. The field is now read by destructuring.
  - LOW: the copied finder combined with a TRAIN drop list would count multiplicity over the reduced split, not split v1. That combination is now refused. Its test fails with the refusal set aside; restored, 28 of 28 pass.
  - LOW, not changed: the DEV-test refusal compares a record count against the list length, not matched ids. Kept ids are unique, because the plan removes duplicate ids, so this cannot fire.
  - LOW, not changed: refusals of DEV-tune ids, removed duplicates and repeated ids are enforced by the code but pinned by no test.
- **The first `wick.sh get` returned `run3/` empty** [claude]. A second `get` copied all five files; no file was lost.
- **The wick runs used the build at `e8eec694f`** [claude]. Its lab code equals `337ceb81d`, before the review fixes. Neither fix changes these runs' output: one is a rewrite of the same comparison, and the refused combination was not used.

## What changed

- The instrument change, in `337ceb81d` and `537f0c0af` [claude]:
  - `SpanSet::holds_copied` and a third TRAIN pass over the copied spans of the counting pass;
  - a DEV-test drop applied in pass 1, which refuses ids that are not kept DEV-test records;
  - the residue list;
  - `d87-audit --write-drop-copied`, `--write-dev-residue` and `--drop-dev-test`;
  - six fixture tests (28 in all).
- Count-only summaries [claude]: `run1/` and `run2/` audit summaries, and `run3/` audit and classify summaries. They hold no corpus text and no record ids; the 14 hex strings in them are SHA-256 digests.
- One notebook row in `docs/research/notebook/data-tokenizer.md` [claude], at the top of the table.
- Laptop-side and untracked, in `local/claude/claude-417/c417/` [claude]:
  - the copied-holder list: 14,416 ids, SHA-256 `fdf8587569d64b9ada7d2892f26348588caa8b827f032d10548b4ef7ea4597bb`;
  - the DEV-test residue list: 1,450 ids, SHA-256 `c239237f020cd11551410086e4173e60301df1344d3a100b8c859ef913b36064`.

## Measurements

Wick job `c417-run-1`: 12 CPUs, 262 shards, the `/workspace/in/d87` shard copies and claude-411's split v1, 22:29:44Z to 22:44:26Z [claude].

| | run 1: split v1 | run 2: TRAIN drop | run 3: both drops |
|---|---|---|---|
| TRAIN records | 519,580 | 505,164 | 505,164 |
| TRAIN tokens | 2,091,560,001 | 1,971,810,995 | 1,971,810,995 |
| DEV-test records | 5,321 | 5,321 | 3,871 |
| DEV-test tokens | 21,354,897 | 21,354,897 | 11,357,142 |
| contaminated at 100 | 658 | 467 | 0 |
| contaminated at 50 (gate) | **1,936** | **1,450** | **0** (PASS) |
| contaminated at 32 | 3,213 | 3,132 | 1,682 |
| novel response targets | 14,190,847 / 18,751,676 (757 per mille) | 14,428,503 / 18,751,676 (769) | 9,251,637 / 10,394,330 (890) |

Run 1 matches claude-411, claude-414 and claude-416 at 1,936 [claude].

## What the numbers say

- **Copied overlap is cheap to remove** [claude].
  - The TRAIN drop clears every span held by at most two TRAIN records.
  - It costs 57 per mille of TRAIN tokens, against 245 for claude-416's length cut.
  - It clears 486 DEV-test records outright.
- **Stock overlap is not cheap to remove from either side** [claude].
  - 1,450 DEV-test records (272 per mille) share a 50-token span with three or more TRAIN records.
  - claude-414's top spans show these are template passages, such as an image-prompt generator's instructions, repeated across tens of thousands of records.
  - Removing them takes a quarter of DEV-test and almost half its tokens.
  - Removing their TRAIN holders instead took a quarter of TRAIN (claude-416).
- **The record-level gate measures the wrong thing for stock text** [claude].
  - Stock overlap means the model may have seen a template the test also uses. That is a fact about the data's mix, not a copied answer.
  - A per-record yes/no cannot separate a record that repeats one template line from a record that copies a whole answer.
  - The novel subset does separate them. It scores only targets whose 13 preceding tokens never occur in TRAIN, so it is position-level and measures generalization directly.

### Decision (claude, top session's decision review; logged for Casey, who can reverse it)

1. **Split v3 is not adopted; split v1 stays** [claude]. Both costs exceed the ORDER's bar.
2. **The copied-holder list is kept as a candidate, not adopted** [claude].
   - Its cost, 57 per mille, misses the 5% line by 0.7 points.
   - Adopting it after the fact would move a pre-registered bar.
   - A later unit may adopt it under a fresh, pre-registered decision. One example: R\* scoring finds the full and novel DEV-test BPB far apart on the copied records.
3. **Bar Q gates on the novel subset, as claude-411 decided** [claude]. The record-level 50-token gate stays in every summary and reads FAIL on split v1. The D87 dedup line now closes, and the next units are R\*.

## Checks run at the head

- G1: the instrument tests [claude]:
  - on wick, at the run's build: 27 passed;
  - on the laptop at `537f0c0af`: 28 passed, 0 failed.
- G2: set-aside negatives, re-run by the orchestrator at `e8eec694f` [claude]:
  - with the multiplicity window opened to any multiplicity, 3 tests fail, including `no_holder_of_a_stock_span_is_listed`;
  - with the DEV-test skip's `return` removed (the refusal count kept), `both_lists_applied_leave_no_50_token_contamination` fails;
  - restored and rebuilt, 27 of 27 pass.
  - The review fix's refusal, set aside, fails `the_copied_finder_refuses_a_train_drop_list`; restored, 28 of 28 pass.
- G3: `UOR_LAB=claude uor-harness loop pr-check --local` at the committed head: see the PR [claude].
- G4: `c417-run-1` returned rc 0 [claude]. Run 1 reads 1,936. Run 3 shows both lists applied (14,416 and 1,450 records, each with its list's SHA-256) and the 50-token PASS at 0 of 3,871.
- G5: both lists are in `local/claude/claude-417/c417/` in the main checkout, outside this worktree, and are not tracked [claude].
- Nothing on `/mnt/plexpool` was touched [claude].

## Skills used

None invoked [claude]. I followed AGENTS.md and the D87 protected-asset rule: read only, copies allowed, no corpus text or id list committed.

## Still open

- The R\* units: continuation counts, modified Kneser–Ney, and BPB equal to the oracle [claude].
- Writing the novel-subset position mask as a file for R\* to read [claude].
- A later, pre-registered adoption of the copied-holder list, if R\* shows the copies matter [claude].

## Plain summary

The project tried a second way to clean the overlap between its training and test data [claude]:
- Removing the training records that copy a test passage, ones shared with at most two training records, works and costs under 6% of the training text.
- But a quarter of the test records still share boilerplate passages with many training records: template text, not copied answers.
- Removing those would cut the test set by a quarter.

So neither side is trimmed further, and the split stays as it is [claude]. Honest scoring comes instead from the "novel" part of the test set: the positions whose preceding text never appears in training. The model is held to its bar there [claude].
