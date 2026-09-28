"""Add the cycle-4 main comparison to the evidence packet (scratch builder; run after both final reports exist)."""
import hashlib, json, os, shutil, sys

S = "/tmp/claude-0/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/scratchpad"
REPO = "/home/user/uor-r4"
PKT = f"{REPO}/docs/evidence/geometric-stack-cycle4-2026-09-27"
M = f"{S}/c4/runs/main"
ARMS = ["transformer_s1", "geometric_s1"]


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def put(src, rel):
    dst = os.path.join(PKT, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copyfile(src, dst)


for arm in ARMS:
    final = f"{M}/{arm}_r4"
    if not os.path.isfile(f"{final}/report.json"):
        sys.exit(f"{final} has no report.json")
    # The final root (resumed from the step-250 checkpoint of *_r1) and its seal.
    for name in ["attempt.json", "report.json", "manifest.json"]:
        put(f"{final}/{name}", f"main/{arm}/{name}")
    put(f"{final}/model/config.json", f"main/{arm}/model/config.json")
    put(f"{M}/{arm}_r4.log", f"main/{arm}/train.log")
    # The first 250 updates, whose checkpoint the final root resumed.
    put(f"{M}/{arm}_r1/attempt.json", f"main/{arm}-attempts/r1/attempt.json")
    put(f"{M}/{arm}_r1/checkpoint/state.json", f"main/{arm}-attempts/r1/checkpoint-state.json")
    put(f"{M}/{arm}_r1.log", f"main/{arm}-attempts/r1/train.log")
    # Attempts that did not continue: the 17:08 launch (host-tuned binary trapped at 17:39), r2 (stopped before
    # its first checkpoint to change the checkpoint interval) and r3 (lost to the 19:29 restart's snapshot).
    for tag, root in [("launch-1708", f"{M}/{arm}"), ("r2", f"{M}/{arm}_r2"), ("r3", f"{M}/{arm}_r3")]:
        if os.path.isfile(f"{root}/attempt.json"):
            put(f"{root}/attempt.json", f"main/{arm}-attempts/{tag}/attempt.json")
        if os.path.isfile(f"{root}.log"):
            put(f"{root}.log", f"main/{arm}-attempts/{tag}/train.log")
put(f"{M}/launch.log", "main/launch-1708.log")
put(f"{S}/c4/chain.log", "main/chain-c4.log")
put(f"{S}/c5/chain.log", "main/chain-c5.log")
put(f"{S}/c4/ckpt-keep/log", "main/checkpoint-keep.log")
for name in ["c5/pipeline-post.sh", "c4/resume-main.sh", "c4/keep-checkpoints.sh"]:
    put(f"{S}/{name}", f"sources/{os.path.basename(name)}")

packet = json.load(open(f"{PKT}/packet.json"))
exe_v3 = sha(f"{S}/c4/bin/geometric-stack-b87acd63-v3")
packet["schema"] = "uor-r4.geometric-stack-cycle4-packet/5"
packet["source_commits"]["b87acd63 (x86-64-v3 rebuild)"] = (
    "main comparison: b87acd63 rebuilt from source with -C target-cpu=x86-64-v3 (SHA-256 " + exe_v3 + ") on the "
    "Cascade Lake host the sandbox moved to; the pinned host-tuned build (05889874...) traps there")
packet["runs"] = [r for r in packet["runs"] if r.get("group") != "main"]
for arm in ARMS:
    r = json.load(open(f"{M}/{arm}_r4/report.json"))
    s = r["settings"]
    packet["runs"].append({
        "group": "main", "run": arm, "source_commit": "b87acd63 (x86-64-v3 rebuild)",
        "arch": s["config"]["arch"], "pattern": s["config"]["pattern"], "read": s["config"]["read"],
        "rotation": s["config"]["rotation"], "mlp_hidden": s["config"]["mlp_hidden"], "lr": s["lr"], "seed": s["config"]["seed"],
        "threads": r.get("threads"), "parameters": r["parameters"], "completed_steps": r["completed_steps"],
        "target_visits": r["target_visits"], "final": r["final"], "train_seconds": r["train_seconds"],
        "tokens_per_second": r["tokens_per_second"], "model_sha256": r["model_sha256"],
        "model_bytes": os.path.getsize(f"{M}/{arm}_r4/model/model.safetensors"),
        "model_retained_outside": f"scratchpad/c4/runs/main/{arm}_r4/model/model.safetensors; transfer branch transfer/cycle4-main-20260928",
        "executable_sha256": r["executable"]["sha256"], "resumed_from": r.get("resumed_from"), "inputs": r["inputs"],
    })
files = []
for dp, _, fs in os.walk(PKT):
    for f in sorted(fs):
        path = os.path.join(dp, f)
        rel = os.path.relpath(path, PKT)
        if rel == "packet.json":
            continue
        files.append({"path": rel, "bytes": os.path.getsize(path), "sha256": sha(path)})
packet["files"] = sorted(files, key=lambda x: x["path"])
with open(f"{PKT}/packet.json", "w") as f:
    json.dump(packet, f, indent=1)
    f.write("\n")
print("packet files", len(files), "runs", len(packet["runs"]))
for arm in ARMS:
    r = json.load(open(f"{M}/{arm}_r4/report.json"))
    print(arm, json.dumps(r["final"]), "tok/s %.1f" % r["tokens_per_second"], "train_s %.0f" % r["train_seconds"],
          "model", r["model_sha256"][:16])
