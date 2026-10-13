#ifndef REPAIR_BOUNDARY_FIXTURES_H
#define REPAIR_BOUNDARY_FIXTURES_H

static const int fixture_empty_guarded[] = {700};
static const int fixture_sample_guarded[] = {12, 18, 9, 21, 15, 24, 777};
static const int fixture_single_guarded[] = {42, 123};
static const int fixture_below_guarded[] = {-1, 0};
static const int fixture_above_guarded[] = {1001, 0};
static const int fixture_too_many_guarded[32] = {0};
static const int fixture_max_guarded[31] = {
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000,
    5
};

#endif
