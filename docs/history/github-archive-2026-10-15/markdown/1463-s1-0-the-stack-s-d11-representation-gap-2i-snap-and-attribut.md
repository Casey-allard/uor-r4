# #1463 — S1.0: the stack's D11 representation gap, 2I-snap and attribution diagnostics

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T17:08:02Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-09-28T17:15:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1463
- **Merged:** 2026-09-28T17:15:50Z
- **Merge commit:** ccd19cb63e47668142635b8b101440122859d5b0
- **Base / Head:** main / UOR-Foundation:lab/claude/s1-stack-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973 under #820. **Lab 1 (Claude), locally.** Evaluation only: no fit, no weights changed.

## Results

Every number below was re-read from sealed report roots. Each root passes the `report_output::verify` rule: every listed BLAKE3 hash matches and no unlisted files exist. See [`s1-stack-serving-measurements-2026-09-28.json`](docs/evidence/s1-stack-serving-measurements-2026-09-28.json), which is assembled by script.

**S1.0: the export gap.** The cycle-4 `geometric_s1` (`3eb1ebbb…`) in the D11 interim format (4-bit maps, groups of 32), on 512 development windows (131,072 targets):

| Export | Integer − float | Weight rounding | Integer arithmetic | Top-1 agreement |
|---|---:|---:|---:|---:|
| Round to nearest | **+0.0362** | +0.0362 | −4.0e-07 | 0.9065 |
| GPTQ, 64 calibration windows from the model's training split | **+0.0257** | +0.0257 | +6.8e-07 | 0.9183 |

Both miss S1.2's gate of ≤ 0.02 nats. The integer arithmetic is exact to 10⁻⁶, so the gap is all weight representation.

**S1.0b: evaluation-only 2I snap.** Every transport quaternion is replaced by the nearest of the 120 unit icosians:
- cycle-4 model: +0.0262 nats;
- D1 `rot_s1`: +0.0256;
- D1 `rot_s2`: +0.0260.

That is above the 0.02 option threshold, so exact group-table transport serving is not adopted. The result is reported, not gated.

**S1.0c: attribution.** The float model is scored with one tensor group at a time from the export. For the GPTQ export:

| Group | Cost (nats) |
|---|---:|
| Head | **+0.0119** |
| MLPs | +0.0061 |
| Mixer maps | +0.0034 |
| Embedding | +0.0026 |
| Per-channel scalars | +0.0013 |

The groups sum to +0.0254, against +0.0257 for all of them together. With every group from the artifact, the tool reproduces `lut-evaluate`'s reference exactly.

**Owner decision, 17:00 UTC.** Lab 1 adds QAT, and D4 gets the measured gap as its codec target. The D11 port (S1.1), the bundle (S1.2) and the audit (S1.3) continue.

## Code

**`StackModel::logits_with_transport(ids, batch, time, snap)`**, evaluation only:
- it runs the composed recurrence reference with each unit transport quaternion optionally snapped to the nearest root;
- `composed_recurrence` gains the `snap` argument;
- the `#[cfg(test)]` on that impl is lifted.

**New test:** `transport_logits_equal_the_forward_until_snapped` (composed equals fused within 1e-4; an identity-only snap changes the logits; a 2I snap stays finite).

**`geometric-stack` modes:**
- `snap-evaluate` (S1.0b);
- `rounding-attribution` (S1.0c). It maps every float tensor to exactly one group and errors on an unknown name. Each group keeps a normalization with the matrices its gain is folded into.

**Docs:**
- the record [`s1-stack-serving-measurements-2026-09-28.md`](docs/integration/s1-stack-serving-measurements-2026-09-28.md);
- ROADMAP §2b rows and a director-log entry;
- the `current-state.md` September 28 entry, including D1 and D2, which were stale.

## Checks

- `cargo fmt --check` passes.
- All 16 `geometric_stack` tests pass on the library code in this PR: `cargo test -p uor-r4-training --release --offline geometric_stack`. After that run, only the example changed (the attribution mode and one moved doc comment), and it builds in release.
- `python3 scripts/check_claim_wording.py` passes.

## Limits

- One reference model on the code development split, not the milestone checkpoint.
- Top-1 agreement is the greedy decision-flip complement, not a generation test.
- The S1.0 roots record command lines but not executable hashes (the snap and attribution roots do). The integer NLL is a deterministic function of the recorded artifact.

## Cost

About 20 minutes of 2-thread evaluation on the M1, with no model slot, and 17 MB of new SSD files.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

