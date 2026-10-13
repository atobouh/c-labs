#include <stddef.h>

#include "sensor_analysis.h"

int main(void)
{
    struct sensor_stats stats = {777, 777, 777, 777, 777.0};
    const enum analysis_status status = analyze_readings(NULL, 0, &stats);
    return status == ANALYSIS_EMPTY
        && stats.count == 0
        && stats.sum == 0
        && stats.minimum == 0
        && stats.maximum == 0
        && stats.mean == 0.0 ? 0 : 1;
}
