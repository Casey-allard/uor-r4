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
        return {"error": str(e)}

print("--- Test 1: Simple Prompt ---")
p1 = {
    "model": MODEL,
    "prompt": "Hello! Introduce yourself briefly.",
    "stream": False
}
r1 = query_ollama("/api/generate", p1)
print("Response:", json.dumps(r1.get("response"), indent=2))
print("Done Reason:", r1.get("done_reason"))
print("Eval count:", r1.get("eval_count"))

print("\n--- Test 2: Prompt with System Prompt but no options/stop ---")
system_prompt = "You are a helpful assistant. Be concise."
p2 = {
    "model": MODEL,
    "system": system_prompt,
    "prompt": "Explain quantum resonance in 1 sentence.",
    "stream": False
}
r2 = query_ollama("/api/generate", p2)
print("Response:", json.dumps(r2.get("response"), indent=2))
print("Done Reason:", r2.get("done_reason"))
print("Eval count:", r2.get("eval_count"))

print("\n--- Test 3: Prompt with stop tokens list ---")
p3 = {
    "model": MODEL,
    "prompt": "User: Hello\nAssistant:",
    "stream": False,
    "options": {
        "stop": ["User:", "\n\nUser"]
    }
}
r3 = query_ollama("/api/generate", p3)
print("Response:", json.dumps(r3.get("response"), indent=2))
print("Done Reason:", r3.get("done_reason"))
print("Eval count:", r3.get("eval_count"))

print("\n--- Test 4: Chat API with simple system + user message ---")
p4 = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": "You are a geometric router AI. Speak scientifically."},
        {"role": "user", "content": "Explain quantum resonance in 1 sentence."}
    ],
    "stream": False
}
r4 = query_ollama("/api/chat", p4)
print("Response:", json.dumps(r4.get("message", {}).get("content"), indent=2))
print("Done Reason:", r4.get("done_reason"))
print("Eval count:", r4.get("eval_count"))

print("\n--- Test 5: Original Prompt with Chat API ---")
system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. If the context is sufficient, answer from it. If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

p5 = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": system_prompt_original},
        {"role": "user", "content": "explain quantum resonance in the manifold"}
    ],
    "stream": False
}
r5 = query_ollama("/api/chat", p5)
print("Response:", json.dumps(r5.get("message", {}).get("content"), indent=2))
print("Done Reason:", r5.get("done_reason"))
print("Eval count:", r5.get("eval_count"))
