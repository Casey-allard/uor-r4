# The open reply panel's 189 failures are diffuse: 43/232 reproduced per category, no dominant mechanism, Phase 2 not opened

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09.
CPU only: no pod, no training, **no float model generation, no grading**. The
model's replies already existed in a sealed report; this record re-reads and
classifies them.

## Question

Acceptance criterion 1 asks for **≥ 116/232 `acceptable`** on the open reply panel.
The recorded number is **43/232** for the 29M chat fine-tune — 2.7× short, and the
largest single gap in M1. Phase 1 (this record, run before any candidate) is:

1. reproduce 43/232 on that panel with that grader, or report that it cannot be
   reproduced;
2. classify the 189 failures **per category** with the grader's own verdict fields;
3. say whether ONE named mechanism dominates — and if they are diffuse, say so and
   **stop**, opening no Phase 2 spend.

## Identities (taken from the record, not from prose)

**The panel is not in the repository.** The open reply panel is three request
files in the ladder store `~/uor-r4-local/ladder/panel/`, preserved as
`icloud:UOR-R4/results/deepseek/ladder.tar` (3,306,919,424 B, md5
`ed9456d63472b7a424fe6ca62d36baa8`, stored 2026-10-09T06:32:39Z). Restore with
`~/.local/share/uor-r4/bin/cloud-store fetch ladder <dest>`; the paths below are
inside that archive.

| File | Rows | Categories | sha256 |
|---|---|---|---|
| `ladder/panel/everyday-32.json` | 32 | `smalltalk` 8, `simple_question` 8, `simple_instruction` 8, `follow_up` 8 | `945c0c97196d39db430b5821c8c30a888a3c719c6e701c218fc7a914b350b480` |
| `ladder/panel/heldout-200-a.json` | 100 | `heldout_first_turn` 100 | `019cc6f65e8ec815d29328f7dd48f49126beaa29b29d1b4bcc7d4d156dffd2c1` |
| `ladder/panel/heldout-200-b.json` | 100 | `heldout_first_turn` 100 | `5434cfd9767c78a179e0899c9568971466dd36e8dcf4f106e1de1566fb29436c` |

**232 rows, 224 of them single-turn** (8 `follow_up` rows have two turns). Each of
the three sha256 values is recorded in the sealed report itself
(`report.requests[].sha256`) and repeated in the archive's own
`ladder/panel/SHA256SUMS.txt`, so the panel bytes are bound to the report from
three independent places. `everyday-32` and `heldout-200*` are **not tracked in
main**: `git ls-files data/panels` has no such file, so criterion 1's instrument is
recoverable only from the archive.

**The sealed report** (this is where 43/232 lives):

| | |
|---|---|
| report root | `ladder/grades/chat-29m-B-lr5e-4-powered-7b` (claimed 2026-10-03T21:47:54Z, sealed 2026-10-03T22:19:13Z) |
| `report.json` | sha256 `72ab32a211c8d57cb20a429c1ccac4eacd794a686f35c6e9140545473fe259d4`, 162,421 B, blake3 `792623bc…` |
| model | `ladder/runs/chat-29m-B-lr5e-4/model`, sha256 `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068`, 28,958,761 params |
| decoding | protocol `uor-r4.literal-role-dialogue/2`, greedy, `max_new_tokens=64`, tokenizer sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| cost | generation 1146.1 s, wall 1879.0 s |
| instrument | `chat-grade` executable sha256 `a5071cfe56c8bec56badc978c8a023f632cfe31d54342aa4db339b447e294239` |
| grader | ollama `qwen2.5:7b`, digest `845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e`, temperature 0, seed 1, offline judge only |

The report's own `executable_sha256` is `a5071cfe…` and the laptop still holds that
exact binary at `~/.local/share/uor-r4/bin/chat-grade-a621514c`
(`shasum -a 256` re-verified equal), so the frozen instrument is present, not merely
named. The taxonomy round's hashes for these two reports (`72ab32a2…59d4`,
`3d5a2914…ccf6`) match these copies byte for byte.

**The rule.** `acceptable = fluent ∧ relevant` (an unparsed judge answer counts as
no). `check_pass` is a different field: it is the frozen deterministic row check,
and **the open panel has none** — the embedded checks files carry only `conv-*`
ids (panels v2/v3/v4), and all 232 rows here are `talk-*`, `do-*`, `ask-*`,
`follow-*`, `heldout-*`. So on this panel `checked_rows = 0` and there is no
`check_pass` reading at all: `acceptable` **is** fluent ∧ relevant. That is the
opposite of the v4 memory panel, where `check_pass` was the primary reading
because it is deterministic. Here the headline number is an LLM judge's, and it
carries the judge's instability with it.

## Method

Nothing was generated and nothing was graded. The sealed report stores, for every
row, the conversation it graded (`conversation: [{user, assistant}, …]`) whose last
assistant entry is the model's trimmed reply. Three steps:

0. **Evidence bundle.** The panel bytes, both sealed reports, the rebuilt replies,
   the classifier and its outputs are stored as
   `icloud:UOR-R4/results/deepseek/reply-panel-open-2026-10-09.tar` (760,832 B, md5
   `1740b21f181779221458bb8c32008f4f`,
   `~/.local/share/uor-r4/bin/cloud-store fetch reply-panel-open-2026-10-09 <dest>`).
   The cited restore command was run and round-trips at that md5, and re-running the
   classifier from the restored copy reproduces 232 rows / 43 acceptable / 0 panel
   mismatches, so every number here is checkable from main.
1. **Binding check.** Every one of the 232 report rows was matched against the three
   panel files by id, category and user turns: **232/232 exact, 0 mismatches**. The
   report is bound to the panel bytes above. This is the classifier's `--panel`
   option, so it re-runs with the numbers.
2. **Classification.** `scripts/open_reply_panel_classify.py` maps each row to one of
   four verdict classes from the report's own `grades.fluent` / `grades.relevant`,
   then applies the deterministic text rules named below. It runs no model, has a
   `--selftest`, and writes the per-row table as `verdicts-29m.tsv` /
   `verdicts-100m.tsv` beside this record.
3. **Re-grade: designed, then declined.** `replies-29m.json` (the 232 conversations
   rebuilt from the sealed report by `rebuild_replies.py`, ready for a
   `grade-replies` pass) is in the bundle, but the pass itself was declined as
   decision-irrelevant before any spend — see Result 1.

## Result 1 — 43/232 is reproduced from the sealed record

`report.actual.acceptable = 43` of 232 (`fluent = 70`, `relevant = 53`,
`unparsed_answers = 0`), and the 232 report rows bind to the three panel files on
id, category and user turns with **0 mismatches**. The baseline the priority
correction quotes is therefore read from the frozen instrument's own report, not
from prose, and its panel bytes are the ones hashed above.

### Re-grade: the judge's stability on this panel — designed, then declined as decision-irrelevant

The 43 is a `qwen2.5:7b` verdict, so it carries the judge's noise, and this panel
has no deterministic check to fall back on. A re-grade on a pod was pre-registered
(sealed replies, same grader digest, `grade-replies`, two passes) and then
**declined by the Lead on 2026-10-09 with this arithmetic, which is the right way to
size it**: the v4 round measured the judge differing on **4 of 64 rows at the same
grader digest**, about 6%, which is roughly **±14 cells on 232 rows**. The gap to
criterion 1 is **73 cells** (116 needed against 43 measured). A ±14-cell noise floor
cannot move that verdict in either direction, so the measurement would be honest,
interesting and decision-irrelevant. It is not run here and it is not scheduled.

One laptop `grade-replies` attempt was started at 16:45 on 2026-10-09, before the
correction prohibiting laptop grading reached this session, and was killed at
17:00; it wrote `attempt.json` and no `report.json`. No number here comes from it.

### The defect this exposes in criterion 1 (recommendation)

Criterion 1 on this panel **rests on a judge-only reading**. `acceptable` is the
only field the panel has: there are no frozen row checks for its ids, the way v4
and v5 have them, so there is nothing deterministic to be primary and nothing to
name a judge-changed row against. The v4/v5 memory panels avoid exactly this by
making `check_pass` the primary reading and `acceptable` secondary. Before anyone
trains against 116/232, the criterion should be repaired in one of two ways:

1. **Deterministic row checks for the reply panel** in the style of
   `conversational-v4-checks.tsv` / `-v5-checks.tsv`, so acceptance is
   `check_pass`-primary; or
2. **a stated tolerance** for the judge's noise, so a claim of "116/232" is not a
   claim of 116 ± unknown.

This is a criterion defect, not a measurement defect, and it is cheap to fix
now — the same lesson as v4 becoming development evidence and v5 having to be
frozen as its replacement.

## Result 2 — the failures per category (grader's own verdict fields)

Every failing row fails on the judge's two yes/no questions. The four cells over the
**189 failures** of the 29M chat fine-tune:

| verdict class | meaning | rows | share of 189 |
|---|---|---:|---:|
| `neither` | fluent **no** and relevant **no** | 152 | **80.4%** |
| `fluent_only` | fluent yes, relevant no | 27 | 14.3% |
| `relevant_only` | fluent no, relevant yes | 10 | 5.3% |
| *`acceptable`* | both yes | *43* | *of 232* |

Per panel category (criterion 3's requirement; `acceptable` / `neither` /
`fluent_only` / `relevant_only`):

| category | rows | acceptable | neither | fluent_only | relevant_only | failure rate |
|---|---:|---:|---:|---:|---:|---:|
| `smalltalk` | 8 | 6 | 0 | 2 | 0 | 25.0% |
| `simple_question` | 8 | 1 | 6 | 1 | 0 | 87.5% |
| `simple_instruction` | 8 | 1 | 6 | 1 | 0 | 87.5% |
| `follow_up` | 8 | 1 | 7 | 0 | 0 | 87.5% |
| `heldout_first_turn` | 200 | 34 | 133 | 23 | 10 | 83.0% |
| **panel** | **232** | **43** | **152** | **27** | **10** | **81.5%** |

Per tier (chat-grade's own tier rule: `everyday` = the everyday categories,
`K-clean` / `K-ill-posed` = `heldout_first_turn` split by the embedded
`data/panels/heldout-ill-posed-v3-ids.txt`):

| tier | rows | acceptable | neither | fluent_only | relevant_only | failure rate |
|---|---:|---:|---:|---:|---:|---:|
| `everyday` | 32 | 9 | 19 | 4 | 0 | 71.9% |
| `K-clean` | 133 | 18 | 105 | 4 | 6 | **86.5%** |
| `K-ill-posed` | 67 | 16 | 28 | 19 | 4 | 76.1% |

Two things follow. The failure is **not** a panel artifact: the clean, well-posed
rows fail *more* often (86.5%) than the ill-posed ones (76.1%). And the
`fluent_only` class concentrates in `K-ill-posed` (19 of 27) — a well-formed
answer to a request that has no answer is exactly what an ill-posed row should
produce, and it is the one place where a per-category reading is sharper than the
panel total.

## Result 3 — the named mechanisms, with counts, and why none dominates

Deterministic text rules over the 189 failures (rules stated verbatim in the
script; each is a marker, not a diagnosis):

| mechanism | rule | failures | share |
|---|---|---:|---:|
| `no_terminal` | the last non-space character is not `. ! ? " ' ) ] } * \` :` — the reply was cut mid-clause | 125 | 66.1% |
| `repeat5` | some 5-word sequence occurs twice or more | 46 | 24.3% |
| `restate_high` | ≥ 70% of the reply's distinct words are in the last user turn | 17 | 9.0% |
| `code_fence` | an odd number of ` ``` ` fences (opened, never closed) | 8 | 4.2% |
| `question` | the reply contains `?` | 8 | 4.2% |
| `echo_user` | the reply contains the whole last user turn verbatim | 3 | 1.6% |
| `stub` | two words or fewer | 2 | 1.1% |
| `refusal` | contains one of chat-grade's own `ABSTAIN_PHRASES` | 1 | 0.5% |
| `other` | no marker fires | 41 | 21.7% |

A refusal where an answer was wanted is *not* a feature of this panel: the
instrument's own abstention list fires once in 189 failures. Widening it to the
obvious extra family ("do not have access") adds 10 rows, so all declinations
together are ≤ 11/189 = 5.8%.

**Does the largest marker behave like a cause?** Over all 232 rows, not just the
failures:

| population | rows | accepted | rate |
|---|---:|---:|---:|
| replies ending mid-clause | 141 | 16 | 11.3% |
| replies ending with terminal punctuation | 91 | 27 | 29.7% |
| replies with a repeated 5-gram | 50 | 4 | 8.0% |
| replies with no repeated 5-gram | 182 | 39 | 21.4% |
| replies with **neither** marker | 86 | 25 | **29.1%** |

Mid-clause endings and verbatim repeats are both associated with failure, but the
association does not isolate the cause: **the 86 rows carrying neither marker are
still accepted only 29.1% of the time (25 of 86), and they are 37.1% of the panel.**
A rule that acted on both markers would leave that population untouched; even if
every one of the 146 marked rows rose to the unmarked group's current rate, the
panel would sit near **68/232**, not 116/232. And 66.1% is not dominance in any
case: the largest single mechanism covers two-thirds of the failures with a
*symptom* (the reply did not finish), not a lever that can be pulled without
regenerating text.

**Independent annotator, same conclusion.** The recorded Step 0a taxonomy
([`docs/research/step0/0a-panel-taxonomy.md`](../../research/step0/0a-panel-taxonomy.md))
read all 189 of these failures by hand with a frozen rubric and found K 26.5%,
N 25.4%, O 29.6%, I 10.1%, T 5.8%, R 2.6% — largest class 29.6%, no majority — and
froze "retrieval cannot move the open panel; panel lever = corpus/knowledge".
Its `T` (truncation at the 64-token budget) is 11 rows against my looser
`no_terminal` 125; the two rules are not the same measurement and the difference is
declared rather than reconciled. **Result 4 below resolves it: the looser rule is
the budget, and the hand rubric's `T` was counting only the replies whose text ends
without any punctuation at all.**

## Result 4 — the mid-clause marker IS the 64-token budget (amendment, 2026-10-09 later)

This section is the piece the Limitations of the first pass said could not be done.
It was, because the tokenizer named in this record **is on the laptop**.

**Where the tokenizer was, and four independent confirmations.** The sealed report's
own `tokenizer_sha256` is `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
The sealed report root's `attempt.json` — extracted for this round from
`icloud:UOR-R4/results/deepseek/ladder.tar` — records the path the run used:

```text
tokenizer=/Users/casey.allard/uor-r4-local/inputs/claude-t4-1433-resume/bundle-learned-1/tokenizer.json
```

The `inputs/` tree is gone, but the same bundle survives under the `workspace/` tree and
its file hashes to exactly that value:

| | |
|---|---|
| path (found) | `~/uor-r4-local/workspace/uor-r4-lab/claude-t4-1433-resume/bundle-learned-1/tokenizer.json` |
| sha256 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| blake3 address reported by the loader | `blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc` |
| attempt.json (sealed, 2026-10-03) | names that bundle path |
| report `tokenizer_sha256` | matches the file's sha256 |
| `chat-grade check` worst case, v3 / v4 | **336** (`conv-v3-mem-16`) and **347** (`conv-v4-mem-23`) — reproduced exactly with this file |

The last row is the instrument check: `crates/uor-r4-tokenizer/examples/token-counts.rs`
(the crate's own engine, the one `chat-grade` loads) recomputes `check_panel`'s
`worst_case_history` from the frozen panels and lands on the two numbers already recorded
on main for `context=384`. The counts below are therefore the frozen instrument's counts,
not a re-implementation's.

**Method.** `scripts/open_reply_panel_token_counts.py extract` pulls the 232 stored replies
out of the sealed report (the last assistant entry of each row's `conversation`, which
`reply_panel` filled with `decode(ids without EOS)`, then `trim()`ed by
`panel_conversations`); the example counts tokens per reply; `summarise` cross-tabulates
against the report's own grades and the classification of Result 3. CPU only: no model, no
generation, no grading, no pod, zero spend.

**Evidence bundle.** `icloud:UOR-R4/results/deepseek/reply-panel-token-budget-2026-10-09.tar`
(540,672 B, md5 `ff006fa0f86cdbfa671e8491b108ea06`,
`~/.local/share/uor-r4/bin/cloud-store fetch reply-panel-token-budget-2026-10-09 <dest>`): the
tokenizer copy, both token-count files, both per-row tables, both summaries, the extracted
reply texts, copies of the two instruments and `verify.txt` (hashes plus the 336/347
instrument check). The store never overwrites a sealed root, so the copy of
`token-counts.rs` inside it is the revision before `cargo fmt` reflowed it; the file on main
is canonical and differs only in whitespace.

**Why the count answers the question.** The loop
(`stack_dialogue::greedy_reply_with`) runs at most `max_new_tokens = 64` steps and leaves
early only on EOS (`Reply::stop_with`) or on a short repeated cycle (`cycle_repeats=3`).
So a reply whose *own text* already needs 64 canonical tokens cannot have come from a short
generation: the model spent essentially the whole budget on it. The reconstruction is not
exact to the token — the stored text is trimmed, and a model token can cross a GPT-2
pre-token boundary, both of which move the count by one or two — so the counts are read in
a band around 64, and the five replies at 65–66 are the calibration of that band (they can
only be cap runs, so the inflation is +1 to +2).

**The counts, 29M (`chat-29m-B-lr5e-4`, 43/232).** Token count of the stored reply over the
232 rows: 141 are at 63 or more, of which **95 are at exactly 64 and 41 at exactly 63**;
below that the mass is thin (2 at 62, 2 at 60, 2 at 59, 1 at 58, and a long sparse tail).
Split by population:

| population | rows | ≥ 64 tokens (at the cap) | = 63 tokens | ≤ 62 tokens |
|---|---:|---:|---:|---:|
| all failures | 189 | **87 (46.0%)** | 37 | 65 |
| **mid-clause failures (`no_terminal`)** | **125** | **85 (68.0%)** | **34 (27.2%)** | **6 (4.8%)** |
| failures with terminal punctuation | 64 | 2 (3.1%) | 3 | 59 |
| tier `K-clean` failures | 115 | **73 (63.5%)** | 20 | 22 |
| tier `K-ill-posed` failures | 51 | **14 (27.5%)** | 15 | 22 |
| tier `everyday` failures | 23 | **0** | 2 | 21 |

**The single answer: CAP-TRUNCATION, and the split is 85 / 34 / 6.**

- **85 of the 125 mid-clause failures (68.0%) ran to the 64-token cap.** The model never
  emitted EOS and never entered a terminal cycle; it was cut mid-clause by the budget.
- **34 more (27.2%) sit at exactly 63**, one token under the budget, which is the band the
  trimming and pre-token-boundary effects live in. Their text already needs 63 canonical
  tokens, so they spent ≥ 61 of the 64 steps. Whether the last step was EOS or the cap,
  they are budget-limited too.
- **6 of 125 (4.8%) are genuine voluntary stops mid-sentence**, well inside the budget
  (`heldout-020` 19 tokens, `heldout-130` 22, `heldout-156` 38, `heldout-060` 55,
  `heldout-135` 56, `heldout-176` 56).

So **119 of the 125 mid-clause failures (95.2%) end within one token of the 64-token
budget**, and the marker the first pass reported as "the largest single mechanism, 66.1% of
the failures" is a *termination* failure, not a failure to finish a thought the model chose
to start. Coverage of the 189: **85 (45.0%) at the cap, 124 (65.6%) within one token of it,
125 (66.1%) mid-clause in total, and 64 (33.9%) not budget-limited at all** — 62 of those
end with terminal punctuation, i.e. completed replies that were graded wrong.

**The well-posed/ill-posed asymmetry the priority correction quotes is the same budget.**
`K-clean` rows sit at the cap far more often than `K-ill-posed` ones: 82 of 133 `K-clean`
rows (61.7%) reach 64 tokens against 18 of 67 `K-ill-posed` rows (26.9%), and among failures
63.5% against 27.5%. A well-posed request makes the model keep writing until the budget
stops it; an unanswerable one makes it stop early — which is exactly the `fluent_only`
concentration in `K-ill-posed`. **The 86.5% fail on `K-clean` versus 76.1% on `K-ill-posed`
is a budget effect, not a knowledge effect.** The `everyday` tier never reaches the cap at
all (0 of 32), which is why it is the healthiest tier.

**Acceptance is cap-sensitive too.** At the cap 13 of 100 replies are `acceptable` (13.0%);
below it 30 of 132 are (22.7%). The cap roughly halves the acceptance rate.

**A second artifact, same shape.** The 96M `chat-100m-C` report (46/232, 186 failures,
token counts computed the same way): **86 of its 117 mid-clause failures (73.5%) are at the
cap**, 30 at 63, and **1** below that (`heldout-055`, 28 tokens); `K-clean` 76/116 failures
at the cap (65.5%) against `K-ill-posed` 8/46 (17.4%). The finding is not a property of the
29M model.

**What this does and does not say.**

- It says the replies were cut by the 64-token decoding budget. It does **not** say the
  model would have answered correctly with more room: a longer cap would make these replies
  longer, not necessarily right, and nothing here regenerates a reply. The next measurement
  that would decide it is a declared run at a larger `max_new_tokens` on the same sealed
  replies' prompts — a decoder-budget experiment, not a training one.
- It explains 45.0% of the 189 failures outright and 65.6% inside the one-token band. It
  does **not** explain the 62 failures that end with terminal punctuation, nor the 6
  voluntary mid-sentence stops; those remain diffuse, and the earlier decision not to open
  Phase 2 is unchanged.
- It also means **this panel's headline number is partly a decoding-budget number**. Any
  candidate scored on it at `max_new_tokens=64` inherits that; a candidate compared against
  43/232 must be compared at the same budget.

**A second artifact, same shape.** The 96M `chat-100m-C` report
(`ladder/grades/chat-100m-C-powered-7b/report.json`, sha256
`3d5a2914a444905d041f510d1c7bdc700cb2ee1c674727fb4668600acadbccf6`, model sha256
`a746e1fc…`, 96,023,745 params, same grader and instrument sha256) scores
**46/232**: `neither` 145/186 = 78.0%, `fluent_only` 34 = 18.3%, `relevant_only` 7 =
3.8%; `no_terminal` 117/186 = 62.9%, `repeat5` 24 = 12.9%, `other` 40 = 21.5%. The
diffusion is not a property of the 29M model; it is stable across a 3.3× scale
step, which is also what the recorded 43–46/232 plateau from 29M to 96M says.

## Result 5 — the token budget does not explain the acceptance gap: 96 tokens moves it by −6 cells (amendment, 2026-10-09 later)

Result 4 measured *that* the mid-clause failures are the 64-token budget (95.2% of them end within one token of it). It could not say **how much of 43/232 the budget is**. This is that measurement, pre-registered on #2029 before any pod was created, and it comes back **null**.

### The frozen binary is macOS-only, and that is a finding in its own right

`chat-grade-a5071cfe…` — the executable the sealed report names — is **Mach-O 64-bit executable arm64**. Uploaded to a Linux pod it exits **126, "cannot execute binary file: Exec format error"**. The sealed report's own `attempt.json` agrees: its `argv[0]` is `/Users/casey.allard/.local/share/uor-r4/bin/chat-grade-a621514c`, a macOS path.

**So the 43/232 baseline was generated on the laptop, not on a pod, by a binary that does not run on Linux.** The panel's "frozen, sealed" number is a macOS-only artifact, and its reproducibility on the platform the project now uses for GPU work was, until this result, unproven. Result 5 closes that: **the sealed replies reproduce on Linux byte for byte** (below). The *binary* is not portable; the *behaviour* is.

### Run set, and why it is four runs' worth of confounds resolved by one control

Two runs on pod `260rd8ondf4mzg` (1 × RTX 5090), both **`device=cpu`** — the sealed run's `attempt.json` passes no `device=` and `device_name(None)` is `cpu`, so the judge is the only GPU user:

| | Run A (control) | Run B (measurement) |
|---|---|---|
| executable | `/workspace/bin/4a5fd0d1…-sm120/chat-grade`, sha256 `77256a3a3058ca9a8a8e74c3bba2485598b801038ebc1b1b5fba06fcfe71c1be` — the pod's own Linux build of main at `4a5fd0d1` | same |
| panel | `everyday-32.json` `945c0c97…`, `heldout-200-a.json` `019cc6f6…`, `heldout-200-b.json` `5434cfd9…` | same |
| model / tokenizer | `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068` / `d36d3e87…` | same |
| grader | `qwen2.5:7b` digest gated at `845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e`, temperature 0, seed 1 | same |
| context, protocol | 384, `uor-r4.literal-role-dialogue/2` | same |
| `max_new_tokens` | **64** | **96** |
| result | **44/232** acceptable (fluent 69, relevant 54), 12m47s | **38/232** (fluent 62, relevant 48), 16m58s |

`check_panel` fits at both caps (worst case 168 positions at 64, 232 at 96, of 384), so context stays 384 and **the cap is the only declared delta between A and B**. The macOS binary could not be used, and the Linux build differs from it in three ways at once — platform, and the four greedy-loop commits since 2026-10-03 (#1846, #2040, #2047, #2062) — so A exists to separate those from the cap. It does:

**Anchor — sealed macOS 64 → Linux control A 64, same cap.** `acceptable` **43 → 44 (+1)**; **all 232 replies are byte-identical**; the single cell is one judge verdict flip on `heldout-199`, with no reply change; exact McNemar p = 1.0. So (a) the four greedy-loop commits are opt-in for this model **as measured, not as asserted** — a model with no `pointer_copy_stop` falls through to `greedy_reply_plain` bit for bit; (b) the platform is not a confound; and (c) the judge alone moved one cell on identical text, which is the noise floor showing up at its smallest scale.

**Run A did more work than its job description.** Without it, a −6-cell swing would have been unreadable: platform, four code changes and the cap would all have been in play at once, and no reading of the result would have been decisive. With it, the platform and the code changes are pinned (byte-identical replies, one cell of judge noise) and the cap is the only thing that moved. **The control converted an ambiguous comparison into a decisive one**, and it cost twelve minutes.

### Primary reading — A (64) versus B (96), same machine and build

**44/232 → 38/232. Delta −6 cells.** The pre-registered noise floor is ±14 cells on 232 (the judge differed on 4 of 64 rows at the same digest in the v4 work, ≈6%), so **−6 is not a result, whatever its sign**. Movement underneath it:

| | count |
|---|---:|
| A's failures | 188 |
| **A's failures that became `acceptable` at 96** | **3** (`heldout-036`, `heldout-114`, `heldout-174`) |
| A's failures unchanged | 185 |
| A's `acceptable` rows that failed at 96 | **9** (`heldout-010`, `-024`, `-061`, `-071`, `-073`, `-082`, `-090`, `-185`, `-192`) |
| exact McNemar p | 0.146 |
| replies identical between the runs | 90 of 232 |

**The sharpest sub-question, and the answer.** Of A's **85 mid-clause failures that were at the 64-token cap** — the exact population Result 4 identified — **2 became `acceptable` at 96 and 83 stayed failed.** Of all 125 mid-clause failures, 3 became acceptable. Giving the cut replies 50% more room converted **2.4% of them**.

**96 is simply the next wall.** B's reply lengths cluster on the new cap exactly as A's did on the old one: **91 replies at exactly 96 tokens and 34 at exactly 95**, against 95 at exactly 64 and 41 at exactly 63. Failures at or above the cap: **92 of 194 (47.4%)** at 96 against **87 of 189 (46.0%)** at 64. B's mid-clause failures are 124, of which 89 are at or above 96. Nothing about the cap's relationship to failure changed; only the wall moved.

**Per tier and per category** (`acceptable`, A → B):

| tier | rows | A | B |
|---|---:|---:|---:|
| `everyday` | 32 | 9 | 9 |
| `K-clean` | 133 | 19 | **12** |
| `K-ill-posed` | 67 | 16 | 17 |

| category | rows | A | B |
|---|---:|---:|---:|
| `smalltalk` | 8 | 6 | 6 |
| `simple_question` | 8 | 1 | 1 |
| `simple_instruction` | 8 | 1 | 1 |
| `follow_up` | 8 | 1 | 1 |
| `heldout_first_turn` | 200 | 35 | **29** |

Verdict classes: `acceptable` 44 → 38, `fluent_only` 25 → 24, `neither` 153 → 160, `relevant_only` 10 → 10. Fluent 69 → 62 and relevant 54 → 48: the longer replies were judged slightly *less* fluent and *less* relevant, not more.

### What this says and does not say

**The frozen decision rule fires, and it fires on the negative side.** Under 14 cells is not a result whatever its sign, so **a −6-cell move is not evidence that more room helps, and it is certainly not evidence that the truncated replies would have been acceptable.** The pre-registration's caveat is now measured rather than asserted: the cut replies were given 50% more room and did not become acceptable. It measures **how much of the gap is the token budget: none of it that this instrument can see.**

Three consequences, stated plainly:

1. **Criterion 1 does not need a declared cap.** Raising the cap does not help acceptance, so a token ceiling is not what is holding the reading down. The criterion's real defect is still the other one — it is judge-only, with no deterministic check to be primary — and that repair stands unchanged.
2. **The completed-but-wrong replies are the whole story.** Failures that finish their sentences and are still rejected are where the capability gap lives: 62 such rows at 64 tokens, 9 acceptable rows newly lost at 96, and 83 of the 85 cap-cut failures still failing with more room. That is where the next piece should point, **not at the decoder**.
3. **No 128-token arm.** The decision rule says stop; the piece is closed; the local-route idea for a second point is moot and it will not be run.

Also recorded, because it rules out a re-labelling artefact: **both judge components fell the same way** — fluent 69 → 62 and relevant 54 → 48 — which is why the `acceptable` count dropped by 6. The longer replies were judged slightly less fluent *and* slightly less relevant, not traded from one class to another.

- **The macOS-only binary finding stands as a result**: the baseline the criterion is written against was produced on a platform the project no longer uses for pod work, and its *behaviour* (not its binary) is what reproduces on Linux. Any future re-run of this panel should use a Linux build of the current source and take this A as its anchor.
- **The 100M artifact stayed out of scope**, as pre-registered.

### Costs, and the two failed launches

Pod `260rd8ondf4mzg` existed 21:51Z–22:41Z, 1 × 5090 at $1.19/h → **≈$1.00**, against a pre-registered estimate of ≈$1.65 and a worst case of $3.57. Two launches failed before the runs and are recorded with their exact symptoms, because they are why the design is right: (1) `uor-pod run … -- sh -c "cd … && …"` lost its shell quoting and ran from `/root` (**exit 127**), fixed by uploading a script and invoking `bash`; (2) the platform failure above (**exit 126**). **Neither reached `report_output::claim`, `grades/` stayed empty until Run A, and no number in this record comes from them.** The lease ran to 00:50:07Z and was released at 22:41Z with nothing running; the pod was then deleted. Its `run-env-*.txt` and `grade-*.log` were not copied back before deletion — every identity they carried (executable, model, tokenizer, grader digest, cap, argv) is in the sealed `report.json` / `manifest.json` / `attempt.json` in the bundle below.

**Evidence bundle.** `icloud:UOR-R4/results/deepseek/reply-panel-cap96-2026-10-09.tar` (1,417,728 B, md5 `3de8c41eb9eb7532e1243450365ce335`,
`~/.local/share/uor-r4/bin/cloud-store fetch reply-panel-cap96-2026-10-09 <dest>`): both runs' sealed reports, manifests and attempts, the sealed macOS report for the anchor, the two comparison summaries and per-row tables, both runs' reply texts and token counts, the comparison script and the run script.

## Result 7 — accepted and completed-but-rejected are NOT separable by any judge-free property, and one canned reply carries a fifth of the accepted set (supplement to Result 6)

Result 6 profiled the failures and contrasted completed against cut. This compares the **accepted set** against the completed-but-rejected set on the same judge-free properties, in both directions, which is the comparison Result 6 could not make. CPU only: one sealed report plus the tokenizer crate's counts, no generation, no grading. **It confirms Result 6's conclusion with a second, independent reason, and it corrects one inference a reader might draw from Result 6's length finding.**

### Reconciliation with Result 6 — both counts are right, under their definitions

Of the 189 failures, **64 rows have `no_terminal=0`** (Result 6's completed-but-rejected) and 125 are cut. Of those 64, **62 are below the 64-token cap and 2 reach it** — `heldout-124` and `heldout-139`, both 64 tokens and both `fluent_only`. So Result 6's 64 and the earlier 62 differ by exactly those two rows and by nothing else; the 62 is "not budget-limited", the 64 is "`no_terminal=0`". **No conclusion in either result changes under either definition.**

One correction of framing that follows: **16 of the 43 ACCEPTED replies also have `no_terminal=1`** — they were cut by the cap and accepted anyway. `no_terminal` is a marker of being cut, not of failing.

### The comparison, with the cut confound removed

Restrict to replies that end with terminal punctuation, so the cap cannot be doing the work: **27 accepted** against **64 completed-but-rejected**. Fourteen properties measurable without a judge, each a text rule; multiplicity handled with Bonferroni across the battery.

| property | accepted (27) | rejected (64) | gap | p |
|---|---:|---:|---:|---:|
| reply **is** one canned greeting (`Hello! How can I help you today?`) | 8 | 1 | +0.281 | **0.0002** |
| reply contains a question mark | 10 | 4 | +0.308 | **0.0006** |
| reply starts hello / hi / hey | 9 | 8 | +0.208 | 0.036 |
| request is an instruction | 2 | 17 | −0.192 | 0.049 |
| reply is under 15 tokens | 18 | 29 | +0.214 | 0.071 |
| reply has an unclosed code fence | 0 | 8 | −0.125 | 0.099 |
| request is a question | 6 | 21 | −0.106 | 0.45 |
| reply uses assistant self-reference | 7 | 22 | −0.084 | 0.47 |
| reply is shorter than the request in words | 8 | 25 | −0.094 | 0.48 |
| reply is over 40 tokens | 5 | 16 | −0.065 | 0.59 |
| reply repeats a 5-gram | 2 | 3 | +0.027 | 0.63 |
| reply contains a digit | 2 | 8 | −0.051 | 0.72 |
| reply has no content-word overlap with the request | 10 | 22 | +0.027 | 0.81 |
| reply restates a 4-gram of the request | 6 | 15 | −0.012 | 1.00 |

Two properties survive the correction — **but they are the same observation**. Eight of the 10 accepted replies that contain a question mark **are that one canned greeting**, so the question-mark line is the greeting line. Remove the single canned reply from both sides and **nothing separates the sets at all**:

| after removing the one canned reply | accepted (19) | rejected (63) | p |
|---|---:|---:|---:|
| median tokens | **14** | **20** | 0.45 (Mann-Whitney) |
| contains a question mark | 10.5 % | 4.8 % | 0.33 |
| starts hello / hi / hey | 5.3 % | 11.1 % | 0.67 |
| under 15 tokens | 52.6 % | 44.4 % | 0.60 |
| over 40 tokens | 26.3 % | 25.4 % | 1.00 |

and acceptance is flat across length buckets (1–10 tokens 30.0 %, 11–20 21.7 %, 21–30 15.4 %, 31–40 20.0 %, 41–70 23.8 %).

### One correction to the natural reading of Result 6's length finding

Result 6's length contrast is **completed versus cut**, and it is correct as stated: cut replies ran to the cap and are long. It should **not** be read as "short replies are rejected". Against the accepted replies of the same terminal-punctuation class the rejected ones are **longer**, not shorter — median 19.5 tokens against 9, Mann-Whitney p = 0.043 before the canned reply is removed, p = 0.45 after. And the shortest replies in the panel are accepted most often. **Length is not the discriminator; it only looks like one when the comparison is against the cap.**

### The instrument statement, now measured

**The judge is drawing a line the text does not show.** In this population the only property that predicts acceptance is that the reply *is* one specific memorised string. Strip that string and two sets of replies that differ by 3× in verdict are statistically indistinguishable on all fourteen properties — length, form, question-back, restatement, self-reference, repetition, digits, code, overlap. That is the measured version of Result 6's qualitative claim, and it is a stronger statement than "the failures are diffuse": **the accepted set is not distinguishable from the completed-but-rejected set by anything measurable without the judge.**

And the canned reply is not a curiosity: it carries a large share of the reading on **both** artifacts.

| artifact | occurrences | accepted | acceptance rate | share of all accepted rows |
|---|---:|---:|---:|---:|
| 29M `chat-29m-B-lr5e-4` (43/232) | 9 | 8 | 88.9 % | **18.6 %** |
| 100M `chat-100m-C` (46/232) | 19 | 11 | 57.9 % | **23.9 %** |

Both are 3–5× the panel's base acceptance rate (18.5 % and 19.8 %). **Between a fifth and a quarter of the accepted rows on this panel are the same memorised greeting.**

### What this adds to Result 6's recommendation

Result 6 concluded that the reply-panel half cannot carry criterion 1 and proposed deterministic checks in the v4/v5 style. Result 7 adds a second, independent reason for the same conclusion and one more requirement on the replacement:

- The failure set is diffuse (Result 6) **and** the accepted set is not separable from it without the judge (Result 7). A judge-only `acceptable` reading therefore measures the judge at least as much as the model.
- Any replacement must also stop a **single canned reply from carrying a quarter of the reading**. Deterministic row checks do that by construction: a check that names the expected content cannot be passed by a greeting. A count of "rows where the reply is `Hello! How can I help you today?`" is itself worth reporting next to any future number on this panel, because it is 18.6 % of the current one.

Evidence: `verdicts-29m.tsv` and `verdicts-100m.tsv` on main; both sealed reports in
`icloud:UOR-R4/results/deepseek/reply-panel-cap96-2026-10-09.tar`; the comparison script
`scripts/open_reply_panel_completed_classify.py`; and this result's own bundle
`icloud:UOR-R4/results/deepseek/reply-panel-accepted-vs-completed-2026-10-09.tar`
(197,632 B, md5 `104f7011b813dc8abd1fbd15a1045740`) with the per-row population table, the
summary JSON, the sealed token counts and `verify.txt`.

## Decision

**The failures are diffuse. Phase 2 is NOT opened: no candidate, no
pre-registration for one, no training, no generation, zero pod spend on this
record.** No single named mechanism dominates the 189 failures: the largest
verdict class (80.4%) is "the judge rejected it on both questions", which is not a
mechanism; the largest text marker (66.1%) is a symptom of unfinished generation,
and the 37.1% of the panel that carries no marker is accepted only 29.1% of the
time — acting on the markers would leave ≈68/232, against a 116/232 target. A 2.7×
gap against that is not one piece. This matches the frozen Step 0a decision rule
and the measured 43→46/232 plateau.

What the classification *does* leave actionable is a corpus/knowledge/capacity
lever that is not bounded by this panel, plus one sharp sub-population:
`K-ill-posed`'s 19 `fluent_only` rows, where a fluent non-answer to an unanswerable
request is being counted as a failure by a judge that has no way to know the request
is unanswerable.

**Amendment (2026-10-09 later).** Result 4 below sharpens the largest marker: it is the
64-token decoding budget, and the `K-clean`/`K-ill-posed` asymmetry is the same budget. The
Phase 2 decision stands unchanged — 85 of 189 failures are budget-caused, but the budget is
a decoder setting, not a mechanism, and the remaining 104 are as diffuse as before. What
Result 4 adds is one bounded, cheap, decision-changing measurement (item 2 of Next) that the
first pass believed was unavailable.

### Pre-registered and declined: the judge-stability re-grade

The re-grade was pre-registered before any spend and then declined by the Lead as
decision-irrelevant (the ±14-cell arithmetic above). The specification is kept here
as the record of what was *not* run, so no one re-derives it later:

- **Pod.** `UOR_POD_VOLUME_DCS=EU-RO-1 uor-pod up --lab deepseek --session reply-panel
  --purpose "open reply panel judge-stability re-grade #2029 phase1" --hours 3`
  (2×RTX 5090, $2.38/h; with the pod already running at the time, $7.36/h of the
  $8.00/h cap).
- **Input.** The three panel files above plus `replies-29m.json` (232 rows,
  88 KB, rebuilt from the sealed report). Nothing generated.
- **Binary.** Release `chat-grade` from main, `grade-replies` path only.
- **Judge.** ollama `qwen2.5:7b` on the pod; hard gate — a pulled digest other than
  `845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e` stops the
  run, because a different digest is a different grader.
- **Run.** `grade-replies out=NEW replies=replies-29m.json grader=qwen2.5:7b
  constants=none`, twice (928 judge calls per pass).

**Cost: $0 spent. No pod was opened for this record.**

### Workspace-rule incident, recorded rather than quietly tidied

This session's first extraction was written to `~/uor-r4-tmp/reply-panel-recon`,
which turned out to be inside another lab's (Claude's) worktree, not an owned
worktree. Claude merged and removed that worktree while the extraction was there
and moved the folder intact to
`~/uor-r4-local/reply-panel-recon-found-20261009/`. It was **my duplicate
extraction of `icloud:UOR-R4/results/deepseek/panel-open-chat-29m-B-lr5e-4-report-root.tar`
(111 MB; its content is `panel-open-chat-29m-B-lr5e-4/checkpoint/model.safetensors` and
`report.json`), not unique
material**; the path was verified as exactly that absolute path and removed at
17:02 on 2026-10-09. All work, scratch and evidence now live only in this session's
own worktree (`~/.worktrees/reply-panel`, i.e. `~/uor-r4-worktrees/reply-panel`),
under the gitignored `local/recon/`. Nothing of this session's remains outside its
own tree.

## Limitations

- **No float generation and no grading was performed**, so nothing here regenerates
  or re-judges a reply. A judge-stability re-grade could only re-judge the same
  text; it could not test a decoder change, a longer token budget or a training
  change, and it was declined as decision-irrelevant (above).
- **The marker rules are text rules, not the judge's reasons.** `qwen2.5:7b`
  answers yes/no only, so every mechanism below the verdict level is my rule and is
  reproducible only as that rule. The `no_terminal`/`repeat5` boundaries were fixed
  in the script before the numbers were read, and the script has a selftest.
- **Cap-truncation and cycle-stop are separated as of Result 4 (2026-10-09 later).** The
  first pass recorded that the tokenizer (`d36d3e87…`) was not on the laptop and that the
  archive has no `inputs/` tree, and left `no_terminal` covering both "hit the 64-token cap"
  and "stopped on the `cycle_repeats=3` rule". That was wrong about availability: the file
  survives at `~/uor-r4-local/workspace/uor-r4-lab/claude-t4-1433-resume/bundle-learned-1/tokenizer.json`,
  hashes to the value the report names, and is the instrument behind the `chat-grade check`
  worst cases already on main. Result 4 uses it: `no_terminal` is the budget, not a cycle
  rule — 85 of the 125 mid-clause failures are at the cap and 119 are within one token of it.
  Exact reply *ids* still do not exist anywhere: the sealed report stores the decoded text
  only, so the counts are canonical re-encodings and are read in a band, as Result 4 states.
- **The judge is not perfectly stable** (the v4 round measured 4 of 64 rows
  changing verdict at the same grader digest) and this panel has no deterministic
  check to fall back on, so **43/232 is a judge verdict with unmeasured noise**.
  The re-grade that would name the unstable rows is pre-registered above and was
  not run.
- **This panel is development evidence.** Step 0a read every failing row of both
  reports on 2026-10-05, so the open panel has been inspected at its misses. A
  future ≥ 116/232 gain on it is open-development evidence, not held-out evidence;
  criterion 1 as written cannot be satisfied as held-out on this panel.
- **One artifact pair, no training seed recorded.** Neither report records a model
  training seed (decoding is greedy, `seed` is not in the graded record), and no
  other artifact on this panel was scored here.


**Follow-up (2026-10-09 later): the deterministic replacement was built, and its coverage is the
headline.** [reply-panel-deterministic-checks-2026-10-09](../reply-panel-deterministic-checks-2026-10-09/README.md)
adds 88 row checks over the rows where a judge-free criterion exists and runs the canned-reply
control through `chat-grade check constants=`: **zero of the 88 checks are passed by any memorised
string**, including `Hello! How can I help you today?`. **88 of 232 rows (37.9 %) are exactly
determinable; the other 144 are open-ended requests for which no judge-free correct answer exists.**
So this panel can carry a deterministic SUB-READING, not a replacement for 116/232, and criterion 1
remains NOT MET.

## Next:

Results 4 to 7 have closed this line's questions on the negative side, one after another: the
cut is the cap (4), the cap is not the constraint (5), the failures are short wrong answers
and diffuse (6), and the accepted set is not separable from them by anything measurable
without the judge (7). **The reply panel's reply half cannot carry criterion 1 as
instrumented**, and what follows replaces the instrument rather than classifying it again.

1. **Build the deterministic replacement for the reply panel's reading** — frozen row checks
   in the v4/v5 style (expected content, forbidden distractors, no word of the distractor's
   key) for the requests that can carry them, so at least one component of the reading cannot
   move between identical runs. **Result 7 adds a requirement to that design: the checks must
   make a canned reply unable to pass**, because one memorised greeting is 18.6 % of all
   accepted rows at 29M and 23.9 % at 100M.
2. **Report the canned-reply count beside any future number on this panel** — occurrences of
   `Hello! How can I help you today?` and their verdicts. It is a quarter of the current
   reading on both artifacts, and a number that does not disclose it is not interpretable.
3. **If a judge is kept at all, state the tolerance and the noise floor with the number**, and
   treat a delta under it as no result. That is the rule already applied to the cap run
   (Result 5) and to the v4 memory panel (4 of 64 rows flipping verdict at the same digest).
4. **Decide whether the reply panel is worth attacking at all.** The recorded lever for the
   completed-but-rejected set is the float model's corpus/knowledge/capacity, which the frozen
   Step 0a decision already named and which no bounded readiness instrument reaches. If it is
   attacked, it needs a fresh sealed panel — Step 0a read every failing row of this one — with
   deterministic checks, a declared cap (Result 5 showed the cap is a measurement choice, not
   a lever) and a diagnosable target of the form "answers a well-posed single-turn request
   without a short wrong answer", which is the defect the 64 rows of Result 6 actually are.

The open reply panel's failures are not one thing, and a 2.7× target is not
reachable by one bounded intervention. That is the result, and no candidate
follows from it.

## Result 6 — the completed-but-rejected failures are SHORT WRONG ANSWERS, and the reply-panel criterion cannot carry criterion 1

Measured from `verdicts-29m.tsv` on main, splitting the 189 failures by whether the reply ended with
terminal punctuation. CPU only: no pod, no generation, no grading, no new artifact.

**Correction: the count is 64 completed-but-rejected, not 62.** Earlier comments in this line said 62;
the per-row table gives **64 rows with `no_terminal=0` out of 189**. The earlier number was mine and is
replaced here.

| property | completed-but-rejected (64) | cut by the cap (125) |
|---|---:|---:|
| `reply_words` median / mean | **10 / 13.9** | **31 / 31.3** |
| `repeat5` | **5 %** | **34 %** |
| `restate_high` | 16 % | 6 % |
| `code_fence` | 12 % | 0 % |
| `question` | 6 % | 3 % |
| `echo_user` / `stub` / `role_leak` / `refusal` | 2 / 3 / 0 / 2 % | 2 / 0 / 0 / 0 % |
| `verdict_class` | neither 41 (64 %), fluent_only 20 (31 %), relevant_only 3 (5 %) | — |
| `tier` | everyday 21, K-ill-posed 21, K-clean 22 | — |

### Three findings

1. **The separator is length, not form.** Completed-but-rejected replies are **short (median 10 words)**;
   cap-cut replies are **long (median 31)**. And `repeat5` at 34 % in the cut set against 5 % in the
   completed set means **repetition was never a mechanism of the real failures — it is a symptom of
   running long.** Any analysis that counts repetition as a failure mode is counting the decoder.
2. **The well-posed-row effect was entirely the cap**, confirmed at the failure level: completed failures
   sit evenly across tiers (21 / 21 / 22). There is no K-clean defect. The earlier structural claim in
   this line is refuted twice — once by the token counts, once here.
3. **They are rejected on both axes.** 64 % are neither fluent nor relevant, with every form marker low.
   These are bad ANSWERS, not mis-formatted ones: the model gives short answers that are wrong.

**They are also diffuse** — 41 neither, 20 fluent-only, 3 relevant-only, no marker above 16 % — which
fires this line's pre-registered stop condition. No bounded intervention is proposed and none was run.

### The reply-panel criterion cannot carry criterion 1

Four measured reasons: the failure set is **diffuse**, classified twice with no dominant mechanism; the
**cap is not the constraint** (2 of 85 cut replies recovered with 50 % more room; 96 tokens scores 38
against 44); the **instrument is judge-borne only** — no frozen row checks exist for this panel, giving a
noise floor of order 14 cells on 232; and the **baseline was a macOS-only binary** until Run A re-grounded
it, its "sealed" status having been assumed rather than tested.

A `>= 116/232` target measured by a judge over diffuse short-wrong-answer failures does not tell anyone
what to build next. That is a criterion defect, not a model result.

### What should replace it

- **Deterministic checks** for the reply panel in the v4/v5 style — expected substring, forbidden
  distractors, no word of the distractor's key — so at least one component of the reading cannot move
  between identical runs. The memory panels have this; the reply panel does not.
- **A diagnosable target.** "Acceptable replies at 116/232" is a judge's opinion aggregated. "The model
  answers a well-posed single-turn request without a short wrong answer" is a defect with a name, a count
  and a fix — and the 64 measured here are exactly that defect, countable deterministically once checks exist.
- **If a judge is kept, state a tolerance and a noise floor** with the number, and treat a delta under it
  as no result — the rule applied to the cap run.

Evidence: `verdicts-29m.tsv` and `verdicts-100m.tsv` on main; both run reports in iCloud as
`reply-panel-cap96-2026-10-09.tar` (1,417,728 bytes, md5 `3de8c41eb9eb7532e1243450365ce335`).
