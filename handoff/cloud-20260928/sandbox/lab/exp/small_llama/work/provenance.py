"""Bind the delivered checkpoint to its data, code, training run and checks."""
import hashlib
import json
import math
import os
import subprocess

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
W = f"{S}/lab/exp/small_llama"
DATA = "/home/user/uor-r4/research/ai-research/ai-router/router-research/data/lm_proxy/raw/wikitext2"
BIN = f"{S}/target/release/examples/kappa-conversion"


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


log = [json.loads(line) for line in open(f"{W}/work/run1/log.jsonl")]
start = next(r for r in log if r.get("event") == "start")
done = next(r for r in log if r.get("event") == "done")
vals = [r for r in log if "val_nll_32win_f32" in r]
probe = json.load(open(f"{W}/probe1/probe.json"))
ev = json.load(open(f"{W}/work/eval_final.json"))
jax_probe = ev["bf16_export"]["probe_windows_nll"]
commit = subprocess.run(["git", "-C", "/home/user/uor-r4", "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
cfg = start["config"]
record = {
    "schema": "small-llama-provenance/1",
    "checkpoint_dir": f"{W}/ckpt",
    "files_sha256": {n: sha(f"{W}/ckpt/{n}") for n in sorted(os.listdir(f"{W}/ckpt"))},
    "parameters": start["params"],
    "architecture": start["cfg"],
    "data": {
        "train": {"path": f"{DATA}/train.txt", "bytes": os.path.getsize(f"{DATA}/train.txt"), "sha256": sha(f"{DATA}/train.txt")},
        "valid": {"path": f"{DATA}/valid.txt", "bytes": os.path.getsize(f"{DATA}/valid.txt"), "sha256": sha(f"{DATA}/valid.txt")},
        "tokenization": "token = byte + 3; specials <|endoftext|>=0 <|im_start|>=1 <|im_end|>=2 never occur in the data",
    },
    "training": {
        **cfg,
        "optimizer": "AdamW b1=0.9 eps=1e-8, decoupled weight decay on 2-D tensors only, global-norm clip",
        "schedule": "linear warmup then cosine to min_lr_frac * lr",
        "sequence": 256,
        "tokens_seen": done["step"] * cfg["batch"] * 256,
        "train_seconds": done["train_seconds"],
        "wall_seconds_first_to_last_log": round(log[-1]["wall"] - log[0]["wall"], 1),
        "numerics": "master weights/moments f32; every matmul bf16 inputs with f32 accumulation (fwd and bwd); norms, RoPE, softmax, loss f32",
        "hardware": "1 pinned core (taskset) of an Intel Xeon (Emerald Rapids, AVX-512/AMX), JAX " + start["jax"],
        "init": "normal std 0.02; o_proj/down_proj std 0.02/sqrt(2*layers); norms 1",
        "monitor_val_nll_32win": [(r["step"], r["val_nll_32win_f32"]) for r in vals],
    },
    "export": "bf16 = round-to-nearest-even upper half of the f32 master weights; safetensors names sorted, contiguous",
    "evaluation": {
        "jax_f32_on_exported_bf16_weights": ev["bf16_export"],
        "jax_f32_on_f32_master_weights": ev.get("f32_master"),
        "rust_probe": {"binary": BIN, "binary_sha256": sha(BIN), "repo_commit": commit,
                       "dot_nll": probe["dot_nll"], "window_starts": probe["window_starts"],
                       "weights_sha256": probe["identity"]["weights_sha256"]},
        "probe_agreement_abs": abs(probe["dot_nll"] - jax_probe),
        "bits_per_byte_full_valid": ev["bf16_export"]["full"]["nll"] / math.log(2),
    },
}
with open(f"{W}/provenance.json", "w") as f:
    json.dump(record, f, indent=2)
print(json.dumps({k: record["evaluation"][k] for k in ("probe_agreement_abs", "bits_per_byte_full_valid")}))

# Measured before training (random large-weight "stress" checkpoint, same exporter and probe):
record["convention_check_before_training"] = {
    "stress_checkpoint": "random weights, q/k std 0.15, embed std 0.5, others 0.08, norm gains 1+0.2N",
    "rust_probe_dot_nll": 23.56602907180786,
    "jax_same_bf16_weights": 23.566022396087646,
    "negative_controls_jax_nll": {"interleaved_rope": 23.214784, "rope_theta_1e4": 23.624337,
                                  "gqa_order_h_mod_kv": 23.368011},
}
record["tokenizer_checks"] = {
    "engine": "uor-r4-tokenizer ByteBpeTokenizer (wrapped 1:1 by HfBpeTokenizer), scratch binary " + f"{S}/target/release/small-llama-tokcheck",
    "vocab_size": 259, "per_id_decode_mismatches": 0,
    "valid_and_train_ids_equal_byte_plus_3": True, "roundtrip_exact": True,
    "adapter": "hf-byte-bpe/1, pre_tokenizers byte-level(add_prefix_space=false), tokenizer_cid blake3:c94707040fcb2ffde9220ccae19fa8011904677498c5dfb080703a5c7b3743d0",
}
with open(f"{W}/provenance.json", "w") as f:
    json.dump(record, f, indent=2)
