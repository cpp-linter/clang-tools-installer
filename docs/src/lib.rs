//! This exposes a function in Python, so an mkdocs plugin can use it to generate the CLI document.
//! For actual library/binary source code look in cpp-linter folder.
use clang_tools_installer::cli;
use clap::{Arg, ArgAction, ArgGroup, Command};
use pyo3::{exceptions::PyValueError, prelude::*};
use std::collections::HashMap;

// const GROUPS_ORDER: [&str; 5] = [
//     "General options",
//     "Source options",
//     "Clang-format options",
//     "Clang-tidy options",
//     "Feedback options",
// ];
const GROUPS_ORDER: [&str; 0] = [];

fn inject_metadata(
    metadata: &HashMap<String, HashMap<String, Py<PyAny>>>,
    out: &mut String,
    name: &str,
) {
    if let Some(map) = metadata.get(name) {
        if let Some(val) = map.get("minimum-version") {
            out.push_str(format!("<!-- md:version {} -->\n", val).as_str());
        }
        if let Some(val) = map.get("required-permission") {
            out.push_str(format!("<!-- md:permission {} -->\n", val).as_str());
        }
        if map.contains_key("experimental") {
            out.push_str("<!-- md:flag experimental -->\n");
        }
    }
}

fn generate_option(
    metadata: &HashMap<String, HashMap<String, Py<PyAny>>>,
    arg: &Arg,
    out: &mut String,
    lvl_pad: &str,
) {
    let long_name = arg.get_long().unwrap_or_default();
    let short_name = arg.get_short();

    let mut arg_summary = vec![];
    if let Some(short) = short_name.to_owned() {
        arg_summary.push(format!("-{short}"));
    }
    if !long_name.is_empty() {
        arg_summary.push(format!("--{long_name}"));
    }
    if !arg_summary.is_empty() {
        out.push_str(
            format!(
                "\n{lvl_pad}### `{}` {}\n\n",
                arg_summary.join(", "),
                if matches!(
                    &arg.get_action(),
                    ArgAction::SetFalse | ArgAction::SetTrue | ArgAction::Count
                ) {
                    ":material-flag:{ .flag title=\"Takes no value.\" }"
                } else {
                    if arg.is_required_set() {
                        ":material-check-decagram-outline:{ .opt-required title=\"Required\" }"
                    } else {
                        ""
                    }
                }
            )
            .as_str(),
        );
    }

    if !long_name.is_empty() || short_name.is_some() {
        if let Some(short) = short_name.map(|c| format!("-{c}")) {
            inject_metadata(metadata, out, &short);
        }
        inject_metadata(metadata, out, format!("--{long_name}").as_str());
    }
    let default = arg.get_default_values();
    if default.is_empty() {
        out.push('\n');
    } else {
        let defaults: Vec<&str> = default.into_iter().filter_map(|v| v.to_str()).collect();
        let delimiter = arg.get_value_delimiter().unwrap_or(' ');
        out.push_str(
            format!(
                "<!-- md:default {} -->\n\n",
                defaults.join(delimiter.to_string().as_str())
            )
            .as_str(),
        );
    }
    if let Some(help) = &arg.get_help() {
        out.push_str(format!("{}\n", help.to_string().trim()).as_str());
    }
}

fn generate_cmd_args(
    metadata: &HashMap<String, HashMap<String, Py<PyAny>>>,
    command: &mut Command,
    out: &mut String,
    level: u8,
) -> PyResult<()> {
    let mut lvl_pad = String::new();
    for _ in 0..level as usize {
        lvl_pad.push('#');
    }
    out.push_str(format!("{lvl_pad}## Arguments\n").as_str());
    for arg in command.get_positionals() {
        out.push_str(
            format!(
                "\n{lvl_pad}### `{}{}` {}\n\n",
                arg.get_value_names()
                    .map(|v| v.iter().map(|m| m.to_string()).collect::<Vec<String>>())
                    .unwrap_or(vec![arg.get_id().to_string()])
                    .join(
                        &arg.get_value_delimiter()
                            .map(|v| v.to_string())
                            .unwrap_or("".to_string())
                    ),
                if matches!(arg.get_action(), ArgAction::Append) {
                    " ..."
                } else {
                    ""
                },
                if arg.is_required_set() {
                    ":material-check-decagram-outline:{ .opt-required title=\"Required\" }"
                } else {
                    arg.get_help_heading().unwrap_or_default()
                }
            )
            .as_str(),
        );
        if let Some(numb_args) = arg.get_num_args() {
            if numb_args.min_values() > 1 {
                if let Some(help) = arg.get_help() {
                    out.push_str(format!("{}\n", help.to_string().trim()).as_str());
                }
            }
        }
    }

    let groups = if GROUPS_ORDER.is_empty() {
        command
            .get_groups()
            .map(|g| g.to_owned())
            .collect::<Vec<ArgGroup>>()
    } else {
        // reorganize groups according to GROUPS_ORDER
        let mut ordered: Vec<ArgGroup> = Vec::with_capacity(command.get_groups().count());
        for group in GROUPS_ORDER {
            let group_obj = command
                .get_groups()
                .find(|arg_group| arg_group.get_id().as_str() == group)
                .ok_or(PyValueError::new_err(format!(
                    "{} not found in command's groups",
                    group
                )))?;
            ordered.push(group_obj.to_owned());
        }
        ordered
    };

    if groups.is_empty() {
        for arg in command.get_arguments() {
            generate_option(metadata, arg, out, &lvl_pad);
        }
    } else {
        for group in groups {
            out.push_str(format!("\n{lvl_pad}## {}\n", group.get_id()).as_str());
            for arg_id in group.get_args() {
                let arg = command
                    .get_arguments()
                    .find(|a| *a.get_id() == *arg_id)
                    .ok_or(PyValueError::new_err(format!(
                        "arg {} in group {} not found in command",
                        arg_id.as_str(),
                        group.get_id().as_str()
                    )))?;
                generate_option(metadata, arg, out, &lvl_pad);
            }
        }
    }
    Ok(())
}

fn generate_command_usage(command: &mut Command, out: &mut String) {
    out.push_str(
        format!(
            "```text title=\"Usage\"\n{}\n```\n",
            command
                .render_usage()
                .to_string()
                .trim_start_matches("Usage: ")
        )
        .as_str(),
    );
}

#[pyfunction]
fn generate_cli_doc(metadata: HashMap<String, HashMap<String, Py<PyAny>>>) -> PyResult<String> {
    let mut out = String::new();
    let mut command = cli::get_arg_parser();
    generate_command_usage(&mut command, &mut out);
    generate_cmd_args(&metadata, &mut command, &mut out, 1)?;

    out.push_str("\n## Subcommands\n");
    for cmd in command.get_subcommands_mut() {
        out.push_str(format!("\n### `{}`\n\n", cmd.get_name()).as_str());
        inject_metadata(&metadata, &mut out, &cmd.get_name());
        generate_command_usage(cmd, &mut out);

        out.push_str(
            format!(
                "{}\n",
                &cmd.get_about()
                    .ok_or(PyValueError::new_err(format!(
                        "{} command has no help message",
                        cmd.get_name()
                    )))?
                    .to_string()
                    .trim()
            )
            .as_str(),
        );

        generate_cmd_args(&metadata, cmd, &mut out, 2)?;
    }
    Ok(out)
}

#[pymodule]
pub fn cli_gen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_cli_doc, m)?)?;
    Ok(())
}
