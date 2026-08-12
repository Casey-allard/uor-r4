import numpy as np
import json
import os

proxy_dir = "/Users/adminamn/gemini-dev/wikitext2_proxy"

files = ["wikitext2_tokens.npz", "wikitext2_proxy.npz", "ppmi_emb.npz"]
for f in files:
    p = os.path.join(proxy_dir, f)
    if os.path.exists(p):
        print(f"=== File: {f} ===")
        try:
            data = np.load(p, allow_pickle=True)
            print("Keys:", list(data.keys()))
            for k in data.keys():
                val = data[k]
                print(f"  Key '{k}': shape {getattr(val, 'shape', None)}, type {type(val)}")
                if isinstance(val, np.ndarray) and val.ndim == 1 and val.size < 20:
                    print("  Value:", val)
                elif isinstance(val, np.ndarray) and val.ndim == 0:
                    print("  Value:", val.item())
        except Exception as e:
            print("Error loading:", e)
    else:
        print(f"File not found: {p}")
