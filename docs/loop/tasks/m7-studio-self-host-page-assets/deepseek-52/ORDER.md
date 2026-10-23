# ORDER: m7-studio-self-host-page-assets, deepseek-52 (stack-bound tokenizer artifact)

Lab: deepseek. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md) (#3075, residual of #2868). Base: `74b5a9538`. `References #3075`.

## Goal

Replace the worker's unbound byte-mod tokenizer label/mapping with a **stack-bound tokenizer artifact**: a build-time tool generates `pkg/tokenizer.json` from the shipped `UORLUT01` stack header (stack sha256, vocabulary size, byte→token table, mode `byte-mod`), the worker loads it when present and labels the capability truthfully (stack digest + vocabulary), and an absent/unmatched artifact falls back to the same mapping but labelled explicitly unbound. This is the tokenizer part of the residual item; a real corpus tokenizer still requires a real trained model (M4 export). No Rust change (external crate is out of item scope), no Pages change (D36), no bar.

## Cancel probe (run first)

Class instrument/build (host-only). At the base: `grep -n 'rawBytes\[i\] % vocabSize' web/studio/assets/js/uor_model_worker.js` shows the unbound mapping and `grep -n 'stack-bound' web/` finds nothing; no `pkg/tokenizer.json` generator exists. Task stands.

## Files in scope

- `web/**` (worker, helper, tool, verification, README), `docs/products/**`, `docs/labs/m7-studio-self-host-page-assets/**`
- this task folder

## Done when

- `web/studio/tools/make_tokenizer.mjs` reads the UORLUT01 header (magic, LE u64 header length, JSON `schema`/`shape.vocab`), writes `pkg/tokenizer.json` with the stack sha256, vocabulary, mode and 256-entry byte→id table, and supports `--check` against a shipped stack.
- The worker loads the artifact by same-origin dynamic JSON import, validates sha/vocab/table, labels it `stack-bound byte-mod tokenizer (vocab N, stack sha256 …)`, and keeps an explicit `no bound artifact` fallback label; the prompt tokenization path uses the artifact's table.
- `verify_studio_worker.mjs` generates the artifact from the stack it already reads, writes it beside the bundle, asserts the live worker reports the stack-bound label, and unit-asserts the fallback label plus table encoding; the full verification passes.
- No Rust, Pages-deploy, bar or owner-path change; item stays `ready`; counted non-author review then queue/verify/cleanup.

## Stop and ask if

- The artifact would require a Rust/`mk-synthetic-stack` change (outside this item's scope), a new Python script (banned), a real tokenizer corpus file not in the repo, or a Pages/deploy change.
