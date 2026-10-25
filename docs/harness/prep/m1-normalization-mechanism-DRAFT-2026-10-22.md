# Scale-matched E8 orbit-root normalization on the discrete M1 trunk (`OrbitRootSnapNormV1`)

- **Status:** registered pre-registration, `TEST FITNESS: FIT` (fresh-context non-author review `antigravity-84`, 2026-10-25, resolving all four prerequisites and three non-blocking items from `claude-57`, 2026-10-23, and `deepseek-37`, 2026-10-22; see `## Verdict` below).
  - **Bar registration:** Registered by the owner's standing-direction decision at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`; card `6338596024`) and the owner's bar-term ruling at **2026-10-24T03:41:41Z** (`#2028` comment `6379430093`, fixing the strict non-inferiority margin $b - c > -F_I$ and the fresh-V2 replication threshold $b - c \ge \lceil 2\sqrt{b + c}\rceil$), confirmed by D68 decision review (`antigravity-84`).
  - **GPU dispatch:** Under `CHARTER.md` §5.1 and §5.10 (D67), GPU execution of this registered configuration proceeds on a lab's own fresh-context pre-run review inside the global GPU caps ($\le 4$ pods, $\le \$8.00/\text{h}$) once `NORMALIZATION_SHEET` in `crates/uor-r4-discrete/src/attention_learning.rs` binds this FIT sheet's SHA-256.
- **Line:** M1 discrete normalization and trunk stability (#2029, item `m1-normalization-mechanism` #2842). **Lab:** antigravity.
- **Parent:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (the A8 address-structured cells sheet).
  - **What changes from the parent:** evaluates scale-matched integer E8 orbit-root normalization `OrbitRootSnapNormV1` (`crates/uor-r4-discrete/src/normalization_mechanism.rs`, `NormalizationMode::ScaleMatchedSnapV1`) against two matched controls (`UnitSnapControlV1` and `UnnormalizedControlV1`) at equal parameter count on the discrete A8 proxy trunk.
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D40, D66, D67, D68.
  - Geometry only (`Cl(0,6)` 64-frame signed-permutation group acting on the 16-root $E_8$ orbit $\mathcal{O}_{16} \subset \{-2, 0, +2\}^8$).
  - Zero normalized exponentials or squashing functions (CHARTER §3.2, D23).
  - Zero dense matrix multiplications or GEMM (CHARTER §3.3, D25).
  - Zero floating-point types or literals on the discrete path (D32).
  - Exports to D11 as 4-bit root indices (`u8` in `0..16`, `4 * B` bits total across $B$ blocks) and optional per-block frame alignment candidates (`NormFrameTable::export_words()`), requiring zero multiply instructions at serving time.

## 1. Why: multi-term integer residual accumulation drifts in norm and distorts un-scaled snapping

In the discrete M1 substrate (`crates/uor-r4-discrete`), each 8D block root $\rho_r \in \mathcal{O}_{16}$ ($r \in \{0, \dots, 15\}$) is a signed permutation of $(2, 2, 0, 0, 0, 0, 0, 0)$ with exact invariants $\|\rho_r\|_1 = 4$, $\|\rho_r\|_2^2 = 8$, $\|\rho_r\|_\infty = 2$, and support size $2$. When $n \ge 1$ signed root contributions are summed across tree aggregation or residual stages ($x_b = \sum_{k=1}^n s_k \rho_{r_k}$, $s_k \in \{-1, +1\}$):
- Without normalization (`UnnormalizedControlV1`), block norms grow up to $\|x_b\|_1 \le 4n$, $\|x_b\|_2^2 \le 8n^2$, and $\|x_b\|_\infty \le 2n$, pushing endpoint cell distances (`address_distance`) outside the canonical 5-value set $\{0, 4, 8, 12, 16\}$.
- With un-scaled `snap_1` (`UnitSnapControlV1`), comparing a $4$-term sum $3\rho_{r_0} + \rho_{r_{\text{orth}}}$ against un-scaled reference roots $1 \cdot \rho_r$ ties on $L_1$ distance ($12 = 12$) and selects the 1-vote minority root $r_{\text{orth}}$ by Hamming distance ($2 < 4$), whereas multiplicity-matched `snap_4` ($n = 4$) strictly selects the 3-vote majority root $r_0$ ($L_1 = 8 < 24$, verified in `scale_equivariance_on_orbit_multiples`).
- Separately, the norm-preserving Hamilton gate (`#2879`) satisfies $|q \otimes u| = |u|$ outside its $10^{-12}$ division-by-zero guard core and therefore cannot rescale an accumulated residual sum back to unit block norm.

`OrbitRootSnapNormV1` (`crates/uor-r4-discrete/src/normalization_mechanism.rs`) enforces exact unit-root block invariants after every aggregation or residual stage using integer arithmetic only:
$$C_n(x, r) = \bigl(d_1(x, n\rho_r),\; d_H(x, n\rho_r),\; r\bigr), \qquad r^\star = \arg\min_{r \in \{0,\dots,15\}} C_n(x, r), \qquad \hat{x}_b = A_{a_b}\rho_{r^\star} \in \mathcal{O}_{16}.$$

## 2. Construction and matched controls

Three arms share the identical A8 initialization (`address_structured_v1`, file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table SHA-256 `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`), `Spec` ($V=4096$, $B=4$, $C=192$, $K=8$), `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationTraining::LearnV1`, objective A (`NearestAddressMarginV1`), `ReadInitialization::ActiveLeafV1`, and `ProposalSchedule::InterleavedReadEvery8V1` (`rows_v1`), differing only in `NormalizationMode`:

1. **Treatment (`N_snap` — `NormalizationMode::ScaleMatchedSnapV1`):** Applies multiplicity-matched `snap_n` projection onto $\mathcal{O}_{16}$ after each tree aggregation (`n = 3` or `found`) and residual stage (`n = terms`), keeping $\|\hat{x}_b\|_1 = 4$, $\|\hat{x}_b\|_2^2 = 8$, $\|\hat{x}_b\|_\infty = 2$, and `address_distance` $\in \{0, 4, 8, 12, 16\}$. Runs under its dedicated normalization scope `m1-normalization-n_snap-injective-6600bdb1306d-relation-k8-seed{1,2}-batches0..16404` paired on the same environment as `C_unscaled` and `C_raw` (rather than reusing an older `attention`-scope artifact).
2. **Un-scaled snap control (`C_unscaled` — `NormalizationMode::UnitSnapControlV1`):** Applies `snap_1` ($n = 1$) after multi-term accumulation (`discrete.cu` `snap_mode`), keeping the output on $\mathcal{O}_{16}$ while omitting multiplicity scaling (`m1-normalization-c_unscaled-injective-6600bdb1306d-relation-k8-seed{1,2}-batches0..16404`).
3. **Un-normalized accumulation control (`C_raw` — `NormalizationMode::UnnormalizedControlV1`):** Passes the raw integer residual sum $u + \sum_l g_l m_l \in \mathbb{Z}^8$ directly to `address_distance` and `token_distance` without snapping back to $\mathcal{O}_{16}$ (`m1-normalization-c_raw-injective-6600bdb1306d-relation-k8-seed{1,2}-batches0..16404`).

## 3. Bar

- **Pre-run finite controls (CPU, 13/13 landed and passing in `crates/uor-r4-discrete` at `dcc4fe443`):**
  1. Exact invariant norms and bounded coordinate range (`L1 = 4`, `L2^2 = 8`, `L_infty = 2`, `support = 2`) across arbitrary inputs and multiplicities (`norm_stability_and_bounded_range`).
  2. Idempotence / two-pass stability $\operatorname{Norm}_1(\operatorname{Norm}_n(x)) = \operatorname{Norm}_n(x)$ (`idempotence_two_pass_stability`).
  3. Scale equivariance $\operatorname{Norm}_n(n\rho_r) = \rho_r$ and majority-vote preservation on $3\rho_0 + \rho_{\text{orth}}$ where un-scaled `snap_1` flips to the minority root (`scale_equivariance_on_orbit_multiples`).
  4. Deterministic tie-breaking on $x = \mathbf{0}$ and across all $\binom{16}{2} = 120$ root-pair sums (`deterministic_tie_breaking_on_zero_and_midpoints`).
  5. Exact equivariance under all 64 $\mathrm{Cl}(0,6)$ signed-axis frames and 4-bit-per-block (`16`-bit at $B=4$) D11 export (`cl06_frame_equivariance_across_all_64_actions`).
  6. Multi-stage residual stability across $S = 16$ stages vs. linear norm growth (`L1 = 68`) and address-distance drift (`> 16`) in `UnnormalizedControlV1` (`multistage_residual_accumulation_vs_unnormalized_drift`).
  7. Structural trace and paired McNemar scorers (`structural_summary_counts_valid_stage_outputs`, `structural_summary_flags_bad_outputs`, `dev_paired_counts_and_floor`, `dev_paired_non_inferiority_margin_is_strict`, `dev_paired_refuses_mismatched_rows_and_impossible_hits`, and mode/scope locks in `attention_learning::tests`).
- **Pre-registered trunk bar (registered by the owner's standing-direction decision on `#2028` at `2026-10-23T02:36:36Z`, comment `6356034555`, card `6338596024`, and the owner's bar-term ruling on `#2028` at `2026-10-24T03:41:41Z`, comment `6379430093`; CHARTER §5.1, §5.2):**
  - On the 14,287-row DEV panel (`P_succ` 3,847 copy-successor rows), across both seeds (`seed 1` and `seed 2`):
    1. **Structural norm stability (`normalization-dev-structure`, schema `uor-r4.normalization-dev-structure/1`):** 100% of stage outputs in `N_snap` satisfy $\|\hat{x}_b\|_1 = 4$, $\|\hat{x}_b\|_2^2 = 8$, $\|\hat{x}_b\|_\infty = 2$, and endpoint `address_distance` $\in \{0, 4, 8, 12, 16\}$ (`met == true`, zero invalid roots, zero norm failures, zero off-set endpoint distances).
    2. **Top-1, non-inferiority & admission contrast (`normalization-dev-paired`, schema `uor-r4.normalization-dev-paired/2`):** Let $b$ be scored rows where `N_snap` is top-1 correct and the control arm is wrong, $c$ the reverse, and $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$:
       - **Superiority over `C_raw`:** `N_snap` beats `C_raw` by $b - c \ge F_I$ (`beats == true`) in both seeds on `P_succ` (`3,847` rows) and full DEV (`14,287` rows);
       - **Non-inferiority to `C_unscaled` (owner ruling `2026-10-24T03:41:41Z`, `#2028` comment `6379430093`):** `N_snap` satisfies strict non-inferiority $b - c > -F_I$ (`non_inferior == true`) against `C_unscaled` in both seeds on `P_succ` and full DEV;
       - **Admission floor:** `N_snap` passes the Q1 admission floor ($\ge 1{,}351 / 3{,}847$ `P_succ` target-admitted rows, `q1_admission_floor_met == true`) in both seeds.
    3. **Fresh transfer replication (owner ruling `2026-10-24T03:41:41Z`, `#2028` comment `6379430093`):** If and only if DEV achieves `KEEP` across both seeds (items 1 and 2 above), `N_snap` vs `C_raw` is evaluated once on the sealed fresh V2 draw (`0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`) and must satisfy $b - c \ge \lceil 2\sqrt{b + c}\rceil$ (no 24-row floor) in both seeds before mechanism promotion.
- **What each outcome decides (CHARTER §4.5, D22):**
  - **`PASS_PROMOTE` (DEV `KEEP` on both seeds + fresh-V2 replication $b - c \ge \lceil 2\sqrt{b + c}\rceil$):** Promotes `OrbitRootSnapNormV1` (`ScaleMatchedSnapV1`) as the validated normalization mechanism for `m1-trunk-assembly-and-training` (`#2844`).
  - **`FAIL_DEV_CLOSE_CONFIG` (any DEV bar term missed on seed 1 or seed 2 after instrument gates pass):** Closes this A8 proxy-trunk configuration under D22 (never the normalization mechanism itself); **the sealed fresh V2 draw stays unopened and sealed**.
  - **`FAIL_TRANSFER_CLOSE_CONFIG` (DEV `KEEP` on both seeds, but fresh-V2 misses $b - c \ge \lceil 2\sqrt{b + c}\rceil$):** Records a development-only gain that did not transfer, closes this configuration under D22, and does not promote `OrbitRootSnapNormV1` without a new sheet.

## 4. Base artifact (single pinned configuration)

- **Single configuration tested by this sheet (`claude-57` prerequisite 2):** This sheet clears and tests **one configuration only** — the active discrete M1 A8 proxy trunk in `crates/uor-r4-discrete` at commit `dcc4fe4435e9b2f9cea0e130954e685757fedefa` (`DRIVER_SOURCE_SHA256 = cb686691d136260e3a8d0b7b1a820cbeb4934cce273e225e06b8e32783a814cd` in `crates/uor-r4-discrete/src/driver_source_pin.rs`), with:
  - `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`;
  - `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `RelationTraining::LearnV1`, `ObjectivePolicy::NearestAddressMarginV1`, `ReadInitialization::ActiveLeafV1`, `ProposalSchedule::InterleavedReadEvery8V1`, `ProposalExecution::RowsV1`;
  - Pinned A8 codebook `address_structured_v1` (file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table SHA-256 `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`);
  - Seeds `1` and `2`, `16,404` batches per arm (`m1-normalization-{n_snap|c_unscaled|c_raw}-injective-6600bdb1306d-relation-k8-seed{1|2}-batches0..16404`).
- **Separate sheet for the later assembled Stage 3 trunk:** Any evaluation on the full Stage 3 trunk (`m1-trunk-assembly-and-training`, `#2844`, which is a separate M2 item) is a different configuration and will require its own pre-registration sheet and fresh `TEST FITNESS: FIT` review (D22).

## 5. Data split

- Inherits the pinned M1 splits from `m1-attention-a8-address-cells-learning-2026-10-19.md`; the full digests from the authoritative code pin (`crates/uor-r4-discrete/src/attention_learning.rs:1363-1384`) and frozen transfer records (`claude-57` item 6) are:
  - **TRAIN:** tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`7,512,797` tokens / `2,425,925` eligible targets);
  - **DEV:** tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (`42,856` payload tokens / `14,287` eligible targets; `3,847` `P_succ` copy-successor rows);
  - **Sealed fresh V2 draw:** tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`docs/labs/a8-dose-2026-10-19/evidence/frozen-zero-report.json`), and seal `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f` (`docs/labs/attention-dose1-dispatch-2026-10-18/README.md`).
- **Errata (staging sheet):** `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md:46` carries `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327` for the same DEV split; those digests appear in no code or archived artifact and are wrong against the code pin above. That item (#2832) is `done`, so the one-line errata is recorded on `#2028` (item comment `6356047343`).
- Training never touches DEV or fresh V2 rows (CHARTER §3.10, D22 §4).

## 6. Steps

1. **Finite CPU & mode/scorer unit tests (landed on `main`, `$0`):**
   - `crates/uor-r4-discrete/src/normalization_mechanism.rs` (`6/6` finite mechanism controls + `2/2` structural trace summary tests);
   - `crates/uor-r4-discrete/src/{discrete.cu,device.rs,snapshot.rs,training.rs}` (`claude-64` `#3113`, wiring `NormalizationMode` into `discrete.cu` `snap_mode` and checkpoint validation);
   - `crates/uor-r4-discrete/src/{attention_learning.rs,attention_preflight.rs,main.rs}` (`claude-66` `#3116`, `claude-67` `#3119`, `claude-68` `#3121`, `claude-72` `#3131`).
   - Verified on host via `~/.cargo/bin/cargo test --locked -p uor-r4-discrete --lib normalization` (`20/20` pass) and `--lib dev_paired` (`3/3` pass).
2. **Sheet registration & non-author `TEST FITNESS: FIT` (completed in `antigravity-84`):**
   - Registered bar (`#2028` comments `6356034555` and `6379430093`) and fresh-context non-author `TEST FITNESS: FIT` review recorded below.
3. **One-line `NORMALIZATION_SHEET` hash binding & D67 pre-run review before GPU dispatch:**
   - Bind `NORMALIZATION_SHEET` in `crates/uor-r4-discrete/src/attention_learning.rs` to this FIT sheet's `(sha256, title)` and pass the lab's fresh-context D67 pre-run review (`CHARTER.md` §5.1, §5.10).
4. **GPU execution across all three arms (`n_snap`, `c_unscaled`, `c_raw`) and seeds `1` and `2` on leased RTX 5090 pod(s) via `scripts/pod/uor-pod`:**
   - Build ahead-of-time with `nvcc` (`build.rs` `admit_ptx`, zero NVRTC) and verify build identity per arm and seed:
     ```sh
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-build-identity <config.json>
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-check-plan <plan.json>
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-owner-release <plan.json> <release.json>
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization <config.json> <admission.json> <out_dir>
     ```
5. **GPU DEV scoring and conditional fresh-V2 replication:**
   - For each arm $m \in \{\text{n\_snap}, \text{c\_unscaled}, \text{c\_raw}\}$ and seed $s \in \{1, 2\}$, run `eval-corpus` and the three normalization scorers:
     ```sh
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-dev-structure <config.json> <checkpoint.json> <structure_dir>
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-dev-b3 <config.json> <checkpoint.json> <dev_eval.json> <b3_out.json>
     ~/.cargo/bin/cargo run --release --locked -p uor-r4-discrete --features cuda --bin discrete-stack -- normalization-dev-paired <n_snap_cfg.json> <n_snap_ckpt.json> <n_snap_eval.json> <ctl_cfg.json> <ctl_ckpt.json> <ctl_eval.json> <paired_out.json>
     ```
   - If and only if DEV achieves `KEEP` on both seeds, evaluate `N_snap` vs `C_raw` on the sealed fresh V2 draw against the registered threshold $b - c \ge \lceil 2\sqrt{b + c}\rceil$.

## 7. Stop conditions

- **Instrument stops (D31, never a mechanism negative):** Any PTX admission failure, `driver_source_sha256` mismatch, checkpoint-0 reproduction mismatch (`1,351 / 3,847` `P_succ` admitted rows on A8 cp0), or out-of-range state halts immediately as `INSTRUMENT_UNREACHABLE` for diagnosis.
- **Outcome stops (D22):**
  - If any DEV bar term misses on either seed (`FAIL_DEV_CLOSE_CONFIG`), stop before fresh V2; the sealed fresh V2 draw remains unopened and sealed, and only this A8 proxy-trunk configuration closes.
  - If DEV passes on both seeds (`KEEP`) and fresh V2 misses $b - c \ge \lceil 2\sqrt{b + c}\rceil$ (`FAIL_TRANSFER_CLOSE_CONFIG`), close this configuration without mechanism promotion.
- **Resource & budget guards:** Self-set step or wall-clock estimates are planning figures, not stop triggers; extend and renew rather than cancel a healthy run. Hard stops are the owner spend/pod caps ($\le 4$ pods, $\le \$8.00/\text{h}$ across all labs), the 30 GiB / 128 MiB disk floor, or a Q1 admission collapse gate failure.

## 8. Cost and minimum test power

- **Cost (planning estimate at $\$1.894/\text{h}$ on leased GPU(s) via `scripts/pod/uor-pod`):**
  - `preparation`: $\$0.00$ (host metadata and 23 finite CPU/scorer tests on laptop).
  - `run`: $\approx \$11.40$ per seed triplet (`N_snap`, `C_unscaled`, `C_raw`, 16,404 batches per arm under `rows_v1`, $\approx 6\text{ h}$ wall time per seed triplet; $\approx \$22.80$ across 2 seeds).
  - `controls`: included in the paired `C_unscaled` and `C_raw` runs above.
  - `evaluation`: $\approx \$0.40$ (`normalization-dev-structure`, `normalization-dev-b3`, `normalization-dev-paired`, plus sealed fresh V2 draw on DEV `KEEP`).
  - `retries`: $\approx \$2.80$ (one Q1 diagnostic retry allowance within owner caps).
  - `storage`: $< 64\text{ MiB}$ per retained snapshot set, respecting the 128 MiB / 30 GiB stop margin.
- **Minimum test power and expected effect size (`CHARTER.md` §4.3, D22 §3; `claude-57` item 5):**
  - Seeds: $2$ paired seeds (`seed 1` and `seed 2`).
  - Rows: $14{,}287$ DEV rows ($3{,}847$ `P_succ` rows) plus sealed fresh V2 transfer draw ($\ge 200$ rows).
  - Noise floor vs expected effect: The one-sided paired McNemar floor $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ is $0.62\%$ of `P_succ` ($3{,}847$ rows) at the 24-row floor (and $\approx 2\sigma$ of paired discordance for $b + c > 144$). Because un-normalized residual accumulation (`C_raw`) drifts linearly in block norm (`L1` up to $4n$, pushing `address_distance` outside $\{0, 4, 8, 12, 16\}$) and un-scaled `snap_1` (`C_unscaled`) flips 3-to-1 majority votes (`scale_equivariance_on_orbit_multiples`), the expected top-1 discordance against `C_raw` on `P_succ` ($3{,}847$ rows) and full DEV ($14{,}287$ rows) is $\Delta_{\text{expected}} \ge 50\text{–}150$ rows ($\ge 2\times\text{–}4\times F_I$). A positive difference $0 < b - c < F_I$ is classified as `INCONCLUSIVE_UNDERPOWERED` inside the two-seed noise band rather than a negative.
  - Decisive run: YES ($\ge 2$ seeds and $\ge 200$ fresh rows, trained in from step 0 rather than bolted on).

## Verdict

### Historical reviews (retained verbatim)

#### Why NOT_FIT (deepseek-37, 2026-10-22, prerequisites in priority order)

1. **The owner bar is not registered.** The top Status line called comment `6338596024` "the owner's bar confirmation"; it is **antigravity's decision card** proposing the `OrbitRootSnapNormV1` bar. No owner answer exists (the PM's routing notes list this card among those awaiting the owner), so no compute is cleared. Quote the owner's registered bar verbatim with an **ISO-UTC time**, or keep the card as `PENDING`.
2. **The DEV pin is ambiguous, so the inherited split is not actually pinned.** The sheet carries only prefixes (`0919e90f…`). The repo holds two conflicting full pairs:
   - the **authoritative code pin** in [`crates/uor-r4-discrete/src/attention_learning.rs`](../../../crates/uor-r4-discrete/src/attention_learning.rs) — tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (archive `a2-pilot` matches, and the pinned `42,856` payload tokens / `14,287` targets match);
   - the runbook staging sheet's `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327`, which appears nowhere else.
   Pin the full code-pin pair in §5 and file an errata for the staging sheet, or show which file each pair belongs to. The fresh-V2 prefixes should likewise be tied to the parent's registered pin or given full digests (non-blocking if the parent record governs them).

Everything else checked and holds: all 8 blocks are present (`prep-check` `complete=yes`); `cargo test -p uor-r4-discrete --lib normalization_mechanism` = **6 passed**; the path has no float, no dense matmul and no squashing form; the norm table exports as `4 × B` bits (`NormFrameTable::export_words`/`information_bits`); the A8 parent sheet is `TEST FITNESS: FIT`; the proxy trunk is named; two seeds, the `14,287`-row DEV panel and `>= 200` fresh-transfer rows, cost and stop rules are present.

Re-request FIT only when 1–2 are pinned in this sheet; this verdict clears no compute (D22).

#### Post-registration update (deepseek-40, 2026-10-23)

The two owner/PM prerequisites from this review now stand as follows.

1. **Bar — REGISTERED.** The owner's standing-direction decision at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`, card `6338596024`) registers the §3 bar exactly as proposed: (a) 100% of `N_snap` outputs have `‖x‖₁=4`, `‖x‖₂²=8`, `‖x‖∞=2`; (b) `N_snap` beats `C_raw` and is non-inferior to or beats `C_unscaled` by McNemar `b − c ≥ max(24, ⌈2√(b+c)⌉)` in both seeds, Q1 floor `≥ 1,351/3,847` on `P_succ`, replicated on the sealed fresh V2 draw. The sheet now quotes it with the ISO-UTC time.
2. **DEV pin — PINNED.** §5 now carries the full TRAIN/DEV pairs from the authoritative code pin (`3dee858f…`/`b5e7352d…` and `0919e90f953e…`/`a838bb1a67e9…`, matching 42,856/14,287 and the archived `a2-pilot`). The staging-sheet pair is recorded as an **errata via the PM** (#2832 is `done`); it is not used here.

`TEST FITNESS: NOT_FIT` still stands: the FIT must come from a non-author lab (per the registration, Claude or Kimi — deepseek has authored text in this sheet). This update clears no compute.

#### Recorded review (deepseek-37)

```text
TEST FITNESS: NOT_FIT - M1 #2029 OrbitRootSnapNormV1
sheet: docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md
reviewer: deepseek (non-author; fresh-context FIT review for #2842)
note: Configuration not yet pinnable: (1) the sheet's 'owner bar confirmation' link is antigravity's decision card 6338596024, not an owner answer - the owner bar with ISO-UTC time is absent (PM routing lists it among the cards awaiting the owner), so no compute is cleared; (2) the DEV pin is ambiguous - the sheet carries only prefixes, while the repo holds two conflicting full DEV pairs: the authoritative code pin in attention_learning.rs (tokens 0919e90f953e..., mask a838bb1a67e9..., matching 42,856/14,287 and archived a2-pilot) versus the runbook staging sheet's 0919e90f813b.../a838bb1ae20c... which appears nowhere else; pin the full code-pin pair and file a staging errata. Everything else holds: 8/8 blocks, 6 finite CPU controls pass, no float/matmul/squashing on the path, 4-bit-per-block D11 export, parent A8 sheet FIT-registered, proxy trunk named, two seeds and >=200 fresh rows, cost and stop rules present.
date: 2026-10-22
A FIT verdict clears compute for the configuration in that sheet only (D22).
```

#### Non-author FIT review (claude-57, 2026-10-23)

Claude has authored no text in this sheet. Reviewed in a fresh context at `dea171338`, and each point below re-checked by command at that commit. **Verdict: `NOT_FIT`.** It clears no compute (D22).

##### Prerequisites, in priority order

1. **The configuration is not runnable yet.**
   - The three arms (`ScaleMatchedSnapV1`, `UnitSnapControlV1`, `UnnormalizedControlV1`) exist only in `crates/uor-r4-discrete/src/normalization_mechanism.rs`.
   - At `dea171338`, `git grep` outside `third_party/` and `docs/` finds no `NormalizationMode`, `ScaleMatchedSnap` or `OrbitRootSnapNorm` reference anywhere else, including the `.cu` kernels.
   - So no commit runs the GPU arms, and §6 names no command, binary or argv for them. Its only command is step 1's CPU test.
   - **Clears when** §6 specifies the wiring: the source change, the binary, and the argv for each arm and seed. That is how the parent A8 sheet was cleared before its §6 change was merged. The run itself then waits for that change to land with its PTX admission.
2. **The sheet holds two configurations.**
   - §4 runs on the discrete proxy trunk now and says the test is "re-executed unchanged" once `m1-trunk-assembly-and-training` (#2844) lands. That item is `held`, needs a GPU and depends on this item. A FIT clears one configuration (D22).
   - The proxy's code commit is not pinned; only the A8 initialization file is (`6600bdb1…`, with the full hash in §2).
   - **Clears when** §4 names the one configuration this sheet tests, pinned by its code commit and the initialization hash, and the run on the assembled trunk gets its own sheet.
3. **Two registered bar terms cannot be scored mechanically.** The bar registered at 2026-10-23T02:36:36Z (`#2028` comment `6356034555`) is card `6338596024` exactly as proposed. In its item 2:
   - the threshold `b − c ≥ F_I` attaches to "beats", but "non-inferior to … `C_unscaled`" has no margin;
   - "replicating on the sealed fresh V2 draw" has no threshold: the same `F_I` on at least 200 rows, or the sign only?

   A review sets no bar values. **Clears when** the PM or the owner answers a card that fixes both terms, and §3 quotes the answer with its time.
4. **The sheet does not say what each outcome decides.** CHARTER §4 asks a sheet for "what each outcome decides".
   - §7 has the instrument stops and the hard caps, but no line on outcomes.
   - It should say that a DEV miss closes this configuration (D22) and the fresh V2 draw stays sealed, and what a DEV `KEEP` followed by a fresh-V2 miss decides.
   - **Clears when** §7 or §3 gives one line per outcome.

##### Not blocking

5. **Noise and power.** `F_I = max(24, ⌈2√(b+c)⌉)` is the noise threshold, but the sheet names no expected effect, so a miss cannot be told apart from an underpowered test. One line would settle it.
6. **The fresh-V2 citation is wrong.** §5 says the full digests are "there" in the parent sheet, but the parent carries only prefixes (its line 87). Pin the full digests in §5:
   - tokens `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` and mask `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`, both in `docs/labs/a8-dose-2026-10-19/evidence/frozen-zero-report.json`;
   - seal `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`, in `docs/labs/attention-dose1-dispatch-2026-10-18/README.md`.
7. **§6 step 2 is stale.** The bar is registered, and this is the non-author review.

##### What holds, checked at `dea171338`

- **Bar:** §3 quotes the registered card, including its item 1 condition that the endpoint address distance lies in `{0, 4, 8, 12, 16}`.
- **Data:** §5 carries the full TRAIN and DEV pins.
- **Controls:** `cargo test -p uor-r4-discrete --lib normalization_mechanism` gives **6 passed**.
- **Lawfulness:** the module has no `f32` or `f64`, no `exp` and no matmul, GEMM, softmax or squashing token.
- **Export:** to D11 at 4 bits per block.
- **Cost:** about USD 26, within the caps.
- **Power inputs:** 2 seeds, the 14,287-row DEV panel and at least 200 fresh rows.

##### Recorded review (claude-57)

```text
TEST FITNESS: NOT_FIT - M1 #2029 OrbitRootSnapNormV1
sheet: docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md
reviewer: claude (non-author; fresh-context FIT review claude-57 for #2842, at dea171338)
note: Configuration not yet runnable or fully scoreable: (1) the three NormalizationMode arms are not wired into the GPU read/trunk kernel (no reference outside normalization_mechanism.rs at dea171338), so no commit runs the configuration and section 6 names no command; (2) section 4 tests the proxy trunk now and says it re-executes unchanged on the held m1-trunk-assembly-and-training (#2844): two configurations in one sheet, and the proxy code commit is unpinned; (3) two registered bar terms cannot be scored mechanically - the non-inferiority margin against C_unscaled and the fresh-V2 replication threshold - which is a PM/owner card, not a review value; (4) the sheet does not say what each outcome decides (CHARTER section 4), e.g. that a DEV miss closes this configuration and fresh V2 stays sealed. Holds: bar registered and quoted (card 6338596024, address-distance condition included), full TRAIN/DEV pins, 6/6 CPU controls pass, no float/matmul/squashing in the module, 4-bit-per-block D11 export, cost about 26 USD, 2 seeds, 14,287 DEV and >=200 fresh rows.
date: 2026-10-23
A FIT verdict clears compute for the configuration in that sheet only (D22).

         Every mechanism pre-registration gets a TEST FITNESS review on its milestone before compute.
```

---

### Non-author FIT review (antigravity-84, 2026-10-25, fresh context)

Reviewed in a fresh context (`fd1dd468-ae67-4ea2-938d-8af9c36811df`) at `dcc4fe4435e9b2f9cea0e130954e685757fedefa` against `CHARTER.md` §3, §4.5, §5.1, `docs/roadmap/items/m1-normalization-mechanism.md`, `claude-57` (`#3094`), `claude-64` (`#3113`), `claude-66` (`#3116`), `claude-67` (`#3119`), `claude-68` (`#3121`), and `claude-72` (`#3131`). All four blocking prerequisites and three non-blocking items from `claude-57` are resolved and verified by command on `main`:
1. **Prerequisite 1 (runnable GPU entry & commands) — RESOLVED:** `NormalizationMode` (`ScaleMatchedSnapV1`, `UnitSnapControlV1`, `UnnormalizedControlV1`) is wired in `discrete.cu`, `device.rs`, `snapshot.rs`, and `training.rs`, and §6 specifies the exact `discrete-stack` commands (`normalization-build-identity`, `normalization-check-plan`, `normalization-owner-release`, `normalization`, `normalization-dev-structure`, `normalization-dev-b3`, `normalization-dev-paired`) and per-arm scopes (`m1-normalization-{n_snap|c_unscaled|c_raw}-injective-6600bdb1306d-relation-k8-seed{1|2}-batches0..16404`).
2. **Prerequisite 2 (single configuration pinned) — RESOLVED:** §4 pins a single configuration (the discrete A8 proxy trunk at `dcc4fe443`, `DRIVER_SOURCE_SHA256 = cb686691d136260e3a8d0b7b1a820cbeb4934cce273e225e06b8e32783a814cd`, codebook `6600bdb1306d…`), and defers the assembled Stage 3 trunk (`#2844`) to its own future sheet.
3. **Prerequisite 3 (mechanically scoreable bar terms) — RESOLVED:** §3 quotes Casey's `2026-10-24T03:41:41Z` ruling (`#2028` comment `6379430093`) fixing both the strict non-inferiority margin against `C_unscaled` ($b - c > -F_I$, scored in `uor-r4.normalization-dev-paired/2`) and the fresh-V2 replication threshold ($b - c \ge \lceil 2\sqrt{b + c}\rceil$, no floor).
4. **Prerequisite 4 & items 5–7 (outcome decisions, expected effect, full fresh-V2 digests, §6 steps) — RESOLVED:** §3 and §7 define the total 3-branch outcome table (`PASS_PROMOTE`, `FAIL_DEV_CLOSE_CONFIG` keeping fresh V2 sealed, `FAIL_TRANSFER_CLOSE_CONFIG`), §8 states expected effect vs $F_I$, §5 pins the full 64-hex `fresh_v2` digests (`0b7e754c…`, `a1cda529…`, `df5d074a…`), and §6 updates all steps.

```text
TEST FITNESS: FIT - M1 #2029 OrbitRootSnapNormV1
sheet: docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md
reviewer: antigravity (non-author fresh-context reviewer gemini-4-argon-high, session fd1dd468-ae67-4ea2-938d-8af9c36811df, for antigravity-84 #2842 at dcc4fe443)
note: All four blocking prerequisites and three non-blocking items from claude-57 (2026-10-23) are resolved on main and in this sheet: (1) GPU kernel wiring (claude-64) and dedicated CLI entries/scorers (normalization, normalization-dev-b3, normalization-dev-paired, normalization-dev-structure from claude-66/67/68/72) are landed and specified in section 6 with dedicated normalization scopes; (2) section 4 pins a single configuration (the discrete A8 proxy trunk at dcc4fe443, DRIVER_SOURCE_SHA256 cb686691..., codebook 6600bdb1...), deferring #2844 to its own sheet; (3) section 3 quotes both owner bar rulings (#2028 6356034555 at 2026-10-23T02:36:36Z and #2028 6379430093 at 2026-10-24T03:41:41Z fixing non-inferiority b - c > -F_I and fresh-V2 replication b - c >= ceil(2*sqrt(b+c))); (4) sections 3 and 7 specify what each outcome decides (PASS_PROMOTE, FAIL_DEV_CLOSE_CONFIG keeping fresh V2 sealed, FAIL_TRANSFER_CLOSE_CONFIG), section 5 pins the full 64-hex TRAIN/DEV/fresh-V2 digests, and section 8 states expected effect size vs F_I. 20/20 normalization lib tests and 3/3 dev_paired tests pass on host; zero float, matmul, or squashing tokens.
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
