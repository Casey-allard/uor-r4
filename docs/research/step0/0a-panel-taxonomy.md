# Step 0a: failure taxonomy of the open 232-request panel

References #820. Instrument from [final-assessment.md, Step 0](../barrier-assessment-2026-10-05/final-assessment.md).
This is training-free. No model ran. The instrument reads existing graded reports only.

## Inputs

| Model | Grade report (open fixture) | report.json sha256 | Acceptable | Failing |
|---|---|---|---|---|
| 96M chat-100m-C (96,023,745 params) | `~/uor-r4-local/ladder/grades/chat-100m-C-powered-7b/report.json` | `3d5a2914…ccf6` | 46/232 | 186 |
| 29M chat-29m-B lr5e-4 (28,958,761 params) | `~/uor-r4-local/ladder/grades/chat-29m-B-lr5e-4-powered-7b/report.json` | `72ab32a2…59d4` | 43/232 | 189 |

Both reports were graded by qwen2.5:7b on everyday-32 + heldout-200-a/b (greedy, 64 new tokens). A row is
failing when the grader did not judge it both fluent and relevant. Exact model, report and label hashes are in
[`0a-panel-taxonomy/taxonomy.json`](0a-panel-taxonomy/taxonomy.json).

**Path note (2026-10-09).** The ladder store `~/uor-r4-local/ladder/` no longer exists on the laptop: it is
archived as `icloud:UOR-R4/results/deepseek/ladder.tar` (3,306,919,424 B, md5
`ed9456d63472b7a424fe6ca62d36baa8`); restore it with
`~/.local/share/uor-r4/bin/cloud-store fetch ladder <dest>` and the two report paths above reappear under
`<dest>/ladder/grades/`. The sha256 values in the table are unchanged, and the panel files the reports graded
are in the same archive under `ladder/panel/`. The two reports, the panel and a re-runnable classifier are
also in `icloud:UOR-R4/results/deepseek/reply-panel-open-2026-10-09.tar`; see the
[open reply panel diagnostic](../../labs/open-reply-panel-2026-10-09/README.md), which reproduces 43/232 and
46/232 from these reports and confirms this taxonomy's diffusion finding on the grader's own verdict fields.

## Method

- **Annotator:** Claude (Opus 5.5) read every failing row: the user turns, any earlier assistant turn and the
  final reply. It did not use the qwen judge to assign labels.
- **Rubric:** fixed before any row was labelled ([`rubric.md`](0a-panel-taxonomy/rubric.md)). Each row gets one
  primary label: R (recall-anaphora-consistency), K (knowledge-absent), N (incoherent), I (instruction),
  T (truncation at 64 tokens) or O (other).
- **Precedence:** N > R > I > K > T > O. R sits second on purpose. This is generous to retrieval, so a low R share
  is a conservative result for this rule.
- **Per-row flags:**
  - `ctx`: the request depends on earlier-turn content or content supplied in the request.
  - `r_any`: R applies even when N won the precedence.
  - `ill`: the request names material that the panel does not contain. Examples are "Dear Dr. X," salutations,
    "the following code:" with no code, `[city]` placeholders and "read the passage below" with no passage.
- **Self-agreement:** a seeded (20261005) random set of 40 96M failing rows was dumped with ids but without the
  first labels or grader verdicts. The annotator labelled it a second time.
- **Tabulation:** small Python ([`tabulate.py`](0a-panel-taxonomy/tabulate.py)) counts the fixed TSV labels.
- **Report root:** `~/uor-r4-local/step0/a-taxonomy/`, claimed exclusively and sealed read-only with
  `MANIFEST.sha256`.

## Results

### Primary label, share of failing rows

| Category | 96M-C (n=186) | 29M-B lr5e-4 (n=189) |
|---|---|---|
| recall-anaphora-consistency (R) | **4 (2.2%)**, Wilson 95% [0.8%, 5.4%] | **5 (2.6%)**, [1.1%, 6.0%] |
| knowledge-absent (K) | 65 (34.9%) | 50 (26.5%) |
| incoherent (N) | 26 (14.0%) | 48 (25.4%) |
| instruction (I) | 17 (9.1%) | 19 (10.1%) |
| truncation at 64 tokens (T) | 19 (10.2%) | 11 (5.8%) |
| other (O) | 55 (29.6%) | 56 (29.6%) |
| *R upper bound (`r_any`)* | 5 (2.7%) | 5 (2.6%) |
| *context-dependent failing rows (`ctx`, any label)* | 14 (7.5%) | 11 (5.8%) |
| *ill-posed requests (`ill`, any label)* | 47 (25.3%) | 49 (25.9%) |

Excluding ill-posed rows (96M n=139, 29M n=140): the R share is 2.2% at 96M and 3.6% at 29M. K is the largest
class at both sizes: 65/139 at 96M and 50/140 at 29M.

### By panel category (96M-C)

| Panel category | Failing | R | K | N | I | T | O |
|---|---|---|---|---|---|---|---|
| follow_up (two-turn) | 7 | 2 | 1 | 2 | 0 | 1 | 1 |
| heldout_first_turn | 162 | 2 | 58 | 20 | 14 | 18 | 50 |
| simple_instruction | 7 | 0 | 2 | 2 | 2 | 0 | 1 |
| simple_question | 6 | 0 | 4 | 1 | 1 | 0 | 0 |
| smalltalk | 4 | 0 | 0 | 1 | 0 | 0 | 3 |

### By panel category (29M-B lr5e-4)

| Panel category | Failing | R | K | N | I | T | O |
|---|---|---|---|---|---|---|---|
| follow_up (two-turn) | 7 | 4 | 0 | 2 | 0 | 0 | 1 |
| heldout_first_turn | 166 | 1 | 46 | 41 | 16 | 11 | 51 |
| simple_instruction | 7 | 0 | 1 | 2 | 2 | 0 | 2 |
| simple_question | 7 | 0 | 3 | 3 | 1 | 0 | 0 |
| smalltalk | 2 | 0 | 0 | 0 | 0 | 0 | 2 |

**R rows:**
- 96M: follow-03, follow-06, heldout-083 and heldout-128. follow-07 is R-any but labelled N.
- 29M: follow-02, follow-03, follow-06, follow-07 and heldout-128.

### Self-agreement

- **Agreement:** 39/40 rows. Observed agreement 0.975, chance agreement 0.256, **Cohen's kappa = 0.966**.
- **The one disagreement:** heldout-178 was K on the first pass and N on the second. That reply is a circular,
  partly ungrammatical sentence, which sits on the K/N boundary.

## Decision rule (frozen in the plan, made numeric in the task)

At 96M, R = 4/186 = **2.2%**, with a 95% upper bound of 5.4%. The `r_any` upper bound is 2.7%. Both are below
15%, so the frozen outcome is:

> **retrieval cannot move the open panel; panel lever = corpus/knowledge (Step 5)**

This outcome does not depend on my R judgments:
- **Structural ceiling:** the panel has only 8 multi-turn rows (224/232 are single-turn). All failing
  context-dependent rows together are 14/186 = 7.5% at 96M. The share stays under 15% even if every
  context-dependent failure is relabelled R.
- **Labelling bias:** the R-favouring precedence works against this outcome, not for it.

## Further observations (measured on these two reports only)

1. **Ill-posed requests.** About a quarter of failing rows (47 at 96M, 49 at 29M) are ill-posed. The heldout-200
   extract keeps only the first line of multi-line source messages, so it includes letter salutations, "the
   following X:" with X missing, and `[placeholder]` templates. No model lever can fix these rows. The panel
   should be repaired or these rows reported separately before the M4 target (≥ 60/232) is read.
2. **Scale shifts incoherence toward knowledge failures.** From 29M to 96M, N falls from 48 to 26 and T rises
   from 11 to 19, while K rises from 50 to 65. As fluency improves, the remaining failures become missing or
   invented content. This fits the Step 5 corpus/knowledge lever. It is one pair of models with one seed each
   and one annotator, so it is suggestive, not established.
3. **Truncation.** 19 failing rows at 96M are on track and cut at the 64-token budget. A larger
   `max_new_tokens` might convert some of them, but that changes the fixed evaluator identity and must be a
   declared, separate evaluation.
4. **Grader false negatives.** The annotator judged a few rows acceptable that the grader failed. Examples are
   do-03 "The moon made me smile." and heldout-176 (three bullets as asked). They are labelled O. Their count is
   small (≤ 5 per model).
5. **Empty code fences.** Several replies to coding requests open a code fence (```) and stop (O at 96M:
   heldout-011, -022, -155 and -172). This looks like a systematic code-block emission failure. It is an
   observation, not diagnosed here.

## Caveats

- **Single annotator.** The second pass ran in the same session, with row ids visible and the first labels earlier
  in the annotator's context. Kappa 0.966 is a self-consistency figure, not independent inter-rater reliability.
  A second human or model annotator is needed to establish reliability.
- **"Failing" set comes from the grader.** The panel judge (qwen2.5:7b) decides which rows are failing. The final
  assessment §6 already notes the risk that comes from depending on it.
- **K vs N boundary.** This boundary is the least stable (the only disagreement). K and N are not separately
  decision-relevant here. Their combined share is 49% (96M) and 52% (29M).
- **Scope.** The decision applies to this open panel only. It says nothing about the D19 MQAR/recall batteries.
  The plan keeps retrieval work gated on sieve-off MQAR there.

## Files

- `0a-panel-taxonomy/rubric.md`: rubric, fixed first.
- `0a-panel-taxonomy/labels-96m-pass1.tsv`: 96M labels, 186 rows, with id, label, ctx, r_any, ill and note.
- `0a-panel-taxonomy/labels-29m.tsv`: 29M labels, 189 rows.
- `0a-panel-taxonomy/blind-subset-ids.txt` and `labels-96m-pass2-blind.tsv`: the 40-row second pass.
- `0a-panel-taxonomy/tabulate.py`: the tabulator.
- `0a-panel-taxonomy/taxonomy.json`: sealed results.
- `0a-panel-taxonomy/MANIFEST.sha256`: sha256 of each file in the sealed report root.

Cost: no compute beyond reading JSON and one tabulation (seconds, 1 thread). About 1.5 h of annotator wall time.
New storage is about 40 KiB.
