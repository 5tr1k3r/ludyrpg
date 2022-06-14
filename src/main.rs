mod ascii;
mod audio;
mod camera;
mod combat;
mod debug;
mod fadeout;
mod graphics;
mod npc;
mod player;
mod start_menu;
mod tilemap;

use bevy::log::LogSettings;
use bevy::prelude::*;

use crate::ascii::AsciiPlugin;
use crate::audio::GameAudioPlugin;
use crate::camera::CameraPlugin;
use crate::combat::CombatPlugin;
use crate::debug::DebugPlugin;
use crate::fadeout::FadeoutPlugin;
use crate::graphics::GraphicsPlugin;
use crate::npc::NpcPlugin;
use crate::player::PlayerPlugin;
use crate::start_menu::MainMenuPlugin;
use crate::tilemap::TileMapPlugin;

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
        .insert_resource(LogSettings {
            filter: "wgpu=error,symphonia=warn".to_string(),
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
        .add_plugin(CameraPlugin)
        .run();
}
