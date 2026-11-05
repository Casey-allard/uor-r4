# ORDER: float-kernel-fusion, claude-283

Lab: claude. Item: [float-kernel-fusion](../../../../roadmap/items/float-kernel-fusion.md) (#3678). Base: `1f6570d59`.

## Goal

The item's *How to start* says to measure before fusing, and its *Cancel if* gives the rule:

> The two chains' share of wall seconds per training step at P2b's recipe, with `--train-as-served 1`, is under 5%, with their launch counts reported alongside. Then done-when 1 lands alone, and the fusion is reported as not worth its pin.

This unit takes that measurement and makes the cancel decision. It takes no done-when of its own: done-when 1 (the generated kernel list) merged in claude-282 (#3684, `1f6570d59`). Done-whens 2 and 3 (the fused kernels) start only if this unit finds that the cancel rule does not fire.

**The claim check.**
- **(a) The step:** THE ORDER's step 5(c), the measure-first fusion (#2028 6634218223). Casey approved the fused kernels on 4 Nov ("Yes, please build this too to do it the smarter way going forward", #2028 6638209391).
- **(b) What it changes:** nothing in the model that trains or serves. The switch it adds is off unless a timing run arms it, and an armed run is not a result. It unblocks done-whens 2 and 3, or cancels them. They would cut the time of every trained-as-served step: P5's scale probe, P6 and every later training run.

## Cancel probe (run first)

**Question:** does any existing measurement already give the two chains' share of a trained-as-served step?

**What was checked:**
- **[claude-157](../../fbs-cost-profile/claude-157/REPORT.md)** profiled a step before training as served existed (claude-258), so its stages hold no served chain.
- **[deepseek-5](../../one-model-end-to-end/deepseek-5/REPORT.md)** measured training as served's 6.83–6.96× slowdown and probed the host. It is not a GPU split by chain.
- **`train-in-space --profile K`** (`in_space_long.rs:10519-10533`, `:11054-11141`) times stages, but cannot split the chains either:
  - in the forward, they run inside other stages (the trunk's sublayers and the FBS projections) through the function pointers at `geometric_stack_native.rs:1406, 1656, 1785, 1788, 2137, 2140`;
  - the backward is one stage, `backward` (`:11095`), so their one-line backwards are not visible apart.

**Result:** no measurement exists, and the existing clock cannot take one. **Class: instrument.** This unit builds the instrument and takes the measurement.

## The instrument

**The switch.** A timing-only switch in `hamilton_sheet.rs`. While a run is armed with its profile window of K pairs, the two chains, `q16_residual` and `activation_quantize16`, return their input unchanged on the training forwards of the window's odd pairs, and run as today everywhere else.
- **Pair j** is training steps `8 + 2j` (profiled) and `9 + 2j` (unprofiled), as `profile_window` defines them.
- **Where it decides:** `with_served_surrogate_training` gets each training forward's step plus one (`in_space_long.rs:302`), so the switch decides there.
- **What it counts and records:** each chain's calls and elements on every forward inside the window, kept and skipped apart, and the training steps it skipped.

**The option.** `train-in-space --time-without-quantizers 1` arms the switch around `run_long` and writes the record to `DIR/quantizer-timing.json`. It is refused unless all of these hold:
- the mode is training (`--out`, no `--recall`);
- `--profile K` is above 0;
- `--train-as-served` is 1;
- `--steps` and `--eval-every` are each above `8 + 2K`, so no evaluation or checkpoint forward falls inside the window;
- the record shows every skipped and kept step exactly once, in the right pairs (the run ends with an error otherwise).

An armed run's losses and checkpoints are not results; the record says so.

**The measurement.** For each arm:
- the chains' share = (median unprofiled step of the kept pairs − that of the skipped pairs) / the kept median;
- the GPU stage medians of each class, which show where the chains' time sits, backward included;
- the calls a forward makes to each chain.

## Files in scope

- `crates/uor-r4-float-train/src/hamilton_sheet.rs`: the switch, its record and its CPU tests.
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: the option, its refusals and the record file.
- `docs/harness/prep/float-kernel-fusion-measurement-2026-11-05.md`: the prep sheet.
- `docs/loop/tasks/float-kernel-fusion/**`: ORDER, REPORT.
- `docs/research/notebook/float-training.md`: one row.
- `docs/roadmap/items/float-kernel-fusion.md`: the item's status and text after the decision.

(All within the item's scope list.)

## Done when

1. **The switch:** in `hamilton_sheet.rs`, with CPU tests.
   - The pair rule: odd pairs skip, even pairs keep, nothing outside the window or with the switch off.
   - On a skipped forward, both chains return their input bit for bit.
   - On a kept forward, and with the switch off, they return today's values bit for bit.
   - The counters, and the record's own check.
2. **The option:** `--time-without-quantizers 1` in `train-in-space.rs`, with CPU tests of each refusal.
3. **The prep sheet** `docs/harness/prep/float-kernel-fusion-measurement-2026-11-05.md`, FIT by a fresh reviewer, and a pre-run review. Both are posted on #3678.
4. **One RTX 5090 run** at P2b's recipe (claude-266's (b), `--train-as-served 1`), each arm with `--profile 64`, sealed and in the store:
   - (L) `--context 2048 --batch 4`;
   - (S) `--context 192 --batch 16`;
   - per arm, one unarmed run U and two armed runs T1, T2.
   - **The instrument checks:**
     - the CUDA library suite;
     - the record's check;
     - I-1: T1's step-8 loss equals U's bit for bit (the switch acts on no step before pair 1);
     - I-2: T1's step-10 loss differs from U's (it acts on pair 1).
5. **The REPORT, negatives first, with the decision:**
   - **Cancel** if the share is under 5% at both arms, with the launch counts alongside. Done-when 1 then stands alone: the item is set done, with the fusion reported as not worth its pin.
   - **Go:** the share is at or above 5% at either arm. Done-whens 2 and 3 are the next units.
   - **Inconclusive:** the share is within twice the noise of 5%. The noise is the A/A spread: kept pairs split into two halves, and T1 against T2. Re-run with a larger K.
   - One notebook row.

## Stop and ask if (stop and decide; never wait)

- **The switch acts anywhere but the two chains on the window's odd-pair training forwards:** I-1 fails, or the record shows a step twice or out of its pair. Then it is an instrument fault: repair it before any number counts.
- **The stage clock's overhead** (`clock_overhead` in `profile.json`) differs between the classes by more than the share being measured. Then use the unprofiled steps only, and say so.
- **The pod or a cap.** At most one 5090 pod, a $3 bound for the unit, a watcher on every job, and results off the pod before `down`.
