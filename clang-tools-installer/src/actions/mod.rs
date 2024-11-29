mod install;
mod uninstall;

use std::{env, path::PathBuf, process::Command};

use anyhow::{anyhow, Result};
pub use install::InstallCommand;
use regex::Regex;
use semver::Version;
pub use uninstall::UninstallCommand;
use which::which;

pub trait Run {
    fn execute(&self, directory: PathBuf) -> Result<()>;
    fn check_directory(directory: PathBuf) -> Result<PathBuf> {
        if !directory.exists() {
            return Err(anyhow!("{directory:?} does not exist"));
        }
        let directory = directory.canonicalize()?;
        if !env::var("PATH")?.contains(directory.to_string_lossy().to_string().as_str()) {
            log::warn!("{directory:?} is not in your environment variable PATH.");
        }
        Ok(directory)
    }

    fn is_installed(clang_tool: &String, version: &Version) -> Option<PathBuf> {
        let version_pattern = Regex::new(r"(?i)version\s*([\d.]+)").unwrap();
        if let Ok(path) = which(format!("{clang_tool}-{}", version.major).as_str()) {
            let mut cmd = Command::new(path.as_os_str());
            cmd.arg("--version");
            log::debug!(
                "Running \"{} {}\"",
                &path.to_string_lossy().to_string().as_str(),
                cmd.get_args()
                    .map(|a| a.to_str().unwrap())
                    .collect::<Vec<&str>>()
                    .join(" "),
            );

            if let Ok(cmd_out) = cmd.output() {
                let out = String::from_utf8_lossy(&cmd_out.stdout);
                if let Some(ver_match) = version_pattern.captures(&out) {
                    let vp = Version::parse(ver_match.get(1).unwrap().as_str());
                    if vp.is_ok_and(|v| {
                        log::info!("Detected {clang_tool} v{} at {path:?}", v.to_string());
                        v.major == version.major
                    }) {
                        return Some(path);
                    }
                }
            }
        }
        if let Ok(exe_path) = which(clang_tool.as_str()) {
            let mut sys_cmd = Command::new(exe_path.as_os_str());
            sys_cmd.arg("--version");
            if let Ok(sys_cmd_out) = sys_cmd.output() {
                let out = String::from_utf8_lossy(&sys_cmd_out.stdout);
                if let Some(ver_match) = version_pattern.captures(&out) {
                    let vp = Version::parse(ver_match.get(1).unwrap().as_str());
                    if vp.is_ok_and(|v| {
                        log::info!("Detected  {clang_tool} v{} at {exe_path:?}", v.to_string());
                        v.major == version.major
                    }) {
                        return Some(exe_path);
                    }
                }
            }
        }
        None
    }
}
