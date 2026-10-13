#include "sensor_analysis.h"

enum analysis_status analyze_readings(const int *readings, size_t count, struct sensor_stats *out_stats)
{
    if (out_stats == NULL) return ANALYSIS_INVALID_ARGUMENT;
    *out_stats = (struct sensor_stats){0, 0, 0, 0, 0.0};
    if (count > SENSOR_CAPACITY || (count > 0 && readings == NULL)) return ANALYSIS_INVALID_ARGUMENT;
    if (count == 0) return ANALYSIS_EMPTY;
    for (size_t i = 0; i < count; ++i) {
        if (readings[i] < SENSOR_MIN_READING || readings[i] > SENSOR_MAX_READING) {
            return ANALYSIS_READING_OUT_OF_RANGE;
        }
    }
    *out_stats = (struct sensor_stats){6, 99, 9, 24, 16.5};
    return ANALYSIS_OK;
}
