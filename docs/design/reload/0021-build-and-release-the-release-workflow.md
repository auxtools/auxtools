# Release workflow setup, checks, and step order

A release (`.github/workflows/release.yml`) is triggered manually from `master` with a version such
as `4.0.0`. It commits the release and the next development version, builds and tests them through
`ci.yml`, pushes the tag and `master`, and publishes the GitHub release and the VS Code extension.

## Required setup

The workflow depends on the following repository and marketplace configuration:

- A `release` environment restricted to `master`.
- A Visual Studio Marketplace Trusted Publishing policy for publisher `sohpea`, scoped to repository
  `auxtools/auxtools`, workflow `release.yml`, and environment `release`.
- An Open VSX trusted publisher for extension `sohpea.auxtools` with the same repository, workflow,
  and environment.
- A deploy key with write access and permission to bypass the rulesets protecting `master` and `v*`
  tags, with its private key stored in the `release` environment's `RELEASE_DEPLOY_KEY` secret.

## Validation and publishing order

The VS Code Marketplace rejects versions with a prerelease suffix. Before committing anything, the
prepare job validates the requested version: it checks that the version has the form
`MAJOR.MINOR.PATCH` with no prerelease suffix, is not older than `master`'s current version or the
latest release, and is not already tagged.

The Publish job follows these rules:

- It installs `vsce` and `ovsx` before pushing the tag and `master`, so a tool installation failure
  stops the release before anything is pushed.
- It pushes the tag and `master` atomically. If `master` advances before the push, a new workflow
  run is started.
- A failed Publish job can be retried to resume publishing. On retry, it skips pushing this run's
  tag if it was already pushed, keeps an existing GitHub release, and skips any extension version
  already published to the Marketplace or Open VSX.

## Version updates

`ci/set-version.py` handles version edits across Cargo and VS Code files:

- In `Cargo.lock`, workspace packages have no `source` line. The script relies on this to update
  only the versions of workspace crates that inherit the workspace version.
- In `package.json`, the script updates the version using a regular expression rather than rewriting
  the JSON so existing formatting is preserved, then verifies that the result parses and carries the
  new version.

## Applies to

- `.github/workflows/release.yml`
- `ci/set-version.py`
