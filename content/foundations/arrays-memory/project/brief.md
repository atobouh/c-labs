# Chapter project: Sensor-data analyzer

Build a reusable C17 analysis module for a bounded set of sensor readings. You are given the public headers, a fixed-input adapter, fixtures, and a grader harness. Implement the analyzer and JSON report functions in separate source files. The adapter supplies `main()` and passes pointer/count pairs directly, so you do not need to parse text input or install a library.

Accept zero through 30 readings, each from 0 through 1000. Return the exact status required by `api.md`; on every non-success result, a valid output structure must remain all zero. For valid nonempty input, report count, sum, minimum, maximum, and arithmetic mean. Empty input is explicit: the API returns `ANALYSIS_EMPTY`, and JSON uses `null` for measurements that do not exist.

The tests include ordinary, repeated, zero, fractional-mean, maximum-capacity, and invalid inputs. They also pass a pointer to a buffer with an extra readable sentinel while declaring a smaller count. The sentinel is physically inside the C array but outside the logical input range; a correct analyzer ignores it. This checks the pointer/count contract without relying on an out-of-bounds crash. The exact-size null/zero case is checked separately against the reference adapter so known incorrect candidates are not run into undefined behavior.

## Suggested implementation order

1. Implement argument checks, empty status, and output initialization.
2. Validate the entire input range before publishing statistics.
3. Traverse only indices `0` through `count - 1` and compute each field.
4. Implement the JSON writer exactly as specified, including `null` fields and the final newline.
5. Build the two learner source files with the supplied adapter and inspect the structured grader checks.

## Self-review

After the automated gate passes, explain why the analyzer receives `count`, what the one-past position means, and how the status prevents an empty dataset from looking like measured zero extrema. The automated tests establish the listed fixtures and report protocol; they do not prove every possible memory property or grade this explanation.
