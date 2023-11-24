use std::time::Duration;

use clap::{error::ErrorKind, value_parser, ArgAction};
use either::Either::{Left, Right};
use metron::{Action, HttpMethod, Plan, RateSegment, TestConfig};
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
///   https://example.com
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
        .disable_version_flag(true)
}

pub(crate) fn parse(matches: &clap::ArgMatches) -> Result<TestConfig, InvalidArgsError> {
    // If a config file was specified then use that.
    if let Some(config) = matches.get_one::<TestConfig>("file") {
        return Ok(config.clone());
    }

    // No config file was specified so parse each of the command line arguments.
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

    let mut segments = Vec::with_capacity(durations.len());

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
            Left(rate) => RateSegment::Fixed { rate, duration },
            Right((rate_start, rate_end)) => {
                if let Some(duration) = duration {
                    RateSegment::Linear {
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

        segments.push(segment);
    }

    let payload = matches
        .get_one::<String>("payload")
        .cloned()
        .unwrap_or_default();

    let target = matches.get_one::<Url>("target").cloned().expect(BAD_CLAP);
    let action = match target.scheme() {
        "http" | "https" => {
            let method = *matches.get_one("http-method").unwrap_or(&HttpMethod::Get);
            let headers = matches
                .get_many("http-header")
                .unwrap_or_default()
                .cloned()
                .collect();

            Action::Http {
                method,
                headers,
                payload,
                target,
            }
        }
        "udp" => {
            for arg in ["http-method", "http-header"] {
                if matches.contains_id(arg) {
                    return Err(InvalidArgsError(
                        command().error(
                            ErrorKind::ArgumentConflict,
                            format!("Argument --{arg} is incompatible with target URL scheme \"udp\"")
                        ).render().to_string()));
                }
            }

            Action::Udp { payload, target }
        }
        _ => panic!("{}", BAD_CLAP),
    };

    Ok(TestConfig {
        plan: Plan {
            segments,
            actions: vec![action],
        },
        runners: None,
        runtime: None,
        telemetry: Default::default(),
    })
}

/// Returns all [`clap::Arg`]s for the `profile` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![
        arg_file(),
        arg_rate(),
        arg_duration(),
        arg_http_method(),
        arg_http_header(),
        arg_payload(),
        arg_threads(),
        arg_target(),
    ]
}

/// Returns the [`clap::Arg`] for `--file`.
fn arg_file() -> clap::Arg {
    const SHORT: &str = "Test configuration file.";
    const LONG: &str = "\
A configuration file to be used as an alternative to individual command line
arguments. Stdin can also be used by specifying hyphen as the file name (i.e.
`--file -`).

When both a configuration file and individual command line arguments are used,
the arguments will override their counterpart properties in the configuration
file.

See --print-config for bootstrapping a configuration file.
";

    clap::Arg::new("file")
        .short('f')
        .long("file")
        .value_name("FILE")
        .value_parser(parser::config_file::<TestConfig>)
        .required_unless_present_all(["rate", "duration"])
        .conflicts_with_all(["rate", "duration"])
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
        .short('r')
        .long("rate")
        .value_name("RATE")
        .value_delimiter(',')
        .value_parser(parser::rate)
        // .num_args(1..)
        .action(ArgAction::Append)
        .required_unless_present("file")
        .conflicts_with("file")
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
        .short('d')
        .long("duration")
        .value_name("DURATION")
        .value_delimiter(',')
        .value_parser(parser::duration)
        // .num_args(1..)
        .action(ArgAction::Append)
        .required_unless_present("file")
        .conflicts_with("file")
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
        .short('m')
        .long("http-method")
        .value_name("METHOD")
        .value_parser(value_parser!(HttpMethod))
        .conflicts_with("file")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--http-header`.
fn arg_http_header() -> clap::Arg {
    const SHORT: &str = "HTTP header in K:V format.";
    const LONG: &str = "\
Sets the specified header to be included in all requests. The value for this
argument should be in K:V format, where K is the header name and V is the
header value.

This argument can be specified multiple times.
";

    clap::Arg::new("http-header")
        .short('H')
        .long("http-header")
        .value_name("K:V")
        .value_delimiter(',')
        .value_parser(parser::header)
        .conflicts_with("file")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--payload`.
fn arg_payload() -> clap::Arg {
    const SHORT: &str = "Request payload.";
    const LONG: &str = "\
Sets the request payload string to use when making requests of the target.

If a payload-based HTTP method such as POST or PUT has been specified
(--http-method), and no payload has been specified (--payload or --payload-file)
then an empty payload will be used.
";

    clap::Arg::new("payload")
        .short('p')
        .long("payload")
        .value_name("PAYLOAD")
        .conflicts_with("file")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--threads`.
fn arg_threads() -> clap::Arg {
    const SHORT: &str = "Number of worker threads to use.";
    const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

The worker threads are the set of threads that are cooperatively scheduled to
perform the load test. This number does not include the thread allocated to the
signaller if a blocking signaller is used (see --signaller).

This argument defaults to the number of cores on the host machine.
";

    clap::Arg::new("threads")
        .short('t')
        .long("threads")
        .value_name("COUNT")
        .value_parser(value_parser!(u64).range(1..1000))
        .conflicts_with("file")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--target`.
fn arg_target() -> clap::Arg {
    const SHORT: &str = "Performance profile target(s).";
    const LONG: &str = "\
Sets the load test target.

Not true: This argument may be specified multiple times to specify multiple targets. The
performance test will evenly distribute requests between the targets using round-robin.
";

    clap::Arg::new("target")
        .value_name("TARGET")
        .value_parser(parser::url)
        .required_unless_present("file")
        .conflicts_with("file")
        .help(SHORT)
        .long_help(LONG)
}
