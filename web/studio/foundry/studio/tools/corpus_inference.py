"""Byte identity corpus: the generated core against the daemon's rules.

Writes model/inference.json, a fixed set of requests with the exact prompt and
params bytes the freeinference daemon (its
src/modules/openai.rs at 5c4ded7) keys its memos on, then runs the core
crate's corpus test, which feeds every request to the generated functions
and compares byte for byte. The expected values here are the daemon's rule
restated once more in Python; the test proves the generated Rust agrees
with it on every case, and the theorems in the model prove the rule's shape.

Run through tools/corpus.py, which writes both case files and runs the core's tests.
"""
import json, pathlib, subprocess, sys

root = pathlib.Path(__file__).resolve().parent.parent

def render_prompt(messages):
    # openai.rs render_prompt: "role: content" lines joined by newline; array content joins its text parts by newline.
    lines = []
    for m in messages:
        role = m.get("role", "user")
        content = m.get("content", "")
        if isinstance(content, list):
            content = "\n".join(part.get("text", "") for part in content if "text" in part)
        lines.append(f"{role}: {content}")
    return "\n".join(lines)

def params_canonical(max_tokens, seed, temperature):
    # openai.rs: json!({"max_tokens", "temperature", "seed"}).to_string() with serde_json's sorted keys.
    # temperature is an f32 there; the model takes the spelled decimal string, so the corpus
    # uses spellings that are unambiguous under f32 Display.
    spell = lambda v: "null" if v is None else str(v)
    return '{"max_tokens":' + spell(max_tokens) + ',"seed":' + spell(seed) + ',"temperature":' + (temperature if temperature is not None else "null") + "}"

cases = []
OR_MODEL = "qwen/qwen3.8-flash"
def openrouter_bytes(model, messages, max_tokens, seed, temperature, stream):
    # encodeOpenRouterRequest: only these fields, in this order, absent when absent, never a key.
    J = lambda v: json.dumps(v, separators=(",", ":"), ensure_ascii=False)
    msgs = ",".join('{"role":' + J(m[0]) + ',"content":' + J(m[1]) + "}" for m in messages)
    return ('{"model":' + J(model) + ',"messages":[' + msgs + "]"
            + ("" if max_tokens is None else ',"max_tokens":' + str(max_tokens))
            + ("" if seed is None else ',"seed":' + str(seed))
            + ("" if temperature is None else ',"temperature":' + temperature)
            + ',"stream":' + ("true" if stream else "false") + ',"usage":{"include":true},"reasoning":{"enabled":false},"provider":{"require_parameters":true}}')
def case(name, messages, max_tokens=None, seed=None, temperature=None):
    cases.append({
        "name": name,
        "messages": [{"role": m[0], "content": m[1]} for m in messages],
        "max_tokens": max_tokens, "seed": seed, "temperature": temperature,
        "prompt": render_prompt([{"role": m[0], "content": m[1]} for m in messages]),
        "params": params_canonical(max_tokens, seed, temperature),
        "openrouter_model": OR_MODEL,
        "openrouter_stream": openrouter_bytes(OR_MODEL, messages, max_tokens, seed, temperature, True),
        "openrouter_plain": openrouter_bytes(OR_MODEL, messages, max_tokens, seed, temperature, False),
    })

case("single user", [("user", "What is the capital of France?")])
case("system and user", [("system", "Be brief."), ("user", "hello")], 64, 1, "0.7")
case("three turns", [("user", "a"), ("assistant", "b"), ("user", "c")], 512, None, "0.0")
case("empty content", [("user", "")])
case("unicode", [("user", "κ addressed, naïve café, 日本語, emoji 🙂")], 32, 0, "1.0")
case("newlines inside content", [("user", "line one\nline two\n"), ("assistant", "ok\n")], 16)
case("colon in content", [("user", "role: content: more")])
case("max seed", [("user", "x")], 4096, 18446744073709551615, "2.0")
case("temperature only", [("user", "x")], None, None, "0.7")
case("max tokens only", [("user", "x")], 1)
case("seed only", [("user", "x")], None, 7)
case("long transcript", [("user", f"turn {i}") if i % 2 == 0 else ("assistant", f"reply {i}") for i in range(16)], 256, 3, "0.5")
case("odd role", [("tool", "result"), ("user", "and?")])
case("whitespace content", [("user", "   "), ("assistant", "\t")])
case("quotes and braces", [("user", 'say "hi" {now}')], 8, 8, "0.7")
case("model like names in content", [("user", "webgpu:Qwen smollm2")], 100, 100, "0.1")

# The wire vectors: what the encoders in the model must emit, byte for byte, restated once in
# Python with json.dumps in the encoders' key order. Python escapes only what the model escapes
# on these inputs: backslash, quote, newline, return, tab.
J = lambda v: json.dumps(v, separators=(",", ":"), ensure_ascii=False)
def completion_bytes(c):
    return ('{"id":' + J(c["id"]) + ',"object":"chat.completion","created":' + c["created"] + ',"model":' + J(c["model"])
            + ',"system_fingerprint":' + J(c["fingerprint"]) + ',"choices":[{"index":0,"message":{"role":"assistant","content":' + J(c["text"])
            + ',"refusal":null},"logprobs":null,"finish_reason":"stop"}],"usage":null}')
def chunk_head(c):
    return '{"id":' + J(c["id"]) + ',"object":"chat.completion.chunk","created":' + c["created"] + ',"model":' + J(c["model"]) + ',"system_fingerprint":' + J(c["fingerprint"])
def role_bytes(c): return chunk_head(c) + ',"choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}'
def delta_bytes(c, d): return chunk_head(c) + ',"choices":[{"index":0,"delta":{"content":' + J(d) + '},"finish_reason":null}]}'
def final_bytes(c): return chunk_head(c) + ',"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"hologram":{"receipt":' + J(c["receipt"]) + '}}'
def error_bytes(m, k): return '{"error":{"message":' + J(m) + ',"type":' + J(k) + ',"param":null,"code":null}}'
def models_bytes(ids): return '{"object":"list","data":[' + ",".join('{"id":' + J(i) + ',"object":"model","created":0,"owned_by":"browser"}' for i in ids) + "]}"

wire = []
def vector(name, completion, delta="", error=("", ""), ids=()):
    wire.append({"name": name, "completion": completion, "delta": delta, "error": {"message": error[0], "type": error[1]}, "ids": list(ids),
                 "completion_bytes": completion_bytes(completion), "role_bytes": role_bytes(completion), "delta_bytes": delta_bytes(completion, delta),
                 "final_bytes": final_bytes(completion), "error_bytes": error_bytes(*error), "models_bytes": models_bytes(ids), "done_bytes": "[DONE]"})
C = lambda **k: {"id": "chatcmpl-1", "created": "1789000000", "model": "webgpu:Qwen", "text": "", "fingerprint": "", "receipt": "", **k}
vector("plain", C(text="One planet is Earth.", fingerprint="did:holo:sha256:aa;did:holo:sha256:bb", receipt="blake3:cc"), "Earth", ("messages must not be empty", "invalid_request_error"), ["webgpu:Qwen"])
vector("empty", C(), "", ("", ""), [])
vector("escapes", C(id='q"uote', text='back\\slash "quoted"\nnew line\r\ttab', fingerprint="f;g", receipt="r"), 'say "hi"\n', ('a "b" \\ c\n', "server_error"), ["a", "b\"c", "d"])
vector("unicode", C(text="κ addressed, naïve café, 日本語, emoji 🙂", receipt="blake3:00"), "🙂", ("日本語", "invalid_request_error"), ["webgpu:Qwen", "webgpu:Qwen"])
vector("long", C(text="x" * 4000, created="2147483647"), "y" * 1000, ("z" * 300, "server_error"), ["m%d" % i for i in range(12)])
wire_path = root / "model" / "wire.json"
wire_path.write_text(json.dumps(wire, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
print(f"wrote {wire_path}: {len(wire)} wire vectors")

out = root / "model" / "inference.json"
out.write_text(json.dumps(cases, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
print(f"wrote {out}: {len(cases)} cases")

if "--write-only" in sys.argv:
    sys.exit(0)

# The shell hash list the service worker precaches from must match hashlib over the files.
import hashlib
manifest_path = root / "shell" / "manifest.json"
if manifest_path.exists():
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    # A listed file the shell does not have is worse than a wrong digest: the worker precaches the whole
    # list at install, one missing file fails the install, and every page then keeps an older shell with
    # no sign of why.
    absent = [f["path"] for f in manifest["files"] if not (root / "shell" / f["path"]).is_file()]
    if absent:
        sys.exit(f"shell/manifest.json lists files the shell does not have, so the worker could not install: {absent}")
    # A complete closure retains dotfiles. Every Pages upload must explicitly carry them.
    hidden = [f["path"] for f in manifest["files"] if any(s.startswith(".") for s in f["path"].split("/"))]
    if hidden:
        import re
        for workflow in ("pages.yml", "try-page.yml"):
            source = (root.parent / ".github" / "workflows" / workflow).read_text(encoding="utf-8")
            steps = re.split(r"(?m)^      - ", source)
            uploads = [step for step in steps if re.search(r"(?m)^\s*(?:uses: )?actions/upload-pages-artifact@", step) or re.search(r"(?m)^\s*uses: actions/upload-pages-artifact@", step)]
            if workflow == "pages.yml":
                transfers = [step for step in steps if "uses: actions/upload-artifact@" in step and re.search(r"(?m)^\s*name:\s*forge-site\s*$", step)]
                if len(transfers) != 1:
                    sys.exit("pages.yml must retain one explicit forge-site intermediate artifact")
                uploads += transfers
            if not uploads:
                sys.exit(f"{workflow} has no explicit Pages upload for the shell closure")
            for step in uploads:
                values = re.findall(r"(?m)^\s*include-hidden-files:\s*([^\n#]+)", step)
                if [value.strip() for value in values] != ["true"]:
                    sys.exit(f"{workflow} Pages upload must explicitly include every hidden closure file: {hidden}")
    bad = [f["path"] for f in manifest["files"] if hashlib.sha256((root / "shell" / f["path"]).read_bytes()).hexdigest() != f["sha256"]]
    if bad:
        sys.exit(f"shell/manifest.json digests differ from hashlib on: {bad}")
    print(f"shell/manifest.json: {len(manifest['files'])} shell files, every SHA-256 matches hashlib; closure {manifest['closure'][:12]}")
# The same vectors through the wasm guest's ABI, as the page calls it.
import shutil
if shutil.which("node") and (root / "shell" / "core.wasm").exists():
    guest = subprocess.run(["node", str(root / "tools" / "guest.mjs")], cwd=root)
    sys.exit(guest.returncode)
print("node or shell/core.wasm missing: guest check skipped")
