// Compare the exact helpers included by ordinary CUDA against native
// unsigned quotient and remainder on the host.
#include "../src/discrete_index_arithmetic.cuh"
#include <array>
#include <cassert>
#include <cstdio>
#include <limits>

using discrete_index_arithmetic::Index;
using discrete_index_arithmetic::Word;

static Word next(Word &state) {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

int main() {
    const Index index_max = std::numeric_limits<Index>::max();
    const Word word_max = std::numeric_limits<Word>::max();
    unsigned encode_cases = 0, carrier_cases = 0;
    auto encode = [&](Index element, Index width) {
        assert(discrete_index_arithmetic::token_index(element, width) == element / width);
        ++encode_cases;
    };
    auto carrier = [&](Word value, Word divisor) {
        assert(discrete_index_arithmetic::carrier_index(value, divisor) == value % divisor);
        ++carrier_cases;
    };
    for (Index width = 8; width <= 128; width += 8) {
        for (Index i = 0; i <= 1024; ++i) encode(i, width);
        for (Index i = 0; i <= 1024; ++i) encode(index_max - i, width);
    }
    const std::array<Word, 13> edges = {
        0, 1, 2, 3, 4095, 4096, Index(-1), 1ull << 32,
        1ull << 48, (1ull << 63) - 1, 1ull << 63, word_max - 1, word_max,
    };
    for (Word divisor : edges) {
        if (divisor != 0) for (Word value : edges) carrier(value, divisor);
    }
    Word state = 0x3e87389caff16e7full;
    for (unsigned i = 0; i < 100000; ++i) {
        const Word value = next(state), divisor = next(state) | 1u;
        encode(Index(value), Index(divisor));
        carrier(value, divisor);
        carrier(value, Index(divisor));
    }
    std::printf("%u exact encode quotients, %u exact carrier remainders\n", encode_cases, carrier_cases);
}
