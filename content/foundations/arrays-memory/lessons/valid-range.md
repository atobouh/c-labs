# The valid range

For a nonempty array with `count` elements, valid indices run from 0 through `count - 1`. The loop condition `i < count` expresses that range directly. If `count` is zero, there is no last element; do not calculate `count - 1` before handling the empty case.

For an actual array of six elements, `samples + 6` may be formed as a one-past pointer. It can mark the end of that array, but it cannot be dereferenced. No readable seventh element exists. A program must stop before evaluating `samples[6]`; C does not define a predictable value for that operation.

An API's `count` may also describe an allowed prefix of a larger buffer. That logical boundary still controls the function: spare physical capacity does not authorize reading beyond the count supplied by the caller. The supplied lab uses an accessible sentinel to test this contract without depending on an out-of-bounds crash.

## Worked example

Run `examples/valid-range.c`. It prints the first and last values only after confirming the count is nonzero, then forms the end pointer and reports its offset without reading it. It also demonstrates an empty range. The end pointer is formed only for a nonempty prefix of the known three-element array.

## Common mix-up

**“The one-past pointer names the last element.”** It marks the boundary after the last element. For six elements, index 5 is readable; index 6 is not. Forming the one-past pointer is permitted in this case; dereferencing it is not.

## Checkpoint

There are six readings. Which index is last and readable, and what does `samples + 6` mean?

**Answer:** Index 5 is the last readable element; `samples + 6` is the one-past boundary. It has no value to read. If your answer was index 6, you confused the boundary with an element.

## Sources

WG14 C17 draft N2176 §6.5.6 paragraphs 8–9 permit forming a one-past pointer within the array bounds and forbid using it as an evaluated element. §6.5.2.1 paragraph 2 relates indexing to pointer indirection. Never illustrate an invalid read with an invented result.
