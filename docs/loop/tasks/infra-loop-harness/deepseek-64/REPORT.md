# REPORT: infra-loop-harness, deepseek-64 (refresh four stale roadmap item sections)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904 (umbrella). Base: `a2e74123a`. Partial unit; no item status, bar or hold touched.

## What did not work, first

- **The prepared patch had two factual slips that review caught.** It cited PR `#3016` for the object-store consumer (that PR is the m1-address consumer pin); the consumer landed as `4797ca883` under **#2960**. It also left the m7 list numbered 1, 2, 4 after merging two bullets. Both fixed before commit.
- **The prepared patch over-simplified the m1-position FIT attribution.** The sheet's FIT is Kimi's non-author post-freeze review (kimi-32, **#3108**), enabled by the owner-registered bar (`6356034555`, 2026-10-23T02:36:36Z) and the frozen 1,620-row slice (**#3046**, `f76dc5c7…`); the text now names each source instead of compressing them into one citation.
- **This unit does not fix the underlying stale-text class elsewhere.** The patch format is not reproducible; the refreshed text was written directly. Other items may still have stale sections (a separate audit is needed).

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/m1-position-mechanism.md` | "No position mechanism is assigned" replaced by delivered mechanism (`3e3967c7`), FIT-enabled sheet (kimi-32/#3108, bar `6356034555`, frozen slice #3046) and the open GPU-wiring/registration run |
| `docs/roadmap/items/m4-lawful-likelihood-bpb.md` | "still unbuilt" replaced by instrument+design delivered (#2940, #2945) and `TEST FITNESS: NOT_FIT` with the export pin and owner §5.1 conditions (#3099) |
| `docs/roadmap/items/m7-studio-self-host-page-assets.md` | three-bullet list becomes: verification and both pages done (#3073 static trap, #3080 runtime trap, #3110/`0d5d6791d` self-hosted demo); tokenizer leg remains; GitHub-API position recorded (numbering contiguous) |
| `docs/roadmap/items/object-store-consumer-m3.md` | "nothing consumes it" replaced by consumer+controls delivered (`4797ca883`, #2960), `export_scope`/`import_scope` wired, interop contract (#3106), bars unrun pending model/FIT/release |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| cited commits on main | `3e3967c7` (m1-position mechanism #2953), `4797ca883` (object-store consumer #2960), `0d5d6791d` (m7 demo #3110) |
| cited merged PRs | #2940, #2945, #3046, #3073, #3080, #3106, #3108 all MERGED |
| item frontmatter/status changes | none (diff touches "What is missing" text only) |

## Checks run at the head

`git diff --check`; `xtask roadmap check` (91 items OK); roadmap render/sync (no diff); `loop report --check`; `loop pr-check --local`; `loop gates --quick` — see the closing line.

## Skills used

- `solution-integrity`: every cited commit/PR was verified merged before wording it as delivered, and two citation errors in the prepared text were corrected.
- `scope-guard`: four roadmap item text lines, no statuses or frontmatter.

## Still open

- **The four items remain at their current statuses** (m1-position ready, m4 lawful ready, m7 ready, object-store ready); the refresh records reality, it does not advance a bar. Remaining blockers are unchanged (model/FIT/release for the bars, tokenizer artifact for m7).

## Plain summary

- Rewrote four stale "What is missing" sections to say what has actually been delivered on main, with citations, while keeping every item's status and scope untouched.