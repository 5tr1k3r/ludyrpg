mod ascii;
mod audio;
mod combat;
mod debug;
mod fadeout;
mod graphics;
mod npc;
mod player;
mod start_menu;
mod tilemap;
mod game_over;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::audio::GameAudioPlugin;
use crate::combat::CombatPlugin;
use crate::fadeout::FadeoutPlugin;
use crate::graphics::GraphicsPlugin;
use crate::npc::NpcPlugin;
use crate::start_menu::MainMenuPlugin;
use ascii::AsciiPlugin;
use debug::DebugPlugin;
use player::PlayerPlugin;
use tilemap::TileMapPlugin;
use crate::game_over::GameOverPlugin;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const TILE_SIZE: f32 = 0.1;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
    StartMenu,
    Overworld,
    Combat,
}

fn main() {
    let height = 650.0;
    App::new()
        .add_state(GameState::StartMenu)
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: height * RESOLUTION,
            height,
            title: "Game".to_string(),
            resizable: false,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .add_plugin(AsciiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(TileMapPlugin)
        .add_plugin(CombatPlugin)
        .add_plugin(FadeoutPlugin)
        .add_plugin(GameAudioPlugin)
        .add_plugin(GraphicsPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(NpcPlugin)
        .add_plugin(GameOverPlugin)
        .add_startup_system(spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.left = -1.0 * RESOLUTION;
    camera.orthographic_projection.right = 1.0 * RESOLUTION;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}
