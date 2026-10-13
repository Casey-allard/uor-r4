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
    worker_count = max(1, cpu_count() // 2)
    print(f"Using {worker_count} workers.")
    pool = Pool(processes=worker_count)
    start_time = time.time()

    try:
        for i, result in enumerate(pool.imap(process_chunk, chunks), 1):
            current_chunk = (last_processed_chunk or 0) + i
            print(f"Processed chunk {current_chunk}/{total_chunks}")

            buffer.extend(result)

            # Write primes for single-chunk scenarios
            if total_chunks == 1:
                save_primes_to_file(buffer)
                buffer.clear()

            # Skip worker restart for small ranges
            if total_chunks > 1 and current_chunk % 500 == 0:
                pool = restart_worker_pool(pool, worker_count)

            # Log memory usage for larger ranges
            if total_chunks > 1 and current_chunk % 100 == 0:
                log_memory_usage()

            if current_chunk % 10 == 0:
                save_checkpoint(current_chunk)

        if buffer:
            save_primes_to_file(buffer)

    except KeyboardInterrupt:
        print("\nExecution interrupted by user.")
    except Exception as e:
        print(f"Multiprocessing error: {e}")
    finally:
        print("Shutting down workers...")
        terminate_workers(pool)

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
