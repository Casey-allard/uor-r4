# T2 — is the gold source the argmax of the read's score vector?

Instrument: 384 held-out items, 6 read layers [0, 1, 2, 3, 4, 5], 4 heads, context 384; instrumented forward logits bit-identical to the scored forward: True (max |gap| 0.000e+00).

## all read rows (every layer and head) (9216 rows, 384 items)

- gold is the argmax over sources: **0.0080**
- gold is the argmax including NoRead: 0.0031
- gold in top-3: 0.0303; top-10: 0.0728
- gold mean weight: 0.0053; p>0.5: 0.0003
- mean NoRead: 0.4055; NoRead wins the row: 0.7700
- candidates: mean 242 (nominal), effective support (w>1e-6) mean 166.7, participation ratio 40.6
- rank buckets: {'1': 74, '2': 102, '3': 103, '4-10': 392, '11-50': 2418, '>50': 6127}

## last read layer 5 (1536 rows, 384 items)

- gold is the argmax over sources: **0.0039**
- gold is the argmax including NoRead: 0.0020
- gold in top-3: 0.0280; top-10: 0.0527
- gold mean weight: 0.0049; p>0.5: 0.0000
- mean NoRead: 0.3166; NoRead wins the row: 0.7904
- candidates: mean 242 (nominal), effective support (w>1e-6) mean 168.4, participation ratio 68.3
- rank buckets: {'1': 6, '2': 31, '3': 6, '4-10': 38, '11-50': 420, '>50': 1035}

## Reference rule (non-learned)

- items: 384
- rule_source_present: 1.0
- rule_source_is_gold: 0.4557291666666667
- rule_first_content_hit: 0.6276041666666666
- rule_full_hit: 0.6276041666666666
- model_correct: 0.041666666666666664
- rule_source_is_model_argmax_last_layer: 0.0026041666666666665
- rule_source_is_model_argmax_max_over_layers: 0.0234375
