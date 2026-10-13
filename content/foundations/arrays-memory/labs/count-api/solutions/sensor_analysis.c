#include <stddef.h>

#include "sensor_analysis.h"

enum analysis_status analyze_readings(
    const int *readings,
    size_t count,
    struct sensor_stats *out_stats)
{
    if (out_stats == NULL) {
        return ANALYSIS_INVALID_ARGUMENT;
    }
    *out_stats = (struct sensor_stats){0, 0, 0, 0, 0.0};

    if (count > SENSOR_CAPACITY || (count > 0 && readings == NULL)) {
        return ANALYSIS_INVALID_ARGUMENT;
    }
    if (count == 0) {
        return ANALYSIS_EMPTY;
    }

    for (size_t i = 0; i < count; ++i) {
        if (readings[i] < SENSOR_MIN_READING || readings[i] > SENSOR_MAX_READING) {
            return ANALYSIS_READING_OUT_OF_RANGE;
        }
    }

    struct sensor_stats result = {count, 0, readings[0], readings[0], 0.0};
    for (size_t i = 0; i < count; ++i) {
        const int value = readings[i];
        result.sum += value;
        if (value < result.minimum) {
            result.minimum = value;
        }
        if (value > result.maximum) {
            result.maximum = value;
        }
    }
    result.mean = (double)result.sum / (double)count;
    *out_stats = result;
    return ANALYSIS_OK;
}
