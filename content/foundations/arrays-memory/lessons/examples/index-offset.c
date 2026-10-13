#include <stddef.h>
#include <stdio.h>

int main(void)
{
    const int samples[] = {12, 18, 9, 21, 15, 24};
    const size_t count = sizeof samples / sizeof samples[0];
    int total = 0;

    for (size_t i = 0; i < count; ++i) {
        total += samples[i];
        printf("index=%zu value=%d total=%d\n", i, samples[i], total);
    }
    printf("sum=%d\n", total);
    return 0;
}
