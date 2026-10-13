# Lab: Trace and sum

Before you run the starter, predict the sum of the six readings `[12, 18, 9, 21, 15, 24]`. Record the index and running total after each iteration. Then implement `sum_readings` and compare your trace with the harness results.

The function receives a pointer, an explicit element count, and an output pointer. Valid inputs contain at most 30 readings, each in the range 0–1000. `(NULL, 0)` is a valid empty input and writes a sum of zero. Invalid arguments, counts, or readings return `-1`; a valid output pointer must contain zero on failure. Do not print or hardcode the sample result.

The supplied starter compiles but returns zero for every valid input. The harness supplies the `main` function and reports each check as JSON Lines, so the exercise stays focused on traversal rather than input parsing. Try a one-value and an all-zero case before attempting the maximum-capacity case. The reference solution is instructor-only.

## Hints

1. First validate the output pointer and initialize it.
2. Handle invalid count and null input before entering the loop.
3. Keep a local total; publish it only after every value has passed validation.

## Inspect

Use the `sample-sum` and `max-capacity` results to check your trace. Explain why the loop condition uses `i < count`, and what the explicit count lets a second translation unit know.
