# Machine Room - Offline C Learning Studio

This repository contains the C-Lab / Machine Room project only. Its Git root is this `c-lab/` directory; neighboring projects such as Tandem are separate repositories.

## Run the desktop app

Install Node.js 22+, Rust 1.90+, the Visual Studio C++ build tools, and WebView2. From this directory:

```powershell
npm ci
npm run tauri:dev
```

Build the small Windows installer (the default). It uses the installed WebView2 runtime; setup needs internet only if that runtime is missing. The app and bundled courses work offline:

```powershell
npm run tauri:build:windows
```

The NSIS installer is written under `src-tauri/target/release/bundle/nsis/`. Download the [small R03 installer](https://github.com/atobouh/c-labs/releases/download/v0.1.0-r03/Machine.Room_0.1.0_x64-setup.exe). For installation on a computer without WebView2 and without internet, `npm run tauri:build:offline` explicitly bundles Microsoft's large offline runtime installer.

## Offline data and current scope

The app embeds the approved local studio, three course outlines, the verified Arrays & Memory pilot package, and its lexicon. It imports and validates these files into a versioned SQLite catalog on first launch; course search and record lookup use that database without network access. Learner drafts live in a separate SQLite database and survive transactional schema upgrades. Existing browser drafts can be imported once without replacing a native draft.

R01 delivers the native host, catalog, course outline view, local search, lexicon, and draft storage. R02 adds real Open Lab compilation/execution with an installed C17-capable GCC or Clang: use Configure beside the compiler status to select an executable when detection is unavailable. Jobs support live output, cancellation, deadlines, output bounds, and Windows process-tree cleanup. C23 is available only when the compiler accepts `-std=c23`. R03 connects the verified pilot: four lessons with runnable examples, three typed exercises, two reversible models, three editable labs, and the two-file graded sensor project. Drafts, hints, reviews, attempts, and version-bound unlocking survive restart. The explanation rubric is explicit self-review. Other chapters remain outlines; R04 full pilot polish is next.

The compiler is an installed prerequisite and is not downloaded automatically. Open Lab executes trusted local code with your privileges, rather than in a security sandbox. See [the runtime contract](handoff/RUNTIME.md) for supported platforms, limits, structured options, and test/data locations.

## Project layout

- `src-tauri/` - Rust host, catalog validation/import, SQLite migrations, and Tauri configuration.
- `design/` - approved HTML/CSS/JS studio and bundled local font.
- `foundation/` and `content/` - course map, source/evidence manifests, and verified pilot records.
- `handoff/` - task queue, contracts, status, and contributor instructions.
- `tools/` - content validators and Windows offline-launch smoke test.
- `screens/`, `specimens/`, and `research/` - reviewed screenshots, small C examples, and research notes.

## Checks

```powershell
py -3 tools/check_foundation.py --release
node --check design/studio.js
node --check design/runtime.js
node --check design/learning.js
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --locked --all-targets --manifest-path src-tauri/Cargo.toml
```

Course authoring checks are documented in `handoff/STATUS.md`. The Windows release workflow runs the content gate, Rust tests, installer build, and a launch smoke test with WebView network access blocked.

Native pilot integration checks run with Node 22 and an isolated data directory: `node tools/test_desktop_r03.mjs <machine-room.exe> <new-evidence-directory>`. See [learning runtime details](handoff/LEARNING-RUNTIME.md).
