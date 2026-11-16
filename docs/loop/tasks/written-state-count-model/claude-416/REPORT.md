# REPORT: written-state-count-model, claude-416 (the D87 span dedup at 100 tokens, and the re-audit)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The dedup costs far more than the 5% line: 245 per mille of TRAIN tokens** [claude]. Dropping every kept TRAIN record that holds a 100-token DEV-test reasoning+answer span removes 72,765 of 519,580 TRAIN records (140 per mille) and 513,322,742 of 2,091,560,001 TRAIN tokens. That is about five times the ORDER's 5% cost line.
- **The 50-token gate still FAILs after the dedup: 1,331 of 5,321 DEV-test records (250 per mille)** [claude]. It was 1,936 before. The bar is at most 1%.
- **The dropped records were mostly the wrong ones** [claude]. The long spans sit in stock template passages, the image-prompt generator instruction claude-414 found, held by tens of thousands of TRAIN records. A 100-token match in those passages drops every record of that template. The copied content survives: 1,261 of the 1,331 records still contaminated hold a span found in only one or two TRAIN records.
- **The 100-token result is 0 by construction, not evidence** [claude]. The run checks it, and it holds, but it only shows the drop list was applied.
- So this configuration, "drop the TRAIN holders of a 100-token span", is closed as negative. The span dedup is not closed.
- **The fresh review found three LOWs, all fixed in `023cb66cf`** [claude]. Its verdict was APPROVE-WITH-LOW, with no HIGH or MEDIUM findings.
  - Nothing pinned that `holds` has no early skip. A new test has a second TRAIN record, in the other shard, holding the same span. With an `is_hit` skip put into `holds`, that test fails and 21 pass; restored, 22 of 22 pass.
  - An empty `--drop-train` file was accepted and left a summary that looked as if a list had been applied. It is now refused.
  - The span prefix was computed twice per TRAIN record. It is now computed once. The wick runs used the old code, whose output is the same.

## What changed

- The instrument change landed in `adeb2080e` [claude]:
  - `SpanSet::holds`;
  - `AuditOptions` with `drop` and `find_drop`;
  - `d87-audit --write-drop` and `--drop-train`;
  - the refusal of drop ids that are not kept TRAIN records;
  - a `train_drop` block in `audit-summary.json` and `classify-summary.json`;
  - four fixture tests.
- `run1/audit-summary.json`, `run2/audit-summary.json` and `run2/classify-summary.json` [claude]. These hold counts only: no corpus text and no record ids.
- One notebook row in `docs/research/notebook/data-tokenizer.md` [claude], at the top of the table.
- The drop list (72,765 ids, SHA-256 `a85d430dc3964f9572efed275e139f19d71fa29386ea36ecbeaae2322a312f21`) and the decoded top spans are laptop-side in `local/claude/claude-416/`, outside the repository [claude].

## Measurements

The job `c416-run-1` ran in wick's container: 12 CPUs, 262 shards, over the `/workspace/in/d87` shard copies and claude-411's split v1 [claude]. Run 1 ran from 21:54:01Z to 21:58:16Z; run 2 from 21:58:16Z to 22:02:34Z.

| | split v1 (run 1) | split v1 minus the drop list (run 2) |
|---|---|---|
| TRAIN records | 519,580 | 446,815 |
| TRAIN tokens | 2,091,560,001 | 1,578,237,259 |
| DEV-test contaminated at n = 100 | 658 | **0** |
| DEV-test contaminated at n = 50 (gate) | 1,936 / 5,321 | **1,331 / 5,321** |
| DEV-test contaminated at n = 32 | 3,213 | 3,062 |
| novel response targets (of 18,751,676) | 14,190,847 | 15,581,555 |

Run 1 matches claude-411 and claude-414 at 1,936 [claude].

Run 2 classification, using claude-414's rule (copied means the span is found in at most 2 kept TRAIN records) [claude]:

- **copied-contaminated: 1,261 of 1,331**;
- stock-only: 70.

The records still contaminated, by their least span multiplicity [claude]:

| least multiplicity | records |
|---|---|
| 1 | 1,183 |
| 2 | 78 |
| 3–9 | 55 |
| 10–99 | 15 |
| ≥ 100 | 0 |

## What the numbers say

- **The long spans are the stock ones** [claude]. A 100-token match is most likely inside a long template that recurs verbatim. So a length threshold selects stock holders, which are many, rather than copy holders, which are few. That is why the drop removes a quarter of TRAIN's tokens and still misses most copies.
- **The copies are short or broken up** [claude]. 1,183 records still hold a 50-token span found in exactly one TRAIN record, but no 100-token span of it. The duplicated conversations must differ every few dozen tokens: re-sampled, re-tokenized at the edges or lightly edited. A length cut cannot reach them. A multiplicity cut can.
- **Multiplicity is the right key** [claude]. claude-414 measured it, and run 2 confirms it: the residue is 95 per cent copied, not stock.

### Decision (claude, top session's decision review; logged for Casey, who can reverse it)

1. **Split v2 is not adopted** [claude]. TRAIN stays as in split v1 until claude-417.
2. **Next unit, claude-417** [claude]:
   - drop only the TRAIN records that hold a copied 50-token DEV-test span (multiplicity at most 2);
   - remove from DEV-test the stock-only records still contaminated after that;
   - re-audit, and report the TRAIN token cost and the DEV-test record cost, each against 5%.
3. **The novel-subset guard on bar Q stays**, as claude-411 decided [claude].

## Checks run at the head

- G1: `cargo test --release` on the instrument [claude]:
  - on wick, on the run's code: 21 passed;
  - on the laptop at `830b1fddc`: 21 passed, 0 failed;
  - on the laptop at `023cb66cf`, with the review's test: 22 passed, 0 failed.
- G2, set-aside negatives on the laptop, each restored with `git checkout` and rebuilt afterwards [claude]:
  - with the `holds` call made false, `a_train_record_holding_a_100_token_span_is_listed_for_drop` fails (20 of 21 pass);
  - with the drop skip's `return` removed, `the_drop_list_removes_the_100_token_contamination_only` fails (20 of 21 pass);
  - restored and rebuilt, 21 of 21 pass.
- G4: `c416-run-1` returned rc 0 [claude]. Run 1's gate reads 1,936 of 5,321. Run 2's summary shows:
  - the drop applied: 72,765 records, with the list's SHA-256;
  - 0 contaminated at 100 tokens;
  - the 50-token FAIL at 1,331, with its classification.
- G5: the drop list and `top-spans.tsv` are in `local/claude/claude-416/` in the main checkout, outside this worktree, and are not tracked [claude].
- G3: `UOR_LAB=claude uor-harness loop pr-check --local` at the committed head: see the PR [claude].
- Nothing on `/mnt/plexpool` was touched [claude].

## Skills used

None invoked [claude]. I followed AGENTS.md and the D87 protected-asset rule: read only, copies allowed, no corpus text and no id list committed.

## Still open

- claude-417, the multiplicity-keyed dedup of decision item 2 [claude].
- Write the novel-subset position mask as a file for the R\* units to read [claude].

## Plain summary

The project tried to clean its training data by removing every training record that shares a long passage (100 tokens) with the test set [claude]. That removed a quarter of the training text, far more than the 5% planned. The test set was still heavily overlapped: a quarter of its records still share a 50-token passage with training [claude]. The long passages were mostly boilerplate from one prompt template that many records repeat, so the cut removed boilerplate holders and missed the actual copies [claude]. The fix is to key the cut on how many training records share a passage, not on its length. That is the next step. Until then, test scores keep being reported on the part of the test set that does not follow training text [claude].
