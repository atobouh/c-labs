# Objects side by side

An array gives one name to a fixed sequence of objects with the same element type. In `int samples[6]`, the six `int` objects are `samples[0]` through `samples[5]`. Each element is a distinct object; the integer stored in it is that object's current value.

```text
conceptual sequence (not process addresses)
samples[0]  samples[1]  samples[2]  samples[3]  samples[4]  samples[5]
    12          18           9          21          15          24
```

The standard requires array elements to be contiguous. This picture names their order; it does not assign byte addresses or imply that an `int` has a particular size. When you write `samples[3] = 20`, you change the value held by the fourth element object. You do not create a new element.

## Worked example

Run `examples/objects-side-by-side.c`. It prints each index beside the value currently held there. Read one row at a time: index 3 selects the fourth object, whose starting value is 21. The source code is a complete C17 program.

The next lesson will follow how an index moves through this sequence. For now, keep the two labels separate: an **index** identifies an element; a **value** is stored in that element.

## Common mix-up

**“Index 3 means the third element.”** C array indexing starts at zero. Index 0 is the first element, so index 3 is the fourth. The last index is one less than the number of elements, provided the array is nonempty.

## Checkpoint

In the sequence above, what value is selected by `samples[4]`?

**Answer:** 15. Index 4 selects the fifth element. If you chose 24, you counted from one and landed on the sixth element.

## Source

WG14 C17 draft N2176 §6.2.5 paragraph 20 defines array objects as contiguous. This is a language guarantee, not a promise about a displayed numeric address. See the linked source in the course's offline source notes.
