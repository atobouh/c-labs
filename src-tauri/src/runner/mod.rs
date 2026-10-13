#[cfg(test)]
mod tests;
#[cfg(windows)]
mod windows;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub const OUTPUT_LIMIT: usize = 65_536;
pub const COMPILE_MS: u64 = 10_000;
pub const RUN_MS: u64 = 2_000;
pub const MAX_SOURCE_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Serialize)]
pub struct Compiler {
    pub available: bool,
    pub path: String,
    pub identity: String,
    pub target: String,
    pub c23: bool,
    pub message: String,
}
impl Compiler {
    fn missing(message: impl Into<String>) -> Self {
        Self {
            available: false,
            path: String::new(),
            identity: String::new(),
            target: String::new(),
            c23: false,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub files: Vec<SourceFile>,
    pub standard: String,
    #[serde(default)]
    pub include_dirs: Vec<String>,
    #[serde(default)]
    pub link_libraries: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub description: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub submission_hash: String,
    pub content_revision: Option<u32>,
    pub rubric_version: Option<u32>,
    pub package_id: Option<String>,
    pub content_hash: Option<String>,
    pub toolchain: String,
    pub status: String,
    pub message: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    pub elapsed_ms: u64,
    pub checks: Vec<CheckResult>,
}

#[derive(Serialize)]
pub struct JobSnapshot {
    pub job_id: String,
    pub phase: String,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
    pub result: Option<JobResult>,
}

#[derive(Default)]
pub(crate) struct Capture {
    stdout: Mutex<Vec<u8>>,
    stderr: Mutex<Vec<u8>>,
    bytes: AtomicUsize,
    exceeded: AtomicBool,
}
impl Capture {
    fn text(&self) -> (String, String) {
        let (mut stdout, mut stderr) = (
            String::from_utf8_lossy(&self.stdout.lock().unwrap()).into_owned(),
            String::from_utf8_lossy(&self.stderr.lock().unwrap()).into_owned(),
        );
        if bound_text(&mut stdout, &mut stderr, OUTPUT_LIMIT) {
            self.exceeded.store(true, Ordering::Relaxed);
        }
        (stdout, stderr)
    }
}

pub struct JobControl {
    pub cancelled: AtomicBool,
    phase: Mutex<String>,
    capture: Mutex<Arc<Capture>>,
    result: Mutex<Option<JobResult>>,
}
impl Default for JobControl {
    fn default() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            phase: Mutex::new("queued".into()),
            capture: Mutex::new(Arc::new(Capture::default())),
            result: Mutex::new(None),
        }
    }
}
impl JobControl {
    pub(crate) fn phase(&self, phase: &str, capture: Arc<Capture>) {
        *self.phase.lock().unwrap() = phase.into();
        *self.capture.lock().unwrap() = capture;
    }
}

pub struct Runner {
    pub root: PathBuf,
    compiler: Mutex<Option<Compiler>>,
    jobs: Mutex<BTreeMap<String, Arc<JobControl>>>,
    next: AtomicU64,
}
impl Runner {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            compiler: Mutex::new(None),
            jobs: Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(1),
        }
    }

    pub fn toolchain(&self) -> Compiler {
        let mut cached = self.compiler.lock().unwrap();
        if let Some(info) = cached.as_ref() {
            return info.clone();
        }
        let configured = std::fs::read_to_string(self.root.join("compiler-path.txt")).ok();
        let info = if let Some(path) = configured {
            probe_compiler(Path::new(path.trim()), &self.root).unwrap_or_else(Compiler::missing)
        } else {
            discover_compiler(&self.root)
        };
        *cached = Some(info.clone());
        info
    }

    pub fn configure_compiler(&self, path: &str) -> Result<Compiler, String> {
        let info = probe_compiler(Path::new(path), &self.root)?;
        std::fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        std::fs::write(self.root.join("compiler-path.txt"), &info.path)
            .map_err(|e| e.to_string())?;
        *self.compiler.lock().unwrap() = Some(info.clone());
        Ok(info)
    }

    pub fn start_open(self: &Arc<Self>, request: RunRequest) -> Result<String, String> {
        validate_files(&request.files)?;
        if !matches!(request.standard.as_str(), "C17" | "C23") {
            return Err("Choose C17 or C23.".into());
        }
        if request.include_dirs.len() > 8
            || request
                .include_dirs
                .iter()
                .any(|p| !Path::new(p).is_absolute() || !Path::new(p).is_dir())
        {
            return Err("Use at most eight existing absolute include-directory paths.".into());
        }
        let compiler = self.toolchain();
        if request.link_libraries.len() > 16
            || request.link_libraries.iter().any(|name| {
                name.is_empty()
                    || name.len() > 64
                    || !name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            })
        {
            return Err(
                "Link libraries must be simple library names, without flags or paths.".into(),
            );
        }
        self.start(move |id, control, root| execute_open(&id, &control, &root, &compiler, &request))
    }

    pub(crate) fn start<F>(self: &Arc<Self>, work: F) -> Result<String, String>
    where
        F: FnOnce(String, Arc<JobControl>, PathBuf) -> Result<JobResult, String> + Send + 'static,
    {
        let mut jobs = self.jobs.lock().unwrap();
        if jobs.values().any(|j| j.result.lock().unwrap().is_none()) {
            return Err("A job is already running. Cancel it or wait for it to finish.".into());
        }
        if jobs.len() >= 64 {
            let oldest = jobs.keys().next().cloned().unwrap();
            jobs.remove(&oldest);
        }
        let sequence = self.next.fetch_add(1, Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let id = format!("job-{}-{stamp}-{sequence}", std::process::id());
        let root = self.root.join(&id);
        std::fs::create_dir_all(&root).map_err(|e| format!("Cannot create run workspace: {e}"))?;
        let control = Arc::new(JobControl::default());
        jobs.insert(id.clone(), control.clone());
        let worker_id = id.clone();
        thread::spawn(move || {
            let started = Instant::now();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                work(worker_id.clone(), control.clone(), root)
            }));
            let result = match outcome {
                Ok(Ok(result)) => result,
                Ok(Err(error)) => error_result(
                    &worker_id,
                    "internal-error",
                    &error,
                    started.elapsed().as_millis() as u64,
                ),
                Err(_) => error_result(
                    &worker_id,
                    "internal-error",
                    "Job stopped after an internal error.",
                    started.elapsed().as_millis() as u64,
                ),
            };
            *control.phase.lock().unwrap() = result.status.clone();
            *control.result.lock().unwrap() = Some(result);
        });
        Ok(id)
    }

    pub fn snapshot(&self, id: &str) -> Result<JobSnapshot, String> {
        let jobs = self.jobs.lock().unwrap();
        let job = jobs
            .get(id)
            .ok_or("This job is no longer in the current session.")?;
        let capture = job.capture.lock().unwrap().clone();
        let (stdout, stderr) = capture.text();
        Ok(JobSnapshot {
            job_id: id.into(),
            phase: job.phase.lock().unwrap().clone(),
            stdout,
            stderr,
            truncated: capture.exceeded.load(Ordering::Relaxed),
            result: job.result.lock().unwrap().clone(),
        })
    }
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let jobs = self.jobs.lock().unwrap();
        let job = jobs.get(id).ok_or("Unknown job.")?;
        job.cancelled.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub fn cancel_all(&self) {
        for job in self.jobs.lock().unwrap().values() {
            job.cancelled.store(true, Ordering::Relaxed);
        }
    }
}

pub fn validate_files(files: &[SourceFile]) -> Result<(), String> {
    if files.is_empty() || files.len() > 16 {
        return Err("A submission needs 1–16 source/header files.".into());
    }
    let mut seen = HashSet::new();
    let mut bytes = 0usize;
    for file in files {
        portable_path(&file.path)?;
        if !matches!(
            Path::new(&file.path).extension().and_then(|s| s.to_str()),
            Some("c" | "h")
        ) {
            return Err("Only .c and .h source files are accepted.".into());
        }
        if !seen.insert(file.path.to_lowercase()) {
            return Err("Duplicate source path.".into());
        }
        bytes = bytes
            .checked_add(file.text.len())
            .ok_or("Source size overflow.")?;
    }
    if bytes > MAX_SOURCE_BYTES || !files.iter().any(|f| f.path.ends_with(".c")) {
        return Err("Provide C source, up to 1 MiB total.".into());
    }
    Ok(())
}

pub(crate) fn portable_path(path: &str) -> Result<(), String> {
    if path.split('/').any(|part| {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    }) {
        return Err("Source paths cannot use reserved Windows names or characters.".into());
    }
    if path.is_empty()
        || path.contains(['\\', ':', '\0'])
        || path.starts_with(['/', '-'])
        || path
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Paths must remain inside the submission workspace.".into());
    }
    Ok(())
}

pub(crate) fn submission_hash(files: &[SourceFile]) -> String {
    let mut ordered: Vec<_> = files.iter().collect();
    ordered.sort_by(|a, b| a.path.cmp(&b.path));
    let mut hash = Sha256::new();
    for file in ordered {
        hash.update((file.path.len() as u64).to_le_bytes());
        hash.update(file.path.as_bytes());
        hash.update((file.text.len() as u64).to_le_bytes());
        hash.update(file.text.as_bytes());
    }
    format!("{:x}", hash.finalize())
}

pub(crate) fn write_files(root: &Path, files: &[SourceFile]) -> Result<(), String> {
    validate_files(files)?;
    for file in files {
        let path = root.join(&file.path);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(path, &file.text).map_err(|e| format!("Cannot write source: {e}"))?;
    }
    Ok(())
}

pub(crate) fn error_result(id: &str, status: &str, message: &str, elapsed_ms: u64) -> JobResult {
    JobResult {
        job_id: id.into(),
        submission_hash: String::new(),
        content_revision: None,
        rubric_version: None,
        package_id: None,
        content_hash: None,
        toolchain: String::new(),
        status: status.into(),
        message: message.into(),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        truncated: false,
        elapsed_ms,
        checks: vec![],
    }
}

pub(crate) struct ProcessOutput {
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}

pub(crate) fn run_process(
    command: &mut Command,
    control: &JobControl,
    phase: &str,
    timeout_ms: u64,
    limit: usize,
) -> Result<ProcessOutput, String> {
    #[cfg(not(windows))]
    {
        let _ = (command, control, phase, timeout_ms, limit);
        Err("Native job supervision currently supports Windows 10/11 only.".into())
    }
    #[cfg(windows)]
    {
        if control.cancelled.load(Ordering::Relaxed) {
            return Ok(ProcessOutput {
                status: "cancelled".into(),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                truncated: false,
            });
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let (mut child, tree) = windows::ProcessTree::spawn(command, phase == "compiling")?;
        let capture = Arc::new(Capture::default());
        control.phase(phase, capture.clone());
        let stdout = reader(child.stdout.take().unwrap(), capture.clone(), false, limit);
        let stderr = reader(child.stderr.take().unwrap(), capture.clone(), true, limit);
        let started = Instant::now();
        let (status, exit_code) = loop {
            if control.cancelled.load(Ordering::Relaxed) {
                break ("cancelled", None);
            }
            if capture.exceeded.load(Ordering::Relaxed) {
                break ("output-limit", None);
            }
            if started.elapsed() >= Duration::from_millis(timeout_ms) {
                break ("timed-out", None);
            }
            match child.try_wait() {
                Ok(Some(code)) => {
                    break (
                        if code.success() { "passed" } else { "failed" },
                        code.code(),
                    );
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    tree.terminate();
                    let _ = child.wait();
                    let _ = stdout.join();
                    let _ = stderr.join();
                    return Err(format!("Cannot wait for process: {error}"));
                }
            }
        };
        // Kill descendants even after their parent exited, before joining pipe readers.
        tree.terminate();
        let _ = child.wait();
        stdout
            .join()
            .map_err(|_| "Output reader stopped unexpectedly.")??;
        stderr
            .join()
            .map_err(|_| "Error reader stopped unexpectedly.")??;
        let (stdout, stderr) = capture.text();
        let truncated = capture.exceeded.load(Ordering::Relaxed);
        Ok(ProcessOutput {
            status: if truncated && status != "cancelled" {
                "output-limit"
            } else {
                status
            }
            .into(),
            exit_code,
            stdout,
            stderr,
            truncated,
        })
    }
}

#[cfg(windows)]
fn reader<R: Read + Send + 'static>(
    mut pipe: R,
    capture: Arc<Capture>,
    is_error: bool,
    limit: usize,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let mut chunk = [0u8; 4096];
        loop {
            let count = pipe
                .read(&mut chunk)
                .map_err(|e| format!("Output read failed: {e}"))?;
            if count == 0 {
                return Ok(());
            }
            let before = capture.bytes.fetch_add(count, Ordering::Relaxed);
            let keep = count.min(limit.saturating_sub(before));
            if before.saturating_add(count) > limit {
                capture.exceeded.store(true, Ordering::Relaxed);
            }
            if keep > 0 {
                let output = if is_error {
                    &capture.stderr
                } else {
                    &capture.stdout
                };
                output.lock().unwrap().extend_from_slice(&chunk[..keep]);
            }
        }
    })
}

fn discover_compiler(root: &Path) -> Compiler {
    let mut candidates = Vec::new();
    let mut failure = String::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            candidates.push(dir.join("gcc.exe"));
            candidates.push(dir.join("clang.exe"));
        }
    }
    for path in [
        "C:/Strawberry/c/bin/gcc.exe",
        "C:/msys64/ucrt64/bin/gcc.exe",
        "C:/msys64/mingw64/bin/gcc.exe",
        "C:/Program Files/LLVM/bin/clang.exe",
    ] {
        candidates.push(PathBuf::from(path));
    }
    for path in candidates {
        if path.is_file() {
            match probe_compiler(&path, root) {
                Ok(info) => return info,
                Err(error) if failure.is_empty() => failure = error,
                Err(_) => (),
            }
        }
    }
    Compiler::missing(format!(
        "No C17-capable GCC or Clang found. Install a local toolchain or choose its executable; no compiler is downloaded automatically. {failure}"
    ))
}

fn probe_compiler(path: &Path, root: &Path) -> Result<Compiler, String> {
    if !path.is_absolute()
        || !path.is_file()
        || path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            != Some("exe".into())
    {
        return Err("Choose an existing absolute GCC/Clang .exe path.".into());
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let canonical = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    // GCC locates subordinate tools from argv[0]; use regular Windows paths.
    let canonical_text = canonical.to_string_lossy();
    let path = if let Some(unc) = canonical_text.strip_prefix("\\\\?\\UNC\\") {
        PathBuf::from(format!("\\\\{unc}"))
    } else {
        PathBuf::from(
            canonical_text
                .strip_prefix("\\\\?\\")
                .unwrap_or(&canonical_text),
        )
    };
    let control = JobControl::default();
    let output = run_process(
        Command::new(&path).arg("--version").current_dir(root),
        &control,
        "detecting",
        2000,
        16384,
    )?;
    if output.status != "passed"
        || !output.stdout.to_lowercase().contains("gcc")
            && !output.stdout.to_lowercase().contains("clang")
    {
        return Err("Executable did not identify as GCC or Clang.".into());
    }
    let identity = output
        .stdout
        .lines()
        .next()
        .unwrap_or("Unknown compiler")
        .to_owned();
    let target = run_process(
        Command::new(&path).arg("-dumpmachine").current_dir(root),
        &control,
        "detecting",
        2000,
        16384,
    )?;
    let probe = |standard: &str| -> Result<bool, String> {
        Ok(run_process(
            Command::new(&path)
                .args([standard, "-x", "c", "-fsyntax-only", "-"])
                .current_dir(root),
            &control,
            "detecting",
            2000,
            16384,
        )?
        .status
            == "passed")
    };
    if !probe("-std=c17")? {
        return Err("This compiler does not accept C17 mode.".into());
    }
    let c23 = probe("-std=c23")?;
    Ok(Compiler {
        available: true,
        path: path.to_string_lossy().into_owned(),
        identity,
        target: target.stdout.trim().into(),
        c23,
        message: "Ready for local trusted-code execution. This is not a sandbox.".into(),
    })
}

pub(crate) fn compiler_command(compiler: &Compiler, root: &Path, standard: &str) -> Command {
    let mut command = Command::new(&compiler.path);
    command.current_dir(root);
    command.args([
        if standard == "C23" {
            "-std=c23"
        } else {
            "-std=c17"
        },
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-fdiagnostics-color=never",
    ]);
    if compiler.target.to_lowercase().contains("mingw") {
        command.arg("-D__USE_MINGW_ANSI_STDIO=1");
    }
    // MinGW installations sometimes keep required compiler/runtime DLLs beside gcc.
    if let Some(dir) = Path::new(&compiler.path).parent() {
        let mut dirs = vec![dir.to_owned()];
        if let Some(paths) = std::env::var_os("PATH") {
            dirs.extend(std::env::split_paths(&paths));
        }
        if let Ok(paths) = std::env::join_paths(dirs) {
            command.env("PATH", paths);
        }
    }
    command
}

pub(crate) fn execute_open(
    id: &str,
    control: &JobControl,
    root: &Path,
    compiler: &Compiler,
    request: &RunRequest,
) -> Result<JobResult, String> {
    let started = Instant::now();
    let mut result = error_result(id, "toolchain-unavailable", &compiler.message, 0);
    result.submission_hash = submission_hash(&request.files);
    result.toolchain = format!("{} [{}]", compiler.identity, compiler.target);
    if !compiler.available {
        return Ok(result);
    }
    if request.standard == "C23" && !compiler.c23 {
        result.status = "toolchain-unavailable".into();
        result.message =
            "This compiler does not accept -std=c23. Choose C17 or configure a newer compiler."
                .into();
        return Ok(result);
    }
    write_files(root, &request.files)?;
    let executable = root.join("learner.exe");
    let mut compile = compiler_command(compiler, root, &request.standard);
    for include in &request.include_dirs {
        compile.arg("-I").arg(include);
        compile.arg("-L").arg(include);
    }
    for file in &request.files {
        if file.path.ends_with(".c") {
            compile.arg(&file.path);
        }
    }
    for library in &request.link_libraries {
        compile.arg(format!("-l{library}"));
    }
    compile.arg("-o").arg(&executable);
    let compiled = run_process(&mut compile, control, "compiling", COMPILE_MS, OUTPUT_LIMIT)?;
    if compiled.status != "passed" {
        result.status = if compiled.status == "failed" {
            "compile-error"
        } else {
            &compiled.status
        }
        .into();
        result.message = "Compilation did not finish successfully.".into();
        result.stdout = compiled.stdout;
        result.stderr = compiled.stderr;
        result.exit_code = compiled.exit_code;
        result.truncated = compiled.truncated;
    } else {
        let mut run = Command::new(&executable);
        run.current_dir(root);
        if let Some(path) = compile
            .get_envs()
            .find(|(key, _)| *key == "PATH")
            .and_then(|(_, value)| value)
        {
            run.env("PATH", path);
        }
        let output = run_process(&mut run, control, "running", RUN_MS, OUTPUT_LIMIT)?;
        result.status = output.status;
        result.exit_code = output.exit_code;
        result.stdout = output.stdout;
        result.stderr = format!("{}{}", compiled.stderr, output.stderr);
        result.truncated = output.truncated;
        result.message = match result.status.as_str() {
            "passed" => "Program exited successfully.",
            "timed-out" => "Program exceeded its 2-second deadline; its process tree was stopped.",
            "cancelled" => "Job cancelled; its process tree was stopped.",
            "output-limit" => {
                "Program exceeded the 64 KiB output limit; its process tree was stopped."
            }
            _ => "Program exited with a failure status.",
        }
        .into();
    }
    if bound_text(&mut result.stdout, &mut result.stderr, OUTPUT_LIMIT) {
        result.truncated = true;
        if result.status == "passed" {
            result.status = "output-limit".into();
            result.message = "Combined output exceeded the 64 KiB result limit.".into();
        }
    }
    result.elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(result)
}

pub(crate) fn bound_text(stdout: &mut String, stderr: &mut String, limit: usize) -> bool {
    if stdout.len() + stderr.len() <= limit {
        return false;
    }
    let mut end = stdout.len().min(limit);
    while !stdout.is_char_boundary(end) {
        end -= 1;
    }
    stdout.truncate(end);
    let mut end = stderr.len().min(limit - stdout.len());
    while !stderr.is_char_boundary(end) {
        end -= 1;
    }
    stderr.truncate(end);
    true
}
