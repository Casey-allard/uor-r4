import time
import numpy as np
import math

M_MAX = 512
np.random.seed(42)
P_PROJ = np.random.randn(512, 2)
Q_PROJ, _ = np.linalg.qr(P_PROJ)

def get_sentence_projection(state_vector: np.ndarray, win_idx: int) -> tuple[float, float]:
    u_raw, v_raw = state_vector @ Q_PROJ
    angle = (win_idx / 16.0) * 2.0 * math.pi
    radius = 20.0
    u = radius * math.cos(angle) + u_raw * 5.0
    v = radius * math.sin(angle) + v_raw * 5.0
    return float(u), float(v)

# Simulate 23466 state vectors
num_items = 23466
states = [np.random.randn(M_MAX) for _ in range(num_items)]
win_indices = [np.random.randint(1, 17) for _ in range(num_items)]

print(f"Profiling projection of {num_items} vectors...")
t0 = time.time()
projections = []
for state, win_idx in zip(states, win_indices):
    u, v = get_sentence_projection(state, win_idx)
    projections.append((u, v))
t1 = time.time()
print(f"Completed in {t1 - t0:.4f} seconds!")
