# Machine Room ? C learning project

This repository contains the C-Lab / Machine Room project only. Its Git root is this `c-lab/` directory; sibling projects such as Tandem are separate and are not part of this repository.

## Preview the approved shell

From this directory, run:

```powershell
py -m http.server 8768 --bind 127.0.0.1
```

Open <http://127.0.0.1:8768/design/studio.html>. The prototype is fully local and demonstrates the course collection, chapter lessons, diagram exercises, guided labs, project gate, lexicon, open lab, and review. Sample progress and checks are design scenarios; compiler execution, application grading, persistence, and the Tauri host are not implemented yet.

Screen captures and their gallery are in `screens/`; design explorations and the approved HTML/CSS/JS shell are in `design/`.

## Project layout

- `assets/` ? bundled font and its license.
- `content/` ? original lesson, exercise, animation, lab, and project records and C fixtures.
- `design/` ? approved shell, visual direction, reference board, previews, and archived prototypes.
- `foundation/` ? course map, reviewed sources, pilot manifest, and verification evidence.
- `handoff/` ? task queue, implementation contracts, QA, and contributor handoff.
- `research/` ? course/product research; third-party desktop captures remain local-only and are ignored by Git pending individual rights review.
- `screens/` ? reviewed screenshots and gallery.
- `specimens/` ? small verified C examples.
- `tools/` ? dependency-free Python validation scripts.

## Checks

Run from the repository root. GCC is required for the C verification commands; the recorded Windows profile uses GCC 13.2.0 MinGW-w64.

```powershell
py -3 tools/check_foundation.py
py -3 tools/test_foundation.py
py -3 tools/test_learning_models.py
py -3 tools/verify_lesson_examples.py
py -3 tools/verify_labs.py --compiler gcc
py -3 tools/verify_project.py --compiler gcc
py -3 tools/check_foundation.py --release
node --check design/studio.js
```

The release checker verifies structure and recorded evidence; it does not prove factual accuracy, license rights, or desktop-runtime readiness. The optional MinGW AddressSanitizer case is unavailable because this GCC distribution lacks `-lasan`.
