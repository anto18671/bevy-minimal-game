# Zombie Arena

A polished, minimalist **2D top-down survival game** written in [Rust](https://www.rust-lang.org/) with the [Bevy](https://bevyengine.org/) game engine.

Survive an endless onslaught of zombies in a closed arena. Move, aim, shoot, and rack up a high score for as long as you can stay alive. The difficulty ramps up the longer you survive.

Everything is drawn with simple colored squares — **no external assets, files, databases, APIs, or network access required**. Just install Rust and `cargo run`.

```
┌─────────────────────────────────────────────┐
│  Score: 120                                   │
│  Health: 74          ■ (red zombie)           │
│                                               │
│              ■ ──── ·  (yellow bullet)        │
│                  ◆ (you, green)               │
│        ■                          ■           │
│                                               │
└─────────────────────────────────────────────┘
```

## Gameplay

<video src="https://github.com/anto18671/bevy-minimal-game/raw/main/assets/gameplay.mp4" controls muted loop width="100%"></video>

> If the player above doesn't load, [watch the clip directly](https://github.com/anto18671/bevy-minimal-game/raw/main/assets/gameplay.mp4).

## Overview

- **Player** — a green square in the center of the arena. Move with `WASD`, aim with the mouse, shoot with the left button.
- **Zombies** — red squares that spawn around the edges and walk straight toward you. Touching one drains your health.
- **Bullets** — yellow squares fired toward the cursor. One bullet destroys one zombie.
- **Goal** — survive, kill zombies for points, and beat your previous score before your health hits zero.

## Controls

| Action       | Input                          |
| ------------ | ------------------------------ |
| Move         | `W` `A` `S` `D` / Arrow keys   |
| Aim          | Mouse                          |
| Shoot        | Left mouse button (hold to keep firing) |
| Restart      | `R` (on the game-over screen)  |

## Features

- **Continuous wave spawning** from random arena edges using a Bevy `Timer`.
- **Homing zombie AI** that always pursues the player's current position.
- **Mouse aiming** with the player sprite rotating to face the cursor.
- **Rate-limited shooting** so holding the button fires at a steady cadence.
- **Health & damage** — contact with zombies drains health over time; reaching zero ends the round.
- **Scoring** — every zombie kill awards points, shown live in the HUD.
- **Difficulty scaling** — spawn rate increases and zombies get slightly faster the longer you survive.
- **Game-over screen** with your final score and a one-key restart.
- **Performance-conscious** — zombie population is capped, off-screen bullets are despawned, and collision checks use squared distances to avoid square roots.
- **Clean ECS architecture** — each concern is an isolated Bevy `Plugin`.

## Project Structure

```
zombie-arena/
├── Cargo.toml
└── src/
    ├── main.rs        # App setup, window, camera, plugin wiring
    ├── game_state.rs  # State machine, shared resources, scoring, restart, RNG
    ├── player.rs      # Player spawning, movement, mouse aiming
    ├── zombie.rs      # Zombie spawning, pursuit AI, collisions, difficulty
    ├── bullet.rs      # Shooting, projectile motion, off-screen cleanup
    └── ui.rs          # HUD (score/health) and game-over overlay
```

The code uses Bevy idioms throughout: **states** (`Playing` / `GameOver`), **components**, **resources**, **messages** (Bevy 0.19's renamed events) for decoupled scoring, **timers** for spawning and fire-rate, and **delta-time** movement so gameplay is frame-rate independent. There is **no `unsafe` code**.

## Build & Run

### Prerequisites

- A recent stable Rust toolchain (install via [rustup](https://rustup.rs/)).

> **Note:** The first build downloads and compiles Bevy and its dependencies, which can take several minutes. Subsequent builds are fast.

### Run

```sh
git clone <this-repo>
cd zombie-arena
cargo run
```

For a smoother, faster experience build in release mode:

```sh
cargo run --release
```

## How It Works

The game is a single Bevy `App` composed of five plugins. The high-level flow:

1. On entering `GameState::Playing`, the score/timer reset, the player spawns in the center, and the HUD appears.
2. Each frame (while playing): input moves and rotates the player, a timer spawns zombies at the edges, zombies home in on the player, bullets fly and are cleaned up off-screen, and collisions are resolved.
3. When the player's health reaches zero, the state switches to `GameState::GameOver`: all gameplay systems pause, and a dimmed overlay shows the final score.
4. Pressing `R` clears the arena and starts a fresh round.

Difficulty is driven by a single accumulating `GameTime` resource: as it grows, the spawn interval shrinks (toward a floor) and newly spawned zombies are a touch faster.

## Future Improvements

- Different zombie types (fast, tanky, ranged).
- Pickups: health packs, weapon upgrades, rapid-fire power-ups.
- Reload / limited ammo mechanic.
- Particle effects and screen shake for hits and deaths.
- Sound effects and music (would introduce audio assets).
- Persistent high-score tracking.
- Wave/boss structure instead of purely continuous spawning.
- A start menu and pause state.

## License

Released under the **MIT** license.
