import sys
t = ("Hello, world!\n  two  spaces\ttab\r\n = Valkyria = \n<unk> @-@ 3 , 1 @,@ 000\n"
     "café 戦場の \U0001F600 it's we'll 12345\n"
     "<|im_start|>user\nhi<|im_end|>\n<|endoftext|>tail<|im_end|><|im_start|>" + "".join(chr(c) for c in range(1, 128)))
open(sys.argv[1], "w", encoding="utf-8").write(t)
b = t.encode("utf-8")
# expected ids: specials atomic, everything else byte+3
sp = {"<|endoftext|>": 0, "<|im_start|>": 1, "<|im_end|>": 2}
ids = []; i = 0
while i < len(t):
    for s, v in sp.items():
        if t.startswith(s, i):
            ids.append(v); i += len(s); break
    else:
        ids.extend(x + 3 for x in t[i].encode("utf-8")); i += 1
import struct
open(sys.argv[2], "wb").write(struct.pack("<%dH" % len(ids), *ids))
print("chars", len(t), "bytes", len(b), "expected tokens", len(ids))
