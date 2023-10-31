use crate::{agent, control, run};

const ABOUT: &str = "\
Metron is a modern load testing toolchain.

Use -h for short help and --help for extended help.

Project home: https://github.com/ashlineldridge/metron";

const USAGE: &str = "\
metron <COMMAND> <OPTIONS>";

const HELP_TEMPLATE: &str = "\
{name} {version}
{author}

{about}

{usage-heading} {usage}

{all-args}";

/// Returns the root [`clap::Command`] for the application.
pub fn command() -> clap::Command {
    use clap::*;

    Command::new("metron")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(USAGE)
        .help_template(HELP_TEMPLATE)
        .subcommand_required(true)
        .subcommands(all_subcommands())
}

fn all_subcommands() -> Vec<clap::Command> {
    vec![agent::command(), control::command(), run::command()]
        .into_iter()
        .map(|c| c.args(common_args()).groups(common_arg_groups()))
        .collect()
}

/// Returns the [`clap::Arg`]s common to all subcommands.
fn common_args() -> Vec<clap::Arg> {
    vec![arg_config_file(), arg_print_config()]
}

/// Returns the [`clap::ArgGroup`]s common to all for the root command.
fn common_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--config-file`.
fn arg_config_file() -> clap::Arg {
    const SHORT: &str = "Configuration file.";
    const LONG: &str = "\
All commands allow a configuration file to be used as an alternative to
individual command line arguments. Stdin can also be used by specifying
a hyphen as the file name (i.e. `--config-file -`).

When both a configuration file and individual command line arguments are used,
the arguments will override their counterpart properties in the configuration
file.

See --print-config for bootstrapping a configuration file.
";

    clap::Arg::new("config-file")
        .long("config-file")
        .value_name("FILE")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--print-config`.
fn arg_print_config() -> clap::Arg {
    const SHORT: &str = "Prints the configuration.";
    const LONG: &str = "\
Generates the configuration for this command and prints it to stdout. This may
be used to bootstrap a configuration file based on command line arguments so
that a configuration file can be used rather than individual command line
arguments.
";

    clap::Arg::new("print-config")
        .long("print-config")
        .help(SHORT)
        .long_help(LONG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_command() {
        // Clap documentation recommends running the following to perform some basic
        // assertions on the top-level command.
        command().debug_assert();
    }
}
