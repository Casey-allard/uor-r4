import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray
import psutil
import time
import pickle
import gc
import os
import signal


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

    for prime in small_primes:
        start = max(prime * prime, ((low + prime - 1) // prime) * prime)
        for j in range(start, high, prime):
            is_prime_segment[j - low] = False

    return [low + i for i in range(segment_size) if is_prime_segment[i]]


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


def log_memory_usage():
    """Log memory and swap usage for debugging."""
    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    print(f"Memory usage: {mem.used / (1024**3):.2f} GB, Swap usage: {swap.used / (1024**3):.2f} GB")


def cleanup_zombie_processes():
    """Forcefully clean up all lingering processes."""
    print("Cleaning up zombie processes...")
    parent = psutil.Process(os.getpid())
    for child in parent.children(recursive=True):
        try:
            child.terminate()
            child.wait(timeout=5)
        except psutil.NoSuchProcess:
            pass
        except psutil.TimeoutExpired:
            child.kill()


def restart_worker_pool(worker_count):
    """Restart the worker pool and return it."""
    cleanup_zombie_processes()
    gc.collect()

    retries = 3
    for attempt in range(retries):
        try:
            print(f"Attempting to restart worker pool (Attempt {attempt + 1})...")
            new_pool = Pool(processes=worker_count)
            print("Worker pool restarted successfully.")
            return new_pool
        except Exception as e:
            print(f"Error restarting worker pool (Attempt {attempt + 1}): {e}")
            time.sleep(2)

    raise RuntimeError("Failed to restart worker pool after multiple attempts.")


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
    worker_count = max(1, cpu_count() // 2)
    pool = Pool(processes=worker_count)
    start_time = time.time()

    try:
        for i, result in enumerate(pool.imap(process_chunk, chunks), 1):
            current_chunk = (last_processed_chunk or 0) + i
            print(f"Processed chunk {current_chunk}/{total_chunks}")

            buffer.extend(result)

            if len(buffer) >= 50000:
                save_primes_to_file(buffer)
                buffer.clear()

            if current_chunk % 500 == 0:
                print(f"Restarting workers at chunk {current_chunk}...")
                pool.terminate()
                pool.join()
                pool = restart_worker_pool(worker_count)

            if current_chunk % 100 == 0:
                log_memory_usage()

            save_checkpoint(current_chunk)

        if buffer:
            save_primes_to_file(buffer)

    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        pool.terminate()
        pool.join()
        cleanup_zombie_processes()

    end_time = time.time()
    print(f"Total time: {end_time - start_time:.2f}s")

    return buffer


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
