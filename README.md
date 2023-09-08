# cargo-deplint

A utility for enforcing rules about crate dependency graphs.

`cargo deplint` can be installed with `cargo install`. The resulting binary
should then be in `$HOME/.cargo/bin`.

```
$ cargo install cargo-deplint
```

Use it by providing the path to a Cargo.lock file and a lints file:
```
$ cargo deplint Cargo.lock deplints.toml
```

# Format of the lints file

The file defining the lints is a toml that looks as follows.

```toml
[[deny]]
name = "foo"
dependencies = [
    "bar",
]
```

The above disallows crate `foo` from depending on crate `bar`, including
transitively.

# TODO

- Return all indirect dep paths in deny output.
- Add features necessary to replace mz's [lint-deps.sh] (allowlist + ???).
- Consider using `cargo metadata` instead of the `Cargo.lock` file so that we
  can distinguish between regular dependencies and dev-dependencies.

[lint-deps.sh]: https://github.com/MaterializeInc/materialize/blob/v0.68.3/ci/test/lint-deps.sh
