import numpy as np
import os

proxy_dir = "/Users/adminamn/gemini-dev/wikitext2_proxy"

tokens_data = np.load(os.path.join(proxy_dir, "wikitext2_tokens.npz"))
vocab_data = np.load(os.path.join(proxy_dir, "ppmi_emb.npz"), allow_pickle=True)

train_ids = tokens_data["train_ids"]
idx_to_token = vocab_data["idx_to_token"]

print("Train ids count:", len(train_ids))
print("Vocab size:", len(idx_to_token))

# Decode first 100 tokens
decoded_words = [idx_to_token[i] for i in train_ids[:100]]
print("First 100 decoded tokens:")
print(" ".join(decoded_words))
