use std::time::Duration;

use clap::{error::ErrorKind, value_parser, ArgAction};
use either::Either::{Left, Right};
use metron::{HttpMethod, LoadTestConfig, PlanSegment};
use url::Url;

use crate::{
    parser::{self, RateArgValue},
    InvalidArgsError, BAD_CLAP,
};

/// Creates the [`clap::Command`] for the `test` subcommand.
///
/// # Examples
/// ```bash
/// metron test \
///   --rate 100 \
///   --duration 20s \
///   --target https://example.com
/// ```
pub fn command() -> clap::Command {
    const SHORT: &str = "Run a load test.";
    const LONG: &str = "\
This command is used to run a load test according to a test plan and stream
the results to a number of potential backends.
";

    clap::Command::new("test")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

pub(crate) fn parse_args(matches: &clap::ArgMatches) -> Result<LoadTestConfig, InvalidArgsError> {
    let mut config = matches
        .get_one::<LoadTestConfig>("config-file")
        .cloned()
        .unwrap_or_default();

    let rates = matches.get_many::<RateArgValue>("rate").expect(BAD_CLAP);
    let durations = matches
        .get_many::<Option<Duration>>("duration")
        .expect(BAD_CLAP);

    if rates.len() != durations.len() {
        return Err(InvalidArgsError(
            command()
                .error(
                    ErrorKind::WrongNumberOfValues,
                    "The number of --rate and --duration arguments must match",
                )
                .render()
                .to_string(),
        ));
    }

    let mut it = rates.zip(durations).peekable();
    while let Some((&rate, &duration)) = it.next() {
        // Check that only the last duration value is infinite.
        if duration.is_none() && it.peek().is_some() {
            return Err(InvalidArgsError(
                command()
                    .error(
                        ErrorKind::ValueValidation,
                        "Only the last --duration value can be \"forever\"",
                    )
                    .render()
                    .to_string(),
            ));
        }

        let segment = match rate {
            Left(rate) => PlanSegment::Fixed { rate, duration },
            Right((rate_start, rate_end)) => {
                if let Some(duration) = duration {
                    PlanSegment::Linear {
                        rate_start,
                        rate_end,
                        duration,
                    }
                } else {
                    return Err(InvalidArgsError(
                        command().error(
                            ErrorKind::ValueValidation,
                            "Only fixed-rate segments may have a --duration value can be \"forever\""
                        ).render().to_string()
                    ));
                }
            }
        };

        config.plan.segments.push(segment);
    }

    config.plan.connections = *matches.get_one::<u64>("connections").expect(BAD_CLAP) as usize;
    config.plan.http_method = *matches.get_one("http-method").expect(BAD_CLAP);
    config.plan.targets = matches
        .get_many::<Url>("target")
        .expect(BAD_CLAP)
        .cloned()
        .collect::<Vec<_>>();

    config.plan.headers = matches
        .get_many("header")
        .unwrap_or_default()
        .cloned()
        .collect();

    config.plan.payload = matches.get_one::<String>("payload").cloned();
    config.plan.latency_correction = *matches.get_one("latency-correction").expect(BAD_CLAP);

    Ok(config)
}

/// Returns all [`clap::Arg`]s for the `profile` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![
        arg_config_file(),
        arg_print_config(),
        arg_rate(),
        arg_duration(),
        arg_target(),
        arg_http_method(),
        arg_payload(),
        arg_header(),
        arg_worker_threads(),
        arg_connections(),
        arg_latency_correction(),
    ]
}

/// Returns the [`clap::ArgGroup`]s for the `profile` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![arg_group_thread_model()]
}

/// Returns the [`clap::ArgGroup`] for the arguments that decide the thread model.
fn arg_group_thread_model() -> clap::ArgGroup {
    clap::ArgGroup::new("group-thread-model").multiple(false)
}

/// Returns the [`clap::Arg`] for `--config-file`.
fn arg_config_file() -> clap::Arg {
    const SHORT: &str = "Test configuration file.";
    const LONG: &str = "\
A configuration file to be used as an alternative to individual command line
arguments. Stdin can also be used by specifying hyphen as the file name (i.e.
`--config-file -`).

When both a configuration file and individual command line arguments are used,
the arguments will override their counterpart properties in the configuration
file.

See --print-config for bootstrapping a configuration file.
";

    clap::Arg::new("config-file")
        .long("config-file")
        .value_name("FILE")
        .value_parser(parser::config_file::<LoadTestConfig>)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--print-config`.
fn arg_print_config() -> clap::Arg {
    const SHORT: &str = "Print the controller configuration.";
    const LONG: &str = "\
Generates the configuration for this command and prints it to stdout. This may
be used to bootstrap a configuration file based on command line arguments so
that a configuration file can be used rather than individual command line
arguments.
";

    clap::Arg::new("print-config")
        .long("print-config")
        .action(ArgAction::SetTrue)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--rate`.
fn arg_rate() -> clap::Arg {
    const SHORT: &str = "Desired throughput rates.";
    const LONG: &str = "\
Sets the desired throughput rate in terms of the number of requests per second
(RPS) that should be generated for each segment of the test.

This argument can receive multiple values and may be used to specify both fixed
and variable rates. To specify a fixed rate, specify an integer value; e.g.
--rate=100 --duration=15m implies a fixed rate of 100 RPS for 15 minutes. To use
a variable rate, specify a range; e.g. --rate=100:200 --duration=15m implies
that the request rate should increase linearly from 100 RPS to 200 RPS over a 15
minute duration.

To specify segments that each have their own rate and duration, specify multiple
comma-separated values; e.g. --rate=100:500,500 --duration=5m,15m will create a
20 minute test plan containing two segments: the initial segment will ramp the
rate up from 100 RPS to 500 RPS over the first 5 minutes and then the rate will
be held at a constant 500 RPS for the next 15 minutes. When multiple values are
specified, both --rate and --duration must receive the same number of values.
";

    clap::Arg::new("rate")
        .long("rate")
        .value_name("RATE")
        .required(true)
        .action(ArgAction::Append)
        .num_args(1..)
        .value_delimiter(',')
        .value_parser(parser::rate)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--duration`.
fn arg_duration() -> clap::Arg {
    const SHORT: &str = "Performance test durations.";
    const LONG: &str = "\
Sets the durations of each test segment.

This argument can receive one or more values; the number of values specified
must match the number of values passed to --rate. Each value defines the
duration of the associated test segment.

To specify segments that each have their own rate and duration, specify multiple
comma-separated values; e.g. --rate=100:500,500 --duration=5m,15m will create a
20 minute test plan containing two segments: the initial segment will ramp the
rate up from 100 RPS to 500 RPS over the first 5 minutes and then the rate will
be held at a constant 500 RPS for the next 15 minutes. When multiple values are
specified, both --rate and --duration must receive the same number of values.

A value of \"forever\" may be specified for fixed rate segments to indicate that
the test should run forever or until CTRL-C is pressed. When specifying multiple
test segments, \"forever\" can only be specified for the last segment. E.g.
--rate=100,200 --duration=5m,forever will will create an infinite test plan
containing two segments: the first segment will rate 100 RPS for 5 minutes and
then the second segment will rate 200 RPS until it is interrupted. Variable rate
segments are not allowed to have a value of \"forever\" as these segments must
be able to be calculated over a finite duration.

See https://docs.rs/humantime/latest/humantime for time format details.
";

    clap::Arg::new("duration")
        .long("duration")
        .value_name("DURATION")
        .required(true)
        .action(ArgAction::Append)
        .num_args(1..)
        .value_delimiter(',')
        .value_parser(parser::duration)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--target`.
fn arg_target() -> clap::Arg {
    const SHORT: &str = "Performance profile target(s).";
    const LONG: &str = "\
Sets one or more target URLs for the performance profile. HTTP and HTTPS URLs
are supported.

This argument may be specified multiple times to specify multiple targets. The
performance test will evenly distribute requests between the targets using round-robin.
";

    clap::Arg::new("target")
        .long("target")
        .value_name("URL")
        .required(true)
        .action(ArgAction::Append)
        .num_args(1..)
        .value_delimiter(',')
        .value_parser(parser::target)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--http-method`.
fn arg_http_method() -> clap::Arg {
    const SHORT: &str = "HTTP method.";
    const LONG: &str = "\
Sets the HTTP method to use when making requests of the target.

If this argument is not specifed and no payload is specified (--payload or
--payload-file) then HTTP GET will be assumed. If this argument is not specified
and a payload is specified then HTTP POST will be assumed.
";

    clap::Arg::new("http-method")
        .long("http-method")
        .value_name("METHOD")
        .default_value(default::HTTP_METHOD.as_str())
        .value_parser(value_parser!(HttpMethod))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--payload`.
fn arg_payload() -> clap::Arg {
    const SHORT: &str = "HTTP payload.";
    const LONG: &str = "\
Sets the HTTP payload string to use when making requests of the target.

If a payload-based HTTP method such as POST or PUT has been specified
(--http-method), and no payload has been specified (--payload or --payload-file)
then an empty payload will be used.
";

    clap::Arg::new("payload")
        .long("payload")
        .value_name("PAYLOAD")
        .value_parser(value_parser!(String))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--header`.
fn arg_header() -> clap::Arg {
    const SHORT: &str = "HTTP header in K:V format.";
    const LONG: &str = "\
Sets the specified header to be included in all requests. The value for this
argument should be in K:V format, where K is the header name and V is the
header value.

This argument can be specified multiple times.
";

    clap::Arg::new("header")
        .long("header")
        .value_name("K:V")
        .value_delimiter(',')
        .value_parser(parser::header)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--worker-threads`.
fn arg_worker_threads() -> clap::Arg {
    const SHORT: &str = "Number of worker threads to use.";
    const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

The worker threads are the set of threads that are cooperatively scheduled to
perform the load test. This number does not include the thread allocated to the
signaller if a blocking signaller is used (see --signaller).

This argument defaults to the number of cores on the host machine.
";

    clap::Arg::new("worker-threads")
        .long("worker-threads")
        .group("group-thread-model")
        .value_name("COUNT")
        .value_parser(value_parser!(u64).range(1..1000))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--connections`.
fn arg_connections() -> clap::Arg {
    const SHORT: &str = "Number of TCP connections to use.";
    const LONG: &str = "\
Sets the number of TCP connections that should be used.

TODO: Elaborate.
";

    clap::Arg::new("connections")
        .long("connections")
        .value_name("COUNT")
        .default_value(default::CONNECTIONS.as_str())
        .value_parser(value_parser!(u64).range(1..))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--latency-correction`.
fn arg_latency_correction() -> clap::Arg {
    const SHORT: &str = "Enable/disable latency correction.";
    const LONG: &str = "\
When latency correction is enabled, the latency that is recorded for each
request is calculated from when the request was scheduled to be sent, rather
than when it was actually sent. This helps to account for the phenomenon
known as \"Coordinated Omission\". Latency correction is enabled by defeault.
";

    clap::Arg::new("latency-correction")
        .long("latency-correction")
        .value_name("BOOL")
        .default_value(default::LATENCY_CORRECTION.as_str())
        .value_parser(value_parser!(bool))
        .help(SHORT)
        .long_help(LONG)
}

mod default {
    use super::*;
    lazy_static::lazy_static! {
        static ref CONFIG: LoadTestConfig = LoadTestConfig::default();
        pub(super) static ref HTTP_METHOD: String = CONFIG.plan.http_method.to_string();
        pub(super) static ref CONNECTIONS: String = CONFIG.plan.connections.to_string();
        pub(super) static ref LATENCY_CORRECTION: String = CONFIG.plan.latency_correction.to_string();
    }
}
