"""Byte identity corpus: the generated core against the host's rule, restated once in Python.

Writes model/corpus.json, a fixed set of project states and capability questions with the exact
preimage bytes and decisions the browser host relies on, then runs the core crate's corpus test,
which feeds every case to the generated functions and compares byte for byte. The theorems in the
model prove the rules' shape; this pins the bytes Lean cannot decide.

Run from the project root: python3 tools/corpus.py
"""
import json, pathlib, subprocess, sys

root = pathlib.Path(__file__).resolve().parent.parent

def escape(v):
    # escapeJson: backslash first, then quote, newline, return; nothing else is touched.
    return v.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\r", "\\r")

def preimage(label, entries, parent):
    # snapshotPreimage: entries in the order given (the host sorts by path, bytewise), each
    # ["path","kappa",bytes]; then the name; then the parent address, "" for the first snapshot.
    lines = ",".join('["' + escape(p) + '","' + escape(k) + '",' + str(n) + "]" for p, k, n in entries)
    return '{"entries":[' + lines + '],"name":"' + escape(label) + '","parent":"' + escape(parent) + '"}'

def decision(endpoints, origin):
    # networkDecision: admitted only when a grant equals the origin; the empty baseline admits nothing.
    return "Accept" if any(e == origin for e in endpoints) else "Refuse"

K1 = "blake3:" + "a1" * 32
K2 = "blake3:" + "b2" * 32
def head_of(refs, branch):
    # headOf: the first ref whose branch equals the name; none otherwise.
    for b_, k in refs:
        if b_ == branch: return k
    return None

cases = []
def case(name, label, entries, parent="", endpoints=(), origin="https://esm.sh", refs=(), branch="main"):
    cases.append({
        "refs": [{"branch": b_, "kappa": k} for b_, k in refs], "branch": branch, "head": head_of(refs, branch),
        "previewPath": "/p/" + K1 + "/",
        "name": name, "label": label,
        "entries": [{"path": p, "kappa": k, "bytes": n} for p, k, n in entries],
        "parent": parent, "preimage": preimage(label, entries, parent),
        "endpoints": list(endpoints), "origin": origin, "decision": decision(endpoints, origin),
        "restore": {"derived": K1, "expected": K1, "decision": "Accept"} if name != "tampered" else {"derived": K2, "expected": K1, "decision": "Refuse"},
    })

case("empty", "a", [])
case("one file", "hello", [("src/main.tsx", K1, 120)])
case("two files sorted", "two", [("src/App.tsx", K1, 512), ("src/main.tsx", K2, 120)])
case("with parent", "child", [("index.html", K1, 44)], parent=K2)
case("quotes in path", 'say "hi"', [('a "b".ts', K1, 1)])
case("backslash in path", "win", [("src\\App.tsx", K1, 2)])
case("newline in label", "line\nbreak", [("x", K1, 3)])
case("return in label", "cr\rlf", [("x", K1, 3)])
case("unicode", "émoji 🚀", [("ünïcode/ファイル.ts", K1, 99)])
case("zero bytes", "z", [("empty", K1, 0)])
case("large bytes", "big", [("blob.bin", K1, 18446744073709551615)])
case("granted origin", "g", [], endpoints=("https://esm.sh",), origin="https://esm.sh")
case("second grant matches", "g2", [], endpoints=("https://cdn.tailwindcss.com", "https://esm.sh"), origin="https://esm.sh")
case("no grants", "n", [], endpoints=(), origin="https://example.com")
case("prefix is not equality", "p", [], endpoints=("https://esm.sh/react",), origin="https://esm.sh")
case("tampered", "t", [("x", K1, 1)])
case("head of main", "h", [], refs=(("main", K1),), branch="main")
case("head of other branch", "h2", [], refs=(("main", K1), ("draft", K2)), branch="draft")
case("first ref wins", "h3", [], refs=(("main", K1), ("main", K2)), branch="main")
case("no such branch", "h4", [], refs=(("main", K1),), branch="draft")

out = root / "model" / "corpus.json"
out.write_text(json.dumps(cases, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
print(f"wrote {out}: {len(cases)} cases")
inference = subprocess.run([sys.executable, str(root / "tools" / "corpus_inference.py")], cwd=root)
if inference.returncode: sys.exit(inference.returncode)
publish = subprocess.run([sys.executable, str(root / "tools" / "corpus_publish.py")], cwd=root)
if publish.returncode: sys.exit(publish.returncode)
run = subprocess.run(["cargo", "test", "--release", "-q", "--", "--nocapture"], cwd=root / "core")
sys.exit(run.returncode)
