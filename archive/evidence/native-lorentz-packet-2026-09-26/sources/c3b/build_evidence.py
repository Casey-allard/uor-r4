"""Assemble the cycle-3b evidence block from sealed report roots (scratch; not committed)."""
import json, os, sys
S = os.environ["S"]
C = f"{S}/lab/exp/c3b"
MODELS = ["dot_s1", "dot_s2", "lorentzflat_s1", "lorentzflat_s2"]

def load(path):
    with open(path) as f:
        return json.load(f)

def parity(root):
    r = load(f"{root}/report.json")
    keep = ["float_nll", "emulator_nll", "integer_nll", "float_bits_per_byte", "emulator_bits_per_byte",
            "integer_bits_per_byte", "emulator_minus_float_nll", "integer_minus_emulator_nll",
            "mean_total_variation_integer_emulator", "maximum_probability_delta_integer_emulator",
            "maximum_state_delta_integer_emulator", "top1_agreement_integer_emulator",
            "top1_agreement_integer_float", "mean_no_read_mass_integer", "mean_no_read_mass_emulator",
            "integer_seconds_per_step"]
    return {
        "root": os.path.relpath(root, S),
        "manifest_sha256_packed": r["inputs"]["packed_manifest"]["sha256"],
        "checkpoint_weights_sha256": r["inputs"]["checkpoint_weights"]["sha256"],
        "windows": r["windows"], "targets": r["targets"], "stride": r["stride"],
        "quantization": r["quantization"],
        "read": {k: r["read"][k] for k in keep},
        "no_read": {k: r["no_read"][k] for k in keep},
        "read_effect_nats": r["read_effect_nats"],
        "seconds": r["seconds"],
    }

block = {
    "label": "Measured (integer kernel, parity, timing, fine-tunes); code corpus development split; width 128, context 256",
    "commits": ["4183830", "d0b417a", "f92cf80"],
    "tables": {"root": os.path.relpath(f"{C}/tables", S),
               "tables_json_sha256": None, "arcosh_json_sha256": None},
    "post_training_quantization": {m: parity(f"{C}/run_{m}") for m in MODELS},
    "timing_idle": [],
}
import hashlib
def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        h.update(f.read())
    return h.hexdigest()
block["tables"]["tables_json_sha256"] = sha(f"{C}/tables/tables.json")
block["tables"]["arcosh_json_sha256"] = sha(f"{C}/tables/arcosh.json")
for name in sorted(os.listdir(C)):
    if name.startswith("timing_") and os.path.isdir(f"{C}/{name}"):
        r = load(f"{C}/{name}/report.json")
        block["timing_idle"].append({"run": name, "windows": r["windows"],
            "read_ms_per_step": r["read"]["integer_seconds_per_step"] * 1000,
            "no_read_ms_per_step": r["no_read"]["integer_seconds_per_step"] * 1000})
ft = {}
for kind in ["qat", "float"]:
    for m in MODELS:
        root = f"{C}/ft_{kind}_{m}"
        if os.path.exists(f"{root}/report.json"):
            r = load(f"{root}/report.json")
            ft[f"{kind}_{m}"] = {"root": os.path.relpath(root, S), "final": r["final"],
                "curve": [{"step": c["step"], "development": c["development"]} for c in r["curve"]],
                "quantization": r.get("quantization"), "init": r.get("init"),
                "optimizer": r["optimizer"], "steps_completed": r["steps_completed"],
                "seconds": r["seconds"], "lorentz": r.get("lorentz")}
block["fine_tunes"] = ft
qp = {}
for m in MODELS:
    root = f"{C}/qatparity_{m}"
    if os.path.exists(f"{root}/report.json"):
        qp[m] = parity(root)
block["quantization_aware_integer"] = qp
json.dump(block, sys.stdout, indent=1)
