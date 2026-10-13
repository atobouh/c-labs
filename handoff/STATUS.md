# Handoff status

Updated: 2026-10-13.

## Baseline

- Design direction approved by user; Open Lab scrollbar/focus polish still pending.
- Approved shell and specimens remain the basis of the desktop app; prior prototype checks are recorded in `QA.md`.
- R01's Rust/Tauri host, offline catalog, search, lexicon, and learner draft storage are implemented. Compiler integration and actual grading remain unimplemented.
- Foundations F01-F06 are complete and the pilot content/evidence release gate passes. The package includes the locked API, four original lessons and compiled examples, lexicon, three diagram exercises, two animation models, three labs, and the sensor-data chapter project. The seven initial research leads were reviewed; R01 integrates the verified pilot offline. Compiler execution, interactive course views, and real grading remain future tasks.

## Next assignment

**R01 release verification in progress.** The local Windows installer and native database smoke test pass. The tagged GitHub workflow must still verify launch with outbound network blocked and publish the release before R01 is complete. R02 has not started.

## Foundation verification

- `py -3 tools/check_foundation.py`: passed draft structure/coverage validation.
- `py -3 tools/test_foundation.py`: 10 mutation tests passed, including fabricated verification and stale artifact/package evidence.
- `py -3 tools/check_foundation.py --release`: rejected the planned pilot as expected: missing artifacts, unverified sources, and absent execution/review evidence. This is an intentional completeness gate, not a failed implementation test.
- Root contributor guide and approved shell were preserved; no compiler host or actual course lessons were generated in this handoff task.

## Task results

### F01 — completed, sources reviewed (2026-10-13)

- Changed `foundation/sources.json`, added `foundation/source-review.md`, and updated this status file. Seven existing source records now include observed version/edition details, retrieval date, license/reuse evidence, and notes. Course/book prose and sample code remain link-only for this original pilot; GCC and LLVM documentation are references and are not reproduced.
- Checked the official WG14 document log and the C17 N2176 draft record; verified pilot-relevant clause locators (§6.2.5 ¶20, §6.5.2.1 ¶2, §6.3.2.1 ¶3, §6.5.6 ¶¶8–9, §6.7.6.3 ¶7). The official N2176 PDF opened but its permissions block local text extraction. To avoid overstating the check, this access limit and the secondary full-text copy used only to cross-check locators are disclosed in the review file. Learner citations still point to WG14.
- Reviewed GCC manual 17.0.0 warning controls and GFDL notice; Clang AddressSanitizer behaviors and limits; CS50x page/license (noting its 2026 page/2025 asset-path mismatch); MIT OCW 2013 course scope/current site terms; Beej’s stated prose/code terms; and Modern C’s C23 edition, text/code terms.
- From `c-lab/`: `py -3 tools/check_foundation.py` — passed draft structure and objective coverage; `py -3 tools/test_foundation.py` — all 10 tests passed. These are structural checks, not a course-content or license compliance certification.
- Unresolved: before any external release, check rights for each individual course/media asset; verify the learner’s means of opening the access-restricted WG14 PDF. No external source content is bundled.

### F02 - completed, API and coverage locked (2026-10-13)

- Changed `foundation/pilot.json`, `content/foundations/arrays-memory/project/api.md`, `tools/check_foundation.py`, `tools/test_foundation.py`, and this status file. The pilot prerequisite now matches the curriculum chain and names the learner skills expected; a machine-checked coverage map connects each objective to one lesson, practice, and the project.
- Locked `analyze_readings` statuses, 30-value/0-1000 boundaries, output-zeroing behavior, sum range, mean formula/tolerance, report JSON schema, and the grader-owned pointer/count adapter. The lab contracts include a bounded sum API and a padded sentinel fixture so an off-by-one submission can be rejected deterministically without relying on undefined behavior.
- Reviewed the project contract against the F01 source audit: N2176 sections 6.2.5 paragraph 20, 6.5.2.1 paragraph 2, 6.5.6 paragraphs 8-9, 6.7.6.3 paragraph 7, and 5.2.4.2.1 paragraph 1 support the array, pointer, parameter, and `int`-range claims. As disclosed in F01, the official N2176 PDF blocked local text extraction; clause text was cross-checked using the named secondary copy. The course comparison remains original and draws on the four differently structured references already reviewed in F01.
- From `c-lab/`: `py -3 tools/check_foundation.py` - passed draft structure and objective coverage; `py -3 tools/test_foundation.py` - all 12 mutation tests passed. These checks validate the planning graph and map; they do not compile future course code or establish teaching quality.
- Unresolved: F03 must author and compile complete lessons; F05 must select a functioning local C compiler and record its actual identity. No app compiler/runtime is claimed.
- Next eligible task: F03, four lessons and the linked lexicon.

### F03 - completed, lessons and lexicon verified (2026-10-13)

- Added four original lesson bodies and metadata, four complete C examples (the count example spans a header and two translation units), and the linked eight-term lexicon. Updated the pilot manifest with companion-file hashes, strengthened the foundation checker so changes to lesson metadata/examples stale their evidence, added a regression test for that case, and documented the evidence format. The four lessons and lexicon are marked verified; future deliverables remain planned.
- Reviewed all lesson and lexicon claims against the exact WG14 C17 locators already audited in F01. Added the F02 `INT_MAX` locator and cross-check disclosure to `foundation/source-review.md`. The official WG14 PDF extraction limitation remains explicit. Course design compares the reviewed CS50x, MIT OCW, Beej, and Modern C structures; all lesson prose, code, checkpoints, and terms are newly authored.
- `py -3 tools/check_foundation.py` - passed draft structure, objective coverage, and fresh evidence; `py -3 tools/test_foundation.py` - all 13 mutation tests passed; `py -3 tools/verify_lesson_examples.py` - all four examples compiled without warnings and matched exact output under GCC 13.2.0 (MinGW-w64 x86_64-ucrt-posix-seh). The script used `-D__USE_MINGW_ANSI_STDIO=1` on MinGW; the first compile exposed its default format-check mismatch for `%zu`/`%td`, and this target-specific profile resolved it without changing standard C code. Temporary binaries were removed.
- Unresolved: official N2176 text extraction is still unavailable in local readers; F04 must build and verify exercises/animation states. No runtime or grader integration is claimed.
- Next eligible task: F04, three exercises and two reversible animation models.

### F04 - completed, exercises and animation models verified (2026-10-13)

- Added three typed diagram exercises and two reversible animation models. Updated the shared record contract with executable typed error-category rules, ordered state behavior, keyboard mappings, manual playback/pause semantics, reduced-motion summaries, and a rule that model-only invalid code is never executed. Added `tools/test_learning_models.py`; all five items are marked verified with artifact hashes.
- Manually reviewed the misconception feedback, progressive hints, complete typed answers, source-line map, sample accumulation (99), boundary stop state, and no-fabricated-value handling. The state models include ArrowRight/ArrowLeft, Space, and Home actions and static summaries. This verifies the content models, not a rendered UI or keyboard integration.
- `py -3 tools/test_learning_models.py` - all 6 tests passed; `py -3 tools/check_foundation.py` - passed with fresh evidence; `py -3 tools/test_foundation.py` - all 13 mutation tests passed. All five model records cite reviewed source IDs.
- Unresolved: keyboard, pause, and reduced-motion behavior cannot be confirmed in the application until R03 connects the models to UI controls. F05 and F06 still need executable lab/project packages.
- Next eligible task: F05, implement and run the three lab packages.

### F05 - completed, three lab packages verified (2026-10-13)

- Added trace/sum, boundary-repair, and multi-file count/API packages with distinct briefs, compiling starters, instructor solutions, C fixtures, JSON Lines harnesses, structured checks, progressive hints, and known-wrong implementations. Added `tools/verify_labs.py`, package-path/manifest checks, optional sanitizer profile metadata, and updated `RECORD-FORMATS.md`, F02 API headers, pilot statuses, hashes, and evidence. All three labs are marked verified.
- `py -3 tools/verify_labs.py --compiler gcc` passed on GCC 13.2.0 MinGW-w64 (`x86_64-w64-mingw32`). Reference harnesses passed 10, 10, and 13 checks respectively. Every starter compiled without warnings and failed its expected checks. All nine included wrong implementations were rejected by named tests. The boundary suite uses physically padded sentinels, so its required failures do not rely on undefined behavior.
- The optional AddressSanitizer profile is **unavailable**, not passed: GCC's MinGW-w64 linker reports `cannot find -lasan`. The exact-size negative test remains declared for supported compiler/runtime profiles; it was not run here. No crash value or universal sanitizer behavior is assumed.
- Manual review confirmed lab 3 uses the same `analyze_readings` declarations and full status/output rules as F02; report writing is staged in its own project header. Package manifests keep solutions and mutants out of learner starter copies, use structured compiler arguments, and set finite bounds. Lab citations point only to reviewed WG14/GCC/Clang source records.
- Remaining limitation: the optional sanitizer result may be rerun on a supported toolchain. This authoring script is not an execution sandbox for arbitrary submissions. F06 completion is recorded below.

## Task result template

For each task append: task ID; completed/draft/blocked; files changed; exact checks run and observed results; source/manual review findings; unresolved issues; next eligible task. Preserve earlier entries. Check commands must be runnable from c-lab or identify a different working directory.


### F06 ? completed, project package verified (2026-10-13)

- Added the bounded sensor-data analyzer project with two supplied headers, two learner translation units, the fixed-input adapter, fixture and test harness, separate instructor reference, seven deliberate wrong implementations, project manifest, and objective-linked rubric. The API and JSON status/output rules match the F02 contract. Solutions and mutants are excluded from learner source paths.
- Reviewed the brief, C17 bounds, report schema, null/empty behavior, sentinel fixture, and four-objective rubric against the already reviewed WG14/GCC sources. Every automated rubric row resolves to manifest checks; the explanation prompt remains learner self-review. No crash or undefined behavior is used to reject the inclusive-end mutation.
- `py -3 tools/verify_project.py --compiler gcc` ? GCC 13.2.0 MinGW-w64; reference passed 43 analysis, report-write, and structured-output checks. The supplied sample adapter and reference-only `(NULL, 0)` case passed separately. The incomplete starter and all seven wrong variants were rejected by their declared checks.
- Final suite: `py -3 tools/check_foundation.py --release` and the draft checker passed; `py -3 tools/test_foundation.py` passed 13 tests; `py -3 tools/test_learning_models.py` passed 6; all four lesson examples compiled warning-free; all three lab references passed 10, 10, and 13 checks and their nine mutations were rejected.
- Limitation: MinGW's linker cannot find `-lasan`, so the optional AddressSanitizer case remains unavailable, not passed. These authoring verifiers run only repository-owned candidates and are not learner-code sandboxes. The release gate validates content structure and recorded evidence; it does not establish UI/runtime readiness, universal memory safety, or external redistribution rights.
- Next eligible task: R01, offline Rust/Tauri host and catalog.

### R01 - implementation complete; release verification pending (2026-10-13)

- Added the Rust/Tauri 2.12.2 Windows host, pinned Cargo/npm dependencies, bundled approved studio/font/icon, transactional catalog and learner migrations, and a content importer that checks schema versions, stable IDs, prerequisites, objective coverage, citations, evidence hashes, bounded lab profiles, and package-contained paths. The app stores the catalog and learner drafts in separate SQLite databases.
- The desktop catalog loads three course outlines, 24 chapters, 14 verified pilot records, and eight lexicon terms. Search and record lookup use the bundled SQLite catalog. Open Lab draft load/save uses learner schema v2; migration tests preserve existing source bytes and browser import does not replace a native draft. Non-catalog course interactions remain labeled previews.
- `py -3 tools/check_foundation.py --release` passed. `node --check design/studio.js`, `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`, `cargo clippy --locked --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings`, and `cargo test --locked --all-targets --manifest-path src-tauri/Cargo.toml` passed; Rust tests: 5 passed, 0 failed.
- Built `src-tauri/target/release/bundle/nsis/Machine Room_0.1.0_x64-setup.exe` locally (209.57 MiB). Launched the packaged executable and checked the real local databases: 3 courses, 24 chapters, 14 catalog items, 8 lexicon entries, searchable pilot text, and learner schema v2. Inspected the course collection and chapter outline screenshots at 1440x1000.
- Remaining release gate: the GitHub tag workflow must block outbound access for the app/WebView2 processes, repeat the database smoke check, and upload the installer. Keep R01 marked pending until that workflow passes.
