#include <stddef.h>
#include <stdio.h>

#include "sensor_analysis.h"
#include "sensor_report.h"

int main(void)
{
    const int readings[] = {12, 18, 9, 21, 15, 24};
    const size_t count = sizeof readings / sizeof readings[0];
    struct sensor_stats stats = {0, 0, 0, 0, 0.0};
    const enum analysis_status status = analyze_readings(readings, count, &stats);
    if (status != ANALYSIS_OK) {
        return 1;
    }
    return write_sensor_report(stdout, status, &stats) == 0 ? 0 : 1;
}
