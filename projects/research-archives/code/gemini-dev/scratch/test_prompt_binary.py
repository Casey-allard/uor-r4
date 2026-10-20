import urllib.request
import json

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def test_config(name, sys_content, user_content):
    payload = {
        "model": MODEL,
        "messages": [
            {"role": "system", "content": sys_content},
            {"role": "user", "content": user_content}
        ],
        "stream": False,
        "options": {
            "num_predict": 30,
            "temperature": 0.3
        }
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{OLLAMA_URL}/api/chat",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            res_json = json.loads(resp.read().decode("utf-8"))
            content = res_json.get("message", {}).get("content", "")
            print(f"[{name}] Output: {repr(content)} | Reason: {res_json.get('done_reason')}")
    except Exception as e:
        print(f"[{name}] Failed: {e}")

# Base case (succeeds)
test_config(
    "1. Base Success Case",
    "You are a geometric router AI. Speak scientifically.",
    "Explain quantum resonance in 1 sentence."
)

# Test if word 'manifold' in query causes issues
test_config(
    "2. Word 'manifold' in user query",
    "You are a geometric router AI. Speak scientifically.",
    "explain quantum resonance in the manifold"
)

# Test if the context structure causes issues
test_config(
    "3. Context added to system prompt",
    "You are a geometric router AI. Speak scientifically.\n\nContext:\n- Quantum mechanics is a fundamental theory in physics.\n- Resonance occurs when a system oscillates at specific frequencies.",
    "Explain quantum resonance in 1 sentence."
)

# Test if metrics cause issues
test_config(
    "4. Curvature and deficit metrics added",
    "You are a geometric router AI. Speak scientifically. Scale Window 1, kappa=0.0230, theta_d=-1.6200.",
    "Explain quantum resonance in 1 sentence."
)

# Test if name 'R4 Prime Router World Model' causes issues
test_config(
    "5. Prime Router World Model in system prompt",
    "You are the R4 Prime Router World Model. Speak scientifically.",
    "Explain quantum resonance in 1 sentence."
)

# Test if combining all of them but keeping them in simple formatting works
test_config(
    "6. Combined Simple",
    "You are the R4 Prime Router. Speak scientifically. Scale Window 1, kappa=0.0230, theta_d=-1.6200. Context: [1] Quantum mechanics is a fundamental theory. [2] Resonance occurs at specific frequencies.",
    "Explain quantum resonance in 1 sentence."
)
