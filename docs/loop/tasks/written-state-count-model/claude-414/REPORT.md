# REPORT: written-state-count-model, claude-414 (classify the D87 TRAIN/DEV-test shared spans as copied or stock)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Split v1 does not stand without a span dedup** [claude]. Of the 1,936 contaminated DEV-test records, 1,868 hold at least one shared 50-token span found in only one or two TRAIN records. That is 964 per mille, far above the 10% bar and clear of the 8–12% borderline band.
- **claude-411's stock-wording reading was wrong for most records** [claude]. claude-411 expected the overlap to be the teacher's habitual wording [claude, hyp]. The widely shared spans are stock, but only 68 contaminated records are stock-only.
- **The ranking by spread hides the copied spans** [claude]. All 200 spans held by the most DEV-test records are stock. Each copied span is held by few records, so a top-200 sample alone would have pointed the wrong way. The record-level count decides the bar, not the sample.
- **Two builders stopped before any run** [claude]. The first stopped at its context limit. A second was blocked by the worktree edit guard. Neither had run anything on wick, so no run was lost or repeated.
- **The unit was first named claude-411c** [claude]. `loop pr-check` parses the `<lab>-<n>` folder name as an integer, so `order` and `report` failed. The folder, the summary's `unit` label and the code comments were renamed to claude-414 after the run; no number changed. The laptop folder moved too, to `local/claude/claude-414/`.
- **The fresh review found the least-multiplicity rule unpinned** [claude]. In the first fixture, every DEV-test record held exactly one shared span, so taking the least, the greatest or the first multiplicity gave the same answer. A fifth test now has one record holding a copied span and a stock span. With `least` changed to `greatest`, that test fails and the other 16 pass; restored, 17 of 17 pass. The review returned APPROVE-WITH-LOW.
- **Multiplicity counts only the TRAIN records split v1 keeps** [claude]. Exact, id and near duplicates were already dropped. A stock passage whose other copies sat in dropped duplicates can therefore read as copied. This matches the rule as written, "distinct TRAIN records", and it errs toward the dedup.
- **This builder's worktree started at `main`, not at the unit's code commit** [claude]. It was fast-forwarded to `77dc47702` before the run; nothing was rewritten.

## What changed

- `docs/loop/tasks/written-state-count-model/claude-414/classify-summary.json` [claude]: counts only, 6,251 bytes, no corpus text.
- One notebook row in `docs/research/notebook/data-tokenizer.md` [claude], at the top of the table, because that file's rows run newest first.
- The instrument change (`--classify-spans`, per-span distinct TRAIN-record multiplicity, four fixture tests) landed in `77dc47702` [claude].
- The decoded top 200 spans are in `local/claude/claude-414/top-spans.tsv`, laptop-side and untracked [claude].

## Measurements

One run on wick's container, 12 CPUs, 262 shards, 21:15:23Z to 21:19:10Z (about 4 minutes), over claude-411's split v1 (`split_summary_sha256` 5146fd27…) [claude].

- Contaminated DEV-test records at n = 50: **1,936 of 5,321**, the same as claude-411 [claude].
- Copied-contaminated: **1,868 of 1,936 (964 per mille)**. Stock-only: 68 [claude].
- Shared distinct 50-token spans: 249,554. Copied: 119,847. Stock: 129,707 [claude].
- The most TRAIN records holding one span: 43,147 [claude].

Shared spans by TRAIN multiplicity [claude]:

| multiplicity | spans |
|---|---|
| 1 | 88,257 |
| 2 | 31,590 |
| 3–9 | 68,696 |
| 10–99 | 48,081 |
| ≥ 100 | 12,930 |

Contaminated records by their least span multiplicity [claude]:

| least multiplicity | records |
|---|---|
| 1 | 1,797 |
| 2 | 71 |
| 3–9 | 54 |
| 10–99 | 14 |
| ≥ 100 | 0 |

The top 200 spans, ranked by the DEV-test records holding them [claude]:

- 200 stock, 0 copied.
- TRAIN multiplicity 2,293 to 43,142; 82 of the 200 sit at 43,142.
- DEV-test records per span: 29 to 219.
- Most are pieces of one image-prompt generator instruction: its "/imagine prompt" format line and its example wording. claude-411's prompt sample found the same template.

## What the numbers say

- **Two kinds of overlap sit side by side** [claude]. A few template passages recur across tens of thousands of TRAIN records. They are stock wording, spread over many DEV-test records. Beneath them, almost every contaminated record also shares a 50-token span with just one or two TRAIN records.
- **1,797 contaminated records have a span found in exactly one TRAIN record** [claude]. A passage of 50 tokens or more that occurs in exactly one other record is a specific duplicate, not recurring phrasing. The rule is deterministic and reads no text, so a rare stock passage would also count as copied. The size of the gap, 964 per mille against a 100 per mille bar, leaves no doubt about which side of the bar the share falls.
- **Novel-subset scoring stays as the guard** [claude]. claude-411's decision item 2 already gates bar Q on the novel subset. This result makes that guard necessary, not optional, until the dedup lands.

### Decision (claude, top session's decision review; logged for Casey, who can reverse it)

1. **Verdict: SPAN-DEDUP** [claude]. The copied share is 1,868 of 1,936, far above 10%, so split v1 does not stand as it is.
2. **Next unit: the span dedup of claude-411's decision item 3** [claude]. Remove the TRAIN records that share a span of 100 tokens or more with DEV-test, then re-audit, including this classification.
3. **Until then, every D87 DEV-test score is also reported on the novel subset, and bar Q gates on it** [claude], as claude-411 decided.

## Checks run at the head

- G1: `cargo +1.97.1 test --release --locked --manifest-path docs/labs/d87-split-audit/instrument/Cargo.toml` on wick, on the run's code: 16 passed, 0 failed. On the laptop at this head, with the fifth test: 17 passed [claude].
- G2 (run by the orchestrator at `77dc47702`): with the two within-record dedup lines removed, 4 tests failed, including `a_span_repeated_in_one_train_record_counts_once`; restored, 16 of 16 passed [claude].
- G4: the wick job `c411c-run-1` returned rc 0. `classify-summary.json` holds the copied share, both histograms and the contaminated total, 1,936 [claude].
- G5: `top-spans.tsv` is in `local/claude/claude-414/`, and `git status --short` is clean [claude].
- G3: `UOR_LAB=claude uor-harness loop pr-check --local` at the committed head: see the PR [claude].
- Nothing on `/mnt/plexpool` was touched; the run read only the `/workspace/in/d87` shard copies [claude].

## Skills used

None invoked [claude]. I followed AGENTS.md and the D87 protected-asset rule: read only, copies allowed, no corpus text committed.

## Still open

- The span dedup and re-audit (decision item 2) [claude].
- Write the novel-subset position mask as a file for claude-413 (R\*) to read [claude].
- Document boundaries for the counter (claude-412) [claude].

## Plain summary

The project checked what the overlap between its training data and its held-out test data actually is [claude]. The most widely repeated passages are boilerplate from one prompt template, which is harmless. But nearly every affected test record, 1,868 of 1,936, also repeats a passage that appears in only one or two training records, which looks like copied content [claude]. That is far over the 10% limit set in advance. So the next step is to remove the training records that share long passages with the test set, then measure again [claude]. Until then, test scores are also reported on the part of the test set that does not follow phrases seen in training [claude].
