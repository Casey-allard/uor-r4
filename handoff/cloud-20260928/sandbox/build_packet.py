"""Assemble the native Lorentz run packet (scratch builder; not committed)."""
import hashlib, json, os, shutil, subprocess, sys
S = os.environ["S"]
REPO = "/home/user/uor-r4"
OUT = f"{REPO}/docs/evidence/native-lorentz-packet-2026-09-26"
C3, C3B, LEAD = f"{S}/c3", f"{S}/lab/exp/c3b", f"{S}/lab/exp/lead_ctx256"

def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()

def put(src, rel):
    dst = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copyfile(src, dst)

if os.path.exists(OUT):
    sys.exit(f"{OUT} exists")
# Reports.
for group, root in [("c3-full", f"{C3}/runs/full"), ("c3-ctx256", f"{C3}/runs/ctx256")]:
    for run in sorted(os.listdir(root)):
        report = f"{root}/{run}/report.json"
        if os.path.isfile(report):
            put(report, f"reports/{group}/{run}.json")
for run in sorted(os.listdir(f"{C3}/runs/pilot")):
    if run.endswith(".json"):
        put(f"{C3}/runs/pilot/{run}", f"reports/c3-pilot/{run}")
for check in ["ck_A", "ck_B", "ck_C", "example_check", "flat_check", "rd_check0", "rd_check1", "save_check"]:
    report = f"{C3}/runs/{check}/report.json"
    if os.path.isfile(report):
        put(report, f"reports/c3-checks/{check}.json")
for name in sorted(os.listdir(C3B)):
    report = f"{C3B}/{name}/report.json"
    if not os.path.isfile(report):
        continue
    if name.startswith("ft_"):
        put(report, f"reports/c3b-finetune/{name[3:]}.json")
    elif name.startswith("run_"):
        put(report, f"reports/c3b-parity-post-training/{name[4:]}.json")
    elif name.startswith("qatparity_"):
        put(report, f"reports/c3b-parity-quantization-aware/{name[10:]}.json")
    elif name.startswith("timing_"):
        put(report, f"reports/c3b-timing/{name[7:]}.json")
for name in ["e3_analysis.json", "e3b_distance_geometry.json", "e3c_controlled.json"]:
    put(f"{LEAD}/{name}", f"analysis/context256/{name}")
# Sources.
put(f"{C3}/Cargo.toml", "sources/c3-harness/Cargo.toml.txt")
for rel in ["src/main.rs", "src/bin/bpe.rs", "src/bin/keys.rs", "src/bin/train.rs"]:
    put(f"{C3}/{rel}", f"sources/c3-harness/{rel}")
for name in sorted(os.listdir(C3)):
    if name.endswith((".sh", ".py")):
        put(f"{C3}/{name}", f"sources/c3-harness/{name}")
put(f"{S}/c3abl/Cargo.toml", "sources/c3abl-harness/Cargo.toml.txt")
for dp, _, fs in os.walk(f"{S}/c3abl/src"):
    for f in sorted(fs):
        put(os.path.join(dp, f), os.path.join("sources/c3abl-harness", os.path.relpath(os.path.join(dp, f), f"{S}/c3abl")))
put(f"{S}/exp/lead/make_code_data.py", "sources/corpus/make_code_data.py")
for name in ["run_e3.sh", "e3_analyze_ctx256.py", "e3b_distance_geometry.py", "e3c_controlled.py"]:
    put(f"{LEAD}/{name}", f"sources/context256-analysis/{name}")
for name in ["run_all.sh", "run_qat.sh", "run_qatparity.sh", "build_evidence.py"]:
    put(f"{C3B}/{name}", f"sources/c3b/{name}")
base = subprocess.run(["git", "-C", REPO, "show", "9df1afab:crates/uor-r4-training/src/joint_model.rs"], capture_output=True, check=True).stdout
os.makedirs(f"{OUT}/sources", exist_ok=True)
with open(f"{S}/joint_model_9df1afab.rs", "wb") as f:
    f.write(base)
patch = subprocess.run(["diff", "-u", "--label", "a/crates/uor-r4-training/src/joint_model.rs (9df1afab)",
    "--label", "b/crates/uor-r4-training/src/joint_model.rs (ablation copy)", f"{S}/joint_model_9df1afab.rs",
    f"{S}/ablate/crates/uor-r4-training/src/joint_model.rs"], capture_output=True).stdout
with open(f"{OUT}/sources/euclidean-control.patch", "wb") as f:
    f.write(patch)
# Packed integer models and tables.
for model in ["dot_s1", "dot_s2", "lorentzflat_s1", "lorentzflat_s2"]:
    for name in ["hard-model.json", "hard-parameters.json", "hard-parameters.bin"]:
        put(f"{C3B}/qatparity_{model}/packed/{name}", f"packed/qat-{model}/{name}")
for name in sorted(os.listdir(f"{C3B}/tables")):
    put(f"{C3B}/tables/{name}", f"packed/tables/{name}")
# Manifest: every packet file, then binaries retained outside the packet.
files = []
for dp, _, fs in os.walk(OUT):
    for f in sorted(fs):
        p = os.path.join(dp, f)
        files.append({"path": os.path.relpath(p, OUT), "bytes": os.path.getsize(p), "sha256": sha(p)})
files.sort(key=lambda e: e["path"])
external = []
def add_external(path, role):
    external.append({"scratch_path": os.path.relpath(path, S), "role": role, "bytes": os.path.getsize(path), "sha256": sha(path)})
for root in [f"{C3}/runs", C3B, f"{S}/lab/exp/resume_check"]:
    for dp, _, fs in os.walk(root):
        for f in sorted(fs):
            if f.endswith(".safetensors"):
                role = "checkpoint" if "checkpoint" in dp else "model"
                add_external(os.path.join(dp, f), role)
for name in ["code_train.bin", "code_valid.bin", "code_tree.npz"]:
    add_external(f"{S}/exp/lead/data/{name}", "corpus-text")
for corpus in ["code", "wiki"]:
    for f in sorted(os.listdir(f"{C3}/data/{corpus}")):
        add_external(f"{C3}/data/{corpus}/{f}", f"corpus-{corpus}-tokens")
for root in [f"{C3}/analysis", f"{LEAD}/e3"]:
    for f in sorted(os.listdir(root)):
        add_external(f"{root}/{f}", "analysis-dump")
external.sort(key=lambda e: e["scratch_path"])
manifest = {"schema": "uor-r4.native-lorentz-packet/1",
    "scratch_root": "the lab session's scratch directory (claude/blissful-wozniak-girwwq session)",
    "packet_files": files, "retained_outside_packet": external,
    "euclidean_control_base_commit": "9df1afab",
    "totals": {"packet_bytes": sum(e["bytes"] for e in files), "outside_bytes": sum(e["bytes"] for e in external)}}
with open(f"{OUT}/manifest.json", "w") as f:
    json.dump(manifest, f, indent=1)
    f.write("\n")
print(len(files), "packet files,", manifest["totals"]["packet_bytes"], "bytes;", len(external), "outside,", manifest["totals"]["outside_bytes"], "bytes")
