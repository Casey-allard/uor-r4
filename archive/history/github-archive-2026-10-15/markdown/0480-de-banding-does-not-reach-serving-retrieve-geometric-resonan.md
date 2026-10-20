# #480 — De-banding does not reach serving: retrieve_geometric_resonance builds a band-only QUERY projection against full-width stored vectors

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-07T14:39:15Z
- **Updated:** 2026-08-07T16:23:20Z
- **Closed:** 2026-08-07T16:19:30Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/480
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Filed from the #434 item 2 measurement (PR #479), and flagged as out-of-scope in the #465 scope note. Confirmed against the tree.

## The asymmetry

PR #465 adopted full-width content-bearing storage: `banded_storage` defaults to `false`, and `lib.rs:1787` keeps the full 512-d vector when it is off. Stored items are full width.

The **query** side did not follow. In `retrieve_geometric_resonance` (`lib.rs`, the `query_projections` block):

```rust
let mut full_state = vec![0.0; 512];
let start = r.active_range[0] as usize;
let end = r.active_range[1] as usize;
full_state[start..end].copy_from_slice(&r.state_vector[..end - start]);
query_projections.insert(r.window_index, full_state);
```

The query vector is zero everywhere outside `active_range`. The cosine against a full-width stored vector therefore only ever sees the band — roughly a sixteenth of the stored content — because the zeroed query coordinates contribute nothing to the dot product regardless of what the stored vector holds there.

## Why it matters

De-banding was adopted on measured evidence: retrieval MRR **0.2348 → 0.8948** and router anchor accuracy **9.3% → 11.4%** (#442 ARM BF, `docs/debanding_audit_434.md`). Those numbers came from a harness that compared full-width query vectors against full-width stored vectors.

The production retrieval surface does not do that. So the adopted gain — already paid for in storage, already regression-gated — is not reaching serving.

Supporting datapoint from PR #479: through `get_top_resonances_native` on 46,342 windows / 500 probes, Spectral measures **MRR 0.7179, top-1 0.6240, recall@20 0.9720** with a dead deranged-key null. That is well below the 0.8948 full-width reference. Some of the gap is a different probe/window regime and I am not claiming the whole of it — but the query-side banding is a known, mechanical cause pointing the right direction, and it is cheap to test.

## Suggested measurement (not run — this needs an owner)

Mirror the storage switch on the query side: build the query projection full-width when `banded_storage` is false, and A/B through `tests/geometry_ablation.rs`, which already has the corpus, ground truth, deranged-key null and pre-declared shape.

```
metric to move:      retrieval MRR / top-1 / recall@20 via get_top_resonances_native
reachability ceiling: bounded above by the #442 full-width figure (0.8948 MRR)
                     against the 0.7179 measured through the production surface
                     today — so on the order of +0.17 MRR, minus whatever the
                     probe-regime difference accounts for
instrument gate:     deranged-key null must stay dead in both arms
exit rule:           positive if full-width query projection beats the banded
                     one by >= 0.05 MRR
if positive:         wire it, and re-run the #421 reconnection + memory-lift
                     regression gates that de-banding was adopted against
if negative:         record that the storage gain does not transfer through the
                     query path, and find out why before any further
                     de-banding work
```

## Notes for whoever takes it

- Check whether `active_range` still means anything for a full-width store, or whether it should simply be ignored when `banded_storage` is false.
- The `#245` consumer audit (`docs/debanding_audit_434.md`) covered *storage* consumers. This is a query-path consumer that the audit did not reach, so it is worth re-reading that audit for siblings before assuming this is the only one.
- κ consequence: this changes retrieval ordering, not the graded code, so it is not a κ re-pin — but it does move the #421 anchor-accuracy rows, which are pinned measurements.

## Comments (2)

### comment 5218769941 — 2026-08-07T15:11:59Z — Casey-allard

Picking this up. One thing found on closer reading that changes the expected size of the prize, and belongs on the record before the run rather than after it.

**The cosine is a tie-breaker, not the ranking.** In `retrieve_geometric_resonance`:

```rust
let sim = cosine_similarity(q_vec, &item.state_vector);
let relevance = (shared_count as f64) * 100.0 + (sim * slice_norm) + scope_boost;
```

`shared_count` is the number of query primes present in the item, multiplied by **100**. `sim` is bounded by 1 and `slice_norm` is the norm of a band slice. So relevance is dominated by lexical overlap, and observed values look like 415 / 215 / 115 — i.e. 4, 2, 1 shared words plus a fractional cosine term. **Fixing the query projection can only reorder candidates that are tied on `shared_count`.**

That does not make the issue wrong — the asymmetry is real and the fix is right on its own terms — but it means my "+0.17 MRR" ceiling in the original filing was too loose. It was the gap to the #442 full-width reference, which was measured by a harness that ranked by cosine alone, with no lexical term. Correcting that estimate now.

**Run contract**

```
metric to move:      retrieval MRR / top-1 / recall@20 via get_top_resonances_native
                     current: 0.7179 / 0.6240 / 0.9720 (measured in #479)
reachability ceiling: absolute headroom is 1 - 0.7179 = 0.28 MRR, but the
                     REACHABLE headroom is only the ranking churn available
                     within shared_count ties, because the lexical term is
                     100x the cosine term. Recall@20 is already 0.9720, so
                     this cannot be a recall fix either — it is a
                     reordering-within-ties fix or it is nothing
instrument gate:     deranged-key null must stay dead in both arms
exit rule:           positive if full-width query projection beats the banded
                     one by >= 0.05 MRR
if positive:         adopt, and re-run the #421 reconnection + memory-lift
                     regression gates de-banding was adopted against
if negative:         record that the storage-side de-banding gain does NOT
                     transfer through this query path, and say why — the
                     lexical term dominates the ranking, so the #442 number
                     was never going to appear here. That is a materially
                     different claim from "de-banding does not work", and it
                     retires this issue rather than leaving it open as a
                     suspected unrealized gain
cost estimate:       minutes; tests/geometry_ablation.rs already has the
                     corpus, ground truth and deranged-key null
```

**The fix mirrors storage exactly.** Storage takes the full 512-d vector when `!banded_storage`:

```rust
Some(full) if !self.banded_storage && full.len() == 512 => full.to_vec(),
```

The query side will do the same, so the two are symmetric by construction rather than by coincidence. `slice_norm` stays as-is — it is a scale factor on the cosine term and changing it in the same patch would confound the measurement.

**Arms:** banded storage + banded query (the pre-#465 world), full storage + banded query (main today, 0.7179), full storage + full query (the fix). Three points make the asymmetry visible instead of just the endpoint.

### comment 5219507020 — 2026-08-07T16:23:20Z — Casey-allard

Closing record (auto-closed on merge of #481, 413542b).

**I filed this issue; its premise is refuted.** The asymmetry is real — storage full-width since #465, query projection band-only — but fixing it does not pay.

| arm | store / query | top-1 | MRR | recall@20 |
|---|---|---:|---:|---:|
| banded | banded / banded | 0.6240 | 0.7195 | 0.9640 |
| **shipped** | **full / banded** | **0.6240** | **0.7179** | **0.9720** |
| symmetric | full / full | 0.6320 | 0.7238 | 0.9540 |

**+0.0059 MRR, +0.0080 top-1, −0.0180 recall** against a +0.05 bar. Deranged-key null 0.0000 everywhere. `banded/banded` also beats `shipped` on MRR, so the de-banding gain doesn't reach this path from the storage side either.

**Why:** `relevance = shared_count * 100 + sim * slice_norm + scope_boost`. The ranking is lexical; the cosine is a tie-breaker, so vector shape only reorders candidates already tied on word overlap. Tie-mass is 81.2%, so the cosine had plenty of room — widening its input just doesn't change the ordering. The 0.8948 figure in #442 came from a harness ranking by **cosine alone**. De-banding is worth what #442 measured *for cosine-ranked retrieval*, and this serving path is not cosine-ranked.

**Not adopted.** The symmetric shape sits behind `set_full_width_query`, default off, and the site now documents the asymmetry as deliberate so it isn't re-filed. Two reasons beyond the numbers: an ordering change moves the pinned #421 anchor-accuracy rows and `router_reconnect.rs` needs a scored R4G1 artifact I didn't have; and it becomes the right default the moment the lexical term stops dominating.

**What it points at, recorded but deliberately not filed:** the `* 100` weight is hard-coded, never measured against alternatives, and dominates a geometric retrieval stack by two orders of magnitude. Measuring that weight — or ranking by cosine *within* lexical buckets rather than summing — is the real experiment. It needs an owner and an exit rule.


---
_Comment/review ids in this file: 5218769941, 5219507020_

