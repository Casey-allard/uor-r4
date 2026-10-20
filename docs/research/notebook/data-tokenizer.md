# Notebook: data and tokenizer

This notebook is for results about what text the model trains on and how that text is cut into tokens. It has no rows yet. The three records dated 19 October 2026 are preparation: an approved plan, a checker and a tokenizer trainer. None of them ran the model or produced a number against a bar.

## Results

Sources: `local/renorm/audit/catalog.jsonl`. Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| | *No result rows yet.* | | | | |

## Why the table is empty

Three records on this line fall on or after 12 October 2026. None is a result:

- 2026-10-19, [d41-offline-donor](../../labs/d41-offline-donor-2026-10-19/README.md) (process): the owner approved a one-time exception. A large open model (gpt-oss-120b) may generate a training corpus offline and is never used again by the student. Running it needs a separate owner release.
- 2026-10-19, [distill-d1-consumer-verifier](../../labs/distill-d1-consumer-verifier-2026-10-19/README.md) (tooling): a host-only checker that validates the planned donor corpus against its registration sheet before admission. Its 32 self-tests pass; no GPU run and no generation took place.
- 2026-10-19, [rust-bpe-trainer](../../labs/rust-bpe-trainer-2026-10-19/README.md) (tooling): a deterministic host-only Rust command to train a byte-level BPE tokenizer for a planned 16,384-token vocabulary. It does not change the active tokenizer and produces no model result.

## What a row here will need

A row needs the data and tokenizer identity (hashes), a bar fixed beforehand and a comparison on the same evaluator. A tokenizer that changes the vocabulary changes what every panel row means, so it is compared on the fixed development panel and reported with its cost.

## Go deeper

- [Data and tokenizer](../../model/data-and-tokenizer.md) and the [tokenizer mechanism](../../mechanisms/tokenizer.md).
- [Format and verdict words](./README.md).
- [`archive/CATALOG.md`](../../../archive/CATALOG.md), filtered by line `data-tokenizer`.
