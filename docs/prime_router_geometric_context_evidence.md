# Evidence note: R4 angular state as a structural context carrier (prime-router lineage)

Status: evidence record for the claim register introduced by the template
rebase (issue #273). Source system: `Casey-allard/prime-router` (Python,
R4 hypersphere router wrapping an ollama backend) — the direct ancestor of
this repository's geometric path. All file/line references below are to
that repository's `server.py` and its research report
("Discrete Prime-Aligned Hex-Hive Topologies, Continuous Hopf Phase
Transport, and Thermodynamic Stability in Recurrent Dynamical Systems",
section 6), recorded 2026-07-28.

## 1. The claim

R4 angular state — the session hypersphere vector, its window (winding)
classification, and angular retrieval over content-derived stored vectors —
carries structural, abstract context sufficient to (a) select thematically
relevant grounding content for a query and (b) condition backend text
generation when injected as a structured state description.

Proposed register level: **some-true** — demonstrated in the ancestor
system with the measurements in section 3; not yet reproduced under this
repository's measurement gates (see section 4).

## 2. Mechanism (as implemented in the ancestor)

- **Content-derived coordinates.** Words receive vectors on the unit
  hypersphere; a sentence or query is the sum of its word vectors
  (`route_query_to_manifold`, server.py:1717). Stored items carry their own
  content-derived `state_vector` — the property this repository lost in the
  Rust port and restored in issue #245.
- **Window routing.** The query signal is projected onto each scale
  window's orthonormal basis; the window with the highest projection
  energy wins (server.py:1717–1760). Session dynamics accumulate Hopf
  phase into a winding number that classifies the dihedral window state
  (server.py:1169–1198).
- **Angular retrieval.** Candidate sentences are scored by a hybrid rule:
  shared prime factors dominate (×100), sub-ranked by cosine resonance
  between the query projection and the stored state vector, scaled by
  window slice norm (server.py:1305–1320).
- **State injection.** `ollama_generate` (server.py:2085–2135) injects the
  routed state into the backend's system prompt as structured text: window
  index and theme, energy κ, deficit angle, Hopf coordinates (χ, δ, α),
  and the top retrieved grounding sentence. A `geometric-retrieval`
  generation mode serves retrieval-only output when the backend is
  offline (server.py:2178).

## 3. Measured evidence (report, section 6)

The ancestor's ablation tables measure where the routed signal lives:

- **Restoration probe.** Masking the initial state trajectory (`no_tau0`)
  drops coordinate-tracking accuracy from 1.0000 to 0.3027; `last_only`
  drops it to 0.2612. The routed trajectory, not any single coordinate,
  carries the signal.
- **Delayed-trainable retest.** Reading the answer directly off the
  initial coordinate (`tau0_direct`) scores 0.2588, while masking that
  same coordinate entirely (`no_tau0`) preserves 1.0000 — the semantic
  signal is distributed along the geodesic path rather than encoded at
  the start, and the final transport step is required for state
  resolution (`no_last` collapses to 0.2578).
- **Mod12 linear decodability.** Decodability of the routed signal rises
  from 0.2937 (initial) through 0.2786 (mid) to 1.0000 at the final
  position in the reduced regime: transport actively structures the
  representation toward a linearly readable final state.

These are measurements of the routing/state layer. The qualitative
injection demonstrations (conditioned backend output tracking window
theme and grounding sentence) were observed interactively and are not
captured in a table.

## 4. Scope: what this evidence does not show

- **No token-level generation by geometry.** In every injection
  demonstration the language surface was the backend LLM; the geometry
  selected and framed context. Whether geometric addressing can replace
  the generation surface itself is exactly the open question measured by
  the issue #244 matrix and the issue #243 Design R/G program — this note
  must not be cited for that claim.
- **No held-out language metric.** The ancestor's tables measure routing
  accuracy and decodability, not next-token prediction on held-out text.
- **Vocabulary-relative coordinates.** Word→prime assignment is
  arrival-ordered, so stored vectors are comparable within a session
  brain but not across independently grown vocabularies (rediscovered in
  issue #245's tests).
- **Pre-gate provenance.** These runs predate this repository's
  reproduction discipline (κ-pinning, wording gates); treat numbers as
  the ancestor's report states them, pending re-run under issue #246/#247
  harnesses.

## 5. Proposed register entry (template rebase, issue #273)

```toml
[[claim]]
id = "geometric-context-carrier"
level = "some-true"
statement = """
R4 angular state (hypersphere session vector + winding-window
classification + angular retrieval over content-derived stored vectors)
carries structural abstract context: it selects thematically relevant
grounding content and conditions backend generation when injected as
structured state.
"""
evidence = [
  "docs/prime_router_geometric_context_evidence.md",
  "ancestor: Casey-allard/prime-router server.py (route_query_to_manifold, ollama_generate)",
  "ancestor report section 6 ablation tables (restoration probe, delayed-trainable, mod12 decodability)",
]
falsifier = """
Re-run the injection path with the state description and grounding
sentence replaced by (a) a shuffled-state control and (b) a random-vector
control: if conditioned output relevance does not degrade versus the
routed state, the geometric state carries no structural context and the
claim drops to open.
"""
depends_on = ["issue #245 (content-bearing storage)", "issue #246/#247 (reconnection harnesses)"]
```

## 6. Relation to open work

Issue #245 restored the content-bearing storage this mechanism requires;
issues #246/#247 reconnect the angular layer to the serving path, and are
the natural home for re-measuring section 3 under this repository's
gates; issue #243 (Design R/G) tests the separate, stronger claim that
geometric addresses can stand in for learned codebook classes inside the
prediction store itself. The serving cascade's geometric tier (#248)
is the runtime seat of the some-true claim above.
