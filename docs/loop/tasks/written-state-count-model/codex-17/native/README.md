# Native all-starts fit and serving scratch

This scratch implementation binds the frozen legacy candidate universe to one integer bank. CUDA compiles each assertion's distinct address-keyed candidate rows in sealed order. It computes leave-one-family-out nearest onset distances and per-shape radii on device. A fixed-width parent record carries source geometry, class, and radius from GPU fit through lossless pack, reload, and Rust or CUDA inference. The bank is shared across all rows. No row contains a copy of the D11 model or bank.

The ordered arm uses the signed D11 orbit L1 distance. Order-neutral canonicalizes roots within each bridge/complete-word segment and class, retaining the original shape. Position-only stores and counts positive/negative votes using only relative connector depth and remaining literal length. It uses the same number and width of retained record slots, ignores lexical roots, and does not see absolute positions. It pools opposed positional votes. The geometric arms report the count of identical cross-class descriptor pairs and refuse that fit. An unknown shape or insufficient geometric evidence abstains, as does a tie or nonpositive top margin. The ranker uses no grade tape at inference.

## Build and commands

From this directory on Wick, with the sibling pinned `all-starts-finite/native-source/` present, build the host adapter using `cargo build --release --locked --offline` and the device with:

```sh
/usr/local/cuda/bin/nvcc -O3 -gencode arch=compute_89,code=sm_89 device.cu -o native-sm89
```

The reproducible test `bash run-wick.sh NEW_SYNTHETIC_RECEIPT_ROOT` builds both, requires the shared Wick GPU lock and 4 GiB free, and runs only the built-in synthetic fit and inference fixtures. The CLI contracts below describe future reviewed inputs. This implementation does not authorize a project fit or claims run.

```sh
host=/absolute/path/all-starts-native
device=/absolute/path/native-sm89
capture=/absolute/path/candidates.json
base_sha=$(jq -er .base_d11 "$capture")
tokenizer_sha=$(jq -er .tokenizer_sha256 "$capture")
site_sha=$("$host" site-key "$capture")
"$host" pack-json "$capture" "$base_sha" "$tokenizer_sha" "$site_sha" train.packet
"$host" fit "$device" train.packet independently-labelled-train.json ordered ordered.bank
"$host" fit "$device" train.packet independently-labelled-train.json neutral neutral.bank
"$host" fit "$device" train.packet independently-labelled-train.json position position.bank

# Pack the appropriate same-seed blind capture separately, without labels.
"$host" pack-json "$blind_capture" "$base_sha" "$tokenizer_sha" "$site_sha" blind.packet
"$host" score "$device" blind.packet ordered.bank ordered.trace
"$host" decode-trace blind.packet ordered.bank ordered.trace ordered-sources.json
```

All output paths and their sibling `.work` directories must be fresh. Keep each score's `.work/bindings.json` with its trace. It binds the exact loaded bank, input packet, blind capture, and output trace. Run future GPU invocations under the Wick custody and lock of the independently reviewed unit.

## Canonical byte and output contract

The input accepts the capture adapter's `uor-r4.legacy-assertion-candidates/1` JSON, keyed by the original prompt ordinal and array-order assertion index. The binder checks expected base D11, tokenizer, and site digest. The site digest names frozen role carrier, codebook, classes, span state, and signed orbit. It also checks both byte encodings of roots and overlapping original slots within each assertion. The 36-word little-endian binary header carries base, tokenizer, site and capture identities. Each 35-word entry has `[ordinal, assertion_index, original_tape_len, fact_end_inclusive, query_start, fence, eligible_mask, word_start_mask]` followed by three original-offset legacy records of nine words each. Their fields are `[source, first_word_end, writer_reachable, two-bit-class-signature, five words containing ten packed u16 roots]`. Missing sources are `0xffffffff`. A no-fact prompt remains one sentinel assertion entry. Cap-rejected mask hits remain visible with zero geometry. Extended starts remain in the capture's separate inventory. Context is bounded at 192, with up to 65,536 flattened assertion entries per invocation and a hard limit of 256 fitted records.

Fit-only JSON is an array of `{ordinal, assertion_index, family, source}`; non-null `source` must be an authorized original absolute address. The controller canonicalizes labels in packet order as four little-endian words per assertion. Null or omitted labels never fit an exemplar. A nonnull gold address outside reachable coverage stops. Each supervised assertion yields one positive and a negative for every other reachable legacy candidate. The fit refuses overflow without replacing or dropping rows. The 8,416-byte learned artifact has a 208-byte binding envelope and a fixed raw bank of 16 header bytes plus 256 × 32-byte records. The hashes bind base, tokenizer, sites, exact TRAIN packet, canonical labels and raw payload. Source and executable identities are separately sealed by the runner. Neither the fitter nor the runtime stores a per-example absolute address in learned geometry.

The output trace starts `[0x32545355, entry_count, 68, mode]`. Each assertion starts with eight words for original ordinal, assertion index, status, selected source, top numerator and divisor, runner-up numerator and divisor. Status codes are 0 selected, 1 no candidate, 2 unseen, 3 tie, and 4 nonpositive. Only status 0 carries a source. The next 60 words are three 20-word candidate traces retaining original source, first word end, eligibility status, shape and class signature, class distances or votes, radius, numerator and divisor, and ten transformed packed roots. JSON decoding gives signed score ratios and named statuses. Null assertion index denotes the no-fact sentinel. Before rewriting the writer input, verify the three trace/packet/bank hashes against the score receipt. Join exactly `(ordinal, assertion_index)` to the original assertion, copy the authorized source unchanged, and feed it through the frozen per-assertion cursor and sequence consensus. An abstention supplies no source. This utility does not choose among assertions or supply its own fallback.

## Verified scope

Two independent cold fits and reloads on Wick produced byte-identical banks, traces and decoded original sources. The independent native Rust scorer matches every raw CUDA trace word for six sets of 1,025 synthetic probes per cold stage. Each probe set includes original tape length 192, a high original source address, and a no-fact row. The known-right ordered and neutral cases choose original offset 5 and the finite writer emits `[13,15,14,45,1]`. Permuted synthetic TRAIN annotations produce offset 4 and the wrong first emitted ID. A positive exact ratio tie and a missing same-shape family radius abstain. Position-only pools 24 opposite-label pairs and yields two positive and four negative votes per key. The overflow test flushes no bank and reports 256 used before refusing the 257th record. Both modes use the same 8,416-byte physical allocation. The CPU helper checks and scores the CUDA-fitted synthetic banks. It does no fitting.

The source format is a separate scratch artifact, with no production paired-copy or consensus hook. The score receipt pins custody at the capture boundary. Authentic capture roots, tokenizer framing, exact annotation custody and actual public deployment must still be independently verified on the future unit. The synthetic benchmark establishes finite fit and score discrimination with device-to-Rust parity. It measures no generalization, production throughput, real bank support, or GPU lease economics.

## Exact Wick receipt

Canonical final run is `~/uor-codex/20261119-fresh-boundary-validity/all-starts-native/run-05`. The preserved compile attempts `run-01` and `run-04` stopped on a C++ parenthesis and a Rust JSON-macro parse error respectively, before GPU calibration. The separately retained `run-02` and `run-03` predate the final decoder.

- Manifest SHA-256 `3053612d44c4113dda2ef223c224c1c6e452465bbcbaea07d80b1f2a23536c3d`.
- Source manifest SHA-256 `1826c68454d8c9101c13007e0113214de6795a8a293454300d879e00705cf6a8`.
- Rust executable SHA-256 `3462fc49ef084ee629be51b16bd3b535d97ce147156cb023396cbbfff3105bcd`.
- SM89 executable SHA-256 `e8dedf21dd5e92866c389e798c97bd0e355221a015dddde74dcf43890b83c226`.
- [Sealed output hashes](receipts/SHA256SUMS) · [Exact sources](receipts/source.sha256) · [Finite observations](receipts/verification.txt)
