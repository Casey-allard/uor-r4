# ORDER: m4-export-uorlut01-cli, kimi-31

Lab: kimi. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Base: `d0527ed8f`. `References #2859`; this host-only partial order does not close the item or release M4.

## Goal

The item's done-when reads:

> M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. In addition:
>
> - The export gap is reported (D37).
> - The static audits are green.
> - A fresh-process restore is bit-identical.

This order supplies the discrete CLI and finite host prerequisites; trained export, held-out scoring and GPU execution remain held.

## Cancel probe (run first)

At `d0527ed8f`, `d11_export::export_d11` already writes `UORLUT01` with schema `uor-r4.lut-discrete/1`, empty matrices and four integer sections; `IntegerDiscreteModel` loads it and evaluates prefixes. No new container or sections are needed. `uor-chat` has only bundle and geometric-stack paths and no discrete CLI or spawned-process discrete test. The task stands. Class: serving instrument, not a model-mechanism result.

## Files in scope

- `crates/uor-r4-integer/**`
- `crates/uor-r4-lut/**`
- `docs/mechanisms/lut-serving.md`
- `docs/model/serving-and-export.md`
- `docs/labs/m4-export-uorlut01-cli/**`

## Plan

1. Post the container/loading design note on #2859. Add `--discrete` with one-shot `--prompt-ids`: direct `IntegerDiscreteModel` greedy predictions, a bounded sliding context, no sampler, and no text REPL or session codec in this PR.
2. TDD a CLI integration test with a self-contained toy `UORLUT01` fixture, requiring no dependency or `Cargo.lock` change. Compare two fresh processes and direct library predictions; refuse a wrong schema and unsupported modes.
3. Record the existing native and WebAssembly audit baselines. Their named roots do not cover the new discrete CLI, so no machine-code compliance claim is made for that root.
4. Update the serving docs, draft the PENDING pre-registration with its missing trained artifact, held-out stream and lawful scorer named as blockers, then write REPORT and the host record. Defer the finite-control notebook row because this item has no `notebook:` metadata and the scope guard refuses it.

## Done when

- `uor-chat --discrete MODEL.lut --prompt-ids CSV` loads the existing schema and emits deterministic greedy ids matching `evaluate_prefix`; invalid tokens/contexts, wrong schemas, nonzero temperature and unsupported text/session modes fail closed.
- The integration test spawns the built CLI twice on one toy artifact, asserts identical nonempty `generated_ids`, and checks them against direct library output; its negative controls pass. `cargo test -p uor-r4-integer --features serving-wasm` is green.
- Both existing audits pass at the head, with their exact symbol scope and the uncovered CLI root recorded. The design note is posted and the serving docs match behavior.
- The draft sheet fixes the 0.90 BPB bar, export-gap method, separate prefill/decode rates, stop rule and cost, while leaving the unavailable checkpoint, frozen held-out chat stream identity and #2861 scorer explicitly PENDING. No FIT or release is claimed.
- REPORT, quick/full gates and local PR checks pass; the item stays `ready`.

## Stop and ask if

A trained checkpoint or GPU is needed; the container format or an audit script must change; text-chat semantics require a new decision; or any file outside scope is needed.

Size estimate: about 650 lines.
