# ORDER: one-model-end-to-end, deepseek-40 (fresh-panel opening transfer)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `6a5d51172`.

## Goal

Measure the existing opening point-key mechanism on the actual
`m2-fresh256` panel, using only reply-SFT TRAIN corpus rows to build the point
table and using the sealed panel labels only as outcomes. Emit the treatment
and shuffled-target control artifacts so the unchanged `served-reply-panel`
can score complete replies from the same model path.

Claude's open claude-321 branch owns the main `one-model-end-to-end` run and
does not touch `cpu-point-transfer-driver`. This unit is a read-only support
diagnostic for that track and does not train, alter a panel, or start a pod.

## Cancel probe

- `cpu-point-transfer-driver` already derives raw point, target-medoid and
  target-prototype tables on a UORT corpus and checks served parity.
- `served-reply-panel` already renders `m2-fresh256` under both registered
  renderings and reports first-target ranks.
- The missing join is panel-case rendering plus first-target point selection.
  No open PR or claim adds that join.
- The sealed SFT corpus, selected integer artifact and all four
  `m2-fresh256` files are locally present at pinned hashes.

## Design

Add an opt-in panel mode to `cpu-point-transfer-driver`:

- Keep the existing UORT source-group mode unchanged when no panel flag is
  supplied.
- Require `--panel-inputs`, `--panel-labels`, `--panel-plan`,
  `--panel-manifest`, `--panel-tokenizer`, `--rendering`, and expected hashes.
- Render panel cases exactly as `served-reply-panel` does for
  `one-user-turn/v1` or `per-turn/v1`.
- Parse each panel label's shortest accepted answer and encode its first
  scored token with the same literal-role dialogue protocol. Labels are used
  only after the point table is complete.
- Build the treatment point table from every eligible TRAIN conversation
  opening, deterministically capped by `--train-examples` if supplied. A
  training reply whose first token is EOS is skipped.
- Compare raw model, treatment point, shuffled-target point, and a global
  training-mode first-token baseline. Report top-1, top-8, top-32 and paired
  two-sigma comparisons overall and per panel stratum.
- Verify every treatment query is byte-identical to the raw-model query and
  every treatment/shuffle selection equals the artifact's served token.
- Write `report.json`, `point.uorlut`, and `shuffled-point.uorlut` to a new
  output directory. The two artifacts must be directly consumable by
  `served-reply-panel`.
- Refuse panel/training overlap by the registered corpus and panel identities;
  no panel label or answer text enters point-table construction.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-point-transfer-driver.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-40/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

## Done when

1. The extended driver builds offline and locked; its existing tests plus new
   tests for point top-k ordering, shuffled controls, first-token encoding and
   fail-closed panel metadata pass.
2. The selected `9f349697...` integer artifact and repaired 6,444,377-token
   reply-SFT corpus run under both registered renderings on all 256 panel rows.
3. The report records raw, point, shuffled and constant baselines, per-stratum
   top-1/top-8/top-32, paired treatment-vs-shuffled comparisons, model/query
   parity and exact artifact hashes.
4. `served-reply-panel` runs unchanged on each emitted treatment and control
   artifact; complete replies and pairs are reported beside the opening result.
5. REPORT, notebook row, report check, policy check, full gates and a fresh
   exact-head review pass; one PR merges.

## Stop and ask

This is a diagnostic screen, not a fresh capability bar. Do not train on panel
labels, tune against panel outcomes, substitute another panel, or claim useful
chat, M2 or GPU qualification. Do not alter Claude's open branch files.
