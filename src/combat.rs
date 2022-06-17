use crate::ascii::{spawn_ascii_text, spawn_nine_slice, AsciiSheet, NineSlice, NineSliceIndices};
use crate::fadeout::create_fadeout;
use crate::game_ui::{CreateTextPopupEvent, TextPopupPosition};
use crate::graphics::{spawn_enemy_sprite, CharacterSheet};
use crate::player::Player;
use crate::{GameState, RESOLUTION, TILE_SIZE};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

pub struct CombatPlugin;

#[derive(Component, Inspectable)]
pub struct Enemy {
    enemy_type: EnemyType,
}

pub struct AttackEvent {
    target: Entity,
    pub damage_amount: isize,
    next_state: CombatState,
}

pub struct ExpReceivedEvent {
    pub(crate) levelup_percentage: f32,
}

pub struct LevelupEvent {
    pub(crate) new_level: usize,
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
    Dead,
}

pub struct AttackEffects {
    timer: Timer,
    flash_speed: f32,
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
                timer: Timer::from_seconds(0.4, true),
                flash_speed: 0.1,
            })
            .add_event::<AttackEvent>()
            .add_event::<LevelupEvent>()
            .add_event::<ExpReceivedEvent>()
            .insert_resource(CombatMenuSelection {
                selected: CombatMenuOption::Fight,
            })
            .add_system_set(
                SystemSet::on_update(CombatState::EnemyTurn(false)).with_system(process_enemy_turn),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Combat)
                    .with_system(process_attack)
                    .with_system(combat_input)
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
                SystemSet::on_update(CombatState::PlayerAttack).with_system(handle_attack_effects),
            )
            .add_system_set(
                SystemSet::on_update(CombatState::EnemyAttack).with_system(handle_attack_effects),
            )
            .add_system_set(
                SystemSet::on_enter(CombatState::Dead)
                    .with_system(show_game_over_screen)
                    .with_system(hide_player),
            );
    }
}

fn show_game_over_screen(mut commands: Commands, ascii: Res<AsciiSheet>) {
    let text = "GAME OVER";
    spawn_ascii_text(
        &mut commands,
        &ascii,
        text,
        Vec3::new(-((text.len() / 2) as f32 * TILE_SIZE), 0.0, 0.0),
    );
}

fn give_reward(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    mut player_query: Query<(&mut Player, &mut CombatStats)>,
    enemy_query: Query<&Enemy>,
    mut keyboard: ResMut<Input<KeyCode>>,
    mut ev_levelup: EventWriter<LevelupEvent>,
    mut ev_exp_received: EventWriter<ExpReceivedEvent>,
    mut ev_text_popup: EventWriter<CreateTextPopupEvent>,
) {
    keyboard.clear();

    let exp_reward = match enemy_query.single().enemy_type {
        EnemyType::Bat => 10,
        EnemyType::Ghost => 30,
    };
    let mut reward_text = format!("Earned {} exp", exp_reward);
    let (mut player, mut stats) = player_query.single_mut();
    if player.level_up(exp_reward, &mut stats) {
        ev_levelup.send(LevelupEvent {
            new_level: player.level,
        });
        reward_text += "\nLevel up!";
    }

    ev_text_popup.send(CreateTextPopupEvent {
        text: reward_text,
        position: TextPopupPosition::Left,
        duration: 2.5,
    });

    let levelup_percentage = player.exp as f32 / player.xp_required_for_current_level() as f32;
    ev_exp_received.send(ExpReceivedEvent { levelup_percentage });

    create_fadeout(&mut commands, None, &ascii);
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
        Vec3::new(-RESOLUTION + TILE_SIZE, -1.0 + TILE_SIZE + 0.035, 0.0) - transform.translation,
    );
    commands.entity(text).insert(CombatText);
    commands.entity(player).add_child(text);
}

fn handle_attack_effects(
    mut attack_fx: ResMut<AttackEffects>,
    time: Res<Time>,
    mut enemy_graphics_query: Query<&mut Visibility, (With<Enemy>, Without<Player>)>,
    mut player_graphics_query: Query<&mut Visibility, (With<Player>, Without<Enemy>)>,
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
    }

    let mut player_sprite = player_graphics_query.iter_mut().next().unwrap();
    if state.current() == &CombatState::EnemyAttack {
        if attack_fx.timer.elapsed_secs() % attack_fx.flash_speed > attack_fx.flash_speed / 2.0 {
            player_sprite.is_visible = false;
        } else {
            player_sprite.is_visible = true;
        }
    }

    if attack_fx.timer.just_finished() {
        enemy_sprite.is_visible = true;
        player_sprite.is_visible = true;
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
    mut attack_event: EventWriter<AttackEvent>,
    mut combat_state: ResMut<State<CombatState>>,
    enemy_query: Query<&CombatStats, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_ent = player_query.single();
    //todo support multiple enemies
    let enemy_stats = enemy_query.iter().next().unwrap();

    attack_event.send(AttackEvent {
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
    let box_center_y = -1.0 + box_height * TILE_SIZE / 2.0 + 0.04;

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

fn process_attack(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    mut attack_event: EventReader<AttackEvent>,
    text_query: Query<&Transform, With<CombatText>>,
    mut target_query: Query<(&Children, &mut CombatStats, Option<&mut Player>)>,
    mut combat_state: ResMut<State<CombatState>>,
) {
    for event in attack_event.iter() {
        let (target_children, mut target_stats, player_option) = target_query
            .get_mut(event.target)
            .expect("Fight target without stats!");

        // Lowest damage possible is 0 so we don't heal the target instead
        let resulting_damage = std::cmp::max(event.damage_amount - target_stats.defense, 0);

        let mut target_is_player = false;
        if let Some(mut player) = player_option {
            target_is_player = true;
            let big_hit_threshold = 0.35;
            let dmg_relative_to_hp = resulting_damage as f32 / target_stats.health as f32;
            let mut trauma = 1.0;
            if dmg_relative_to_hp < big_hit_threshold {
                trauma = dmg_relative_to_hp / big_hit_threshold;
            }

            player.trauma += trauma;
        }

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
            if target_is_player {
                combat_state.set(CombatState::Dead).unwrap();
            } else {
                combat_state.set(CombatState::Reward).unwrap();
            }
        } else {
            combat_state.set(event.next_state).unwrap();
        }
    }
}

#[allow(unused)]
fn test_give_player_trauma(mut player_query: Query<&mut Player>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Key1) {
        let mut player = player_query.single_mut();
        player.trauma += 0.1;
    }
    if keyboard.just_pressed(KeyCode::Key2) {
        let mut player = player_query.single_mut();
        player.trauma += 0.2;
    }
    if keyboard.just_pressed(KeyCode::Key3) {
        let mut player = player_query.single_mut();
        player.trauma += 0.3;
    }
    if keyboard.just_pressed(KeyCode::Key4) {
        let mut player = player_query.single_mut();
        player.trauma += 0.4;
    }
    if keyboard.just_pressed(KeyCode::Key5) {
        let mut player = player_query.single_mut();
        player.trauma += 0.5;
    }
    if keyboard.just_pressed(KeyCode::Key6) {
        let mut player = player_query.single_mut();
        player.trauma += 0.6;
    }
    if keyboard.just_pressed(KeyCode::Key7) {
        let mut player = player_query.single_mut();
        player.trauma += 0.7;
    }
    if keyboard.just_pressed(KeyCode::Key8) {
        let mut player = player_query.single_mut();
        player.trauma += 0.8;
    }
    if keyboard.just_pressed(KeyCode::Key9) {
        let mut player = player_query.single_mut();
        player.trauma += 0.9;
    }
    if keyboard.just_pressed(KeyCode::Key0) {
        let mut player = player_query.single_mut();
        player.trauma += 1.0;
    }
}

fn combat_input(
    mut commands: Commands,
    ascii: Res<AsciiSheet>,
    keyboard: Res<Input<KeyCode>>,
    mut fight_event: EventWriter<AttackEvent>,
    player_query: Query<&CombatStats, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut menu_state: ResMut<CombatMenuSelection>,
    combat_state: ResMut<State<CombatState>>,
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

    if keyboard.just_pressed(KeyCode::E) {
        match menu_state.selected {
            CombatMenuOption::Fight => {
                let player_stats = player_query.single();
                //todo handle multiple enemies and enemy selection
                let target = enemy_query.iter().next().unwrap();
                fight_event.send(AttackEvent {
                    target,
                    damage_amount: player_stats.attack,
                    next_state: CombatState::PlayerAttack,
                })
            }
            CombatMenuOption::Run => create_fadeout(&mut commands, None, &ascii),
        }
    }
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

fn hide_player(
    mut player_query: Query<&mut Visibility, With<Player>>
) {
    for mut visibility in player_query.iter_mut() {
        visibility.is_visible = false;
    }
}
