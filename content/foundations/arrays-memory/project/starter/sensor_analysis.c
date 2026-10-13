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
    (void)readings;
    if (count == 0) {
        return ANALYSIS_EMPTY;
    }
    return ANALYSIS_OK;
}
