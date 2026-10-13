import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import os
import time
import pickle

def sieve_segment(low, high, small_primes):
    try:
        segment_size = high - low
        is_prime_segment = bitarray(segment_size)
        is_prime_segment.setall(True)

        for prime in small_primes:
            start = max(prime * prime, ((low + prime - 1) // prime) * prime)
            for j in range(start, high, prime):
                is_prime_segment[j - low] = False

        return [low + i for i in range(segment_size) if is_prime_segment[i]]

    except Exception as e:
        print(f"Error processing range {low}-{high}: {e}")
        return []

def calculate_segment_size(M):
    total_memory = psutil.virtual_memory().available
    num_workers = cpu_count()
    max_segment_size = (total_memory // num_workers) // 8  # Approx memory per worker
    return min(max(10**6, max_segment_size), M // num_workers)

def save_checkpoint(primes, checkpoint_file="primes_checkpoint.pkl"):
    with open(checkpoint_file, "wb") as f:
        pickle.dump(primes, f)

def load_checkpoint(checkpoint_file="primes_checkpoint.pkl"):
    try:
        with open(checkpoint_file, "rb") as f:
            return pickle.load(f)
    except FileNotFoundError:
        return []

def compute_primes_segmented(M, checkpoint_file="primes_checkpoint.pkl"):
    sqrt_limit = int(math.sqrt(M)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    for i in range(2, sqrt_limit):
        if sieve[i]:
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    small_primes = [i for i in range(2, sqrt_limit) if sieve[i]]

    segment_size = calculate_segment_size(M)
    print(f"Using segment size: {segment_size}")

    chunks = [(low, min(low + segment_size, M), small_primes)
              for low in range(sqrt_limit, M, segment_size)]

    primes = load_checkpoint(checkpoint_file)
    if primes:
        print(f"Resuming from checkpoint: {len(primes)} primes loaded.")

    try:
        with Pool(processes=cpu_count()) as pool:
            for i, result in enumerate(pool.imap(lambda args: sieve_segment(*args), chunks), 1):
                primes.extend(result)
                save_checkpoint(primes, checkpoint_file)
                print(f"Processed chunk {i}/{len(chunks)}")
    except Exception as e:
        print(f"Multiprocessing failed: {e}")

    return primes

if __name__ == "__main__":
    M = int(input("Enter the value of M: "))
    start_time = time.time()
    primes = compute_primes_segmented(M)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Last Prime Found: {primes[-1]}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
