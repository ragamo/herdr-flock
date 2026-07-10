# herdr-flock

Rust TUI plugin for [herdr](https://herdr.dev) that shows AI coding agents as pixel-art sheep on a top-down farm.

## Build & Run

```bash
cargo build          # debug
cargo build --release
cargo run            # demo mode (mock data, no herdr needed)
```

No test suite. Verify changes by running `cargo run` in a real terminal.

## Architecture

```
src/
  main.rs           — event loop (100ms poll), terminal setup, socket discovery
  app.rs            — App struct, Atmosphere (day/night + weather), event handling
  herdr.rs          — herdr socket client (agent.list polling every 5s)
  mock.rs           — mock farm for demo mode
  storage.rs        — SQLite persistence via rusqlite
  model/
    sheep.rs        — Sheep struct, SheepState, Direction
    farm.rs         — Farm struct, tick loop, terrain generation
  animation/
    sprites.rs      — pixel art sheep sprites (10px tall, half-block rendering)
  ui/
    mod.rs          — tab bar, screen routing
    farm.rs         — farm renderer: terrain, sheep, tooltip, status bar
    log.rs          — graveyard renderer: table, epitaph panel, sorted_log_indices
```

## Key Concepts

**Rendering**: Sprites are 11×10 pixel maps rendered as 11×5 terminal chars using Unicode half-blocks (`▀▄█`). Each pixel is one of: `T`(transparent), `K`(black), `W`(white/wool), `B`(beige/face), `Z`(zzz), `M`(mouth/red).

**Sheep identity**: `id = "{pane_id}:{name}"` — unique per life. `pane_id` is used for herdr matching. New agent session on the same pane → new sheep, not a resurrection.

**Terrain**: Generated once per session (or on resize). River path is a sinusoidal curve with random start/end rows. Trees (5×6) and rocks (6×4) avoid the river band per-column. All use deterministic LCG + random entropy seed.

**Day/night cycle**: 2400-tick cycle (~4 min). `night_factor()` 0.0=day, 1.0=night. All terrain colors and sprite colors interpolate via `lerp_color()` / `dim_color()`.

**Weather**: State machine: `Clear → FadeIn → Active → FadeOut → Clear`. Rain and snow particles stored in `Atmosphere.precipitation`, rendered as a post-pass over the terrain `Vec<Line>`.

**herdr integration**: Polls `agent.list` every 5s via Unix socket. Socket discovered from `HERDR_SOCKET_PATH` env var, then `~/.config/herdr/herdr.sock`, then `herdr status server --json`. Falls back to mock data if no socket found.

**Storage**: SQLite at `~/Library/Application Support/herdr-flock/flock.db` (macOS) or `~/.local/share/herdr-flock/flock.db` (Linux). Schema migrates automatically (ALTER TABLE for new columns).

## Conventions

- No comments unless the why is non-obvious
- `find_free_spawn_in()` in `app.rs` is the canonical spawn function — use it everywhere
- `sorted_log_indices()` in `ui/log.rs` must be used for both rendering and click handling to keep them in sync
- `terrain_cell()` priority order: fence → trees → rocks → river → pond → stars → grass
- Sheep in `live_mode` never change state via the tick loop — states come from herdr poll only. Cosmetic sub-activities (eating/sleeping) are allowed when base state is Idle.
- Commits only when explicitly requested
