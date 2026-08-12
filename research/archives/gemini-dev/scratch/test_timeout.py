import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

payload = {
    "model": MODEL,
    "system": system_prompt_original,
    "prompt": "explain quantum resonance in the manifold",
    "stream": False
}

print("Querying /api/generate with no options and 180s timeout...")
start_time = time.time()
data = json.dumps(payload).encode("utf-8")
req = urllib.request.Request(
    f"{OLLAMA_URL}/api/generate",
    data=data,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    with urllib.request.urlopen(req, timeout=180) as resp:
        elapsed = time.time() - start_time
        res_json = json.loads(resp.read().decode("utf-8"))
        content = res_json.get("response", "")
        print(f"Succeeded in {elapsed:.2f} seconds!")
        print(f"Output: {repr(content)}")
        print(f"Reason: {res_json.get('done_reason')}")
        print(f"Eval: {res_json.get('eval_count')}")
except Exception as e:
    print(f"Failed: {e}")
