# REPORT: the public status pages brought to 14 Nov 2026, claude-382

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Order: [ORDER.md](./ORDER.md). Facts: [FACTS.md](./FACTS.md). Base: `d5eb4a55656726b04ea79b06968f91905933c925`.

Every claim below is [claude]'s.

## What did not work, or is not done

- **`uor-harness loop claim` refused the item**, as it did for claude-375: the item's scope lists the shared file `docs/harness/README.md`. The worktree, ORDER and draft PR were made by hand.
- **The bench inbox was absent.** `local/bench/INBOX.md` does not exist in the main checkout, so there were no waiting messages to act on.
- **Two pages outside this scope are stale.** `docs/model/gpu-and-cuda.md` is stale because #3977 changed its crate sources; its live copy no longer carries "0 of 8". `docs/research/lines/m7-distribution.md` lists `ROADMAP.md` as its only source, and this unit edits ROADMAP. Its text was read against the new ROADMAP and is still correct ("Pre-alpha: 1 of 8 milestones is complete"; public M1 top-8 prediction in the Studio). It is outside this unit's scope, so its `checked=` was not moved. The same holds for any page this unit edits and whose sources include another page this unit edits: after the squash, `book-fresh` counts the merge commit as a later change. Any `checked=` on a branch commit dies with the squash, so moving it here cannot help.
- **The crate READMEs were left as they are.** They are dated "Status through 31 Oct" and are sources of `docs/README.md`. Editing them would have made the hub stale after the merge. They were removed from the ORDER's scope before any edit.
- **Not refreshed here:** `docs/model/pipeline-map.md` (its line "order now in force (31 Oct)") and `docs/mechanisms/**` (`lut-serving.md`, `admission.md`) belong to claude-375's refresh and the ORDER excludes them. PLAN.md is not touched; its 30 Oct owner-direction rows read as of that date.
- **The live Studio is not changed by this PR.** `web/studio/index.html` already reads "Pre-alpha · M1 complete · 1 of 8", and so does the live `/studio/` (the 9 Nov publish, gh-pages `4a435ab8a`). No edit was needed. The Studio republish after the merge carries #3915's task workspace, which so far exists in source only.
- **A slip was fixed before review.** The facts sheet's first wording of F4 named "claude-358's served speed" as if it held for all three models. It holds for p-s2, retrained. The sheet and every page now say "one of them, p-s2".
- **The fresh-context review found three low, wording-only slips, now fixed.** First, the web-client page stated the live Studio's status text with no row behind it; FACTS P6 now records a `curl` of the live `/studio/`. Second, the pages called claude-366's integer-served E8 result "inconclusive", where its own verdict is "Not called" (M = +5.3); the sheet and this unit's pages now say "not called". `docs/model/pipeline-map.md` line 128 still says "inconclusive"; that page is outside this unit's scope (claude-375's refresh). Third, three lines said "V2" where the work in progress is V2(a). A second fresh review of the fix delta approved it, and its wording nits (the open-questions mermaid node, the codex-8 sentence's scope) are applied.
- **One fact landed after the base.** #3977 (codex-8, merged 14 Nov 23:54Z) serves the trained Q31 state in the integer crate. It is added as FACTS N3 and one sentence on `where-we-are.md`, after merging `origin/main` into the branch.

## What changed

- **`FACTS.md` (new):** 4 status rows (S1–S4), 10 model-line rows (F1–F10), 2 native-space rows (N1–N2) and 5 product rows (P1–P5). Each row gives its kind (measurement or record) and the merged REPORT, notebook row, PR or decision it comes from. No number on a changed page is missing from the sheet.
- **Front pages:**
  - `docs/README.md`: the status paragraph now reads "M1 closed 30 Oct; current research through 14 Nov 2026" and covers claude-358's recall, V0, D85 and codex-5, with V1 and V2(a) as in progress.
  - `docs/start/where-we-are.md`: a new section on the FBS trunk since 31 Oct (human text, recall, V0, a table of the three additions, the native-space route, work in progress), a "facts from the vectors alone" bullet, and a "Where it stands" table with recall, V0 and work-in-progress rows. The claude-232 parity miss is no longer named as the current gate.
  - `docs/start/where-were-going.md`: "The current order of work" now gives D85's four steps from PLAN, the V0 gap and V1/V2(a). "The biggest unknown" and the table follow.
  - `docs/start/faq.md`: answer 5 is rewritten (claude-358, V0, D85), answer 4 is reworded, and answer 16 gains p-s2's speed.
  - `docs/start/what-is-uor-r4.md`: a "Since 31 Oct" paragraph and the current-research table row.
  - `ROADMAP.md`: the caption, and a "Since 31 Oct (14 Nov 2026)" paragraph after "M2 is the next capability milestone".
  - `README.md`: the history link says "June to November 2026".
  - `docs/products/web-client.md`: a dated 14 Nov status line. The live Studio was still the 9 Nov publish, and #3915 was in source only.
- **Other book pages:**
  - `docs/products/README.md`, `language-model.md`, `docs/model/architecture.md` and `serving-and-export.md` each gain a dated "State on 14 Nov" or "Since 10 Nov" paragraph. `serving-and-export.md` also gains the pair-row refusal and a D85 bullet. The older claude-232 text stays, as history.
  - `docs/research/findings.md`: seven new rows (human text, E8 address memory, recall, pair experts, pair rows, V0, the Q31 contract test).
  - `docs/research/history.md`: "Phase H: replies in conversation and one native state (1 to 14 November)".
  - `docs/research/open-questions.md`: a state paragraph and a mermaid node for V0.
- **Freshness records:** each edited page with a record has `checked=` set to the commit it was read against: the base `d5eb4a556`, or for the five pages whose sources include `crates/uor-r4-integer` or `crates/uor-r4-discrete` (`docs/README.md`, `language-model.md`, `web-client.md`, `architecture.md`, `serving-and-export.md`), `abfad0604`, after they were re-read against #3977; `serving-and-export.md` gains one codex-8 sentence.
- **`docs/roadmap/items/docs-status-refresh.md`:** this unit's task folder added to the scope, nothing else.

## Measurements

None. This is a documentation unit with no model run, no GPU and no code. Every number it writes is copied from a row of [FACTS.md](./FACTS.md), and each row names its merged source.

Live read before this PR, at 14 Nov ~23:59Z, after run 43653653317 (`docs-site.yml`, target gh-pages, head `d5eb4a556`, conclusion success, 14 Nov 23:54Z):
- `/` shows "1 of 8 milestones complete";
- `/start/where-we-are.html` shows "1 of 8 milestones is complete";
- `/mechanisms/index.html` shows the 14 Nov "Results since 31 Oct with no board row yet";
- `/model/gpu-and-cuda.html` (last-modified 14 Nov 23:58:51 GMT) no longer carries "0 of 8".

## Checks run at the head

- `cargo run -q --locked -p xtask -- book-stage`: 105 pages and 70 figure files staged.
- `mdbook build` (mdBook 0.5.4 with mdbook-mermaid 0.17.1, the versions in `docs-site.yml`): exit 0, 0 lines with ERROR.
- `cargo run -q --locked -p xtask -- book-check`: 107 pages, 0 broken links.
- `cargo run -q --locked -p xtask -- links`: 7,540 sources, 1,288 broken repo-wide. None has its source in a file this unit changed; I filtered `links --json` by source path, and all 18 changed files show 0.
- `cargo run -q --locked -p xtask -- book-fresh`: every page this unit edited is fresh. One page is stale (`docs/research/lines/m7-distribution.md`, above).
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness check`: 0 errors. All warnings are branch hygiene unrelated to this diff.
- Studio:
  - `verify_studio_page.mjs`: failures [];
  - `verify_studio_worker.mjs`: "ALL STUDIO WORKER CHECKS PASSED", status chip "pre-alpha / M1 complete / 1 of 8 milestones";
  - `verify_m7_parity.mjs` and `verify_studio_migration.mjs`: exit 0.
- Fresh-context review (a `delegate:reviewer` given only the ORDER, the FACTS sheet and the diff) at `c2ac364e2`: APPROVE on all three gates (numbers, scope, claims), with three low wording findings, fixed above. Each FACTS row F1–F8, N1, S2 and P1 was checked against its source. The review record and `uor-harness loop pr-check` are on the PR at the final head.
- Studio rehearsal: `build-and-publish.sh --typed --target gh-pages` (dry run) from a clean worktree at `origin/main` `abfad0604`. Every verifier passed, including the Chrome page-load trap. Against the live `studio/`, `models.json`, `pkg/synthetic-stack.lut` and `pkg/tokenizer.json` are byte-identical (same git blobs). The engine bundle is 901,727 bytes against 807,378 live, and `index.html` changes only two hashed asset names. So the 9 Nov options carry the same catalog.

## Skills used

None.

## Still open

- **Owner paths.** None is stale on status. `AGENTS.md` says "1 of 8 … M1 closed on 30 October 2026 … no model holds a useful conversation". `CHARTER.md` says "Pre-alpha: 1 of 8 milestones complete … no model holds a useful conversation yet". Neither names claude-232's parity miss as the current gate. Nothing is listed for the owner.
- **After the merge** (this unit's ORDER): republish the live book from `main`, then the Studio with `scripts/studio/build-and-publish.sh --typed --target gh-pages` from a clean worktree at `origin/main`, after a dry run whose staged tree is compared with the live `studio/`. The live root, a start page and `/studio/` are read back, and the result goes in a closing comment on the PR.
- `docs/research/lines/m7-distribution.md` needs its `checked=` moved after the merge, and so does any other page whose source this unit changed.
- `book-fresh` could stop counting a page's own record commit and a merge that only touches a source's freshness line. That is a tooling change for the harness line, not this unit.
- V1 (#3979) and V2(a) (#3978) are described as in progress only. Their results go onto these pages when they merge.

## Plain summary

The project's public pages had stopped at about 1 Nov. They named the claude-232 parity miss as the current gate, and the live book had not been published since 20 Oct. Now the source pages say what is true on 14 Nov:
- M1 is complete (1 of 8) and no model holds a useful conversation.
- The current models recall a told value on 314 to 325 of 578 questions.
- Without their exact turn memory, they give 171 to 211 exact replies of 2,218, against 377 to 441 with it, so the facts come from that table and not the learned state.
- The route is now one native integer state for training and serving (D85).
- V1 and V2(a) are in progress.

Every number links its source on the facts sheet. The live book was republished from the base before this PR, and is republished again, with the Studio, after the merge.
