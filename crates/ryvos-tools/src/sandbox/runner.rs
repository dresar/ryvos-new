use std::path::Path;
use std::time::Duration;

use ryvos_core::error::{Result, RyvosError};
use ryvos_core::types::ToolResult;

use super::bubblewrap::BubblewrapSandbox;
use super::landlock::LandlockSandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    Bubblewrap,
    Landlock,
    Fallback,
}

pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute_in_sandbox(
    workspace: &Path,
    command: &str,
    timeout_secs: u64,
) -> Result<ToolResult> {
    let mode = if BubblewrapSandbox::is_available() {
        SandboxMode::Bubblewrap
    } else if LandlockSandbox::is_available() {
        SandboxMode::Landlock
    } else {
        SandboxMode::Fallback
    };

    let mut cmd = match mode {
        SandboxMode::Bubblewrap => {
            let mut c = BubblewrapSandbox::build_command(workspace, command);
            unsafe {
                c.pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
            c
        }
        SandboxMode::Landlock => LandlockSandbox::build_command(workspace, command),
        SandboxMode::Fallback => {
            let mut c = tokio::process::Command::new("bash");
            c.arg("-c").arg(command);
            c.current_dir(workspace);
            unsafe {
                c.pre_exec(|| {
                    libc::setpgid(0, 0);
                    let _ = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
                    Ok(())
                });
            }
            c
        }
    };

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.kill_on_drop(true);

    let child = cmd.spawn().map_err(|e| RyvosError::ToolExecution {
        tool: "bash".to_string(),
        message: format!("Failed to spawn command in {:?} sandbox: {}", mode, e),
    })?;

    let child_id = child.id().unwrap_or(0) as libc::pid_t;
    let timeout = Duration::from_secs(timeout_secs);

    let wait_res = tokio::time::timeout(timeout, child.wait_with_output()).await;

    match wait_res {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let mut content = String::new();
            if !stdout.is_empty() {
                content.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !content.is_empty() {
                    content.push('\n');
                }
                content.push_str("STDERR:\n");
                content.push_str(&stderr);
            }

            if content.len() > 30000 {
                content.truncate(30000);
                content.push_str("\n... (output truncated)");
            }
            if content.is_empty() {
                content = "(no output)".to_string();
            }

            if output.status.success() {
                Ok(ToolResult::success(content))
            } else {
                let code = output.status.code().unwrap_or(-1);
                Ok(ToolResult::error(format!("Exit code {}\n{}", code, content)))
            }
        }
        Ok(Err(e)) => Err(RyvosError::ToolExecution {
            tool: "bash".to_string(),
            message: e.to_string(),
        }),
        Err(_) => {
            if child_id > 0 {
                unsafe {
                    libc::kill(-child_id, libc::SIGKILL);
                }
            }
            Err(RyvosError::ToolTimeout {
                tool: "bash".to_string(),
                timeout_secs,
            })
        }
    }
}
