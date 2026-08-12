# RH Bridge Candidate B: Formal Lemma Stack (Frozen v2026-02-17)

Generated: February 17, 2026
Constants file: `/Users/adminamn/Documents/New project/research/output/rh_bridge_candidate_b_constants_2026-02-17.json`

## 1. Frozen Canonical Object
Define `gamma_j > 0` as zeta-zero ordinates. For cutoff `M`:

\[
F_M(u)=\sum_{j\le M}\frac{e^{i\gamma_j u}}{\tfrac12+i\gamma_j},\qquad
F'_M(u)=\sum_{j\le M}\frac{i\gamma_j e^{i\gamma_j u}}{\tfrac12+i\gamma_j}.
\]

For `n>=2`, set
\[
X_n=(\Re F_M(\log n),\Im F_M(\log n),\Re F'_M(\log n),\Im F'_M(\log n)).
\]

On wheel class `gcd(n,W)=1`, channels are `Y11s(n), Y20(n), su2_trace(n), phase_vel(n)`.

Frozen bridge statistic:
\[
G(n)= -Y11s(n)-Y20(n)-phase\_vel(n).
\]
(`su2_trace` coefficient is frozen to `0`.)

Define cumulative bridge process on sample points `x`:
\[
H_W(x)=x^{-1/2}\sum_{n\le x,\,gcd(n,W)=1}G(n).
\]

Residual target:
\[
E(x)=\psi(x)-x.
\]

## 2. Canonical Empirical Regime (to be proved, not assumed true asymptotically)
Frozen batch winner:
- kernel: `none`
- zero cutoff: `M=64`
- bases: `W in {30,210,2310,30030}`
- scales tested: `x<=10^6`

Measured transfer fit (`/Users/adminamn/Documents/New project/research/output/hx_transfer_lemma_probe_batch_none_s0_z64.json`):
- global slope `a = -0.00975423378904867`
- global intercept `b = -0.10078074955107089`
- pooled RMSE `0.20485466184815526`
- pooled corr(`aH+b`, `E/sqrt(x)`) `0.27683236410990925`
- empirical envelope `C_emp = 0.6051556269873964`

`n=10^6` per-base corr(`H`, `E/sqrt(x)`):
- `W=30`: `-0.28875839503243117`
- `W=210`: `-0.2896771192456138`
- `W=2310`: `-0.2899860888826384`
- `W=30030`: `-0.29095481903686515`

Prime-label gate result (`/Users/adminamn/Documents/New project/research/output/spinning_top_r4_prime_label_probe_candidate_b_t20_strict.json`):
- global pass = `True`
- transfer score = `2.312329588275253`
- all four bases pass.

## 3. Truncation/Tail Control Targets
Using gaussian kernel(scale=100) reference study at `n<=200000` and `M_ref=512`:
`/Users/adminamn/Documents/New project/research/output/hx_tail_bound_probe_gauss100_ref512_n200k.json`

At `M=128`:
- worst max diff vs ref (across bases): `8.256154829737739e-04`

At `M=192`:
- worst max diff vs ref: `8.871863617088138e-07`

At `M=256`:
- worst max diff vs ref: `9.299760961312131e-11`

Analytic coordinate-tail bounds (same run, at `M=256`):
- `tail_F_bound <= 1.3546846701849444e-12`
- `tail_Fp_bound <= 6.643992170229376e-10`

These are finite-range constants, not yet asymptotic proofs.

Extended Lemma B calibration (`/Users/adminamn/Documents/New project/research/output/lemma_b_truncation_program_gauss100_ref512.json`):
- tested `X in {3e5, 1e6}`, `M in {64,128,192,256}`, `M_ref=512`
- all monotonicity checks (by base and X): `True`
- global worst `B_M(X)` over tested bases/windows:
  - `B_64 <= 1.1427371695007338e-01`
  - `B_128 <= 7.994631679153485e-04`
  - `B_192 <= 8.81680755959735e-07`
  - `B_256 <= 8.807621298956292e-11`

## 4. Proof-Shaped Lemma Stack
### Lemma A (Channel Analyticity)
Each frozen channel in `G(n)` is an explicit smooth function of finite zero sums `(F_M(\log n),F'_M(\log n))` and one-step phase transport; hence `G(n)` is a deterministic functional of the zero set and `n`.
Implementation-level derivation and checker: `/Users/adminamn/Documents/New project/research/output/lemma_a_channel_derivation_note.md`, `/Users/adminamn/Documents/New project/research/output/lemma_a_channel_derivation_check_none_z64_n1m.json`.

### Lemma B (Uniform Truncation Control)
For fixed smoothing/kernel parameters, there exists `B_M -> 0` such that
\[
\sup_{x\le X,\,W\in\mathcal W}|H_W^{(M)}(x)-H_W^{(\infty)}(x)|\le B_M(X).
\]
Current numeric calibration gives candidate magnitudes above (Section 3).

#### O2 Formalization: Infinite-Tail Truncation Lemma Candidate
Working theorem-form statement (from `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.md`):
\[
\forall W\in\mathcal W,\ \forall x\ge x_0,\ \forall M\ge M_0:\quad
\Delta_M(x;W)\le C_\Delta (\log x)^\beta \tau_\infty(M),\quad \tau_\infty(M)\to 0.
\]
with
\[
\Delta_M(x;W)=|H_W^{(M)}(x)-H_W^{(M_{ref})}(x)|
\]
in the current frozen scaffold.

Frozen constants currently used in the finite-window scaffold:
- `beta = 2.6`
- `C_delta_uplifted = 1.3231345976502436e-03`
- gaussian kernel scale `100`
- `M_ref = 512`
- `gamma_ref = 826.905810954`
- density model: `n_diff_explicit` with
  `nbound_c1=0.1038`, `nbound_c2=0.2573`, `nbound_c3=9.3675`, `nbound_h=1.0`

Current tail majorant table (`M in {64,128,192,256}`):
- `tau_infty(64)  ~= 7.29868701081e-01`
- `tau_infty(128) ~= 3.20370327984e-03`
- `tau_infty(192) ~= 2.96243079220e-06`
- `tau_infty(256) ~= 6.64399217023e-10`

Finite-window validation snapshot:
- train holds: `True`
- valid holds: `True`
- valid ratio max: `0.947859440603`
- valid max gap `(delta-rhs)`: `-5.23935442499e-10`

To make O2 fully analytic (remaining obligations):
1. Fix theorem-side citation assumptions for explicit zero-count bound constants used in `N'(t)_up` (current source tag: `HSW2022`).
2. Prove sum-to-integral domination for weighted tail term
   `t/sqrt(1/4+t^2) * K(t)` in theorem form.
3. Prove monotonicity and vanishing of `tau_\infty(M)` with explicit rate.
4. Derive final `C_delta, x0, M0` independent of sampled `n`/`M` grids.

Proof-template and checker artifacts for O2 iteration:
- template: `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_proof_template_2026-02-17.md`
- checker: `/Users/adminamn/Documents/New project/research/output/a2_tail_majorant_checker_2026-02-17.md`
- explicit-density lemma skeleton: `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.md`
- sum-to-integral domination checker (finite-reference): `/Users/adminamn/Documents/New project/research/output/a2_sum_to_integral_domination_checker_2026-02-17.md`

### Lemma C (Affine Transfer on Finite Windows)
There exist `(a,b)` and residual `R_W(x)` with
\[
\frac{E(x)}{\sqrt x}=aH_W(x)+b+R_W(x),
\]
and finite-window envelope candidate `|R_W(x)| <= C_emp` with `C_emp=0.6051556...` on tested ranges.
Explicit-surrogate calibration (`/Users/adminamn/Documents/New project/research/output/lemma_c_transfer_program_none_z64.json`):
- with `S_M(x)=-2\\sum_{j\\le M}\\Re(e^{i\\gamma_j\\log x}/(1/2+i\\gamma_j))`, mean `|corr(S_M,E/\\sqrt x)|=0.892837...`
- mean `|corr(H,S_M)|=0.471302...`
- composed finite-window envelope candidate `C_C <= 0.560106228778126`.
Finite-window inequality probe (`/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_probe_none_z64_ref512.json`):
- calibrated form `|E/\\sqrt{x}-(aH+b)| <= C1*\\Delta_M + C2*R_{smooth}` with
  `a=-0.009754242579...`, `b=-0.100780835808...`, `C1=0.375`, `C2=0`
- zero violations on tested grid (`x<=10^6`, all four bases, `M=64`, `M_ref=512`).
Inequality family comparison (`/Users/adminamn/Documents/New project/research/output/lemma_c_inequality_family_note.md`) selected stable finite-window candidate:
- kernel=`none`, `M=128`, `M_ref=512`
- `a=-0.009292146698774062`, `b=-0.11850135707099743`, `C1=0.375`, `C2=0`
- zero violations, strict margin on tested grid.
Triangle-form theorem candidate (`/Users/adminamn/Documents/New project/research/output/lemma_c_theorem_candidate_note.md`):
- with `(a_ref,b_ref)` frozen on `H^{(M_ref)}`, deterministic inequality
  `|E/\\sqrt{x}-(a_ref H^{(M)}+b_ref)| <= C0 + |a_ref|\\Delta_M`
- calibrated at `(kernel=none, M=128, M_ref=512)`:
  `a_ref=-0.004707542306...`, `b_ref=-0.079037266724...`, `C0=0.612628916836...`
- zero violations and strict margin on tested grid.

#### O3 Formalization: A3 Bridge-Growth Analytic Closure Candidate
Working theorem-form statement (from `/Users/adminamn/Documents/New project/research/output/a3_analytic_closure_lemma_skeleton_2026-02-17.md`):
\[
\forall W\in\mathcal W,\ \forall x\ge x_0:\quad
|H_W(x)|\le C_H(\log x)^{A_H},
\]
with proof-primary chain
\[
\eta_+(x;W)\le C_\eta(\log x)^{A_\eta},\qquad
|H_W(x)|\le \sqrt{(1+\eta_+(x;W))E_2(x;W)/x}.
\]

Frozen constants currently used in the O3 scaffold (primary/stress pair):
- `A_eta = 4.0`
- `C_eta_uplifted = 1.2170134478356474`
- primary branch: `A_H = 1.2`, `C_H = 2.607225675383175`
- stress branch: `A_H = 1.1`, `C_H = 4.526748350185781`
- `eta_safety = 3.0`, `m_zero = 128`, `kernel = none`
- deterministic sign-sensitive fallback (explicit but loose, `k_tail=1`): `C_eta = 39.284726883111246`
  from `/Users/adminamn/Documents/New project/research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17_deterministic_k1.md`

Finite-window validation snapshot:
- primary train/valid holds: `True / True`
- primary valid ratio max: `0.945541199909483`
- stress valid holds: `True`
- stress valid ratio max: `0.8032835508104562`
- eta4 justification valid ratio over `C_eta`: `0.6932140804096792`

To make O3 fully analytic (remaining obligations):
1. Replace empirical eta safety uplift by explicit sign-sensitive offdiag inequality constants.
2. Prove `\eta_+(x;W)\le C_\eta(\log x)^{A_\eta}` uniformly for all `x>=x0` and all `W in {30,210,2310,30030}`.
3. Prove an analytic bound for `E_2(x;W)/x` from lemma-level inputs (no finite-grid calibration dependence).
4. Derive final asymptotic `x0, C_H, A_H` directly from theorem constants.

Proof-template and checker artifacts for O3 iteration:
- skeleton: `/Users/adminamn/Documents/New project/research/output/a3_analytic_closure_lemma_skeleton_2026-02-17.md`
- checker: `/Users/adminamn/Documents/New project/research/output/a3_offdiag_eta_majorant_checker_2026-02-17.md`

### Lemma D (Base-Transfer Consistency)
For wheel family `\mathcal W={30,210,2310,30030}`, statistics of `H_W` and transfer residuals remain in a common regime (same sign, similar magnitude), enabling transfer from one base calibration to the family.
Uniform triangle-transfer probe (`/Users/adminamn/Documents/New project/research/output/lemma_d_base_uniformity_probe_none_z128_ref512.json`):
- shared constants `(a_ref,b_ref,C0_ref)` from reference process:
  `a_ref=-0.004707542306...`, `b_ref=-0.079037266724...`, `C0_ref=0.612628916836...`
- uniform inequality check over all bases and tested windows:
  `|E/\\sqrt{x}-(a_ref H_W^{(M)}+b_ref)| <= C0_ref+|a_ref|\\Delta_M`
  with zero violations on tested grid.
- uniformity score (relative-spread aggregate): `0.03330189089416037`.

### Lemma E (Asymptotic Upgrade)
If Lemmas B-D are proved with explicit asymptotic bounds (not finite-window fits), then bridge bounds imply a bound on `E(x)` of RH type. This is the theorem-critical step.
Finite-window endpoint indicator (`/Users/adminamn/Documents/New project/research/output/lemma_e_endpoint_probe_none_z128_ref512.json`):
- tested candidate envelope `|E(x)|/\\sqrt{x} <= C (\\log x)^A` on scales up to `5e6`
- observed residual best stabilized exponent: `A ~= 0.7`
- triangle-RHS best stabilized exponent: `A ~= 0.5`
- both indicators support slow polylog-style growth on tested windows (evidence only).

## 5. What Is Still Missing For a Real RH Proof
1. Replace empirical affine fit by analytic derivation of `(a,b)` from explicit-formula kernels.
2. Prove Lemma B with true asymptotic `B_M(X)` (uniform in `X`, not only sampled windows).
3. Close O3 analytically: prove eta/offdiag and `E2/x` bounds that imply uniform `|H_W(x)| <= C_H(log x)^A_H`.
4. Convert Lemma E to a standard RH-equivalent endpoint with complete hypotheses.

## 6. Immediate Research Program (Math-first)
1. Derive symbolic linearization of `G(n)` against explicit-formula oscillatory terms `x^{rho}/rho` under logarithmic sampling.
2. Prove a rigorous comparison inequality between `H_W` and a smoothed explicit-formula remainder.
3. Establish base-uniform constants over `W in {30,210,2310,30030}`.
4. Promote finite constants in Section 3 to explicit asymptotic constants.

## 7. Active Theorem Framework
Assumption -> theorem blueprint is now tracked in:
`/Users/adminamn/Documents/New project/research/output/rh_assumption_theorem_draft.md`

Current status:
- finite-window constants and deterministic inequalities are in place,
- asymptotic proof obligations are explicitly isolated (A1-A4 in the draft),
- A2 now has explicit finite-window candidate constants (see `/Users/adminamn/Documents/New project/research/output/a2_truncation_decay_note.md`),
- A2 now has fixed-beta non-regression theorem-constant scaffold with split validation (see `/Users/adminamn/Documents/New project/research/output/a2_theorem_constant_pack_note.md`),
- A2 now has an infinite-tail uplift candidate with explicit tail-majorant construction and held-out-valid constants on current grid (see `/Users/adminamn/Documents/New project/research/output/a2_infinite_tail_uplift_note.md`),
- A2 now also has an explicit asymptotic-tail candidate model (see `/Users/adminamn/Documents/New project/research/output/a2_tau_asymptotic_note.md`),
- A1 now has explicit finite-window candidate constants (see `/Users/adminamn/Documents/New project/research/output/a1_reference_residual_note.md`),
- A1 now has smoothing-decomposition uplift scaffold with split validation (see `/Users/adminamn/Documents/New project/research/output/a1_smoothing_uplift_note.md`),
- A3 now has explicit finite-window candidate constants (see `/Users/adminamn/Documents/New project/research/output/a3_bridge_growth_note.md`),
- A3 now has channel-energy uplift scaffold with deterministic bridge and tighter split-validated envelope (current refresh: `A_H=4.4`, `C_H=4.0835159936256787e-4`; see `/Users/adminamn/Documents/New project/research/output/a3_channel_energy_uplift_note.md`),
- A3 now has a density-transfer majorant variant with analytic \(k/x\) guardrail `phi(W)/W + phi(W)/x0` and held-out-valid constants (see `/Users/adminamn/Documents/New project/research/output/a3_density_transfer_majorant_refresh_2026-02-17.json`),
- A3 now also has a quadratic-energy guardrail branch using \(B_k \le \sqrt{\sum g_i^2}\) with analytic density transfer (see `/Users/adminamn/Documents/New project/research/output/a3_quadratic_energy_majorant_note.md`),
- A3 also has a mean-square direct-\(|H|\) majorant branch (see `/Users/adminamn/Documents/New project/research/output/a3_mean_square_majorant_note.md`),
- A3 offdiag-energy decomposition probe is now documented as a diagnostic branch (held-out-sensitive \(\eta\) guardrail; see `/Users/adminamn/Documents/New project/research/output/a3_offdiag_energy_majorant_note.md`),
- A3 offdiag-dynamic branch now provides a held-out-valid lower-exponent candidate (`A_H=2.5`) with larger constant, tracked in `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_majorant_note.md`,
- piecewise dynamic-\(\eta\) variant was evaluated and rejected due held-out failure; global-log dynamic-\(\eta\) remains active,
- composite \(\eta_+(x)\)+\(E_2/x\) branch is now tracked (valid, lower \(C_H\), higher \(A_H\)); see `/Users/adminamn/Documents/New project/research/output/a3_e2x_dynamic_majorant_note.md`,
- A3 candidate comparison leaderboard is now tracked in `/Users/adminamn/Documents/New project/research/output/a3_branch_leaderboard_2026-02-17.md`,
- constrained fixed-\(A_\eta\) offdiag-dynamic branch now leads current A3 ranking (`A_H=1.2`), see `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_eta4_fixed_note.md`,
- stress validation up to `n=10^7` with denser sampling is now documented in `/Users/adminamn/Documents/New project/research/output/a3_offdiag_dynamic_stress_note.md`,
- fixed-\(A_\eta=4\) justification probe on stress split is tracked in `/Users/adminamn/Documents/New project/research/output/a3_eta4_justification_probe_2026-02-17_sf3.md`,
- A3 theorem-side handoff scaffold is now tracked in `/Users/adminamn/Documents/New project/research/output/a3_eta_lemma_scaffold_2026-02-17.md`,
- A3 absolute symbolic-chain checkpoint (too loose, but deterministic) is tracked in `/Users/adminamn/Documents/New project/research/output/a3_offdiag_symbolic_chain_note.md`,
- A3 lag-decomposition planning scaffold is tracked in `/Users/adminamn/Documents/New project/research/output/a3_offdiag_lag_decomposition_probe_2026-02-17.md`,
- A3 deterministic sign-sensitive constant replacement pack is now tracked in `/Users/adminamn/Documents/New project/research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17_deterministic_k1.md` (held-out valid, deterministic tail mode, but much looser than primary),
- A4 combined finite-window uniform pack is now validated (see `/Users/adminamn/Documents/New project/research/output/a4_uniform_assumption_note.md`),
- uplifted A1/A2/A3 constants are now consolidated into one finite-window theorem pack with A4 rows and unified margin summary (see `/Users/adminamn/Documents/New project/research/output/uplift_theorem_pack_refresh_2026-02-17.md`),
- extended A4 check is now validated up to `x=10^7` with working finite-window `x0` candidate (see `/Users/adminamn/Documents/New project/research/output/asymptotic_uplift_memo.md`),
- conservative train-vs-held-out constant replacement pack is now in place (see `/Users/adminamn/Documents/New project/research/output/analytic_constant_replacement_pack_none_z128_ref512_fast.md`),
- pipeline bottlenecks for A4/theorem checks were reduced via max-`n` per-base streaming reuse and zero-term precompute (see `/Users/adminamn/Documents/New project/research/output/pipeline_optimization_note_2026-02-17.md`),
- endpoint theorem is blocked on upgrading A1-A4 from calibrated finite-window statements to uniform asymptotic proofs.

Status (February 17, 2026): strong finite-range evidence and tightened lemma targets; still not a proof.

Progress update (February 17, 2026):
- Lemma A channel identities: established at formula level and numerically verified to machine precision on canonical regime.
- Lemma B finite-window calibration: completed with monotone `B_M(X)` decay across bases and two X-scales.
- Lemma C finite-window transfer via explicit surrogate `S_M`: calibrated with stable constants.
- Lemma C finite-window inequality form calibrated with zero violations on tested grid.
- Lemma C finite-window inequality family explored; stable working candidate fixed at `(kernel=none, M=128)`.
- Lemma C triangle-form transfer candidate established (non-optimizer, deterministic inequality).
- Lemma D finite-window base-uniformity established for triangle-form transfer constants.
- Lemma E finite-window endpoint indicator calibrated with polylog-style envelope candidates.
- Active target: convert endpoint indicator into theorem: explicit asymptotic `x_0,C,A` with proved hypotheses.
