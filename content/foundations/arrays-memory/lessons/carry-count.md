# Carry the count

At a call site, an array expression usually converts to a pointer to its first element. Separately, C adjusts a function parameter declared as an array to a pointer parameter. The function receives a pointer; the original array length is not carried along in that parameter.

That is why a reusable function that traverses an array needs a count too. `sum_values(values, count)` has two pieces of information: where the first value is and how many values the caller permits the function to read. The caller is responsible for passing a pointer and count that describe a valid range. A count does not make an invalid pointer safe.

The caller can compute an element count while `values` is still an actual array: `sizeof values / sizeof values[0]`. Do this before passing it to the function. Inside a parameter declared with `[]`, `sizeof values` measures the adjusted pointer, not the caller's array.

## Worked example

Build the three files in `examples/carry-count/` together. The caller computes the six-element count and passes it to `sum_values` in a separate source file. The function loops while `i < count` and returns 99. Its precondition is that `values` points to at least `count` readable integers and the sum fits in `int`.

## Common mix-up

**“`sizeof values / sizeof values[0]` always finds the array length.”** It works at the caller while `values` is an array. Inside the adjusted function parameter, `values` is a pointer and no array length can be recovered from `sizeof` it. Compute the count at the owner and pass it explicitly.

## Checkpoint

Inside `sum_values(const int values[], size_t count)`, what does `count` tell the function that the adjusted `values` parameter does not?

**Answer:** The number of elements the caller says are in the readable range. If you answered the number of bytes, remember that `count` is an element count; pointer arithmetic applies the element type's size.

## Sources

WG14 C17 draft N2176 §6.7.6.3 paragraph 7 specifies array-parameter adjustment. §6.3.2.1 paragraph 3 describes array-to-pointer conversion and its exceptions. These rules explain why the count must be passed separately.
