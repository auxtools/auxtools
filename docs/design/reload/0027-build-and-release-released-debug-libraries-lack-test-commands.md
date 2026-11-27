# Keeping test console commands out of released debug libraries

The integration test suites inspect a running world through console commands that exist only when
the `debug_server/test-commands` Cargo feature is enabled. Released debug libraries must not include
these commands.

Each CI build job compiles and verifies both variants in order:

1. Build the release debug library without `debug_server/test-commands`.
2. Verify that it lacks the test commands by searching the binary for the text
   `asked for with #panic`.
3. Save that copy under `released/`.
4. Build the workspace again with `--features debug_server/test-commands` in a separate target
   directory (`target/test-commands`), take the debug library from it, and verify that it contains
   the test commands. Using a separate target directory avoids overwriting the first build, because
   the debug library's filename contains no hash and sharing one directory would prevent either
   build from staying cached across runs.

The build job packs copies of Cargo's output from a directory outside `target` so the cached build
retains its debug information. The artifact bundle passed to downstream jobs contains both
libraries: the test jobs use the build with test commands, while the `package` job unpacks the
`released/` copies, verifies again that neither contains the commands, and packs those into the VS
Code extension.

If the string `asked for with #panic` changes in the debug server, the checks in
`.github/workflows/ci.yml` must change with it so they continue distinguishing the two libraries.

## Applies to

- `.github/workflows/ci.yml`: `build-linux`, `build-windows`, and `package` jobs
- `ci/run-tests.sh`: expects the debug library built with test commands
