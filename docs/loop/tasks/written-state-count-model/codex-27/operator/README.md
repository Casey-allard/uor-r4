# D27 query-conditioned assertion selector

The opt-in Rust operator accepts the original D11 prompt IDs, integer root words and word classes. It receives the requested and excluded person from the role stage and the active query fence from the caller. The operator has no fit, learned update, semantic dictionaries or additional serialized state. The root table and original prompt belong to the D11 parent. The scorer receives no gold target, expected value, value-source address or grading data.

Each candidate starts with an exact whole-word occurrence of the requested person at an incumbent-admitted assertion head. The first contiguous whole word after the person must end at or before the first incumbent stop token. That word supplies the relation evidence. All complete words within the query fence may supply matching evidence, with the requested and excluded person words and stops removed. An explicitly supplied relation can narrow the query evidence to one eligible original query word for a diagnostic arm.

The exact arm compares complete BPE ID sequences. The geometry arm gives a literal match precedence and otherwise compares equal-piece-count words by mean squared Euclidean displacement across the four ordered D11 root blocks per piece. Each block uses the parent's eight signed integer coordinates. The radius admits at most 16 squared units per piece on average, with normalized scores compared through integer cross-products. A distinct ID sequence at geometric distance zero produces a collision and withholds release in the absence of literal evidence. Equal best scores across distinct cue words withhold release. All assertions with the uniquely winning cue word are returned to the existing downstream value-consensus gate.

This bounded representation targets a clause with a single directly post-person relation cue. Constructions with auxiliaries, multiword relations, reverse syntax, polysemy or unsupported polarity can exceed its evidence. Literal mentioning of a second candidate relation inside the query fence can produce ambiguity. Native root proximity can also produce adverse matches with no semantic link. The finite canaries exercise the algorithm and do not measure any of those conditions in real prose.

## Host and CUDA differential

```sh
cargo run --locked --manifest-path "$D/Cargo.toml" --bin d27-relation-canary -- "$Q/canary.in" "$Q/canary.oracle"
nvcc -O3 -std=c++17 -arch=sm_89 -code=sm_89 "$D/score.cu" -o "$Q/d27-score"
"$Q/d27-score" "$Q/canary.in" > "$Q/canary.cuda"
cmp "$Q/canary.oracle" "$Q/canary.cuda"
```

Here `D` is this operator directory and `Q` is a new scratch directory. Compile on the authorized CUDA host. The executable requires compute capability 8.9, launches the scoring kernel, and reports its device and row count on stderr. There is no host scoring fallback. The finite planted cases establish exact identity, an admissible nonliteral root step, root collision, lexical and geometric ties, retention of duplicate cues, person masking, empty support, misleading values, a competing name, and full split-word identity.

The input begins with the eight ASCII bytes `UORREL01` and 128 signed little-endian 32-bit coordinate words in D11 root-major order. Every following row starts with three little-endian u32 words for tape length, cue count and query word count. It holds 192 `[id, packed_root]` pairs followed by 192 `[cue_start, cue_end, query_start, query_end]` entries. Indices refer to half-open spans in the original tape. Padding is zero. Candidate order comes from Rust's original tape discovery. The GPU independently scores and selects over those discovered cue and query spans.

The output row holds exact and geometry outcome codes followed by 192 five-word entries in candidate order. Each entry contains literal evidence, root collision, best admitted nonzero squared distance, exact selection bit and geometry selection bit. The outcome codes are 0 for empty support, 1 for selected, 2 for tie and 3 for collision. Absent distance is `u32::MAX`. Other padded output slots are zero. Rust serializes an oracle using the same layout. A byte match covers per-candidate evidence and membership of the selected assertion set on that input. Role inference, discovery, value consensus and emission require their own replay.

## Panel staging

```sh
cargo run --locked --manifest-path "$D/Cargo.toml" --bin d27-relation-stage -- stage MODEL BLIND NEW_DIR
cargo run --locked --manifest-path "$D/Cargo.toml" --bin d27-relation-stage -- stage MODEL BLIND NEW_DIR PERSON RELATION
"$Q/d27-score" NEW_DIR/input.bin > CUDA_OUTPUT
cargo run --locked --manifest-path "$D/Cargo.toml" --bin d27-relation-stage -- verify NEW_DIR CUDA_OUTPUT
```

The first stage form reads only blind prompt IDs. The second additionally opens separate supplied-person and supplied-relation annotations and emits all three lanes. The supplied-person lane changes the requested span and preserves ordinary exclusion. The supplied-relation lane additionally restricts the query evidence to one eligible word. The artifact computes query framing in all three lanes. The scorer never opens the gold answer file.

The stage directory contains `input.bin`, `oracle.bin`, `native.jsonl` and `binding.json`. Its JSONL includes the candidate spans, evidence and decisions keyed by panel row and lane. Missing roles and unavailable supplied relation words receive an explicit skip status and have no device row. The verifier uses the recorded zero-based indices to identify the first differing output word and rejects missing or trailing data.

The post-hoc transfer audit reads the sealed packet and its matching CUDA output. It uses the held keys to compare each known target cue against the supplied query word and records the first failed comparison, any competing evidence and the source hashes. It performs no role inference, generation or threshold tuning.

```sh
cargo run --locked --manifest-path "$D/Cargo.toml" --bin d27-relation-stage -- audit-transfer STAGE_DIR HELD_GOLD HELD_RELATION CUDA_OUTPUT
```
