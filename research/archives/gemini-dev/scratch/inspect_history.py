import json

logfile = "/Users/adminamn/.gemini/antigravity/brain/6b7b60ba-8d52-4cec-a3de-ef4d9f1c008d/.system_generated/logs/transcript.jsonl"

def inspect():
    with open(logfile, "r") as f:
        steps = []
        for line in f:
            if not line.strip():
                continue
            try:
                steps.append(json.loads(line))
            except Exception as e:
                pass
                
    for s in steps:
        idx = s.get("step_index", 0)
        if 5445 <= idx <= 5460:
            print(f"--- Step {idx} ---")
            print(f"Source: {s.get('source')}, Type: {s.get('type')}, Status: {s.get('status')}")
            # If it's a command result, print it
            if s.get("type") == "COMMAND_RUN" or s.get("type") == "COMMAND_OUTPUT":
                print(s.get("content"))
            elif s.get("tool_calls"):
                for tc in s["tool_calls"]:
                    print(f"TOOL: {tc['name']}")
                    if tc['name'] == 'run_command':
                        print(tc['args'].get('CommandLine'))
            elif s.get("content"):
                print(s.get("content")[:500])
            print("-" * 50)

if __name__ == "__main__":
    inspect()
