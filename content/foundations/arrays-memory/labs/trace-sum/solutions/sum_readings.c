#include <stddef.h>

#include "sum_readings.h"

int sum_readings(const int *readings, size_t count, int *out_sum)
{
    if (out_sum == NULL) {
        return -1;
    }
    *out_sum = 0;

    if (count > SENSOR_CAPACITY || (count > 0 && readings == NULL)) {
        return -1;
    }

    int total = 0;
    size_t i = 0;
    while (i < count) {
        if (readings[i] < SENSOR_MIN_READING || readings[i] > SENSOR_MAX_READING) {
            return -1;
        }
        total += readings[i];
        ++i;
    }

    *out_sum = total;
    return 0;
}
