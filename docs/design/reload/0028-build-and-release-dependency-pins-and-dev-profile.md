# Dependency pins and the dev build profile

The workspace `Cargo.toml` configures three external dependencies directly in the manifest and
overrides the optimization level of `byond-scan` for dev builds:

- **`dreammaker` and `dap-types`:** Sourced from `https://github.com/auxtools/SpacemanDMM` (branch
  `master`), because dmasm's proc compiler relies on parser fixes that live in auxtools' fork of
  SpacemanDMM rather than upstream.
- **`byond-scan` pin:** Pinned to a tested Git revision directly in `Cargo.toml`, because external
  projects that depend on auxtools do not use auxtools' `Cargo.lock` and receive only pins declared
  in the manifest.
- **`byond-scan` dev optimization:** `[profile.dev.package.byond-scan]` compiles `byond-scan` with
  `opt-level = 3` in dev builds. auxtools locates BYOND's functions by scanning BYOND's code once
  for each pattern with `byond-scan`, which takes tens of seconds at the start of every debugged
  world when unoptimized.

## Applies to

- `Cargo.toml`: `[workspace.dependencies]`, `[profile.dev.package.byond-scan]`
