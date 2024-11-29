use anyhow::Result;
use clang_tools_installer::{
    actions::Run,
    cli::{get_arg_parser, Cli},
    logger,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    logger::init()?;
    let parser = get_arg_parser();
    let parsed_cli = parser.get_matches();
    if parsed_cli.subcommand_matches("version").is_some() {
        println!("{VERSION}");
        return Ok(());
    }

    let cli = Cli::new(&parsed_cli)?;
    if let Some(cmd) = &cli.install {
        return cmd.execute(cli.directory);
    }
    if let Some(cmd) = &cli.uninstall {
        return cmd.execute(cli.directory);
    }
    Ok(())
}
