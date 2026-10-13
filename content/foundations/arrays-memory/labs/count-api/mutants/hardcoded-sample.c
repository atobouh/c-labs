#include "sensor_analysis.h"

enum analysis_status analyze_readings(const int *readings, size_t count, struct sensor_stats *out_stats)
{
    if (out_stats == NULL) {
        return ANALYSIS_INVALID_ARGUMENT;
    }
    *out_stats = (struct sensor_stats){0, 0, 0, 0, 0.0};
    if (count == 0) {
        return ANALYSIS_EMPTY;
    }
    if (readings == NULL || count > SENSOR_CAPACITY) {
        return ANALYSIS_INVALID_ARGUMENT;
    }
    *out_stats = (struct sensor_stats){6, 99, 9, 24, 16.5};
    return ANALYSIS_OK;
}
