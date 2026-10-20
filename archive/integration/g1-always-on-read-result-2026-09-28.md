# G v1: the always-on address-driven read — result

September 28, 2026. References #973 and #820.
- **Lab:** Lab 2 (OpenCode), code commit `d8a11080` ([#1469](https://github.com/UOR-Foundation/uor-r4/pull/1469)).
- **Pre-registration:** [#973 comment 5876711862](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5876711862), posted before any fit.
- **Evidence:** [g1-always-on-read-2026-09-28.json](../evidence/g1-always-on-read-2026-09-28.json), assembled from the sealed roots; supervision receipt `g1.supervision.json` beside them.

**Status.** Complete, three seeds, six runs, all roots sealed with manifests and no error. **The frozen gate FAILS in every seed.** Under [D12](DECISIONS.md#d12--gates-promote-never-kill-reopen-geometric-candidates-port-the-native-engines-mechanisms-keep-a-geometric-toolbox) (gates promote, never kill), this is **not yet promoted at this scope**: the exact store and G stay active, and the diagnosed next step is recorded below. No seed, dose or codebook sweep follows.

## What was tested

D2's failure was the **read**: a learned per-token trigger placed on the last user-turn token fired 0 of 826 gold reads on held-out query phrasing, giving held-out Updated accuracy 0.000/0.000/0.015 (with a gold register, 1.000). G v1 changes exactly one policy: an entity or relation tag with a defined address reads the exact store **always** — both the current and the previous register — at the token that completes the address; reads no longer close the clause, and the trigger head, its gold labels and its loss are unchanged. The memory branch carries a full sub-branch per register. Everything else (world, data, schedule, seeds, evaluation, control) is D2's.

## The gate

Applied mechanically by `aerm-probe summarize … gates=g1`; the registered gate is held-out-template Updated accuracy ≥ 0.90 in every seed, with an equal-parameter control and development text NLL within 0.05.

| Seed | Memory arm, held-out Updated | Control, held-out Updated | Text NLL memory − control | Parameters memory / control |
|---:|---:|---:|---:|---:|
| 1 | **0.1575** (43/273) | 0.4652 (127/273) | −0.0162 | 1,428,816 / 1,428,556 |
| 2 | **0.0733** (20/273) | 0.7729 (211/273) | −0.0118 | 1,428,816 / 1,428,556 |
| 3 | **0.0293** (8/273) | 0.8132 (222/273) | −0.0335 | 1,428,816 / 1,428,556 |

**Outcome: FAIL.** Held-out Updated misses 0.90 in every seed; the text gate (≤ 0.05) and parameter parity (0.018 %) pass.

## Measured beside the gate (reported, not gated)

**Held-out templates and names** (correct/total):

| Class | Memory, seeds 1/2/3 | Control, seeds 1/2/3 |
|---|---|---|
| First | 31/144, 27/144, 53/144 | 41/144, 57/144, 75/144 |
| **Updated** | **43/273, 20/273, 8/273** | **127/273, 211/273, 222/273** |
| Reasserted | 10/31, 7/31, 15/31 | 14/31, 22/31, 22/31 |
| Previous | 60/195, 41/195, 39/195 | 43/195, 44/195, 54/195 |
| PreviousAbsent | 141/157, 149/157, 135/157 | 39/157, 100/157, 79/157 |
| Absent | 257/378, 263/378, 330/378 | 3/378, 107/378, 90/378 |
| Updated recency trap | 7/24, 3/24, 1/24 | 7/24, 13/24, 10/24 |

**In-distribution, fresh dialogues:** the memory arm is **perfect on every non-abstaining class** (138/138 First, 343/343 Updated, 26/26 Reasserted, 159/159 Previous, 168/168 PreviousAbsent) and free-running exact **32/32** in every seed. The control's seed-1 figures are 71/138, 308/343, 19/26, 88/159, 67/168 with free-running 21/14/24 (seeds 2–3: 307/343, 97/159 and 308/343, 95/159).

**Read diagnostics.** `read_events` 2,294 / 2,574 / 2,706 — the always-on read **does fire** (D2's trigger fired ~0). Failure trace: `Unavailable` 460 / 484 / 519, `NotSelected` 175 / 172 / 59, `Emission` 1 / 15 / 20, `WrongValue` absent (no key). Tag accuracy 0.975 / 0.974 / 0.977.

## Reading

1. **The always-on read removed the measured trigger failure and did not recover held-out accuracy.** Reads fire whenever the model's own tags define an address, yet held-out Updated improves only from 0.000/0.000/0.015 to 0.158/0.073/0.029 — nowhere near 0.90.
2. **The dominant held-out failure is now `Unavailable`:** the model's own store diverges from the gold store at the query key on held-out episodes. In D2 the dominant failure was a read that never fired (`NotSelected`); with reads always on, the surviving failures are on the **write/key side** — role-tag or address-key formation under unseen names and phrasings, not the read decision. The trace is coarse and does not separate a missed write from a wrong key from an eviction.
3. **The exact-store arm is more overfit to the training phrasings than the equal-parameter dense control on this world.** In distribution it is perfect where the control is not; held out, the control's recency-like policy answers Updated 0.47–0.81 while the store arm answers 0.03–0.16. Abstention (`Absent`, `PreviousAbsent`) still generalises far better in the memory arm.
4. **The always-on read is not yet promoted at this scope.** It removes the explicit learned read-trigger condition but is **not sufficient** for language-to-address generalisation at 1.4M probe scale. D12 keeps the exact store and G active; the failing seam is recorded as this mechanism's next diagnosed step.

**Next unit (recommendation, for the shared-model blocker).** A bounded diagnostic on the same world and roots that separates the `Unavailable` cause — role-tag/address-key errors by slot and class, missed writes, and eviction. If the split is address-key formation, the next intervention targets held-out-name/phrasing tag generalisation with the same control; if it is writes, the write path. This is the recorded step under D12 item 1 and the D2/G reopening (D12 item 2, "I1's store, with G's learned read aimed at held-out phrasing"), together with the reopened geometric candidates that belong to G: the geometric sparse index with magnitude carried, and 2I/E8 read codes trained in rather than snapped post hoc. No store rewrite follows from this negative.

## Scope and limits

- One parent (D2's world and schedule), one read policy, three seeds, a 1.4M-parameter probe on synthetic single-token slots with five template families; evaluation on 512 fresh and 512 held-out episodes plus 32 free-running answers.
- The trace is a four-class diagnostic, not a decomposition of the store divergence.
- No trained weights are saved by this runner; the sealed roots hold reports only.
- This audits the **probe's** exact-store interface, not the stack's Lorentz read, and establishes no serving, dialogue or general-language claim.

## Resources

- **Run:** 19:27:54Z → 22:53:41Z, **12,347 s wall**; three processes × 2 Rayon threads; per-process 12,305–12,315 s; peak sampled RSS 1.54 / 1.70 / 1.68 GiB (3 GiB guard never approached); ~100 MB under `/Volumes/UOR-Workspace/uor-r4-lab/opencode-g1-gread/`.
- **Guard history:** the pre-registered 9,000 s wall guard was extended twice (to 12,000 s then a 14,500 s hard ceiling) because the owner-authorized merge review's `rustc`/`cargo test` ran concurrently (load average 18–35); both extensions are recorded prospectively on #973 before use. The monitor loop was handed to the lead before the first guard; the run completed with no guard kill. All fixed conditions and the gate are unchanged.
- **Preparation/build/check:** ~1,100 s plus a 6.6-minute focused test build; the independent review and its two lead-run checks are linked from #1469.
