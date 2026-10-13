import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

def test_num_predict(num_predict_val):
    payload = {
        "model": MODEL,
        "prompt": "Say hello in 5 words.",
        "stream": False,
        "options": {
            "num_predict": num_predict_val
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
            print(f"num_predict={num_predict_val} | Output: {repr(content)} | Reason: {res_json.get('done_reason')} | Time: {time.time() - start:.2f}s")
    except Exception as e:
        print(f"num_predict={num_predict_val} | Failed: {e}")

test_num_predict(20)
test_num_predict(50)
test_num_predict(100)
