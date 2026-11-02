# REPORT: infra-loop-harness, deepseek-75 (record the M7 Studio live/model-sources item)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).

References #3440 (the recorded item).

## What did not work, first

- No Studio code is changed here; this is the records PR only.
- The first draft claimed the engine serves uor-r4.lut-stack/1..7 and that nothing loads a user-chosen model file. The fresh-context review corrected both: the engine parses /1-7 but serves /1-/6 (a /7 turn-memory stack is refused), and the source page already has a /1-only local sidecar loader plus a Model Manager. The item now states both accurately.
- The natural home for the work is an owner-path file (.github/workflows/docs-site.yml); a lab cannot change it. The item therefore records the build/publish helper under scripts/studio/ plus a workflow patch proposal for the owner, and says so.
- The owner's model choice was demo + loader + API options, so the item explicitly does not promise a public trained-model artifact and records that the private HF store is not published.
- The first item draft omitted the required "Rules that apply" and "Sources" sections; roadmap check refused it, and the two sections were added before the sync.
- The existing docs-site workflow preserves any studio/ directory on gh-pages, which is why a manually published studio survives future book deploys; the item records that as the interim publish path.

## What changed

- New item docs/roadmap/items/m7-studio-live-model-sources.md (M7, area serving, status ready, open_to lab, needs_gpu false, depends_on the two done M7 Studio items, notebook docs/research/notebook/m4-serving.md), with why/missing/done-when/cancel-if/how-to-start/rules/sources.
- docs/roadmap/issues.json (one line, m7-studio-live-model-sources -> 3440) and docs/roadmap/README.md are written only by roadmap sync --apply / roadmap render (never hand-edited).
- docs/loop/tasks/infra-loop-harness/deepseek-75/{ORDER,REPORT}.md.
- Size: 5 files (+150/-4 at this head): the item, the ORDER/REPORT, and the generated issues.json/README; code 0, tests 0.

## Measurements

No measurement. Roadmap counts at the head: 115 items OK; by status ready 18, held 10, decision 1, done 86; by milestone M7 4 (was 3). Sync plan: 0 labels, 1 issue to create (#3440), 0 updates, 0 closes, 1 milestone checklist (#2035). Applied: created #3440, updated #2035, wrote issues.json (115) and re-rendered.

## Checks run at the head

- cargo run -q --locked -p xtask -- roadmap check -> 115 items OK.
- cargo run -q --locked -p xtask -- roadmap render --check -> current.
- cargo run -q --locked -p xtask -- roadmap sync (dry) -> exactly one create and the #2035 checklist change.
- cargo run -q --locked -p xtask -- roadmap sync --apply -> created #3440, updated #2035, wrote the generated files.
- `cargo run -q --locked -p uor-r4-claim-wording -- .` and `git diff --check` passed; the local PR check passed.
- The records review also re-ran the source page's two page-load harnesses (verify_studio_page.mjs, verify_studio_page_load.mjs): both pass, external: []. That is context for the item, not a claim about a Studio change in this PR.
- The exact-head review record is posted at the head before this PR goes ready.

## Skills used

working-discipline, scope-guard, solution-integrity; the studio/deploy specialists' findings (Lead's team) for the honest feasibility statements.

## Still open

- The implementation unit follows after this item exists: cleaner web/studio page, model-source/API manager, working wasm bundle, scripts/studio build+publish, live gh-pages publish, docs update.
- The owner-only workflow patch (path filters + build steps) is proposed in the implementation REPORT, not applied.

## Plain summary

This records one M7 item for fixing the published Studio: cleaner page, real wasm, selectable local/URL/API model sources, and a live gh-pages publish from a script under scripts/studio/. No code is changed in this PR.
