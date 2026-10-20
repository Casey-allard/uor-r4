import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def run_test(prompt):
    payload = {
        "model": MODEL,
        "prompt": prompt,
        "stream": False,
        "options": {
            "num_predict": 50
        }
    }
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
            print(f"Prompt: {repr(prompt)} | Output: {repr(content)} | Reason: {res_json.get('done_reason')} | Time: {time.time() - start:.2f}s")
    except Exception as e:
        print(f"Prompt: {repr(prompt)} | Failed: {e}")

run_test("Say hello")
run_test("Explain quantum")
run_test("Explain quantum resonance")
run_test("Explain quantum resonance in 1 sentence.")
run_test("What is a borehole?")
