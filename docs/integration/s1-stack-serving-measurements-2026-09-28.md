# S1: the stack's representation gap at the D11 interim format — measurements

September 28, 2026. References #973 under #820.
- **Lab:** Lab 1 (Claude), locally (owner, 16:15 UTC).
- **Work card:** [S1 on #973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5874321776), posted before any run.
- **Evidence:** [`s1-stack-serving-measurements-2026-09-28.json`](../evidence/s1-stack-serving-measurements-2026-09-28.json), assembled by script from the sealed roots.

**Status.** Every step here is evaluation only; there was no fit. S1.1 (the D11 integer port) is being built separately.
- **S1.0.** The export gap misses S1.2's fidelity gate (≤ 0.02 nats against float), with either quantizer:
  - round to nearest: +0.0362;
  - GPTQ: +0.0257.
- **Integer arithmetic costs nothing measurable.** The whole gap is weight representation.
- **S1.0b.** Snapping every transport quaternion to the nearest unit icosian costs +0.026 nats on each of three models. That is above the 0.02 option threshold, so exact group-table serving of the transport is **not** adopted as an option. The result is reported, not gated.
- **S1.0c.** One tensor group at a time from the export shows where the gap comes from. The largest single share is **the output head**.

## What was measured

**Model.** The cycle-4 `geometric_s1`:
- `rrarra`, Lorentz reads, rotation;
- 7,153,860 parameters, SHA-256 `3eb1ebbb…`;
- trained on the cycle-3 code split and the registry stream.

It is a reference model, not the milestone checkpoint.

**Data.**
- **Development set:** the cycle-3 code split (`valid.u16`, `3f7c50ef…`), 512 evenly spaced windows of 256 tokens (131,072 targets), with the windows `evaluate` uses.
- **GPTQ calibration:** 64 windows (16,384 positions) of the model's own training split (`train.u16`, `b12707b0…`), never the development set.

**Export.** The existing D10 container (`UORLUT01`), which is also the D11 interim weight format:
- 4-bit maps in groups of 32, each group's scale one byte with a 4-bit mantissa;
- a 4-bit-mantissa grid code for per-channel scalars (conv taps, decay rates, Lorentz β);
- 2⁻¹⁶ fixed point for biases.

**Engine.** The frozen D10 engine (`uor-r4-lut`) scores the artifact. Its integer arithmetic is exact, and S1.1's D11 port must reproduce its logits bit for bit. So these numbers are the D11 port's numbers too, once S1.1 passes its gate.

**Arms.**
- **Float:** the model's own float forward.
- **Float reference:** the artifact's own values in f32, through the float kernels.
- **Integer:** the D10 engine.

Integer − float therefore splits into weight rounding (reference − float) and integer arithmetic (integer − reference).

## Results

The tables below are generated from the evidence packet.

### S1.0: the export gap (512 windows, 131,072 targets)

| Export | Float NLL | Float reference NLL | D10 integer NLL | Integer − float | Weight rounding | Integer arithmetic | Top-1 agreement | Decision-flip rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Round to nearest | 1.998113 | 2.034342 | 2.034342 | **+0.0362** | +0.0362 | -4.0e-07 | 0.9065 | 0.0935 |
| GPTQ, 64 calibration windows | 1.998113 | 2.023859 | 2.023860 | **+0.0257** | +0.0257 | +6.8e-07 | 0.9183 | 0.0817 |

- Round to nearest: artifact `9ad3313303df…` (4,969,732 bytes); D10 engine 955 tokens/s (neon, 2 threads, step time only).
- GPTQ: artifact `43e97b054a37…` (4,970,052 bytes); D10 engine 932 tokens/s (neon, 2 threads, step time only).

### S1.0b: every transport quaternion snapped to the nearest of the 120 unit icosians

| Model | Unsnapped NLL | Snapped NLL | Snapped − unsnapped | Top-1 agreement | Composed − fused max abs logit |
|---|---:|---:|---:|---:|---:|
| cycle-4 `geometric_s1` (7,324 updates) | 1.998113 | 2.024263 | **+0.0262** | 0.9170 | 3.1e-04 |
| D1 `rot_s1` (1,000 updates) | 2.599845 | 2.625444 | **+0.0256** | 0.8936 | 1.1e-04 |
| D1 `rot_s2` (1,000 updates) | 2.580265 | 2.606230 | **+0.0260** | 0.8966 | 1.4e-04 |

### S1.0c: one tensor group at a time from the artifact, everything else float

| Group | GPTQ export: NLL − float | Top-1 agreement | Round-to-nearest export: NLL − float | Top-1 agreement |
|---|---:|---:|---:|---:|
| embed | +0.0026 | 0.9742 | +0.0026 | 0.9742 |
| **head** | +0.0119 | 0.9467 | +0.0149 | 0.9395 |
| l0.mixer_maps | +0.0004 | 0.9910 | +0.0007 | 0.9881 |
| l0.mixer_scalars | +0.0001 | 0.9988 | +0.0001 | 0.9988 |
| l0.mlp | +0.0027 | 0.9772 | +0.0043 | 0.9718 |
| l1.mixer_maps | +0.0004 | 0.9887 | +0.0006 | 0.9855 |
| l1.mixer_scalars | +0.0000 | 0.9978 | +0.0000 | 0.9978 |
| l1.mlp | +0.0003 | 0.9944 | +0.0009 | 0.9923 |
| l2.mixer_maps | +0.0001 | 0.9916 | +0.0003 | 0.9842 |
| l2.mixer_scalars | +0.0002 | 0.9955 | +0.0002 | 0.9955 |
| l2.mlp | +0.0004 | 0.9871 | +0.0009 | 0.9839 |
| l3.mixer_maps | +0.0004 | 0.9899 | +0.0004 | 0.9876 |
| l3.mixer_scalars | +0.0001 | 0.9985 | +0.0001 | 0.9985 |
| l3.mlp | +0.0004 | 0.9857 | +0.0009 | 0.9833 |
| l4.mixer_maps | +0.0003 | 0.9874 | +0.0007 | 0.9853 |
| l4.mixer_scalars | +0.0000 | 0.9984 | +0.0000 | 0.9984 |
| l4.mlp | +0.0003 | 0.9861 | +0.0005 | 0.9834 |
| l5.mixer_maps | +0.0018 | 0.9817 | +0.0032 | 0.9744 |
| l5.mixer_scalars | +0.0008 | 0.9949 | +0.0008 | 0.9949 |
| l5.mlp | +0.0021 | 0.9745 | +0.0037 | 0.9695 |
| **all_maps** | +0.0243 | 0.9193 | +0.0348 | 0.9071 |
| **all_scalars** | +0.0013 | 0.9919 | +0.0013 | 0.9919 |
| **all** | +0.0257 | 0.9184 | +0.0362 | 0.9065 |

- gptq: sum of single groups +0.0254 against all groups together +0.0257; head +0.0119, embed +0.0026, MLPs +0.0061, mixer maps +0.0034, mixer scalars +0.0013.

- rtn: sum of single groups +0.0360 against all groups together +0.0362; head +0.0149, embed +0.0026, MLPs +0.0113, mixer maps +0.0059, mixer scalars +0.0013.

**Consistency check.** With every group from the artifact, the attribution's `all` row equals `lut-evaluate`'s float reference, which comes from a separate code path: +0.0257 and +0.0362. The single groups add up to within 0.0003 of it, so the split is close to additive.

## Reading

This interprets the results; no gate changes.

1. **Serving arithmetic is solved; representation is not.** Integer arithmetic costs ≤ 10⁻⁶ nats. All of the +0.026–0.036 comes from the 4-bit export.
2. **GPTQ closes about 30% of the gap** (0.0362 → 0.0257). That is not enough for 0.02.
3. **The head is the largest single share:** +0.0119 of the GPTQ gap (+0.0149 with round to nearest).
   - Next come the last read layer (`l5`) and the first MLP (`l0`), then the embedding.
   - The per-channel scalars (grid codes and fixed point) cost only +0.0013 in total, so a finer scalar grid would not close the gap.
   - Removing the head's error alone would bring the GPTQ export to about +0.014, if the shares stay additive.
   - The artifact's head matrix carries the final norm's gain folded into its columns.
   - The float model ties it to the embedding; the artifact quantizes it separately.
4. **The 2I snap costs about the same on every model** (+0.0256 to +0.0262), whether the model had 1,000 or 7,324 updates.
   - The earlier parked figure, +0.16–0.18 bits/byte from a different model, does not describe this stack.
   - Snapping at evaluation, with no training, is above the 0.02 option threshold. Whether snap-aware training recovers it was not measured.

## What it changes

- **S1.1–S1.3 continue now:** the bit-identical D11 port, the I2 `stack` bundle with reload equality, and the M1 audit. Engineering proceeds and promotion waits (ruling 12).
- **S1.2's fidelity gate cannot pass with the current weight format.** Closing it needs one of:
  - training with the served representation in the loop;
  - a better 4-bit codec (D4);
  - a contract change for the head.

- **Owner decision, 17:00 UTC: QAT (Lab 1) plus a D4 target.**
  - **Lab 1** adds quantization-aware training: the export's own 4-bit maps and grid scalars in the forward pass, with a straight-through gradient and a pluggable codec. It is validated by a pre-registered short fine-tune of this model, with saved weights.
  - **D4 (Lab 3)** gets this model's +0.0257 nats at 4.25 bits per weight (4-bit codes plus one scale byte per 32) as its codec target, attributed by group. Its codec can plug into the same training hook.
  - **The milestone fit** uses whichever reaches ≤ 0.02 first.
- **Not pursued:** an 8-bit head (it would need a contract change to R3) and a finer scalar grid (the scalars cost +0.0013).

## Scope and limits

- **One model and one development set:** the cycle-4 reference on the code split. The milestone checkpoint does not exist yet.
- **Evaluation only.** No weights were trained or changed. The snap is a test-time substitution.
- **Top-1 agreement is the greedy decision-flip complement** on the 131,072 development positions, not a generation test.
- **Source identity.**
  - The snap and attribution roots record their executables (`4125f51b…` and `52a20eeb…`).
  - The S1.0 roots record their command lines, not their executable. The GPTQ pair ran while the snap runs were recording `4125f51b…` at the same path; the round-to-nearest pair's build is not recorded.
  - The integer NLL is a deterministic function of the artifact, whose hash is recorded.
  - After the snap runs, one doc comment moved in the source, with no code change.

## Cost

Measured on the M1, all at 2 threads (light jobs, no model slot):
- **S1.0:** each 512-window evaluation took about 3 minutes (3:03 and 3:09 wall); the exports took 1 s (round to nearest) and 11 s (GPTQ, including 5 s of calibration), by their roots' claim and seal times.
- **S1.0b:** 45–49 s per model.
- **S1.0c:** about 20 s per group for 24 groups per export.
- **Storage:** 17 MB of new SSD files under `/Volumes/UOR-Workspace/uor-r4-lab/claude-s1/`:
  - about 10 MB of sealed roots, including two 4.97 MB artifacts;
  - a 6.8 MB copy of the attribution build;
  - logs.
- **Peak RSS:** about 0.87 GB in the attribution runs, 434 s and 448 s wall.
