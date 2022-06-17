use crate::combat::{ExpReceivedEvent, LevelupEvent};
use crate::GameState;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_inspector_egui::Inspectable;

pub struct GameUiPlugin;

pub struct UiAssets {
    pub(crate) font: Handle<Font>,
    pub(crate) font_bold: Handle<Font>,
    pub(crate) button: Handle<Image>,
    pub(crate) button_pressed: Handle<Image>,
}

#[derive(Component)]
pub struct LevelupText;

#[derive(Component)]
pub struct TextPopup {
    timer: Timer,
    when_start_fading: f32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TextPopupPosition {
    Left,
    Center,
}

#[derive(Component, Inspectable)]
pub struct ExpBar {
    width: f32,
    target_width: f32,
    progress_duration: f32,
    progress_step: f32,
}

pub struct CreateTextPopupEvent {
    pub(crate) text: String,
    pub(crate) position: TextPopupPosition,
    pub(crate) duration: f32,
}

#[derive(Component)]
pub struct HealthBarBg;

#[derive(Component)]
pub struct HealthBar {
    pub(crate) entity: Entity,
}

pub enum HealthBarType {
    Player,
    Enemy,
}

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateTextPopupEvent>()
            .add_startup_system(setup_ui)
            .add_system_set(
                SystemSet::on_enter(GameState::Overworld)
                    .with_system(spawn_level_text)
                    .with_system(show_help_initially)
                    .with_system(spawn_exp_bar),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat).with_system(handle_levelup_event),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Overworld).with_system(show_help_on_button_press),
            )
            .add_system(handle_exp_received_event)
            .add_system(handle_text_popup_event)
            .add_system(update_text_popups);
    }
}

pub fn create_health_bar(commands: &mut Commands, hb_type: HealthBarType, owner: Entity) -> Entity {
    let health_bar_bg = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: match hb_type {
                    HealthBarType::Player => Color::rgb(0.08, 0.31, 0.02),
                    HealthBarType::Enemy => Color::rgb(0.31, 0.08, 0.02),
                },
                custom_size: Some(Vec2::new(0.104, 0.022)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.07, 0.5),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("HealthBarBg"))
        .insert(HealthBarBg)
        .id();

    let health_bar = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: match hb_type {
                    HealthBarType::Player => Color::rgb(0.31, 0.66, 0.23),
                    HealthBarType::Enemy => Color::rgb(0.66, 0.31, 0.23),
                },
                custom_size: Some(Vec2::new(0.1, 0.018)),
                anchor: Anchor::CenterLeft,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(-0.05, 0.0, 0.1),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("HealthBar"))
        .insert(HealthBar { entity: owner })
        .id();

    commands.entity(health_bar_bg).add_child(health_bar);

    health_bar_bg
}

fn show_help_initially(ev_text_popup: EventWriter<CreateTextPopupEvent>) {
    show_help(ev_text_popup);
}

fn show_help(mut ev_text_popup: EventWriter<CreateTextPopupEvent>) {
    let text = r"Controls:
  WASD: movement
  Esc: go to menu
  Up, Down, M: volume control
  Num+, Num-, Home: camera control
  E: interact
  A, D: select option
  H: show help"
        .to_string();
    ev_text_popup.send(CreateTextPopupEvent {
        text,
        position: TextPopupPosition::Left,
        duration: 5.0,
    });
}

fn show_help_on_button_press(
    ev_text_popup: EventWriter<CreateTextPopupEvent>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::H) {
        show_help(ev_text_popup);
    }
}

fn update_text_popups(
    mut commands: Commands,
    mut query: Query<(&Parent, &mut Text, &mut TextPopup)>,
    time: Res<Time>,
) {
    for (parent_entity, mut text, mut popup) in query.iter_mut() {
        popup.timer.tick(time.delta());
        let percent_left = popup.timer.percent_left();
        if percent_left < popup.when_start_fading {
            text.sections[0]
                .style
                .color
                .set_a(percent_left / popup.when_start_fading);
        }

        if popup.timer.just_finished() {
            commands.entity(parent_entity.0).despawn_recursive();
        }
    }
}

fn handle_text_popup_event(
    mut commands: Commands,
    mut ev_text_popup: EventReader<CreateTextPopupEvent>,
    ui_assets: Res<UiAssets>,
    popup_query: Query<&Parent, With<TextPopup>>,
) {
    for event in ev_text_popup.iter() {
        // Destroy old popup
        for popup_parent_node in popup_query.iter() {
            commands.entity(popup_parent_node.0).despawn_recursive();
        }

        // Create new one
        create_text_popup(
            &mut commands,
            &ui_assets,
            event.text.as_str(),
            event.position,
            event.duration,
        );
    }
}

fn handle_exp_received_event(
    mut ev_exp_received: EventReader<ExpReceivedEvent>,
    mut exp_bar_query: Query<(&mut Style, &mut ExpBar)>,
    time: Res<Time>,
    game_state: Res<State<GameState>>,
) {
    if game_state.current() == &GameState::StartMenu {
        return;
    }

    let (mut style, mut exp_bar) = exp_bar_query.single_mut();
    for event in ev_exp_received.iter() {
        exp_bar.target_width = event.levelup_percentage;
        if exp_bar.target_width < exp_bar.width {
            exp_bar.target_width += 1.0;
        }

        let width_delta = exp_bar.target_width - exp_bar.width;
        exp_bar.progress_step = time.delta_seconds() * width_delta / exp_bar.progress_duration;
    }

    if exp_bar.progress_step != 0.0 {
        if exp_bar.width > exp_bar.target_width {
            exp_bar.width = exp_bar.target_width;
            exp_bar.progress_step = 0.0;
        }

        exp_bar.width += exp_bar.progress_step;
        if exp_bar.width >= 1.0 {
            exp_bar.width -= 1.0;
            exp_bar.target_width -= 1.0;
        }

        style.size.width = Val::Percent(exp_bar.width * 100.0);
    }
}

fn handle_levelup_event(
    mut ev_levelup: EventReader<LevelupEvent>,
    mut levelup_text_query: Query<&mut Text, With<LevelupText>>,
) {
    for event in ev_levelup.iter() {
        let mut levelup_text = levelup_text_query.single_mut();
        levelup_text.sections[0].value = format!("Level {}", event.new_level)
    }
}

fn spawn_exp_bar(mut commands: Commands) {
    let style = Style {
        position_type: PositionType::Absolute,
        position: Rect {
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Percent(0.0),
        },
        size: Size {
            width: Val::Percent(0.0),
            height: Val::Percent(1.0),
        },
        ..default()
    };

    commands
        .spawn_bundle(ImageBundle {
            style,
            color: UiColor::from(Color::GOLD),
            ..default()
        })
        .insert(Name::new("ExpBar"))
        .insert(ExpBar {
            width: 0.0,
            target_width: 0.0,
            progress_duration: 0.6,
            progress_step: 0.0,
        });
}

fn spawn_level_text(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let text_style = TextStyle {
        font: ui_assets.font_bold.clone(),
        font_size: 20.0,
        color: Color::GOLD,
    };

    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    let style = Style {
        align_self: AlignSelf::Center,
        margin: Rect {
            left: Val::Auto,
            right: Val::Auto,
            top: Val::Auto,
            bottom: Val::Percent(1.1),
        },
        ..default()
    };

    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section("Level 1", text_style, text_alignment),
            style,
            ..default()
        })
        .insert(Name::new("LevelupText"))
        .insert(LevelupText);
}

fn setup_ui(mut commands: Commands, assets: Res<AssetServer>) {
    let ui_assets = UiAssets {
        font_bold: assets.load("fonts/QuattrocentoSans-Bold.ttf"),
        font: assets.load("fonts/QuattrocentoSans-Regular.ttf"),
        button: assets.load("img/button.png"),
        button_pressed: assets.load("img/button_pressed.png"),
    };
    commands.insert_resource(ui_assets);
    commands.spawn_bundle(UiCameraBundle::default());
}

pub fn create_text_popup(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    text: &str,
    position: TextPopupPosition,
    duration: f32,
) {
    let text_style = TextStyle {
        font: ui_assets.font.clone(),
        font_size: 30.0,
        color: Color::rgb(0.9, 0.9, 0.9),
    };

    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Left,
    };

    let node_style = Style {
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexEnd,
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        ..default()
    };

    let style = Style {
        position_type: PositionType::Relative,
        align_self: AlignSelf::Center,
        margin: Rect {
            left: match position {
                TextPopupPosition::Left => Val::Percent(1.0),
                _ => Val::Auto,
            },
            right: Val::Auto,
            top: Val::Percent(1.0),
            bottom: Val::Auto,
        },
        ..default()
    };

    commands
        .spawn_bundle(NodeBundle {
            style: node_style,
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(text, text_style, text_alignment),
                    style,
                    ..default()
                })
                .insert(Name::new("TextPopup"))
                .insert(TextPopup {
                    timer: Timer::from_seconds(duration, false),
                    when_start_fading: 0.3,
                });
        });
}
