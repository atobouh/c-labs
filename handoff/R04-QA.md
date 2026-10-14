# R04 native pilot verification

Completed locally on 14 October 2026 against the final small Windows build. The approved studio remains the visual baseline. Only the Arrays & Memory pilot is implemented; the remaining chapters are outlines.

## Corrections and evidence

| Before | After | Actual verification |
| --- | --- | --- |
| Compact layouts hid lesson and project file controls | Lesson navigation and learner/read-only header controls remain reachable | Ten routes at 1024, 768, and 390px; project second file selected successfully |
| Updates lost focus or restored another exercise's input | Focus, same-exercise answers, per-file selection/scroll, and independent disclosures survive updates | Keyboard hint/phase tests and file switching passed |
| Editor Tab handling obstructed normal backward navigation | Shift+Tab moves backward; Escape reaches Run | Native keyboard events passed; editor outline/shadow remain none |
| Model shortcuts affected other screens | Shortcuts apply inside the model view | Space marked an actual review card instead of changing a hidden model |
| Restored edited drafts could show an unqualified old pass | Previous submissions stay labelled | Saved source edited, navigated away, restored with previous-result label |
| Incomplete compiler target could omit MinGW's format profile | Incomplete probes are rejected; known MinGW identity preserves C17 stdio configuration | Real project regression and interrupted-job recovery passed |

## Native test matrix

`tools/test_desktop_r03.mjs` plus `tools/audit_desktop_r04.mjs` passed **28 checks**, with WebView networking disabled and zero page JavaScript exceptions. The fresh isolated learner/WebView profile exercised all four executable examples, three typed exercises, reversible models, three graded labs, project acceptance, stale/cancelled/interrupted results, relaunch, and immutable history. Required lab checks passed 11/11/14; project checks passed 45.

Thirty route/layout cases cover courses, chapter outline, reader, exercises, labs, project, project collection, review, lexicon, and Open Lab. Loading, package error/retry, empty review/search, compiler error, and unavailable compiler states were captured at each width. Package read failure/latency uses a temporary test namespace; actual compiler absence/configuration and diagnostics use native commands. No production learner data is used.

Captured screens were visually inspected through route contact sheets and full-size compact reader, project/editor, error, unavailable compiler, and reduced-motion images. Dark editor scrolling stays contained; fields have no editor outline or resize handle. The caret/filename marker and normal control focus remain the keyboard cues. Secondary counts no longer wrap at 768px.

## Reproduce

```powershell
py -3 tools/check_foundation.py --release
py -3 tools/test_foundation.py
py -3 tools/test_learning_models.py
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --offline --all-targets --manifest-path src-tauri/Cargo.toml
cargo clippy --offline --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings
node tools/test_desktop_r03.mjs <machine-room.exe> <new-evidence-directory> tools/audit_desktop_r04.mjs
```

Rust: 16 tests passed; foundation mutations: 13; learning models: 6. JavaScript syntax and whitespace checks passed. The runtime suite compiles all references/starters and rejects the 16 bundled wrong candidates. Final evidence: `evidence/r04-native-windows.json`; selected native captures: `../screens/r04-*.png`.

## Delivery and limits

Local installer: 3,051,310 bytes (2.91 MiB), SHA-256 `a84eba63d9ee8e59eeb7f8ec907c5db89f6cb6bd80a84a6a1d9c6fea1d7de03c`. The compiler remains a prerequisite. The installer reuses existing WebView2; installation may download it when absent. CI now ensures this prerequisite before native testing, following [Microsoft's distribution instructions](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution). Remote CI success is not inferred from local results.

Native minimum remains 1024x768; smaller widths are emulated. This is a bounded keyboard/state/layout audit, not screen-reader or WCAG certification. The optional sanitizer remains unavailable on the local MinGW profile, and the explanation rubric remains self-review. The next content task is Chapter 1, Build & Run.
