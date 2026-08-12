import numpy as np


def sieve_is_prime(N: int) -> np.ndarray:
    is_p = np.ones(N + 1, dtype=bool)
    if N >= 0:
        is_p[:2] = False
    for p in range(2, int(N ** 0.5) + 1):
        if is_p[p]:
            is_p[p*p:N+1:p] = False
    return is_p
