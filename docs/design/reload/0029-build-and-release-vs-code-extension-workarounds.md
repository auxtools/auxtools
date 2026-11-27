# VS Code extension compatibility workarounds

The packaged VS Code extension starts the launcher as its debug adapter and populates debug
configurations, working around five behaviors of VS Code, other extensions, and DM syntax:

- **Executable bit on Linux:** Files inside a packaged extension can lose their executable
  permission on Linux, so `ensureExecutable` sets the launcher to mode `755` before running it
  unless it is already executable.
- **Missing `launch.json`:** When no `launch.json` exists, VS Code calls `resolveDebugConfiguration`
  with an empty configuration containing no `type`, `request`, or `name`. The extension treats that
  as the no-`launch.json` case and prompts for a world file.
- **DreamMaker language client paths:** The DreamMaker language client resolves its `CurrentDME` and
  `CurrentDMB` variables only inside its own debugger. The extension's configurations therefore call
  the language client's `dreammaker.getFilenameDme` and `dreammaker.getFilenameDmb` commands
  instead.
- **Combined `-D` define flags:** After variable substitution, the SS13 Build Flags extension's
  command gives multiple `-D` arguments in a single string. The extension splits any `defines`
  entry starting with `-D` into separate defines and drops empty entries.
- **Global hover expressions:** In DM, `::name` reads the global variable even when a local variable
  shares the same name. The hover expression search preserves a leading `::` so the debugger
  evaluates the global.

## Applies to

- `editors/vscode/extension.js`: `ensureExecutable`,
  `ConfigurationProvider.provideDebugConfigurations`, `resolveDebugConfiguration`,
  `resolveDebugConfigurationWithSubstitutedVariables`
- `editors/vscode/expression.js`: hover expression search
