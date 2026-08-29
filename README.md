# R⁴ — Geometric Intelligence on Local Hardware

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)](rust-toolchain.toml)

R⁴ is an open research project building a **transformerless local AI agent**.
Its goal is to replace transformer attention, mixture-of-experts routing, and
dense learned matrix operations in the serving path with deterministic
geometric routing and lookup.

That is a very real engineering goal, not a claim that the goal has already
been reached. The long-term target is frontier-like capability on ordinary
local hardware. The project is testing whether language context, inference,
and reasoning can emerge from routes through a canonical geometric memory. The
target serving engine uses no Ollama, hosted model, or source-model weights.

> **Primary direction after the equal-manifold-budget direct-attention V3 reveal (2026-08-28):** routing, exact R4/spin
> state, least-cost selection, and multiscale hierarchy remain the geometric
> substrate, but routing is not being equated with attention. The first natural
> document-scale componentwise-Frechet placement was causally active and still
> harmful: 8.367592% versus frozen #953 at 12.221651%, with its shuffled and
> operator-permuted controls also slightly stronger. The first bounded
> `GeometricGatedDeltaRetentionR4V1` core then passed structural checks but was
> weaker than plain delta on its sealed synthetic fixture (16/28 versus 23/28
> next-token; 55/112 versus 98/112 association wins). Direct-attention V2 then
> appeared positive but is preserved as `NON_PROMOTABLE_BUDGET_MISMATCH`: its
> plain/current comparators had fewer effective degrees of freedom. The fresh,
> pre-reveal-kappa-bound 12-case V3 corrected every arm to normalized R4
> parameters. Full H4 scored 3/12, matched plain attention 12/12,
> current-token-only 6/12, and an inference-time coherent alternative-connection
> swap 10/12; that alternative was not separately trained.
> The direct learning/softmax/value path therefore works, but the current
> mixed-gauge H4 projection/connection/optimizer combination does not transfer;
> the exact H4 group action itself remains algebraically valid. The separately
> trained `ConnectionGaugeCovarianceV4` Phase-I construction/frame gate is now
> positive: H4-compatible, alternative-tangent, and fixed-frame plain arms each
> fit 16/16 with numerical/decision covariance; current-only remained 8/16; all
> 120 frames, 14,400 ordered connections, analytical gradients, live controls,
> causal reads, and byte replay passed. The independent Phase-II freeze now
> binds 12 matched pairs/24 target-free prefixes, a `12/12` aggregate balance,
> and a salted label commitment with zero validation predictions or scoring
> joins. It awaits protected merge in PR #1001 before one immutable reveal. Actual
> paired-E8 hierarchy/fiber/torsion binding and the frozen #953 corpus oracle
> follow only after that held-out seam qualifies. Softmax is a research oracle, not the serving
> design. After the geometric oracle qualifies, its
> weighting law is replaced by the positive normalized multi-resonance sieve,
> whose mode sums are factored into bounded recurrent state and finally lowered
> to H4/Q29/integer tables. See the
> [V4 connection-gauge record](docs/connection_gauge_covariance_v4_973.md), the
> [direct-attention history](docs/direct_causal_geometric_attention_973.md), the
> [resonance audit](docs/multi_resonance_attention_sieve_audit_973.md),
> [ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md) and the
> [Geometric Intelligence Programme](docs/geometric_intelligence_programme.md).

> **Retained bounded-global evidence (2026-08-28):** #973's independently frozen
> `BoundedGlobalNoncommutingExactSpinR4V2` reached
> `RETAIN_BOUNDED_GLOBAL_NONCOMMUTING_EXACT_SPIN_ATTENTION_CONTINUE_CORPUS_INDUCTION`.
> The canonical population repair witnesses exact stored-S3-to-H4
> noncommutation, distinct nonidentity left-ordered folds, central Q29 phases,
> same-address class-result reuse, and incompatible unique candidate-relative
> winners under `C^-1*G` lexicographic least cost. With equal admitted support
> and executed work, the decoded matrix was real 2/2, identity-disabled 1/2,
> class/operator-permuted 0/2, and support-reversed real 2/2; support/work
> mismatches were zero and exact period-plus-EOS termination was 6/6. The
> target preimage was loaded once only after the target-free gate. Operator,
> population-audit, target-free-census, and decoded-smoke identities are
> `blake3:1cf08604fb4a1c545984f4cab41194e0ffcf1d7551b6e438ed57b49a0066a6e9`,
> `blake3:16ebc6d36f01e4cb324d3c46fc059aca4ffea84ba467e860b55f983cd83f4a9c`,
> `blake3:c3fb3568028f924fb12971c888193cc5780111a7af14503e240f39fbeb58dd4a`,
> and `blake3:41207999bb088e3b5f186cce983951cc27c2962d34ef8046a0beae4754b44218`.
> This establishes one bounded synthetic causal global geometric-attention
> witness, not corpus induction, general semantics, reasoning, correctness, or
> product readiness. Corpus induction was this result's contemporaneous next
> step; `ConnectionGaugeCovarianceV4` has passed its construction-only Phase I
> and now has a separately frozen target-free Phase-II population awaiting
> protected merge and one-time reveal, with
> paired-E8 and corpus qualification later. #954 remains blocked. See the
> [bounded-global record](docs/bounded_global_exact_spin_attention_973.md).

> **Earlier bounded-global V1 negative (2026-08-28):** #973's independently frozen
> exact-spin contrast stopped at the target-free relation gate with
> `RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION`.
> Both same-multiset snapshot carriers had distinct canonical epochs/roots,
> four references, three exact classes, and one byte-identical same-address
> class-result reuse, while sharing one byte-identical lower artifact and equal
> admitted support/work. But `Pavel` and `helix` map to the same H4 root and
> `prism` maps to identity, so both frozen orders produce the same complete
> `-1` fold, fiber, torsion, real `helix` role, and permuted `prism` role. The
> frozen incompatible-winner premise was false. Target loads were zero and the
> decoded smoke is `NOT_RUN`. Operator and target-free census identities are
> `blake3:f6b36cdf3e6cf96e1e9a345980843ee9eaffd25f5b864d4b4ed45a30ae6f746f`
> and `blake3:6c0a9f89a29584a09d917ae427a494b53c06b76e56482f665870ae86c1cd130a`.
> This rejected one frozen global relation, not geometry generally. The V1
> result remains append-only history; V2 supplied the noncommuting repair and
> historically advanced #973 to a corpus-induction gate. Later negative results
> supersede that action; `ConnectionGaugeCovarianceV4` Phase II is frozen in
> PR #1001 and its protected merge/reveal is current. See the
> [bounded-global record](docs/bounded_global_exact_spin_attention_973.md).

> **Retained conversation-scope result (2026-08-28):** Before the global V2
> qualification, #973 retained three narrow
> mechanisms: Gate 0's exact-candidate prior-prefix copy mechanism, one
> construction-bound exact-descriptor paragraph path selector, and now
> `ConversationEntitySpinPathR4V1` at
> `RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL`.
> In the frozen conversation contrast, both held-out inputs had identical
> lexical multisets, immediately preceding/current turns, current-through-
> paragraph identities and ordered H4 states, global identity/state/snapshot,
> admitted candidates, and work. Only the older entity-to-descriptor binding
> changed. The decoded matrix was real 2/2, conversation-disabled 1/2, cross-
> turn-binding-permuted 0/2, and parsed-binding-row-reversed 2/2, with exact
> target-free and decoded replay. Neither candidate token occurred in the
> observed held-out conversations and Gate 0 abstained. The complete stored-
> spin lexicographic path was load-bearing, but this run did not separately
> qualify an H4-shell, fiber, or torsion coordinate. This is one bounded
> synthetic exact-descriptor cross-turn entity-role selector, not semantic or
> natural transfer, a geometric advantage over direct ordered binding lookup,
> a general entity/conversation model, or general conversation/global
> attention. The operator, target-free census, and decoded-smoke identities are
> respectively
> `blake3:343c961b06605f6ae9bb6160ac34a98224991715b706156349a8fd544b6dbb35`,
> `blake3:649d733a194469aa648101a873d9e2ee323266b18872ced412d1da2cc6a56635`,
> and `blake3:6930de3c07d30df4420bb68e60ea74531c8076516bcfef1c016240eddf1b9ca2`.
> The subsequent V1 bounded-global contrast failed its target-free relation
> gate; the independently frozen V2 repair later passed its bounded decoded
> contract. Later corpus placement and bounded recurrence results were negative;
> the direct reference has since run and V3 is negative. #973's
> `ConnectionGaugeCovarianceV4` construction/frame preflight is positive; its
> target-free held-out freeze is now sealed in PR #1001; protected merge and
> reveal are next. #954 remains blocked. See the
> [conversation record](docs/conversation_entity_spin_path_attention_973.md).

> **Accepted capability-first evidence (2026-08-28):** #953's frozen
> `MultiscaleCountRadiusR4V1` comparison is positive. Against #989's unchanged
> 99,362/446,342 (22.261404%) table reference, the construction-only R4 tie
> overlay scored 103,604/446,342 (23.211797%), +4,242 correct choices and
> +0.950392 percentage points. The declared-work ledger and candidate support
> matched at all teacher-forced positions.
> The fixed prompt still emitted 16 valid UTF-8 units, but geometry changed the
> bounded continuation from the date-fragment branch to
> `. It is the most important thing to do so. The first people to live`.
> Two complete executions produced byte-identical base artifacts, overlays, and
> reports; that external replay check promoted the reports' pending decision to
> the frozen positive terminal. This establishes only causal incremental value
> for the exact fixed-point R4 evidence-radius tie intervention over the frozen lexical
> table; it is not attention, semantics, correctness, reasoning, chat, or
> release evidence. See the
> [#953 evidence record](docs/source_free_table_geometric_intervention_953.md).

> **Evidence boundary before the predictive-memory reset:** the geometric storage/identity foundation, one bounded
> causal R4/S3 path selector, and reusable provider-free decode/render/append
> plumbing exist. The first #953 smoke was an exact lexical relabel of #969, so
> it did not qualify a natural grammar loop. `PrimaryThenAdjacentSpinFallbackV1`
> repaired the frozen agreement admission to exact `{still}` then `{run,runs}`
> support under equal work, but the one permitted four-arm run chose `still run`
> for both full-path prompts and `still runs` for both state-disabled prompts.
> The frozen `LocalSameObjectContextPlacementV1` preflight then reproduced 7/7
> construction prototypes with zero class collisions and zero
> padding-identity aliases, but real placement selected 0/2 intended candidates
> while the same-artifact placement-permuted and order-shuffled controls selected
> 2/2 and 1/2. Generation and replay were `NOT_RUN`; the terminal remains
> `REVISE_I1_GENERATOR_IN_PLACE`. Independent #983 then formed pure
> construction classes but transferred to 0/6 held-out decisions and closed at
> `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER` before selection. #986 then closed
> `UNAVAILABLE_FRAME_OR_POPULATION`: the pinned raw corpus reproduced, but its
> exact #986 codec/pair commitment and a complete same-frame lexical SpiralCore
> operator map were unavailable. Placement, diffusion, Gate 0, calibration,
> sealed labels, selection, and #953 were `NOT_RUN`. The later #989 table reset
> supplied a working reference, and the one matched #953 R4 tie intervention
> has now passed its held-out and decoded-output contract. #973 Gate 0 has also
> retained one exact-candidate prior-prefix copy-attention mechanism. Its
> frozen paragraph slice retained one construction-bound exact-descriptor/
> entity-binding stored-phase path selector, and its frozen conversation slice
> retained one construction-bound exact-descriptor cross-turn entity-role path
> selector, each with the narrow boundary above. The first independent bounded-
> global exact-spin relation failed target-free because its swapped states
> commute; the independently frozen V2 repair then established one bounded
> noncommuting global mechanism. Later corpus-placement and recurrent results
> were negative; `ConnectionGaugeCovarianceV4` has passed Phase I and its
> independent target-free Phase-II freeze is sealed in PR #1001. Protected
> merge and the one-time reveal are the current active rung.
> #954 remains blocked behind #973. General higher-scope attention, correct
> answers, and reasoning do not exist yet. The
> dashboard is an interactive window into the research substrate, not a
> frontier model or a ChatGPT replacement.

## Try the project

With Git and a current Rust toolchain installed:

```bash
git clone https://github.com/UOR-Foundation/uor-r4.git
cd uor-r4
cargo run --bin r4 -- demo
```

Open <http://127.0.0.1:8000>.

The dashboard lets you interact with the existing geometric router and inspect
its state. It is the quickest way to see the project in motion without
downloading or compiling a language model. A first Rust build may take longer
than five minutes on some machines; later launches reuse it.

To inspect one route from the command line instead:

```bash
cargo run --bin r4 -- route "geometry is the route"
```

To run the one fixed canonical-ingestion witness:

```bash
cargo run --bin r4 -- lexical-ingestion-witness
```

To compile and evaluate the established #989 source-free lexical table path:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --json
```

The corpus directory must also contain its pinned `manifest.json`. The command
uses only the D3 construction partition for its vocabulary and integer
unigram/bigram/trigram counts, evaluates held-out next-unit prediction, writes
the deterministic packed artifact, and emits the exact decoded continuation.
It is a statistical lexical baseline command, not an attention, semantic,
correctness, chat, or release surface.

To run the one frozen #953 comparison against that unchanged table baseline:

```bash
cargo run --bin r4 -- source-free-table \
  --corpus /path/to/articles.jsonl \
  --prompt "The United States" \
  --continuation-cap 16 \
  --artifact-out /path/to/source-free-table.bin \
  --geometric-intervention \
  --geometry-overlay-out /path/to/multiscale-count-radius-r4.bin \
  --json
```

`--geometric-intervention` enables only the frozen
`MultiscaleCountRadiusR4V1` tie-breaking overlay. Both arms retain the table's
first nonempty row, maximum-count tie set, lexical codec, decoder, and shared
declared-work ledger. The report compares held-out choices and both fixed-prompt
continuations; `--geometry-overlay-out` writes the deterministic overlay bound
to the base table artifact. The overlay is a bounded causal geometry experiment. Even a
positive comparison does not establish attention, semantics, correctness,
reasoning, chat quality, performance superiority, formal closure, or release
readiness.

To reproduce the bounded A1R associative ordered-summary decision:

```bash
cargo run --bin r4 -- associative-ordered-summary-a1r-probe
```

To reproduce the corrected A1P paired-H4-derived exact R4-heatmap
identifiability decision:

```bash
cargo run --bin r4 -- candidate-relative-identifiability-a1p-probe
```

To run the #953 decoded loop against a canonical route artifact:

```bash
cargo run --bin r4 -- bounded-geometric-generate \
  --artifact /path/to/canonical-route.json \
  --prompt "active agile athletes run" \
  --continuation-cap 2 --json
```

This research command loads no provider or source weights. It currently accepts
only a canonical artifact whose embedded construction/global input can fully
reconstruct the parent codec registry; subset-observation artifacts fail closed.
Plain output labels both the appendable continuation and typed stop reason;
`--json` emits the full deterministic witness. Trailing prompt whitespace is
also rejected fail closed so the lexical-boundary contract cannot silently
rewrite the prompt. The command is bounded to that reconstructed vocabulary and
the local #969 path; it is not `ask`, `chat`, or a correctness-qualified answer
surface.

The A1R command uses only the frozen construction/evaluation fixture and exact
finite tables. Its frozen report kappa is
`blake3:f0db7a5d5c81d51ebf3b4bf8a2715c4960ec16b14161e8bf7598d7b98c48c881`.
The associative state passed the declared scope, independent-global, fold,
incremental, and support invariants. The full arm produced distinct `ll`/`rr`
relative states on all 6 queries, but shortest Cayley distance mapped both to
energy 2 and tied every query. The terminal verdict is `RETAIN_STATE_ONLY`: it
does not generate text or establish full attention.

The A1P command preserves those six queries as regression-only evidence,
prepares construction and sealed-validation geometry/support without labels,
and derives S4 parity from each exact history and the frozen role order before
joining the separate label ledgers. Its paired contract computes
`X=C(H,c)`, `Y=C(P_c,c)`, and `D=X*Y^-1` in the signed `(1,i)` R4 chart. The
exact endpoint rule is `sin=±1, cos=0 -> 1` with chirality retained and
`sin=0, cos=±1 -> 0` with cosine polarity retained; `q0=q1=0` is typed-null
abstention, not a threshold shortcut. `q2` and `q3` remain in the full `D`
witness but are not scorer-key fields.

The target-free structural census covers 120×120 = 14,400 ordered pairs, 120
relative rows, 45 exact heatmap classes, and 480 typed-null pairs. Across 36
fixture decisions, 14 classes were exercised; construction coverage was 12/12
and pure, construction classes covered 10/12 validation decisions, the
no-class-splitting oracle ceiling was 10/12, strict construction transfer was
0/6, and eight heatmap classes were incompatible. The hard gate therefore
stops before scalar search; every downstream selection, control, and placement
row is `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS. Its terminal literal is
`RETAIN_H4_STATE_ONLY_ADVANCE_MULTICHANNEL_A1Q`. Contract, universe, and report
kappas are
`blake3:2daacf538c022fab9580d1e124af6c18d0b06da04604fbc962a01bda57f08a98`,
`blake3:dca725c0ec6060166bcd0023df956e1ff029661b5fa7800ccb9f20808712b796`,
and `blake3:5f9239150dea8c0c27c4dfa6ad2e4d0068bc3d18afc127b315c0ec358ceddb3f`.
This negative is bounded to the paired-H4-derived heatmap readout. Fixed-zeta
phases, ordered n-lets, exact `phi` radial transport, and the typed
`sqrt(2) <-> 2i <-> [0,2]` adapters remain structural under
`STRUCTURAL_BINDING_ONLY_NO_ZETA_NLET_TO_PHI_EXPONENT_RULE`; they are not
scorer inputs. It does not establish attention or generation, and #969 becomes
the next stage only after protected #970 merge. #969 has since delivered one
bounded causal path selector. #953 has driven it through real decoded-loop
plumbing and tiered admission on the frozen preflight, but the natural agreement
run made the same full-path choice for both prompts and did not qualify a
natural grammar result.

The ingestion witness maps two turns of text through the pinned lexical codec,
prime/spin route state, canonical hierarchy manifest, strict reload, and exact
lexical reconstruction. It also exercises the declared fail-closed unknown-unit
path. It loads no model and establishes reversible state plumbing only—not
attention, inference, correctness, or reasoning.

The additive serving envelope is
`uor-r4.canonical-lexical-route-manifest/1`; it transitively embeds the frozen
`uor-r4.prime-route-spin-manifest/2` bytes. Its codec identity is
`uor-r4.unicode-lexical-runs/1`: UTF-8 identity normalization, caller-declared
sentence/paragraph/turn boundaries, canonical surface-byte vocabulary order,
and rejection of unknown units before mutation. The parent keeps the complete
codec route-address registry in stable lexical-unit order; the unchanged child
manifest contains only addresses witnessed by its causal sentences. The fixed
input ceiling is 8 turns, 32 paragraphs, 31 sentences, 128 units per sentence,
512 total units, and a 64-unit content-addressed global snapshot.

Downstream code consumes `CanonicalRouteArtifact::decode_canonical`,
`attention_consumer_trace`, `attention_consumer_trace_for_cursor`,
`attention_consumer_trace_with_ordered_h4`,
`incremental_update_trace`, `incremental_cursor`,
`lookup_shared_class_trace`, `scope_ceilings`, and `reconstruct_input`. The
attention handoff is ordered current, previous, last-two, sentence, paragraph,
conversation, then bounded global; the cursor resolver returns those same seven
slots and marks not-yet-established boundaries absent. S0 serializes state and
numeric geometry only: every candidate row ceiling is zero and marked
`NOT_IMPLEMENTED_S0_STATE_ONLY`. #952 established candidate/value reachability
but found its reusable summaries order-erasing. #967 landed the exact ordered
state repair but retained it as state only after the candidate tie. #970's
corrected paired-H4-derived exact R4-heatmap gate stopped at bounded readout
identifiability without searching another scalar. #969 then qualified one local
causal path selector, and #953 implemented the first bounded decoded
library/CLI plumbing. Its relabelled smoke terminated
`REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` then
recovered exact `{still}` then `{run,runs}` primary support while consulting
and truthfully tracing adjacent-spin rows, which remained non-admitting until
the primary tier was empty. The one permitted four-arm run produced `still run`
for both full-path prompts and `still runs`
for both state-disabled prompts, with deterministic replay. The terminal
remains `REVISE_I1_GENERATOR_IN_PLACE`. The first frozen local same-object,
order-sensitive candidate-placement preflight then failed before generation or
replay: real placement selected 0/2 intended candidates while its same-artifact cyclic
placement control selected 2/2. #983's later independent construction-return
classes then transferred to 0/6 held-out decisions. #986's later local
qualification stopped before geometry because neither its exact corpus/codec
population nor a complete lexical Cl(0,6)/SpiralCore frame was available.
#953's historical H4/placement fixtures remain untouched. The later B0 reset
accepted a separate fixed-point R4 table-tie intervention and closed #953 at
its positive terminal. #973 Gate 0 has since retained one bounded prior-prefix
copy mechanism. Its frozen paragraph and conversation slices retained one
exact-descriptor/entity-binding path selector apiece at their respective
   scopes. The first bounded-global exact-spin relation failed target-free; its
   independently frozen V2 noncommuting repair then passed the bounded decoded
   contract. The first natural corpus placement later failed in PR #997, and
   the first bounded gated-delta core trailed plain delta on its sealed smoke.
   #973 now owns the direct transported Q/K/V/O oracle, multi-resonance
   replacement, and recurrent factorization while #954 stays blocked.
See the [append-only #953 record](docs/local_geometric_generation_953.md).
See the [accepted table-tie record](docs/source_free_table_geometric_intervention_953.md).
See the [#973 Gate 0 record](docs/prior_sentence_count_radius_attention_973.md).
See the [#973 paragraph record](docs/paragraph_entity_spin_path_attention_973.md).
See the [#973 conversation record](docs/conversation_entity_spin_path_attention_973.md).
See the [append-only #973 bounded-global record](docs/bounded_global_exact_spin_attention_973.md).
See the [#986 evidence record](docs/corpus_signed_transport_attention_986.md)
for the exact feasibility boundary and deliberately unrun stages.
Stored H4/Hopf/zeta/icosian and related route fields remain
structural state, diagnostics, or controls unless the owning stage qualifies a
specific term.

These commands exercise the no-model research substrate. `demo` does not start
the historical artifact-discovery server, and `route` does not claim to answer
the prompt; it exposes how the current geometry represents it.

The browser-only WASM surface is published at
[uor-foundation.github.io/uor-r4](https://uor-foundation.github.io/uor-r4/).
With `just` and `wasm-pack` installed, `just wasm-dashboard` builds and serves
the same local surface without model weights.

## What R⁴ is trying to build

The central hypothesis is simple:

> **The geometry is the route, and the data is the location.**

Text is reversibly assigned to canonical geometric addresses. As a sequence
unfolds, its route carries local and accumulated context. A bounded geometric
query evaluates possible next locations, chooses an admitted least-cost route,
and decodes that location back to text.

```text
text
  → reversible lexical address
  → prime / semiprime route
  → spin, phase, torsion, and radial state
  → current + sentence + conversation + global context
  → bounded next-route selection
  → text
```

The working design brings together:

- primes and semiprimes as addressable atoms and route experts;
- spherical harmonics as the working description of related spin states;
- fixed zeta-zero channels with changing phase and torsion;
- S³/R⁴ transport, Hopf projection, and golden-ratio radial shells;
- a paired-H4/E8 bridge for coupled geometric state; and
- recursive context at route, sentence, paragraph, conversation, and global
  scopes.

Kappa provides canonical identity and serialization. It is not itself the
tokenizer, semantic distance, attention mechanism, or language model. A pinned
lexical codec supplies reversible text boundaries; the intelligence must come
from the geometry.

## What exists now

The current foundation can represent and rebuild prime-route state, preserve
transported trajectory and overlapping context summaries, and perform bounded
deterministic candidate lookup.

It has **not** yet demonstrated:

- prompt-to-answer source-free chat;
- recursive geometric attention that generalizes beyond recall;
- a qualified natural grammatical generation loop;
- correctness and calibrated abstention;
- multi-step reasoning; or
- frontier-class capability or an energy advantage.

Earlier compiler, graph, proof, conformance, and teacher-derived systems remain
in the repository as research evidence and reusable components. They are not
the current product path and are not prerequisites for trying the dashboard.

## Current roadmap

The programme is deliberately sequential so that infrastructure and testing do
not become substitutes for working intelligence:

1. **Retain the established source-free table baseline (#989)** — 22.261404%
   held-out top-1 versus 5.413561% unigram on 446,342 known targets, exact
   bounded decoding, and byte-identical replay. Preserve its artifact and claim
   boundary as a statistical lexical reference.
2. **Retain the accepted R4 tie intervention (#953)** — 23.211797% held-out
   top-1, +4,242 correct choices over the unchanged table, a distinct bounded
   continuation, matched support and declared-work ledger, and byte-identical
   replay.
3. **Establish and compress geometric attention (#973)** — retain the literal
   causal Q/K/V/O scaffold and the now-positive V4 construction-scale
   connection-gauge covariance result. Its target-free 24-case Phase-II inputs
   and salted label commitment are now frozen in a separate public PR; protect
   that freeze, then reveal once and require the three
   trained gauges to retain held-out parity while current-only, order, value,
   and gauge-mismatch controls fail as declared. Bind actual paired-E8/fiber/
   torsion inputs and their disabled arm only after that connection result
   qualifies. Then replace softmax with the multi-resonance sieve,
   retaining the full S3 fiber/torsion state, and then factor the mode sums into
   `GeometricGatedDeltaRetentionR4V1`. Only an approximation that retains the
   frozen construction-validation effect advances toward the protected D3 join;
   exact/table lowering follows the qualified recurrent path. Do not
   scale the rejected componentwise center.
4. **Establish correctness** — relevance, contradiction handling, and honest
   abstention.
5. **Establish reasoning** — bounded multi-step route composition.
6. **Connect and ship the accepted engine** — chat integration, measured
   optimization, and only then release QA.

The CLI and WASM dashboard remain usable research surfaces throughout this
sequence so each new mechanism can become visible before the final engine is
complete.

The active dependency chain is tracked in
[#820](https://github.com/UOR-Foundation/uor-r4/issues/820). #989 established
the frozen table reference, #953 established one matched R4 tie intervention
over it, and #973 retained one bounded prior-prefix copy mechanism plus bounded
exact-descriptor/entity-binding path selectors at paragraph and conversation
scope. Its first bounded-global relation remains closed-negative history; the
independently frozen V2 repair passed its bounded contract; and PR #997 rejected
the first natural componentwise placement. A bounded gated-delta core is
structurally implemented but negative against plain delta on its sealed smoke.
Direct-attention V2 is non-promotable; its equal-manifold-budget V3 rejects the
tested mixed-gauge H4 projection/connection/optimizer combination against a
working plain arm. Its `10/12` alternative-connection score is diagnostic only
because that arm was swapped at inference time rather than trained separately.
Connection/gauge Phase I is positive within #973; the separate target-free
Phase-II population and salted commitment are frozen in PR #1001 with no
validation prediction. Protected merge and one-time reveal are active. Multi-resonance replacement and
recurrent factorization remain gated behind the held-out geometric oracle.
#954 remains blocked behind #973. The exact contract is
[ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md).

## Find your way around

- `src/` — the `r4` executable, local server, chat shell, and WASM surface.
- `crates/uor-r4-core` — current geometric route/manifest foundation plus
  preserved runtime research.
- `crates/uor-r4-router` — geometric router, memory, and dashboard backend.
- `crates/uor-r4-graph-*` — preserved graph-format/compiler/runtime research.
- `docs/` — current programme, mathematical decisions, evidence, and archive.

Start with the [documentation guide](docs/README.md). The
[Geometric Intelligence Programme](docs/geometric_intelligence_programme.md)
is the architecture and sequencing authority. Historical records remain
available through the documentation guide without dominating the front door.

## Contributing

This is an obscure and ambitious research problem, and useful contributions
are welcome. The most valuable work advances the first unblocked roadmap stage
and produces an observable user-facing capability. Expensive experiments and
broad QA stay dormant unless a current decision truly requires them.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change.

## License

MIT — see [LICENSE](LICENSE). © 2026 UOR Foundation.
