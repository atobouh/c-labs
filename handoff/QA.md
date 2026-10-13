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

## Native editor polish verification - 2026-10-13

This addition covers the rebuilt R01 host's editor, plus the shared guided-lab browser preview. The original report above describes the earlier design stage.

| Before | After | Why |
| --- | --- | --- |
| White native scrollbar, arrow buttons, resize handle | Thin dark thumb, matching track, no arrows or resize handle | Keep browser-owned controls consistent with the workbench |
| Large editor focus rectangle | Caret and small filename focus marker; no outline/shadow | Show input focus without framing the entire editing surface |
| Small console type and light footer | 13px console text, 14px code, consistent dark footer | Improve readability and visual continuity |

The final native app was inspected through WebView2 CDP after rebuilding, without injected styles. Computed editor outline/shadow were `none`, resize was `none`, and scrollbar buttons were hidden. Temporary 80-line DOM content scrolled in both directions and was restored without triggering draft persistence. The native screenshot is `../screens/open-lab-native-polished.png`.

Final browser checks at 1024x768 and 390x844 found no page-level horizontal overflow. The narrow preview and guided-lab preview were visually inspected. JavaScript syntax, whitespace checks, and the Windows installer build passed. Full pilot/runtime QA remains R04; compiler execution and grading remain future tasks.
