# Pilot record formats (version 1)

These are the production authoring conventions for F02-F06 and the importer in R01. The foundation checker currently checks manifests/evidence only. R01 must add record validation and reject unknown versions, duplicate IDs, dangling references, invalid bounds, and paths outside their package. Add format changes here before using them in content or readers; never let two tasks invent incompatible records.

## Common conventions

Every JSON artifact has `schema_version: 1`, `id` matching pilot.json, `kind`, and `objective_ids`. References use stable IDs. Citations are objects: `{ "source_id": "wg14-c17", "locator": "verified exact clause here", "claim": "the specific statement supported" }`. That example is a placeholder, not a usable citation. Complete citations must refer to reviewed sources. Every record contains `citations`, even if empty for a purely navigational record. JSON keys are snake_case. Strings are plain text unless a field is explicitly Markdown.

Lessons use Markdown with adjacent metadata named `<lesson-name>.json`. Metadata contains common fields plus `title`, `body_path`, `prerequisite_ids`, `example_paths`, `checkpoint_ids`, and `citations`. The Markdown file is the pilot artifact; verification evidence must include review of its adjacent metadata/examples and rerun when these change. Lexicon is one JSON artifact containing `entries`; each entry has `id`, `term`, `definition`, `example`, `misconception`, `related_ids`, and `citations`.

If a manifest item has `supporting_artifacts`, each path is a project-relative companion that is part of the deliverable (for example, lesson metadata and complete C examples). Verified evidence records include an exact `supporting_artifact_sha256` map for that item. The checker rejects missing, extra, changed, or out-of-project companions; changing one invalidates its verification.

## Diagram exercises

Required fields: `prompt`, `model`, `answer`, `feedback`, `hints`, `review_tags`, plus common fields. Model contains explicit values and valid index range. Answers contain a `type` and `value`: index uses an integer; classification uses a declared enum or typed object. For the one-past exercise use an object with `can_form: true`, `can_dereference: false`, and `index: 6`; the grader compares typed values, not display text. Feedback contains `correct` and misconception-specific `incorrect` entries keyed by declared error category. `error_categories` are ordered; each declares a typed `when.all` list with `path`, `op` (`eq` or `neq`), and literal `value`. A final `when.otherwise: true` category catches remaining wrong answers. The first matching rule wins. Do not mark an arbitrary unrelated answer correct because its index matches.

## Animation models

Required fields: `model_type: "conceptual"`, `code`, `states`, `initial_state_id`, and common fields. Each state has `id`, `source_line` (1-based or null), `caption`, `variables`, `selected_indices`, and `evaluation_status` (ready, complete, invalid-operation). `states` is ordered; forward/back navigate this immutable list. Reset selects the initial state. Declare keyboard mappings for step forward, step back, pause, and reset; manual models set `auto_play: false` and define pause behavior. The terminal boundary state explains the rejected dereference and contains no fabricated value; a model-only invalid specimen is never executed. Validate every `source_line` against code length. Include a static/reduced-motion summary so motion is never the only way to inspect state. A state must be understandable without motion; captions cannot be blank.

## Lab manifests

`lab.json` contains common fields plus `brief_path`, `starter_files`, `learner_source_paths`, `solution_files`, `test_source_paths`, `fixture_paths`, `profile`, `checks`, and `hints`. File paths are package-relative and cannot escape it. The user-facing starter copies exclude solutions and instructor-only files. `profile` contains `standard: "c17"`, compiler family/version requirements, compiler argument array, `compile_timeout_ms`, `run_timeout_ms`, and `output_limit_bytes`; platform-specific `target_adjustments` remain structured argument arrays, never shell strings. Use the finite F02 bounds. `mutations` may name instructor-only incorrect candidates and the required checks that must reject each; these files never enter learner starter copies. `starter_must_fail` names expected failing checks for an intentionally incomplete starter.

Each check has stable `id`, `description`, `kind`, and applicable typed inputs/expected results. Kinds: compile, exit-code, stdout, assertion-harness, sanitizer. Checks are required by default; `required: false` marks an optional tool-specific check whose absence is recorded as unavailable, never as a pass. Optional sanitizer checks declare compiler arguments, supported profile, negative test source, intentional mutant, and diagnostic category. Stdout checks declare normalization explicitly; mean comparisons declare tolerance. Assertion/sanitizer checks declare supported profiles. A missing required check is unavailable, never a pass. Version-specific diagnostic checks match documented categories, not a single universal error string. Check IDs map to rubric evidence and UI feedback.

## Project manifests

`project.json` uses the executable lab fields plus `rubric_version` (positive integer), `rubric`, and `unlock_chapter_id`. Each rubric row contains `id`, `objective_ids`, `check_ids`, and `review_mode` (automated or self-review). Every automated row references existing required checks. Self-review rows are visible explanation prompts, not hidden AI grading. F02 specifies the exact API/output contract in project/api.md. Importer tests enforce both rubric coverage and unlock target identity.

## Native results

R02 defines the final serialized protocol before R03 consumes it. Required semantics: job ID, submission hash, content revision, rubric version, toolchain identity, overall status, per-check results, exit code when available, bounded stdout/stderr, truncation indicator, and elapsed time. Status distinguishes passed, failed, compile-error, timed-out, cancelled, toolchain-unavailable, and internal-error. Results from an older job cannot overwrite a newer submission's state. Only passed results meeting the current gate can unlock.
