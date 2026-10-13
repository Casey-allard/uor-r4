import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import time

def sieve_segment(low, high, small_primes):
    """Sieve a segment of the range [low, high)"""
    segment_size = high - low
    is_prime_segment = bitarray(segment_size)
    is_prime_segment.setall(True)

    try:
        for prime in small_primes:
            start = max(prime * prime, ((low + prime - 1) // prime) * prime)
            for j in range(start, high, prime):
                is_prime_segment[j - low] = False
    except Exception as e:
        print(f"Error in sieving range {low}-{high}: {e}")
        return []

    return [low + i for i in range(segment_size) if is_prime_segment[i]]

def compute_primes_segmented(M, segment_size):
    """Compute primes up to M using a segmented sieve with multiprocessing"""
    sqrt_limit = int(math.sqrt(M)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    # Regular sieve to find small primes
    for i in range(2, sqrt_limit):
        if sieve[i]:
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    small_primes = [i for i in range(2, sqrt_limit) if sieve[i]]

    # Divide the range into chunks
    chunks = [(low, min(low + segment_size, M), small_primes)
              for low in range(sqrt_limit, M, segment_size)]

    # Use multiprocessing to process chunks
    with Pool(processes=cpu_count()) as pool:
        results = pool.starmap(sieve_segment, chunks)

    # Combine results
    primes = small_primes + [prime for segment in results for prime in segment]

    return primes

def dynamic_segment_size(M):
    """Determine segment size based on available memory"""
    total_memory = psutil.virtual_memory().available
    # Use 10% of available memory for the segment size
    segment_size = max(10**6, min(total_memory // 10, M))
    return segment_size

if __name__ == "__main__":
    M = int(input("Enter the value of M: "))
    segment_size = int(input("Enter the segment size (Default: calculated dynamically): ") or dynamic_segment_size(M))

    start_time = time.time()
    primes = compute_primes_segmented(M, segment_size)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
    print(f"Last Prime Found: {primes[-1]}")
    print(f"{math.ceil(math.log(M, 2))} bits required for Max {M}")
