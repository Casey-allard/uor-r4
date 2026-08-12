import urllib.request
import json

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def test_generate(name, payload):
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{OLLAMA_URL}/api/generate",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=45) as resp:
            res_json = json.loads(resp.read().decode("utf-8"))
            content = res_json.get("response", "")
            print(f"[{name}] Output: {repr(content)} | Reason: {res_json.get('done_reason')} | Eval: {res_json.get('eval_count')}")
    except Exception as e:
        print(f"[{name}] Failed: {e}")

# Generate API with no options
test_generate(
    "1. Generate API (No Options)",
    {
        "model": MODEL,
        "system": "You are a geometric AI. Speak scientifically.",
        "prompt": "Explain quantum resonance in 1 sentence.",
        "stream": False
    }
)

# Generate API with num_predict=50
test_generate(
    "2. Generate API (With num_predict=50)",
    {
        "model": MODEL,
        "system": "You are a geometric AI. Speak scientifically.",
        "prompt": "Explain quantum resonance in 1 sentence.",
        "stream": False,
        "options": {
            "num_predict": 50
        }
    }
)

# Generate API with original system prompt and larger timeout
system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

test_generate(
    "3. Generate API (Original System Prompt, No Options)",
    {
        "model": MODEL,
        "system": system_prompt_original,
        "prompt": "explain quantum resonance in the manifold",
        "stream": False
    }
)
