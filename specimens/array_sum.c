#include <stdio.h>
#include <stddef.h>

int sum(const int *data, size_t count) {
    int total = 0;
    for (size_t i = 0; i < count; ++i) {
        total += data[i];
    }
    return total;
}

int main(void) {
    int samples[] = {12, 18, 9, 21, 15, 24};
    size_t count = sizeof samples / sizeof samples[0];
    printf("%d\n", sum(samples, count));
    return 0;
}
