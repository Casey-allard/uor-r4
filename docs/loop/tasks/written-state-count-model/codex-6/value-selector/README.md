# [codex] Value address selection and ordered connector registration

The bound D11 models, #4087 role states, #4089 span states, token classes and learned geometric connector bank are unchanged. The new state stores only the observed connector lengths and IDs. AOT CUDA fit deduplicates each TRAIN slice between the annotated fact end and value start, retaining first observation order. The six sequences occupy 24 u32 words (96 bytes including padding). Both fits have the same connector records. The state envelopes bind each to its own model, span state and training data. Fit and blind inference consumed no held-out grade file.

Blind prediction recomputes the frozen span from bare tokens with Rust and checks all seven words against #4089. The device receives only the immutable prompt region defined by the frozen span; previously unavailable sources retain a null result. AOT CUDA and serving Rust independently produce four sources and three bitmasks indexed by connector length minus one. Current reproduces the original geometric ranking. WordStart filters its length candidates to the tokenizer's ASCII alphabetic START class, then applies the original ranking and longer-length tie. Ordered compares all IDs of each candidate to the six learned sequences and selects the longest matching length. Both takes the longest in the intersection of ordered and word-start eligible lengths. An ordered match may be a valid short prefix of a longer unregistered connector. The encoder's positions and first six frozen words accompany each emitted choice group.

## Conditional source diagnosis

These counts grade the sealed source choices before consulting emitted tokens. Both released seeds give the same source choices. For the following table, all four frozen entity endpoints must be correct. “Split-specific” refers to the three primary held-out connectors that share no complete sequence with TRAIN.

| Primary surface / length | Named spans correct | Current | WordStart | Ordered | Both |
|---|---:|---:|---:|---:|---:|
| TRAIN-shared, 1 ID | 109 | 109 | 109 | 109 | 109 |
| TRAIN-shared, 2 IDs | 103 | 49 | 49 | 103 | 103 |
| TRAIN-shared, 3 IDs | 84 | 36 | 36 | 84 | 84 |
| Split-specific, 1 ID | 127 | 123 | 123 | 0 | 0 |
| Split-specific, 2 IDs | 105 | 0 | 60 | 0 | 0 |
| Split-specific, 3 IDs | 99 | 0 | 0 | 0 | 0 |

On the primary two-ID “chose” rows, WordStart selected 60 requested values and overshot the other 45 by one ID, selecting a legal word start within the multi-word value. On all 99 three-ID “also chose” rows it backed up to ID 641, the legal word start of “ chose”, which lies two tokens before the value. On the fresh panel with both entity spans correct, Ordered and Both selected the requested value for all 483 complete TRAIN connectors: 349 from the shared “has” family and 134 with the one-ID connector “found”. Unknown extensions of a shorter learned prefix yield 0/152 and 0/96 for those arms at lengths two and three. On wholly unseen fresh three-ID connectors, WordStart reaches 142/142 correct starts versus 36/142 for Current. The hard ordered arm abstains. Ordered and Both make identical choices across every primary and fresh row. The all-case comparison and first-error trajectories are in the unit REPORT.

## Run and replay

Build with feature value-selector using nvcc on Wick's RTX 4090. The fit-start/fit-end files cover both artifact loads, CUDA fits, native comparison and serialization (12 seconds). Historical blind prediction took 34 seconds and fresh prediction took 12 seconds. A separate process for each repeated panel reproduced all eight prediction files byte for byte. The comparison covers 7,520 query/seed rows with seven independently checked u32 source and mask words each. The two independent state files and eight primary JSONs are identified by outputs.sha256; replay/verified.sha256 identifies the eight repeat files. The retained Wick directories are results/value-01 and results/value-repeat under /home/callard/uor-codex/20261118-value-connectors.

```sh
value-selector fixture
value-selector fit MODEL SPAN_STATE TRAIN OUT_STATE
value-selector predict MODEL CONTENT_STATE SPAN_STATE OUT_STATE BARE_PREFIXES SPAN_PICKS OUT
```
