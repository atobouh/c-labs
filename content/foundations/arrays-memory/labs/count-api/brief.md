# Lab: Carry a count across files

Implement `analyze_readings` in one source file, then use it from the supplied test driver in another translation unit. The header is the contract: a pointer alone does not tell the analyzer how many values it may inspect, so every call supplies `count` explicitly.

Return `ANALYSIS_EMPTY` for count zero and leave every output field zero. Return `ANALYSIS_OK` only for valid nonempty readings. Invalid arguments and out-of-range readings have distinct statuses and also leave the output zero. For valid readings, compute count, sum, minimum, maximum, and mean. The project adds the separate JSON report function; this lab concentrates on the reusable analysis API.

## Suggested order

1. Read the public header before editing the implementation.
2. Initialize the output safely, then validate pointer and count.
3. Handle empty input before reading an element.
4. Validate all values, then calculate statistics in a bounded traversal.
5. Build the two translation units together and inspect each structured check.

## Hints

1. A valid output object should start in a known all-zero state.
2. Do not derive count from `sizeof` a function parameter.
3. Use the returned status to distinguish “no readings” from “bad input”; zero extrema alone cannot express that difference.

## Explain

Why does this API take both `readings` and `count`? Why does `ANALYSIS_EMPTY` avoid pretending that zero is a measured minimum or maximum? The automated harness checks the returned values; your explanation is a self-review, not an automated semantic grade.
