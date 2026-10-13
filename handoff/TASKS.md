# Bounded task queue

Run one task per assignment. Dependencies are delivery gates, not suggestions. A task that changes a previously verified artifact must refresh its evidence. Preserve other contributors' edits. Tasks F01-F06 establish content independently of the desktop runtime; R01-R04 integrate it. Do not expand all 24 chapters before the pilot is usable.

## F01 - Verify source inventory

Inputs: `../research/course-and-product-research.md`, `../foundation/sources.json`, CONTENT-CONTRACT sources section.
Ownership: sources.json, `../foundation/source-review.md`, STATUS.md.
Open actual official references, record edition/access date, exact relevant sections and license evidence. Separate link-only material from resources permitted in a release. At least one reviewed primary C-semantics source and official compiler reference must support the pilot. Preserve unresolved leads as unreviewed; do not manufacture completeness. Checks: structural checker; manually verify every recorded license decision and section locator. Done: pilot facts have accessible verification sources and redistribution decisions are evidence-backed.

## F02 - Lock prerequisites, coverage, and project API

Requires F01. Inputs: PILOT.md, RECORD-FORMATS.md, curriculum.json, pilot.json.
Ownership: these manifest files, `../content/foundations/arrays-memory/project/api.md`, STATUS.md.
Resolve the project's exact API/status/output protocol, mean tolerance, supplied input adapter boundary, lab scaffolding, and rubric. Cite language contracts. Validate prerequisite coverage; do not introduce advanced concepts casually. Keep course outlines distinct from finished chapters. Checks: checker; table mapping every objective to lesson, practice, project; review empty and invalid input contracts. Done: F03-F06 can implement without inventing different interfaces.

## F03 - Author four lessons and linked lexicon

Requires F02. Inputs: contract lessons/sources sections, RECORD-FORMATS.md, pilot item paths.
Ownership: four lesson Markdown files, lexicon JSON, their evidence records, STATUS.md.
Write original concise explanations, worked examples, misconceptions, checkpoints, and verified citations. Provide lexicon entries for array, element, index, bounds, pointer, one-past, count, and array parameter adjustment. Compile complete examples; identify fragments and assumptions. Checks: exact fact/citation review; compile/run complete examples; structural checker. Done: objectives taught accurately without reliance on online reading. Mark verified only with evidence.

## F04 - Author exercise and animation models

Requires F03. Inputs: PILOT.md, RECORD-FORMATS.md, contract exercises/animation sections.
Ownership: three exercise JSON files, two animation JSON files, tests for their state models, evidence, STATUS.md.
Specify accepted answers, distractor feedback, hints, captions, deterministic states, reverse/reset behavior, and source-line mapping. Test sample sum 99, valid index bounds, and stopping before invalid evaluation. Include accessible static representations. This task writes models; UI controls are R03. Done: models contain enough data to render/grade without hardcoded alternative truths.

## F05 - Implement three lab packages

Requires F02; use F03 terminology. Inputs: RECORD-FORMATS.md, contract labs/runtime sections, project API.
Ownership: three lab directories and a repeatable local verification script, evidence, STATUS.md.
Implement distinct starters, solutions, fixtures, hints, and compiler profiles. Build/run each package and plausible incorrect solutions; show each incorrect solution is rejected. Document intentional failures and tool-specific checks. Do not call a timeout runner a sandbox. Done: repeatable checks demonstrate solution success and relevant mutation failures on a declared toolchain. Grading returns structured results, not screenshots.

## F06 - Implement chapter project package

Requires F04/F05. Ownership: project directory, rubric/fixtures/reference solution, coverage review, evidence, STATUS.md.
Implement the F02 API exactly and PILOT.md's boundary cases. Test supplied adapter separately. Prove multiple hardcoded/wrong solutions fail without assuming UB crashes. Keep solution inaccessible to normal grader input. Check all objective coverage. Done: run `py -3 tools/check_foundation.py --release` successfully plus C package verification. This proves content structure/evidence completeness, not desktop readiness.

## R01 - Create offline Rust/Tauri host and catalog

Requires F02. Inputs: existing shell, RECORD-FORMATS.md, course-database-plan.md, contract runtime/storage sections.
Ownership: new app host/frontend build files, catalog importer/migrations, build instructions, STATUS.md.
Preserve the approved UI. Select and pin minimal supported dependencies; verify current official Tauri configuration. Bundle font/assets and authoring content. Separate catalog from learner data, use stable IDs and migration transactions. Do not force a shell rewrite for its own sake. Checks: actual dev/build commands, network-disabled launch, content lookup/search, migration preserving existing drafts. State platform/toolchain prerequisites. Done: desktop app loads local course data; previews remain explicitly labeled.

## R02 - Implement native compiler/job controller

Requires R01/F05. Ownership: Rust runner, result protocol, job tests, toolchain status UI, STATUS.md.
Implement structured compile/run commands with output limits, cancellation/deadlines, and process-tree cleanup. Show compiler unavailable distinctly. Run learner files, never the reference solution in their place. Test compile error, normal output, failing test, nonterminating program, output flood, cancellation, paths with spaces, and subsequent job after failure. Set/document concrete bounds and supported OS behavior. Done: real outputs with reproducible evidence; no false sandbox claim.

## R03 - Connect lessons, diagrams, labs, and progression

Requires F06/R02. Ownership: content views, animation/exercise controllers, grading/progression persistence, integration tests, STATUS.md.
Replace prototype scenarios with actual records/results. Preserve existing routes and clear navigation. Implement reversible animations and reduced-motion/static views. Associate project attempts with submission/content/rubric versions. Test wrong answers, hints, real success/failure, stale results, relaunch, interrupted persistence, and next-chapter unlock only after real pass. Done: complete pilot works offline through every learning stage.

## R04 - Polish and verify the complete pilot

Requires R03. Ownership: focused visual fixes, affected screenshots, final QA report, STATUS.md.
Fix dark editor scrollbar/focus mismatch while retaining visible keyboard focus. Inspect all affected screens and loading/error/empty states at 1024/768/390 widths. Verify keyboard navigation, selection/scroll behavior, reduced motion, local draft restore, offline font/content/search, and honest toolchain status. Avoid broad redesign. Done: content release check, C/runtime/integration checks, desktop offline run, and actual visual inspection all recorded. Update README with real commands; distinguish completed pilot from unbuilt chapters.

## Expansion

After R04, author chapter 1 using the same contracts and checks; then chapters 2/3 so beginners can reach the pilot. Complete Foundations before starting Systems and Reliable C. Add OS/toolchain-specific advanced content only with labeled profiles and tested fixtures. Do not generate filler chapters to inflate lesson counts.
