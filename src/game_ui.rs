use crate::combat::{ExpReceivedEvent, LevelupEvent};
use crate::GameState;
use bevy::prelude::*;

pub struct GameUiPlugin;

pub struct UiAssets {
    pub(crate) font: Handle<Font>,
    pub(crate) button: Handle<Image>,
    pub(crate) button_pressed: Handle<Image>,
}

#[derive(Component)]
pub struct LevelupText;

#[derive(Component)]
pub struct ExpBar;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui)
            .add_system_set(
                SystemSet::on_enter(GameState::Overworld)
                    .with_system(spawn_level_text)
                    .with_system(spawn_exp_bar),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat)
                    .with_system(handle_levelup_event)
                    .with_system(handle_exp_received_event),
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
        .insert(Name::new("Exp Bar"))
        .insert(ExpBar);
}

fn spawn_level_text(mut commands: Commands, ui_assets: Res<UiAssets>) {
    let text_style = TextStyle {
        font: ui_assets.font.clone(),
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
        .insert(Name::new("Levelup Text"))
        .insert(LevelupText);
}

fn setup_ui(mut commands: Commands, assets: Res<AssetServer>) {
    let ui_assets = UiAssets {
        font: assets.load("fonts/QuattrocentoSans-Bold.ttf"),
        button: assets.load("img/button.png"),
        button_pressed: assets.load("img/button_pressed.png"),
    };
    commands.insert_resource(ui_assets);
    commands.spawn_bundle(UiCameraBundle::default());
}
