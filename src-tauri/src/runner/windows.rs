use std::{
    io,
    mem::size_of,
    os::windows::{io::AsRawHandle, process::CommandExt},
    process::{Child, Command},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        },
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
            JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JobObjectExtendedLimitInformation, SetInformationJobObject, TerminateJobObject,
        },
        Threading::{
            CREATE_NO_WINDOW, CREATE_SUSPENDED, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
        },
    },
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: this uniquely owned, non-inherited handle is valid until drop.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub struct ProcessTree(Handle);
impl ProcessTree {
    pub fn spawn(command: &mut Command, compiling: bool) -> Result<(Child, Self), String> {
        // SAFETY: no security descriptor/name; handle is owned locally and not inherited.
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if raw.is_null() {
            return Err(format!(
                "Cannot create process job: {}",
                io::Error::last_os_error()
            ));
        }
        let tree = Self(Handle(raw));
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_JOB_MEMORY;
        limits.BasicLimitInformation.ActiveProcessLimit = 8;
        limits.ProcessMemoryLimit = if compiling { 512 } else { 256 } * 1024 * 1024;
        limits.JobMemoryLimit = if compiling { 1024 } else { 512 } * 1024 * 1024;
        // SAFETY: correctly sized initialized structure and valid job handle.
        if unsafe {
            SetInformationJobObject(
                raw,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(format!(
                "Cannot configure process job: {}",
                io::Error::last_os_error()
            ));
        }
        command.creation_flags(CREATE_SUSPENDED | CREATE_NO_WINDOW);
        let mut child = command
            .spawn()
            .map_err(|e| format!("Cannot start process: {e}"))?;
        // Attach before any user/compiler code can run or spawn descendants.
        // SAFETY: std owns the live child process handle; the job outlives this call.
        if unsafe { AssignProcessToJobObject(raw, child.as_raw_handle() as HANDLE) } == 0 {
            let error = io::Error::last_os_error();
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("Cannot supervise process tree: {error}"));
        }
        if let Err(error) = resume_initial_thread(child.id()) {
            tree.terminate();
            let _ = child.wait();
            return Err(error);
        }
        Ok((child, tree))
    }

    pub fn terminate(&self) {
        // SAFETY: valid job owned by self. Also terminates surviving descendants.
        unsafe {
            TerminateJobObject(self.0.0, 1);
        }
    }
}

fn resume_initial_thread(pid: u32) -> Result<(), String> {
    // std::process exposes the child process handle, not its primary thread.
    // A freshly created suspended process has exactly its initial thread.
    // SAFETY: documented snapshot API; handle checked before use and uniquely closed.
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(format!(
            "Cannot enumerate initial thread: {}",
            io::Error::last_os_error()
        ));
    }
    let snapshot = Handle(raw);
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    // SAFETY: valid snapshot and properly initialized output structure.
    let mut found = unsafe { Thread32First(snapshot.0, &mut entry) };
    while found != 0 {
        if entry.th32OwnerProcessID == pid {
            // SAFETY: only request resume access to the suspended child's thread.
            let raw = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if raw.is_null() {
                return Err(format!(
                    "Cannot open initial thread: {}",
                    io::Error::last_os_error()
                ));
            }
            let thread = Handle(raw);
            // SAFETY: valid child thread, suspended during CREATE_SUSPENDED.
            if unsafe { ResumeThread(thread.0) } == u32::MAX {
                return Err(format!(
                    "Cannot resume initial thread: {}",
                    io::Error::last_os_error()
                ));
            }
            return Ok(());
        }
        // SAFETY: snapshot and output entry remain live.
        found = unsafe { Thread32Next(snapshot.0, &mut entry) };
    }
    Err("Suspended process has no discoverable initial thread.".into())
}
