# Pre-registration — D34 successor-table control

## 6.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Authority:** [D34](../../integration/DECISIONS.md:1642-1660); [finite-relational-head](../../mechanisms/finite-relational-head.md); [M4 sheet](m4-finite-relational-head-2026-10-15.md); [relation sub-panel audit](../../labs/relation-subpanel-audit-2026-10-15/README.md).
- **Line:** starling read / exact addressed memory; M4 count **3/3** unchanged; M1 count **0/3** unchanged. No mechanism negative, KEEP or count reset.
- **Delivery:** docs only; all new observations `NOT_RUN`. The Relation arm is blocked on an artifact that does not exist. Q4 **NOT_EXECUTABLE**.
- **Scope:** diagnostic, not a capability claim. Training/M2/Stage B **HELD**. These are not D-Release-class decisions; no owner hold changes.

## 6.2 Question and smallest falsifiable hypothesis

**Question.** Does the geometric Relation read beat a **model-free successor table** on the rows it should own — the 3,847-row copy-successor sub-panel (26.9266% of DEV) — at the same admission and the same training data? The Relation comparison uses the **Relation read's own target-admitted rows (`B3_R`, measured at execution)**, not the Route B3 count. The sealed **435** is the **Route** B3 on `P_succ` and applies only to the Route interim.

**H_geom.** On the `B3_R` `P_succ` rows where the Relation read admits the target cell, `Relation ≥ T2 + floor`: its top-1 exceeds the strongest **context-free** TRAIN successor table (T2) by at least the paired floor.

**H_table.** `T2 ≥ Relation + floor`: the table beats the Relation read by at least the same paired floor; the geometry adds nothing beyond a token→successor lookup at this configuration.

**H_null.** Neither arm beats the **computed `P_succ` unigram (`U_succ`)** by its paired floor; the predictors fail on the sub-panel at this scale (the panel may still be informative), so the sub-panel cannot support a geometry-vs-table decision.

**Minimum-N / observed-power rule.** `B3_R` is measured, not assumed. The primary comparison requires `B3_R ≥ 200`. If `B3_R < 200`, run the exact paired `b+c` **observed-power (post-hoc) guard** at the observed difference: if observed power is <80%, the result is **UNDERPOWERED** and no configuration decision fires; fallback: report the exact 2×2 counts, `U_succ`, and descriptive sub-panel counts, and the lead resolves (authorize more Relation data, accept the Route interim, or defer). A reduced-N comparison at `B3_R < 200` with ≥80% observed power is allowed but labeled reduced-N. This is a guard against over-reading a small matched set, not a prospective power claim.

**Disconfirming observations.**
- H_geom is disconfirmed if `Relation < T2 + floor` (the symmetric within-floor band is INCONCLUSIVE, not a geom win) **or if H_null fires**.
- H_table is disconfirmed if `T2 < Relation + floor` **or if H_null fires**; the symmetric within-floor band is INCONCLUSIVE, not an H_table win.
- H_null is disconfirmed if either arm equals or exceeds `U_succ` + its paired floor.
- The minimum-N observed-power guard takes precedence: no hypothesis fires from an underpowered `B3_R`.

## 6.3 The saturation exposure (the most important design correction)

The proposed context-local "most-recent-successor table" is **tautological** on `P_succ`:
- `P_succ` is defined as: target = `tokens[j*+1]` where `j*` is the most recent earlier occurrence of the query token in the window (`relation-subpanel-audit-2026-10-15/README.md:24-25`).
- The context-local rule applies exactly that definition. It scores **3,847/3,847 = 100%** on `P_succ` by construction.
- The Relation read's `same_token` edge (e=3, `discrete.cu:201-205`) selects the most recent prior same-token node, and its `sequential_successor` edge (e=2, `discrete.cu:199-200`) links a node to the following node. The `P_succ` panel aligns with that copy-successor construction (most recent prior occurrence → its successor), not literally with e=3 alone.

Therefore:
- **T1 (context-local rule) is a panel-definition check, not a comparator.** It must be reported as 3,847/3,847 (or a fault if not), and it must never be used to decide that "the geometry adds nothing".
- **T1b (second-most-recent rule) is the non-tautological control on the most-recent-copy rule itself.** For each `P_succ` row it uses the successor of the second-most-recent earlier occurrence `j**` (the most recent occurrence strictly before `j*`), with a `U_succ` fallback when no such occurrence exists. It cannot be 100% by construction. T1b tests whether a non-defining recency rule can reproduce the result; it is **not** the query-identity control. **T2 is the query-identity/context-free control**: it sees only the query token and a context-free successor distribution, with no window.
- The decisive comparator is **T2**, a **context-free** table built from TRAIN: for each query token `x`, the most frequent successor `y` in the same 1,049,856 TRAIN targets the Relation arm consumed. T2 does not see the DEV window. It is an ordinary, model-free comparator, matched on data and cost but deliberately **not** matched on information: it is context-free.
- The informative rows are the **1,335 conflict rows** (`P_any_succ \ P_succ`, where the target is the successor of some earlier occurrence but not the most recent) and the **10,440 complement rows**. These are **descriptive only and cannot fire a decision**; they are reported because they show where a context-free table and the relational read diverge, not because they are extra decision denominators. The decision uses the `B3_R` matched `P_succ` rows (Relation) or the 435 matched `P_succ` rows (Route interim).

## 6.4 Controls and matching

1. **T1 context-local most-recent rule** — tautological panel-definition check; expected 3,847/3,847. Not a comparator.
2. **T1b second-most-recent rule** — non-tautological control on the most-recent-copy rule; target = `tokens[j**+1]`, where `j**` is the most recent earlier occurrence strictly before `j*`; `U_succ` fallback if none. Report exact counts; it cannot fire the decision.
3. **T2 context-free most-frequent-successor table** — primary ordinary comparator. Pair construction: at each of the 1,049,856 masked TRAIN target positions `p`, count the pair `(x = tokens[p-1], y = tokens[p])`; `y` is the **next raw token**, not the next masked target (which would skip non-target positions). Every `p` is an eligible in-document target, so no pair crosses a document boundary. 4,096 entries; tie by lowest token id; model-matched unigram fallback for unseen query tokens. Conservation sum = **1,049,856**.
4. **T3 last-seen-successor table** — secondary deterministic table on the same pair universe: for each query token `x`, the successor `y` at the **last** masked TRAIN target position with `tokens[p-1] = x`. One entry per seen query token; its count sum is the number of distinct seen query tokens (≤4,096), not 1,049,856. Report both sums.
5. **T4 query-token bigram** — identical to T2 if T2 is the most-frequent successor; report the same table under both names to avoid double counting.
6. **T5 computed `P_succ` unigram (`U_succ`)** — the model-matched unigram built on the same 1,049,856 TRAIN targets and scored on the 3,847 `P_succ` rows. The exact top-1/top-8 must be computed and reported for `P_succ`; the full-panel **1,226/14,287** is the reference only, not the sub-panel value. `U_succ` is pre-registered as a quantity to compute, not as the full-panel number.
7. **Admission-matched comparison** — restrict the table's prediction to the read's admitted cells per row, exactly:
   - `cell(t) = root_id(table[t,0])*16 + root_id(table[t,B-1])` for each token `t`; any `ABSENT` root is an instrument fault;
   - `P_adm = argmax_{t : cell(t) ∈ admitted} count(x,t)`, tie by lowest token id;
   - if no admitted token has a count entry for `x`, use the unigram top-1 restricted to the admitted set; if the admitted set is empty, the row has no prediction (`ABSENT`) and is excluded from the matched denominator;
   - never recompute admission on the host (D24). The same admitted set is used for the table and the read.
8. **Data-matched** — the same 1,049,856 TRAIN targets the Relation arm consumed, not the full 2,425,925.
9. **Cost-matched** — host table (seconds) vs one zero-apply forward (minutes).

## 6.5 Design and dose

**Host-only now ($0 GPU).**
1. Build T1/T1b/T2/T3/T5 from the pinned TRAIN (`3dee858f…`, `b5e7352d…`); build the DEV partition from `0919e90f…` / `a838bb1a…`; verify 3,847 / 5,182 / 10,440 / 1,335 and buckets `[50, 2872, 900, 25]`.
2. Score T1/T1b/T2/T3/T5 on all 14,287 DEV rows and on `P_succ`, `P_comp`, the conflict rows, and the four distance buckets. Emit exact integer counts, per-row records, and the T2 conservation sum.
3. Report T1 = 3,847/3,847 as the panel-definition check; report T1b exact counts (expected <3,847) as the non-tautological control.
4. Compute and report `U_succ` (top-1/top-8) on the 3,847 `P_succ` rows.
5. For the Route interim only, reproduce the sealed Route `P_succ` 29/435 conditional top-1 and 20 strict wins (`relation-subpanel-audit-2026-10-15/README.md:61-64`); the **435 is the Route B3**, not the Relation denominator.

**Relation arm (blocked).**
- No frozen Relation checkpoint exists. `ReadSpec::Relation` is implemented (`discrete.cu:78-267`; `device.rs:341-356`, `:1231-1246`, `:1512-1514`; `lib.rs:76-79`, `116-127`, `140-153`) and qualified by the Stage M1 construction controls **C1–C10**: eight test functions in `relation_qualification.rs` cover C1–C8; C9 is the 17 base `Route`/`Flock` controls; C10 is `admit_ptx` (`m1-finite-relational-head-2026-10-14.md`). The M2 trained-in discriminator is `HELD` behind G1–G5 (`m4-finite-relational-head-2026-10-15.md:8`, `86-100`).
- When a frozen Relation checkpoint at cursor 16,404 exists, run one zero-apply `Device::forward` over the 224 DEV batches with `ReadSpec::Relation { kept: 8, sweeps: 1 }`; capture `predictions` and `admitted_cells` per row; join by ordinal; **measure `B3_R`**; compare on the `B3_R` matched `P_succ` rows. Apply the `B3_R ≥ 200` / observed-power guard before any decision. Cost 5–15 GPU-minutes, <$0.25; one 1-hour lease at $0.908/h = $0.908, one RTX 5090 through `scripts/pod/uor-pod` (Vast.ai).
- If the Relation artifact does not exist, the arm is **`UNAVAILABLE`**, not a negative.

**Optional interim Route arm (executable now).**
- The sealed Q4 Route checkpoint (`4477eb5c…`) has **29/435** conditional successor top-1 and **20** strict wins on `P_succ` (`relation-subpanel-audit-2026-10-15/README.md:61-64`). The **435** is the sealed **Route B3** and is the Route-interim denominator only.
- A zero-apply `Device::forward` over the 224 DEV batches gives the per-row admitted cells and predictions; the same table comparison on the 435 Route-admitted `P_succ` rows is executable now. Label it a **Route-configuration** result, not a Relation result. It answers "does the current trained geometric read beat a context-free table on the rows it admits?", not "does the Relation mechanism?".
- Cost 5–15 GPU-minutes, <$0.25; one 1-hour lease at $0.908/h = $0.908, `uor-pod` Vast.

## 6.6 Bars and uncertainty

- **Relation matched set, `B3_R` rows.** Paired discordance `c − b`; null SE `sqrt(b+c)`; bar = `max(24, 2·sqrt(b+c))`. **24 rows** is a rounded 2-SE floor at 149/14,287 (2 SE = 24.3); the paired `2·sqrt(b+c)` bar is operative; the 24-row floor is the minimum absolute floor for top-1 matched-set comparisons. Require `B3_R ≥ 200` or the observed-power guard. Report the exact 2×2 table.
- **Symmetric one-sided bands.** `Relation ≥ T2 + floor` is H_geom; `T2 ≥ Relation + floor` is H_table; the open band `|T2 − Relation| < floor` is INCONCLUSIVE/UNDERPOWERED. The same floor is used on both sides; there is no zero-row tie bar.
- **Route interim matched set, 435 rows.** Same floor and same symmetric bands; the 435 is the Route B3. Report the exact 2×2 table.
- **3,847-row full sub-panel.** **39-row floor**, the M4 `P_succ` floor (`m4-finite-relational-head-2026-10-15.md:30-31`). It is **top-8-derived and conservative**; report end-to-end counts (29/3,847 for Route; the Relation's number when available) and the table's admission-restricted upper bound.
- **Conflict rows (1,335).** Report exact counts; **descriptive only, cannot fire a decision**; no pre-registered bar beyond the paired 2 SE.
- **Complement (10,440).** Report exact counts; **descriptive only, cannot fire a decision**; the 63-row top-8 floor applies only if a top-8 claim is made on the complement.
- **`U_succ`.** Report the exact `P_succ` unigram top-1/top-8 and its paired comparison against Relation and T2; the full-panel 1,226/14,287 is the reference, not the sub-panel value.
- **Wilson intervals** for every rate.

## 6.7 Decision rule

Apply in order; the first matching row fires. "Floor" means `max(24, 2·sqrt(b+c))` on the matched set; "within-floor" means `|T2 − Relation| < floor`.

| Precedence | Observation | Interpretation at this configuration | Next decision |
| --- | --- | --- | --- |
| 0 | T1 ≠ 3,847/3,847, partition/hash/epoch/apply fault | INSTRUMENT FAULT | Diagnose; do not use the panel |
| 1 | Relation arm unavailable | Commissioned comparison cannot execute | Lead resolves: authorize M2, accept the Route interim, or defer |
| 2 | `B3_R < 200` and the observed-power (post-hoc) guard is <80% | UNDERPOWERED (reduced-N) | Report exact 2×2 and `U_succ`; no configuration decision; lead resolves |
| 3 | Both Relation and T2 ≤ `U_succ` + their respective paired floors | H_null: the predictors fail on the sub-panel at this scale (the panel may still be informative); diagnose the panel/table | Diagnose the table/panel; do not infer a mechanism negative |
| 4 | `Relation ≥ T2 + floor` | H_geom: the geometry adds information beyond a context-free successor lookup | Proceed to the M2 B1/B2/B3 bars; no capability claim yet |
| 5 | `abs(T2 − Relation) < floor` | INCONCLUSIVE / UNDERPOWERED (symmetric within-floor band) | Report the exact 2×2; do not close the configuration; lead resolves whether to power up |
| 6 | `T2 ≥ Relation + floor` | H_table: the table beats the geometry by the paired floor on the rows it should own | Close the `(Relation, kept=8, sweeps=1, C192/B4/V4096, P_succ)` configuration (D22); report the table as the strong baseline; owner decides whether to change the read or the panel |

**What changes the decision.** The single number is the paired difference on the `B3_R` matched `P_succ` rows. If `T2 ≥ Relation + floor`, the configuration is closed and the mechanism family stays open (D22). If `Relation ≥ T2 + floor`, the M2 discriminator proceeds. The symmetric within-floor band `|T2 − Relation| < floor` is **INCONCLUSIVE**, not a closure and not a geom win: report the exact 2×2 and the `b+c`; do not relabel a within-noise T2 edge as H_table. If the artifact is missing, no decision can fire — that is the current state.

## 6.8 Instrument validity

1. **Partition exactness.** 14,287 eligible; 3,847 `P_succ`; 5,182 `P_any_succ`; 10,440 `P_comp`; 1,335 conflict; buckets `[50, 2872, 900, 25]`; zero at `d=1`; `ordered_row_binding_sha256 = e4ef44fb…`.
2. **T1 identity.** T1 must score 3,847/3,847 on `P_succ`; any deviation is a partition/instrument fault.
3. **T1b control.** Report the exact T1b counts; expected <3,847. T1b = 3,847 is not a fault but must be reported and diagnosed as a saturated alternative rule; it never replaces T2 as the comparator.
4. **Table conservation.** T2 counts sum to the 1,049,856 matched TRAIN targets; T3 has one entry per seen query token and its count sum equals the number of distinct seen query tokens (≤4,096); no token ≥ 4,096; deterministic tie/fallback.
5. **`U_succ`.** The `P_succ` unigram is computed from the model-matched 1,049,856 targets and scored on the 3,847 rows; its exact numerator/denominator and top-8 are recorded. The full-panel 1,226 is not substituted.
6. **Relation forward parity and `B3_R`.** If the Relation arm runs, its per-row predictions/admitted cells must match the sealed M2 evaluation receipt; snapshot/epoch unchanged; `updates_applied: 0`; `B3_R` is measured from those rows and reported before the decision table is applied.
7. **Admission match.** The table's admitted-restricted prediction must use the same cells as the read on each row, via `cell(t) = root_id(table[t,0])*16 + root_id(table[t,B-1])` over `{t : cell(t) ∈ admitted}`, tie by lowest token id, unigram fallback; no host recomputation of admission (D24).
8. **Symmetric bands.** The reported 2×2 must place every row in exactly one of `Relation ≥ T2 + floor`, `T2 ≥ Relation + floor`, or `|T2 − Relation| < floor`; a tie is inside the open within-floor band, never a closure.
9. **D23/D25/D32 ratchets** on any new source; the Relation construction remains C1–C10 (`relation_qualification.rs` C1–C8; C9 base controls; C10 `admit_ptx`).

**Which negatives only trigger diagnosis.** A T1 deviation, a partition/hash mismatch, a table-conservation failure, a Relation forward parity failure, or an epoch/apply fault is an instrument fault and triggers repair, not a mechanism negative. A `T2 ≥ Relation + floor` result closes a **configuration**, not the relational attention family; the owner decides on stopping the mechanism. The conflict/complement rows are descriptive and cannot trigger a closure.

## 6.9 Loaded artifacts, lineage and resource projection

| Item | Value |
| --- | --- |
| Base | `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe` |
| DEV tokens / mask | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| TRAIN tokens / mask | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`docs/labs/k16-qualification-restart4-2026-10-16/data-file-sha256.txt`) |
| Tokenizer | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| Partition | 3,847 / 5,182 / 10,440 / 1,335; buckets `[50, 2872, 900, 25]` |
| Route B3 on `P_succ` | **435** (29/435 conditional top-1, 20 strict); **Route interim denominator only, not the Relation denominator** |
| Relation denominator | `B3_R` measured at execution; `B3_R ≥ 200` or the observed-power guard; not 435 |
| `P_succ` unigram | `U_succ` computed from the same 1,049,856 targets and scored on the 3,847 rows; full-panel 1,226 is the reference only |
| Q4 Route checkpoint | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`; evaluation `4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8` |
| Relation implementation | `discrete.cu:78-267`, `:166-218`, `:270-273`; `device.rs:341-356`, `1231-1246`, `1512-1514`; `lib.rs:76-79`, `116-127`, `140-153` |
| Relation qualification | C1–C10: `relation_qualification.rs` C1–C8 (8 test functions), C9 = 17 base `Route`/`Flock` controls, C10 = `admit_ptx` (`m1-finite-relational-head-2026-10-14.md`) |
| Stale brief | `docs/mechanisms/finite-relational-head.md:183` — "zero code exists" is false; corrected 2026-10-17 |
| D34 | `docs/integration/DECISIONS.md:1642-1660` |

**Resource projection.** Host table: **$0 GPU**, seconds, <10 MiB records. Relation arm (when released): one zero-apply forward, 224 batches, 5–15 GPU-minutes, <$0.25; one 1-hour lease at $0.908/h = **$0.908**, one RTX 5090 through `scripts/pod/uor-pod` (Vast.ai). Optional Route interim: the same forward on the sealed Q4 checkpoint, same lease. No training; no new spending class.

**Stop conditions.** First partition/hash/T1/epoch fault stops the arm and preserves the partial record. The Relation arm stops on parity, identity, apply, `B3_R` measurement, resource or disk-floor faults. No on-lease patch or extra sample.

**Record schema.** `uor-r4.successor-table-control/1`, with `applied_updates: 0`, `model_compute_executed: false` (host table) / `true` (Relation/Route forward), per-row predictions, admitted cells, sub-panel labels, the exact 2×2 tables, the symmetric-band assignment, `B3_R`, `U_succ`, T1/T1b/T2/T3 conservation sums, and exact aggregates.

## 6.10 D22, claim discipline, FIT and assumptions

**D22.** A `T2 ≥ Relation + floor` result closes the `(Relation, kept=8, sweeps=1, C192/B4/V4096, P_succ)` configuration, never the relational attention mechanism family. The owner decides on stopping the mechanism. A within-floor result (|T2 − Relation| < floor) closes nothing.

**Claim discipline.** No capability claim. The copy-successor sub-panel is a diagnostic panel; a win is not a general language or reasoning result, and the panel is definitionally aligned with the most-recent-successor rule. T1b and the conflict/complement splits are descriptive controls, not capability evidence.

TEST FITNESS: FIT (design) — Kimi PM seat with independent DeepSeek architecture review, [PR #2572](https://github.com/UOR-Foundation/uor-r4/pull/2572) at `f53c5198a`, copyedits `b67567d01`. FIT authorizes no compute; execution releases are separate PM steps.

**FIT / execution release.** Design FIT only; no compute. Q4 remains **NOT_EXECUTABLE**. The host table is non-model work; the Relation/Route forward needs a separate PM release and, for the Relation arm, the M2 artifact and its own gates. Training remains HELD.

**Assumptions the lead must resolve.**
1. **The Relation checkpoint does not exist.** The lead must either authorize the M2 run (a separate, larger release), accept the Route interim as the executable substitute, or defer this pre-reg. Without one of these, the commissioned comparison cannot execute.
2. Confirm that the **context-free** table (T2) is the intended comparator, not the context-local rule (T1), which is saturated by the panel definition.
3. Confirm that the Relation denominator is **`B3_R` with the ≥200 minimum-N / observed-power guard**, and that the 435-row set applies only to the Route interim.
4. Confirm the T2 pair construction (`(tokens[p-1], tokens[p])` at the 1,049,856 masked TRAIN targets; next raw token; no document-boundary crossing) and the T3 count-sum semantics.
5. Confirm the computed `P_succ` unigram `U_succ` as the sub-panel marginal reference, with the full-panel 1,226 retained only as a reference.
6. Confirm the symmetric one-sided bands: `Relation ≥ T2 + floor` (H_geom), `T2 ≥ Relation + floor` (H_table), and `|T2 − Relation| < floor` (INCONCLUSIVE, no closure).
7. Confirm that a model-free table is an acceptable ordinary comparator under D34, even though it is not a proposed serving mechanism.

## Base artifact

- base: the Relation arm's frozen checkpoint **does not exist** (the arm is UNAVAILABLE; §6.5, §6.7 row 1); Route interim reference Q4 `4477eb5c…`; sealed sub-panel audit [relation-subpanel-audit-2026-10-15](../../labs/relation-subpanel-audit-2026-10-15/README.md). Authoritative artifact table: §6.9.

## Data split

- split: T2/T3/T5 built on the model-matched 1,049,856 masked TRAIN targets; evaluation on the 3,847-row `P_succ` sub-panel with the 10,440-row complement and 1,335 conflict rows descriptive only (§6.2–§6.3). Training data never includes frozen panel rows. No fresh draw: this sheet is diagnostic, not KEEP-class.

## Steps

- steps: §6.5 — host T2/T3/T5/`U_succ` tables ($0 GPU); optional Route interim forward (<$0.25, separately released); Relation zero-apply forward only when a frozen Relation checkpoint exists.

## Minimum test power (D22 section 3)

- seeds: N/A (no training; zero-apply forwards are deterministic at a pinned checkpoint).
- fresh rows: N/A for this diagnostic; no KEEP is claimed, so no fresh transfer draw is required.
- decisive guard: the primary comparison requires `B3_R ≥ 200`, else the exact paired observed-power guard applies and an underpowered result fires no decision (§6.2, §6.6). The bars are never narrower than the panel noise: `max(24, 2·sqrt(b+c))` paired floors throughout.

## Cost (charged cumulatively, before execution)

- preparation: host table construction, $0 GPU (§6.5).
- run: Relation arm 5–15 GPU-minutes, <$0.25; one 1-hour lease at $0.908/h = $0.908 through `scripts/pod/uor-pod` (§6.5, §6.9) — **not currently released**.
- controls / evaluation / retries / storage: included in the same single-lease envelope; archive MD5 round-trip before teardown.

## Stop rule

- stop: Relation arm unavailable → the commissioned comparison cannot execute (§6.7 row 1); instrument faults (init/conservation/hash) → diagnosis, never a mechanism negative (§6.8); owner caps and the disk floor are hard stops. Self-set budgets are estimates, not stop rules.
