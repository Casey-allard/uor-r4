# Copy the lead's tiny bf16 checkpoint with every q_proj/k_proj multiplied by 3 (exact in bf16 up to rounding).
import json, struct, numpy as np, sys
src, dst, c = sys.argv[1], sys.argv[2], float(sys.argv[3])
raw = open(src, "rb").read(); n = struct.unpack("<Q", raw[:8])[0]; hdr = json.loads(raw[8:8+n]); body = bytearray(raw[8+n:])
for name, m in hdr.items():
    if name == "__metadata__" or not (name.endswith("q_proj.weight") or name.endswith("k_proj.weight")): continue
    a, b = m["data_offsets"]; u = np.frombuffer(bytes(body[a:b]), dtype=np.uint16).astype(np.uint32) << 16
    f = u.view(np.float32) * c
    body[a:b] = (f.view(np.uint32) >> 16).astype(np.uint16).tobytes()
open(dst, "wb").write(raw[:8+n] + bytes(body))
