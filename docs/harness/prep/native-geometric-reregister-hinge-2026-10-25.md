# Pre-Registration Sheet: Native Geometric Engine with Rank-Margin Hinge (`native-geometric-reregister-hinge`)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap Item**: `native-geometric-reregister-hinge` ([`#2871`](https://github.com/UOR-Foundation/uor-r4/issues/2871))
- **Mechanism**: [`native-geometric`](../../mechanisms/native-geometric.md) (`PARKED`) + [`rank-margin`](../../mechanisms/rank-margin.md) (`ACTIVE`)
- **Run**: `native-geometric-hinge-m2-v1` (held until D29 kernel rebuild, shared-writer join, and M2 route preconditions in `PLAN.md` land)

## 1. Question, Hinge Formulation, and Base Artifact

**Question**: When the banned softplus pooled-ranking loss $\ell = \operatorname{softplus}(\log(m_{\text{rival}}/m_{\text{gold}}))$ and log-sum-exp episode aggregation (`D23`/`D25`, `archive/labs/m2-pooled-rank-2026-10-10/README.md:21-27`) in the October cross-state native geometric engine (`saved177` parent, `115,200` Q4 coefficients over $8 \times 120 \times 120$ categorical $H_4$ banks) are replaced by the lawful rank-margin hinge (`docs/mechanisms/rank-margin.md:21-36`) and executed on GPU via rebuilt D29-compliant kernels joined to the shared integer vector writer (`docs/mechanisms/native-geometric.md:94-97`), does the exported D11 integer artifact simultaneously clear **`>= 256 / 512` complete correct replies on `m2-reply512`** and **`pairs_both_complete_distinct >= 52 / 128` (`complete >= 104 / 256`) on the sealed 256-row `m2-fresh256` panel** across two seeds?

**Lawful rank-margin hinge formulation (`CHARTER.md` §3.1–§3.7 compliant, zero softplus/softmax/log-sum-exp/matmul)**:
- **Cross-state $H_4$ field score (exact integer Q24 / Q4)**: For token $t$ and lane $l \in \{0,\dots,7\}$, given factual post-bridge $H_4$ state $f_l \in 2I$ (`120` icosians) and local reply-prefix $H_4$ state $p_l \in 2I$:
  $$\Delta s(t) = 2^{22} \sum_{l=0}^{7} E_l\big[f_l^{-1} \cdot \pi_{t,l},\ p_l^{-1} \cdot \pi_{t,l}\big], \qquad E_l \in \{-7,\dots,+7\}^{120 \times 120}\ (\text{Q4}).$$
  Group inversion $f_l^{-1}$ and composition $f_l^{-1} \cdot \pi_{t,l}$ use the exact 120-element icosian lookup table (`GROUP_ORDER = 120`, $\mathbb{Z}[\varphi]$ with $\varphi^2 = \varphi + 1$), and $s(t) = s_{\text{base}}(t) + \Delta s(t)$ is an exact integer score over pooled Generate and Copy actions without exponential masses.
- **Rank-margin hinge over top-$k$ rivals (`k = 8`, margin $\delta = 2^{22}$ = `1.0` Q4 unit)**: Let $y$ be the gold target token at teacher-prefix step $(e, t)$ and $\mathcal{R}_k(e, t)$ be the $k = 8$ highest-scoring non-target tokens $v \neq y$ under integer score $s(v)$ (smallest token ID breaks ties):
  $$\ell_{\text{hinge}}(e, t) = \frac{1}{k} \sum_{r \in \mathcal{R}_k(e, t)} \max\!\big(0,\ \delta - \big(s(y) - s(r)\big)\big), \qquad J(e) = \frac{1}{T_e} \sum_{t=1}^{T_e} \ell_{\text{hinge}}(e, t).$$
  Subgradient credit is $-1/k$ on $y$ and $+1/k$ on each active rival $r \in \mathcal{R}_k(e, t)$ with $s(y) - s(r) < \delta$, and `0` outside the margin band (`0` calls to `exp`, `log`, `softplus`, `sigmoid`, `tanh`, or `softmax`). Under D37 (`CHARTER.md` §3.7), offline float master weights (`[8, 14400]` F32 master rounded at every step to `[-7, +7]` Q4 via straight-through identity) execute in a separately scanned GPU module and are evaluated solely through the rounded Q4 integer artifact (`export_gap` reported against the float master on `m2-reply512`).
- **Base artifact (`saved177` pre-pooled-rank checkpoint, `archive/labs/m2-pooled-rank-2026-10-10/README.md:67-73`)**:
  - `saved177` field (`65,536` B padded Q4 payload): `dfb945d567ce25f5c200967ce1f9e5e31740f9cfaa8f6a1a6d6dd11d156a2e13`
  - `saved177` F32 master (`460,800` B): `0c7dec5f627b8c9c44557922a4d17d30846796381d26a055edc84551f8149acc`
  - `saved177` manifest: `10db47cb5e29d562df485e8364fe60bf503b5b4bd40f7b4797f40a4c65e30156`
  - Frozen upstream manifest: `b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca`

## 2. Arms, Data Split, and Steps

**Four registered arms** (`256` updates from `saved177`, `batch = 8`, `6,664` answer/EOS positions across `512` episodes, `context <= 128`, `max_gen = 32` with EOS, Q4 clamp `[-7, +7]`):

| Arm | Objective & Update Rule | Role |
|---|---|---|
| **`H_hinge_q4`** | Top-$k$ (`k = 8`) linear rank-margin hinge $\ell_{\text{hinge}}$ + uniform episode mean $J(e)$, Q4 straight-through rounding (`seed = 1001`, `seed = 1002`) | **Primary treatment**: lawful hinge replacement for banned softplus pooled ranking |
| **`C_parent177`** | Zero updates (`saved177` field `dfb945d5...` evaluated as-is) | **Baseline parent control**: `177 / 512` on `m2-reply512` (`0` updates) |
| **`C_rival_shuf`** | Identical `256`-update schedule and Q4 rounding as `H_hinge_q4`, but rival indices $\mathcal{R}_k(e, t)$ drawn by deterministic permutation instead of top-$k$ hardest rivals | **Rival-selection null control**: isolates hardest-rival margin pressure from generic update drift |
| **`C_question_blind`** | Deterministic non-learned heuristics (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) | **Grader non-vacuity control**: all score `0 / 128` pairs on `m2-fresh256` |

**Data split and SHA-256 pins (`docs/harness/panels/m2-fresh256.json`)**:
- **Development panel (`m2-reply512`, `512` rows / `256` histories)**: `inputs.json`: `b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a`; `labels.json`: `84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155`.
- **Sealed fresh held-out panel (`m2-fresh256`, `256` rows / `128` opposite-role pairs across `16` unordered blocks, `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` at commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`)**:
  - `plan.json` (`122,268` B): `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`
  - `inputs.json` (`524,850` B): `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`
  - `labels.json` (`68,000` B): `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`
  - `manifest.json` (`1,384` B): `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`
- **Tokenizer (`tokenizer.json`)**: `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

**Execution steps**:
1. **Step 0 (Finite controls `I-1..I-6` on GPU)**: Run all six finite controls in Section 4 (`0` language claims if any fails).
2. **Step 1 (Two-seed hinge training on `m2-reply512`)**: Run `H_hinge_q4` and `C_rival_shuf` for `256` updates on `seed = 1001` and `seed = 1002` from `saved177`, export to Q4 integer tables, and score complete replies and D37 export gap on `m2-reply512`.
3. **Step 2 (Single sealed evaluation on `m2-fresh256`)**: If and only if `H_hinge_q4` clears `>= 256 / 512` on `m2-reply512` with export gap `<= 12` rows on both seeds, evaluate the exported Q4 integer artifacts once on `m2-fresh256` (`pairs_both_complete_distinct` and `complete`).

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum test power (`CHARTER.md` §4.3)**: Two independent seeds (`seed = 1001`, `seed = 1002`) across `512` development rows (`m2-reply512`) and `256` genuinely fresh rows / `128` opposite-role history pairs (`m2-fresh256`, `0` exact, `0` content-word, and `0` word 2-gram overlaps against all `74` historical literals).
- **Noise band (`docs/harness/panels/m2-fresh256.json:11`)**:
  - On `m2-reply512`: single-model 16-block cluster SE is `6.24` rows (`2\text{SE} = 13` rows).
  - On `m2-fresh256` at the `40%` bar: 16-block cluster-adjusted SE is `8.64` rows (`4.32` pairs; $2\sigma$ band $\pm 17$ rows / $\pm 9$ pairs). Every question-blind control scores `0 / 128` pairs (`52 / 128` is $12.0\times$ the cluster SE above `0 / 128`).
- **Primary promotion bar (`B_native_hinge`)**: On **each of two seeds (`seed = 1001` and `seed = 1002`)**, the exported Q4 integer artifact must satisfy:
  1. `complete_replies(m2-reply512) >= 256 / 512` (`50.0%`, and `> C_parent177 + 13` rows = `190 / 512`);
  2. `pairs_both_complete_distinct(m2-fresh256) >= 52 / 128` (`40.625%`) and `complete_replies(m2-fresh256) >= 104 / 256`;
  3. `pairs_both_complete_distinct(H_hinge_q4) - pairs_both_complete_distinct(C_rival_shuf) >= 9` pairs on `m2-fresh256`;
  4. D37 float-to-Q4 export gap `|complete_float - complete_q4| <= 12` rows (`<= 2 * SE`) on `m2-reply512`, and `0` multiply/float instructions in the served Q4 evaluator (`CHARTER.md` §3.6, §3.7).

| Outcome | Condition Across Two Seeds | Classification & What It Unlocks |
|---|---|---|
| **`PASS_REOPEN_NATIVE_GEOMETRIC`** | `I-1..I-6` pass and `H_hinge_q4` meets `B_native_hinge` (`>= 256/512` dev, `>= 52/128` fresh pairs, `>= 104/256` fresh rows, export gap `<= 12`) on both seeds | **Mechanism positive (`native-geometric` reopened from `PARKED` to `READY`/`ACTIVE`)**. Closes `#2871` with fresh-row results reported. |
| **`CLOSE_PANEL_FIT_PERSISTS`** | `I-1..I-6` pass and `H_hinge_q4` clears `>= 256/512` on `m2-reply512` but scores `< 52/128` pairs on `m2-fresh256` | **Configuration negative on the 115,200-weight cross-state Q4 bank under linear hinge** (`CHARTER.md` §4.1: proves the `437 -> 0` transfer failure was representation/capacity-bound rather than softplus-induced). Closes `#2871` with exact fresh-row counts recorded. |
| **`CLOSE_HINGE_UNDERFITS_DEV`** | `I-1..I-6` pass, but `H_hinge_q4` fails to reach `256/512` on `m2-reply512` on either seed (`m2-fresh256` kept sealed, `0` fresh evaluations) | **Configuration negative on the `k = 8`, $\delta = 2^{22}$ linear hinge schedule** (`CHARTER.md` §4.1). Closes `#2871` without burning the sealed `m2-fresh256` panel. |
| **`INSTRUMENT_STOP`** | Any finite control `I-1..I-6` fails on GPU | **Instrument defect (`CHARTER.md` §4.2)**; repair before any training or panel evaluation. |

## 4. Six Finite Instrument & Algebraic Controls

1. **`I-1` (Zero-squash, zero-matmul, and D29 kernel audit)**: Static and PTX symbol audit of the rebuilt native-context CUDA module confirms `0` calls to `exp`, `log`, `softplus`, `sigmoid`, `tanh`, `softmax`, or dense GEMM/contraction (`HISTORICAL_KERNELS` remain blocked).
2. **`I-2` (Exact $H_4$ $\mathbb{Z}[\varphi]$ window-fold & `2I` group table parity)**: Verifies $(a+b\varphi)(c+d\varphi) = (ac+bd) + (ad+bc+bd)\varphi$ and `120 x 120` icosian product/inverse table bit-identity between GPU and host across all `14,400` pairs (`0` mismatches).
3. **`I-3` (Shared integer vector writer join, `docs/mechanisms/native-geometric.md:97`)**: Verifies that token/action emissions pass through `crates/uor-r4-training/src/native_vector_writer.rs` with separate exact occurrence/version identity (`CHARTER.md` §3.12, `0` identity collisions).
4. **`I-4` (Hinge zero-loss above margin & exact linear subgradient)**: On a 32-row synthetic fixture, rows with $s(y) - s(r) \ge \delta$ for all $r$ produce `loss == 0` and `grad == 0` (`0` non-zero entries), while rows with $s(y) - s(r) < \delta$ reduce $\ell_{\text{hinge}}$ monotonically after one Q4 update step.
5. **`I-5` (`saved177` baseline reproduction)**: Zero-update GPU evaluation of `C_parent177` (`dfb945d5...`) on `m2-reply512` (`b9661606...`) reproduces `177 / 512` complete replies with `0 / 512` row-ID or token-sequence mismatches (`archive/labs/m2-pooled-rank-2026-10-10/README.md:81-84`).
6. **`I-6` (`m2-fresh256` seal & question-blind control check)**: Verifies SHA-256 digests of all four `m2-fresh256` files, confirms `inputs.json` has `0` occurrences of `"accepted"`, and confirms all four `C_question_blind` heuristics score `0 / 128` pairs (`docs/labs/m2-fresh-heldout-panel/README.md:55-60`).

## 5. Cost and Stop Rule

- **Compute & cost**: `1x NVIDIA GeForce RTX 5090` via `scripts/pod/uor-pod` inside fleet caps (`<= 4` pods, `<= $8.00/h`). `256` updates $\times$ `2` arms (`H_hinge_q4`, `C_rival_shuf`) $\times$ `2` seeds + `m2-reply512` / `m2-fresh256` evaluation takes `~25 minutes` (`~$0.25` at `$0.563/GPU/h`; hard cap `$1.50`).
- **Stop rule**:
  1. **Immediate instrument stop**: Halt at Step 0 if any of `I-1..I-6` fails (`0` model claims).
  2. **Development gate before fresh evaluation**: Do not evaluate `m2-fresh256` unless `H_hinge_q4` clears `>= 256 / 512` on `m2-reply512` with export gap `<= 12` rows on both seeds (preserves `m2-fresh256` unexposed if dev underfits).
  3. **Execution hold until D29 kernel rebuild & M2 route open**: Do not rent a GPU pod while `native-geometric-reregister-hinge`'s `scope:` excludes Rust crates, before the 15 quarantined D29 native-context kernels are rebuilt without dense contractions (`docs/mechanisms/native-geometric.md:96`), and before M1 (`#2029`) closes to open the M2 route (`PLAN.md` lines 75–77).

## 6. Review Verdict

TEST FITNESS: FIT - research #2871 native-geometric-reregister-hinge
sheet: docs/harness/prep/native-geometric-reregister-hinge-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer 5f00472e-08a8-477c-9cf6-8946b52f4175; https://github.com/UOR-Foundation/uor-r4/issues/2871#issuecomment-6396265555)
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
