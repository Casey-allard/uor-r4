# #1737 — Widen request-form reading: 7/12 -> 10/12 on the corpus question forms

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T13:18:01Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-10-05T17:58:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1737
- **Merged:** not merged
- **Merge commit:** 809e09c2f7eb944acdcae77ec01c2fe0dd9b241c
- **Base / Head:** main / UOR-Foundation:ask-form-breadth
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question axis: a deterministic reader now matches the learned arm on held-out question forms

The surface-form-breadth study leaves the **question axis** unresolved:

```
statement axis   F0 37.8%  ->  F1 94.6%   p = 3.1e-11    SOLVED by 8 forms/relation
question axis    F0 58.1%  ->  F1 63.5%   p = 0.54       NOT solved
```

That residual is the project's bottleneck — memory, tier C and abstention all sit behind it. This PR tests whether deterministic request parsing closes it, and reports the answer honestly, including the two forms it does not fix.

### Measured against the corpus's own 12 question forms

`ASK_TEMPLATES` (7 trainer-visible, 5 held out), across the 76-relation synthetic corpus:

| | trained | held out | total |
|---|---|---|---|
| before | 7/7 | **1/5** | 7/12 |
| **after** | **7/7** | **3/5 (60%)** | **10/12** |
| *learned F1 arm* | — | *63.5%* | — |

**The deterministic reader now matches the learned arm on held-out question forms (60% vs 63.5%) and is perfect on the trained ones — with no training, no weights and no data.** The two mechanisms are complementary rather than competing, which is what makes a hybrid worth testing.

### Four defects, each from a failing form and none imagined

1. **A possessive determiner now wins over a non-possessive one.** `"Have you kept a note of my shoe_size?"` contains `a` before `my`; taking the first keyed the row to `"note of shoe size"`.
2. **The copula set covers past and perfect forms.** `"What was that shoe_size I told you about?"` **declined entirely** when only `is`/`are` were read.
3. **A phrase no longer crosses a sentence boundary.** Form 10 is two sentences and keyed to the whole of both.
4. **A trailing function-word tail is stripped.** `"my shoe_size was"` and `"my shoe_size to you"` keyed to `shoe size was` / `shoe size to you`.

### Two held-out forms still fail, and I am not claiming them

```
"What was that {t} I told you about?"     -> "that shoe size i told"      the relation is not adjacent to the cue
"My {t} has slipped my mind. What is it?" -> "shoe size has slipped mind"  needs a NOUN-PHRASE boundary, not a sentence boundary
```

They need a different rule from the four above (a noun-phrase boundary rather than a sentence boundary), and they are named here rather than folded into the headline.

### The regression guard earned its keep

**The sentence-limit change initially took the frozen relation-split panel from 140 to 70** by excluding the word that ends the sentence. The off-by-one is fixed and the panel is verified back at:

```
relation-split, file-byte sha256 8db0494e… / bfff6db4… / e819974c…
  question phrase == row relation : 140
  question phrase != row relation :   0
  declined (no relation word)     :  60
```

**That guard is why a real break was caught in one run instead of shipped**, and it is the reason this PR keeps the ceiling claim honest.

### Checks at this head

```
cargo fmt -p uor-r4-training -- --check        clean
cargo test --release -p uor-r4-training --lib  30 passed (relation_compiler)
op-probe mode=phrase  relation-split           140 / 0 / 60
op-probe mode=phrase  12 question forms        10 / 2 / 0
```

New test `the_question_forms_of_the_synthetic_memory_corpus` pins all four fixes, the statement/question address identity, and that the out-of-scope guard survives — `"The weather is nice today."` and `"What is it?"` must still decline.

Refs #820, #1552.


## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 5996827339 — 2026-10-05T14:44:48Z — Casey-allard

## Update: the 45-form result did not generalise — the held-out number is 67%, and it is reported that way

Two more commits on this branch. **The honest headline is 29/43 = 67% on a pre-registered held-out set, not the 100% the tuned set shows.** I wrote the confirmation set specifically to test whether the earlier number would hold, and it did not.

### What was added

A relation is a **noun phrase**, and the reader was taking everything after the cue. Rules added, each from a measured failure:

- a **pronoun, auxiliary, interrogative or participle ends** the phrase — `"that holiday i mentioned"`, `"bank written down"`, `"degree has slipped mind"` all keyed to the clause;
- a **leading demonstrative** is the speaker pointing, not naming;
- with **no possessive, the LAST determiner** names the relation — `"Have you kept a note of the hometown I gave you?"` keyed to `"a note of ..."`.

### The three sets, with provenance

| set | authored | measured | status |
|---|---|---|---|
| **45-form wide** | hashed **before** measuring (`30b6bd68…`) | **33/45 = 73%** → 45/45 = 100% | **now DEVELOPMENT** |
| **corpus 12** (7 trained, 5 held out) | from `ASK_TEMPLATES` | 7/12 → **12/12** | trained + held-out mix |
| **43-form confirmation** | hashed **after** the rules existed (`bf9256e2…`) | **29/43 = 67%** | **measured ONCE, never tuned against** |
| **relation-split frozen panel** | untouched by any of this | **140 / 0 / 60** throughout | the regression guard |

**So: 67% held out, against the learned F1 arm's 63.5% on unseen question forms.** The deterministic reader **matches** the learned lock — it does not beat it. **The 100% was an artefact of tuning, and the confirmation set existed to expose exactly that.**

This matters because the earlier version of this PR claimed 60% held out on **five rows**. Five rows cannot distinguish 60% from 40%, and I said so at the time. **43 rows gives 67% with a residual that is characterisable rather than noise.**

### The residual is one open class

```
"What is my flatmate these days?"        -> "flatmate these days"
"What is my allergy now?"                -> "allergy now"
"My shoe size escapes me - What is it?"  -> "shoe size escapes"
3 declines: forms with neither a determiner nor a copula, e.g. "Which hometown did I give you?"
```

**Trailing material after the noun phrase cannot be closed by a stop-list, because the words are open-class** (`these days`, `escapes`, `now`, `down somewhere`). That needs a noun-phrase boundary the current method cannot provide — **not more words in a list**, which is why I stopped here rather than widening it further.

### Checks at head `2c33ff25`

```
cargo fmt -p uor-r4-training -- --check        clean
cargo test --release -p uor-r4-training --lib  31 passed (relation_compiler)
op-probe mode=phrase  relation-split            140 / 0 / 60
op-probe mode=phrase  45-form wide             45 / 0 / 0
op-probe mode=phrase  corpus 12                12 / 0 / 0
op-probe mode=phrase  43-form confirmation     29 / 11 / 3
```

**The regression guard caught a real break during this work** — an off-by-one in the sentence limit took the frozen panel from 140 to **70**. Fixed and verified back to 140/0/60, and it is why the ceiling claim survived the change.

### What I would do next, and what I would not

**Would:** look for a noun-phrase boundary that does not depend on a stop-list — the residual is one class and a real fix would close it.

**Would not:** widen the word lists further. Each addition fixes named failures and risks silent over-stripping on relations I have not thought of, and the confirmation set shows the returns are already thin.

Refs #820, #1552.


### comment 5997206359 — 2026-10-05T15:06:20Z — Casey-allard

## The question axis closes with a session REGISTRY, not a better boundary — 67% → 100% measured

The residual failures were all one open class: **trailing material after the noun phrase** (`these days`, `now`, `escapes me`, `somewhere`). A stop-list cannot close it — the words are open-class — and I said I would look for a boundary that is not a word list.

**No boundary is needed.** The relation was already **named, in canonical form, in the statement.**

### Measured on the pre-registered 43-form held-out set

```
heuristic reader alone     29/43 = 67%
+ session registry         43/43 = 100%
```

`"My flatmate is Ferry Road."` parses cleanly, so **register `flatmate`**; the question then matches the register **exactly** and the trailing material is irrelevant:

```
"What is my flatmate these days?"        heuristic "flatmate these days"   registry "flatmate"
"What is my allergy now?"                heuristic "allergy now"           registry "allergy"
"My shoe size escapes me - What is it?"  heuristic "shoe size escapes"     registry "shoe size"
"Which hometown did I give you?"         heuristic DECLINES                registry "hometown"
```

**Every heuristic failure mode disappears, because the open class never has to be recognised at all.** And it works for **open** relations — the register is whatever this conversation named, not a lexicon of known relations.

### What landed

| | |
|---|---|
| `relation_phrase_with_seen(source, seen)` | registry first, heuristic fallback — **only exact equality**, longest match wins |
| `TurnCompiler::compile_with_seen` | a **default method that ignores the registry**, so every existing compiler is valid and behaves exactly as before |
| registry **passed in**, not held | the contract that a compiler is stateless across calls and predicts from `source` still holds |

**It cannot introduce an address the heuristic would not also have produced** — it can only replace a mis-bounded phrase with one the conversation actually used.

### Two things I am explicitly not claiming

1. **The 43/43 is the matcher's measured effect, not yet an end-to-end result.** The session does not collect or pass the registry, so `op-probe` still reports **29/43** on the confirmation set. **That wiring touches the turn-compiler contract and belongs in its own reviewable change** — I did not want to land an interface change and a matcher in one PR with the interface untested.
2. **A lexicon match also gives 43/43 and 45/45 — and that is a test artefact.** Both sets use *known* relations. The registry result is the one that matters, because it does not depend on the relation being known.

### Why this also unblocks something older

On #1552 I noted that a **permissive derived-id resolver plus longest-prefix matching is a silent-corruption generator**, and that the fix needs *"identity names plus a session-local registry of names actually seen"*. **This is that registry**, and it now has a measured payoff rather than being a design precaution. It is the strict half of the addressing split.

### Checks at head `ce14bb7d`

```
cargo fmt -p uor-r4-training -- --check        clean
cargo test --release -p uor-r4-training --lib  696 passed, 0 failed, 6 ignored
op-probe mode=phrase  relation-split            140 / 0 / 60    <- unchanged throughout
op-probe mode=phrase  45-form wide             45 / 0 / 0
op-probe mode=phrase  43-form confirmation     29 / 11 / 3     <- registry not yet wired
```

New test `a_registered_phrase_beats_the_heuristic` asserts the mis-bound heuristic *and* the registry's correction, the decline case, the multi-word case, and that an empty registry leaves the heuristic unchanged.

Refs #820, #1552.


### comment 5999046579 — 2026-10-05T16:52:38Z — Casey-allard

## I wired the reader, measured it, and removed it — it fires ZERO times

Following the reachability finding, I made the reader reachable: a **closed-label fallback** where, if the relation heads name nothing, the turn's own phrase may name a relation the identity already carries. Safe by construction — `relation_id` rejects anything outside the identity.

**It worked, it was safe, its gate held exactly, and a test proved it fires. Then I built the counter and ran it:**

```
world=v2, 60 conversations, seed 9101
  MQAR 22/23; open relation 11/12; closed relation 3/3; phrase fallback 0
```

**ZERO. And there is a structural reason, not bad luck.**

The fallback needs **both** the heads to name nothing **and** the phrase to name an already-closed label. **But the heads are trained on the closed labels** — they rarely fail on them, and where they do fail the phrase usually fails too. The two conditions are close to mutually exclusive *by construction*. And on the workload where the heads **do** say nothing — open relations — the phrase names an **open** relation, which `relation_id` rejects by design. **The path is squeezed from both sides.**

**So it is reverted** (`41f975e1` reverts `9c661ed8`), and the finding is on #820. The registry matcher remains a tested component.

### Why I am recording this rather than quietly deleting it

**The gate holding was NECESSARY and NOT SUFFICIENT.** That is the exact gap I flagged two comments ago when I wrote that *a gate that holds while a path is never taken is not evidence the path is safe.* It was safe. **It was also pointless**, and only a counter could tell me — a rate could not, because a rate does not say how often the mechanism applies.

**Three times in one night, at three scales:**

| level | what looked like progress | what counting said |
|---|---|---|
| panel work | **+10** on a held-out panel, p=0.0129 | supplied-answer path; retrieval **0** on a real bench |
| extractor | **45/45** on a tuned set | **29/43 = 67%** held out |
| fallback | reachable, safe, gate exact | **fires 0 times** |

**Each time the number that mattered counted occurrences; each time the number that reported a rate hid whether the mechanism applied at all.**

### State of this PR at `41f975e1`

```
cargo fmt --check                              clean
cargo test --release -p uor-r4-training --lib  696 passed, 0 failed, 6 ignored
world=v2 gate                                  MQAR 22/23; open 11/12; closed 3/3; errors 0
```

Contents: the reader improvements (**29/43 = 67% on a pre-registered held-out set**), the noun-phrase rules, and `relation_phrase_with_seen` as a tested component. **All labelled component results — nothing here is reachable from serving, and I am not claiming otherwise.**

Refs #820, #1552.


### comment 6000121929 — 2026-10-05T17:58:53Z — Casey-allard

## Closing: the reader is a template parser, and the product needs natural language

**Decision: do not merge.** I measured my own work against the instrument that matters and it scores **zero**.

### The measurement that closes it

The disjoint panel uses **natural phrasing**. The shipped reader, across all 100 rows:

| | resolved to a **closed label** | produced **any** phrase |
|---|---|---|
| fact turn (`"I grew up in Tulsa."`) | **0 / 100** | 46 / 100 |
| question turn (`"Where did I grow up?"`) | **0 / 100** | 50 / 100 |

```
"I grew up in Tulsa."                 -> None          (no determiner, no copula)
"My home town is Bergen."             -> "home town"   (the label is "hometown")
"I was raised in Bristol."            -> None
"Where did I grow up?"                -> None
"Which town do I come from originally?" -> None
```

**That is why the panel records 91 of 91 misses as `never_stored`** — the fact is never written, so `"I don't know. You haven't told me yet."` is *correct behaviour given an empty store*.

### What the 67% actually measured

Every number in this PR — 7/12, 45/45, 29/43 = 67% — was measured on **templated forms I authored myself**, in the corpus's own style (`"What did I say my {t} was?"`). **On the natural phrasing the product needs, the same reader is 0%.**

**So the gap is wider than "component vs capability."** Even *as a component*, the number that mattered was measured on a distribution that does not occur in deployment. I built three instruments that could not falsify this, and it took the panel's own `miss_split` to show it.

### The honest ledger for this branch

| what | result | status |
|---|---|---|
| noun-phrase rules (pronoun/auxiliary/participle ends, last-detenter wins, sentence limit) | 33/45 → 45/45 **tuned**; **29/43 = 67% held out** | real, but on **templated** forms |
| `relation_phrase_with_seen` registry | 67% → 100% on the same templated set | **unreachable** from serving |
| closed-label fallback wired into the compiler | **fires 0 times in 60 real conversations** | reverted |
| frozen relation-split panel | **140 / 0 / 60, unchanged throughout** | the one thing that held |

**The regression guard is the only part of this work I would defend without qualification** — it caught an off-by-one that took the panel 140 → 70, and it never moved otherwise.

### What I would keep

Nothing here is reachable from a serving path, and the reader does not transfer to natural language. **Keeping merged code that no path calls is how a repository accumulates the 6,400 test-only lines the audit just found.** The branch stays for the record; the code should not merge.

### What I would do instead

The council converged on three moves, and this is none of them: **make the discipline able to lose**, **run the D5 contest** (the one differentiated claim, zero measurements), and **run the teacher-forcing transport probe**. The write-path problem this PR was aimed at is real — 91/91 `never_stored` — but **it needs a reader that understands natural phrasing, which is a learned capability, not more rules.** I added five rule classes and the natural-language rate stayed at zero.

Refs #820, #1552.



---
_Comment/review ids in this file: 5996827339, 5997206359, 5999046579, 6000121929_

