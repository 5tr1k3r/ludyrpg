use crate::combat::CombatState;
use crate::player::{player_movement, Player};
use crate::{GameState, RESOLUTION};
use bevy::prelude::*;
use bevy::render::camera::{Camera2d, ScalingMode};
use rand::{thread_rng, Rng};

pub struct CameraPlugin;

pub struct OverworldCameraData {
    scale: Vec3,
}

const CAMERA_STEP: f32 = 1.5;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverworldCameraData { scale: Vec3::ONE })
            .add_system_set(SystemSet::on_enter(GameState::Overworld).with_system(spawn_camera))
            .add_system_set(
                SystemSet::on_update(GameState::Overworld)
                    .with_system(camera_movement.after(player_movement)),
            )
            .add_system_set(
                SystemSet::on_pause(GameState::Overworld).with_system(save_and_reset_camera_scale),
            )
            .add_system_set(
                SystemSet::on_resume(GameState::Overworld).with_system(restore_camera_scale),
            )
            .add_system_set(SystemSet::on_update(CombatState::Dead).with_system(zoom_into_center))
            .add_system_set(
                SystemSet::on_update(GameState::Combat).with_system(shake_camera_based_on_trauma),
            );
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

fn save_and_reset_camera_scale(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut overworld_camera_data: ResMut<OverworldCameraData>,
) {
    let mut camera_transform = camera_query.single_mut();
    overworld_camera_data.scale = camera_transform.scale;
    camera_transform.scale = Vec3::ONE;
}

fn restore_camera_scale(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    overworld_camera_data: Res<OverworldCameraData>,
) {
    let mut camera_transform = camera_query.single_mut();
    camera_transform.scale = overworld_camera_data.scale;
}

fn shake_camera_based_on_trauma(
    player_query: Query<&Player>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let player = player_query.single();

    camera_transform.translation.x = 0.0;
    camera_transform.translation.y = 0.0;
    camera_transform.rotation = Quat::IDENTITY;

    if player.trauma > 0.0 {
        let mut rng = thread_rng();
        let shake_amount = player.trauma * player.trauma;
        let max_angle = 15.0f32.to_radians();
        let max_offset = 0.2;

        let angle = max_angle * shake_amount * rng.gen_range(-1.0..1.0);
        let offset_x = max_offset * shake_amount * rng.gen_range(-1.0..1.0);
        let offset_y = max_offset * shake_amount * rng.gen_range(-1.0..1.0);

        camera_transform.translation.x += offset_x;
        camera_transform.translation.y += offset_y;
        camera_transform.rotation *= Quat::from_rotation_z(angle);
    }
}

fn zoom_into_center(mut camera_query: Query<&mut Transform, With<Camera2d>>, time: Res<Time>) {
    let mut camera_transform = camera_query.single_mut();
    let step = 0.3;
    if camera_transform.scale.x > 0.3 {
        camera_transform.scale.x -= step * time.delta_seconds();
        camera_transform.scale.y -= step * time.delta_seconds();
    }
}

fn spawn_camera(mut commands: Commands, old_camera_query: Query<Entity, With<Camera2d>>) {
    // Despawn old cameras if they exist
    for ent in old_camera_query.iter() {
        commands.entity(ent).despawn_recursive();
    }

    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.left = -1.0 * RESOLUTION;
    camera.orthographic_projection.right = 1.0 * RESOLUTION;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}
