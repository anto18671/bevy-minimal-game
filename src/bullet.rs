//! Bullets: yellow squares fired from the player toward the cursor. They travel
//! in a straight line, are rate-limited by a cooldown timer, and are despawned
//! once they leave the arena so they never accumulate.

use bevy::prelude::*;

use crate::game_state::{GameState, GameplayEntity};
use crate::player::{PLAYER_SIZE, Player};
use crate::{ARENA_HEIGHT, ARENA_WIDTH};

pub const BULLET_SPEED: f32 = 750.0;
pub const BULLET_SIZE: f32 = 8.0;

/// Minimum delay between shots while the fire button is held.
const FIRE_COOLDOWN: f32 = 0.16;

const BULLET_COLOR: Color = Color::srgb(1.0, 0.9, 0.25);

/// A projectile carrying its own constant velocity.
#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
}

/// Gates the fire rate. Starts already elapsed so the very first click fires
/// immediately.
#[derive(Resource)]
pub struct FireCooldown(Timer);

impl Default for FireCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(FIRE_COOLDOWN, TimerMode::Once);
        // Mark it finished up front so there's no startup delay on the first shot.
        timer.tick(timer.duration());
        FireCooldown(timer)
    }
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireCooldown>().add_systems(
            Update,
            (shoot, move_bullets, despawn_out_of_bounds).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Fire a bullet on left-click (or hold), respecting the cooldown.
fn shoot(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cooldown: ResMut<FireCooldown>,
    player: Query<(&Transform, &Player)>,
) {
    cooldown.0.tick(time.delta());

    if !mouse.pressed(MouseButton::Left) || !cooldown.0.is_finished() {
        return;
    }

    let Ok((transform, player)) = player.single() else {
        return;
    };

    cooldown.0.reset();

    let direction = player.aim;
    // Spawn just outside the player so bullets don't overlap it.
    let origin = transform.translation.truncate() + direction * PLAYER_SIZE;

    commands.spawn((
        Sprite::from_color(BULLET_COLOR, Vec2::splat(BULLET_SIZE)),
        Transform::from_translation(origin.extend(1.0)),
        Bullet {
            velocity: direction * BULLET_SPEED,
        },
        GameplayEntity,
    ));
}

/// Advance every bullet along its velocity using delta time.
fn move_bullets(time: Res<Time>, mut bullets: Query<(&mut Transform, &Bullet)>) {
    let dt = time.delta_secs();
    for (mut transform, bullet) in &mut bullets {
        transform.translation += (bullet.velocity * dt).extend(0.0);
    }
}

/// Despawn bullets once they exit the arena (with a small margin) to keep the
/// entity count and work-per-frame bounded.
fn despawn_out_of_bounds(mut commands: Commands, bullets: Query<(Entity, &Transform), With<Bullet>>) {
    let max_x = ARENA_WIDTH / 2.0 + BULLET_SIZE;
    let max_y = ARENA_HEIGHT / 2.0 + BULLET_SIZE;
    for (entity, transform) in &bullets {
        let pos = transform.translation;
        if pos.x.abs() > max_x || pos.y.abs() > max_y {
            commands.entity(entity).despawn();
        }
    }
}
