# #1785 — mqar-bench: Stage-0 frozen-vector readout swap — the representation is the missing component

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:32:41Z
- **Updated:** 2026-10-09T13:22:32Z
- **Closed:** 2026-10-06T04:33:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1785
- **Merged:** 2026-10-06T04:33:52Z
- **Merge commit:** 81de3ca51cd7ed880c8939ad0ebf69ccae5c5239
- **Base / Head:** main / UOR-Foundation:deepseek/stage0-rank
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Example-only, +507/-0. Holds every learned vector fixed and changes only the score function, separating H_head from H_repr. Pre-registered before any run.

INSTRUMENT GATE PASS twice: A3 = 1.0000 on 192/192 rows, and m-world route world=v2 conversations=2000 seed=9101 cells=true reproduced 747/747 bit-for-bit against #1552.

Vectors proven frozen: A1 re-dumps bitwise identical after restore; run reproduces the documented reference exactly (0.2554).

RESULTS (192 rows, chance 1/N = 0.0377):
  A1 fused L2      key 0.0041  value 0.0595  value r@16 0.2205  NoRead 0.7757
  A2b dot (age 0)  key 0.0048 CI [0.0000,0.0091]  value 0.0093  value r@16 0.1664
  A3 sieve         1.0000
VERDICT: H_repr FIRES; H_head DOES NOT (CI upper bound ~44x below the bar). Under D20 this names a missing component; it retires nothing.

THREE QUALIFICATIONS: (1) 'no usable content identity' is too strong - the fused read reaches 5.95% value top-1 = 1.6x pair-chance and 21x the positional floor, so identity exists but is norm-entangled; a third possibility the dichotomy does not name. (2) The bench's 25.5% is NOT read recall - gold is top-1 0.4-6% of the time and the read contributes diffusely. (3) NoRead dominance is a symptom: A2b cuts it to 0.1334 and retrieval still fails.

CORRECTION verified here: the companion claim that dump_scores=1 is 'parsed but never implemented' is FALSE - it is implemented at mqar_bench_step2/fact.rs:836-843 and the sealed T2 root holds held_out.weights.f32 at 56,623,104 bytes. The claim came from grepping one file.

Also confirmed: ArmSpec has a single Stack variant and stack_config hardcodes StackArch::Geometric, so the geometric read has never been compared to a parameter-matched ordinary attention model (D20 s2 condition 1).

D11: A1/A2 not shippable; A3 is the only D11-clean arm and the only one that retrieves.

Cost: 1,462 s wall / 1,138.5 s train, 1800/1800 steps, ~1 core, zero external cost. Attempt 1 lost to a silent OOM kill; preserved as evidence. Peak RAM not instrumented (honest gap). References #820, #1747, #1552, #1746.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

