# herdr-flock

A [herdr](https://herdr.dev) plugin that visualizes your AI coding agents as pixel-art sheep living on a top-down farm.

Each agent gets its own sheep. While it works, the sheep grazes and glows. When the session ends, the sheep dies and is remembered in the Graveyard.

![demo mode screenshot placeholder]

## Features

### The Flock
- Pixel-art sheep rendered with Unicode half-blocks — front, back, and side views
- Idle sheep wander, eat, and sleep with animated sprites
- Working agents pulse yellow
- Sheep collide and navigate around each other
- Mouse click to inspect a sheep (name, project, agent type)

### Living Terrain
- Procedurally generated river that curves differently each session
- Trees (5×6) and rocks (6×4) scattered at random positions
- Perimeter fence with box-drawing characters
- Grass texture variation via cell hash
- Day/night cycle (~4 min) with color interpolation and stars at night
- Rain and snow weather events with fade in/out transitions

### The Graveyard
- Full history of every sheep that ever lived, persisted in SQLite
- Epitaph panel with gravestone when selecting a dead sheep
- Lifespan, birth/death timestamps (local time), agent type, project
- Dead rows fade darker the older they are
- Separator between alive and departed

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/herdr-flock`.

## Usage

### Standalone (demo mode)
```bash
cargo run
```
Launches with mock sheep so you can explore without herdr running.

### As a herdr plugin
With herdr running, the app auto-discovers the socket:
```bash
herdr-flock
# or explicitly:
HERDR_SOCKET_PATH=/path/to/herdr.sock herdr-flock
```

To register as a plugin, copy `herdr-plugin.toml` to your herdr plugins directory. The app will then be available as an overlay pane within herdr.

### Keyboard & Mouse

| Key / Action | Effect |
|---|---|
| `Tab` | Switch between Flock and Graveyard |
| `q` | Quit |
| `↑↓` | Scroll in Graveyard |
| `f` | Cycle filter (All / Alive / Dead) in Graveyard |
| Click on sheep | Show tooltip with name, project, agent |
| Click on tab | Switch screen |
| Click on row (Graveyard) | Show epitaph panel for dead sheep |

## Data

Sheep history is stored at:
- **macOS**: `~/Library/Application Support/herdr-flock/flock.db`
- **Linux**: `~/.local/share/herdr-flock/flock.db`

Each sheep has a unique identity of `{pane_id}:{name}` so reused pane IDs (new agent sessions) always create a new sheep rather than resurrecting the old one.

## Agent → Sheep mapping

| herdr state | Sheep behavior |
|---|---|
| `working` | Pulses yellow, wanders |
| `blocked` | Eating animation |
| `done` | Sleeping |
| `idle` | Walking around |
| pane closed | Sheep dies, enters Graveyard |

## Tech

- **Rust** — ratatui 0.29 + crossterm 0.28
- **Storage** — SQLite via rusqlite (bundled)
- **herdr integration** — `agent.list` polling every 5s over Unix socket
