# Retrieval by an exact conversation log and a prime sieve: design memo

*1 October 2026. Claude lab. Required by [D18](DECISIONS.md#d18--one-retrieval-question-for-114-october-track-b-cost-and-memory-port-work-parked-with-re-entry-conditions) outcome D (§3, §10), which the owner confirmed on 1 October. Nothing in this memo was run for it. Each statement is tagged **measured** (with its source), **derived** or **design**. This is a proposal for the owner's decision, not a result.*

## 1. What the evidence requires

**Soft reads do not bind a query to its key at this scale (measured, #1552).**
- In A1, no read retrieved without a pointer head (MQAR 1/109): Lorentz, trained flock, and the transformer control at 5,180 steps.
- The pointer arms scored 0.28–0.44, falling as about 1/N (0.34–0.64, 0.17–0.25 and 0.08–0.17 at N = 2, 4, 8). That matches the untrained R-recency rule (0.36; 0.49, 0.28, 0.11). They learned to **copy the latest stated value**.
- Their misses are wrong-key answers: another pair's value.

**Exact stores work; key formation from language does not (measured).**
- D2's exact store scored 1.000 on training templates and 0.000–0.015 on held-out phrasing. Its read trigger fired 0 of 826 times there ([`d2-aerm-probe-result-2026-09-28.md`](d2-aerm-probe-result-2026-09-28.md)).
- In G v2 (#1505), held-out key accuracy was 0.327 (2I) and 0.491 (softmax). Decoding the **entity** reached 0.963/0.994, but decoding the **relation** only 0.450/0.491.
- So binding fails on the relation word under paraphrase, not on the entity.

**The prime router never had its front end (measured).**
- #958 ended `RETAIN_STORAGE_RECALL_ONLY`. Stage 4 was NOT_RUN for lack of "a manifest-bound compiler from arbitrary prompt tokens into those addresses" and of a payload decoder ([`prime_route_attention_qualification_958.md`](../prime_route_attention_qualification_958.md)).
- The `stack_prime_route` "sieve" (#1487) is a pairwise membership scan; no gcd sieve has been built or measured.
- Its held-out Updated scores were 0.27/0.44/0.33 against a gold-register ceiling of 0.81/0.84/0.68, on one shared evaluation draw.

**The §8 panel exceeds the window (measured, owner-run count).** It needs up to 317 positions with a 32-token reply budget, against the stack's 256.

**What the M-world instrument asks for (measured: source, `milestone_world_v2.rs`):**
- **MQAR keys** are generated words repeated **verbatim** between the statement ("{k} is {v}") and the query, in both phrasing splits. Only function words change.
- **Relation queries in development phrasing change the content words.** For example, the assert is "My buddy is {v}" and the query is "Remind me who my friend is".
- **The query wants the latest value of the asked relation.** A companion relation stated in between is a distractor, and an unasked relation must be abstained on.
- **A reply such as "It's {v}." passes MQAR and relation checks.** The checks require the value (`AnyOf`) and forbid stale or other values (`NoneOf`).

**Design consequence.** Binding has two different parts:
- **Identity binding**, for a word repeated verbatim (MQAR keys, names, codes). It can be exact.
- **Relation binding**, for the same relation in different words. It must be learned, because a prime or hash identity is not a semantic distance (AGENTS.md).

The design keeps the two in separate channels, so the exact channel's correctness never depends on the learned one.

## 2. The design (design)

Three parts. Only the third involves the trained stack.

### 2.1 The exact log

- **Contents.** An append-only record of every user turn, kept as **word atoms**: normalized words with their byte spans, turn index, clause index and an occurrence id. Assistant replies are logged too, marked as the model's own (never as user facts).
- **Words, not BPE tokens.** The log keys on words. Under the #1017 tokenizer, a word's first token differs after a space (`Ġm`=283) and at a reply start (`m`=79) (measured, #1552). Word identity removes that.
- **Reuse.**
  - `canonical_lexical_ingestion.rs` already builds an exact, invertible transcript: turn/sentence occurrences, spans, unit ids, payload CIDs and primes (`reconstruct_conversation`). Its caps (8 turns, 512 units, 256-word vocabulary) must be raised, and its unknown-word rejection replaced by registration on first sight.
  - `scoped_memory.rs` supplies version semantics and abstention outcomes, and keeps an unfollowed link from being treated as absence.
  - `relation.rs` and `historical_read.rs` supply ring and eviction proofs: a record is "evicted" only when its slot was overwritten.
- **Bound.** A declared cap, for example 4,096 words, with oldest-first eviction recorded per record. Under the cap, no fact is lost when the attention window slides. That is what makes a 256- or 384-position window plus eviction compatible with §8's 317 positions (derived).

### 2.2 The sieve: identity channel (exact)

- **Primes.** Each word atom gets a prime on first sight, from a registry table (`stack_prime_route` already maps ids to primes). A clause's address is its squarefree set of content-word primes plus their **ordered** pairs (n-lets), which keep direction: "Alex's friend is Sam" ≠ "Sam's friend is Alex" (#1487 keeps direction by routing the value through the key).
- **Candidate admission (exact).** A clause is admitted if it shares at least one query content prime. On squarefree products this is exactly `gcd(query, clause) > 1`; it can be computed as a sorted merge of factor lists, so no product is formed.
- **Ranking (exact, separate from admission).**
  1. More shared query primes first.
  2. Then the latest occurrence, so the latest version wins.
  3. Then the ordered-pair match ("{k} is" before "is {k}").
- **Value extraction.** The words after the matched key up to the clause end. For relation statements, the value is the clause word that the learned channel marks as the slot (§2.3).
- **Which words count as content.** In M-world, template words are known (`reserved_words()`). In general English the cut is learned from frequency: a stop list bound to the tokenizer's corpus statistics, recorded in the artifact.
- **Serving (design).**
  - Primes come from a table, and admission is a sorted merge. A binary gcd, if one is needed, uses only shift, subtract and compare.
  - None of this needs a multiplier. `uor-r4-integer` has no gcd today (measured).
  - The D11 session needs **no new hook**. The retrieved fact enters as ordinary tokens (§2.4), stepped through the existing `step(token)`.

### 2.3 The relation channel (learned): the missing compiler

- **The compiler.** A small learned head reads the stack's state at the end of a user turn. It outputs:
  - an intent: assert, update, query or none;
  - a relation atom from a closed table of the relations in use (M-world: 10; §8: the relation types it uses), or "other";
  - the slot word's position (for an assert or update).
- The relation atom becomes a prime. A record's key is the canonical pair of entity atom and relation atom (as in #1487). The user is the default entity.
- **This is the open problem.** G v2 decoded held-out relations at 0.45–0.49 (measured).
- **Differences from G v2 (design; untested):**
  - The head reads the full stack state after the turn, not a single tagged token.
  - It is trained on M-world train phrasings plus chat-v0 paraphrase pairs. Development phrasings stay held out.
  - Its output is a small closed classification, not a key embedding.
- **Gate.** If it cannot reach ≥ 0.9 held-out relation-atom accuracy, relation binding is not solved by this route at this scale. That result is recorded (D12) and the next step returns to the owner.

### 2.4 Emission: put the found value where the pointer already looks

- **The found value goes into the window as the latest stated value.** One recall line (protocol role `system`, e.g. "Memory: your friend is Zelpur.") is inserted just before the assistant marker. With no record, it reads "Memory: none."
- **Why this should work (derived from measured behaviour, untested):**
  - The A1 pointer arms reliably copy the most recently stated value. With a single injected value, N = 1, so recency copying is the correct behaviour.
  - Putting the value after a space ("It's {v}") avoids the copy-boundary failure that made copy 0/33 (measured).
  - The reply is still generated by the model. No serving response is authored by a teacher or template (AGENTS.md).
- **Abstention** follows the sieve's outcome: Found, Absent, NoHistory or Evicted. "Absent" is reported only when the log provably holds no record, never because a candidate failed ranking (AGENTS.md: a rejected link is not proof of absence).

## 3. Experiments, in order (design; none run)

**E1 and E2 need no training**, so they fit outcome D's rule. **E3 and E4 are new training** and need the owner's approval of this memo.

| | Question | Method | Gate | Cost |
|---|---|---|---|---|
| **E1** | Does exact identity binding solve the verbatim cases? | An untrained `R-sieve` rule in `m-world baselines`, using §2.1–2.2 on M-world word atoms. On the four cells: MQAR by distance and N; relations by asked/companion/updated | Development-cell MQAR ≥ 0.9 at d16, d64 and d200, at every N | CPU seconds; source only in `milestone_world_v2.rs` and `m-world.rs` |
| **E2** | Does the trained pointer copy one injected value? | `m-world evaluate … recall=oracle` on existing weights (`arm-L1b-1`, `arm-C-2`, `arm-C-s2`). The frozen oracle's value is injected as the recall line. Also `recall=none` | Development-cell MQAR and open relation ≥ 0.9 with the oracle line; abstention ≥ 0.9 with "none" | Evaluation only, about 5 min per model |
| **E3** | Can a learned head bind relations under paraphrase? | The §2.3 compiler on a frozen trunk (A1 weights), trained on train phrasings | Development-phrasing relation-atom accuracy ≥ 0.9; intent ≥ 0.95 | Training; owner approval |
| **E4** | End to end | E1 + E3 + E2's emission, `evaluate-cells` | `a1_gate` on the development cell | Evaluation |

**Decision table:**
- **E1 passes and E2 passes:** MQAR is solved by construction plus copying. Relations hinge on E3.
- **E2 fails:** emission needs training with recall lines in the curriculum. That is a scoped training item for the owner.
- **E3 fails at ≥ 0.9:** recorded as a negative. The relation channel falls back to identity-only binding, which covers verbatim relation words and fails paraphrase. Paraphrase-robust relation binding at this scale is then an open owner decision: teacher-labelled data, a larger trunk, or accepting the limit for §8.

## 4. What this does not claim

- **No measurement in this memo is new.** Every number is cited.
- No chat, geometric advantage, retrieval capability, runtime saving or energy saving is claimed.
- **E1 would test the instrument, not the model.** A rule passing E1 shows that M-world's MQAR keys repeat verbatim; it says nothing about English.
- **§8 is English.** Its names and values are real words, and its relations are worded freely. The identity channel covers repeated names and values. The relation channel is the part §8 actually tests.
- **The runtime-cost thesis is untouched.** It still needs D5 selected weight access (DeepSeek's memo, D18 §9).

## 5. Inputs reused (measured: source)

- `canonical_lexical_ingestion.rs`: exact transcript and inverse.
- `scoped_memory.rs`: versions and abstention outcomes.
- `relation.rs` and `historical_read.rs`: ring and eviction proofs.
- `stack_prime_route.rs`: id → prime, canonical pair keys, direction through routing.
- `stack_aerm.rs` `RelationStore`: exact pair store with an eviction status.
- `uor-r4-integer` kernels: table multiply, shift-subtract division. A binary gcd is not yet implemented.
- M-world v2.1 and its frozen oracle, unchanged.
