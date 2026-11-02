# Decision card: Q5's causal prefix holds the readout read's earlier keys (integer-nearest-readout, claude-254, 2 Nov 2026)

Lab: claude. ORDER: [claude-254](ORDER.md). This card amends one definition in claude-247's [card](../claude-247/DECISION.md), Q5's rule, before any trace value of Q5's set is read. It changes how a bar is applied, so it is a decision card (CHARTER §5.1, §5.2). claude-247 says "Changing it needs a new card": this is that card.

## The question

**Which decisions does the forward at a position p read, so that the forced replay compares exactly those?**

claude-247 (lines 132–136) defines p's causal prefix as the decisions "the forward at p reads exactly":
- at every position q ≤ p: every stream read's codes and kept sets, every layer's rate buckets, and the turn memory's lookups;
- at p alone: the readout read's codes and kept set, and the nearest.

It then rules (line 157) that "a kept set or a memory lookup that differs is never a near tie. With equal codes and equal token ids, both sides must decide it the same way."

## What is measured

Read at `098afdf63` (base `f1405cc10`). No trace value of Q5's set was read.

- **A read's kept set at row t reads key codes recorded at earlier steps.**
  - A head's key code at step t is "the code of `x_{t−1}`, which every later row reads for candidate `t`" (`DecisionHead::key`, `crates/uor-r4-integer/src/fbs_snapshot.rs:561–562`).
  - A kept set at row t chooses among candidates 1 ≤ s < t by the distance from t's query code to each candidate's key code (`kept_set` and `kept_in_order`, the same file and `in_space_long.rs`).
  - So the readout read's kept set at p reads the readout's key codes recorded at steps 1 to p − 1.
- **The listing leaves those keys out.** It places the readout read's codes at p alone. A readout key that differs at some q < p is then never compared, and the first difference the replay sees is the readout's kept set at p.
  - Line 157's reason then fails: the two sides' codes are not all equal, so they need not decide the kept set the same way.
  - The replay would end "not accounted for" at a kept set whose cause it never judged. That cause might be a near tie.
- **The key at step 0 is no decision.**
  - It names no candidate (`fbs_snapshot.rs:562`): no row reads it.
  - R writes 0 there (`hamming.rs:687`, `:701`). The float sides write their coordinates' sign packing at every row, step 0 included (`decision_read`'s `key: h.key[t]`, `crates/uor-r4-float-train/src/in_space_long.rs:1689`).
  - On DEV s1 window 41: `jq -c '.steps[0].readout.heads | map(.key)'` on claude-253's window extract `run/base-w0.json` (`22b64304…`) gives `[10116873416631713224,4072491265759236392,17609115278902124046,11679911603265082676]`. `jq -c '.windows[0].steps[0].readout.heads | map(.key)'` on R's trace (claude-249's `smoke/trace-dev-s1.json`, `11e6b7d9…`) gives `[0,0,0,0]`.
  - R refuses to force it: "a forced key at step 0 names no candidate" (`crates/uor-r4-integer/src/stack/hamming.rs:639`).
  - So in the listing, every step that compares a float side with R would differ at step 0 by convention alone, and the replay could not force the difference on R.
- **The keys at step p itself are read by no decision at p** (candidates are s < p). The listing includes them. Including them can only fail p, never let it pass.

## The options

- **A. Keep the listing as written.**
  - A readout key flip at q < p ends the replay at p's kept set, judged never a near tie, by a cause the replay did not compare.
  - Every step against R differs at step 0.
  - Set aside: it contradicts the card's own principle (line 132) and its own reason (line 157).
- **B. Add the readout read's earlier keys, and drop the step-0 key (recommended).**
  - p's prefix gains the readout read's key codes at every step 1 ≤ q < p.
  - The key at step 0 leaves every prefix, and leaves G₁₀'s family values: a family is "a set of values that make one kind of decision" (line 139), and this value makes none.
  - Nothing else changes. Defined exactly below.
- **C. The exact causal prefix: B, and also drop every key at step p.**
  - This removes decisions that cannot change p's pick. But the listing keeps them, and claude-247 is strict by design (line 239): "A clear flip that did not matter still fails the position" (line 264).
  - It is not needed for the rule to be sound.
  - Set aside: it would loosen the card where the card chose strictness.
- **D. Add the whole readout read at every q < p.**
  - The readout's query codes and kept sets at q < p reach only q's own pick (line 136, `hamming.rs:556–563`, `637–656`).
  - Set aside: it would fail p by decisions p never reads.

## Recommendation: B

**p's causal prefix** is, in the forward's order:
- at every step q from the window's start to p:
  - each layer in layer order: its rate buckets in lane order, then the stream read after it, if one runs. Within that read, heads in head order, and within a head: the query code, the key code (only for q ≥ 1), then the kept set;
  - the readout read:
    - at q < p, each head's key code alone, in head order (only for q ≥ 1);
    - at p, as claude-247 lists it: within each head, the query code, the key code (only for p ≥ 1), then the kept set;
  - the turn memory's lookup, where one runs;
- at p, last: the nearest.

**The families.** G₁₀(φ, P) counts every value of φ that P's trace records in the cell, as claude-247 sets, except a key's coordinates at step 0.
- Leaving them out changes N, and so k, by one row's coordinates per window. The k-th smallest can then move either way.
- That is not a loosening: the rule leaves out a value that makes no decision, and it is fixed before any value is read.

**Everything else in claude-247's card stands:** the near-tie rules, the pick's scores, the replay's steps and order, Row 0, the outcomes and the readings.

**What B changes.** It changes only a position whose readout key differs at some q < p, or a step that compares a code at step 0.
- In A, such a position ends "not accounted for" at the kept set. In B, the key is compared first: a near tie is forced and the replay goes on; a clear flip fails p, as any clear flip in the prefix does.
- So B judges more than A, at the place the difference arises. It never counts a clear flip as a near tie.

## Constraints

- **Fixed before it is read.** No trace value of Q5's set has been read. Whether Q5 holds is not known; it can fail.
- **The writer:** the claude lab. The grader (this unit, claude-254) implements B. The sheet that reads Q5 registers claude-247's rule with this amendment verbatim, with its FIT and pre-run review.
- **Scope:** none beyond the item's. The grader lives in `fbs_snapshot.rs` and `fbs-toks.rs`, both in the item's scope.
- **Cost:** none. The prefix gains at most 190 × 4 key codes per position: the readout's keys at steps 1 to p − 1 of a window of at most 192 positions, for its 4 heads on R's artifact. The traces already record them.
- **Left for the sheet: the turn memory.** claude-247's line 157 also assumes that equal token ids decide a memory lookup.
  - `DecisionMemory::key` records token ids (`fbs_snapshot.rs:578`).
  - R serves no point key: `--forced` refuses an artifact that does (`crates/uor-r4-integer/src/bin/fbs-toks.rs:358–363`).
  - claude-247's I-Complete checks that "no point key or served key changes F's teacher-forced nearest".
  - The sheet cites the code that forms the lookup's key on each side. If a key reads the stream, a further card comes before any value is read.
- **Casey can reverse it:** logged on #3462 as `[claude] owner log:`.

## Decision review

A fresh-context reviewer read CHARTER §4 and §5, the item, claude-245's and claude-247's cards, this card and its ORDER, and the code it cites. **Verdict: APPROVE WITH NITS**, no blocker.
- **The premise is true.** A kept set reads each candidate's key, forced or computed (`hamming.rs:204`, `:384`; `kept_set`, `fbs_snapshot.rs:728–731`). Every key is a lookup by token id (`hamming.rs:687`, `:701`, `:346`), so its place inside step q cannot reorder cause and effect.
- **B is sound and no looser than A.** It only adds decisions to the prefix (the step-0 key aside, which no read uses). It accounts for a position A fails only when that failure came from an uncompared near-tie key.
- **The options are fair.** D is rightly set aside, since a payload is written before the read's sum (`hamming.rs:650–652`). No other read has the gap.
- **The nits, all fixed in this card:**
  - the step-0 values' exact jq paths and the float writer's line;
  - the step-p readout key's "only for p ≥ 1";
  - that leaving the step-0 values out of G₁₀ can move it either way;
  - what the cost's 4 counts;
  - the memory key, left for the sheet with its citations.
