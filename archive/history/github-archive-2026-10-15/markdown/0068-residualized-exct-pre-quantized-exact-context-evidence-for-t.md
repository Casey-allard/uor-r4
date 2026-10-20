# #68 — Residualized EXCT: pre-quantized exact-context evidence for the deployed path

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:18Z
- **Updated:** 2026-07-23T02:02:09Z
- **Closed:** 2026-07-23T02:02:09Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/68
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64, and the F-ablation decision. The deployed R4G1 path (src/r4g1.rs) currently disables EXCT because the reference scorer quantizes counts to log-domain at probe time (float in the runtime path — contract violation). Move the quantization to compile time: store ΔX entries as ScoreQ residuals in EXCT with the standard storage descriptor, so the deployed scorer reads integers only and exact-context evidence becomes deployable. Acceptance: deployed scorer uses EXCT with zero float ops (source scan); Gate C deployed-mode (with residualized EXCT) numbers equal or beat the probe-quantized reference; witness replay covers the new entries (Theorems 6/10 intact); R4G1.md EXCT payload spec updated.


---

## Definition of Done
- EXCT residuals are quantized at compile time into ScoreQ entries with the standard storage descriptor; the deployed scorer reads integers only.
- The deployed path (src/r4g1.rs) re-enables exact-context evidence under the integer-only contract.

## Acceptance criteria
- [ ] Source scan: zero float ops in the deployed EXCT read path.
- [ ] Gate C deployed-mode (residualized EXCT) equals or beats the probe-quantized reference numbers.
- [ ] Witness replay covers the new EXCT entries; Theorems 6/10 tests stay green.
- [ ] `docs/transformerless/R4G1.md` EXCT payload spec updated to the residualized layout.

## Comments (1)

### comment 5053503110 — 2026-07-23T02:02:09Z — auser

Complete (7550422). Acceptance evidence against the DoD:

1. Zero float ops in the deployed EXCT read path: RX1 residuals are baked at compile time (score.rs emit_residual_exct) and parsed/read as integers only (from_le_bytes); the machine-checked source scan asserts the accumulation core carries no f32/f64 and no */% value arithmetic, and the single probe-time float helper (quantize_exct) is delimited compiler-side, reached only by the legacy TLS1 certifier path — never by deployed RX1.
2. Deployed-mode equals or beats the probe-quantized reference: new test rx1_baked_residuals_match_probe_time_quantization proves every admitted baked score equals the probe formula exactly (add-one smoothed exact value with root prior cancelled). Deployed Gate C (Rule 1+2 with RX1) matches the TLA3 baseline's argmax on all 30,036 held-out positions with better bits/token (9.86 < 11.88).
3. Witness replay covers the RX1 entries: 64/64 replay (Theorems 6/10 preserved; teacher_cid chain enforced when a teacher container is supplied, and r4g1.rs passes Some(teacher_bytes)).
4. R4G1.md EXCT payload spec updated: full RX1 layout in section 5 (magic, levels, per-context key/total/entries, bake formula, integer-only deployed path, legacy TLS1 kept certifier-side); §9.5 records the layout adoption.

Note for the record: the deployed server adapter (src/r4g1.rs) passes Some(teacher_bytes), so the live EXCT probe is wired (artifacts present); constructing the scorer with teacher=None intentionally skips EXCT for the no-EXCT Gate C columns.


---
_Comment/review ids in this file: 5053503110_

