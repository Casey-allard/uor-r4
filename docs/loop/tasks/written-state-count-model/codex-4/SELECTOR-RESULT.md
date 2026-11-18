# [codex] Native content selector on the two-entity panel

The trained selector missed the registered held-out source bar on both released artifacts. Dropping the learned query-context terms improved the number of requested sources it returned. Among the trained selector's 214 misses, 73 joined the correct requested entity in both mentions but stopped at the first BPE piece of its name. The source reader then returned the final piece of the name as its value.

| Frozen arm, per released seed | Correct query / 288 | Correct fact / 288 | Correct value start / 288 | Correct on both questions / 144 | Distractor start / 288 | Original panel sources / 16 |
|---|---:|---:|---:|---:|---:|---:|
| Learned query, fact and relation | 74 | 74 | 74 | 18 | 44 | 0 |
| Query context ablated | 96 | 96 | 96 | 24 | 48 | 0 |

Each row with an exact requested fact endpoint returned the correct value start. Every correct source also had the exact requested query and fact. Comparing the two arms per held-out record, both succeeded on 68, both missed on 186, training the query terms helped on six and hurt on 28. All counts apply independently to each seed on the same texts. The tables of fitted prototypes were byte-identical between seeds once the artifact-binding metadata was excluded.

## Where a single token lost entity identity

A post hoc inspection of the frozen predictions found 74/144 requested sources for the single-token name set and 0/144 for the split-name set. The tokenizer and saved positions permit an exact decomposition across all 288 held-out prompts. The trained query anchor had the requested final-token ID on 74 and the other name's final-token ID on 44. The remaining 170 selected the following token IDs.

| Decoded tokenizer entry | ID | Selected / 288 |
|---|---:|---:|
| Leading piece of Ada (`ĠA`) | 439 | 67 |
| Leading piece of Noah (`ĠNo`) | 2990 | 52 |
| `Ġtook` | 614 | 49 |
| `Ġsil` | 3585 | 2 |

In 73 of the 119 name-leading choices, both the selected query and selected fact were exactly two positions before their annotated requested endpoints. On all 73 the selected value was `fact + 2`, which was the final token of the name. These remain measured retrieval failures. The lexical-set change also changed names, contents and associations. Estimating the effect of tokenization alone would require a matched intervention. A follow-on test could carry the end of an exact multi-token mention into relation lookup, with shared-prefix names and varied fact-to-value relation lengths held out. The 49 verb anchors and the other-entity selections identify additional failure routes.

## Reproduction and scope

The fit writes four banks of packed integer root prototypes after exact deduplication on the GPU. In order requested, excluded, fact, and intervening relation, the counts were 6, 6, 7 and 1. Each learned relation in TRAIN spanned one intervening token. There are 92 u32 storage slots in the four retained prototype banks, including 14 unused sentinel slots in the fixed-width relation. Each seed's trained serialized JSON was 939 bytes. The ordinary client implementation and the AOT CUDA implementation agreed on all three indices for all 1,216 predictions (304 prefixes × 2 arms × 2 artifacts). Both seeds used identical prefix IDs, so 1,216 counts device replay checks.

The exact inputs are in the panel manifest. The two fit commands and eight separate predict commands ran in 54 seconds on the owned Wick RTX 4090, including model loads, disk writes and Rust replay. The private Wick run root was `/home/callard/uor-codex/20261118-content-selector`. The binary was built from `docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml` using `--locked --target x86_64-unknown-linux-gnu --features content-selector --bin content-selector` and `UOR_NVCC_ARCH=compute_89`. With a new output directory per fit, the commands take the following arguments.

```sh
content-selector fixture
content-selector fit "inputs/seed$seed.d11" panels/train.json "$out"
content-selector predict "inputs/seed$seed.d11" "$out/$arm.json" "panels/$split-blind.json" "$out/$split-$arm.json"
```

The two arm names are `trained` and `query_ablated`. The two blind splits are `held` and `original`. The manifest covered all 12 immutable prediction/state files before grading. [Selector manifests and raw outputs](selector/outputs.sha256) · [CUDA finite trace](selector/finite/fixture.txt) · [Input and factorial audit](PANEL.md)
