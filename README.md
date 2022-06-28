# Metron

Metron is an L7 performance profiler.

## Design

Example feedback loop for searching for max rate that meets throughput requirements (e.g., 99.999% latency < 5 ms).

```
Plan --> Signaller (thread) --> MPSC Channel --> Coord Task --> Client Tasks (worker pool) -->
MPSC Channel --> Control Task --> Plan
```

## TODO

- Check that the CLI makes sense and is consistent
- Check that the long help messages are just as helpful in terms of hints for enums (e.g., --log-level), range options (e.g., )
- Think about cli/ package structure, file names, etc
- Add CLI tests
