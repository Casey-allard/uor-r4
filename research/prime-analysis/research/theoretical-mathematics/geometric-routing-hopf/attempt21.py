import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import time
import pickle
import gc


def calculate_segment_size(M):
    """Determine segment size based on available memory."""
    total_memory = psutil.virtual_memory().available
    num_workers = cpu_count()
    segment_size = min(16 * 1024 * 1024, (total_memory // num_workers) // 64)
    return max(segment_size, 5 * 10**5)


def sieve_segment(low, high, small_primes):
    """Sieve a segment of the range [low, high)."""
    segment_size = high - low
    is_prime_segment = bitarray(segment_size)
    is_prime_segment.setall(True)

    try:
        for prime in small_primes:
            start = max(prime * prime, ((low + prime - 1) // prime) * prime)
            for j in range(start, high, prime):
                is_prime_segment[j - low] = False
        primes = [low + i for i in range(segment_size) if is_prime_segment[i]]
    except Exception as e:
        print(f"Error processing segment {low}-{high}: {e}")
        primes = []

    return primes


def process_chunk(args):
    """Helper function to unpack arguments for multiprocessing."""
    return sieve_segment(*args)


def save_primes_to_file(primes, filename="primes.txt"):
    """Append primes to a file."""
    with open(filename, "a") as f:
        for prime in primes:
            f.write(f"{prime}\n")


def save_checkpoint(last_processed_chunk, checkpoint_file="checkpoint.pkl"):
    """Save intermediate results."""
    with open(checkpoint_file, "wb") as f:
        pickle.dump(last_processed_chunk, f)


def load_checkpoint(checkpoint_file="checkpoint.pkl"):
    """Load intermediate results."""
    try:
        with open(checkpoint_file, "rb") as f:
            return pickle.load(f)
    except (FileNotFoundError, EOFError, pickle.UnpicklingError):
        print("Checkpoint file missing or corrupted. Starting fresh.")
        return None


def compute_primes_segmented(M):
    """Compute primes up to M using a segmented sieve with multiprocessing."""
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
    if chunks[-1][1] != M:
        chunks.append((chunks[-1][1], M, small_primes))

    total_chunks = len(chunks)
    print(f"Total chunks to process: {total_chunks}")

    last_processed_chunk = load_checkpoint()
    if last_processed_chunk is not None:
        if last_processed_chunk >= total_chunks:
            print("Checkpoint exceeds the current range. Resetting checkpoint.")
            last_processed_chunk = None
            save_checkpoint(None)
        else:
            print(f"Resuming from checkpoint: {last_processed_chunk} processed.")
            chunks = chunks[last_processed_chunk:]

    buffer = []
    all_primes = []
    worker_count = max(1, cpu_count() // 2)
    print(f"Using {worker_count} workers.")
    start_time = time.time()

    try:
        with Pool(processes=worker_count) as pool:
            for i, result in enumerate(pool.imap(process_chunk, chunks), 1):
                current_chunk = (last_processed_chunk or 0) + i
                print(f"Processed chunk {current_chunk}/{total_chunks}")

                buffer.extend(result)
                all_primes.extend(result)

                if len(buffer) >= 100000:
                    save_primes_to_file(buffer)
                    buffer.clear()

                if current_chunk % 10 == 0:
                    save_checkpoint(current_chunk)

        if buffer:
            save_primes_to_file(buffer)
            all_primes.extend(buffer)

    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except Exception as e:
        print(f"Multiprocessing error: {e}")
    finally:
        print("Shutting down workers...")

    end_time = time.time()
    print(f"Total time: {end_time - start_time:.2f}s")

    return all_primes


if __name__ == "__main__":
    try:
        M = int(input("Enter the value of M: "))
        start_time = time.time()
        primes = compute_primes_segmented(M)
        end_time = time.time()

        if primes:
            print(f"Number of primes up to {M}: {len(primes)}")
            print(f"Last Prime Found: {primes[-1]}")
        else:
            print("No primes were found.")

        print(f"Time taken: {end_time - start_time:.2f} seconds")
        print(f"{math.ceil(math.log(M, 2))} bits required for Max {M}")
    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except ValueError:
        print("Invalid input. Please enter an integer.")
    except Exception as e:
        print(f"An error occurred: {e}")
