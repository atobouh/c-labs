# Native runtime contract

## R02 commands and results

`toolchain_status` detects an installed GCC/Clang executable and reports its absolute path, version, target, and whether `-std=c23` is accepted. `set_compiler_path` validates and persists an explicit executable. No compiler downloads, services, or shell commands are used. C17 is the default; unsupported C23 is unavailable rather than silently replaced with a draft language mode.

`start_run` accepts `{ files: [{ path, text }], standard, include_dirs, link_libraries }`. File paths are portable relative `.c`/`.h` paths without parent traversal, drive prefixes, backslashes, or duplicate names. At most 16 files / 1 MiB are accepted. Include directories must exist and be absolute (maximum eight); link-library names are plain names (maximum 16), passed as separate compiler arguments. Source, paths, and options are never shell-interpolated.

`get_run({ job_id })` returns a snapshot with ID, phase, bounded current stdout/stderr, truncation, and an optional completed result. The JavaScript invoke arguments use Tauri's camelCase (`jobId`). `cancel_run` requests cancellation. One job can execute at a time; the latest 64 completed session snapshots remain available. Source and binary workspaces are retained under the app-data `jobs/` folder; results/progression storage is added by R03.

Every completed result has `job_id`, deterministic `submission_hash`, optional `package_id`, `content_revision`, `content_hash`, `rubric_version`, `toolchain`, `status`, `message`, `exit_code`, `stdout`, `stderr`, `truncated`, `elapsed_ms`, and `checks`. Open Lab success is process exit success, not a graded course pass. Statuses distinguish `passed`, `failed`, `compile-error`, `timed-out`, `cancelled`, `output-limit`, `toolchain-unavailable`, and `internal-error`. The UI associates results with their job/source and marks changed source as previous-run output.

## Windows execution bounds

- Compiler deadline: 10 seconds. Program deadline: 2 seconds.
- Combined stdout/stderr capture per phase and displayed result: 64 KiB; readers continue draining without accumulating excess data while the controller terminates the process tree.
- Windows job limits: eight simultaneous processes; compiler phase 512 MiB per process / 1 GiB total; program phase 256 MiB per process / 512 MiB total. Kill on final job-handle close.
- Each child starts suspended, joins its job before executing, and resumes through the documented thread API. No unsupervised fallback is permitted if job assignment fails.
- Timeout, cancellation, output overflow, parent exit, and app shutdown stop descendant processes. Pipe readers are joined after tree termination, including when a surviving child inherited the pipes.

This currently supports Windows 10/11. It executes trusted local C with the user's privileges; time, output, memory, and process-count limits **are not a security sandbox**. Programs can access local files and networks. No OS isolation claim is made.

The normal installer reuses the installed WebView2 runtime; it downloads WebView2 only if absent. The compiler is an installed prerequisite. The app/course data operate offline after these prerequisites are present. `CLAB_DATA_DIR` is a development/test override for isolated learner data; set it before launch to avoid touching normal learner progress during integration tests.

## Primary implementation references

- [Microsoft: Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) — descendant association, nested jobs, and kill-on-close behavior.
- [Microsoft: AssignProcessToJobObject](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject) — assignment and failure conditions.
- [Microsoft: process creation flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags) — suspended startup and hidden console behavior.
- [GCC warning options](https://gcc.gnu.org/onlinedocs/gcc/Warning-Options.html) — `-Wall`, `-Wextra`, `-Wpedantic`; output wording is kept as actual diagnostics.
