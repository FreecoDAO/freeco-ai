//! Spawning child processes without flashing a console window.
//!
//! On Windows, a GUI application that spawns a console program gets a console
//! window for it. FreEco.ai probes `docker`, `nvidia-smi`, `powershell` and
//! others while rendering ordinary screens, so opening a tab in the dashboard
//! made black windows pop up and vanish. It looks like malware, it steals
//! focus mid-typing, and there is no way for a user to tell it apart from
//! something actually going wrong.
//!
//! `CREATE_NO_WINDOW` fixes it. The flag has no effect on other platforms, so
//! this wrapper is unconditional at the call site and conditional inside -
//! which is the only way it stays applied. A rule that says "remember to add
//! the flag" gets followed for a while and then does not: of 67 spawn sites in
//! this workspace, 2 had it.

/// Create a `std::process::Command` that will not open a console window.
///
/// Use in place of `std::process::Command::new` for anything the user did not
/// explicitly ask to see.
pub fn quiet(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    apply_no_window(&mut cmd);
    cmd
}

/// Create a `tokio::process::Command` that will not open a console window.
pub fn quiet_async(program: impl AsRef<std::ffi::OsStr>) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    apply_no_window_async(&mut cmd);
    cmd
}

/// 0x08000000 - CREATE_NO_WINDOW. Named here rather than pulled from winapi so
/// this module needs no extra dependency and compiles identically everywhere.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn apply_no_window(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn apply_no_window(_cmd: &mut std::process::Command) {}

#[cfg(windows)]
fn apply_no_window_async(cmd: &mut tokio::process::Command) {
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn apply_no_window_async(_cmd: &mut tokio::process::Command) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wrapper must still produce a working command. A quiet spawn that
    /// does not run is a worse bug than a visible one.
    #[tokio::test]
    async fn a_quiet_command_still_runs() {
        let program = if cfg!(windows) { "cmd" } else { "sh" };
        let args: &[&str] = if cfg!(windows) {
            &["/C", "echo quiet-ok"]
        } else {
            &["-c", "echo quiet-ok"]
        };
        let out = quiet_async(program)
            .args(args)
            .output()
            .await
            .expect("the wrapped command should still execute");
        assert!(String::from_utf8_lossy(&out.stdout).contains("quiet-ok"));
    }

    /// The sync form too - several probes are blocking.
    #[test]
    fn the_sync_form_also_runs() {
        let program = if cfg!(windows) { "cmd" } else { "sh" };
        let args: &[&str] = if cfg!(windows) {
            &["/C", "echo sync-ok"]
        } else {
            &["-c", "echo sync-ok"]
        };
        let out = quiet(program).args(args).output().expect("should execute");
        assert!(String::from_utf8_lossy(&out.stdout).contains("sync-ok"));
    }
}
