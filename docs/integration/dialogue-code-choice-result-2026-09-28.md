# Response-aware legal codes for the dialogue child: result

September 28, 2026. References #973, #962 and #1433 under #820.
- **Lab:** Lab 1 (Claude) ran Lab 4's (Codex's) frozen study unchanged, after the owner transferred it because Codex's allowance is exhausted.
- **Status:** executed. The candidate **fails the pre-registered retention rule**: lower development loss, fewer relation answers.
- **Disposition:** nearest-hard stays the same-child baseline. The learned artifact is preserved as a negative candidate, not promoted.
- **Evidence:** [`dialogue-code-choice-result-2026-09-28.json`](../evidence/dialogue-code-choice-result-2026-09-28.json), assembled from the sealed report roots. It holds all 58 turns in all five forms, the relation review, the fidelity traces, the code-change counts and both supervision records.

## What ran

**Study.** [Preparation](dialogue-code-choice-preparation-2026-09-27.md) and [sequential correction](dialogue-code-choice-sequential-preparation-2026-09-27.md).
- It learns a choice between the two legal neighbouring 4-bit codes of every coordinate of the width-576 child.
- The objective is complete-prefix response and EOS cross-entropy on the chat-v0 training corpus.
- Parameters: 512 alpha updates, learning rate 0.01, and a β schedule from 20 toward 2.
- It uses Codex's frozen `b49a2810` binaries: rounding `613e5a3f…`, integer `bef905df…`, observer `fd6edc7c…`.

**Fit resume** (updates 231 → 512).
- The campaign (`2f8c775d`) differs from Codex's supplied `c46af4d6` only in `resume_from`, the process time cap and the stop file.
- It ran 05:28:03–06:32:04 UTC (3,841 s) on 2 threads. Peak sampled RSS was 4.69 GB against a 6 GiB cap. It ended `COMPLETE_FIXED_RECIPE`.
- This attempt added 281 updates and 318,829 supervised targets; cumulatively the fit saw 579,259.

**Endpoints**, exactly as fixed before the run ([#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5863711869)).
- Steps: the learned and nearest evaluations on the 161-response panel, the learned integer pack, and the learned and nearest 58-turn native observations.
- Every step exited 0. Together they took 164 s, with peak RSS 343 MB.
- The same-binary nearest re-observation reproduces Codex's original nearest observation exactly: 909/3/4 greedy disagreements and identical NLLs.

**Size of the change.** The learned artifact moves **347,714 of 5,429,826 codes (6.40%)** away from nearest.
- The moves are spread over every weight matrix, at 4.4–7.6% each.
- The exception is the output norm, at 24.7%.

## Results

**Development panel.** 161 responses, 13,353 supervised targets, QQ F32 emulation:

| | Nearest | Learned | Δ |
|---|---:|---:|---:|
| Response mean NLL | 2.849148 | 2.796064 | −0.053084 |
| First-four-target NLL | 1.925600 | 1.824609 | −0.100991 |

**Fidelity to the continuous child.** Measured on its own 3,914 saved positions: 1,508 assistant and 2,406 other targets.

| | Nearest | Learned |
|---|---:|---:|
| Greedy disagreements, continuous → packed parameters | 909 | **962** |
| Interface and integer steps (QF→QQ, QQ→integer) | 3 and 4 | 6 and 9 |
| Assistant-target NLL (QQ; continuous is 1.035924) | 1.303527 | 1.307358 |
| Other-target NLL (QQ) | 3.9287 | 4.1101 |

**Memory relations at the question turn.** The actual native replies to turn 3 of each 3-turn conversation. QQ and integer agree on all ten except where noted.

| Relation | Continuous child | Nearest | Learned |
|---|---|---|---|
| Name: Alex | ✓ | ✓ | ✓ |
| Cat: Momo | ✓ | ✗ "The cat is named" | ✗ "Yes, I can help with that." (Momo restated at turns 1–2) |
| Colour: green | ✓ | ✗ (integer: "greenhouse") | ✗ generic (green echoed at turn 1 only) |
| Job: teacher | ✓, malformed | ✗ "The job is nearby." | ✗ "Yes, I can help with that." |
| Sister: Tokyo | ✓ | ✓ | ✗ "Yes, I can help with that." (Tokyo lost from turn 1) |
| Brothers: two | ✗ | ✗ | ✗ |
| Birthday: July | ✓, partial | ✗ | ✗ |
| Car: blue | ✓ | ✓ | ✓, weak ("The blue car is a great way to keep your carbon footprint.") |
| Instrument: piano | ✓, partial | ✗ | ✗ "Yes, I can help with that." |
| Food: pizza | ✓ | ✓, malformed | ✗ "Yes, I can help with that." |
| **Value present** | **9 / 10** | **4 / 10** | **2 / 10** |

**Other categories:**
- **Greeting.** Learned recovers "Hello! How can I help you today?" for "Hi there!", equal to the continuous child; nearest had lost it. Learned also answers "Thank you very much!" with a greeting.
- **Facts.** Paris is correct in every form. The other seven factual questions fail in every form.
- **Instructions and refusals.** All seven instructions and all eight underspecified requests fail in every form.
- **Length.** Learned ends at EOS more often: 37 of 58 turns in QQ, against 28 for nearest.
- **New failure mode.** Learned answers **5 of 10** "What is my …?" questions with the same "Yes, I can help with that."

## Decision

The frozen rule: "a repair must improve useful complete replies against nearest-hard and preserve the continuous child's demonstrated relations … lower NLL … alone do not qualify it."
- Learned lowers the panel NLL by 0.053, but it answers fewer relation questions than nearest: 2 against 4 of 10.
- It recovers neither Momo nor green when asked.
- It loses Tokyo and pizza, which nearest kept.

**The candidate fails.** It is preserved, and nearest-hard stays the same-child baseline. As pre-registered, no dose, seed, scale or decoder follow-up is selected.

## What this changes

*Reading, not a new capability claim.*

- **The objective pulled the artifact toward the corpus and away from its parent.**
  - Toward the corpus: lower corpus-response NLL, generic assistant phrasing and more EOS stops.
  - Away from the parent: 53 more greedy disagreements, and worse likelihood on the parent's own turns.
  - Corpus cross-entropy does not target what the retention rule measures, which is the parent's context-dependent relations.
  - *Hypothesis:* a **fidelity objective** targets it directly: distillation from the continuous child, a declared offline teacher, on its own complete-prefix trajectories.
- **Local moves on the fixed grid could not fix it.**
  - Moving 6.4% of codes by one legal step on the same per-row power-of-two grid did not restore the relations.
  - This is consistent with the synthesis finding (§1.4) that the grid itself, the weight representation, is the bottleneck.
- **For D4** (Lab 3; synthesis §4):
  - the "#1433 legal-code" arm is now a measured negative;
  - D4 compares Hadamard + E8 2-bit and Hadamard + grouped 4-bit codes against nearest;
  - its quantization-aware objective is **fidelity to the continuous child**;
  - its retention check is the ten question-turn relations above, including Momo and green.

  This is fixed before D4 runs.

## Scope and cost

**Scope:**
- This is open development evidence: a previously exposed 161-response panel and 58 turns. It is not a fresh held-out qualification.
- Dense parameter access is unchanged. This is not a selected-access or energy result.
- General chat, prose and instruction following remain unestablished for every form.

**Cost:**
- Fit resume: 3,841 s.
- Endpoints: 164 s.
- Director review and delivery: about 40 minutes (06:36–07:15 UTC).

**Storage:**
- 236 MB, on the owner SSD under `/Volumes/UOR-Workspace/uor-r4-lab/claude-t4-1433-resume/`. The fit root is 191 MB, including three checkpoints; the two observations are 20 MB each.
- Internal free space stayed at or above 49.9 GiB throughout.

The shared-ledger cursor is not claimed here; its reconciliation is an open ROADMAP §6 item.
