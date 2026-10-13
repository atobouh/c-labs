#include <stddef.h>
#include <stdio.h>

#include "sensor_analysis.h"
#include "sensor_report.h"
#include "../fixtures/fixtures.h"

static int failures;

static void emit(const char *check_id, int passed)
{
    fprintf(stderr, "{\"check_id\":\"%s\",\"passed\":%s}\n",
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
    double expected_mean,
    struct sensor_stats *out_stats,
    enum analysis_status *out_status)
{
    struct sensor_stats actual = {777, 777, 777, 777, 777.0};
    const enum analysis_status status = analyze_readings(readings, count, &actual);
    const int passed = status == expected_status
        && actual.count == expected_count
        && actual.sum == expected_sum
        && actual.minimum == expected_minimum
        && actual.maximum == expected_maximum
        && near_mean(actual.mean, expected_mean);
    *out_stats = actual;
    *out_status = status;
    return passed;
}

static void report_case(
    const char *analysis_check_id,
    const char *report_check_id,
    const char *report_id,
    const int *readings,
    size_t count,
    enum analysis_status expected_status,
    size_t expected_count,
    int expected_sum,
    int expected_minimum,
    int expected_maximum,
    double expected_mean)
{
    struct sensor_stats stats;
    enum analysis_status status;
    emit(analysis_check_id, matches(readings, count, expected_status, expected_count,
                                    expected_sum, expected_minimum, expected_maximum,
                                    expected_mean, &stats, &status));
    fprintf(stderr, "{\"report_id\":\"%s\"}\n", report_id);
    emit(report_check_id, write_sensor_report(stdout, status, &stats) == 0);
}

int main(void)
{
    report_case("analysis-sample", "report-write-sample", "stdout-sample",
                fixture_sample, 6, ANALYSIS_OK, 6, 99, 9, 24, 16.5);
    report_case("analysis-empty", "report-write-empty", "stdout-empty",
                fixture_empty_guarded, 0, ANALYSIS_EMPTY, 0, 0, 0, 0, 0.0);
    report_case("analysis-single", "report-write-single", "stdout-single",
                fixture_single, 1, ANALYSIS_OK, 1, 42, 42, 42, 42.0);
    report_case("analysis-zeros", "report-write-zeros", "stdout-zeros",
                fixture_zeros, 3, ANALYSIS_OK, 3, 0, 0, 0, 0.0);
    report_case("analysis-repeated", "report-write-repeated", "stdout-repeated",
                fixture_repeated, 4, ANALYSIS_OK, 4, 28, 7, 7, 7.0);
    report_case("analysis-fractional-mean", "report-write-fractional-mean", "stdout-fractional-mean",
                fixture_fractional_mean, 3, ANALYSIS_OK, 3, 13, 1, 10, 13.0 / 3.0);
    report_case("analysis-max-capacity", "report-write-max-capacity", "stdout-max-capacity",
                fixture_max_capacity, 30, ANALYSIS_OK, 30, 30000, 1000, 1000, 1000.0);
    report_case("analysis-count-sentinel", "report-write-count-sentinel", "stdout-count-sentinel",
                fixture_sentinel, 3, ANALYSIS_OK, 3, 12, 2, 6, 4.0);
    report_case("analysis-invalid-low", "report-write-invalid-low", "stdout-invalid-low",
                fixture_below_range, 2, ANALYSIS_READING_OUT_OF_RANGE, 0, 0, 0, 0, 0.0);
    report_case("analysis-invalid-high", "report-write-invalid-high", "stdout-invalid-high",
                fixture_above_range, 1, ANALYSIS_READING_OUT_OF_RANGE, 0, 0, 0, 0, 0.0);
    report_case("analysis-invalid-tail", "report-write-invalid-tail", "stdout-invalid-tail",
                fixture_invalid_tail, 3, ANALYSIS_READING_OUT_OF_RANGE, 0, 0, 0, 0, 0.0);
    report_case("analysis-too-many", "report-write-too-many", "stdout-too-many",
                fixture_too_many, 31, ANALYSIS_INVALID_ARGUMENT, 0, 0, 0, 0, 0.0);
    report_case("analysis-null-input", "report-write-null-input", "stdout-null-input",
                NULL, 1, ANALYSIS_INVALID_ARGUMENT, 0, 0, 0, 0, 0.0);

    emit("analysis-null-output",
         analyze_readings(fixture_sample, 6, NULL) == ANALYSIS_INVALID_ARGUMENT);
    const struct sensor_stats zero_stats = {0, 0, 0, 0, 0.0};
    emit("report-null-stream",
         write_sensor_report(NULL, ANALYSIS_OK, &zero_stats) == -1);
    emit("report-null-stats",
         write_sensor_report(stdout, ANALYSIS_OK, NULL) == -1);
    emit("report-unknown-status",
         write_sensor_report(stdout, (enum analysis_status)99, &zero_stats) == -1);
    return failures == 0 ? 0 : 1;
}
