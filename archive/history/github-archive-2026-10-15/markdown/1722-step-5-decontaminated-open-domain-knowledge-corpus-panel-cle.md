# #1722 — Step 5: decontaminated open-domain knowledge corpus, panel-clean subset and the 29M knowledge A/B pod script (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:14:09Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T07:11:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1722
- **Merged:** 2026-10-05T07:11:52Z
- **Merge commit:** 8308d52f5dc1b9278a67937f80ee618922999bfa
- **Base / Head:** main / UOR-Foundation:codex/step5-knowledge-corpus
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. This PR implements Step 5 of [final-assessment.md](docs/research/barrier-assessment-2026-10-05/final-assessment.md). The gate is Step 0a (#1708), which found that knowledge-absent is the largest failure class (65 of 186 failing rows at 96M).

The design, licence and frozen decision rule are in [docs/research/step5/knowledge-ab.md](docs/research/step5/knowledge-ab.md). **This PR claims no model result.** The lab lead runs the full A/B on a GPU pod.

## What changed

**`uor_r4_training::knowledge_corpus`** (panel decontamination)
- Uses the existing M-world probe rule: `EXCLUSION_NGRAM = 8`, with the same word normalisation.
- Also excludes the whole word sequence of 4–7-word panel turns, so short questions are not left uncovered.
- Hash hits are confirmed exactly, so a hash collision never drops an article.
- Also cleans article text.

**`prepare-knowledge-corpus`** (new Rust bin)
- Reads jsonl, or parquet (`--features parquet-input`, wikimedia schema).
- Writes UORT `train/` and `heldout/` stores. Every 200th kept article goes to `heldout/`, which is used for the knowledge-NLL manipulation check.
- Ends each article with `<|eos|>`. Each article must decode back to its exact text.
- Writes `excluded.jsonl`: every dropped article, with the matched words and the panel request id.
- Writes `corpus.json`, which binds the input SHA-256, tokenizer, panel files, licence and counts.
- The root is claimed with `report_output::claim`, then sealed and verified.

**Corpus choice**
- Simple English Wikipedia 20231101 (`wikimedia/wikipedia`, pinned commit).
- Licence: **CC BY-SA 4.0**. It is recorded in `corpus.json` and the doc. The derived stores are not published.
- The doc also lists the corpora the repo could already produce, and why each falls short.

**Panel-clean subset** (`docs/research/step5/panel-clean/`)
- All 232 rows are labelled with a reason: 143 clean, 69 ill, 20 underspecified.
- `clean-ids.txt` holds the 143 clean ids. `wellposed-ids.txt` holds the 163 ids that are clean or underspecified.
- `check_clean.py` checks the ids and their order against the panel files. It also checks that all 55 rows 0a flagged as ill are labelled `ill`.
- Existing grades re-read on the clean subset:

| Model | All 232 rows | Clean 143 rows |
|---|---|---|
| 96M-C | 46 | 20 |
| 29M-B | 43 | 19 |

The full-panel score is inflated by ill-posed rows.

**`scripts/pod/step5-knowledge.sh` and `scripts/pod/step5-tabulate.py`**
- Arms A and K on the 29M lr5e-4 recipe, 2 seeds each, 433.8M matched tokens per run.
- Arm K scales the current mix by 0.75 and adds 0.25 Simple Wikipedia.
- Each base then gets the Arm C fine-tune. Evaluation covers knowledge NLL, D19 sessions with the sieve on and off, panel replies and the qwen2.5:7b grade, and ends with the frozen decision.

## Tests (at this head, 076d0559, rebased on main 0df35259)

| Check | Result |
|---|---|
| `cargo fmt --check` | Clean for the touched files. The workspace run also reports a diff in `relation_compiler.rs`, which is already on main (#1707) and untouched here. |
| `cargo test -p uor-r4-training --lib knowledge_corpus` | **6 passed** |
| `cargo test -p uor-r4-training --bin prepare-knowledge-corpus` | **2 passed** |
| Warnings | None new in touched files. The 4 dead-code warnings are in other modules on main. |
| `python3 docs/research/step5/panel-clean/check_clean.py` | ids match (232, in order). All 55 rows 0a flagged as ill are `ill` here. |
| `bash -n scripts/pod/step5-knowledge.sh` | OK |
| SHA-256 re-check on the laptop | The script's values match for the tokenizer, the 5 panel files, the chat-v0-p2 mask, the dev manifest, the sieve compiler and the trunk config. |

**Not run:** the parquet test `wikimedia_parquet_rows_are_read_in_order`. The `parquet-input` dependencies are not cached for an offline build on the laptop. The pod script's `build` stage runs this test before any data is prepared.

## Smoke

Laptop, 2 threads, 22 s wall. The job was registered and removed afterwards.

**Corpus preparation**
- Input: a 600-row synthetic jsonl. Row 7 has a verbatim `heldout-000` turn appended, row 13 has the 5-word `talk-01` turn appended, and row 20 is a stub.
- Exclusion built against all 5 panel files: 464 requests, 472 turns, 1,563 distinct 8-grams.
  - 373 turns covered by 8-grams.
  - 52 turns matched whole.
  - 47 turns under 4 words, so not covered.
- Kept: 595 train articles (134,276 tokens) and 2 held-out articles (463 tokens).
- Dropped:
  - 1 short (the stub).
  - 1 panel 8-gram: `heldout-000`, "create a drama about a group of people".
  - 1 panel short request: `talk-01`, "hi how are you today".
  - 0 round trip.
- The root was sealed.

**Training and evaluation**
- A tiny `geometric-stack train` (CPU, 30 steps) ran on the exact four Arm K streams and weights: TinyStories / TinyDialogues / chat-v0-p2 / knowledge = 0.45 / 0.1125 / 0.1875 / 0.25. `report.json` records those weights.
- `geometric-stack evaluate` on the knowledge held-out store: full NLL 7.487 over 448 targets.
- `step5-tabulate.py` read that file and reported `DECISION: INCOMPLETE` because grades are missing.

This proves only that the data path works. It is not a model result.

## Pod run script

`scripts/pod/step5-knowledge.sh` in this PR holds every argument. Its header lists the inputs to upload, with their SHA-256. Target: one A100 80 GB.

```bash
git clone https://github.com/UOR-Foundation/uor-r4.git /root/uor-r4
cd /root/uor-r4 && git checkout 076d0559afbc08e76f4013d89e44e7157a1326d2
export REPO=/root/uor-r4 D=/root/data R=/root/runs/step5 GPU=0 THREADS=16 CUDA_COMPUTE_CAP=80
mkdir -p /root/runs
# all = fetch build data knowledge base ft eval replies
# fetch: Simple English Wikipedia 20231101 (~157 MB) by curl from the pinned HF commit
#   https://huggingface.co/datasets/wikimedia/wikipedia/resolve/cf26be8eef6bc935da5d2281be5aad4813e49e90/20231101.simple/train-00000-of-00001.parquet
#   checked against SHA-256 31bded16768a47c286becd292079122f5d7d4397a17b87d4250a00ccd581e6f0 (Hub LFS pointer);
#   bytes + sha256 recorded in $D/knowledge/raw/FETCH.txt
nohup bash scripts/pod/step5-knowledge.sh all > /root/runs/step5-all.out 2>&1 &
# then on the laptop (Ollama qwen2.5:7b), after copying $R/replies-* to ~/uor-r4-local/step5/runs:
#   R=~/uor-r4-local/step5/runs bash scripts/pod/step5-knowledge.sh grade
#   R=~/uor-r4-local/step5/runs bash scripts/pod/step5-knowledge.sh tabulate
```

Key settings in the script:
- **Base runs:** `arch=geometric width=576 heads=8 layers=10 pattern=rrarrarrar context=384 read=l2 rotation=true key_shift=false steps=70609 batch=16 lr=0.0005 warmup=200 device=cuda tf32=true data_parallel=1 max_seconds=21600`
- **Fine-tune:** Arm C `dialogue-train` with `pointer=32 protocol=2 policy=full_prefix steps=4000 lr=0.0003`
- **Sessions:** `m-world session world=v2 log_recall=sieve|off`
- **Replies:** `chat-grade reply protocol=2 max_new_tokens=64`

**Expected wall time** (a projection, not a measurement): about 12 h on one A100 80 GB, with a range of 9–14 h.

| Stage | Expected time |
|---|---|
| fetch, build, data, knowledge | about 0.5 h |
| 4 base runs | about 1.5–2.5 h each |
| 4 fine-tunes | about 0.4 h each |
| eval | about 1.5 h |
| replies | about 0.5 h |

New storage is about 2 GB.

## Caveats
- **Wikipedia checksum.** The SHA-256 is the value from the Hub's LFS pointer. It was not computed locally, because nothing was downloaded here.
- **Parquet path.** The parquet reader runs for the first time on the pod.
- **Hardware.** The 29M rung was trained on an RTX 4090, but both arms here run on A100 TF32. Compare the arms with each other, not with the ladder numbers.
- **Statistical power.** Power is weak: 2 seeds, a 232-row panel and a single grader (qwen2.5:7b).
- **Clean-subset labels.** One annotator (Claude) labelled the clean subset. It has not been reviewed independently.
- **Licence.** It is unsettled whether CC BY-SA reaches trained weights. This is recorded for the owner.
- **Untested stages.** `m-world session` and the fine-tune were not smoke-run with new models. Their arguments are copied from the validated 29M/96M scripts.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5989395382 — 2026-10-05T06:35:40Z — Casey-allard

Adversarial review at 076d0559afbc08e76f4013d89e44e7157a1326d2:

**Verdict: changes requested.** Two small fixes are needed before any pod time is spent, because the decision rule is frozen and cannot be changed after the run. The Rust data path itself looks correct.

## What I ran at the exact head
I ran these in a scratch copy made with `git archive`, using `CARGO_TARGET_DIR=~/.cache/uor-claude-target` and `-j 2`.

- **rustfmt:** clean on both new Rust files. Workspace `cargo fmt --check` fails only on `relation_compiler.rs`, which was already failing on main.
- **`cargo test -p uor-r4-training --lib knowledge_corpus`:** 6/6 passed.
- **`cargo test -p uor-r4-training --bin prepare-knowledge-corpus`:** 2/2 passed.
- **The same test with `--features parquet-input`:** 3/3 passed, including `wikimedia_parquet_rows_are_read_in_order`. This was not run offline; it needed the network to fetch dependencies. So the parquet reader's unit test is now measured, not only deferred to the pod. The real 157 MB file has still not been read.
- **`check_clean.py`:** ids match (232, in order); all 55 rows 0a flagged as ill are labelled ill here; 143 clean, 69 ill, 20 underspecified.
- **Binary smoke:** I ran the debug binary on a 399-row jsonl with the real tokenizer and all 5 panel files.
  - Panel counts: 464 requests, 472 turns, 1,563 distinct 8-grams.
  - The planted "why do leaves change color in the fall" article was dropped as `panel_8gram` with request `ask-01`. The stub was dropped as short.
  - The root contains `attempt.json` and `manifest.json` and is sealed.
- **Uncovered turns:** I counted them independently with an approximate word splitter. On the 232 panel there are 23 turns under 4 words, all `salutation_only`, and none is in the clean subset. This matches the claim in §3.
- **Wikipedia SHA-256:** the LFS pointer's oid is the SHA-256 of the file content, so the pinned value is correct by construction.

## Interfaces checked against source
- `evaluate` writes `evaluation.json` with `full.nll`.
- `m-world session` takes `world=v2`, `model_root`, `op_policy`, `log_recall` and `arms=default`. Its report has `arms.default.by_category.<Cat>.{pass,of}`.
- `chat-grade grade-replies` writes `report.json`. Its `rows[].grades.fluent/relevant` are `Option<bool>`, so the tabulator's `bool(a and b)` is correct.
- The base train report has `final.nll`.
- Arm K's weights sum to 1.

## Blocking
1. **Matched tokens are asserted, not checked.**
   - `geometric-stack train` stops at `max_seconds=21600`, still writes a sealed model and records `stopped_early` and `completed_steps`. `dialogue-train` behaves the same way at 10800 s.
   - `fresh()` then treats the root as done, and `step5-tabulate.py` never reads these fields.
   - So a base that hits the cap (on a slower or shared GPU, or with a slow sampler on the 4-stream arm) would silently break the "433,821,696 tokens per run" claim, and the frozen rule would still issue a verdict.
   - **Fix:** in the tabulator, require `completed_steps == 70609` and `stopped_early == false` for all 4 bases, and `completed_steps == 4000` for all 4 fine-tunes. Otherwise return INCOMPLETE.
2. **The frozen decision table has overlapping conditions.**
   - NULL includes "`d_1` and `d_2` of opposite sign". PROMOTE does not exclude it.
   - Example: `d = (+22, -1)` gives `mean_d = 10.5`. If p < 0.05 and both clean deltas are positive, the tabulator returns PROMOTE, while §6 also says NULL.
   - The order of conditions in `step5-tabulate.py` decides this silently.
   - **Fix:** before the run, add `d_1 > 0 and d_2 > 0` to the panel criterion in both the doc and the code, or state the precedence explicitly.

## Non-blocking
- **Missing manipulation-check data reads as a failure.** If knowledge NLL is missing, `manip` evaluates False and the result is "INSTRUMENT … did not land". It should be INCOMPLETE.
- **The pooled McNemar is anti-conservative.** It treats the same request under two seeds as independent pairs (464). Report the per-seed p values alongside it, or say this in §7.
- **A tokenizer failure is miscounted.** In `prepare-knowledge-corpus`, a failed per-thread tokenizer rebuild in `map_init` is counted as `RoundTrip` instead of returning an error. It cannot happen in practice, since the same bytes already parsed, but an error would be more honest.
- **A doc example is wrong.** The `knowledge_corpus.rs` module doc gives "Why do leaves change color in the fall" as a short request. It has 8 words and is matched by the 8-gram rule, as the smoke shows. Use a 4–7-word example such as "What do bees make?".
- **No manifest check on `mixed-c`.** In `stage_data`, `mixed-c/manifest.json` is not SHA-checked, although `dialogue-train` reads it. Tokens and mask are checked, so this is low risk.
- **Knowledge NLL may be optimistic.** Simple Wikipedia has many templated near-duplicate pages (years, municipalities). An article-level 1/200 split leaves near-duplicates in training, so knowledge held-out NLL is optimistic. It is fine as a manipulation gate; do not quote it as knowledge.
- **Clean-subset labels lack independent review.** One annotator labelled them, as already stated. A second reader of the 20 underspecified rows would help, since the clean guard depends on these labels.

No `unwrap`, `expect` or panic outside tests on recoverable paths. The root is claimed after validation and before output, then sealed and verified. Hits are confirmed exactly, so a hash collision cannot drop an article.


### comment 5989576512 — 2026-10-05T06:51:29Z — Casey-allard

Changes at `31e18cc571f80444ffa0543c69a0e4555d8e11ae`, which address the review at `076d0559`. The frozen rule was changed before any pod run; no run has started.

## Blocking 1: matched tokens are now checked
**`scripts/pod/step5-tabulate.py`, new step 0 (completeness).** The tabulator returns **INCOMPLETE** and gives no verdict unless all of these hold:
- all 4 base reports: `completed_steps == 70609` and `stopped_early is False`;
- all 4 Arm C fine-tune reports: `completed_steps == 4000`;
- base knowledge NLL, grades and sieve-on sessions with an Instruction category are present for all 4 runs.

Each problem is printed and recorded under `completeness.problems` in the JSON.

**`scripts/pod/step5-knowledge.sh`.** A sealed base or fine-tune root now counts as done only at its full step count. The check is `completed ROOT STEPS STRICT`, which reads `report.json`; STRICT also requires `stopped_early=false` and applies to the bases. When a sealed root is capped, `fresh_train` does three things:
- moves it aside to `ROOT.capped-<UTC>`, never deleting it;
- moves aside the roots built from it (`*.stale-<UTC>`): ft, knowledge evals, sessions, replies and grades;
- continues the run in a fresh root of the same name with `resume=<aside>/checkpoint`.

`stage_ft` also refuses to start before its base has completed 70,609 steps. Unsealed roots are still refused, as before.

## Blocking 2: the decision table no longer overlaps
The panel criterion now requires **`d_1 > 0` and `d_2 > 0`**, together with `mean_d ≥ 10`, `p < 0.05` and the clean guard. The rule order is explicit in both `knowledge-ab.md` §6 (an Order column) and the tabulator; the first outcome reached is the decision:

1. INCOMPLETE
2. INSTRUMENT
3. PROMOTE
4. BLOCKED
5. NULL
6. WEAK

PROMOTE/BLOCKED and NULL are now disjoint. The review's example `d = (+22, −1)` is NULL.

## Non-blocking items addressed
- **Missing knowledge NLL** is INCOMPLETE (step 0), not INSTRUMENT.
- **Per-seed McNemar.** The exact McNemar for each seed (232 pairs) is printed and stored in `mcnemar_per_seed` beside the pooled p. The pooled p stays decisive as frozen. §7 now says the pooled test is anti-conservative.

Not changed in this push (Rust, non-blocking): the `map_init` tokenizer error, the module-doc example, the `mixed-c/manifest.json` SHA, and a second reader for the clean labels.

## Checks run (no model run)
- **`bash -n step5-knowledge.sh`:** OK.
- **Shell harness:** it sources the script's functions and runs them on synthetic sealed roots:
  - a complete base is skipped;
  - a capped base (51,234 steps, `stopped_early`) is moved aside with its dependent ft and resumes from its checkpoint;
  - a base with `stopped_early` at the full step count is rerun;
  - an ft at 4,000 steps is skipped;
  - an absent root runs;
  - an unsealed root is refused.
- **Synthetic tabulator cases** (232 real panel ids, clean list) gave the expected decisions:
  - promote → PROMOTE;
  - `d = (+22, −1)` with p = 0.00145 and both clean deltas positive → **NULL** (before this change: PROMOTE);
  - base capped → INCOMPLETE;
  - ft 3,999 steps → INCOMPLETE;
  - knowledge NLL missing → INCOMPLETE;
  - equal NLL → INSTRUMENT;
  - Instruction 80 vs 100 → BLOCKED;
  - `mean_d = 7` → WEAK.
- **`scripts/check_claim_wording.py`:** passed.

No Rust changed, so no cargo build was run.


### comment 5989601445 — 2026-10-05T06:53:41Z — Casey-allard

Adversarial review at 31e18cc571f80444ffa0543c69a0e4555d8e11ae:

**Verdict: APPROVE.** Both blocking items are closed at this head. Advisory only: this PR has not been merged.

How I checked: I read the diff `076d0559..31e18cc5` (3 files: tabulator, run script, design doc §6/§7) and ran tests from a `git archive` copy of the exact head in `$TMPDIR`. No cargo build was needed because no Rust changed.

**1. Matched tokens / capped runs.** Closed.
- The tabulator now returns INCOMPLETE before any verdict unless all of these hold:
  - each base has `completed_steps == 70609` and `stopped_early is False`;
  - each fine-tune has `completed_steps == 4000`;
  - every base knowledge NLL is present;
  - all 4 grades are present;
  - all 4 sieve-on Instruction counts are present.
- I checked that the trainer actually writes these fields. In `geometric-stack.rs` at this head, both `train` (L2182-2183) and `dialogue-train` (L4320-4321) write `completed_steps: progress.step` and `stopped_early`. A max_seconds stop saves a checkpoint and keeps it (L2137-2160 and L4263-4274).
- Resuming restores `progress.step`, and the loop is `while progress.step < steps`. So a resumed root reports the total step count, and resuming a root that stopped early at exactly 70,609 steps just finalizes it.
- `dialogue-train` accepts `resume=` beside `init=`: when resuming, the init branch is skipped.
- Run script:
  - `fresh_train` moves a capped sealed root to `.capped-<UTC>` along with its `.log`, and moves the roots built from it to `.stale-<UTC>`. Nothing is deleted. It then sets `resume=<aside>/checkpoint`.
  - An unsealed root is still refused.
  - `stage_ft` refuses to start until its base has completed 70,609 steps.
- Harness: I sourced the extracted functions and ran them on synthetic roots. Results:
  - complete base: skipped;
  - capped base (51,234 steps, stopped early): moved aside, its ft and grade roots marked stale, resume argument set;
  - base at full steps but with `stopped_early=true`: moved aside and resumed;
  - fine-tune at 4,000 steps: skipped;
  - absent root: runs;
  - unsealed root: refused with rc=1.

**2. Overlapping decision table.** Closed.
- The panel criterion now requires `d_1 > 0 and d_2 > 0`, `mean_d >= 10`, `p < 0.05` and the clean guard.
- The code applies the order INCOMPLETE → INSTRUMENT → PROMOTE → BLOCKED → NULL → WEAK, matching the new doc table.
- Synthetic tabulator cases (real 232 ids and the 143 clean ids):

| Case | Decision |
|---|---|
| promote | PROMOTE |
| `d=(+22,-1)`, pooled p=1.9e-5, clean deltas (+9,+1) | NULL (per-seed p 4.8e-7 / 1.0) |
| capped base | INCOMPLETE |
| full steps but `stopped_early` | INCOMPLETE |
| ft at 3,999 steps | INCOMPLETE |
| missing knowledge NLL | INCOMPLETE |
| equal NLL | INSTRUMENT |
| Instruction guard fails | BLOCKED |
| `mean_d=7` | WEAK |
| `d=(12,8)` | PROMOTE |
| gains only outside the clean set | WEAK |

- The per-seed McNemar is reported and kept out of the decision rule, as frozen. §7 records that the pooled test is anti-conservative.

**Other checks:** `bash -n` passes.

**Non-blocking notes:**
- The completeness check does not confirm that every grade report covers the same 232 ids. A short grade file would raise a KeyError, not return INCOMPLETE. This was already true before the fix.
- `fresh_train` runs inside `|| continue`, so `set -e` is off inside it. If an `mv` fails, the trainer's report-root claim still refuses the run, so it fails loudly anyway.
- Rarely, a resume that has no steps left (cap reached on the last step) calls `step_timing` on an empty list; I did not check whether that can panic.
- I re-checked none of the earlier non-blocking Rust items (tokenizer `map_init` error, module-doc example, `mixed-c` manifest SHA). They stay open as the author noted.


### comment 5989811653 — 2026-10-05T07:11:03Z — Casey-allard

Lab-lead exact-head check at 31e18cc571f80444ffa0543c69a0e4555d8e11ae: the adversarial review approved with no blocking items. Truncated and capped runs are now refused, or moved aside and re-run, and decision rules can no longer overlap. Queuing.


---
_Comment/review ids in this file: 5989395382, 5989576512, 5989601445, 5989811653_

