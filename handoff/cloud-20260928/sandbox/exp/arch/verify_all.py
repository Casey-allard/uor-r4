import json, sys
path = "/root/.claude/projects/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/subagents/agent-a82d1aaed7b242fa8.jsonl"
tool_text = []
for line in open(path):
    try: o = json.loads(line)
    except Exception: continue
    msg = o.get("message") or {}
    content = msg.get("content")
    if isinstance(content, list):
        for c in content:
            if isinstance(c, dict) and c.get("type") == "tool_result":
                cc = c.get("content")
                if isinstance(cc, list):
                    for x in cc:
                        if isinstance(x, dict) and x.get("type") == "text": tool_text.append(x.get("text",""))
                elif isinstance(cc, str): tool_text.append(cc)
blob = "\n".join(tool_text)
needle = sys.argv[1]; maxn = int(sys.argv[2]) if len(sys.argv)>2 else 3; w = int(sys.argv[3]) if len(sys.argv)>3 else 300
start = 0; n = 0
while n < maxn:
    i = blob.find(needle, start)
    if i < 0: break
    print(f"--- [{n}] ...{blob[max(0,i-w):i+w]}...".replace("\n"," "))
    start = i + len(needle); n += 1
if n == 0: print("MISSING", needle)
