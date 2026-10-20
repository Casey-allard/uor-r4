# Tiered eval v2: tier C results for three ladder models (#820)

References #820. Instrument: `chat-grade` from branch `codex/tiered-eval-v2` at `b04baa47`
(instrument review recorded as approve, no blocking findings), so these results are not
provisional. Measured behaviour on one fixed 160-row panel per table; no capability claim
beyond these rows.

## Setup

- **Models.** `chat-100m2-balp-s1` (recall-off balanced curriculum with paraphrases;
  model.safetensors sha256 `a52473ba…016e`), `chat-100m-C` (oracle-curriculum 96M reference,
  `a746e1fc…0df4`), `chat-29m-B-lr5e-4` (`d8a3c971…8068`). Tokenizer
  `claude-t4-1433-resume/bundle-learned-1/tokenizer.json` (`d36d3e87…f89`).
- **Decoding.** `chat-grade reply`, protocol 2, greedy, `max_new_tokens=64`, float model.
  Each multi-turn row is generated turn by turn; the model's own earlier replies are in its
  history.
- **Grading.** `chat-grade grade-replies grader=qwen2.5:7b` (ollama, temperature 0, seed 1,
  digest `845dbda0…697e`). Acceptable = fluent and relevant and, on a checked row, passing the
  frozen row check. Controls: the three default constant replies (1: "I'm not sure. Can you
  tell me more about what you mean?", 2: "That sounds nice! Thank you for telling me.",
  3: a TinyStories opening), the derangement, and the check-only echo, copy and binding-swap
  controls. Paired comparisons: `chat-grade compare tier=C`.
- **Panels.** Both files of each panel, hashes checked against `data/panels/MANIFEST.sha256`.
  The requested `exact` memory scorer and `abstain_exact` check exist only on the
  tiered-eval-v2 panel, which is named `conversational-v3` (the name `conversational-v2` was
  already taken by the #1733 re-score panel; see `data/panels/README.md`). Both panels were
  graded: v3 is the headline; v2 is reported for continuity with #1733 and uses the looser
  `any` memory and `abstain` checks.
- **Binary.** `chat-grade-b04baa47`, sha256 `f7456266…ba29`. `chat-grade check` on v3: pass
  (worst-case history 336 positions at context 384).

## Headline reading rule

Per category, a model counts only where it beats that category's best constant reply on a
paired two-sided exact McNemar test (p < 0.05, more model-only rows). A tier total is never
the headline. The best constant is chosen after grading, which favours the constant.
"Check only" columns apply the frozen row check without the grader.

## Results

### Panel `conversational-v3` (tiered eval v2 panel: `exact` memory, `abstain_exact`)


`chat-100m2-balp-s1`: tier C acceptable 35/160; beats best constant in: ['self_contained_instruction', 'smalltalk_feelings']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 2 | 1 | 0 | 2 | 0 | 0.5 | 7/0 of 30, p=0.0156 |
| self_contained_instruction | 30 | 6 | 1 | 0 | 6 | 0 | 0.0313 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 8 | 1 | 21 | 0 | 13 | 0.000244 | 15/24 of 24, p=0.00391 |
| unknowable_or_impossible | 24 | 2 | 1 | 15 | 0 | 13 | 0.000244 | 13/24 of 24, p=0.000977 |
| smalltalk_feelings | 26 | 17 | 1 | 8 | 12 | 3 | 0.0352 | - |
| story_continuation | 26 | 0 | 3 | 2 | 0 | 2 | 0.5 | - |

`chat-100m-C`: tier C acceptable 27/160; beats best constant in: ['self_contained_instruction']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 0 | 1 | 0 | 0 | 0 | 1 | 1/0 of 30, p=1 |
| self_contained_instruction | 30 | 6 | 1 | 0 | 6 | 0 | 0.0313 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 6 | 1 | 21 | 0 | 15 | 6.1e-05 | 12/24 of 24, p=0.000488 |
| unknowable_or_impossible | 24 | 0 | 1 | 15 | 0 | 15 | 6.1e-05 | 0/24 of 24, p=1.19e-07 |
| smalltalk_feelings | 26 | 15 | 1 | 8 | 11 | 4 | 0.118 | - |
| story_continuation | 26 | 0 | 3 | 2 | 0 | 2 | 0.5 | - |

`chat-29m-B-lr5e-4`: tier C acceptable 34/160; beats best constant in: ['self_contained_instruction', 'smalltalk_feelings']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 1 | 1 | 0 | 1 | 0 | 1 | 2/0 of 30, p=0.5 |
| self_contained_instruction | 30 | 8 | 1 | 0 | 8 | 0 | 0.00781 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 5 | 1 | 21 | 1 | 17 | 0.000145 | 7/24 of 24, p=1.53e-05 |
| unknowable_or_impossible | 24 | 1 | 1 | 15 | 0 | 14 | 0.000122 | 2/24 of 24, p=4.77e-07 |
| smalltalk_feelings | 26 | 18 | 1 | 8 | 13 | 3 | 0.0213 | - |
| story_continuation | 26 | 1 | 3 | 2 | 1 | 2 | 1 | - |


Paired, balp-s1 against 100m-C (same rows, same grader), tier C:

| Category | rows | balp-s1 | 100m-C | balp-only | C-only | McNemar p | check balp/C (balp-only, C-only, p) |
|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 2 | 0 | 2 | 0 | 0.5 | 7/1 (7, 1, 0.0703) |
| self_contained_instruction | 30 | 6 | 6 | 3 | 3 | 1 | 0/0 (0, 0, 1) |
| clarify_or_on_topic | 24 | 8 | 6 | 3 | 1 | 0.625 | 15/12 (6, 3, 0.508) |
| unknowable_or_impossible | 24 | 2 | 0 | 2 | 0 | 0.5 | 13/0 (13, 0, 0.000244) |
| smalltalk_feelings | 26 | 17 | 15 | 5 | 3 | 0.727 | - |
| story_continuation | 26 | 0 | 0 | 0 | 0 | 1 | - |
| **tier C (all)** | 160 | 35 | 27 | 15 | 7 | 0.134 | 35/13 (26, 4, 5.95e-05) |

### Panel `conversational-v2` (#1733 record panel: `any` memory, `abstain`)


`chat-100m2-balp-s1`: tier C acceptable 37/160; beats best constant in: ['self_contained_instruction', 'smalltalk_feelings']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 4 | 1 | 0 | 4 | 0 | 0.125 | 11/0 of 30, p=0.000977 |
| self_contained_instruction | 30 | 6 | 1 | 0 | 6 | 0 | 0.0313 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 7 | 1 | 18 | 0 | 11 | 0.000977 | 14/21 of 24, p=0.0156 |
| unknowable_or_impossible | 24 | 3 | 1 | 15 | 1 | 13 | 0.00183 | 14/24 of 24, p=0.00195 |
| smalltalk_feelings | 26 | 17 | 1 | 8 | 12 | 3 | 0.0352 | - |
| story_continuation | 26 | 0 | 3 | 2 | 0 | 2 | 0.5 | - |

`chat-100m-C`: tier C acceptable 27/160; beats best constant in: ['self_contained_instruction']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 1 | 1 | 0 | 1 | 0 | 1 | 4/0 of 30, p=0.125 |
| self_contained_instruction | 30 | 6 | 1 | 0 | 6 | 0 | 0.0313 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 5 | 1 | 18 | 0 | 13 | 0.000244 | 10/21 of 24, p=0.000977 |
| unknowable_or_impossible | 24 | 0 | 1 | 15 | 0 | 15 | 6.1e-05 | 0/24 of 24, p=1.19e-07 |
| smalltalk_feelings | 26 | 15 | 1 | 8 | 11 | 4 | 0.118 | - |
| story_continuation | 26 | 0 | 3 | 2 | 0 | 2 | 0.5 | - |

`chat-29m-B-lr5e-4`: tier C acceptable 34/160; beats best constant in: ['self_contained_instruction', 'smalltalk_feelings']

| Category | rows | model | best constant (#) | best-const score | model-only | const-only | McNemar p | check only (no grader): model/const, McNemar p |
|---|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 1 | 1 | 0 | 1 | 0 | 1 | 2/0 of 30, p=0.5 |
| self_contained_instruction | 30 | 8 | 1 | 0 | 8 | 0 | 0.00781 | 0/0 of 6, p=1 |
| clarify_or_on_topic | 24 | 5 | 1 | 18 | 1 | 14 | 0.000977 | 7/21 of 24, p=0.000122 |
| unknowable_or_impossible | 24 | 1 | 1 | 15 | 0 | 14 | 0.000122 | 2/24 of 24, p=4.77e-07 |
| smalltalk_feelings | 26 | 18 | 1 | 8 | 13 | 3 | 0.0213 | - |
| story_continuation | 26 | 1 | 3 | 2 | 1 | 2 | 1 | - |


Paired, balp-s1 against 100m-C (same rows, same grader), tier C:

| Category | rows | balp-s1 | 100m-C | balp-only | C-only | McNemar p | check balp/C (balp-only, C-only, p) |
|---|---|---|---|---|---|---|---|
| multi_turn_memory | 30 | 4 | 1 | 4 | 1 | 0.375 | 11/4 (11, 4, 0.118) |
| self_contained_instruction | 30 | 6 | 6 | 3 | 3 | 1 | 0/0 (0, 0, 1) |
| clarify_or_on_topic | 24 | 7 | 5 | 3 | 1 | 0.625 | 14/10 (6, 2, 0.289) |
| unknowable_or_impossible | 24 | 3 | 0 | 3 | 0 | 0.25 | 14/0 (14, 0, 0.000122) |
| smalltalk_feelings | 26 | 17 | 15 | 5 | 3 | 0.727 | - |
| story_continuation | 26 | 0 | 0 | 0 | 0 | 1 | - |
| **tier C (all)** | 160 | 37 | 27 | 18 | 8 | 0.0755 | 39/14 (31, 6, 4.13e-05) |

## Reading

- **Against the best constant (v3).** balp-s1 and 29m-B beat their best constant on
  `self_contained_instruction` and `smalltalk_feelings`; 100m-C beats it only on
  `self_contained_instruction` (smalltalk 15 vs 8, p = 0.118). No model beats constant 1 on
  `clarify_or_on_topic` or `unknowable_or_impossible`; constant 1 is significantly better there
  for every model. No model beats a constant on `story_continuation` or, under the grader, on
  `multi_turn_memory`.
- **Memory, exact check.** Without the grader, balp-s1 states the asked value bound to the
  right key on 7/30 v3 memory rows against 0/30 for every constant (p = 0.0156); 100m-C 1/30,
  29m-B 2/30. The copy controls pass 15/30 and the binding swap 0/30, so 7/30 is below a
  copy of the first stated fact. Under the grader only 2 of the 7 are acceptable: replies such
  as "Your name is Shelby." to "Which name did we give the turtle?" carry the right value in a
  training-template frame the grader judges irrelevant. Many balp-s1 memory misses are
  relation templates with a wrong slot ("Your lucky number is 10.", "You live in Sunday.").
- **Abstention.** balp-s1 passes `abstain_exact` on 13/24 unknowable rows (100m-C 0/24,
  29m-B 2/24), mostly "I don't know. You haven't told me yet."; the grader accepts 2 of them
  (it marks that reply fluent but usually not relevant). The constant passes 24/24 on the
  check and 15/24 under the grader, so the category stays a constant win.
- **balp-s1 against 100m-C (paired).** Acceptable: 35 vs 27 on v3 (15 vs 7 discordant,
  p = 0.134) and 37 vs 27 on v2 (p = 0.0755); no single category differs at p < 0.05.
  Check pass, without the grader: 35 vs 13 on v3 (26 vs 4 discordant, p = 6e-05), driven by
  `unknowable_or_impossible` (13 vs 0, p = 0.00024) and memory (7 vs 1, p = 0.070).
- **Reproduction.** 100m-C scores 27/160 on v2 with this binary, the same total as the #1733
  re-score with `chat-grade-f3737c28`.

## Cost and artifacts

- Replies: 6 runs of about 1.5–3 minutes each (08:00–08:13 EDT). They ran with the default
  thread pool (about 5 cores), above the 2-thread admission declared in the job lock; grading
  and comparison are single-process and ollama-bound.
- Grading: 6 runs, 1030–1199 s each (08:13–10:04 EDT), one at a time, after the other lab's
  ollama job released its GPU lock. Build: 4 min 36 s in the shared target dir.
- Storage: 3.4 MiB at `~/uor-r4-local/ladder/grades/tiered-v2-2026-10-05/` (`replies-*`,
  `grade-*`, `cmp-*`, `run.sh`, `run.log`). Each report directory was claimed exclusively and
  not reused.

## Limitations

- One grader (qwen2.5:7b) and one panel draw; the grader rejects some value-correct and
  abstaining replies, so the acceptable counts are a lower bound for those categories.
- The panels are development panels, not a held-out final evaluation.
- `story_continuation` and `smalltalk_feelings` have no row check; their result rests on the
  grader alone.
