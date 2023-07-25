# Metron

Metron is a load testing toolchain. The request rate can be controlled precisely
and both fixed and linearly changing rates are supported. Metron has been my personal
project for exploring Rust concurrency and attempting to provide an implementation for
addressing [coordinated omission](https://www.youtube.com/watch?v=lJ8ydIuPFeU).

Metron aims to provide a general-purpose engine that executes jobs at a specified rate
and streams telemetry data in real-time. The rate at which Metron executes jobs can be
submitted as a declarative plan or set in real-time (e.g. via gRPC) when the Metron
profiler is running in server mode. Metron will support running both as a self-contained
CLI application and in a distributed mode where an arbitrary number of data plane Metron
instances are coordinated by a control plane and the data plane instances stream their
telemetry to a central observability backend. In addition to be able to specify the rate
at which Metron operates, Metron will also have a mode that accepts a target latency
value; it will then adjust the rate until it finds the maximum rate at which the target
latency can be achieved. This provides a straightfoward way of solving problems such as,
"My latency SLO is 50ms at 99.9%, what is the maximum throughtput I can support?"

## Profile CLI

```
> metron profile --help

metron-profile
Runs a performance test against the specified target(s) and produces a report.

The report can be written to stdout and/or streamed to a metrics backend.

USAGE:
    metron profile [OPTIONS] --rate <RATE>... --duration <DURATION>... --target <URL>...

OPTIONS:
        --rate <RATE>...
            Sets the desired throughput rate in terms of the number of requests per second
            (RPS) that should be generated for each segment of the test.

            This argument can receive multiple values and may be used to specify both fixed
            and variable rates. To specify a fixed rate, specify an integer value; e.g.
            --rate=100 --duration=15m implies a fixed rate of 100 RPS for 15 minutes. To use
            a variable rate, specify a range; e.g. --rate=100:200 --duration=15m implies
            that the request rate should increase linearly from 100 RPS to 200 RPS over a 15
            minute duration.

            To specify multiple test plan segments, where each segment can have its own rate
            and duration, specify multiple comma-separated values; e.g. --rate=100:500,500
            --duration=5m,15m will create a 20 minute test plan containing two segments: the
            initial segment will ramp the rate up from 100 RPS to 500 RPS over the first
            5 minutes and then the rate will be held at a constant 500 RPS for the next 15
            minutes. When multiple values are specified, both --rate and --duration must
            receive the same number of values.

        --duration <DURATION>...
            Sets the durations of each test segment.

            This argument can receive one or more values; the number of values specified
            must match the number of values passed to --rate. Each value defines the
            duration of the associated test segment.

            To specify multiple test plan segments, where each segment can have its own rate
            and duration, specify multiple comma-separated values; e.g. --rate=100:500,500
            --duration=5m,15m will create a 20 minute test plan containing two segments: the
            initial segment will ramp the rate up from 100 RPS to 500 RPS over the first
            5 minutes and then the rate will be held at a constant 500 RPS for the next 15
            minutes.

            A value of "forever" may be specified for fixed rate segments to indicate that
            the test should run forever or until CTRL-C is pressed. When specifying multiple
            test segments, "forever" can only be specified for the last segment. E.g.
            --rate=100,200 --duration=5m,forever will will create an infinite test plan
            containing two segments: the first segment will rate 100 RPS for 5 minutes and
            then the second segment will rate 200 RPS until it is interrupted. Variable rate
            segments are not allowed to have a value of "forever" as these segments must
            be able to be calculated over a finite duration.

            See https://docs.rs/humantime/latest/humantime for time format details.

        --target <URL>...
            Sets one or more target URLs for the performance profile. HTTP and HTTPS URLs
            are supported.

            This argument may be specified multiple times to specify multiple targets. The
            performance test will evenly distribute requests between the targets using round-robin.

        --http-method <METHOD>
            Sets the HTTP method to use when making requests of the target.

            If this argument is not specifed and no payload is specified (--payload or
            --payload-file) then HTTP GET will be assumed. If this argument is not specified
            and a payload is specified then HTTP POST will be assumed.


            [default: get]
            [possible values: get, post, put, patch, delete, head, options, trace, connect]

        --payload <PAYLOAD>
            Sets the HTTP payload string to use when making requests of the target.

            If a payload-based HTTP method such as POST or PUT has been specified
            (--http-method), and no payload has been specified (--payload or --payload-file)
            then an empty payload will be used.

        --payload-file <FILE>
            Sets the HTTP payload file to use when making requests of the target.

            If a payload-based HTTP method such as POST or PUT has been specified
            (--http-method), and no payload has been specified (--payload or --payload-file)
            then an empty payload will be used.

        --header <K:V>...
            Sets the specified header to be included in all requests. The value for this
            argument should be in K:V format, where K is the header name and V is the
            header value.

            This argument can be specified multiple times.

        --worker-threads <COUNT>
            Sets the number of worker threads to be used by the runtime to COUNT.

            The worker threads are the set of threads that are cooperatively scheduled to
            perform the load test. This number does not include the thread allocated to the
            signaller if a blocking signaller is used (see --signaller).

            This argument defaults to the number of cores on the host machine.

        --single-threaded
            Forces all operations to run on the main thread.

            The utility of this argument is unknown beyond providing interesting data on how
            the number of threads affects performance of the tool itself. This argument
            forces all operations to run on the main thread whereas --worker-threads=1 will
            result in the main thread creating a single worker thread to perform the
            requests.

            This argument is incompatible with --worker-threads and --signaller=blocking.

        --connections <COUNT>
            Sets the number of TCP connections that should be used.

            TODO: Elaborate.


            [default: 1]

        --signaller <NAME>
            Selects the type of signalling system that should be used to generate request
            timing signals. This is an advanced feature and the default behaviour will
            generally be what you want.


            [default: blocking]
            [possible values: blocking, cooperative]

        --no-latency-correction
            Disables latency correction that accounts for coordinated omission.

            When latency correction is enabled, the latency that is recorded for each
            request is calculated from when the request was scheduled to be sent, rather
            than when it was actually sent. This helps to account for the phenomenon
            known as "Coordinated Omission". Latency correction is enabled by defeault.

        --stop-on-client-error
            Sets whether the profiling operation should stop if the client encounters an
            error when sending requests to the target(s). This setting only affects *client-
            side* errors (e.g. too many open files) and not HTTP error statuses returned by
            the target(s).

            See --stop-on-http-non-2xx for setting HTTP status stopping behaviour.

        --stop-on-non-2xx
            Sets whether the profiling operation should stop if a non-2XX HTTP status is
            retured.

            See --stop-on-client-error for setting error stopping behaviour.

        --log-level <LEVEL>
            Sets the minimum logging level. Log messages at or above the specified
            severity level will be printed.


            [default: off]
            [possible values: off, info, debug, warn, error]

        --config-file <FILE>
            All commands allow a configuration file to be used as an alternative to
            individual command line arguments. Stdin can also be used by specifying
            a hyphen as the file name (i.e. `--config-file -`).

            When both a configuration file and individual command line arguments are used,
            the arguments will override their counterpart properties in the configuration
            file.

            See --print-config for bootstrapping a configuration file.

        --print-config
            Generates the configuration for this command and prints it to stdout. This may
            be used to bootstrap a configuration file based on command line arguments so
            that a configuration file can be used rather than individual command line
            arguments.

    -h, --help
            Print help information
```

## Echo Server CLI

The server is basically featureless at the moment.

```
> metron server --help

metron-server
Runs an echo server that may be used within performance profile tests.

This command starts a echo server that may be configured in terms of its
responses, latency, and other properties.

USAGE:
    metron server [OPTIONS]

OPTIONS:
        --log-level <LEVEL>
            Sets the minimum logging level. Log messages at or above the specified
            severity level will be printed.


            [default: info]
            [possible values: off, info, debug, warn, error]

        --port <PORT>
            Sets the server listening port to PORT. Defaults to 8000.


            [default: 8000]

        --worker-threads <COUNT>
            Sets the number of worker threads to be used by the runtime to COUNT.

            If this value is not specified it will default to the number of cores on the
            host machine.

        --config-file <FILE>
            All commands allow a configuration file to be used as an alternative to
            individual command line arguments. Stdin can also be used by specifying
            a hyphen as the file name (i.e. `--config-file -`).

            When both a configuration file and individual command line arguments are used,
            the arguments will override their counterpart properties in the configuration
            file.

            See --print-config for bootstrapping a configuration file.

        --print-config
            Generates the configuration for this command and prints it to stdout. This may
            be used to bootstrap a configuration file based on command line arguments so
            that a configuration file can be used rather than individual command line
            arguments.

    -h, --help
            Print help information
```
