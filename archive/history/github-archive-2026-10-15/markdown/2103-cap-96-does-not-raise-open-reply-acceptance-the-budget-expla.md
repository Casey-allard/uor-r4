# #2103 — Cap 96 does not raise open-reply acceptance: the budget explains the cut, not the failure (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:37:11Z
- **Updated:** 2026-10-09T22:43:54Z
- **Closed:** 2026-10-09T22:41:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2103
- **Merged:** 2026-10-09T22:41:14Z
- **Merge commit:** c7fce45b19ffb0804b7ef81b697b9ef4beb3637d
- **Base / Head:** main / UOR-Foundation:reply-cap96
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question, and the answer

Result 4 measured that the open reply panel's mid-clause failures are the 64-token budget. This is the measurement it justified, pre-registered on #2029 before any pod was created: **give the model 50% more room and see whether acceptance moves.** It does not.

**Run A (control, 64 tokens): 44/232. Run B (96 tokens): 38/232. B − A = −6 cells**, inside the pre-registered ±14-cell judge-noise floor, therefore **NOT A RESULT**. The frozen decision rule fires on the negative side: **the budget explanation is bounded.** It explains why replies are *cut* (45.0% of the 189 failures) and not why the model *fails*.

| | acceptable | fluent | relevant |
|---|---:|---:|---:|
| sealed macOS, cap 64 (2026-10-03) | 43/232 | 70 | 53 |
| **Run A, Linux, cap 64 (control)** | **44/232** | 69 | 54 |
| **Run B, Linux, cap 96** | **38/232** | 62 | 48 |

**The frozen binary is macOS-only, and that is a result in its own right.** `chat-grade a5071cfe…` is **Mach-O arm64** and exits **126** on Linux; its `attempt.json` records a macOS `argv[0]`. So the sealed 43/232 was generated on the laptop, not on a pod, and its reproducibility on the platform the project now uses for GPU work was unproven. **Run A closes that.**

## The control converted an ambiguous comparison into a decisive one

| anchor: sealed macOS 64 → Linux control A 64 | |
|---|---|
| `acceptable` | **43 → 44 (+1)** |
| replies | **all 232 byte-identical** |
| the one cell | a judge verdict flip on `heldout-199`, no reply change |
| exact McNemar p | 1.0 |

So (a) the platform is **not** a confound, and (b) the four greedy-loop commits since 2026-10-03 (#1846, #2040, #2047, #2062) behave as their opt-in documentation claims — **measured, not asserted**. Without A, a −6 swing would have been unreadable: platform, code changes and cap all in play at once.

## Movement underneath the total

- **3 of A's 188 failures became `acceptable`** (`heldout-036`, `-114`, `-174`); **9 acceptable rows were lost** (`heldout-010`, `-024`, `-061`, `-071`, `-073`, `-082`, `-090`, `-185`, `-192`); exact McNemar **p = 0.146**.
- **Of the 85 mid-clause failures that were AT the 64-token cap — the exact population Result 4 identified — 2 became acceptable and 83 stayed failed.**
- **96 is simply the next wall:** 91 replies at exactly 96 and 34 at exactly 95 (against 95 at 64 and 41 at 63); failures at or above the cap **92/194 (47.4%)** against **87/189 (46.0%)** at 64.
- **Both judge components fell the same way** (fluent 69 → 62, relevant 54 → 48), so the drop is not a re-labelling artefact.

| tier | rows | A | B | | category | rows | A | B |
|---|---:|---:|---:|---|---|---:|---:|---:|
| `everyday` | 32 | 9 | 9 | | `smalltalk` | 8 | 6 | 6 |
| `K-clean` | 133 | 19 | **12** | | `simple_question` | 8 | 1 | 1 |
| `K-ill-posed` | 67 | 16 | 17 | | `simple_instruction` | 8 | 1 | 1 |
| | | | | | `follow_up` | 8 | 1 | 1 |
| | | | | | `heldout_first_turn` | 200 | 35 | **29** |

## Three consequences, stated plainly

1. **Criterion 1 does not need a declared cap.** Raising it does not help, so a token ceiling is not what is holding the reading down. The surviving defect is that the criterion is judge-only, with no deterministic check to be primary.
2. **The 62 completed-but-wrong replies are the whole story.** Failures that finish their sentences and are still rejected are where the capability gap lives — 62 such rows at 64 tokens, 9 acceptable rows newly lost at 96, and 83 of the 85 cap-cut failures still failing with more room. That is where the next piece should point, **not at the decoder**.
3. **No 128-token arm.** The decision rule says stop; the piece is closed; the 100M artifact stayed out of scope.

Framing, as pre-registered: this measures **how much of the gap is the token budget**, and it does **not** show the truncated replies would have been correct with more room — nothing was repaired, the model was simply given more room on the same questions, and the pre-registration's caveat is now measured: they were given 50% more room and did not become acceptable. A null is a good result.

## Run set

Pod `260rd8ondf4mzg`, 1 × RTX 5090, released and deleted at 22:41Z with nothing running; **≈$1.00**, against a pre-registered ≈$1.65 and a $3.57 worst case. Both runs `device=cpu` (the sealed run passes no `device=`, and the default is cpu), on the pod's own Linux build of main at `4a5fd0d1` (`chat-grade` sha256 `77256a3a…`), same panel bytes (`945c0c97…`, `019cc6f6…`, `5434cfd9…`), same model `d8a3c971…`, same tokenizer `d36d3e87…`, grader digest gated at `845dbda0…`, context 384, protocol 2; `check_panel` fits at both caps (168 and 232 positions of 384), so **the cap is the only declared delta**.

**Two failed launches are recorded with their exact symptoms** — lost shell quoting running from `/root` (exit 127) and the platform error (exit 126). Neither reached `report_output::claim`, `grades/` stayed empty until Run A, and **no number in this record comes from them.**

## Delivery

- **Result 5** added to `docs/labs/open-reply-panel-2026-10-09/README.md` — an **amendment, not a new record**: it is the same instrument and the same pre-registered line, and Results 4 and 5 are only readable together (Result 4 defines the population Result 5 measures).
- Newest-first `docs/integration/current-state.md` entry ending in **Next:**, now pointing at the completed-but-wrong set rather than the budget.
- `docs/compute/ladder-runbook.md` updated for operators: the sealed binary is macOS-only, a Linux build of current main reproduces its replies byte-for-byte, and the cap is a measurement choice to be declared.
- **STATUS.md / ROADMAP.md / #2028 unchanged** — checked, not assumed: 43/232 is unchanged and criterion 1 is still **NOT MET**.
- `scripts/open_reply_panel_cap_compare.py` carries the comparison (per-category/per-tier tables, failure-set movement, exact McNemar, cap-usage distribution).
- `check_claim_wording.py` passes.
- Evidence bundle `icloud:UOR-R4/results/deepseek/reply-panel-cap96-2026-10-09.tar` (1,417,728 B, md5 `3de8c41eb9eb7532e1243450365ce335`): both runs' sealed reports/manifests/attempts, the sealed macOS report, both comparison summaries and per-row tables, both runs' reply texts and token counts, the comparison and run scripts.

**References #2029** — does not close it: criterion 1 is unmeasured on this panel pending the judge-only repair.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

