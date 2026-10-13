# R03 learning runtime

The bundled Arrays & Memory pilot is live: four lessons and executable examples, three typed diagram exercises, two reversible conceptual models, three editable lab packages, and the two-file sensor project. The other chapters remain outlines. The pilot assumes compilation, integer arithmetic, conditions, loops, and functions.

## Boundaries

`learning.rs` verifies package fingerprints before opening/grading, snapshots supplied files into a dedicated job directory, and accepts exactly the manifest's learner paths. Instructor solutions and mutants never enter the grading workspace. Tests run through R02's Windows job controller with the same deadlines, output limits, cancellation, and process-tree cleanup. C17 warnings become errors. This executes trusted local code with the user's privileges; it is not a sandbox.

Lab JSON Lines must contain each declared assertion exactly once and agree with the exit code. The project additionally validates report identifiers/order, exact JSON key order, integer types, null/status rules, mean tolerance, final newlines, and a separately compiled adapter. Optional sanitizer checks are unavailable until verified for that toolchain; they cannot become fabricated passes.

## Persistence and gates

Learner schema 3 migrates existing Open Lab drafts transactionally. SQLite stores chapter/package drafts, read flags, model positions, hints, typed exercise attempts, review cards, local library settings, and grading attempts. A running attempt and its source bytes are persisted before compilation. Relaunch marks unfinished attempts interrupted. Completed attempt results and submitted source snapshots are immutable; the UI shows the latest 100 attempts while the database retains earlier records.

Each attempt records source hash, content hash/revision, rubric version, and compiler identity. Only a current real project pass with the latest saved source and explicit learner self-review can grant the `foundations-strings-input` gate. Failed, cancelled, interrupted, stale, or obsolete-rubric attempts cannot grant it. Later edits retain a previously valid pass; revised bundled content displays older attempts without claiming they passed the new revision. Self-review is an acknowledgement, not machine assessment of the explanation.

## Verification

Run `cargo test --locked --all-targets --manifest-path src-tauri/Cargo.toml` with a local GCC/Clang. Native compiler tests serialize their finite jobs. All references/starters and 16 bundled mutants pass the expected acceptance/rejection assertions.

For the actual packaged WebView2 UI, Node 22+ requires no test dependencies:

```powershell
node tools/test_desktop_r03.mjs <path-to-machine-room.exe> <new-evidence-directory>
```

The script creates isolated learner/WebView data, starts only its own app process, disables WebView network access, checks every pilot stage, stale/cancelled/interrupted grading, relaunch, and narrow layouts, and records screenshots plus `evidence.json`. It refuses an existing evidence directory and never edits production learner data. Its reference submissions are explicit test fixtures; normal learners receive only starters. Screenshots supplement actual results, not substitute for them.
