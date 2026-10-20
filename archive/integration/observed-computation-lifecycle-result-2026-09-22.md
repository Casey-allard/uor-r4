# Observed computation lifecycle — returned result

**Principal qualification:** [independent review](observed-computation-lifecycle-review-2026-09-22.md) retains the central result and corrects the account below:72/72 primary ingests;32-case first evaluation in attempt2 and exposed replay in4; independent audit replaces counter-only reconstruction; final grounding/consumption is atomic and child-reload store was typed constructed. Reported unrelated-test attribution remains unverified. The text below preserves the submitted account, not current scheduling or unqualified acceptance.

September 22 UTC, 2026. Continuation of the [principal review of PR #1347](consumed-geometric-state-review-2026-09-22.md)
on base `323040a4` (reviewed content `323040a43cc7169c8aff08f758b756d3bba4a0f8`). Source
`de5991fd10fa9c50f7d3e95a3d94299fc3af45fe`. Sealed root
`.uor-models/realtext-prior-2026-09-20/observed-computation-lifecycle-4`, 19 files sealed, 0 unlisted.
Executed receipt [evidence](../evidence/observed-computation-lifecycle-2026-09-22.json) with its
[independent reconstruction](../evidence/observed-computation-lifecycle-2026-09-22.py). Active brief:
[one learned observed-memory/control/computation artifact](deepseek-observed-computation-lifecycle-step-2026-09-22.md).

Bound identities: `scoped_memory.rs` `a4925b95…` and `grounded_session.rs` `cd0cb162…` are **unchanged**
from the reviewed merge; `competitive-reader.rs` `b2245a4d…`; executable `435cde43…`.

## What was asked, and the diagnosis

The brief asked for **one** learned observed-text artifact that composes ordinary scoped memory with the
retained consumed geometric computation: it must decide, from the request and actually retrieved
content, whether to follow another source, begin the requested computation, consume its result,
continue to a later source or emit the complete answer — preserving correction, temporal intent,
independent scope/relation and restart.

The reviewed baseline localized the gap concretely: the submitted candidate kept 56/56 development and
24/24 exposed computation results but only **17/38** prior primary outcomes (13/34 language questions
plus 4/4 API queries), with 22/37 ingestion checks and four interpretation errors. Seven corrections
were classified as computation and suppressed, project statements aliased to the office relation, a
declared nonasserting input wrote an asserting redirect, and one value boundary dropped a leading byte.

Reading the harness showed why: `cgs_development_supervision` fitted the learned binder and intent
tables on the **computation forms alone** — `AskOffice`, `AssertOffice`, `Compute`, `RedirectOffice`.
`STMT_CORRECT`, `STMT_NONASSERTING`, the project relation, the redirect relation in scoped syntax and the
`previous`/`initial` question intents had **no learned support at all**. This is omitted supervision,
not arithmetic: the retained computation artifact already worked. Fitting longer on the same support
cannot create a decision boundary the features never observe.

## What was built

1. **One combined observation/intent bundle.** `combined_development_supervision` fits a single binder
   and a single intent table over the scoped-language forms (`Assert`/`Correct`/`Negate`/`Redirect` ×
   office/project and the six question forms) *and* the computation forms, on one shared tokenizer and
   one exact lexical-identity policy. The previous narrower fit is retained in the same run as a named
   ablation on identical inputs, so combined support is a measured change rather than an assertion.
2. **Primary worlds built by the candidate's own ingest path.** `cgs_ingest_world` constructs each
   serving world by passing its actual observed assertion text through `observe` + `ingest`, retaining
   raw text, alignment, learned relation/intent/role, continuation and the committed record as
   receipts. `cgs_memory` is retained as the named **typed-record comparator** arm on the identical
   request set, so an observed-ingest failure is separable from a computation failure.
3. **Redirect-to-operand routing.** `CgsWorld` gains a declared person-level redirect, so a requested
   operation can only reach its operand after following an observed redirect. The independent reference
   walk now follows redirects *before* applying and counts the derived read the same way the session
   does; the request alone cannot predict the read count.
4. **A genuinely fresh withheld population** (`fresh_withheld`): disjoint people and destinations,
   three-operation orders absent from every earlier population, and the redirect-before-operand
   routing depth. Evaluated only after the design was selected.
5. **A cheap actual-loaded multi-turn check** (`cgs_mixed_session`), run before the preservation and
   final campaigns: it ingests observed assertions, a redirect, a declared nonasserting input and a
   correction; answers ordinary and historical questions; follows an observed redirect to a computation
   operand; retains a pinned in-flight answer across a later correction while a fresh question sees the
   new value; and continues identically after a real save/reload.

The request path, the exact store, the version authority, the `NoWrite` behaviour, the snapshot/restore
validation and the reviewed provenance invariants are unchanged.

## Measured behaviour

| Panel | Complete requests |
| --- | ---: |
| Development (ordinary + computed, two scopes) | **56 / 56** |
| Exposed composition (previous post-selection panel) | **24 / 24** |
| **Fresh withheld** (disjoint strings, new orders, redirect-to-operand) | **32 / 32** |

**The same artifact now retains the earlier lifecycle.**

| Prior primary panel (raw scripts, learned path) | Matched | Ingestion | Errors |
| --- | ---: | ---: | ---: |
| Combined candidate | **38 / 38** | 37 / 37 | 0 |
| Migrated genuine prior v1 parameters (unchanged) | 38 / 38 | 37 / 37 | 0 |
| **Ablation: computation forms only** (the submitted design) | **17 / 38** | 22 / 37 | 4 |

The ablation reproduces the reviewed baseline exactly, which is the causal attribution: the only
difference is whether the shared observation/intent support covers both form families. Combined fit
receipt: binder **132/132** exact joint spans+cue-roles on 132 clauses, intent **92/92** statement and
**40/40** question. The ablation's binder is 48/48 on its 48 clauses with only roles 0 and 1 observed.

Learned ingest: **48/48** observed world assertions committed with the declared entity, value, relation
and continuation, and **0** row disagreements against the typed-record comparator on every evaluated
request.

**Saved-state continuation.** Ten checkpoint phases (`Read`, three `Apply`, `Read`, three `Emit`,
two `Stop`) are restored in a **separate process** from the persisted store and raw request text, and
every phase reproduces the complete final frame — covering before the computation starts, during a
nonempty operation sequence, after grounding and before consumption, and after the later source capture.
An in-process restored runtime mid-operation-sequence also finishes identically, and a disk reload
reproduces the same answer.

**Mixed session, 7/7 checks:** 12/12 ingest, 6/6 ordinary and historical questions, 2/2 computations,
3/3 post-correction questions, 3/3 after reload; the pinned in-flight answer stayed `Quarry` across a
correction whose fresh answers read `Harbor`; the question-offered-as-input committed nothing.

Controls on identical inputs (40 requests each): **NoRead 0/40**, **ApplyDisabled 12/40**,
**ConsumeDisabled 12/40**, **Unscoped 40/40** (pre-existing: this panel asks one scope per world, so the
scope collapse is not observable here), **folded central sign 24/40**, **fitted shared-transition model
26/40**, **directly tabulated finite control 40/40**. Order sensitivity and the identity negative
control are unchanged: `i j` → key `Alma` / answer `Bramble`; `j i` → key `Bert` / answer `Quarry`; the
central-sign fold gives `Bert`/`Quarry` for both orders; identity leaves the retained state and derived
key equal to the operand.

## Geometric contribution actually measured

**Competence and reuse; a tie, not superiority.** The directly tabulated finite control — one shared
permutation per observed primitive — ties the signed exact-factorization arm at 40/40 on this path. The
central-sign projection still loses an answer-relevant order distinction, which is the concrete role
retained signed state plays. The project's fitted shared-transition model remains a weaker comparator
here at 26/40 because its fit objective includes a decoder and stop policy this path does not use.

**No geometric code search was re-run.** The retained H4 descriptor objective was already at an
exact-hit ceiling and a repeat sweep is uninformative; the brief says so explicitly. The binder used
here is the retained **ordered categorical** arm (position and bigram features give it order
information), and it reaches 132/132 exact joint hypotheses on the combined support — so there is no
headroom for a code search to demonstrate anything on this objective. Geometry is retained where it is
load-bearing: the signed Q8 state whose projection demonstrably loses the distinction. The active H4
artifact is unchanged and untouched. Hopf fiber, paired-H4/icosian E8/S7 and harmonic mechanisms remain
conditional tools for a witnessed need and are **not** claimed to help here.

## Deviations, with falsifiers

* **Combined supervision instead of a new objective or decoder.** *Falsifier:* if the complication-only
  ablation had also restored the prior lifecycle, combined support would not be the cause. It did not
  (17/38 against 38/38). The ablation is retained in the run.
* **Person-level redirect added to the declared world.** This is a new *routing composition*, not new
  surface syntax: the clause forms are the familiar declared ones. *Falsifier:* if the runtime had
  followed redirects differently from the declared reference walk, the fresh panel's read counts would
  disagree; they agree 32/32.
* **The mixed-session read-count oracle was first written one read too low** and failed 3 rows while the
  *answers* were already correct. This was an error in my independent expectation, not in the model; the
  corrected oracle is used and the failing attempt is preserved as `observed-computation-lifecycle-2`.
* **Primary worlds are now built by learned ingest**, so the two constructions can disagree. *Falsifier:*
  any row where the learned and typed arms differ. Zero disagreements were measured; the comparator arm
  is retained so a future disagreement localizes immediately.
* **The fresh population reuses the label and operation vocabularies.** Declared, not hidden: novelty is
  lexical (people/destinations), operation-order and routing-depth, all inside the declared forms. A
  genuinely new vocabulary would be a different claim and is not made.

## Limitations

Eight declared state labels, three observed primitives, four people per world; one deterministic
process-local store. The prior primary panel is a **retained development panel**, not a fresh acceptance
set; its 38/38 demonstrates that the new combined artifact retains the earlier competence, not that it
generalizes beyond it. The learned lexicon is fitted from supervised pairs, not unlabeled text. The
`Unscoped` control does not discriminate on this panel (single-scope worlds) and is reported as
measured rather than as a pass. Energy is `UNAVAILABLE`; whole-path D0-b is unqualified; the historical
hash/metadata path is not a bounded low-allocation serving kernel. Broad prose, general reasoning and
frontier capability remain unestablished. Four library tests in untouched modules fail
(`geometric_attention`, `lowbit_attention`); no library source was modified by this change (empty diff
under `native_geometric/`), so they are pre-existing and unrelated.

## Validation actually executed

`cargo fmt -p uor-r4-core --check` clean; release build; **27** `scoped_memory` module tests, **23**
`observed_text_session` module tests and **25** runner tests pass with one legacy artifact-dependent
test explicitly ignored, matching the reviewed baseline; actual loaded execution of the sealed attempt
including the separate-process raw-text restart. The evidence script recomputes panel, control and
preservation counts from `rows.jsonl` and reports zero discrepancies against the harness summary.

## Next architectural decision

The combined lifecycle now holds one artifact that observes, remembers, corrects, answers historically,
follows redirects and consumes a geometric derived value. The next step is the retained E/S donor as an
explicit **local emission option** through this same session — source-separated text and complete
interactive answers, with equal recent tails and changed older evidence, and the earlier over-strong
copy boosts kept out. Q8 consumption is retained rather than extended. Durable scopes, executed Rust
generation and matched-cost laptop qualification follow on the same path.
