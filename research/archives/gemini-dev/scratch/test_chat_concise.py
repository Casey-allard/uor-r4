import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

payload = {
    "model": MODEL,
    "messages": [
        {"role": "system", "content": "You are a geometric AI. Answer in exactly 1 short sentence of under 15 words. Be extremely brief."},
        {"role": "user", "content": "explain quantum resonance"}
    ],
    "stream": False
}

print("Querying /api/chat with strict 1-sentence constraint...")
start_time = time.time()
data = json.dumps(payload).encode("utf-8")
req = urllib.request.Request(
    f"{OLLAMA_URL}/api/chat",
    data=data,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    with urllib.request.urlopen(req, timeout=30) as resp:
        elapsed = time.time() - start_time
        res_json = json.loads(resp.read().decode("utf-8"))
        content = res_json.get("message", {}).get("content", "")
        print(f"Succeeded in {elapsed:.2f} seconds!")
        print(f"Output: {repr(content)}")
        print(f"Reason: {res_json.get('done_reason')}")
        print(f"Eval: {res_json.get('eval_count')}")
except Exception as e:
    print(f"Failed: {e}")
