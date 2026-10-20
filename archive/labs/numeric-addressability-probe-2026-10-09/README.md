# The untrained addressability probe: the value DOES have an ordered home — addressability is refuted at the storage layer

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. 2026-10-09. **Probe only: CPU only, no
pod, no GPU, no training, no model run, $0.** One focused test over the numeral codec the memory path
already uses.

## What the probe was asked, and the answer

The plan ([numeric-addressability-plan-2026-10-09](../numeric-addressability-plan-2026-10-09/README.md))
named the untrained memory write/read probe as the smallest decisive step. It asked four things. It
answers them from `crates/uor-r4-core/src/native_geometric/numeral.rs` — the codec the memory path
already uses — with **all five numeral tests passing, including the probe added here**.

| question | answer |
|---|---|
| **(a)** does `84` come back as the two tokens `8` then `4`? | **Yes — and the codec recovers it as ONE value `84`**, with the two-token run recorded as its inclusive interval `start 0, end 1` |
| **(b)** is ORDER preserved? | **Yes.** `8`,`4` → **84** and `4`,`8` → **48**: the same two digits in the other order are a *different value*. The identity is carried by order, not by a bag of digits. |
| **(c)** is `74` distinguishable from `84` on read? | **Yes** — different accumulated value, and both recovered from a two-token run |
| **(d)** leading zeros and repeated digits? | `0`,`7` → **7** (leading zeros distil to the value); `7`,`7` → **77**; the same value twice → two literals **separated by interval** |

**Verdict: REFUTED — a two-digit value has an ordered, positioned home at the layer the memory path
reads through.** The premise the plan acted on — *"a two-token value has no single address to live
in"* — is not true of this representation. The codec accumulates digits left-to-right across token
boundaries and retains the inclusive token interval, so order is not merely representable, it is the
thing that carries the value's identity; and repeated values at different positions are already
distinguished by interval, which is the property the project's own rule demands (*a prime/hash identity
is not a semantic distance*; equal tokens at different positions must not share identity).

## What the probe does NOT settle

**It is an untrained probe and it tests the representation, not the learned path.** It cannot say
whether the model's read/emit path *uses* this codec to deliver a stored value. The diagnosis's
measured emission failure — **10 of 12 failing numeric rows emit no digit at all**, 2 emit a digit that
was never stored, the one passing numeric row emits the stored value exactly — **stands unchanged and
is now localized to the learned path rather than to the representation.**

So the honest statement is narrow and precise: **the storage layer can hold a two-digit value as an
ordered unit; something downstream of it is not delivering that unit.** Those are different failures
and this probe separates them.

## Consequence for the plan's two options

- **Option A (number-aware tokenizer: one token → one address) loses its stated rationale.** Its
  motivation was that a two-token value has nowhere to live and a one-token value would. The probe
  shows the two-token value already has an ordered home, so **Option A goes back on the shelf** — and
  it was the option that invalidates every artifact trained on `d36d3e87…` and needs the whole ladder
  rebuilt. Not spending that on a refuted premise is the probe's value.
- **Option B (explicit numeric slot in the memory path) is not refuted, but its premise is
  weakened.** "The value is not delivered because it has no slot" is now wrong at the storage layer.
  What survives is the *emission* question: a correctly stored, correctly ordered value is not being
  selected and emitted.

**The next question is therefore not "where does the value live" but "why does a correctly stored,
correctly ordered value fail to be emitted".** The plan's falsification condition 1 —
*"the probe shows the memory path carries a two-digit value as an ordered unit today"* — **has fired**,
and by the plan's own terms the addressability fix is the wrong lever.

## The ledger, at its own strength — four readings, three dead

Preserved as structure, as the plan requires, because the surviving reading is only worth acting on
because the others were killed:

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers tokenize in **2 tokens against 3.74** for words — length runs the wrong way |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a **transposition** of its expected `84` and it **passes**; **0 of 12 numeric failures name the distractor** |
| Value addressability — "no single address" | **REFUTED by this probe** | the codec recovers a two-token run as **one ordered value with an interval**, preserves order, distils leading zeros and separates repeated values — all five numeral tests pass |
| The learned read/emit path does not deliver a correctly stored value | **OPEN — and now the only survivor** | 10 of 12 numeric failures emit no digit; the one passing row emits the stored value exactly; `expected_value` passes 40 of 40 |

## Limits

- **Untrained.** The probe exercises the codec and its tested properties; it makes no claim about
  learned behaviour.
- **It does not measure the memory ring's read path directly** — it measures the numeral codec the
  memory path reads through, which is the representation the plan's premise was about. A probe that
  drives the ring's learned reader would need a trained model and was out of scope by the plan's own
  constraint.
- **It says nothing about whether a one-token encoding would still help** for other reasons (a shorter
  digit run is cheaper at emission). It refutes the *addressability* rationale, not every possible
  benefit of Option A.

## What does not change

**Criterion 1 remains NOT MET on both halves and 43/232 is unchanged. Nothing in this probe changes
that, and no step of it could.** The v5 declared run's 10 of 40 stands as its declared reading, and
**v5 was not re-run.**

## Next

**Ask the emission question, not the storage question.** The surviving reading is that a correctly
stored, correctly ordered value fails to be **selected and emitted**. The cheapest next step is
therefore a probe of the **learned** path's value selection rather than another representation change:
for the 13 numeric v5 rows, check whether the read path *selects* the right token at all before
emission — i.e. is the failure in selection or in emission? That distinction is measurable on the
existing sealed replies plus a read-path probe, and it decides whether the next fix belongs to the
reader or to the emitter. **Option A stays on the shelf; do not rebuild the ladder on a refuted
premise.**
