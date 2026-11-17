# ORDER for the November public status render

Lab: codex, acting as Kimi's cover for this refresh. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row 8. ORDER base: `e19cf2188cc672a38cffeb7b4ea1f748d142025f`. References #2904, #3319, #4060 and #4069.

## Goal

Regenerate the four shared views after the public documentation inputs in #4060 and the decision-owner renderer in #4069 reach main. Charter §5.2 assigns the PM cover role to the lab Casey names. D68 and §5.1 assign decisions about bars, status, scope and stop rules to the lab's top session after its decision review. This render transcribes committed inputs, including the recorded decision owner.

## Cancel probe (run first)

Confirm both PRs merged and their merge commits are ancestors of freshly fetched `origin/main`. Merge that main forward into `kimi/render`. Compute `git merge-base origin/main HEAD`, require it to equal `git rev-parse origin/main`, and use that full SHA as the render stamp. Cancel the refresh if all four views already match at that stamp. Repeat the merge-base check and regenerate if main advances before delivery.

## Files in scope

- `STATE.md`
- `HANDOFF.md`
- `docs/harness/LESSONS.md`
- `AGENTS.md`, only the generated section between the two `loop:lessons-top-ten` markers
- This ORDER and its REPORT under `docs/loop/tasks/infra-loop-harness/codex-8/`

## Done when

Build the harness from merged source and run `uor-harness loop render --at <merge-base>`, followed by `uor-harness loop render --check`. Check the four stamps explicitly, compare the rest of AGENTS.md byte-for-byte to the merge base, and verify the roadmap with `xtask roadmap check` and `xtask roadmap render --check`. The REPORT records the actual base, the decision-owner readback, full gates, local policy and exact-head review. Deliver through the protected queue and verify the views against their stamped sources on fresh main.

## Owner approval

Recorded at `2026-11-17T21:00:58Z` (transcription time). Casey named Codex as Kimi's cover for this generated-view refresh. His reply was "Yes, for this refresh". The approval covers regenerating the marked lessons block in AGENTS.md as part of these four views.

## Stop and decide if

Stop if the renderer would change an input, another file, or text outside the AGENTS.md markers, if either input PR is absent from the chosen stamp, or if the generated decision owner conflicts with its source. The top session owns any separate decision and public delivery.
