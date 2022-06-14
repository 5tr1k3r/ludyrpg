use crate::combat::CombatState;
use crate::player::Player;
use crate::{GameState, RESOLUTION};
use bevy::prelude::*;
use bevy::render::camera::{Camera2d, ScalingMode};
use rand::{thread_rng, Rng};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Overworld).with_system(spawn_camera))
            .add_system_set(SystemSet::on_update(CombatState::Dead).with_system(zoom_into_center))
            .add_system_set(
                SystemSet::on_update(GameState::Combat).with_system(shake_camera_based_on_trauma),
            );
    }
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
    // camera_transform.scale = Vec3::ONE;

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
