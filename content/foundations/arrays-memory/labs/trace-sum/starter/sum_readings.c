#include <stddef.h>

#include "sum_readings.h"

int sum_readings(const int *readings, size_t count, int *out_sum)
{
    if (out_sum == NULL) {
        return -1;
    }

    *out_sum = 0;
    (void)readings;
    (void)count;
    return 0;
}
