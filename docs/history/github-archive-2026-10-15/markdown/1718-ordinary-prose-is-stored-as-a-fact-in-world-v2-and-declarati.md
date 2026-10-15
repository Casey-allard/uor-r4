# #1718 — Ordinary prose is stored as a fact in world=v2, and declarative sentences can overwrite stored values

- **Type:** Issue
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-05T05:08:46Z
- **Updated:** 2026-10-09T14:43:17Z
- **Labels:** bug
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1718
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Ordinary prose is stored as a fact in `world=v2`, and some constructions OVERWRITE

Found while measuring whether the relation extractor could be extended to real English statements on #1552. **This is a defect in shipped `world=v2` behaviour and is independent of that work.**

### Measured

Ordinary prose through the saved compiler (`compiler-save-op-v25-rawtable` + `op-model-v25`), 10 sentences:

```
unresolved   "That sounds interesting."
unresolved   "I had a good day today."
ASSERT   3   "The weather is nice today."        span "nice today"
unresolved   "Thanks for your help."
ASSERT  10   "I like reading books."             span "books"
CORRECT  4   "It was quite busy at work."        span "busy"
unresolved   "My favourite part was the ending."
unresolved   "The traffic was terrible this morning."
unresolved   "I think it might rain later."
ASSERT   9   "She said the food was lovely."     span "lovely"
```

**4 of 10 are stored — 40%.**

**The `CORRECT` is the severe one:** `"It was quite busy at work."` compiles to a *correction* on relation 4, which **overwrites** the stored value rather than adding a stray one. A speaker making small talk can silently replace a remembered fact.

### Why it happens

The span head marks a value-like phrase (`"nice today"`, `"books"`, `"busy"`, `"lovely"`) and the relation head names a closed label, so the turn compiles as a statement. Nothing checks whether the turn is *about the speaker's record* — only whether the heads found a relation and a span.

### Honest scope

**10 sentences is a small sample; 40% is a point estimate with wide error.** The claims I will stand behind: the rate is **non-trivial** on ordinary prose, and **at least one common construction compiles to `Correct`, i.e. an overwrite.** A proper rate needs a larger, pre-registered prose set authored before measurement.

### Proposed next step

1. Author a pre-registered prose set (smalltalk / non-relation turns, several hundred sentences, hashed before running).
2. Measure the false-write rate and the **overwrite** rate separately — an overwrite is strictly worse than a stray assert.
3. Measure the same on the world's own `Responsive` cell in a `world=v2` session, so the rate is in-distribution as well as out-of-distribution.
4. Only then decide the fix — the candidates are a memory-turn discriminator, or a confidence/margin gate on the statement path, and the choice should follow the measured shape rather than precede it.

Refs #1552. Related: the out-of-scope guard in the extractor exists precisely to avoid this, and it is why extending the extractor to real English statements is blocked.


## Comments (2)

### comment 5988822120 — 2026-10-05T05:43:59Z — Casey-allard

## Proper measurement: 20.8% false-write, 8.7% overwrite — on a pre-registered 264-sentence set

The original report used a 10-sentence probe (40%, wide error). Replaced with a **pre-registered** set: **264 sentences, authored and hashed before any measurement**, file-byte sha256 `085a256d3fe0c050abf6a5e513fd2c581fd933946190e67ee41d908b808a8c88`.

| | shipped `world=v2` |
|---|---:|
| unresolved | 209 |
| ASSERT (adds a fact) | 32 |
| **CORRECT (overwrites)** | **23** |
| **false-write rate** | **55/264 = 20.8%** |
| **overwrite rate** | **23/264 = 8.7%** |

**One in five ordinary conversational turns is stored as a fact; one in eleven silently replaces a stored value.**

Worst examples:

```
CORRECT  rel=7  span="problem"     "It is not a problem."          <- a NEGATION stored as a fact
CORRECT  rel=6  span="train was"   "The train was delayed again."  <- the stored span is a FRAGMENT
CORRECT  rel=4  span="busy"        "It was quite busy at work."
ASSERT   rel=9  span="lovely"      "She said the food was lovely."
```

### Cause, now located

**The false writes come from the OP MODEL, not from any relation-phrase extraction.** Measured by running the same set with open-relations addressing both off and on:

| | closed | open |
|---|---:|---:|
| false-write rate | 20.8% | **20.8%** |
| overwrite rate | 8.7% | 8.0% |

**Identical.** So the defect is in the op model's statement path — it emits a closed relation and a span for prose that states nothing — and the deterministic extractor's out-of-scope guard is unrelated to it.

**That guard is also why this issue matters beyond its own scope:** it exists to prevent exactly this, and the measurement above (open path no worse than closed) is evidence the guard works. Extending the extractor to real English statements would forfeit that protection.

### Repro

```
op-probe mode=prose open=0 prose=~/uor-r4-local/ladder/prose/prose-set.txt \
  compiler=…/compiler-save-op-v25-rawtable trunk=…/op-model-v25 \
  tokenizer=…/bundle-learned-1/tokenizer.json panel=…/rs-requests.json
```

### Next

The rate is measured; the fix is not designed. Candidates, in the order the measurement suggests: a **margin/confidence gate** on the statement path (the false writes have low-margin spans like `"train was"`), and a **negation check** (`"It is not a problem."` must never store). Both need their own pre-registered evaluation, and the verifier caution applies — a gate must be measured on **both** the writes it prevents and the legitimate writes it costs.


### comment 6083192534 — 2026-10-09T14:43:16Z — Casey-allard

Kept open as a bug in the 9 October tracker rebuild (#2028); related milestone: #2031.


---
_Comment/review ids in this file: 5988822120, 6083192534_

