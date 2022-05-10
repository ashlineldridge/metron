# Metron

Metron is an L7 performance profiler.

## Design

Example feedback loop for searching for max rate that meets throughput requirements (e.g., 99.999% latency < 5 ms).

```
Plan --> Signaller (thread) --> MPSC Channel --> Coord Task --> Client Tasks (worker pool) -->
MPSC Channel --> Control Task --> Plan
```

## TODO

1. Read and compose super rough error handling note:
  - https://blog.rust-lang.org/inside-rust/2021/07/01/What-the-error-handling-project-group-is-working-towards.html
  - https://www.reddit.com/r/rust/comments/obw7lu/what_the_error_handling_project_group_is_working/
  - https://nick.groenen.me/posts/rust-error-handling/
  - https://www.reddit.com/r/rust/comments/gj8inf/rust_structuring_and_handling_errors_in_2020/
  - https://github.com/dtolnay/thiserror (example [here](https://github.com/sharkdp/bat/blob/master/src/error.rs))
  - https://www.reddit.com/r/rust/comments/8dvldm/why_rusts_error_handling_is_awesome/
  - https://www.reddit.com/r/rust/comments/asf5h7/handling_multiple_error_types_in_a_large_web/

2. Implement advice from above in error handling for this project.

3. Make sure that we've got:
  - Pretty printing of errors (e.g., clap error colorization isn't lost)
  - Backtraces
  - Extensibility (cater for all future possible error types / scenarios)
  - Why use `enum Error` vs `struct Error` + `enum ErrorKind`?

4. Clean up and complete error handling note.

## Links

- https://github.com/clap-rs/clap/blob/master/examples/derive_ref/README.md
