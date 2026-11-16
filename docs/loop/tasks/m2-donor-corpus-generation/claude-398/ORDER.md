# ORDER: m2-donor-corpus-generation, claude-398 (the donor throughput probe on wick, D86)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md) (#2848). This unit references the item and does not close it.
Base: `origin/main` at `6f99bbaa8`, which holds claude-390 (#4002), the sealed ~1M-token pilot.

## Why this unit

The pilot (claude-390) measured gpt-oss-20b on wick at **503.6 generated and 220.7 kept-answer tokens/s**. That is about ten times below the review's hypothesis, so 100M kept tokens would take about 126 hours of wick. Its REPORT ("Still open", item 1) makes throughput production's first question: production is sized only after a short probe of the untested levers. This unit is that probe. Production itself is a later unit with its own sheet.

The pilot's server log points to a likely cause. With 32 slots busy, a decode step took about 55 ms, while each answer had ~49 generated tokens and the GPU averaged 47%. The pilot sends `top_k 0`, so the engine's CPU sampler handles the full 201k-token vocabulary for every slot at every step. This is a hypothesis from the log. The probe tests it with one extra diagnostic arm, llama.cpp's own `--backend-sampling`, beside the five levers the REPORT names.

## The release this unit cites at dispatch (§3.10, §5.1)

D86 (Casey, 15 Nov 2026, in `docs/integration/DECISIONS.md` and the 15 Nov entry of `docs/rules/changes.md`): "I would say yes release it, but why is it going to cost money? if we are running it locally on wick, wouldn't it be free?"; "Allow up to 4 (Recommended)"; "Defer (b), ban (c)(d) (Recommended)"; "gpt-oss-20b on wick (Recommended)"; "Drop driver math, keep pilot (Recommended)". The probe is part of that donor pass: it uses the same donor, engine, GGUF and prompt set on wick, and it produces no corpus.

## The work

1. **The instrument** (`docs/labs/d41-driver-math-pilot/instrument/`) gains, at the head:
   - `--instrument-commit SHA` on `generate`, required and written into `run.json`, so every receipt names the client commit (pilot REPORT, item 5);
   - `--mode seq|n4|g-n3`: four requests per prompt as now; one request with `n` = 4; or one greedy request plus one with `n` = 3, which keeps the pilot's seed rule because llama.cpp seeds child completions `seed + k`;
   - `--tokenize 0`, which skips the per-answer `/tokenize` call, and a `count` command that tokenizes the kept answers of a finished directory outside the timed window;
   - `--system-file FILE`, which replaces the system message with the text of FILE.
   The pilot's sampling values, record format, `seal` and `verify` are unchanged.
2. **The smoke first:** the head `generate` runs on 4 prompts in each mode against `llama-server` before any timed arm. Its integer sampling request has never been sent to the engine (pilot REPORT, item 5).
3. **The timed arms on wick**, each a 300 s dispatch window from the start of the pilot's fixed prompt set (`6a6630b2…`): the pilot's settings, re-measured with the head client; `-np` 64 and 128 with larger `-b`/`-ub`; a shorter system message; no `/tokenize`; four answers in one request (`n4` and `g-n3`); backend sampling; and one arm combining the levers that won. vLLM is optional and is named in the receipt if run.
4. **Storage:** each arm's `metrics.json`, `run.json`, server log and GPU log go to the HF store under `results/claude/d41-throughput-2026-11-15/`. The probe's answer texts are stored beside them, marked as probe output: not corpus, not for training.

## Owner terms and lab conditions (D86), and how each is met

- **Donor and machine:** gpt-oss-20b at `6cee5e81`, the pilot's GGUF, on wick. Nothing else.
- **Hard answer texts only:** the probe requests and stores no logits, probabilities, scores or ranks. Backend sampling changes where the engine samples, not what is requested or stored.
- **Outside every compiled module (§3.4):** the engine stays in Claude's wick container. The instrument is a client with no model code.
- **No corpus:** nothing from the probe is sealed as `distill-d1` or used to train.

## Sheet

`docs/harness/prep/d41-donor-throughput-2026-11-15.md`, checked by `uor-harness prep-check`, with a fresh-context non-author FIT before the timed arms (§4.5).

## Native-space five items (SKILL)

Not applicable. This unit measures the throughput of an external engine and touches no model state, operator, optimizer, carry, memory or artifact. It is an inventory-class unit.

## Cancel probe

At base, no measurement exists of any lever beyond the pilot's single setting (`-np 32`, four requests per prompt, `/tokenize` per answer). `uor-store ls results/claude/` holds no `d41-throughput` folder.

## Files in scope

- `docs/labs/d41-driver-math-pilot/instrument/**` and `docs/labs/d41-driver-math-pilot/wick-throughput.sh` (new)
- `docs/harness/prep/d41-donor-throughput-2026-11-15.md`
- this folder
- `docs/research/notebook/data-tokenizer.md` (one row)

## Stop list

No production corpus and no sealed `distill-d1` output. No student training, evaluation or serving. Nothing in the repository reads a donor weight, layer, hidden state or logit. No pod. No other user's process on wick is touched; only this job's own PIDs are stopped.

## Done when

1. The REPORT gives the best measured kept-answer tokens/s and its settings, and sizes the production pass in hours of wick for 100M kept tokens from that number.
2. Each lever's measured rate is reported against the re-measured baseline, with the negatives first.
3. The head `generate` passed its smoke against `llama-server`, and `run.json` names the instrument commit.
4. The notebook row is written, the PR has a fresh exact-head review, and it goes through `uor-harness loop queue --pr N`.
