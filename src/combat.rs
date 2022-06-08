use crate::ascii::{spawn_ascii_text, spawn_nine_slice, AsciiSheet, NineSlice, NineSliceIndices};
use crate::fadeout::create_fadeout;
use crate::graphics::{spawn_enemy_sprite, CharacterSheet};
use crate::player::Player;
use crate::{GameState, RESOLUTION, TILE_SIZE};
use bevy::prelude::*;
use bevy::render::camera::Camera2d;
use bevy_inspector_egui::Inspectable;

pub struct CombatPlugin;

#[derive(Component, Inspectable)]
pub struct Enemy {
    enemy_type: EnemyType,
}

pub struct FightEvent {
    target: Entity,
    pub damage_amount: isize,
    next_state: CombatState,
}

#[derive(Component, Inspectable)]
pub struct CombatStats {
    pub health: isize,
    pub max_health: isize,
    pub attack: isize,
    pub defense: isize,
}

pub const MENU_COUNT: isize = 2;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum CombatMenuOption {
    Fight,
    Run,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CombatMenuSelection {
    selected: CombatMenuOption,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CombatState {
    PlayerTurn,
    PlayerAttack,
    EnemyTurn(bool),
    EnemyAttack,
    Reward,
    Exiting,
}

pub struct AttackEffects {
    timer: Timer,
    flash_speed: f32,
    screen_shake_amount: f32,
    current_shake: f32,
}

#[derive(Component)]
pub struct CombatText;

#[derive(Clone, Copy, Inspectable)]
pub enum EnemyType {
    Bat,
    Ghost,
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(CombatState::PlayerTurn)
            .insert_resource(AttackEffects {
                timer: Timer::from_seconds(0.7, true),
                flash_speed: 0.1,
                screen_shake_amount: 0.1,
                current_shake: 0.0,
            })
            .add_event::<FightEvent>()
            .insert_resource(CombatMenuSelection {
                selected: CombatMenuOption::Fight,
            })
            .add_system_set(
                SystemSet::on_update(CombatState::EnemyTurn(false)).with_system(process_enemy_turn),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat)
                    .with_system(damage_calculation)
                    .with_system(combat_input)
                    .with_system(combat_camera)
                    .with_system(highlight_combat_buttons),
            )
            .add_system_set(
                SystemSet::on_enter(GameState::Combat)
                    .with_system(spawn_enemy)
                    .with_system(start_combat)
                    .with_system(spawn_player_health)
                    .with_system(spawn_combat_menu),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Combat)
                    .with_system(despawn_menu)
                    .with_system(despawn_all_combat_text)
                    .with_system(despawn_enemy),
            )
            .add_system_set(
                SystemSet::on_enter(CombatState::Reward)
                    .with_system(despawn_enemy)
                    .with_system(give_reward),
            )
            .add_system_set(
                SystemSet::on_update(CombatState::Reward).with_system(handle_accepting_reward),
            )
            .add_system_set(
                SystemSet::on_update(CombatState::PlayerAttack).with_system(handle_attack_effects),
            )
            .add_system_set(
                SystemSet::on_update(CombatState::EnemyAttack).with_system(handle_attack_effects),
            );
    }
}

fn handle_accepting_reward(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    keyboard: Res<Input<KeyCode>>,
    mut combat_state: ResMut<State<CombatState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        combat_state.set(CombatState::Exiting).unwrap();
        create_fadeout(&mut commands, None, &ascii);
    }
}

fn give_reward(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    mut player_query: Query<(&mut Player, &mut CombatStats)>,
    enemy_query: Query<&Enemy>,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    keyboard.clear();

    let exp_reward = match enemy_query.single().enemy_type {
        EnemyType::Bat => 10,
        EnemyType::Ghost => 30,
    };
    let reward_text = format!("Earned {} exp", exp_reward);

    let text = spawn_ascii_text(
        &mut commands,
        &ascii,
        &reward_text,
        Vec3::new(-((reward_text.len() / 2) as f32 * TILE_SIZE), 0.0, 0.0),
    );
    commands.entity(text).insert(CombatText);

    let (mut player, mut stats) = player_query.single_mut();
    if player.level_up(exp_reward, &mut stats) {
        let level_text = "Level up!";
        let text = spawn_ascii_text(
            &mut commands,
            &ascii,
            level_text,
            Vec3::new(
                -((level_text.len() / 2) as f32 * TILE_SIZE),
                -1.5 * TILE_SIZE,
                0.0,
            ),
        );
        commands.entity(text).insert(CombatText);
    }
}

fn despawn_all_combat_text(mut commands: Commands, text_query: Query<Entity, With<CombatText>>) {
    for entity in text_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_player_health(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    player_query: Query<(Entity, &CombatStats, &Transform), With<Player>>,
) {
    let (player, stats, transform) = player_query.single();

    let health_text = format!("Health: {}", stats.health);
    let text = spawn_ascii_text(
        &mut commands,
        &ascii,
        health_text.as_str(),
        Vec3::new(-RESOLUTION + TILE_SIZE, -1.0 + TILE_SIZE, 0.0) - transform.translation,
    );
    commands.entity(text).insert(CombatText);
    commands.entity(player).add_child(text);
}

fn handle_attack_effects(
    mut attack_fx: ResMut<AttackEffects>,
    time: Res<Time>,
    mut enemy_graphics_query: Query<&mut Visibility, With<Enemy>>,
    mut state: ResMut<State<CombatState>>,
) {
    attack_fx.timer.tick(time.delta());
    let mut enemy_sprite = enemy_graphics_query.iter_mut().next().unwrap();
    if state.current() == &CombatState::PlayerAttack {
        if attack_fx.timer.elapsed_secs() % attack_fx.flash_speed > attack_fx.flash_speed / 2.0 {
            enemy_sprite.is_visible = false;
        } else {
            enemy_sprite.is_visible = true;
        }
    } else {
        attack_fx.current_shake = attack_fx.screen_shake_amount
            * f32::sin(attack_fx.timer.percent() * 2.0 * std::f32::consts::PI);
    }

    if attack_fx.timer.just_finished() {
        enemy_sprite.is_visible = true;
        if state.current() == &CombatState::PlayerAttack {
            state.set(CombatState::EnemyTurn(false)).unwrap()
        } else {
            state.set(CombatState::PlayerTurn).unwrap()
        }
    }
}

fn start_combat(mut combat_state: ResMut<State<CombatState>>) {
    //TODO speed and turn calculations
    let _ = combat_state.set(CombatState::PlayerTurn);
}

fn process_enemy_turn(
    mut fight_event: EventWriter<FightEvent>,
    mut combat_state: ResMut<State<CombatState>>,
    enemy_query: Query<&CombatStats, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_ent = player_query.single();
    //todo support multiple enemies
    let enemy_stats = enemy_query.iter().next().unwrap();

    fight_event.send(FightEvent {
        target: player_ent,
        damage_amount: enemy_stats.attack,
        next_state: CombatState::EnemyAttack,
    });

    combat_state.set(CombatState::EnemyTurn(true)).unwrap();
}

fn despawn_menu(mut commands: Commands, button_query: Query<Entity, With<CombatMenuOption>>) {
    for button in button_query.iter() {
        commands.entity(button).despawn_recursive();
    }
}

fn highlight_combat_buttons(
    menu_state: Res<CombatMenuSelection>,
    button_query: Query<(&Children, &CombatMenuOption)>,
    nine_slice_query: Query<&Children, With<NineSlice>>,
    mut sprites_query: Query<&mut TextureAtlasSprite>,
) {
    for (button_children, button_id) in button_query.iter() {
        for button_child in button_children.iter() {
            if let Ok(nine_slice_children) = nine_slice_query.get(*button_child) {
                for nine_slice_child in nine_slice_children.iter() {
                    if let Ok(mut sprite) = sprites_query.get_mut(*nine_slice_child) {
                        if menu_state.selected == *button_id {
                            sprite.color = Color::RED;
                        } else {
                            sprite.color = Color::WHITE;
                        }
                    }
                }
            }
        }
    }
}

fn spawn_combat_button(
    commands: &mut Commands,
    ascii: &AsciiSheet,
    indices: &NineSliceIndices,
    translation: Vec3,
    text: &str,
    id: CombatMenuOption,
    size: Vec2,
) -> Entity {
    let nine_slice = spawn_nine_slice(commands, ascii, indices, size.x, size.y);

    let x_offset = (-size.x / 2.0 + 1.5) * TILE_SIZE;
    let text = spawn_ascii_text(commands, ascii, text, Vec3::new(x_offset, 0.0, 0.0));

    commands
        .spawn()
        .insert(Transform {
            translation,
            ..default()
        })
        .insert(GlobalTransform::default())
        .insert(Name::new("Button"))
        .insert(id)
        .add_child(nine_slice)
        .add_child(text)
        .id()
}

fn spawn_combat_menu(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    nine_slice_indices: Res<NineSliceIndices>,
) {
    let box_height = 3.0;
    let box_center_y = -1.0 + box_height * TILE_SIZE / 2.0;

    let run_text = "Run";
    let run_width = (run_text.len() + 2) as f32;
    let run_center_x = RESOLUTION - (run_width * TILE_SIZE) / 2.0;

    spawn_combat_button(
        &mut commands,
        &ascii,
        &nine_slice_indices,
        Vec3::new(run_center_x, box_center_y, 100.0),
        run_text,
        CombatMenuOption::Run,
        Vec2::new(run_width, box_height),
    );

    let fight_text = "Fight";
    let fight_width = (fight_text.len() + 2) as f32;
    let fight_center_x = RESOLUTION - (run_width * TILE_SIZE) - (fight_width * TILE_SIZE / 2.0);

    spawn_combat_button(
        &mut commands,
        &ascii,
        &nine_slice_indices,
        Vec3::new(fight_center_x, box_center_y, 100.0),
        fight_text,
        CombatMenuOption::Fight,
        Vec2::new(fight_width, box_height),
    );
}

fn damage_calculation(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    mut fight_event: EventReader<FightEvent>,
    text_query: Query<&Transform, With<CombatText>>,
    mut target_query: Query<(&Children, &mut CombatStats)>,
    mut combat_state: ResMut<State<CombatState>>,
) {
    for event in fight_event.iter() {
        let (target_children, mut target_stats) = target_query
            .get_mut(event.target)
            .expect("Fight target without stats!");

        // Lowest damage possible is 0 so we don't heal the target instead
        let resulting_damage = std::cmp::max(event.damage_amount - target_stats.defense, 0);

        target_stats.health = std::cmp::max(target_stats.health - resulting_damage, 0);

        // Update Health
        for child in target_children.iter() {
            if let Ok(transform) = text_query.get(*child) {
                // Delete old text
                commands.entity(*child).despawn_recursive();

                let new_health = spawn_ascii_text(
                    &mut commands,
                    &ascii,
                    &format!("Health: {}", target_stats.health),
                    transform.translation,
                );
                commands.entity(new_health).insert(CombatText);
                commands.entity(event.target).add_child(new_health);
            }
        }

        if target_stats.health == 0 {
            combat_state.set(CombatState::Reward).unwrap();
        } else {
            combat_state.set(event.next_state).unwrap();
        }
    }
}

fn combat_input(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    keyboard: Res<Input<KeyCode>>,
    mut fight_event: EventWriter<FightEvent>,
    player_query: Query<&CombatStats, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut menu_state: ResMut<CombatMenuSelection>,
    mut combat_state: ResMut<State<CombatState>>,
) {
    if combat_state.current() != &CombatState::PlayerTurn {
        return;
    }

    let mut new_selection = menu_state.selected as isize;

    if keyboard.just_pressed(KeyCode::A) {
        new_selection -= 1;
    }
    if keyboard.just_pressed(KeyCode::D) {
        new_selection += 1;
    }

    new_selection = (new_selection + MENU_COUNT) % MENU_COUNT;

    menu_state.selected = match new_selection {
        0 => CombatMenuOption::Fight,
        1 => CombatMenuOption::Run,
        _ => panic!("Bad menu selection"),
    };

    if keyboard.just_pressed(KeyCode::Space) {
        match menu_state.selected {
            CombatMenuOption::Fight => {
                let player_stats = player_query.single();
                //todo handle multiple enemies and enemy selection
                let target = enemy_query.iter().next().unwrap();
                fight_event.send(FightEvent {
                    target,
                    damage_amount: player_stats.attack,
                    next_state: CombatState::PlayerAttack,
                })
            }
            CombatMenuOption::Run => {
                create_fadeout(&mut commands, None, &ascii);
                combat_state.set(CombatState::Exiting).unwrap();
            }
        }
    }
}

fn combat_camera(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    attack_fx: Res<AttackEffects>,
) {
    let mut camera_transform = camera_query.single_mut();
    camera_transform.translation.x = attack_fx.current_shake;
    camera_transform.translation.y = 0.0;
}

fn spawn_enemy(mut commands: Commands, ascii: Res<AsciiSheet>, characters: Res<CharacterSheet>) {
    let enemy_type = match rand::random::<f32>() {
        x if x < 0.5 => EnemyType::Bat,
        _ => EnemyType::Ghost,
    };

    let stats = match enemy_type {
        EnemyType::Bat => CombatStats {
            health: 3,
            max_health: 3,
            attack: 2,
            defense: 1,
        },
        EnemyType::Ghost => CombatStats {
            health: 5,
            max_health: 5,
            attack: 3,
            defense: 2,
        },
    };

    let health_text = spawn_ascii_text(
        &mut commands,
        &ascii,
        &format!("Health: {}", stats.health),
        Vec3::new(-4.5 * TILE_SIZE, 3.0 * TILE_SIZE, 100.0),
    );

    commands.entity(health_text).insert(CombatText);

    let sprite = spawn_enemy_sprite(
        &mut commands,
        &characters,
        Vec3::new(0.0, 0.6, 100.0),
        enemy_type,
    );

    commands
        .entity(sprite)
        .insert(Enemy { enemy_type })
        .insert(stats)
        .insert(Name::new("Enemy"))
        .add_child(health_text);
}

fn despawn_enemy(mut commands: Commands, enemy_query: Query<Entity, With<Enemy>>) {
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
