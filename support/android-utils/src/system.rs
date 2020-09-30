//! Various system level utilities.

use crate::{AndroidProperties, PropertyGetter};
use libc::{pid_t, sysconf};
use log::{debug, error};
use procfs::process::Process;
use std::fs;
#[cfg(target_os = "android")]
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

const CONTROL_PATH: &str = "/sys/class/leds/sublcd-backlight/brightness";

// Iterate through processes information and return the full list and
// the Nuwa process p_id.
fn find_process_pid(process_prefix: &str) -> Option<pid_t> {
    fs::read_dir("/proc/")
        .expect("Can't read /proc/")
        .find_map(|entry| {
            if let Ok(pid) = entry.ok()?.file_name().to_str()?.parse::<pid_t>() {
                if let Ok(procinfo) = Process::new(pid) {
                    if procinfo
                        .exe()
                        .unwrap_or_else(|_| PathBuf::new())
                        .starts_with(process_prefix)
                    {
                        return Some(pid);
                    }
                }
            }
            None
        })
}

// Returns whether we've been able to modify the value.
pub fn adjust_process_oom_score(process_prefix: &str, score: i32) -> bool {
    if let Some(pid) = find_process_pid(process_prefix) {
        // Open /proc/$pid/oom_score_adj and write the new value.
        match fs::OpenOptions::new()
            .write(true)
            .open(format!("/proc/{}/oom_score_adj", pid))
        {
            Ok(mut file) => match file.write_fmt(format_args!("{}", score)) {
                Ok(_) => return true,
                Err(err) => error!("Failed to write in /proc/{}/oom_score_adj : {}", pid, err),
            },
            Err(err) => error!("Failed to open /proc/{}/oom_score_adj : {}", pid, err),
        }
    }

    false
}

// Kills the first process found that starts with this prefix.
// Returns true if successful.
pub fn kill_process(process_prefix: &str) -> bool {
    if let Some(pid) = find_process_pid(process_prefix) {
        let res = unsafe { libc::kill(pid, libc::SIGKILL) };
        return res == 0;
    }

    false
}

// Returns the amount of memory in MB
pub fn total_memory() -> libc::c_long {
    unsafe { sysconf(libc::_SC_PHYS_PAGES) * sysconf(libc::_SC_PAGE_SIZE) / (1024 * 1024) }
}

// Opaque struct used to save and restore changes to the system state.

#[cfg(target_os = "android")]
static MINFREE_PATH: &str = "/sys/module/lowmemorykiller/parameters/minfree";

#[derive(Debug)]
pub struct SystemState {
    minfree: Option<String>, // The saved value of the MINFREE_PATH file.
}

impl Default for SystemState {
    fn default() -> Self {
        debug!("Total usable memory is {}M", total_memory());
        SystemState { minfree: None }
    }
}

impl SystemState {
    #[cfg(target_os = "android")]
    fn service_action(action: &str, service: &str) -> bool {
        debug!("Running `{} {}`", action, service);
        let status = ::std::process::Command::new(action)
            .arg(service)
            .status()
            .unwrap();
        if !status.success() {
            error!("Failed to run `{} {}`", action, service);
        }
        status.success()
    }

    #[cfg(target_os = "android")]
    fn stop_updater_daemon(&self) {
        debug!("Stopping updater daemon");
        let _ = Self::service_action("stop", "updater-daemon");
    }

    #[cfg(target_os = "android")]
    fn start_updater_daemon(&self) {
        debug!("Starting updater daemon");
        let _ = Self::service_action("start", "updater-daemon");
    }

    #[cfg(target_os = "android")]
    pub fn enter_high_priority(&mut self) {
        debug!("Entering high priority mode");
        if total_memory() <= 256 {
            self.lower_min_free();
            self.stop_updater_daemon();
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn enter_high_priority(&mut self) {}

    #[cfg(target_os = "android")]
    pub fn leave_high_priority(&mut self) {
        debug!("Leaving high priority mode");
        if total_memory() <= 256 {
            self.restore_minfree();
            self.start_updater_daemon();
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn leave_high_priority(&mut self) {}

    // Changes the minfree parameter to give a little more space to the current foreground app.
    #[cfg(target_os = "android")]
    fn lower_min_free(&mut self) {
        if self.minfree.is_some() {
            // We are already in this mode, bail out.
            return;
        }

        // Read the current value in /sys/module/lowmemorykiller/parameters/minfree
        match fs::File::open(MINFREE_PATH) {
            Ok(mut file) => {
                let mut content = String::new();
                match file.read_to_string(&mut content) {
                    Ok(_) => {
                        // The content is a comma delimited string such as: 2560,3328,3584,4608,5120
                        // Each number is mapped to an adj score level    : 0,67,134,400,667
                        let mut minkb: Vec<u32> = content
                            .split(',')
                            .map(|item| item.parse::<u32>().unwrap_or(5120))
                            .collect();
                        if minkb.len() != 5 {
                            error!("Expected 5 minfree value, got {}", content);
                            return;
                        }

                        //  Amount are in pages.
                        //  oom_score_adj min_free
                        //              0  4096 KB
                        //             67  5120 KB
                        //            134  6144 KB
                        //            400  8192 KB
                        //            667 20480 KB
                        let mut page_size: u32 = 4;
                        let system_page_size =
                            unsafe { sysconf(libc::_SC_PAGE_SIZE) / 1024 } as u32;
                        if system_page_size > 0 {
                            page_size = system_page_size;
                        }
                        minkb[0] = 4096 / page_size;
                        minkb[1] = 5120 / page_size;
                        minkb[2] = 6144 / page_size;
                        minkb[3] = 8192 / page_size;

                        if self.write_minfree(&format!(
                            "{},{},{},{},{}",
                            minkb[0], minkb[1], minkb[2], minkb[3], minkb[4]
                        )) {
                            self.minfree = Some(content);
                        }
                    }
                    Err(err) => error!("Failed to read {} : {}", MINFREE_PATH, err),
                }
            }
            Err(err) => error!("Failed to open {} : {}", MINFREE_PATH, err),
        }
    }

    #[cfg(target_os = "android")]
    fn write_minfree(&self, content: &str) -> bool {
        match fs::OpenOptions::new().write(true).open(MINFREE_PATH) {
            Ok(mut file) => match file.write_all(content.as_bytes()) {
                Ok(_) => true,
                Err(err) => {
                    error!("Failed to write in {} : {}", MINFREE_PATH, err);
                    false
                }
            },
            Err(err) => {
                error!("Failed to open {} : {}", MINFREE_PATH, err);
                false
            }
        }
    }

    #[cfg(target_os = "android")]
    fn restore_minfree(&mut self) {
        if let Some(minfree) = &self.minfree {
            if self.write_minfree(&minfree) {
                self.minfree = None;
            }
        }
    }
}

impl Drop for SystemState {
    fn drop(&mut self) {
        self.leave_high_priority();
    }
}

fn write_sys_file(file_name: &str, data: &str) -> bool {
    match fs::OpenOptions::new().write(true).open(file_name) {
        Ok(mut file) => match file.write_all(data.as_bytes()) {
            Ok(_) => return true,
            Err(err) => error!("Failed to write {} in {} : {}", data, file_name, err),
        },
        Err(err) => error!("Failed to write {} in {} : {}", data, file_name, err),
    }
    false
}

pub fn update_cpu_sleep_state(allow: bool) {
    let file = if allow {
        "/sys/power/wake_unlock"
    } else {
        "/sys/power/wake_lock"
    };

    write_sys_file(file, "api-daemon");
}

pub fn set_ext_screen_brightness(brightness: u32) -> bool {
    let control_path = AndroidProperties::get("screen.secondary.brightness", CONTROL_PATH)
        .unwrap_or_else(|_| CONTROL_PATH.into());
    write_sys_file(&control_path, brightness.to_string().as_str())
}
