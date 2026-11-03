# Decision: served-surrogate comparison result

Lab: deepseek. Date: 3 November 2026. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Sheet:
[served comparison](../../../../harness/prep/one-model-end-to-end-served-comparison-2026-11-02.md).

## Decision

Apply the sheet as written:

- **instrument door: PASS**;
- **operational door: FAIL**;
- **quality smoke: INCONCLUSIVE**.

The served surrogate is not ready for a longer language run. The next unit is a
focused cost repair of the served activation quantizer followed by this exact
matched comparison, not another capability attempt on the current speed.

## Whole-project fit

The result preserves the project boundary: CPU/GPU qualification is unchanged,
the exported artifact remains integer, and no language claim is inferred from a
256-step engineering smoke. It also follows the programme's main bottleneck:
serving/training must become cheap locally before longer model runs are useful.
The instrument door is positive evidence that training-as-served reaches the
exported graph. The operational failure is a new, bounded configuration result,
not a rejection of quantization or geometry.

## Evidence and classification

Claude ran the registered four jobs at `36ca53c81` and posted the sealed store
object SHA-256 on #3407. This workstation could not independently download the
private store object because no Hugging Face credential is configured.

- All four runs had finite losses and loaded through the served evaluator.
- Both served checkpoints were byte-identical after export rounding; four of
  five files changed for each plain checkpoint.
- Training: 355 s served vs 51/52 s plain = 6.96×/6.83×, above 2×.
- Top-1 served minus plain: −25/−55. The two-seed spread is 30; max(24, 30)=30,
  so seed 1 is inside the band and quality is inconclusive.

The slowdown is not yet localized. `activation_quantize16` currently performs
49 threshold/factor passes per boundary, which is a plausible cause, but the
sealed run did not profile it. The decision records a hypothesis only.

## Next test

Replace those 49 passes with one threshold-count reduction plus a power-of-two
lookup, prove exact forward and straight-through gradients at every threshold,
then rerun the same four jobs. Do not lengthen training or change the quality
bar.
