//! The player: a green square that moves with WASD and continuously rotates to
//! face the mouse cursor. Its aim direction is stored on the component so the
//! bullet module can fire in the right direction.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game_state::{GameState, GameplayEntity};
use crate::{ARENA_HEIGHT, ARENA_WIDTH};

pub const PLAYER_SPEED: f32 = 320.0;
pub const PLAYER_SIZE: f32 = 30.0;
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

const PLAYER_COLOR: Color = Color::srgb(0.25, 0.85, 0.35);

/// The player avatar. Holds current health and the unit vector pointing at the
/// mouse cursor (its aim).
#[derive(Component)]
pub struct Player {
    pub health: f32,
    pub aim: Vec2,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (move_player, aim_at_cursor).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Spawn the player in the centre of the arena at the start of each round.
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(PLAYER_COLOR, Vec2::splat(PLAYER_SIZE)),
        // z = 2 keeps the player drawn above zombies (z = 0) and bullets.
        Transform::from_xyz(0.0, 0.0, 2.0),
        Player {
            health: PLAYER_MAX_HEALTH,
            aim: Vec2::X,
        },
        GameplayEntity,
    ));
}

/// WASD / arrow-key movement with delta-time scaling, clamped to the arena.
fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    // Normalising prevents diagonal movement from being faster.
    let movement = direction.normalize() * PLAYER_SPEED * time.delta_secs();
    transform.translation += movement.extend(0.0);

    // Keep the player fully inside the arena bounds.
    let max_x = ARENA_WIDTH / 2.0 - PLAYER_SIZE / 2.0;
    let max_y = ARENA_HEIGHT / 2.0 - PLAYER_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-max_x, max_x);
    transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
}

/// Rotate the player to face the mouse cursor and cache the aim direction.
fn aim_at_cursor(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut player: Query<(&mut Player, &mut Transform)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok((mut player, mut transform)) = player.single_mut() else {
        return;
    };

    // No cursor over the window this frame: keep the previous aim.
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    // Translate the screen-space cursor into world space.
    let Ok(world_cursor) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    let to_cursor = world_cursor - transform.translation.truncate();
    if to_cursor.length_squared() > f32::EPSILON {
        player.aim = to_cursor.normalize();
        // Rotate the sprite so it visibly points toward the cursor.
        let angle = player.aim.y.atan2(player.aim.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
