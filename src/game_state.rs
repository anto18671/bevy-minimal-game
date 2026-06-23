//! Global game state: the play/game-over state machine, shared resources
//! (score, elapsed time, RNG), the "zombie killed" message and the systems
//! that tie scoring, difficulty and restarting together.

use bevy::prelude::*;

/// High-level state of the application, driven by Bevy's state machine.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// The arena is active: things move, spawn and collide.
    #[default]
    Playing,
    /// The player has died; everything is frozen behind the game-over screen.
    GameOver,
}

/// Marker placed on every transient gameplay entity (player, zombies, bullets)
/// so they can be wiped in one query when a new round starts.
#[derive(Component)]
pub struct GameplayEntity;

/// The player's running score for the current round.
#[derive(Resource, Default)]
pub struct Score(pub u32);

/// Seconds survived in the current round. Difficulty scales off this value.
#[derive(Resource, Default)]
pub struct GameTime(pub f32);

/// A tiny self-contained xorshift RNG so we don't pull in an external crate
/// just to scatter zombies around the edges of the arena.
#[derive(Resource)]
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        // Any non-zero seed works for xorshift.
        Rng(0x2545_F491_4F6C_DD1D)
    }
}

impl Rng {
    /// Advance the generator and return the next raw 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A uniform `f32` in the half-open range `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        // Use the high 24 bits, which are the best-mixed ones.
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A uniform `f32` in `[min, max)`.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

/// Emitted whenever a zombie is destroyed. Decoupling "a zombie died" from
/// "award points" via a message keeps the bullet/zombie collision code unaware
/// of scoring.
#[derive(Message)]
pub struct ZombieKilled;

/// Points granted per zombie kill.
const SCORE_PER_KILL: u32 = 10;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<Score>()
            .init_resource::<GameTime>()
            .init_resource::<Rng>()
            .add_message::<ZombieKilled>()
            // Reset per-round state when a round begins. `Playing` is the
            // default state, so this also runs once at startup.
            .add_systems(OnEnter(GameState::Playing), reset_round)
            // Clear out leftover entities only when leaving the game-over
            // screen, i.e. right before the next round spawns fresh ones.
            .add_systems(OnExit(GameState::GameOver), despawn_gameplay_entities)
            // Scoring and the difficulty clock only tick while playing.
            .add_systems(
                Update,
                (tick_game_time, award_score).run_if(in_state(GameState::Playing)),
            )
            // Restarting is only possible from the game-over screen.
            .add_systems(Update, restart_on_key.run_if(in_state(GameState::GameOver)));
    }
}

/// Zero out the score and difficulty clock at the start of every round.
fn reset_round(mut score: ResMut<Score>, mut game_time: ResMut<GameTime>) {
    score.0 = 0;
    game_time.0 = 0.0;
}

/// Remove every transient gameplay entity so the next round starts clean.
fn despawn_gameplay_entities(
    mut commands: Commands,
    entities: Query<Entity, With<GameplayEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

/// Accumulate survival time, which drives the difficulty curve elsewhere.
fn tick_game_time(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.0 += time.delta_secs();
}

/// Convert "zombie killed" messages into score. Reading the messages each
/// frame also drains them so they don't pile up.
fn award_score(mut kills: MessageReader<ZombieKilled>, mut score: ResMut<Score>) {
    for _ in kills.read() {
        score.0 += SCORE_PER_KILL;
    }
}

/// Press `R` on the game-over screen to begin a fresh round.
fn restart_on_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}
