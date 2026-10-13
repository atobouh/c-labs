# Lab: Repair a boundary

The starter has a loop that includes `count` as though it were a readable index. Diagnose the loop condition, repair it, and run the supplied fixtures. The API's logical range is the first `count` values, regardless of any spare physical capacity behind the pointer.

Every ordinary boundary fixture includes one extra in-range sentinel after the logical range. For example, the sample buffer stores `[12, 18, 9, 21, 15, 24, 777]` but passes count 6; the expected sum is 99. A broken `i <= count` loop reads the sentinel and produces a deterministic wrong result while staying inside the C array object. This avoids relying on undefined behavior or a crash to grade the repair.

The starter is still an intentionally incorrect general function: with an exact-size backing array, its extra read may be undefined behavior. Run it only through the padded harness. An optional AddressSanitizer profile checks a separate exact-size negative fixture when the installed toolchain supports it. If the sanitizer runtime is missing, the check is recorded as unavailable; the sentinel contract checks remain the required grading evidence.

## Steps

1. Mark the valid indices for the passed count.
2. Identify the first index the starter visits outside that logical range.
3. Change the condition, then run the empty-with-sentinel, one-element, sample, and capacity cases.
4. Explain why a safe test allocation does not make reading past the API count correct.

## Hints

1. For count 3, indices 0, 1, and 2 are included.
2. The one-past logical position is index `count`.
3. Continue while `i < count`; do not evaluate the element at `i == count`.
