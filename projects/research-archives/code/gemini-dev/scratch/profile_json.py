import time
import json
import random

num_items = 23466
points = []
for i in range(num_items):
    points.append({
        "sentence": "Physics is the scientific study of matter, its fundamental constituents, its motion and behavior through space and time,"[:120],
        "window_index": random.randint(1, 16),
        "u": random.uniform(-100, 100),
        "v": random.uniform(-100, 100),
        "kappa": random.uniform(0.001, 0.1),
        "prime_product_mod": random.randint(1, 10000)
    })

payload = {"points": points, "total": len(points)}

print("Profiling JSON serialization of map data...")
t0 = time.time()
serialized = json.dumps(payload)
t1 = time.time()
print(f"json.dumps completed in {t1 - t0:.4f} seconds!")

t2 = time.time()
encoded = serialized.encode('utf-8')
t3 = time.time()
print(f"encode completed in {t3 - t2:.4f} seconds!")
print(f"Total size: {len(encoded) / 1024 / 1024:.2f} MB")
