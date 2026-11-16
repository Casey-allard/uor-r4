# A2 outside-repo inventory (partial; stopped at the context budget, 16 Nov 2026)

Builds on end-to-end-gap-analysis-2026-11-16.md (read first; not redone).

## Checked listings
- ~/lexlean-r4/gpu (checked: `ls`). Besides the v9/v10/v11 trainers, it holds kernels the gap analysis does not name:
  `cuda_stack.cu`, `discrete.cu`, `spiralcore_fbs.cu`, `spiralcore_fbs_arms.cu`, `spiralcore_fbs_arms_prelude.cu`, `general_glm.cu`,
  `train_native_q31_r4.rs`, `train_uorlut_v8.rs`, `recompile_r4_teacher.rs`, `recompile_holo_v13.rs`, `short.uorlut`, `served-s1.header.json`,
  `rtx5090_glm_weights.bin`, `weights/`. It also holds these RTX receipts: `rtx4090_native_q31_d85_receipt.json`, `rtx4090_gllm_v11_receipt.json`,
  `rtx4090_r4_recompiled_v12_receipt.json`, `rtx4090_holo_v13_qwen3_0_6b_receipt.json`, `rtx4090_holo_v13_qwen3_1_7b_receipt.json`,
  `rtx5090_gllm_v9_receipt.json`, `rtx5090_gllm_v10_receipt.json`, `rtx5090_training_receipt.json`.
  - hypothesis: `discrete.cu`, `cuda_stack.cu` and `spiralcore_fbs*.cu` are copies or forks of uor-r4 kernels. Diff them against the repo's `uor-r4-discrete` before claiming any port.
  - hypothesis: `train_native_q31_r4.rs` with `rtx4090_native_q31_d85_receipt.json` is a 4090 run of a D85 native Q31 trainer (line B). This is the receipt to read first.
  - hypothesis: the `holo_v13_qwen3_*` receipts are recompiles of Qwen3 transformer weights, a float donor. That is probably not usable under §3. Read them to confirm.
- wick (checked: wick.sh `ls`, `find -maxdepth 3`, `nvidia-smi`): /root holds build, leases and work. /workspace holds bin, boot, d41, in, jobs, kit, kit-backup, out, toolchain and uor-r4.
  - `/workspace/d41/donor-hf/model-0000{0,1,2}-of-00002.safetensors` (checked): the D41 distill-d1 donor checkpoint (D41 released 15 Nov). It is a float transformer donor, so it can only be a teacher signal and never model state. Model id not opened (hypothesis).
  - The RTX 4090 was idle when checked: 4 MiB of 24,564 MiB used.
  - No *.cu file was found under /root or /workspace to depth 3 (checked). The v10 copy noted earlier must sit deeper (not re-found).
- ~/spiralcore: per the gap analysis, a 26 Oct snapshot of uor-r4 with no new GPU code (not re-checked).
- Other ~/ folders (checked `ls`): cs1.618 and me are empty. mudbenchmark looks like a MUD benchmark (hypothesis: unrelated). prism-code is the Prism formal-app SDK (unrelated). r4-laboratories is a website (CNAME, LICENSE). ~/uor-r4-local and ~/wick were listed but not opened.
- /Users/casey.allard/uor-r4/local sizes (checked `du`): claude 9.9G (kits and saved models), mvm-review 94M, antigravity 50M, pod-provider-target 34M, lead 8.5M. Everything else is under 1M.

## Not reached
- The contents of every lexlean receipt, so no measured numbers.
- The diff of lexlean `discrete.cu`, `cuda_stack.cu` and `spiralcore_fbs*.cu` against the repo.
- On wick: /root/work, /workspace/out, /workspace/jobs, CorvusWing and the lexlean copies.
- The HF store listing (no hf call made).
- ~/uor-r4-local, ~/wick and the local/claude/* breakdown.

## Addendum (main session, 16 Nov): receipt numbers in ~/lexlean-r4/gpu, read with jq (checked)

| Receipt | What it is | Key numbers |
|---|---|---|
| rtx4090_native_q31_d85 | Native Q31 trainer, UORQ31V1 | d_model 256, 4 layers, 4 heads, vocab 384; 16 E8 roots; 8 epochs; 2,273,280 accepted Hamilton steps; 867 KB checkpoint; CUDA and WASM logit checksums equal. Sample reply is an echo/garble. A toy: parity is proven, quality is not. |
| rtx5090_gllm_v9 / rtx4090 v10 / v11 (UORGLLM9) | 6 layers, 128 d, 16 heads, hinge loss, softmax_ops 0, matmul 0 | v10: 100k training sequences, val exact-token accuracy 99.0% on a narrow chat/tool corpus, 10 MB, 837 s on the 4090. v11 uses **RoPE** (CORDIC Q12), which is banned here, and has 96.1% eval accuracy and a 6.9 MB container. These are narrow-corpus memorisers, not language models. |
| rtx5090_training (UR4GLM08) | Basin, vocab and horizon rows; ternary gates | 280 basin rows and 10,781 vocab rows; kernel time 170 ms. A toy. |
| rtx4090_r4_recompiled_v12 | **A recompile of the SmolLM2-135M transformer** into a 4-bit WHT/dyadic integer container (72 MB) | top-1 logit parity with the teacher 75%, top-5 100%, KL 0.64. Replies are fluent-ish. It is still a transformer (RoPE, RMSNorm, attention). |
| rtx4090_holo_v13_qwen3_1_7b | **A recompile of the Qwen3-1.7B transformer** into a 1.14 GB integer container, float_ops 0, dense_matmul_ops 0 | top-1 parity 91.67%, top-5 100%, mean KL 0.030 nats over 12 OOD prompts. That is a 1.7B-param model served as integers, but it is a transformer. Whether its attention uses a table form of softmax (banned by §3.2) is UNCHECKED. |

**Decision-relevant point:** "recompile a donor transformer into integer tables" already produced a ~1.7B-class integer artifact in about 1.1 GB. It is the transformer architecture, though: banned as model state by §3 and the no-transformer rule, unless the PI finds a geometric form. D41 (released 15 Nov) allows distillation from a donor as a teacher only. The PI and red team must weigh "distill a donor into OUR geometry" against "train from D87 counts".

**Checked (main session):** holo v13 is BANNED under §3, whatever its receipt's labels say. Its runtime is ~/lexlean-r4/crates/wasm/src/holo_geometric_runtime.rs. That file has a SiLU lookup table (lines 26-27, 747-748, 1665) and SwiGLU (1721, 1870). It also reserves space for an exp table at line 1098 (`exp_len`), which suggests a table-form softmax. Its projections are integer GEMV (`gemv_i4_rht` and `gemv_i8_rht`, 1875-1878), which is dense matrix-vector work as a hand loop. The receipt's "float_ops 0 / dense_matmul_ops 0" counts only float GEMM. Under §3.2 and §3.3 (table forms and hand loops are included) this is out as model state. It stays useful only as (a) a D41-style teacher, or (b) an engineering reference for integer serving speed and memory layout on the laptop.

## A2b (subagent, 16 Nov): kernel diffs, compliance, wick, HF, local saved models

### 1. lexlean kernels against the repo (checked: `diff -q` / `diff`, bash, from the payload-probe-387 worktree at origin/main)
| lexlean ~/lexlean-r4/gpu | Repo counterpart | Result |
|---|---|---|
| cuda_stack.cu | crates/uor-r4-float-train/src/cuda_stack.cu | IDENTICAL |
| cuda_stack.cu | crates/uor-r4-training/src/cuda_stack.cu | 3 comment lines differ (the banned-op list in the header comment). No code difference |
| spiralcore_fbs.cu | crates/uor-r4-float-train/src/spiralcore_fbs.cu | IDENTICAL |
| spiralcore_fbs.cu | crates/uor-r4-training/src/spiralcore_fbs.cu | The training-crate copy is the OLDER one: it uses float `atomicAdd`. lexlean, like float-train, has the fixed-order reduce (claude-179, R6). The repo already holds the newer version in float-train |
| spiralcore_fbs_arms.cu, _arms_prelude.cu | crates/uor-r4-float-train/src/ | IDENTICAL |
| discrete.cu (43,200 B, 3 Nov) | crates/uor-r4-discrete/src/discrete.cu (49,520 B, 16 Nov) | lexlean is an OLD snapshot: 10 lexlean-only lines (old forms of `route_vector`, the gating and the mode checks) against 132 repo-only lines. No lexlean-only kernel |
- **Verdict (checked):** no kernel, trainer or read in these five files is missing from the repo. Every one is a copy or an older snapshot of repo code. Nothing to port.
- The files that ARE lexlean-only are `general_glm.cu`, `train_gllm_v10.cu`, `train_v11_gpu.cu`, `train_native_q31_r4.rs` and `train_uorlut_v8.rs` (checked: they have no counterpart under `crates`). Compliance for each:
  - **general_glm.cu (UR4GLM05, 947 lines)** (checked: grep): its four kernels are `train_general_glm_manifold_kernel`, `train_glm_vocab_manifold_kernel`, `train_glm_token_horizons_kernel` and `train_glm_horizon_transitions_kernel`. They do Q12 coordinate sweeps, Hopf, E8 and Icosian snaps, and an L1 `|diff|` accumulation (line 153). Grep found no softmax, exp, sigmoid, SiLU, tanh, GEMM, GEMV or RoPE. It trains basin, vocabulary and horizon rows from about 280 basins of curated prose, so it is a toy (see the addendum, rtx5090_training). Verdict: **compliant on grep; toy scale** (whether it is compliant in substance is a hypothesis, because I did not read every loop).
  - **train_uorlut_v8.rs (lut-stack/8, 1,717 lines)** (checked): it runs coordinate updates on `writer.table` (4096×256 Q12), builds `ServedTurnMemory` and `ServedPointKey`, and holds a hard-coded dialogue corpus (Kant, CRISPR, self-description). Its readout `lut_stack_exec::stack_whole_table_top_k` (~/lexlean-r4/crates/wasm/src/lut_stack_exec.rs:2672) scans all vocab rows by an L1 distance. That is a nearest-row search, not a dot product. Grep found no banned function names. Verdict: **compliant on grep. A memoriser**: it learns canned turns, not language.
  - **train_gllm_v10.cu** (checked: lines 655-680 and 935-952): the attention is "4th-order rational PRLL" with weights `w*1e6/sum_raw`, a weighting normalised to sum to one over every position. Its readout is "T-MAC", a lookup table that stands in for the vocab×D dot products. It trains with momentum. Verdict: **likely banned** (hypothesis). Normalised attention is a renamed softmax (§3.2), and T-MAC is a table form of matmul (§3.3), whatever the header's "ZERO softmax/GEMM" says.
  - **train_v11_gpu.cu** (checked: lines 3, 111-156): CORDIC **RoPE**, T-MAC Q/K/V/FFN tables and `adam_step_kernel`. Verdict: **banned** (RoPE, and a transformer laid out as table matmuls).
  - **train_native_q31_r4.rs** (checked: header and kernel list): one kernel, `k_q31_hamilton_train`. Grep finds no banned ops in code; "RoPE" and "RMSNorm" appear only inside corpus strings. Verdict: compliant on grep, at toy scale (d256, vocab 384; see the addendum).

### 2. short.uorlut, served-s1.header.json, weights/ (checked: `xxd`, `jq`, `du`)
- **short.uorlut** is 2,156,480 B with magic `UORLUT01`. Its header is the repo's own exporter (`uor-r4-integer::stack::export_fbs_trunk_uorlut01`), schema `uor-r4.lut-stack/7`. Shape: width 256, vocab 4096, six recurrent layers (`rrrrrr`), mlp 384, spiralcore_fbs projection, hamilton gate, read `dot`, context 192. It is an early-November repo export (hypothesis: the FBS trunk with about 1M params, judging by its 2.1 MB). The header's `numerics` block lists `silu_step_log2`, `gelu_step_log2`, `exp_step_log2` and `rms_eps`. Those are SiLU/GELU/exp **table parameters**. Verdict: **banned under §3.2 if the stack uses them** (hypothesis: the lut-stack/7 reader builds those tables). It is not better than anything in the repo, because it comes from the repo.
- **weights/** is 302 MB.
  - It holds `uor_r4_glm_v8.uorlut.part000-359`: 360 parts of 512 KB, 188.7 MB in all.
  - Its manifest says `model_id uor-r4-glm-v8-2b` and `total_parameters 2,074,146,944`. That count is inflated (checked: the same manifest has `reply_entries 11,405,402`, `ternary_assoc_rows 483,328`, `point_rows 46,669`, width 256 and 6 layers). The "2B" counts table cells in a 189 MB reply-lookup store. It is a memoriser and **not** a 2B-equivalent model.
  - The manifest also lists `huggingface_repo: caseyallard/uor-r4-store` (the HF *model* repo below).
  - The subfolders are v9 (12 MB), v10 (108 MB, parity bins), and v12, v13_qwen3_0_6b and v13_qwen3_1_7b. The v12 and v13 folders hold manifests only (212-564 KB). The v13 1.7B manifest gives 1,140,250,782 bytes, with the bytes held elsewhere. v13 is banned (see the addendum).

### 3. wick, read-only (checked: `wick.sh sh` runs inside the container; nothing was written)
- /root/work is 4.9 GB. It holds kits c369, c370 (a/b/c/x/y), c387-c389, c395, c404 (122 MB) and c408 (1.2 GB, run1-3 the latest jobs), plus gate-*.log files.
- /workspace/out is 11 GB.
  - c370 is 6.9 GB. Its pair_rows.safetensors are 192 MB each, at steps 8 to 64, and they are smoke runs.
  - c395 is 3.5 GB. It holds **four 815 MB `.q31` states** (`train-4k-s1/s2.q31`, `train-ext-s1/s2.q31`, dated 16 Nov). These are the largest native states on wick (hypothesis: D87-stream count/row states from claude-395).
  - c389 is 419 MB and c404 is 60 MB.
- /workspace/jobs is 2.2 MB and holds 284 files. /workspace/kit is 72 KB (c370, job-386.sh, drive-386.sh). /workspace/uor-r4 is 12 KB (empty jobs/ and pods/). /workspace/d41 is 28 GB (the donor).
- `find / -maxdepth 4 -name 'train_gllm_v10*'` found nothing, and the same search for `*corvus*` found nothing, both in the container. /mnt/plexpool/uor-r4-corpus is **not mounted in the container** (an empty `ls`), so it was not touched. Hypothesis: the corpus and any v10 copy are on the wick host, outside the container.
- The GPU was idle: 4 MiB used, 0% utilisation.

### 4. HF store (checked: hf_whoami returned user caseyallard with no orgs; hub_repo_search and hf_fs ls; no token printed)
- Dataset `caseyallard/uor-r4-store` (private, modified 16 Nov) has these top-level folders: bases, bin, codex, data, inputs, local-archive, models, next02, pod-setup, research-archive, results, toolchain, vast-archive and w2-inputs. `data/` holds step5-inputs.tar (1.47 GB), tinystories_train.u16 (1.11 GB) and ft-balp.tar (327 MB). `models/` holds chat-214m-planA-af-b8 and chat-214m-smoltalk-b16.
- Model `caseyallard/uor-r4-store` holds `uor_r4_glm_v8.uorlut` (271.5 MB, the lexlean v8 memoriser) and weights/.
- Model `uor-r4-geometric-214m` (9 Oct) holds base-planA, base-tinystories, chat-planA-b8 and chat-smoltalk-b16. These are the old float 214M line (hypothesis: pre-D85 float, so not native state).
- Model `uor-r4-m1-model` holds two t16 trunks, `.d11`, about 85 MB each. Model `uor-r4-m1-discrete` holds two `.lut` files of 542 KB.
- Model `uor-r4-fbs-dual-c131` holds two `.uorlut` files of 2.14 MB (the same size class as short.uorlut).
- Dataset `uor-r4-data` holds dev, memory-dialogue, panels, s1-4-parity and tinystories. Dataset `spiralcore-experiment-artifacts` is private.
- No D87 corpus repo showed up in the listing under this author (hypothesis: it sits inside uor-r4-store under data/hot or inputs, which I did not open).

### 5. local/claude (checked: `du -sh … | sort -h | tail -15`, `find -size +40M`)
- The largest folders:
  - claude-380 is 5.0 GB. Of that, fetched/ is 4.5 GB of score snapshot.json files, which are not models.
  - claude-373 is 1.9 GB, claude-374 922 MB, claude-384 902 MB, claude-386 400 MB, claude-376 364 MB, target-370i 233 MB (a build dir) and claude-370 132 MB.
- **The most-trained saved models are the pair-rows runs at step 16,384** (checked: these are the highest step dirs found):
  - claude-373 `rare-s2/s3` and `pair-s2/s3` `models/step-016384/pair_rows.safetensors` (192 MB each).
  - claude-374 `boost`/`pair` tarballs (185 MB each).
  - The integer serve export of that line is claude-384 `art-15/pair-s3.uorlut` (116 MB).
- On wick, the 815 MB c395 `.q31` states are bigger, but how far they were trained is unknown (hypothesis).

### Summary
- Nothing outside the repo beats it on code. The lexlean `.cu` files are copies or older snapshots.
- Every lexlean model is either a memoriser (v8 "2B", v10, UR4GLM) or a transformer recompile (v11 RoPE, v12, v13).
- Compute-wise, the largest native artifacts are the c395 `.q31` states on wick and the step-16,384 pair rows.
