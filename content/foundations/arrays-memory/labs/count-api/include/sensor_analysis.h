#ifndef SENSOR_ANALYSIS_H
#define SENSOR_ANALYSIS_H

#include <stddef.h>

#define SENSOR_CAPACITY 30
#define SENSOR_MIN_READING 0
#define SENSOR_MAX_READING 1000

enum analysis_status {
    ANALYSIS_OK,
    ANALYSIS_EMPTY,
    ANALYSIS_INVALID_ARGUMENT,
    ANALYSIS_READING_OUT_OF_RANGE
};

struct sensor_stats {
    size_t count;
    int sum;
    int minimum;
    int maximum;
    double mean;
};

enum analysis_status analyze_readings(
    const int *readings,
    size_t count,
    struct sensor_stats *out_stats);

#endif
