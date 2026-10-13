import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def run_test(name, payload):
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{OLLAMA_URL}/api/generate",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    start = time.time()
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            res_json = json.loads(resp.read().decode("utf-8"))
            content = res_json.get("response", "")
            print(f"[{name}] Output: {repr(content)} | Reason: {res_json.get('done_reason')} | Eval: {res_json.get('eval_count')} | Time: {time.time() - start:.2f}s")
    except Exception as e:
        print(f"[{name}] Failed: {e}")

# 1. System prompt + num_predict=50
run_test(
    "1. System Param + num_predict=50",
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

# 2. System prompt only (No num_predict)
run_test(
    "2. System Param (No num_predict)",
    {
        "model": MODEL,
        "system": "You are a geometric AI. Speak scientifically.",
        "prompt": "Explain quantum resonance in 1 sentence.",
        "stream": False
    }
)

# 3. Concatenated prompt + num_predict=50 (No system parameter)
run_test(
    "3. Concatenated Prompt + num_predict=50",
    {
        "model": MODEL,
        "prompt": "<start_of_turn>user\nYou are a geometric AI. Speak scientifically.\n\nExplain quantum resonance in 1 sentence.<end_of_turn>\n<start_of_turn>model\n",
        "stream": False,
        "options": {
            "num_predict": 50
        }
    }
)

# 4. Concatenated prompt + num_predict=50 with original system prompt
system_prompt_original = """You are the R4 Prime Router World Model — a geometric AI whose knowledge is indexed on a Riemann zeta manifold. Your response is geometrically grounded: you have been routed to Scale Window 1 (Origins & Foundations) with curvature κ=0.0230, deficit θd=-1.6200. Routing regime: resonant (symmetric orbit) — answers should be precise and convergent.

The following corpus sentences were retrieved as your highest-resonance context — use them as your primary knowledge source and synthesise a clear, accurate answer:

  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies.

Answer the user's query directly and accurately. Keep your response extremely brief, strictly 1 or 2 sentences max (under 40 words)."""

run_test(
    "4. Concatenated Original + num_predict=50",
    {
        "model": MODEL,
        "prompt": f"<start_of_turn>user\n{system_prompt_original}\n\nexplain quantum resonance in the manifold<end_of_turn>\n<start_of_turn>model\n",
        "stream": False,
        "options": {
            "num_predict": 50
        }
    }
)
