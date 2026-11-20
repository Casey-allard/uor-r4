# [codex] Preserve and replay the S5 fitted mixture

The archived S5 A1 result records 306 Pick-column Q32 weights for fired buckets observed on reserved TRAIN. The original fitter computed the full bucket-by-expert matrix in memory and discarded it after scoring. The historical report lacks the full KN, per-order ML and uniform columns. The packer rejected it before creating an output. Original S5 also filtered retained count rows against its query stream. Those rows cannot serve arbitrary prompts, and the capped D27 S2 artifact uses different count ranks and backoff. Reusing it under the S5 weights would change the predictor.

The fitter now publishes the exact original weights as bucket-major `weights_q32` in each arm record. The `FrozenPickMixture` codec preserves every A1 word in little-endian `u64` and binds the enclosing report by SHA-256. An order-ten A1 payload is **59,180 bytes** for 528 buckets × 14 weights plus its 44-byte header. The codec rejects a wrong binding, a malformed length, weights outside Q32, a row that fails to sum to one, and nonzero weight in an unavailable expert. Deployment also pins the payload hash and supplies lookup state matching the fit manifest.

The reloaded `predict_mass` entry point takes the original causal target's KN Q48 mass, order-by-order count and total words, and Pick class/hit. It uses the same row construction, bucket assignment and final product accumulation as the fitter, with a single right shift after accumulation and the same one-unit floor. This is the native scoring slice. The ordinary client still needs a query-independent storage and lookup implementation that produces those exact words from the count state; the weight artifact alone carries no count indexes, token codebook, or candidate enumeration.

## Finite replay and original historical columns

The deterministic fixture used two synthetic BOS/EOS records with order-two lookup inputs, the existing causal Pick operator and the registered S5 integer EM machinery. The fit used only the first record's nine targets, with number of updates, initialization and bits function equal to S5. Two processes independently saved and loaded all 864 words, and both files and receipts were byte-identical. All sixteen bucket and Q48 mass words replayed, including seven targets from the unused second record. The fixture manifest binds token IDs, codes, lookups, fit mask, order and EM configuration. The artifact occupied **6,956 bytes**, SHA-256 `f32eed9a80b9d250442faaa99c094e47de262d06737c50b96bd30f770ef17261`. Summed fit Q12 bits went from 110,592 to 32,777. This is finite serialization and operator conformance. It does not measure corpus quality. [Replay receipt](finite.json)

The historical probe pins the original 100%-C S5 report and the real seed-1 D11. It derives the four block codes by the original `root_codes` path, runs the original `code_pick`, and indexes the published A1 Pick column with explicit finite count totals. On the repeated-pair prefix it copied ID 4 at distance zero, using bucket 200 with original weight 3,127,093,301, for Q48 contribution 204,937,186,574,336. On the nonidentical-pair prefix it copied ID 11 at distance 12, with three best-distance candidates and latest position nine. Bucket 527 had original weight 1, for Q48 contribution 65,536. Every other target gets zero from Pick in either probe. The supplied totals choose the published buckets without querying protected count tables. [Exact probe](historical-exact.json) · [Near probe](historical-near.json)

## Exact refit and remaining deployment gate

A fresh approved replay can recover the exact matrix if it reproduces the historical input, source and operator identities. Pin the four original split assets against `run2/split.sha256`, the original ID-byte census against report hash `befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9`, and the seed-1 D11 against `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`. The original split used 2,075,608,357 C tokens and 15,947,584 reserved-R fit targets. The published combined query includes DEV-tune and its copy controls, with DEV-test still sealed. The original complete command shape is:

```sh
kn-reference c.tokens.u16 q.tokens.u16 ID-BYTES.u8 q.classes.u8 10 16777216 \
  NEW-REPORT.json --per-mille 1000 --pick-arms q.segments.u8 seed1.d11
s5-mixture pack NEW-REPORT.json NEW-WEIGHTS.s5w
```

The original 100%-C run took 3,585 seconds and self-reported peak RSS of 20,076,956 KiB on Wick's RTX 4090. Its authorized run2 container cap was 38 GiB. Treat these as a measured planning anchor and admit the new run only with checked capacity, inputs, a new sheet and FIT, an owned device and a completion watcher. The original split and C420 base inputs are live and byte-verified inside Wick's existing workspace volume. The read-only retrieval instructions and exact 1%-C/R preparation bounds are recorded separately. [Input custody and reproducibility](INPUTS.md) Review the current CUDA plus integer-host fitter division under the charter before dispatch. No GPU ran in D28's count slice.

An R-only fitter that omits DEV-tune is a separate development route. It must show that query filtering preserves raw and adjusted lookup words on all reserved-R rows and that every recovered published Pick weight and fitted-bit trace matches exactly. A matched refit supplies weights only. P1 deployment still requires a query-independent representation of raw and modified KN tables under a named byte budget, matching causal bucket/Pick decisions and per-target Q48 mass on ordinary serving and GPU replay, followed by the registered P0 and dialogue comparisons. A pruned count table changes its available experts and would need its own registered fit and comparison.

## Reproduce locally

From the repository root, set `target` to an absolute build directory, `scratch` to an existing scratch directory, and `trunk` to the pinned public seed-1 D11.

```sh
instrument=docs/labs/kn-reference/instrument/Cargo.toml
history=docs/loop/tasks/written-state-count-model/claude-439/run2/pick-p1000.json
CARGO_TARGET_DIR="$target/instrument" cargo build --locked --offline --manifest-path "$instrument" --bin s5-mixture
"$target/instrument/debug/s5-mixture" fixture "$scratch/finite.s5w"
"$target/instrument/debug/s5-mixture" historical "$history" "$trunk" 1 256 0,2,3,4,2,3
"$target/instrument/debug/s5-mixture" historical "$history" "$trunk" 10 32768 0,2,3,4,5,6,7,8,9,10,11
CARGO_TARGET_DIR="$target/workspace" cargo test --locked --offline -p uor-r4-discrete --lib kn_ceiling
```

The isolated workspace run passed all 21 ceiling tests, and the detached instrument passed all 11 Pick-fit tests, including an assertion that every reported historical-style fired Pick weight agrees with the corresponding complete matrix word. Build outputs were isolated because reusing the target between the workspace and detached manifests produced conflicting serde and serving feature artifacts. No dependency change was required.
