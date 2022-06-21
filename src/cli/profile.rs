use clap::{ArgAction, value_parser};

use crate::cli::parser;

/// Creates the [`clap::Command`] for the `profile` subcommand.
///
/// # Examples
/// ```bash
/// metron profile \
///   --rate 100 \
///   --duration 20s \
///   https://example.com
/// ```
pub fn command() -> clap::Command<'static> {
    const SHORT: &str = "Runs a performance profile test.";
    const LONG: &str = "\
Runs a performance test against the specified target(s) and produces a report.

The report can be written to stdout and/or streamed to a metrics backend.
";

    clap::Command::new("profile")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
}

/// Returns all [`clap::Arg`]s for the `profile` subcommand.
fn all_args() -> Vec<clap::Arg<'static>> {
    vec![
        arg_rate(),
        arg_duration(),
        arg_target(),
        arg_http_method(),
        arg_payload(),
        arg_payload_file(),
        arg_header(),
        arg_worker_threads(),
        arg_single_threaded(),
        arg_connections(),
        arg_signaller(),
        arg_stop_on_client_error(),
        arg_stop_on_non_2xx(),
        arg_log_level(),
    ]
}

/// Returns the [`clap::ArgGroup`]s for the `profile` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup<'static>> {
    vec![arg_group_payload(), arg_group_thread_model()]
}

/// Returns the [`clap::ArgGroup`] for the arguments that decide the request payload.
fn arg_group_payload() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-payload").multiple(false)
}

/// Returns the [`clap::ArgGroup`] for the arguments that decide the thread model.
fn arg_group_thread_model() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-thread-model").multiple(false)
}

/// Returns the [`clap::Arg`] for `--rate`.
fn arg_rate() -> clap::Arg<'static> {
    const SHORT: &str = "Desired throughput rates.";
    const LONG: &str = "\
Sets the desired throughput rate in terms of the number of requests per second
(RPS) that should be generated for each segment of the test.

This argument can receive multiple values and may be used to specify both fixed
and variable rates. To specify a fixed rate, specify an integer value; e.g.,
--rate=100 --duration=15m implies a fixed rate of 100 RPS for 15 minutes. To use
a variable rate, specify a range; e.g., --rate=100:200 --duration=15m implies
that the request rate should increase linearly from 100 RPS to 200 RPS over a 15
minute duration.

To specify multiple test plan segments, where each segment can have its own rate
and duration, specify multiple comma-separated values; e.g., --rate=100:500,500
--duration=5m,15m will create a 20 minute test plan containing two segments: the
initial segment will ramp the rate up from 100 RPS to 500 RPS over the first
5 minutes and then the rate will be held at a constant 500 RPS for the next 15
minutes. When multiple values are specified, both --rate and --duration must
receive the same number of values.
";

    clap::Arg::new("rate")
        .long("rate")
        .value_name("RATE")
        .required(true)
        .multiple_values(true)
        .multiple_occurrences(true)
        .require_value_delimiter(true)
        .value_delimiter(',')
        .value_parser(parser::rate)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--duration`.
fn arg_duration() -> clap::Arg<'static> {
    const SHORT: &str = "Performance test durations.";
    const LONG: &str = "\
Sets the durations of each test segment.

This argument can receive one or more values; the number of values specified
must match the number of values passed to --rate. Each value defines the
duration of the associated test segment.

To specify multiple test plan segments, where each segment can have its own rate
and duration, specify multiple comma-separated values; e.g., --rate=100:500,500
--duration=5m,15m will create a 20 minute test plan containing two segments: the
initial segment will ramp the rate up from 100 RPS to 500 RPS over the first
5 minutes and then the rate will be held at a constant 500 RPS for the next 15
minutes.

A value of \"forever\" may be specified for fixed rate segments to indicate that
the test should run forever or until CTRL-C is pressed. When specifying multiple
test segments, \"forever\" can only be specified for the last segment. E.g.,
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
        .multiple_values(true)
        .require_value_delimiter(true)
        .value_delimiter(',')
        .value_parser(parser::duration)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--target`.
fn arg_target() -> clap::Arg<'static> {
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
        .multiple_values(true)
        .require_value_delimiter(true)
        .value_delimiter(',')
        .value_parser(parser::target)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--http-method`.
fn arg_http_method() -> clap::Arg<'static> {
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
        .default_value("GET")
        .value_parser([
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "PATCH", "TRACE",
        ])
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--payload`.
fn arg_payload() -> clap::Arg<'static> {
    const SHORT: &str = "HTTP payload.";
    const LONG: &str = "\
Sets the HTTP payload string to use when making requests of the target.

If a payload-based HTTP method such as POST or PUT has been specified
(--http-method), and no payload has been specified (--payload or --payload-file)
then an empty payload will be used.
";

    clap::Arg::new("payload")
        .long("payload")
        .group("group-payload")
        .value_name("PAYLOAD")
        .value_parser(value_parser!(String))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--payload-file`.
fn arg_payload_file() -> clap::Arg<'static> {
    const SHORT: &str = "HTTP payload file.";
    const LONG: &str = "\
Sets the HTTP payload file to use when making requests of the target.

If a payload-based HTTP method such as POST or PUT has been specified
(--http-method), and no payload has been specified (--payload or --payload-file)
then an empty payload will be used.
";

    clap::Arg::new("payload-file")
        .long("payload-file")
        .group("group-payload")
        .value_name("FILE")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--header`.
fn arg_header() -> clap::Arg<'static> {
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
        .multiple_values(true)
        .require_value_delimiter(true)
        .value_delimiter(',')
        .multiple_occurrences(true)
        .value_parser(parser::header)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--worker-threads`.
fn arg_worker_threads() -> clap::Arg<'static> {
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

/// Returns the [`clap::Arg`] for `--single-threaded`.
fn arg_single_threaded() -> clap::Arg<'static> {
    const SHORT: &str = "Don't spawn threads.";
    const LONG: &str = "\
Forces all operations to run on the main thread.

The utility of this argument is unknown beyond providing interesting data on how
the number of threads affects performance of the tool itself. This argument
forces all operations to run on the main thread whereas --worker-threads=1 will
result in the main thread creating a single worker thread to perform the
requests.

This argument is incompatible with --worker-threads and --signaller=blocking.
";

    clap::Arg::new("single-threaded")
        .long("single-threaded")
        .group("group-thread-model")
        .action(ArgAction::SetTrue)
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
        .value_parser(value_parser!(u64).range(1..))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--signaller`.
fn arg_signaller() -> clap::Arg<'static> {
    const SHORT: &str = "Method for generating timing signals.";
    const LONG: &str = "\
Selects the type of signalling system that should be used to generate request
timing signals. This is an advanced feature and the default behaviour will
generally be what you want.
";

    clap::Arg::new("signaller")
        .long("signaller")
        .value_name("NAME")
        .default_value("blocking")
        .value_parser(["blocking", "cooperative"])
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--stop-on-client-error`.
fn arg_stop_on_client_error() -> clap::Arg<'static> {
    const SHORT: &str = "Whether to stop if on error.";
    const LONG: &str = "\
Sets whether the profiling operation should stop if the client encounters an
error when sending requests to the target(s). This setting only affects *client-
side* errors (e.g., too many open files) and not HTTP error statuses returned by
the target(s).

See --stop-on-http-non-2xx for setting HTTP status stopping behaviour.
";

    clap::Arg::new("stop-on-client-error")
        .long("stop-on-client-error")
        .action(ArgAction::SetTrue)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--stop-on-non-2xx`.
fn arg_stop_on_non_2xx() -> clap::Arg<'static> {
    const SHORT: &str = "Whether to stop on non-2XX HTTP status.";
    const LONG: &str = "\
Sets whether the profiling operation should stop if a non-2XX HTTP status is
retured.

See --stop-on-client-error for setting error stopping behaviour.
";

    clap::Arg::new("stop-on-non-2xx")
        .long("stop-on-non-2xx")
        .action(ArgAction::SetTrue)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--log-level`.
fn arg_log_level() -> clap::Arg<'static> {
    const SHORT: &str = "Minimum logging level.";
    const LONG: &str = "\
Sets the minimum logging level. Log messages at or above the specified
severity level will be printed.
";

    clap::Arg::new("log-level")
        .long("log-level")
        .value_name("LEVEL")
        .default_value("off")
        .value_parser(["off", "debug", "info", "warn", "error"])
        .help(SHORT)
        .long_help(LONG)
}
