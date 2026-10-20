# #1620 — Compile geometric values from strict q4 and align training hard choices

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T14:34:06Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-10-02T14:37:18Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1620
- **Merged:** 2026-10-02T14:37:18Z
- **Merge commit:** 1b3be3c0b6efe08bfe3aa8b7dab6bf7ef2dc532a
- **Base / Head:** main / UOR-Foundation:codex/geometric-value-q4-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The K2 geometric value producer previously retained unrestricted learned coefficients, and the composition training path could not pass ordinary answer gradients into them. This adds an opt-in strict signed-four-bit source at fixed q/4 scale, independently regenerated native tables, and value credit through the frozen two-sided H4 composition bank.

The first loaded projection exposed a training/serving hard-choice mismatch: 302/512 packet traces differed despite identical final answer predictions. The strict training caller now uses authoritative native context/span codes and current packed coefficients for integer hard choices on every forward; F32 scores supply only the declared backward surrogate. Legacy source/artifact behavior remains available.

Fixed retained panels, initial projection evidence and all accepted parents are preserved. No optimizer fit, coefficient-scale sweep, value-alphabet change or GroundedSession edit is included. Other wide producers, bounded parameter access and the floating tail remain open.

Validation: all12 initial focused cases plus the new hard-choice/live-update case and integrated-reader replay pass. `cargo fmt --check`, claim-wording and diff checks pass; release example rebuilt and exercised through independent source/native export/reload.

Both repaired projection attempts have zero hard-packet/Q16 mismatches on all512 rows. Every native prediction, answer logit and reader record stays identical to the initial projection; maximum source/native answer-logit difference is0.00009871. Four diagnostic B8 ordinary-answer backwards connect all ten value families with no bank gradients, frozen-source mutation or optimizer updates. Workers10.054/10.265 seconds at about310MB RSS. Native answers remain451/512 versus467/512 parent (4 gains20 losses; CE worse in all four panels).

The canonical plan/current-state now prescribe preregistered bounded answer-only learning of these q4 values with bank/upstream/tail frozen, retaining both projected and unrestricted controls. No perfect recovery gate or scale/alphabet sweep is introduced.

References #1512 and #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5954773908 — 2026-10-02T14:36:25Z — Casey-allard

Independent mathematical and engineering reviews approve the changed source at its stated scope. The non-author result audit independently compared all512 native rows, answer-logit bits and saved query-reader records against the first projection: unchanged. All16 source/native/envelope files perseed are byte-identical, including metadata. All512 repaired packet/Q16 traces agree; the302 initial mismatches remain preserved. Four27-file sealed attempts, frozen file/base identities, all40 finite family-gradient observations, absent bank gradients, zero updates and unchanged RNG/input bytes were checked.

Actual diagnostics contain no PRESENT_ZERO atoms; zero-escape behavior is covered by the focused authored test only, not claimed as naturally measured learning. Projection quality remains451/512 vs467/512 parent with4 gains20 losses. No q4 fit is claimed.

Result-review SHA25666a2b5212dbdce854eefc90d1aaa199ffe147ea0730b2b425a2c292d65ba5bdf; mathematical bridge review1f2ffc74a1dceafd34d7740a38df59002003fb8eed36dcdf715c8d2e03b7586c; driver review7e13c5e0bc40e91cfa0b80d6027e058903e88f4b95099dbb7a68f49b3d490334.

The five required historical CI statuses passed as compatibility acknowledgements; they are not local execution evidence. The advisory delivery-evidence publisher failed and is not represented as passing. Local executed checks and retained loaded attempts supply the scoped validation reported here. Protected merge queue only; no admin bypass.



---
_Comment/review ids in this file: 5954773908_

