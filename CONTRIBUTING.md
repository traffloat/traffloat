# Contributing guidelines

Traffloat is an open-source software.
To foster a comfortable environment for all contributors,
contributions should comply with the quality standards.
All contributions are welcome, even if they are as simple as a typo fix.

## Reporting a bug
Head to [Issues][issues] to report a bug.
Provide steps for reproducing, screenshots, console log and stack trace where appropriate.

## Proposing a feature
New features, especially big ones, should first be discussed at [Discussions][discussions]
to avoid wasting time writing code that will not be used.
However, small proof-of-concept patches could be helpful for the discussion if relevant.

Also see the [Vision](wiki-vision) document for a brief introduction
for the long-term principles for the development of this software.

## Contributing code
To contribute codw, you have to create a pull request:

1. [Fork][fork] the repository on GitHub
2. Create a branch for your change, most likely from the master branch.
3. Commit your changes on this new branch.
4. [Open a pull request][pulls] for review.

### Getting help
The [wiki][wiki] contains some technical documents explaining the codebase.
Also feel free to reach out at [Discussions][discussions] to ask about the code.

### Licensing
Traffloat is licensed under [Affero GNU Public License (AGPL) Version 3](LICENSE).
By creating a pull request, you agree to your code being distributed within Traffloat
under the same license.
If your code is taken from other parties or includes new libraries,
the code or new libraries must also be compatible with AGPL version 3.
(Most common open-source licenses,
including MIT License, Apache License, LGPL, GPL and AGPL,
are compatible with AGPL ersion 3).

### Well-defined scope
Avoid mixing unrelated changes in a pull request.
It slows down code review and increases the chance of rejection
if part of the other changes are inappropriate for merging.

### Code style
Traffloat is mostly written in Rust.
Every commit must be rustfmt-compliant
(we have a custom `rustfmt.toml` that specifies more strict styles).
Simply run `cargo fmt --all` to reformat the code.

Create a file `.git/hooks/pre-commit` with the following contents:

```shell
#!/bin/sh

# Called by "git commit" with no arguments.  The hook should
# exit with non-zero status after issuing an appropriate message if
# it wants to stop the commit.

test -z "$SKIP_COMMIT_CHECKS" || exit 0
cargo fmt --all -- --check || exit 1
cargo clippy --release --all || exit 1
cargo test --all || exit 1
```

If you are on MacOS/Linux, also `chmod +x .git/hooks/pre-commit`.
This script will automatically check code styles,
perform clippy lint and run tests before allowing a commit.
Under rare conditions (e.g. explicitly testing for a failing CI
or creating a temporary commit that will not be merged into master),
run `SKIP_COMMIT_CHECKS=1 git commit` for committing.

In addition to the rustfmt checks,
Traffloat also has the following code practices:

- Modules with submodules should be written as `name/mod.rs` instead of `name.rs`.
  While this is not consistent with common practices,
  it is the standard practice within the project for now.
- HTML styles should be written in-place using the `style!` macro.
  While this is not an ideal solution,
  it prevents some other problems (such as lack of compile time validation)
  that I am personally more concerned about.
  If you have better ideas, please participate in the discussion [here][discuss248].
- All comments and documentation must be written in American English for consistency.

### Tests
Ideally, all code should be unit-tested.
However, in the current early stage of development,
testing mechanisms are not yet completely available
(especially with complexity of dealing with the Legion ECS mechanism).

Please participate in the discussion [here][discuss249]
if you have any insights on more ergonomic unit testing approaches.

Meanwhile for now, manual tests and screenshots suffice
for mechanisms that require heavy environment setup,
but standalone components (e.g. mathematical functions) should use unit tests.

[discuss248]: https://github.com/traffloat/traffloat/discussions/248
[discuss249]: https://github.com/traffloat/traffloat/discussions/249
[discussions]: https://github.com/traffloat/traffloat/discussions
[fork]: https://github.com/traffloat/traffloat/fork
[issues]: https://github.com/traffloat/traffloat/issues
[pulls]: https://github.com/traffloat/traffloat/pulls
[wiki]: https://github.com/traffloat/traffloat/wiki
[wiki-vision]: https://github.com/traffloat/traffloat/wiki/Vision
