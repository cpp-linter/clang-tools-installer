use std::path::PathBuf;

use super::SpecifiedVersion;
use anyhow::Result;
use clap::ArgMatches;

use crate::actions::{InstallCommand, UninstallCommand};

pub struct Cli {
    pub install: Option<InstallCommand>,
    pub uninstall: Option<UninstallCommand>,
    pub directory: PathBuf,
}

impl Cli {
    pub fn new(args: &ArgMatches) -> Result<Self> {
        let directory = args
            .get_one::<PathBuf>("directory")
            .map(|v| v.to_owned())
            .unwrap();
        let install = args
            .subcommand_matches("install")
            .map(|matches| InstallCommand {
                tool: matches
                    .get_many::<String>("tool")
                    .expect("`tool` arg is required")
                    .map(|v| v.to_owned())
                    .collect::<Vec<String>>(),
                version: matches
                    .get_one::<SpecifiedVersion>("version")
                    .expect("`version` arg is required")
                    .to_owned(),
                force: matches.get_flag("force"),
            });
        let uninstall = args
            .subcommand_matches("uninstall")
            .map(|matches| UninstallCommand {
                tool: matches
                    .get_many::<String>("tool")
                    .expect("`tool` arg is required")
                    .map(|v| v.to_owned())
                    .collect::<Vec<String>>(),
                version: matches
                    .get_one::<SpecifiedVersion>("version")
                    .expect("`version` arg is required")
                    .to_owned(),
            });
        Ok(Self {
            install,
            uninstall,
            directory,
        })
    }
}
