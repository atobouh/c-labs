#include "sum_values.h"

int sum_values(const int values[], size_t count)
{
    int total = 0;
    for (size_t i = 0; i < count; ++i) {
        total += values[i];
    }
    return total;
}
