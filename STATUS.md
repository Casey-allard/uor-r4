# Status

Updated 9 October 2026. **Pre-alpha: no model holds a useful conversation yet.** Broad
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
| Softmax-free reads: soft (A), flock rank (B), B + prime-route copy (C), Hamming-rank (D) | M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032) | pre-registered | Can served reads drop the table-emulated softmax with no loss? |
| Route-holonomy read | M1 #2029 | pre-registered | Can the angle of h_j⁻¹·h_t rank earlier positions, order-aware and softmax-free? |
| Exact icosian holonomy lanes (E1) | M1 #2029 | pre-registered | Does an exact 2I group product beside the r-layer help, beyond a shuffled-geometry control? |
| Octonion-signed binding, then transport | M1 #2029 | pre-registered | Does a Fano-signed XOR keep order and grouping that plain XOR loses? |
| Shared `BitCode` primitive | M4 #2032 | planned (engineering) | One Hamming/popcount type for the native learner, R4G1 and the integer engine |

Pictures of the pre-registered mechanisms are in [docs/geometry.md § 11](docs/geometry.md#11-pre-registered-mechanisms-not-yet-measured); none of them has a measured result yet.

## Two model lines

| Line | Best measured result |
|---|---|
| A. Geometric stack language model | Open reply panel 43/232 `acceptable` (target 116) and v5 memory 10/40 `check_pass` (target 34) before the pointer-gate fine-tune; that run moves the memory half to 21/40 (22/40 matched control) and the unknowable rows 1/24 → 16–19/24, and the pre-registered supervision weight is not the cause ([record](docs/labs/pointer-gate-finetune-2026-10-10/README.md)). 19.9M chat stack: 0.933 BPB served at the recorded **64-window** protocol, multiplier-free (11.9 MB); the same artifact reads 0.877550 BPB float / 0.886838 BPB served at a [pinned 512-window protocol](docs/labs/criterion2-protocol-pin-2026-10-09/README.md), so the number is protocol-dependent (0.046 BPB between the two counts on identical bytes) and every BPB claim must name its window count and byte basis. criterion 2 is not met either way. 214M Plan A base: dev NLL 2.073. Code and arithmetic answers are still wrong. The ~96M run's result is recorded in the #820 history, not repeated here. |
| B. Native geometric learner (compiler, exact store, emitter) | Accepted 8/512 complete replies; separate conditional gate9/15. [Stratified reply fitting](docs/labs/m2-stratified-reply-2026-10-10/README.md) regresses8→2/512 and8→2/24 trained replies; REJECTED, inherited line count2/3. Recipe stopped; closed constructor line remains archived. |

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
