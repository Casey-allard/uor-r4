# Step 5: open-domain knowledge corpus and a matched-token 29M A/B

References #820. This implements Step 5 of [final-assessment.md](../barrier-assessment-2026-10-05/final-assessment.md). The gate is Step 0a ([0a-panel-taxonomy.md](../step0/0a-panel-taxonomy.md)). In 0a, knowledge-absent (K) is the largest failure class: 65 of 186 failing rows at 96M and 50 of 189 at 29M. Recall accounts for 2.2%.

This document fixes the corpus, the decontamination, the arms and the decision rule before any run. Nothing here is a model result. The only things executed so far are the laptop smokes listed in the PR.

## 1. Knowledge corpora the repository can already produce

| Tool | Corpus | Licence | Fit for Step 5 |
|---|---|---|---|
| `prepare-text-corpus` (`format=tinystories\|tinydialogues`) | TinyStories V2, TinyDialogues | as recorded for the ladder | Already in the mix. Narrative and dialogue, not world knowledge. |
| `prepare-chat-parquet` | SmolTalk magpie-ultra (Apache-2.0), UltraChat 200k (MIT) | permissive | Already chat-v0/chat-v1. Instruction data with incidental knowledge. |
| `prepare-simplewiki-tokens` | Simple English Wikipedia 20231101, first 32 × 1,024 tokens | CC BY-SA 4.0 | A teacher fixture only. It uses the SmolLM2 tokenizer, not the project's 4,096-id tokenizer, and holds 32k tokens. |
| `scripts/fetch_simple_wiki_20k.py`, `fetch_d3_corpus.py` | Simple English Wikipedia rows through the datasets-server API | CC BY-SA 4.0 | Python fetchers of samples of at most about 20k articles. Not a model-data path. |
| `geometric-stack corpus` | Rust sources from a Cargo registry | per crate | Code, not open-domain knowledge. |

No repository path produced a full, decontaminated, project-tokenizer knowledge store. This PR adds one: `prepare-knowledge-corpus` (Rust) with its library module `uor_r4_training::knowledge_corpus`.

## 2. Chosen corpus and licence

**Simple English Wikipedia, 1 November 2023 dump**, as exported by Wikimedia on the Hugging Face Hub:

- Dataset `wikimedia/wikipedia`, config `20231101.simple`.
- One file, `20231101.simple/train-00000-of-00001.parquet`, about 157 MB.
- Pinned to commit `cf26be8eef6bc935da5d2281be5aad4813e49e90`, the commit that added the file.
- Expected SHA-256 `31bded16768a47c286becd292079122f5d7d4397a17b87d4250a00ccd581e6f0`. This value is taken from the Hub's LFS pointer. It was not computed locally, because nothing was downloaded on the laptop. The pod script checks it and records size and SHA-256 after the fetch in `FETCH.txt`.

**Why this corpus:**
- It is human-written encyclopaedic text in a simple register, close to the register of the open panel ("Why do leaves change color in the fall?").
- Its about 0.24M articles fit one A/B at about one epoch (see §4).
- The project already records this source and licence as its natural partition ([transformerless/BASELINE.md](../../transformerless/BASELINE.md), SPDX `CC-BY-SA-4.0`).

**Licence: CC BY-SA 4.0** (Wikipedia text is also available under GFDL).
- Attribution and share-alike apply to the corpus and to any redistributed derived text, such as the prepared token stores.
- `corpus.json` records the source, commit and licence.
- The prepared stores stay on the pod and laptop. They are not published.
- Whether trained weights count as an adaptation under CC BY-SA is not settled. This is recorded for the owner and does not block an internal experiment.

**Rejected:**
- Full English Wikipedia: about 20 times larger and in a harder register.
- Cosmopedia: model-generated text.
- FineWeb-Edu: web text with mixed underlying rights.
- Wikibooks: smaller.

## 3. Article preparation and decontamination

`prepare-knowledge-corpus` (see its header for every argument) does the following.

**Article text.**
- Each article becomes `title`, a blank line, then the body.
- `\r\n` is normalised, line ends are trimmed and blank-line runs are collapsed (`clean_article`).
- Articles with fewer than `min_words=20` words are dropped as stubs.

**Panel exclusion.** This uses the same rule as the existing M-world probe exclusion: `milestone_world_v2_probe::EXCLUSION_NGRAM = 8`, over the same word normalisation (lowercase; split on characters other than letters, digits and apostrophes).
- An article is dropped whole when any of its 8-word windows equals an 8-word window of any panel user turn.
- Panel turns of 4–7 words cannot produce an 8-gram. For those, the whole turn's word sequence is excluded instead. Otherwise short open-panel questions such as "What do bees make?" would be silently uncovered.
- Turns under 4 words are counted as uncovered and are not matched. In the 232-request panel these are 23 turns, every one a `salutation_only` row (e.g. "Hi Dr. Smith,"), and all of them are outside the clean subset (§5).
- Matching hashes windows for speed and confirms every candidate exactly. A hash collision therefore never drops an article.

**Panel files** used for the exclusion: `everyday-32`, `heldout-200-a`, `heldout-200-b`, `heldout-200` and `stretch-32` (464 requests, 472 turns, 1,563 distinct 8-grams). Every dropped article is listed in `excluded.jsonl` with the matched words and the panel request id.

**Round trip.** Every kept article must decode back to its exact text, and is followed by `<|eos|>`.

**Split.** Kept article `k` goes to `heldout/tokens.u16` when `k % 200 == 199`, about 0.5% of articles. Every other kept article goes to `train/tokens.u16`. The held-out store is never trained on. It measures knowledge NLL, which is the manipulation check below.

**Root.** The root is claimed exclusively, then sealed and verified.

## 4. A/B design (matched tokens)

The base recipe is the 29M lr 5e-4 rung ([ladder-runbook §3.3](../../compute/ladder-runbook.md)):
- Shape: width 576, 8 heads, 10 layers, pattern `rrarrarrar`, context 384, L2 read, rotation on, `key_shift=false`.
- Optimiser: 70,609 steps × batch 16 = **433,821,696 tokens** per run; lr 5e-4, warmup 200.
- Single GPU: `tf32=true`, `data_parallel=1`. Both arms use the same setting, so TF32 affects both arms equally. The arms are compared with each other, not with the 4090-trained rung.

| Arm | Streams and weights | Tokens drawn (TS / TD / chat-v0 / wiki) |
|---|---|---|
| **A** (current mix) | TinyStories 0.6, TinyDialogues 0.15, chat-v0-p2 0.25 | 260.3M / 65.1M / 108.5M / 0 |
| **K** (+knowledge) | the same three × 0.75 (0.45, 0.1125, 0.1875), Simple Wikipedia 0.25 | 195.2M / 48.8M / 81.3M / 108.5M |

- **Seeds:** 1 and 2. The base `seed=` seeds the initialisation and the window sampler. The fine-tune uses `data_seed=` equal to the same seed.
- **Wiki exposure:** about 108.5M tokens. The pod script prints the actual epoch count and stops above 4 epochs. Expected: about one epoch.
- **Chat fine-tune:** Arm C, the recipe of the 96M `chat-100m-C`, on all four bases.
  - Data: the `mixed-c` store, which is M-world parar ×3 followed by chat-v0-p2: 84,142,760 tokens, tokens SHA-256 `7282b7a6…`. This store was rebuilt on the laptop with `mix-chat-corpus` and matched the Arm C input bytes exactly.
  - Settings: pointer 32, protocol 2, `full_prefix`, 4,000 steps × 16, lr 3e-4, development seed 20260930.
- **Evaluation:**
  - **Knowledge held-out NLL:** `geometric-stack evaluate`, bases and fine-tunes.
  - **D19 grounded sessions** (`m-world session`, world v2, 300 conversations, 1,075 scored turns), sieve on and off, with the 29M session arguments.
  - **Open 232-request panel:** `chat-grade reply`, greedy, 64 new tokens, graded by qwen2.5:7b with `grade-replies`. These are the same grader and questions as every ladder panel. The report records the grader digest.

The run script is [`scripts/pod/step5-knowledge.sh`](../../../scripts/pod/step5-knowledge.sh). The tabulator is [`scripts/pod/step5-tabulate.py`](../../../scripts/pod/step5-tabulate.py).

**Projected cost on one A100 80 GB:** about 12 h wall (range 9–14 h), with 4 base runs of about 1.5–2.5 h each. This is a projection, not a measurement; the 29M rung was trained on an RTX 4090. New storage is about 2 GB.

## 5. Panel-clean subset (repairing 0a's ill-posed rows)

0a found 47 (96M) and 49 (29M) failing rows whose referenced material is missing. The heldout-200 extract keeps only the first line of each source message. 0a flagged only failing rows. [`panel-clean/panel-clean-232.tsv`](panel-clean/panel-clean-232.tsv) labels **all 232 rows** with a status and a reason.

| Status | Rows | Reasons |
|---|---|---|
| clean | 143 | |
| ill | 69 | `salutation_only` 24, `missing_material` 19, `fragment` 11, `unresolved_reference` 7, `placeholder` 5, `missing_instruction` 3 |
| underspecified | 20 | `constraints_only` 13 (format/length constraints with no topic), `statement_no_request` 7 |

- **Lists.** `clean-ids.txt` holds the 143 clean rows. `wellposed-ids.txt` holds the 163 rows that are clean or underspecified.
- **Check.** [`check_clean.py`](panel-clean/check_clean.py) verifies that the 232 ids match the panel files in order. It also verifies that every row 0a flagged ill (55 distinct ids across both label files) is `ill` here.
- **Repair.** These rows cannot be repaired from the panel itself. The missing text was never extracted. The repair is to read M4 on the clean subset alongside the full panel.

Existing reports re-read on the subsets (tabulation only, no new grading):

| Model (grade report) | all 232 | well-posed 163 | clean 143 |
|---|---|---|---|
| 96M chat-100m-C | 46 | 25 | 20 |
| 96M chat-100m-B | 45 | 26 | 21 |
| 29M chat-29m-B lr 5e-4 | 43 | 25 | 19 |
| 29M chat-29m-B lr 1e-3 | 37 | 23 | 16 |

**Observation (M, these four reports).**
- The grader accepted 21 of 96M-C's 46 acceptable replies on ill-posed rows, mostly replies to bare salutations.
- On the clean subset, the 29M-to-96M gain is 19 → 20 of 143.
- The full-panel score therefore overstates capability on well-posed requests. A model can also gain panel rows on ill rows without any knowledge gain.
- For that reason the decision rule below requires the clean-subset delta to be positive on both seeds.

## 6. Decision rule (frozen before any run)

Definitions:
- `acc(arm, s)` is the number of acceptable rows (fluent and relevant) on the 232-row panel for seed `s`.
- `d_s = acc(K, s) − acc(A, s)`.
- `mean_d = (d_1 + d_2) / 2`.
- `p` is the exact two-sided McNemar over the 464 pooled (request, seed) pairs. This is the decisive test. The per-seed exact McNemar p over each seed's 232 pairs is reported beside it but does not enter the rule (see §7).
- **Panel criterion:** `d_1 > 0` **and** `d_2 > 0`, `mean_d ≥ 10`, `p < 0.05`, and the clean guard.

The checks and outcomes, applied in this order. The first outcome reached is the decision.
0. **Completeness (matched tokens).** Every input the rule reads must be present and complete:
   - all 4 base train reports with `completed_steps = 70609` and `stopped_early = false` (433,821,696 tokens each);
   - all 4 Arm C fine-tune reports with `completed_steps = 4000`;
   - base knowledge held-out NLL for all 4 bases;
   - all 4 panel grade reports;
   - all 4 sieve-on D19 session reports, each with an Instruction category.

   Otherwise the outcome is **INCOMPLETE** and no verdict is issued. A run stopped by `max_seconds` still seals a model, but it is not the matched-token arm this rule was frozen for. The run script does not treat such a root as done: it moves the root aside (never deletes it), moves aside the roots built from it, and continues the run from its checkpoint in a fresh root.
1. **Manipulation check.** K's base knowledge held-out NLL is at least 0.10 nats below A's on both seeds. If this fails, the outcome is **INSTRUMENT**: the corpus did not reach the model. Check the data path first. The panel is not read. Missing NLL is INCOMPLETE (step 0), not INSTRUMENT.
2. **Clean guard** (part of the panel criterion). `d_s` on the 143 clean rows is greater than 0 on both seeds.
3. **Instruction guard.** The final assessment requires "instruction following not worse". Here that means the pooled D19 Instruction passes (sieve on, 2 × 98 turns) satisfy `K ≥ A − max(3, |A_s1 − A_s2|)`.

| Order | Outcome | Condition | Next action |
|---|---|---|---|
| 0 | **INCOMPLETE** | completeness fails | Complete the missing or capped runs; no verdict. |
| 1 | **INSTRUMENT** | manipulation check fails | Check the data path; the panel is not read. |
| 2 | **PROMOTE** | panel criterion (`d_1 > 0`, `d_2 > 0`, `mean_d ≥ 10`, `p < 0.05`, clean guard) and instruction guard hold | Adopt the knowledge mix for the next base runs. A 96M knowledge base becomes eligible under M4. |
| 3 | **BLOCKED** | panel criterion holds, instruction guard fails | One rebalanced A/B with knowledge weight 0.15, then stop. |
| 4 | **NULL** | `mean_d < 5`, or `d_1` and `d_2` of opposite sign (`d_1 · d_2 < 0`) | Kill, per the plan. The plateau is coherence/capacity, and the next lever is capacity per J/token, not data. |
| 5 | **WEAK** | anything else | Record it. No promotion and no repeat at 29M. The knowledge mix becomes an optional arm only if the owner opens a capacity rung. |

Because the panel criterion requires both seed deltas to be positive and `mean_d ≥ 10`, PROMOTE/BLOCKED and NULL cannot both hold. For example, `d = (+22, −1)` fails the panel criterion and is NULL. The order is stated anyway, and the tabulator applies it literally.

- **Reported but not decisive:** ts-valid NLL (the cost of displacing TinyStories), D19 totals with the sieve on and off, sieve-off MQAR, and knowledge NLL after the fine-tune.
- **No excuse clause:** a null is evidence.

## 7. Limits

- **Single grader and single judge path.** The panel's grader is qwen2.5:7b (final assessment §6). The clean subset was labelled by one annotator, Claude, with reasons. It was not independently reviewed.
- **Weak power.** Two seeds per arm. A 10-row effect on 232 rows is near the limit of what the paired test can resolve.
- **Pooled McNemar is anti-conservative.** It treats the same request under two seeds as independent pairs (464). The two seeds' outcomes on one request are correlated, so the pooled p understates the uncertainty. The tabulator therefore reports each seed's exact McNemar p (232 pairs) beside the pooled one. The decisive test stays the pooled p frozen here, and the both-seeds-positive requirement limits a single-seed effect.
- **Corpus reach.** Simple Wikipedia covers encyclopaedic facts. It does not cover how-to, coding or advice requests. Some panel K rows (recipes, code, workplace advice) are not addressable by this corpus.
- **Decontamination scope.** The exclusion is against panel requests, not against reference answers. The panel has no reference answers. An article that answers a panel question without sharing its wording is kept, and is meant to be kept: that is the knowledge the arm supplies.
- **Untested here.**
  - The parquet reader path is untested on the laptop. The parquet feature's dependencies are not vendored offline. The pod script runs its unit test, which writes and reads a wikimedia-schema parquet file, before preparing the store.
  - `m-world session` was not smoke-run with the new models. Its arguments are copied from the validated 29M/96M session scripts.
