#include <stddef.h>
#include <stdio.h>

int main(void)
{
    const int samples[] = {12, 18, 9, 21, 15, 24};
    const size_t count = sizeof samples / sizeof samples[0];

    puts("index | value");
    for (size_t i = 0; i < count; ++i) {
        printf("%zu | %d\n", i, samples[i]);
    }
    return 0;
}
