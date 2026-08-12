import json
import numpy as np

cache_file = "/Users/adminamn/gemini-dev/manifold_cache.json"
print("Loading cache...")
with open(cache_file, "r") as f:
    cache = json.load(f)

vocab = cache["vocabulary"]
word_primes = cache["word_primes"]
vocab_vectors = cache["vocab_vectors"]
corpus_index = cache["corpus_index"]

print(f"Vocabulary size: {len(vocab)}")
print(f"Number of primes registered: {len(word_primes)}")

for term in ["r4", "sph", "uor", "router", "gateway", "default"]:
    in_vocab = term in vocab
    in_primes = term in word_primes
    in_vectors = term in vocab_vectors
    prime_val = word_primes.get(term, None)
    print(f"Term '{term}': in_vocab={in_vocab}, in_primes={in_primes}, in_vectors={in_vectors}, prime={prime_val}")

print("\nCounting occurrences of terms in corpus sentences:")
sentence_counts = {"r4": 0, "sph": 0, "uor": 0, "router": 0, "gateway": 0, "default": 0}
for win_idx, items in corpus_index.items():
    for item in items:
        sentence = item["sentence"].lower()
        for term in sentence_counts:
            if term in sentence:
                sentence_counts[term] += 1

for term, count in sentence_counts.items():
    print(f"  '{term}': found in {count} sentences")
