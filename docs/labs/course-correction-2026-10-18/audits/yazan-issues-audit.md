# Outside-collaborator audit — Yazan Barakat (`@Cyborg-Hawk-AI`)

**Read-only audit. Nothing posted, nothing modified.** Evidence gathered 2026-10-18 from `gh` and the local checkout at `main` (`21224b446`).

---

## 1. Summary verdict

The **code review was genuinely good**; the **tracker handling was not**. A real outside contribution arrived with real bugs, and the maintainers found and fixed them in detail — the SIMD lookup was wrong, the QUBO PRNG was broken, and LLVM `i32.mul` leakage was real. That is a strong review.

But the disposition of the collaborator's three issues was superficial and partly **factually wrong**:

- All three were closed within 90 seconds of each other, six minutes after the merge (`15:26:33Z`, `15:26:36Z`, `15:26:38Z`), each with one copy-paste "Thank you @Cyborg-Hawk-AI!" comment pointing at the same PR.
- **#2655's closing comment is false on provenance**: it says the `stack_snap_select` / `stack_snap_rotation` path "landed in `origin/main` via PR #2654". Those functions landed on **13 October** in `9556c1bd3` (PR #2320). PR #2654 touched `stack/kernels.rs` only `+2/−2` (a `black_box` change). No parity run and no pre-registration — the issue's actual ask — was ever done.
- **#2657's substantive proposal was never adjudicated.** The D5-a/D5-b split is not in `DECISIONS.md`; `D5` (`docs/integration/DECISIONS.md:244`) is unchanged. The project merged a memo that critiques D5 into `ops/` and then said the "analysis landed".
- **#2656's receipt has an internal arithmetic inconsistency** and a statistically understated CI, yet was accepted without comment.
- The merged delivery also contains a **non-runnable demo**, an **unverifiable paid external run**, and **unintegrated SIMD kernels presented as serving kernels**.

**Professionalism grade: B−.** Code review A−, issue triage C. It is courteous and technically serious, but it is not complete, and an outside contributor would come away unsure what was actually accepted.

---

## 2. Authorship verification

Exactly **four** items across the entire org, all in `UOR-Foundation/uor-r4`:

| Item | Type | State | Created | Closed/merged |
|---|---|---|---|---|
| PR #2654 | PR | merged `966c28519` | 09:58:29Z | 15:20:59Z |
| #2655 | issue | closed | 09:58:37Z | 15:26:34Z |
| #2656 | issue | closed | 09:58:42Z | 15:26:36Z |
| #2657 | issue | closed | 09:58:43Z | 15:26:39Z |

- No GitHub login `yazan` exists (searches returned zero). The profile is **Yazan Barakat**, bio "founder of Bare AI", account created 2025-03-22, one follower. `author_association: CONTRIBUTOR` on both the PR and issues. Branch prefix `bai/wasm-serving-runtime` ("Bare AI").
- **No harness claim or ledger entry exists** for him: `grep -i "bai\|yazan\|cyborg" docs/harness/claims.json docs/harness/ledger/*.jsonl` returns nothing.
- The squash merge `966c28519` is **authored by "Yazan Barakat <cyborghawkai@gmail.com>"** even though commits `aa5c21d91` and `c921e0b56` (the actual hardening) are by Casey Allard. Git history therefore credits the collaborator with maintainer-written fixes — a provenance wrinkle the project otherwise cares about.

---

## 3. PR #2654 — what actually happened

**Timeline**

- `06:43`–`09:58Z`: nine commits by "Cyborg Hawk AI", including an unposted `handoff/work-cards.md` draft (`667db6749`).
- `10:06Z`: owner relays Antigravity/ag1's `CHANGES_REQUESTED` as a plain PR comment (#2654 comment **6248567721**), not a review.
- `15:11`–`15:15Z`: three maintainer commits (`aa5c21d91`, `9d6a31be9`, `c921e0b56`) rewrite the code.
- `15:18:59Z`: formal `APPROVED` review (**5563647788**) by `Casey-allard`.
- `15:20:59Z`: merge.

**What was genuinely good**

The `CHANGES_REQUESTED` review is excellent. It correctly identified:

- `swizzle_dot_arm_t` using `i8x16_swizzle` over individual bytes of the i16 table and truncating `i32` entries — the kernel was wrong, not just slow.
- `dot_dot_arm_m` computing `Σ n_c · table[c]` instead of `Σ table[n_c]` (`crates/uor-r4-integer/src/stack/simd.rs` at the reviewed head).
- `Lcg::next_f64` dividing a 31-bit value by `u64::MAX` — this was real and is now fixed (`xtask/src/qubo_codec.rs:110-112`, dividing by `1u64 << 31`, with test at `:142`).
- i64 mul leakage in `stack_recurrence`, later fixed with `black_box`.
- A hardcoded `sudo` password and `/home/yaz/loop.sh` in `ops/measure_final_rnd.py`, correctly removed.

The final `simd.rs` is mathematically correct: four unsigned/signed byte planes, `i8x16_swizzle`, pairwise `extadd`, recombined with `<<8/16/24`; `dot_dot_arm_m` now delegates to Arm T. The PRNG test spans the unit interval. These are real wins.

**What was not resolved**

1. **The SIMD kernels are not wired into serving.** `swizzle_dot_arm_t`, `dot_dot_arm_m` and `byte_plane_swizzle_dot` have **no callers outside `simd.rs`'s own tests** (verified by grep across `crates/`). `stack_gemv_pairs` (`crates/uor-r4-integer/src/stack/kernels.rs:675`) still uses the scalar pair-table path. So the 7/7 zero-mul audit never exercises them, and the "Arm T/Arm M dense-map kernels" are **fixtures, not the hot path**. The lab record (`docs/labs/wasm-serving-runtime-2026-10-18/README.md`) does not say this.
2. **`simd.rs`'s module header is now false.** Lines 13–17 still advertise Arm M as "the same inner loop with `i32x4.dot_i16x8`, a hardware multiplier … reported beside Arm T under their D12". The merged code deleted that arm (`simd.rs:200-205` delegates). The claimed D12 evidence row does not exist.
3. **The D29 objection was accepted, not resolved.** `serving_root.rs:47,51,54` still declares `model`, `sampling` and `tables` inside the `serving-wasm` build. The reviewer's own standard was "presence counts, not only execution." The merge relies on the feature defaulting off plus link-time removal. That may be defensible, but it was never recorded in `DECISIONS.md` or the lab record, so the standard was silently changed.
4. **PR body is stale.** As merged, the PR still says "5/7 serving symbols PASS" and "The one open problem … step FAILS by 2 LLVM muls". The approval fixed it to 7/7; the body was never updated. A reader of the merged PR page sees the wrong state.
5. **No claim, no pre-registration, no work card.** The branch had no `docs/harness/claims.json` entry, and the collaborator's `handoff/work-cards.md` — which *did* follow the project's card shape — was deleted rather than helped into the process.
6. **Scope creep.** One PR carried an M7 serving runtime, an M5 energy instrument, a D5 decision memo, a QUBO solver, and an AWS receipt. The review flagged this; the merge kept the non-#1172 material anyway.

---

## 4. Per-issue assessment

### #2655 — "S1.4: 2I icosian transport serving kernel — pre-registration"

**Technical content: mostly right premise, wrong attribution.**

- The snap functions exist at `crates/uor-r4-integer/src/stack/kernels.rs:966` and `:991`, are called from `stack/session.rs:2137-2140`, and are exported from `stack/mod.rs:78`. The issue's file path ("`uor-r4-core/src/native_geometric/learner/group_table.rs`") is wrong — that file exists but does not hold these functions.
- The parity record is real and the numbers match exactly: `docs/evidence/s1-4-snap-parity-2026-09-29.json` gives `top1_agreement = 0.991943359375`, `root_mismatches = 2477`, `selections_compared = 1177878`. Good citation.
- The audit command is real: `scripts/audit_zero_matmul_serving.py` documents `--stack` at line 13, and `uor-r4-stack` exists (`crates/uor-r4-integer/Cargo.toml:63`).

**Roadmap fit:** belongs to the closed S1.4 line (#1506 / #1533), not M4. Redirecting to #2032 is defensible for the *parity* half, but #2032 is "one served model, D11 and CLI"; the S1.4 framing is lost.

**Handling: incorrect.** Closing comment **6251665759** claims the snap path landed via #2654. It did not; it pre-dated that PR by five days. The parity run was not performed, and **no pre-registration exists** (`docs/harness/prep/` contains nothing matching snap/wasm/2655).

**Follow-up owed:** yes — a correction, and a real S1.4 successor or an explicit "parked" statement.

### #2656 — "Energy protocol v2: a second-instrument RAPL measurement"

**Technical content: honest, but with a verifiable arithmetic defect.**

- The cited owner baseline is real: #820 comment **5862960224** reports 0.00413 and 0.00096 J/token, matching the issue's "4.13 vs 0.96 mJ/tok".
- `ops/measurement_report_rnd.json` reports `gross mean 0.003172 J/tok`, `17,190 tok/s`, `work_rate_w 40.5`, `idle_rate_w 27.307`, `net 0.00153 J/tok`, `ci95 3.7e-05`.

  **The fields disagree.** `0.003172 × 17,190 = 54.5 W`, not 40.5 W. The net figure is consistent with gross minus idle (`1.530 mJ × 17,190 = 26.3 W`; `26.3 + 27.307 = 53.6 W ≈ 54.5 W`). So `work_rate_w: 40.5` is the outlier — an unexplained 25% gap. `ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md` §2.3 repeats it.
- **CI is understated.** The three gross values (3.135, 3.172, 3.199) have sample sd ≈ 0.0321 mJ. A proper t-based 95% CI for n=3 is **±0.080 mJ**, not ±0.037. The reported figure is exactly the z-based `1.96·sd/√3` — wrong distribution for three samples.
- Scope caveats are genuinely good: synthetic 15,968-weight stack, spawn-inclusive, DRAM NOT MEASURED, not the 7.15M main line.

**Roadmap fit:** M5 (#2033) is right for the *instrument*, but M5's headline is "measured J/token, RSS and tokens/s on the M1 with selected parameter access". A 15,968-weight fixture on an x86 VM is not that, and the issue says so itself.

**Handling:** comment **6251666162** is factually fine ("receipt landed") but never addresses the arithmetic, the CI, or where the protocol should live. The protocol was placed in a **new top-level `ops/` directory** that is not in `docs/PROJECT_MAP.md` and not part of `docs/evidence/`.

**Follow-up owed:** yes — reconcile or annotate the receipt; either adopt the protocol into M5 properly or mark it provisional.

### #2657 — "D5 split: D5-a residency vs D5-b sparsity, with a QUBO formulation"

**Technical content: the most substantive item, and the least handled.**

- The critique is coherent and cites the project's own first-principles review correctly (ROADMAP R5: "Bytes touched per token set the energy floor", `ROADMAP.md:197`).
- The memo `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md` is internally inconsistent: it defines `D[c1,c2]` (geometric distance between cells) in the setup but the objective never uses it, penalising only same-cell collisions. Either the distance is unnecessary (then drop it) or the formulation is incomplete.
- The memo says "The solver is the same classical SA as the codec QUBO (`xtask/src/qubo_codec.rs`)". **That is false.** `GroupQubo` (`xtask/src/qubo_codec.rs:26-65`) solves per-weight round-up/down with a squared-output-error objective. It has no assignment variables, no one-hot constraints, and no co-access matrix. The D5-b solver does not exist.
- The memo's D18 gate reference is stale: D18 was "in force for 1–14 October" (`DECISIONS.md:687-689`), and its fortnight restrictions were later superseded (`:826`).
- The Braket receipt `ops/braket_qaoa_toy_receipt.json` is **not QAOA**: the circuit is `H q[0]; CNOT q[0],q[1]` — a Bell state. `bell_fidelity: 1.0` is at best the Z-basis even-parity fraction (52/48), not a state fidelity. The receipt is unverifiable (no hash/signature), and its cost line implies paid AWS spend while the lab record claims "**$0 new paid resource**". The project's compute rule authorises external compute only through `uor-pod`; a Braket task is a new paid class.

**Roadmap fit:** the issue is a proposal to amend D5 and therefore belongs in `DECISIONS.md`, not M5. M5 consumes a D5 design; it does not decide one.

**Handling: incomplete.** Comment **6251666544** says "D5-a/D5-b analysis landed" — the memo landed; the **decision did not**. The project now has an unratified critique of D5 sitting in a merged file, with the roadmap still using the unsplit D5. A future reader can easily mistake the memo for adopted policy.

**Follow-up owed:** yes, and it is the most important one — ratify or reject the split explicitly.

---

## 5. PR #2669 and the closed loop

`#2669` (merged `9b234ae40`) restored `docs/wasm-serving/wasm_serve_demo.html`, `docs/wasm-serving/verify.mjs` and `ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`, corrected symbol/path references, and released nine `antigravity/*` claims. It did **not** close the loop:

- `wasm_serve_demo.html:33` imports `./pkg-serving/uor_r4_integer.js`; `.gitignore:67-68` ignores `/pkg-serving/`. `verify.mjs:2,4` imports `./uor_r4_integer_bg.wasm` and `./uor_r4_integer.js` from the same directory. Neither file exists there. **From a clean clone, both restored files are non-runnable**, and the lab record gives no wasm-pack output command or artifact hash.
- `#2669` also restored the energy protocol but did not reconcile its numbers.
- `#2669` released `antigravity/*` claims, but no claim ever existed for `bai/wasm-serving-runtime`.

---

## 6. Dangling / inaccurate references in the repo

| Location | Problem |
|---|---|
| `ROADMAP.md:149` | M7 "not started" / "local API only" — a WASM serving runtime merged today |
| `README.md:103` | Same stale M7 row |
| `STATUS.md` | No mention of the WASM runtime at all |
| `docs/integration/DECISIONS.md:244` | D5 unchanged; no D5-a/D5-b split recorded |
| `docs/integration/DECISIONS.md` (D29) | No entry recording the accepted `serving-wasm` interpretation |
| `crates/uor-r4-integer/src/stack/simd.rs:1-19` | Header describes an Arm M `i32x4.dot_i16x8` arm the code no longer contains |
| `docs/labs/wasm-serving-runtime-2026-10-18/README.md:13,26` | Describes SIMD kernels as delivered; does not say they are unintegrated; claims demo/verify "preserved" though they cannot run |
| `ops/measurement_report_rnd.json` | `work_rate_w` inconsistent with gross/net; CI z-based on n=3 |
| `ops/braket_qaoa_toy_receipt.json` | Labelled QAOA, contains a Bell state; paid AWS class unaddressed |
| `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md:28` | Claims the codec solver applies to D5-b; it does not |
| `docs/PROJECT_MAP.md` | Does not index the new top-level `ops/` directory |
| `docs/integration/project-track.md:1393` | #1172 listed as a responsibility; no status update anywhere |
| #2032 / #2033 / #2035 | **Zero comments** from this delivery, despite three closing comments saying "tracked on …" |
| `#2655` comment `6251665759` | False provenance claim (snap path "landed via PR #2654") |
| PR #2654 body | Still says 5/7 and an open `step` FAIL after the merge fixed it |

**Accurate and good:** `README.md:389` acknowledges "Yazan Barakat (@Cyborg-Hawk-AI): research and development work on the WebAssembly D11 integer serving runtime (#1172, PR #2654)". `docs/integration/current-state.md:33` is a fair, well-scoped entry. `docs/labs/wasm-serving-runtime-2026-10-18/README.md` is honest about the model-quality scope.

---

## 7. What an outside contributor would experience

Reading these threads as Yazan:

1. I open a substantial PR with honest caveats, including a named failure I could not fix.
2. An agent review arrives as a **comment**, not a review, and lists five blocking findings — including that my SIMD kernels are wrong and my PRNG is broken.
3. Nobody asks me to fix anything. Five hours later the code is rewritten under my branch, merged, and the merge commit carries **my** name as author.
4. I never get a single message saying "we rewrote your kernel; here is what changed and why" — only a public `APPROVED` review that opens "Reviewed @Cyborg-Hawk-AI's … and applied targeted maintainer hardening directly".
5. Six minutes after the merge, my three issues are closed in 90 seconds with three near-identical thank-you notes. One tells me my snap kernel "landed via this PR" when it has been in `main` since 13 October. Another tells me my D5 split "landed" when the decision record still says the opposite.
6. My work-card draft — the project's own required artifact — is deleted from the tree without a word about how to post it.
7. Nothing appears on #2032/#2033/#2035, where the closing notes say the work is now tracked.
8. The demo I built is restored to `docs/` but cannot run from a clone.

That is polite, fast, and technically competent — and it is not a loop closed. The contributor cannot tell what was accepted, what was rejected, or what to do next.

---

## 8. Concrete follow-up actions for the PM

**Must do (accuracy and integrity)**

1. **Correct #2655.** Post a follow-up comment: the snap path landed in `9556c1bd3` (#2320, 13 Oct); PR #2654 changed only `black_box` in `stack/kernels.rs`; no parity run and no pre-registration were done. Then either reopen as an S1.4 successor or open a new issue under #2032 naming the required artifacts (`docs/evidence/s1-4-snap-parity-2026-09-29.json`, S4-arm model, `chat-v0` held-out set) and the exact command.
2. **Adjudicate the D5 split in `DECISIONS.md`.** Either add D5-a/D5-b (and update `ROADMAP.md:197` R3 and M5 #2033), or add one line to D5 and to `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md` saying it is a proposal, not adopted. Do not leave it ambiguous.
3. **Fix or annotate `ops/measurement_report_rnd.json`.** Reconcile `work_rate_w` (implied 54.5 W from gross × throughput) or remove the field; replace the z-based ±0.037 with a t-based ±0.080, or mark the CI as "z-based, n=3, understated". Mark the receipt provisional if it is not re-run.
4. **Decide the Braket receipt's status.** It is a Bell-state parity check, not QAOA. Relabel it, attach the task hash/signature, and record whether the paid AWS run was authorised. Correct the "`$0 new paid resource`" claim in the lab record and `current-state.md:13`.
5. **Fix the D29 record.** Add a short entry (DECISIONS or the lab record) stating that `serving-wasm` compiles `model`/`sampling`/`tables` and that compliance rests on the default-off feature plus link-time removal — or remove those modules from `serving_root.rs`. Do not leave the reviewer's blocking finding silently downgraded.

**Should do (documentation and loop-closing)**

6. **Make the demo runnable or mark it non-runnable.** Add the wasm-pack build command and output path to `docs/labs/wasm-serving-runtime-2026-10-18/README.md`, or commit a pinned bundle with its hash. Right now `docs/wasm-serving/*` cannot execute from a clone.
7. **Label the SIMD kernels honestly.** Either wire `swizzle_dot_arm_t` into `stack_gemv_pairs` or state in the lab record and `current-state.md` that they are unintegrated fixtures pending integration. Fix the stale Arm M header in `simd.rs:13-17`.
8. **Update stale status rows:** `ROADMAP.md:149`, `README.md:103` (M7: component landed, not qualified), and consider a STATUS line. Index `ops/` in `docs/PROJECT_MAP.md` or move the files into `docs/evidence/` and `docs/mechanisms/`.
9. **Post the handoff on #2032, #2033 and #2035.** The closing comments claim the work is tracked there; currently those issues have no comment from this delivery.
10. **Correct the PR body of #2654** (or add a pinned comment) so it no longer states a 5/7 audit and an open `step` failure that the merge fixed.
11. **Fix the QUBO memo's solver claim.** `ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md:28` says the codec SA solves D5-b; it cannot. Either write the assignment solver or say it is unimplemented.

**Nice to do (relationship)**

12. **Write Yazan a real reply on #2654 or by email**: what was accepted, what was rewritten and why (the SIMD kernel was wrong, the PRNG was broken, the mul leakage was real), what remains open (parity, D5 decision, demo build), and an invitation to take one of the follow-ups. Thank him specifically for the snap-parity record citation and the honest caveats.
13. **Explain the work-card/claim path** he tried to follow and had deleted, so his next contribution can be posted rather than removed.
14. **Note the squash-commit attribution** in the lab record: `966c28519` is authored by Yazan but contains maintainer-written hardening; `git blame` will misattribute it.

---

## 9. Grade and specific gaps

**Tracker handling: B−.** Sub-scores:

| Dimension | Grade | Reason |
|---|---|---|
| Code review quality | A− | Found four real, non-obvious bugs; verified fixes at exact heads; hardened security issues |
| Responsiveness | B | Same-day review and merge; but a 5-hour gap with no dialogue |
| Issue triage | C | Mass-close in 90 seconds; one false provenance claim; one unadjudicated decision proposal; no milestone comments |
| Accuracy of the record | C+ | Stale PR body, false #2655 claim, contradictory $0/Braket claims, stale Arm M docs, stale M7 rows |
| Contributor experience | C+ | Courteous but non-dialogic; PR rewritten without discussion; work card deleted |
| Follow-through | C | "Tracked on #2032/#2033" is asserted but not done; demo restored but broken; D5 split neither adopted nor rejected |

**The single biggest gap:** treating a merge as a disposition. For code, that is right — the code is in `main`. For a decision proposal (#2657) and an unrun experiment (#2655), a merge cannot close the issue, because the thing the issue asked for is a *decision* and a *measurement*, neither of which exists. Those two should be reopened or explicitly re-scoped, with the collaborator told which.

**The single easiest fix:** one paragraph to Yazan on #2654 describing what changed in his code and what is still open. That single message would convert a B− into an A− engagement outcome.