<!-- rendered by uor-harness loop render from 5b560b6fbb35f8df2210ce26ae202123ab99ec90; do not edit -->
# HANDOFF

Rendered from `docs/roadmap/items/*.md` at the commit in the stamp above. Each open-item row gives the first sentence of the item's `Done when` and `Cancel if` sections; blocked items are listed once, in the blocked table, with their owner; `not recorded` means the item has no such section or key yet.

## Open items

Items not `held`, `decision` or `done`. Held and decision items are listed once, in the blocked table below.

### ready (13)

| Item | Title | Done when | Cancel if | Size | Stands |
|---|---|---|---|---|---|
| d11-exporter-float-trunk | Write the D11 exporter for a D37 float trunk | A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. | not recorded | not recorded | ready; after float-training-module, m1-d11-exporter-discrete, owner-decision-float-path-scope |
| infra-loop-harness | Build the five-lab loop harness (the loop subcommands of uor-harness) | Every PR in the build plan on [the loop page](docs/loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. | not recorded | not recorded | ready |
| m1-close-saved-model | Close M1 with one saved discrete model passing all five bars | One saved discrete model: (a) starting reach at no fewer than 4,894 of 14,287; (b) reach kept on both seeds (G-Adm); (c) more than 16 of 256 cells in use plus the D39 occupancy condition (G-Occ); (d) integer top-1 through the integer export beats the unigram table (1,226) and the next-word table (6,300) on each of two seeds, with tokens per second reported; (e) the gain holds on a fresh sealed panel. | not recorded | not recorded | ready; after m1-a8-full-dose-rerun, m1-bvg-k-stage-t-training, m1-export-gap-and-toks |
| m1-fresh-panel-transfer | Evaluate the gain on the fresh sealed panel | '(e) Fresh transfer. | not recorded | not recorded | ready; after m1-export-gap-and-toks |
| m2-train-grounded-replies | Train the student for grounded replies from exact memory | M2 bar: '>= 256/512 saved questions and >= 40% of genuinely fresh ones, with the existing graders unchanged'; the student stays on the discrete path unless the owner decides otherwise (D41); it exports to D11 with the export gap reported (D37). | not recorded | not recorded | ready; after m2-donor-corpus-generation, m2-fresh-heldout-panel, m2-pin-train-mask-sheet, m1-close-saved-model, m1-d11-exporter-discrete |
| m3-qualify-memory | Qualify durable memory on the selected native model | On the selected artifact: | not recorded | not recorded | ready; after m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests |
| m4-export-uorlut01-cli | Serve the chosen model as UORLUT01 through the uor-chat CLI | M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. | not recorded | not recorded | ready; after m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory |
| m4-lawful-likelihood-bpb | Build a lawful likelihood so bits per byte can be scored | A likelihood with no softmax or squashing (D23, D25) feeding a registered instrument that scores the chat held-out stream in bits per byte, so the M4 bar "held-out quality at or below 0.90 bits per byte" can be tested. | not recorded | not recorded | ready |
| m5-laptop-full-path-measurement | Measure M1-class laptop cost on the accepted served model | M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". | not recorded | not recorded | ready; after m4-export-uorlut01-cli |
| m6-qualify-reasoning-coding | Qualify reasoning and coding on the served model | M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory. | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism |
| m7-parity-library-cli-browser | Build the library, CLI and browser bit-identity parity harness | M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. | not recorded | not recorded | ready; after m7-studio-connect-wasm-engine, m4-export-uorlut01-cli |
| m8-independent-verification | Independently verify the alpha on consumer hardware | A third party reproduces the install and the M1 to M7 bars from the checksum-pinned artifact: the serving-side bars on consumer hardware, and the panel-scored bars on a GPU as CHARTER §3.6 requires. | not recorded | not recorded | ready; after m8-release-packaging |
| m8-release-packaging | Prepare the checksummed alpha release package and install path | M8 bar ([`ROADMAP.md`](ROADMAP.md) line 21): "One versioned, checksum-pinned model file, declared alpha by the owner". | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser |

## Blocked

Items with status `held` or `decision`, each listed once. The owner is `owner` for a decision or for `open_to: owner`, otherwise the item's `open_to`.

| Item | Title | Status | Owner |
|---|---|---|---|
| m2-multi-turn-bar-number | Register a number for the multi-turn reply bar blocked under R14 | decision | owner |
| d41-driver-math-pilot | Optimize the driver math and run the ~1M-token pilot (D41 item 3) | held | lab |
| e8-orbit-memory-sigmoid-free-iteration | Build a sigmoid-free gate for E8 orbit memory and rerun it on GPU | held | lab |
| hopf-retained-fibre-test | Run the Hopf retained-fibre language test | held | lab |
| m1-w4-float-credit-sheet | Register and run one float-credit training sheet (W4) | held | lab |
| m2-donor-corpus-generation | Generate the one-time offline donor answer corpus (distill-d1) | held | lab |
| native-geometric-reregister-hinge | Re-register the native geometric engine with the hinge loss | held | lab |
| object-store-consumer-m3 | Wire the object store into the model and run the durable-memory bars | held | anyone |
| r-full-epoch-revisit-schedules | Test full-epoch and frequent-token revisit schedules | held | lab |
| r-lawful-pointer-forced-copy-oracle | Build a lawful pointer, then run the forced-copy oracle | held | lab |
| r-vsa-hamming-binding | Test discrete Hamming binding (VSA) on the current writer | held | lab |
| read-binding-on-lawful-read | Port read binding to a Flock or Relation read | held | lab |

## Not doing

none recorded
