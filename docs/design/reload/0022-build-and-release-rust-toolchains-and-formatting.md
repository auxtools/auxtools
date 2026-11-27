# Rust toolchains, formatting, and minimum Rust versions

auxtools compiles for the two targets BYOND runs on: `i686-unknown-linux-gnu` and
`i686-pc-windows-msvc`. Each CI build job builds one of those targets.

## Build toolchains

`rust-toolchain.toml` specifies the Rust version CI builds with along with both targets for local
development. Its channel must match `RUST_VERSION` in `.github/workflows/ci.yml`.

In CI, `RUSTUP_TOOLCHAIN` is set to that same version so Cargo uses the toolchain installed by the
job. If rustup followed `rust-toolchain.toml` directly, it would also fetch the target that the job
does not build. The `format` job checks that `RUST_VERSION`, `RUSTUP_TOOLCHAIN`, and the channel in
`rust-toolchain.toml` all agree, so any Rust version update must change `rust-toolchain.toml` and
both variables in `ci.yml` together.

## Formatting

Code formatting uses a pinned nightly toolchain (`RUSTFMT_TOOLCHAIN` in `ci.yml`) rather than the
build toolchain, because the workspace `.rustfmt.toml` relies on unstable rustfmt options (every
option after `unstable_features = true` is unstable). Locally, format with `cargo +nightly fmt`.

`dmasm/.rustfmt.toml` keeps rustfmt's defaults with edition 2018, preserving the formatting style
dmasm used as a separate repository. It overrides the workspace `.rustfmt.toml` for files under
`dmasm/`.

## Minimum Rust versions

External projects depend on `auxtools`, `auxtools-impl`, and `dmasm-bytecode` and should not be
forced onto a recent compiler. The workspace declares `rust-version` `1.95`, while `auxtools`,
`auxtools-impl`, and `dmasm-bytecode` declare `1.88`.

The `minimum-rust` job builds each group with its declared Rust version and checks that the
manifests match. Using a newer Rust language feature in any of those three dependency crates fails
the `1.88` build.

## Applies to

- `rust-toolchain.toml`
- `.rustfmt.toml`
- `dmasm/.rustfmt.toml`
- `.github/workflows/ci.yml`: `env` block, `format` job, and `minimum-rust` job
- `Cargo.toml`: `rust-version` field
- `auxtools/Cargo.toml`: `rust-version` field
- `auxtools-impl/Cargo.toml`: `rust-version` field
- `dmasm/bytecode/Cargo.toml`: `rust-version` field
