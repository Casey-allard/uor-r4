import urllib.request
import json
import sys

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def query_ollama(endpoint, payload):
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{OLLAMA_URL}{endpoint}",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read().decode("utf-8")
            return json.loads(raw)
    except Exception as e:
        return {"error_exception": str(e)}

system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

print("--- Test 5a: Original Prompt with Chat API ---")
p5a = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": system_prompt_original},
        {"role": "user", "content": "explain quantum resonance in the manifold"}
    ],
    "stream": False
}
r5a = query_ollama("/api/chat", p5a)
print("Raw r5a response:")
print(json.dumps(r5a, indent=2))

print("\n--- Test 5b: Original Prompt with Generate API ---")
p5b = {
    "model": MODEL,
    "system": system_prompt_original,
    "prompt": "explain quantum resonance in the manifold",
    "stream": False
}
r5b = query_ollama("/api/generate", p5b)
print("Raw r5b response:")
print(json.dumps(r5b, indent=2))

print("\n--- Test 5c: Original Prompt with Generate API (No unicode symbols) ---")
system_prompt_no_unicode = """You are the R4 Prime Router World Model - a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins and Foundations) with curvature kappa=0.0230, deficit theta_d=-1.6200. Routing regime: resonant (symmetric orbit) - answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context - use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

p5c = {
    "model": MODEL,
    "system": system_prompt_no_unicode,
    "prompt": "explain quantum resonance in the manifold",
    "stream": False
}
r5c = query_ollama("/api/generate", p5c)
print("Raw r5c response:")
print(json.dumps(r5c, indent=2))
