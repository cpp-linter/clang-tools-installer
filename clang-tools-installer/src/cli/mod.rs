mod structs;
use std::path::PathBuf;

use semver::Version;
pub use structs::Cli;

use clap::{builder::TypedValueParser, value_parser, Arg, ArgAction, Command, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecifiedVersion {
    Semantic(Version),
    Path(String),
}

#[derive(Debug, Clone, Copy)]
struct VersionParser;

impl TypedValueParser for VersionParser {
    type Value = SpecifiedVersion;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        let value = value.to_string_lossy();
        Ok(match lenient_semver::parse(&value) {
            Err(e) => {
                log::warn!(
                    "`{value}` is not a valid semantic version number. {}. Cannot {}",
                    e.to_string(),
                    cmd.get_name()
                );
                SpecifiedVersion::Path(value.to_string())
            }
            Ok(ver) => SpecifiedVersion::Semantic(ver),
        })
    }
}

/// Builds and returns the Command Line Interface's argument parsing object.
pub fn get_arg_parser() -> Command {
    // shared args among subcommands
    let version_arg = Arg::new("version")
        .long("version")
        .short('v')
        .value_parser(VersionParser)
        .required(true)
        .help("The version of the clang tool. This value shall be in the form of a semantic version (`x.y.z`, `x.y`, `x`).");
    let tools = Arg::new("tool")
        .action(ArgAction::Append)
        .default_values(["clang-format", "clang-tidy"])
        .help("The clang tool name to manage. This value can be a space-separated list for multiple clang tools.");

    // global arg
    let directory = Arg::new("directory")
        .long("directory")
        .short('d')
        .value_parser(value_parser!(PathBuf))
        .default_value(".")
        .help("The directory where the clang-tools are installed.");

    // subcommands
    let uninstall_cmd = Command::new("uninstall")
        .about("Uninstall a given version of specified clang tool(s).")
        .arg(tools.clone())
        .arg(version_arg.clone());
    let install_cmd = Command::new("install")
        .about("Install a given version of specified clang tool(s).")
        .arg(tools)
        .arg(Arg::new("force").long("overwrite").short('f').action(ArgAction::SetTrue).help("Force overwriting the symlink to the installed binary. This will only overwrite an existing symlink."))
        .arg(
            Arg::new("no-progress-bar")
                .long("no-progress-bar")
                .short('b')
                .action(ArgAction::SetTrue)
                .help("Do not display a progress bar for downloads."),
        )
        .arg(version_arg);
    let version_cmd = Command::new("version").about("Display the cpp-linter version and exit.");
    Command::new("clang-tools")
        .arg(directory)
        .subcommands([install_cmd, uninstall_cmd, version_cmd])
        .subcommand_required(true)
}
