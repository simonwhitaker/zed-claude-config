# Claude Config — Zed Extension

A Zed extension that automatically sorts and deduplicates permission arrays in Claude Code settings files (`.claude/settings.json`, `.claude/settings.local.json`) on save.

## What it does

Formats the `permissions.allow`, `permissions.ask`, and `permissions.deny` arrays in Claude Code settings files — sorting them alphabetically and removing duplicates.

## Local development

Build the LSP binary:

```bash
cd claude-settings-lsp && cargo build
```

Install the dev extension in Zed:

- `Cmd+Shift+P` → "zed: install dev extension"
- Select the `zed-claude-config` directory

Add this to your Zed settings (`~/.config/zed/settings.json`):

```json
{
  "lsp": {
    "claude-settings-lsp": {
      "binary": {
        "path": "/absolute/path/to/zed-claude-config/claude-settings-lsp/target/debug/claude-settings-lsp"
      }
    }
  },
  "languages": {
    "Claude Settings": {
      "formatter": "language_server",
      "format_on_save": "on"
    }
  }
}
```

Open `~/.claude/settings.json` — it should detect as "Claude Settings" in the status bar. Format with `Cmd+Shift+P` → "editor: format document", or just save.
