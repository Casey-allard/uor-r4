# ORDER: the public status pages brought to 14 Nov 2026, claude-382

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Base: `d5eb4a55656726b04ea79b06968f91905933c925`.

On 14 Nov 2026 Casey found that the live pages still show stale status (for example "0 of 8 milestones are complete" on `model/gpu-and-cuda.html`), and asked for the live pages and the Studio to be up to date. claude's main session traced the stale live book to `docs-site.yml`: it publishes to `gh-pages-preview` on every push, and only a manual dispatch with `target=gh-pages` updates the live site, last done on 20 Oct. A live publish from `d5eb4a556` was dispatched at 14 Nov ~23:31Z (run 43653653317).

The source pages are partly current: `README.md` says 1 of 8 and links the 14 Nov mechanism board (claude-375, PR #3969), but `docs/README.md`, the start pages and several book pages stop at claude-232 (about 1 Nov) and name its parity miss as the current gate. This unit brings them to 14 Nov 2026.

## Goal

1. **A source-linked facts sheet first**, `FACTS.md` in this folder: every number the changed pages state, each with the merged REPORT, notebook row or record it comes from, from the merged work since 31 Oct. Pages are edited only from it.
2. **The forward-facing pages:** `README.md`, `docs/README.md`, `docs/start/where-we-are.md`, `where-were-going.md`, `what-is-uor-r4.md`, `faq.md`, `ROADMAP.md`, `docs/products/web-client.md` (its status header) and the Studio's visible status text in `web/studio/index.html`.
3. **Other book pages whose current-status lines are stale:** `docs/products/README.md`, `docs/products/language-model.md`, `docs/model/architecture.md`, `docs/model/serving-and-export.md`, `docs/research/findings.md`, `docs/research/open-questions.md`, and `docs/research/history.md`. Only lines that state the current state change; dated rows stay.
4. **Freshness records:** each page edited here that has a freshness record gets `checked=` set to the commit it was read against, so `book-fresh` reports it fresh.

Claims are scoped (CHARTER §5.6): a number names its unit, seeds and panel. M1 is complete; no model holds a useful conversation. V1 (PR #3979) and V2(a) (PR #3978) are described as in progress only.

## Scope

- the pages named in goals 2 and 3
- `docs/loop/tasks/docs-status-refresh/claude-382/ORDER.md`, `FACTS.md` and `REPORT.md`
- `docs/roadmap/items/docs-status-refresh.md`, only to add this unit's task folder to the item's scope

No other file may change. Not changed here: owner paths (`AGENTS.md`, `CHARTER.md`, `docs/integration/DECISIONS.md`, `.github/**`); a stale one is listed in the REPORT for the owner. Not rewritten: dated records (REPORTs, notebook rows, `docs/labs/**`, `archive/`). Not hand-edited: rendered views (`STATE.md`, `HANDOFF.md`, `docs/roadmap/README.md`). Not touched: `docs/model/pipeline-map.md` and `docs/mechanisms/**`, refreshed today by claude-375; PLAN.md; the files of V1 (#3979) and V2(a) (#3978). No status, bar, scope or stop rule changes. No code, no model run, no GPU.

## Cancel probe

GO at 14 Nov 2026 ~23:50Z.
- `origin/main` is `d5eb4a556`, this base.
- Open PRs #3979, #3978, #3977, #3941, #3937 and #3889 touch none of the scoped pages. #3978, #3937 and #3889 touch notebook pages; #3977 and #3941 touch `docs/start/FILEMAP.md` and Studio source outside `web/studio/index.html`. None overlaps.
- `uor-harness loop claim` refuses this item ("scope glob `docs/harness/README.md` covers the shared file"), as it did for claude-375; the worktree, ORDER and draft PR are made by hand, as claude-375's were.

## Done when

- The diff is docs-only plus the Studio status text, inside the scope above.
- Every number in the changed lines matches `FACTS.md`, and every `FACTS.md` row names its source.
- `cargo run -q --locked -p xtask -- book-stage`, `mdbook build` with 0 errors, `cargo run -q --locked -p xtask -- book-check` with 0 broken links, the docs link check, `cargo run -q --locked -p uor-r4-claim-wording -- .`, `uor-harness check` and the Studio `verify_*.mjs` status checks pass.
- A fresh-context exact-head review (docs-only, so a same-lab record is allowed) approves; `uor-harness loop pr-check --pr N` passes.
- The PR is queued with `uor-harness loop queue --pr N` and checked with `uor-harness loop close --verify --pr N` from the main checkout.
- After the merge: the live book is republished with `gh workflow run docs-site.yml --ref main -f target=gh-pages`, and the Studio with `scripts/studio/build-and-publish.sh --typed --target gh-pages` (the 9 Nov options), from a clean worktree at `origin/main`, after a dry run whose staged tree is compared with the live `studio/`. The live root, a start page and `/studio/` are read with `curl` and their status lines quoted.

## Stop and report

Stop after the pages, the review, the merge, the two publishes and the live read-back. Do not write the V0–V5 vector roadmap into any page as a plan; it is named only as the work in progress.
