# Contributing to presence-rs

presence-rs is small, and it holds every change to the same bar: tests
written first, every scenario of a feature pinned by a test, no function
over a CRAP score of 10, and no surviving mutant on a changed line. This
page is what you need to get a pull request through that.

The repository has no issue tracker. Send a pull request, even for a
question or an idea.

## Which kind of change is yours

**A bug fix.** Write the failing test first, then the smallest change that
makes it pass.

**A new feature or a change in behaviour.** It starts as a spec in
`specs/`: a short context section and Given/When/Then scenarios, one per
observable behaviour. Take the next number, follow the shape of the specs
already there, and open the spec as its own pull request. Implementation
starts once the maintainer approves it, and from then on the spec is the
acceptance criteria. A spec changes only with the maintainer's approval.

**Docs, comments, anything with no change in behaviour.** Send the pull
request. If a test fails because of it, it changed behaviour after all.

## Setup

Rust stable. The minimum supported version is 1.85, and CI checks it.

```bash
rustup component add rustfmt clippy
```

The coverage, CRAP and mutation gates use
[Keeler](https://github.com/minikin/keeler)'s recipes. To run them locally,
install `just`, `cargo-nextest`, `cargo-llvm-cov`, `cargo-mutants` and
`cargo-crap`, and put Keeler's `bin/keeler` on your `PATH`.

## Before you push

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

With Keeler installed, `keeler dev` runs all of that plus coverage and the
CRAP gate, and `keeler mutants-diff main` mutation-tests the lines you
changed. A surviving mutant means a test would still pass with that line
broken. Fix it with the test that catches it, never by reshaping the code
until the mutant goes away.

CI runs the same gates and adds:

- the tests on Linux, macOS and Windows, on stable and beta, and on nightly
  on Linux
- a check on Rust 1.85, and builds for a `no_std` target with each feature
- the tests under Miri
- a build against the lowest dependency versions `Cargo.toml` allows
- `cargo-semver-checks` against the latest release
- a docs build as docs.rs runs it
- `cargo audit`
- a CRAP comparison against `crap-baseline.json`, which fails when any
  function's score went up

## Tests

- Unit tests sit in a `#[cfg(test)]` block next to the code. When the code
  has an invariant (ordering, round-trip, precedence), add a `proptest`
  property beside the examples. If proptest finds a counterexample it
  writes a file under `proptest-regressions/`: commit it.
- `tests/acceptance.rs` holds one test per spec scenario, named after the
  scenario, with Given/When/Then comments.
- Every public item has a doc comment with an example, and the README's
  code blocks run as doctests.

## Features

`serde` and `schemars` are optional. The default build and the `serde`
feature must stay `no_std` without an allocator, and `tests/allocation.rs`
checks that. Code that needs `alloc` goes in a module behind its feature,
as `src/schemars.rs` does.

## Commits and pull requests

Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`,
`ci:` or `chore:`, then a lowercase summary of what changed. The body says
what changed and why, in a few sentences of prose. A pull request
description does the same: the problem, then the change and how it was
verified.

## Releases

The maintainer bumps the version in `Cargo.toml`, dates the CHANGELOG
entry, merges, and pushes a `vX.Y.Z` tag on `main`. The release workflow
checks the tag against the crate version, runs the tests and the `no_std`
and MSRV builds, and publishes to crates.io through trusted publishing.

## Code of conduct

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) applies to everyone who takes
part.
