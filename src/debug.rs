use crate::combat::CombatStats;
use crate::player::{EncounterTracker, Player};
use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(WorldInspectorPlugin::new())
                .register_type::<EncounterTracker>()
                .register_inspectable::<CombatStats>()
                .register_inspectable::<Player>();
        }
    }
}
