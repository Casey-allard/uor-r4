# REPORT: the README front page in plain terms, claude-299

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Order: [ORDER.md](./ORDER.md).

## What did not work, or is not done

- **The repository has no GitHub description and no topics.** `gh repo edit` returns 404 for this account. A UOR-Foundation org admin must set them; the proposed text is on #2028 (6688130273).
- **The first draft overstated two results**, and the fresh review caught both:
  - It said more training text helped at both seeds. That lever is decided at 8,192 steps only, and reads `mixed` at 32,768.
  - It said the model beats neither baseline. claude-297 beats the unigram; only T2 is unbeaten.
- **The first draft also dropped the web Studio's optional OpenAI-compatible API disclosure.** It is restored.

## What changed

- **README, "What this is":** the bet and the pre-alpha status, in two paragraphs.
- **README, "Where we are":** the M1 table with the unigram and T2 baselines defined in words, and the M1 limit beside it (templated panels; 62.5% of the held-out panel's reference replies appear verbatim in TRAIN).
- **README, "The open research question":** whether the model learns human-written text.
  - claude-295: flat loss, behind the unigram.
  - claude-297: ahead of the unigram; inside T2's band at 32,768 steps; data is the lever decided at 8,192 steps.
  - claude-298 is next, on #3407.
- **README, "Where we're going":** the human-text bar is named as the immediate work, following the owner's one-track direction (#2028 6675163438).
- **CONTRIBUTING.md:** the milestone line reads "M2 is blocked and M3 is held", matching #2030 and #2031.

## Measurements

None. This is a documentation unit with no model run.

## Checks run at the head

- Every relative link in README.md resolves (`test -e`).
- `cargo run -q --locked -p uor-r4-claim-wording -- .` passes.
- **Fresh review, round 1: CHANGES.** There were four wording findings and one note; every number matched its source. All were applied at `53232156057cc4b9f5c8d530b9cebf4f37c7552a`, and the approval is recorded at the exact head.

## Skills used

None.

## Still open

- **The GitHub description and topics** need an org admin.
- **Three internal working files remain at the root:** STATE.md, STATUS.md and HANDOFF.md. ROADMAP.md and PLAN.md still carry session-id language. They need moving or rewriting, which is outside this unit.

## Plain summary

The README now tells an outside reader, in plain words:
- what the project is;
- what milestone 1 showed, and what it does not show;
- the open question: the model does not yet beat a simple next-word table on human-written text.

References #3380.
