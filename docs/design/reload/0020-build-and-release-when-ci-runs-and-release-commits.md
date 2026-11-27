# When CI runs and how it handles releases

The CI workflow (`.github/workflows/ci.yml`) cross-compiles the Linux and Windows binaries on Linux,
tests the BYOND matrix on Linux and under Wine in every run (`test-linux-wine`), tests on native
Windows when invoked by a release (`test-windows`, gated on `inputs.version`), packs both targets
into the VS Code extension, and builds with each minimum Rust version declared in the manifests (see
[Rust toolchains, formatting, and minimum Rust versions](0022-build-and-release-rust-toolchains-and-formatting.md)).
The [release workflow](0021-build-and-release-the-release-workflow.md) calls `ci.yml` to build and
test each release.

## Fork gating

Forks of the repository would otherwise run the full build and test matrix on every push. CI
therefore runs automatically only in repositories owned by `willox` or `auxtools`, enforced by the
jobs' `if` conditions. Other forks must trigger CI manually through `workflow_dispatch` (**Actions**
-> **CI** -> **Run workflow**).

## Release runs

A release creates two commits. The release commit differs from the `master` commit that CI has
already linted and built only in its version numbers. After the release, `release.yml` pushes a
"Start development on X.Y.Z-dev" commit as `github-actions[bot]`, differing from the tested release
tree only in its version.

When `release.yml` calls `ci.yml`, it passes the release version as `inputs.version` and the release
commits as the `release.bundle` artifact:

- The `build-linux`, `build-windows`, `test-linux-wine`, and `package` jobs check out the release
  tag from `release.bundle`.
- The `build-linux` and `build-windows` jobs skip restoring the crate cache, the Windows SDK cache,
  the Cargo build-output cache, and the step that restores file modification times.
- The `clippy` and `minimum-rust` jobs are skipped.

## Cache ownership

Only `build-linux` and `build-windows` save the downloaded-crates cache using `actions/cache`. The
`clippy` and `minimum-rust` jobs only restore it with `actions/cache/restore`, ensuring each cache
key has a single writer. `build-windows` also owns the Windows SDK cache.

## Applies to

- `.github/workflows/ci.yml`: `on:` triggers and job `if` conditions
- `.github/workflows/release.yml`: "Commit release and next development version" step
