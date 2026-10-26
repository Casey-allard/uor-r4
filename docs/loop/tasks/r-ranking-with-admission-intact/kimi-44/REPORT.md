# REPORT: r-ranking-with-admission-intact, kimi-44

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision: [../kimi-43/DECISION.md](../kimi-43/DECISION.md).

Closes #2884 at its unchanged diagnostic done-when only.

## What did not work, first

- **The writer loses to the same-admitted-row unigram control.** Sealed #2831 Q1 admits 4,292 DEV targets and ranks 871 first (20.29%). Host counting of those stored GPU rows finds 972 targets equal to token 1, the full-panel unigram argmax (22.65%), 101 more than the writer. The earlier `754 / 1,920` table belongs to Rung-0 Q4 and is not Q1 admission-matched.
- **Admission remains a separate ceiling.** Q1 leaves 9,995/14,287 targets outside the shortlist. Tying T2 requires at least 6,300/14,287 = 44.10% admission even with perfect ranking; at 4,292 admitted rows it would require an impossible 146.78% conditional top-1. Tying Unigram requires 28.56%.
- **Admission “intact” is gate-relative, not unchanged.** Q1 held 256/256 cells and mean 126.49 tokens/row, with full admission 4,292/14,287, but P_succ fell 245 rows from 1,351 to 1,106.
- **The proposed six-run full-dose ladder was not run.** D67 failed (#2884 comment `6424741498`): `Arm-Rel-K16` is rejected by the A8-only validator and baseline, prior measured Relation throughput projects above the 35-minute gate, current training stops at Q1/consecutive rejections, and three-arm grading outputs are absent. No pod launched; additional GPU spend was $0. The original FIT is preserved unchanged.
- **Stage T is not two-seed Q4 evidence.** Its two seed bindings produce identical table/gate trajectories; FrozenZero stopped at batch 3,008 and Relation at Q1. It is context only, not the satisfying run for this close.

## What changed

- Added the D67/D68 decision record and superseding kimi-44 ORDER; retained the kimi-43 ORDER and original FIT as history, with the FIT unchanged after its review. D68 owner log: #2884 comment `6424978255`.
- Closed the item `done` strictly on its existing diagnostic contract, corrected its stale missing-evidence text without changing the done-when, and re-rendered the roadmap.
- Updated the lab record with the same-row unigram negative, cross-checkpoint labeling, P_succ drop, T2 admission arithmetic and explicit `NOT_RUN` scope; added one negative notebook row.
- Size: 8 files, `+192 / -16`; 0 code and 0 test lines, documentation `+189 / -13`, generated roadmap `+3 / -3`. The ORDER gave no numeric line estimate; the ratio is not applicable.

## Measurements

Evidence is the sealed RTX 5090 #2831 Q1 run at batch 4,096, epoch 258, seed 1. No new model or parity execution occurred on host or pod.

- HF store object `caseyallard/uor-r4-store:results/antigravity/a8-full-dose-rerun-2026-10-25/a8-full-dose-rerun-20261025.tar.gz`, SHA-256 `4a744188f14c934f89e57a1be83ecc8f2cb211620290a6539217837f809bae55`; locally extracted under `local/kimi/ranking-admission/q1-archive/` for checksum and row inspection.
- Q1 evaluation SHA-256 `6b924afa04103467814b6c71773f7df2c7e6df342f045d83434d3dafe8372943`; Q1 checkpoint `driver-04096.json` SHA-256 `9a7dd77037dc3e239a73a3900c62fad41279a936727b950b9224f11f2b273184`.
- Admission: 4,292/14,287 full and 1,106/3,847 P_succ; 256/256 occupied cells; mean 126.49 admitted tokens per DEV row (`1,807,135 / 14,287`).
- Ranking: 871/4,292 = 20.29% conditional top-1 and 1,455/4,292 = 33.90% conditional top-8. Full-panel top-1 is 871/14,287, below Unigram 1,226 and T2 6,300.
- Ordinary same-row always-token-1 control: 972/4,292 = 22.65% unigram argmax hits. Command, from the extracted sealed Q1 directory:

```sh
jq '{admitted: ([.evaluation.batches[].rows[] | select((.admitted_cells - .address_rivals) == 1)] | length), token1_on_admitted: ([.evaluation.batches[].rows[] | select((.admitted_cells - .address_rivals) == 1 and .target_token == 1)] | length), top1: ([.evaluation.batches[].rows[] | select(.top1_correct == 1)] | length), top8: ([.evaluation.batches[].rows[] | select(.top8_correct == 1)] | length)}' q1-dev.json
```

It returned `admitted=4292`, `token1_on_admitted=972`, `top1=871`, `top8=1455`. This is host counting of stored GPU output, not model evaluation. Paired counts are both correct 755, writer-only 116, control-only 217 and neither 3,204; the 101-row difference is reported descriptively, not as an admission-restricted evaluation or a new significance test.

## Checks run at the head

- `shasum -a 256 a8-full-dose-rerun-20261025.tar.gz`: matched `4a744188…`; `q1-dev.json` and `driver-04096.json` matched the pins above. `q1.json` also matched `1e436e66a243cf7f9367c14372ef44ff1b9459fce98bf172100f28c5323f5ef7`.
- `uor-harness loop gates --quick`: `fmt-claim-wording`, `fmt-harness`, `claim-wording` GREEN.
- `uor-harness loop gates --full`: 11/11 GREEN at the review head; task receipt under `local/kimi/gates/`.
- `uor-harness loop report --check --order docs/loop/tasks/r-ranking-with-admission-intact/kimi-44/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS; scope and commit-order warnings are advisory under D76.
- `git diff --check`: PASS.

## Skills used

- `uor-research-architect`: artifact-bound measurement, exact negative scope, ordinary same-row control and protected delivery.
- Fresh-context `uor-experiment-designer` review for D67 and `uor-architect-reviewer` review for D68; the lead made and logged the decision.

## Still open

- The exact six-run full-dose ladder is `NOT_RUN`. Any future version needs a repaired design and implementation, a new sheet, fresh FIT and D67. The original FIT and D67 failure preserve the instrument findings.
- The registered Q1 admission-matched GPU `U_restr/T_restr/G_restr` evaluation was not run; the 972 result is only the ordinary same-row unigram argmax control. No paired T2 or read-code comparison is claimed on those exact rows.
- Stage T seeds are determinism replicates, not independent training samples. Fresh transfer remains sealed; no above-unigram, fresh-panel, M1 or mechanism-promotion claim follows.
- The related longer-coverage route is #2878, which does not authorize or subsume this ladder. Out-of-scope record issues in `docs/mechanisms/admission.md` and #2836 were not changed.

## Plain summary

The model first chooses a small set of possible token cells, then ranks tokens inside them. On the saved Q1 run, the correct cell was available for 4,292 of 14,287 development rows. The model chose the right token first on 871 of those rows. Simply always choosing the most frequent training token would have been right on 972 of those same rows, so the model's inside-cell ranking still loses to a basic count table.

The diagnostic requested by #2884 therefore already existed. The six longer runs would answer a stronger full-dose question, not this item's literal done-when. The item closes with that negative result and exact limits. The larger full-dose experiment was not performed, and this result does not show fresh-data transfer, independent-seed replication, M1 completion, or a working general language model.
