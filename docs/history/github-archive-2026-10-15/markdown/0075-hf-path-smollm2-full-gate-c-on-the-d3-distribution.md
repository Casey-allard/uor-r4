# #75 — HF-path (SmolLM2) full Gate C on the D3 distribution

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:33Z
- **Updated:** 2026-07-23T11:18:51Z
- **Closed:** 2026-07-23T08:39:52Z
- **Labels:** graph-compiler
- **Milestone:** Phase 5: Allocation-free graph runtime
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/75
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64, D3 observation run. The M.V.G. checkpoint's judge must run on the actual deployment model and declared distribution, not the legacy fixture corpus. Run cover + score + Gate C for the SmolLM2-135M compile on the D3 partitions (teacher-generated continuity + Simple Wiki natural), producing the checkpoint inputs for BASELINE.md §4 targets 1-2. Acceptance: a score_report.json per partition with the 4-way table; the §4 target rows filled with measured values (pass or fail stated plainly).


---

## Definition of Done
- Cover + score + Gate C run end-to-end for the SmolLM2-135M compile on both D3 partitions (teacher-generated continuity + Simple Wiki natural).
- The M.V.G. target rows 1–2 in `docs/transformerless/BASELINE.md` §4 are filled with measured values and plain pass/fail verdicts.

## Acceptance criteria
- [ ] One `score_report.json` per partition with the full 4-way table, posted in this thread.
- [ ] BASELINE.md §4 rows 1–2 updated with measured agreement and bits/token vs the HF-path TLA3 baseline.
- [ ] Any gap between fixture-corpus and D3-distribution numbers is analyzed (not just reported).

## Comments (4)

### comment 5056231172 — 2026-07-23T08:39:51Z — auser

Complete — merged via the merge queue as PR #97 (mergedAt 2026-07-23T08:38:45Z).

DoD evidence: (1) Both partitions run end-to-end with full 4-way tables. Natural partition (400-article Simple Wiki slice; observation: 90,019 records, 400/400 articles, partition 72,562/17,457 matching the blake3 split rule exactly (320/80), merged κ blake3:43d961d7... with independent byte-replication; determinism double-run proven): Rule 1+2 top-1 15.0427% / bits 13.3008 vs TLA3 baseline 15.0427% / 18.7562 (argmax-identical, +0/−0 win/loss). Continuity partition (200k teacher-generated records, 40,342 held-out): Rule 1+2 14.7613% / 14.6207 vs baseline 14.7613% / 20.6330 (argmax-identical). Witness replay 64/64 each. (2) M.V.G. §4 rows 1–2 verdicts: Row 1 floor PASS / +5pp stretch FAIL on both partitions; Row 2 PASS on both (−5.46 natural, −6.01 continuity). Deployed quality gate: PASS on both at full f64 precision. (3) BASELINE.md §3.1 carries the D3 first-pass block with corpus CIDs, both tables, verdicts, and the gap analysis. (4) Gap analysis: D3-scale SmolLM2 agreement (~15%) vs fixture (31.7%) is a teacher difference (SmolLM2 vs legacy stories15M), not a harness regression — directly relevant to the #73 ceiling evaluation; Rule 1 (graph residuals alone) collapses on natural text (0.19%) because 8-token contexts repeat far less on natural text — the recovery is entirely exact-context precedence, and 100% of positions are ExactContext status on both partitions (Graph/Novel still corpus-unmeasured at scale). (5) Tokenizer fix included: scenarios.rs encode now sizes by bytes (multi-byte UTF-8 fallback could overflow on natural text). Verified: 49 suites, clippy -D warnings, fmt clean; PR checks all green; merged through the queue.

### comment 5057603436 — 2026-07-23T10:57:42Z — auser

## HF-path (SmolLM2-135M) Gate C on both D3 partitions — complete

Teacher: SmolLM2-135M-Instruct (κ `blake3:12d2cd8a…`), TLA4 artifacts κ `blake3:d4623b3a…`. Both runs re-induced the cover at the default config; witness replay 64/64 on both.

### Continuity partition (SmolLM2 teacher-generated, 150,000 records / 1505 stories, 80/20 ordinal cut, 31,696 held-out; 38 regions)

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old) | 0.0% | 47.0064 |
| graph chain (Rule 1) | 6.1% | 45.7397 |
| graph chain+EXCT (1+2) | **4.7%** | **15.3717** |
| Rule 1 no-F | 6.1% | 16.0913 |
| Rule 1+2 no-F | 4.7% | 15.3717 |
| TLA3 store baseline | 4.7% | 18.1105 |

Status: ExactContext 31,696 / Graph 0 / Novel 0. Scored artifact κ `blake3:1e3dcfc3…`. Note Rule 1 alone beats Rule 1+2 on top-1 here (6.1% vs 4.7%) — exact-context memory of generated text disagrees with teacher argmax more than the chain does.

### Natural partition (sealed Simple Wiki, 361,232 records / 3000 articles, CID `blake3:194db0ee…`, 72,131 held-out by content-hash rule; 14 regions)

| scorer | top-1 agree | bits/token |
|---|---|---|
| graph Σ-cloud (old) | 0.0% | 36.0506 |
| graph chain (Rule 1) | 6.1% | 24.5887 |
| graph chain+EXCT (1+2) | **28.1%** | **11.3565** |
| Rule 1 no-F | 6.1% | 15.4585 |
| Rule 1+2 no-F | 28.1% | 11.3565 |
| TLA3 store baseline | 28.1% | 11.2481 |

Status: ExactContext 72,131 / Graph 0 / Novel 0. Scored artifact κ `blake3:3c99b707…`.

### M.V.G. §4 verdicts (rows 1–2)

| target | continuity | natural | verdict |
|---|---|---|---|
| 1. agreement ≥ baseline + 5pts | 4.7% vs 4.7% (+0.0) | 28.1% vs 28.1% (+0.0) | **FAIL** both (floor met — identical prediction sets — but +5 missed) |
| 2. bits/token ≤ baseline − 0.3 | 15.37 vs 18.11 (−2.74) | 11.36 vs 11.25 (+0.11) | **PASS** continuity, **FAIL** natural |

### Gap analysis (fixture vs D3 distribution)

1. **EXCT dominance is total.** On every D3 partition measured so far (llama natural, SmolLM2 continuity, SmolLM2 natural), 100% of held-out predictions resolve via exact context; the graph rules never fire. The fixture-corpus 31.7% era had the same structural pattern (EXCT carryover), but the D3 runs make it unambiguous: at current corpus scale the scorer is an exact-context memory with a graph fallback that never engages. Candidate recall tells the same story (natural Rule 1+2 top-1 recall 77.5% yet zero Graph status).
2. **The baseline is teacher- and distribution-dependent.** TLA3 agreement: 31.7% (llama fixture) → 4.7% (SmolLM2 continuity) → 28.1% (SmolLM2 natural). The M.V.G. targets were anchored to the fixture era and need re-anchoring; a +5pts-over-baseline target means something very different at a 4.7% baseline than at 31.7%.
3. **Generated vs natural text behave oppositely on bits.** Continuity: graph wins by −2.74 bits. Natural: graph loses by +0.11. Exact-context replay of the teacher's own generations is well-calibrated; on natural text the graph's emission smoothing costs slightly against the raw store.
4. **Where the graph could matter, it doesn't get to.** Until held-out Novel/Graph status is nonzero (larger corpus, weaker EXCT, or a judge that excludes exact matches), targets 1–2 measure EXCT-table quality, not graph quality. Recommend the checkpoint review treat EXCT-free evaluation as a follow-up measurement before the redesign call.

Per the §4 contract, missing targets 1–2 is a stop-or-redesign signal. BASELINE.md §4.1 updated with these numbers in PR #96; the decision itself is the maintainer's.

### comment 5057677886 — 2026-07-23T11:06:26Z — auser

**Reconciliation with the merged first pass (#97):** the tables above are the full declared natural corpus (n=3000, 361,232 records); #97's §3.1 block scored a 400-article prefix slice. The numbers are not interchangeable: at n=3000 the baseline's natural bits/token improves 18.76 → 11.25 (exact-context coverage grows with corpus size) while the graph stays ~flat, so the **natural target-2 verdict flips from PASS (slice) to FAIL (full corpus)** — 11.3565 vs the ≤ 10.9481 threshold. Continuity is unaffected (the declared 200k corpus row stands; my independent 150k regeneration corroborates its shape: 4.7%/15.37 vs 4.7%/18.11). PR #96 carries the reconciliation in BASELINE.md (§3.1 keeps the n=400 record, §4.1 has the consolidated target table) plus the encode-buffer regression test and `observe` merged.bin output.

### comment 5057782565 — 2026-07-23T11:18:51Z — auser

Landed via the merge queue in f9c1602 (#96): encode regression test, `observe` merged.bin output, and the BASELINE.md reconciliation (§3.1 full-corpus n=3000 natural record + §4.1 consolidated target table with the target-2 natural verdict at the declared corpus size).


---
_Comment/review ids in this file: 5056231172, 5057603436, 5057677886, 5057782565_

