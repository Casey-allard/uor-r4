# REPORT — public research guide and issue reconciliation

Lab: codex. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md). References #3380.

## What did not work

The initial November 17 Studio check at `90dea3e24` found only the synthetic fixture. A model-only URL import verified the M1 stack while leaving its text tokenizer unbound. The public guide now distinguishes artifact availability, deployment and accepted-model parity. PR #4066 subsequently published the paired loader. Local mdBook 0.4 produced a different page count from the publication toolchain. The pinned Mermaid plugin still warns about mdBook 0.5.0 versus 0.5.4; #3972 tracks that warning.

## What changed

The README and book entry points lead with the research question, the M1 weights and measured baselines, the D87 comparison and the separate LexLean formalization. The editorial illustration is AI-generated and depicts a physical geometric model. The M1 numbers and diagrams retain their original scope. The cross-window and vector-read descriptions now link the merged outcome.

Reviewed all 52 issues open when the unit started. The roadmap now registers previously standalone defects against their existing IDs, preserves the pilot chronology, and identifies the frozen-reference experiment (#4063) and shared interfaces (#4064). The parent, seven milestone bodies and compute board preserve the earlier descriptions as dated history. #3801 closed on delivered instrumentation. #3876 closed after the contributor repair passed on merged main. #2846, #3450, #3462 and #3594 closed as not planned, with their historical evidence and revival conditions retained. The M1 milestone closed after its final untriggered fallback was moved. Source, render and live tracker have distinct verification steps.

## Measurements

The production Studio check on November 17 used main `e19cf2188` and Pages `2878cdc02`. The catalog listed `synthetic-seed-5` and `m1-discrete-trunk-seed-1`. Chrome made zero off-origin requests before the explicit model load, verified the bound weights and tokenizer, and returned the native IDs 223, 952, 2359, 2471, 23, 263, 964, 1520 offline for `The capital of France is`. The CLI with `--max-tokens 1` returned `[223]`. [Published catalog](https://github.com/UOR-Foundation/uor-r4/blob/2878cdc029c1d232e7468e4751a610cc91fc8ab0/studio/models.json) · [Reproduction](../../../../products/web-client.md#run-the-seed-1-example)

The issue census counted 52 originally open issues and found 156 source-backed issue mappings after registration. The historical pilot text contains 842 lines and 56,170 preserved bytes.

## Checks run at the head

- 156 roadmap items validated and mapped. After the #2850 decision, the roadmap rendered 32 ready items, 11 held and 113 done. The final prepublication sync preview at this stage proposed 48 existing issue edits, changes to the M1 and M2 child checklists, zero new issues and zero closures. The scoped #2850 preview removed `owner-decision`. The 77 Markdown table headers checked on changed pages matched their delimiter rows.
- Pinned mdBook 0.5.4 and mdbook-mermaid 0.17.1 built the book. The 241 xtask tests passed. The built link check counted 107 HTML pages and zero broken links. The claim-wording gate passed.
- Compared the preserved CPU pilot text with its source, checked the new local links, and ran the branch diff whitespace check. No research execution or scientific bar changed in this unit.

## Skills used

The parallel audit skill separated issue reconciliation, PR review and live Studio checks. The image-generation skill supplied the editorial illustration.

## Still open

Publication uses the protected branch, followed by the existing docs-site workflow and a live URL check. Milestone verification remains with the original registered experiments.

## Plain summary

The research guide now gives a reproducible M1 example and a dated account of the next experiment. The roadmap records what was delivered, superseded or still requires a measurement.
