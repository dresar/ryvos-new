use std::path::Path;

use tokio::process::Command;

pub struct LandlockSandbox;

impl LandlockSandbox {
    pub fn is_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                let res = libc::syscall(444, std::ptr::null::<libc::c_void>(), 0, 0);
                res >= 0 || *libc::__errno_location() != libc::ENOSYS
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    pub fn apply_landlock_and_seccomp(workspace: &Path) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                let ret = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
                if ret != 0 {
                    return Err(std::io::Error::last_os_error());
                }

                #[repr(C)]
                struct LandlockRulesetAttr {
                    handled_access_fs: u64,
                }

                #[repr(C)]
                struct LandlockPathBeneathAttr {
                    allowed_access: u64,
                    parent_fd: i32,
                }

                let access_all: u64 = (1 << 13) - 1;
                let access_ro: u64 = (1 << 0) | (1 << 2) | (1 << 3);

                let attr = LandlockRulesetAttr {
                    handled_access_fs: access_all,
                };

                let ruleset_fd = libc::syscall(
                    444,
                    &attr as *const LandlockRulesetAttr as *const libc::c_void,
                    std::mem::size_of::<LandlockRulesetAttr>(),
                    0,
                ) as libc::c_int;

                if ruleset_fd >= 0 {
                    let ro_paths: [&[u8]; 6] = [
                        b"/\0".as_ref(),
                        b"/usr\0".as_ref(),
                        b"/bin\0".as_ref(),
                        b"/lib\0".as_ref(),
                        b"/lib64\0".as_ref(),
                        b"/etc\0".as_ref(),
                    ];
                    for path in &ro_paths {
                        let fd = libc::open(
                            path.as_ptr() as *const libc::c_char,
                            libc::O_PATH | libc::O_CLOEXEC,
                        );
                        if fd >= 0 {
                            let rule = LandlockPathBeneathAttr {
                                allowed_access: access_ro,
                                parent_fd: fd,
                            };
                            libc::syscall(
                                445,
                                ruleset_fd,
                                1,
                                &rule as *const LandlockPathBeneathAttr as *const libc::c_void,
                                0,
                            );
                            libc::close(fd);
                        }
                    }

                    let rw_paths = [b"/tmp\0", b"/dev\0"];
                    for path in &rw_paths {
                        let fd = libc::open(
                            path.as_ptr() as *const libc::c_char,
                            libc::O_PATH | libc::O_CLOEXEC,
                        );
                        if fd >= 0 {
                            let rule = LandlockPathBeneathAttr {
                                allowed_access: access_all,
                                parent_fd: fd,
                            };
                            libc::syscall(
                                445,
                                ruleset_fd,
                                1,
                                &rule as *const LandlockPathBeneathAttr as *const libc::c_void,
                                0,
                            );
                            libc::close(fd);
                        }
                    }

                    if let Ok(ws_str) =
                        std::ffi::CString::new(workspace.to_string_lossy().as_bytes())
                    {
                        let ws_fd = libc::open(ws_str.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
                        if ws_fd >= 0 {
                            let rule = LandlockPathBeneathAttr {
                                allowed_access: access_all,
                                parent_fd: ws_fd,
                            };
                            libc::syscall(
                                445,
                                ruleset_fd,
                                1,
                                &rule as *const LandlockPathBeneathAttr as *const libc::c_void,
                                0,
                            );
                            libc::close(ws_fd);
                        }
                    }

                    libc::syscall(446, ruleset_fd, 0);
                    libc::close(ruleset_fd);
                }
            }
        }
        let _ = workspace;
        Ok(())
    }

    pub fn build_command(workspace: &Path, command: &str) -> Command {
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(command);
        cmd.current_dir(workspace);
        let ws_path = workspace.to_path_buf();

        unsafe {
            cmd.pre_exec(move || {
                libc::setpgid(0, 0);
                let _ = LandlockSandbox::apply_landlock_and_seccomp(&ws_path);
                Ok(())
            });
        }

        cmd
    }
}
