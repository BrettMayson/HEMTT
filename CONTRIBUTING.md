# Contributing

## Testing

When running tests locally, it is highly recommended to use `cargo-nextest`.

It runs the tests much faster, and can detect slow and leaky tests, among other features.

```bash
cargo install cargo-nextest
cargo nextest run
```

## Lint Tests

Tests for lints are using [insta](https://docs.rs/insta) for snapshot testing.

When you make changes to the lints, that causes tests to fail, you will need to review the changes.

```bash
cargo install cargo-insta
# Run the tests before reviewing, you can just run an individual test, or use any testing tool
cargo nextest run
# Review the changes
cargo insta review

# Alternatively, you can run the tests with insta to review the changes in one command
cargo insta test --review
```

When reviewing changes with ANSI characters, you can press `d` to toggle between the diff and a rendered view.
