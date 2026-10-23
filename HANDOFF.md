<!-- rendered by uor-harness loop render from 1cf7de6992586cf3d409dbdacf39ff962e0f4f20; do not edit -->
# HANDOFF

Rendered from `docs/roadmap/items/*.md` at the commit in the stamp above. Each open-item row gives the first sentence of the item's `Done when` and `Cancel if` sections; blocked items are listed once, in the blocked table, with their owner; `not recorded` means the item has no such section or key yet.

## Open items

Items not `held`, `decision` or `done`. Held and decision items are listed once, in the blocked table below.

### ready (33)

| Item | Title | Done when | Cancel if | Size | Stands |
|---|---|---|---|---|---|
| d11-exporter-float-trunk | Write the D11 exporter for a D37 float trunk | A float-trunk snapshot with recurrence and the Hamilton gate exports to D11 with the audits green and the export gap measured inside the band. | not recorded | not recorded | ready; after float-training-module, m1-d11-exporter-discrete, owner-decision-float-path-scope |
| float-training-module | Create the separate scanned D37 float training module | A new module builds CPU-only on the laptop with CPU tests, passes the banned-token scan under its own scanned entry, and carries the 24-kernel extract with no dependency on `crates/uor-r4-training`. | not recorded | not recorded | ready; after owner-decision-float-path-scope |
| infra-desktop-client-tauri | Build the Tauri desktop client against the synthetic stand-in | A Tauri app sharing the web interface that runs the integer engine natively, offline, with durable memory, matching the written specification. | not recorded | not recorded | ready; after m7-studio-connect-wasm-engine |
| infra-loop-harness | Build the five-lab loop harness (the loop subcommands of uor-harness) | Every PR in the build plan on [the loop page](docs/loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. | not recorded | not recorded | ready |
| legal-construction-exact-solve | Solve the saved legal-construction case in exact arithmetic | The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. | not recorded | not recorded | ready |
| m1-address-as-read | Build the address-as-read mechanism and an object-store consumer | A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address. | not recorded | not recorded | ready |
| m1-address-key-full-vector | Replace the 5-value address key with a full-vector metric | Stage 1 bar: "enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847", measured before any training. | not recorded | not recorded | ready; after m1-w2-capacity-capture |
| m1-bvg-k-block-apply-c2 | Draft the BVG-K Stage T pre-registration sheet (C2 has passed) | Already met: Stage C2 finite controls PASS (block apply exact) and are recorded, with the receipt above. | not recorded | not recorded | ready |
| m1-export-gap-and-toks | Measure export gap and tokens per second on a trained model | Stage 4 bar: 'the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply.' Decode and context rates are reported separately. | not recorded | not recorded | ready; after m1-d11-exporter-discrete |
| m1-faster-writer | Replace the full-vocabulary writer scan | A writer whose measured per-row comparison count is below the full-vocabulary scan (about 5×10⁶ today), with finite controls that match the full-scan writer, and a CUDA qualification. | not recorded | not recorded | ready |
| m1-fresh-panel-transfer | Evaluate the gain on the fresh sealed panel | '(e) Fresh transfer. | not recorded | not recorded | ready; after m1-export-gap-and-toks |
| m1-normalization-mechanism | Design a lawful normalization mechanism | A normalization mechanism with finite controls obeying CHARTER §3, plus a pre-registered test of its effect in the trunk. | not recorded | not recorded | ready |
| m1-position-mechanism | Design a lawful position mechanism | A position mechanism with finite controls that obeys CHARTER §3 (geometry only, no softmax or squashing, no matmul), plus a pre-registered test that it helps order. | not recorded | not recorded | ready |
| m1-wider-code-carrier | Prototype a wider code carrier (more roots or blocks) | A wider-carrier build passes finite CUDA qualification and a zero-training capacity measurement against the Stage 1 bar: the alias ceiling clears Bar (i) 1,941 net of admission (G-Cap), and admission at checkpoint 0 is above A8's 1,351 of 3,847 on the same panel. | not recorded | not recorded | ready; after m1-address-key-full-vector |
| m2-fresh-heldout-panel | Build a genuinely fresh held-out reply panel for M2 | A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'. | not recorded | not recorded | ready |
| m2-pin-train-mask-sheet | Settle whether the pinned M2 TRAIN-mask sheet is live after D37, and re-pin it | A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. | not recorded | not recorded | ready |
| m3-memory-turn-discriminator-gate | Add a memory-turn discriminator or margin gate | The last clause of the M3 bar in `docs/integration/project-track.md:116-120`: non-memory conversational prose does not silently overwrite stored facts. | not recorded | not recorded | ready; after m3-nonrelation-prose-panel |
| m3-nonrelation-prose-panel | Build a pre-registered non-relation prose panel | A pre-registered non-relation prose panel with grading, plus the baseline false-assert and false-overwrite rates on world=v2. | not recorded | not recorded | ready |
| m3-qualify-memory | Qualify durable memory on the selected native model | On the selected artifact: | not recorded | not recorded | ready; after m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests |
| m4-export-uorlut01-cli | Serve the chosen model as UORLUT01 through the uor-chat CLI | M4 bar: 'One model exported as an integer file under the serving contract (D11), usable from a command line, with held-out quality at or below 0.90 bits per byte'. | not recorded | not recorded | ready; after m1-d11-exporter-discrete, m4-lawful-likelihood-bpb, m1-close-saved-model, m3-qualify-memory |
| m4-flock-coupling-prereg | Finish the flock-coupling pre-registration | A registered C1 flock-coupling sheet with a non-author FIT, or a recorded decision that it is superseded. | not recorded | not recorded | ready |
| m4-lawful-likelihood-bpb | Build a lawful likelihood so bits per byte can be scored | A likelihood with no softmax or squashing (D23, D25) feeding a registered instrument that scores the chat held-out stream in bits per byte, so the M4 bar "held-out quality at or below 0.90 bits per byte" can be tested. | not recorded | not recorded | ready |
| m5-laptop-full-path-measurement | Measure M1-class laptop cost on the accepted served model | M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". | not recorded | not recorded | ready; after m4-export-uorlut01-cli |
| m6-qualify-reasoning-coding | Qualify reasoning and coding on the served model | M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory. | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism |
| m6-reasoning-panel-freeze | Build and freeze the m6-reasoning panel and oracles | A frozen, pre-registered panel with deterministic oracles for compositions, altered constraints and code tasks (D28: no external teacher). | not recorded | not recorded | ready |
| m7-parity-library-cli-browser | Build the library, CLI and browser bit-identity parity harness | M7 bar: "giving bit-identical tokens across the library, command line and browser", with identical session snapshots, on the accepted `UORLUT01` artifact. | not recorded | not recorded | ready; after m7-studio-connect-wasm-engine, m4-export-uorlut01-cli |
| m7-studio-connect-wasm-engine | Connect the Studio to the browser WASM engine | M7 criterion 3: the Studio runs entirely client-side in a browser Web Worker with truthful capability reporting and zero remote model calls; the `.wasm` module passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7 PASS, mul=0, div=0, float=0). | not recorded | not recorded | ready |
| m8-independent-verification | Independently verify the alpha on consumer hardware | A third party reproduces the install and the M1 to M7 bars from the checksum-pinned artifact: the serving-side bars on consumer hardware, and the panel-scored bars on a GPU as CHARTER §3.6 requires. | not recorded | not recorded | ready; after m8-release-packaging |
| m8-release-packaging | Prepare the checksummed alpha release package and install path | M8 bar ([`ROADMAP.md`](ROADMAP.md) line 21): "One versioned, checksum-pinned model file, declared alpha by the owner". | not recorded | not recorded | ready; after m4-export-uorlut01-cli, m5-laptop-full-path-measurement, m6-qualify-reasoning-coding, m7-parity-library-cli-browser |
| object-store-consumer-m3 | Wire the object store into the model and run the durable-memory bars | A consumer reads and writes through the object store on the model path, and the durable-memory bars run with recorded results. | not recorded | not recorded | ready |
| r-lawful-pointer-forced-copy-oracle | Build a lawful pointer, then run the forced-copy oracle | A lawful pointer with finite controls, then the forced-copy oracle run once under a registered sheet that has a non-author TEST FITNESS: FIT review. | not recorded | not recorded | ready |
| r-prime-zeta-vs-nulls | Test the prime-zeta code against injective nulls at zero updates | Zero-update reach and capacity numbers for prime-zeta, random-injective and quota-only-injective codes on the same panel, recorded as a notebook row. | not recorded | not recorded | ready |
| r-read-off-vs-read-on | Compare read-off and read-on for Route and Flock on checkpoints | Paired read-on versus read-off results on the same retained checkpoints are recorded as notebook rows, with the classification instrument versus mechanism stated. | not recorded | not recorded | ready |

## Blocked

Items with status `held` or `decision`, each listed once. The owner is `owner` for a decision or for `open_to: owner`, otherwise the item's `open_to`.

| Item | Title | Status | Owner |
|---|---|---|---|
| m2-multi-turn-bar-number | Register a number for the multi-turn reply bar blocked under R14 | decision | owner |
| owner-decision-board-status-disputes | Settle the disputed D11 and Hopf observation statuses on the board | decision | owner |
| owner-decision-float-path-scope | Decide whether a float (D37) path may count toward M1, M2 and the product | decision | owner |
| owner-ruling-spin-ternary-arithmetic | Rule on spin-native weights and ternary/XOR, or build a geometric alphabet | decision | owner |
| carrier-credit-hard-selection | Build a hard-selection step for carrier credit | held | lab |
| d41-driver-math-pilot | Optimize the driver math and run the ~1M-token pilot (D41 item 3) | held | lab |
| e8-orbit-memory-sigmoid-free-iteration | Build a sigmoid-free gate for E8 orbit memory and rerun it on GPU | held | lab |
| hamilton-gate-offline-float-sheet | Run a Hamilton gate offline-float sheet with language fit and export gap | held | lab |
| hopf-retained-fibre-test | Run the Hopf retained-fibre language test | held | lab |
| m1-a8-finite-cuda-qualification | Run A8 admission-anchor controller CUDA qualification and restoration | held | lab |
| m1-a8-full-dose-rerun | Run the full A8 dose with reach protected | held | lab |
| m1-bvg-k-stage-t-training | Train the BVG-K batched-coordinate law on two seeds | held | lab |
| m1-close-saved-model | Close M1 with one saved discrete model passing all five bars | held | lab |
| m1-trunk-assembly-and-training | Assemble and train the full lawful trunk (Stage 3) | held | lab |
| m1-w2-capacity-capture | Run the W2 read-code capacity capture | held | lab |
| m1-w4-float-credit-sheet | Register and run one float-credit training sheet (W4) | held | lab |
| m2-donor-corpus-generation | Generate the one-time offline donor answer corpus (distill-d1) | held | lab |
| m2-train-grounded-replies | Train the student for grounded replies from exact memory | held | lab |
| native-geometric-reregister-hinge | Re-register the native geometric engine with the hinge loss | held | lab |
| r-finite-relational-head | Train the finite relational head | held | lab |
| r-full-epoch-revisit-schedules | Test full-epoch and frequent-token revisit schedules | held | lab |
| r-quaternion-transport-order | De-confound the quaternion-transport ordering result | held | lab |
| r-rank-objective-keeps-cells | Design a rank objective that keeps cells occupied | held | lab |
| r-ranking-with-admission-intact | Test ranking with admission intact | held | lab |
| r-secant-surrogate-descent | Check whether a secant surrogate gives descent for recurrence gates | held | lab |
| r-vocab-4096-vs-16384 | Compare 4,096 and 16,384 tokenizer pieces | held | lab |
| r-vsa-hamming-binding | Test discrete Hamming binding (VSA) on the current writer | held | lab |
| read-binding-on-lawful-read | Port read binding to a Flock or Relation read | held | lab |
| recurrence-gates-six-recurrence-fit | Run the six-recurrence fit for the recurrence gates | held | lab |
| spiralcore-fbs-offline-float-sheet | Measure SpiralCore/FBS descent and tokens per second in a float sheet | held | lab |

## Not doing

none recorded
