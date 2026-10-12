# Starling read with a training-only softmax surrogate: content selection is learned, recall lifts off chance unevenly (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** starling read · count 3/3 (pivot; owner decision) · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6115450007](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115450007).
**Test fitness:** round 1 NOT FIT (a value-gradient leak, fixed before compute); round 2 [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115570151).

## Question

The pivot diagnostics ([record](../starling-pivot-2026-10-11/README.md)) found two things:
- the geometric selection and the value path work;
- q·k never learns to pick the source among the kept candidates (ranked first 0.03–0.29 of the time).

Does a training-only softmax over the kept candidates, the flock rank reads' recipe, teach it?

## Change

`set_starling_surrogate` (training only, never saved) and `mqar-bench starling_surrogate=true`:
- **Read.** The read adds `read_soft − read_soft.detach()`, where `read_soft` mixes the **detached** kept values by a softmax over the kept slots' content scores q·k/√d.
- **Forward.** The forward value is unchanged bit for bit, so the runtime read stays softmax-free (acceptance item 0).
- **Backward.** q and k get the softmax's competitive gradient; the values keep the softmax-free read's gradient.
- **Tests:**
  - logits are bit-identical with and without the surrogate;
  - the query-map gradient differs;
  - the value-map gradient is bit-identical;
  - an address route refuses the surrogate.
- **Negative gate:** removing the values' `.detach()` fails the value-gradient check (run on the pod).
- **Round-1 review** caught the original version leaking a softmax-weighted gradient into the values. It was fixed before any run.

## Runs

- **Bench:** rung 1–3's (context 512, 1,800 steps, `pattern=rarr key_shift=true`), with `starling_anneal=0.5`, 3 seeds per arm.
- **Hardware:** one RTX 4090; 6 runs in about 6 min.
- **Checks:** unit tests and the negative gate ran on the pod's CPU, not the laptop.

## Result

| Arm | Recall per seed | Mean | Hit rate | **Source ranked first when kept** |
|---|---|---:|---|---|
| L, L2 softmax read (rung 1) | 0.669, 0.373, 0.809 | 0.617 | — | — |
| SR, shared + ranked, no surrogate | 0.013, 0.010, 0.049 | 0.024 | 0.53–0.81 | 0.03–0.12 |
| **SGS, shared + ranked + surrogate** | **0.044, 0.288, 0.091** | **0.141** | 0.39–0.45 | **0.55–0.78** |
| SG7, ranked + surrogate (unshared) | 0.010, 0.011, 0.005 | 0.009 | 0.39–0.55 | 0.37–0.49 |

**Decision: reading 3, by the pre-registered bars.**
- SGS's seed SD is 0.13, which sets the reading-2 bar at base + 2×noise = 0.28. The mean, 0.141, is below it.
- Row 1 (≥ 0.36) is not met.
- The pre-registration sends the line to the owner, with a recommendation to pause.

**What the data adds beyond the bars, reported, not used to re-score:**
- **The surrogate teaches content selection.** When the source is kept, it is ranked first 0.55–0.78 of the time, against 0.03–0.12 without the surrogate.
- **Recall lifts off chance in the shared arm,** to 0.288 on one seed, but very unevenly across seeds.
- **The bottleneck has moved.** It was "q·k does not learn to rank the source". It is now **per-seed instability of recall**, probably the address learning: SGS's hit rate (0.39–0.45) is below SR's (0.53–0.81).
- **Unshared (SG7) stays at chance** despite a source-first rate of 0.37–0.49. With the source kept in one head of four, the other heads dilute it, so the shared kept set matters.

## Next

The owner decides the line (D22: stopping needs the owner), on #2032.
- **The pre-registered recommendation is to pause.**
- **The new evidence supports one more targeted step:** stabilising address learning under the surrogate, for example a longer helper phase, or keeping the helper's address gradient on after the anneal.

## Files

Logs are in cloud-store `claude/starling-surrogate-20261012`.
