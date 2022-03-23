use anyhow::Result;
use hyper::Uri;
use std::str::FromStr;
use wrkr::Rate;

use crate::load::{Header, TestConfig};

const ABOUT: &str = "
wrkr is a modern performance characterization tool.

Use --help for more details.

Project home: https://github.com/ashlineldridge/wrkr
";

/// Parses the command-line arguments into a [clap::ArgMatches].
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn parse_clap() -> clap::ArgMatches {
    use clap::*;

    let app = Command::new("wrkr")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .subcommand(load::command())
        .subcommand(server::command())
        .subcommand_required(true);

    // let matches = app.get_matches();

    // match matches.subcommand() {
    //     Some(("test", matches)) => {
    //         if !matches.is_present("worker-threads") {
    //             println!("No worker threads specified. Will use the default");
    //         } else {
    //             let worker_threads = matches.value_of_t_or_exit::<usize>("worker-threads");
    //             println!("You specified {} worker threads", worker_threads);
    //         }
    //     }
    //     Some(("serve", matches)) => todo!(),
    //     _ => todo!(),
    // };

    std::process::exit(0);
    todo!()
}

fn root_command() -> clap::Command<'static> {
    use clap::*;

    Command::new("wrkr")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .subcommand(load::command())
        .subcommand(server::command())
        .subcommand_required(true)
        .arg(Arg::new(""))
}

mod load {
    pub(super) fn command() -> clap::Command<'static> {
        clap::Command::new("load")
            .arg(arg_worker_threads())
            .arg(arg_connections())
            .arg(arg_duration())
            .arg(arg_rate())
            .arg(arg_ramp_duration())
            .arg(arg_ramp_rate_start())
            .arg(arg_ramp_rate_end())
    }

    /// Returns the [`clap::Arg`] for `--worker-threads`.
    fn arg_worker_threads() -> clap::Arg<'static> {
        const SHORT: &str = "Number of worker threads to use.";
        const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

The worker threads are the set of threads that are cooperatively scheduled to
perform the load test. This number does not include the thread allocated to the
signaller if a blocking-thread signaller is used (see --signaller-type).

This argument defaults to the number of cores on the host machine.
";

        clap::Arg::new("worker-threads")
            .long("worker-threads")
            .value_name("COUNT")
            // Example of custom validation:
            // https://github.com/clap-rs/clap/blob/master/examples/tutorial_builder/04_02_validate.rs#L25
            .validator(|s| s.parse::<usize>())
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--connections`.
    fn arg_connections() -> clap::Arg<'static> {
        const SHORT: &str = "Number of TCP connections to use.";
        const LONG: &str = "\
Sets the number of TCP connections that should be used.

TODO: Elaborate.
";

        clap::Arg::new("connections")
            .long("connections")
            .value_name("COUNT")
            .default_value("1")
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--duration`.
    fn arg_duration() -> clap::Arg<'static> {
        const SHORT: &str = "Primary load test duration.";
        const LONG: &str = "\
Sets the primary duration of the load test to DURATION where DURATION is
specified in human-readable time format. For example, a value of
\"1hour 30min 30s\" will run the test for 1 hour, 30 minutes, and 30 seconds.

The duration specified by this argument does not include any ramp duration
(see --ramp-duration and --ramp-rate-start). The total duration for a load
test is the primary duration plus any ramp duration.

If this argument is not specified, the test will run forever, or until Ctrl+C
is pressed.

See https://docs.rs/humantime/latest/humantime for time format details.
";

        clap::Arg::new("duration")
            .long("duration")
            .group("main")
            .value_name("DURATION")
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--rate`.
    fn arg_rate() -> clap::Arg<'static> {
        const SHORT: &str = "Primary request rate (per second).";
        const LONG: &str = "\
Sets the primary request rate for the load test to be fixed to RATE requests
per second.

If a ramp is being used (see --ramp-duration and --ramp-rate-start),
this primary request rate will kick in after the ramp is complete.

If this argument is not specified, the load test will operate at the maximum
achievable rate.

See https://docs.rs/humantime/latest/humantime for time format details.
";

        clap::Arg::new("rate")
            .long("rate")
            .group("main")
            .value_name("RATE")
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--max-rate`.
    fn arg_max_rate() -> clap::Arg<'static> {
        const SHORT: &str = "Perform a maximum rate load test.";
        const LONG: &str = "\
Specifies that a maximum rate load test should be performed rather that one
where the desired request timing can be predicted ahead of time.

During a maximum rate load test there is no waiting between timing signals
like there is for rate-based load tests. Requests will be sent to the target(s)
as quickly as host resources allow.

This argument is incompatible with --rate, --ramp-duration, --ramp-rate-start,
and --ramp-rate-end.
";

        clap::Arg::new("max-rate")
            .long("max-rate")
            .value_name("RATE")
            .conflicts_with_all(&["main", "ramp"])
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--ramp-duration`.
    fn arg_ramp_duration() -> clap::Arg<'static> {
        const SHORT: &str = "Ramped throughput duration.";
        const LONG: &str = "\
Sets the ramp duration of the load test to DURATION where DURATION is specified
in human-readable time format. For example, a value of \"5min 30s\" will ramp
the throughput over a period of 5 minutes and 30 seconds.

If this argument is specified, both --ramp-rate-start and --ramp-rate-end must
also be specified.

See https://docs.rs/humantime/latest/humantime for time format details.
";

        clap::Arg::new("ramp-duration")
            .long("ramp-duration")
            .group("ramp")
            .requires_all(&["ramp-rate-start", "ramp-rate-end"])
            .value_name("DURATION")
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--ramp-rate-start`.
    fn arg_ramp_rate_start() -> clap::Arg<'static> {
        const SHORT: &str = "Starting rate for the ramp.";
        const LONG: &str = "\
Sets the starting request rate (in requests per second) for the throughput ramp
that will be used to begin the load test.

If this argument is specified, both --ramp-rate-duration and --ramp-rate-end must
also be specified.
";

        clap::Arg::new("ramp-rate-start")
            .long("ramp-rate-start")
            .group("ramp")
            .requires_all(&["ramp-rate-duration", "ramp-rate-end"])
            .value_name("RATE")
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--ramp-rate-end`.
    fn arg_ramp_rate_end() -> clap::Arg<'static> {
        const SHORT: &str = "Ending rate for the ramp.";
        const LONG: &str = "\
Sets the ending request rate (in requests per second) for the throughput ramp
that will be used to begin the load test.

If this argument is specified, both --ramp-rate-duration and --ramp-rate-start must
also be specified.
";

        clap::Arg::new("ramp-rate-end")
            .long("ramp-rate-end")
            .group("ramp")
            .requires_all(&["ramp-rate-duration", "ramp-rate-start"])
            .value_name("RATE")
            .help(SHORT)
            .long_help(LONG)
    }
}

mod server {
    /// Creates the `server` subcommand.
    pub(super) fn command() -> clap::Command<'static> {
        clap::Command::new("server")
            .arg(arg_port())
            .arg(arg_worker_threads())
    }

    /// Returns the [`clap::Arg`] for `--port`.
    fn arg_port() -> clap::Arg<'static> {
        const SHORT: &str = "Port to serve on.";
        const LONG: &str = "\
Sets the server listening port to PORT. Defaults to 8000.
";

        clap::Arg::new("port")
            .long("port")
            .value_name("PORT")
            .default_value("8000")
            .validator(|s| s.parse::<u16>())
            .help(SHORT)
            .long_help(LONG)
    }

    /// Returns the [`clap::Arg`] for `--worker-threads`.
    fn arg_worker_threads() -> clap::Arg<'static> {
        const SHORT: &str = "Number of worker threads to use.";
        const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

If this value is not specified it will default to the number of cores on the
host machine.
";

        clap::Arg::new("worker-threads")
            .long("worker-threads")
            .value_name("COUNT")
            .validator(|s| s.parse::<usize>())
            .help(SHORT)
            .long_help(LONG)
    }
}

/// The subcommand to run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    /// Run a performance test
    Load,
    /// Start an echo server
    Server,
}

// // #[derive(Debug, Parser)]
// pub struct TestArgs {
//     /// Number of threads
//     // #[clap(long, value_name = "COUNT", default_value_t = 4)]
//     pub threads: usize,

//     /// Number of connections to use
//     // #[clap(long, value_name = "COUNT", default_value_t = 1)]
//     pub connections: usize,

//     /// Requests per second
//     // #[clap(long)]
//     pub rate: Option<u32>,

//     /// Execution duration
//     // #[clap(long)]
//     pub duration: Option<humantime::Duration>,

//     /// Duration over which to ramp up throughput
//     // #[clap(long, value_name = "DURATION", group = "ramp")]
//     pub ramp_duration: Option<humantime::Duration>,

//     /// Initial requests per second for ramped throughput
//     // #[clap(long, value_name = "RATE", group = "ramp")]
//     pub ramp_rate_start: Option<u32>,

//     /// Header to pass in request (may be repeated)
//     // #[clap(long, value_name = "NAME:VALUE")]
//     pub header: Vec<Header>,

//     // Target URL
//     pub target: Uri,
// }

// // #[derive(Debug, Parser)]
// pub struct ServeArgs {
//     /// Port to listen on
//     // #[clap(short, long, default_value_t = 8080)]
//     pub port: u16,

//     /// Number of threads
//     // #[clap(short, long, value_name = "COUNT", default_value_t = 4)]
//     pub threads: usize,
// }

// impl From<TestArgs> for TestConfig {
//     fn from(cli: TestArgs) -> Self {
//         let app = clap::App::new("wrkr");
//         app.try_get_matches();

//         Self {
//             connections: cli.connections,
//             worker_threads: cli.threads,
//             async_signaller: false,
//             rate: cli.rate.map(|r| Rate(r)),
//             duration: cli.duration.map(|d| d.into()),
//             init_rate: cli.ramp_rate_start.map(|r| Rate(r)),
//             ramp_duration: cli.ramp_duration.map(|d| d.into()),
//             headers: cli.header,
//             target: cli.target,
//         }
//     }
// }

// impl From<ServeArgs> for ServeConfig {
//     fn from(cli: ServeArgs) -> Self {
//         Self {
//             port: cli.port,
//             threads: cli.threads,
//         }
//     }
// }

// impl FromStr for Header {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let (name, value) = s.split_once(':').unwrap_or((s, ""));

//         Ok(Self {
//             name: name.trim().into(),
//             value: value.trim().into(),
//         })
//     }
// }
