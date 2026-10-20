# Cycle 5: sparse memory layers, and which geometry should address them

2026-09-27 · Claude lab track · References #820, #973 · Follows [cycle 4](geometric-stack-cycle4-2026-09-27.md)

**Status.** A plan and implementation note from the lab track, not a decision record. The implementation exists and is tested. The experiment in §4 is **NOT_RUN** and **held**: under the director plan no new learner starts until the base decision ([ROADMAP](../../ROADMAP.md) §9, 2026-09-28). Since then the cycle-4 main comparison has been read ([cycle 4 §6](geometric-stack-cycle4-2026-09-27.md#6-main-comparison)), and the director's D0 decision ([#820](https://github.com/UOR-Foundation/uor-r4/issues/820), 2026-09-28) made the `rrarra` stack the main-line core. So the reads-only base of §4 is superseded. If this experiment runs, its base is the `rrarra` stack as settled by D0 and the transport attribution D1, and §4 is amended before any arm runs. Arms on the reads-only base count only as transformer-comparator index evidence (ruling 8). Labels: **Measured**, **Derived**, **Literature**, **Hypothesis**.

## 0. Why

**Every model so far reads every weight per token.**
- The cycle-4 stack, its transformer control and the retained native model all read their whole weight store per token.
- [D5](DECISIONS.md#d5--per-token-parameter-sparsity-is-the-terminal-serving-invariant) makes per-token sparsity the end-state serving contract: "geometry replaces matmul" keeps its content only if routing selects which learned rows are read.
- D5 also names the contest that has never been run: geometric routers (prime/zeta/H4/VSA) against ordinary learned indexes at a matched access budget.

**The phase-2 roadmap already places the geometry there.**
- Its M5 rung is parameter memory: none, learned product keys, E8 or sign sub-codebooks, or n-gram addressing. Its criterion is at least 0.02 nats, or a factual-recall gain, at equal active parameters ([phase-2 note §4](geometric-lab-phase2-2026-09-26.md)).
- D10 left one question open: "memory layers addressed by fixed geometric codes (sparse parameter access without a learned gate)."

**Cycle 4 says where not to look.**
- The Lorentz score makes no measurable difference inside the dense stack: two seeds give paired differences of +0.021 and −0.021 nats.
- The quaternion transport's effect cannot be separated from the MLP width it trades against.
- So this cycle moves the geometry from the score of a dense read to the index of a sparse memory.

## 1. The memory layer

`crates/uor-r4-training/src/stack_memory.rs`. It is a product-key memory (*Literature*: Lample et al. 2019, arXiv 1907.05242) and replaces the MLP of chosen layers.
- **Selection.** Per head, the query's two halves score two sets of `S` sub-keys. The top `k` of each side form `k²` candidate slots, each scored by the sum of its halves. The best `k` slots are read.
- **Values.** One table of `S²` rows is shared by all heads. The softmax weights of the selected rows mix them, and the heads' reads are summed.
- **Access.** A token reads `heads × k` value rows. The table can be several times larger than all the dense layers together.

The index geometry is the variable:

| Index | Sub-keys | Score | Slots (`S²`) | Key width per half |
|---|---|---|---:|---:|
| Learned, Dot | learned | `<q, k>` | 65,536 | 64 |
| Learned, Lorentz | learned | `−β d(q, k)`, the hyperboloid distance of the lifted points, `β` per head | 65,536 | 64 |
| Fixed H4 | the 120 unit icosians (600-cell vertices) | `<q, k>` | 14,400 | 4 (a quaternion) |
| Fixed E8 | the 240 E8 roots at unit norm | `<q, k>` | 57,600 | 8 |

- A fixed codebook addresses the memory by fixed geometric codes: only the query map and the values learn.
- The H4 index reads a quaternion half-query against the binary icosahedral group, the same group the project's paired-H4 geometry uses.
- The selection is discrete. Its gradient holds the selected slots fixed, as in the original, and ties break toward the lower index so the backward pass reselects exactly.

**Checks (*Measured*, unit tests).**
- The product-key selection equals a brute-force top-`k` over all `S²` slots.
- The op's gradients match central finite differences on every coordinate for the Dot and Lorentz scores (228 and 230 coordinates).
- Both codebooks have their defining inner products: the 600-cell's `{0, ±1/2, ±φ/2, ±1/(2φ), −1}` and E8's `{0, ±1/2, −1}` at unit norm.
- Codebook keys receive no gradient while the query map and values do.
- Stacks with memories are causal and round-trip through save and load.

**Training cost (*Measured*, one thread on the shared sandbox, 8 updates of 4,096 tokens).**
- Reads-only stack: 651 tokens/s.
- The same stack with a 65,536-slot memory in layer 3: 490 tokens/s. It has 25.6M parameters, of which 6.8M are read per token, against 7.15M read of 7.15M.
- Before this cycle's optimizer change the memory stack ran at 378 tokens/s. `StackAdamW` now updates each variable in one parallel pass with the same f32 operations in the same order: a test checks the step bit for bit, and eight-update runs give identical model hashes.

## 2. Parameters, active and idle

*Derived*, for width 288 with 4 heads, `k = 32` and a memory in place of one MLP (hidden 764):

| Part | Parameters | Read per token |
|---|---:|---:|
| MLP it replaces | 660,096 | 660,096 |
| Memory query map (`4 × 128 × 288`) | 147,456 | 147,456 |
| Learned sub-keys (`4 × 2 × 256 × 64`) | 131,072 | 131,072 |
| Values (`65,536 × 288`) | 18,874,368 | 36,864 (`4 × 32` rows) |

The memory arm reads 4.8% fewer parameters per token than the dense arm and holds 3.6 times as many.

## 3. Serving plan

Nothing in this section is implemented. Serving follows the programme's native rules ([ROADMAP](../../ROADMAP.md) §0, R1–R5):
- no floating point;
- no multiplier instruction in served kernels;
- dense per-token access only as a labelled interim.

D10's runtime-multiplier exception does not apply (ROADMAP ruling 10). The memory would serve as follows:
- **Sub-key scores.** Learned sub-keys are a learned map of the query, served as a ≤4-bit table GEMV of adds, shifts and table reads. Fixed codebooks need no learned map:
  - The H4 index can use the exact integer H4 classifier of #1435.
  - Unscaled, the E8 roots have coordinates 0, ±1/2 and ±1, so their inner products are signed sums with one halving. The unit-norm scale is common to all roots and folds into the exp table.
- **Top-k.** Integer comparisons.
- **Softmax.** The exp table.
- **Value mixing.** Runtime weights times learned 4-bit values, through tables of each selected weight's multiples and shift-add group scales, as R2 requires for products of runtime values.

The dense per-token work would then be the reads and the remaining MLPs, and the memory's share of access becomes sparse.

## 4. The experiment (NOT_RUN)

**Base.** The best reads-only configuration of cycle 4, chosen by the reads-only pair's final result before this experiment starts: Dot reads if they are within 0.03 nats of Lorentz reads, otherwise the better score.
- **Chosen: Lorentz reads.** At seed 1, Lorentz scored 2.5531 and Dot 2.6291, 0.076 apart ([cycle 4 §7](geometric-stack-cycle4-2026-09-27.md#7-ablations)).
- **Amended before any arm ran** (2026-09-27, 17:40 UTC). A container restart rolled the lab sandbox back to its 08:24 UTC state and moved it to a different CPU: a Cascade Lake Xeon at 2.8 GHz, where the earlier runs had a Xeon at 2.1 GHz with AVX512-VBMI.
  - The matrix library chooses its blocking from the host's caches, so the same code no longer reproduces the earlier model hashes. The same configuration without a memory now gives a different 3-update hash.
  - The base is therefore **re-run** beside the memory arms, with the same executable on the same host, so all five arms differ only in the memory.
  - The cycle-4 Lorentz run (2.5531) stays as a reference. It ran alone with four threads on the earlier host.
  - This amendment changes no arm, setting or decision rule. The rollback also lost this branch's commits; they were rebuilt by replaying this session's recorded edits, and the rebuilt diff matches the recorded one line for line.
- Its MLP is 764 wide, and its settings are the ablations':
- learning rate 4e-3;
- 1,000 updates (4,096,000 target visits) on the repository code split;
- seed 1;
- the 512-window final evaluation (131,072 targets).

**Arms.** The base, and the base with a memory in place of layer 3's MLP (4 heads, `k = 32`), in each of the four index geometries of §1. The lab sandbox runs them two at a time with two threads each, after the cycle-4 main comparison: the base with learned Dot, learned Lorentz with H4, then E8.

**Decisions, fixed before any arm runs.**
1. **Does sparse memory help?** The learned Dot memory ≥ 0.02 nats better than the base → yes at this scale. Otherwise memories are recorded as not helping at this budget, and the index comparison is still read as relative evidence.
2. **Does the index geometry matter?** Any index ≥ 0.02 nats from the learned Dot index → a second seed of that pair decides. Otherwise the geometries are equivalent at this scale.
3. **Fixed codes against learned keys.** If a fixed codebook is within 0.02 nats of the learned Dot index, fixed geometric addressing matches learned addressing at this budget. That result answers D10's open question 2 at this scope.

**Cost (*Derived*).**
- Five runs, two at a time with two threads each, at about 85 minutes per arm.
- About four and a quarter hours of the sandbox, after the main comparison.
- Storage: each memory model is 103 MB (25.6M fp32 parameters), and its checkpoint is 3 times that.

## 5. What would change the plan

- **A negative result is kept at its scope.** A memory that does not help at 4M tokens may still help at the M1's larger budgets. This cycle does not sweep sizes.
- **The next rung on a positive result.** A memory in every MLP slot, with the dense MLPs removed, would make the stack's per-token access mostly sparse. That, not the score of a dense read, is where D5 needs the geometry.
