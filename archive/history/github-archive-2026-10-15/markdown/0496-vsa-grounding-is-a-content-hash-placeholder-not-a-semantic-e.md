# #496 — VSA grounding is a content-hash placeholder, not a semantic encoder — decide: real encoder vs deprecate the Vsa switch

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T03:55:45Z
- **Updated:** 2026-08-08T21:08:06Z
- **Closed:** 2026-08-08T21:08:05Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/496
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Split out from #493 (disposition of #434). This is an **engine-design decision for the owners**, not a bug fix.

## What #493 measured
VSA retrieval (`GeometryType::Vsa`) cannot rank by content similarity, for a reason deeper than the scoring wiring:

- `index_corpus` / `index_sentence` **do** populate `facet_store` (via `index_sentence_routed` → `index_semantic_object`), so the candidate **set** from the facet intersection is real.
- But `retrieve_vsa_multi_facet_resonance` scores each candidate with `cosine_similarity(query 1024-dim VSA hypervector, stored 512-dim spectral content vector)`. `cosine_similarity` returns exactly `0.0` on a length mismatch, so **every candidate scores 0.0** — the dead ranking #434 saw.
- Making the comparison commensurable (re-ground each candidate to its own 1024-dim hypervector) does **not** fix it: `VsaGeometry::ground` derives the hypervector by hashing the exact content string (`expand_atom`), so related sentences get unrelated random ±1 vectors. Measured: a "quick brown fox" query ranked the fox sentence **last**. The grounding is a **content-hash placeholder, not a semantic encoder**.

## The decision
VSA retrieval has no semantic signal to rank by until `VsaGeometry::ground` is a real encoder. Two directions:

1. **Give VSA a real encoder** — replace the content-hash `expand_atom` grounding with a semantic embedding, store the VSA-space vector alongside each item, score commensurably, then re-run `crates/uor-r4-router/tests/geometry_ablation.rs` under the #490-style gate. Significant work; only worth it if VSA (facet-structured, multi-index retrieval) is wanted as a distinct capability from Spectral.
2. **Deprecate the VSA switch** — if Spectral (now with the #490 content-vector query) is the intended semantic path, remove or hard-deprecate `set_geometry_type("vsa")` rather than carry a non-functional geometry.

#493 already landed the honest interim state (the knob warns loudly; `tests/vsa_scoring_honesty.rs` pins the corrected facts; `docs/geometry_ablation_434.md` carries the measured mechanism). This issue is the fix-vs-deprecate call.

Unassigned — for Ari / Alex (engine quality-gate policy).


## Comments (3)

### comment 5227094819 — 2026-08-08T16:52:32Z — Casey-allard

Context for the fix-vs-deprecate decision (input, not a decision — this stays an Alex/Ari engine-policy call):

Since this was filed, the Spectral path has moved. #490 made the content-vector query the deployed default and #502 (PR #504) just dropped the lexical weight on it, so Spectral is now a working content-similarity retriever on the D3 fixture (0.8763 MRR / 0.8160 top-1 / 0.99 recall, deranged null dead). That sharpens the framing here: the decision is no longer "do we have a semantic retriever" (Spectral is one) but specifically "do we want VSA's *facet-structured, multi-index* retrieval as a **distinct capability** from Spectral — enough to build `VsaGeometry::ground` into a real encoder (option 1)?" If the answer is that Spectral covers the intended semantic path, option 2 (hard-deprecate `set_geometry_type("vsa")`) removes a non-functional geometry rather than carrying it.

No action taken — leaving open and unassigned for the owners. Just connecting the #490/#502 serving-path change to this decision so it is not made against a stale picture of what Spectral can now do.


### comment 5227433377 — 2026-08-08T18:09:24Z — Casey-allard

## Decision: build a real encoder (Casey's call) — design

Picking this up. The design keeps VSA **consistent with the goal** (multiplication-free, no GPU, no transformer embedding): ground the VSA hypervector from the **same zeta word-sum content signal Spectral already uses**, rather than a content hash. Related content then gets similar vectors *because the base is the content construction that already carries signal* (0.876 MRR on Spectral post-#490/#502).

### The two defects (confirmed in current code)

1. **`VsaGeometry::ground` (geometry.rs:226)** calls `expand_atom(object_type, content, space_cid)` — a hash of the content string expanded to ±1 bits. Related sentences → unrelated random vectors. No semantic signal.
2. **`retrieve_vsa_multi_facet_resonance` (lib.rs:2358)** cosines the 1024-dim grounded query vector against `item.state_vector`, the **512-dim spectral** stored vector → length mismatch → exactly 0.0 (#493's dead ranking). Even commensurable, defect (1) makes it chance.

### The fix

- **Semantic grounding.** Replace the hash with the zeta word-sum: for each content word, `identity_to_qimc_prime` → `get_word_vector` → sum → normalize (the exact loop `SpectralGeometry::ground` runs at geometry.rs:80–103, and those symbols are already in scope here). Place the normalized 512-d content signal in the hypervector; the VSA facet algebra (`encode` → type/entity multi-index) rides on top unchanged. This is the honest realization of "VSA = the content vector **plus** facet-structured multi-index retrieval," which is the distinct capability the switch is for.
- **Commensurable scoring.** Re-ground each *candidate's* sentence the same way and cosine VSA-vs-VSA (1024-d vs 1024-d), instead of query-VSA vs stored-spectral. #493 tried re-grounding but it couldn't help while `ground` was a hash; with a semantic ground it ranks by content similarity.
- **Update `vsa_scoring_honesty.rs`** — it currently pins the *broken* behavior on purpose ("quick brown fox ranked last") with a note to update once scoring is commensurable. After the fix it asserts the fox sentence ranks well.

### The gate (DoD)

Re-run `crates/uor-r4-router/tests/geometry_ablation.rs` under the **#490-style gate** (deranged-key null dead; VSA must clear it and be reported against Spectral on the same harness). VSA's added value over Spectral is the facet-structured candidate *set*; the ablation measures whether that pays or merely matches. Ships only if it clears the gate; if VSA merely matches Spectral, that itself is the recorded finding (and reopens the deprecate option honestly).

Implementing the encoder + commensurable scoring + unit test now (concurrent with the 360M P2 run); the ablation validation queues behind that run so it isn't measured under core contention. Assigned to me while active.


### comment 5227836760 — 2026-08-08T19:53:31Z — Casey-allard

## Encoder built + measured — PR #507

Built the real encoder as decided, and ran it through the gate. Result, honestly:

**The encoder works.** `VsaGeometry::ground` now grounds from the zeta word-sum content signal (not a content hash), and scoring is commensurable VSA-vs-VSA. VSA went from **dead (0.0000 on every arm)** to functional — `geometry_ablation` (300 stories/30 probes, deranged-null dead): VSA 0.47 top-1 / 0.50 MRR / 0.53 recall@20.

**But Spectral still wins decisively:** 0.90 / 0.93 / 1.00 vs VSA 0.47 / 0.50 / 0.53 — Spectral +0.43 MRR, past the 0.05 gate (full-caps Spectral 0.83). The cause is in the recall: VSA's **facet candidate-set drops ~47% of targets** (0.53 vs 1.00). So the grounding is fixed, but the **facet index — not the encoder — is now the bottleneck.** Separately, query-time re-grounding is O(candidates) and slow at full scale, which is exactly the "store the VSA vector at index time" step this issue named.

## The decision this leaves (now on evidence, not a placeholder)

VSA's distinct value was always the facet-structured multi-index retrieval. The encoder is no longer the blocker; the facet index is. So the call is:

1. **Build a real facet-index** — make the type/entity/relation facets actually discriminate (they currently bucket coarsely: `type = sum%100`, `entity` hardcoded `[100,200]`), and store the VSA vector at index time for O(1) scoring. Substantial, and only worth it if facet-structured retrieval is wanted as a capability distinct from Spectral.
2. **Deprecate the switch** — Spectral (0.83 MRR) already serves the semantic path; a functional-but-losing VSA may not earn its keep.

Keeping #496 open for that call. PR #507 makes the VSA path real and measured either way (it does not change the default — Spectral stays the path).



---
_Comment/review ids in this file: 5227094819, 5227433377, 5227836760_

