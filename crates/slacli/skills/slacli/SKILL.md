---
name: slacli
description: Operate Slack from the terminal — send messages, delete messages, update status/presence. Use when the user wants to interact with Slack.
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
- `$XDG_CONFIG_HOME/slacli/credentials.toml` — tokens only (permission 0600, do NOT commit)

You can enter only one token at a time — existing tokens are preserved (merge mode).

## Multi-profile support

Multiple Slack workspaces can be managed via profiles. Use `--profile <name>` to switch:

```bash
slacli --profile personal chat send -c general -t "hello"
```

If `--profile` is omitted, `default_profile` from `config.toml` is used.

## Before any operation

Always run `slacli config --see` first (without `--profile`) to discover all profiles and their channel aliases. When the user's request spans multiple profiles (e.g., "all channels"), operate on every profile — not just the default.

## Confirmation required before sending messages

Before executing `slacli chat send`, you MUST use AskUserQuestion to ask the user which profile and channel to send to. Present the available profiles and their channel aliases (from `slacli config --see`) as choices so the user can select the destination.

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

```bash
slacli chat delete --channel <CHANNEL_ID_OR_ALIAS> --timestamp <TS>
```

- Requires Bot Token (`xoxb-`, scope: `chat:write`)
- `--channel` accepts a channel ID or alias
- `--timestamp` is the Slack message `ts` value (e.g. `1710000000.000100`)
- Bot can only delete messages it posted
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

# Set status with expiration (Unix timestamp)
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

## Tips

### Delete a message interactively with fzf

```bash
slacli logs --type chat-send \
  | jq -r '[.ts, .channel, ._slacli_profile, .message.text] | @tsv' \
  | fzf --with-nth=4.. --prompt="Delete> " \
  | { read ts channel profile _; slacli --profile "$profile" chat delete --channel "$channel" --timestamp "$ts"; }
```

### Look up past profile info from logs

Since `users.profile:read` scope is not granted, `profile edit` logs are the only way to retrieve past profile state (display_name, status, etc.).

```bash
# Show last profile snapshot
slacli logs --type profile-edit | tail -1 | jq '.profile | {display_name, status_emoji, status_text}'

# Browse profile history with fzf and restore a previous status
slacli logs --type profile-edit \
  | jq -r '[.profile.status_emoji, .profile.status_text, ._slacli_profile] | @tsv' \
  | fzf --prompt="Restore status> " \
  | { read emoji text profile; slacli --profile "$profile" profile edit --set "status_emoji=$emoji" --set "status_text=$text"; }
```

## Channel Aliases

Define channel aliases in `config.toml` under each profile so AI agents can easily find the right channel.

```toml
[profiles.work.channels]
dev = { id = "C01ABCDEF", description = "Dev team channel" }
general = { id = "C02XYZXYZ", description = "General announcements" }
```

Then use `slacli chat send --channel dev --text "hello"` instead of the full channel ID.

To discover available channel aliases, run `slacli config --see`.

## Notes

- All Slack commands output raw API JSON to stdout
- Errors are written to stderr as JSON: `{"ok": false, "error": "...", "detail": "..."}`
- Token resolution is credentials.toml only (no environment variable fallback)

## Error handling

When a Slack API error is returned (e.g. `missing_scope`, `not_allowed_token_type`), check the error and guide the user with available workarounds:

| Scope / Error | Workaround |
|---------------|------------|
| `users.profile:read` not granted | Use `slacli profile edit` response to read current profile, or `slacli logs --type profile-edit` for past profile state |
