#include "sensor_report.h"

int write_sensor_report(
    FILE *stream,
    enum analysis_status status,
    const struct sensor_stats *stats)
{
    if (stream == NULL || stats == NULL) {
        return -1;
    }

    switch (status) {
    case ANALYSIS_OK:
        return fprintf(stream,
                       "{\"status\":\"ok\",\"count\":%zu,\"sum\":%d,\"minimum\":%d,\"maximum\":%d,\"mean\":%.17g}\n",
                       stats->count, stats->sum, stats->minimum, stats->maximum, stats->mean) < 0
            ? -1 : 0;
    case ANALYSIS_EMPTY:
        return fprintf(stream,
                       "{\"status\":\"empty\",\"count\":0,\"sum\":0,\"minimum\":null,\"maximum\":null,\"mean\":null}\n") < 0
            ? -1 : 0;
    case ANALYSIS_INVALID_ARGUMENT:
        return fprintf(stream,
                       "{\"status\":\"invalid_argument\",\"count\":0,\"sum\":0,\"minimum\":null,\"maximum\":null,\"mean\":null}\n") < 0
            ? -1 : 0;
    case ANALYSIS_READING_OUT_OF_RANGE:
        return fprintf(stream,
                       "{\"status\":\"reading_out_of_range\",\"count\":0,\"sum\":0,\"minimum\":null,\"maximum\":null,\"mean\":null}\n") < 0
            ? -1 : 0;
    }
    return -1;
}
