# External teacher referee — issue #300

The D3 teacher floor was independently measured outside the r4 observation,
corpus-record, and compiled-artifact pipeline.

## Run

```bash
uv run --with torch --with transformers --with safetensors --with blake3 \
  python scripts/external_referee.py \
  --model .uor-models/sources/smollm2-135m-instruct \
  --corpus .uor-models/corpora/simple-wiki-20231101/articles.jsonl \
  --out /tmp/i300-external-referee.json
```

The script applies the canonical D3 split directly to article IDs, tokenizes
raw article text with the local Hugging Face tokenizer, keeps the first 128
tokens of each held-out article, and scores the next-token labels with
`AutoModelForCausalLM`. It does not read `corpus.records`, observation
vectors, stores, graphs, or score reports. No BOS token or chat template is
added.

## Result

| measurement | value |
| --- | ---: |
| held-out articles | 596 |
| scored target tokens | 71,714 |
| external bits/token | **3.8425** |
| r4 story-contiguous floor | 11.17 |

The external teacher is approximately 7.33 bits/token better than the r4
floor on the same D3 raw-text distribution. This satisfies the issue's
decision rule: the residual floor is pipeline-internal, not an intrinsic
SmolLM2 quality bound. The next investigation should focus on r4's
tokenization/segmentation and teacher-forcing surface, not on reinterpreting
the model floor.
