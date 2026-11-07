use std::path::Path;

use tokio::process::Command;

pub struct BubblewrapSandbox;

impl BubblewrapSandbox {
    pub fn is_available() -> bool {
        std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn build_command(workspace: &Path, command: &str) -> Command {
        let mut cmd = Command::new("bwrap");

        cmd.arg("--ro-bind").arg("/usr").arg("/usr");

        if Path::new("/lib").exists() {
            cmd.arg("--ro-bind").arg("/lib").arg("/lib");
        }
        if Path::new("/lib64").exists() {
            cmd.arg("--ro-bind").arg("/lib64").arg("/lib64");
        }
        if Path::new("/bin").exists() {
            cmd.arg("--ro-bind").arg("/bin").arg("/bin");
        }
        if Path::new("/etc/resolv.conf").exists() {
            cmd.arg("--ro-bind")
                .arg("/etc/resolv.conf")
                .arg("/etc/resolv.conf");
        }
        if Path::new("/etc/ssl").exists() {
            cmd.arg("--ro-bind").arg("/etc/ssl").arg("/etc/ssl");
        }

        cmd.arg("--bind").arg(workspace).arg(workspace);
        cmd.arg("--chdir").arg(workspace);
        cmd.arg("--unshare-net");
        cmd.arg("--unshare-pid");
        cmd.arg("--die-with-parent");
        cmd.arg("--clearenv");
        cmd.arg("--setenv")
            .arg("PATH")
            .arg("/usr/bin:/bin:/usr/local/bin");
        cmd.arg("bash").arg("-c").arg(command);

        cmd
    }
}
