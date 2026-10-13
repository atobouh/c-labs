#ifndef SUM_READINGS_H
#define SUM_READINGS_H

#include <stddef.h>

#define SENSOR_CAPACITY 30
#define SENSOR_MIN_READING 0
#define SENSOR_MAX_READING 1000

int sum_readings(const int *readings, size_t count, int *out_sum);

#endif
