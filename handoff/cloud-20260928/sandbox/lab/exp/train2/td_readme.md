# TinyDialogues

This repository contains the code and data for the paper:

**[Is Child-Directed Speech Effective Training Data for Language Models?](https://aclanthology.org/2024.emnlp-main.1231/)**  

**Authors:** [Steven Y. Feng](https://styfeng.github.io/), [Noah D. Goodman](https://cocolab.stanford.edu/ndg), and [Michael C. Frank](https://web.stanford.edu/~mcfrank/) (Stanford University).

> Please contact syfeng@stanford.edu if you have any questions or concerns.

---

## 📦 Data

- **TinyDialogues Dataset** is hosted on HuggingFace: [styfeng/TinyDialogues](https://huggingface.co/datasets/styfeng/TinyDialogues)
- Other datasets can be found under the `data/` folder (organized into `.zip` files).
- **Expected format**:
  - Each `.txt` file (for train/val) contains one example per line.
  - Each line must end with the token `<|endoftext|>`.

### Repeated Buckets Setup for Curriculum Experiments

#### CHILDES:

```bash
python scripts/repeated_buckets/childes_repeated_buckets_setup.py \
  <CHILDES_train_txt_file> <num_buckets> <repeats_per_bucket>
```

This splits the given [CHILDES data](https://github.com/styfeng/TinyDialogues/blob/main/data/CHILDES_data.zip) file into `<num_buckets>` buckets, and repeats each one `<repeats_per_bucket>` times before moving onto the next bucket.

#### TinyDialogues:

```bash
python scripts/repeated_buckets/TD_repeated_buckets_setup.py \
  <train/val> <repeats_per_bucket>
```

This uses the TD individual age data files (found [here](https://huggingface.co/datasets/styfeng/TinyDialogues/blob/main/individual_age_data.zip)) as buckets (one bucket per age) and repeats each one `<repeats_per_bucket>` times before moving onto the next bucket.

---

## 🧠 Model Configs & Tokenizers

- Pretrained tokenizers (for each dataset) can be found under the `tokenizers/` folder.
- Default GPT-2-small and RoBERTa-base model configs can also be found there.

---

## ⚙️ Environment Setup

### Step 1: Install Miniconda (if needed)
```bash
mkdir -p ~/miniconda3
wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh -O ~/miniconda3/miniconda.sh
bash ~/miniconda3/miniconda.sh -b -u -p ~/miniconda3
rm ~/miniconda3/miniconda.sh
export PATH="~/miniconda3/condabin:$PATH"
source ~/.bashrc
conda init
```

### Step 2: Create and configure training environment
```bash
conda create -n babyLM_train python=3.10.9
conda activate babyLM_train
cd transformers
pip install .
cd examples/pytorch/language-modeling
pip install -r requirements.txt
pip install accelerate tokenizers nltk
pip install numpy==1.24.2
```

#### Optional: Verify installation
```bash
python -c "import torch; print(torch.__version__)"
python -c "import transformers; print(transformers.__version__)"
python -c "import numpy; print(numpy.__version__)"
python -c "import accelerate; print(accelerate.__version__)"
nvcc --version
dpkg -l | grep libnccl
```

#### Optional: GPU test script
```python
import torch
print(torch.__version__)
print(torch.version.cuda)
print(torch.cuda.is_available())
print(torch.cuda.device_count())
print(torch.cuda.get_device_name(0))
```

#### Optional: CUDA compatibility fix

```bash
pip install torch==2.1.2+cu121 torchvision==0.16.2+cu121 torchaudio==2.1.2+cu121 --extra-index-url https://download.pytorch.org/whl/cu121
```

#### Optional: NCCL multi-GPU training fix

```bash
sudo apt-get install libnccl2=2.18.3-1+cuda12.1 libnccl-dev=2.18.3-1+cuda12.1
```

#### Optional: Other fixes
If you encounter this error when training GPT-2: `TypeError: TextConfig.__init__() got an unexpected keyword argument 'use_auth_token'`, comment out all lines that include `use_auth_token` in `run_clm_no_shuffling.py`.
	
---

## 🧪 Model Training

### First Step: Tokenizer Training

```bash
python scripts/tokenizers/train_GPT2_tokenizer.py <train_file> <val_file> <output_folder>
python scripts/tokenizers/test_GPT2_tokenizer.py <output_folder>
```

> Note: do this for every unique dataset that you want to train a model on. Then, make sure to use that tokenizer while training that model on that particular dataset.
> We also include some pretrained tokenizers in the `tokenizers/` folder.

### GPT-2 (Causal LM)

Our script `run_clm_no_shuffling.py` is a modified version of HuggingFace’s `run_clm.py`, with credit to the [HuggingFace Transformers repository](https://github.com/huggingface/transformers) for the original code. Our version disables random data shuffling during training, needed for curriculum experiments.

#### Generic command:
```bash
bash scripts/language_model_training/GPT2_CHILDES_4-GPUs_train.sh \
  <train_file> <val_file> <tokenizer_folder> \
  tokenizers/GPT2-small_config <model_output_path> \
  gpt2 <lr> <epochs> <batch_size> <seed>
```

#### Example:
```bash
bash scripts/language_model_training/GPT2_CHILDES_4-GPUs_train.sh \
  train_data/CHILDES_train_ordered.txt train_data/CHILDES_val_ordered.txt \
  tokenizers/GPT2_CHILDES tokenizers/GPT2-small_config \
  trained_GPT2_models/GPT2-small_CHILDES_ordered_1e-04 \
  gpt2 1e-04 1 8 42
```

> Note: you can change the number of GPUs to use by modifying the script accordingly.  

> We set the `SAVE_TOTAL_LIMIT` argument to 2 to save space (hardcoded in the current script), but you can modify it accordingly to save more intermediate checkpoints (e.g., per epoch).

> Please see our paper for more training details and hyperparameters.

### RoBERTa (Masked LM)

#### Generic command:

```bash
python scripts/language_model_training/train_roberta_directly_seed.py \
  <train_file> <val_file> <tokenizer_folder> <model_output_path> \
  <vocab_size> <text_chunk> <line_by_line> <lr> <num_epochs> <batch_size> <seed>
```

#### Recommended Defaults:
- `vocab_size=30000`
- `text_chunk=512`
- `line_by_line='no'`
- `lr=5e-05`, `num_epochs=50`

#### Example:
```bash
python scripts/language_model_training/train_roberta_directly_seed.py \
  train_data/CHILDES_train_ordered.txt train_data/CHILDES_val_ordered.txt \
  tokenizers/roberta_CHILDES \
  trained_roberta_models/roberta-base_CHILDES_ordered_50-epochs_seed42_5e-05 \
  30000 512 no 5e-05 50 32 42
```

---

## 🧪 Evaluation

### Zorro Evaluation

This is adapted from the BabyLM workshop's evaluation (2023) [repo](https://github.com/babylm/evaluation-pipeline). Here is the original Zorro [repo](https://github.com/phueb/Zorro).

We have already:
- Preprocessed and filtered Zorro evaluation examples by CHILDES and TinyDialogues vocabulary
- Converted them to the BLIMP format
- Saved outputs in `evaluation-pipeline/zorro_data` and `evaluation-pipeline/filter-data_zorro{...}/`
- Modified scripts accordingly

#### Setup:
```bash
conda create -n babyLM_zorro python=3.9
conda activate babyLM_zorro
cd evaluation-pipeline
pip install -e ".[colab]"
pip install promptsource==0.2.3
pip install numpy==1.24.2
```

#### Optional: CUDA compatibility fix
```bash
pip install torch==1.10.1+cu111 torchvision==0.11.2+cu111 torchaudio==0.10.1 -f https://download.pytorch.org/whl/cu111/torch_stable.html
```

#### Optional: Make evaluation scripts executable
```bash
chmod u+x finetune_all_tasks.sh
chmod u+x finetune_model.sh
```

#### Optional: Other Fixes

If accelerate causes conflicts with lmeval, replace: `"accelerate@git+..."` with `"accelerate==0.12.0"` in `setup.py`.

If you run into a `KeyError: 'null_prompt'` error in the PromptSource library while running Zorro, please replace the `promptsource` folder in your conda environment with the `promptsource` folder in this repo. For example, `/home/user/miniconda3/envs/babyLM_zorro/lib/python3.9/site-packages/promptsource`.

#### Evaluation command:
```bash
python babylm_eval_zorro.py \
  <path_to_model> <ncoder/decoder> <eval_format> \
  <results_txt> <final_avg_csv> <results_csv> <results_jsonl>
```

- `<encoder/decoder>` should be `"decoder"` for GPT-2, `"encoder"` for RoBERTa.
- `<eval_format>`: `"zorro"`, `"zorro_dialogue-format-CHILDES_CHI"`, `"zorro_dialogue-format-CHILDES_MOT"`, `zorro_dialogue-format-tinydialogue_Child`, or `zorro_dialogue-format-tinydialogue_Mom`. See our paper for more info.

#### Evaluate multiple models:
```bash
bash scripts/evaluation/zorro_eval_script.sh
```

Modify this script to iterate over lists of models and run evaluation automatically.

---

### Word Relatedness Evaluation

Based on [LexiContrastiveGrd](https://github.com/EvLab-MIT/LexiContrastiveGrd). Credits to Chengxu Zhuang and coauthors of [arXiv:2310.13257](https://arxiv.org/abs/2310.13257) and [arXiv:2403.14551](https://arxiv.org/abs/2403.14551).

#### Setup:
```bash
cd LexiContrastiveGrd
conda create -n babyLM_WR python=3.9
conda activate babyLM_WR
pip install -e .
pip install git+https://github.com/chengxuz/lm-evaluation-harness.git
pip install pytest pycountry openpyxl scipy sacrebleu sklearn
```

#### Evaluation command:
```bash
export PYTHONPATH="${PYTHONPATH}:$(pwd)/src"
cd src/llm_devo/word_sim
python eval_word_sim.py \
  --ckpt_path <path_to_model> \
  --output_file <results_txt> \
  --output_csv <results_csv> \
  --output_final_avg_score_csv <avg_csv>
```

#### Evaluate multiple models:
```bash
bash scripts/evaluation/word-relatedness_eval_script.sh
```

Modify this script to iterate over lists of models and run evaluation automatically.

> Note: To re-run evaluation on the same model, delete the corresponding `.pkl` files in `LexiContrastiveGrd/src/llm_devo/word_sim/llm_devo_word_sim_results/human_sim/miniBERTa` or rename the folder to avoid `RuntimeWarning: Mean of empty slice.` errors.

---

## 📑 Citation

If you use this codebase or dataset, please cite:

```bibtex
@inproceedings{feng-etal-2024-child-directed,
	title = "Is Child-Directed Speech Effective Training Data for Language Models?",
	author = "Feng, Steven Y. and Goodman, Noah D. and Frank, Michael C.",
	editor = "Al-Onaizan, Yaser and Bansal, Mohit and Chen, Yun-Nung",
	booktitle = "Proceedings of the 2024 Conference on Empirical Methods in Natural Language Processing (EMNLP)",
	month = nov,
	year = "2024",
	address = "Miami, Florida, USA",
	publisher = "Association for Computational Linguistics",
	url = "https://aclanthology.org/2024.emnlp-main.1231/",
	doi = "10.18653/v1/2024.emnlp-main.1231",
	pages = "22055--22071"
}
```
