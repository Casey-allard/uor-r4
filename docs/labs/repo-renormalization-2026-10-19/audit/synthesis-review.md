# Adversarial review of `synthesis.md`

Reviewer: read-only audit fleet, 19 Oct 2026. Every finding was checked against the repo at the worktree snapshot (`a15435627`) or the audit inputs. Paths are repo-relative. Audit inputs are under `docs/labs/repo-renormalization-2026-10-19/audit/`. Severity: **HIGH** means a claim is refuted, or the plan breaks an owner rule or decision. **MEDIUM** means a claim is overstated or a negative is left out. **LOW** means a numeric or citation slip.

Tally: 8 HIGH, 12 MEDIUM, 6 LOW.

## A. The shortest path conflicts with owner rules and decisions

1. **HIGH: Stage 5 breaks D41(d).** The synthesis fine-tunes the D37 float trunk from Stage 3 on `distill-d1`. D41 item 2(d) says there is "no student-side exception: training stays the D32 discrete integer path". So a float student may not consume `distill-d1` without a new owner decision. There is a second problem. The cite `eras.md #1958` is an era-26 compliance finding from before D41, not the D41 corpus. Under D41 item 3, `distill-d1` has not been generated, and its sequence (driver math optimization, then a ~1M-token pilot) comes first. Evidence: `docs/integration/DECISIONS.md` D41 items 2(d) and 3; `docs/labs/repo-renormalization-2026-10-19/audit/eras/eras.md` line 51.

2. **HIGH: Stage 2 overturns the owner-approved D39 item 4 sequence without saying so.** D39(4) sets the order: W2 (read-code capture, already FIT'd), then W3 (BVG-K, "registered before any float sheet"), then W4 (float) "only if W2 decides H_law and W3 fails". Stage 2 instead races float against BVG-K and drops W2 entirely. That may be the right call, but it needs an owner decision. The synthesis presents it as reusing built parts. The architect review it cites says the same: W4 "must not start before W2/W3". Evidence: `docs/integration/DECISIONS.md` D39 item 4; `docs/labs/course-correction-2026-10-18/audits/d37-reactivation-architect.md` §6 W3/W4 and line 158.

3. **HIGH: Stage 3 reuses `uor-r4-training`, which D37 and the architect rule out as the float home.** D37 item 6 says a float path is "a separate module" with its own scanned allowlist entry. The architect says W4 runs on "a new scanned module (never `crates/uor-r4-training`)". `code.json` lists `uor-r4-training` as status `held`. It also carries the historical dense and sigmoid stack. Evidence: `DECISIONS.md` D37 item 6; architect §6 W4 line 153; `docs/labs/repo-renormalization-2026-10-19/audit/code.json` (uor-r4-training entry).

4. **HIGH: Stage 4 assumes an exporter that does not exist.** "Export to `uor-r4-integer`" has no path for a Hamilton-gate plus recurrence trunk:
   - The recurrence dossier says "export and qat refuse the mode", and the Hamilton gate "still needs the icosian snap".
   - `training/src/stack_export.rs` is compiled out with `#[cfg(any())]`.
   - The integer schemas /1–/2 serve softmax through exp tables. They are D23 non-compliant.

   So Stage 4 is new build work, not reuse. Evidence: `mechanisms/recurrence-gates.json` rule_compat.D11; `code.json` (uor-r4-lut notes); `mechanisms/lut-serving.json` rule_compat.D23.

5. **HIGH: the plan never mentions the standing holds.** D37 item 6 says "Training, M2 and Stage B stay HELD … releases no hold". ROADMAP lists M2 as `held`. Stage 3 (training) and Stage 5 (M2 bars, ≥256/512 saved) both sit behind these holds, and the synthesis is silent about them. Evidence: `DECISIONS.md` D37 item 6; `ROADMAP.md` lines 155, 178, 191.

6. **MEDIUM: the shortest path never closes M1 as M1 is defined.** D39(1) closes M1 only on "one saved **discrete** rung-0 artifact" that passes all of (a) through (e). Stages 3–5 move to a float trunk, so that artifact cannot close M1 without redefining the milestone, and no stage scores (b), (c) or (e). The synthesis does flag that bar (d) has no unit. It does not flag that the path leaves M1's definition behind. Evidence: `DECISIONS.md` D39 item 1.

7. **MEDIUM: "one release per stage" is presented as settled, but it changes standing gates.** D39(7) says "Every gate is unchanged: no compute without FIT + PM dispatch + owner release where a hold applies". D40 requires owner-typed releases. Merging the per-arm releases is a governance change that needs the owner. Evidence: `DECISIONS.md` D39 item 7, D40.

## B. Claims the evidence refutes or does not support

8. **HIGH: the headline diagnosis, "a 16-bit code holds it back", is contradicted by the K16 outcome record.** That record's own correction says training broke much of the aliasing, with A reaching a Q4 ceiling of about 9,000 rows. Yet A's top-1 "stays at 65 and 463". "So aliasing does **not** explain A's final ranking failure". The architect says 16 bits/vector over a 12-bit vocabulary means "an injective embedding is *possible*". A8 already runs on the injective PrimeZeta codebook (4,096 of 4,096 unique). The case for "Stop tuning admission objectives on a 16-bit code" therefore rests on a cause the project's own records rejected. What the evidence actually leaves open is ranking capacity, the read and the schedule. Evidence: `docs/labs/k16-outcome-call-2026-10-19/README.md` line 59; architect §3.1 line 38; `docs/labs/prime-zeta-construction-2026-10-14`.

9. **HIGH: the Stage 1 bar is already met, so it tests nothing.** "G-Pre ≥ 4,894 at checkpoint 0" was cleared by every K16 arm at checkpoint 0: 14,287 of 14,287 admitted, with only 16 occupied cells. A wider-carrier pre-flight at K≥16 would pass trivially. Stage 1 needs a bar that measures the capacity it claims to raise, for example distinct vectors, alias ceiling, or B3_init/P_succ against A8's 1,351. Evidence: `docs/labs/k16-rows-qualification-2026-10-18/README.md` lines 87–95.

10. **MEDIUM: the position evidence is mislabeled and confounded.** The 0.07-nat result (`transport-attribution-d1`) is about recurrent state transport in an `rrarra` core, not about position. That core has softmax attention layers, a sigmoid/exp gate and dense MLPs, and was run for 1,000 updates in the "lab sandbox". The record itself says identity has 332,928 fewer parameters and asks for "a parameter-matched real-gate control". The recurrence dossier adds that the quaternion arm was "no better than Householder at one paired seed". None of this is lawful-substrate evidence. Evidence: `docs/integration/transport-attribution-d1-2026-09-28.md` §0–1, line 83; `mechanisms/recurrence-gates.json` failed_only_on_configuration.

11. **MEDIUM: the softmax-free read figure leaves out its REJECT.** The +0.0172 BPB (arm W) is recorded as **REJECT**. It missed the 0.01 bar and was 15% slower (195.2 against 228.4 tok/s). Its q/k gradients were the flock-softmax straight-through gradient, and it ran through the host fallback (CPU). The headline arms B and D missed by +0.040 and +0.259. Listing it as "best-evidenced" understates the negative. Evidence: `docs/labs/softmax-free-read-2026-10-10/README.md` lines 5, 19, 23, 145–151.

12. **MEDIUM: the starling claim is mis-cited.** The starling-tree record reports recall "not met" and "the gap to softmax … 0.329 against 0.999". The "3/5 seeds match softmax" result comes from the ROADMAP row and needed a **dense bootstrap**. The tree's salience was also trained against "the dense teacher's mean attention" with a distillation KL. Evidence: `docs/labs/starling-tree-2026-10-12/README.md` lines 4, 15–21, 60; `ROADMAP.md` line 212.

13. **MEDIUM: the D11 arithmetic row overstates the evidence.**
    - The 0.93277 BPB served stack missed M1 ("NOT MET by 0.033").
    - The charted models served 1.20–1.24 BPB.
    - Serving read every weight every token (D5-b non-compliant) and used exp-table softmax reads.
    - The board's own disagreement, that lut-serving should be READY rather than ACTIVE, is dropped.

    §5 says the old stack "reached about 0.88–0.93 BPB", which hides the 1.20–1.24 negative. Evidence: `docs/labs/d11-serving-2026-10-10/README.md` lines 1, 46–49; `mechanisms/lut-serving.json` rule_compat; `mechanisms/board.md` "Where this audit disagrees".

14. **MEDIUM: the stitching map's summary contradicts the map itself.** It says "Every geometric part except position and normalization is built". Its own rows say "Address as the read has never been built", that nothing consumes the object store, that the writer's O(V) scan breaks D5-b, and that no lawful model has been exported. Stage 1's "wider carrier" also needs new `build.rs`/`discrete.cu` code and a new CUDA qualification, so "every stage reuses built parts" is not true. Evidence: `synthesis.md` §1 rows KV, Output head and Arithmetic; architect §3.1 and §3.5; `mechanisms/discrete-learning.json` never_tested ("Full 240-root E8 coverage").

15. **MEDIUM: Stage 3 relies on "the 7–30M scale where the old stack worked" and "1.39B tokens".** That stack never met M1. Its 0.877/0.933 result was trained on the 82M-token chat-v0-p2 split, and the dossier says a larger set was "none … reachable (#1913)". Neither point supports a lawful trunk at that scale. Evidence: `mechanisms/geometric-stack.json` status_reason and never_tested; `docs/labs/d11-serving-2026-10-10/README.md` line 49.

16. **MEDIUM: Stage 3 contradicts the architect's dispositions without saying so.** The architect rates the F32 operators (G2 Hamilton gate, recurrence gates) "RETIRE at tested scope for now", E8 memory "No useful site", and spin weights "RETIRE". The synthesis builds its trunk from these and cites the architect elsewhere. It should state the disagreement and the evidence that overrides the architect. Evidence: architect §5 table rows "F32 operator", "E8 orbit memory", "Spin-native weights".

17. **MEDIUM: "process is the bottleneck" ignores a confound.** On 12–13 Oct, D23/D24/D25/D29/D30/D32 forced a substrate rebuild. D29 moved 33 kernels out of every module. D27 (13 Oct) made Rust-first mandatory, which drove the `*-port` harness work counted as tooling. The drop in the results share after 11 Oct is partly owner-mandated rebuilding, not discretionary process. The counts themselves check out: 34/116/157 for 12–18 Oct, and 190 tooling with 114 harness-governance in W42. Evidence: `DECISIONS.md` D27, D29, D32; `mechanisms/geometric-stack.json` rule_compat.D23; recount over `records/*.jsonl`.

## C. Missed candidates and never-tested items

18. **HIGH: the W2 read-code capacity capture is missing.** It is already FIT'd, costs about $0.91 and rides the same lease. It decides H_rep against H_law, which is exactly the question Stage 2 tries to settle with a ~$15+ race. It also carries the admission-matched count-table arms that the synthesis lists as "never run". Evidence: architect §6 W2 line 139; `docs/integration/NOW.md` sequence row 6 ("queued"); `DECISIONS.md` D39 item 4.

19. **MEDIUM: several open items from the dossiers are left out.** All of the following are cheaper or closer than a new trunk:
    - the already-FIT'd admission-preserving A8 rerun, which is the current M1 work and which the synthesis dismisses as "process";
    - the never-tested ranking-after-admission-intact (RANK_BOTTLENECK) branch;
    - the occupancy-regularised rank objective, the registered route after K16-R;
    - full-epoch and frequent-token revisit schedules (a pass covers 1,049,856 of 2,425,925 targets);
    - the finite relational head, whose Stage M2 prerequisite K16-A satisfies, per the architect, "KEEP on the K16 line";
    - the GPU discrete Hamming binding (VSA) as a lawful KV option.

    Evidence: `mechanisms/admission.json`, `mechanisms/discrete-learning.json`, `mechanisms/rank-margin.json` never_tested; `mechanisms/vsa.json`; architect §5 row "Finite relational head".

20. **MEDIUM: three stage bars are not measurable as written.**
    - Stage 3 says "beat Stage 2's best artifact by more than the noise band" but names no metric. Top-1 from a table-plus-read model and a trunk's score are not obviously comparable.
    - Stage 5's "multi-turn check on fresh rows" has no number.
    - The M2 graders must be deterministic under D28. That needs confirming, since the v5 reply grader is BLOCKED.

    Evidence: `synthesis.md` §4; `DECISIONS.md` D28; `ROADMAP.md` line 191.

21. **LOW: the cheap test #7, the forced-copy oracle on "stored outputs", cannot be run as stated.** The oracle intervenes on a model by supplying the slot. Stored outputs from the softmax-pointer artifact cannot answer an interventional question. It needs a lawful pointer model first. Evidence: `mechanisms/pointer-key.json` never_tested ("Any pointer on the D32 discrete path…"); `mechanisms/board.md` reopen-condition note.

## D. Numeric and citation slips

22. **LOW: "9:1" is closer to 8:1.** W42 non-results against results is 275:34, about 8.1:1, and the 12–18 Oct lab records give 273:34. The 9–11 Oct row recounts as 83 results and 4 governance, not 84 and 5. Evidence: jq recount over `docs/labs/repo-renormalization-2026-10-19/audit/records/*.jsonl`.

23. **LOW: Stage 2 is costed at "about $15".** The architect's estimates are W3 ≈ $12 plus W4 ≈ $5–10 plus a host build, which comes to $17–22 before W2. Evidence: architect §6 W3 and W4.

24. **LOW: the admission-matched comparator is called "$0, host-only (architect §3.4 item 5)".** That item says only that it is "designed in the read-code sheet but unrun". The read-code sheet is the GPU W2 capture. Evidence: architect §3.4 item 5 and §6 W2.

25. **LOW: "four owner-typed releases" is wrong for K16.** Codex typed the four K16 releases, and the owner ratified them afterwards. That is what prompted D40. Evidence: `DECISIONS.md` D40 item 1.

26. **LOW: the top-1 figure is quoted as the best seed only.** The record gives K16-A top-1 as 65 (seed 1) and 463 (seed 2), with admission 2,237 and 3,167, and both seeds fail G-Adm. Quoting only 463 hides a 7× seed spread, and that spread bears on the two-seed bars in Stages 2–3. Evidence: `docs/labs/k16-outcome-call-2026-10-19/README.md` lines 24–26.

## Confirmed (no finding)

These checked out against the evidence:
- K16 accepted moves 407/130/410/129, DEV 463/14,287, and unigram 1,226 and T2 6,300 (`NOW.md`, `host-diagnostics`).
- A8 1,351 → 551 against the control's 857 (`NOW.md`).
- The 5-valued address key and ≈5×10⁶ comparisons per row (architect §3.2 and §3.5).
- The rows kernel at about 56× (`counterfactual-rows-parity`, 55.6–57×).
- Route at 20 proposals in 16,404 (`attention-state-2026-10-18`).
- 19 D-entries from D23 to D41.
- The "failed only on configuration" list matches the dossiers.
