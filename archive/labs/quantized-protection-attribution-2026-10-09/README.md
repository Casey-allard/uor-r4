# Saved quantized-protection attribution

## Question and decision

[Claim on M2](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6090942116).
Why do the four already measured displacements from [#2101](https://github.com/UOR-Foundation/uor-r4/pull/2101)
fail the protected surrogate screen after their continuous direction passes?
This is a saved-arithmetic analysis, with no new candidate, model execution,
backward pass, native scoring, held-out draw or change to acceptance policy.

**KEEP** the exact reconstruction and bounded attribution. A protection-aware
joint discrete transaction remains **NOT YET PROMOTED**; no model is repaired
or qualified by this analysis. The previous candidate remains **REJECT** and
accepted whole-answer development performance remains 8/512.

## Exact artifact, data and configuration

The immutable input is the completed original-parent attempt3 from #2101:

- model source `2c31a22e6fbef3bd37dade8cbb7daae79f13876a`;
- binary SHA-256 `1c5af0fb13b4d957b5665037ca437486985a882260882dca5c07c82d4fdbc848`;
- configuration SHA-256 `8af92899f3aff976ed2acd030837767f9f65827c03b4ea3e559594f759d501b0`;
- report SHA-256 `b1d27f7b15e65d0aa8ea7e0a995b8bc76979610f97dbcb3055c8bbb434d0ac9b`;
- manifest SHA-256 `cdadd7c5cbbbdca2225a71006ab58f70129e8df9e015b92f35b19359028a7167`.

The source archive is `icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar`,
482,161,152 bytes, MD5 `467c102adc510a6352951596b969f1be` (verified restore).
Its model root is `sol-protected-joint-20261009/runs/protected-joint-0001-attempt3`
beneath the archive's enclosing directory. The [previous configuration](../protected-joint-construction-2026-10-09/model-config.json)
retains the complete data/operator identity: development episode
`development-diverse-length4-07-swap0-q0-forward-home`, input245,15 conditional
positions including EOS,17 reference roles,380 original winner guards and391
unique states. Original coupled initializer/frozen U,960 Prefix coefficients and
960 Generate unary coordinates are unchanged. This is not the accepted no-U
parent or fresh/held-out evidence.

## Method and verification

The [analyzer](analyze.py) verifies the complete 1,676-file set and BLAKE3 seals,
SHA-256-bound masters/gradients/Jacobians and receipt chain. It reads the saved
continuous direction without recomputing the 256-pass projection, reconstructs
all four original radii1/2/4/7, and matches **7,680/7,680 destination f32 bit
patterns**, every actual displacement, saved normalized-J residual, tolerance
and CE-gradient dot product exactly. The [result](result.json) binds the analyzer
hash and input seals. These are arithmetic checks, not another native evaluation.

For each coordinate, let `s = eta*d`, `x = fl64(m+s)`, `c = clamp(x)`,
`f = f32(c)`, `z = round_away(4*f)/4`, and `v = m` when the native code is
unchanged, otherwise `v = z`. The six contributions are:

1. intended motion `s`;
2. addition/subtraction residual `(x-m)-s`;
3. clamping `c-x`;
4. f32 conversion `f-c`;
5. quarter rounding `z-f`;
6. fractional-master preservation `v-z`.

The measured closure `(v-m)-sum(contributions)` is recorded, not redistributed.
It is zero for every coordinate here; raw-J dot closure maxima range from
1.24e-15 to4.44e-14 across the four trials. Original/destination bits retain
signed zero. Focused arithmetic checks cover both tie signs, negative-zero
retention, fractional no-op, saturation, a changed negative tie and shape rejection.
No Rust training/serving path is modified or rebuilt.

Every row retains signed raw-J stage and Prefix/Generate contributions. Unit-L2
Jacobian rows are used for the existing protection screen and descriptive
concentration statistics; raw J is used with log-mass margins. Positive and
negative contributions are both retained so cancellation is visible.

## Observed attribution

All numbers below concern the same four saved vectors and380 original guard
rows; each row's attribution totals are conditioned on that vector's violated
set, not a common population across radii.

| Radius | Violated rows /380 | Changed Prefix /960 | Changed Generate /960 | Quarter-round share of adverse conversion-stage sums | Top10 coordinate share of adverse conversion pressure | Prefix share of that pressure |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 104 | 11 | 1 | 95.32% | 41.41% | 89.14% |
| 2 | 170 | 81 | 1 | 96.33% | 32.67% | 89.08% |
| 4 | 98 | 154 | 6 | 95.33% | 20.21% | 79.93% |
| 7 | 93 | 279 | 42 | 90.41% | 14.37% | 68.31% |

The stage share divides `sum_i max(0,-unitJ_i dot stage)` for quarter rounding
by the same sum over all five conversion stages, on violated rows. It is a
descriptive sum after coordinate cancellation within each stage, **not a fraction
of net failure caused**. Quarter rounding dominates this statistic. Clamp-stage
adverse sums rise from0.06139 to0.68735, compared with quarter-round sums
2.42126/9.32223/7.90874/6.63919; f32-cast adverse sums remain below8.8e-7 and
addition/subtraction residual sums below1.9e-15. There are **zero exact half
ties** in all four vectors. A special tie-breaking repair is unsupported.

Coordinate pressure is `sum_i max(0,-unitJ_ik*(actual_k-intended_k))` over
violated rows. Corresponding relief totals are5.28466/11.31893/9.86023/11.70223
against adverse totals7.67175/20.69073/17.97458/18.77221. These substantial
opposing terms prohibit reading pressure as net guard loss or as a list of
coordinates to remove. The top100 shares are70.74/61.96/50.13/43.07%; the larger
radii distribute pressure more broadly. Eighteen rows violate every radius,
which is not an infeasibility certificate. Prefix pressure dominance is
conditional evidence about these vectors, not global Prefix deficiency.

Generate clamping occurs at368/368/369/369 coordinates, versus Prefix0/0/0/2;
count alone overstates its contribution relative to rounding. Fractional-bit
preservation acts nontrivially at19/19/17/16 Generate coordinates and zero Prefix
coordinates. It is retained exactly; removing it is not supported by this audit.

## Original margins and the native boundary

The diagnostic `ln(original_winner_mass/original_rival_mass) + rawJ dot actual`
uses the original selected pair and a local surrogate. It does not rerank native
tokens, evaluate the changed donor, or bound curvature/other competitors.

| Radius | Screen-violating rows | Of those, positive linearized original margin | Of those, nonpositive linearized margin |
| --- | ---: | ---: | ---: |
| 1 | 104 | 101 | 3 |
| 2 | 170 | 164 | 6 |
| 4 | 98 | 94 | 4 |
| 7 | 93 | 90 | 3 |

Thus most screen failures are predicted margin reductions without a predicted
crossing for that fixed pair. A positive prediction does not establish actual
winner retention; a nonpositive prediction does not establish actual loss.
There were **zero native proposals and zero commits** in the source experiment.
The accepted artifact, strict protection policy and all native acceptance checks
remain unchanged. No guard weakening follows from this table.

## Reproduction, cost and retained evidence

Run `python3 analyze.py --self-test`, then `python3 analyze.py MODEL_ROOT NEW_OUTPUT`
from this directory, with the existing Python `blake3` library available. This
is evidence arithmetic, not a Python model or new product dependency. Outputs
are exclusive and retain all coordinates, all guards and signed contributors;
failed output roots must be preserved and retries use a new root.
The measured analysis took10.7032 seconds on local CPU, no GPU/pod spend,
with a claimed limit of2threads/1GiB analysis RAM and3GiB transient storage;
final-head verification records process timing/RSS separately.
Complete preparation/review/delivery/cleanup is charged to the shared ledger;
no token budget was introduced. The retained analysis archive is
`icloud:UOR-R4/results/codex/codex-quantized-protection-attribution-20261009.tar`;
its checksum and final-head review/check identities are recorded with the delivery
receipt after upload. It contains the complete per-coordinate/per-guard outputs
and references the original archive rather than duplicating it.
Two [read-only specialist reviews](source-reviews.json) found no blocker; they
did not execute the analyzer or independently score a model.

**Next:** Specify a bounded protection-aware joint discrete transaction: choose
legal Prefix/Generate changes jointly against the actual-displacement surrogate
constraints and original task-descent objective, instead of independent rounding
after continuous projection. Freeze its finite search, cost and distinct outcome
decisions before implementation/execution; retain every native CE/reference/guard
and whole-answer qualification condition. The current evidence justifies this
question, not a particular solver or a claim that a feasible update exists.
No radius/pass sweep, coordinate exception, candidate composition, new substrate
or weaker guard is authorized by this result.
