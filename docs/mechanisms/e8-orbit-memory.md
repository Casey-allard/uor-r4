# E8 orbit memory

**Status: PARKED.** Replacing half of the feed-forward layers with this memory stayed inside its quality bar (+0.033 bits per byte), but a learned address book of the same size beat the fixed E8 addresses by 0.023. So the geometry earned no credit. The read weight also uses a sigmoid, which the project now bans.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

Instead of running a full feed-forward layer, the model reads a handful of rows from a very large table. The row addresses come from the 240 special points of the E8 lattice, an eight-dimensional pattern of points (the E8 roots). A fast exact rule finds the nearest special point to each half of the query. The hope is a fixed, exact address book that touches a few rows per token rather than running a full dense layer. In the one trial, the fixed address book did slightly worse than an address book the model learns for itself.

## What it replaces in a transformer

The feed-forward block (also called the MLP). In a transformer this is the large layer after attention that processes each word, and it holds most of the arithmetic. Here a table lookup of a few rows takes its place. The table is far larger, but only a few rows are read per token.

![The 240 E8 roots drawn as a ring of 15 coloured clusters of 16. A query half snaps to its nearest root and the other 15 members of that root's orbit light up as candidates. Beside it, a 240 by 240 grid of value slots where two selections cross and the top 8 cells light up, next to a dense layer drawn as a full block. Below, a four-bar chart of held-out bits per byte with the 0.0054 noise band marked.](../figures/mechanisms/e8-orbit-memory.svg)

*Left, how a query picks candidate rows. Right, the geometry stays inside the quality bar but behind its learned twin.*

## How it works

**The E8 roots (the count is proof; the integer decode was checked by measurement on 20,000 queries).** The 240 roots are 112 vectors of the form $(\pm1,\pm1,0,0,0,0,0,0)$ in any arrangement, plus 128 vectors $(\pm\tfrac12,\dots,\pm\tfrac12)$ with an even number of minus signs. The code stores them in doubled integer coordinates, so they are exact integers.

**Per head, in four steps.**

1. **Cut and decode.** The query is cut into two 8-dimensional halves. Each half is decoded to its nearest E8 root in $O(8)$ operations: the two largest $|x_i|$ with sign rules, or all signs with a parity fix for the half-integer roots. No 240-way scan is needed. It was checked against a full scan on 20,000 queries.
2. **Orbits.** SpiralCore's group of 64 signed axis permutations ($\mathrm{Cl}(0,6)$) splits the 240 roots into 15 orbits of 16 (240 = 15 × 16). A half's candidates are the 16 roots of its decoded root's orbit.
3. **Slots.** The two halves together address a grid of $240\times240=57{,}600$ value slots. The head keeps the top $k$ by combined score ($k=8$ in the rung that ran).
4. **Read.** A kept slot of rank $r$ is weighted by

$$w_r = R[h][r]\cdot\sigma(\beta s+\gamma),$$

   a learned rank table $R$ times a logistic-sigmoid gate on the score $s$. There is no softmax. The output is the weighted sum of the kept value rows. Slot choice is detached from training, so only the query map, rank table, gate scale and the values learn.

**Size.** The value table has $57{,}600\times256$ float values, about 59 MB per layer (proof by arithmetic: $57{,}600\times256\times4$ bytes). A token reads 32 of its rows. The design arithmetic is about 27 thousand multiply-adds per token against 270 thousand for the dense layer it replaces, roughly tenfold. That figure is arithmetic, **not** a measured speed or energy saving.

**Rule check.**

| Rule | Verdict |
|---|---|
| No softmax (D23) | Broken as built. The read weight uses a sigmoid, and sigmoid is banned along with softmax |
| No matrix multiplication (D30) | Not clean. The query map and value mixing are dense float operations. The static E8 and SpiralCore tables were rebuilt without a generic 8 by 8 matrix product ([record](../../archive/labs/native-vector-transport-2026-10-12/README.md)) |
| Integer-only serving (D11) | No served form. Export refuses, and integer serving refuses product-key memory. The value tables are float. The decode, orbit tables and route transport are exact integer lookups |
| Discrete path (D32) and offline float (D37) | Not on the discrete path. [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) would allow offline float training only as a new reviewed sheet with an integer export and a reported gap |

## What has been tried

BPB means bits per byte on held-out text, lower is better.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 26 Sep | Associative recall with a minimal head: E8 block codes against sign-bit codes and float scores, 64 pairs for training, up to 512 pairs at test | E8 recalled 100.0% at 512 pairs. At 8 bits E8 25.8% against sign bits 21.9%. In a language model, E8 trained worse than sign bits | inconclusive | [record](../../archive/integration/geometric-attention-2026-09-26.md) |
| 27 Sep | Implementation of product-key layers with learned, fixed-H4 (14,400 slots) and fixed-E8 (57,600 slots) sub-keys | Selection equals brute-force top-k. Gradients match finite differences. The experiment was not run | inconclusive | [record](../../archive/integration/memory-layers-cycle5-2026-09-27.md) |
| 28 Sep | Addressing contest on a fitted quaternion parent: E8, H4 and k-means codes as an index over events | Not qualified. Recall at 1: k-means 0.9813, H4 0.9696, E8 0.9450. Read loss against full: +0.007201, +0.009313, +0.012083. The comparison is confounded by magnitude (the record says so) | negative | [record](../../archive/integration/addressing-contest-result-2026-09-28.md) |
| 11 Oct | **Rung 1.** E8 replacing half (layers 1, 3, 5, 7: E8h) or all 8 (E8a) feed-forward layers. Width 256, 2,000 steps, 2 seeds, laptop CPU. Dense control D256 and a learned product-key twin P of equal capacity | BPB: D256 1.4078, P 1.4175 (+0.010), **E8h 1.4406 (+0.033)**, E8a 1.5237 (+0.116). Noise 0.0054. Orbit entropy 3.85 to 3.87 of 3.91 bits and 83 to 87% of slots read, so routing did not collapse. CPU speed 1,330 tokens per second (E8h), 405 (E8a), 2,140 (P) | inconclusive: keep and iterate | [record](../../archive/labs/e8-orbit-memory-2026-10-11/README.md) |
| 11 Oct | Integration ledger citing the rung | The tenfold multiply-add saving is design arithmetic. Broad entropy and coverage rule out collapse overall, not every query-level address failure | inconclusive | [record](../../archive/labs/unified-geometric-integration-2026-10-11/README.md) |
| 11 Oct | Whole-path source reuses the E8 tables, compiling 15,360 action images once | Source kept, no training. E8 is used as an exact route beside exact content identity, not as an MLP replacement. The saved 437 of 512 model is unchanged | inconclusive | [record](../../archive/labs/native-bound-content-2026-10-11/README.md) |
| 12 Oct | Removed generic 8 by 8 matrix multiplication from the static tables | 3 static-algebra tests pass. All 240 doubled roots are preserved bijectively under each of the 64 actions and inverses. No model run | positive | [record](../../archive/labs/native-vector-transport-2026-10-12/README.md) |
| 12 Oct | Whole-path source with a 240-way write key and 15 orbit-family profiles | Source kept. 5 E8 regression tests pass on the CPU. GPU run not done. The sampled producer was later blocked by the no-softmax rule | inconclusive | [record](../../archive/labs/whole-path-recovery-2026-10-12/README.md) |
| 12 Oct | Project-manager review of the lab record | Restates the outcome as keep and iterate: learned keys beat fixed E8 addresses by 0.023 BPB | inconclusive | [record](../../archive/labs/kimi-direction-2026-10-12/README.md) |

**Reading these honestly.** The rung's decision rule was "half the layers within the larger of 0.05 or two times the noise", and E8h (+0.033) met it. That is why the outcome is keep-and-iterate, not a failure. But the learned twin beat E8h by 0.0231 BPB, about four times the noise. Learned sub-keys addressed the slots better than fixed E8 roots at this scale. The record could not tell whether the gap comes from the fixed addresses or from the value capacity per slot (hypothesis on either side). So the geometry earned no credit. Replacing all eight layers cost +0.116, which says too much capacity was removed at width 256 for 2,000 steps. It does not say E8 addressing fails. The result is one configuration: 2,000 steps, 2 seeds, a laptop CPU, a sigmoid gate. It was the last rung of its kind on the laptop CPU; geometric-stack training now goes to rented GPUs. The speed numbers are host-side slot selection, not an integer table read.

## What has never been tested

- Any E8 memory run after rung 1: no run with $k=16$, no learned per-orbit offsets, no GPU run of the layer.
- E8 memory trained in from pretraining. The rung trained a 256-wide stack for 2,000 steps from scratch on its own recipe.
- A gate with no softmax and no sigmoid, or any discrete form of the layer.
- Integer value tables and a served form, with measured tokens per second and an export gap.
- E8 memory as the read itself ("address instead of search"). Only the feed-forward layer was replaced.
- E8 as the write address of facts, with prime semiprimes as the exact check.
- Query-level address quality. The entropy and coverage statistics are aggregates.

## A fair test needs

*From the earlier brief, [archive/mechanisms/e8-orbit-memory-2026-10-20.md](../../archive/mechanisms/e8-orbit-memory-2026-10-20.md).*

- A capacity-matched learned-key control (P).
- Routing health (entropy and slot share), to tell collapse from capacity.
- Matched training steps and the same panel.
- GPU training: a CPU run of E8a took about 6 h.

## What would reopen it

1. **Run the next rung the record names**, GPU only and pre-registered with a fresh `TEST FITNESS: FIT` review: raise $k$ from 8 to 16, and add learned per-orbit residual offsets on the E8 frame. The offsets separate "fixed addresses" from "value capacity" as the cause of the 0.023 gap.
2. **Replace the sigmoid gate** with one that obeys the no-softmax rule, such as a rank table with a hard threshold or exact selection. Register it as an offline float sheet (D37) in a separately scanned module with an integer export (integer value tables, E8 decode as table lookups) and a reported export gap. An owner release is required.
3. **Keep the fair-test conditions.** A capacity-matched control (P, the learned-key twin, which uses a float sigmoid gate), taken from stored rung-1 outputs or replaced by a lawful count-table control because new neural controls are barred until the model works end to end (charter section 3.10); routing health (orbit entropy and slot share), matched training steps, the same panel, and at least 2 seeds.
4. **Provide a served form.** Integer serving refuses product-key memory today, and a multiply-add saving did not appear in tokens per second for the learned-key twin. Report tokens per second.

## Also known as

Exact-E8-addressed memory; E8 memory in place of the MLP; `e8_memory` (arms E8h and E8a); fixed E8 product-key memory.

## Code

- [`e8_memory.rs`](../../crates/uor-r4-training/src/e8_memory.rs): `e8_tables`, `E8Tables::nearest`, `e8_select`, `e8_memory_read`, usage logging. The training flag is `e8_memory=LAYERS:HEADS:K`.
- [`stack_memory.rs`](../../crates/uor-r4-training/src/stack_memory.rs): the product-key layer with fixed H4 or E8 sub-keys, and the capacity-matched learned-key control P.
- [`spiralcore_operator.rs`](../../crates/uor-r4-core/src/spiralcore_operator.rs): signed-axis composition and `apply_root_key`, which build the orbits and the 64 action tables without a generic matrix product.
- [`whole_path_learning.rs`](../../crates/uor-r4-training/src/whole_path_learning.rs): the whole-path source that reuses the E8 tables as a bounded record index (CPU fixtures only).

## Related mechanisms

[exact-memory](./exact-memory.md): the parent family. This is its product-key memory with fixed E8 sub-keys, replacing a feed-forward layer. [spiralcore-fbs](./spiralcore-fbs.md): supplies the 64-element group whose actions define the 15 orbits. [spin-weights](./spin-weights.md): the 120-icosian H4 variant of the same layer. [admission](./admission.md): the addressing contest used E8 codes as an event index. [carrier-credit](./carrier-credit.md) and [discrete-learning](./discrete-learning.md): the whole-path and discrete lines that reuse the exact tables. [ternary-xor](./ternary-xor.md): E8 is a lifted Hamming code, the bridge to XOR and bit-count arithmetic.
