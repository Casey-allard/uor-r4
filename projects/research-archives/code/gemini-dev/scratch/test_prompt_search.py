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
        with urllib.request.urlopen(req, timeout=15) as resp:
            raw = resp.read().decode("utf-8")
            return json.loads(raw)
    except Exception as e:
        return {"error": str(e)}

system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

print("--- Variation 1: Chat API with official messages (Original System Prompt) ---")
p1 = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": system_prompt_original},
        {"role": "user", "content": "explain quantum resonance in the manifold"}
    ],
    "stream": False,
    "options": {
        "num_predict": 50,
        "temperature": 0.3
    }
}
r1 = query_ollama("/api/chat", p1)
print("Response:", json.dumps(r1.get("message", {}).get("content"), indent=2))
print("Done Reason:", r1.get("done_reason"))

print("\n--- Variation 2: Generate API with System Parameter (Ollama Native System Prompt support) ---")
p2 = {
    "model": MODEL,
    "system": system_prompt_original,
    "prompt": "explain quantum resonance in the manifold",
    "stream": False,
    "options": {
        "num_predict": 50,
        "temperature": 0.3
    }
}
r2 = query_ollama("/api/generate", p2)
print("Response:", json.dumps(r2.get("response"), indent=2))
print("Done Reason:", r2.get("done_reason"))

print("\n--- Variation 3: Chat API with slightly simplified System Prompt ---")
system_prompt_simple = """You are the R4 Prime Router World Model. You are routed to Scale Window 1 with curvature kappa=0.0230, deficit theta_d=-1.6200.

Context:
- Quantum mechanics is a fundamental theory in physics.
- Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately using the context. Keep the answer to 2-3 sentences."""

p3 = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": system_prompt_simple},
        {"role": "user", "content": "explain quantum resonance in the manifold"}
    ],
    "stream": False,
    "options": {
        "num_predict": 50,
        "temperature": 0.3
    }
}
r3 = query_ollama("/api/chat", p3)
print("Response:", json.dumps(r3.get("message", {}).get("content"), indent=2))
print("Done Reason:", r3.get("done_reason"))

print("\n--- Variation 4: Generate API with standard Gemma prompt formatting (no chat endpoints, no system param) ---")
# Gemma uses <start_of_turn>user and <start_of_turn>model
gemma_prompt = f"""<start_of_turn>user
{system_prompt_simple}

User query: explain quantum resonance in the manifold<end_of_turn>
<start_of_turn>model
"""
p4 = {
    "model": MODEL,
    "prompt": gemma_prompt,
    "stream": False,
    "options": {
        "num_predict": 50,
        "temperature": 0.3
    }
}
r4 = query_ollama("/api/generate", p4)
print("Response:", json.dumps(r4.get("response"), indent=2))
print("Done Reason:", r4.get("done_reason"))
