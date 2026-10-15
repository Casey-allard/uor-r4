# #486 — Why does the cosine fail to identify a sentence from itself? (#484 root cause)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T22:18:45Z
- **Updated:** 2026-08-08T00:16:35Z
- **Closed:** 2026-08-08T00:07:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/486
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

#484 measured something that should not be possible in a working retrieval stack, and stopped short of diagnosing it. This issue owns the diagnosis.

## The observation

Ranking by bare cosine through `retrieve_geometric_resonance`, over 46,342 indexed windows with the target guaranteed present (containment 1.0000):

| probe | median rank of the target | full-list MRR |
|---|---:|---:|
| shipped (every other token, reversed) | 21,401 | 0.0001 |
| ordered (reversal removed) | 21,401 | 0.0001 |
| **identity (the exact stored sentence)** | **21,082** | 0.0001 |

Random over this candidate set is median 23,171.

**Querying with the exact text of a stored sentence, the cosine ranks that sentence 21,082nd out of 46,342.** It is not the probe form, not subsampling, not word order — `docs/lexical_weight_484.md` ruled those out by construction.

## Why this is strange rather than merely negative

The project has a measurement showing cosine retrieval *does* work on this stack: #442's de-banding result, retrieval MRR 0.2348 → 0.8948, from a harness that ranks by **cosine alone**. So the same stored vectors support 0.8948 MRR in one harness and sit at chance in this one.

That gap has to live somewhere specific, and it is almost certainly on the **query** side or in how the two vectors are produced, not in the stored content vector itself. Which makes this a bisection, not an open-ended investigation.

## Bisection ladder — cheapest first, and step 1 may be enough

**Step 1 — compare the vectors directly, no ranking at all.** For a probe sentence `S`, obtain the query projection `q(S)` that `retrieve_geometric_resonance` builds and the stored vector `v(S)` that `index_corpus` wrote. Report the distribution of `cosine(q(S), v(S))` against `cosine(q(S), v(T))` for random `T`. If self-similarity is not separated from cross-similarity, the two objects do not correspond and everything downstream is explained. This needs no retrieval loop and should run in seconds.

**Step 2 — is it the band-only query projection?** #480 measured that `set_full_width_query(true)` does not change *ordering*; nobody checked whether it changes *self-identification*. Re-run step 1 with the knob on. This is the leading hypothesis: storage is full-width (#465) and the query is band-only, so the cosine only ever sees one sixteenth of the query's coordinates.

**Step 3 — are the stored vectors mutually distinguishable at all?** Pairwise cosine distribution over a sample of stored vectors. If they cluster near 1.0 the content vector is saturated and there is nothing to discriminate, which would be a bug with a fix rather than an architectural fact.

**Step 4 — determinism.** Index the same sentence twice; assert the vectors match. Cheap, and it rules out a whole class of surprise.

**Step 5 — reconcile with #442.** Find what that harness did differently (it ranked by cosine alone and reported 0.8948 on this stack). The delta between its setup and `retrieve_geometric_resonance` is the answer by construction.

## Run contract

```
metric to move:       separation between cosine(q(S), v(S)) and
                      cosine(q(S), v(T)) for random T. Currently
                      indistinguishable, inferred from a median rank of
                      21,082 / 46,342 on the identity probe.
reachability ceiling: not applicable — this is a diagnostic, not a lever.
                      It cannot fail to inform: every outcome below names a
                      different next action.
instrument + verdict: steps 1-4 above, all cheap, no teacher, no checkpoint.
                      Step 1 is binding: if self and cross similarity ARE
                      separated, the fault is in ranking rather than in the
                      vectors and steps 2-4 are skipped.
exit rule:            not a threshold — a classification. Report which of
                      (a) saturated stored vectors, (b) non-corresponding
                      query/stored objects, (c) band-projection loss, or
                      (d) weak-but-real embedding the evidence supports.
if (a) or (c):        a BUG with a fix; geometric retrieval on this route
                      becomes reachable and the #480 / #484 negatives get
                      re-run against the fix.
if (b):               architectural, and it is the decision flagged below.
if (d):               the embedding is weak; that is a different programme
                      and should be scoped separately, not smuggled in here.
cost estimate:        under an hour including the harness. Nothing blocks
                      on it.
```

## Scope

`retrieve_geometric_resonance` and `index_corpus` on the D3 token corpus with synthetic word renderings (the standing #422/#423 law). This does not touch the transformerless engine or the graph runtime.

## Why it matters beyond this function

Until this is answered, the project has **no measured evidence that geometric retrieval works on the deployed serving route**. The one number that looked like evidence — 0.7179 MRR, quoted from #434 as Spectral's score — is produced by the lexical term (see the companion issue). #442's 0.8948 remains real evidence that cosine retrieval *can* work here; it simply describes a different harness. Closing that gap is the whole point.

**Unassigned.**


## Comments (1)

### comment 5223456063 — 2026-08-08T00:16:34Z — Casey-allard

Delivered in **#489**. Record: `docs/geometry_selfmatch_486.md`.

## The answer

A **category error**, not a tuning or shape problem. `retrieve_geometric_resonance` built its query vector from the **routing** path; `index_sentence_internal` stores a **content** vector (`content_state_vector`, #245). Different objects from different code paths — their cosine is noise by construction.

The bisection ladder this issue proposed ran in order and two hypotheses died on the first rungs:

| arm | self percentile | cross sd | separation | top-1 |
|---|---:|---:|---:|---:|
| identity probe, band query (deployed) | 0.4938 | 0.0447 | −0.01 sd | 0.0000 |
| identity probe, full-width query | 0.4690 | 0.0430 | −0.06 sd | 0.0000 |
| **identity probe, CONTENT query** | **1.0000** | 0.1005 | **+9.42 sd** | **1.0000** |
| **shipped probe, CONTENT query** | **1.0000** | 0.0963 | **+7.57 sd** | **0.8000** |

0.5 is chance.

- **(a) saturated stored vectors — ruled out.** Cross-candidate spread is 0.045 sd; the stored vectors are perfectly distinguishable from each other. The query simply does not land near its own.
- **(c) band-projection loss — ruled out.** `set_full_width_query(true)` does not help. #480 measured that the symmetric shape does not change *ordering*; it does not change *self-identification* either.
- Indexing is deterministic (two independent indexings retrieve identically), so none of this is noise.

**(b) non-corresponding objects** is what the evidence supports, and the code confirms it.

## Why #442's 0.8948 was real and never appeared here

The harnesses that measured cosine retrieval working on this stack — `memory_lift_ab.rs`, `zeta_projection_quality.rs` — build their query vector by indexing the query text as a corpus item and reading its stored vector back, so **both sides go through one encode**. Nobody had noticed that was doing the work. It is why that number was correct and why it was unreachable at serving.

## The fix, measured

| ranking | top-1 | MRR | recall@20 |
|---|---:|---:|---:|
| deployed (routing query, W=100) | 0.6240 | 0.7179 | 0.9720 |
| content query, W=100 (weight unchanged) | 0.7840 | **0.8542** | 0.9900 |
| content query, W=0 | **0.8160** | **0.8763** | **0.9900** |

Deranged null 0.0000 in every arm. Changing one thing is **+0.1363 MRR, +0.16 top-1, and recall UP 0.018**, against a bar that has been +0.05 on this path throughout. The 0.8763 landing beside #442's 0.8948 is the check that this is the same quantity that measurement was describing.

## What it revises

**#484's flat weight sweep was conditional on a broken cosine**, and `docs/lexical_weight_484.md` now carries that correction inline rather than being silently superseded. With a working geometric term the curve lifts to 0.854–0.876 and the optimum moves to `W = 0` — the opposite end from the shipped value. The `W >= 1` plateau survives; it just sits at 0.8542 instead of 0.7179.

**#480 is now fully explained.** Reshaping a query vector that was never comparable to the stored vector could not have paid at any shape.

## Not adopted

`set_content_query_vector` defaults **OFF** and no deployed behaviour changed. An ordering change moves the pinned #421 anchor-accuracy rows and needs `router_reconnect` against a scored R4G1 artifact — the gate #480 recorded. Filed as **#490**, which also carries the question this measurement cannot answer: the routing state vector is presumably in that position deliberately (session state, drift, personalization), and this shows it is the wrong thing to take a cosine against without showing that nothing else depended on it being there. #490 recommends the minimal variant first so the gate can attribute any movement to one change.



---
_Comment/review ids in this file: 5223456063_

