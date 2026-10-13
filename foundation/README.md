# Machine-readable foundation

- `curriculum.json`: 24 outline chapters with stable IDs, prerequisites, and project titles, matching the approved shell.
- `pilot.json`: four objectives and 14 planned artifacts, with coverage and required evidence checks.
- `sources.json`: seven unreviewed research leads. F01 verifies text, editions, licenses, and locators before use.
- `evidence.json`: initially empty. Populate only after real checks/review; see CONTENT-CONTRACT.md.

Run from `c-lab/`:

```powershell
py -3 tools/check_foundation.py
py -3 tools/test_foundation.py
py -3 tools/check_foundation.py --release
```

The first command validates planning references and coverage. The second tests rejection of broken references, prerequisite cycles, escaping paths, invented completion, and stale evidence. The third intentionally fails while deliverables are planned; its success later only establishes artifact/evidence structure. Inspect actual evidence and run content/runtime tests before declaring delivery.

Lab/project evidence also records `package_files`, a SHA-256 map of all files beneath the manifest's directory. The checker rejects missing or stale package maps. Keep generated binaries and reports outside these directories; rerun verification after any package edit.
