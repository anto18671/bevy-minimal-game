//! Zombie Arena — a minimalist 2D top-down survival game.
//!
//! The whole game is built on Bevy's ECS. Each gameplay concern lives in its
//! own module and exposes a `Plugin`, which keeps `main` tiny and makes the
//! systems easy to follow:
//!
//! * [`game_state`] — global state machine, shared resources, scoring & restart
//! * [`player`]      — the controllable green square (movement + aiming)
//! * [`zombie`]      — enemy spawning, pursuit AI and collisions
//! * [`bullet`]      — shooting, projectile motion and despawning
//! * [`ui`]          — the heads-up display and the game-over screen
//!
//! Run it with `cargo run`. No assets, files or network access required.

mod bullet;
mod game_state;
mod player;
mod ui;
mod zombie;

use bevy::prelude::*;

use bullet::BulletPlugin;
use game_state::GameStatePlugin;
use player::PlayerPlugin;
use ui::UiPlugin;
use zombie::ZombiePlugin;

/// Logical size of the arena / window in pixels.
pub const ARENA_WIDTH: f32 = 1280.0;
pub const ARENA_HEIGHT: f32 = 720.0;

/// Dark gray play-field background.
pub const BACKGROUND_COLOR: Color = Color::srgb(0.10, 0.10, 0.12);

fn main() {
    App::new()
        // DefaultPlugins gives us windowing, rendering, input, time, etc.
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Zombie Arena".to_string(),
                resolution: (ARENA_WIDTH as u32, ARENA_HEIGHT as u32).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Clearing the screen each frame paints our dark-gray background.
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // Each gameplay concern is a self-contained plugin.
        .add_plugins((
            GameStatePlugin,
            PlayerPlugin,
            ZombiePlugin,
            BulletPlugin,
            UiPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .run();
}

/// Spawn the single 2D camera. It lives for the whole program, independent of
/// the game state, so the world stays visible on the game-over screen too.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
