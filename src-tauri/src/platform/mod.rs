//! Platform Abstraction Layer (PAL).
//! Cross-platform abstractions for process isolation, binary discovery, trust stores, and hosts.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Setup a process command to ensure proper isolation and anti-zombie cleanup.
pub fn configure_process_isolation(_cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // process_group(0) sets the process group ID to the child's PID upon spawn.
        _cmd.process_group(0);
    }
}

/// Attach a spawned child process to the OS anti-zombie isolation mechanism.
pub fn attach_child_isolation(#[allow(unused_variables)] pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        // On Windows, attach child to job object
        attach_to_job_object(pid)?;
    }
    Ok(())
}

/// Terminate an entire process group gracefully (SIGTERM), falling back to SIGKILL.
pub fn terminate_process_tree(pid: u32, _force: bool) -> Result<(), String> {
    #[cfg(unix)]
    {
        let pgid = pid as i32;
        let signal = if _force { libc::SIGKILL } else { libc::SIGTERM };
        let ret = unsafe { libc::killpg(pgid, signal) };
        if ret != 0 {
            // Also try direct kill on pid if killpg failed
            let ret_direct = unsafe { libc::kill(pid as i32, signal) };
            if ret_direct != 0 {
                let err = std::io::Error::last_os_error();
                return Err(format!("Failed to signal process group {}: {}", pid, err));
            }
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        // On Windows, taskkill or TerminateProcess
        let status = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(format!("taskkill returned exit code {:?}", s.code())),
            Err(e) => Err(format!("Failed to execute taskkill: {}", e)),
        }
    }
}

#[cfg(windows)]
fn attach_to_job_object(pid: u32) -> Result<(), String> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return Err("Failed to create Win32 Job Object".to_string());
        }

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        let set_res = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );

        if set_res == 0 {
            CloseHandle(job);
            return Err("Failed to configure Job Object limits".to_string());
        }

        let process_handle: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        if process_handle.is_null() {
            CloseHandle(job);
            return Err(format!("Failed to open process {}", pid));
        }

        let assign_res = AssignProcessToJobObject(job, process_handle);
        CloseHandle(process_handle);

        if assign_res == 0 {
            CloseHandle(job);
            return Err(format!("Failed to assign process {} to Job Object", pid));
        }

        // Intentionally leak or store job handle so it stays open until BAMP exits
        std::mem::forget(job);
        Ok(())
    }
}

/// Find a service binary by inspecting portable `bin/` directory first,
/// falling back to system PATH.
pub fn resolve_binary(
    base_dir: &Path,
    service_id: &str,
    binary_names: &[&str],
) -> Option<PathBuf> {
    // 1. Check portable `bin/<service_id>/`
    let portable_dir = base_dir.join("bin").join(service_id);
    for name in binary_names {
        let candidate = portable_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let exe_name = format!("{}.exe", name);
            let candidate_exe = portable_dir.join(exe_name);
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }

    // Also check directly under `bin/`
    let bin_dir = base_dir.join("bin");
    for name in binary_names {
        let candidate = bin_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let exe_name = format!("{}.exe", name);
            let candidate_exe = bin_dir.join(exe_name);
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }

    // 2. Check system PATH
    for name in binary_names {
        if let Ok(path) = which::which(name) {
            return Some(path);
        }
    }

    None
}

mod which {
    use std::path::PathBuf;

    pub fn which(binary_name: &str) -> Result<PathBuf, ()> {
        let path_var = std::env::var_os("PATH").ok_or(())?;
        for dir in std::env::split_paths(&path_var) {
            let full_path = dir.join(binary_name);
            if full_path.is_file() {
                return Ok(full_path);
            }
            #[cfg(windows)]
            {
                let full_path_exe = dir.join(format!("{}.exe", binary_name));
                if full_path_exe.is_file() {
                    return Ok(full_path_exe);
                }
            }
        }
        Err(())
    }
}
