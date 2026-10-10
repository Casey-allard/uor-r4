# Status

Updated 10 October 2026. **Pre-alpha: no model holds a useful conversation yet.** Broad
reasoning, coding, frontier capability and lower complete-path energy are not established.

This page is navigation, not a results ledger. Results and artifact identities are in
[current state](docs/integration/current-state.md); the plan is the [ROADMAP](ROADMAP.md)
(milestones M1-M8) with long-form history in the [canonical plan](docs/integration/project-track.md).
Coordination and live status are on the tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028).

## Experiments in flight

| Experiment | Milestone | Status | Question |
| --- | --- | --- | --- |
| Native VSA retraining (4 arms × 2 seeds) | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: KEEP** ([record](docs/labs/vsa-native-test-2026-10-09/README.md)) | Trained VSA (fixed codes) improves held-out BPB by 0.014–0.023. Icosian-root codes don't: they collapse token identity. Mode 2 (root + per-token residual) is worse than fixed codes on all 4 cells: not KEEP |
| Pointer-gate fine-tune on generated recall dialogues (3 arms, matched control) | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: KEEP the data change, REJECT the supervision** ([record](docs/labs/pointer-gate-finetune-2026-10-10/README.md)) | v5 memory `check_pass` 10/40 → 21/40 pre-declared arm, 22/40 matched control (p = 1.00 between them), unknowable 1/24 → 16–19/24, at a reply-panel cost of 43/232 → 28/232 (29/232 bar, p = 0.0237; control 22/232): the recall-dialogue mixture carries both the gain and the cost, `gate_supervised_loss` adds nothing measurable on either frozen panel. Criterion 1 still unmet (≥ 34/40 memory, ≥ 116/232 reply) |
| Mixture dose: recall share 0 / 10 / 25 / 55 % of response episodes | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: REJECT as a keeper, frontier kept** ([record](docs/labs/mixture-dose-2026-10-10/README.md)) | v5 memory `check_pass` 10/40 (base) → 5/40 (0 %) → **20/40 (10 %)** → 20/40 (25 %) → 22/40 (55 %); unknowable 1/24 → 0/24 → 0/24 → 15/24 → 19/24; reply `fluent_and_relevant` 43/232 → **37/232 at 10 % (paired p = 0.42, not significant)** → 28/232 (25 %, p = 0.0081) → 22/232 (55 %, p = 0.0002). The pre-registered bar needs memory ≥ 21/40 **and** reply ≥ 29/232 together; the new doses are one row short on memory. D55 re-ran bit-identically to #2145's matched control |
| Read-binding supervision (`read_binding_supervision`) on the 10 % mixture | M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) | **done: REJECT — objective met, panel flat** ([record](docs/labs/read-binding-2026-10-10/README.md)) | Training stream: bound-value mass 0.732 → **0.896** (W = 0.1) / **0.934** (W = 0.5), competing mass halved, binding NLL 0.67 → 0.13–0.18. Frozen panel: memory `check_pass` 20/40 (anchor) → **19/40** → **17/40**, unknowable 0/24 → 5/24 and 4/24. Failure-cause split: **wrong-value rows 17 → 12 → 14** (9 repaired, 4 broken). With the copy gate (#2145) also flat, the mixture remains the only lever this line has moved |
| Softmax-free reads: soft (A), flock rank (B), B + prime-route copy (C), Hamming-rank (D) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | pre-registered | Can served reads drop the table-emulated softmax with no loss? |
| Route-holonomy read | M1 #2029 | pre-registered | Can the angle of h_j⁻¹·h_t rank earlier positions, order-aware and softmax-free? |
| Exact icosian holonomy lanes (E1) | M1 #2029 | pre-registered | Does an exact 2I group product beside the r-layer help, beyond a shuffled-geometry control? |
| Octonion-signed binding, then transport | M1 #2029 | pre-registered | Does a Fano-signed XOR keep order and grouping that plain XOR loses? |
| Shared `BitCode` primitive | M4 #2032 | planned (engineering) | One Hamming/popcount type for the native learner, R4G1 and the integer engine |

Pictures of the pre-registered mechanisms are in [docs/geometry.md § 11](docs/geometry.md#11-pre-registered-mechanisms-not-yet-measured); none of them has a measured result yet.

## Two model lines

| Line | Best measured result |
|---|---|
| A. Geometric stack language model | **Supervision paths are flat on the acceptance panel**: the read-binding objective reaches bound-value mass 0.90 on the training stream and leaves v5 memory at 19/40 (W = 0.1) / 17/40 (W = 0.5) against the 20/40 mixture anchor, repairing 9 wrong-value rows and breaking 4 ([record](docs/labs/read-binding-2026-10-10/README.md)); the copy gate was flat in #2145. Mixture dose (10 % recall share): v5 memory 10/40 → **20/40** `check_pass` with the open reply panel statistically indistinguishable from the base's (43 → 37/232, p = 0.42); abstention (unknowable rows) is bought separately and only at 25 %+ (1/24 → 15/24) ([record](docs/labs/mixture-dose-2026-10-10/README.md)). Before that, the pointer-gate fine-tune had moved memory 10/40 → 21/40 (22/40 matched control) at a reply cost ([record](docs/labs/pointer-gate-finetune-2026-10-10/README.md)). Open reply panel baseline 43/232 `acceptable` (target 116) and base memory 10/40 `check_pass` (target 34) are unchanged and v5 memory 10/40 `check_pass` (target 34) before the pointer-gate fine-tune; that run moves the memory half to 21/40 (22/40 matched control) and the unknowable rows 1/24 → 16–19/24, and the pre-registered supervision weight is not the cause ([record](docs/labs/pointer-gate-finetune-2026-10-10/README.md)). 19.9M chat stack: 0.933 BPB served at the recorded **64-window** protocol, multiplier-free (11.9 MB); the same artifact reads 0.877550 BPB float / 0.886838 BPB served at a [pinned 512-window protocol](docs/labs/criterion2-protocol-pin-2026-10-09/README.md), so the number is protocol-dependent (0.046 BPB between the two counts on identical bytes) and every BPB claim must name its window count and byte basis. criterion 2 is not met either way. 214M Plan A base: dev NLL 2.073. Code and arithmetic answers are still wrong. The ~96M run's result is recorded in the #820 history, not repeated here. |
| B. Native geometric learner (compiler, exact store, emitter) | [Episode-bottleneck learning](docs/labs/m2-bottleneck-2026-10-10/README.md) improves saved complete replies **145→175/512**: 34 gains, 4 losses,141 retained; six of the original eight remain. KEEP, line count 0/3; target 256 then fresh 40% still unmet. The new log-mean-exp objective replaces phase balancing for the same 115,200 Q4 field with frozen Source48/Generate64 upstream. Length8 improves 2→8/128, but entry correctness 454→340 and evaluator CE3.39448→3.48474 regress. The rejected 139 phase-balanced continuation remains preserved and stopped. Next: after protected delivery/cleanup, pre-register continuation from saved 175 under the successful new objective. Ordinary24/96 and constructor/attribution/solver lines remain closed. |

Both lines join at M4, one model served under D11.

## Serving

The D11 engine is bit-exact with the float path. `uor-chat --stack <ARTIFACT.lut>` serves the
geometric stack (PR #2050, merged 9 October 2026; greedy decoding only, and a measured bundle
negative is recorded there). The Pages Studio still runs the older R4G1 router, not the native
or stack model. Energy savings are unmeasured.

## Compute and labs

- Pods only through `scripts/pod/uor-pod`: ladder 5090, then 4090, then PRO 6000; at most 4
  pods and $8/h across all labs. See [compute](docs/labs/compute.md). Leases and the event log
  are on the compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037).
- Labs: Claude, Codex, DeepSeek. Work is claimed on the milestone issues
  ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) to [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036)); see the [lab protocol](docs/labs/protocol.md).
- `origin/main` is the single source of truth; branches are temporary delivery steps.
- Protected PRs need recorded exact-head review and checks run at that head. The queue
  acknowledgement jobs are not compile or test evidence.
- Open bugs: [#1542](https://github.com/UOR-Foundation/uor-r4/issues/1542), [#1476](https://github.com/UOR-Foundation/uor-r4/issues/1476), [#1718](https://github.com/UOR-Foundation/uor-r4/issues/1718), [#1738](https://github.com/UOR-Foundation/uor-r4/issues/1738).
