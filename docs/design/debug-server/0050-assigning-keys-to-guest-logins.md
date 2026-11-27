# Assigning keys to guest logins

BYOND allows only one client per key, closing the first client's connection and handing its mob to
the second when two clients log in with the same key. Because clients not signed in to BYOND join as
guests named `Guest-NUMBER` from their `computer_id`, multiple guests from one computer receive the
same key and disconnect each other. To let a developer connect several guest clients to a local
world, the debug library patches BYOND's guest key formatting call on demand so upcoming guest
logins receive assigned keys instead.

## Intercepting BYOND's guest formatter

In BYOND's login-handling function, when a client sends `Guest` (or `Guest-` followed by letters and
digits), the server builds the guest's key by calling `StringEditor::AddPrintf` with the format
string `"Guest-%u"` (`"Guest-%lu"` on BYOND builds `515.1637–515.1642`) and a 32-bit hash of the
client's ID equal to `client.computer_id`. The resulting key then passes through BYOND's ordinary
key check, which rejects keys longer than 30 characters before any `|` (closing the connection
before `/client/New()` runs at 31 characters), keys with no alphanumeric character (empty ckey), and
keys containing `<`, `>`, `"`, `\`, or byte `0xFF`. BYOND derives `client.ckey` from the formatted
key in its usual way (for example, `Ada Lovelace` gives `adalovelace`), and `client.key`,
`client.ckey`, and `mob.key` reflect the result.

BYOND's binaries reference the `Guest-%u` format from four sites. Only the call in the server's
login function is patched. What the other three sites are for is not established.

The registry entry `guest_key_formatter_call` resolves to the 4-byte relative operand of that `call`
instruction across all 52 supported BYOND builds on each platform:

- **Windows (`byondcore.dll`), all supported builds:** one signature, operand at offset 7.
- **Linux (`libbyond.so`), `515.1644` and later:** one signature, operand at offset 13.
- **Linux (`libbyond.so`), `515.1637–515.1643`:** a separate signature because those builds keep
  frame pointers, operand at offset 25.

Each signature matches exactly once in every supported build on its platform, and `check_recipes`
checks `guest_key_formatter_call` like any other recipe. Each signature fixes the byte immediately
before the operand as `0xE8` (a near `call`), and the debug library verifies that `0xE8` byte again
before patching.

In all three shapes, the caller passes three arguments on the stack in C order: the string being
built, the format string, and the numeric ID hash. `formats_guest_key` is an `extern "C"` function
with that signature. When called, it invokes BYOND's original formatter and returns whatever the
formatter returns:

- If a key is waiting and the format string passed by BYOND starts with `Guest-%`, it passes the
  waiting key in place of the format string with every `%` doubled, so `StringEditor::AddPrintf`
  copies the key literally and ignores the numeric argument.
- If no key is waiting, or if the format string does not start with `Guest-%`, it passes BYOND's own
  format string through unchanged so the guest receives `Guest-NUMBER`.

## Lazy patching and world lifecycle

`take_the_call` patches the call site the first time a key is requested rather than when the world
loads, so a world that no one joins this way is never modified. To perform the write, `writing`
temporarily makes the code page writable and restores its protection afterward: on Linux it calls
`mprotect` to switch to read-write-execute and back to read-execute, and on Windows it uses
`region::protect_with_handle`.

When a world ends, a `#[shutdown]` function calls `give_the_call_back` to restore BYOND's original
call operand, because the debug library can be unloaded while the host process continues running.
When the next world loads, an `#[init(partial)]` function calls `take_the_call` again if any key was
ever requested.

## Queue order, reboots, and reconnects

`KEYS` holds two lists:

- `given`: every key ever requested since the process started, in the order first requested, without
  duplicates.
- `waiting`: keys queued for upcoming guest logins, taken from the front.

`give_next_guest` pushes a newly requested key to the front of `waiting` (so when several keys are
waiting, the most recently requested one is given first) and records it in `given` if it is not
already present. Each waiting key goes to a single guest login. A signed-in client does not trigger
the guest formatter, so it joins with its own key and leaves `waiting` unchanged.

`give_keys_again` refills `waiting` from `given` in first-requested order. It runs both when a world
ends and when `Server::handle_request` handles `Request::Reconnect`. Because guests from one
computer report the same `computer_id` hash, the server cannot tell returning guests apart after a
reboot or reconnect and therefore cannot match each client to its previous key. Reissuing the full
`given` list keeps the returning guests distinct so they stay connected, even though which client
receives which key depends on login order. As a consequence, a key whose original client has since
closed is still reissued after a reboot or reconnect to whichever guest logs in next.

## Validation, protocol, and adapter interface

Key validation lives in a single function, `guest::unusable` in `auxtools-protocol`: at most 30
characters (`LONGEST_KEY`), printable ASCII only, at least one letter or digit, and none of `<`,
`>`, `"`, `\`, or `|` (`|` splits off a gender suffix in BYOND). `%` is allowed and handled by
doubling it before formatting. Both the debug adapter (`Adapter::give_next_guest`) and the debug
library validate keys with `guest::unusable` before sending or accepting them.

`Request::GuestKey { key }` and `Response::GuestKey(Result<(), String>)` were appended at the end of
`Request` and `Response`, leaving `PROTOCOL_VERSION` unchanged as permitted by
[protocol versioning and wire compatibility](../cli/0005-protocol-compatibility.md). The feature is
exposed only through the debug adapter, with no CLI command. The custom DAP request `auxtoolsJoin`
takes an optional `ckey` argument (handled by `Adapter::join` and `JoinArguments`), allowing any DAP
client to assign a valid key to the next guest login before joining.

VS Code's `joinAs` command (**auxtools: Join with DreamSeeker as a New Guest**, command ID
`auxtools.joinAs`) does not prompt for a key or run its own validation. Instead, it generates a key
in BYOND's guest form (`Guest-` followed by a random 32-bit integer from `crypto.randomInt` below
`GUEST_NUMBERS`, 2 to the power of 32) and sends it in `auxtoolsJoin`. Because the generated key
always satisfies `guest::unusable` and preserves the guest form, the world applies its normal guest
handling while keeping each guest distinct.

## Applies to

- `auxtools-protocol/src/guest.rs`: `LONGEST_KEY`, `Unusable`, `unusable`
- `auxtools-protocol/src/messages.rs`: `Request::GuestKey`, `Response::GuestKey`
- `debug_server/src/guest_key.rs`: `give_next_guest`, `give_keys_again`, `take_the_call`,
  `give_the_call_back`, `formats_guest_key`, `KEYS`, `writing`
- `debug_server/src/server.rs`: `Server::handle_request`
- `auxtools/src/registry/linux.rs`: the `guest_key_formatter_call` recipes
- `auxtools/src/registry/windows.rs`: the `guest_key_formatter_call` recipe
- `auxtools-cli/src/adapter/join.rs`: `JoinArguments`, `Adapter::give_next_guest`, `Adapter::join`
- `editors/vscode/extension.js`: `joinAs`, `GUEST_NUMBERS`
