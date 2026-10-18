# Energy Measurement Protocol v2 (Second-Instrument RAPL Cross-Check)

- **Date:** 2026-10-18
- **Author:** Yazan Barakat (`@Cyborg-Hawk-AI`)
- **Related issues / PR:** [#820](https://github.com/UOR-Foundation/uor-r4/issues/820), [#2656](https://github.com/UOR-Foundation/uor-r4/issues/2656), [#2033](https://github.com/UOR-Foundation/uor-r4/issues/2033) (`[M5]`), [PR #2654](https://github.com/UOR-Foundation/uor-r4/pull/2654)
- **Raw receipt:** [`ops/measurement_report_rnd.json`](measurement_report_rnd.json)

## 1. Purpose

Provides a second-instrument Linux RAPL (`powercap` package domain) energy measurement protocol for the D11 integer stack runtime (`crates/uor-r4-integer`) to complement the Apple Silicon M1 `macmon` measurement protocol (`scripts/energy_per_token.py`).

## 2. Protocol & Reporting Shape

1. **Fixture generation (`mk-synthetic-stack`):**
   - Build a deterministic `UORLUT01` synthetic `ra` stack artifact (`seed = 5`, `469,252` bytes, `weights_per_token = 15,968`).
2. **Baseline idle power (`idle_rate_w`):**
   - Sample Linux RAPL package energy counters (`/sys/class/powercap/intel-rapl:0/energy_uj`) over an unloaded window prior to execution (`27.307 W` on the `rnd` 16-core `x86_64` host).
3. **Sustained workload measurement (`work_rate_w`, `gross_j_per_tok`, `net_j_per_tok`):**
   - Run 3 sustained token-generation passes on the D11 stack engine (`17,304`, `17,025`, `17,241` tok/s; mean `17,190` tok/s, peak RSS `3.67 MB`).
   - Report both **gross** energy per token (`3.172 mJ/tok` mean; `3.135` min, `3.199` max) and **net idle-rate-subtracted** energy per token (`1.530 mJ/tok` mean, CI95 `±0.037 mJ/tok` at `40.5 W` work rate).

## 3. Scope & Caveats

- **Provisional synthetic-stack scope:** These numbers measure the `15,968` weights/token synthetic `ra` fixture, not a full-scale trained model artifact, and include process spawn overhead in the loop.
- **DRAM counter availability:** Hardware uncore/IMC DRAM read/write counters were unavailable on the `rnd` VM (`NOT MEASURED` in `ops/measurement_report_rnd.json`).
- **Authoritative `[M5]` record:** Per project policy, the owner's M1 `macmon` measurement on the accepted `[M4]` (`#2032`) `UORLUT01` model artifact remains the authoritative `[M5]` (`#2033`) qualification gate.
