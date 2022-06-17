use crate::combat::CombatStats;
use crate::game_ui::{CreateTextPopupEvent, TextPopupPosition};
use crate::player::Player;
use crate::{GameState, TILE_SIZE};
use bevy::prelude::*;

pub struct NpcPlugin;

#[derive(Component)]
pub enum Npc {
    Healer,
}

#[derive(Component)]
pub struct NpcText;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Overworld).with_system(npc_speech));
    }
}

fn npc_speech(
    mut player_query: Query<(&Player, &mut CombatStats, &Transform)>,
    npc_query: Query<&Transform, With<Npc>>,
    keyboard: Res<Input<KeyCode>>,
    mut ev_text_popup: EventWriter<CreateTextPopupEvent>,
) {
    let (player, mut stats, transform) = player_query.single_mut();
    if !player.active {
        return;
    }

    if keyboard.just_pressed(KeyCode::E) {
        for npc_transform in npc_query.iter() {
            if Vec2::distance(
                npc_transform.translation.truncate(),
                transform.translation.truncate(),
            ) < TILE_SIZE * 1.5
            {
                let text = if stats.health == stats.max_health {
                    "You seem to be doing just fine without me!".to_string()
                } else {
                    stats.health = stats.max_health;
                    "You seem weak, let me heal you!".to_string()
                };

                ev_text_popup.send(CreateTextPopupEvent {
                    text,
                    position: TextPopupPosition::Center,
                });
            }
        }
    }
}
