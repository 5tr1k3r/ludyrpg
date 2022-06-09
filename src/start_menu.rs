use crate::ascii::AsciiSheet;
use crate::fadeout::create_fadeout;
use crate::GameState;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub struct MainMenuPlugin;

struct UiAssets {
    font: Handle<Font>,
    button: Handle<Image>,
    button_pressed: Handle<Image>,
}

#[derive(Component)]
pub struct ButtonActive(bool);

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_menu)
            .add_system_set(SystemSet::on_pause(GameState::StartMenu).with_system(despawn_menu))
            .add_system(handle_start_button);
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

fn setup_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let ui_assets = UiAssets {
        font: assets.load("fonts/QuattrocentoSans-Bold.ttf"),
        button: assets.load("img/button.png"),
        button_pressed: assets.load("img/button_pressed.png"),
    };

    commands.spawn_bundle(UiCameraBundle::default());
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
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
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
                    });
                });
        });

    commands.insert_resource(ui_assets);
}
