"""Build the D1 evidence packet from the sealed report roots (scratch builder).

Usage: python3 build_packet.py OUT_DIR [--partial]
Copies each arm's final root records (attempt, report, seal manifest, model config) and training log, the runner and
chain log, and writes packet.json with the runs, the pre-registered decision and SHA-256 of every packet file.
"""
import hashlib, json, os, shutil, sys

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
R = f"{S}/d1/runs"
ARMS = [("rot_s1", True, 1), ("id_s1", False, 1), ("rot_s2", True, 2), ("id_s2", False, 2)]
THRESHOLD = 0.02
out = sys.argv[1]
partial = "--partial" in sys.argv


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def latest(base):
    last, n = (base if os.path.isdir(base) else None), 0
    while os.path.isdir(f"{base}_r{n + 1}"):
        n += 1
        last = f"{base}_r{n}"
    return last


def put(src, rel):
    dst = os.path.join(out, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copyfile(src, dst)


os.makedirs(out, exist_ok=True)
runs, finals = [], {}
for name, rotation, seed in ARMS:
    root = latest(f"{R}/{name}")
    if root is None or not os.path.isfile(f"{root}/report.json"):
        if partial:
            continue
        sys.exit(f"{name} has no final report")
    r = json.load(open(f"{root}/report.json"))
    s = r["settings"]
    c = s["config"]
    assert c["rotation"] == rotation and c["seed"] == seed and c["mlp_hidden"] == 749 and c["pattern"] == "rrarra"
    for f in ["attempt.json", "report.json", "manifest.json", "model/config.json"]:
        put(f"{root}/{f}", f"{name}/{f}")
    put(f"{root}.log", f"{name}/train.log")
    model = f"{root}/model/model.safetensors"
    assert sha(model) == r["model_sha256"], name
    finals[name] = r["final"]["nll"]
    runs.append({
        "run": name, "rotation": rotation, "seed": seed, "root": os.path.relpath(root, S),
        "resumed_from": r.get("resumed_from"), "pattern": c["pattern"], "read": c["read"], "mlp_hidden": c["mlp_hidden"],
        "parameters": r["parameters"], "completed_steps": r["completed_steps"], "target_visits": r["target_visits"],
        "final": r["final"], "curve": [{"step": p["step"], "dev_nll": p["dev"]["nll"]} for p in r["curve"]],
        "train_seconds": r["train_seconds"], "tokens_per_second": r["tokens_per_second"], "threads": r.get("threads"),
        "model_sha256": r["model_sha256"], "model_bytes": os.path.getsize(model),
        "model_retained_outside": f"scratchpad/{os.path.relpath(model, S)}",
        "executable": r["executable"], "inputs": r["inputs"],
    })
put(f"{S}/d1/run-d1.sh", "sources/run-d1.sh")
put(f"{S}/d1/chain.log", "chain.log")

decision = {"rule": "keep quaternion transport only if it is at least 0.02 nats better than identity in both seeds; "
                    "any other outcome drops rotation (#1453 §4, work card #973 comment 5864504193)",
            "threshold_nats": THRESHOLD, "per_seed": {}}
for seed in (1, 2):
    a, b = f"rot_s{seed}", f"id_s{seed}"
    if a in finals and b in finals:
        diff = finals[b] - finals[a]
        decision["per_seed"][str(seed)] = {"identity_minus_quaternion_nats": diff, "quaternion_better_by_threshold": diff >= THRESHOLD}
if len(decision["per_seed"]) == 2:
    keep = all(v["quaternion_better_by_threshold"] for v in decision["per_seed"].values())
    decision["outcome"] = "keep quaternion transport" if keep else "drop rotation"
else:
    decision["outcome"] = "incomplete"

packet = {
    "schema": "uor-r4.transport-attribution-d1-packet/1",
    "question": "Is the geometric (quaternion) transport in the main-line rrarra core load-bearing at matched MLP width?",
    "source_commit": "4eef03e6a6505fae8dcbf0dce6728a7e74c5a256",
    "build": "cargo build --release --example geometric-stack with RUSTFLAGS=-C target-cpu=x86-64-v3 on the lab sandbox's "
             "Cascade Lake host (Intel Xeon @ 2.80GHz, 4 cores)",
    "runs": runs, "decision": decision,
}
files = []
for dp, _, fs in os.walk(out):
    for f in sorted(fs):
        path = os.path.join(dp, f)
        rel = os.path.relpath(path, out)
        if rel == "packet.json":
            continue
        files.append({"path": rel, "bytes": os.path.getsize(path), "sha256": sha(path)})
packet["files"] = sorted(files, key=lambda x: x["path"])
with open(f"{out}/packet.json", "w") as f:
    json.dump(packet, f, indent=1)
    f.write("\n")
print("runs", [r["run"] for r in runs], "files", len(files))
for r in runs:
    print(r["run"], "nll %.6f bpb %.6f" % (r["final"]["nll"], r["final"]["bits_per_byte"]), "tok/s %.1f" % r["tokens_per_second"],
          "train_s %.0f" % r["train_seconds"], "params", r["parameters"], "curve", [round(p["dev_nll"], 4) for p in r["curve"]])
print(json.dumps(decision, indent=1))
