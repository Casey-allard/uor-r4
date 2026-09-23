# One shared Generate/Copy/Stop path: ordinary-text continuation and grounded exact copying

September 23, 2026. Base: protected PR #1352 merge `44c2ccd27cea5d0178f19cca2f33332db4a95ce1`.
This step executes the [next DeepSeek brief](deepseek-ordinary-lexical-step-2026-09-23.md) that the
[principal V3 review](transferable-lexical-principal-review-2026-09-23.md) set for
[PR #1352](transferable-lexical-result-2026-09-22.md).

## The responsibility that was missing

PR #1352 retained an exact typed observation and emitted-token feedback, but the served contract
supervised an **exactly owned span** over eight authored insert words, its computed answers were
fitted, and ordinary-text learning was **NOT_RUN** with roughly 2% prose-token coverage. There was no
single learned path that could both continue ordinary text and answer a grounded request with an
exact owned span, and no evidence that any post-copy vocabulary choice depended on the token actually
copied rather than on a supplied route flag.

## What was built

One shared served model, `learner::transferable_lexical`, implements the briefed responsibility:

```text
c        = pinned request meaning + selected evidence/result + exact provenance
h_0      = learned_init(c)
a_t      = learned Generate(token) / Copy(owned occurrence) / Stop
x_t      = token actually emitted by a_t
h_(t+1)  = learned_update(h_t, representation(x_t), event, c)
```

* **Action space.** The readout has `vocab + 2` rows: `Generate(v)` over the whole pinned 4096-token
  vocabulary, then `Copy`, then `Stop`. The generation vocabulary is the declared tokenizer
  vocabulary, so the head is an ordinary vocabulary head rather than eight authored words.
* **Legal support.** `Copy` is removed from the scored set whenever the session reports no live owned
  occurrence, and cross-entropy is normalised over the legal rows only. This is a declared hard
  constraint, not a learned preference, and it is labelled as such.
* **Exact ownership.** The owned span, version authority and the copy **cursor** stay outside the
  compressed state; the model only chooses the action, and a copied token is copied exactly.
* **Shared representation.** One signed 4-bit embedding table serves both roles: a token's row is the
  recurrent feedback `representation(x_t)`, and the same rows are summed into a bounded evidence
  fingerprint `m` with a **fixed cyclic position rotation** per content position, so a reordered span
  changes `m` with no position-indexed parameter table and a truncated span keeps its final token in
  a dedicated tail slot. The typed causal block is the retained exact 15-coordinate observation.
* **Serving contract (D0-b).** Ternary weights with a power-of-two per-row shift, 4-bit embeddings
  with a power-of-two per-row shift, `clamp(emb + (W_h·h >> k) + W_f·u + b)`, and one artifact-bound
  dyadic score scale `Z · 2^-score_shift`. Every served operation is an add, a subtract, a shift, a
  compare or a table read: no multiply, no divide, no float in the declared numerical kernel.

`crates/uor-r4-core/src/bin/ordinary-lexical.rs` trains and executes that one model on two
source-separated populations and seals an exclusive report root.

## Preflight A — temporal meaning authored from typed state

The authored oracle is computed from typed source state and never from the learner's renderer: a value
was temporal-updated exactly when a mutation was **committed** at the answered address **and** an
older committed value at that address was **superseded**. The world crosses four typed regimes over
four declared single-token values, and the regimes are **identity-balanced**: every value appears in
the temporal and the non-temporal regimes, so no token identity is a shortcut for the answer.

* Regime 1 — changed derived address, **equal** value: non-temporal. Its `key_changed` is true.
* Regime 2 — same address, **different** value, **no commit**: non-temporal.
* Regime 3 — plain read, no commit: non-temporal.
* Regime 4 — **committed** supersession at the answered address: temporal.

**Executed: 32/32 authored regimes exact on the loaded artifact**, 88/88 teacher-forced actions, with
the four regimes' typed blocks pairwise distinct and identical *within* a regime. The retained
route-key marker (`changed address → now`) is **contradicted on 16 of the 32 cases**: it fires on
regime 1, which is not a temporal change, and misses regime 4, which is. That is the falsification the
principal asked for, and it is realised on the served artifact rather than asserted.

**Prospective oracle revision, disclosed.** An earlier oracle additionally required
`!key_changed`. A held-out composition with a superseded older value at the answered address reached
through a changed derived key failed it: `prior_differs` is already a fact about the answered
address, so a committed supersession there is a real change of that entity whichever route reached
it. The conjunct was over-specified, was revised after that exposure, and the four typed regimes are
unchanged by the revision. `preflight_a.authored_rule_revision` in the sealed receipt records this.

## Preflight B — the post-copy vocabulary decision depends on copied identity

Two grounded cases hold the request, the typed facts, the copy length, the events and every
nonsemantic prefix fixed; only the copied token identity differs, and the accepted post-copy word is
authored from it. The post-copy words are **declared class labels, not semantic claims**: the
preflight isolates causal dependence on copied identity and nothing more.

**Executed on the loaded artifact: 2/2 actual arms exact; the identity-erased arms alias (identical
post-copy decision for both cases); the source-disabled arm loses the exact copy.** Erasing the
copied token's identity from both the evidence fingerprint and the recurrent feedback destroys the
distinction, so the decision genuinely used the token that was emitted. This is the property V3
lacked: under V3, equal-length unfamiliar copies with the same typed facts and action symbols reached
an identical state.

## Ordinary-text learning through the same artifact

Source-separated, pinned real text: 429 project Markdown documents, exact-duplicate grouped
(0 duplicates), split by content hash into 355 fit / 38 tune / 36 development documents, tokenized by
the pinned derived 4096-token BPE. 6,000 fit windows (334,796 scored targets) and 24 development
documents (96 windows, 5,376 scored targets) stratified by document length. Configuration, seed,
selection rule and exposure are pinned in the receipt.

**Executed (2000 steps, one artifact, 471 s fitting):**

| Arm (same tokenizer, same 5,376 development targets) | bits/target |
|---|---:|
| cold start (declared extended-alphabet marginal) | 9.0830 |
| exact fit-only unigram | 9.0827 |
| **shared Generate/Copy/Stop model (delivered artifact)** | **7.3722** |
| loaded donor prior E (disclosed exposure on the pinned corpus) | 6.8701 |
| tuned interpolated (prev,cur) count reference on the tune split | **5.1217** |

Paired document-cluster bootstrap intervals (2,000 draws):

* shared model − unigram: **+1.7105 bits/target** [+1.4973, +1.9359] — excludes zero, improving.
* shared model − count reference: **−2.2505** [−2.4454, −2.0734] — excludes zero, **the model is
  worse than the count reference**.
* shared model − donor E: **−0.5021** [−0.5777, −0.4234] — excludes zero, **worse than E**.

Document-level variation is wide: per-document bits range 5.64 to 8.71, median 7.37.

**This is the honest negative in this step.** On this small, highly repetitive pinned corpus a tuned
interpolated `(prev, cur)` count model is the strongest reference by a wide margin, and the shared
low-bit recurrent path does not reach it, nor the retained E prior. Greedy generation is
**degenerate**: all three loaded continuations collapse into a repeated token after a locally
plausible prefix. The count reference degrades similarly on the same prompts. Sampling is NOT_RUN.

## Held-out source/relation composition

Three compositions combine typed dimensions and requested views that **no training case presents**
(history views 1 and 3 are absent from training, which uses histories 0 and 2). **Executed: 3/3
exact** — a committed supersession under a previous view emits the temporal word; a committed
supersession reached through a changed derived key emits it; an observed value difference with **no
commit** under a previous view stays plain. The first draft of this panel exposed the oracle defect
above, which is why the revision is documented rather than silently absorbed.

## Mixed loaded session and separate-process restore

One loaded artifact executes five phases: observe ordinary text and continue vocabulary; ask a
current question and copy the owned span; a committed correction; continue vocabulary **after** the
copied span; and the pinned control in which a pinned answer retains its causal view while a new
answer sees the correction. 32 served decisions, 5 saved phases. A **separate process** reloaded the
artifact from the sealed root and reproduced all 36 authored cases exactly (`REPLAY_OK 36 cases`),
including the served action list, the emitted tokens and the final state digest of every case.

## Whole-path cost on the named M1 machine

* served latency **673.6 µs/token** (1,484 token/s) over 1,280 served tokens from the loaded artifact;
* artifact **466,711 B**; total table **466,622 B**; **56,457** nonzero weight reads per served step
  (dense ternary maps, honestly dense — ternary does not make them sparse or geometric);
* peak RSS **98,435,072 B (93.9 MiB)** measured externally (`/usr/bin/time -l`);
* declared structural heap allocations per served token **8** — *not* counter-measured, because a
  counting global allocator needs `unsafe` and this binary is `forbid(unsafe_code)`;
* quantised 4-bit embedding collisions: **0 pairs over 4,097 rows**, so the delivered embedding is
  injective on this fit and no held-out decision is exposed to an embedding alias;
* **energy UNAVAILABLE** (not measured). Whole-path D0-b for a product is not claimed.

## Local validation

11 focused `transferable_lexical` tests pass, including: the served integer kernel equal to its exact
float reference for every map; `Copy` illegal without a live owned occurrence; cross-entropy
normalised over legal rows only; an order-sensitive evidence fingerprint; the recurrence consuming the
actually emitted token; the authored temporal oracle separating the four typed regimes; artifact
round-trip with corruption rejection; declared low-bit bounds; an exact finite-difference check of the
loss gradient on the one unquantised parameter; measured descent on a synthetic source; and the
post-copy identity preflight including its blinding control. `cargo fmt --check` passes; the release
binary builds; the sealed root verifies with 0 unlisted files; and the two independent delivery runs
(roots `ordinary-lexical-2` and `ordinary-lexical-3`) produced the **byte-identical artifact**
`sha256 fe3e8a638cd50a19741065cb00ac787b63bb59f8ad5776fd8bfcb162c37a912a`.

## Limits and claim boundaries

A local, pinned, small corpus does not establish general conversation. The grounded world is authored;
only the *ordinary-text* half is learned from real text, and it does not beat a competent count
reference. The post-copy preflight words are declared class labels, not semantic claims. The
identity-erased control is a diagnostic arm, not a serving mode. Dense additive maps are reported as
what they are; no geometric advantage is claimed and no geometric mechanism participates in this
serving path. The `score_shift`, the cold-start marginal and the phased curriculum are declared
design choices, not learned. Energy is UNAVAILABLE. Sampling, donor adaptation through a compatible
emission interface, broader conversation and executed Rust remain open.

## One evidence-supported next step

The measured obstruction is now specific: on this population the model is **worse than a tuned
`(prev,cur)` count reference by 2.25 bits/target**, its state is a 64-dimensional ternary recurrence
over an 8-token prefix, and its generation collapses. The next step should therefore attack exactly
that: give the shared path a **longer effective context with a measured role for order**, i.e. test
ordered signed transport or a shared geometric residual as the *context-mixing* operator against an
equal-information ordinary recurrent control, and report bits/target against the same tuned count
reference. A finite, local count reference is beatable only if the learned state carries information
the reference cannot see; the honest next experiment is to demonstrate or refute that.
