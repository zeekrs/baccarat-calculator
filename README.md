# Baccarat Calculator

Reusable Rust baccarat calculator workspace extracted from `baccarat-next`.

## Crates

- `crates/calculator`: pure baccarat probability, EV, default odds, and settlement logic.
- `crates/types`: shared baccarat card and bet type definitions used by `calculator`.

## Use From Another Project

Until this is published as a crates.io package, consume it through Git:

```toml
[dependencies]
calculator = { git = "<your-git-repo-url>", package = "calculator" }
```

If Cargo cannot resolve workspace path dependencies from your Git source, depend on the workspace member directly:

```toml
[dependencies]
calculator = { git = "<your-git-repo-url>", package = "calculator" }
types = { git = "<your-git-repo-url>", package = "types" }
```

## Verify

Run from the workspace root:

```sh
cargo test -p calculator
cargo check -p calculator
```
