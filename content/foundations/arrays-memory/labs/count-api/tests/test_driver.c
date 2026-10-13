#include <stddef.h>
#include <stdio.h>

#include "sensor_analysis.h"
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

static int near_mean(double actual, double expected)
{
    double difference = actual - expected;
    if (difference < 0.0) {
        difference = -difference;
    }
    return difference <= 1e-6;
}

static int matches(
    const int *readings,
    size_t count,
    enum analysis_status expected_status,
    size_t expected_count,
    int expected_sum,
    int expected_minimum,
    int expected_maximum,
    double expected_mean)
{
    struct sensor_stats actual = {777, 777, 777, 777, 777.0};
    const enum analysis_status status = analyze_readings(readings, count, &actual);
    return status == expected_status
        && actual.count == expected_count
        && actual.sum == expected_sum
        && actual.minimum == expected_minimum
        && actual.maximum == expected_maximum
        && near_mean(actual.mean, expected_mean);
}

int main(void)
{
    emit("sample-stats", matches(fixture_sample, 6, ANALYSIS_OK, 6, 99, 9, 24, 16.5));
    emit("empty-null", matches(NULL, 0, ANALYSIS_EMPTY, 0, 0, 0, 0, 0.0));
    emit("empty-nonnull", matches(fixture_single, 0, ANALYSIS_EMPTY, 0, 0, 0, 0, 0.0));
    emit("single-value", matches(fixture_single, 1, ANALYSIS_OK, 1, 42, 42, 42, 42.0));
    emit("zeros", matches(fixture_zeros, 3, ANALYSIS_OK, 3, 0, 0, 0, 0.0));
    emit("repeated-values", matches(fixture_repeated, 4, ANALYSIS_OK, 4, 28, 7, 7, 7.0));
    emit("fractional-mean", matches(fixture_fractional_mean, 3, ANALYSIS_OK, 3, 13, 1, 10, 13.0 / 3.0));
    emit("max-capacity", matches(fixture_max_capacity, 30, ANALYSIS_OK, 30, 30000, 1000, 1000, 1000.0));
    emit("invalid-low", matches(fixture_below_range, 2, ANALYSIS_READING_OUT_OF_RANGE, 0, 0, 0, 0, 0.0));
    emit("invalid-high", matches(fixture_above_range, 1, ANALYSIS_READING_OUT_OF_RANGE, 0, 0, 0, 0, 0.0));
    emit("too-many", matches(fixture_too_many, 31, ANALYSIS_INVALID_ARGUMENT, 0, 0, 0, 0, 0.0));
    emit("null-input", matches(NULL, 1, ANALYSIS_INVALID_ARGUMENT, 0, 0, 0, 0, 0.0));
    emit("null-output", analyze_readings(fixture_sample, 6, NULL) == ANALYSIS_INVALID_ARGUMENT);
    return failures == 0 ? 0 : 1;
}
