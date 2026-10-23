# Pre-registration refresh - M2 TRAIN-mask rung-0 sheet under D37 (`m2-pin-train-mask-sheet`)

- date: 2026-10-23; item #2851; base `8720c5d6d`
- milestone: M1 #2029 rung 0 on the compliant trunk (the design record this refresh re-pins); it is the M2 pre-registration source (`#2030`).
- mechanism: rank-margin-objective; mechanism brief `docs/mechanisms/rank-margin-objective.md`.
- old sheet: [`m1-rank-margin-rung0-2026-10-13.md`](m1-rank-margin-rung0-2026-10-13.md) (Amendments 1-7; Amendment 7 froze it under D32).
- **legal basis: D37 + CHARTER §4 rule 5** - floating point is legal in **offline training only**, every float candidate reports its **export gap on the same evaluator**, and **a gap above the two-seed noise band disqualifies it** (CHARTER §3.7). D37 revises D32's learning-path float ban; serving stays integer (§3.1-§3.3, §3.6).
- **this is a NEW sheet, not a revival.** CHARTER §4 rule 5: "Archived float sheets return only as new sheets with a fresh FIT." The old Amendment 6 sheet stays an archived design record; this page is the new D37-basis sheet and carries its own `TEST FITNESS: PENDING` (and needs its own non-author FIT).
- status: **refresh, `TEST FITNESS: PENDING`** - it needs the PM's recorded live/superseded answer and a non-author FIT before the milestone can treat it as the live pre-registration. It runs nothing and releases no hold; training and rental stay HELD.
- scope: this refresh re-pins the old sheet's four Amendment 6 mechanical pins against current `main` and re-states its bars unchanged. No bar, hold or spend is changed here; a bar change is a decision card (CHARTER §5.2).

## Bar

The **decisive bars, the reference, the anti-vacuity guards, the dev evaluation policy, the noise floor and the diagnostics are unchanged from the old sheet's `## Bar` (Amendments 1-5) and are not restated to be reworded here** - restating a bar risks changing it by paraphrase. In full, they are:

- **(i) raw-score top-1** over the scored dev-panel set beats the count predictor's top-1 by **>= 5 percentage points on both seeds** (seeds 1 and 2).
- **(ii) recall@8** over `top_token_ids(8)` beats the count predictor's recall@8 by **>= 5 percentage points on both seeds**.
- **reference:** token-frequency count table over the **response-masked TRAIN target positions** (`mw-balp/train/`: 7,512,797 tokens, 2,425,925 response-masked targets; ties by smallest token identity), computed in-run on the GPU; the DEV-response oracle is a diagnostic only and never the reference.
- **anti-vacuity floors:** final dev token-margin hinge < 0.5 raw-score units and dev unviolated-target rate >= max(5x same-seed untrained, 1% absolute), on both seeds.
- **noise:** scored denominator n = 14,287 (mask sum over scored positions), Wilson 95% half-width < 0.83 points; the 5-point margin sits far above it; between-seed disagreement is reported.
- **fresh transfer:** the decisive bars are re-measured on a fresh held-out draw for a KEEP; the frozen reply panels are not scored by rung 0.

Nothing above is narrower than the old sheet's registered bars; the old sheet's full text remains the authoritative wording.

## Base artifact

- **base: none** - trained from seed initialization at the pinned config (the historical 4.4M `rrarra` smoke config is context only, not a base artifact).
- **pinned config (unchanged), with the per-arm objective made explicit:** `arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 gate=hamilton rec_gates=geometric projection=spiralcore_fbs head=spiralcore_product device=cuda precision=f32 data_parallel=1`; **`objective=hinge` for M1 (1 rival) and `objective=hinge8` for M8 (8 rivals)** - the old sheet's arm table (line 213) and the trainer's accepted values (`geometric-stack-native.rs:451/494`: `hinge|hinge8`).
- **D37 export gap and its outcome rule, stated:** the run trains in offline f32; the exported artifact is the integer artifact, and the run report must state the measured gap between the f32 training evaluations and the exported integer artifact's evaluations on the same evaluator (D37; CHARTER §3.7). **Outcome rule: a gap above the two-seed noise band disqualifies the configuration** - the non-author FIT fixes the numeric band for this sheet before any run (D39 item 5; the roadmap notes no numeric band is registered yet), and the run reports its between-seed disagreement against that fixed band; a float arm cannot KEEP with an export gap above it. Serving is integer only.

## Data split and pins

- **TRAIN archive:** `data/ft-balp.tar`, 326,860,800 bytes, md5 `45fbd266ecab1e5c113e10ec7ae1e918`, sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`. The run reads only `mw-balp/train/`: 7,512,797 u16 tokens (tokens sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`), response mask `mw-balp/train/response_mask.u8` (**7,512,797 bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`**), **`expected_train_targets=2425925`**.
  - **re-verified this review:** the mask digest appears in the current code pin (`crates/uor-r4-discrete/src/attention_learning.rs:1367`, with 7,512,797/2,425,925) and the count reference archive carries the 2,425,925 count (`archive/labs/count-reference-2026-10-13/README.md:32,84,96`). The DEV tar hashes and lengths above are **restated** from the old sheet's Amendment 4 record, not re-computed (the DEV tar is not in this checkout).
- **DEV panel:** `data/ft-dev.tar`, 143,360 bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`; `ft/dev/tokens.u16` = 64-byte UORT header + **42,856** u16 payload tokens; `ft/dev/response_mask.u8` = 42,856 bytes with **14,287** eligible within-document targets across 400 documents; **`expected_dev_targets=14287`**; 111 global 384-token inventory blocks + 232 tail tokens.
- **tokenizer:** sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab 4096, bos 0 / eos 1 / unk 2.
- **dose:** 7,324 steps x batch 16 x context 384 = **44,998,656** tokens (~6 passes over the 7,512,797-token train split). (The old sheet's `44,957,696` is an arithmetic typo; the step/batch/context pins are unchanged, so the run is unaffected.)
- No corpus is regenerated, padded or changed by this refresh; nothing here is training data for the frozen reply panels.

## Steps

- **exact CLI (unchanged from Amendment 6; verified present in `crates/uor-r4-training/examples/geometric-stack-native.rs`). The CLI below is the M1 arm (`objective=hinge`); for the M8 arm substitute `objective=hinge8` and change nothing else:**

```text
geometric-stack train arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 gate=hamilton rec_gates=geometric projection=spiralcore_fbs head=spiralcore_product objective=hinge device=cuda precision=f32 data_parallel=1 train=/workspace/rung0/data/mw-balp/train/tokens.u16 train_mask=/workspace/rung0/data/mw-balp/train/response_mask.u8 valid=/workspace/rung0/data/ft/dev/tokens.u16 valid_mask=/workspace/rung0/data/ft/dev/response_mask.u8 tokenizer=/workspace/rung0/data/tokenizer-d36d3e87.json expected_dev_targets=14287 expected_train_targets=2425925 batch=16 steps=7324 lr=0.002 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1 eval_every=128 eval_windows=111 final_windows=111 checkpoint_every=128 sample_tokens=0 max_seconds=inf seed=SEED out=EXCLUSIVE_REPORT_ROOT
```

- four runs on one leased GPU, sequential: **M1** (`objective=hinge`, 1 rival) and **M8** (`objective=hinge8`, 8 rivals) at **seed 1 and seed 2**; paired seeds, same initializer and window RNG stream, only the objective/rival count differs.
- this refresh executes none of it: training and rental remain HELD until the owner releases them (AGENTS.md; CHARTER §4.5/§5.1), and a FIT releases nothing.

## Stop conditions

- **what the CLI actually enforces (verified in `crates/uor-r4-training/examples/geometric-stack-native.rs` and `native_rung0.rs`):** it refuses without `train_mask`; it binds the TRAIN input content (records bytes + sha256) and aborts when the masked TRAIN reference count differs from `expected_train_targets` (2,425,925); it checks the mask length equals the token count; it stores `expected_dev_targets` in the panel plan and **reports** any DEV denominator mismatch (14,287) as a diagnostic rather than aborting; on resume it requires the checkpoint's saved input binding to match.
- **manual pre-run verification this sheet requires (restored from the old sheet's step list):** before launching, the operator verifies the declared tar/member identities and mask inventory by hand - `data/ft-balp.tar` and `data/ft-dev.tar` sizes/md5/sha256 above, the mask member's 7,512,797 bytes and sha256 `b5e7352d…`, the 42,856 DEV tokens and the 14,287 mask sum - and records each result in the run report. A mismatch stops the launch; it is not absorbed.
- **lawful stops:** the enforced abort above, a failed manual hash/count check, a hashing/read error, a GPU/lease failure, or an owner stop. A self-set wall-clock or budget estimate is not a stop rule; a healthy run is extended and its lease renewed (AGENTS.md). The DEV denominator mismatch is reported, not absorbed, and the report must name it.
- **outcome rule (D37/§3.7):** the export gap on the same evaluator must be stated in the report, and **a gap above the two-seed noise band disqualifies the configuration**; that is a negative, and it closes this configuration, not the mechanism.
- **negative classification:** a failed bar closes this configuration (offline float, hinge objective/arm objectives M1 `hinge` / M8 `hinge8`, rival counts, pinned dose), never the mechanism; the old sheet's verdict wording and §4 rule 1 apply.
## Cost

- preparation: no GPU (source, sheet, pins; the data is already published).
- run: **one Vast pod, 1x RTX 5090 or the next rung, <= $4 total UNVERIFIED projection** for the four runs (history: <= $2 on a 1x4090), within the owner caps (**<= 4 running pods and <= $8/h across all labs**: CHARTER §5 rule 4, not D26 - D26 is only that Vast is the default provider).
- no separate control pod: the count predictor and the untrained same-seed guard are computed on the same GPU inside the run; retention: small checkpoints/report only, cloud-store round-trip, 128 MiB physical stop margin.

## Minimum test power

- **seeds:** 1 and 2, paired across M1/M8: 2 seeds satisfy D22 section 3's decisive-run requirement (>= 2 seeds **or** >= 200 fresh rows).
- **trained in, not bolted on:** the full pinned run (7,324 steps, ~6 passes) gives the base time to adapt; a short add-on would be a probe.
- **noise floor:** Wilson 95% half-width < 0.83 points at n = 14,287; between-seed disagreement is additionally reported; no bar narrower than the noise.
- **fresh-transfer plan for a KEEP:** re-measure top-1 and recall@8 on a fresh held-out draw of mw-balp windows the run never trained on, with the anti-vacuity guards reported alongside.

## TEST FITNESS line

TEST FITNESS: PENDING

This is a **refresh**, so it carries no FIT: the old sheet's Amendment 6 FIT is retained in its own record, and this page is the D37-basis restatement. A non-author FIT review (Claude, Kimi or antigravity; deepseek authored this refresh, so not deepseek) plus the PM's recorded live/superseded answer are the two prerequisites; this page clears no compute.