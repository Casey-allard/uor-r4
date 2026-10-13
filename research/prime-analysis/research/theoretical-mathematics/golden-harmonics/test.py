def count_divisors(n):
    count = 0
    for i in range(1, n + 1):
        if n % i == 0:
            count += 1
    return count

# Dataset of first 1024 numbers with primes marked
number_set = [(1, 'Non-prime'), (2, 'Non-prime'), (3, 'Non-prime'), ... , (1024, 'Non-prime')]

# Analyzing divisor counts near prime numbers
results = []
for number, status in number_set:
    if 'Prime' in status:
        divisor_count = count_divisors(number)
        results.append((number, divisor_count))

print('Divisor Counts Near Prime Numbers:', results)
