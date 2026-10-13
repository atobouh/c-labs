# Sensor analyzer contract (F02)

This file is the binding interface for the chapter project and the `lab-count-api` precursor. IDs and behavior are fixed before lesson and grader authoring. The project stays within C17 and uses original course material; citations support language facts, not the product choices below.

## Prerequisites and scope

The pilot follows `foundations-functions-lifetime`. Learners are expected to compile and run a C17 program, use integer arithmetic, conditions and bounded loops, and define/call functions with parameters and return values. Earlier course chapters provide those skills in sequence. This chapter does not require dynamic allocation, strings, recursion, pointer casts, parsing, or external libraries.

Input arrives as a pointer plus an explicit `size_t` count. The supplied grader adapter owns `main()` and injects each test vector directly; learners do not parse stdin or command-line text. The analyzer and report functions are learner code, built as separate translation units against their supplied headers. This keeps the project about a reusable C API and its behavior rather than an untaught parser.

## Public API

```c
/* sensor_analysis.h */
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
```

The project also supplies `sensor_report.h`, which includes `sensor_analysis.h`:

```c
/* sensor_report.h */
#ifndef SENSOR_REPORT_H
#define SENSOR_REPORT_H

#include <stdio.h>
#include "sensor_analysis.h"

int write_sensor_report(
    FILE *stream,
    enum analysis_status status,
    const struct sensor_stats *stats);

#endif
```

### `analyze_readings`

- `out_stats` must point to a writable `struct sensor_stats`. If it is null, return `ANALYSIS_INVALID_ARGUMENT`; no output can be written.
- Counts above 30 are invalid. A null `readings` pointer is allowed only when `count == 0`.
- With a valid output pointer, every non-`ANALYSIS_OK` result leaves all fields zero. Empty input returns `ANALYSIS_EMPTY`, with `count`, `sum`, `minimum`, `maximum`, and `mean` all zero. The status carries the no-data meaning; zero extrema are not reported as measured values.
- For 1–30 readings, accept only values from 0 through 1000 inclusive. If any value is outside this range, return `ANALYSIS_READING_OUT_OF_RANGE` and leave all fields zero.
- On valid nonempty input, return `ANALYSIS_OK`; set the actual count, sum every reading once, and set the measured minimum, maximum, and `mean = (double)sum / (double)count`.
- A valid nonempty input is never partially committed: validate all readings before publishing statistics. The caller owns and keeps the input alive for the call.

The largest valid sum is 30,000. C17 requires `INT_MAX` to be at least 32,767, so this contract's sum fits in `int` on every conforming implementation. Array elements are contiguous, and array indexing is defined through pointer addition and indirection; this contract still limits access to the first `count` values. See WG14 N2176 §§6.2.5p20, 6.5.2.1p2, and 6.5.6pp8–9. N2176 §5.2.4.2.1p1 gives the minimum range for `int`.

### `write_sensor_report`

Write exactly one JSON object followed by `\n`. The object has exactly these keys, in this order: `status`, `count`, `sum`, `minimum`, `maximum`, `mean`. JSON whitespace is insignificant to checks. The status strings are `ok`, `empty`, `invalid_argument`, and `reading_out_of_range`.

- For `ANALYSIS_OK`, emit numeric `count`, `sum`, `minimum`, `maximum`, and `mean` values. Emit the mean with `%.17g`; the initial C locale is required and the grader compares parsed means using an absolute tolerance of `1e-6`.
- For `ANALYSIS_EMPTY`, emit `count: 0`, `sum: 0`, and `null` for `minimum`, `maximum`, and `mean`.
- For either invalid status, emit `count: 0`, `sum: 0`, and `null` for the three measurement fields, regardless of the supplied stats contents.
- `stream` and `stats` must be non-null, and `status` must be one of the four declared values. On a bad argument or failed write, return `-1`; otherwise return `0`. Invalid input status is a reportable result, not a report-writer error.

Examples (formatting shown compactly):

```json
{"status":"ok","count":6,"sum":99,"minimum":9,"maximum":24,"mean":16.5}
{"status":"empty","count":0,"sum":0,"minimum":null,"maximum":null,"mean":null}
```

The grader's supplied adapter calls the API with several independent arrays, checks returned fields and statuses, then checks the parsed report. It does not trust a submission-selected sample, a printed answer in place of the API, or report values for invalid input.

## Lab scaffolding

- `lab-trace-sum` introduces `int sum_readings(const int *readings, size_t count, int *out_sum)`. Valid inputs have at most 30 values, each from 0 through 1000; `(NULL, 0)` is valid. Return `0` on valid input and write the sum. Return `-1` for an invalid count, null output pointer, null input with nonzero count, or out-of-range value; if `out_sum` is valid, leave it zero on failure. The supplied harness checks empty, one-value, sample, zero-valued, and invalid-input cases.
- `lab-repair-boundary` uses the same signature. Its boundary fixture provides an accessible sentinel at index `count`; the contract excludes that value. This catches `i <= count` deterministically without expecting undefined behavior or a crash. Sanitizer output is supplemental only.
- `lab-count-api` implements the same complete `analyze_readings` contract from the public header in one source file and calls it from a separate harness translation unit. Its focused exercises emphasize explicit count, empty status, and zeroed output; project acceptance adds the full input matrix and `write_sensor_report` integration.

## Shared compiler and runner profile

Use a user-selected GCC or Clang C17 compiler and record its exact identity/version with each run; the course does not claim one local compiler is universally installed. Compile arguments are the array `[-std=c17, -Wall, -Wextra, -Wpedantic]`; never pass them through a shell string. On MinGW-w64, add `-D__USE_MINGW_ANSI_STDIO=1` so the verified profile accepts standard C99/C17 `printf` length modifiers such as `%zu` and `%td`. Default compile timeout is 10,000 ms, run timeout is 2,000 ms, and captured stdout plus stderr are capped at 65,536 bytes. A missing sanitizer is reported as unavailable, never as a pass. These bounds limit the teaching harness; they do not constitute a security sandbox.

## Source notes

- WG14, [C17 draft N2176](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n2176.pdf), cited by exact clause above. The official PDF's text extraction was unavailable during F01; the locators were cross-checked against a disclosed secondary copy, while the official WG14 document remains the citation. No standard prose is reproduced here.
- The course compares different teaching structures reviewed in F01 (CS50x's staged problem sets, MIT OCW's daily assignments, Beej's focused explanations, and Modern C's reference-driven progression). Those sources inform the original lesson→diagram practice→bounded lab→integrated project sequence; their text, code, and exercises are not copied.
