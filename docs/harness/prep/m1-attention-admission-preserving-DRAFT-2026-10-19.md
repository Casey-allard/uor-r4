# M1 admission-preserving acceptance rule for the A8 discrete learning path (DRAFT)

- **Status:** DRAFT pre-registration, **TEST FITNESS: PENDING**. No compute, no pod, $0 so far. Authorship transferred to deepseek by the PM (Kimi, #2029 08:32Z) after Claude's 08:30Z deadline passed; Claude keeps the review seat.
- **Line:** M1 learned attention (#2029). **Lab:** deepseek (draft author). **Reviews:** Claude (admission/design); Codex (device/evaluator/state-identity and cost); Antigravity (non-author exact-head review; consumer verifier on request).
- **Parent:** `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md` (FIT, SHA-256 `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`, 12,697 B). Everything not restated here is inherited from the parent unchanged.
- **Why this sheet exists:** A8's Relation arm hit the registered Q1 diagnostic stop: P_succ **551 / 3,847** vs init **1,351 / 3,847**; FrozenZero reached **857 / 3,847** at Q1 and **797 / 3,847** at completion (16,404 batches). The retained-log census verifier (`28fdaade09183b1249e0cfe4daa1745032361fde38f6cb21971c877dada56746`) shows direct cell changes are a minority channel, and the two-sided decomposition puts most of the net admission loss on rows whose **target token's cell did not move** (Relation Q1 65.25%, FrozenZero Q1 67.81%, final 58.30%; descriptive strata, not causal bounds). A table-preserving/Vector-only guard is therefore not sufficient.
- **D21:** the attention line is at D21 count 2; this sheet's run is the decisive **third** step, scored on the acceptance panel. A diagnostic stop is not a mechanism negative; no KEEP follows from this sheet.
- **Rules:** D22, D23/D25/D30 unchanged, D24, D29, D31, D11, D40. No student-side exception; no donor/corpus work.

## 1. Question

Can one pre-registered **admission-preserving acceptance rule** keep the fixed TRAIN-derived anchor's admitted count within a pre-registered margin through Q1 and the full dose, on both A8 arms, while still permitting learning? The rule is the changed variable; the table, arms, data, schedule and read are the parent's.

## Base artifact

None beyond the parent's fresh-model construction. Both arms build a fresh model through the parent's
`PreparedDriver::new_model` with the pinned A8 table; routes, gates, seed and epoch come from the device
initializer. The guard and anchor are execution-layer inputs; they do not alter model parameters, the table,
the read or the mechanism. Parent source/config/table pins are inherited unchanged.

## Data split

The parent's pins are unchanged: TRAIN / DEV and their masks, and the sealed fresh V2 slice. The audit anchor
is drawn **only** from TRAIN: 3,847 genuine TRAIN `P_succ` rows (filtered by `is_copy_successor` in the C=192
in-document window) taken in corpus order from the **unvisited TRAIN tail** at eligible index `1,049,856` onward; no
DEV, heldout or fresh row enters the anchor or the guard, and no anchor row is in the 16,404-batch training prefix. DEV is read only by the parent's inherited evaluation/decision path,
and fresh remains sealed. No new split is introduced.

## 2. Design (PM-binding; no re-litigation)

### 2.1 Gate placement and cadence

**Interval-level gate**, audited every **64 completed batches** (the checkpoint cadence), on both arms, plus a **mandatory final audit at 16,404** covering the 20-batch tail (off cadence); partial terminal states must state their audit disposition. This is the PM disposition: erosion is gradual and visible at 64-batch cadence. The interval placement reuses the checkpoint schedule, but the post-update TRAIN-anchor evaluator is **not** an existing admitted path (Codex 6263219926) and is a source prerequisite in 5; the candidate-level gate is priced below as a successor option, not used here.

### 2.2 Fixed anchor (TRAIN-derived)

- At registration, build a deterministic anchor from the pinned **TRAIN** UORT/mask only: filter eligible TRAIN targets by `attention_evidence::is_copy_successor` in the C=192 in-document window, then take the first **3,847** such rows in corpus order from the **unvisited TRAIN tail** starting at eligible index **1,049,856** (the end of the 16,404×64 training prefix). No anchor row is in the training prefix; no in-sample overlap. This is a TRAIN-domain instrument, **not** the parent's DEV P_succ set; do not relabel TRAIN as DEV or bypass any release guard. Row keys use the canonical `(row_ordinal, target_position, target_token, is_psucc)` order. **Host-derived identity (no model compute):** from pinned TRAIN `tokens.u16` `3dee858f…` / `response_mask.u8` `b5e7352d…`, eligible targets = 2,425,925; the anchor is the first 3,847 `P_succ` rows at or after eligible index 1,049,856 — actual rows 1,049,857…1,064,112, all `is_psucc=1`, SHA-256 of the UTF-8 CSV lines `row_ordinal,target_position,target_token,is_psucc\n` = `8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1` (3,847 lines); no anchor row is in the training prefix.
- Bind: ordered row-key list SHA-256 (target token, in-corpus position), corpus token/mask pins inherited from the parent, and the anchor construction rule in the registration checklist. The anchor is an **instrument input**: the audit never supplies optimization loss, gradients or target selection, and never removes anchor rows from the parent's unchanged TRAIN corpus/order. The TRAIN audit chooses only the stop/inspection state; it is not held-out confirmation.
- At checkpoint 0, evaluate admitted count on the anchor with the **new exact TRAIN-anchor evaluator** (5.2 source prerequisite). Admission uses the same `target_admitted` predicate as the parent's P_succ evaluation (writer admitted-cell/rank rule); no new admission metric is introduced. The **anchor baseline must be >= 500**; otherwise the guard cannot detect an erosion and the run is `INSTRUMENT_UNREACHABLE` (D31), not a negative. The existing `eval-corpus` path is not sufficient: it returns `TrainingHeld` once `accepted_updates != 0` and scores `Split::Dev`, not this TRAIN anchor.
- **Rollback/controller prerequisite (same or a separate exact-head-reviewed source PR):** implement active interval rejection: selective rejection by interval batch (whole interval if the eroding update cannot be isolated), per-update rejection receipts, restore of model/counters/epoch from the last passing checkpoint, schedule-cursor advance past the rejected interval, idempotent gates, and tests proving rejected updates never commit and the next interval starts from the restored state. The run cannot start until evaluator and controller are merged and their source SHAs are bound.
- **Source prerequisite (not optional):** before any run, a separately exact-head-reviewed source PR adds an evaluator that (a) accepts a post-update checkpoint (`accepted_updates != 0`), (b) reconstructs each ordered TRAIN anchor row with its **original context/document boundaries** (no re-tokenization or re-cutting), (c) never reads DEV/heldout/fresh, (d) returns the admitted count for the exact anchor hash under the precise predicate, and (e) exposes the evaluated cursor/epoch/model-checkpoint SHA. The PR carries positive and negative refusal tests: wrong checkpoint/epoch/anchor, pre-update-vs-post-update mismatch, and any DEV/heldout/fresh read. If that source is unavailable or fails, the gate is `INSTRUMENT_UNREACHABLE` (D31); there is no DEV-anchor fallback.
- Baseline anchor admitted count and anchor SHA are recorded in the release packet before any update.

### 2.3 Active interval rejection (PM disposition 09:04Z, Option A)

- Proposed **margin = 24 rows** (a prospective FIT choice; the FIT review may tighten it, but not after observing run results).
- At each 64-batch audit boundary, after the synchronous barrier and exact anchor evaluation:
  - **Pass** (`anchor_admitted >= baseline - 24`): the interval's updates are committed; continue.
  - **Breach**: preserve the failed interval checkpoint and all append-only transition/audit evidence; restore the last passing model/counters/epoch; **reject that interval's updates** (selective per-update rejection if the eroding update can be isolated; otherwise reject the whole interval's updates); record `AUDIT_BREACH` and one rejection receipt per rejected update/interval; advance the schedule cursor past the rejected interval; continue training.
- Rejected updates never commit; `accepted_updates` counts only committed updates. The next interval starts from the restored state with the advanced schedule cursor.
- Audits use a **synchronous barrier**: pause transitions at the audit boundary, pin model storage/snapshot + model epoch + driver cursor + family counters + lineage/plan/release + anchor identity, evaluate that exact state on device, and continue only after a passing receipt or a completed rejection. No update may run concurrently with an audit.
- No audit may be skipped. A missing/unreadable audit is an instrument fault (D31), not a pass.
- Both arms use the **same anchor, rule and rejection semantics**; FrozenZero's lock proof still requires zero relation acceptances. The historical A8 unguarded receipts are reference evidence, not a substitute for this run.

### 2.4 No-op counter-metric (teeth)

A guard that blocks all learning must fail. **Rejection-rate no-op counter:** with `rejected_intervals` and `audited_intervals`, stop `RULE_EFFECTIVELY_NOOP` for diagnosis if either (a) `rejected_intervals / audited_intervals > 0.50` at Q1 or full, or (b) **3 consecutive full-interval rejections** occur. **Precedence:** interval audit/rejection first; at Q1 the parent's inherited Q1 gate applies; the no-op check applies at Q1 and at full/stop. Records include rates, consecutive rejections, accepted updates, and every audit/rejection receipt. The 24-row margin, 50% rate and 3-consecutive bounds are **prospective FIT choices, not inherited MDE consequences**. A no-op stop is not a mechanism negative.

## 3. Bar

**Pre-run gates (inherited, both arms at checkpoint 0):** G-Init (including the unpinned routes/gates/seed/epoch comparison or the recorded waiver), G-Occ, B3_init **1,351 / 3,847** reproduction, full **4,244 / 14,287**, top-1 **133**, Bar(i) **86**, net G-Cap > max(500, Bar(i)). Any difference is `INSTRUMENT_UNREACHABLE` (D31).

**Guard bars:**
1. Anchor baseline >= 500, bound before updates.
2. Active rejection keeps the anchor admitted count within margin at every audit; the rejection rate stays <= 0.50 and no 3 consecutive full-interval rejections occur; both arms complete Q1 and the full schedule.
3. The parent's Q1 admission gate, DEV decision bars and fresh seal remain unchanged. The active rule changes the trajectory and is intended to let Q1 pass where the unguarded run failed; it does not widen any parent bar or add a KEEP claim.
4. No-op counter-metric in 2.4 satisfied.

**Decision:** a no-op threshold breach, a source/instrument fault or a parent Q1-gate failure is a diagnostic stop for analysis, never a mechanism negative. A full-dose completion with the guard and the parent's own bars satisfied is a diagnostic result; KEEP/H_geom/fresh claims remain governed by the parent's 7.7 and the sealed fresh decision. No capability claim follows.

## 4. Cost

- Parent cost basis inherited (~$11.4 full dose both arms at the historical rate; planning figures, not caps).
- **Gate latency is `UNMEASURED` at FIT.** Measuring it requires a bounded qualification/calibration stage with its own applicable FIT/owner-cost approval/PM dispatch, or measurement inside the final authorized run if that run plan includes it. No GPU is rented while this sheet is in assembly/review.
- Parent `$11.4` is dated planning, not a current all-in quote. The qualified costed plan must bind selected hardware/rate, both-arm dose estimate, complete gate overhead (audit/serialization/load/restore/archival, not only device time), setup/custody/close, and provenance. Planned evaluations: **258 per arm including baseline** = 1 baseline + 256 regular 64-batch audits + 1 final 20-batch-tail audit at 16,404; ×2 arms = 516 evaluations. Any partial terminal state has its own stated audit disposition.
- **15% readiness predicate (explicit):** `R = added_gate_cost / ungated_two_arm_dose_cost`, where the numerator is all added gate cost — 516 evaluations at the measured per-eval cost plus added serialization, checkpoint load/restore and archival I/O — and the denominator is the **same-hardware ungated two-arm dose cost** for the same schedule, normal checkpoint I/O and training. Shared setup, custody and close are excluded from both numerator and denominator (common to both). The qualification stage supplies both measured values; readiness requires `R <= 0.15`. If `R > 0.15`, **stop before training and replan**; it never cancels an already healthy authorized dose, and the healthy-run renewal rule prevails. The release packet records the formula, both measured totals and all-in spend.
- Owner caps (<=4 pods, <=$8/h) and disk floors are hard stops.

## Stop rule

- Rejected intervals themselves do not stop the run: they are recorded and training continues from the restored state with the schedule cursor advanced.
- A no-op threshold breach, a missing/unreadable audit, a wrong anchor hash, a failed rollback/evaluator source check, a pre-run gate mismatch or any unexplained status is **D31 diagnosis**, not a pass. The run is non-resumable after such a stop.
- A parent Q1-gate failure is the inherited diagnostic stop.
- Owner caps (<=4 pods, <=$8/h), the disk floors and a failed/diverged run are hard stops; a healthy run is renewed, not cancelled.
- A new rule/run needs a new FIT/registration decision; failed/rejected evidence is preserved.

## Minimum test power (D22 section 3)

- Inherited hidden test panel: 14,287 rows / **3,847 P_succ**; parent's `B3_I >= 200` MDE rule; seed 1 first.
- Guard anchor: **3,847 genuine TRAIN P_succ rows from the unvisited tail** (same size as the hidden P_succ panel;
  `is_copy_successor` filtered), baseline admitted >= 500, evaluated at every 64-batch audit on both arms.
- This is the decisive third step on the line at D21 count 2; a diagnostic stop does not reset the count. No fresh
  transfer is used; no KEEP follows without the parent's H_geom/fresh decision.

## 5. Steps

1. FIT this sheet with fresh-context adversarial review; post every finding.
2. **Source prerequisite:** land the exact-head-reviewed evaluator/provider and active interval-rejection controller from 2.2/2.3 (post-update TRAIN-anchor evaluation, synchronous audit, selective/whole-interval rejection receipts, restore+cursor advance, epoch/checkpoint identity, no DEV/fresh); merge and bind their source SHAs. The source PR(s) must name the exact commands/schema and pin the SHAs; this sheet does not pre-name an implementation path. No run starts without them; unavailable => `INSTRUMENT_UNREACHABLE` (D31), no DEV fallback.
3. Stage the two arms with the parent configs; only the guard/anchor binding is added.
4. PM dispatch with the measured costed plan; Casey's typed D40 release.
5. Pre-run gates and anchor baseline; Q1/full under active interval rejection; mandatory final 16,404 audit before any final evaluation/decision.
6. Decision per 3; record; no fresh use unless the parent's H_geom path applies.
7. Record `docs/labs/admission-preserving-2026-10-19/README.md` and a newest-first current-state entry.

## 6. Out of scope

- Candidate-level gate (priced as successor if interval erosion outruns 64-batch cadence); new kernel work; donor/corpus work; fresh transfer; KEEP/paired-DEV/H_geom claim; any table, arm or mechanism change.

## 7. Registration checklist (when FIT)

1. Post-FIT sheet SHA-256.
2. PM authorship/design disposition reference (this sheet's 08:32Z transfer).
3. Parent sheet SHA and A8 table/config pins.
4. Anchor construction rule, corpus token/mask pins, exact anchor SHA-256 `8c1b80e9…` (3,847 canonical CSV rows, indices 1,049,857…1,064,112), and serialization format.
5. Audit cadence (64), margin (24), synchronous-pause/epoch identity, active selective/whole-interval rejection receipts, restore+cursor-advance semantics, stop rule.
6. No-op counter-metric: rejection rate <=0.50, 3-consecutive bound, precedence table.
7. Evaluator source/commit SHA, tests, original-context/window reconstruction and the 258-per-arm evaluation plan.
8. Synchronous-barrier receipts, model/epoch/cursor/family-counter/lineage/anchor pins, inspection-state identity, attempted-vs-retained batch accounting, and append-only failed-evidence preservation.
9. Qualification-stage measured gate cost, ungated two-arm dose cost, the explicit 15% readiness formula and `R`; hardware/rate/setup/custody/close provenance.
10. D31 classification table and non-resumable stop rule.
11. PM dispatch id and Casey's typed D40 release record.
12. Record/current-state paths.

---

TEST FITNESS: PENDING. Draft author: deepseek; Claude review seat; Codex source/evaluator review; Antigravity non-author review/consumer. No pod is rented and no token or model update is generated by this sheet alone.
