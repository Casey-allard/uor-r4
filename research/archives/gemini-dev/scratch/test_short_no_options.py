import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

# We strictly instruct the model to keep the answer under 25 words and 1 sentence.
system_prompt_concise = """You are a geometric AI. Answer the query using the context. Keep your response extremely brief, strictly 1 short sentence of under 20 words. No paragraphs, no extra explanation.

Context:
  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies."""

payload = {
    "model": MODEL,
    "system": system_prompt_concise,
    "prompt": "explain quantum resonance in the manifold",
    "stream": False
}

print("Querying /api/generate with strict 1-sentence constraint and no options...")
start_time = time.time()
data = json.dumps(payload).encode("utf-8")
req = urllib.request.Request(
    f"{OLLAMA_URL}/api/generate",
    data=data,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    with urllib.request.urlopen(req, timeout=30) as resp:
        elapsed = time.time() - start_time
        res_json = json.loads(resp.read().decode("utf-8"))
        content = res_json.get("response", "")
        print(f"Succeeded in {elapsed:.2f} seconds!")
        print(f"Output: {repr(content)}")
        print(f"Reason: {res_json.get('done_reason')}")
        print(f"Eval: {res_json.get('eval_count')}")
except Exception as e:
    print(f"Failed: {e}")
