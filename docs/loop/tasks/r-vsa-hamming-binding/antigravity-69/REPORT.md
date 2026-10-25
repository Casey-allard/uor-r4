# REPORT: r-vsa-hamming-binding, antigravity-69

Lab: antigravity. Item: [r-vsa-hamming-binding](../../../../roadmap/items/r-vsa-hamming-binding.md) (#2888).

References #2888

## What did not work

- **Direct GPU execution of `DiscreteVsaHammingWriterV1` in this unit is blocked by item scope and `#2836` (`m1-bvg-k-stage-t-training`):**
  1. `docs/roadmap/items/r-vsa-hamming-binding.md` scopes this unit strictly to `[docs/harness/prep/r-vsa-hamming-binding*, docs/labs/r-vsa-hamming-binding/**, docs/loop/tasks/r-vsa-hamming-binding/**]`, which excludes `crates/uor-r4-discrete/**`.
  2. In `#2831` (`m1-a8-full-dose-rerun`), single-coordinate (`K = 1`) updates coupled with `64`-batch whole-interval rollback halted early at batch `1,600` (`Relation`) and batch `5,248` (`FrozenZero`) on `STOP_FOR_DIAGNOSIS: consecutive_rejections` (`mechanism_negative: false`), so running a `16,404`-batch, 2-seed VSA comparison requires the Stage T `BVG-K` (`K = 16`) update law from `#2836` (`m1-bvg-k-stage-t-training`, Step 4 of `PLAN.md`).
- **Why earlier VSA runs did not settle discrete Hamming binding on the current writer (`CHARTER §4.1`, `D22`):**
  1. The 9 October 2026 fixed-code VSA run (`archive/labs/vsa-native-test-2026-10-09/README.md`, `Mode 0`) improved held-out BPB by `0.0144` to `0.0226` across 2 seeds and 2 slices (`95%` within-model ablation CI `[+0.0128, +0.0163]`), but trained on the legacy CPU learner (`train-native-prose`), barred since 12 October 2026 by `D24` (`CHARTER §3.5`), and used a runtime `Q1.15` integer multiply (`(vsa_scale_q15 * corr) >> 16`) unaudited against `D11` (`CHARTER §3.6`).
  2. `Mode 1` (`120` icosian-root codes across `4,096` tokens) failed (`+0.0003` to `-0.0054` BPB) because tokens sharing a root received identical codes, and `Mode 2` (root code $\oplus$ readout residual rebuilt every `1,000` steps) failed (`+0.013` to `+0.028` BPB worse than fixed codes) because rebuilding the codebook during training created a moving-target confound (`docs/mechanisms/vsa.md:81`).

## What changed

- **Size:** `6` files changed (`tests` `0` files `+0 / -0`, `code` `0` files `+0 / -0`, `docs` `5` files `+299 / -1`, `generated` `1` file `+3 / -3`). Ratio to estimate: the item file carried no numeric line estimate (`unknown`); all `5` documentation/task files and `1` rendered roadmap table stay well under the `600`-line review slice ceiling.
- **Cancel probe (`docs/loop/tasks/r-vsa-hamming-binding/antigravity-69/ORDER.md`):** Audited `crates/uor-r4-discrete/src/discrete.cu` (`root_cost` line 37, `address_distance` line 328, `token_distance` line 331, `writer` lines 361–399), `crates/uor-r4-core/src/native_geometric/vsa/hypervector.rs`, `archive/labs/vsa-native-test-2026-10-09/README.md`, and `docs/mechanisms/vsa.md`. Confirmed that `writer(...)` ranks candidates inside admitted cells solely by static 4-block E8 L1 distance $d_{\text{L1}}(q, v(t))$ with zero context-bound XOR/Hamming term, leaving `3,421 / 4,292` admitted rows (`79.71%`) unranked at `top-1` in `#2831` `Q1` (`871 / 4,292`, of which `192 / 871 = 22.04%` sit on an exact L1 tie broken by token ID).
- **One-page pre-registration sheet (`docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md`) and lab specification (`docs/labs/r-vsa-hamming-binding/README.md`):**
  - Defined `DiscreteVsaHammingWriterV1`: each token $t \in [0, 4096)$ carries a frozen 64-bit hypervector $h(t) \in \{0,1\}^{64}$ (`1` `u64` word per token = `32 KiB` read-only table). Over the odd causal prefix window $p \in [\max(0, L - 15), L - 1]$ ($m \le 15$ terms), each context token is bound by cyclic 64-bit rotation $\operatorname{rotl}_{64}(h(\text{id}_p), (\Delta p \cdot 4 + a_q + 1) \ \& \ 63) \oplus h(\text{id}_{L-1})$ and bundled via a 4-bit vertical bit-plane majority vote into $B_{\text{ctx}}(\text{row}) \in \{0,1\}^{64}$.
  - Inside `writer(...)`, candidate distance becomes $d_{\text{total}}(q, B_{\text{ctx}}, t) = (d_{\text{L1}}(q, v(t)) \ll 2) + (\operatorname{__popcll}(B_{\text{ctx}}(\text{row}) \oplus h(t)) \gg \sigma_{\text{vsa}})$, using shifts, XOR, `__popcll`, and addition only (`0` floats, `0` multiplies, `D11 R1–R5` and `D32` compliant).
  - Pre-registered the 3-arm $\times$ 2-seed RTX 5090 experiment (`Arm-0-VSA-Off`, `Arm-1-VSA-Fixed-64` Mode 0, and `Arm-2-VSA-Geom-Frozen-64` frozen E8 root-label $\oplus$ per-token residual Mode 2), the shift-weight sweep $\sigma_{\text{vsa}} \in \{\text{off}, 3, 2, 1, 0\}$, and the count-table comparison (`Unigram = 1,226 / 14,287`, `W2 Code Table = 3,414 / 14,287`, `T2 Bigram = 6,300 / 14,287`).
- **Fresh-context `TEST FITNESS: FIT` & `D68` decision review (`#2888` comment `6397106918`):** Reviewed by fresh-context subagent `d662d613-31f5-4bd2-825f-212acec9efbe`, set `docs/roadmap/items/r-vsa-hamming-binding.md` to `status: held`, and re-rendered `docs/roadmap/README.md`.

## Measurements

- **Baseline Writer & Count-Table Reference Numbers on `DEV` (`val.bin`, `14,287` rows; `P_succ = 3,847` rows) from Sealed `#2831` and `#2845` (`W2`) Artifacts:**
  - **Count Table `Unigram (C_uni)`:** `1,226 / 14,287` (`8.58%` full-panel `top-1`), `3,891 / 14,287` (`27.23%` `top-8`), `400 / 3,847` (`10.40%` `P_succ top-1`).
  - **Count Table `W2 Read-Code Table (#2845)`:** `3,414 / 14,287` (`23.89%` full-panel `top-1`), `6,881 / 14,287` (`48.16%` `top-8`), `2,280 / 3,847` (`59.27%` `P_succ top-1`).
  - **Count Table `T2 Query Bigram (C_T2)`:** `6,300 / 14,287` (`44.10%` full-panel `top-1`), `10,124 / 14,287` (`70.86%` `top-8`), `2,713 / 3,847` (`70.52%` `P_succ top-1`).
  - **`#2831` `Batch 0` (`AddressCellsA8`, `VSA-Off`):** `b_3(Full) = 4,244 / 14,287` (`29.71%`), `DEV top-1 = 133 / 14,287` (`3.13%` of admitted rows), `DEV top-8 = 513 / 14,287` (`12.09%` of admitted rows).
  - **`#2831` `Q1` (`Batch 4,096`, `FrozenZero`, `VSA-Off`, `q1-dev.json` SHA-256 `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`):** `b_3(Full) = 4,292 / 14,287` (`30.04%`), `DEV top-1 = 871 / 14,287` (`20.29%` of admitted rows: `679` strict wins + `192` L1-tie wins), `DEV top-8 = 1,455 / 14,287` (`33.90%` of admitted rows).

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap render` (wrote `docs/roadmap/README.md`, 92 items)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- roadmap check` (clean)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- links --json local/antigravity/links.json` (`0` broken links)
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -p xtask -- docs-catalog --check` (clean)
- `uor-harness loop gates --full` (`4/4` gates PASS)
- `uor-harness loop report --check --order docs/loop/tasks/r-vsa-hamming-binding/antigravity-69/ORDER.md` (PASS)
- `uor-harness loop pr-check --local` (all hard and advisory checks PASS)

## Skills used

- `uor-research-architect`: designed `DiscreteVsaHammingWriterV1` using multiplier-free 64-bit cyclic shifts, XOR role-filler binding, vertical bit-plane majority bundling, and `__popcll` Hamming distance on the `D32` writer.
- `scope-guard`, `solution-integrity`, `working-discipline`, `gates`, `session-close`: ran cancel probe first, kept all edits inside the item's `docs/**` scope, obtained fresh-context `TEST FITNESS: FIT` and `D68` reviews, and verified all gates.

## Still open

- Kernel implementation of `DiscreteVsaHammingWriterV1` in `crates/uor-r4-discrete` and execution of the 3-arm $\times$ 2-seed RTX 5090 GPU run (`docs/harness/prep/r-vsa-hamming-binding-2026-10-25.md` Steps 1–3) once `m1-bvg-k-stage-t-training` (`#2836`) completes and `crates/uor-r4-discrete/**` is free.

## Plain summary

Vector-symbolic architecture (`VSA`, also called hyperdimensional computing) gives every vocabulary word a fixed 64-bit binary code, glues context words to their positions using bit rotations and exclusive-OR (`XOR`, which flips bits where two codes differ), merges the recent context words into a single 64-bit summary by taking a majority vote at each bit position, and scores candidate next words by counting how many bits differ (`Hamming distance`, computed in one hardware `popcount` instruction). An earlier test on 9 October 2026 showed that fixed VSA codes improved language predictions, but it ran on a legacy CPU learner (now barred by the GPU-only rule `D24`), used a runtime multiplication (`Q1.15`, not verified against the multiplier-free integer serving rule `D11`), and failed when geometric codes were rebuilt every `1,000` steps during training (`Mode 2`) because the model had to chase moving codes. In this unit, we inspected the current GPU integer model's output scorer (`writer` in `discrete.cu`) and showed that even when cell admission is intact (`4,292` of `14,287` development test rows admitted at `Q1` in `#2831`), `79.71%` of admitted rows (`3,421 / 4,292`) still lose to rival words in the same cell, and `192` of the `871` top-1 wins sit on an exact coordinate-distance tie broken only by word ID. We authored the complete specification (`DiscreteVsaHammingWriterV1`) and pre-registration sheet (`r-vsa-hamming-binding-2026-10-25.md`) for adding a multiplier-free 64-bit XOR/Hamming context-binding term to `writer` — testing both fixed hash codes (`Arm-1-VSA-Fixed-64`) and frozen E8 geometric root-plus-residual codes (`Arm-2-VSA-Geom-Frozen-64`) against `VSA-Off` and the count-table baselines (`Unigram = 1,226`, `W2 Code Table = 3,414`, `T2 Bigram = 6,300`) — obtained a fresh-context `TEST FITNESS: FIT` and `D68` decision review on `#2888`, and placed `#2888` in `held` status until `#2836` (`Stage T` `BVG-K` training) completes.
