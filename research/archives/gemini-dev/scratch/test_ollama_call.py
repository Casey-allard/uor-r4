import sys
import os

# Add parent directory to path to import server
sys.path.append(os.path.abspath('.'))

import server

print("USE_OLLAMA:", server.USE_OLLAMA)
print("OLLAMA_URL:", server.OLLAMA_URL)
print("OLLAMA_MODEL:", server.OLLAMA_MODEL)

# Define custom test to print details of raw HTTP request and response
import urllib.request
import json

prompt = "explain quantum resonance in the manifold"
context_sentences = ["Quantum mechanics is a fundamental theory in physics.", "Resonance occurs when a system oscillates at specific frequencies."]
window_idx = 1
metrics = {"kappa": 0.023, "deficit_angle": -1.62}

theme = server.WINDOW_THEMES_SHORT.get(window_idx, f"Window {window_idx}")
kappa = metrics.get("kappa", 0.0)
deficit = metrics.get("deficit_angle", 0.0)
regime = "resonant (symmetric orbit) — answers should be precise and convergent"

ctx_block = "\n".join(f"  [{i+1}] {s}" for i, s in enumerate(context_sentences[:5]))

system_prompt = f"""You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed \
on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to \
Scale Window {window_idx} ({theme}) with curvature κ={kappa:.4f}, deficit θd={deficit:.4f}. \
Routing regime: {regime}.

The following corpus sentences were retrieved as your highest-resonance context — use them \
as your primary knowledge source and synthesise a clear, accurate answer:

{ctx_block}

Answer the user's query directly and accurately. If the context is sufficient, answer from it. \
If not, draw on your general knowledge. Keep the answer concise (2-4 sentences)."""

full_prompt = f"{system_prompt}\n\nUser: {prompt}\nAssistant:"

body = {
    "model": server.OLLAMA_MODEL,
    "prompt": full_prompt,
    "stream": False,
    "options": {
        "temperature": 0.7,
        "num_predict": 400,
        "stop": ["User:", "\n\nUser"],
    }
}
print("Request body:")
print(json.dumps(body, indent=2))

data = json.dumps(body).encode("utf-8")
url = f"{server.OLLAMA_URL}/api/generate"
print("Url:", url)

try:
    req = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        print("Status code:", resp.status)
        raw_body = resp.read()
        print("Raw response body:")
        print(raw_body.decode("utf-8"))
        data = json.loads(raw_body.decode("utf-8"))
        print("Parsed response:", data.get("response"))
except Exception as e:
    print("Ollama call failed with exception:", e)

