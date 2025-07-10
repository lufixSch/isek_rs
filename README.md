# ISEK-rs

Simple TUI for displaying tasks from ical files

## Features

- Display tasks from multiple calendars
- Mark tasks as done/undone
- Sort tasks based on date, priority or the "isek index"

## Getting Started

Pull the repo and build and install the tool with cargo:

```bash
cargo install --path .
```

You can start the TUI now with the `isek` command. After the first start a config is generated at `$XDG_CONFIG_DIR/isek/config.toml` (usually `~/.config/isek/config.toml`, this may differ on Windows or MacOS) which you can modify to your liking.

First you need to add a calendar currently only the **VDIR** structure is supported (If needed other solutions could be added). You can add as many calendars as you like by repeating the example below with another path. The color and name of the calendar is taken from the corresponding files in the directory.

```toml
[[calendars]]
[calendars.VDIR]
path = "path/to/your/synced/calendars/calendarname"
```

> [!NOTE]
> Make sure to select the whole calendar directory and **NOT** directly the directory used by pimsync or similar

## Syncing with other Calendars

Isek itself doesn't have the ability to sync with remote calendars. Use something like [vdirsyncer](https://vdirsyncer.pimutils.org/en/stable/index.html) or [pimsync](https://git.sr.ht/~whynothugo/pimsync) (Not tested!) for that. The resulting directories can then be added to the isek config as sources.

## Known Issues

- Multiple calendars with the same name (but different paths) could lead to tasks not being displayed correctly. Avoid if possible!
