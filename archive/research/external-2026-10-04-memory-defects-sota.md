# Three conversational-memory defects: state of the art and standard practice

**External research report — 2026-10-04**
Scope: an experimental 3-turn conversational memory (FACT / DISTRACTOR / QUESTION), a learned
compiler mapping each turn to a `(relation, act)` pair, and an exact addressed store written and read.
Serving constraint: **no float matmul at serving time** — integer/table-lookup only.

Evidence grading used throughout:

- **[EST]** well-established: peer-reviewed, ≥5 years old, replicated, in textbooks/surveys.
- **[REC]** recent (2025–2026): preprint or single-paper result, plausible and relevant but not yet
  independently replicated. Treat numbers as provisional.

---

## DEFECT 1 — 60% of question turns are not recognised as queries

Symptom: question turns classified as statements or unresolved; no read attempted. Questions and
statements share the relation phrase ("my dentist" appears in both).

### (a) Established approaches

**1. Frame it as dialogue-act tagging, not as question detection. [EST]**
The canonical treatment is Stolcke et al. 2000, which models *Statement, Question, Backchannel,
Agreement, Disagreement, Apology* etc. as a hidden Markov model over dialogue-act n-grams with
lexical, collocational and prosodic cues. Their 42-tag Switchboard result: **65%** accuracy from
errorful recognised words + prosody, **71%** from word transcripts, against **35%** chance and
**84%** human agreement. Two things matter for you:

- A 42-way act problem is genuinely hard; the **binary `Question` vs `Statement` collapse of the same
  tagset is far easier**. Inventory design is the lever, not classifier capacity.
- The tagset explicitly separates `Statement-non-opinion`, `Yes-No-Question`, `Wh-Question`, and
  `Declarative Yes-No-Question`. Your `unresolved` bucket should be a **rejection** decision, not a
  third learned class.
  <https://arxiv.org/abs/cs/0006023> · Computational Linguistics 26(3):339–373 (2000)

**2. Interrogative detection from lexical/syntactic cues. [EST]**
Margolis & Ostendorf 2011 is the reference point for question detection from *text alone*: wh-word
presence, sentence-initial auxiliary inversion, question mark, and first-token n-grams, with prosody
adding most of its value only for *declarative* questions (statement syntax, question intonation).
This is the standard baseline reused in later speech and clinical question-detection work.
<https://aclanthology.org/P11-2021/> · ACL 2011

**3. In slot-filling systems the act label is `(act_type, slot_argument)`. [EST]**
This is the direct answer to "how do systems exploit the shared relation phrase". In Rastogi et al.
2017 (scalable multi-domain DST, §3.2.2 "Slot related features") the act vector is
`a^t_s(s)` — *a binary vector denoting the presence of dialogue acts having slot `s` as the
argument*, e.g. `request(s)`, `deny(s)`, `inform(s)`. So the statement and the question differ in
**act_type** while sharing **the same slot argument**, and the architecture makes that explicit by
conditioning the act head on the relation id. In ATIS/Snips-style joint NLU the same idea appears as
a single intent label space (e.g. `inform_dentist` vs `ask_dentist`).
<https://ar5iv.labs.arxiv.org/html/1712.10224> · ASRU 2017
Goo et al. 2018, Slot-Gated joint slot filling + intent prediction: <https://aclanthology.org/N18-2118/>
Chen, Zhuo & Wang 2019, BERT for joint intent classification and slot filling: <https://arxiv.org/abs/1902.10909>
Coucke et al. 2018, Snips Voice Platform (embedded SLU): <https://arxiv.org/abs/1805.10190>

**4. Delexicalisation as the mechanism that removes the shared phrase. [EST]**
Rastogi et al. 2017 delexicalise **only the values**, deliberately keeping the slot names, because the
slot name is what identifies the slot. Applied to your case: replace "my dentist" with
`delex(dentist)` before the act decision. The statement and the question then differ only in the
act-bearing residue — copula + filled value vs. wh-word/auxiliary inversion + **empty value slot**.

**5. The cheapest reliable signal is "is the value slot filled?" [EST]**
DST-Reader (Gao et al. 2019) decomposes slot tracking into three sequential decisions: (i) binary
**slot carryover**, (ii) slot **type** ∈ {Yes, No, DontCare, Span}, (iii) **span** start/end inside the
dialogue. An *empty span prediction* is exactly a query signal, and it is far cheaper and more
reliable than reasoning over the relation phrase. Their per-slot span accuracy reached ~96%, while the
carryover head sat at ~72% — i.e. **value-slot detection is the easy part and you should lean on it.**
<https://ar5iv.labs.arxiv.org/html/1908.01946> · Gao et al. 2019

**6. Explicit out-of-scope rejection. [EST]**
Your `unresolved` bucket is functionally an OOS class. Larson et al. 2019 (CLINC150, 150 intents, 10
domains) is the standard framing and its headline finding is a caution: classifiers "perform well on
in-scope intent classification, they **struggle to identify out-of-scope queries**". Standard practice
is therefore a **margin/threshold rejection rule** on a calibrated score, not a learned third class.
<https://aclanthology.org/D19-1131/> · EMNLP-IJCNLP 2019

**7. LLM prompting. [REC]** Best raw accuracy on paraphrases, but requires float matmul at serving
time and is non-deterministic — excluded by your constraint.

### How much does adding synthetic question paraphrases help?

Three results, and they point in different directions:

- **Helps a lot in the extreme low-data / new-class regime. [EST]** Jolly et al. 2020 (COLING industry
  track) generate paraphrases of *seed* utterances to **bootstrap a new intent** in a production
  task-oriented dialogue system, and report improvements for **both intent classification and slot
  labeling** on a public dataset and a real-world system.
  <https://aclanthology.org/2020.coling-industry.2/>
- **Helps most when data is scarce, for non-pretrained models. [EST]** EDA (Wei & Zou 2019):
  "EDA demonstrates particularly strong results for smaller datasets; on average, across five
  datasets, training with EDA while using only **50% of the available training set** achieved the same
  accuracy as normal training with all available data." <https://arxiv.org/abs/1901.11196>
- **Documented negative result for pretrained transformers. [EST]** Longpre et al. 2020, "How
  Effective is Task-Agnostic Data Augmentation for Pretrained Transformers?" — across 5 tasks, 6
  datasets, 3 pretrained transformers, EDA and back-translation "**fail to consistently improve
  performance for pretrained transformers, even when training data is limited**."
  <https://aclanthology.org/2020.findings-emnlp.394/>

**Reading for your system:** if your compiler is small and non-pretrained (as a geometric
integer/table-lookup compiler is), expect real gains from synthetic question forms, concentrated in
the low-data regime — EDA's "half the data, same accuracy" is the shape to expect. If you already have
a well-covered in-domain set, expect ≈0 and possible regression from label noise. Generate the
paraphrases **deterministically from your own stored fact triples** (template inversion of
`My X is Y` → `What is my X?`), not from a generative model: labels are then exactly correct and the
data is integer-representable.

### (b) Tradeoffs

| Approach | Pros | Cons |
|---|---|---|
| Keyword/rule interrogative detector | Transparent, integer-only, no training, deterministic | Fails on declarative questions; false-fires on embedded questions ("I wonder who my dentist is") |
| Small classifier on act-residue features | Robust, tolerates paraphrase, tiny feature budget | Needs labels + a calibrated rejection threshold; threshold is the real failure mode |
| Relation-conditional act head `(act_type, relation)` | Directly removes the shared-phrase confound; matches ATIS/DST practice | Requires the relation tagger to be right first — errors propagate |
| Value-slot-filled consistency rule | Nearly free; converts relation-recognised-but-no-value into a query | Depends on a value-slot head existing |
| LLM prompting | Best paraphrase robustness | Excluded (float matmul, non-deterministic) |

### (c) Most appropriate for a small laptop-scale, integer/table-lookup system

A **three-stage cascade**, all stages integer-representable:

1. **Deterministic interrogative rules (free):** question mark; sentence-initial wh-word
   (who/what/where/when/which/whose/why/how); sentence-initial auxiliary
   (is/are/do/does/did/can/could/will/would/have/has).
2. **Structural consistency rule (the high-yield one):** if the compiler decoded a relation but the
   **value slot is empty**, force `act = query` and attempt the read. This targets your exact 60%
   symptom without new training.
3. **Relation-conditional act classifier** over *delexicalised act residue*, with a **margin threshold**
   producing `unresolved` as a rejection (CLINC-style), not as a class.
4. **Deterministic template paraphrases** of your own stored facts as extra question-form training data.

### (d) Citations

- Stolcke et al. 2000 — <https://arxiv.org/abs/cs/0006023>
- Margolis & Ostendorf 2011 — <https://aclanthology.org/P11-2021/>
- Rastogi, Hakkani-Tür & Heck 2017 — <https://ar5iv.labs.arxiv.org/html/1712.10224>
- Goo et al. 2018 — <https://aclanthology.org/N18-2118/>
- Chen, Zhuo & Wang 2019 — <https://arxiv.org/abs/1902.10909>
- Coucke et al. 2018 — <https://arxiv.org/abs/1805.10190>
- Gao et al. 2019 (DST-Reader) — <https://ar5iv.labs.arxiv.org/html/1908.01946>
- Larson et al. 2019 (CLINC150 / OOS) — <https://aclanthology.org/D19-1131/>
- Wei & Zou 2019 (EDA) — <https://arxiv.org/abs/1901.11196>
- Jolly et al. 2020 (paraphrase bootstrapping) — <https://aclanthology.org/2020.coling-industry.2/>
- Longpre et al. 2020 (augmentation negative result) — <https://aclanthology.org/2020.findings-emnlp.394/>

---

## DEFECT 2 — the distractor's relation wins the read (14 of 24 errors)

Symptom: two statements in one context mention different relations; the question resolves to the
distractor's relation.

### (a) Established approaches

**1. Understand first that soft attention over keys is *structurally* distractor-fragile. [REC but
well-grounded]**
End-to-end Memory Networks (Sukhbaatar et al. 2015) and Key-Value Memory Networks (Miller et al. 2016)
retrieve with a **softmax over key similarity**. A distractor whose surface form overlaps the query
necessarily receives softmax mass; this is dilution, not a bug. KV-MemNN's own remedy inside the
family is to encode **keys and values with separate feature maps** and to read the value addressed by
the key — which is exactly the design your exact store already has.
<https://ar5iv.labs.arxiv.org/html/1503.08895> · NeurIPS 2015
<https://aclanthology.org/D16-1147/> · EMNLP 2016

**2. The failure mode now has a quantitative theory: "context poisoning" as extreme-value attention
interference. [REC]**
Ghaffari et al. 2026 derive that under softmax retrieval the decisive-evidence score is **upper
bounded** while the maximum distractor score **grows with the number of effective distractors `N`**,
so holding a fixed accuracy target above base rate requires the evidence margin to scale as
**Ω(√log N)**. Their controlled experiments find that **the same-format distractor condition produces
the largest accuracy drop at fixed context length** — precisely your case (two statements of identical
form, different relation). Their recommended remedies: **evidence bottlenecks, alias-resistant
representations, retrieve-then-reason architectures, verifier-mediated memory**.
<https://arxiv.org/abs/2609.22101> (Aug 2026, preprint)

**3. Make carryover-vs-update an explicit per-slot decision. [EST — and the strongest number here]**
DST-Reader (Gao et al. 2019) introduces the **slot carryover** module: a binary decision whether a
slot's previous value is retained or replaced. Their ablation on MultiWOZ-2.0: joint goal accuracy
41.10% for the full system, **60.18% with an oracle carryover model (+19.08 points)** — while the
learned carryover model itself achieved only **~72%** per turn. Two conclusions transfer directly:
the "should this slot change at all?" decision is the **dominant error term** in slot tracking; and an
*unrelated* turn should produce `carryover`, i.e. **write nothing**.
<https://ar5iv.labs.arxiv.org/html/1908.01946>

**4. Restrict the retrieval to a bounded per-slot candidate set. [EST]**
Rastogi et al. 2017 build a candidate set per slot per turn from (i) values in the current user
utterance, (ii) values in the system utterance, (iii) previous-turn candidates by descending score,
capped at `|C| ≤ K` with `K = 7` (which "rarely" saturated in their experiments), plus explicit
`NULL` (not yet specified) and `DONTCARE` values. Scoring is confined to that candidate set. Effect
for you: the distractor relation's value **is not even a candidate** for the queried slot.
<https://ar5iv.labs.arxiv.org/html/1712.10224> · ASRU 2017

**5. Treat the state as a selectively-overwritten key-value memory with a discrete operation set.
[EST]**
SOM-DST (Kim et al. 2020, NAVER Clova) defines the dialogue state as a *fixed-size key-value memory*
and predicts one of four operations per slot per turn:
`carryover | delete | dontcare | update`, then generates values **only** for the slots predicted
`update`. "The previous dialogue state is directly utilized like a memory." Results: **51.72%** joint
goal accuracy on MultiWOZ 2.0 and **53.01%** on MultiWOZ 2.1 in the open-vocabulary setting (state of
the art at publication, averaged over ten runs). Their error analysis concludes that **improving state
operation prediction is the highest-leverage direction** — matching the DST-Reader ablation.
<https://ar5iv.labs.arxiv.org/html/1911.03906> · ACL 2020

**6. Hard / exact-match addressing instead of soft addressing. [REC]**
Latest Exact Match Attention (LEMA, Brösamle 2026) binarises queries and keys and has each query attend
**only to the latest exactly matching key**, with dictionary-based inference whose dictionaries live in
main memory. It proves LEMA transformers with chain of thought can simulate word-RAMs, and gives the
converse simulation at cost per token **independent of context length**. On a synthetic associative
recall task LEMA beats gated DeltaNet; at scale it matches softmax transformers of roughly half its
size in loss but stays behind on needle retrieval. This is the theoretical licence for your design:
**exact-match addressing is not a limitation to apologise for, it is a design that removes the
distractor-interference term entirely.**
<https://arxiv.org/abs/2609.25802> (Sep 2026, preprint)

**7. Multi-hop / collision handling. [REC]** Boesch & Wee 2026 report that fixed-state recurrence cells
which solve 32-pair recall nonetheless "**fall to chance retrieving 4 pairs from a distractor
haystack**", flat across sequence lengths; and that "**collision-key retrieval needs two layers**".
Their diagnosis is interference under sparse supervision, not capacity, and a distance curriculum
takes the same architecture from 0.021 to 1.000 recall. Relevant if you later make keys collide.
<https://arxiv.org/abs/2609.16183> (Sep 2026, preprint; notes itself as preliminary)

### (b) Tradeoffs

| Approach | Pros | Cons |
|---|---|---|
| Exact/argmax key addressing (relation id as key) | Zero distractor interference; integer/table-lookup; deterministic; auditable | All error moves into relation tagging; no graceful degradation on a wrong tag |
| Bounded candidate set per key per turn (Rastogi) | Cheap; removes distractor values from the scoring space | Needs a candidate generator; small `K` can flush a value |
| Explicit `carryover`/`update` op per key (SOM-DST) | Makes "unrelated turn ⇒ no write" the default; largest measured gain in DST | Requires a reliable op classifier — which is itself the hard part (72% in DST-Reader) |
| Soft attention over keys | Tolerant of tag errors; single mechanism | Provably dilutes with same-format distractors; Ω(√log N) margin requirement |
| Verifier / evidence bottleneck after retrieval | Catches residual errors | Adds latency for the distractor case |

### (c) Most appropriate for a small laptop-scale, integer/table-lookup system

**Make the address the relation, not the surface string — you already have the tuple.**

1. **Key = `relation_id` (+ entity id), never a surface phrase.** Your compiler already emits a
   relation. Once the key is a tagged relation, "the distractor's value" is stored under a *different
   key* and is structurally unreachable. Exact-match addressing (LEMA-style) makes this a table
   lookup — no float, no matmul, constant time in context length.
2. **Per-key candidate restriction** as a second line of defence, with explicit `ABSENT` / `DONTCARE`
   values (Rastogi).
3. **Per-key operation gate:** only turns whose act is value-bearing (`inform`-like) may write;
   everything else yields `carryover`. This is the DST-Reader/SOM-DST result and it makes the
   distractor turn a no-op by construction.
4. **Keep a packed-integer consistency check** (does the read key equal the query key?) as the
   integer-only analogue of a verifier gate.

### (d) Citations

- Sukhbaatar et al. 2015, End-To-End Memory Networks — <https://arxiv.org/abs/1503.08895>
- Miller et al. 2016, Key-Value Memory Networks for Directly Reading Documents — <https://aclanthology.org/D16-1147/>
- Ghaffari et al. 2026, Context Poisoning as Extreme-Value Attention Interference — <https://arxiv.org/abs/2609.22101>
- Gao et al. 2019, DST as a Neural Reading Comprehension Approach (slot carryover) — <https://ar5iv.labs.arxiv.org/html/1908.01946>
- Rastogi et al. 2017, Scalable Multi-Domain Dialogue State Tracking (candidate sets) — <https://ar5iv.labs.arxiv.org/html/1712.10224>
- Kim et al. 2020, Efficient DST by Selectively Overwriting Memory (SOM-DST) — <https://ar5iv.labs.arxiv.org/html/1911.03906>
- Brösamle 2026, Latest Exact Match Attention — <https://arxiv.org/abs/2609.25802>
- Boesch & Wee 2026, Anatomy of Associative Recall in Fixed-State Recurrences — <https://arxiv.org/abs/2609.16183>
- Xie et al. 2024, Adaptive Chameleon or Stubborn Sloth (knowledge conflicts) — <https://arxiv.org/abs/2305.13300>

---

## DEFECT 3 — write order / which value is current (7 of 17 succeeded, 10 failed)

Symptom: fact and distractor resolve to the **same key**; the read returns whichever was written,
sometimes the distractor's value. Also: an *unrelated later* statement can evict a relevant earlier one.

### (a) Established approaches

**1. Last-write-wins (LWW) register. [EST]**
The canonical semantics: the current value is the write with the greatest timestamp, ties broken
deterministically (e.g. by replica id). Formally a CRDT — Shapiro, Preguiça, Baquero & Zawirski 2011,
"Conflict-free Replicated Data Types", SSS 2011.
<https://hal.sorbonne-universite.fr/hal-00932836v1> · DOI 10.1007/978-3-642-24550-3_29
Operationally identical to **Kafka log compaction**, which retains only the latest record per key and
uses a null *tombstone* for deletion.
<https://www.conduktor.io/kafka/kafka-topic-configuration-log-compaction>
LWW is *correct* for "which value is current" **iff** writes are ordered and the latest write is the
one you want. It is *wrong* exactly when an unrelated later write touches the same key — your failure.

**2. Append-only event log + replay (event sourcing). [EST]**
Every write is an immutable event; "current state" is a fold over the log. This is what makes any
as-of query and any audit possible. But it does **not** by itself define currency: the fold is still
LWW unless you specify something better, and the log grows without bound.

**3. Temporal and bitemporal databases; "as-of" queries. [EST]**
Snodgrass & Ahn 1986, "Temporal Databases", *IEEE Computer* 19(9):35–42, DOI 10.1109/MC.1986.1663327;
Jensen & Snodgrass 1999, "Temporal Data Management", *IEEE TKDE* 11(1), DOI 10.1109/69.755613.
The standard model has two independent time axes:

- **valid time** — when the fact was true in the modelled world;
- **transaction time** — when the database recorded it.

SQL:2011 standardised both (`application-time period tables`, `system-versioned tables`), and the
read is literally `SELECT ... FOR SYSTEM_TIME AS OF <t>`. This is the established, correct semantics
for "which value is current", and it is the only approach here that cleanly separates *"the world
changed"* from *"the system learned later"* — the distinction your fact/distractor pair keeps
conflating.

**4. Belief-state tracking (POMDP). [EST]**
Williams & Young 2007 model the dialogue state as a **probability distribution** over slot values
maintained by Bayesian update; "current" is the argmax of belief, and superseded values decay rather
than being deleted. DST's `NULL` / `DONTCARE` special values are the practical residue of this.
Graceful under uncertainty, but floats at serving time — excluded by your constraint.
<https://dlnext.acm.org/doi/10.1016/j.csl.2006.06.008> · Computer Speech & Language 21(2), 2007

**5. Revocation / supersession as an explicit memory lifecycle. [REC — and the key negative result]**
TEPA (Zhou et al. 2026) represents observations as **keyed precedents** and **revokes** the active
precedent when fresh evidence contradicts it under the same key, keeping the revoked record for audit
so retrieval draws only from current evidence. Their controlled-drift experiment:

| Memory policy | Score under full reversal |
|---|---|
| Append-only | **0.210** |
| Last-write-wins | **0.210** |
| **No memory at all** | **0.309** |
| TEPA (revocation) | **0.950** |

Real file-backed execution reproduced it (append-only 0.203, no memory 0.298, TEPA 0.950). On a clean
MemoryAgentBench SH-6k setting TEPA merely *matched* a strong LWW cache — "confirming that current-key
replacement is the decisive operation for single-hop fact consolidation."
<https://arxiv.org/abs/2608.07429> (Aug 2026, preprint)

**6. Supersession as a *measurement* problem. [REC]**
Supersede (Patel 2026) isolates the ability to answer from the current value and discard superseded
ones, on the knowledge-update subset of LongMemEval:
replacing an agent's full context with a bounded self-maintained memory dropped accuracy **92% → 77%**
even on a frontier model (gpt-5.4), paired McNemar **p < 0.005**; as the conversation grew **24×**
accuracy fell **68% → 28%**; and granting proportionally more memory produced **no detectable recovery
(28% → 28%, n = 25)**. Their conclusion: "the bottleneck is therefore **memory maintenance, not
comprehension**". They then show it is partly *trainable*: GRPO on Qwen2.5-3B nearly doubled held-out
supersession accuracy (9.0% → 16.7%, single run).
<https://arxiv.org/abs/2606.27472> (Jun 2026, preprint)

**7. Agent-memory system designs, for context. [REC]**
Generative Agents (Park et al. 2023) score retrieval by recency + importance + relevance — i.e. an
explicit recency term is standard practice, but a recency *score* is not the same as LWW semantics.
Zep/Graphiti (Rasmussen et al. 2025) maintains a **temporal knowledge graph** with `valid_at` /
`invalid_at` edge intervals and invalidates contradicted edges. MemGPT (Packer et al. 2023) manages a
tiered memory hierarchy. A 2026 survey (Du, "Memory for Autonomous LLM Agents") formalises agent
memory as a **write–manage–read loop** and names **contradiction handling** and **write-path
filtering** as explicit engineering concerns.
<https://dl.acm.org/doi/fullHtml/10.1145/3586183.3606763> · UIST 2023
<https://ar5iv.labs.arxiv.org/html/2501.13956> · Zep
<https://arxiv.org/abs/2603.07670> · survey, Mar 2026

### (b) Tradeoffs

| Semantics | "Current" means | Pros | Cons |
|---|---|---|---|
| LWW register | max timestamp per key | O(1) write/read; trivially integer; standard in KV stores | Loses history; **an unrelated later write destroys a relevant earlier one**; no audit |
| Append-only log | fold over log | Full history, audit, replay, any as-of query | Unbounded growth; the fold is still LWW unless specified; **worst measured option under reversal** |
| Bitemporal (valid + transaction time) | value valid at query time `t` | Correctly separates world-change from learning-time; SQL:2011-backed | 2× bookkeeping; every read needs a time argument; most machinery |
| Belief state (POMDP) | argmax of belief | Graceful under uncertainty; no destructive overwrite | Floats (excluded at serving); needs a generative model of state change |
| Keyed revocation (TEPA) | latest *non-revoked* precedent for the key | Best measured reversal behaviour; keeps audit trail | Newer, unverified, single-paper; needs a contradiction check on write |

### (c) Most appropriate for a small laptop-scale, integer/table-lookup system

**Versioned key-value with an explicit write gate, plus a separate append-only log.**

1. **Key = (relation_id, entity_id); value record = (value, turn_index, source_act, revoked_flag).**
   Read = the record with the **maximum `turn_index` among non-revoked records for that exact key**.
   All integer, all table lookup, deterministic.
2. **Write gate:** only turns whose act is value-bearing (`inform`-like) *and* whose value slot is
   filled may write. The unrelated later statement then **cannot evict** the relevant earlier one —
   this is the single change that addresses "an unrelated later statement evicting a relevant earlier
   one" without any temporal machinery.
3. **Revoke rather than delete on genuine contradiction** (same key, later value-bearing turn, different
   value): set `revoked_flag` on the earlier record, keep it. TEPA's numbers are the argument for this
   over plain LWW, and it costs one flag bit.
4. **Keep the append-only event log separately** (turn index is the timestamp) for audit and as-of
   replay. Do **not** make the log the read path — TEPA measured append-only as the *worst* policy
   under reversal.
5. **Skip bitemporal and POMDP** at this scale: bitemporal's valid-time axis only pays off when you
   must distinguish "was true then" from "recorded then", and POMDP needs floats.

### Is any of this a known trap with a documented negative result? — Yes.

**Three explicit traps:**

- **"A bigger memory will fix supersession." Documented negative. [REC]** Supersede: accuracy fell
  68% → 28% as the conversation grew 24×, and granting proportionally more memory gave **no detectable
  recovery (28% → 28%, n = 25)**. The failure scales with conversation length, not compression ratio.
  <https://arxiv.org/abs/2606.27472>
- **"Append-only is the safe choice." Documented negative. [REC]** TEPA: append-only (0.210) and
  last-write-wins (0.210) both scored **below no memory at all (0.309)** under full reversal.
  Append-only preserves the stale value *and* makes it retrievable.
  <https://arxiv.org/abs/2608.07429>
- **"Soft attention will learn to ignore the distractor." Documented negative. [REC]** Context
  poisoning: the distractor's max score grows with distractor count while evidence is upper-bounded,
  requiring a margin of Ω(√log N); **same-format distractors caused the largest drop**.
  <https://arxiv.org/abs/2609.22101>

**And one trap on the Defect-1 side:** **"synthetic paraphrases always help." Documented negative for
pretrained models. [EST]** Longpre et al. 2020 found EDA and back-translation "fail to consistently
improve performance for pretrained transformers, even when training data is limited."
<https://aclanthology.org/2020.findings-emnlp.394/>

**A caution about verification itself [REC]:** the deterministic-retrieval-chain report
(Chanhnourack 2026) documents a negative control in which a verifier "repaired three wrong drafts but
broke eleven correct drafts" — i.e. a post-hoc verifier can be net-negative. Any verifier gate you add
should be measured as a delta on both repaired and broken cases.
<https://arxiv.org/abs/2609.38021>

### (d) Citations

- Shapiro, Preguiça, Baquero & Zawirski 2011, Conflict-free Replicated Data Types — <https://hal.sorbonne-universite.fr/hal-00932836v1>
- Snodgrass & Ahn 1986, Temporal Databases, *IEEE Computer* 19(9):35–42 — DOI 10.1109/MC.1986.1663327
- Jensen & Snodgrass 1999, Temporal Data Management, *IEEE TKDE* 11(1) — DOI 10.1109/69.755613
- Williams & Young 2007, POMDPs for spoken dialog systems — <https://dlnext.acm.org/doi/10.1016/j.csl.2006.06.008>
- Zhou et al. 2026, TEPA: Revoking Stale Memories for Conflict-Robust Language Agents — <https://arxiv.org/abs/2608.07429>
- Patel 2026, Supersede: Diagnosing and Training the Memory-Update Gap in LLM Agents — <https://arxiv.org/abs/2606.27472>
- Park et al. 2023, Generative Agents — <https://dl.acm.org/doi/fullHtml/10.1145/3586183.3606763>
- Rasmussen et al. 2025, Zep: A Temporal Knowledge Graph Architecture for Agent Memory — <https://ar5iv.labs.arxiv.org/html/2501.13956>
- Du 2026, Memory for Autonomous LLM Agents (survey) — <https://arxiv.org/abs/2603.07670>
- Chanhnourack 2026, Auditable Long-Term Memory (deterministic retrieval chain) — <https://arxiv.org/abs/2609.38021>
- Kafka log compaction (LWW/tombstone in practice) — <https://www.conduktor.io/kafka/kafka-topic-configuration-log-compaction>

---

## Cross-cutting summary

| Defect | Single highest-leverage change | Basis |
|---|---|---|
| 1 — queries not recognised | Force `act = query` when a relation is decoded but the **value slot is empty**; add deterministic template question forms | DST-Reader slot-type/span decomposition [EST]; EDA low-data regime [EST] |
| 2 — distractor relation wins | Key on the **tagged relation id**, not the surface phrase; write only on value-bearing acts | KV-MemNN key/value separation [EST]; SOM-DST carryover op [EST]; LEMA exact-match [REC] |
| 3 — wrong value is "current" | `(relation_id, entity_id)` key + `turn_index` + **write gate** + revocation flag, with a separate append-only audit log | TEPA revocation (0.950 vs 0.210 LWW vs 0.309 no memory) [REC]; CRDT LWW-Register [EST]; bitemporal as-of queries [EST] |

**All three recommended fixes reduce to one theme:** the defects are **addressing and write-policy**
problems, not similarity or capacity problems. Because your compiler already emits `(relation, act)`,
you can make relation the address, make act the write permission, and make turn index the currency
order — a fully integer, deterministic, table-lookup design with no float matmul, and one that
leaves an auditable log behind.

**Grading recap.** The load-bearing recommendations rest on **[EST]** results (SRT/SWBD-DA tagging,
DST-Reader, SOM-DST, Rastogi candidate sets, KV-MemNN, CRDT LWW-Register, SQL:2011 temporal tables,
the EDA and augmentation-negative-result literature). The 2026 preprints (context poisoning, TEPA,
Supersede, LEMA, interference wall) supply the *quantitative sizing* of the failure modes and the
strongest negative results, but each is a single unreplicated paper and is labelled **[REC]** above.
