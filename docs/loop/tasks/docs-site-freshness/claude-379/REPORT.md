# REPORT: docs-site-freshness, claude-379 (read book pages against their sources; fix the six wrong ones)

Lab: claude. Order: [ORDER.md](ORDER.md). PR #3976. References #3972. Base: `a70d6189d` (`a70d6189df3368575e745d351ed692cb000e9388`).

## What did not work, first

- **70 pages are still unchecked.** I read 23 pages whole this unit, out of the 93 that were unchecked at the base. That is the 18 research line pages, plus 5 of the six pages claude-378 named wrong. The other 70 were not read, so none of them has a `checked=` value. They are listed under "Still open".
- **`projects/README.md` is fixed but stays unchecked.** It names `projects/**` as its source, so it counts every change to a page under `projects/`. This PR edits `projects/msa/README.md`. The merge queue squash-merges, so main gets one new commit that touches its sources. With `checked=a70d6189d` that commit would make the page stale as soon as it lands. It can be checked in a later unit, against the merged commit.
- **`book-fresh` counts commits that change only a record line.** When one page under a hub page's source glob gets `checked=`, the hub page goes stale, even though no claim changed. I worked around it here by leaving `projects/README.md` unchecked. It is a tooling follow-up for the item, listed under "Still open".
- **Wrong claims found while reading, all now corrected** (each page's sources contradicted it):
  - `harness-governance` said 185 records, ops 4, and a span ending 19 Oct. The catalog has 187 records, ops 6, and two status snapshots dated 22 Oct. "None (175 of 185)" is now 177 of 187. The lines index carried the same 185 and a 1,173 total; the catalog now holds 1,175, and the tooling-lines sum is 209, not 207.
  - `m5-energy` said "none has been exported". The discrete M1 pair and the FBS `lut-stack` exports serve at their declared scopes. What is still missing is the M5 measurement itself: J/token, memory and tokens/s on the M1 laptop. Its bar also cited only `ROADMAP.md`, but "selected parameter access" is in `STATE.md`.
  - `r4g1-transformerless` called the fate of its crates "an open owner question". The owner decided on 19 Oct to repair them rather than archive them, and the mechanism stays PARKED (`docs/mechanisms/r4g1-graph.md`, "Decided on 19 October 2026").
  - `data-tokenizer` said the number-aware tokenizer had never been built. A three-arm comparison built it on 22 Oct, and all 100 two-digit pieces form (#2872, test `number_aware_three_arm_comparison_and_digit_piece_census`). No model has been trained on it.
  - The lines index's "Since 27 Oct" section called itself "current research through 31 Oct". It is now headed "27 to 31 October", is written in the past tense, and points to `PLAN.md` for current work.
- **No dated record was edited.** Notebook pages, REPORTs, `docs/labs/**` and `archive/` are untouched, as the ORDER's stop list requires.
- **Bench inbox:** `local/bench/INBOX.md` does not exist in the main checkout on this machine, so there was nothing to read or mark.

## What changed

- **The six pages claude-378 named wrong** (commit `da63f862d`):
  - `docs/start/repository-layout.md`: adds the rows for `bake-off/`, `rules/` and `src-tauri/`, and the top-level files `PLAN.md`, `STATE.md` and `HANDOFF.md`; marks `STATUS.md` retired; corrects the rows for `.codex-lab`, `third_party/` and `tools/`. Checked.
  - `projects/README.md`: says there are eight projects, adds a row for `uor-cuda-inverted-file` (#3476), and says six project pages instead of seven. Not checked (see above).
  - `projects/msa/README.md`: describes the folder's real contents (`README.md` and `code/README.md`) and fixes its links. Checked.
  - `docs/model/gpu-and-cuda.md`: `discrete.cu` has 749 lines; adds the `native-vector` module (2 entries, so ten modules with 44 entries); adds `ldmatrix`, `stmatrix` and `movmatrix` to the audited tokens; recounts the other-CUDA table from the files. Checked.
  - `docs/products/web-client.md`: the export list now has the memory, reply-start, discrete-prediction and `exactArithmetic` exports. Checked.
  - `docs/minimal_client.md`: `r4-app.sh` is now described as an archived launcher that no longer runs, since there is no `r4` binary. The client command is `cargo run --release -p uor-r4-wasm-router -- client …`, and the page shows the current banner and all 13 slash commands (`/exit` was added after the first fresh review found the count unconfirmed). Checked.
- **The 18 research line pages** (commit `9b4a50a01`): each was read whole against `archive/catalog.jsonl` (counts, spans, outcomes, milestones, recounted with `jq`), the records it links, the mechanism board and `findings.md`, and each records `checked=a70d6189d`. The fixes are listed above. `m1-discrete-attention` and the lines index also list `archive/catalog.jsonl` as a source now.
- **The item** (commit `9eddd2cb5`): its Sources sentence now says it "edits a page's text only to correct a claim its sources contradict before the page is checked". No bar, status or other scope changed. The item stays `ready`, because done-when 5 ("no version warning") is still open.

## Measurements

`book-fresh` on the base `a70d6189d`: **104 pages: 2 fresh, 0 stale, 93 unchecked, 2 no-sources, 7 unrecorded, 0 broken, 0 malformed.**

`book-fresh` at the head: **104 pages: 25 fresh, 0 stale, 70 unchecked, 2 no-sources, 7 unrecorded, 0 broken, 0 malformed.**

## Checks run at the head

- These ran with `local/mdbook-0.5/bin` first on `PATH` (mdbook v0.5.4, mdbook-mermaid 0.17.1, the versions the workflow pins):
  - `book-stage` staged 105 pages and 70 figure files. The staged `research/lines/m5-energy.md` opens with the checked banner, and no staged page still holds a record line.
  - `mdbook-mermaid install .` passed, and changed no tracked file.
  - `mdbook build` exited 0 with 0 `ERROR`. Its one warning is the known mermaid version warning (built against 0.5.0, called from 0.5.4).
  - `xtask book-check` reported 107 pages and 0 broken links.
- `xtask roadmap check` reported 146 items OK, and `roadmap render --check` reported the roadmap README current, after the item edit.
- `uor-harness loop report --check` and `loop pr-check --local` both pass. The pr-check `filemap` advisory names five missing rows, all outside this diff.
- **Two fresh reviews.** Both reviewers were read-only and saw only the item, the ORDER and the diff. Each hit its context budget after part of the diff. Between them, and with my own checks of the groups neither reached, every changed claim was checked:
  - **Review 1, at `86f30f3b5`: APPROVE WITH CHANGES, no blocker.** It confirmed that no forbidden path was touched, that the record lines are well formed, the catalog counts, book-fresh 0 stale and 70 unchecked, the 749 lines and the archived `cargo run --bin r4`.
    - SHOULD: the `ldmatrix`/`stmatrix`/`movmatrix` audit tokens were unconfirmed. They are at `crates/uor-r4-discrete/build.rs:72-74`, with self-tests at lines 127-129.
    - SHOULD: m5-energy's export claim was unverified. `m4-serving.md` states both exports (lines 4, 13 and 63).
    - NIT: the slash-command count was unconfirmed. Reading `COMMAND_DEFS` in `chat.rs` showed 13 commands, including `/exit`, and completion filters on that table. The page now lists all 13.
  - **Review 2, at `e9cee8ef1`: REQUEST CHANGES, on one point.**
    - SHOULD: the banner block dropped the two shortcut lines `chat.rs:2471-2478` still prints. Both are back.
    - It also confirmed the `native-vector` module, its two entries, the 10 modules and 44 entries, every line count in the other-CUDA table, and the `client` subcommand and banner lines.
    - Its two NITs needed no change: the ASCII art is not reproduced, and the prose already says it is there; 9,762 counts the kernel files only, as the row says.
  - **Checked by me, outside both reviews:**
    - The seven new web-client exports are in `crates/uor-r4-integer/src/wasm_bindgen_exports.rs`.
    - `bake-off/` holds the three rounds and `reply-control-v3.json`, and `rules/` holds the two hook scripts.
    - `src-tauri` is in the root `exclude`, and `STATUS.md` says it is retired.
    - `projects/` has 8 folders, and `uor-cuda-inverted-file` came in `5c0317f15` (#3476), with a `host-test` target.
    - `projects/msa` holds `README.md` and `code/README.md`.
    - `r4g1-graph.md` has "Decided on 19 October 2026".
    - The tokenizer test is at `training.rs:242`, from `ce7782a74` (#2872).

## Skills used

None applied. This is a documentation-only unit, so the native-space-cycle skill does not apply.

## Still open

1. **70 unchecked pages**, none read in this unit:
   - `docs/`: `CONFIGURATION.md`, `README.md`, `RELEASE_PIPELINE.md`, `formal_vocabulary.md`, `reproducibility.md`.
   - Geometry: all 8 `docs/geometry/` pages.
   - Mechanisms: 24 `docs/mechanisms/*.md` pages.
   - Model: `architecture`, `data-and-tokenizer`, `serving-and-export` and `training`.
   - Products: `README`, `desktop-client`, `language-model` and `releases-and-artifacts`.
   - Reference: `README` and `storage`.
   - Research: `README`, `findings`, `history` and `open-questions`.
   - Rules: `README` and `decision-map`.
   - Start: `faq`, `glossary`, `what-is-uor-r4`, `where-we-are` and `where-were-going`.
   - Projects: `projects/README.md` (fixed, see above), and the `ai-research`, `prime-analysis`, `spiralcore` and `wasm-gemm-gnaf` READMEs.
   - Notebooks: 7 notebook pages (`docs/research/notebook/README.md`, plus `cpu-training`, `data-tokenizer`, `m1-discrete-attention`, `m1-prime-zeta`, `m3-durable-memory` and `m4-serving`). These are dated records, so their text is never rewritten. A check there could only confirm the page; it could never correct one.
2. **Tooling: a record-line edit makes hub pages stale.** `book-fresh` could skip a commit whose only change to a source is that page's first-line record, so checking one page under a glob stops making the hub page stale. That is an `xtask` change, outside this unit's stop list.
3. **The mermaid version warning** (the item's done-when 5) is unchanged.

## Plain summary

The documentation website has about a hundred pages. Each one records which files it describes and when someone last compared the two. Before this unit, only two pages had been compared. I read 23 more pages line by line against the files and records they describe. I fixed each claim those files contradicted: wrong record counts, a milestone page that said no model had been exported when two have been, a question the owner had already decided, and a tokenizer that had in fact been built. Then I marked each page as checked. 70 pages are still waiting for their first read; they are listed above.
