# Machine Room implementation handoff

## Objective

Build a fast, fully offline C-learning app with original, verified lessons, diagram exercises, meaningful animations, executable labs, and a substantial project per chapter. Rust owns persistence and process management; Tauri hosts bundled UI. The working name is Machine Room. No account, paid API, online model, CDN, or internet connection is required for learning.

The user approved the existing light native studio direction. `../design/studio.html` is the visual baseline, not a production app. Preserve the course-centered navigation and reading quality. Do not copy Octet's bookshelf or start another design exploration.

## What exists

- A local HTML/CSS/JS prototype and screenshots in `../screens/`.
- Three eight-chapter course outlines; only Arrays & Memory has detailed sample views.
- Separate verified C specimens; they do not make the prototype an executable IDE.
- Research leads and prior QA, not a complete vetted source library or course database.
- This handoff, a structured curriculum plan, pilot blueprint, and structural checker.

## Execution method for a smaller model

1. Take **one task ID** from `TASKS.md`; do not implement the entire roadmap in one turn.
2. Read `STATUS.md`, the task's inputs, and the relevant contract sections. Inspect current files before changing them.
3. State the concrete deliverables. Resolve ordinary implementation choices using these contracts. Ask only when a missing answer materially changes correctness or scope.
4. Implement and run that task's checks. Do not substitute optimistic summaries for evidence.
5. Update status with actual results, then hand over the next task. If blocked, name the missing dependency and preserve partial work as draft.

Do not keep rereading research, reinstall design skills, regenerate reference images, or rewrite approved layouts. Research only the specific claims or technical decisions needed for the current task. Use existing primary-source leads, and verify the actual source text before citing it.

## Delivery boundary

The first working milestone is **one complete Arrays & Memory chapter**, not all 24 chapters. This chapter assumes the first three chapters' skills. Record that prerequisite explicitly; pilot completion does not make the course beginner-complete. Expand from chapter 1 after the pilot validates the authoring and execution pipeline.

Complete means: four reviewed lessons; three diagram exercises; three real labs; two controllable animations; a real sensor-analyzer project with checks; linked lexicon; offline persistence; authentic grading and unlock behavior. See `PILOT.md` for exact coverage and `TASKS.md` for dependencies.

## Copyable assignment

> Work in c-lab. Follow its AGENTS.md. Read handoff/START-HERE.md and STATUS.md, then implement task F01 from TASKS.md only. Preserve the approved shell. Verify claims and licenses against actual primary pages, record evidence, run the specified checks, and update STATUS.md. Keep unverified work marked draft. Do not build later tasks or claim the course/app is complete.

Change only the task ID for subsequent assignments. A Luna handoff does not require another agent to be spawned now.
