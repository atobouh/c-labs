#include <stddef.h>

#include "sum_readings.h"

int main(void)
{
    const int exact_size[] = {10, 20, 30};
    int sum = 0;
    const int status = sum_readings(exact_size, 3, &sum);
    return status == 0 && sum == 60 ? 0 : 1;
}
