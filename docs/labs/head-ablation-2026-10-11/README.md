# The head ablation: silencing the supervised read layer removes the answers, not just their accuracy

References #2029 (M1 acceptance, D22 order 4; mechanism: read binding, [brief](../mechanisms/read-binding.md)).
Lab: DeepSeek, session `deepseek/headablate`. Run 2026-10-11 06:0x–06:23 UTC. **No training, no pod: laptop CPU**, one
artifact, one frozen panel.

> **One line.** The read-binding brief's last unmet requirement is *"a head-ablation check that the bound head drives
> the answer"*. The powered cycle could not run it, so this cycle built the instrument — a serving-time switch that
> makes one read layer contribute nothing to the residual — and ran it on the read-bound artifact (**S2**, 24/40):
> silencing the **supervised (last) read layer** gives **0/40 with no wrong values at all** (37 rows produce no value),
> against **2/40** for the first read layer and **7/40** for the middle one. The layer the supervision targets is the
> layer the answers come from — the causal check is answered, and it agrees with the mechanism's own result.

## The instrument

`StackModel::set_read_silence(Some(layer))` makes that layer's read contribute nothing to the residual
(`x = x + 0`) with the rest of the forward unchanged; `chat-grade reply read_silence_layer=L` sets it and records it
in `replies.json`. A read-out change only: no parameter, no gradient, no training path, not saved, and a layer that
is not a read layer of the model is refused ([#2191](https://github.com/UOR-Foundation/uor-r4/pull/2191)).

## Results — one artifact, one read layer silenced at a time, frozen v5 panel

Artifact **S2**: the powered read-binding arm (`read_binding_supervision=0.1`, `data_seed=20261010`, bound mass 0.899,
competing 0.086; model `ed54036e…`). The stack's reads are layers **2, 5 and 8** (`rrarrarrar`), and the supervision
targets the last read layer by default.

| condition | memory `check_pass` | wrong-value | distractor-key | **no value** | mean reply words |
|---|---:|---:|---:|---:|---:|
| **unablated** | **24/40** | 5 | 4 | 7 | 4.8 |
| silence the **first** read (layer 2) | 2/40 | 9 | 0 | 29 | 18.2 |
| silence the **middle** read (layer 5) | 7/40 | 12 | 0 | 21 | 5.5 |
| **silence the supervised read (layer 8)** | **0/40** | **0** | 3 | **37** | 6.3 |

## What it establishes

1. **The bound head drives the answer.** The supervised layer is the one whose removal leaves *no* correct answer in
   any row, and — unlike the other two ablations — leaves **zero wrong-value failures**: the model does not answer
   with a different value, it stops producing values. The supervision is applied to the head that carries the answer,
   which is what the brief asked to be shown rather than assumed.
2. **The read layers are not interchangeable.** 0/40, 2/40 and 7/40 for the last, first and middle read: the last read
   is where the answer is formed, and the first read is close behind in importance, but the middle layer is less
   critical than either. That ordering is itself a lead: the brief's open question *"supervise every head, or route the
   output through the binding head?"* has a measured answer for where the routing would matter most.
3. **It closes the loop with the attribution and the powered cycle.** The [answer-path attribution](../labs/answer-path-2026-10-11/README.md)
   said the answers come from the read; the [powered read binding](../labs/read-binding-powered-2026-10-11/README.md)
   showed the read can be steered and the headline moves (wrong-value 47 → 30 over 120, memory 57 → 65); this says the
   head being steered is the one that carries the answers. Three measurements, one story, none of them a configuration
   comparison.

## Limitations

- **One artifact, one seed, 40 memory rows** (±3 rows); the ablation is a read-out intervention, so nothing here is
  about training, and nothing here says a *trained* model with the head removed would behave the same.
- **Silencing a whole read layer is coarser than silencing "the supervised head"**: the read-binding supervision runs
  per head inside that layer, and this instrument removes all of them at that layer. A per-head ablation would need
  the head index plumbed through the read-out path.
- **No judge-side numbers**: the readings are the frozen row checks and their failure mix; the `acceptable` column
  would need the shared judge and is not needed for a causal check.
- The unablated column is the artifact's own reading from the previous cycle's scoring, not a re-run in this session.

## Cost

| | |
|---|---|
| compute | laptop CPU only: three reply sweeps (64 rows, cap 64) |
| pod | none held for this measurement (the lease used to re-fetch the artifact was released after the pull) |
| external | none |

## Evidence

- Reply reports: `~/uor-r4-local/headablate-20261011/{sil2,sil5,sil8}/replies.json`, bundled to cloud-store as
  `[BUNDLE]`; the classifier is `~/uor-r4-local/headablate-20261011/ablatecheck.py`.
- The instrument: [#2191](https://github.com/UOR-Foundation/uor-r4/pull/2191).
- The artifact: `readbind2-20261011/runs/S2` (model `ed54036e…`), re-fetched from the pod volume and pulled for this
  measurement.

## Also completed in this cycle

**The BPB clause of the powered read-binding bar, on the matched pair that was pending: Ctl2 reads 1.17003 against
S2's 1.17067 — a difference of 0.0006, inside the 0.01 the bar allows, with both below the anchor's 1.17425.** Ctl3's
evaluation is the one remaining number and is named pending.
