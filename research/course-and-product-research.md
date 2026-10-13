# C Learning Tool: Research Notes

Research snapshot: 13 October 2026. This is a curated starting library, not a claim that any single course is “the best.” The tool should teach from its own tested lessons, then use external works as references and optional further study.

The current shell uses three original course outlines—C foundations, Systems in C, and Reliable C—with eight chapters and a project per chapter. See [the course database plan](../handoff/course-database-plan.md) for the source inventory, authoring, verification, lexicon, and offline-pack workflow. The first detailed design scenario is Arrays & memory, with lessons, diagram exercises, labs, and a project gate.

## Learning material worth building on

- **CS50x — C, Arrays, Memory, Data Structures.** A particularly useful progression for turning memory into a visible model: the current course has dedicated C, arrays, memory, and data-structure weeks, with problem sets. Borrow the sequencing and concrete memory explanations. Our course should remain standard C and avoid depending on CS50 helper libraries. [Week 1: C](https://cs50.harvard.edu/x/weeks/1/), [Week 4: Memory](https://cs50.harvard.edu/x/weeks/4/), [Week 5: Data Structures](https://cs50.harvard.edu/x/weeks/5/).
- **MIT OCW 6.S096 — Introduction to C and C++.** An intensive university course centered on practical work, assignments, and a small project; useful for the workshop cadence and project checkpoints. It is from 2013 and mixes C++ with C, so use it selectively rather than as the course spine. [Course](https://ocw.mit.edu/courses/6-s096-introduction-to-c-and-c-january-iap-2013/).
- **Beej’s Guide to C Programming.** A broad, approachable C tutorial with compiler-specific examples and a separate reference volume. The author explicitly calls the guide beta quality. The book is CC BY-NC-ND 3.0, so link to it; do not copy or adapt its prose into the offline bundle without permission. Its code samples have a separate public-domain grant. [Guide and license](https://beej.us/guide/bgc/).
- **Modern C by Jens Gustedt.** A substantial path through modern C, valuable for instructor research and an advanced C23 extension. The book is CC BY-NC-ND 4.0; its code samples are MIT licensed. Link to the book and only bundle code with its license notice. [Book and license](https://gustedt.gitlabpages.inria.fr/modern-c/).
- **Exercism’s exercise design.** Useful model for small concept exercises with explicit prerequisites, learner-facing instructions, progressive hints, and test data. We should create original exercises and tests, not mirror its exercise text. [Concept exercise design](https://github.com/exercism/docs/blob/main/building/tracks/concept-exercises.md).

## Primary references for correctness and inspection

- **WG14 / ISO C.** WG14 identifies C23 as ISO/IEC 9899:2024 and publishes drafts and defect reports. Use the standard and WG14 papers to check claims about C semantics. A standards document is a verification source, not a beginner lesson. [WG14 project status](https://open-std.org/jtc1/sc22/wg14/www/projects.html).
- **cppreference C.** Fast navigation for language and library topics across C89 through C23; useful as a secondary reference, not the final authority where wording matters. [C reference](https://en.cppreference.com/w/c).
- **GDB manual.** The real reference for inspecting frames, variables, registers, and machine state. [Frames](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Frames.html), [registers](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Registers.html).
- **Clang sanitizers.** AddressSanitizer detects memory errors such as out-of-bounds access and use-after-free; UndefinedBehaviorSanitizer detects selected undefined behaviors. Teach what each tool can and cannot prove. [ASan](https://clang.llvm.org/docs/AddressSanitizer.html), [UBSan](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html).
- **SEI CERT C.** A deep secure-coding reference for later lessons on integer conversion, bounds, strings, and allocation. It belongs after learners can reason about the language basics. [CERT C](https://wiki.sei.cmu.edu/confluence/display/c).

## Initial topic inventory

The default course is **portable C17**, with C23 differences marked as an optional extension. The standard mode is explicit in every lab. Each concept follows one loop: read a short original explanation → predict output or memory → commit the prediction → compile and run → inspect a trace → explain the mismatch → revisit later if needed.

1. Compile and link: source, preprocessing, object files, executable, diagnostics.
2. Values in bytes: objects, types, integer conversions, expressions, storage.
3. Control and functions: branches, loops, parameters, return values, scope.
4. Arrays and pointers: addresses, valid pointer arithmetic, array bounds.
5. Strings as byte sequences: terminators, bounds, input, encoding caveats.
6. The call stack: frames, locals, arguments, recursion, debugger stepping.
7. Dynamic memory: allocation, ownership conventions, lifetime, failure paths.
8. Data layout: structs, padding, alignment, object representation.
9. Data structures: linked structures, queues, invariants, cleanup.
10. Files and binary data: byte order, parsing, validation, error handling.
11. Machine boundary: compile to assembly, debug symbols, optimization, ABI vocabulary.
12. Capstone: build a bounded binary/hex inspector, tested with malformed inputs.

Lab types: **Trace** (predict one step), **Find the fault** (diagnose a failing specimen), **Repair** (make tests pass), **Build** (combine concepts under constraints), and **Compare** (inspect compiler output or sanitizer reports). Hints reveal one clue at a time; solutions remain available after an explicit reveal. Review cards come from errors and uncertain answers, then return on a spaced schedule.

## Desktop interface references

- **VS Code debugging UI** provides a familiar source-editor plus variables, call stack, breakpoints, and console arrangement. I inspected the [official active-debug-session screenshot](https://code.visualstudio.com/assets/docs/debugtest/debugging/debug-session.png). Borrow the stable placement and clear stepping controls, not its full IDE density. [Debugging](https://code.visualstudio.com/docs/editor/debugging) and [user interface](https://code.visualstudio.com/docs/getstarted/userinterface).
- **Zed debugger** demonstrates a focused, keyboard-friendly editor with debugger panels and a lightweight desktop feel. Keep inspection close to the code and make each pane independently useful. [Debugger](https://zed.dev/blog/debugger).
- **JetBrains CLion** separates frames, variables, watches, and console into deliberate debugger views. I inspected its [official memory-view screenshot](https://resources.jetbrains.com/help/img/idea/2026.2/cl_memview_stepping.png); it informed a dedicated memory panel that stays tied to the highlighted source line. [Debug tool window](https://www.jetbrains.com/help/clion/debug-tool-window.html).
- **Exercism** and **Brilliant** inform learning structure: small concept-based exercises, explicit progression, and frequent active checkpoints. Adapt the teaching patterns with original lessons and tasks. [Exercism concept exercises](https://github.com/exercism/docs/blob/main/building/tracks/concept-exercises.md), [Brilliant learning paths](https://brilliant.org/help/learning-paths/).
- **Things** provides an unrelated desktop reference for readable task grouping, restrained context labels, and progressive disclosure. Four official desktop images are captured in the local reference board. Use the principles in the course and review lists; do not imitate a bookshelf. [Official feature examples](https://culturedcode.com/things/features/).
- **Linear** explains separating location from view controls, improving hierarchy, and reducing distracting chrome. This informed the distinction between the course breadcrumb and the chapter's Learn, Exercises, Labs, and Project controls. [Design refresh rationale](https://linear.app/now/behind-the-latest-design-refresh).
- **Affinity** separates the document canvas, contextual controls, and supporting panels. This informed giving lesson diagrams and C workspaces the main area while keeping relevant context alongside them. [Interface reference](https://www.affinity.studio/help/workspace-interface/).

These references guided layout and interaction choices; their screenshots and branded assets are not included in the product. The HTML proposal uses original CSS and locally rendered components.

## Offline and implementation constraints

Use Tauri 2 with a Rust-owned course catalog and runner controller, and a compiled local web frontend. Tauri can embed local frontend assets and bundle versioned resources; Cargo offers `--offline`/`--frozen` for network-independent dependency resolution. No runtime account, analytics, CDN, or network request should be required. Store progress and review state locally. Keep toolchain setup separate from the lesson catalog: detect local Clang/GCC/GDB, show their versions, and offer a documented optional toolchain package rather than silently downloading one.

Run learner code in a constrained child process with a temporary workspace, execution timeout, output cap, and platform-specific resource controls. The UI must never imply that Tauri itself sandboxes C code. Every shipped sample, expected output, and test fixture must compile and run under the declared compiler/mode in CI; assertions about language behavior must cite the exact standard clause or a primary WG14 source. Keep educational content original unless the source license permits redistribution and adaptation, and retain attribution/license files for included code.

## Interface direction

Treat the app like a **bench instrument for tracing a program**, not a course marketplace. The learner sees the next small action, makes a prediction before execution, then checks it against the real run. The memory view should be tied to the currently selected source line and display only the current frame, live objects, addresses, and lifetime changes. A course map provides the long path; lab view stays calm and focused. Offline/toolchain status is visible but quiet. Fast keyboard-driven compile/run/step actions should feel immediate; no decorative background motion.

The current shell is `design/studio.html`, with local CSS, JavaScript, and a bundled font. It shows the course collection, chapter reader, diagram exercises, guided labs, project gate, lexicon, open lab, and review. Sample C programs are verified separately. Compiler/debugger integration, real grading, and the full course database are not implemented at this design stage.
