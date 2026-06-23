//! User interface: the in-game HUD (score + health) and the game-over overlay.
//! UI entities are tied to their state with `OnEnter`/`OnExit` so they appear
//! and disappear automatically as the game transitions.

use bevy::prelude::*;

use crate::game_state::{GameState, Score};
use crate::player::{PLAYER_MAX_HEALTH, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // HUD lives only while playing.
            .add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnExit(GameState::Playing), despawn::<Hud>)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            // Game-over overlay lives only on the game-over screen.
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn::<GameOverScreen>);
    }
}

/// Root marker for the HUD container.
#[derive(Component)]
struct Hud;

/// Marks the text node displaying the score.
#[derive(Component)]
struct ScoreText;

/// Marks the text node displaying the player's health.
#[derive(Component)]
struct HealthText;

/// Root marker for the game-over overlay.
#[derive(Component)]
struct GameOverScreen;

/// Spawn the top-left HUD: a score line and a health line.
fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            Hud,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Score: 0"),
                TextFont::from_font_size(26.0),
                TextColor(Color::WHITE),
                ScoreText,
            ));
            parent.spawn((
                Text::new("Health: 100"),
                TextFont::from_font_size(26.0),
                TextColor(Color::srgb(0.4, 1.0, 0.4)),
                HealthText,
            ));
        });
}

/// Keep the HUD text in sync with the live score and player health.
fn update_hud(
    score: Res<Score>,
    player: Query<&Player>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<HealthText>)>,
    mut health_text: Query<&mut Text, (With<HealthText>, Without<ScoreText>)>,
) {
    if let Ok(mut text) = score_text.single_mut() {
        text.0 = format!("Score: {}", score.0);
    }

    if let Ok(player) = player.single()
        && let Ok(mut text) = health_text.single_mut()
    {
        let health = player.health.ceil().clamp(0.0, PLAYER_MAX_HEALTH) as i32;
        text.0 = format!("Health: {health}");
    }
}

/// Spawn the centred, dimmed game-over overlay with the final score and the
/// restart prompt.
fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            // Translucent black to dim the frozen arena behind it.
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont::from_font_size(80.0),
                TextColor(Color::srgb(0.9, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", score.0)),
                TextFont::from_font_size(36.0),
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press R to Restart"),
                TextFont::from_font_size(28.0),
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

/// Generic cleanup helper: despawn every entity carrying marker component `T`.
fn despawn<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
