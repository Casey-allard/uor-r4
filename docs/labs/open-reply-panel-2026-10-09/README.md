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
declared rather than reconciled.

**A second artifact, same shape.** The 96M `chat-100m-C` report
(`ladder/grades/chat-100m-C-powered-7b/report.json`, sha256
`3d5a2914a444905d041f510d1c7bdc700cb2ee1c674727fb4668600acadbccf6`, model sha256
`a746e1fc…`, 96,023,745 params, same grader and instrument sha256) scores
**46/232**: `neither` 145/186 = 78.0%, `fluent_only` 34 = 18.3%, `relevant_only` 7 =
3.8%; `no_terminal` 117/186 = 62.9%, `repeat5` 24 = 12.9%, `other` 40 = 21.5%. The
diffusion is not a property of the 29M model; it is stable across a 3.3× scale
step, which is also what the recorded 43–46/232 plateau from 29M to 96M says.

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
- **Cap-truncation and cycle-stop are not separated.** The tokenizer
  (`d36d3e87…`) is not on the laptop and the archive has no `inputs/` tree, so
  exact reply token counts are unavailable; `no_terminal` covers both "hit the
  64-token cap" and "stopped on chat-grade's `cycle_repeats=3` rule".
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

## Next:

Two decisions, neither of them another diagnostic:

1. **Repair criterion 1's reading** — add deterministic row checks to the reply
   panel (the v4/v5 pattern) or state a tolerance for the judge — before anyone
   trains against 116/232. The criterion currently cannot distinguish a real gain
   from judge noise, and no candidate number on it should be accepted until that is
   fixed.
2. **Decide whether the reply panel is worth attacking at all.** On this record the
   lever is the float model's corpus/knowledge/capacity, which the frozen Step 0a
   decision already named and which no bounded readiness instrument reaches; if it
   is attacked, it needs a fresh sealed panel, because Step 0a read every failing
   row of this one.

The open reply panel's failures are not one thing, and a 2.7× target is not
reachable by one bounded intervention. That is the result, and no candidate
follows from it.
