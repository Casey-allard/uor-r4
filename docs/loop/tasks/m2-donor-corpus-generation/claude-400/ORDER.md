# ORDER: m2-donor-corpus-generation, claude-400 (D87's first sealed donor corpus: 1B kept tokens from DeepSeek V4.1 Flash, reasoning kept)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md) (#2848). This unit references the item. It closes the item only if its "Done when" is met.
Base: `origin/main` at `6f99bbaa8`. D87 is in PR #4027 (`claude/d87-deepseek-donor`) and is not yet on `main`. Nothing is leased until #4027 is merged on `main`.

## Why this unit

The pilot (claude-390) sealed about 1M tokens from gpt-oss-20b on wick. It measured 220.7 kept tokens/s, so 1B kept tokens would take about 52 days. D87 moves the production pass to rented GPUs, uses a stronger teacher, and keeps the teacher's reasoning text. This unit produces that first corpus.

## The release this unit cites at dispatch (§3.10, §5.1)

D87 (Casey, 16 Nov 2026, recorded `2026-11-16T02:37:46Z`): "gpt-oss-120b on B200s (Recommended)"; "1B first as suggested above"; "Raise to $35/h, 4 pods (Recommended)"; "if the small models cant offer real intelligence close to claude opus4.6 reasoning, then we need to focus on a model that can"; then, to "Start the first 1B tokens with DeepSeek V4.1 Flash, reasoning traces kept?": "Yes, start it". This unit cites D87 only once it is on `main`.

## Pinned inputs (checked 15 Nov 2026)

- **Donor:** `deepseek-ai/DeepSeek-V4.1-Flash-1017`. The model card says MIT, and the repo's `LICENSE` is the MIT text. It is about 511 GB on disk in 48 safetensors shards: MXFP4 experts, FP8 dense weights and 183 GiB of FP8 Engram tables. It has 552B backbone parameters plus 196.6B of Engram rows. The exact commit is pinned in the sheet and the receipt.
- **Engine:** vLLM 0.31.0 or later (the release image, named by digest in the receipt). It serves `DeepseekV41ForCausalLM` with `--reasoning-parser deepseek_v41`, so a reply's reasoning and final answer arrive as separate text fields. On H200 the vLLM recipe runs TP4 per replica with the Engram tables offloaded to host RAM, so one 8×H200 pod runs two replicas.
- **Pod:** Vast's cheapest verified 8×H200 offer was $34.25/h on 15 Nov (about 2 TB of RAM, 6 TB of disk and 8 Gb/s, in Iceland). That is the only shape found under D87's $35/h cap. Two 8×H200 pods would cost about $68/h, so the cap allows **one pod, not four**. 8×B200 was $86/h and 4×B200 was $43.5/h.
  - **Amended 16 Nov:** that offer was gone at lease time, and the cheapest 8×H200 was $46.16/h, over the cap. The run uses one 4×H200 (SXM) instead, which is the recipe's one-replica shape (TP4, Engram tables in host RAM). It was $23.08/h for offer 45686955 (reliability 0.995). The cap buys about one replica, so this unit does not fan out. If the measured hour misses a bound, the REPORT gives the fleet the cap would need.

## The work

1. **Instrument** (`docs/labs/m2-donor-corpus/instrument/`, a new standalone Rust package with no model code, like the pilot's):
   - `bank` builds the seeded prompt bank from the licensed sources, with the pilot's panel guard;
   - `generate` is a vLLM OpenAI-API client. It keeps `reasoning` and `answer` as two separate hard-text fields, resumes from finished shards, and logs the rate each minute;
   - `seal` and `verify` cover the Merkle receipt, which names the instrument commit, the engine image digest and the GPUs;
   - `count` counts kept tokens in the project's 4,096-token tokenizer.
2. **Prompt bank:** about 600k–900k prompts, mostly single-turn with about 10% two-turn exchanges. Every assistant turn is the teacher's own. The sources and their licences are registered in the sheet. Only user turns are taken from outside data.
3. **Smoke** on the leased pod: 32 prompts through the head `generate` before the timed hour.
4. **One measured hour** on one pod. Projected cost and time for 1B kept tokens are taken from that hour plus the setup time already spent. If the projection is over $1,000 or over 48 h, the unit does not switch donors: the pod is stopped, the measured hour is saved and sealed as a partial set, and the unit reports the numbers with its fallback recommendation (gpt-oss-120b on B200s) for the parent session to put to Casey. Casey chose DeepSeek V4.1 Flash and chose no fallback (parent correction from the exact-head review of #4027, head 6751be0bd).
5. **Run to 1B kept tokens.** The rate cap is `UOR_POD_MAX_RATE=35` for this run only. A completion watcher is armed. Shards are copied off the pod continuously and before `down`.
6. **Seal:** the receipt, upload to the HF store, readback verify, the REPORT (negatives first), and a `data-tokenizer` notebook row.

## Owner terms and lab conditions (D87), and how each is met

- **Teacher:** DeepSeek V4.1 Flash, MIT, pinned to an exact checkpoint, licence read on the card. Recommended fallback, not an owner term: gpt-oss-120b (Apache 2.0).
- **Hard text only:** records hold `reasoning` and `answer` strings and nothing else. No logits, probabilities, scores or ranks are requested or stored. `verify` rejects any other key. (b) stays deferred; (c) and (d) stay banned.
- **Outside every compiled module (§3.4):** the engine runs on the pod. The instrument is an HTTP client.
- **Caps:** 4 pods and $35/h across all labs, for this run only. Other labs' pods are counted at lease time.
- **One measured hour before any fan-out.** A completion watcher on every pod (L-004). The receipt is bound at generation. A sheet with a non-author FIT (§4.5).

## Sheet

`docs/harness/prep/m2-donor-corpus-2026-11-15.md`, checked by `uor-harness prep-check`, with a fresh-context non-author FIT before any lease.

## Native-space five items (SKILL)

Not applicable. This unit makes training data and touches no model state, operator, optimizer, carry, memory or artifact. It is an inventory-class unit.

## Files in scope

- `docs/labs/m2-donor-corpus/**` (new)
- `docs/harness/prep/m2-donor-corpus-2026-11-15.md`
- this folder
- `docs/research/notebook/data-tokenizer.md` (one row)

## Stop list

No student training, evaluation or serving. Nothing in the repository reads a donor weight, layer, hidden state or logit. No lease before D87 is on `main`. Nothing on wick. Never more than $35/h or 4 pods across all labs. No second pod while the measured hour is unfinished.

## Done when

1. 1B kept tokens, counted in the donor's tokenizer, are sealed with a receipt that verifies after readback from the HF store. They are also counted in the 4,096-token tokenizer if that is feasible.
2. The REPORT gives the measured tok/s, the wall time and the dollars spent, with the negatives first.
3. Every pod is down, and the PR has a fresh exact-head review and goes through `uor-harness loop queue --pr N`.
