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

Every lint and preprocessor code needs at least one test. `bin/tests/code_coverage.rs` fails if a code in `libs/*/src/analyze/lints` or `libs/preprocessor/src/codes` is never referenced by a test or fixture. Most lints are tested with fixtures in their own crate. Lints that need a whole project, such as `L-S01` (it reads `requiredVersion` from `CfgPatches`), are tested in `bin/tests/lints.rs`.

## CLI Tests

The tests in `bin/tests/` run real commands against copies of the fixture workspaces in that folder, using the helpers in `bin/tests/common/mod.rs`:

```rust
mod common;

use common::{fixture, run, write};
use sealed_test::prelude::*;

#[sealed_test]
fn broken_sqf_fails_check() {
    let _directory = fixture("workspace_simple");
    write("addons/main/broken.sqf", "private _a = ;\n");
    run(&["check"]).fails();
}
```

`run` returns both the result and everything the command printed: `.succeeds()`, `.fails()`, `.prints(..)` and `.does_not_print(..)` show the full output when they fail. In debug builds a failing command returns an error instead of exiting the process, so tests can check the exit status. A command that can't do its job must return an error, not just log one, so test the failure cases too.

Use `#[sealed_test]` for these tests, because commands change the current directory and set up global state.

`common::cba_project()` writes a realistic CBA style project, with shared macros, `XEH_PREP` functions, a stringtable, an optional and an `include` folder. It checks with no diagnostics, so a test can add one problem and look for it. Most tests in `bin/tests/project.rs` use it, and look at what is built (PBO contents, properties, signatures, the release archive), not only at whether the command succeeded.

`TemporaryDirectory` points `HEMTT_ARMA3_PATH` at an empty `arma3` folder inside itself, so `hemtt dev` and other commands never create links in a real Arma 3 installation.
