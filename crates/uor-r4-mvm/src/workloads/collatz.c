#include <stdio.h>

/* The native driver defines limit as an integer from 1 through 512. */
int main(void) {
    long longest = 0;
    long first = 1;
    long start = 1;
    while (start <= limit) {
        long value = start;
        long steps = 0;
        while (value != 1) {
            if (value % 2 == 0) value = value / 2;
            else value = 3 * value + 1;
            steps = steps + 1;
        }
        if (steps > longest) {
            longest = steps;
            first = start;
        }
        start = start + 1;
    }
    printf("limit=%ld start=%ld steps=%ld\n", limit, first, longest);
    return 0;
}
