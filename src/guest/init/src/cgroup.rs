//! Per-container memory cgroup (cgroup v2) for OOM accounting.
//!
//! The CRI `LinuxContainerResources.memory_limit_in_bytes` is enforced inside
//! the guest by placing the container process in its own cgroup v2 memory
//! cgroup with `memory.max` set to the limit. When the workload (or the page
//! cache / tmpfs it touches) exceeds the limit, the kernel OOM-killer reaps the
//! cgroup; `memory.events`'s `oom_kill` counter then lets us report the
//! container exit reason as `OOMKilled` — matching how runc/containerd behave.
//!
//! This is Linux-only and entirely best-effort: any failure (cgroup v2 absent,
//! permission denied, controller unavailable) degrades to "no enforcement, no
//! OOM detection" rather than failing the container launch.

#![cfg(target_os = "linux")]

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::{debug, warn};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
static CGROUP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Ensure cgroup v2 is mounted at `/sys/fs/cgroup` with the `memory` controller
/// delegated to child cgroups. Idempotent; returns `false` if cgroup v2 (with a
/// usable memory controller) could not be made available.
fn ensure_cgroup2_ready() -> bool {
    let controllers_path = format!("{CGROUP_ROOT}/cgroup.controllers");
    if std::fs::metadata(&controllers_path).is_err() {
        // Not mounted yet — mount the unified hierarchy.
        use nix::mount::{mount, MsFlags};
        let _ = std::fs::create_dir_all(CGROUP_ROOT);
        if let Err(error) = mount(
            Some("cgroup2"),
            CGROUP_ROOT,
            Some("cgroup2"),
            MsFlags::empty(),
            None::<&str>,
        ) {
            warn!(error = %error, "cgroup: failed to mount cgroup2");
            return false;
        }
    }

    // The memory controller must be listed as available and delegated to
    // children via the root's subtree_control (the root cgroup is exempt from
    // the no-internal-processes rule, so enabling it here is allowed).
    match std::fs::read_to_string(&controllers_path) {
        Ok(controllers) if controllers.split_whitespace().any(|c| c == "memory") => {
            let subtree = format!("{CGROUP_ROOT}/cgroup.subtree_control");
            if let Ok(current) = std::fs::read_to_string(&subtree) {
                if !current.split_whitespace().any(|c| c == "memory") {
                    if let Err(error) = write_cgroup_file(&subtree, "+memory") {
                        warn!(error = %error, "cgroup: failed to delegate memory controller");
                        return false;
                    }
                }
            }
            true
        }
        Ok(_) => {
            warn!("cgroup: memory controller not available in cgroup.controllers");
            false
        }
        Err(error) => {
            warn!(error = %error, "cgroup: cannot read cgroup.controllers");
            false
        }
    }
}

fn write_cgroup_file(path: &str, value: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.write_all(value.as_bytes())
}

/// A per-container memory cgroup. Dropping it removes the cgroup directory.
pub struct MemoryCgroup {
    path: String,
}

impl MemoryCgroup {
    /// Create a memory cgroup with `memory.max = limit_bytes`. Returns `None`
    /// when cgroup v2 / the memory controller is unavailable, in which case the
    /// caller proceeds without enforcement (and without OOM detection).
    pub fn create(limit_bytes: u64) -> Option<Self> {
        if limit_bytes == 0 || !ensure_cgroup2_ready() {
            return None;
        }
        let seq = CGROUP_SEQ.fetch_add(1, Ordering::Relaxed);
        let path = format!("{CGROUP_ROOT}/box-mem-{}-{}", std::process::id(), seq);
        if let Err(error) = std::fs::create_dir(&path) {
            warn!(error = %error, path, "cgroup: failed to create memory cgroup");
            return None;
        }
        if let Err(error) =
            write_cgroup_file(&format!("{path}/memory.max"), &limit_bytes.to_string())
        {
            warn!(error = %error, "cgroup: failed to set memory.max");
            let _ = std::fs::remove_dir(&path);
            return None;
        }
        // Kill the whole cgroup on OOM so the container's main process dies even
        // if the over-allocating process was a child (best-effort).
        let _ = write_cgroup_file(&format!("{path}/memory.oom.group"), "1");
        debug!(path, limit_bytes, "cgroup: created memory cgroup");
        Some(Self { path })
    }

    /// Move a process into this cgroup.
    pub fn add_pid(&self, pid: u32) {
        if let Err(error) =
            write_cgroup_file(&format!("{}/cgroup.procs", self.path), &pid.to_string())
        {
            warn!(error = %error, pid, "cgroup: failed to add process to memory cgroup");
        }
    }

    /// Number of OOM kills recorded in this cgroup (`memory.events` `oom_kill`).
    /// A non-zero value means the container was OOM-killed.
    pub fn oom_kills(&self) -> u64 {
        let events = match std::fs::read_to_string(format!("{}/memory.events", self.path)) {
            Ok(events) => events,
            Err(_) => return 0,
        };
        events
            .lines()
            .find_map(|line| line.strip_prefix("oom_kill "))
            .and_then(|count| count.trim().parse::<u64>().ok())
            .unwrap_or(0)
    }
}

impl Drop for MemoryCgroup {
    fn drop(&mut self) {
        // Safe only once the cgroup is empty (the container has been reaped).
        if let Err(error) = std::fs::remove_dir(&self.path) {
            debug!(error = %error, path = %self.path, "cgroup: cleanup rmdir failed");
        }
    }
}
