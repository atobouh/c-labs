# Repository Guidelines

## Scope and current state

Machine Room is an offline C-learning desktop app planned for Rust and Tauri. The user approved the light studio design in `design/studio.html`; preserve its layout, local fonts, SVG icons, and restrained controls. Compiler execution, grading, SQLite, and the desktop host are not implemented. Preview results in `design/studio.js` are not actual learner execution.

The Git repository root is this `c-lab/` directory only. Never initialize or commit from its parent; sibling projects, especially Tandem, stay separate. Preserve the parent `AGENTS.md` and sibling projects. Do not overwrite an existing contributor guide. Do not discard other agents' edits.

## Starting a task

Read `handoff/START-HERE.md`, then the single assigned task in `handoff/TASKS.md`. Read only its listed context; use targeted searches rather than loading the whole repository. The approved design satisfies the parent validation workflow for this direction. Obtain validation for a materially new visual direction, not routine implementation of the approved one.

## Learning and content rules

Use portable C17 as the baseline; label C23 extensions. Follow `handoff/CONTENT-CONTRACT.md`. Every normative C claim needs a precise primary citation. Author original prose, diagrams, exercises, and projects; verify licenses before bundling external material. Do not invent citations or execution evidence. Model animations as reversible teaching states; never depict undefined behavior as deterministic.

Every lesson objective must map to practice and project evidence. Every shipped complete C example, starter, reference solution, and grader fixture must compile and be checked. Explain snippets separately; intentional failures need an explicit diagnostic contract. Project unlocking requires a real passing submission for that content revision.

## Working and verification

Use `py -3 tools/check_foundation.py` to validate the foundation. Use `--release` only when assessing delivery completeness. Its checks are structural, not proof of factual accuracy or runtime correctness. Run `node --check design/studio.js` after shell edits. Capture and inspect affected screens after visual changes, including focus, scrolling, narrow windows, and reduced motion.

Keep tasks small. Update `handoff/STATUS.md` with files, commands, observed results, unresolved issues, and the next task. Never mark work done from screenshots, expected output, or unexecuted tests alone. Do not install new tools, frameworks, skills, or dependencies without a concrete task need.
