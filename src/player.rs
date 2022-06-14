use crate::ascii::AsciiSheet;
use crate::combat::CombatStats;
use crate::fadeout::create_fadeout;
use crate::graphics::{CharacterSheet, FacingDirection, FrameAnimation, PlayerGraphics};
use crate::tilemap::{EncounterSpawner, TileCollider};
use crate::{GameState, TILE_SIZE};
use bevy::prelude::*;
use bevy::render::camera::Camera2d;
use bevy::sprite::collide_aabb::collide;
use bevy_inspector_egui::Inspectable;

pub struct PlayerPlugin;

#[derive(Component, Inspectable)]
pub struct Player {
    speed: f32,
    pub(crate) active: bool,
    pub(crate) just_moved: bool,
    pub(crate) walked_ground_type: WalkedGroundType,
    pub exp: usize,
    pub(crate) trauma: f32,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EncounterTracker {
    timer: Timer,
}

#[derive(Inspectable)]
pub enum WalkedGroundType {
    Normal,
    Grass,
}

const CAMERA_STEP: f32 = 1.5;

impl Default for Player {
    fn default() -> Self {
        Player {
            speed: 3.0,
            active: true,
            just_moved: false,
            walked_ground_type: WalkedGroundType::Normal,
            exp: 0,
            trauma: 0.0,
        }
    }
}

impl Player {
    pub fn level_up(&mut self, exp: usize, stats: &mut CombatStats) -> bool {
        self.exp += exp;
        let exp_needed = 50;
        if self.exp >= exp_needed {
            stats.health += 2;
            stats.max_health += 2;
            stats.attack += 1;
            stats.defense += 1;
            self.exp -= exp_needed;

            return true;
        }

        false
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_resume(GameState::Overworld).with_system(show_player))
            .add_system_set(SystemSet::on_pause(GameState::Overworld).with_system(hide_player))
            .add_system_set(
                SystemSet::on_update(GameState::Overworld)
                    .with_system(player_movement)
                    .with_system(camera_movement.after(player_movement))
                    .with_system(player_encounter_checking.after(player_movement)),
            )
            .add_system_set(SystemSet::on_enter(GameState::Overworld).with_system(spawn_player));
    }
}

fn hide_player(
    mut player_query: Query<&mut Visibility, With<Player>>,
    children_query: Query<&Children, With<Player>>,
    mut child_visibility_query: Query<&mut Visibility, Without<Player>>,
) {
    let mut player_vis = player_query.single_mut();
    player_vis.is_visible = false;

    if let Ok(children) = children_query.get_single() {
        for child in children.iter() {
            if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                child_vis.is_visible = false
            }
        }
    }
}

fn show_player(
    mut player_query: Query<(&mut Player, &mut Visibility)>,
    children_query: Query<&Children, With<Player>>,
    mut child_visibility_query: Query<&mut Visibility, Without<Player>>,
) {
    let (mut player, mut player_vis) = player_query.single_mut();
    player.active = true;
    player_vis.is_visible = true;

    if let Ok(children) = children_query.get_single() {
        for child in children.iter() {
            if let Ok(mut child_vis) = child_visibility_query.get_mut(*child) {
                child_vis.is_visible = true
            }
        }
    }
}

fn player_encounter_checking(
    mut commands: Commands,
    mut player_query: Query<(&mut Player, &mut EncounterTracker, &Transform)>,
    encounter_query: Query<&Transform, (With<EncounterSpawner>, Without<Player>)>,
    ascii: Res<AsciiSheet>,
    time: Res<Time>,
) {
    let (mut player, mut encounter_tracker, player_transform) = player_query.single_mut();
    let player_translation = player_transform.translation;

    if player.just_moved
        && encounter_query
            .iter()
            .any(|&transform| wall_collision_check(player_translation, transform.translation))
    {
        player.walked_ground_type = WalkedGroundType::Grass;
        encounter_tracker.timer.tick(time.delta());

        if encounter_tracker.timer.just_finished() {
            player.active = false;
            create_fadeout(&mut commands, Some(GameState::Combat), &ascii);
        }
    } else if player.just_moved {
        player.walked_ground_type = WalkedGroundType::Normal;
    }
}

fn camera_movement(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera2d>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let player_transform = player_query.single();
    let mut camera_transform = camera_query.single_mut();

    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;

    if keyboard.pressed(KeyCode::NumpadAdd) {
        let step = CAMERA_STEP * time.delta_seconds();
        camera_transform.scale *= Vec3::new(1.0 - step, 1.0 - step, 1.0);
    }

    if keyboard.pressed(KeyCode::NumpadSubtract) {
        let step = CAMERA_STEP * time.delta_seconds();
        camera_transform.scale *= Vec3::new(1.0 + step, 1.0 + step, 1.0);
    }

    if keyboard.pressed(KeyCode::Home) {
        camera_transform.scale = Vec3::ONE;
    }
}

fn player_movement(
    mut player_query: Query<(&mut Player, &mut Transform, &mut PlayerGraphics)>,
    wall_query: Query<&Transform, (With<TileCollider>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player, mut transform, mut player_graphics) = player_query.single_mut();
    player.just_moved = false;
    if !player.active {
        return;
    }

    let mut y_delta = 0.0;
    if keyboard.pressed(KeyCode::W) {
        y_delta += TILE_SIZE * player.speed * time.delta_seconds();
        player_graphics.facing = FacingDirection::Up;
    }

    if keyboard.pressed(KeyCode::S) {
        y_delta -= TILE_SIZE * player.speed * time.delta_seconds();
        player_graphics.facing = FacingDirection::Down;
    }

    let mut x_delta = 0.0;
    if keyboard.pressed(KeyCode::A) {
        x_delta -= TILE_SIZE * player.speed * time.delta_seconds();
        player_graphics.facing = FacingDirection::Left;
    }

    if keyboard.pressed(KeyCode::D) {
        x_delta += TILE_SIZE * player.speed * time.delta_seconds();
        player_graphics.facing = FacingDirection::Right;
    }

    let target = transform.translation + Vec3::new(x_delta, 0.0, 0.0);
    if !wall_query
        .iter()
        .any(|&transform| wall_collision_check(target, transform.translation))
    {
        if x_delta != 0.0 {
            player.just_moved = true;
        }
        transform.translation = target;
    }

    let target = transform.translation + Vec3::new(0.0, y_delta, 0.0);
    if !wall_query
        .iter()
        .any(|&transform| wall_collision_check(target, transform.translation))
    {
        if y_delta != 0.0 {
            player.just_moved = true;
        }
        transform.translation = target;
    }
}

fn wall_collision_check(target_player_pos: Vec3, wall_translation: Vec3) -> bool {
    let collision = collide(
        target_player_pos,
        Vec2::splat(TILE_SIZE * 0.9),
        wall_translation,
        Vec2::splat(TILE_SIZE),
    );

    collision.is_some()
}

fn spawn_player(mut commands: Commands, characters: Res<CharacterSheet>) {
    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: characters.player_down[0],
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(2.0 * TILE_SIZE, -2.0 * TILE_SIZE, 900.0),
            texture_atlas: characters.handle.clone(),
            ..default()
        })
        .insert(FrameAnimation {
            timer: Timer::from_seconds(0.2, true),
            frames: characters.player_down.to_vec(),
            current_frame: 0,
        })
        .insert(PlayerGraphics {
            facing: FacingDirection::Down,
        })
        .insert(Name::new("Player"))
        .insert(Player::default())
        .insert(CombatStats {
            health: 10,
            max_health: 10,
            attack: 2,
            defense: 1,
        })
        .insert(EncounterTracker {
            timer: Timer::from_seconds(1.0, true),
        });
}
