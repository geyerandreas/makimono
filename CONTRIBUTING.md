# Contributing to makimono

Thanks for your interest in improving **makimono**!
This guide walks through the development environment.

## Set up the Rust development environment

**Install Rust with `rustup`** (recommended)

Install `rustup` from <https://rustup.rs> if you do not already have it.
Then install the toolchain pinned in `rust-toolchain.toml`:

```bash
rustup show
```

Finally, add the common developer components:

```bash
rustup component add rustfmt clippy
```

If you are contributing new functionality, please include coverage via
**unit tests** (in `src/…` using `#[cfg(test)]`) or **integration tests** (under `tests/`).

Thanks again for contributing!
