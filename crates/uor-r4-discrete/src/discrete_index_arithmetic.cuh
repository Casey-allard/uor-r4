#pragma once

namespace discrete_index_arithmetic {
using Index = unsigned int;
using Word = unsigned long long;

#if defined(__CUDACC__)
#define DISCRETE_INDEX_FUNCTION __host__ __device__ __forceinline__
#else
#define DISCRETE_INDEX_FUNCTION inline
#endif

// Exact quotient for a positive width. Encode's usual power-of-two widths
// remain a shift. The restoring path avoids compiler-supplied floating
// reciprocals for the other admitted layouts.
DISCRETE_INDEX_FUNCTION Index token_index(Index element, Index width) {
    if ((width & (width - 1)) == 0) {
#if defined(__CUDA_ARCH__)
        return element >> (unsigned(__ffs(static_cast<int>(width))) - 1u);
#else
        return element >> unsigned(__builtin_ctz(width));
#endif
    }
    Index trial = width;
    Index bit = 1;
    while (trial <= (element >> 1)) {
        trial <<= 1;
        bit <<= 1;
    }
    Index quotient = 0;
    do {
        if (element >= trial) {
            element -= trial;
            quotient |= bit;
        }
        trial >>= 1;
        bit >>= 1;
    } while (bit != 0);
    return quotient;
}

// Exact remainder for a positive carrier divisor. Shift the trial divisor
// only while it fits below the dividend, including at the u64 boundary.
DISCRETE_INDEX_FUNCTION Word carrier_index(Word value, Word divisor) {
    if ((divisor & (divisor - 1)) == 0) return value & (divisor - 1);
    Word trial = divisor;
    while (trial <= (value >> 1)) trial <<= 1;
    do {
        if (value >= trial) value -= trial;
        trial >>= 1;
    } while (trial >= divisor);
    return value;
}

#undef DISCRETE_INDEX_FUNCTION
} // namespace discrete_index_arithmetic
