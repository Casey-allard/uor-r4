import json, re, sys
sys.argv = ["x", "zzzz_never"]
src = open("verify_clean.py").read().split("needle = sys.argv[1]")[0]
exec(src)
m0 = re.search(r"1M\s+8\s+layers\s+What are you doing here", blob)
print("start", m0.start() if m0 else None)
seg = blob[m0.start()-50:m0.start()+20000]
for m in re.finditer(r"(\d+(?:\.\d+)?M\s+\d+\s+layers?)|(GPT2-XL|GPT-Neo|GPT2|1\.5B)|Consistency:\s*(\d+)/10|Prompt\s|Table\s+\d+", seg):
    print(repr(m.group(0)))
