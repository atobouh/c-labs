#ifndef TRACE_SUM_FIXTURES_H
#define TRACE_SUM_FIXTURES_H

static const int fixture_sample[] = {12, 18, 9, 21, 15, 24};
static const int fixture_single[] = {42};
static const int fixture_zeros[] = {0, 0, 0};
static const int fixture_below_range[] = {-1};
static const int fixture_above_range[] = {1001};
static const int fixture_too_many[31] = {0};
static const int fixture_max_capacity[30] = {
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000
};

#endif
