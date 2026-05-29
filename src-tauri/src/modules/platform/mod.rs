/// Defines the contract for OS-specific operations that VoltEnv needs.
///
/// Each supported platform provides its own implementation
/// (`WindowsAdapter`, `UnixAdapter`). Callers use the `Platform` type
/// alias to invoke the correct implementation at compile time.
pub trait PlatformAdapter {
    /// Terminates the process identified by `pid`.
    ///
    /// - `force = false`: soft kill  (SIGTERM / `taskkill /PID`)
    /// - `force = true`:  force kill (SIGKILL / `taskkill /F /T`)
    async fn kill_process(pid: u32, force: bool) -> Result<(), String>;

    /// Returns `true` if a process with the given `pid` is still running.
    ///
    /// This is a *best-effort* check — a process that exits between the
    /// call and the return may produce a stale `true`.
    async fn is_process_alive(pid: u32) -> bool;
}

#[cfg(not(target_os = "windows"))]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;

/// Compile-time alias to the active platform adapter.
///
/// - `cfg!(target_os = "windows")` → `WindowsAdapter`
/// - otherwise → `UnixAdapter`
#[cfg(target_os = "windows")]
pub type Platform = crate::modules::platform::windows::WindowsAdapter;

#[cfg(not(target_os = "windows"))]
pub type Platform = crate::modules::platform::unix::UnixAdapter;
