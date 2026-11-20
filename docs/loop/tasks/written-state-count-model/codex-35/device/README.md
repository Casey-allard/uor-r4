# D35 paired-source CUDA witness

The device consumes the original candidate spans and integer coordinates captured by the ordinary D26/D34 reader. The local coordinate is the positional geometry of the exact source span in the fact-to-fence interval. The fact-prefix contextual coordinate uses D26's causal recurrence before the fact start, expanded into four eight-coordinate orbit roots. It is shared by all candidates in that fact, so those contextual displacements are zero. The candidate-prefix condition expands the same recurrence immediately before each candidate's original start, excluding the candidate and later tokens. Both conditions use the same eligibility, local coordinates, pair order, executable and arithmetic.

The CUDA fit kernel subtracts each eligible rival's coordinates from those of the TRAIN target on the same row. It retains every resulting displacement in packet row order and eligible rival start order, including duplicates. The CUDA score kernel computes each ordered candidate-minus-rival displacement and the two minimum L1 distances to the bank under positive and negative prototype orientation. A candidate is selected only when its forward distance is strictly smaller for every rival. The reversed control tests the converse inequality over the same recorded distance words. When the pool has one member, both orientations use the frozen D26 singleton selection, including its abstention floor. Empty pools abstain in both orientations.

Independent device arithmetic covers displacements, bank order and serialization, paired distance reductions, strict dominance and winners. The device takes eligibility, original span boundaries, local and recurrence coordinates, and singleton selection from packet capture. Ordinary Rust separately replays the selector over the byte-identical bank and executes the copy writer. Ordinary output and the supplied-fact diagnostic use the same frozen bank with independent candidate packets.

## Binary formats

All integers are little-endian. Each input starts with the 4-byte ASCII magic `D351`, then mode and row count as u32. A row has two u32 header words, candidate count and marker. In mode 1 (fit), the marker is the original TRAIN target start. In mode 2 (score), the marker is the frozen D26 singleton's selected start or `UINT32_MAX`. Every multi-candidate or empty score pool uses `UINT32_MAX`. Each candidate carries u32 start and exclusive end, 32 sign-extended i16 coordinates in i32 words, and 32 i32 contextual coordinates. Candidate starts strictly increase. The packet admits at most 4,096 rows, original spans within 192 tokens, and at most 1,000,000 ordered score pairs.

A bank begins with the 8-byte ASCII magic `D35PAIR1` and u32 prototype count from 1 to 8,192. Each prototype holds 32 i32 local and 32 i64 contextual differences, 384 bytes per prototype. For coordinates at both limits the per-comparison L1 bound is `32 * (2 * 65535 + 2 * 4294967295)`. Distances use u64 throughout.

A score output begins with 8-byte ASCII magic `D35RANK1` and u32 row count. Each row contains u32 candidate count, forward winner and reversed winner, with `UINT32_MAX` for abstention. For each candidate it includes u32 start and end, then u64 forward and reverse distances for every rival except itself in original candidate order. The host decoder checks reciprocal distances and the strict winner from the raw pairs. Its JSON row index `packet_row` is zero-based and is joined against the packet sidecar.

## Wick phases

Use fresh directories and distinct reviewed admission manifests. Each admission manifest directly binds `aot/stage.sha256` and the phase's packet manifests. The score admission additionally binds the fit output's `results.sha256`. Manifest entries use paths relative to their manifest location. Also bind the corresponding FIT, GO and other required receipts in the admission manifest. Launches require the standing Codex lock, the RTX 4090 UUID, compute capability 8.9 and an idle compute process list. The device build emits SM89 cubins with no embedded PTX, enumerates both application kernels and checks the SASS inventory. Each primary and cold execution is a fresh CUDA process.

```bash
bash source/build-wick.sh aot
rustc --edition 2021 -O source/fixture.rs -o generate-fixture
./generate-fixture finite
(cd finite && sha256sum train.bin ordinary.bin bank.expected ordinary.expected \
    bank.corrupt ordinary.corrupt ordinary.changed > packets.sha256)
bash aot/run-wick.sh finite finite finite-admission.sha256 "$finite_admission_sha" finite-output

bash aot/run-wick.sh fit ../split fit-admission.sha256 "$fit_admission_sha" fit-output
bash aot/run-wick.sh score ../split score-admission.sha256 "$score_admission_sha" held-output fit-output

rustc --edition 2021 -O source/decode.rs -o paired-value-decode
./paired-value-decode held-output/candidate-prefix/primary/ordinary.out
```

The split directory contains `fact-prefix/` and `candidate-prefix/`, each with `train.bin`, `ordinary.bin`, `diagnostic.bin` and `packets.sha256`. During fitting the CUDA executable only reads the TRAIN packet. During scoring it only reads the selected blind candidate packet and reloaded fitted bank. The two stages preserve separate result manifests. The runner verifies byte-identical cold fit and scores, as well as equal bank lengths across geometries.

The planted finite inputs cover context-only selection with identical local coordinates, direction reversal, strict tie, maximum-width coordinate differences, empty input, and singleton selection and abstention. They also require rejection of a prototype beyond its valid coordinate range, illegitimate singleton metadata, and a mutated expected winner.

## Independent TRAIN bank and writer audit

Build the audit on Wick using the release dependencies of the source-pinned native client. The explicit library hashes below are the Cargo filenames from that build.

```bash
lib="$Q/native/pinned-target/release/deps"
rustc --edition 2021 -O -L "dependency=$lib" \
    --extern "serde=$lib/libserde-8fcd8d5076ce198f.rlib" \
    --extern "serde_json=$lib/libserde_json-6bd635c5b0e61624.rlib" \
    --extern "uor_r4_integer=$lib/libuor_r4_integer.rlib" \
    bank-audit.rs -o bank-audit
./bank-audit train "$Q/frozen/train" "$Q/train-projection" "$Q/fit-output"
./bank-audit writer "$D34_EXPOSED_GOLD" \
    "$Q/native/pinned-legacy-exposed/full-detached/30100.json" 30100
```

The TRAIN check reconstructs admission from the sealed TRAIN labels and both captured projections, independently rebuilds each complete device packet and checks its index and omissions. It serializes the gold-minus-rival records in admitted row and rival order and requires byte equality for the primary and cold banks in both geometries. All admitted row, fact, candidate and rival identities must match across geometries.

The writer check supplies the onset address directly to the same `ExactCopyState` used in ordinary serving. It replays the positive original onset, a different eligible onset in the same assertion, and a disabled source. It checks the expected original IDs through EOS and records all writer state words, including the first incorrect emitted ID for the adverse address. The exposed labels enter this writer-only diagnostic and stay outside the ordinary selector.
