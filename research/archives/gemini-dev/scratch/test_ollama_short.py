import json
import urllib.request
import urllib.error
import time

def test_ollama():
    prompt = "Tell me about the Gambia borehole locations."
    context = "I can help you coordinate water borehole data for the Gambia project."
    
    # Let's test the compact system prompt structure
    kappa = 0.8716
    deficit = -1.2045
    window_idx = 6
    theme = "Hyperbolic Fluctuation"
    
    system_prompt = f"R4 Prime Router (W{window_idx}, κ={kappa:.4f}, θd={deficit:.4f}). Context: {context}. Answer directly, under 30 words."
    full_prompt = f"System: {system_prompt}\nUser: {prompt}\nAssistant:"
    
    body = json.dumps({
        "model": "gemma4:e4b",
        "prompt": full_prompt,
        "stream": True,
        "options": {
            "temperature": 0.7,
            "stop": ["User:", "\n\nUser"],
        }
    }).encode("utf-8")
    
    print("Sending request to Ollama...")
    t0 = time.time()
    try:
        req = urllib.request.Request(
            "http://localhost:11434/api/generate",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=90) as resp:
            text = ""
            for line in resp:
                if not line:
                    continue
                chunk = json.loads(line.decode("utf-8"))
                resp_text = chunk.get("response", "")
                text += resp_text
                print(resp_text, end="", flush=True)
                
                # Early stop condition
                if ("." in resp_text or "?" in resp_text or "!" in resp_text or "\n" in resp_text) and len(text.strip()) > 15:
                    print("\n[Early stopped]")
                    break
            print(f"\nTotal time: {time.time() - t0:.2f}s")
            print("Response:", text.strip())
    except Exception as e:
        print("Error calling Ollama:", e)

if __name__ == "__main__":
    test_ollama()
