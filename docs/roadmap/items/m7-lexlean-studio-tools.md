---
id: m7-lexlean-studio-tools
title: Repair Studio model identity and share exact geometric arithmetic
milestone: M7
area: serving
status: ready
open_to: lab
needs_gpu: false
notebook: docs/research/notebook/m4-serving.md
depends_on: [m7-studio-live-model-sources, infra-desktop-client-tauri]
scope: [crates/uor-r4-integer/src/m6_exact_arithmetic.rs, crates/uor-r4-integer/src/wasm_bindgen_exports.rs, src-tauri/**, web/studio/**, docs/products/**, docs/loop/tasks/m7-lexlean-studio-tools/**, docs/superpowers/plans/2026-11-05-lexlean-studio-tools.md, docs/research/notebook/m4-serving.md]
done_class: host
---

## Why it matters

The owner asked to inspect LexLean-R4's repository and hosted model, use it as the Studio default if possible, and port its useful behavior to the web and Tauri clients. The live full-pack browser probe answered `17 + 25` correctly through its exact-arithmetic path, but returned an unrelated Hunger Games review for self-description and a Fibonacci example instead of recalling `42`. The deployed runtime is an older `/8` layout with different memory keys and table-form squashing, so it is not a compatible or charter-lawful drop-in default. UOR already owns the geometric arithmetic engine used for the useful path, but does not expose it as a distinct tool result. The Studio also leaves a mismatched worker identity silently stuck in `finalizing`.

## What is missing

- A truthful, shared way to answer bounded exact arithmetic without sending the prompt through an unhelpful synthetic model or attributing the result to learned generation.
- An explicit terminal error when the worker reports an artifact identity different from the requested file or expected hash.
- Regressions proving that a catalog's presentation metadata cannot silently replace the user's active model, plus real browser and native evidence for both tool and ordinary paths.

## Done when

1. A strict, bounded decimal addition/subtraction prompt returns an exact result and actual geometric place-value trace through the existing Rust engine, with the answer first and an explicit not-model-generation label. Ordinary prompts fall through unchanged; unsupported input fails loudly.
2. The same shared function serves the browser worker and the packaged Tauri transport. Browser and native tool results report zero model completion tokens rather than inventing generation metrics; normal model streaming remains unchanged.
3. A worker `model_loaded` filename or expected-SHA mismatch produces an explicit load error instead of remaining pending. The no-storage default remains the bundled synthetic demo and a catalog `primary` flag alone does not select a model.
4. Real browser and packaged-app checks exercise an arithmetic response and an ordinary synthetic response. Existing browser, Rust, native, runtime and delivery checks pass, then the protected PR is merged, fresh main is verified and Pages is published with the existing five-card catalog preserved.
5. ORDER/REPORT, the product pages and the notebook record the LexLean source identities, exact negative scope, tests and remaining absence of useful general conversation. No external WASM, second model format, new dependency, committed binary/corpus, paid compute, new default model, charter change or useful-chat claim is introduced.

## Cancel if

The shared engine cannot produce the same exact result in both clients, the native tool cannot return an OpenAI-compatible fallback without corrupting normal streaming, or the repair requires a second model runtime, a banned mechanism or a new dependency. Stop and re-scope rather than hiding those constraints.

## How to start

Read `docs/loop/tasks/m7-lexlean-studio-tools/kimi-1/ORDER.md`, `crates/uor-r4-integer/src/m6_exact_arithmetic.rs`, the Studio worker/source adapter and desktop transport. Add failing regressions first, then implement the shared Rust tool and two thin transport adapters. Do not import LexLean's heuristic corpus, sigmoid tables, WASM, stale labels or cosmetic streaming.

## Rules that apply

- CHARTER §3.1–3.4 and §3.6: reuse only the existing geometric integer engine; no second runtime, squashing table, dense matrix product or external WASM enters the served path.
- CHARTER §3.11–3.13 and §4: Rust first, no new dependency or committed binary/corpus, exact artifact identity, finite positive and negative controls, and no capability claim beyond the measured deterministic tool.
- CHARTER §5.5: ORDER/REPORT, fresh exact-head review, protected merge queue, fresh-main verification, Pages publication and cleanup. `References` the new item until every done-when row is satisfied.
- Live Pages publication follows the owner's existing live-deploy decision recorded in [m7-studio-live-model-sources](m7-studio-live-model-sources.md); this registration performs no publication.

## Sources

- [LexLean-R4 repository](https://github.com/Casey-allard/lexlean-r4) at `23436bdcd69e5238fcda51ef2529bbd4a5dc1e73`; deployed tree `563a7549e432fc035bc603504bb3a162eb332212` and [hosted page](https://casey-allard.github.io/lexlean-r4/).
- [Issue #3704](https://github.com/UOR-Foundation/uor-r4/issues/3704), which already carries the `m7-lexlean-studio-tools` roadmap marker. No duplicate issue or generated issue mapping is introduced by this registration.
- [CHARTER.md](../../../CHARTER.md), [the desktop product contract](../../products/desktop-client.md), [the web product contract](../../products/web-client.md), and [the existing geometric arithmetic](../../../crates/uor-r4-integer/src/m6_exact_arithmetic.rs).
- [m7-studio-live-model-sources](m7-studio-live-model-sources.md) and [infra-desktop-client-tauri](infra-desktop-client-tauri.md), whose existing browser/native shell work this unit preserves. The desktop item's legacy status/mapping mismatch is outside this registration.
