use bevy::prelude::*;
use bevy::render::camera::Camera2d;
use crate::ascii::{AsciiSheet, spawn_ascii_text};
use crate::combat::CombatState;
use crate::TILE_SIZE;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(SystemSet::on_enter(CombatState::Dead).with_system(show_game_over_screen))
            .add_system_set(SystemSet::on_update(CombatState::Dead).with_system(zoom_into_center));
    }
}

fn show_game_over_screen(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
) {
    let text = "GAME OVER";
    spawn_ascii_text(
        &mut commands,
        &ascii,
        text,
        Vec3::new(-((text.len() / 2) as f32 * TILE_SIZE), 0.0, 0.0),
    );
}

fn zoom_into_center(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let mut camera_transform = camera_query.single_mut();
    let step = 0.3;
    if camera_transform.scale.x > 0.3 {
        camera_transform.scale.x -= step * time.delta_seconds();
        camera_transform.scale.y -= step * time.delta_seconds();
    }
}
