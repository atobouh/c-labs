# Original course collection and lexicon

The shell proposes three courses, each with eight chapters and a substantial chapter project: **C foundations**, **Systems in C**, and **Reliable C**. These are authored outlines, not a populated production course database.

## Build the corpus before the full course UI

1. Inventory primary references and high-quality learning sources. Record author, URL, version, access date, license, redistribution terms, and the concepts covered.
2. Map prerequisites and learning outcomes. Avoid gaps between a chapter's lessons and the techniques required by its project.
3. Author original explanations and lexicon entries from verified facts. Cite primary language/tool references. Link to books whose licenses prohibit adaptation or offline redistribution of their prose.
4. Design custom diagrams and exercises around a specific misconception: index versus byte offset, pointer versus value, lifetime versus returned value, or boundary versus element.
5. Add labs with original C source, fixtures, progressive hints, expected diagnostics, and a declared compiler/standard profile. Compile and run every shipped specimen and grader fixture in CI.
6. Write a project brief that combines the chapter's outcomes. Define constraints, fixtures, boundary cases, a review rubric, and explicit pass conditions. Passing the project unlocks the next chapter; outlines remain browsable.
7. Package reviewed content, permitted resources, and license notices into versioned offline packs. Keep learner progress separate so updating a course does not erase work.

## Records

| Record | Core fields |
| --- | --- |
| Course | Title, objectives, prerequisites, chapters, revision |
| Chapter | Order, learning outcomes, lessons, exercises, labs, project gate |
| Lesson | Original body, diagrams, concept tags, source citations |
| Diagram exercise | Model, prompt, correct state, explanation, misconception tags |
| Lab | Files, compiler mode, tests, time/output bounds, hints |
| Project | Brief, stages, constraints, fixtures, rubric, unlock rule |
| Lexicon entry | Term, definition, category, example context, related terms, citations |
| Source | Author, URL, version, license, redistribution permissions |

Use a local SQLite catalog with full-text search for courses and the lexicon. Store progress, submissions, review history, and open-lab projects locally. The default app must not require a network connection, account, analytics service, or CDN.

## Lightweight C workspace

Rust owns compiler discovery, bounded compilation/execution, results, and local persistence; Tauri hosts the bundled interface. An open lab has files, a declared C mode, local compiler options, a console, and optional include/linker configuration for installed external libraries. The interface must show real runner results when connected. Tauri does not itself sandbox arbitrary C programs.

## Pilot content shown in the shell

Arrays & memory has four sample lessons, three diagram exercises, three guided lab views, and a sensor-data analyzer project. Its project input is deliberately bounded to at most 30 readings in 0–1000, keeping the sum representable in int. The shell previews grading and unlocking; its expected-output specimen and boundary cases are separately compiled and checked.
