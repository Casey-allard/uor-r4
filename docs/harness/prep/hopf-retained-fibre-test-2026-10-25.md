# Pre-Registration Sheet: Hopf Retained-Fibre Language Test (`hopf-retained-fibre-test`)

- **Date**: 2026-10-25
- **Lab**: `antigravity`
- **Roadmap Item**: `hopf-retained-fibre-test` ([`#2825`](https://github.com/UOR-Foundation/uor-r4/issues/2825))
- **Mechanism**: [`hopf-observation`](../../mechanisms/hopf-observation.md) (`READY` for kept-fibre map; `PARKED` for sector routing per owner ruling D72)
- **Run**: `hopf-retained-fibre-language-v1` (held until discrete/integer consumer wiring and Stage T `#2836` base checkpoint land)

## 1. Question, Geometric Construction, and Base Artifact

**Question**: In the discrete/integer geometric language model (`crates/uor-r4-discrete` exported to `UORLUT01` in `crates/uor-r4-integer`), does retaining the $S^1$ Hopf fibre phase $\psi(q)$ alongside the $S^2$ Hopf base shadow $\pi(q)$ inside the context-read scoring path improve integer DEV prediction (`top1_correct`, `top8_correct`, and `b_3(P_succ)` target admission on the 14,287-row DEV panel) over a base-only ($S^2$ shadow) control (`C_base_only`), an explicit non-geometric bank-tag control (`C_bank_tag`) at **strictly equal retained bytes**, and a fibre-permuted null control (`C_fibre_perm`) across two seeds?

**Exact multiplier-free discrete/integer Hopf construction (`CHARTER.md` §3.1–§3.6 & §3.12 compliant)**:
- **Canonical Hopf map & fibre decomposition**: For a 4D unit quaternion block $q = (a, b, c, d) \in S^3$ (or each 4D half-block $(q_L, q_R) \in H_4 \oplus \varphi H_4$ of an 8D $E_8$ orbit root in `crates/uor-r4-discrete/src/geometry.rs`, `ROOTS: [[i8; 8]; 16]`, and Q1.30 `UnitS3Q30` in `crates/uor-r4-integer/src/math.rs:405-482`):
  $$\pi(q) = \big(2(ac+bd),\ 2(bc-ad),\ a^2+b^2-c^2-d^2\big) \in S^2, \qquad \psi(q) = \operatorname{atan2}_{\text{CORDIC}}(b, a) \text{ (or } \operatorname{atan2}_{\text{CORDIC}}(d, c) \text{ at } a^2+b^2=0) \in S^1.$$
  On the 120 icosians ($2I \subset S^3$), $\pi$ maps the 120 elements onto **30 base points** of $S^2$ with **every fibre of exact size 4** (`docs/mechanisms/hopf-observation.md:32`, `docs/geometry/hopf-observation.md:30`). On the finite discrete path (`crates/uor-r4-discrete`), both the 16 orbit roots (`ROOTS[0..16]`) and the 64 `Cl(0,6)` stabilizer frames (`candidate_to_frame(0..64)`) admit an **exact, precomputed, multiplier-free lookup table** `HOPF_TABLE[root_or_frame] -> (base_id, fibre_id)` (and on Q1.30 states, `UnitS3Q30::hopf_fiber_project` in `crates/uor-r4-integer/src/math.rs:473` uses `mul_i32_radix4`, restoring `isqrt`, `div_round`, and 16-step `atan2_q30` with zero hardware multipliers and zero floats).
- **Equal-byte retained summary contract (`8` bits = `1` byte per block summary)**:
  Every Hopf-augmented arm retains **strictly `1` byte (`8` bits) per block per context position** (`4` bytes per 4-block token state), with exact token/occurrence identity carried separately in the unmodified token/writer pathway (`CHARTER.md` §3.12):
  - **`H_base_fibre` (Base + Kept Fibre, `6` bits base + `2` bits fibre = `8` bits)**: Encodes the quantized $S^2$ base index in `6` bits (`0..64`, covering the 30 icosian base points / 64 octant-refined base cells) and the $S^1$ fibre phase quadrant in `2` bits (`0..4`, matching the exact size-4 icosian fibre $\mathbb{Z}_4$). Candidate context positions are scored by the integer L1 / discrete Fubini-Study + cyclic $\mathbb{Z}_4$ fibre distance $d_{\text{Hopf}}((b_q, \psi_q), (b_k, \psi_k)) = d_{S^2}(b_q, b_k) + w_{\psi}\,\min(|\psi_q - \psi_k|, 4 - |\psi_q - \psi_k|)$ (all integer table lookups, zero softmax/squashing, zero dense matmul).
  - **`C_base_only` (Base-Only Control at Equal Retained Bytes, `8` bits base + `0` bits fibre = `8` bits)**: Drops the $S^1$ fibre (`w_{\psi} = 0`) and gives all `8` bits (`256` $S^2$ base cells) to the $S^2$ base shadow $\pi(q)$ alone, so `C_base_only` has identical retained byte width (`1` byte/block) to `H_base_fibre`.
  - **`C_bank_tag` (Explicit Bank-Tag Control at Equal Retained Bytes, `6` bits base + `2` bits bank tag = `8` bits)**: Retains the identical `6`-bit $S^2$ base index as `H_base_fibre` and replaces the `2`-bit $S^1$ fibre slot with an explicit non-geometric position bank tag `tag = (pos & 3) as u8` (`2` bits), scored with the same integer distance table.
- **Base artifact**: The qualified A8 address-structured codebook `address-structured-v1.json` (`file_sha256 = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, `table_sha256 = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`, `256` occupied cells, `4,096` distinct vectors) and the Stage T (`#2836`) trained discrete checkpoint exported through `UORLUT01` (`#2838` / `#2839`).

## 2. Arms, Data Split, and Steps

**Five registered arms** (all at `vocabulary = 4,096`, `blocks = 4`, `context = 192`, `kept_cells = 8`, `64` rows/batch, `ObjectivePolicy::NearestAddressMarginV1`, active D40 admission-preserving rollback `AuditPolicy::registered()`):

| Arm | Retained Summary Bytes / Block | Base Bits | Fibre / Tag Bits | Role |
|---|---|---|---|---|
| **`H_base_fibre`** | `1` byte (`8` bits) | `6` bits ($S^2$ base) | `2` bits ($S^1$ fibre $\psi \in \mathbb{Z}_4$) | **Primary treatment**: base + kept $S^1$ fibre phase |
| **`C_base_only`** | `1` byte (`8` bits) | `8` bits ($S^2$ base) | `0` bits (fibre dropped) | **Equal-byte base-only control**: full `8`-bit budget on $S^2$ shadow |
| **`C_bank_tag`** | `1` byte (`8` bits) | `6` bits ($S^2$ base) | `2` bits (`pos mod 4` bank tag) | **Equal-byte bank-tag control**: non-geometric 2-bit tag in place of $\psi$ |
| **`C_fibre_perm`** | `1` byte (`8` bits) | `6` bits ($S^2$ base) | `2` bits (deranged fibre $\psi_{\sigma(t)}$) | **Fibre-permuted null control**: deterministic cyclic derangement $\sigma(t) = (t + 1) \bmod L$ of fibre values across context positions |
| **`C_no_hopf`** | `0` Hopf summary bytes | — | — | **Baseline discrete read control**: unmodified `ReadSpec::Flock` / `ReadSpec::Relation` on the same base checkpoint |

**Data split and SHA-256 pins**:
- **TRAIN corpus** (`mw-balp/train/tokens.u16`): `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; `response_mask.u8`: `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **DEV evaluation panel** (`ft/dev/tokens.u16`): `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8`: `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (`14,287` scored target positions: `3,847` `P_succ` copy/successor rows and `10,440` complement rows).
- **TRAIN admission anchor** (`3,847` rows, `uor-r4.train-copy-successor-anchor/v1`): `row_csv_sha256 = 8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1`, `manifest_sha256 = f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de`.
- **Tokenizer** (`tokenizer.json`): `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **Fresh transfer panel** (`fresh_v2`, `>= 2,000` rows): kept sealed during development; evaluated once if and only if `H_base_fibre` passes the DEV bar on both seeds (`CHARTER.md` §4.3).

**Execution steps**:
1. **Step 0 (Finite algebraic & instrument controls `I-1..I-6` on GPU)**: Run all six finite controls in Section 4 before any language evaluation or training step. Any failure halts immediately as an `instrument` defect (`0` mechanism claims).
2. **Step 1 (Zero-training & Stage-T checkpoint evaluation on integer DEV metrics)**: Evaluate `H_base_fibre`, `C_base_only`, `C_bank_tag`, `C_fibre_perm`, and `C_no_hopf` on the 14,287-row DEV panel (`top1_correct`, `top8_correct`, `b_3(P_succ)`, `b_3(Full)`, and `UORLUT01` integer `tokens/s`).
3. **Step 2 (Two-seed matched training comparison, `seed = 1` and `seed = 2`, `4,096` batches to `Q1` / `16,404` batches full dose)**: Run the five arms on `seed = 1` and `seed = 2` under `AuditPolicy::registered()`, export each completed checkpoint through `UORLUT01`, and score integer DEV `top1_correct`, `top8_correct`, `b_3(P_succ)`, export gap (`0` mismatches required for discrete integer export), and `tokens/s`.
4. **Step 3 (Fresh transfer verification if DEV bars pass)**: If `H_base_fibre` beats `C_base_only`, `C_bank_tag`, and `C_fibre_perm` outside the noise band on both seeds, evaluate once on `fresh_v2` (`>= 2,000` rows).

## 3. Bar, Minimum Test Power, and Decision Table

- **Minimum test power (`CHARTER.md` §4.3)**: Two independent seeds (`seed = 1`, `seed = 2`) on the `14,287`-row DEV panel (`3,847`-row `P_succ` slice + `10,440`-row complement) plus a `>= 2,000`-row fresh transfer evaluation on any positive DEV gain.
- **Noise band (`CHARTER.md` §4.3)**:
  - On the `3,847`-row `P_succ` slice: paired McNemar / binomial half-width $H_{\text{succ}} = \max\!\big(24,\ \lceil 2\sqrt{n_{10} + n_{01}}\rceil,\ |\Delta_{\text{seed }1} - \Delta_{\text{seed }2}|\big)$ rows.
  - On the full `14,287`-row DEV panel: $H_{\text{dev}} = \max\!\big(59,\ \lceil 2\sqrt{n_{10} + n_{01}}\rceil,\ |\Delta_{\text{seed }1} - \Delta_{\text{seed }2}|\big)$ rows.
- **Primary promotion bar (`B_hopf_fibre`)**: On **each of two seeds (`seed = 1` and `seed = 2`)**, through the `UORLUT01` integer export:
  1. `top1_correct(H_base_fibre) - top1_correct(C_base_only) >= H_dev` (or `>= H_succ` on `P_succ` with zero full-DEV regression);
  2. `top1_correct(H_base_fibre) - top1_correct(C_bank_tag) >= H_dev` (proves the geometric $S^1$ fibre beats an equal-bit explicit bank tag);
  3. `top1_correct(H_base_fibre) - top1_correct(C_fibre_perm) >= H_dev` (proves fibre alignment is load-bearing);
  4. `target_admitted_rows(H_base_fibre) >= target_admitted_rows(C_no_hopf) - 24` (`G-Adm` preserved) and integer `tokens/s` within `>= 90%` of `C_no_hopf`;
  5. Positive transfer ($\Delta_{\text{fresh}} > 0$ outside the fresh-slice $2\text{SE}$ band) on `fresh_v2`.

| Outcome | Condition Across Two Seeds | Classification & What It Unlocks |
|---|---|---|
| **`PASS_KEEP_HOPF_FIBRE`** | `I-1..I-6` pass and `H_base_fibre` meets `B_hopf_fibre` (`> C_base_only + H_dev`, `> C_bank_tag + H_dev`, `> C_fibre_perm + H_dev`, `G-Adm` kept, positive fresh transfer) on both seeds | **Mechanism positive (`hopf-observation` kept-fibre map promoted from `READY` to `ACTIVE`)**. Closes `#2825` (`hopf-retained-fibre-test`) and satisfies Reopen Conditions 1 & 2 of `docs/mechanisms/hopf-observation.md:73-74`. |
| **`CLOSE_FIBRE_TIES_BASE_OR_TAG`** | `I-1..I-6` pass, but `H_base_fibre` ties or trails `C_base_only` or `C_bank_tag` within $\pm H_{\text{dev}}$ on DEV `top1_correct` across both seeds | **Configuration negative on the 8-bit (`6b` base + `2b` fibre) discrete retained-fibre readout** (`CHARTER.md` §4.1: closes this configuration only, never the mechanism). Closes `#2825` with exact DEV and `P_succ` counts recorded (`fresh_evaluations == 0`). |
| **`INSTRUMENT_STOP`** | Any finite control `I-1..I-6` fails on GPU | **Instrument defect (`CHARTER.md` §4.2)**; repair the kernel/table before reading any DEV language number (`0` mechanism verdicts). |

## 4. Six Finite Instrument & Algebraic Controls (`docs/mechanisms/hopf-observation.md:73`)

Before any DEV language score is read, the GPU binary must pass all six deterministic controls (`I-1..I-6`):
1. **`I-1` (Common-phase $S^2$ invariance & $S^1$ fibre sensitivity)**: Applying the 4-element icosian fibre subgroup $\{\pm 1, \pm i\}$ to each state leaves its `6`-bit $S^2$ base index bit-identical across all states (`0` base changes) while cycling the `2`-bit fibre phase $\psi \in \{0, 1, 2, 3\}$ injectively (`100%` fibre transitions).
2. **`I-2` (Common-action covariance)**: Applying a common `Cl(0,6)` / icosian action $g$ to both query $q \mapsto g \cdot q$ and key $k \mapsto g \cdot k$ preserves the relative Hopf base+fibre distance $d_{\text{Hopf}}(g \cdot q, g \cdot k) = d_{\text{Hopf}}(q, k)$ while changing individual base coordinates.
3. **`I-3` (One-endpoint vs both-endpoints contrast)**: Rotating only one endpoint ($k \mapsto e^{i\varphi} k$, $\varphi \neq 0$) along its fibre strictly changes $d_{\text{Hopf}}(q, k)$ in `H_base_fibre` while leaving `C_base_only` invariant (`0` score change); rotating **both** endpoints by the same common fibre phase ($q \mapsto e^{i\varphi} q,\ k \mapsto e^{i\varphi} k$) leaves $d_{\text{Hopf}}$ invariant in both `H_base_fibre` and `C_base_only`.
4. **`I-4` (Fibre removal & permutation separation on same-shadow keys)**: On a synthetic 64-row fixture where target key $k^+$ and rival key $k^-$ lie on the **same $S^2$ base fibre** ($\pi(k^+) = \pi(k^-)$) with distinct fibre phases ($\psi(k^+) = \psi(q) \neq \psi(k^-)$), `H_base_fibre` selects $k^+$ on `64 / 64` rows, `C_base_only` scores $k^+$ and $k^-$ identically (`0` margin on `64 / 64` rows), and `C_fibre_perm` degrades below `H_base_fibre`.
5. **`I-5` (Exact occurrence identity separation, `CHARTER.md` §3.12)**: Two distinct token IDs $v_1 \neq v_2$ deliberately mapped to the identical `(base_id, fibre_id)` summary retain distinct exact token identities in the shared token-vector writer and copy-successor indicator (`0` identity collisions).
6. **`I-6` (Equal-byte, zero-squash/zero-matmul/zero-float, and GPU-vs-host parity)**: Asserts `sizeof(HopfSummary) == 1` byte/block across `H_base_fibre`, `C_base_only`, `C_bank_tag`, and `C_fibre_perm`, `0` GPU-vs-host mismatches across all 5 arms on a 64-row fixture, and `0` banned tokens (`D23`/`D25`/`D32`).

## 5. Cost and Stop Rule

- **Compute & cost**: `1x` or `2x NVIDIA GeForce RTX 5090` via `scripts/pod/uor-pod` inside the fleet caps (`<= 4` pods, `<= $8.00/h`). Step 0 finite controls + Step 1 zero-training DEV evaluation take `~5 minutes` (`~$0.08`). Step 2 Q1 (`4,096` batches $\times$ `5` arms $\times$ `2` seeds) takes `~2.5 GPU-hours` (`~$1.45` at `$0.563/GPU/h`; hard cap `$6.00`).
- **Stop rule**:
  1. **Immediate instrument stop**: Halt at Step 0 if any of `I-1..I-6` fails (`0` language claims).
  2. **Q1 early stop (`batch = 4,096`)**: Stop any arm at `Q1` that triggers `consecutive_full_rejections = 3` (`AuditPolicy::registered()`) or drops below `DEV b_3(P_succ) = 500 / 3,847`. If `H_base_fibre` trails both `C_base_only` and `C_bank_tag` by more than `2 * H_dev` on both seeds at `Q1` while all controls pass, stop and record `CLOSE_FIBRE_TIES_BASE_OR_TAG`.
  3. **Execution hold until consumer scope & Stage T base land**: Do not rent a GPU pod while `hopf-retained-fibre-test`'s `scope:` excludes `crates/uor-r4-discrete/**` and `crates/uor-r4-integer/**` and before `#2836` (`m1-bvg-k-stage-t-training`) produces the Stage T checkpoint.

## 6. Review Verdict

TEST FITNESS: FIT - research #2825 hopf-retained-fibre-test
sheet: docs/harness/prep/hopf-retained-fibre-test-2026-10-25.md
reviewer: antigravity (fresh-context non-author reviewer 0fdc6a0a-4c05-43f9-9ee3-c0b210867fb6; https://github.com/UOR-Foundation/uor-r4/issues/2825#issuecomment-6396126182)
date: 2026-10-25
A FIT verdict clears compute for the configuration in that sheet only (D22).
