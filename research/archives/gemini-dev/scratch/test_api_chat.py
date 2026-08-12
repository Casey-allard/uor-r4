import urllib.request
import json
import time

URL = "http://localhost:8000/api/chat"

payload = {
    "text": "explain how the r4 router works in this project",
    "temperature": 0.3,
    "max_tokens": 100
}

print("Querying /api/chat at http://localhost:8000/api/chat...")
start_time = time.time()
data = json.dumps(payload).encode("utf-8")
req = urllib.request.Request(
    URL,
    data=data,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    with urllib.request.urlopen(req, timeout=60) as resp:
        elapsed = time.time() - start_time
        res_json = json.loads(resp.read().decode("utf-8"))
        print(f"Succeeded in {elapsed:.2f} seconds!")
        print(f"Archetype: {res_json.get('archetype')}")
        print(f"Generation Mode: {res_json.get('generation_mode')}")
        print(f"Response (Description): {repr(res_json.get('description'))}")
        print(f"LLM Connected: {res_json.get('llm_connected')}")
        print(f"Routing Latency: {res_json.get('routing_latency_ms')} ms")
        print(f"Generation Latency: {res_json.get('gen_latency_ms')} ms")
except Exception as e:
    print(f"Failed: {e}")
