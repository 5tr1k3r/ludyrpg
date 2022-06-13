use crate::combat::{AttackEvent, CombatState};
use crate::player::{Player, WalkedGroundType};
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioChannel, AudioPlugin, AudioSource};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct GameAudioPlugin;

struct BgmChannel;

struct CombatChannel;

struct SfxChannel;

pub struct AudioState {
    bgm_handle: Handle<AudioSource>,
    combat_handle: Handle<AudioSource>,
    hit_handle: Handle<AudioSource>,
    reward_handle: Handle<AudioSource>,
    death_handle: Handle<AudioSource>,
    bgm_volume: f32,
}

struct NormalFootsteps(Vec<Handle<AudioSource>>);

struct GrassFootsteps(Vec<Handle<AudioSource>>);

struct FootstepsTimer {
    timer: Timer,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_audio_channel::<BgmChannel>()
            .add_audio_channel::<CombatChannel>()
            .add_audio_channel::<SfxChannel>()
            .add_startup_system_to_stage(StartupStage::PreStartup, load_audio)
            .add_startup_system(start_bgm_music)
            .add_system(bgm_volume_control)
            .add_system(play_hit_sfx)
            .add_system_set(SystemSet::on_enter(GameState::Combat).with_system(start_combat_music))
            .add_system_set(SystemSet::on_enter(CombatState::Reward).with_system(play_reward_sfx))
            .add_system_set(SystemSet::on_enter(CombatState::Dead).with_system(play_death_sfx))
            .add_system_set(
                SystemSet::on_update(GameState::Overworld).with_system(play_footsteps_sfx),
            )
            .add_system_set(SystemSet::on_exit(GameState::Combat).with_system(stop_combat_music));
    }
}

fn play_footsteps_sfx(
    normal_footsteps: Res<NormalFootsteps>,
    grass_footsteps: Res<GrassFootsteps>,
    mut footsteps_timer: ResMut<FootstepsTimer>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    player_query: Query<&Player>,
    time: Res<Time>,
) {
    let player = player_query.single();
    if !player.just_moved {
        return;
    }

    footsteps_timer.timer.tick(time.delta());
    if footsteps_timer.timer.just_finished() {
        let fs_sound = match player.walked_ground_type {
            WalkedGroundType::Normal => pick_random_sound(&normal_footsteps.0),
            WalkedGroundType::Grass => pick_random_sound(&grass_footsteps.0),
        };
        sfx_channel.play(fs_sound);
    }
}

//noinspection RsTypeCheck
fn pick_random_sound(sounds: &Vec<Handle<AudioSource>>) -> Handle<AudioSource> {
    let mut rng = thread_rng();

    sounds.choose(&mut rng).unwrap().clone()
}

fn play_death_sfx(combat_channel: Res<AudioChannel<CombatChannel>>, audio_state: Res<AudioState>) {
    combat_channel.stop();
    combat_channel.set_volume(0.4);
    combat_channel.play(audio_state.death_handle.clone());
}

fn play_reward_sfx(sfx_channel: Res<AudioChannel<SfxChannel>>, audio_state: Res<AudioState>) {
    sfx_channel.play(audio_state.reward_handle.clone());
}

fn play_hit_sfx(
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    audio_state: Res<AudioState>,
    mut attack_event: EventReader<AttackEvent>,
) {
    if attack_event.iter().count() > 0 {
        sfx_channel.play(audio_state.hit_handle.clone());
    }
}

fn bgm_volume_control(
    keyboard: Res<Input<KeyCode>>,
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    mut audio_state: ResMut<AudioState>,
) {
    let step = 0.05;

    if keyboard.just_pressed(KeyCode::Up) {
        audio_state.bgm_volume += step;
    }
    if keyboard.just_pressed(KeyCode::Down) {
        audio_state.bgm_volume -= step;
    }

    audio_state.bgm_volume = audio_state.bgm_volume.clamp(0.0, 1.0);
    bgm_channel.set_volume(audio_state.bgm_volume);
}

fn start_bgm_music(bgm_channel: Res<AudioChannel<BgmChannel>>, audio_state: Res<AudioState>) {
    bgm_channel.play_looped(audio_state.bgm_handle.clone());
}

fn start_combat_music(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    combat_channel: Res<AudioChannel<CombatChannel>>,
    audio_state: Res<AudioState>,
) {
    bgm_channel.pause();
    combat_channel.play_looped(audio_state.combat_handle.clone());
}

fn stop_combat_music(
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    combat_channel: Res<AudioChannel<CombatChannel>>,
) {
    combat_channel.stop();
    bgm_channel.resume();
}

fn load_audio(
    mut commands: Commands,
    assets: Res<AssetServer>,
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    combat_channel: Res<AudioChannel<CombatChannel>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
) {
    let bgm_handle = assets.load("music/bip-bop.ogg");
    let combat_handle = assets.load("music/ganxta.ogg");
    let hit_handle = assets.load("sounds/hit.wav");
    let reward_handle = assets.load("sounds/reward.wav");
    let death_handle = assets.load("sounds/dead.wav");

    let bgm_volume = 0.05;
    let combat_volume = 0.2;
    let sfx_volume = 0.1;

    bgm_channel.set_volume(bgm_volume);
    combat_channel.set_volume(combat_volume);
    sfx_channel.set_volume(sfx_volume);

    commands.insert_resource(AudioState {
        bgm_handle,
        combat_handle,
        hit_handle,
        reward_handle,
        death_handle,
        bgm_volume,
    });

    let normal_footsteps: Vec<Handle<AudioSource>> = [
        "sounds/footstep_concrete_000.ogg",
        "sounds/footstep_concrete_001.ogg",
        "sounds/footstep_concrete_002.ogg",
        "sounds/footstep_concrete_003.ogg",
        "sounds/footstep_concrete_004.ogg",
    ]
    .iter()
    .map(|&name| assets.load(name))
    .collect();

    let grass_footsteps: Vec<Handle<AudioSource>> = [
        "sounds/footstep_grass_000.ogg",
        "sounds/footstep_grass_001.ogg",
        "sounds/footstep_grass_002.ogg",
        "sounds/footstep_grass_003.ogg",
        "sounds/footstep_grass_004.ogg",
    ]
    .iter()
    .map(|&name| assets.load(name))
    .collect();

    commands.insert_resource(NormalFootsteps(normal_footsteps));
    commands.insert_resource(GrassFootsteps(grass_footsteps));
    commands.insert_resource(FootstepsTimer {
        timer: Timer::from_seconds(0.35, true),
    });
}
