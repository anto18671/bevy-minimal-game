//! Zombies: red squares that spawn around the arena edge and home in on the
//! player. They hurt the player on contact and die to bullets. Spawn rate and
//! speed both scale with survival time for a simple difficulty curve.

use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::*;

use crate::bullet::{BULLET_SIZE, Bullet};
use crate::game_state::{GameState, GameTime, GameplayEntity, Rng, ZombieKilled};
use crate::player::{PLAYER_SIZE, Player};
use crate::{ARENA_HEIGHT, ARENA_WIDTH};

pub const ZOMBIE_SIZE: f32 = 28.0;
/// Speed of a freshly spawned zombie at the very start of a round.
pub const ZOMBIE_BASE_SPEED: f32 = 55.0;
/// Damage dealt to the player per second of contact.
pub const ZOMBIE_DAMAGE_PER_SEC: f32 = 28.0;
/// Hard cap on simultaneously alive zombies, to keep performance steady.
pub const MAX_ZOMBIES: usize = 250;

const ZOMBIE_COLOR: Color = Color::srgb(0.80, 0.18, 0.18);

/// An enemy that pursues the player at its own (difficulty-scaled) speed.
#[derive(Component)]
pub struct Zombie {
    pub speed: f32,
}

/// Drives periodic zombie spawns. Its interval shrinks over time.
#[derive(Resource)]
pub struct SpawnTimer(Timer);

impl Default for SpawnTimer {
    fn default() -> Self {
        SpawnTimer(Timer::from_seconds(SPAWN_INTERVAL_START, TimerMode::Repeating))
    }
}

/// Longest gap between spawns (start of a round).
const SPAWN_INTERVAL_START: f32 = 1.20;
/// Shortest gap between spawns (late game).
const SPAWN_INTERVAL_MIN: f32 = 0.25;

pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnTimer>()
            .add_systems(OnEnter(GameState::Playing), reset_spawn_timer)
            .add_systems(
                Update,
                (
                    spawn_zombies,
                    move_zombies,
                    bullets_hit_zombies,
                    zombies_hit_player,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Reset the spawn cadence to its starting value for a new round.
fn reset_spawn_timer(mut timer: ResMut<SpawnTimer>) {
    timer
        .0
        .set_duration(Duration::from_secs_f32(SPAWN_INTERVAL_START));
    timer.0.reset();
}

/// Spawn zombies on a timer whose interval tightens as the round goes on.
/// New zombies appear just off a random arena edge and are faster the longer
/// the player has survived.
fn spawn_zombies(
    mut commands: Commands,
    time: Res<Time>,
    game_time: Res<GameTime>,
    mut rng: ResMut<Rng>,
    mut timer: ResMut<SpawnTimer>,
    zombies: Query<(), With<Zombie>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    // Difficulty scaling: ramp the spawn interval down toward the minimum.
    let interval = (SPAWN_INTERVAL_START - game_time.0 * 0.012).max(SPAWN_INTERVAL_MIN);
    timer.0.set_duration(Duration::from_secs_f32(interval));

    // Respect the population cap so the arena never bogs down.
    if zombies.iter().count() >= MAX_ZOMBIES {
        return;
    }

    let position = random_edge_position(&mut rng);
    // Zombies get slightly faster over time, but never out-run the player.
    let speed = ZOMBIE_BASE_SPEED + game_time.0 * 1.5;

    commands.spawn((
        Sprite::from_color(ZOMBIE_COLOR, Vec2::splat(ZOMBIE_SIZE)),
        Transform::from_translation(position.extend(0.0)),
        Zombie { speed },
        GameplayEntity,
    ));
}

/// Pick a random point just outside one of the four arena edges.
fn random_edge_position(rng: &mut Rng) -> Vec2 {
    let half_w = ARENA_WIDTH / 2.0 + ZOMBIE_SIZE;
    let half_h = ARENA_HEIGHT / 2.0 + ZOMBIE_SIZE;
    match rng.next_u64() % 4 {
        0 => Vec2::new(rng.range(-half_w, half_w), half_h), // top
        1 => Vec2::new(rng.range(-half_w, half_w), -half_h), // bottom
        2 => Vec2::new(-half_w, rng.range(-half_h, half_h)), // left
        _ => Vec2::new(half_w, rng.range(-half_h, half_h)),  // right
    }
}

/// Every zombie walks straight toward the player's current position.
/// `Without<Player>` makes the two `Transform` queries provably disjoint.
fn move_zombies(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut zombies: Query<(&mut Transform, &Zombie), Without<Player>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let target = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (mut transform, zombie) in &mut zombies {
        let position = transform.translation.truncate();
        let to_player = target - position;
        if to_player.length_squared() > f32::EPSILON {
            let step = to_player.normalize() * zombie.speed * dt;
            transform.translation += step.extend(0.0);
        }
    }
}

/// Resolve bullet/zombie collisions: destroy both and report a kill. A bullet
/// can only kill one zombie, and each entity is despawned at most once.
fn bullets_hit_zombies(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    zombies: Query<(Entity, &Transform), With<Zombie>>,
    mut killed: MessageWriter<ZombieKilled>,
) {
    // Squared distance comparison avoids per-pair square roots.
    let hit_radius = (BULLET_SIZE + ZOMBIE_SIZE) / 2.0;
    let hit_radius_sq = hit_radius * hit_radius;

    let mut despawned: HashSet<Entity> = HashSet::new();

    for (bullet_entity, bullet_transform) in &bullets {
        let bullet_pos = bullet_transform.translation.truncate();
        for (zombie_entity, zombie_transform) in &zombies {
            if despawned.contains(&zombie_entity) {
                continue;
            }
            let zombie_pos = zombie_transform.translation.truncate();
            if bullet_pos.distance_squared(zombie_pos) <= hit_radius_sq {
                commands.entity(bullet_entity).despawn();
                commands.entity(zombie_entity).despawn();
                despawned.insert(bullet_entity);
                despawned.insert(zombie_entity);
                killed.write(ZombieKilled);
                break; // this bullet is spent
            }
        }
    }
}

/// A zombie touching the player drains health over time; reaching zero ends
/// the round.
fn zombies_hit_player(
    time: Res<Time>,
    mut player: Query<(&Transform, &mut Player)>,
    zombies: Query<&Transform, With<Zombie>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_transform, mut player)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    let contact = (PLAYER_SIZE + ZOMBIE_SIZE) / 2.0;
    let contact_sq = contact * contact;

    let mut touching = false;
    for zombie_transform in &zombies {
        if player_pos.distance_squared(zombie_transform.translation.truncate()) <= contact_sq {
            touching = true;
            break;
        }
    }

    if touching {
        player.health -= ZOMBIE_DAMAGE_PER_SEC * time.delta_secs();
    }

    if player.health <= 0.0 {
        player.health = 0.0;
        next_state.set(GameState::GameOver);
    }
}
