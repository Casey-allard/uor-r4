# Prep sheet: the measured ~1M-token donor pilot on wick (d41-driver-math-pilot, claude-390, 15 Nov 2026)

Lab: claude. Item: [d41-driver-math-pilot](../../roadmap/items/d41-driver-math-pilot.md). Order: [claude-390](../../loop/tasks/d41-driver-math-pilot/claude-390/ORDER.md). Code: branch `claude/d41-donor-pilot`, PR #4002. Release cited at dispatch: D86 (§3.10).

TEST FITNESS: FIT - research, PR #4002, one fresh-context non-author round at `6974b14d05443ff9bdba23180840a6e45c74ac04` (claude-390)
- No blocker. Its seven should-fix findings are folded in before the run, at the head the job log names: (1) the guard reads every panel file (`.json`, `.jsonl`, `.tsv`, `.txt`, `.md`), and (2) it matches after lower-casing and dropping spacing and punctuation; (3) the LFS files are checked against Hugging Face's published sha256, and a download lands whole or not at all; (4) `/tokenize` failures are counted on their own (`tokenize_errors`); (5) the receipt binds `metrics.json` by sha256 and content, and `verify` checks it; (6) the prompt-set hash is written below; (7) the converter's log filter cannot stop the job, and the GGUF must exist. Its notes are kept too: both `verify` results are stored, and the receipt states that no consumer may weight by an answer's position or a prompt's answer count.

## The question

How fast does `openai/gpt-oss-20b` on wick produce hard answer texts, and how big is the result? The production corpus `distill-d1` is sized from these two numbers (D41 item 3, as D86 amends it). The pilot also checks the whole chain once at small scale: prompt set, engine, the kept answers, the receipt and the readback.

## Steps

1. **Setup** (`docs/labs/d41-driver-math-pilot/wick-setup.sh`, runs no model): llama.cpp `b12000` (`68d20b884c3878217de79eade0a249efde96ab38`) built with nvcc for `sm_89`, and the donor's files at revision `6cee5e81ee83917806bbde320786a8fb61efebee`, with the four LFS files checked against Hugging Face's published sha256 (all four matched on 15 Nov).
2. **Convert** once with llama.cpp's own converter at that commit: MXFP4 expert tensors kept as MXFP4, the rest in the released bf16. The GGUF's sha256 goes into the receipt.
3. **Prompt set:** `d41-pilot prompts --seed 20261115 --count 6000 --panels data/panels`, run on the laptop at the instrument's commit and copied to wick. Its sha256 is fixed here before the run: `6a6630b2eb0078c3c795b04e005da47c810113cf41d15888c7632bb96d37cca0`. The same command on wick must print the same hash. With the guard of FIT findings 1–2 it reads 32 panel files (4,267 strings), drops 199 prompts and leaves 0 hits in the set; the set is byte-identical to the one built under the first guard.
4. **Smoke** (instrument check, not corpus): `d41-pilot generate` on a 4-prompt file cut from the set, `--parallel 1`. It checks that `content` holds only the final channel (no markup) and that `/tokenize` answers. Its output is kept under `smoke/` in the store and is never part of the corpus.
5. **Run:** `llama-server -m gpt-oss-20b.gguf -ngl 99 -fa on -np 32 -c 131072 --jinja`, then `d41-pilot generate --budget 1000000 --parallel 32 --seed 20261115 --max-tokens 1024 --shard-records 512`.
6. **Seal and verify:** `d41-pilot seal` writes `receipt.json`, and `d41-pilot verify` checks it on wick, then again after a fetch back from the HF store on the laptop.

## Bar

The pilot is a measurement. Its bar decides whether its two numbers can size production.
- **B1 size:** generated tokens ≥ 1,000,000, so the budget stop was reached.
- **B2 receipt:** `verify` passes after the round trip through the store. That covers the shard sha256s, the Merkle root, the prompt-set hash, `metrics.json`'s sha256, each record's messages against its prompt, and the structural check that no record carries any field but `id`, `kind`, `messages` and `answers`.
- **B3 quality floor:** truncated or markup answers ≤ 5% of requests; prompts with at least one kept answer ≥ 95% of those dispatched; request errors ≤ 1% of prompts; `tokenize_errors` = 0, so the kept-token count that sizes production is whole.
- **B4 panels:** the prompt set holds no panel string (the guard drops them; the dropped count is reported).

**Recorded, with no bar:** generated tokens/s and kept-answer tokens/s (donor tokenizer), kept bytes, the distinct-answers histogram, request p50/p95 seconds, and the kind mix.

**Reading map** (fixed before the run):
- **P:** B1–B4 hold. The REPORT sizes production as: hours for N kept tokens = N / (kept-answer tokens/s) / 3600, at N = 100M and 1B. It also gives the storage per million kept tokens.
- **I (instrument):** B2 or B4 fails. The instrument is repaired. The numbers are not used for sizing until a new sheet's run passes.
- **Q (quality):** only B3 fails. The corpus is still sealed and stored. Production starts only after the cause (truncation length, markup, prompt form) is fixed in its own sheet.
- **S (short):** only B1 fails. The REPORT gives the cause. The rates are still reported, marked as measured on fewer than 1M tokens.

## Base artifact

- **Donor:** `openai/gpt-oss-20b` revision `6cee5e81ee83917806bbde320786a8fb61efebee` (Apache-2.0, not gated), released MXFP4 weights.
- **Engine:** llama.cpp `b12000`, outside the repository (§3.4).
- **Instrument:** `docs/labs/d41-driver-math-pilot/instrument` at the commit the job log names. It is a client and holds no model code.
- **Machine:** wick, NVIDIA GeForce RTX 4090, UUID `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, driver 595.99.02, in Claude's container (`local/claude/wick/wick.sh`).

## Data split

- Every prompt comes from the seeded templates, so none comes from TRAIN, DEV or any panel.
- The prompt and each of its sentences are checked against every string in `data/panels/*.json` and `*.tsv`, and a match is dropped.
- The corpus is training data only. It is never a panel, a grader or a comparator (D41 (c)).

## Minimum test power

- This is not a decisive mechanism run, so there is no mechanism claim and no seed requirement.
- The rates come from about 8,000 requests, and each is reported with its request p50 and p95.
- One run, seed 20261115.

## Cost

Wick is the owner's own GPU, so there is no rental cost (D86). Estimated wick time: about 30 minutes of setup and conversion. At an assumed 1–3k generated tokens/s, the run takes about 6–17 minutes. No pod, and nothing near the §5.4 caps. Wick is shared, so `wick.sh gpu` is read first, and the run leaves at least 3 GB free.

## Stop rule

- Dispatch stops at 1,000,000 generated tokens, and the prompts in flight finish.
- Hard stop: if the run passes 3 hours of wall time, the lab stops its own job (by its PID) and reports S with the partial numbers.
- One pass only (D41 (f)). A failed run is reported and is not rerun under this sheet.
