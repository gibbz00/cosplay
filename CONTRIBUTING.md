# Contributing Guidelines

There are some things that should be kept in mind when contributing to this project.
- Commit messages must follow the [conventional commits](https://www.conventionalcommits.org) specification. This enables automated [CHANGELOG.md](CHANGELOG.md) generation by using [`git-cliff`](https://git-cliff.org).
- Each commit is should pass CI on its own.
- Bug fixes and feature additions should be accompanied by tests.

### Recommended developer tools:

##### LSPs

* [`typos-lsp`](https://github.com/tekumara/typos-lsp)
* [`taplo`](https://github.com/tamasfe/taplo)
* [`rust-analyzer`](https://github.com/rust-lang/rust-analyzer)
### Using pre-commit hooks

This project makes use of [`pre-commit`](https://pre-commit.com/) hooks. A `pre-commit install` is recommended once the repository has been cloned, unless it has been [auto-enabled](https://pre-commit.com/#automatically-enabling-pre-commit-on-repositories).

### Running tests

Pretty standard procedure apart from noting that some tests may be behind feature flags, so:

```sh
cargo test --all-features
```

### Generating and opening documentation

```sh
cargo doc --no-deps --all-features --open
```

## Release (for maintainers)

1. Make sure CI is not failing.
2. Bump version in the workspace `Cargo.toml` file.
3. Finally:

```sh
# $_release should match [0-9]+.[0-9]+.[0-9]+
_release='X.X.X'
git cliff 0.1.0.. -o CHANGELOG.md --tag $_release
git commit -am "chore: prepare $_release release"
git tag $_release
git push && git push --tags
```
