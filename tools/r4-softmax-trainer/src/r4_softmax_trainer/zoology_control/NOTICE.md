# Zoology source attribution and modification notice

Portions of this directory are adapted from
[HazyResearch/Zoology](https://github.com/HazyResearch/zoology) at the ICLR24
release commit
[`de4e258784224e09909c257ff3ea040f089ed660`](https://github.com/HazyResearch/zoology/tree/de4e258784224e09909c257ff3ea040f089ed660).
The upstream work is licensed under the
[Apache License, Version 2.0](https://github.com/HazyResearch/zoology/blob/de4e258784224e09909c257ff3ea040f089ed660/LICENSE.md).
The complete upstream license is redistributed beside this notice as
[`LICENSE-APACHE-2.0.md`](LICENSE-APACHE-2.0.md).

Copyright 2021 The Meerkat Team.

The pinned upstream tree contains `LICENSE.md` and no separate upstream NOTICE
file. Redistribution of source-derived code remains subject to Apache-2.0,
including retention of applicable copyright, patent, trademark, and
attribution notices. The upstream work is provided without warranties or
conditions except as stated in that license. Reference to Zoology identifies
origin and does not imply endorsement.

## Upstream material adapted

- `zoology/mixers/attention.py`: combined biased Q/K/V projection, one-head
  scaled causal softmax, attention dropout, value aggregation, and biased
  output projection.
- `zoology/model.py`: learned token/absolute-position embeddings, released
  `TransformerBlock` residual and LayerNorm flow, identity state mixer, tied
  output embedding, and released initialization including the second
  model-level initialization pass.
- `zoology/data/associative_recall.py`: `_mqar` key/value serialization,
  power-law query placement, shifted labels, zero-filler option, and seed
  separation.
- `zoology/data/utils.py` and `zoology/train.py`: shuffled tensor loading,
  AdamW defaults, query-masked cross-entropy/accuracy, and cosine scheduling.
- `zoology/experiments/paper/figure2.py`: the selected attention configuration,
  width, learning rate, dropout, layer count, sequence length, vocabulary, and
  K/V-pair settings.

The later Zoology commit
[`1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb`](https://github.com/HazyResearch/zoology/tree/1ad20d193b6113cae1e8f3c655c300d7b4b3f4bb)
is recorded for #1045 provenance only. It is not the executable source oracle.

## UOR-R4 modifications

The files in this directory are changed/adapted for the bounded
`ZoologyMQARControlV1` experiment. The declared changes are:

- device-neutral, CPU-only construction and execution in place of CUDA device
  assumptions;
- vocabulary projection only at labelled query positions, preserving the
  labelled logits and masked query loss while bounding CPU memory;
- deterministic release-style shuffled ordering;
- create-once UOR provenance, work-ledger, artifact, and result containers;
- a scaled 8,192/1,024 source-native calibration with batch 64 rather than the
  published 100,000/3,000 Figure 2 experiment; and
- a second control using #1045's exact open token rows and query targets, with
  categorical role bytes checked only for provenance and never supplied to the
  copied model.

These adaptations are not represented as byte-identical Zoology code or as a
full reproduction of the published Figure 2 sweep. Source-derived Python files
in this directory must retain a prominent pointer to this notice and the
upstream Apache-2.0 license.
