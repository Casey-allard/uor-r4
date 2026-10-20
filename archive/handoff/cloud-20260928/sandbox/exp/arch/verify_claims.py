import json, sys
path = "/root/.claude/projects/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/subagents/agent-a82d1aaed7b242fa8.jsonl"
tool_text = []
for line in open(path):
    try:
        o = json.loads(line)
    except Exception:
        continue
    msg = o.get("message") or {}
    content = msg.get("content")
    if isinstance(content, list):
        for c in content:
            if isinstance(c, dict) and c.get("type") == "tool_result":
                cc = c.get("content")
                if isinstance(cc, list):
                    for x in cc:
                        if isinstance(x, dict) and x.get("type") == "text":
                            tool_text.append(x.get("text",""))
                elif isinstance(cc, str):
                    tool_text.append(cc)
    tr = o.get("toolUseResult")
    if tr is not None:
        tool_text.append(json.dumps(tr) if not isinstance(tr,str) else tr)
blob = "\n".join(tool_text)
print("tool-result chars:", len(blob))
needles = sys.argv[1:]
for n in needles:
    i = blob.find(n)
    if i < 0:
        print(f"MISSING  {n!r}")
    else:
        ctx = blob[max(0,i-140):i+160].replace("\n"," ")
        print(f"FOUND    {n!r}\n   ...{ctx}...")
