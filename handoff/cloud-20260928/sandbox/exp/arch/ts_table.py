import json, re, sys
sys.argv = ["x", "zzzz_never"]
src = open("verify_clean.py").read()
src = src.split("needle = sys.argv[1]")[0]
exec(src)
print("blob len", len(blob))
i = blob.find("1M 8 layers")
print("idx", i)
seg = blob[i-50:i+15000]
for m in re.finditer(r"(\d+(?:\.\d+)?M\s+\d+\s+layers?)|Consistency:\s*(\d+)/10|Prompt\s", seg):
    print(m.group(0))
