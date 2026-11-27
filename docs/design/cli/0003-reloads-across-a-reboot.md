# Restoring hot reloads, skins, and resources across a reboot

When a world reboots while hot reload is active, the new world reloads its `.dmb` (which may have
been rebuilt in the meantime and may already contain recent edits), and the reboot also clears
BYOND's resource cache, discarding any previously reloaded resources. auxtools therefore discards
stale reloads prepared before the reboot and restores skins, resources, and code changes in a strict
post-reboot order.

- **Stale reloads are skipped.** The debug adapter compares the world generation a reload was
  prepared for against the current generation. If they differ, it prints
  `"Hot reload skipped because the world restarted after it was prepared"` and does not send the
  reload. The edits are not lost, because the source worker's catch-up step prepares them again
  against the new world.
- **Edited skins are restored first.** After a reboot, edited skin files are sent before the world
  is released so clients logging in see the updated skin.
- **Resources are restored after world data loads.** Restoring resources earlier would be undone
  when the world loads its own resources. The world applies the restored resources after loading
  its own data and before atoms' `New()` procs run, so atoms created at startup see the edited
  files.
- **Resource reloads recompile their users.** A resource reload affects more than the file itself,
  because code and variable defaults refer to the resource's ID and existing appearances still use
  the old ID. Code and variable defaults referencing the reloaded resources are recompiled in the
  same reload, while the world caches the new resource IDs and updates appearances that use the old
  ones.

The post-reboot sequence must stay in this order: skin first, then the world's own resources, then
the restored resources, and finally the rest of the catch-up.

## Applies to

- `auxtools-cli/src/adapter.rs`: `code::Event::Reload` arm of `Adapter::code_event` (generation
  check)
- `auxtools-cli/src/code.rs`: `catch_up`, `restore_resources`, `reload_resources`, and the wait for
  `Command::Reloaded` in `reload`
- `debug_server/src/launched.rs`: `world_new`
- `debug_server/src/resource_reload.rs`: world-side resource reload handling
