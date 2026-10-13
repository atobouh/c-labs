#include <stddef.h>
#include <stdio.h>

#include "sum_readings.h"
#include "../fixtures/fixtures.h"

static int failures;

static void emit(const char *check_id, int passed)
{
    printf("{\"check_id\":\"%s\",\"passed\":%s}\n",
           check_id, passed ? "true" : "false");
    if (!passed) {
        ++failures;
    }
}

static int matches_sum(const int *readings, size_t count, int expected_status, int expected_sum)
{
    int actual_sum = 777;
    const int status = sum_readings(readings, count, &actual_sum);
    return status == expected_status && actual_sum == expected_sum;
}

int main(void)
{
    emit("sample-sum", matches_sum(fixture_sample, 6, 0, 99));
    emit("empty-null", matches_sum(NULL, 0, 0, 0));
    emit("single-value", matches_sum(fixture_single, 1, 0, 42));
    emit("zeros", matches_sum(fixture_zeros, 3, 0, 0));
    emit("max-capacity", matches_sum(fixture_max_capacity, 30, 0, 30000));
    emit("invalid-low", matches_sum(fixture_below_range, 1, -1, 0));
    emit("invalid-high", matches_sum(fixture_above_range, 1, -1, 0));
    emit("too-many", matches_sum(fixture_too_many, 31, -1, 0));
    emit("null-input", matches_sum(NULL, 1, -1, 0));
    emit("null-output", sum_readings(fixture_sample, 6, NULL) == -1);
    return failures == 0 ? 0 : 1;
}
