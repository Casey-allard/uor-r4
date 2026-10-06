# T2 — is the gold source the argmax of the read's score vector?

Instrument: 1536 held-out items, 6 read layers [0, 1, 2, 3, 4, 5], 4 heads, context 384; instrumented forward logits bit-identical to the scored forward: True (max |gap| 0.000e+00).

## all read rows (every layer and head) (36864 rows, 1536 items)

- gold is the argmax over sources: **0.0292**
- gold is the argmax including NoRead: 0.0077
- gold in top-3: 0.0699; top-10: 0.1565
- gold mean weight: 0.0054; p>0.5: 0.0002
- mean NoRead: 0.6228; NoRead wins the row: 0.8452
- candidates: mean 244 (nominal), effective support (w>1e-6) mean 150.0, participation ratio 38.5
- rank buckets: {'1': 1075, '2': 859, '3': 644, '4-10': 3193, '11-50': 10422, '>50': 20671}

## last read layer 5 (6144 rows, 1536 items)

- gold is the argmax over sources: **0.0114**
- gold is the argmax including NoRead: 0.0000
- gold in top-3: 0.0438; top-10: 0.1576
- gold mean weight: 0.0047; p>0.5: 0.0000
- mean NoRead: 0.5984; NoRead wins the row: 0.9933
- candidates: mean 244 (nominal), effective support (w>1e-6) mean 163.1, participation ratio 63.0
- rank buckets: {'1': 70, '2': 100, '3': 99, '4-10': 699, '11-50': 1674, '>50': 3502}

## Reference rule (non-learned)

- items: 1536
- rule_source_present: 1.0
- rule_source_is_gold: 0.4505208333333333
- rule_first_content_hit: 0.6263020833333334
- rule_full_hit: 0.6263020833333334
- model_correct: 0.03125
- rule_source_is_model_argmax_last_layer: 0.01171875
- rule_source_is_model_argmax_max_over_layers: 0.22721354166666666
