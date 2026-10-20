# Transferable lexical learner: corrected observation, emitted-token feedback, post-copy continuation

September 22, 2026. **Principal qualification:** [the September 23 review](transferable-lexical-principal-review-2026-09-23.md) retains this as an exposed V3 interface result. Its 4/4 pair changes operation and `key_changed` alongside payload, so it does not isolate learned content use. All held-out cases are non-computed familiar forms; temporal wording in computed fixtures follows a route-key change without a committed mutation. The submitted E/S probe generated with E while S was only loaded; the principal replay exercises S separately. The original sealed root remains preserved.

Base: protected PR #1351 merge `6ae7625ffa11e86eeb176e45e22d6fef609b8ac6`. This
submission executes the retained observed-memory/computation session through a corrected
state-conditioned lexical contract (artifact version 3) under the existing `StateLexicalV1`
serving contract, and reports the loaded E/S reference assessment and the ordinary-text remainder.

## The missing distinction

The retained version-2 content observation collapsed three causally different facts into one lossy,
permutation-invariant bag:

* **what** the selected payload and the consumed operand actually are — the first four *fitted* tokens
  were summed, so order and any suffix beyond four tokens disappeared and an unfitted token contributed
  nothing;
* **whether** the selected value equals the operand — the only witness was a count of differing
  coordinates of that lossy bag;
* **whether a mutation was actually committed** — the label compared derived and operand *addresses*
  while the decoder observed *payloads*, and the fixtures made those coincide.

Two observationally identical but causally different worlds therefore received identical features, and
no amount of fitting could separate them (e.g. `(unknown_A, unknown_A)` and `(unknown_A, unknown_B)`;
a reordered payload; a changed suffix past the truncation window).

## Competing hypotheses and the decisive test

* **H1 — more fitting / more slots.** Rejected: the collapse is information-theoretic, not capacity.
* **H2 — a larger lossy embedding.** Helpful for order/suffix, but cannot represent *exact* equality
  of arbitrary unfamiliar strings.
* **H3 — typed causal observation + token-identity feedback** (adopted). Expose the exact,
  causally-available distinctions as declared integer observations, and let the actual emitted token
  identity advance the recurrence. Exact byte/key comparison is input infrastructure, not a gold
  response class.
* **H4 — signed ordered transport (`q⁻¹k`).** A structured alternative for order/role; common-left
  invariance erases the central sign, so it is a component, not a complete answer. Not adopted here.

**Decisive test.** (a) Measure the collapse on the loaded candidate's own observation (the runner's
`representation_diagnostics`); (b) require the corrected observation to separate the same pairs;
(c) require the learned artifact to produce distinct, correct continuations on unfamiliar source
documents and to continue *after* the copied span.

## What changed (artifact version 3)

`crates/uor-r4-core/src/native_geometric/learner/state_lexical.rs`, reusing the retained
`TernaryLinear` recurrence and the bounded integer serving path:

1. **Position-resolved content.** A token's learned row is indexed by `(token, position)`, so a
   reordered payload produces a different feature. Position `SL_MAX_CONTENT_TOKENS` is a dedicated
   **tail** slot carrying the final token of a truncated payload, so a long payload's suffix survives.
2. **Explicit unknown.** Row 0 is reserved for unfitted tokens, so unsupported content is a real
   learned row rather than a meaningful zero. Unfitted tokens are declared (`oov_tokens`) at fit time
   and excluded from the content index, so an unknown-content control is faithful.
3. **Exact typed causal block (15 coordinates).** Requested-history one-hot; `derived`;
   `prior_differs`; `committed` (a store mutation was actually committed for the address, distinct
   from a different value); `key_changed` (the derived query key differs exactly from the operand
   key); and the exact selected/operand identity relations: full equality, bounded common prefix,
   bounded exact divergence, both lengths, and both unknown counts.
4. **Emitted-token identity in the recurrence.** `h_{t+1}` consumes the action symbol, the **actual
   emitted token's** shared learned row, and the typed block. A copied token is no longer collapsed
   into one `Copy` symbol; an unfitted copied token shares the reserved row (a declared residual
   collision).
5. **Post-copy continuation.** The authored changed-derived-key form places a word *after*
   the owned span, so the fit exercises a vocabulary decision that follows the copies. The word
   labels a route change in these fixtures, not an independently verified committed mutation.

`crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs` computes the typed facts from the
executed session (`state_lexical_facts`, exposed read-only as `observed_lexical_facts`) and passes the
emitted token into `transition`. The retained version-2 artifact bytes and results are preserved; the
v2 code path is superseded by the version-3 artifact gate.

## Executed results (root `state-lexical-v3-10`, reused sealed artifact from `state-lexical-v3-8`)

* **Learned state-lexical 31/31**, all terminals `Complete`: fitting documents 23/23 and
  **held-out source documents 8/8**.
* **Teacher-forced agreement 791/791** through the exported integer artifact (122 declared sequences,
  791 steps; served loss `3.2e-13`).
* **Retained legacy copy contract 31/31** on the same panel.
* **Retained comparator** (finite realization table fitted on the same declared text with the runtime
  causal context) **20/31**.
* **Ablations change output:** recurrence disabled 0/31, context disabled 6/31 (the learned words are
  not the flag-only sequence).
* **Paired computed requests:** each fitting document's unchanged-key and changed-key operations
  produce different output and final recurrent state (4/4). The operation, payload and supplied
  `key_changed` fact vary together; this does not isolate a payload-sensitive word choice.
* **Restart:** 10 in-process boundaries restore exactly, and the separate-process child continuation
  matches (`restart_all_boundaries`, `restart_child_process`).
* **Representation diagnostics on the loaded candidate:** reordered known tokens alias **false**;
  suffix after four tokens aliases **false**; the full observation distinguishes equal-and-unequal
  unknown pairs (`false`), while the content feature alone still collides (`true`) — the exact block
  carries the distinction.
* **Submitted E/S reference probe:** `empirical.cpl2` (454,788 B) and
  `separable_older_query_read.cpx3` (53,555 B) load at pinned SHAs. E generates four tokens
  (`load_and_probe_ms = 9`); S was deserialized and validated but did not make a query-read
  decision in this submitted root. The principal replay exercises S separately. Neither donor
  emits or owns the session's exact copied span; this is no matched language-quality comparison.
* **Ordinary-text exposure (source-separated local project documentation, distinct from the authored
  state world):** the final submitted evidence records fitted content coverage **4,816/227,777**
  tokens (2.1%) and **307/16,406** (1.9%). These are measurements of the then-current source files;
  earlier draft counts were from an earlier file snapshot.
  **NOT_RUN as a learning task:** the scoped contract supervises an exactly owned span, not open prose
  continuation, and the fitted content index does not cover ordinary vocabulary.

All eight declared checks pass; the runner seals an exclusive root and the reuse path verifies the
sealed parent before replay.

## Limits and claim boundaries

Authored response fixture over a small declared state world. The sentences, the vocabulary and the
computed-op probes are declared; this is **not general prose** and no geometric advantage is claimed.
`key_changed` and `committed` are exact causal observations, but a derived-key change is not a
temporal mutation, and `committed` here reports a previous assertion at the captured address. The
sentence family is authored; it is not independently validated truthful prose. Energy is **UNAVAILABLE**
without measurement. The dense ternary maps are executed with bounded additive/shift/table operations
under D0-b; quantization does not make them sparse or geometric, and their parameter traffic is not a
geometric claim. Unfitted copied tokens share the reserved OOV row. Ordinary-text learning through
this interface is the open remainder.

## Next step

The first item remains ordinary-text learning through the retained session: a declared prose corpus
split by source, with a continuation objective over the same low-bit state, so the content index and
the vocabulary decisions are learned from prose rather than from an authored span. E/S adaptation
requires a donor-conditioned vocabulary decision over an exactly owned span — a new interface, not a
reuse. Broader conversation and executed Rust follow through the same candidate once useful lexical
continuation exists.
