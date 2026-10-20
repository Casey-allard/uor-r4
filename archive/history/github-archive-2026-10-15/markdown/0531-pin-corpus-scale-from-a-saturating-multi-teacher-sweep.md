# #531 — Pin corpus-scale β from a saturating multi-teacher sweep

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-09T07:12:19Z
- **Updated:** 2026-08-11T11:10:54Z
- **Closed:** 2026-08-11T11:10:54Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/531
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up from #514 (closed). The corpus-scale law, `recommend-scale` calculator (#517), and `scale_sweep.sh` harness are shipped and verified end-to-end. What remains is pinning the exponent **β** (currently provisional 0.5, bounded [0.45, 1.0]) from real saturation knees.

The 360M sweep (#514) showed the largest corpus we can observe on dev hardware (360,924 records) is still below the ~1–2M coverage knee, so it yields no `N_knee`. Pinning β needs:

- at least one teacher observed **past** its knee (a ~2M-record observe — a multi-hour-to-multi-day teacher run), ideally the 15M / 135M / 360M trio so β falls out of least-squares on `log N_knee` vs `log S`;
- then `scale_sweep.sh <corpus> 49152 <sizes spanning the knee>` per teacher, and the β fit written into `docs/scaling_law.md`, replacing the provisional value.

This is compute-bound, not tooling-bound. See `docs/scaling_law.md` (Status) and the #514 sweep table.

## Comments (1)

### comment 5246514380 — 2026-08-10T21:59:16Z — auser

Open-weight recompilation dependency note (2026-08-10): no competing corpus-size or “improve model quality” issue is being opened. Phase 6 will consume #531’s saturation-selected scale and β result as an input, while source identity (#597) and source-adapter work remain separately bounded. The plan will preserve #531’s current compute-bound scope and use its eventual corpus/source CIDs in target-operator fitting and certification; no change to #531’s exit rule is requested.



---
_Comment/review ids in this file: 5246514380_

