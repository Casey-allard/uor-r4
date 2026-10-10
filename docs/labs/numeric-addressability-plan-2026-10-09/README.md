# Plan: making a numeric value addressable (Options A and B, costed, with the test and its falsifier)

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **PLANNING ONLY: $0, no pod,
no training, no model run, CPU only.** This document is the deliverable; nothing here is started.

## What this plans against

From [#2114 on main](../v5-numeric-diagnosis-2026-10-09/README.md), measured on the v5 declared run's
sealed replies:

- **10 of the 12 failing numeric rows emit NO digit at all**; 2 emit a digit that was never stored
  (`mem-008` stored `68` and `mem-024` stored `54`, both answering *"Your sister's locker number is
  1."*); **0 name the planted distractor**.
- **All 8 distractor confusions are WORD rows** — the two failure modes separate cleanly by value type.
- The **one passing numeric row emits the stored value exactly**: `mem-040` → *"Your sister's locker
  number is 84."*
- The value **was in the context for 40 of 40 rows** (13 of 13 numeric). Not context loss.
- Numeric rows fail **12 of 13 (92.3 %)**, word rows **18 of 27 (66.7 %)**, and `expected_value` passes
  **40 of 40** — the check is not the obstacle.
- **Vocabulary, measured:** 4096 tokens; **10 pure single-digit tokens; ZERO multi-digit tokens**; the
  tokenizer is **byte-level BPE** (3 specials + 256 byte tokens + 3837 merges).

The behaviour is: fluent, correctly framed sentences **with the value slot empty or filled by something
never stored.** The value does not arrive. The hypothesis this plan acts on is **addressability** — a
two-digit value is two tokens and therefore has no single address to write to or read from.

## Background: three readings were tested and two died

Recorded as structure, not compressed to a conclusion, because the third reading is only worth acting
on because the first two were killed:

| reading | status | what killed it |
|---|---|---|
| **Token count explains the cluster** | **REFUTED** | Numbers tokenize **better** than words — 2 tokens against a 3.74 mean — so length runs the wrong way. |
| **Minimal pairs / digit order** | **REFUTED** | `mem-040`'s planted distractor `74` is a **transposition** of its expected `84` — the most confusable pair the panel can construct — and it **passes**, while `mem-037` (`98` vs `76`, no shared digit position) fails. And **0 of 12 numeric failures name the distractor at all**. |
| **Value addressability** | **OPEN, with a replicated control** | `mem-037`/`mem-040` share a key, a kind and a history and differ only in the value, with opposite outcomes; replicated within-key across `brother` (numeric 1/6), `grandpa` (word 2/3), `piano` (0/3), `treehouse` (0/2). |

**Both refuted readings were the Lead's hypotheses and they stay in the record at that strength.** The
surviving reading is a hypothesis with a control, not a proven mechanism, and this plan is written so
that it can be falsified rather than confirmed.

## A correction to the framing, before either option is costed

The earlier framing said three of the thirteen digit slots are **superscripts `¹ ² ³`**, and treated
them as reclaimable budget. **That is wrong, and the mapping shows why:**

| character | byte | note |
|---|---|---|
| `¹` | `0xB9` | byte token |
| `²` | `0xB2` | byte token |
| `³` | `0xB3` | byte token |

All **256 byte tokens are present** in the vocabulary, and `¹ ² ³` are the byte-level BPE
representations of three high bytes, not digit slots. **They cannot be reclaimed without breaking
byte-level coverage** — removing them would make the tokenizer unable to encode those bytes at all.

**So Option A cannot be funded by the superscripts.** Its funding must come from the **3837 merges**
(the least-used ones are removable, and that is measurable against the corpus) or from **growing the
vocabulary**, which costs embedding parameters and invalidates artifacts in any case. The corrected
fact is recorded here rather than quietly dropped, on the same discipline that kept the two dead
readings.

## Option A — number-aware tokenizer: give the value one token and therefore one address

**Scope.** The tokenizer's vocabulary and merge table, not the model. Concretely: add multi-digit
tokens so that a value like `84` is **one** token.

**Sizing, which is a measurement the plan must take first.** The panel is all **two-digit** values, but
the corpus is not, and the slot count is a function of the ranges actually used:
- two-digit coverage (`00`–`99`): **100 slots**;
- plus one/three/four-digit coverage as the corpus requires — the count must be **measured from the
  corpus**, not assumed, and the plan's first task is that count.

**Funding.** 4096 is not a hard ceiling but a size. Three funding routes, in preference order:
1. **retire the least-used merges** — measurable: tokenize the corpus, count merge usage, retire the
   tail. This is lossless for the corpus and costs nothing to train.
2. **grow the vocabulary** by the required slots. Each token costs `width` embedding parameters at the
   input and the output (`576` × 2 per token at this config), so 100 slots ≈ 115k parameters on a
   29M model — 0.4 %. Cheap in parameters, **not** cheap in what it invalidates (below).
3. **Renumber/compact the byte alphabet** — not recommended: it changes every token id and gains
   nothing.

**What it invalidates, exactly.** Every artifact whose weights were trained with `d36d3e87…` becomes
**incompatible**, because a token id means something different: `chat-29m-B-lr5e-4`, the `chat-100m-*`
family, and **every sealed result that cites `d36d3e87…`** — including the reply panel's 43/232, the
cap-96 comparison, and the v5 declared run's 10 of 40. Those results **stay valid as results** (they
are measurements of the artifacts they name) but **cannot be compared to a new-tokenizer model as if
the instrument were unchanged.** A new tokenizer means re-drawing nothing but **re-tokenizing
everything**, and the ladder rungs trained on the old tokenizer cannot serve as a baseline for a
same-panel comparison.

**Retraining cost.** Option A requires **a full retrain of every rung you want to compare**, because
the tokenizer is an input to training. It is not a fine-tune: the embedding matrix changes shape.

**What it does not change.** The change is **purely additive to the vocabulary** if funded by route 2:
non-numeric text keeps its existing tokenization only if the merges above it are unchanged, which is
true only if the new tokens are appended and no existing merge is altered. **If funding comes from
retiring merges (route 1), the encoding of non-numeric text DOES change**, and that must be measured
and reported per corpus, not assumed away.

## Option B — an explicit numeric slot in the memory path: address the value as a value

**Scope.** The memory path — how a value is written to and read from exact addressed memory — not the
tokenizer and not the embedding.

**Where the slot lives.** The memory path already stores keyed records addressed by identity. The
change is that a value recognised as numeric is stored as **one slot holding the value's digits as a
unit**, with the address computed over the value rather than over its token run. The decoder still
emits digits as tokens; what changes is that the memory read returns the value as an ordered unit
instead of a two-position sequence.

**What carries the value's identity.** The value's own digits under an exact numeric encoding — the
natural fit for a project whose memory is exact and addressed, and consistent with the existing
principle that a hash identity is not a semantic distance: the slot must carry the value, not a hash
of it.

**What it does not invalidate.** **Existing artifacts stay valid**: the tokenizer is untouched
(`d36d3e87…`), the embedding is untouched, and a model trained without the slot can still be graded on
the old panels. This is Option B's decisive practical advantage.

**Can it be tested without retraining the whole model?** **Partly, and this is the important
qualification.** Two questions must be separated:
- *Does the memory path deliver a numeric value at all?* — testable **without training** by a
  write/read probe over the memory path (the project already has probe binaries of this shape, e.g.
  `binding-probe`, `op-probe`), which is CPU-only and cheap.
- *Does a model trained to use the slot deliver values?* — needs training, because the write/read
  policy is learned (D8/D19: the operators are learned, not authored).

So Option B splits into **a cheap untrained probe** that can kill or support the mechanism, and **a
training run** that would be needed to move the number.

## Recommendation, and the smallest version that could still be decisive

**Recommendation: take the untrained probe first, and do not choose between A and B until it reports.**
The probe is CPU-only, costs no GPU time, and answers the one question both options rest on: *does the
memory path carry a two-digit value as an ordered unit today?* If it does, addressability is refuted
and neither option is the lever. If it does not, the probe says **where** it fails, which is exactly
what decides between A (the value never had one address) and B (it had one and the read path lost it).

**Smallest decisive version, and it is deliberately tiny:**

1. **A write/read probe over the memory path with one two-digit value** — write `84`, read it back,
   and report whether the delivered run is `8`,`4`, whether the order is preserved, and whether `74`
   (a transposition) is distinguishable from `84`. **CPU only, no training, no GPU.** This is the
   smallest thing that can be decisive, and it is the direct test of the mechanism the diagnosis
   named.
2. **If and only if the probe shows the value is lost in the memory path**, the next step is **Option
   B's smallest form** — the numeric slot on **one** value type, trained at the **existing 29M rung**
   against a **fresh sealed numeric panel** (below). Not a new ladder; the smallest model that can
   show the effect.
3. **Option A is held** until the probe reports, because it invalidates every artifact for a mechanism
   that may not be the cause.

## The test: a fresh sealed panel, drawn under the v5 discipline

**v5 is not re-run** — it is development evidence now, and its 10 of 40 stands as its declared reading.
A fix needs a **fresh sealed panel**, and the diagnosis tells us exactly what it must contain.

**What the panel must contain to distinguish "one address fixed it" from "the model was never going to
deliver the value":**

| requirement | why |
|---|---|
| **Numeric and word rows in matched pairs, same key, same check kind, same history** | the `mem-037`/`mem-040` control, replicated by construction rather than found afterwards |
| **Value held fixed, distractor varied by digit transposition, and vice versa** | separates "the value is not delivered" from "the distractor is chosen" |
| **Magnitude spread: one-, two-, three- and four-digit values** | the diagnosis could not test magnitude because the panel was all two-digit; the fix must be shown to scale |
| **Position spread: the value at the first, middle and last user turn** | separates addressability from recency |
| **Values carrying leading zeros and repeated digits** (`07`, `77`) | a one-token encoding and a two-token encoding differ most here |
| **Both value types present in the same conversation** | rules out a per-conversation artefact |
| **Deterministic checks with per-row provenance, controls verified before sealing** | the v5 discipline: byte-reproducible draw, `expected_value` / `binding_swap` / `copy` / `echo` / constants / derangement all verified **before** sealing, and `check_panel` run with `check_tokenizer` and the worst-case position reported |
| **The binding swaps actually embedded** | the v5 control was vacuous for its whole life because the swaps file was never added to `EMBEDDED_SWAPS`; a new panel must assert its own control is non-vacuous at seal time |

**Pre-declared target, frozen before the panel is drawn** (mirroring v5's shape, and stated here so it
cannot be improvised):

> **Primary:** ≥ **34 of the 40** numeric memory rows at `check_pass`, on the fresh panel, with the
> same declared cap, context and controls as v5.
> **Secondary, and the one that actually tests the mechanism:** **≥ 30 of the 40 numeric rows emit a
> digit run equal to the stored value**, measured by string equality on the reply's digit run — because
> the diagnosis says the failure is emission, not selection, and a `check_pass` number can rise
> without the mechanism being fixed if the model learns to guess.
> **Control, non-negotiable:** `binding_swap` must report **40 checked rows** and **0 passes**, and the
> `expected_value` control must pass **40 of 40**, or the panel is void.

## The falsification condition, stated in advance

**This fix is the wrong lever if any of the following holds**, and each is measured, not argued:

1. **The untrained probe shows the memory path carries a two-digit value as an ordered unit today**
   (order preserved, `84` distinguishable from `74`). Then addressability is refuted, and neither A nor
   B addresses the defect.
2. **On the fresh panel, `check_pass` rises to the target while the digit-run-emission number does
   not.** That would mean the model is guessing values the memory did not deliver, and the mechanism
   was not the fix.
3. **Numeric and word rows improve together.** If a change to numeric addressability also moves the
   word rows, the change was not value-type-specific and the cluster was not a numeric defect.
4. **The numeric deficit fails to track magnitude or leading zeros on the fresh panel.** If `7`, `84`,
   `007` and `1024` fail at the same rate under a one-token encoding, the value's token count was not
   the constraint.

Each of these is a result. **None of them requires a v5 re-run.**

## Cost estimate, with the GPU/CPU split

**CPU (this laptop) — calibrated from two measured runs tonight:**

| step | measured basis |
|---|---|
| drawing and sealing the fresh panel | minutes (v5's draw is byte-reproducible from a seed) |
| `chat-grade check` (structural validation) | seconds |
| generating replies, 64 rows at 29M, cap 64 | **2m04s measured** (2,154 ids) |
| grading 64 rows with the local judge | **9m46s measured** |
| the untrained memory probe | **unknown until built; CPU-only and small** |

**GPU (via `uor-pod`, caps ≤ 4 pods and ≤ $8/h) — a projection, and its one unknown must be measured
before anything is committed:**

| step | basis |
|---|---|
| a timed calibration run at the 29M rung | **required first**: the ladder's training throughput is not recorded in the repo at a level I can cite, and inventing it would be exactly the error this line has been avoiding all night. One short timed run turns the projection into an estimate. |
| the full Option B retrain at 29M | `calibration_throughput × corpus_tokens × epochs`, at the pod rate |
| the full Option A retrain | the same, **plus every rung you want to compare**, because a new tokenizer invalidates them all |

**The cost asymmetry is the plan's main economic point:** Option B leaves every existing artifact
valid and needs one retrain at one rung; **Option A invalidates the whole ladder** and needs each rung
rebuilt before any comparison. That is a large multiple, and it is why A is not recommended until the
probe says the value never had an address in the first place.

## What does not change

**Criterion 1 remains NOT MET on both halves, and 43/232 is unchanged.** The memory half is measured
and missed (10 of 40 against a declared ≥ 34); the reply half cannot carry the criterion as
instrumented. **Nothing in this plan changes that, and no step of it would**: it is a plan to test a
mechanism, and the acceptance criterion moves only when a candidate is trained and scored on a fresh
sealed panel.

## Next

1. **Build the untrained memory write/read probe** — one two-digit value, CPU only, no GPU, no
   training. It is the smallest decisive step and it decides between A and B. **This is the piece to
   do next.**
2. **If the probe says the value is lost**, take **Option B's smallest form**: a numeric slot for one
   value type at the existing 29M rung, against the fresh sealed panel defined above.
3. **Hold Option A** until the probe reports, and if it is ever taken, fund it by **measured merge
   retirement** (not by the superscript slots, which are byte tokens) and accept that the whole ladder
   must be rebuilt before any comparison.
4. **Never re-run v5.**

Evidence: this plan rests on [#2114](../v5-numeric-diagnosis-2026-10-09/README.md) and its bundle
(`v5-numeric-diagnosis-2026-10-09.tar`, 25,600 B, md5 `1b7b37d5c039e5412653228067c47428`); on the
frozen tokenizer `d36d3e87…`; and on the two CPU timings measured in the v5 acceptance run
(`v5-declared-acceptance-2026-10-09.tar`, md5 `27518f606964dc316631e6afb0d52239`).
