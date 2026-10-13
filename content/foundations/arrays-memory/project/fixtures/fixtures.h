#ifndef SENSOR_PROJECT_FIXTURES_H
#define SENSOR_PROJECT_FIXTURES_H

static const int fixture_sample[] = {12, 18, 9, 21, 15, 24, 777};
static const int fixture_empty_guarded[] = {700};
static const int fixture_single[] = {42, 123};
static const int fixture_zeros[] = {0, 0, 0, 100};
static const int fixture_repeated[] = {7, 7, 7, 7, 1};
static const int fixture_fractional_mean[] = {1, 2, 10, 77};
static const int fixture_sentinel[] = {2, 4, 6, 777};
static const int fixture_below_range[] = {20, -1, 33};
static const int fixture_above_range[] = {1001, 0};
static const int fixture_invalid_tail[] = {1, 2, 1001, 4};
static const int fixture_too_many[32] = {0};
static const int fixture_max_capacity[31] = {
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    5
};

#endif
