# #489 — fix(router): #486 — the serving path compared a routing vector to a content vector

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T23:48:32Z
- **Updated:** 2026-08-08T00:16:37Z
- **Closed:** 2026-08-08T00:07:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/489
- **Merged:** 2026-08-08T00:07:51Z
- **Merge commit:** b9c059a9a4c2d3df6a26884dd4755cc736a63f5d
- **Base / Head:** main / UOR-Foundation:issue-486-geometry-selfmatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#484 found the cosine at chance on this path and could not say why. It is a **category error**, not a tuning or shape problem.

## Diagnosis

Self-match over 46,342 candidates — where does the target's own cosine fall among all candidates' cosines? **0.5 is chance.**

| arm | self pct | self sim | cross sd | separation | top-1 |
|---|---:|---:|---:|---:|---:|
| identity probe, band query (deployed) | 0.4938 | −0.0006 | 0.0447 | −0.01 sd | 0.0000 |
| identity probe, full-width query | 0.4690 | +0.0056 | 0.0430 | −0.06 sd | 0.0000 |
| **identity probe, CONTENT query** | **1.0000** | **+1.0000** | 0.1005 | **+9.42 sd** | **1.0000** |
| **shipped probe, CONTENT query** | **1.0000** | **+0.7156** | 0.0963 | **+7.57 sd** | **0.8000** |

Querying with the **exact stored sentence**, its own vector ranks at chance. Two hypotheses died on the first rows: cross-candidate spread is 0.045 sd, so the stored vectors are perfectly distinguishable (**not saturated**), and full-width query does not help (**not the band projection**). Indexing is deterministic.

## Cause

`retrieve_geometric_resonance` builds its query vector from the **routing** path. `index_sentence_internal` stores a **content** vector (`content_state_vector`, #245). Different objects from different code paths — their cosine is noise by construction.

The harnesses that measured cosine retrieval *working* on this stack (#442, MRR 0.8948 — `memory_lift_ab.rs`, `zeta_projection_quality.rs`) index the query text as a corpus item and read its stored vector back, so **both sides share one encode**. That is why 0.8948 was real and why it never appeared at serving.

## Fix, measured

| ranking | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| deployed (routing query, W=100) | 0.6240 | 0.7179 | 0.9720 |
| content query, W=100 (weight unchanged) | 0.7840 | **0.8542** | 0.9900 |
| content query, W=0 | **0.8160** | **0.8763** | **0.9900** |

**Changing one thing is worth +0.1363 MRR and +0.16 top-1, with recall UP 0.018** — against a bar that has been +0.05 on this path throughout. Deranged null 0.0000 in every arm. 0.8763 lands beside #442's 0.8948, which is the check that this is the same quantity that measurement was talking about.

## Revises #484

Its flat sweep was **conditional on a broken cosine**. With a working geometric term the curve lifts to 0.854–0.876 and the optimum moves to `W = 0` — the opposite end from the shipped value. `docs/lexical_weight_484.md` now carries that correction inline rather than being silently superseded.

## Not adopted

`set_content_query_vector` defaults **OFF**; no deployed behaviour changes here. An ordering change moves the pinned #421 anchor-accuracy rows and needs `router_reconnect` against a scored R4G1 artifact — the gate #480 recorded. Filed separately as the adoption issue.

Also open by design: the routing state vector is presumably in that position for a reason (session state, drift, personalization). This shows it is the wrong thing to take a cosine against; it does not show nothing else depended on it.

Records: `docs/geometry_selfmatch_486.md`, `docs/lexical_weight_484.md`

Closes #486


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

