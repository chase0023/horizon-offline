# Horizon with offline accounts

A modified build of the [Horizon launcher](https://github.com/horizon-rust/horizon) (Rust,
LGPL-3.0-or-later) that adds **first-class local offline profiles** next to the existing
Microsoft sign-in. An offline profile is a real launch path, not a hidden button: choosing one
starts the installed game with that player's name, and the game derives the normal Minecraft
offline UUID from it.

The Microsoft account path is unchanged. Offline profiles never create or read a Microsoft
token, never write a session file, and never bypass Xbox or Minecraft authentication.

## What you get

- **Microsoft account** — the existing device-code sign-in, owner check, session file and
  splitscreen support.
- **Offline account** — a local name (3–16 letters, digits or underscores), a local offline
  UUID, saved in `accounts.json` on this computer. No email, password, token or keychain item.
- **Profile management** — create, rename, delete (with confirmation), switch, and remember the
  last player. Names are validated and cannot duplicate another profile or a signed-in
  Microsoft account.
- **Account screens** — first-run/refused-Play **Choose how to play** screen with Microsoft
  sign-in beside the Offline panel, plus **Settings → Accounts** with the profile list and
  **Play as / Edit / Delete**. The top-bar chip shows the active player and account type.
- **Same worlds and settings** — switching account types does not move the game folder,
  options, worlds or instance. Only the player identity changes.

## Quick start (Windows 11 x64)

1. Run `dist/horizon-launcher.exe`.
2. On **Choose how to play**, type a name under **Offline account** and press **Create profile**
   (or sign in with Microsoft).
3. Press **Play**.

The compiled build is `dist/horizon-launcher.exe` (release, x86-64 Windows).

The launcher self-updates from this fork's own GitHub releases, never from Horizon's official
launcher releases. Game and Nightly updates still come from upstream. This keeps offline profiles
through launcher upgrades while still receiving fixes.

### Microsoft sign-in in this build

The code and tests for Microsoft sign-in are preserved, but the binary was built without an
Azure application client ID. Horizon's own registration may not be used by forks, so this build
reports **sign-in is not configured** until it is given a client ID of its own.

To enable Microsoft sign-in, register an Azure public client and set its Application (client)
ID, then rebuild (or set the same variable when running):

```powershell
$env:HORIZON_MSA_CLIENT_ID = "<your-azure-application-client-id>"
.\dist\horizon-launcher.exe
```

Mojang must also approve that application for the Minecraft services; see
[docs/microsoft-account.md](docs/microsoft-account.md) and
[docs/forking.md](docs/forking.md#microsoft-sign-in-in-a-fork). Offline profiles need none of
this.

## Offline launch path

`account::session_for_game` returns an offline `GameSession` without touching the keychain or
the network. The command builder (`GameSession::apply`, shared by the window, `play`, modpacks
and MCP) then adds only:

```text
horizon-client --name <profile name>
```

and removes inherited `HORIZON_SESSION_FILE`, `HORIZON_ACCOUNT_SESSIONS`,
`HORIZON_LAUNCHER_EXE` and `HORIZON_PLAYER_NAME`. The game sees no session, computes
`offline_uuid(name)` exactly as vanilla does, and plays offline. The owner gate is skipped for
an offline profile. See [docs/offline-accounts.md](docs/offline-accounts.md) for the full
contract and the audit trail of the change.

### Internet and multiplayer

| Action | Offline profile | Microsoft account |
| --- | --- | --- |
| Single-player worlds | Yes | Yes |
| LAN worlds | Yes | Yes |
| Authenticated (online-mode) servers | No — the server refuses the player normally | Yes, with an account that owns the game |
| Realms, friend invites, secure chat | No | Yes |
| Create/select an offline profile | Yes, no internet needed | n/a |
| Download/update Minecraft | No — prepare from an existing install | Yes |
| Play an already-prepared version | Yes, no internet needed | Session renewal may need internet |

There is no global “internet off” switch. Offline mode removes the account session; it does not
disable networking, mod downloads or LAN discovery.

## Command line

```text
horizon-launcher account list [--json]
horizon-launcher account offline-create <name>
horizon-launcher account offline-rename <old-name-or-uuid> <new-name>
horizon-launcher account offline-delete <name-or-uuid>
horizon-launcher account select <name-or-uuid>
horizon-launcher account sign-in
horizon-launcher play
```

`account list --json` includes `offline_profiles`, the selected profile and `plays_as`
(`kind: "offline"` or `"microsoft"`). The MCP `account_list` tool exposes the same data.

## Where data lives

- Launcher data: `%APPDATA%\horizon-launcher` (override with `HORIZON_LAUNCHER_HOME`).
  - `accounts.json` — Microsoft account names/UUIDs, `offline_profiles`, `selected_offline`.
  - `settings.json` — versions, instance, appearance and the first-run flags.
- Game data: `HORIZON_HOME`, else `%APPDATA%\horizon`; worlds are `saves/<world>` there.

`accounts.json` stays readable by upstream Horizon: the extra `offline_profiles` and
`selected_offline` fields are ignored. An upstream launcher will drop them when it next saves,
so back up the file if you switch launchers.

## Build from source (Windows)

Requires Rust stable (MSVC toolchain) and the Visual Studio C++ build tools.

```powershell
cd launcher
cargo build --release
.\target\release\horizon-launcher.exe
```

The launcher is its own Cargo workspace inside this repository; build it from `launcher/`, not
from the repository root. The full game can be built from the root with the workspace’s normal
Horizon instructions, but the launcher only needs the prebuilt/prepared game versions already
in its data folder (or a Minecraft install to prepare from).

To put your own Microsoft application ID into the binary:

```powershell
$env:HORIZON_MSA_CLIENT_ID = "<your-azure-application-client-id>"
cargo build --release
```

## Tests

The offline work is covered by unit tests in `launcher/src/account/offline.rs` and
`launcher/src/account/tests.rs`:

```powershell
cd launcher
cargo test --lib account::
cargo test --lib account::offline
```

They cover profile creation/reload, validation and duplicate rejection, rename (UUID follows the
name and selection follows the profile), delete and next-selection, switching between Microsoft
and offline without losing the other, the name-only launch (`--name`, no session file and no
inherited session variables), and that an offline launch never asks the account services.
Existing Microsoft-account, session-file, owner, refresh and splitscreen tests still run in the
same suite.

End-to-end check performed with this build on Windows 11 x64: create `Builder` with
`account offline-create`, then `play --world OfflineTest`; the game log showed
`Builder[/127.0.0.1:0] logged in with entity id 2` and the process exited 0. No Microsoft
sign-in, keychain or session-file step appeared.

## Layout

- `launcher/src/account/offline.rs` — profile type, name rules and UUID derivation.
- `launcher/src/account.rs` — account selection, offline CRUD, `GameSession` and the launch
  handoff.
- `launcher/src/offline_ui.rs` — the offline card, create/rename/delete controls, glass and
  legacy looks.
- `launcher/src/owner_page.rs`, `switch_page.rs`, `glass_account.rs`, `blocks_settings.rs`,
  `blocks_play.rs` — account-choice and account surfaces.
- `launcher/src/cli.rs`, `launcher/src/mcp.rs` — CLI and MCP account commands.
- `docs/offline-accounts.md` — user-facing offline mode details and limits.
- `dist/horizon-launcher.exe` — release build of this tree.

## Licence and attribution

This is a modified version of Horizon, licensed under **LGPL-3.0-or-later**. The original
copyright notices, `LICENSE`, `LICENSES/` and `THIRD_PARTY_NOTICES.md` are retained. See
[docs/upstream-readme.md](docs/upstream-readme.md) for the upstream project README.

Minecraft is a trademark of Mojang Studios. This project is not affiliated with or endorsed by
Mojang or Microsoft.
