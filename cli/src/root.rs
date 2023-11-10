use crate::{controller, runner, test};

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
        .subcommands([test::command(), runner::command(), controller::command()])
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
