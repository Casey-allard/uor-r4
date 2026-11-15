# REPORT: d41-driver-math-pilot, claude-390 (the measured ~1M-token donor pilot on wick, D86)

Lab: claude. Item: [d41-driver-math-pilot](../../../../roadmap/items/d41-driver-math-pilot.md) (#2821). ORDER: [ORDER.md](ORDER.md). Sheet: [d41-driver-math-pilot-2026-11-15](../../../../harness/prep/d41-driver-math-pilot-2026-11-15.md) (FIT). Release cited at dispatch: D86 (§3.10).

The sheet's reading map says **P**: B1–B4 all hold. The pilot corpus is sealed, stored and verified by readback. These are measurements of the data pipeline only. Under D41 (e), they make no claim about the model.

## What did not work, first

- **The donor is about ten times slower than the review assumed.** The measured rate is **503.6 generated tokens/s** and **220.7 kept-answer tokens/s**. The platform review's estimate was "about 6 h / ≤$20 for 100M tokens" at about 5k tok/s (arch-plat.md; marked there as a hypothesis, for a 1–3B model). At the measured kept rate, 100M kept tokens take **126 hours (5.2 days)** of wick, and 1B take **52 days**. This configuration does not make a 1B-token corpus practical. See "Still open" item 1.
- **Most generated tokens are discarded.** Only 43.8% of generated tokens are kept answer text (440,146 of 1,004,450). The rest is the analysis channel, which gpt-oss emits even at `reasoning_effort: low`, plus channel tokens.
- **The GPU was not saturated.** Utilization averaged 47% (0–74%) over 67 samples, with 16.3 of 24.6 GB in use. Prefill was not the cause: the prefix cache meant only 160,502 of 2,746,816 prompt tokens were actually prefilled (5.8%). With 32 requests in flight and a p50 of 2.92 s per ~49-token request, the rate matches llama.cpp's batched MoE decode at 32 slots as the limit. This is an inference from the timings; nothing in this unit isolated it.
- **The first setup job ended rc=1 from its last diagnostic line**, not from a real failure. The gpt-oss converter code moved to `conversion/gpt_oss.py` at `b12000`, so the closing `grep` matched nothing, and `pipefail` failed the job. The rerun (`d41-setup-2`) passed with rc 0, including the four LFS checks.
- **The prompt templates are narrow.** By prompts kept, the mix is 68.7% grounded, 30.7% open and 0.6% small talk (3,511 / 1,571 / 30 of 5,112), against the 45/45/10 draw, because duplicates were skipped. The 6,000-prompt set skipped 3,297 duplicate candidates. Small talk has only 30 texts, and single-noun open frames allow only 600 combinations.
- **The corpus cannot be regenerated bit for bit.** Batched decoding makes sampled outputs depend on batch composition. The corpus is fixed by its hashes and Merkle root instead, which is what D41 (b) asks.

## What changed

- **`docs/labs/d41-driver-math-pilot/instrument/`** (new standalone Rust package `d41-donor-pilot`; it holds no model code). Its binary `d41-pilot` has four commands:
  - `prompts` builds the seeded prompt set. It drops any prompt whose text, or one of whose sentences, matches a string in any `data/panels/` file after lower-casing and removing spacing and punctuation.
  - `generate` asks `llama-server` for one greedy answer and three answers sampled at T=1.0, top-p 1.0, each with its own seed. It keeps only final-channel texts that end on `stop` and hold no channel markup. Distinct texts are stored in byte order, at most four per prompt. It stops dispatching at the token budget or the time limit.
  - `seal` writes the receipt.
  - `verify` re-hashes the shards, the Merkle root, the prompt set and `metrics.json`. It checks each record's messages against its prompt, and checks that each record has exactly the keys `id`, `kind`, `messages`, `answers`.
- **`wick-setup.sh` and `wick-run.sh`** are the recipe, run in Claude's wick container:
  - llama.cpp `b12000` (`68d20b884c3878217de79eade0a249efde96ab38`), built with nvcc for sm_89 (setup);
  - the donor files at `6cee5e81ee83917806bbde320786a8fb61efebee`, checked against Hugging Face's LFS sha256 (setup);
  - one conversion with `--outtype bf16`, which repacks the MXFP4 experts without requantizing them (run);
  - the prompt-set reproduction check, the smoke, the run, the seal and the verify (run).
- **After the run, number formats only.** `loop pr-check` counts every float type and float literal in a `docs/labs/*/instrument` as a D32 site (§3.4). So at the head the instrument records timings as integer milliseconds and rates as integer thousandths of a token per second. Its request sends the same sampling values as JSON integers (temperature 0 or 1, top-p 1, min-p 0). Two test fixtures use integer values. The run itself used `74e415803`, so the stored `metrics.json` keeps that commit's field names (`wall_seconds`, `request_seconds`, `*_tokens_per_second`). Prompt building, the record format, `seal` and `verify` are unchanged.
- **The sheet** with its FIT. The FIT found no blocker and seven should-fix findings, and all seven were folded in at `74e415803` before the run.
- **The notebook row** in `docs/research/notebook/data-tokenizer.md`, and the item set to `done`.
- **Not committed:** the corpus, the weights, the GGUF and the engine. The corpus is in the HF store at `results/claude/d41-pilot-2026-11-15/`, with `out/` (corpus, receipt, metrics, run, prompts), `meta/` (identity files, server and GPU logs, both verify results, the job log) and `smoke/`.

## Measurements

The run was job `d41-run` on wick, at instrument commit `74e415803c3028e2e7d1da17ea931143ea60eae7`, from 2026-11-15T12:48:33Z to 13:21:48Z (1,994.6 s).

| Quantity | Value |
|---|---|
| Generated tokens (donor tokenizer, analysis channel included) | **1,004,450** (B1 bar 1,000,000) |
| Generated tokens/s, aggregate | **503.6** |
| Kept answer tokens / bytes | **440,146** / 1,850,532 |
| Kept answer tokens/s | **220.7** |
| Prompts dispatched / records kept | 5,112 / 5,112 (a contiguous prefix of the 6,000-prompt set) |
| Answers kept | 17,711 (3.46 per prompt) |
| Prompts by distinct answers 1 / 2 / 3 / 4 | 298 / 609 / 625 / 3,580 |
| Requests; truncated; markup; errors; tokenize errors | 20,448; 1; 1; 0; 0 (B3: 0.01% dropped, 100% of prompts kept) |
| Request seconds p50 / p95 / max | 2.92 / 5.26 / 63.9 |
| Generated tokens per prompt, grounded / open / small talk | 173 / 249 / 161 (3,511 / 1,571 / 30 prompts) |
| Panel guard | 32 files, 4,267 strings, 199 prompts dropped, 0 hits in the set (B4) |
| "Not told" rows | 847 prompts. 3,185 of their 3,357 answers say they were not told (94.9%, phrase match, a lower bound). |
| Corpus shards | 10 files, 3,891,805 bytes; Merkle root `a93e9b2336c45db44d49a76ae4af8928a2f10f7c9df8cb3f2a768e4fb166d5eb` |
| Receipt sha256 | `5dfdef74559fdf10a362d4f5273f781b3e58441e57fbf9f5e096eb4033247666` |
| Prompt set sha256 | `6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0` (seed 20261115) |
| GGUF sha256 | `529e7bd6dfe34473c9777e8c4ed34bb7580c614e04d27faa4f910c4143e43503` (13,792,638,656 bytes) |
| GPU | NVIDIA GeForce RTX 4090, `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, driver 595.99.02; 16.3 GB used; utilization mean 47% |

**Sizing production (the sheet's P arithmetic).** Hours for N kept tokens = N / 220.7 / 3,600.

| Kept answer tokens | wick time at the measured rate | Stored size at 8.84 bytes per kept token |
|---|---|---|
| 1M | 1.26 h | 8.8 MB |
| 100M | 126 h (5.2 days) | 0.88 GB |
| 1B | 1,259 h (52 days) | 8.8 GB |

The stored size counts whole records, prompt and system message included. Each record repeats the system message, so a production format that stores it once would be smaller.

## Checks run at the head

- **Laptop (`aarch64-apple-darwin`), at `74e415803`:** `cargo test --release` in the instrument passes 8 tests, `cargo clippy --release --all-targets` gives 0 warnings, and `cargo fmt` is applied. A `seal`/`verify` round trip on a two-record fixture passes. A changed shard byte and a changed `metrics.json` each fail `verify` (rc 1). Both scripts pass `bash -n`. `uor-harness prep-check` shows the sheet as FIT with every block present.
- **Laptop, at the head (the number-format change):**
  - `cargo test --release` passes 8 tests and clippy gives 0 warnings.
  - The head binary rebuilds the prompt set to the same sha256 `6a6630b2…`.
  - It re-verifies the fetched-back corpus: 5,112 records, 17,711 answers, root `a93e9b23…`, verified.
  - The head's `generate` has not been run against the engine. The integer sampling values are untested there.
- **wick, job `d41-setup-2`:** rc 0, and `sha256sum -c` gives OK for the three safetensors files and `tokenizer.json`.
- **wick, job `d41-run`:** rc 0. The prompt set was rebuilt on wick to the same sha256 and was byte-identical (`cmp`). The smoke passed: 4 records, 0 markup, 0 errors. `verify` returned `{"records":5112,"answers":17711,"verified":true}` with the root above.
- **Readback:** `uor-store get` of `out/` to a fresh folder, then `d41-pilot verify` on the laptop, gave the same records, answers and root (`meta/verify-laptop.json`).
- **The sheet's FIT:** a fresh-context non-author review at `6974b14d0` returned `TEST FITNESS: FIT`.
- **Pre-run review (§5.10), in this session:**
  - The chain is D86, then this pilot, then production sizing.
  - Every input existed: the built engine, the hash-checked donor files and the fixed prompt set.
  - P sizes production, and I, Q and S each name a different fix.
  - The sheet had its FIT.
  - Wick was idle (4 MiB used), at $0, with no pod and nothing near the caps.

## Skills used

- The loop's ORDER/REPORT form, with a one-page sheet and a fresh non-author FIT.
- The wick kit (`local/claude/wick/wick.sh`: put, job, wait, tail, get), with completion watchers armed.
- `scripts/pod/uor-store` for the store upload and readback.
- `native-space-cycle`: not applicable. This unit made training data and touched no model state.

## Still open

1. **Throughput is the production unit's first question.** Before production, a short sheeted probe on wick should measure the untested levers, each a hypothesis:
   - more slots (`-np 64` or 128) and larger batch flags (`-b`, `-ub`);
   - a shorter system message;
   - no per-answer `/tokenize` call;
   - asking for all four answers in one request;
   - a different external engine, such as vLLM, which is Python and would need to be named in the receipt as this one is.

   Production is sized only after that probe.
2. **No count in the project's tokenizer.** All token counts here use the donor's tokenizer (o200k-harmony). The student trains in the 4,096-token tokenizer (`d36d3e87…`), so its token count for this corpus is not measured.
3. **Single-turn prompts from a narrow template bank.** Production needs a larger prompt bank, and two-turn exchanges if they are wanted, designed in its own sheet.
4. **Answer quality is not graded.** That is by design (D41 (c), (e)). The "not told" phrase match is a description, not a bar.
5. **The receipt does not name the instrument commit.** The sealed corpus came from `74e415803`, and its `run.json` and `metrics.json` carry that commit's field names. The head client writes integer fields under other names, and its request (integer sampling values) has never been sent to `llama-server`. The production unit should add the instrument commit to the receipt and run the head `generate` on a smoke before the corpus pass.
6. **Disk on wick.** Claude's container still holds the donor files (13.8 GB), the GGUF (13.8 GB), the venv and the llama.cpp build for the production pass. Wick has about 190 GB free. Remove them with `wick.sh sh` (`rm -rf /workspace/d41/donor-hf /workspace/d41/gpt-oss-20b.gguf`) once the production corpus is sealed.

## Plain summary

The donor, gpt-oss-20b, ran once on Casey's own RTX 4090, outside the repository, and wrote about a million tokens. It answered 5,112 simple questions, with up to four answers each and 17,711 in all. Only the answer texts were kept: no probabilities, scores or rankings. They are sealed with a receipt and stored in the project's Hugging Face store, and the copy fetched back checks out exactly. The surprise is speed: about 221 kept tokens per second, about ten times slower than the architects' estimate. A 100-million-token corpus would take about five days of wick at this rate. The next step is a short throughput test of the server settings before the full corpus is sized.
