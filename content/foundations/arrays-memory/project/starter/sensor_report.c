#include "sensor_report.h"

int write_sensor_report(
    FILE *stream,
    enum analysis_status status,
    const struct sensor_stats *stats)
{
    if (stream == NULL || stats == NULL) {
        return -1;
    }
    (void)status;
    return 0;
}
