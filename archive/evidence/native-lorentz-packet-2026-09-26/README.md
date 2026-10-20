# Native Lorentz read: run packet (cycles 3 and 3b)

Retained reports, sources and identities behind [cycle 3](../../integration/hyperbolic-cycle3-2026-09-26.md)
(§4 context 128, §10 context 256) and its §11 integer read. The fourth lab asked for this on
[#1401](https://github.com/UOR-Foundation/uor-r4/pull/1401), so the next decision can reuse this work instead of
repeating the training campaign. **Measured** development results at reduced scale; nothing here is a final
holdout or a language qualification.

`manifest.json` lists the SHA-256 and size of every file in this directory. It also lists every binary that stays
outside it (models, checkpoints, corpus, dumps and harness binaries, 465 MB) at its path in the lab session's
scratch directory. Those binaries have no durable home yet; see the last section.

## Contents

| Path | What | Notes section |
|---|---|---|
| `reports/c3-pilot/` | 600-step pilot runs, including the scratch Euclidean and flat-Dot arms | §3 |
| `reports/c3-full/` | 2,000-step runs at context 128 (code, 4 seeds; WikiText, 1 seed) | §4 |
| `reports/c3-ctx256/` | 2,000-step runs at context 256 (code, 2 seeds) | §10 |
| `analysis/context256/` | probe-dump analyses: per category, per copy distance, distance-controlled regression, read geometry | §10 |
| `reports/c3-checks/` | functional checks: checkpoint and resume, flat start, save | §7 |
| `reports/c3b-parity-post-training/` | post-training 4-bit packing and integer parity, 64 windows | §11 |
| `reports/c3b-finetune/` | 300-update quantization-aware (`qat_*`) and float (`float_*`) fine-tunes | §11 |
| `reports/c3b-parity-quantization-aware/` | integer parity of the quantization-aware models on §10's 512 final windows | §11 |
| `reports/c3b-timing/` | idle-machine integer step timing, Dot and Lorentz | §11 |
| `packed/qat-*/` | the four quantization-aware packed models (`hard-model.json`, `hard-parameters.*`) | §11 |
| `packed/tables/` | the sealed integer table root, including the arcosh table | §11 |
| `sources/` | the scratch harnesses, corpus and tokenizer preparation, launchers and analysis scripts | all |

The packed models load directly into the integer runtime:
`IntegerModel::load_with_tables(Path::new("packed/qat-lorentzflat_s1"), Path::new("packed/tables"))`, or through
`joint-integer-parity model=…` after restoring the float checkpoint. They are width 128 and context 256, with
full admission.

## Run groups and how they were produced

- **Run naming.** Suffixes mark container restarts:
  - `_r2`: relaunched from scratch after the first restart, which came before checkpoints existed;
  - `_resumed`: continued from a step-1,500 checkpoint after the second restart;
  - `_r1` at context 256: continued from a step-750 checkpoint.

  A continued root records its parent under `resumed_from`, and the interrupted parent's report is kept.
- **Final runs.** A run's final numbers are those of the last root in its chain. Examples: `code_dot_s1`,
  `code_dot_s2_r2`, `code_dot_s3_resumed`, and `dot_s1_r1` at context 256.
- **Committed example.** Two groups used the committed `joint-read-geometry` example, built from this branch at
  the time (`bin_example`, `bin_example3`), with the settings recorded in each report:
  - the §4 Dot and Dot-matched Lorentz arms;
  - the §10 context-256 runs, with `lorentz_start=flat` for Lorentz.
- **Ablation copy.** The flat-start arms at context 128 (`lorentzflat`, `dotflat`), the WikiText flat Lorentz run
  and the pilot's variant arms used an ablation copy of the training crate.
  - It is commit `9df1afab` plus `sources/euclidean-control.patch`.
  - Environment variables select the variant:
    - `UOR_ABLATE_LOG_BETA` and `UOR_ABLATE_OFFSET` override the initial log scale and radius;
    - `UOR_ABLATE_READ=dotflat` scores β·⟨q,k⟩, a flat-start Dot read with the same parameters;
    - `UOR_ABLATE_READ=euclid` scores β(δ − |q − k|), the flat-metric control.
  - These arms therefore report `read_geometry: lorentz` in their config. The launch lines below say which arm
    each run is.
  - The flat Lorentz arm set the log scale to 0 through `UOR_ABLATE_LOG_BETA=0`. The committed option
    `lorentz_start=flat` reproduces that run's step-0 evaluation exactly (§7).
- **Launch lines retained from the session record** (`$COMMON` held the settings in each report):
  - `UOR_ABLATE_READ=euclid UOR_ABLATE_LOG_BETA=0.4904 UOR_ABLATE_OFFSET=13.0639 ./bin_train_abl2 data=data/code width=128 context=128 batch=16 steps=600 eval_every=100 eval_windows=32 final_windows=256 shards=1 out=runs/pilot/euclid_s1.json geometry=lorentz seed=1`
  - `UOR_ABLATE_READ=dotflat UOR_ABLATE_LOG_BETA=-2.124 ./bin_train_abl $COMMON out=runs/pilot/dotflat_s{1,2}.json geometry=lorentz seed={1,2}`
  - `UOR_ABLATE_LOG_BETA=0 ./bin_example_abl … out=runs/full/code_lorentzflat_s$sd geometry=lorentz seed=$sd $COMMON`, and with `UOR_ABLATE_READ=dotflat UOR_ABLATE_LOG_BETA=-2.124` for `code_dotflat_s$sd`
  - `./bin_example3 … out=runs/ctx256/dot_s$sd geometry=dot seed=$sd $COMMON`, and `… geometry=lorentz lorentz_start=flat …` for `lorentzflat_s$sd`
- **Integer-read runs.** The §11 runs used the committed `joint-read-geometry` (`init=`, `quantize_ramp=`) and
  `joint-integer-parity` examples, through the launchers in `sources/c3b/`. `sources/c3b/build_evidence.py`
  assembled their evidence entry.

## Corpus and tokenizer

- **Code corpus.** `sources/corpus/make_code_data.py` concatenated every `.rs` file under `crates/` of the lab
  checkout on 2026-09-26 at 01:47 UTC. The branch was then at or after `29698ce1`.
- **Split.** Files split 95/5 by the SHA-256 of their *absolute* path, and directories were walked in filesystem
  order. **Rebuilding elsewhere is therefore not bit-exact.** The corpus identity is the hashes of
  `code_train.bin`, `code_valid.bin` and the token files in `manifest.json`.
- **Tokenizer.** `sources/c3-harness/src/bin/bpe.rs` is a byte-level BPE: 256 bytes plus 3,840 merges, trained on
  the training split only, with ties to the smallest pair and exact round trips.
  - It wrote `train.u16`, `valid.u16`, `lens.u16`, `merges.txt` and `stats.json` for each corpus.
  - It is not the retained 4,096-token project tokenizer, and it has no BOS, EOS or role tokens.
  - It is independent of `uor_r4_tokenizer::dialogue::DialogueProtocol`.
- **WikiText-2 (raw).** Tokenized the same way; its token files are listed in the manifest.

## Scope to preserve when reusing

- **Context 256.** The flat-Dot equal-start control was **not** run at context 256; the comparison there is flat
  Lorentz against the retained (sharp-start) Dot read. Dot seed 2 is read-dependent: NoRead mass 0.004.
- **Euclidean control.** It ran only in the 600-step pilot, with one seed and a sharp start.
- **Integer parity.** Integer and emulator NLL agree within 0.0001 nats. The **largest single deviation**
  exceeds the retained 0.01 limit over 131,072 positions for both geometries:
  - with the read on: 0.012–0.018;
  - with the read off, in read-dependent Dot s2: 0.031 and 0.034.
- **Precision.** The Lorentz read adds two learned signed16 scalars and a Q32 scale derived at load, beside the
  ≤4-bit weight maps. Their status under D10 awaits the owner.
- **Scale and data.** Width 128 throughout, on the code development split. There is no final holdout or
  TinyStories run at the retained scale.

## Binaries outside this packet

`manifest.json` → `retained_outside_packet` lists 136 files (465 MB) with their hashes. They are:
- the float models of every finished run;
- the checkpoints of the interrupted attempts;
- `code_train.bin`, `code_valid.bin` and the token files of both corpora;
- the probe dumps;
- the x86-64 harness binaries.

They live in the lab session's scratch directory, which is not durable. A durable copy needs the owner's choice:
- a separate evidence branch;
- the owner's SSD;
- or selected files in the repository.

Until then the hashes pin their identity. The four packed models above, the tables and every report are durable
here.
