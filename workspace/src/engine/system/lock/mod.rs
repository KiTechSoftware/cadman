use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use veltrix::os::unistd::{self, Pid};

use crate::engine::{ErrorCode, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// RAII file lock for a named project or operation.
///
/// The lock file is placed at `<lock_dir>/locks/<safe_name>.lock` and stores the
/// PID of the holding process. On acquire, stale locks (dead PID) are automatically
/// removed. The lock is released when this struct is dropped.
pub struct ProjectLock {
    path: PathBuf,
}

impl ProjectLock {
    /// Acquire with the default 30 second timeout and 250 ms poll interval.
    pub fn acquire(lock_dir: &Path, name: &str) -> Result<Self> {
        Self::acquire_with_timeout(lock_dir, name, DEFAULT_TIMEOUT, DEFAULT_POLL_INTERVAL)
    }

    /// Acquire with a custom timeout and poll interval.
    pub fn acquire_with_timeout(
        lock_dir: &Path,
        name: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Self> {
        let safe_name = sanitize_lock_name(name);
        let path = lock_dir.join("locks").join(format!("{safe_name}.lock"));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                ErrorCode::StateLockFailed
                    .error()
                    .with_context("operation", "create_lock_dir")
                    .with_context("path", parent.display().to_string())
                    .with_context("error", err.to_string())
            })?;
        }

        let deadline = Instant::now() + timeout;
        loop {
            match try_acquire(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(AcquireError::LiveConflict(pid)) => {
                    if Instant::now() >= deadline {
                        return Err(ErrorCode::StateLockFailed
                            .error()
                            .with_context("reason", "timed out waiting for lock")
                            .with_context("held_by_pid", pid.to_string())
                            .with_context("lock", path.display().to_string()));
                    }
                    std::thread::sleep(poll_interval);
                }
                Err(AcquireError::Io(msg)) => {
                    return Err(ErrorCode::IoFailure
                        .error()
                        .with_context("operation", "acquire_lock")
                        .with_context("path", path.display().to_string())
                        .with_context("error", msg));
                }
            }
        }
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Sanitize a name for safe use as a lock file name.
/// Replaces non-alphanumeric characters (except `-` and `_`) with `_`, lowercased.
pub fn sanitize_lock_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_ascii_lowercase()
}

// ── Internal ─────────────────────────────────────────────────────────────────

enum AcquireError {
    LiveConflict(u32),
    Io(String),
}

/// Attempt a single atomic lock acquisition using O_CREAT|O_EXCL.
/// If a stale lock is found (dead PID), it is removed and acquisition retried once.
fn try_acquire(path: &Path) -> std::result::Result<(), AcquireError> {
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path);

    match result {
        Ok(mut file) => {
            write_lock_pid(&mut file, path)?;
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            match read_lock_pid(path) {
                Some(pid) if pid_is_alive(pid) => Err(AcquireError::LiveConflict(pid)),
                _ => {
                    // Stale, unreadable, or corrupt lock – remove and retry once
                    let _ = fs::remove_file(path);
                    let result2 = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o600)
                        .open(path);

                    match result2 {
                        Ok(mut file) => {
                            write_lock_pid(&mut file, path)?;
                            Ok(())
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            // Another process acquired it between our remove and retry
                            match read_lock_pid(path) {
                                Some(pid) => Err(AcquireError::LiveConflict(pid)),
                                None => Err(AcquireError::Io("lock contention".to_string())),
                            }
                        }
                        Err(e) => Err(AcquireError::Io(e.to_string())),
                    }
                }
            }
        }
        Err(err) => Err(AcquireError::Io(err.to_string())),
    }
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

/// Check if a PID corresponds to a live process using `kill(pid, 0)`.
fn pid_is_alive(pid: u32) -> bool {
    unistd::pid_is_alive(Pid::from_raw(pid as i32))
}

fn write_lock_pid(file: &mut fs::File, path: &Path) -> std::result::Result<(), AcquireError> {
    let pid = std::process::id();

    write!(file, "{pid}").map_err(|err| {
        let _ = fs::remove_file(path);
        AcquireError::Io(format!("failed to write lock pid: {err}"))
    })?;

    file.sync_all().map_err(|err| {
        let _ = fs::remove_file(path);
        AcquireError::Io(format!("failed to sync lock pid: {err}"))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(suffix: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("cadman_lock_{}_{}", std::process::id(), suffix));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_acquire_and_release() {
        let dir = test_dir("acquire");
        let lock_path = dir.join("locks").join("my-project.lock");

        let lock = ProjectLock::acquire(&dir, "my-project").unwrap();
        assert!(lock_path.exists());
        drop(lock);
        assert!(!lock_path.exists());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_stale_lock_is_cleared() {
        let dir = test_dir("stale");
        let lock_dir = dir.join("locks");
        fs::create_dir_all(&lock_dir).unwrap();
        // PID 9999999 cannot exist on Linux
        fs::write(lock_dir.join("stale.lock"), "9999999").unwrap();

        let lock = ProjectLock::acquire(&dir, "stale").unwrap();
        drop(lock);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_live_pid_conflict_times_out() {
        let dir = test_dir("live");
        let lock_dir = dir.join("locks");
        fs::create_dir_all(&lock_dir).unwrap();
        // Own PID is definitely alive
        fs::write(lock_dir.join("live.lock"), std::process::id().to_string()).unwrap();

        let result = ProjectLock::acquire_with_timeout(
            &dir,
            "live",
            Duration::from_millis(100),
            Duration::from_millis(10),
        );
        assert!(result.is_err());

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_sanitize_lock_name() {
        assert_eq!(sanitize_lock_name("my-project"), "my-project");
        assert_eq!(sanitize_lock_name("My Project"), "my_project");
        assert_eq!(sanitize_lock_name("test_123"), "test_123");
        assert_eq!(sanitize_lock_name("foo.bar/baz"), "foo_bar_baz");
        assert_eq!(sanitize_lock_name("UPPER"), "upper");
    }
}
