# ASSESSMENT: why no served reply is complete (one-model-end-to-end, claude-326)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Owner decision: "Yes, pivot assessment" (#2028, 6726003019). No compute was spent. Every number below is cited to a row of [float-training.md](../../../../research/notebook/float-training.md) on `main` at `fa5004861` (cited as "row N", the file's line number), a REPORT, or a source line.

## The answer in one paragraph

**The replies fail at one position: the reply's first token, where the model has to choose which value in the prompt the question asks for.** Data, scale and the objective each moved something measurable from 4 to 8 November: stopping, held-out text, and copying. None of them reliably moved this choice.

**The leading hypothesis is the mechanism.**
- The read's copy heads match the current token, so they cannot make the choice by construction.
- Its only question-dependent path is the type heads' learned query, and no learned-query read on record has learned where to look.

**The record cannot yet tell data from mechanism at the first token:**
- The first-token measurement exists only for DeepSeek's checkpoints. They do not open with either prompt value (median rank 956–1,189 of 4,096). Their mix scored recall replies that were not bare values, so that rank mixes reply format with choice.
- claude's own served models do put a prompt value first, and one seed picks the asked one about twice as often as the other (76 to 36, row 15).

Three readings decide it, in this order:
1. claude-325, the first run trained on panel-shaped bare answers, which is training now;
2. a laptop reading of claude's own seals at the first token (Q4, Part A);
3. a synthetic recall probe whose query is not the key, run under the panel's header gap (Q4, Part B).

## What did not work, first

- **0 complete replies and 0 pairs on `m2-fresh256` in every served configuration, 4–8 Nov** (bar: 104 of 256 complete, 52 of 128 pairs):
  - claude: rows 13, 14, 15, 22, 25, 28 and 34;
  - DeepSeek's SFT: rows 17, 21, 23 and 26;
  - claude-259's six served artifacts: row 30.
- **Every lever on this line was one knob on (b64L)'s recipe, read on the same panel.** None measured the step where the replies fail.
- **D21's pivot card was due long before this unit.** Three merged PRs on one line without the headline moving force one, and this line has had more than ten since claude-309 (rows 13–34).
- **The first-token measurement exists only for DeepSeek's checkpoints** (rows 17, 18). No claude checkpoint from claude-308 to claude-325 has one.

## Q1. What blocks a complete reply: data, scale, objective or mechanism?

### The panel's task, and where the replies fail

A panel prompt states two facts and asks for one, and the reply is the asked value. claude-324 lists the panel's served layout (`one-user-turn/v1`) and builds its TRAIN corpus in that shape. claude-324's REPORT gives one generated row:

```
User: I am a bowl mall round teacup turtle snowy by trade. I just moved to shrimp donut hairbrush coffee. What is my job?
Assistant: bowl mall round teacup turtle snowy.<|eos|>
```

The reply's first target is the asked value's first word. Every later target either continues that value or closes the reply.

DeepSeek scored this split teacher-forced on one SFT artifact, the content-ratio recipe's selected step 512 (row 18; [deepseek-34 REPORT](../deepseek-34/REPORT.md)):

| Position | Rendering | top-1 | top-8 | top-32 | rank min / median / max (of 4,096) |
|---|---|---|---|---|---|
| **First target** | `per-turn/v1` | 0/256 | 0/256 | 0/256 | 67 / 956 / 4,055 |
| **First target** | `one-user-turn/v1` | 0/256 | 0/256 | 0/256 | 43 / 1,189 / 4,060 |
| **Later targets,** after a correct first token | `per-turn/v1` | 1,614/2,576 | 2,574/2,576 | n/a | n/a |
| **Later targets,** after a correct first token | `one-user-turn/v1` | 1,600/2,576 | 2,574/2,576 | n/a | n/a |

- **The prompt barely moves the answer position.** The same run's opening query spread was 0.26249 there, against 2.06252 over all rows.
- **Repeated openings did not help.** deepseek-36 repeated reply openings eight times in training (row 17; one artifact, rendering not split):
  - first-target top-1 and top-8 were 0/256 at every checkpoint, with median ranks 1,025–1,134;
  - first-target top-32 was 0/256 at the selected step 256.
- **These artifacts do not open with prompt content at all.** A median rank near 1,000 means neither of the prompt's values is near the top, which is a failure to open with prompt content, not only a failure to choose between two values.
- **Their training mix held recall replies, but not bare ones, as far as the record shows.**
  - Row 32: OASST2's reply-masked TRAIN root plus 20,000 M-World oracle-memory recall conversations, median reply 11 tokens.
  - Row 21's content-ratio mix is 67.98% M-World by tokens and 43.35% by scored targets.
  - The panel's first target is the bare value's first word. A model trained on framed replies ("Your job is …") ranks its frame's first word on top, so the bare value ranks low there however well it would choose a few tokens later.
  - So **on these artifacts the first-target rank measures format and choice together.** Whether M-World's replies open with a frame is not checked here; Part A's teacher-forced reading on claude-325, trained on bare replies, does not have this confound.

### claude's served replies: copying, sometimes selective

claude's models trained on D3 + mw-balp mixes do put a value from the prompt on the reply's first line. The counts below are asked value / the other value; a row holds one or the other, never both:

| Run | Seed 1 | Seed 2 |
|---|---|---|
| claude-321, capped × 3 (row 15) | 25 / 21 (the row: "near chance at seed 1") | **76 / 36** |
| claude-322, capped × 6 (row 14) | 14 / 11 | 2 / 6 |
| claude-323, mixed keys `2,1` (row 13) | 0 / 0 | 32 / 30 |

- **Seed 2 of claude-321 is the strongest selective reading on record,** about 2 to 1. No run selects consistently across seeds, and none completes a reply.
- **These counts are free-generation first lines, not teacher-forced ranks.** No claude checkpoint has a first-target rank, so the two tables above are not directly comparable.

### Why later targets succeed: the read's design

The Hamming read's module doc (`crates/uor-r4-float-train/src/flock_read_hamming.rs` on `main`) describes two kinds of head:
- **The copy heads:** the key of an earlier position `s` is the table point of token `x_{s−1}`. A copy head's query is the current token `x_t` through the same rotation, so "an earlier copy of the current token is an exact match" (lines 14–17). What it brings back is "the token after the last copy of the current token" (lines 47–49).
- **The type heads:** their query is "the residual through a second rotation, which must learn what to ask for" (line 18).

**The later targets** fit the copy heads' design. Once the first value word is given, that word also sits in the prompt, and the copy head brings back the next word. That is the induction match it is built for.

**At the first target** the current token is the role header's last token, so a copy head can bring back only what followed that token's earlier copies. Choosing *job* or *home* from "What is my job?" needs a query that depends on the question, and only the type heads have one.

**Every learned-query read on record has missed that dependence:**
- The read's doc: reads keyed on the trunk's state "never learned where to look (claude-144, claude-145, and this read's first form)" (lines 4–5).
- The 27 Oct reads, each in its row's own words:
  - the content read: "Replies still ignore the prompt" (row 110);
  - the induction read's language run: "Replies still mostly ignore the prompt; one carries the code word's first pieces" (row 108);
  - the type heads with SpiralCore's spin: "Every arm picks 1–4 distinct opening tokens, so openings track which template wins; replies still ignore the prompt" (row 106).

**The hypothesis has one weak point.** claude-321's seed 2 (76 / 36) shows that something in the trained read prefers the asked value some of the time. Whether that is the type heads, or a copy match through a token both the question and the fact share, is not measured. Part A's trace reads it.

### Each candidate against the record

| Candidate | What it moved | What it did not move | What the record says |
|---|---|---|---|
| **Data**: mw-balp alone, D3 mixes × 1 / × 3 / capped × 3 / capped × 6, D4, DeepSeek's ratio, dose and opening weight (rows 14, 15, 17, 21–23, 25, 26, 28, 39) | **Stopping**: 0 → 253 / 244 EOS stops (row 28), and EOS median rank 7 → 2 (row 26). **Copying a prompt value first**: rows 13–15 | Complete replies: 0 in every one. Selection was inconsistent across seeds (the table above) | **Not excluded.** Every corpus so far scored long or stock replies. A corpus whose scored reply *is* the choice, in the panel's shape, was first built in claude-324 and is training now (claude-325) |
| **Scale**: width 128 → 256 (row 16), 512 and 1,024 (rows 16, 41–43), D3 → D4 (row 39), steps (rows 38, 46), the capacity sweep (row 81) | Held-out text: width 256 is ahead of 128 by d +115,279 / +134,421 (row 16) | Recall: "No arm carries" (row 81). Width 512 and above never trained to the end on D3 (b64L) (rows 16, 41–43). Row 81's (w) 512/8 trained, and is inconclusive until its export is repaired | **Not shown to be the cause.** The read's design recalls exactly on the 27 Oct probe (row 109), so retrieval itself is within reach. A path from the question to the value is what has not been shown |
| **Objective**: the nearest objective against cell, fixed targets and learning rate (rows 46–48) | Human text learned: (m1) and (near) are ahead of the control (row 47); (b64L) meets the one-track bar (row 44) | n/a | **Not separable from the mechanism at the first token on this record.** Part B's control arm and its header arm separate them |
| **Mechanism**: the read's query path | Exact-token recall 0.984–0.994 in the far buckets, no read 0.000–0.002 (row 109); continuation copying (row 18) | First-token choice: 0/256 top-32 on DeepSeek's artifacts (row 18); inconsistent on claude's (rows 13–15); every learned-query read on record (rows 106, 108, 110) | **The leading hypothesis, untested.** Consistent with every row except the partial selection at claude-321 seed 2 |

**Serving is not the blocker:**
- teacher-forced float and integer agree on token choice (row 20: 0 and 1 mismatches of 2,832 targets);
- the first target fails teacher-forced, before any free generation.

Free-generation parity is still not established: 27 of 64 whole replies differ (row 35).

## Q2. What is established about selecting the queried value, and what was never measured

**Established:**
1. **Exact-token retrieval** (row 109): the induction-keyed read recalls 0.984–0.994 at distances 33–180 on the MQAR-style probe, at seeds 1 and 2. **In that probe the query writes the key again** (`in_space_long.rs:11803`, "Its query writes the key again at `q = p + d`"). So it tests only the case where the question token is the key token, and where the key is the current token.
2. **Continuation once aligned** (row 18): later targets are 1,614 / 2,576 top-1 (`per-turn/v1`) and 2,574 / 2,576 top-8.
3. **On DeepSeek's two SFT artifacts, the first target is never near the top:**
   - 0/256 top-32 under both renderings on the content-ratio artifact (row 18);
   - 0/256 top-8 at every checkpoint of the opening-weight run (row 17).
4. **Copying, sometimes selective, in claude's served replies** (rows 13–15): the first line often holds one of the prompt's two values. Selection is about 2 to 1 at claude-321's seed 2 and near chance elsewhere.

**Never measured:**
1. **The first-target rank on claude's own checkpoints:**
   - claude-313 to claude-325, teacher-forced, under `served-reply-panel`'s `first_target` tally (added by deepseek-34);
   - the seals of claude-322, claude-323 and claude-325 are stored;
   - a laptop measurement.
2. **Recall where the query is not the key, and where it sits a few tokens before the scored position.** Both hold in the panel, where "job" names the value after "I am a … by trade" and the scored position is past `Assistant:`. The 27 Oct probe posed neither.
3. **What the read keeps at the first reply position:** whether the asked value's position is among a head's 16 kept positions (`HAMMING_KEPT`), for copy and for type heads, and with what weight.
   - The integer serve already writes Hamming traces (`HammingTrace`, used by `fbs-toks`).
   - claude-259's trace is the one prior reading. The module doc cites it only for the one-token copy key reading every earlier `" is"` alike (lines 32–34).
4. **Whether the answer position's state carries the question word.**
   - The module doc says a token 16 back moves the trunk's prediction by 1–2% of the last token (lines 3–4).
   - The question word is about 4 tokens before the answer position, and that distance has not been measured.
   - Row 18's spread (0.26249 against 2.06252) suggests the state carries little.

## Q3. The levers: closed, parked, never tried

**Closed configurations** (rows read negative or "closed"; a configuration, never a mechanism, §4.1):
- **Corpora:**
  - mw-balp alone (row 28);
  - D3 + mw-balp × 1 (row 25), × 3 (row 22), capped × 3 (row 15) and capped × 6 (row 14).
- **DeepSeek's SFT:** the ratio (row 21), the dose (row 23) and opening weight 8 (row 17), each "closed" in its row.
- **Copy keys:** the slot key's atom rule, negative on its oracle (row 66).
- **Reads:** the content read (row 110), the type spin (row 106) and the induction read's language run (row 108), all at their 27 Oct recipes.
- **Capacity:** the sweep's (h) arm (row 81: "(h) negative at this data and recipe").
- **Long context:** P2b's read configuration (rows 53, 57).

**Inconclusive, open:**
- D4 (row 39);
- copy key 2 (rows 67, 76);
- mixed keys `2,1` at × 6 (row 13: inconclusive; the configuration closed, the lever open);
- the sweep's (w) arm (row 81).

**Parked or blocked:**
- width 512 and 1,024 (memory, rows 16, 41–43);
- 32,768 steps (stopped by decision, row 38);
- the decode-time read (bounded at 5 rows, claude-323);
- `integer-nearest-readout` (owner stop, 2 Nov);
- writer admission (#3394, open).

**Never tried:**
- **(a) The panel-shaped bare-answer corpus** (claude-324), where the scored reply is the choice. claude-325 trains it now.
- **(b) A read whose query is built from the question.** The read would find a content word of the question (a first Hamming match) and use what it found to key the copy match. This is the composition a transformer's induction circuit uses, built here with hard Hamming top-k and no exponential.
- **(c) A synthetic probe of recall where the query is not the key,** with or without a header gap.
- **(d) The first-target rank and a Hamming trace on claude's checkpoints.**

## Q4. The smallest structural test, and what each result unlocks

**A proposal for the next unit (claude-327), not a run.** Part B is a GPU run and takes a one-page pre-registration with a non-author FIT and a pre-run review first (§4.5).

### Part A (laptop, no GPU): read what claude's served models do at the first token

On the stored seals of claude-321 (if stored), claude-322 and claude-323, and claude-325's when they land, at both seeds:
1. **The first-target ranks:** `served-reply-panel`'s `first_target` tally.
2. **A Hamming trace at the first reply position of each row:**
   - for each head (copy and type), whether its 16 kept positions include the asked value's first token, the other value's first token, or neither;
   - each head's weight.

Each finding unlocks one next step:
- **First-target ranks near the top and selective:** claude's recipes are close, and the gap is in finishing, not choosing. The next lever is on the language side, starting from claude-325's reading.
- **Type heads keep the asked value's position with low weight:** the query finds the value and the combine step does not use it. The next lever is the read's weights or margin, tested first on Part B's probe.
- **Heads keep both values' positions about equally:** the query reaches the value slots but not the question. Part B decides whether a learned query can learn the choice.
- **Neither value is kept by any head:** the query never reaches the value slots. Part B is required before any further language run.

### Part B (GPU, finite, about $0.5–1): recall where the query is not the key

Take the 27 Oct probe (row 109; `recall_window`, `--recall`) at (b64L)'s read settings, and change the query in two steps. Each arm adds one change to the arm before it.

| Arm | The query | Separates |
|---|---|---|
| **B0, the control** | the key again, at row 109's own settings (its sheet's trunk, `--recall 4`, 4,096 steps); must reproduce its ≥ 0.98, or the run is an instrument fault. B1 and B2 then run at those same settings | instrument |
| **B1, two-class** | each window holds values of two slot types, marked by type tokens; the query names a type with a token from a disjoint range, and the target is that type's latest value. This is the panel's choice | choosing by a learned query |
| **B2, header gap** | B1, with a fixed 3–4-token header between the query token and the scored position, as `?` … `Assistant:` sits in the panel | carrying the question to the answer position |
| **B-none** | B2 with no read | chance |

At seeds 1 and 2. **The bar for B1 and B2 is row 109's own:** R ≥ 0.50 and R ≥ R(B-none) + 0.10, at both seeds.

Each result unlocks one next step:
- **B0 fails:** an instrument fault, repaired before any reading.
- **B1 fails, B0 passes:** this read's configuration (type-head rotation query, read 4, width 256) cannot learn a query-conditioned choice. That is a mechanism-side result for the configuration (§4.1, §4.2). The next unit builds lever (b), the question-keyed read, under its own sheet. It must pass B1 and B2 before any language run.
- **B1 passes, B2 fails:** the read can choose, but the trunk does not carry the question four tokens forward. The lever is the carry: a read at the question token whose result is written forward, or a header-free answer position. It is tested on B2 first.
- **B1 and B2 pass:** a learned query can make the panel's choice in this read. The blocker is then data or the language setting, and the next language run is claude-325's recipe, made larger or given a curriculum from B2's form to the panel's.

### How claude-325 folds in

claude-325 trains the panel-shaped bare-answer corpus (claude-324), so it is the language-side test of the same question. Its rows are pre-registered (its sheet's outcome rows 0a–f):
- **(b), complete above 0 at a seed:** a learned query *can* make the choice in language when the scored target is the choice. Data was a blocker, and Part B ranks behind scaling that recipe.
- **(c), bare replies but 0 complete, or (e), no bare replies:** the mechanism hypothesis gains weight. Part B goes first.

Either way, Part A runs on its seals.

## Recommendation: a decision card for claude-327 (ORDER claim check (a))

No bar, scope or cap changes here.
1. **Start no further one-knob language run on this line** (corpus mix, ratio, copy keys, steps) until Part A is read and Part B has run. This is a lab decision under §5.1, recorded in this unit's REPORT and taken up in claude-327's ORDER.
2. **claude-327 is Part A plus Part B's sheet:** a pre-registration, a non-author FIT and a pre-run review. Then comes the run, in this item's scope.
3. **The roadmap item's next step** names this test in place of the next corpus lever.

Nothing here needs Casey: §3, the caps and the charter are unchanged.
