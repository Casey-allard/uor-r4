# The metric the product needs: measured tokens/s, GPU on a pod — and the addressed read loses 1.65× there

References #2029 (M1 acceptance; metric direction: owner, 11 Oct — *"since we are reimagining what it means to be a token in
our product, I'm not sure BPB against standard transformers is the correct metric … it's much more likely tokens/s is the best
way to compare"*, and *"use GPU on a pod — do not use CPU"*). Lab: DeepSeek, session `deepseek/leanaddr`.
Measurement 2026-10-11 15:10–15:12 UTC on a 2 × RTX 5090 pod; three repeats per arm.

> **One line.** Measured the way the product is used — **generated tokens per second, on the GPU, on a pod, three repeats,
> same harness, same panel, same tokenizer (`d36d3e87…`)** — the addressed read is **1.65× slower than the read it replaces**:
> **Ctl1/Ctl2 637.7 / 636.9 ids/s against L1/L2/L3 388.1 / 385.5 / 384.3 ids/s** (repeat spread under 0.5 %). Its analytic
> MAC count at the replaced layer is **−90 %**, and that saving **does not appear in tokens/s** at all: a query map, sub-key
> scoring and a **gather** do not use the matrix units a GPU is built from, and this configuration is not fused. Under the
> metric the owner has named, **the address loses on throughput as implemented** — on top of the 2.4× slower training, 7 fewer
> memory rows of 120 and +0.10 diagnostic BPB that the last cycle measured. The compute argument for addressing is real in
> **arithmetic** and unproven in **time**; the honest next instrument is the native, multiplier-free serving path where a table
> read is the intended primitive.

## The measurement

| arm | configuration | ids/s (repeats 1 / 2 / 3) | generated ids |
|---|---|---:|---:|
| **Ctl1** | read (control), seed 20261009 | 636.6 / 637.7 / 638.4 | 1,806 |
| **Ctl2** | read (control), seed 20261010 | 637.4 / 635.8 / 636.9 | 1,756 |
| **L1** | addressed read, seed 20261009 | 389.7 / 388.1 / 386.8 | 1,976 |
| **L2** | addressed read, seed 20261010 | 383.8 / 385.5 / 385.3 | 1,845 |
| **L3** | addressed read, seed 20261011 | 384.9 / 383.8 / 384.3 | 1,772 |

**Ratio: the read is 1.65× the address's throughput**, on every arm and every repeat (spread ≤ 0.5 % within an arm), on a
fourth condition that differs only in whether the last read layer is a softmax read or a product-key memory.

**Why this is the measurement that counts, and what it contradicts.** The two cycles before this one reported the address's
arithmetic: **−90 % MACs at the replaced layer** (1,769,472 → 173,056 at context 384) and **−97 % at 4,096**. Both numbers are
correct as arithmetic and **both are invisible in tokens/s on this hardware**: the read is matmuls (which a GPU is for), the
address is scoring plus a gather (which it is not), and the configuration has no fused kernel. A ledger that reports MACs
without a measured rate invites exactly the mistake this cycle corrects — and the owner's metric is what caught it.

**Training follows the same direction**: 245 s for 2,000 steps against the control's 104 s (**2.4×**), same recipe, same store,
same batch — better than the fat codebook's 20× but still a loss.

## How the metric direction changes this lab's reporting, from here

| | before | **now** |
|---|---|---|
| headline comparison | BPB against the line's held-out stream | **generated tokens/s, GPU, on a pod, ≥ 3 repeats**, always beside the project's own panel acceptance — *a throughput number never travels without its quality number* (the rule the geometry audit already set as B1) |
| quality | BPB as a gate | **panel acceptance is the gate** (the project's own criterion); BPB is reported as a **diagnostic**, not a bar |
| cost ledger | MACs/token, parameters, wall | the same, **plus the measured rate**, so an analytic saving that does not appear in time is visible as such |

**What does not change, and I want this on the record rather than smoothed over:** the addressed read still lost **7 memory
rows of 120** and still costs **+0.10 BPB** as a diagnostic, and it now also loses **1.65× on tokens/s** and 2.4× on training
time. The *only* column it wins is the analytic MAC count at the replaced layer, which is precisely the column this
measurement shows is not a proxy for throughput on GPU. **The mechanism is not closed** (D22 §1) — its compute case belongs to
the **native multiplier-free serving path** where a table read replaces multiplies and there is no matrix unit to waste, and
that path is the D11 target rather than the f32 GPU harness measured here. That is the next instrument, and it is named rather
than assumed.

## Limitations

- **The f32 GPU harness is not the product's serving path.** These rates measure the training/inference harness on a 5090; the
  native multiplier-free contract (D11) is where an address's arithmetic advantage should convert into time, and it is untested
  for this configuration.
- **Context is ≤ 384 tokens** for the panel's replies, where the read is cheapest; the MAC ledger's advantage grows with context
  (−97 % at 4,096), so a long-context rate measurement is the natural companion and was not run.
- **Three repeats, one pod, one GPU model**; the spread within an arm is under 0.5 %, so the comparison is stable, but it is one
  machine and one kernel situation.
- **CPU-side rates were discarded as unusable**: two repeats of the same arm on the shared laptop read 6.9 and 17.0 ids/s, a
  2.5× spread that swamps the effect. That is why this measurement is GPU-on-pod, per the owner's direction, and the earlier
  CPU figures are not reported as evidence.

## Evidence

- Reply reports with `panel.cost.ids_per_second` for every repeat: `/workspace/uor-r4/deepseek/leanaddr-20261011/rate-gpu/`
  on the pod volume, pulled and bundled to cloud-store as `[BUNDLE]`; the requests file and the tokenizer digest
  (`d36d3e8700a123e6…`) are in the same directory.
- Configurations and per-token ledger: [lean addressed read](../labs/addressed-read-lean-2026-10-11/README.md) and
  [addressed read](../labs/addressed-read-2026-10-11/README.md).

## CORRECTION (added 2026-10-11, same day): the arms compared differ in the MLP at layer 8, not in the read

**The rates are right; the labels are wrong.** The four arms differ in whether **layer 8's MLP** is a SwiGLU MLP or a
product-key memory (`memory_layers=8` replaces the MLP — see `StackModel::mlp` and `MemoryConfig::layers`). The read at
every layer, including layer 8, is the same softmax read in all four arms.

So in the table above, *"the softmax read (controls)"* should read **"SwiGLU MLP at layer 8 (controls)"**, and *"the
addressed read"* should read **"product-key memory in place of that MLP"**. The finding is unchanged and now reads
correctly: **replacing an MLP with a lookup costs 1.65× the throughput on this GPU**, because a lookup does not use the
matrix units the MLP does. That is a statement about lookup-versus-matmul on this hardware, **not** about addressing the
read, which has not been measured.
