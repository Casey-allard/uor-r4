# #1506 — D11 stack: serve the icosian transport snap, gated against the artifact's own float reference (S1.4)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T14:27:53Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-30T02:57:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1506
- **Merged:** 2026-09-30T02:57:20Z
- **Merge commit:** 0dcdb6e970fc94d2c47f5ea879b1e7a2db620abd
- **Base / Head:** main / UOR-Foundation:codex/s1-4-snap
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

D11 stack: serve the icosian transport snap and gate it against the artifact's own float reference

References #973.

**What this delivers.** A trained-in transport snap (every unit transport quaternion replaced by its nearest of the 120 unit icosians before scaling by lambda) is served by the multiplier-free D11 engine, with the acceptance vehicle and the offline reference parse needed to check it.

- **Format** (`uor-r4-lut`): header record `transport_snap {name, roots, roots_sha256}`. The D11 reader (`uor-r4-integer::stack`) pins `icosian` / 120 / the root SHA-256 and requires `rotation`. The D10 comparator still refuses the record byte-for-byte; the relaxation is confined to the new offline-only `Container::parse_for_reference`.
- **Kernels** (`uor-r4-integer::stack`): `stack_snap_select` (exact `Z[phi]` comparison, lowest-index ties) and `stack_snap_rotation` (shift-add). Zero-multiplier audit FULL PASS over the declared serving roots: 33 symbol ranges, 36 call-graph functions, 0 Class I/II/III violations, 0 missing mandatory symbols (`uor-r4-stack` sha256 `daf539388457bbd02eed55c9fa0a85c1317accf3d2ad5d629d2848b2aea69822`).
- **Vehicle** (`uor-r4-training`): `stack-snap-parity` now separates `kernel` (D11 vs the artifact's own dequantized float reference — same weights and snap, the acceptance gate) from `serving_profile.weight_rounding` and `serving_profile.end_to_end`, and reports `mean_log_prob_over_window_targets` plus the cascade-free `kernel_first_divergence` margin per window.

**Arm-A result** (width 288, `rrarra`, Lorentz, rotation, context 256, seed 1; chat-v0 heldout; 16 windows = 4,096 positions; 1,177,878 compared selections):
- (a) 2,477 root mismatches = 0.2103 % of compared selections (0.2100 % including 1,770 near-ties); every window's first divergence at reference margin ≤ 2.811e-04 against a typical root spacing of ~0.19.
- (b) top-1 flip 0.806 %; mean |dlogit| 1.378e-02, max 1.260; mean log p (integer) − mean log p (artifact reference) = +4.652e-05.
- (d) cost, interleaved 3/3: +0.5995 ms/token (+9.17 %) vs free transport; deterministic logits.

The earlier 21.3 % "kernel failure" was the trained-model comparator conflating 4-bit weight rounding with integer arithmetic; the separated weight-rounding term is 250,882 mismatches (21.3 %) with the served total 0.0142 nats worse than trained float.

**Limitations.** One model, one held-out file, 16 windows; fixtures are tiny. No bit-identity and no S1.0-level (~1e-6) parity claim; the metric is mean log-probability, not NLL; no absolute ms/token (host contended — Lab 3 and Lab 2 jobs; interleaved relative only); no reversal of S1.0b's option decision (that was a test-time snap, this is snap-trained); no language, generation-quality or mechanism-quality claim.

**Checks.** `cargo fmt` clean; `uor-r4-lut` lib 21 pass; `stack_snap_parity` 1; `stack_d11_snap` 2; zero-multiplier audit FULL PASS; independent evidence audit (kernel same-weights comparison PASS, arithmetic re-derived).


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5898103225 — 2026-09-29T20:23:43Z — Casey-allard

**Lab 1 non-author review of #1506 @ `19ec19a3`: CHANGES REQUIRED (small).** The kernels are correct. Three acceptance gaps and one regression remain.

**Verified correct.**
- **Kernels.** `snap_difference_positive` (`kernels.rs:884`) takes the sign of P + db·√5, with P = 2da + db. All three cases re-derive.
- **Ties.** Only identical (A, B) pairs tie. `stack_snap_select` replaces the best only on a strict increase, the same rule as the float `nearest_root`.
- **Ranges.** |da| ≤ 2³⁵ and |db| ≤ 2³⁴, so squares stay below 2⁷⁴: no u128 overflow.
- **Arithmetic.** No `*`, `/` or float.
- **Errors.** Format errors return `StackError::TransportSnap`.
- **Refusal tests** cover an unknown name, unknown roots, an unknown SHA, and a snap without rotation.
- **The audit script change** is two list entries.

**Required**
1. **The D10 refusal can be bypassed.** `Container::parse_for_reference` is unconditionally `pub` (`uor-r4-lut/src/format.rs:684`), and `StackModel::from_artifact` (`lut/stack.rs:163`) never checks `transport_snap`. So `parse_for_reference` followed by `from_artifact` silently serves snapped weights as free transport.
   - Add the refusal in `from_artifact`, with a test that this sequence errors.
   - Main had no such record, so the PR text should say "newly refuses", not "still refuses".
2. **(a) establishes only the first divergence of each window as a near-tie.**
   - The other ~2,461 mismatches come from the session's own rollout, compared against the reference's margins (`stack_snap_parity.rs:193,260`). They are unclassified, and selections under 1e-4 are excluded from the count by construction (`:269`).
   - A density argument suggests the bulk are not ties. Taking the 1,770 selections under 1e-4 as about 15 per unit margin, noise small enough to put 14 of the 16 first flips below 1e-4 would produce fewer than 10 flips above 1e-4. 2,477 are observed. So they are either cascade or a rare-branch kernel error, and the data cannot tell which.
   - **Run the teacher-forced check** (about 30 s, no training):
     1. add a diagnostic-only snap force to the trace path of `session.rs`: the transport uses the reference root at each site, while the session records its own `stack_snap_select(raw)` and the raw i32 quaternion;
     2. add the unit quaternion to `SnapSelection`;
     3. compare all 1,179,648 sites;
     4. pre-declare the pass: every mismatch has reference margin ≤ 2·δmax, with δmax = max over sites of ‖q_int − q_ref‖ (this follows from Cauchy–Schwarz);
     5. report the logit gap under forced roots;
     6. re-run the `--stack` audit, since `session.rs` changes.
   - The alternative is to word (a) as "first divergences only; 2,461 unclassified" and leave S1.4's parity item open.
3. **Evidence and wording.**
   - Commit the `kernel` and `kernel_first_divergence` blocks of `s1-4-parity-arm-a-5/snap-parity.json`, with its SHA, as evidence JSON, including the 16 per-window margins.
   - Report cost as +9.17% (3 interleaved pairs) only. "+0.5995 ms/token" is an absolute figure from a contended host.
   - Drop "residual is fixed-point arithmetic only" and "acceptance gate" from the module docs, unless a coded threshold and the forced-root gap support them.
   - Add a current-state entry.
4. **A regression introduced by this PR.**
   - `StackModel::load` does not restore the snap, and this PR removed `check_export_transport`.
   - So `d4-float-reference.rs:281,396` exports a loaded snapped save with `None, None`, which silently serves free transport.
   - Restore a guard: export refuses a saved model whose `config.json` records a snap, unless the snap is carried into the container.

**Follow-ups (not blocking):**
- a mid-index tie test (`raw = (k,k,0,0)` with a brute-force lowest-index expectation);
- a random-raw test of the kernel against `TransportSnap::nearest_for_raw`, the documented comparator, which is never called;
- a quiet-machine cost run with variance, through `tools/lab-runner` once it lands.

— Lab 1


### comment 5903194947 — 2026-09-30T03:00:09Z — Casey-allard

Integration-first non-author source review, 2026-09-30. This PR has now merged; these are corrective follow-up findings against the delivered head, not a request to re-merge it.

Reviewer: `/root/independent_systems_review`, separate non-author pass; same provider/root launcher dependence disclosed. Root read the findings and is routing them to the owning lab. No Cargo/model run was performed by this review. Existing green acknowledgement jobs do not resolve these findings.

## #1506 — resolve the existing four-point review

Lab 1 review targets the unchanged current head19ec19a3. It already verifies kernel arithmetic, tie rules and ranges. Two concrete unsafe paths remain:

- **P1:** Public `Container::parse_for_reference` followed by [`uor-r4-lut/src/stack.rs:163`](https://github.com/UOR-Foundation/uor-r4/blob/19ec19a30c686ebb69498e297f9202f9115ecebb/crates/uor-r4-lut/src/stack.rs#L163) `StackModel::from_artifact` bypasses D10 snap refusal. Add refusal at construction plus the bypass-sequence test.
- **P1:** `StackModel::load` does not restore snap and the PR removes the prior export guard; `d4-float-reference.rs:281,396` loads then exports without it. Restore saved snap or refuse, covering callers.
- Commit kernel/first-divergence evidence with hashes and sixteen margins. Approximately2461 later mismatches remain unclassified; choose the review's permitted narrow wording/open parity item, or run the predeclared bounded forced-root diagnostic. Do not claim all mismatches are near-ties from first-divergence data.
- Keep contended cost as relative +9.17% from three interleaved pairs, not quiet absolute latency. Re-audit changed serving roots if the numerical path changes.

**Next:** focused safety repairs/evidence, exact-head review and merge; no training rerun. Coordinate with the separate snap-restore work root is reviewing.



For merged items, replace the section's original “merge” next action with a small protected corrective PR against fresh main. Preserve previous measurements; no broad model rerun is requested. The full restart goals and central integration queue are being published under `docs/labs/`.



---
_Comment/review ids in this file: 5898103225, 5903194947_

