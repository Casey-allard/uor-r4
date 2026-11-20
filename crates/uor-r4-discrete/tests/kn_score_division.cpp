// Compile with a host C++17 compiler. This tests the exact function included
// by the CUDA scorer, against the host compiler's unsigned 128-bit quotient.
#include "../src/kn_score_division.cuh"
#include <array>
#include <cassert>
#include <cstdio>
#include <limits>

using kn_score_division::Word;
using kn_score_division::Wide;

static Word next(Word &state) {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    return state;
}

int main() {
    const Word max = std::numeric_limits<Word>::max();
    const std::array<Word, 13> edges = {
        0, 1, 2, 3, 4095, 4096, (1ull << 32) - 1, 1ull << 32,
        1ull << 48, (1ull << 63) - 1, 1ull << 63, max - 1, max,
    };
    unsigned cases = 0;
    unsigned high_paths = 0;
    auto check = [&](Word hi, Word lo, Word d) {
        const Wide numerator = (Wide(hi) << 64) | Wide(lo);
        assert(kn_score_division::divide(numerator, d) == numerator / d);
        ++cases;
        high_paths += unsigned(hi >= d);
    };
    for (Word d : edges) {
        if (d == 0) continue;
        for (Word hi : edges) {
            for (Word lo : edges) check(hi, lo, d);
        }
    }
    Word state = 0xc8c73e4deb15d4c3ull;
    for (unsigned i = 0; i < 100000; ++i) {
        const Word hi = next(state);
        const Word lo = next(state);
        const Word d = next(state) | 1;
        check(hi, lo, d);
    }
    assert(high_paths > 45000 && high_paths < 55000);
    std::printf("%u exact quotients, %u high-word division paths\n", cases, high_paths);
}
