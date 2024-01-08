# Worklog

- `cargo-expand` integration
  - followed [@tamuratak][gh-tamuratak]'s suggestion
  - moved all tests from `tests/` to `examples/`
  - modified `tests/progress.rs` to reflect new paths
  - now, can expand w/ `cargo expand --example 01-parse-header` and test with `cargo test --tests`

[gh-tamuratak]: https://github.com/tamuratak
