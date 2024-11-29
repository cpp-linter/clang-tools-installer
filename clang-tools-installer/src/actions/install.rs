use std::path::PathBuf;

use super::Run;
use crate::cli::SpecifiedVersion;
use anyhow::Result;

pub struct InstallCommand {
    pub tool: Vec<String>,
    pub version: SpecifiedVersion,
    pub force: bool,
}

impl Run for InstallCommand {
    fn execute(&self, directory: PathBuf) -> Result<()> {
        let version = match &self.version {
            SpecifiedVersion::Path(_) => return Ok(()),
            SpecifiedVersion::Semantic(ver) => ver,
        };
        let directory = Self::check_directory(directory)?;
        log::info!(
            "installing {:?} v{} to {directory:?}",
            self.tool,
            version.to_string(),
        );

        for clang_tool in &self.tool {
            if let Some(installed_path) = Self::is_installed(clang_tool, version) {
                log::debug!(
                    "Found {clang_tool} v{} at {installed_path:?}",
                    version.major
                );
            } else {
                log::info!("Found no installed {clang_tool} v{}", version.major);
            }
        }
        Ok(())
    }
}
