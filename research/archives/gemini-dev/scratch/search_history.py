import json
import os
import sys

logfile = "/Users/adminamn/.gemini/antigravity/brain/6b7b60ba-8d52-4cec-a3de-ef4d9f1c008d/.system_generated/logs/transcript.jsonl"
if not os.path.exists(logfile):
    print("Logfile not found")
    sys.exit(1)

print("Scanning for prime_router_package.py write events...")
matches = []
with open(logfile, "r") as f:
    for line in f:
        if "prime_router_package.py" in line:
            try:
                step = json.loads(line)
                matches.append(step)
            except:
                pass

print(f"Found {len(matches)} events for prime_router_package.py.")
for step in matches:
    print(f"Step {step.get('step_index')}: Type={step.get('type')}")
