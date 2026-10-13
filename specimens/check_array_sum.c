#include <assert.h>

#define main specimen_main
#include "array_sum.c"
#undef main

int main(void) {
    const int readings[] = {12, 18, 9, 21, 15, 24};
    const int single[] = {24};
    const int zeros[] = {0, 0, 0};
    assert(sum(readings, sizeof readings / sizeof readings[0]) == 99);
    assert(sum(NULL, 0) == 0);
    assert(sum(single, 1) == 24);
    assert(sum(zeros, 3) == 0);
    puts("Array sum: sample, empty, single, and zero inputs passed.");
    return 0;
}
