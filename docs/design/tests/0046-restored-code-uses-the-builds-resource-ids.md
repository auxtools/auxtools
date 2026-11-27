# Restored code using built resource IDs after a reboot

After a reboot, the launcher restores earlier hot reloads before the world has reloaded its
resources. Whether a restored reload applies immediately or waits until `world/New` depends on
whether its referenced resources exist in the build:

- **Resources in the build:** Restored code that names a resource file from the build uses the
  build's resource ID and applies immediately without waiting. `dap_test` checks that
  `icon_reloaded`, reloaded to return `'reload_built_2.dmi'`, still returns that icon's states
  after a reboot, and that the output does not contain "deferred until world/New".
- **Resources outside the build:** Restored code that names a resource file not in the build
  requires the loaded resource cache. The launcher reports "deferred until world/New because these
  resource files require the loaded resource cache: reload_new.dmi. Map atoms use the compiled code
  until then" and applies the reload when `world/New` starts.

## Applies to

- `tests/dap_test/src/groups/reloading.rs`: "restored reload names a built resource" check
- `debug_server/src/world_resources.rs`
