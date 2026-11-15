# ORDER: d41-driver-math-pilot, claude-390 (the measured ~1M-token donor pilot on wick, D86)

Lab: claude. Item: [d41-driver-math-pilot](../../../../roadmap/items/d41-driver-math-pilot.md).
Base: `origin/main` at `e5a32f458`, which holds claude-388 (#3997) and with it D86.

## Why this unit

The five-architect review of 15 Nov found the project's own TRAIN corpus is about 39k usable windows, two to three orders of magnitude short of what one served conversational model needs. Its path has six steps: (1) the eval-only hard-copy read (done, claude-388); (2) a native Q31 read operator (another unit); (3) this pilot, then the production donor corpus; (4) native CUDA kernels; (5) bound keys; (6) one model trained and served natively. D41 item 3, as D86 amends it, puts a measured ~1M-token pilot ahead of the production corpus `distill-d1`: the production corpus is sized from the pilot's measured tokens/s and size. The pilot shares no file with step 2.

## The release this unit cites at dispatch (§3.10, §5.1)

D86 (Casey, 15 Nov 2026, recorded in `docs/integration/DECISIONS.md` and the 15 Nov entry of `docs/rules/changes.md`): "I would say yes release it, but why is it going to cost money? if we are running it locally on wick, wouldn't it be free?"; "Allow up to 4 (Recommended)"; "Defer (b), ban (c)(d) (Recommended)"; "gpt-oss-20b on wick (Recommended)"; "Drop driver math, keep pilot (Recommended)".

## The work

1. **An instrument, Rust, in `docs/labs/d41-driver-math-pilot/instrument/`**, its own cargo package and lock file. It runs no model. It:
   - builds the prompt set deterministically from a seed and committed templates, drops any prompt whose text equals a string in `data/panels/` (no panel row reaches the donor), and hashes the set;
   - sends each prompt to the external engine over HTTP, asks for up to four answers, and keeps only the final answer texts;
   - writes the corpus as JSONL shards, then the receipt: per-shard sha256 and a Merkle root;
   - verifies the receipt by readback, and checks structurally that no record carries a logit, probability, score or rank field.
2. **The engine, outside the repository (§3.4).** llama.cpp at tag `b12000` (commit `68d20b884c3878217de79eade0a249efde96ab38`, 29 Oct 2026), built from source with nvcc for `sm_89` inside Claude's own wick container. `llama-server` runs the donor; its code never enters a compiled module of this repository.
3. **The donor.** `openai/gpt-oss-20b` at revision `6cee5e81ee83917806bbde320786a8fb61efebee` (Apache-2.0, not gated), the released MXFP4 weights. They are converted to GGUF once by llama.cpp's own converter at the same commit, with the MXFP4 expert tensors kept as MXFP4 and the rest in their released bf16. The receipt binds the source files' sha256 and the GGUF's.
4. **The run, once, offline, on wick**: one job, `wick.sh job`, with a completion watcher. It stops dispatching prompts once 1,000,000 donor-generated tokens are counted, and finishes the prompts in flight.
5. **Storage.** The corpus, receipt and metrics go to the HF store under `results/claude/d41-pilot-2026-11-15/` through `scripts/pod/uor-store`, fetched back and verified. They are not committed.

## Owner terms and lab conditions (D86), and how each is met

- **Donor and machine:** gpt-oss-20b on wick (the owner's RTX 4090). The receipt names wick's GPU (name, UUID, driver).
- **Up to four hard answer texts per prompt:** stored as texts. No logits, probabilities, scores or ranks are requested, stored or consumed. Each prompt's distinct answers are stored in byte order, so their position says nothing about which the donor preferred.
- **Pinned revision with the released MXFP4 weights:** item 3 above.
- **Outside every compiled module (§3.4):** item 2 above. The instrument is a client that holds no model code.
- **Receipt bound at generation (D41 item 2b):** donor repo and revision, quantization, engine and version, prompt-set hash, seeds, sampling parameters, machine identity (wick's GPU), UTC window, per-shard sha256 and the Merkle root. It is verified by readback from the store before the corpus is called sealed.
- **(c) layer or hidden-state fitting and (d) logit targets stay banned:** nothing in this unit reads a donor weight, layer or distribution.
- **The donor never runs in student training, evaluation or serving:** this unit trains, evaluates and serves nothing.

## Sheet

`docs/harness/prep/d41-driver-math-pilot-2026-11-15.md`, checked by `uor-harness prep-check`, with a fresh-context non-author FIT before the run (§4.5).

## Native-space five items (SKILL)

Not applicable: this unit produces training data and touches no model state, operator, optimizer, carry, memory or artifact. It is an inventory-class unit.

## Cancel probe

At base, no pilot corpus, no receipt and no measured donor tokens/s exist for gpt-oss-20b on wick (`uor-store ls results/claude/` holds none, and the item's "done when" is open).

## Files in scope

- `docs/labs/d41-driver-math-pilot/**` (the instrument and the wick job script)
- `docs/harness/prep/d41-driver-math-pilot-2026-11-15.md`
- this folder
- `docs/research/notebook/<line>.md` (one row)
- `docs/roadmap/items/d41-driver-math-pilot.md` (status `done` when done)

## Stop list

No student training, evaluation or serving. No donor weight, layer, hidden state or logit is read by anything in the repository. No pod. No second generation pass: a failed run is reported, and any rerun is a new sheet. No commit of the corpus, weights or engine.

## Done when

1. The pilot corpus exists with its sha256 and receipt, stored in the HF store and verified by readback, not committed.
2. The measured tokens/s and size are recorded, with the arithmetic that sizes the production corpus from them.
3. The REPORT has the seven headings, and the notebook row is written.
4. The PR goes through a fresh exact-head review, `uor-harness loop queue --pr N`, and `loop close --verify` from `~/uor-r4`.
