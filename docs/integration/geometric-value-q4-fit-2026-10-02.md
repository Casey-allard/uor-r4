# Answer-only learning of strict q4 geometric values

This fixed continuation asks whether ordinary answer learning can recover losses introduced by the strict four-bit K2 value projection. The loaded [construction](geometric-value-q4-2026-10-02.md) scored451/512 versus467/512 for the unrestricted producer parents. Its training/serving hard-choice mismatch was repaired before this fit: all512 packet/Q16 traces agree, with native outputs unchanged. Finite connected gradients are an implementation result, not a convergence guarantee.

The [work card](https://github.com/UOR-Foundation/uor-r4/issues/1512#issuecomment-5954982171) fixes640 updates perseed, B8, ordinary query-answer CE with denominator8, AdamW learning rate0.003, betas0.9/0.999, epsilon1e-8, zero decay and global selected-gradient norm clip1. Only the128,896 value-source shadows update; they are projected to[-1.75,1.75] after each update. The q/4 coefficient grid, roots, radius bins, K2 atoms and zero-anchor surrogate remain unchanged. The learned composition bank and all surrounding operators remain frozen. There is no auxiliary packet loss or coefficient-scale sweep.

The driver loads each sealed projection2 source with its exact shadow bits. Training starts from the composition-final RNG at absolute2560 and uses the unchanged alternating two-/four-fact sampler. Construction/check draws do not consume the training stream. Checkpoints0/every80/final-or-partial preserve source weights and RNG; they do not serialize Adam moments and are not exact optimizer resumptions. Failed attempts retain their batch/current-source status separately from the last completed checkpoint.

The retained128original/128stress panels perseed are exposed development evidence. Prefit replay verifies the admitted q4 projection and unrestricted parent. Final source save/reload, native compile/reload and unchanged NoRead/composition numerical payloads bind the learned result before evaluation. Every row retains answers, logits, CE, source/native packet/Q16 differences, gains/losses against both controls and frozen reader/null comparisons. Actual and padded sequence windows, positions, q4 flips, projections, zeros, gradients and full preparation/build/fit/export/evaluation cost are reported separately.

Both fixed fits complete all640 updates and backward batches. After independent source/native export and reload:

| Seed / panel (128 rows each) | Projection | Unrestricted parent | Learned q4 | Native answer CE: projection / parent / learned | Gains / losses versus parent |
| --- | ---: | ---: | ---: | --- | --- |
| 1 original | 94 | 102 | 112 | 1.143260 / 1.099175 / 0.965844 | 11 / 1 |
| 1 stress | 104 | 109 | 113 | 1.112837 / 1.091402 / 0.930197 | 13 / 9 |
| 2 original | 127 | 128 | 126 | 0.486264 / 0.470687 / 0.389766 | 0 / 2 |
| 2 stress | 126 | 128 | 128 | 0.487668 / 0.458134 / 0.380357 | 0 / 0 |

Learned total479/512 versus451 projection and467 unrestricted parent; this is24 parent gains and12 losses, not uniform preservation. Relative to projection there are39 gains and11 losses. CE improves in all four panels against both controls. Controls received no matching extra640-update dose, so this does not establish a benefit attributable to geometry or quantization.

All512 source/native answer and full packet/Q16 traces agree. Maximum logit drift across every actual position is0.000364304; this is broader than the prior construction's query-only metric. NoRead and composition numerical payloads remain byte-identical after dependency rebinding. On all12 parent-loss rows, frozen reader weights agree and head0 assigns98.7506–99.9800% of its total mass to the correct source occurrence. These losses occur despite strong correct-source selection; they do not justify changing the address selector. Value/composition/tail compatibility remains a hypothesis requiring a causal intervention to localize further.

Each seed trains5120 episodes. Actual/padded positions are245330/286496 and245334/285824. Batch time ranges36–80 and34–78, episode lengths24–80 and24–78, model context128, width32. Training continues exact saved RNG461503785751815875 and1220542080958524937 to17847168693577953940 and8892160473256393877. There are72786/104512 cumulative discrete coefficient flips and2669119/3758681 cumulative projected coordinates; these are repeated transition/boundary counts, not unique changed parameters. Neither seed clips an update's selected gradient norm.

PRESENT_ZERO occurs on4/9 actual training atoms across3/7 batches. This exercises the branch, but does not establish its causal benefit or adequate sampling for a general zero-value claim.

Both focused driver cases pass and the release example builds. Both actual check-mode attempts complete with zero updates, preserved RNG/weights, full baseline replay and independent final export/reload. Tests/build take345.419/7.173s; check workers22.407/26.321s. Fit workers148.275/124.302s, measured peak RSS512278528/478838784 bytes. The fit helper alone takes110.177/97.736s, including103.349/91.226s forward/backward/update. Full preparation/review/delivery cost is charged separately in the cumulative receipt. [Machine evidence](../evidence/geometric-value-q4-fit-2026-10-02.json).

The finite-choice straight-through backward is biased. The [Yin et al. analysis](https://arxiv.org/abs/1903.05662) establishes descent properties only under its stated simplified network/distribution conditions; it is not a convergence theorem for this H4 mechanism. Actual hard-model measurements above carry the result.

A useful result retains improvements and tradeoffs and advances the next numerical dependency. A valid unchanged/worse fit remains evidence at this fixed dose, optimizer, parents and surrogate; inspect hard coefficient movement and lost-row stages before any follow-up. Execution or integrity failure is not quality evidence. Perfect recovery and donor packet imitation are not gates, and neither outcome retires the geometry family. Broader offline donor compilation remains available.

Other unrestricted context/event/potential coefficients, selected parameter access, floating trunk/output, natural-input/session integration, general language/coding/reasoning and whole-path energy qualification remain open. This authored bounded task does not qualify them.

Decision: retain both learned q4 producers and all projected/unrestricted controls; end this fixed dose. Advance the remaining attention coefficient boundary through connected geometric source/compiler/training/native callers. Do not repeat the value/bank dose, demand perfect recovery, sweep scale/alphabet or detour to the vocabulary head. Claude's D19/session work stays concurrent and stack_grounded_session.rs is untouched.
