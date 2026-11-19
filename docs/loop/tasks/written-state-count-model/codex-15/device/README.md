# Wick ordered-role backend

The SM89 kernel shares the scalar transition between deterministic two-pass fit and inference. It carries i16 features and prototype coordinates, uses the pinned D11 orbit roots and signed axes, and widens coordinates only in the wire transcript. Each training pass reads rows and complete candidate starts in sealed order. It binds the observed prototype bytes to the original D11 trunk, tokenizer, frozen word classes, orbit/action tables, ordered TRAIN receipt and framing/operator contract.

The Rust driver reconstructs each original BOS-inclusive tape from its unmarked user text and protocol-2 framing. TRAIN grade rows must bind their original tape hash, contain both explicitly declared role fields, and enumerate each supplied span. GPU fit emits the fitted bank and the raw update transcript. The host decodes the TRAIN transcript without repeating training arithmetic. Rust update, clamp and post-update coordinate comparisons run on finite synthetic fixtures. Public and bounded inference check each device candidate, contextual slot, signed feature, class distance and rank against the read-only native selector.

## Reproduce the finite checks

Build on Wick from a source root containing the listed crate dependencies and this directory.

```sh
bash device/build-wick.sh SOURCE_ROOT NEW_BUILD
bash device/finite-wick.sh NEW_BUILD ASSETS NEW_FINITE
```

Use the two frozen paired artifacts, their individually bound tokenizers and the asset `SHA256SUMS`. The build records source hashes. The finite runner uses the shared Wick lock and checks the executable and asset hashes. It covers zero signal, maximum width, missing context, clips that cut a multipart endpoint, structural slots, intra-candidate and inter-candidate ties, both clamp directions, invalid state and tape rejection, exact bank reload and a fixed-competitor decision with a competing global tie. The 1/32-row timing cases use repeated maximum-width dense synthetic tapes.

## Collect one reviewed stage

After the final FIT and pre-run review, use a new output root for each stage.

```sh
bash device/capture-wick.sh BUILD ASSETS PANEL PANEL_SHA256SUMS NEW_STAGE
cmp STAGE_1/deterministic.sha256 STAGE_2/deterministic.sha256
```

Keep TRAIN grades and held bounds separate from the blind tape. The collection runner supplies only blind text and original IDs to the full-prefix public scorer. The bounded scorer receives a separately hashed interval and clips actual contextual slots against it. The fixed-competitor packet reads only the two already captured bounded spans, their winning classes and their evidence. Its CUDA and Rust decisions are compared without recalculating features. The eligibility output lists removed spans and shared spans whose clipped context or signed feature changed.

Each arm emits a 400-byte paired-copy `bank.role`, its `train.receipt`, the TRAIN update transcript, both zero-bank signatures and both fitted decision traces. It also emits `fixed.json` and `eligibility.json`. Timing and memory fields are written separately so repeated deterministic manifests cover only state and decisions.
