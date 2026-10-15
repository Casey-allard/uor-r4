# #63 — wasm32 build broken: runtime.rs imports cfg-gated compiler module

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T19:46:26Z
- **Updated:** 2026-07-22T20:09:31Z
- **Closed:** 2026-07-22T20:08:14Z
- **Labels:** bug
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/63
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Found during #12 work (pre-existing; CI has no wasm job since D6 demoted wasm to optional). cargo check -p uor-r4-core --target wasm32-unknown-unknown fails: transformerless/runtime.rs:21-22 unconditionally imports super::compiler, which is #[cfg(not(target_arch = "wasm32"))]-gated in transformerless/mod.rs:159. Gate the import or the compiler-dependent runtime items consistently.

## Comments (1)

### comment 5050937243 — 2026-07-22T20:09:31Z — auser

Fixed in 0d33644: compiler/teacher/scenarios modules are now portable with per-item fs gating on exactly the fs-dependent functions (corpus load/generate, artifact save/load, HF safetensors loaders, tokenizer loaders, read_file); progress and transitions ungated (portable). Verified with the pipeline's own command: wasm-pack build --target web succeeds. Native gates unchanged. The deploy workflow should go green on this push. Note: no wasm CI job was added (D6 demotes wasm) — re-add wasm-pack to ci.yml if the dashboard becomes a primary target again.


---
_Comment/review ids in this file: 5050937243_

