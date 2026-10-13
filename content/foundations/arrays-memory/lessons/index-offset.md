# An index is an offset

An index counts elements from the first element. It does not count bytes. The pointed-to type determines how pointer arithmetic advances: adding 3 to a pointer to `int` selects the element three `int` elements later, when that position is in the same array or its one-past position. No lesson or diagram here assumes a byte size for `int`.

For an array named `samples`, C defines `samples[i]` in terms of pointer addition and indirection. That relationship connects the familiar bracket notation to a pointer offset, while the index remains an element position.

## Worked trace

Run `examples/index-offset.c`. It traverses `[12, 18, 9, 21, 15, 24]`, adding each selected value exactly once.

| index `i` | selected value | total after this step |
|---:|---:|---:|
| 0 | 12 | 12 |
| 1 | 18 | 30 |
| 2 | 9 | 39 |
| 3 | 21 | 60 |
| 4 | 15 | 75 |
| 5 | 24 | 99 |

At each step, `i` chooses an element and `total` remembers the accumulated result. They answer different questions. The loop ends when `i` reaches the count, before it tries to select that position.

## Common mix-up

**“`samples + 3` advances three bytes.”** It advances three elements of the pointed-to type. The corresponding byte distance depends on the implementation's `sizeof(int)`; code should use the language's typed pointer operations rather than guessing that distance.

## Checkpoint

After the loop has processed indices 0, 1, and 2, what are `i` and `total` just before the next iteration?

**Answer:** The next `i` is 3 and `total` is 39. If you answered 4, you advanced past an element before accounting for it.

## Sources

WG14 C17 draft N2176 §6.5.2.1 paragraph 2 gives the subscript relationship; §6.5.6 paragraphs 8–9 define pointer addition and its array limits. These clauses support the element-offset model, not a fixed byte layout.
