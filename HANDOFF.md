<!-- rendered by uor-harness loop render from e4264acf167c8552639d6f2ead4d523c1867b038; do not edit -->
# HANDOFF

Rendered from `docs/roadmap/items/*.md` at the commit in the stamp above. Each open-item row gives the first sentence of the item's `Done when` and `Cancel if` sections; blocked items are listed once, in the blocked table, with their owner; `not recorded` means the item has no such section or key yet.

## Open items

Items not `held`, `decision` or `done`. Held and decision items are listed once, in the blocked table below.

### ready (32)

| Item | Title | Done when | Cancel if | Size | Stands |
|---|---|---|---|---|---|
| cpu-training-pilot | Run and grade the bounded CPU training pilot | - The pre-registration exists before compute and names pass, fail and stop outcomes. | Any dependency still fails parity or deterministic resume. | not recorded | ready; after cpu-discrete-training-path, cpu-shared-vector-train, cpu-quantization-exact-memory, cpu-whole-path-exact-updates |
| d5b-coaccess-assignment | Test D5-b address assignment with measured co-access and distance | Export the real co-access matrix, use the distance term in the objective, and record the comparison with the current assignment. | not recorded | not recorded | ready |
| discrete-clippy-k16-coverage | Track discrete Clippy drift, PR coverage and the K16 timing instrument | Resolve the registered lint, CI-coverage and K16 timing findings with the corresponding verification for each, preserving source-pin integrity. | not recorded | not recorded | ready |
| docs-site-freshness | Book page freshness records, a staged banner, and mdBook 0.5 | 1. | not recorded | not recorded | ready |
| docs-status-refresh | Bring every status page, picture and tracker up to 31 Oct 2026 | 1. | not recorded | not recorded | ready |
| end-to-end-integration | Connect training and integer serving behind shared stage interfaces | 1. | not recorded | not recorded | ready; after integer-hamming-read, fbs-serving-kernels, native-space-state-contract |
| float-kernel-fusion | One kernel list for the float crate, and fused kernels for the served quantizers | 1. | - The two chains' share of wall seconds per training step at P2b's recipe, with `--train-as-served 1`, is under 5%, with their launch counts reported alongside. | not recorded | ready |
| icosian-snap-parity | Build a lawful GPU comparator for the historical S1.4 icosian snap replay | A reviewed GPU comparator reproduces the named measurements on the pinned inputs under its own FIT-reviewed sheet. | not recorded | not recorded | ready |
| infra-loop-harness | Maintain the loop harness and shared roadmap registration | Every PR in the build plan on [the loop page](docs/loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. | not recorded | not recorded | ready |
| m2-multi-turn-bar-number | Register a number for the multi-turn reply bar blocked under R14 | A threshold is registered with deterministic graders (D28) and the R14 block is lifted for that check. | not recorded | not recorded | ready |
| m2-train-grounded-replies | Train the student for grounded replies from exact memory | The same native student must pass both graders. | not recorded | not recorded | ready; after m2-donor-corpus-generation, m2-fresh-heldout-panel, m2-pin-train-mask-sheet, m1-close-saved-model, m1-d11-exporter-discrete |
| m3-qualify-memory | Qualify durable memory on the selected native model | On the selected artifact: | not recorded | not recorded | ready; after m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests |
| m4-export-uorlut01-cli | Serve the chosen model as UORLUT01 through the uor-chat CLI | M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. | not recorded | not recorded | ready; after m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory |
| m5-laptop-full-path-measurement | Measure M1-class laptop cost on the accepted served model | M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". | not recorded | not recorded | ready; after m4-export-uorlut01-cli |
| m6-qualify-reasoning-coding | Qualify reasoning and coding on the served model | M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory. | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism |
| m7-parity-library-cli-browser | Build the library, CLI and browser bit-identity parity harness | M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. | not recorded | not recorded | ready; after m7-studio-connect-wasm-engine, m4-export-uorlut01-cli |
| m8-independent-verification | Independently verify the alpha on consumer hardware | A third party reproduces the install and the M1 to M7 bars from the checksum-pinned artifact: the serving-side bars on consumer hardware, and the panel-scored bars on a GPU as CHARTER §3.6 requires. | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser |
| m8-release-packaging | Prepare the checksummed alpha release package and install path | M8 bar ([`ROADMAP.md`](ROADMAP.md) line 21): "One versioned, checksum-pinned model file, declared alpha by the owner". | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser |
| native-space-state-contract | Define one persistent native training/serving state and its operator parity | 1. | The source inventory cannot identify one native transition that exists in both training and serving without inventing a new mechanism. | not recorded | ready |
| object-store-consumer-m3 | Wire the object store into the model and run the durable-memory bars | A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. | not recorded | not recorded | ready |
| one-model-end-to-end | Score one model end to end on prompts it has not seen, with tokens per second | 1. | not recorded | not recorded | ready; after turn-memory-reply |
| past-window-scorer-refusals | Refuse unreadable inputs to the past-window scorer | The scorer refuses the four malformed cases in the original report, with regression evidence. | not recorded | not recorded | ready |
| runpod-reaper-api | Preserve RunPod pod cleanup across the API migration | A fixture or test account confirms discovery, authorization, and idempotent termination for the reaper. | not recorded | not recorded | ready |
| scale-probe | Measure how ability grows with width and data | 1. | not recorded | not recorded | ready; after one-model-end-to-end |
| studio-ide-harness | Deliver the native-style Studio IDE harness and real connection workflows | - The green placeholder is replaced by a reviewed neutral Claude-style shell in both themes and at desktop/mobile sizes, with Playwright evidence for every view. | not recorded | not recorded | ready; after studio-typed-shell-upgrade |
| studio-lexlean-mvm-workbench | Import the LexLean MVM workbench and approved GitHub workflow | - A `uor-r4-mvm` crate compiles to native Rust and WASM with pinned LexLean provenance/license notices and no committed binary. | The pinned LexLean engine cannot pass its exact-output native/browser preflight without canned behavior, or the base already supplies the complete item. | not recorded | ready; after studio-cursor-hosted |
| studio-token-activity-trace | Expose a verified Studio token timeline and optional 3D activity view | Show events emitted by the worker with real token metadata in an accessible timeline, default to metadata-only export, and supply reduced-motion and 2D alternatives to the optional activity view. | not recorded | not recorded | ready |
| studio-typed-shell-upgrade | Migrate the Studio to a typed lifecycle shell without changing its engine | - A foundation PR adds the typed lifecycle store, narrow engine/persistence adapters, and fixture-backed tests without changing the live page, engine/worker/tokenizer/`studio_api.js`, publisher, or real browser data. | not recorded | not recorded | ready; after m7-studio-live-model-sources |
| training-step-cost | Split the training backward, and run a cheaper rival search against (a) | 1. | not recorded | not recorded | ready; after fbs-cost-profile |
| turn-memory-integer | Serve the turn memory from the integer artifact on the laptop | 1. | not recorded | not recorded | ready; after route-plus-memory |
| writer-admission | Make the native writer's admission hold at every seed | 1. | not recorded | not recorded | ready; after one-decoder |
| written-state-count-model | Test the written-state geometric count model against the frozen reference | Follow the accepted phase table, including its stop and fallback branches. | not recorded | not recorded | ready; after m2-donor-corpus-generation |

## Blocked

Items with status `held` or `decision`, each listed once. The Owner column shows the item's `open_to` value.

| Item | Title | Status | Owner |
|---|---|---|---|
| e8-orbit-memory-sigmoid-free-iteration | Build a sigmoid-free gate for E8 orbit memory and rerun it on GPU | held | lab |
| hopf-retained-fibre-test | Run the Hopf retained-fibre language test | held | lab |
| integer-nearest-readout | Serve the trained model's readout in the integer artifact | held | lab |
| m1-w4-float-credit-sheet | Register and run one float-credit training sheet (W4) | held | lab |
| native-geometric-reregister-hinge | Re-register the native geometric engine with the hinge loss | held | lab |
| point-key-integer | Serve the point key's opening in the integer artifact on the laptop | held | lab |
| r-full-epoch-revisit-schedules | Test full-epoch and frequent-token revisit schedules | held | lab |
| r-lawful-pointer-forced-copy-oracle | Build a lawful pointer, then run the forced-copy oracle | held | lab |
| r-vsa-hamming-binding | Test discrete Hamming binding (VSA) on the current writer | held | lab |
| read-binding-on-lawful-read | Port read binding to a Flock or Relation read | held | lab |
| served-rounding-cost | Bring training as served back inside the 2x operational gate | held | lab |

## Not doing

none recorded
