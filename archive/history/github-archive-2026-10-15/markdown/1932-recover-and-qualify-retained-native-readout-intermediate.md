# #1932 — Recover and qualify retained native readout intermediate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T16:57:46Z
- **Updated:** 2026-10-09T13:17:57Z
- **Closed:** 2026-10-08T17:00:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1932
- **Merged:** 2026-10-08T17:00:26Z
- **Merge commit:** 445fcee6f2cb899861e9fe6ea86285b137740238
- **Base / Head:** main / UOR-Foundation:codex/sol-readout-intermediate-candidate-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The original readout construction reached a factual correction at accepted order 147 and lost it before its endpoint. Recover the globally earliest recorded accepted crossing across the fixed factual panel, then apply the original 89-term native objective and all 377 winner guards before coherent export/reload and actual candidate generation. This offline selection changes no serving policy and performs no new gradient, proposal or optimizer update.

The recovered checkpoint passes the unchanged gate (combined CE 5.465593983756579 → 3.4177030191650397), preserves all controls and reproduces native scores after reload. Actual replies remain 8/512 with no gained or lost complete replies, 4/256 complete swap pairs and 491 changed output sequences: useful_candidate=false. One factual reply reaches the next word boundary, retained as the next causal diagnostic question. Parent outputs are authenticated and reused without repeating parent inference.

Validation: 65 focused fitter tests, formatting and optimized CUDA-feature build passed at executed source 4232ad32877f81dfd0a27b0880b2eed95f49256b; exact final-head checks and review follow below. Independent saved-native-algebra audit passed for all 466 objective/guard pools, packing/reload and output membership/EOS/pairs, without independent encoder/backward/full numeric rollout replication. Full sealed result privately preserved and fresh-download verified at HF 2bb7b60de59d74d14bb651a09840c6f212e1cda3. Structured evidence binds source, executable, configuration, report, seal, reader, costs and limitations.

References #820.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6064929665 — 2026-10-08T16:59:31Z — Casey-allard

Exact final head ca5d8040a156fa641bb436e02ed567b2123c1167 validation PASS on owned pod codex/sol-prototype-20261008, job 20261008T165744Z-sol-prototype-20261008-gpu0: standalone formatting for the three touched fitter files, all 65 release/CUDA-feature fitter tests, optimized build; exit=0, completed 16:59:08 UTC. Final build-only executable SHA256 b22b5477385175bb09fc3e654fe558e54330fa8dec7f9dbcc7b15ecc53b959f4; it did not rerun the model experiment. Numerical source byte-identical to executed4232 and actual experiment binary85a6e80c... remains separately bound.

Independent adversarial exact final-head source/document review PASS, no concrete blocker. Saved independent finite-algebra audit78dc59a... and its encoder/backward/full numeric rollout limitations retained. Actual candidate8/512complete, zero gains/losses,491changed outputs,usefulfalse. Complete result private HF2bb7b60de59d74d14bb651a09840c6f212e1cda3 freshly verified. Claim-wording and whitespace checks pass. Historical ruleset acknowledgements execute no tests; validation above is actual exact-head execution.



---
_Comment/review ids in this file: 6064929665_

