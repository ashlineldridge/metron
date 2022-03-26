use crate::cli::validate::{self, validate};

/// Creates the [`clap::Command`] for the `load` subcommand.
///
/// # Examples
/// ```bash
/// wrkr load \
///   --duration 20s \
///   --rate 100 \
///   https://example.com
/// ```
pub fn command() -> clap::Command<'static> {
    clap::Command::new("load")
        .args(all_args())
        .groups(all_arg_groups())
}

/// Returns all [`clap::Arg`]s for the `load` subcommand.
fn all_args() -> Vec<clap::Arg<'static>> {
    vec![
        arg_duration(),
        arg_forever(),
        arg_rate(),
        arg_max_rate(),
        arg_ramp_duration(),
        arg_ramp_rate_start(),
        arg_ramp_rate_end(),
        arg_target(),
        arg_multi_target(),
        arg_http_method(),
        arg_payload(),
        arg_payload_file(),
        arg_header(),
        arg_worker_threads(),
        arg_single_threaded(),
        arg_connections(),
        arg_signaller(),
    ]
}

/// Returns the [`clap::ArgGroup`]s for the `load` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup<'static>> {
    vec![
        arg_group_primary(),
        arg_group_primary_duration(),
        arg_group_primary_rate(),
        arg_group_ramp(),
        arg_group_payload(),
    ]
}

/// Returns the [`clap::ArgGroup`] for the primary load test arguments.
///
/// The primary load testing arguments are the arguments which dictate how the
/// primary portion of the load test runs. The primary portion refers to the
/// duration of the load test that follows any throughput ramp.
fn arg_group_primary() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-primary").multiple(true)
}

/// Returns the [`clap::ArgGroup`] for the primary duration arguments.
///
/// This argument group ensures that a primary duration has been set (using
/// `--duration` or `--forever`).
fn arg_group_primary_duration() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-primary-duration")
        .multiple(false)
        .required(true)
}

/// Returns the [`clap::ArgGroup`] for the primary rate arguments.
///
/// This argument group ensures that a primary rate has been set (using
/// `--rate` or `--max-rate`).
fn arg_group_primary_rate() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-primary-rate")
        .multiple(false)
        .required(true)
}

/// Returns the [`clap::ArgGroup`] for the ramp load test arguments.
///
/// The ramp load testing arguments are the arguments which dictate how the
/// ramp portion of the load test runs. The ramp precedes the primary portion
/// of the load test.
fn arg_group_ramp() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-ramp").multiple(true)
}

/// Returns the [`clap::ArgGroup`] for payload arguments.
///
/// This argument group ensures that only one payload argument is specified.
fn arg_group_payload() -> clap::ArgGroup<'static> {
    clap::ArgGroup::new("group-payload").multiple(false)
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

Either --duration or --forever must be specified, but not both.

See https://docs.rs/humantime/latest/humantime for time format details.
";

    clap::Arg::new("duration")
        .long("duration")
        .groups(&["group-primary", "group-primary-duration"])
        .value_name("DURATION")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--forever`.
fn arg_forever() -> clap::Arg<'static> {
    const SHORT: &str = "Run forever (or until Ctrl+Ci pressed).";
    const LONG: &str = "\
Specifies that the load test should run forever, or until Ctrl+C is pressed.
This flag applies to the primary portion of the load test, after any ramp has
been executed.

Either --duration or --forever must be specified, but not both.
";

    clap::Arg::new("forever")
        .long("forever")
        .groups(&["group-primary", "group-primary-duration"])
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

Either --rate or --max-rate must be specified, but not both.

See https://docs.rs/humantime/latest/humantime for time format details.
";

    clap::Arg::new("rate")
        .long("rate")
        .groups(&["group-primary", "group-primary-rate"])
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
        .groups(&["group-primary", "group-primary-rate"])
        .conflicts_with("group-ramp")
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
        .group("group-ramp")
        .requires_all(&["ramp-rate-start", "ramp-rate-end"])
        .value_name("DURATION")
        .validator(validate::<humantime::Duration>)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--ramp-rate-start`.
fn arg_ramp_rate_start() -> clap::Arg<'static> {
    const SHORT: &str = "Starting rate for the ramp.";
    const LONG: &str = "\
Sets the starting request rate (in requests per second) for the throughput ramp
that will be used to begin the load test.

If this argument is specified, both --ramp-duration and --ramp-rate-end must
also be specified.
";

    clap::Arg::new("ramp-rate-start")
        .long("ramp-rate-start")
        .group("group-ramp")
        .requires_all(&["ramp-duration", "ramp-rate-end"])
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

If this argument is specified, both --ramp-duration and --ramp-rate-start
must also be specified.
";

    clap::Arg::new("ramp-rate-end")
        .long("ramp-rate-end")
        .group("group-ramp")
        .requires_all(&["ramp-duration", "ramp-rate-start"])
        .value_name("RATE")
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the positional [`clap::Arg`] for `<TARGET>`.
fn arg_target() -> clap::Arg<'static> {
    const SHORT: &str = "Load test target.";
    const LONG: &str = "\
Sets the target URL for the load test. HTTP and HTTPS URLs are supported.

If this argument is specified, both --ramp-duration and --ramp-rate-start must
also be specified.
";

    clap::Arg::new("target")
        .group("group-target")
        .value_name("URL")
        .validator(validate::url)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--multi-target`.
fn arg_multi_target() -> clap::Arg<'static> {
    const SHORT: &str = "Load test multiple targets.";
    const LONG: &str = "\
Sets one or more target URLs for the load test. HTTP and HTTPS URLs are supported.

This argument may be specified multiple times to specify multiple targets. The load
test will evenly distribute requests between the targets using round-robin.

This argument is incompatible with the <TARGET> positional argument.
";

    clap::Arg::new("multi-target")
        .long("multi-target")
        .group("group-target")
        .value_name("URL")
        .multiple_occurrences(true)
        .validator(validate::url)
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
also be specified.
";

    clap::Arg::new("http-method")
        .long("http-method")
        .value_name("METHOD")
        .possible_values(&[
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
        .validator(validate::file)
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
        .multiple_occurrences(true)
        .validator(validate::key_value)
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
signaller if a blocking-thread signaller is used (see --signaller-type).

This argument defaults to the number of cores on the host machine.
";

    clap::Arg::new("worker-threads")
        .long("worker-threads")
        .value_name("COUNT")
        .validator(validate::<usize>)
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

This argument is incompatible with --worker-threads and
--signaller=blocking-thread.
";

    clap::Arg::new("single-threaded")
        .long("single-threaded")
        .value_name("COUNT")
        .validator(validate::<usize>)
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
        // .validator(validate::validate::<usize>)
        .validator(validate::<usize>)
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

By default, a blocking-thread signaller will be used unless --max-rate has been
specified, in which case an on-demand signaller is used as maximum throughput
tests don't require timing signals.
";

    clap::Arg::new("signaller")
        .long("signaller")
        .value_name("NAME")
        .possible_values(&["blocking-thread", "on-demand", "cooperative"])
        .help(SHORT)
        .long_help(LONG)
}
