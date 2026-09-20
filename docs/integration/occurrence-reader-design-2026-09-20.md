# Design note — learned geometric selection of exact token occurrences

Written **before** any fresh evaluation, per the execution prompt. This records the equation, causal
data flow, estimated cost, likely failure and falsifier, and where I deviate from the sketch with a
reason.

## Frozen local baseline

`z_local(v) = z_E(v) + u_S(b)` where `b = gamma_S[B_S(x_i)]` is the frozen separable artifact's
query state for the current token and `u_S(s) = (Σ_j W_S[v,j] · R_S[s,j]) << 7` is its single-row
residual. Following S's immutable absence behaviour, **the residual is applied only where S's own
`inference_rows` is `Some(..)` (`i ≥ 2`, non-empty older prefix); otherwise `z_local = z_E`.** S's
history row is never added. `E` and the S artifact are hash-pinned and loaded, not retrained.

## Mechanism

**Ring.** A fixed circular buffer of `RING_CAP = 128` observed tokens with a session `seq`.
`observe` writes the next absolute index; `reset` increments `seq` and clears the window, so every
older `OccurrenceRef { seq, abs }` is rejected by sequence identity. The prediction is computed
**before** `observe(x_i)` for position `i`, so the ring never contains the target.

**Admission (causal, exact).** Scanning the live window newest-first, candidate `j` is admitted iff
`token(j) == x_i`, bounded to `MAX_CANDIDATES = 24` visits. Admission reads only `x_{≤i}`; the
successor/payload is never inspected. Evicted or stale postings are reported as evictions, and an
unfollowed posting is never treated as proof of absence.

**Features (all 0/1, so every weight product is a conditional add).** With the query context
`(x_i, x_{i-1}, x_{i-2})` and the frozen S write map `A(t) = gamma_S[a_codes_S[t]]`:

| Feature | Definition |
| --- | --- |
| `f0` | `x_{j-1} == x_{i-1}` (exact role match) |
| `f1` | `x_{j-2} == x_{i-2}` (exact second-order match) |
| `f2` | `A(x_{j-1}) == A(x_{i-1})` (geometric role-slot equality) |
| `f3` | `A(x_{j-2}) == A(x_{i-2})` (geometric second-order slot equality) |
| `f4` | `j == i-1` (adjacent occurrence) |
| `f5` | `x_{j+1} == x_i` (payload repeats the query token) |
| `f6` | `j` is the newest admitted candidate |
| `f7` | `f0 ∧ f2` (exact match that is also geometrically equivalent) |
| `f8` | `f2 ∧ ¬f0` (geometric collision: equal slot, different token) |

`f2`/`f3` are genuinely geometric and are *coarser* than exact identity: 4,096 tokens map onto 8
palette slots, so `f2` fires for many non-identical roles. `f8` isolates that collision, and a learned
negative weight on `f8` is the mechanism by which geometry can be used without trusting it. This is
the shared invariant of `source_routing.rs:68–102` (relative element + rank table, explicit
`NO_SOURCE`) re-expressed for the BPE token path, whose artifact and grammar are **not** reused.

**Selection.** `score(c) = Σ_k θ_k · φ_k(c)` in `i32` (each term a conditional add, no multiplier);
the explicit NoRead action scores the constant `μ`. Choose the highest-scoring candidate iff
`score > μ`, ties to the newer candidate; otherwise NoRead. `θ` and `μ` are quantized to signed ≤4-bit
integers (`|·| ≤ 7`).

**Emission.** `z(v) = z_local(v) + A · 1[v == payload(j*)]`, with `A = 1 << amp_shift` a declared
power of two, so the residual is a table write / add. NoRead adds nothing, so read-disabled is
exactly `z_local`.

Amplitude is **not** learned. It is chosen on fit/tune from the margin distribution
`m = max_v z_local(v) − z_local(target)` over covered positions: the smallest power of two covering a
declared quantile. This separates "the selector chose the wrong source" from "the correct source
could not win", which the prompt requires be reported separately.

## Causal data flow

```
tokens x_0..x_{i-1}  ──►  ring (seq-bound, bounded)
        x_i          ──►  admission: j with x_j == x_i  ──►  ≤24 candidates
 (x_i, x_{i-1}, x_{i-2}) ─►  features ──►  θ,μ  ──►  selection or NoRead
                                                          └─►  payload(j) = x_{j+1}
z_E(prev,cur) + u_S(b)  ──►  z_local  ──►  + A on payload  ──►  argmax  ──►  x_{i+1}
        observe(x_i)      ──►  ring
```

Nothing downstream of `i` enters admission or selection. Future-token causality is a focused test.

## Learning target and label convention

Supervision is the **observed next token**, restricted to positions with a non-empty candidate set.
Label = the **newest admitted candidate whose payload equals the target**; if none, NoRead. A
position may admit several payload-correct candidates; the hard metric credits *any* of them, while
training uses the newest (declared convention, not a label leak — the label uses only observed
tokens at `j+1 ≤ i`).

Objective: cross-entropy of a softmax over `{candidates ∪ NoRead}` with the same features, plus a
small L2 penalty on `θ`. Optimizer: Adam over 10 scalars. Then quantize. The **hard exported
predictor** — not the surrogate — is what all reported metrics measure.

## Estimated cost

Selector: 10 parameters, 64-bit parameter bytes negligible. Per-position serving work: one parent
forward (`dv=128` × 4,096 rows), one 16-wide residual row add, one backward scan of at most 128
ring entries, ≤24 candidate feature vectors of 9 bits. All integer; no multiplier.
Fitting: ≤ 20,000 positions × ≤ 25 actions × 9 features × a few hundred steps — seconds.
Memory: ring 128 × 12 B ≈ 1.5 KiB; no token-pair logit cache in the served path.

## Likely failure and falsifier

*Likely failure:* the synthetic panel is solvable by `f0` (exact role match) alone, so the geometric
features add nothing and the recommendation is "exact-memory integration works; a geometric ranking
advantage is unestablished". This is a legitimate outcome and will be reported as such rather than
tuned away.

*Falsifiers.* The mechanism is rejected at this design if (a) read-disabled does not reproduce
`z_local` exactly; (b) appending future tokens changes an earlier prediction; (c) any selected
reference does not resolve through its own sequence identity; (d) the hard exported reader does not
beat both the frozen local baseline and the fixed latest-occurrence selector on the fresh panel at
the predeclared margin; or (e) the raw-text probe shows the reader worsening all-target loss beyond
the tolerated local regression.

*Primary endpoint (predeclared).* On the fresh synthetic panel, at **reader-relevant positions**
(positions where the correct next token is not the local argmax but is the payload of some admitted
candidate): reader hard accuracy must exceed both `z_local` argmax accuracy and the fixed
latest-occurrence selector's accuracy by **≥ 0.20 absolute**, with a paired-by-sequence interval
excluding zero. Secondary: all-position micro CE must not regress by more than **+0.01 bits/token**
on the fresh synthetic panel or **+0.02 bits/token** on the bounded raw-text probe.

## Deviations from the sketch, with reasons

1. **Amplitude is a declared constant, not a learned score.** Learning a continuous amplitude would
   require a second quantization argument and would conflate "wrong source" with "too small a
   residual". The prompt explicitly permits a declared shift scale chosen on fit/tune and requires
   the two failure modes be reported separately.
2. **All features are 0/1 indicators.** This makes the served dot product a sequence of conditional
   adds, which is exactly the D0-b kernel, with no quantization of a continuous feature.
3. **No reuse of `Model`/`ValueState`.** The historical path is `pub(super)` over the old word
   artifact; the review forbids implying compatibility. Only its invariants are reused.
4. **Ring capacity 128 with a ≤24-candidate newest-first bound** rather than a postings index. At
   this size a bounded scan is cheaper to implement and measure, and the prompt says not to
   pre-optimise an unmeasured routing bottleneck. Evictions are counted, not hidden.
