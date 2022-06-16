use crate::ascii::AsciiSheet;
use crate::audio::{AudioState, BgmChannel};
use crate::combat::CombatState;
use crate::fadeout::create_fadeout;
use crate::game_ui::UiAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_kira_audio::AudioChannel;

pub struct MainMenuPlugin;

#[derive(Component)]
pub struct ButtonActive(bool);

#[derive(Component)]
pub struct StartMenuButton;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_pause(GameState::StartMenu).with_system(despawn_menu))
            .add_system_set(SystemSet::on_update(GameState::Overworld).with_system(return_to_menu))
            .add_system_set(SystemSet::on_update(CombatState::Dead).with_system(return_to_menu))
            .add_system_set(
                SystemSet::on_enter(GameState::StartMenu)
                    .with_system(reset_game)
                    .with_system(spawn_menu),
            )
            .add_system(handle_start_button);
    }
}

fn reset_game(
    mut commands: Commands,
    entity_query: Query<Entity, Without<CameraUi>>,
    mut combat_state: ResMut<State<CombatState>>,
    bgm_channel: Res<AudioChannel<BgmChannel>>,
    audio_state: Res<AudioState>,
) {
    // Despawn all entities except UI camera
    for ent in entity_query.iter() {
        commands.entity(ent).despawn_recursive();
    }

    // Reset combat state
    if combat_state.current() != &CombatState::PlayerTurn {
        combat_state.set(CombatState::PlayerTurn).unwrap();
    }

    // Start bg music
    bgm_channel.stop();
    bgm_channel.play_looped(audio_state.bgm_handle.clone());
}

fn return_to_menu(mut commands: Commands, ascii: Res<AsciiSheet>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        create_fadeout(&mut commands, Some(GameState::StartMenu), &ascii);
    }
}

fn despawn_menu(mut commands: Commands, button_query: Query<Entity, With<Button>>) {
    for ent in button_query.iter() {
        commands.entity(ent).despawn_recursive();
    }
}

fn handle_start_button(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Children, &mut ButtonActive, &Interaction),
        Changed<Interaction>,
    >,
    mut image_query: Query<&mut UiImage>,
    ui_assets: Res<UiAssets>,
    ascii: Res<AsciiSheet>,
) {
    for (children, mut active, interaction) in interaction_query.iter_mut() {
        let child = children.iter().next().unwrap();
        let mut image = image_query.get_mut(*child).unwrap();

        match interaction {
            Interaction::Clicked => {
                if active.0 {
                    image.0 = ui_assets.button_pressed.clone();
                    create_fadeout(&mut commands, Some(GameState::Overworld), &ascii);
                    active.0 = false;
                }
            }
            Interaction::Hovered | Interaction::None => {
                image.0 = ui_assets.button.clone();
            }
        }
    }
}

fn spawn_menu(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn_bundle(ButtonBundle {
            node: Default::default(),
            button: Default::default(),
            style: Style {
                align_self: AlignSelf::Center,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Percent(20.0), Val::Percent(10.0)),
                margin: Rect::all(Val::Auto),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(ButtonActive(true))
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    image: ui_assets.button.clone().into(),
                    ..default()
                })
                .insert(FocusPolicy::Pass)
                .insert(StartMenuButton)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                "Start Game",
                                TextStyle {
                                    font: ui_assets.font.clone(),
                                    font_size: 40.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                Default::default(),
                            ),
                            focus_policy: FocusPolicy::Pass,
                            ..default()
                        })
                        .insert(StartMenuButton);
                });
        });
}
