# Arrays & Memory pilot specification

## Prerequisites and limits

This is C foundations chapter 4. Learners must already compile a C program, use integers, loops, conditions, functions, parameters, and return values. Arrays & Memory is not an introductory lesson about all of C. No malloc, strings, recursion, pointer casts, or external libraries are required for this chapter. Use C17; explicitly establish valid element ranges and pointer-plus-count contracts.

## Objectives

| ID | Learner demonstrates | Required practice | Project evidence |
| --- | --- | --- | --- |
| obj-array | Interpret array elements and conceptual adjacent storage | Lesson 1; exercise 1 | Known input array yields correct statistics |
| obj-index | Trace indexing and offsets without confusing bytes/elements | Lesson 2; exercise 1; animation 1; lab 1 | Every reading contributes once |
| obj-bounds | Recognize valid elements and one-past limits | Lesson 3; exercises 2/3; animation 2; lab 2 | Empty/single/full-capacity cases, no traversal beyond count |
| obj-count | Pass and honor explicit pointer-plus-count APIs | Lesson 4; lab 3 | Correct reusable analyzer API and empty-input policy |

The machine-readable manifest must agree with this table. Coverage alone does not prove quality.

## Lessons and exercises

1. **Objects side by side:** six readings [12,18,9,21,15,24]; element vs value, conceptual layout, valid initial objects.
2. **An index is an offset:** follow a bounded loop and distinguish index, element offset, and accumulated result. Do not assume an int byte size.
3. **The valid range:** indices 0 through count-1; a permitted one-past pointer is not a readable element. Explain the count-zero case before subtracting one.
4. **Carry the count:** array parameter adjustment and what size information a callee lacks; explicit count and valid-input preconditions. Cite the exact standard clauses after verifying them.

Exercise 1: select the object/value addressed by samples+3. Exercise 2: identify the last valid element for six elements. Exercise 3: classify forming vs dereferencing the one-past pointer. Each gets misconception-specific feedback and progressive hints. Wrong answers may enter review; hints do not fake a successful answer.

## Two meaningful animations

**Bounded traversal:** state zero, each iteration, then completion; show i, condition, selected object, total, and source line. The sample sum is 99. Each step can be reversed. Reset does not erase exercise attempts.

**Boundary check:** show last element, one-past, and a forbidden dereference proposal. The display stops at the invalid operation; it never fabricates the read's result. Provide the same information with motion disabled and with keyboard controls.

## Three distinct labs

1. **Trace and sum:** predict the result, implement a bounded sum, inspect real output and tests. Fixtures: sample, empty with NULL/count 0 policy, single value, zeros.
2. **Repair a boundary:** diagnose and repair an off-by-one traversal. Use deterministic harness checks and, where supported, an explicit sanitizer profile. An unsafe specimen's output is not an expected-output fixture. A passing solution cannot merely print the answer.
3. **Carry a count:** implement reusable statistics across two translation units. Honor the interface for empty and nonempty input, expose a clear status, and show that sizeof a parameter is not a count. This builds the API used by the project, not a third view of the same starter.

## Chapter project: sensor-data analyzer

Accept 0 to 30 integer readings, each 0 to 1000, through a provided input adapter. Students implement the reusable analysis core and compose the report. The adapter introduces no untaught parsing requirement; mark it supplied, test it independently, and defer robust parsing instruction to the strings chapter.

Report count, sum, minimum, maximum, and mean. Empty input has a documented no-data status, never divides by zero, and reports no fabricated extrema. Choose and document an exact API and machine-readable output fixture during F02. Preserve the bounded sum (maximum 30000); define mean comparison tolerance or exact formatting. Error cases must be distinguishable from valid empty input.

Checks: sample, empty, single, zeros, repeated values, capacity 30, maximum-value sum, invalid count/range at the input boundary. Include multiple non-sample inputs to reject hardcoded output. Test wrong implementations: skip first/last, traverse one too far, wrong denominator, assume six elements, bad empty handling. Check each failure reproducibly; do not rely on an out-of-bounds read crashing.

The rubric states which properties automated tests establish and which require code inspection. A learner explanation asks why count is explicit and why one-past is not readable. Completion requires the real configured automated gate; the explanation is a clearly labeled self-review until human review exists. Do not claim tests prove every possible memory property.

## Completion evidence

All 14 manifest artifacts exist and have reviewed evidence: 4 lessons, 3 exercises, 2 animations, 3 labs, 1 project, 1 linked lexicon collection. No copied proprietary content, invented citations, missing compiled fixtures, simulated passing runs, or network dependencies. Run the structural release check and the task-specific C, runtime, offline, interaction, and visual checks.
