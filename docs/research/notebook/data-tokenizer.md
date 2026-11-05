# Notebook: data and tokenizer

This notebook is for results about what text the model trains on and how that text is cut into tokens. Its first row is the real-text corpus of 5 November 2026. The three records dated 19 October 2026 are preparation: an approved plan, a checker and a tokenizer trainer. None of them ran the model or produced a number against a bar.

## Results

Sources: [`archive/catalog.jsonl`](../../../archive/catalog.jsonl). Verdict words are defined on the [format page](./README.md#the-five-verdict-words).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-11-05 | [claude] **The real-text corpus** (real-text-corpus, claude-293; done-when 2–5; data preparation on the laptop, no model and no GPU). `prepare-real-text` at `9911a2e1c` built four roots from the stored raw sources: OASST2 in full, and Wikipedia shards 00000, 00020 and 00040, each cut to about 8M TRAIN tokens. Settings: the pinned tokenizer, protocol 2, a 5% held-out split by source id, and the exclusion screen over nine panels. The five frozen panels' user turns were read back by the new `panel-turns`; its turns equal p-long's and p-long4k's stored files (8,474 and 17,180). Then `train-in-space --census 1` ran once per TRAIN split. | **TRAIN `27,968,113` tokens:** OASST2 `3,773,419`; Wikipedia `8,011,538`, `8,255,877` and `7,927,279`. Held out `1,491,618`. 56 documents excluded (51 of them OASST2), 0 dropped, 2.35–2.61 bytes per token. **Exact prompt share for P-dev, p-long and p-long4k: 0.0 against all four splits.** DEV `0.0351` and Fresh V2 `0.0407` against OASST2 only: six generic phrases under four words ("Thank you!", "Hello!", "Thanks!", "What are you?", "Who are you?", "How are you?"), in 33 of 5,071 threads. A Wikipedia split has no prompt runs, so it reads 0.0 by construction. Census peak 2.5–6.1 GB. | The ORDER's, fixed before the count: exact prompt share 0.0 for every frozen panel, or each nonzero share explained by the documents behind it; turns read back equal to the panels' stored turns | **positive** at this scope. The corpus exists and is screened. DEV's and Fresh V2's shares are the six phrases, kept and recorded; a scorer reports those rows apart. Wikipedia was cut from 15,000 articles a shard to 900–8,400 by the ORDER's stop rule: the census needs 0.67–0.75 GB per million real-text tokens | Store `results/claude/real-text-corpus-2026-11-05/` (README, `SHA256SUMS`, read back); [REPORT](../../loop/tasks/real-text-corpus/claude-293/REPORT.md) |

## Earlier records, which are not results

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
