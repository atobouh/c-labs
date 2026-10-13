# Shell review — 13 October 2026

## Visual inspection

Captured and inspected the course collection, course progression, chapter reader, diagram exercise, guided lab, chapter project, project collection, lexicon, open lab, and review. Also captured correct-answer and project-pass states. See `screens/index.html`.

## Interaction checks

- Course → lesson → diagram exercises → guided labs → project → next-chapter unlock scenario passed.
- Lesson diagram inspection does not preselect an exercise answer.
- Wrong and correct answer states, the three-exercise sequence, and lab continuation passed.
- Lexicon filtering, empty search, related terms across categories, global term search, and Escape passed.
- Open-lab drafts survived reload; new drafts preserved a restorable previous draft.
- Library settings and C-mode selection behaved correctly during rendering. Edited drafts did not display a false execution result.
- A three-card review session updated the queue count correctly.
- No page script errors were observed in the interaction checks.

## Offline and layout checks

A fresh headless Edge context with network access disabled loaded the shell directly from disk, rendered the local font, and opened the chapter from the course screen. No CDN is needed.

Thirty layout cases covered ten views at widths 1024, 768, and 390 pixels. A narrow-window lab overflow was fixed by allowing the grid child to shrink; its targeted recheck measured a 390px document in a 390px viewport. All cases have no page-level horizontal overflow. The source editor retains its own horizontal scrolling.

## C verification

GCC compiled specimens with `-std=c17 -Wall -Wextra -Wpedantic`. `frame.c` printed `8`; `array_sum.c` printed `99`. The array check executable passed sample, empty, single-element, and zero-value inputs. JavaScript syntax checks passed.

## Scope

This is a design shell with sample course outlines and interactions. Compiler execution, actual project grading, the complete original course dataset, SQLite persistence, and the Tauri host are later implementation stages. Local draft and library configuration use browser storage in this artifact.
