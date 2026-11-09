use std::{
    ffi::OsString,
    io::{self, Read},
    path::Path,
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_CAPTURE: usize = 1024 * 1024;

#[derive(Debug)]
pub enum ProcessError {
    Unavailable,
    TimedOut,
    Cancelled,
}

pub struct ProcessOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub truncated: bool,
}

pub fn run_process(
    executable: &Path,
    args: &[OsString],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> Result<ProcessOutput, ProcessError> {
    run_process_streaming(executable, args, timeout, cancelled, |_| {})
}

pub fn run_process_streaming(
    executable: &Path,
    args: &[OsString],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
    on_stdout: impl FnMut(&[u8]) + Send,
) -> Result<ProcessOutput, ProcessError> {
    run_process_inner(executable, args, timeout, cancelled, on_stdout, MAX_CAPTURE)
}

pub fn run_process_with_limit(
    executable: &Path,
    args: &[OsString],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
    stdout_limit: usize,
) -> Result<ProcessOutput, ProcessError> {
    run_process_inner(executable, args, timeout, cancelled, |_| {}, stdout_limit)
}

fn run_process_inner(
    executable: &Path,
    args: &[OsString],
    timeout: Duration,
    cancelled: impl Fn() -> bool,
    on_stdout: impl FnMut(&[u8]) + Send,
    stdout_limit: usize,
) -> Result<ProcessOutput, ProcessError> {
    if !executable.is_absolute() {
        return Err(ProcessError::Unavailable);
    }
    if cancelled() {
        return Err(ProcessError::Cancelled);
    }
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|_| ProcessError::Unavailable)?;
    #[cfg(windows)]
    let tree = match windows_job::ProcessTree::attach(&child) {
        Ok(tree) => tree,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProcessError::Unavailable);
        }
    };
    let stdout = child.stdout.take().expect("stdout configured as a pipe");
    let stderr = child.stderr.take().expect("stderr configured as a pipe");
    thread::scope(|scope| {
        let stdout = scope.spawn(|| capture(stdout, on_stdout, stdout_limit));
        let stderr = scope.spawn(|| capture(stderr, |_| {}, MAX_CAPTURE));
        let started = Instant::now();
        let outcome = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => {}
                Err(_) => break Err(ProcessError::Unavailable),
            }
            if cancelled() {
                break Err(ProcessError::Cancelled);
            }
            if started.elapsed() >= timeout {
                break Err(ProcessError::TimedOut);
            }
            thread::sleep(Duration::from_millis(25));
        };
        #[cfg(windows)]
        drop(tree);
        if outcome.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let status = outcome?;
        let (stdout, stdout_truncated) = stdout
            .join()
            .map_err(|_| ProcessError::Unavailable)?
            .map_err(|_| ProcessError::Unavailable)?;
        let (stderr, stderr_truncated) = stderr
            .join()
            .map_err(|_| ProcessError::Unavailable)?
            .map_err(|_| ProcessError::Unavailable)?;
        Ok(ProcessOutput {
            status,
            stdout,
            stderr,
            truncated: stdout_truncated || stderr_truncated,
        })
    })
}

fn capture(
    mut pipe: impl Read,
    mut on_data: impl FnMut(&[u8]),
    limit: usize,
) -> io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    let mut truncated = false;
    loop {
        let read = pipe.read(&mut buffer)?;
        if read == 0 {
            return Ok((output, truncated));
        }
        on_data(&buffer[..read]);
        let remaining = (limit - output.len()).min(read);
        output.extend_from_slice(&buffer[..remaining]);
        truncated |= remaining < read;
    }
}

#[cfg(windows)]
mod windows_job {
    use std::{
        io, mem,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::Child,
        ptr,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub struct ProcessTree {
        _handle: OwnedHandle,
    }

    impl ProcessTree {
        pub fn attach(child: &Child) -> io::Result<Self> {
            // The unnamed handle is owned here and never inherited by the child.
            let raw = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let configured = unsafe {
                SetInformationJobObject(
                    raw,
                    JobObjectExtendedLimitInformation,
                    ptr::from_ref(&limits).cast(),
                    mem::size_of_val(&limits) as u32,
                )
            };
            if configured == 0
                || unsafe { AssignProcessToJobObject(raw, child.as_raw_handle()) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { _handle: handle })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "child fixture"]
    fn runner_child_fixture() {
        println!("stdout fixture");
        eprintln!("stderr fixture");
        thread::sleep(Duration::from_millis(300));
    }

    #[test]
    fn captures_both_streams_and_times_out() {
        let executable = std::env::current_exe().unwrap();
        let args = [
            "--exact",
            "process_runner::tests::runner_child_fixture",
            "--ignored",
            "--nocapture",
        ]
        .map(OsString::from);
        let output = run_process(&executable, &args, Duration::from_secs(5), || false).unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("stdout fixture"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("stderr fixture"));
        assert!(!output.truncated);
        assert!(matches!(
            run_process(&executable, &args, Duration::from_millis(50), || false),
            Err(ProcessError::TimedOut)
        ));
    }
}
