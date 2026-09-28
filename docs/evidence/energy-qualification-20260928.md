# Apple Silicon M1 Marginal Serving Energy Evidence & Forensic Qualification (Track T3)

**Author:** Anti-Gravity (Lab 3: Runtime Lab)  
**Date:** September 28, 2026  
**Status:** Qualified Empirical Evidence (References #820, #963, #964, #973)  
**Hardware Platform:** Apple Silicon M1 (MacBook Air / Mac mini, Firestorm performance cores)  
**Serving Boundary Contract:** D11 (Strictly 0 hardware multipliers, 0 hardware dividers, 0 floating-point instructions in served numerical runtime)  

---

## 1. Executive Summary

Under Track T3 marginal energy profiling (`scripts/integer-m1-energy.sh`, run root `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-energy-20260927-233447`), marginal energy consumption was evaluated across sliding full256 context using same-input token stepping to eliminate startup and initialization transients:

$$\Delta E = E(32768) - E(10240), \quad \Delta N = 32768 - 10240 = 22,528 \text{ tokens}$$
$$\text{Marginal Energy} = \frac{\Delta E}{\Delta N} \quad [\text{Joules / token}]$$

- **Native Integer Serving Bundle (`bundle-quaternion-1`, width 256):**
  - $E(10240) = 29.2017\text{ J}$ (mean across $n=3$ repeats: $[23.060, 38.013, 26.532]\text{ J}$)
  - $E(32768) = 122.3207\text{ J}$ (mean across $n=3$ repeats: $[121.902, 129.698, 115.362]\text{ J}$)
  - **Marginal Energy:** $0.00413348\text{ J/token}$ ($[0.003943..0.004388]\text{ J/token}$)
  - Decode rate: $\approx 476.7\text{ tok/s}$ baseline, accelerating to $\approx 906.7\text{ tok/s}$ with grouped LUTs
- **Continuous Float Feedforward Parent (`fit256-quaternion-3`, width 256):**
  - $E(10240) = 32.1687\text{ J}$ (mean across $n=3$ repeats: $[49.376, 23.898, 23.232]\text{ J}$)
  - $E(32768) = 53.7863\text{ J}$ (mean across $n=3$ repeats: $[62.975, 52.846, 45.538]\text{ J}$)
  - **Marginal Energy:** $0.00095959\text{ J/token}$ ($[0.000604..0.001285]\text{ J/token}$)
  - Decode rate: $\approx 1412.3\text{ tok/s}$
- **Marginal Ratio:** Integer is $\approx 4.31\times$ the marginal energy of continuous float per supplied-token step on this test harness.

---

## 2. Mandatory Limitations & Scientific Qualifications

As recorded in the oversight audit (`oversight-audit-20260928-0343/OVERSIGHT_HANDOFF.md`), the following limitations are strictly binding:

1. **Non-Identical Checkpoints (Lineage Divergence):**
   - `bundle-quaternion-1` was packed from `fit-quaternion-rounding-3/packed-model` (training step 8348), which underwent learned parameter rounding from `fit-quaternion-projected-1` (step 7836), originating from `fit256-quaternion-3` (step 7324).
   - Therefore, `bundle-quaternion-1` and `fit256-quaternion-3` are **different models** with different weights and convergence states. This is **not a quality-matched product comparison**.
2. **Whole-System Host Power Telemetry (Lack of Isolated SoC Rail Validation):**
   - Measurements were collected via non-root `macmon` sampling `sys_power` (whole-system battery/charger rail Watts).
   - Hardware SoC internal rail powermetrics validation was unavailable due to non-root execution constraints (`sudo powermetrics` required).
   - Under non-root host sampling, background OS daemon activity and display/system power contribute variance; one inspected integer repeat reported zero net SoC power during host fluctuations.
3. **Attribution: Software Emulation vs Memory Bandwidth:**
   - The $4.31\times$ energy difference cannot be scientifically claimed as resulting solely from software multiplication emulation.
   - Per-token memory traffic:
     - Integer width-256 reads $847,876\text{ bytes}$ of hard parameters plus lookup tables ($\approx 0.98\text{ MB/token}$) in discrete integer representations.
     - Integer width-576 reads $2,735,420\text{ bytes}$ ($\approx 2.6\text{ MB/token}$) of hard parameters (vocabulary projection alone touches $4096 \times 288 = 1.18\text{ MB/token}$).
     - Memory traffic, cache-line pressure, and memory bus power contribute significantly to Apple Silicon DRAM controller energy.

---

## 3. Profiling-Guided Grouped-LUT (T-MAC Style) Acceleration

Execution profiling using `/usr/bin/sample` on 32,768 steps identified two major execution hotspots in native integer serving:
1. **Probability Mixture Loop:** Generic 128-bit `checked_mul` and `div_round` accounted for 38.7% of step time.
   - **Resolution:** Replaced with precomputed radix-16 table lookup (`build_fraction_table`, `mul_fraction_radix16`, `div_round_100m_raw`, and `mul_shift_add_i64`), operating strictly with zero multiplier instructions.
2. **Vocabulary Projection:** 4096-row inner products accounted for 19.9% of step time.
   - **Resolution:** Grouped LUT (T-MAC style) pair tables:
     - Width 256: `build_pair_tables_256` and `low_bit_dot_8_pair_tables` pre-combine 2 adjacent 4-bit nibbles into 256-entry tables, halving table lookups and additions.
     - Width 576: `build_pair_tables_576` and `low_bit_dot_4_contiguous_wide_pair_tables` in `project_vocab_with_products_into` pre-fetch contiguous packed parameter rows, reducing memory traffic and load latency.
   - **Output Invariant:** Verified 100% bit-for-bit identical output over 32,768 steps (trace SHA-256 `a88db0edad0b20c227cf06201bc381009d7382237bbee6bceaae25dd62055179`).

---

## 4. Serving Runtime & Frontend Audit Receipts

- **Binary Identity:**
  - Artifact: `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/uor-chat`
  - SHA-256: `c389dc768c0efec1ced9923b8d8bf64dc0fc2340157f96a222634a5b682d0c0d`
  - Compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
  - Flags: `release profile (opt-level=3, codegen-units=16)`
- **Transitive Disassembly Audit:**
  - 55 matched symbol ranges checked: 0 Class I (multipliers), 0 Class II (dividers), 0 Class III (floats).
  - 113 reachable call-graph functions checked from serving roots: 0 violations.
  - Missing mandatory symbols: 0.
  - Strict TAP v13 suite: 5/5 OK.
- **58-Turn Parity Verification (`dialogue576_turn_parity`):**
  - 38 requests, 58 turns, 2,526 step calls replayed.
  - 1,433 generated token decisions: **0 departures (100% bit-for-bit exact token match)**.
  - Peak RSS: $27.33\text{ MB}$ (invariant: $< 35.0\text{ MB}$ ceiling).
  - Average step latency: $2.611\text{ ms/step}$ (invariant: $\le 4.0\text{ ms/step}$ ceiling).

---

## 5. Interface Dependency on Claude's Selected Model

The runtime lab's native integer serving engine is fully verified for width 576. Further integration depends on:
- Completion of the 281 legal-code updates on T4 #1433 by Claude (Lab 1, branch `codex/dialogue-child-code-choice-20260927`).
- Once released, the new rounded child bundle will be loaded via `Bundle::load` to verify that dialogue weights roundtrip without precision distortion.
