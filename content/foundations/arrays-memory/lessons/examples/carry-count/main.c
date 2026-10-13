#include <stddef.h>
#include <stdio.h>

#include "sum_values.h"

int main(void)
{
    const int readings[] = {12, 18, 9, 21, 15, 24};
    const size_t count = sizeof readings / sizeof readings[0];
    const int sum = sum_values(readings, count);

    printf("sum=%d (count=%zu)\n", sum, count);
    return 0;
}
