# REPORT: m1-normalization-mechanism, claude-57 (the non-author FIT review the sheet asks for from Claude or Kimi)

Lab: claude. Order: [ORDER.md](ORDER.md). Item #2842 (`References`; the item stays `ready`). Base: `dea171338`.

## What did not work, first

- **The verdict is `NOT_FIT`, so the sheet still clears no compute.** Four prerequisites remain:
  - two are build work: the GPU wiring and the configuration pin;
  - one needs a PM or owner card: two terms of the bar;
  - one is a sheet edit: what each outcome decides.
- **I dropped one finding from the fresh-context reviewer because the card disproved it.** It said §3's endpoint address-distance condition is missing from the registration. The registered card `6338596024` has that condition in its item 1. deepseek-40's paraphrase in the sheet's Verdict left it out, but §3 itself is right.
- **The pasted record differs from `prep fit`'s output by one space.** `prep::render_fit` (`crates/uor-r4-harness/src/prep.rs:510`) ends the D22 line with a trailing space, which `git diff --check` rejects, so I removed that space. The same string also indents the last sentence by nine spaces, because a line continuation lost its `\`. Both are harness bugs; see the gaps below.
- **The first fresh review (APPROVE at `12e955074`) found two non-blocking wording problems, now fixed:**
  - prerequisite 1's clearance asked for the wiring to land, which is stricter than the parent A8 sheet's precedent. It now asks for the wiring to be specified.
  - "no command" now reads "no command for the GPU arms", because step 1's CPU test is a command.

  The recorded `note:` keeps its original wording, because the ledger receipt carries it.
- **The verdict parser reads the first record, not the latest.** `prep::test_fitness` (`prep.rs:97`) returns the first line that starts with `TEST FITNESS:`.
  - This sheet now has two records, deepseek-37's first.
  - Both say `NOT_FIT`, so the parsed status is right today.
  - A later `FIT` appended below them would still parse as `NOT_FIT`.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` | The Status line names this review. A new `### Non-author FIT review (claude-57, 2026-10-23)` subsection holds four prerequisites in priority order (each with its evidence and what clears it), three items that do not block, what holds, and the rendered `prep fit` record. |
| this folder | The ORDER (committed first, alone) and this REPORT. |

**Size:** about 60 docs lines; docs-only. The receipt went to the LIVE ledger shard, so there is no `receipts.jsonl`.

## Measurements

All at `dea171338`.

- **Controls:** `cargo test -p uor-r4-discrete --lib normalization_mechanism` gives **6 passed**, 0 failed.
- **GPU wiring:** `git grep -E 'NormalizationMode|ScaleMatchedSnap|OrbitRootSnapNorm' dea171338 -- ':!third_party' ':!docs'` finds these names only in `crates/uor-r4-discrete/src/normalization_mechanism.rs`. Nothing in a `.cu` kernel or any other crate refers to them.
- **Lawfulness:** `grep -E '\bf32\b|\bf64\b|\.exp\(|matmul|gemm|softmax|tanh|sigmoid'` over the module (641 lines) finds nothing.
- **The assembled trunk:** item `m1-trunk-assembly-and-training` (#2844) is `status: held` with `needs_gpu: true`, and it depends on `m1-normalization-mechanism`.
- **The bar:** card `6338596024` was registered by `6356034555` at 2026-10-23T02:36:36Z. In its item 2:
  - the threshold `F_I` attaches to "beats";
  - "non-inferior to … `C_unscaled`" has no margin;
  - "replicating on the sealed fresh V2 draw" has no threshold.
- **Fresh V2 digests:**
  - tokens `0b7e754c…e1b8` and mask `a1cda529…bf5e` are in `docs/labs/a8-dose-2026-10-19/evidence/frozen-zero-report.json`;
  - seal `df5d074a…395f` is in `docs/labs/attention-dose1-dispatch-2026-10-18/README.md`;
  - the parent sheet carries only prefixes (its line 87).
- **The record:** `prep fit -m m1 --verdict NOT_FIT --mechanism OrbitRootSnapNormV1` recorded one receipt, and `prep list` reads the sheet as `NOT_FIT`.

## Checks run at the head

`loop report --check --order`, `loop pr-check --local`, `git diff --check`, and a fresh-context review record at the head (docs-only).

## Harness gaps (for Kimi as PM)

1. `prep::test_fitness` returns the first `TEST FITNESS:` line. A sheet that collects reviews over time needs the newest record to win. Either the parser takes the last record, or `prep fit` says where the record goes so that the newest comes first.
2. `prep::render_fit` emits `(D22). ` with a trailing space, then `\n\n         Every mechanism…` with a nine-space indent. Pasting the record verbatim fails `git diff --check`.

## Skills used

- `solution-integrity`: each point in the verdict was re-checked by a command at the base, and a reviewer finding that the card disproved was dropped rather than kept.
- `scope-guard`: the review sets no bar value. The two terms that cannot be scored go to a card.

## Still open

- **The four prerequisites,** listed in the sheet.
- **The card for the two bar terms,** which belongs to the PM or the owner. I filed it on #2028 with a recommendation and left the routing to the PM.
- **The run itself,** which still needs the GPU wiring, a pinned configuration, PM dispatch and Casey's release.

## Plain summary

- Claude reviewed the normalization sheet as the non-author lab and recorded `NOT_FIT`. No compute is cleared.
- The mechanism's code and its six controls are sound. The sheet can't clear yet for four reasons:
  - the GPU arms don't exist yet;
  - it mixes two configurations;
  - two terms of the bar can't be scored;
  - it doesn't say what each outcome decides.

References #2842
