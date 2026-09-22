# Truthful state-conditioned lexical realization — submitted executed result, qualified by principal review

**PR #1351, principal review:** retain a learned seven-slot ternary recurrent realizer as an exposed component. The submitted 31/31 matches authored targets (23 fitting cases, eight unfamiliar-value cases); every computed contrast is fitted, and all targets put vocabulary before copying. Feedback distinguishes Insert slots but merges all copied tokens into one Copy symbol. The submitted finite reference 0/31 is invalid as a comparison because its training emission count was fixed at zero. Computed `became`/`still` compares route changes on self-valued fixtures; it does not establish truthful temporal mutation. [Principal review](state-conditioned-lexical-review-2026-09-22.md), [independent audit](../evidence/state-lexical-principal-audit-2026-09-22.json) and [corrected checks](../evidence/state-lexical-principal-checks-2026-09-22.json) own the corrected scope. Original artifacts remain preserved.

The remaining sections preserve the submitted interpretation for comparison. In particular, “actual emitted-token feedback,” “truthful” computation language, “source-separated text,” and the cause assigned to finite-table 0/31 are superseded by the principal review. No historical learned artifact or sealed result is rewritten. The current next action is the transferable lexical brief, not immediate executed-Rust expansion.

September 22, 2026. Supersedes the exposed four-word construction in the [PR #1349 submission](grounded-lexical-realization-review-2026-09-22.md) at the exact scope below. Base merge `6295510d20509c2af604781fe374ab0beb477665`; isolated full worktree; owner checkout preserved. Sealed root `.uor-models/realtext-prior-2026-09-20/state-lexical-1`; [evidence summary](../evidence/state-lexical-2026-09-22.json).

## What was built

One learned, state-conditioned lexical decoder served through the **retained scoped session**. The
emission contract is the responsibility split in the [execution brief](deepseek-state-conditioned-lexical-step-2026-09-22.md):

```text
c       = causal pinned request + selected evidence/derived result + provenance
h_0     = learned_init(c)
a_t     = learned_readout(h_t, c, exact_copy_state)
x_t     = execute(a_t)
h_(t+1) = learned_transition(h_t, x_t, c)
```

* `c` is a **content** feature: a learned embedding of the selected owned payload and, for a consumed
  computation, of the operand the computation started from. The selected and operand contributions
  occupy separate halves, and the input also carries their per-coordinate difference and disagreement
  count, so "the computation left the same office in force" is *represented* rather than left to a
  summed-embedding accident. It is not a supplied `changed` flag.
* `h_t` is a bounded integer state vector. `learned_transition` consumes the symbol the session
  **actually emitted**, so two equal-length prefixes over different symbols reach different states.
  `Copy` enters as one symbol; the copied byte identity stays owned by the session and is never
  replaced by lossy state.

The decoder is a new module `crates/uor-r4-core/src/native_geometric/learner/state_lexical.rs`,
bound through a new prospectively versioned `OutputContract::StateLexicalV1`. `RealizedV1` and
`LegacyWords` are retained and loadable; the earlier finite table was not rewritten.

## Truthful temporal semantics

The declared response language (authored, then learned from text) ties tense to the **requested
history** and the change wording to the **computation's actual effect**:

| Meaning | Declared response | Justification |
|---|---|---|
| current value, not computed | `it is <value>` | a plain current request asks for the current value |
| previous assertion / previous distinct | `it was <value>` | the request asks for a historical position |
| initial | `at first it was <value>` | an explicit historical request, four uncopied words |
| consumed computation left the same office | `it is still <value>` | the operation did not change which office applies |
| consumed computation changed the office | `it became <value>` | the operation changed which office applies |

A predecessor alone no longer makes the current value historical: a superseded current value is
`it is <value>`, and past tense follows the request. Computation provenance is not used as elapsed
time.

## Executed measurement

Fitting is offline and floating point: teacher forcing with Adam and quantisation-aware
straight-through estimators, then exported to a ternary (`<=` 2-bit, power-of-two per-row scale)
integer artifact with 4-bit table-read embeddings. Served execution is table reads, integer
add/subtract, shifts, compares and an argmax — no multiplier and no floating point.

| Panel | Result |
|---|---|
| Learned decoder, all cases | **31/31** complete and byte-correct |
| Learned decoder, fitting documents | **23/23** |
| Learned decoder, **held-out documents** | **8/8** |
| Retained `LegacyWords` copy contract | **31/31** byte-for-byte |
| Retained finite table (`RealizedV1`) on the same text | **0/31** |
| Context ablation (`RealizationContextDisabled`) | **7/31** |
| Recurrence ablation (`RecurrenceDisabled`) | **0/31** |
| Teacher-forced agreement through the exported integer artifact | **322/322** steps |

* **Actual emitted-token feedback.** With the recurrence held at its initial state the decoder emits
  `it it it it it it <value>` — the exposed clipped-count aliasing, reproduced under the corrected
  contract — and no case is completed (0/31). Lifting the ablation is what makes a variable-length
  prelude possible; the four-word `at first it was` continuation is the distinguisher from the old
  clipped counter. Every terminal is `Complete` with the table's own `Stop`; no response is ended by
  the bound override.
* **Selected evidence and result content.** With `derived`, `history` and `prior_differs` held equal,
  changing only the consumed computation changes an uncopied word in **4/4** fitting documents:
  `it is still Bert` versus `it became Cora`, and likewise for Ivo, Cedar and Oren. Provenance alone
  cannot receive credit for that difference.
* **Source-separated text.** Four fitting documents and three held-out source documents share no
  entity, and the held-out documents use value strings absent from fitting. The flag-driven sequence
  and the temporal wording transfer (8/8); the change/`still` distinction relies on fitted content
  embeddings and is not claimed to transfer to wholly unfamiliar computed values.
* **Owned continuation and restart.** Ten emission boundaries in process and ten saved frames resumed
  in a **separate process** each reach the identical complete answer. Exact owned copying, memory and
  consumed computation are unchanged, and the new state is carried in the frame.

## Controls and preserved evidence

* The retained **finite table** (`RealizedV1`) fitted on the same declared text scores 0/31: its key
  omits the emitted symbols and the evidence content, so it cannot produce the four-word prelude or
  the value-sensitive wording. That is a comparator result at this scope, not a claim about all
  finite tables.
* `RealizationContextDisabled` zeroes the content and flags and changes the output (7/31).
* `RecurrenceDisabled` holds the state and changes the output (0/31).
* The original PR #1349 artifacts, its incorrect targets and the retained `RealizedV1` runtime path
  are preserved; this delivery does not relabel them.

## Boundaries

* Authored development scope over a small declared state world. **Not** general prose, conversation or
  reasoning, and **no geometric advantage is claimed**. The decoder is a bounded low-bit recurrence;
  geometry remains the preferred architecture and is not load-bearing here.
* The E/S local vocabulary donor was not loaded as a comparator; the executed comparators are the
  retained finite table and the copy-only contract.
* Whole-path multiplier-free D0-b was not measured end to end. The new served decoder executes no
  multiplier and no floating point, and the artifact declares ternary maps and 4-bit embeddings.
* Energy is **UNAVAILABLE**; there was no paid or external compute.

## Recommended next step

The lexical responsibility split now works at the exposed scope: evidence and consumed computation
change learned uncopied words, the emitted symbols drive the continuation, and termination is
learned. The remaining bottleneck is **content generalization**: the change/`still` distinction is a
learned association over fitted content embeddings, so it does not transfer to unfamiliar computed
values, and the declared vocabulary is seven words.

Next: train and evaluate the same decoder on a **declared prose/conversation corpus** with
source-separated splits and a much larger learned vocabulary, and carry a computed result into an
executed Rust generation through the same native path. Geometry should be reintroduced only where a
witnessed aliasing, interference or cost problem needs it, measured against an equal-information
ordinary mechanism.
