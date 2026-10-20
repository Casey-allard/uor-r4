import urllib.request
import json
import time

OLLAMA_URL = "http://localhost:11434"
MODEL = "gemma4:e4b"

system_prompt_concise = """You are a geometric AI. Answer the query using the context. Keep your response extremely brief, strictly 1 short sentence.

Context:
  [1] Quantum mechanics is a fundamental theory in physics.
  [2] Resonance occurs when a system oscillates at specific frequencies."""

payload = {
    "model": MODEL,
    "system": system_prompt_concise,
    "prompt": "explain quantum resonance in the manifold",
    "stream": True  # Enable streaming!
}

print("Querying /api/generate with stream=True and early-stopping on punctuation...")
start_time = time.time()
data = json.dumps(payload).encode("utf-8")
req = urllib.request.Request(
    f"{OLLAMA_URL}/api/generate",
    data=data,
    headers={"Content-Type": "application/json"},
    method="POST"
)

try:
    with urllib.request.urlopen(req, timeout=30) as resp:
        text = ""
        sentence_count = 0
        token_count = 0
        
        # Read the stream line-by-line
        for line in resp:
            if not line:
                continue
            chunk = json.loads(line.decode("utf-8"))
            resp_text = chunk.get("response", "")
            text += resp_text
            token_count += 1
            
            # Print token in real-time
            print(repr(resp_text), end=" ", flush=True)
            
            # If we hit a sentence-ending punctuation and have some content, stop!
            if ("." in resp_text or "?" in resp_text or "!" in resp_text or "\n" in resp_text) and len(text.strip()) > 10:
                print("\n[Early stop triggered!]")
                break
                
        elapsed = time.time() - start_time
        print(f"\nSucceeded in {elapsed:.2f} seconds!")
        print(f"Final Text: {repr(text.strip())}")
        print(f"Tokens read: {token_count}")
except Exception as e:
    print(f"\nFailed: {e}")
