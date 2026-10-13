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

The NSIS installer is written under `src-tauri/target/release/bundle/nsis/`. Download the [small R01 installer](https://github.com/atobouh/c-labs/releases/download/v0.1.0-r01/Machine.Room_0.1.0_x64-setup-slim.exe). For installation on a computer without WebView2 and without internet, `npm run tauri:build:offline` explicitly bundles Microsoft's large offline runtime installer.

## Offline data and current scope

The app embeds the approved local studio, three course outlines, the verified Arrays & Memory pilot package, and its lexicon. It imports and validates these files into a versioned SQLite catalog on first launch; course search and record lookup use that database without network access. Learner drafts live in a separate SQLite database and survive transactional schema upgrades. Existing browser drafts can be imported once without replacing a native draft.

R01 delivers the native host, catalog, course outline view, local search, lexicon, and draft storage. Lesson, exercise, animation, lab execution, project grading, and chapter progression remain clearly marked previews. The app does not compile or execute learner code.

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
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --locked --all-targets --manifest-path src-tauri/Cargo.toml
```

Course authoring checks are documented in `handoff/STATUS.md`. The Windows release workflow runs the content gate, Rust tests, installer build, and a launch smoke test with WebView network access blocked.
