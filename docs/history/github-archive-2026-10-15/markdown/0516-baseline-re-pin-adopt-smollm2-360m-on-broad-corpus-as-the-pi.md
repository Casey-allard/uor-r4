# #516 — Baseline re-pin: adopt SmolLM2-360M-on-broad-corpus as the pinned baseline (keep stories15M as continuity)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T23:42:26Z
- **Updated:** 2026-08-09T06:26:35Z
- **Closed:** 2026-08-09T06:26:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/516
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Execute the #320 P3 baseline re-pin, now that #509 returned POSITIVE (a competent-broad teacher lifts the broad-text substrate off the floor).

## Decision (maintainer-approved)
- Adopt **SmolLM2-360M** (`models/smollm2-360m-instruct.json`, rev `2366112...`, source kappa `eb23c3e8...`) as the new **pinned baseline teacher**.
- **Keep stories15M alongside** as the continuity partition (unchanged). BASELINE.md sec 2 already declares both partitions; this adds wiki-360M as the certified natural-partition baseline rather than replacing stories.
- Scale: **~10k-15k Simple-Wiki articles (~2-3M records)** with scaled-capacity cover **on** and `capacity_ref_n` retuned to the new corpus size. Confirm the exact knee with a sub-sample saturation sweep (see #514).

## Steps
1. **[dev hardware]** Observe SmolLM2-360M on ~10-15k Simple-Wiki articles -> ~2-3M records. (The sandbox cannot finish this: the #509 observe managed 176/3000 articles at ~15s/article. This is a multi-day teacher run on the pinned dev machine.)
2. Turn scaled capacity on and retune `capacity_ref_n` (currently fixed 400k; the #460 arms anchored at ref50k) to the chosen scale.
3. Recorded compile -> cover -> score (`relative_tla`) -> `evaluate-report` (teacher floor). Confirm the saturation sweep flattens at the chosen N.
4. Update the certified fidelity baseline: `docs/transformerless/BASELINE.md` sec 3 (fidelity table + era note) and `PROOF.md` P-certificate with the new pinned numbers + artifact kappa.
5. Gate E kappa fixture (`tests/fixtures/baseline_kappa.json`): re-pin **only if** the tiny determinism-corpus compiler inputs change - `dump_baseline_kappa` -> review diff -> `TLESS_REPIN_WRITE=1`. If only the fidelity baseline moves, leave it.
6. Decide whether to also flip the deployed default (`src/model.rs:13 DEFAULT_CHAT_MODEL`, server/tless paths) from 135m to 360M, or re-pin the certificate only. These are separable.

## Reference
#509 broad-corpus result and provenance: docs/smollm2_teacher_baseline_320.md (P3 section). Scale sizing: #514.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

