# Content and implementation contracts

## Authoring format and record identity

Use `RECORD-FORMATS.md` for the shared record shapes. Content tasks and importer work must use the same versioned fields.

Author text in UTF-8 Markdown, executable material in C and fixture files, metadata in JSON. The authoring files are the source of truth; SQLite is a generated application catalog. Do not make the HTML prototype's JS literals the production content store.

`foundation/curriculum.json` lists course/chapter identities and project titles. `foundation/pilot.json` is the pilot's delivery manifest. IDs are stable lowercase kebab-case; renaming titles must not change IDs. Content revision is a positive integer, separate from schema version. Source records live in `foundation/sources.json`. Relative artifact paths resolve from `c-lab/` and must remain inside it. Avoid absolute machine-specific paths.

Each pilot item has: `id`, `kind`, `title`, `objective_ids`, `artifact`, `status`, and `checks`. Kinds: lesson, exercise, animation, lab, project, lexicon. Statuses: planned, draft, verified. `checks` names required evidence, not an assertion that checks ran. Evidence lives in `foundation/evidence.json`, tied to artifact SHA-256; verified records require reviewed evidence. Do not mark a whole chapter verified when individual deliverables are missing.

## Sources and claims

Each source records its ID, title, author/organization, official URL, version/edition, actual access date, license evidence URL, redistribution decision, review status, and notes. Allowed decisions: link-only, original-reference, bundle-with-notice. `original-reference` means our own authored explanation based on verified facts; it does not grant permission to bundle the source.

Every lesson/lexicon claim about C semantics includes a citation with source ID, exact clause/page/heading, and the claim it supports. Cite official compiler documentation for tool behavior. A source lead is not a verified source. Do not invent section numbers, access dates, or license grants. An inaccessible source is an unresolved claim, not permission to guess.

No copied course prose, screenshots, videos, or problem sets without verified redistribution rights. Retain notices for permitted third-party code/assets. Source pages can be used during development; learning after installation must not depend on opening them. External further-reading links are optional and clearly labeled.

## Lessons and lexicon

Every lesson has prerequisites, objective IDs, an original explanation, one worked example, one misconception with correction, a checkpoint with feedback, and precise citations. Short focused sections are preferable to padding. Explain snippets as snippets; mark complete programs separately. Glossary entries need definition, concrete context, misconception, related IDs, and citations. Avoid circular definitions or presenting implementation details as language guarantees.

## Exercises and animation

Exercises declare the model, initial state, question, accepted answer, incorrect-answer explanations, progressive hints, and review tags. Answers must be evaluated against the model, not whichever cell was last clicked in a lesson.

Animation assets contain named states, source-line association, object/index/value labels, transitions, captions, and supported actions: step forward, step back, pause, reset. A user can inspect every state without autoplay. Reduced motion changes transitions, not information or controls. Use SVG/HTML for selectable labels and lightweight rendering; avoid video for code traces.

Label memory layouts as conceptual unless measured from a declared toolchain. A conceptual cell address must not pretend to be an actual process address. Never animate an out-of-bounds read as a guaranteed value. Pause at the violated precondition and explain why normal evaluation stops.

## Executable labs

Each lab directory contains `brief.md`, starter files, a reference solution, tests/fixtures, and `lab.json`. The metadata declares objective IDs, C mode, supported toolchain, structured compiler arguments, input fixtures, expected exit/output behavior, timeout/output bounds, solution-reveal policy, and citations.

Use portable C17. Compile with `-std=c17 -Wall -Wextra -Wpedantic` on supported GCC/Clang profiles. Check warnings; do not silently ignore them. Do not hardcode diagnostic wording across compiler versions. An intentionally failing specimen declares whether failure is compile-time, runtime, or a sanitizer finding, with matching evidence; undefined behavior has no prescribed stdout.

Run every complete starter and reference solution, plus success/boundary fixtures. For an intentionally incomplete starter, prove it compiles and produces the documented failing checks. Test plausible wrong solutions to demonstrate that grading rejects them. Reference solutions cannot be imported into the learner submission. Assertions are suitable for a dev harness, but must remain enabled; production grading must not disappear under NDEBUG.

## Project and progression

The project has a real brief, API/input constraints, starter, independent reference implementation, automated rubric, boundary fixtures, and an explanation checklist. Objective coverage must be explicit. Separate machine-verifiable checks from human self-review; do not claim automated semantic understanding.

Only a real passing run unlocks the next chapter. Associate the pass with submission hash, content revision, toolchain profile, and rubric version. Failure, cancellation, stale evidence, or a preview cannot unlock. Keep previous attempts; later edits do not rewrite their results. Course outlines remain browsable. A revised rubric must not silently erase existing learner work or pretend old submissions passed new tests.

## Runtime boundary

Compilation and execution are native Rust responsibilities behind structured commands. Never interpolate source, file paths, or flags into shell command strings. Use a dedicated job directory, controlled compiler selection, bounded capture, deadlines, cancellation, and process-tree cleanup. Compiler error, failed test, timeout, cancellation, toolchain unavailable, and internal error are distinct results.

Tauri is not a sandbox. Time/output limits do not prevent arbitrary local code from reading files or networking. Do not advertise sandboxing without an implemented, tested OS boundary. For the pilot, clearly describe trusted local code execution; do not install system services or silently download a compiler. Prefer a bundled, licensed toolchain for a self-contained release; a documented detected-toolchain mode is acceptable for development, with unsupported environments shown explicitly.

## Offline storage and UI

Keep versioned content separate from learner SQLite data: drafts, attempts, progress, review cards, local library configuration. Apply migrations transactionally; preserve existing work and report failures. No runtime network requests or remote assets. Content citations must remain readable offline even when their optional links cannot open.

Maintain the approved shell. Replace preview controls with real states when functionality exists; retain explicit preview labeling elsewhere. Loading, missing compiler, failed checks, empty search, long output, cancelled run, and restored draft must have readable states. Preserve keyboard focus, code selection, scroll position, and accessible labels. Fix the Open Lab scrollbar/focus treatment without removing visible keyboard focus.

## Evidence format

`foundation/evidence.json` holds `{ "schema_version": 1, "records": [...] }`. Each record contains `item_id`, `artifact_sha256`, `reviewed`, and nonempty `results`. Each result contains `check`, `command_or_method`, `observed`, and `passed`. Lab/project records also contain `package_files`: a map of every package file's project-relative path to SHA-256, including the manifest, starters, solutions, tests, and fixtures. Keep generated binaries and verification logs outside package directories. Any package change invalidates evidence and requires rerunning checks.

A failed result prevents verification. Reviewer evidence can record a manual source/visual review; name the actual method and findings. The structural checker validates presence and freshness only. Factual review and executed tests are still required. A model can fabricate an evidence record; its presence is never a substitute for reviewing the commands and results.
