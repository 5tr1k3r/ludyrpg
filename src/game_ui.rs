use crate::combat::{ExpReceivedEvent, LevelupEvent};
use crate::GameState;
use bevy::prelude::*;

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

#[derive(Component)]
pub struct ExpBar;

pub struct CreateTextPopupEvent {
    pub(crate) text: String,
    pub(crate) position: TextPopupPosition,
}

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateTextPopupEvent>()
            .add_startup_system(setup_ui)
            .add_system_set(
                SystemSet::on_enter(GameState::Overworld)
                    .with_system(spawn_level_text)
                    .with_system(spawn_exp_bar),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat)
                    .with_system(handle_levelup_event)
                    .with_system(handle_exp_received_event),
            )
            .add_system(handle_text_popup_event)
            .add_system(update_text_popups);
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
        );
    }
}

fn handle_exp_received_event(
    mut ev_exp_received: EventReader<ExpReceivedEvent>,
    mut exp_bar_query: Query<&mut Style, With<ExpBar>>,
) {
    for event in ev_exp_received.iter() {
        let mut exp_bar = exp_bar_query.single_mut();
        exp_bar.size.width = Val::Percent(event.levelup_percentage * 100.0);
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
        .insert(ExpBar);
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
                    timer: Timer::from_seconds(3.0, false),
                    when_start_fading: 0.3,
                });
        });
}
