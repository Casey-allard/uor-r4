# REPORT: correct final evidence citations, kimi-80

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `b9750be653d451691694853b62d4a98b256f48f3`, tree `77487ea04c695bc38526b1acbdd89b469cc22695`; merge base `0895aa65ff2f63ed184e78e81c28e67ffa3f34aa`. Agent-63 returned CHANGES only for record identities, citations and evidence durability.

## What did not work

- Current-state named only the earlier `d399` head and treated `3c7176d5f`-and-later as record-only without naming the actual source/assertion head `988155ae5` or distinguishing intermediate `b9750be6` from the final exact delivery identity. It also linked the pre-work ORDER rather than the binding REPORT.
- `c17105e7` policy/gate logs existed only in the worktree, and were cited instead of the durable b975 intermediate logs. Notebook counts lacked the correct `988155ae5` source/assertion attribution. Desktop/README links stopped at the older report.

## What changed

- Current-state, notebook, desktop/README and kimi-79's summary distinguish `d399` earlier acceptance and `988155ae56dc0fdd872b7428018ead566179fe3f` source/assertion (tree `e6542fb375518c1b38639cff825b9ac65f7b3594`) from record-only deltas. `c17105e7`, `b9750be6`/tree `77487ea04c695bc38526b1acbdd89b469cc22695`, and `3e33a175bcf928e1e42e7bbe8c7a149709095da3`/tree `fc3df19e7e556d5f3c43b385a0525d6184f96255` are intermediate record-only heads. Agent-64 identified the remaining contradictory labels; [kimi-81](../kimi-81/REPORT.md) corrects them. The final identity is in the PR body and main-checkout `local/kimi/studio-final-acceptance/<final-head>/identity.json`, not this report.
- c171's local-policy and full-gate logs were copied byte-identically into `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/c17105e704e1ee9b02271062a13fc23747649604/`. b975 policy/gate citations use its durable copies and explicitly identify it as intermediate evidence.
- This repair is documentation/evidence only: no production code, model/API/engine/worker/tokenizer or new feature claim was added.

## Measurements

- `988155ae5` complete default **11/3/21/7/13/8** and GUI **13/3/25/8/13/9** logs are retained under `local/kimi/studio-final-acceptance/988155ae56dc0fdd872b7428018ead566179fe3f/`, also copied to the main checkout.
- `b9750be653d451691694853b62d4a98b256f48f3` has retained `pr-check-local.log` (PASS with disclosed advisories), `full-gates.log` (**11/11 PASS**), and a clean loopback, all copied byte-identically to the main checkout. Host SHA-256 `430ac620f81ac73e67f9bf4aa63f70bff4f9f9add8285f24f390d1d7f92e9dd4`; dist SHA-256 `30358639f919fc55fb8fa75b6daf6df71e33e10ac9da7cb5b02452ce51fade55`.
- The d399 summary/runner are durable in the main checkout; the summary SHA-256 is `723ba3ee1e110db412525e8ce78a263512a33a9fda0b7c398e9b595416c44c17`.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/acceptance-tests" cargo test --manifest-path src-tauri/Cargo.toml`: complete default PASS; the same with `--features gui`: complete GUI PASS, with the `988155ae5` logs above.
- Mixed-case source regression: RED 200 → GREEN 404; regular-file explicit root refusal and valid canonical-directory behavior retained.
- Complete authorized MCP hanging JSON-RPC body and protocol/session headers asserted, expected disconnects tolerated, unexpected fixture panics fail. Original 15 ms/25 ms limits unchanged.
- Final `uor-harness loop pr-check --local` and `uor-harness loop gates --full` are run after this record-only commit and retained; independent review follows.

## Skills used

Fresh exact-head review, source/tree/merge-base binding, evidence preservation, documentation synchronization, and verification before completion. No implementation change or model claim was introduced.

## Still open

Fresh exact-head approval, required GitHub checks, protected merge and fresh-main verification. No production publication, signed release, full Cowork parity, useful-chat result, or fresh packaged-GUI acceptance is claimed.

## Plain summary

Agent-64 found that the attempted alignment still left contradictory labels in kimi-79 and an unsupported final-binding claim here. [kimi-81](../kimi-81/REPORT.md) corrects those findings. The final identity is in the PR body and main-checkout identity.json; no self-hash is claimed here. The separate raw mixed-case RED log was not retained.