#pragma once

namespace kn_score_division {
using Word = unsigned long long;
using Wide = unsigned __int128;

// One word of restoring division. The incoming remainder is below the
// positive denominator, so each next bit needs at most one subtraction.
#if defined(__CUDACC__)
#define KN_DIV_FUNCTION __host__ __device__ __forceinline__
#else
#define KN_DIV_FUNCTION inline
#endif

KN_DIV_FUNCTION Word divide_word(Word input, Word denominator, Wide &remainder) {
    Word quotient = 0;
    for (int bit = 63; bit >= 0; --bit) {
        remainder = (remainder << 1) | Wide((input >> bit) & 1ull);
        if (remainder >= Wide(denominator)) {
            remainder -= Wide(denominator);
            quotient |= 1ull << bit;
        }
    }
    return quotient;
}

// Exact floor(numerator / denominator) for denominator > 0. Explicit shift
// and subtraction keep ptxas from lowering a variable divide into a
// floating-point reciprocal. A small high word is already a remainder.
KN_DIV_FUNCTION Wide divide(Wide numerator, Word denominator) {
    const Word high = Word(numerator >> 64);
    Word quotient_high = 0;
    Wide remainder = high;
    if (high >= denominator) {
        remainder = 0;
        quotient_high = divide_word(high, denominator, remainder);
    }
    const Word quotient_low = divide_word(Word(numerator), denominator, remainder);
    return (Wide(quotient_high) << 64) | Wide(quotient_low);
}

#undef KN_DIV_FUNCTION
}  // namespace kn_score_division
