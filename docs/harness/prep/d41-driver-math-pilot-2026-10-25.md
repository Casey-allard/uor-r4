# D41 item 3 ~1M-token donor pilot (`distill-d1-pilot-1m`) — 2026-10-25

- **Roadmap item:** [`d41-driver-math-pilot`](../roadmap/items/d41-driver-math-pilot.md) (#2821), prerequisite for [`m2-donor-corpus-generation`](../roadmap/items/m2-donor-corpus-generation.md) (#2857, `distill-d1`).
- **Lab:** `antigravity`. **Milestone:** M2 (grounded reply from exact memory).
- **Authority & dispatch rule:** `CHARTER.md` §3.10 and §5.1 (D41 items 1–5, D67). Normal GPU runs proceed on the lab's own D67 pre-run review, with one explicit charter exception: **"This record does not change §3, so D41's donor pass still needs his release at dispatch (§3.10)."** Per D41 item 3 (`archive/integration/DECISIONS-2026-10-19.md:1817`) and `PLAN.md:75-77`, the driver-math optimization and M1 Stage T (`m1-bvg-k-stage-t-training`, #2836) complete first; the ~1M-token donor pilot on `openai/gpt-oss-120b` runs after Stage T and the owner's dispatch release.

## 1. Question

Does one local, offline execution of the owner-confirmed open-weight donor `openai/gpt-oss-120b` (MXFP4 at a pinned Hugging Face revision) on a leased `uor-pod` GPU pod produce a `1,048,576`-token hard-target-only integer token corpus (`distill-d1-pilot-1m`) that:
1. contains zero logits, probabilities, soft distributions, or floating-point teacher scores (`CHARTER.md` §3.10, D41 item 4);
2. is 100% disjoint from every evaluation panel in `data/panels/MANIFEST.json`;
3. binds a complete cryptographic generation receipt (donor repo, commit revision, MXFP4 shard SHA-256s, engine version, prompt-set SHA-256, seed, sampling parameters, pod ID, UTC window, per-shard SHA-256s, and Merkle root) verified by round-trip readback from `caseyallard/uor-r4-store` before pod teardown (D41 item 2(b)); and
4. measures end-to-end generation throughput (`tokens/s`) and cost (`<= $2.50` for `1,048,576` tokens) so the full `distill-d1` corpus (#2857) is sized before production dispatch?

## 2. Arms, steps, and minimum test power (defined exactly)

| Arm | Definition | Target tokens | Output contract |
|---|---|---:|---|
| `DeterminismCanary_64` | 64 fixed prompts from the pilot prompt manifest run twice (`pass_a`, `pass_b`) at `seed = 1`, `temperature = 0`, greedy argmax token selection, max completion `256` tokens, tokenized with the student BPE tokenizer (`d36d3e87…`, `vocab_size = 4096`). | `16,384` per pass | Hard integer token IDs (`u16` in `0..4096`) + stop reason only; measures exact token-ID replay rate before the 1M-token pass. |
| `Pilot1M_Donor` | `openai/gpt-oss-120b` (MXFP4, pinned commit revision bound at preflight) run locally on the leased pod over `2,048` deterministic prompts (`max_new_tokens = 512`, `seed = 1`, `temperature = 0`), tokenized into student token IDs (`0..4096`) across `16` shards of `65,536` target tokens each. | `1,048,576` | Hard integer token IDs (`u16` in `0..4096`), prompt SHA-256, and stop reason only; zero logits, probabilities, or distributions stored anywhere on disk or in the archive. |

## 3. Bars and decision table

| Gate | Bar | What it checks |
|---|---|---|
| `G-D41-1` | `0` forbidden keys / float values across all `16` shards | Schema & byte scan of every shard verifies only `{prompt_sha256, prompt_token_ids, target_token_ids, stop_reason}` with all token IDs in `0..4096` and zero `logits`, `logprobs`, `probs`, `scores`, or floating-point literals (`CHARTER.md` §3.10, D41 item 4). |
| `G-D41-2` | `0` overlapping prompts or 32-byte spans with `data/panels/` | Exact SHA-256 and 32-byte substring check against all evaluation panels in `data/panels/MANIFEST.json` (`DEV` `0919e90f…` / `a838bb1a…` and fresh panels). |
| `G-D41-3` | Round-trip Merkle root & `SHA256SUMS` match `100%` | Generation receipt binds donor repo, revision, MXFP4 weight hashes, engine version, prompt-set hash, seed, sampling parameters, pod ID, UTC window, per-shard SHA-256s, and Merkle root, verified after download from `caseyallard/uor-r4-store` before `uor-pod down` (D41 item 2(b)). |
| `G-D41-4` | `>= 1,000,000` target tokens generated within `<= $2.50` pod spend | Measured `tokens/s` and pod wall-clock cost (`<= $2.50` all-in at `<= $8.00/h`) recorded to size `m2-donor-corpus-generation` (#2857). |

- **If `G-D41-1..4` all PASS:** `d41-driver-math-pilot` (#2821) closes (`status: done`) and sizes `m2-donor-corpus-generation` (#2857). Per D41 item 2(e), generated data is an instrument input only; no student capability claim follows and student training remains D32 discrete integer-only (`CHARTER.md` §3.7, §3.10).
- **If `G-D41-1..3` FAIL:** Classed `instrument`; reject the shard archive, repair the serializer/disjointness/receipt tool on host, and re-run only the failed check.
- **If `G-D41-4` FAILS (cost `> $2.50` or throughput below budget):** Closes this engine/batching configuration; adjust batch size or tensor-parallel layout before sizing `distill-d1`.

## 4. Instrument checks (run before the 1M-token pass)

1. **`C-Leak` (negative control):** Feed a synthetic shard record containing `"logprobs": [-1.2]` to the shard verifier; it must exit non-zero and refuse to emit a receipt.
2. **`C-Overlap` (negative control):** Include one `DEV` panel prompt in a test prompt list; the disjointness gate must exit non-zero before any GPU process starts.
3. **`C-Tamper` (negative control):** Flip 1 bit in shard 0 of a synthetic bundle; round-trip Merkle verification must exit non-zero.
4. **`DeterminismCanary_64`:** Verify `pass_a` and `pass_b` produce valid `u16` token IDs in `0..4096` and record exact token-ID match rate (`>= 99.0%` required at `temperature = 0`).

## 5. Base artifact, data split, qualified driver-math prerequisites, and pins

- **Qualified driver-math parity baseline (first half of D41 item 3, PASS on CUDA):**
  - Row-parallel counterfactual parity (`docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json`, SHA-256 `be44d32ec5009055b5ad5dbc9a6f4de0e172e7b0be0d67ede90f32f08b33d70d`; `attempt4/rows-parity.json`, SHA-256 `fedd5c6fde77326e3152c924e88d71c2d80d911ff55254921eb188a57e3df340`).
  - Address-structured cell occupancy (`docs/labs/address-cells-preflight-2026-10-19/evidence/grading.json`, SHA-256 `90c113adbcc2de7c2321a6c6fca9b89a7e72478f8123ae6e3a67dd6d3ee61bb3`).
  - BVG-K Stage C1 batched scoring (`docs/labs/bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json`, SHA-256 `d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`).
  - BVG-K Stage C2 batched block apply (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`, SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`).
- **Student tokenizer pin:** `d36d3e8707d6338641512926780299c2f0c4132833c708e882b1518d718b2798` (`vocab_size = 4096`).
- **DEV evaluation panel pins (excluded by `G-D41-2`):** `0919e90f74a3ce88f0ec6ca3ef2e711fd70f9f7d86712377c64d89b4f82e664b` / `a838bb1ad5ca7826f60417f70b8421b18eb7de81f470c327245996e217e5b0f4`.

## 6. Cost, stop rule, and dispatch gate

- **Cost:** 1 leased `uor-pod` instance inside the global `<= 4` pods and `<= $8.00/h` cap (`CHARTER.md` §5.4), `<= 0.75 h` wall time, target `~$2.00` (`<= $2.50` hard stop).
- **Stop rule:** Stop immediately on any `C-Leak`/`C-Overlap`/`C-Tamper` failure, any shard schema violation, or if `DeterminismCanary_64` projects `> 45` minutes for `1,048,576` tokens.
- **Dispatch gate:** Held until (1) M1 BVG-K Stage T (`m1-bvg-k-stage-t-training`, #2836) completes (`PLAN.md` lines 75–77, D41 item 3) and (2) the owner issues the `CHARTER.md` §3.10 / §5.1 dispatch release for the D41 donor pass.

## 7. Non-author `TEST FITNESS` review (`CHARTER.md` §4.5)

TEST FITNESS: FIT

- Reviewed by fresh-context non-author reviewer (`antigravity` subagent `54de3c0f-ec5e-40ca-a027-19348bbde0a1`, [`#2821#issuecomment-6394217694`](https://github.com/UOR-Foundation/uor-r4/issues/2821#issuecomment-6394217694), 2026-10-25).

