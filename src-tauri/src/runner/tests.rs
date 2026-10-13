use super::*;

fn root(label: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("c lab runner tests {label} {stamp}"));
    std::fs::create_dir_all(&root).unwrap();
    root
}
fn request(code: &str) -> RunRequest {
    RunRequest {
        files: vec![SourceFile {
            path: "source with spaces.c".into(),
            text: code.into(),
        }],
        standard: "C17".into(),
        include_dirs: vec![],
        link_libraries: vec![],
    }
}
fn compiler(root: &Path) -> Compiler {
    let info = discover_compiler(root);
    assert!(info.available, "{}", info.message);
    info
}
fn run(code: &str) -> JobResult {
    let root = root("process");
    execute_open(
        "test-job",
        &JobControl::default(),
        &root,
        &compiler(&root),
        &request(code),
    )
    .unwrap()
}

#[test]
fn filenames_and_submission_identity_are_bounded() {
    for invalid in [
        "../escape.c",
        "C:/escape.c",
        "/escape.c",
        "a/../escape.c",
        "a\\escape.c",
        "-flags.c",
        "a//b.c",
    ] {
        assert!(
            validate_files(&[SourceFile {
                path: invalid.into(),
                text: String::new()
            }])
            .is_err()
        );
    }
    let mut a = request("int main(void){return 0;}").files;
    let hash = submission_hash(&a);
    a[0].text.push(' ');
    assert_ne!(hash, submission_hash(&a));
    a.push(SourceFile {
        path: "helper.h".into(),
        text: "int f(void);".into(),
    });
    let hash = submission_hash(&a);
    a.reverse();
    assert_eq!(hash, submission_hash(&a));
}

#[test]
fn unavailable_compiler_is_distinct() {
    let _native_guard = TEST_JOB_LOCK.lock().unwrap();
    let root = root("missing");
    let result = execute_open(
        "missing",
        &JobControl::default(),
        &root,
        &Compiler::missing("Compiler is absent"),
        &request("int main(void){return 0;}"),
    )
    .unwrap();
    assert_eq!(result.status, "toolchain-unavailable");
    assert!(!result.submission_hash.is_empty());
    assert_eq!(result.exit_code, None);
}

#[test]
fn serialized_output_respects_byte_limits_without_splitting_unicode() {
    let mut stdout = "🙂".repeat(30_000);
    let mut stderr = "error".repeat(30_000);
    assert!(bound_text(&mut stdout, &mut stderr, OUTPUT_LIMIT));
    assert!(stdout.len() + stderr.len() <= OUTPUT_LIMIT);
    assert!(stdout.ends_with('🙂'));
}

#[cfg(windows)]
#[test]
fn normal_output_compile_error_and_failed_test_use_real_learner_code() {
    let _native_guard = TEST_JOB_LOCK.lock().unwrap();
    let good = run("#include <stdio.h>\nint main(void){puts(\"99\");return 0;}\n");
    assert_eq!(good.status, "passed");
    assert_eq!(good.stdout.replace("\r\n", "\n"), "99\n");
    assert_eq!(good.exit_code, Some(0));
    let bad = run("int main(void){ syntax broken }\n");
    assert_eq!(bad.status, "compile-error");
    assert!(!bad.stderr.is_empty());
    let failed = run(
        "#include <stdio.h>\nint main(void){puts(\"{\\\"check_id\\\":\\\"sample\\\",\\\"passed\\\":false}\");return 1;}\n",
    );
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.exit_code, Some(1));
    assert!(failed.stdout.contains("false"));
    assert_eq!(run("int main(void){return 0;}\n").status, "passed");
}

#[cfg(windows)]
#[test]
fn nonterminating_and_output_flood_programs_are_stopped() {
    let _native_guard = TEST_JOB_LOCK.lock().unwrap();
    let timeout = run("int main(void){for(;;){} }\n");
    assert_eq!(timeout.status, "timed-out");
    assert!(timeout.elapsed_ms < COMPILE_MS + RUN_MS + 2000);
    let flood = run(
        "#include <stdio.h>\nint main(void){setvbuf(stdout,0,_IONBF,0);for(;;){puts(\"01234567890123456789012345678901234567890123456789\");}}\n",
    );
    assert_eq!(flood.status, "output-limit");
    assert!(flood.truncated);
    assert!(flood.stdout.len() + flood.stderr.len() <= OUTPUT_LIMIT);
    assert_eq!(run("int main(void){return 0;}\n").status, "passed");
}

#[cfg(windows)]
#[test]
fn cancellation_and_next_job_are_reliable() {
    let _native_guard = TEST_JOB_LOCK.lock().unwrap();
    let root = root("cancel");
    let runner = Arc::new(Runner::new(root));
    let id = runner
        .start_open(request("int main(void){for(;;){} }\n"))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(12);
    while runner.snapshot(&id).unwrap().phase != "running" {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        runner
            .start_open(request("int main(void){return 0;}\n"))
            .is_err()
    );
    runner.cancel(&id).unwrap();
    let result = wait(&runner, &id);
    assert_eq!(result.status, "cancelled");
    let next = runner
        .start_open(request("int main(void){return 0;}\n"))
        .unwrap();
    assert_eq!(wait(&runner, &next).status, "passed");
}

#[cfg(windows)]
fn wait(runner: &Runner, id: &str) -> JobResult {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(result) = runner.snapshot(id).unwrap().result {
            return result;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(windows)]
#[test]
fn surviving_child_process_is_terminated_after_parent_exit() {
    let _native_guard = TEST_JOB_LOCK.lock().unwrap();
    let code = r#"#include <windows.h>
#include <stdio.h>
int main(int argc, char **argv) {
    (void)argv;
    if(argc > 1){Sleep(INFINITE);return 0;}
    char exe[MAX_PATH], cmd[MAX_PATH+32];
    GetModuleFileNameA(NULL,exe,MAX_PATH);
    snprintf(cmd,sizeof cmd,"\"%s\" child",exe);
    STARTUPINFOA startup={0};startup.cb=sizeof startup;
    PROCESS_INFORMATION child={0};
    if(!CreateProcessA(NULL,cmd,NULL,NULL,TRUE,CREATE_NO_WINDOW,NULL,NULL,&startup,&child)){return 2;}
    printf("child_pid=%lu\n",child.dwProcessId);fflush(stdout);
    CloseHandle(child.hThread);CloseHandle(child.hProcess);return 0;
}
"#;
    let result = run(code);
    assert_eq!(
        result.status, "passed",
        "{} | {} | {} | {} ms",
        result.message, result.stdout, result.stderr, result.elapsed_ms
    );
    let pid: u32 = result
        .stdout
        .trim()
        .strip_prefix("child_pid=")
        .unwrap()
        .parse()
        .unwrap();
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        // SAFETY: read-only handle to the known generated child's PID, uniquely closed.
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            break;
        }
        let mut code = 259;
        // SAFETY: valid process handle and writable exit-code storage.
        unsafe {
            GetExitCodeProcess(handle, &mut code);
            CloseHandle(handle);
        }
        if code != 259 {
            break;
        }
        assert!(Instant::now() < deadline, "Descendant survived cleanup");
        thread::sleep(Duration::from_millis(10));
    }
}
