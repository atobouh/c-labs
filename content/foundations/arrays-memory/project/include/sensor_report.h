#ifndef SENSOR_REPORT_H
#define SENSOR_REPORT_H

#include <stdio.h>

#include "sensor_analysis.h"

int write_sensor_report(
    FILE *stream,
    enum analysis_status status,
    const struct sensor_stats *stats);

#endif
