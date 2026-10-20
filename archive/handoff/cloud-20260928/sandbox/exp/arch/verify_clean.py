import json, sys
path = "/root/.claude/projects/-home-user-uor-r4/e71805df-3dbc-5124-8067-d8a48084429c/subagents/agent-a82d1aaed7b242fa8.jsonl"
tool_inputs = {}
chunks = []
stop = False
for line in open(path):
    if stop: break
    try: o = json.loads(line)
    except Exception: continue
    msg = o.get("message") or {}
    content = msg.get("content")
    if not isinstance(content, list): continue
    for c in content:
        if not isinstance(c, dict): continue
        if c.get("type") == "tool_use":
            s = json.dumps(c.get("input"))
            if "verify_claims.py" in s or "verify_all.py" in s or "verify_clean.py" in s:
                stop = True; break
            tool_inputs[c.get("id")] = s
        elif c.get("type") == "tool_result":
            inp = tool_inputs.get(c.get("tool_use_id"), "")
            if "reports/arch.md" in inp: continue
            cc = c.get("content"); txt = ""
            if isinstance(cc, list):
                txt = "\n".join(x.get("text","") for x in cc if isinstance(x, dict) and x.get("type")=="text")
            elif isinstance(cc, str): txt = cc
            if "CS/ML architecture and scaling review (agent: arch)" in txt: continue
            chunks.append(txt)
blob = "\n".join(chunks)
needle = sys.argv[1]; maxn = int(sys.argv[2]) if len(sys.argv)>2 else 2; w = int(sys.argv[3]) if len(sys.argv)>3 else 250
start = 0; n = 0
while n < maxn:
    i = blob.find(needle, start)
    if i < 0: break
    print(f"--- [{n}] ...{blob[max(0,i-w):i+w]}...".replace("\n"," "))
    start = i + len(needle); n += 1
if n == 0: print("MISSING", needle)
