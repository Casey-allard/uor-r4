# Cross-turn attribute binding: diagnosis and ranked proposals

Review, 2026-10-08. Grounded in the entry-isolation worktree at `a990f906b` and
`docs/evidence/*2026-10-08*`. Read-only: no training, no serving, no code change.

---

## 0. What I checked before reasoning (so the reader can weigh the claims)

* Episode construction and the loss weights: `crates/uor-r4-training/src/dialogue_episodes.rs:502-610`.
  One episode = one response run; every response token weight 1.0, only the terminating
  `<|eos|>` takes `UOR_TERMINAL_WEIGHT` (`:572-593`). A two-turn document contributes two
  independently sampled episodes (turn 1 alone, and turn 2 with turn 1 in its prefix).
* Panel/serving path: `reply_panel` (`crates/uor-r4-training/src/stack_dialogue.rs:790-847`)
  feeds the model's **own** turn-1 reply back into the history for turn 2, inserting EOS if the
  model did not stop. So a degenerate turn 1 changes turn 2's input.
* Architecture actually trained: `scripts/geometric-stack-chat-m1.sh` + the shape records in
  `docs/evidence/*2026-10-08*`. Shapes are `rrarra` w288 (7.16 M, **2 of 6 layers are read
  layers**), `rrarrarr` w512 (19.93 M, **3 of 8**), w640 L10 (33.77 M, **3 of 10**). Episode
  context is 256 (`EPISODE_CONTEXT`), model context 384. No pointer, no flock, no memory layer
  in tonight's arms.
* Tokenizer (`issue-1019/tokenizer/tokenizer.json`, ByteLevel BPE, 4096): `Ġtwo`=1082,
  `Ġcat`=462, `Ġcats`=3022, digits `0`..`9`=18..27 as single ids. `Noted` is **not** a single
  token (so "Noted." is at least two ids and its first id carries most of the signal).
* The ACK families are not incidental: `demand-store-attrshort.rs:906` fixes
  `ACKS = ["Noted.", "I see.", "Got it.", "Right.", "Understood."]` as **every** count/attribute
  family's turn-1 answer; family examples at `:74-84`.
* Instruments that already exist and are undeployed for this question: the pre-registered
  binding probe (`crates/uor-r4-training/src/bin/binding-probe.rs`,
  `binding_probe.rs:1-31`, classification fixed by owner decision before data) and the
  read-binding supervision of Step 7d (`geometric_stack.rs:7816`, landed 2026-10-06,
  never yet run on the chat path).

---

## 1. DIAGNOSIS

### 1.0 Three evidence facts that reframe the problem

The brief says binding "survived three data designs and a capacity increase" and that the
remaining candidates are the objective and the architecture. Three measurements in tonight's
own record point somewhere more specific.

**(i) The same model, before the demand store, DID bind — once, in prose.**
`docs/evidence/probe_fixed_2026-10-08.txt` (round 61): the full_prefix + mask arm, no demand
store, first two content tokens of the retention turn = **"You have two cats, but…"**. The
question is identical to tonight's failing pair ("I have two cats." / "How many cats do I
have?"). A later same-file measurement at 19.9 M gives "You can try keeping an…" — topical,
count absent. So binding fires for "name" (round 101, both retention turns → "My name") and
once for "count" in a prose register, and never in the demanded short register.

**(ii) The failure output is the store's own register, at the slot where the store's answers
live.** "Noted." / "I see." are the store's turn-1 ACK set; "3." is the count family's only
answer when the list has three items. A hard-vocabulary model that had merely failed to bind
would sample from the corpus register ("I", "The", "You"). Sampling the training mixture's own
highest-mass response strings instead is a statement about **where the probability mass sits at
the answer position**, not about information having decayed.

**(iii) The interpolation is the failure.** Round 58/61: no store, prose answers → binds.
Round 113/119: store added, short answers → "Noted."/"I see."/"3.". Long-premise store:
compliance collapses to 2/10, content wrong. Dilution 4×: form recovers, content still wrong.
Witnessed capacity: 19.9 M → "My cat," (entity right, attribute wrong); 33.77 M → "Noted." /
"I see." (register wrong). Every one of these is a movement **along the store's own answer
families**, never a movement toward the context value.

### 1.1 The mechanism (leading hypothesis, confidence: high on the mechanism, medium on the split)

Binding is not absent; it is **conditionally wired and currently outcompeted**.

1. What the model learned is a *phrase-level, corpus-shaped* copy behaviour: when the required
   value sits in a slot it has seen many times in the pre-training corpus
   (`their name is X.` → `What is their name?` → `X`), attention to the value and emission of it
   are coupled. That is what "My name" and "You have two cats" are.
2. What the demand store teaches is a *second-order* dependency: turn 2's question refers to the
   user's turn-1 premise **across the assistant's own intervening utterance**. The store's turn-1
   answers are pure ACKs (`Noted.`), which (a) put a high-frequency, context-independent string in
   the response slot at the turn-1 position, and (b) **never require the model to attend to the
   premise in order to produce turn 1's answer**. So the store trains the association
   "user premise → acknowledge" and simultaneously gives no gradient that discriminates *which*
   span of the premise matters.
3. The store's dependency check does not test what it is believed to test. The "transformation
   fraction 1.000000" rule only asserts that the answer is not a verbatim copy of turn 1's
   *answer* (`demand-store-attrshort.rs:25-32`). It does not assert that the answer is absent from
   turn 1's **premise** — and in the families that matter it is present there (`Metals: iron,
   copper, zinc.` → `How many items did I list?` → `3.`; `Colours: red, orange, yellow.` →
   `What was the second colour?` → `orange.`). The natural induction is therefore
   "bind to the premise when the question's key is present there", which is precisely the correct
   behaviour — but the model only ever had to perform it in a *single* family template, and the
   readout it acquired is the family's canonical answer rather than the context value.
4. At the answer position, the mixture's response-slot prior is dominated by store families
   (short, single-token, repeated 1,100–2,860 times) against a chat corpus whose response-length
   distribution is median 350 tokens with **0.0 %** of replies at ≤ 3 tokens
   (`response_length_distribution_2026-10-08.txt`). Any weak, noisy binding signal at that
   position loses the argmax to the family prior: `3.`, `Noted.`, `I see.`.
5. There is **no explicit span-selection or copy pathway** to rescue it. The `a` layers are
   content-based attention with learned Q/K; copying a specific earlier *token identity* has to
   emerge as an induction-like circuit from data alone. With 2–3 read layers out of 6–10, 384
   context and ~1,000–3,000 binding documents out of 82 M tokens, it does not emerge under a
   loss that also has to fit the store's ACK families.
6. "My cat," at 19.9 M is the most informative single output: the entity is right and the count is
   missing. That is **mass on the correct source span with the wrong feature read out of it**,
   i.e. a readout/selection failure downstream of retrieval — not an unreachable span.

### 1.2 The four candidates, scored honestly

**(a) "Never rewarded, because next-token loss on a mixed-length corpus is dominated by fluent
continuation."** *Partly true, now largely superseded, and not the operative term here.*
Superseded because shape compliance shows the loss does buy commitment when the target is short
(the terminal weight plus the demand store moved 0/6 → 10/10). Not the operative term because the
failures are not "keeps talking" — they are specific wrong strings at the first answer token.
Continuation mass matters only through the response-slot prior, which is (b)+(c) territory.
*Verdict: real, third-order. Do not spend the next run on it.*

**(b) "The mixture's turn-1 answers hijack the second-turn distribution — a positional/role
confusion rather than a memory failure."** *Best supported of the four, and it is two separable
claims.* (b1) The ACK set is the mixture's highest-mass *content-independent* response at the
turn-1 slot, and in the retention panel turn 1's user text ("I have two cats.") is exactly the
surface class the store pairs with an ACK; the model emits "Noted." there, which then enters
turn 2's prefix as a low-information utterance. (b2) At turn 2 the answer distribution is
dominated by store families. Evidence for (b) beyond the outputs: retention failure tracks the
**presence** of the demand store across four mixtures at 7.16 M (`retention_cause_narrowed…`),
present whether the store has 700 or 2,860 documents, whether dependency-carrying or not, and
whether the terminal weight is 1 or 60 — i.e. it is a property of *what the mixture makes likely*,
not of its size or shape. But (b) does **not** explain "My cat," (an anchored attempt) or the
33.77 M "Noted." for a *name* question whose premise never mentioned acknowledging anything. So
(b) is a large, measured contribution, not the whole mechanism.
*Verdict: leading contributor; treat as a data/conditioning defect that a mechanism change must
not be asked to paper over.*

**(c) "Absence of any mechanism that marks WHICH span of the context the question refers to."**
*Most likely the deepest cause, and the one the project can act on.* The model is never told, in
any example or any loss term, which positions of the window hold the answer's value. Attention
is free-form; the read has no key-content constraint; and step 7d's read-binding supervision —
which *is* exactly "put mass on the positions of the expected value" — exists in the tree and has
never been run on this path. "My cat," (entity span selected, attribute not) is the signature of a
readout that resolves *topic* rather than *key→value*.
*Verdict: leading mechanistic cause; this is where a mechanism change is justified.*

**(d) "Architectural limit: bounded causal model with shared typed operators has no explicit
pointer/copy pathway to a specific earlier span."** *True as an architectural statement, but not
sufficient as a diagnosis of this failure, for a reason that matters for sequencing:* the same
architecture bound "name" correctly at 7.16 M and "cats" once at 7.16 M, and cross-entropy does
assign a lower loss to copying the context value than to emitting a prior — the training signal
*can* prefer binding in principle. What is missing is (i) a training pressure that makes the
binding solution win, and (ii) an addressing shortcut so that it does not have to be discovered by
gradient descent. The pointer head is best understood as removing (ii); it should be tried, but it
addresses the *capacity to bind*, not the *incentive to bind*, and the incentive part is currently
mis-set by the mixture (b) and the unmarked spans (c).
*Verdict: plausible and worth one arm, ranked below the cheap incentive/labelling fixes.*

### 1.3 My ordering

`(b)` and `(c)` jointly — mixture conditioning sets the incentive, and the absence of span
marking/selection removes the pathway — with `(d)` as the available mechanism once `(c)` is
confirmed, and `(a)` demoted. Note also a distinct fifth possibility worth one cheap check:
**the failure may partly be output-form, not retrieval** (the panel's first two content tokens
cannot distinguish "does not know 'two'" from "knows the fact in prose form"). The un-store arm
answered in prose; tonight's unanswerable arms get judged as short forms. This is a measurement
gap until the log-probability probe below is run.

### 1.4 The observations that discriminate

| # | Observation | Discriminates |
|---|---|---|
| D1 | **Context-value swap, measured as a log-probability (not a generated string):** for the same turn-2 question, compare `log p(first token of expected value \| context)` with the premise's value replaced by a distractor. If the answer flips and the margin is large, binding exists and the problem is form/readout. If the margin is ≈0, binding is not being used. | (a)/(b) vs (c)/(d); separates "no signal" from "no mechanism" |
| D2 | **Binding probe** (`binding-probe`, owner-preregistered rule, `TAU=0.10`): at the marker/decision query, max attention mass over read layers × heads (and pointer) on the expected-value span vs the distractor-value span, plus the decoded probability of the expected first token. Verdict classes: `unreached` (no mass — the question never addresses the value), `binding-swap` (mass on the wrong key's value), `readout` (mass on the expected span, wrong output). | Splits (c) from (d): `unreached` ⇒ missing span addressing (d); `readout` ⇒ fix the readout/gate/shape (b,d); `binding-swap` ⇒ read-key content (the `a` layers' key representation) |
| D3 | **Pin turn 1's reply** to a fixed, canonical, non-ACK string ("Sure.") and re-run the same two-turn items. If the answer changes, the model's own ACK was blocking its own binding (b1). If it does not, self-conditioning is exonerated. | (b1) vs (c)/(d) |
| D4 | **Logit lens**: decode the hidden state at the answer position with the tied embedding (as `sealed-evaluate` already does) and read the rank of the correct token (`Ġtwo`, `ĠAna`) versus the emitted token. Rank ≤ ~20 with emitted ≠ correct ⇒ readout/form; rank in the thousands ⇒ not represented at that position. | (c)/(d) vs the form/readout variant |

D1–D4 are all **inference-only** on checkpoints that already exist. That is the cheapest
information available in this project right now, and none of it has been run.

---

## 2. PROPOSALS, RANKED BY EXPECTED INFORMATION PER UNIT COMPUTE

Notation: all arms keep the proven frame — chat corpus (hash-proven mask), `policy=full_prefix`,
protocol 2, greedy, panel judged as first two content tokens, plus the guard that a change which
buys compliance by destroying form compliance is not progress. Cost unit ≈ one 19.9 M
`dialogue-train` arm at 12,207 steps (≈8 GPU-minutes at the measured 164 k tokens/s, plus
export/eval).

### P1. Run the binding probe with pinned-turn-1 and swapped values on the existing checkpoints.
**Cost: ~0 (inference only).** *Information: highest of anything available.*

* **Change:** no training. Build a 4-row panel and a `checks.tsv`:
  `cats2 / cats5` ("I have two cats." vs "I have five cats." → "How many cats do I have?");
  `ana / ben` ("My name is Ana." vs "My name is Ben." → "What is my name?"). Checks lines are
  `id<TAB>exact<TAB>-<TAB>expected<TAB>forbidden<TAB>key` (`binding_probe.rs:177-206`), e.g.
  `cats2 exact two five cats`. Run `binding-probe model=… tokenizer=… requests=panel.json
  checks=checks.tsv out=… protocol=2 category=recall`. Add a pinned-turn-1 variant (a ~30-line
  helper that supplies turn 1's reply instead of generating it) so turn 1 is fixed to `Sure.`
  (avoid `Noted.`, which is the model's own default and would confound D3).
* **Why mechanistically:** the probe is the only instrument here that measures *where the mass
  went*, not what string came out. It converts "binding failed" into one of three engineering
  targets with a pre-registered threshold, using the tied-embedding readout the sealed evaluator
  already uses.
* **Implementable now:** the binary, the classification, the thresholds and the span kinds exist
  (`binding_probe.rs:1-31`, `SPAN_KINDS` at `:246`). Pointer fields are `Option`
  (`SpanProbe.pointer`, `geometric_stack.rs:14886`), so a pointerless model is scored on its reads
  alone.
* **Decisive test:** report, per row: mass pattern, `o_E`, `o_D`, and — as an addition — the
  binding margin `Δ = log p(expected first token | correct value) − log p(expected first token |
  swapped value)`. Proposed pre-registered reading (fix before looking): `Δ ≥ 0.7` with correct
  argmax ⇒ binding works, fix form/readout; `|Δ| < 0.1` ⇒ binding not used, go to P3/P4;
  `unreached` class ⇒ go to P6/P7.

### P2. Pinned-turn-1 + prefix-perturbation grid, with the full transcripts.
**Cost: ~0 (inference only).** Cheap and closes the largest measurement gap.

* **Change:** three variants of the same items — (i) the model's own turn 1 (the frozen rule);
  (ii) turn 1 pinned to `Sure.`; (iii) turn 1 pinned to `Noted.` (matching the model's own output,
  as a control for D3). And re-run the panel with `max_new_tokens=40` and report complete strings,
  not two tokens. Additionally score the single-turn form that removes the multi-run structure
  entirely: `"I have two cats. How many cats do I have? Answer with a number only."`.
* **Why mechanistically:** isolates three currently confounded causes — (b1) self-conditioning on
  the model's own ACK, the number of response runs, and the prefix grammar (`\nUser: …\nAssistant:`
  after a reply vs. within one user turn). It also tests the "wrong form, right content" variant
  for free.
* **Implementable now:** `reply_panel` already threads the model's own replies
  (`stack_dialogue.rs:790-847`); pinning is a supplied history rather than a generated one.
* **Decisive test:** `P(answer correct)` under (i)/(ii)/(iii). If (ii) fixes it while (i) does not,
  the mixture's ACK turn-1 answers are the operative defect and P4 is the fix. If the single-turn
  form fixes it, the multi-run structure or the prefix grammar is.

### P3. Commitment-token weighting: put the loss where the decision is.
**Cost: ~1 arm, a small edit to `dialogue_episodes.rs:572-593`.**

* **Change:** keep the terminal weight, and add a *first content token of a demanded answer* weight.
  Concretely: in the per-position weight assignment, identify each masked response run's first
  content token (`position + 1 == prefix_positions`, i.e. the first supervised position) and set
  its weight to `UOR_COMMITMENT_WEIGHT` (sweep 30 / 100). The terminal `<|eos|>` keeps
  `UOR_TERMINAL_WEIGHT` (30 in these arms — the 60× measurement shows the dose band is narrow and
  100× already loses yes/no, so do not push both at once). Everything else stays 1.0.
* **Why mechanistically:** the failure is at the *first* answer token, and the current objective
  spends its supervised mass uniformly over a response plus a large multiplier on EOS. The first
  content token is the whole commitment: it selects the answer's identity before any local surface
  form can dominate. The project already validated this class of intervention with the terminal
  weight (a single scalar moved compliance up, down and to a better point); the commitment token is
  the same lever applied at the other end of the same run, and unlike a length penalty it cannot
  reward truncation.
* **Implementation:** ~10 lines in the weight loop. Note the episode builder sees one response run,
  so "first content token" is unambiguous; no manifest or store change is needed and it applies to
  the chat corpus and every store at once.
* **Decisive test:** one arm at 19.9 M, commitment 30 / terminal 30, same mixture as the
  attrshort-failure recipe. Success is *not* "3. disappears" — it is a **positive Δ from P1** plus
  at least two of four retention rows correct in the demanded form. It also needs a fluency guard
  (chat-only sample) because a first-token weight can push toward store-family openers.
* **Honest risk:** if the store's first answer tokens are mostly ACKs and numerals, up-weighting
  them mainly sharpens the *family prior* the diagnosis blames. This proposal is therefore
  **conditional on P4** in the same arm: pair it with a store whose answers are content-bearing.

### P4. Rebuild the demand store so that turn 1's own answer is content-bearing and the sequence
forces premise binding. **Cost: 1 store generator (~400 lines, modelled on
`demand-store-attrshort.rs`) + 1 arm.**

* **Change, exactly:** two-turn documents, all answers ≤ 4 content tokens (keep the compliant
  envelope: ~1,100 documents, ≤ 10 k response tokens), with three construction rules:
  1. **Turn-1 answers are short, content-bearing in-context copies, not ACKs.** Replace
     `ACKS` with the queried value restated (`Metals: iron, copper, zinc.` → `Iron, copper, zinc.`;
     `My name is Ana.` → `Ana.`; `The number is 20.` → `20.`). This keeps the response mass
     compliant while putting the value **inside the assistant's own utterance**.
  2. **The turn-2 question never contains the answer, and the premise always does**, and the
     question is *paraphrased* across templates (≥ 6 surface forms per family) so the key→span map
     cannot be a template match.
  3. **Value randomization**: draw the list length / ordinal slot / symbol / number from a
     distribution that is uniform *conditional on the question template*, and randomize the
     association between the question template and the answer across the store. This is the
     "cannot be solved by surface statistics" property the brief asks for, obtained without
     inventing nonce tokens (which would be a decoder-only copy target and would not transfer).
* **Why mechanistically:** it attacks (b1) and (c) at once. With ACK turn-1 answers, turn 2's
  question can be answered from the assistant's own prior utterance, and the value is separated
  from the question by an intervening turn; with content-bearing turn-1 answers the value is
  restated *and* the model is forced to decide *which* content to restate. Rule 3 removes the
  family-prior solution, so the only way to lower the loss is to read the premise.
* **Decisive test:** the P1 probe plus the panel. Prediction if (c) is right: `Δ` becomes large and
  positive, expected-span mass rises above `TAU` on the read layers, and the retention rows return
  the correct value in the demanded form on both pairs, with form compliance unchanged (10/10).
  If compliance is preserved but `Δ` stays ≈ 0, the mixture's families are still winning and the
  binding pressure must come from the loss (P5/P6).
* **Cheap checkpoint first (30 minutes, no training):** count, over the *corpus*, how many existing
  two-turn episodes have the queried value in the user's earlier turn and a *short* reply — the
  population that could already teach this. If it is in the thousands, the mixture is the problem
  and P4's store should be small; if it is near zero, P4 is the whole fix.

### P5. Contrastive / direct-margin term on the answer value. **Cost: 1 arm + ~200 lines.**

* **Change:** for binding documents, emit **paired episodes** — the same window with the premise's
  value swapped — and train `L = NLL(correct) + λ · max(0, m − (NLL(swapped) − NLL(correct)))`
  computed over the answer span only (the same masks/weights already used). Fix `m = 2` nats,
  `λ = 1`, and freeze the reading rule before the run. Implementation: the episode builder emits
  pairs; the training loop keeps the two forward passes in one batch (batch 4 pairs instead of 8
  singles to hold memory) and one extra hinge tensor.
* **Why mechanistically:** this optimizes exactly the quantity that separates binding from
  priors — the *difference* in answer likelihood when the context's value changes — which
  token-level MLE can leave flat whenever the surface form is fixed. It is the discriminative form
  of "reward binding", and it is the objective-level analogue of the store-level randomization in
  P4 rule 3.
* **Decisive test:** `Δ` (P1) under paired vs unpaired data at fixed mixture and steps; success is
  `Δ` increasing by ≥ 0.5 nat with unchanged form compliance. **Do this only if P1 shows
  `|Δ| ≈ 0` *with* expected-span mass present** — i.e. as a readout-level fix, not as a cure for
  an unreached span.

### P6. Exact span supervision (Step 7d's read-binding objective), with the mixture fix from P4.
**Cost: 1 label sidecar + 1 arm + re-export. Machinery exists in-tree but has never been run.**

* **Change:** emit teacher-only labels for the store — for each answer with an expected value
  stated earlier, mark the window positions of that value (and of competing, forbidden values)
  and the answer positions that predict it, exactly the `binding_labels.jsonl` /
  `ReadSupervisionTarget` schema already consumed by
  `dialogue-train read_binding_supervision=W read_binding_labels=DIR read_binding_source=LABEL`
  (`geometric_stack.rs:7816-7880`, sidecar format in
  `crates/uor-r4-training/src/bin/dialogue-recall-corpus.rs:5063-5090`). Loss:
  `W · mean(-log(mass on bound value + 1e-6))` added to the masked response NLL, `W ≈ 1`.
* **Why mechanistically:** it is a direct statement of "the answer's value lives at these
  positions"; no amount of mixture tuning can say that. It also tells you which read head picked
  the value up (`bound`/`competing` per head are reported), which is exactly the diagnostic the
  project needs if P6 fails.
* **Caveats to plan for:** needs `policy=full_prefix`, full read admission, `precision=f32` and a
  *geometric* arch; the label generator must be extended to the store (the recall corpus tool has
  it, the demand-store generators do not); and the objective only supervises the read, not the
  output — pair it with P3 or P5 for the readout.
* **Decisive test:** the probe's expected-span mass before/after at fixed everything else. If mass
  goes above `TAU` and `Δ` still stays ≈0, the failure is downstream of attention and the readout
  (P3/P7) is the only remaining lever. That is a decisive negative and worth the arm for it.

### P7. Pointer-copy head with gate supervision, trained from the no-pointer base.
**Cost: 1 fine-tune arm (medium) + export verification. Ranked here, not first, because it is
downstream of P1/P6.**

* **Change:** `pointer=32 pointer_score=lorentz pointer_select=none`, `pointer_gate_supervision=W`
  (the objective at `geometric_stack.rs:7708-7800`: `W·(BCE(g,1) − log p_copy(target))` where a
  source holds the target, `W·BCE(g,0)` where none does). Train by fine-tuning the existing
  19.9 M chat-only base (`init=`) for a few thousand steps on the corrected mixture (P4), not from
  scratch — the pointer adds ~width·dim parameters with fresh initialisation, exactly as the
  step-5 knowledge arm adds a pointer to a trained base.
* **Why mechanistically:** it is the only proposal that supplies an explicit
  question-key → context-span → token-identity pathway. The base's learned attention does the
  semantic work; the pointer gives identity copying a home with its own gate, and the gate
  supervision forces the gate to be open exactly where the answer is present in the context — which
  is what makes the copy pathway trainable at this scale rather than hoping an induction circuit
  self-assembles.
* **Honest limitations, stated before the run:**
  * the pointer keys are `W_k h_j` (hidden state), not token ids, so the head must *learn* identity
    matching; there is no built-in "copy the same token" bias. If the probe shows the pointer's
    copy mass never lands on the expected span, that is this limitation, and the fix is a
    token-identity key (a small, well-scoped change: key = embedding of the source token, i.e. use
    the tied table rather than a learned projection).
  * a *selected* pointer (`top:K`) is refused by `dialogue-train` (no gradient with one kept
    source) and by export (no D11 port), so training and serving must use the full-source
    (unselected) pointer. That is trainable and exportable, but it also means the copy
    distribution is diluted over the whole window; keep `dim` at 32–64 and check the pointer's
    softmax sharpness in the report.
  * pointer models change the evaluation protocol (`snap-evaluate` refuses them); score with
    `evaluate`/`dialogue-train` development so comparisons are on one path.
* **Decisive test:** probe the pointer's own attention mass on the expected span plus `Δ` before
  and after fine-tuning. If the pointer mass on the expected span rises above `TAU` and `Δ` stays
  flat, the readout path (not the copy path) is the bottleneck, and P3/P5 are the answer.

### P8. Curriculum: chat-only first, then the demand mixture (the interference is destructive, so
sequence it). **Cost: 2 stages of one arm.**

* **Change:** stage A = chat corpus only to near-convergence (this reproduces the arm that binds
  "My name"); stage B = continue for ~2–3 k steps on the mixture with the demand mass diluted 4×
  and the terminal weight at 30. Compare against the single-stage mixture at matched total tokens.
* **Why mechanistically:** the four measured facts (presence of the store destroys retention;
  dilution does not restore it; a 70 % mass cut does not restore it; capacity does not restore it)
  say the damage is not proportional to store size but to *what the store makes likely at the
  moment the chat behaviour is being learned*. A second, short, low-weight phase lets the chat
  behaviour set the response-slot prior first and then be adjusted, rather than being outvoted
  throughout. It is also the cheapest test of "is this forgetting/interference, or is it a
  capability that was never there?".
* **Decisive test:** retention and `Δ` after stage B; the prediction under (b) is that retention
  survives better than in the single-stage arm at the same token count, and `Δ` stays ≈0 under (c).

### P9. The native geometric machinery — name the specific mechanism and its missing piece.
**Cost: high; ranked last deliberately.**

The honest statement is that the project *has* address machinery and it is **not wired into the
chat stack**:

* `geometric_address.rs` (separated content/context 2I-root addressing, exact relative element
  `table[inverse(query), key]`, learned unary/bilinear potentials, radius bins, presence flag) and
  `geometric_span.rs` (learned HOLD/OPEN/APPEND/COMMIT ordered spans over exact 2I root products,
  which is literally an ordered-span *addressing* mechanism) are registered in
  `uor-r4-training/src/lib.rs:37,56` and used only by the standalone
  `attention-geometric-*` examples — nothing in `geometric-stack.rs`'s chat path consumes
  `geometric_address`, and `read_span_probe` explicitly refuses models with either
  (`geometric_stack.rs:7373-7386`).
* The **missing piece** for binding is therefore concrete: (i) construct candidate spans from the
  prefix with a control signal (the `OPEN/APPEND/COMMIT` controller already exists and is
  trainable), (ii) let the *question* address those spans — the exact match table
  `table[inverse(query), key]` gives an exact, multiplier-free address comparison — and (iii) feed
  the selected span's *token identity* to the output as a copy channel (the pointer head's gate is
  the natural readout). Until (i)–(iii) are connected, the geometric span machinery is an unused
  lane rather than an explanation for tonight's failure.
* The **prime-route pointer** (`pointer_route=prime:WINDOW`, ADR-0003) is *not* the right tool here:
  it admits sources by shared prime factors / n-gram match between the query's last WINDOW tokens
  and each source's preceding WINDOW. Our question shares no n-gram with the premise-value span,
  so the route admits nothing relevant. It solves a different problem (phrase lookup).
* **Cheapest honest way to use this lane:** run `attention-geometric-span` /
  `attention-geometric-address`'s existing finite-task protocol on a *binding* task with the
  store's format (unique value per key, paraphrase the question) before wiring anything into the
  chat stack. If the exact-address read cannot beat the plain read on that finite task, do not wire
  it; if it does, the wiring above is the follow-on.

### P10. Protocol echo — recommend against. **Cost: high, payoff: low, risk: medium.**

Making the model echo the relevant span before answering converts a retrieval problem into a
generation problem: it lengthens the response (against the measured rule that answer length in the
mixture governs compliance), it fights the 60× terminal weight (two EOS-bearing runs per turn), it
needs data to teach the echo policy, and the echo string is itself a copy target — so the pointer
(P7) achieves the same thing at lower cost and without a protocol change. The one useful variant is
*diagnostic, not a fix*: a serving-time extractor that reads the count/number out of whatever prose
the model emits, purely to measure whether the fact is expressed somewhere. Label it as such so it
never becomes an oracle in the served path.

---

## 3. THE MOST DECISIVE SINGLE EXPERIMENT

**Run the pre-registered binding probe on the best existing 19.9 M checkpoint, with turn 1 pinned
to `Sure.` and the premise value swapped, over four retention rows — and read the answer's
log-probability margin, not the generated string. Inference only; no training.**

**Exact configuration**

* Models: the 19.9 M `rrarrarr` w512 chat arm **without** the demand store (`init` base) *and* the
  19.9 M arm trained on the 2,860-document two-turn store; optionally the 33.77 M attrshort
  checkpoint. Same tokenizer (`issue-1019`), protocol 2, greedy, `max_new_tokens=24`.
* Rows (4 + 4 swapped): `cats2` = "I have two cats." / "How many cats do I have?";
  `cats5` = "I have five cats." / same question; `ana` = "My name is Ana." / "What is my name?";
  `ben` = "My name is Ben." / same question. Question turns phrased twice (with and without
  "Answer with a number only. / Reply with one word only.") so form demand is not a hidden factor.
* `checks.tsv` (six columns, tab-separated):
  `cats2 <TAB> exact <TAB> - <TAB> two <TAB> five <TAB> cats`, and analogues
  (`ana … ana ben name`). `binding_probe.rs:177-206` requires expected and forbidden terms.
* Turn 1's reply pinned to `Sure.` (a canonical chat-corpus reply that is not in the ACK set),
  then `binding-probe … category=recall`.
* Report per row: mass pattern (`expected`/`distractor`/`neither`) with `E`, `D` at marker and
  decision queries; pointer gate and copy mass (null if no pointer); `o_E`, `o_D`; and the
  **binding margin** Δ = log p(first token of expected value | correct premise) − log p(same token
  | swapped premise). Pre-register the reading before the run: Δ ≥ 0.7 nats with correct argmax in
  both members of a pair ⇒ binding present, fix form/readout; |Δ| < 0.1 ⇒ binding not in use.

**Predictions under each diagnosis**

* (a) loss dominated by continuation: Δ large and positive, correct argmax, wrong *length/register*
  in the string; probe mass on the expected span. Predicted by nothing else.
* (b) ACK/positional hijack: with turn 1 pinned, Δ becomes clearly positive (the model can bind
  when its own ACK is not in the prefix); with its own `Noted.` turn 1 (natural run), Δ collapses.
  The pair `(pinned, natural)` differing is the signature.
* (c) no span marking: Δ ≈ 0 in **both** pinned and natural runs, while the probe's max read mass
  on the expected span is **below TAU** (mass class `neither`) at the marker query — the question
  never addresses the value. Possibly mass on the value at the *decision* query if the reply
  happens to name it.
* (d) architectural copy gap with the span reachable: max read mass **above TAU** on the expected
  span but the output preferring a distractor or a prior (class `readout`, or `binding-swap` if the
  mass lands on the competing value). This is the configuration in which P7/P6 are the right tools.
* Form-only variant: Δ positive but the demanded short form not produced (prose answer carrying the
  value) ⇒ the retention failure is a form-transfer problem; fix with P3 + demand-shape data, not
  with a copy mechanism.

**Falsifier for the leading diagnosis**

The leading diagnosis is (b)+(c): the mixture's register supplies the wrong answer and no
mechanism marks the queried span. It is **ruled out** if, with turn 1 pinned to a canonical
non-ACK reply, Δ is large and positive (≥ 0.7 nats, correct argmax) on all four rows **while form
compliance for those rows is what the panel records** — i.e. the model binds whenever it is asked
in a register it recognises. That would relocate the problem to register/form transfer, where the
fix is a data-shape and commitment-weighting problem (P3/P4), not a mechanism one. A second
partial falsifier: if the probe shows expected-span mass above TAU and Δ ≥ 0.7 but the *string*
still opens with a prior token, the failure is readout-only and (c)/(d) are both unnecessary.

---

## 4. WHAT NOT TO DO

1. **Do not scale capacity again** (7.16 M → 19.93 M → 33.77 M is already run; 1.7× changed the ACK
   variant and nothing else). Likewise no 2×/4× step increases: the 24,414-step arm changed
   nothing on the target.
2. **Do not build more dependency-carrying documents of the same design.** The transformation
   fraction is not the property that matters — it constrains the answer against turn 1's *answer*,
   not against the *premise*, and it never checks the answer position's readout. Three such
   designs have already failed.
3. **Do not add long premises or long answers** to fix binding. Answer length in the mixture is a
   measured compliance killer (2/10 with the long-premise store); a binding curriculum must keep
   answers ≤ ~4 content tokens.
4. **Do not tune the terminal weight further** (20× partial, 60× best, 100×/200× collapse is a
   complete bracket). If a new arm needs headroom, lower it to 30 and use commitment weighting (P3)
   — but never push both multipliers up together.
5. **Do not select or tune on development NLL.** Four arms now disagree with behaviour in both
   directions. Use the panel plus the probe's mass/margin numbers, fixed before the run.
6. **Do not try to buy binding with prompt-side scaffolding, few-shot exemplars, or an echo
   protocol.** Round 46 measured scaffolded prompts changing nothing; the fitted template merely
   moved into the output ("Question: Answer with one"). Binding must appear in the training signal
   or the architecture.
7. **Do not "fix" the panel by widening it before instrumenting it.** The first-two-content-token
   reading is frozen and comparable; add the log-probability and mass channels *alongside* it
   rather than changing what the panel reads. In particular, always report full transcripts at
   `max_new_tokens ≥ 24` in addition — "My cat," and "Noted." are prefixes, and the current record
   cannot distinguish a bad answer from a truncated one.
8. **Do not reach for the geometric span/address lane as a fix before it has been measured on a
   binding task.** The machinery is real but unused by the chat stack; wiring it in without a
   finite-task result would be the largest available spend with the weakest available evidence.
9. **Do not adopt a serving-time oracle extractor as the answer.** It can be a diagnostic, but it
   would move the capability out of the model, which contradicts the mission contract.

---

## 5. Uncertainty, stated plainly

* **High confidence:** the failure is mixture- and readout-conditioned rather than a pure capacity
  or pure objective defect; the ACK turn-1 answers and the store's short-answer families are
  causally involved (the retention failure tracks the store's presence across four mixtures); no
  mechanism currently marks the queried span; the discrimination experiments are cheap and have not
  been run.
* **Medium confidence:** that (c) (missing span addressing) is the deepest cause rather than (b)
  (mixture conditioning) — D2/P1 settles this, and it is not settled by anything measured so far.
* **Speculation (flagged as such):** that a token-identity-keyed pointer (P7 with an embedding key
  rather than `W_k h_j`) is required rather than the existing learned-key pointer; and that the
  native exact-address lane (P9) can be trained to beat plain attention on this task inside the
  chat stack. Both are plausible and neither is measured.
* **Not claimed:** any scaling law beyond 7–34 M; any statement about models that did not run
  tonight; any inference from the entry-scorer board, which this review did not consult.
