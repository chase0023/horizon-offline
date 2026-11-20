# Offline accounts

This fork adds a second, first-class account type to Horizon's launcher: a **local offline
profile**. It is not a Microsoft account, it holds no token, and it never reaches the Microsoft,
Xbox or Minecraft services.

## What an offline profile is

- A name the player picks (3 to 16 letters, digits or underscores), saved in `accounts.json` in
  the launcher's data folder, with the offline UUID the game derives from that name
  (`UUID.nameUUIDFromBytes("OfflinePlayer:" + name)`, vanilla's rule) and a creation time.
- The profile that plays is remembered (`selected_offline`), so the next start offers the same
  player. Microsoft sign-ins stay signed in beside it, and switching back is one click.
- No email, password, Microsoft token, XUID or keychain item is created or read for an offline
  profile. Nothing private is written: `accounts.json` keeps names, UUIDs and which one plays.

## How Play reaches the game

The launcher's Play path is `account::session_for_game`: it loads `accounts.json`; when an
offline profile is selected it returns a `GameSession` with that profile and **no** session file
or environment entries, without asking the keychain or the network. The command builder
(`GameSession::apply`, used by the window, `horizon-launcher play`, modpacks and MCP `play`)
passes:

```
horizon-client --name <profile name>
```

and removes any inherited `HORIZON_SESSION_FILE`, `HORIZON_ACCOUNT_SESSIONS`,
`HORIZON_LAUNCHER_EXE` and `HORIZON_PLAYER_NAME`. The game sees no session, computes the
offline UUID from the name, and plays offline exactly as it already did when started without a
session. The owner gate and the keychain are not involved.

Single-player, creating and loading worlds, local settings and saves, and LAN worlds work. An
**online-mode** (authenticated) server still refuses an offline player: the game shows its
normal "This server is in online mode…" message. Horizon does not fake a token or a Microsoft
session, so an offline profile cannot reach authenticated Xbox or Minecraft services.

## What the launcher's UI offers

- First-run **Choose how to play** screen: Microsoft sign-in beside an Offline account panel
  with a name field, Create profile, and the profiles already saved.
- Settings → Accounts: who plays, the Microsoft accounts, and the Offline profiles card with
  **Play as**, **Edit** (rename), **Delete** (confirmed) and **Create profile**.
- The top-bar account chip and its menu list both kinds, with an `Offline` label.
- Play is enabled for an offline profile without a Microsoft owner. A version that is not
  prepared can be prepared from the player's own Minecraft install; an offline profile cannot
  download Minecraft from Mojang.

## Limits and deliberate choices

- Renaming a profile changes its UUID, because the game derives the offline UUID from the name.
  Worlds stay, but the renamed player is a new identity to worlds and servers (vanilla behaves
  the same way for an offline name change).
- Authentication is not bypassed: authenticated online servers and Xbox services still need a
  real Microsoft account that owns Minecraft: Java Edition. Online mod downloads, updates and
  server lists still need a working internet connection.
- Offline profiles do not appear in the Minecraft services, have no skin/cape profile, and
  cannot use Microsoft account features (Wardrobe cosmetics, Realms, friend invites, chat
  signing on secure-chat servers).
- The launcher updates itself only from this fork's own releases (`chase0023/horizon-offline`).
  It never installs an official Horizon launcher, which would not know offline profiles. Game and
  Nightly updates still come from upstream. Every launcher update is checked against a signed
  SHA-256 manifest before it is installed.
- `accounts.json` is the same file as upstream's; profiles are extra fields
  (`offline_profiles`, `selected_offline`) that an upstream launcher ignores and drops when it
  saves. Back up the file if switching launchers.
