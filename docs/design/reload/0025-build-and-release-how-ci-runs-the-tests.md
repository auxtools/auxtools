# How CI runs the test suites

The CI build jobs compile the test binaries once and hand them to other machines to run against each
tested BYOND build. Pull requests and pushes to `master` test ten builds (`515.1637`, `515.1644`,
`515.1647`, `516.1652`, `516.1657`, `516.1659`, `516.1670`, `516.1674`, `516.1685`, and `516.1689`),
while a release tests all 52 supported builds listed in `ci/supported-builds.txt`.

## Build matrix and test environments

The ten-build matrix covers the oldest and newest working supported builds of each major BYOND
version (`515.1637`, `515.1647`, `516.1652`, and `516.1689`), plus builds where auxtools or dmasm
changes behavior. auxtools 4.0.0 fails on ten of the 52 supported builds, which are listed in
`ci/known-broken-builds.txt`.

- **Linux and Wine test service:** The project's test service (`auxtools-cloud`, containers on
  Cloudflare) runs the test suites on Linux and under Wine for all 52 builds in 7–10 minutes, and
  for the ten builds in 4–6 minutes. The "Test on Linux and Wine" (`test-linux-wine`) job uploads
  the two build bundles and the commit's source with `ci/cloud.sh`, waits for completion, and passes
  or fails with the service. The service runs the Linux and Wine suites for each BYOND build, in a
  container each, and checks the registry's patterns against each build's Linux and Windows files.
  The job writes no comments or checks, and its only permission beyond reading the repository is
  `id-token: write`.
- **Fork pull requests:** The test service authenticates GitHub jobs through their OIDC identity
  token, which names the repository running the job. Pull requests from forks do not receive this
  token. `test-linux-wine` does not run on them. A fork pull request's world tests run after merge
  or when its author starts CI manually in a repository the service allows.
- **Release retries and known-broken builds:** In a release, a failure on a build listed in
  `ci/known-broken-builds.txt` does not fail the job, and a failed job is retried twice rather than
  once.
- **Native Windows tests:** The Wine tests execute the same Windows binaries on the same BYOND
  builds as native Windows tests. Native Windows tests (`dd.exe` on Windows Server 2022, in
  `test-windows`) run only during a release, on the ten builds. Elsewhere the Wine tests stand in
  for them, so a regression that happens only on real Windows surfaces first in a release run rather
  than in a pull request.
- **Unit tests:** Unit tests run in the build jobs on GitHub's runners. The test service runs only
  suites that require a BYOND build.

## DreamMaker bytecode comparison

On every tested BYOND build, the test service also compares dmasm's generated code with DreamMaker's
for each case in `dmasm/compiler/tests/dreammaker`. A test case can restrict itself to a range of
builds with a `Builds: A-B` comment and is skipped on all other builds. Only `515.1637`, `515.1647`,
`516.1659`, `516.1685`, and `516.1689` together execute every case, so dropping any of those five
builds from the matrix can leave comparison cases unrun.

The comparison runs as its own job inside the service so a dmasm failure does not stop the test
suites.

## Test runner and helper scripts

- **Parallel isolation (`ci/run-tests.sh`):** Each integration test compiles and edits its world in
  place under `tests/*_host` in the current directory. `ci/run-tests.sh` runs integration tests in
  parallel by giving each test its own directory (`target/run-tests/TEST`) containing a copy of the
  test worlds. Setting `SERIAL=1` runs tests sequentially from the repository root with no time
  limit.
- **Per-test timeouts:** A test's output is buffered until it finishes, so if the CI job's own time
  limit ended the step, the output would never be shown. `ci/run-tests.sh` enforces its own timeout
  (`TIMEOUT` seconds, `900` by default), terminates any test that exceeds it, and prints the
  buffered output.
- **Unit test layout (`ci/unit-tests.sh`):** Cargo runs a package's unit tests with that package's
  directory as the working directory, and tests look for the launcher in their own directory.
  `ci/unit-tests.sh` collects unit test binaries grouped by package directory and names a binary
  crate's unit test executable `NAME-bin` so it does not collide with the launcher's filename.
  `ci/run-tests.sh` changes into each package directory before running those binaries so tests built
  on one machine run as Cargo would run them.
- **Clippy deduplication (`ci/clippy.sh`):** Cargo reports a warning in a file compiled into two
  binaries once for each binary. `ci/clippy.sh` deduplicates repeated rendered messages before
  counting warnings. It reports warnings without failing on them and fails only when Cargo itself
  fails.

## Build caching and file timestamps

Cargo rebuilds a workspace crate whenever any of its files is newer than the build output, and a
fresh checkout gives every file a current timestamp. Content-based freshness
(`-Zchecksum-freshness`) is unstable in Cargo 1.99.1. With only dependencies cached, a build job
took 10 minutes instead of 12. In addition, a build script watching a directory with
`cargo:rerun-if-changed=DIRECTORY` (such as the one in the `auxtools` crate) reruns whenever the
directory or anything inside it is newer than its output. Because the debug library's filename
contains no hash, building it with different features in the same target directory forces Cargo to
rebuild the library and the test programs that depend on it (see
[keeping test console commands out of released debug libraries](0027-build-and-release-released-debug-libraries-lack-test-commands.md)).

To avoid full rebuilds outside a release:

- The build jobs cache Cargo's build outputs (`deps`, `build`, `.fingerprint`, and `examples` across
  both target directories), keyed by Rust version, `Cargo.lock`, and commit. Each run restores the
  newest cache matching its Rust version and `Cargo.lock`.
- `ci/source-times.sh` sets each file and directory whose contents match the cached run (by Git blob
  and tree IDs) back to the timestamp it had in that run, so Cargo rebuilds only changed crates and
  their dependents. The list of contents and timestamps is cached alongside Cargo's output. Each
  file keeps the timestamp of the run in which its contents were first seen rather than one fixed
  old timestamp, because otherwise a crate that was not built in a run would appear already built
  from files that changed in that run.
- The steps that strip and pack binaries operate on copies outside `target`, because Cargo's output
  binaries are hard links into `deps` and stripping one in place would strip the cached file as
  well.
- A release uses no cache and builds everything from scratch.

A cached run with no changed crates took 6 minutes (2.5 minutes for the Linux build job, under 2
minutes for the Windows build job, and 4 minutes for the tests on the ten builds), compared to 16
minutes for a run with nothing cached (10 of them in the Linux build job). Editing a crate that most
of the workspace depends on (`auxtools` or the dmasm crates) still rebuilds most of the workspace.

## Applies to

- `ci/run-tests.sh`
- `ci/unit-tests.sh`
- `ci/clippy.sh`
- `ci/cloud.sh`
- `ci/source-times.sh`
- `ci/supported-builds.txt`
- `ci/known-broken-builds.txt`
- `.github/workflows/ci.yml`: build jobs, `test-linux-wine`, and `test-windows`
- `dmasm/compiler/examples/compare_dreammaker`
