use crate::combat::EnemyType;
use crate::player::Player;
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

pub struct GraphicsPlugin;

pub struct CharacterSheet {
    pub handle: Handle<TextureAtlas>,
    pub player_up: [usize; 3],
    pub player_down: [usize; 3],
    pub player_left: [usize; 3],
    pub player_right: [usize; 3],
    pub bat_frames: [usize; 3],
    pub ghost_frames: [usize; 3],
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Inspectable)]
pub enum FacingDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Inspectable)]
pub struct PlayerGraphics {
    pub facing: FacingDirection,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct FrameAnimation {
    pub(crate) timer: Timer,
    pub(crate) frames: Vec<usize>,
    pub(crate) current_frame: usize,
}

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, Self::load_graphics)
            .add_system(Self::update_player_graphics)
            .add_system(Self::frame_animation);
    }
}

pub fn spawn_enemy_sprite(
    commands: &mut Commands,
    characters: &CharacterSheet,
    translation: Vec3,
    enemy_type: EnemyType,
) -> Entity {
    let mut sprite = match enemy_type {
        EnemyType::Bat => TextureAtlasSprite::new(characters.bat_frames[0]),
        EnemyType::Ghost => TextureAtlasSprite::new(characters.ghost_frames[0]),
    };
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    let frames = match enemy_type {
        EnemyType::Bat => characters.bat_frames.to_vec(),
        EnemyType::Ghost => characters.ghost_frames.to_vec(),
    };

    let animation = FrameAnimation {
        timer: Timer::from_seconds(0.2, true),
        frames,
        current_frame: 0,
    };

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: characters.handle.clone(),
            transform: Transform {
                translation,
                scale: Vec3::new(5.0, 5.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(animation)
        .id()
}

impl GraphicsPlugin {
    fn load_graphics(
        mut commands: Commands,
        assets: Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    ) {
        let columns = 12;

        let image = assets.load("img/characters.png");
        let atlas = TextureAtlas::from_grid_with_padding(
            image,
            Vec2::splat(16.0),
            columns,
            8,
            Vec2::splat(2.0),
        );
        let atlas_handle = texture_atlases.add(atlas);

        commands.insert_resource(CharacterSheet {
            handle: atlas_handle,
            player_down: [3, 4, 5],
            player_left: [columns + 3, columns + 4, columns + 5],
            player_right: [columns * 2 + 3, columns * 2 + 4, columns * 2 + 5],
            player_up: [columns * 3 + 3, columns * 3 + 4, columns * 3 + 5],
            bat_frames: [columns * 4 + 3, columns * 4 + 4, columns * 4 + 5],
            ghost_frames: [columns * 4 + 6, columns * 4 + 7, columns * 4 + 8],
        });
    }

    fn update_player_graphics(
        mut sprites_query: Query<(&PlayerGraphics, &mut FrameAnimation), Changed<PlayerGraphics>>,
        characters: Res<CharacterSheet>,
    ) {
        for (graphics, mut animation) in sprites_query.iter_mut() {
            animation.frames = match graphics.facing {
                FacingDirection::Up => characters.player_up.to_vec(),
                FacingDirection::Down => characters.player_down.to_vec(),
                FacingDirection::Left => characters.player_left.to_vec(),
                FacingDirection::Right => characters.player_right.to_vec(),
            }
        }
    }

    fn frame_animation(
        mut sprites_query: Query<(
            &mut TextureAtlasSprite,
            &mut FrameAnimation,
            Option<&Player>,
        )>,
        time: Res<Time>,
    ) {
        for (mut sprite, mut animation, player_option) in sprites_query.iter_mut() {
            let current_frame = match player_option {
                // if standing still, only show the middle sprite (index 1 out of [0, 1, 2])
                Some(player) if !player.just_moved => 1,
                _ => (animation.current_frame + 1) % animation.frames.len(),
            };

            animation.timer.tick(time.delta());
            if animation.timer.just_finished() {
                animation.current_frame = current_frame;
                sprite.index = animation.frames[animation.current_frame];
            }
        }
    }
}
