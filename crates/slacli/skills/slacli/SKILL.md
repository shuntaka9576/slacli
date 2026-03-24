---
name: slacli
description: Operate Slack from the terminal — send messages, reply to threads, delete messages, update status/presence, and edit profiles. Use this skill whenever the user mentions Slack messaging, posting to a channel, replying to a thread, changing Slack status, editing their Slack profile, or references a Slack message URL. Also trigger when the user wants to send something on Slack, change their status, or interact with Slack in any way.
license: MIT
compatibility:
  - claude
  - codex
allowed_tools:
  - Bash(slacli:*)
  - Read
---

# slacli

Rust-based Slack CLI. Requires prior setup via `slacli init`.

## Setup

```bash
slacli init
```

Prompts interactively for profile name, description, and tokens (Bot `xoxb-` / User `xoxp-`). Configuration is split into two files:

- `$XDG_CONFIG_HOME/slacli/config.toml` — profile metadata and channel aliases (safe to commit to dotfiles)
- `$XDG_CONFIG_HOME/slacli/credentials.toml` — tokens only (permission 0600, do NOT commit to avoid token leakage)

You can enter only one token at a time — existing tokens are preserved (merge mode).

### Channel Aliases

Define channel aliases in `config.toml` under each profile so AI agents can easily find the right channel.

```toml
[profiles.work.channels]
dev = { id = "C01ABCDEF", description = "Dev team channel" }
general = { id = "C02XYZXYZ", description = "General announcements" }
```

Then use `slacli chat send --channel dev --text "hello"` instead of the full channel ID.

To discover available channel aliases, run `slacli config --see`.

## Multi-profile support

Multiple Slack workspaces can be managed via profiles. Use `--profile <name>` to switch:

```bash
slacli --profile personal chat send -c general -t "hello"
```

If `--profile` is omitted, `default_profile` from `config.toml` is used.

## Before any operation

Run `slacli config --see` first (without `--profile`) to discover all profiles and their channel aliases. This is important because channel aliases and profile names vary per user's setup, and without this context you risk sending to a wrong channel or using a nonexistent profile. When the user's request spans multiple profiles (e.g., "all channels"), operate on every profile — not just the default.

## Confirmation required before sending messages

Before executing `slacli chat send`, use AskUserQuestion to confirm the destination profile and channel with the user. Sending a message to the wrong channel is irreversible and potentially embarrassing — especially in a work Slack — so always verify before sending. Present the available profiles and their channel aliases (from `slacli config --see`) as choices so the user can select the destination.

## Commands

### Show configuration

```bash
slacli config --see
```

- Shows current configuration as JSON (channel aliases, profiles, editor, etc.)
- Does NOT include tokens
- Use this command to find channel aliases and profile information
- With `--profile`: shows only that profile's settings

### Send a message

```bash
slacli chat send --channel <CHANNEL_ID_OR_ALIAS> --text "Hello from slacli"
```

- Requires Bot Token (`xoxb-`, scope: `chat:write`)
- `--channel` accepts a channel ID (e.g. `C01ABCDEF`) or an alias defined in `config.toml`
- Output: raw JSON from `chat.postMessage` API

#### Reply to a thread

```bash
slacli chat send --channel <CHANNEL_ID_OR_ALIAS> --thread-ts <TS> --text "Reply text"
```

- `--thread-ts` (`-T`) specifies the parent message's `ts` to reply under
- Combine with `jq` to send and reply in one line:

```bash
ts=$(slacli chat send -c general -t "Parent" | jq -r '.ts')
slacli chat send -c general -t "Reply" -T "$ts"
```

#### Sending to multiple profiles

When the user wants to send to channels across different profiles, run separate commands per profile:

```bash
slacli chat send --channel times --text "hello"
slacli --profile personal chat send --channel personal --text "hello"
```

### Delete a message

The bot can only delete messages it posted — attempting to delete another user's message will fail with `cant_delete_message`.

```bash
slacli chat delete --channel <CHANNEL_ID_OR_ALIAS> --timestamp <TS>
```

- Requires Bot Token (`xoxb-`, scope: `chat:write`)
- `--channel` accepts a channel ID or alias
- `--timestamp` is the Slack message `ts` value (e.g. `1710000000.000100`)
- Use `slacli logs --type chat-send` to find the `ts` of a previously sent message
- Output: raw JSON from `chat.delete` API

### View sent message logs

```bash
slacli logs --type chat-send
```

- Shows locally stored logs of messages sent via `slacli chat send`
- Output: JSONL (one JSON object per line)
- Log data is stored at `$XDG_STATE_HOME/slacli/logs/chat-send.jsonl`
- The `ts` field can be used with `slacli chat delete --timestamp <TS>`

### View profile edit logs

```bash
slacli logs --type profile-edit
```

- Shows locally stored logs of profile changes made via `slacli profile edit`
- Output: JSONL (one JSON object per line)
- Log data is stored at `$XDG_STATE_HOME/slacli/logs/profile-edit.jsonl`
- Since `users.profile:read` scope is not granted, these logs serve as the only way to look up past profile state

### Purge logs

```bash
slacli logs --type chat-send --purge
slacli logs --type profile-edit --purge
```

- Removes all logs of the specified type

### Edit profile / status

```bash
# Set status
slacli profile edit --set status_text="In a meeting" --set status_emoji=":calendar:"

# Set status with expiration (absolute Unix timestamp, not relative)
slacli profile edit --set status_text="Lunch" --set status_emoji=":fork_and_knife:" --set status_expiration=1710763200

# Clear status
slacli profile edit --set status_text="" --set status_emoji=""

# Change name / display name
slacli profile edit --set first_name="Taro" --set last_name="Yamada"
slacli profile edit --set display_name="shuntaka"
```

- Requires User Token (`xoxp-`, scope: `users.profile:write`)
- `--set` (`-s`) accepts any `users.profile.set` field as `key=value` (repeatable)
- Omitted fields are left unchanged
- Output: raw JSON from `users.profile.set` API (includes full profile in response)

## Error handling

slacli outputs errors to stderr as JSON (`{"ok": false, "error": "...", "detail": "..."}`). Common errors and how to handle them:

- `not_authed` / `invalid_auth` — token is missing or invalid. Suggest the user run `slacli init` to configure tokens
- `channel_not_found` — channel ID or alias doesn't exist. Run `slacli config --see` to verify available channels
- `cant_delete_message` — bot can only delete messages it posted

If any command fails with `not_authed` on first use, the user likely hasn't run `slacli init` yet. Guide them through the interactive setup.

## Tips

### Identify a message from a Slack URL

Given a Slack message URL like `https://workspace.slack.com/archives/<CHANNEL_ID>/p<TS_WITHOUT_DOT>`:

- **channel**: the path segment after `/archives/` → `<CHANNEL_ID>`
- **ts**: the `p`-prefixed number with a dot inserted before the last 6 digits (e.g. `p1234567890123456` → `1234567890.123456`)

These values can be used with any command that takes `--channel` and a timestamp.

```bash
# Reply to a thread
slacli chat send --channel <CHANNEL_ID> --thread-ts <TS> --text "Reply from slacli"

# Delete the message
slacli chat delete --channel <CHANNEL_ID> --timestamp <TS>
```

### Retrieve information from logs

slacli operates with minimal Slack API scopes. Logs provide a way to retrieve information that would otherwise require additional scopes (e.g. `users.profile:read`).

#### Identify a sent message

Since `channels:history` scope is not granted, sent message logs are the only way to look up a previously sent message's `ts`.

```bash
slacli logs --type chat-send | jq -r '[.ts, .channel, ._slacli_profile, .message.text] | @tsv'
```

Returns `ts`, `channel`, and `profile` for each sent message. Use these to delete a message or reply to its thread.

#### Look up profile info

Since `users.profile:read` scope is not granted, logs are the only way to retrieve past profile state.

```bash
slacli logs --type profile-edit | tail -1 | jq '.profile | {display_name, status_emoji, status_text}'
```
