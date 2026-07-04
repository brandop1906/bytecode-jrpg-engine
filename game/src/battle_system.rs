use bevy::ecs::entity;
use bevy::ecs::system::command;
use rand::Rng;
use bevy::prelude::*;
use crate::party;
use crate::party::*;
use crate::scene::*;
use crate::state::*;
use crate::stats;
use std::collections::HashMap;
use crate::player::*;
use crate::stats::*;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ActionReady;


#[derive(Resource)]
pub struct EncounterTracker {
    pub danger: f32,
}

pub fn encounter_check_system(
    mut encounter_tracker: ResMut<EncounterTracker>,
    time: Res<Time>, player_moving: Res<ButtonInput<KeyCode>>,
    current_scene: Res<SceneLibrary>, mut next_state: ResMut<NextState<GameState>>,
) {
    let mut rng = rand::thread_rng();
    if player_moving.pressed(KeyCode::ArrowLeft) || player_moving.pressed(KeyCode::ArrowRight) ||
       player_moving.pressed(KeyCode::ArrowUp) || player_moving.pressed(KeyCode::ArrowDown) ||
       player_moving.pressed(KeyCode::KeyA) || player_moving.pressed(KeyCode::KeyD) ||
       player_moving.pressed(KeyCode::KeyW) || player_moving.pressed(KeyCode::KeyS) {
        if let Some(scene_def) = current_scene.get_current_scene() {
            encounter_tracker.danger += scene_def.encounter_rate * time.delta_secs();
            if encounter_tracker.danger >= scene_def.encounter_threshold {
                let roll: f32 = rng.gen_range(0.0..1.0);
                encounter_tracker.danger = 0.0;
                if roll < 0.9 {
                    println!("Encounter triggered! Transitioning to battle state.");
                    next_state.set(GameState::Battle);
                } 
            }
        }
    }
}

#[derive(Component)]
pub struct BattleEntity;

#[derive(Clone)]
pub struct EnemyDef {
    pub name: String,
    pub sprite: String,
    pub stats: BattlerStats,
}

#[derive(Resource)]
pub struct EnemyLibrary {
    pub enemies: HashMap<String, EnemyDef>,
}

impl EnemyLibrary {
    pub fn new() -> Self {
        EnemyLibrary { enemies: HashMap::new() }
    }

    pub fn add_enemy(&mut self, id: String, enemy: EnemyDef) {
        self.enemies.insert(id, enemy);
    }
}

pub fn setup_battle(mut commands: Commands, asset_server: Res<AssetServer>, mut player_query: Query<&mut Visibility, With<PlayerControlled>>,
    enemy_lib: Res<EnemyLibrary>, player_lib: Res<PlayerLibrary>, party_state: Res<PartyState>, window_query: Query<&Window>) {
    
    if let Ok(mut visibility) = player_query.single_mut() {
        *visibility = Visibility::Hidden;
    }

    commands.spawn((
        BattleEntity,
        Sprite {
            image: asset_server.load("battle_reactor.png"),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let enemy = enemy_lib.enemies.get("mako_guard").unwrap();
    commands.spawn((
        BattleEntity,
        Enemy,
        enemy.stats.clone(),
        Sprite {
            image: asset_server.load(enemy.sprite.clone()),
            custom_size: Some(Vec2::new(64.0, 64.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 1.0),
    ));

    let player = player_lib.get_player("Zane".to_string()).unwrap();
    let mut party_stats = party_state.members[0].clone();
    party_stats.atb_timer = 0.0;
    commands.spawn((
        BattleEntity,
        Player,
        party_stats,
        Sprite {
            image: asset_server.load(player.sprite.clone()),
            custom_size: Some(Vec2::new(64.0, 64.0)),
            ..default()
        },
        Transform::from_xyz(-200.0, 0.0, 1.0),
    ));

    commands.spawn((
        BattleEntity,
        PlayerAtbUi {
            left_edge_x: 450.0,
            full_width: 100.0,
        },
        Sprite {
            color: Color::srgb(1.0, 0.5, 0.5),
            custom_size: Some(Vec2::new(100.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(500.0, -280.0, 2.1),
    ));

    commands.spawn((
        BattleEntity,
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(100.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(500.0, -280.0, 2.0),
    ));

    commands.spawn((
        BattleEntity,
        HpText,
        Text2d::new("HP --/--"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-500.0, -280.0, 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));

    commands.spawn((
        BattleEntity,
        MpText,
        Text2d::new("MP --/--"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-500.0, -310.0, 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));

    commands.spawn((
        BattleEntity,
        Visibility::Hidden,
        MenuOption { index: 0 },
        Text2d::new("Attack"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-150.0, -270.0, 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));

    commands.spawn((
        BattleEntity,
        Visibility::Hidden,
        MenuOption { index: 1 },
        Text2d::new("Magic"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-150.0, -300.0, 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));

    commands.spawn((
        BattleEntity,
        Visibility::Hidden,
        MenuOption { index: 2 },
        Text2d::new("Item"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-150.0, -330.0, 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));


    // Get the screen width from the primary window (auto-fits any resolution)
    let screen_width = window_query
        .single()
        .map(|w| w.width())
        .unwrap_or(1280.0);          // fallback if the window isn't found

    let bar_width = screen_width + 20.0;   // slight overshoot so it reaches both edges
    let bar_y = -290.0;                     // bottom of screen
    let bar_height = 140.0;
    commands.spawn((
        BattleEntity,
        Sprite {
            color: Color::srgb(0.4, 0.6, 0.9),
            custom_size: Some(Vec2::new(bar_width, bar_height)),
            ..default()
        },
        Transform::from_xyz(0.0, bar_y, 1.5),
    ));
    // Fill (dark navy), inset vertically only so a top/bottom border shows
    commands.spawn((
        BattleEntity,
        Sprite {
            color: Color::srgb(0.05, 0.1, 0.35),
            custom_size: Some(Vec2::new(bar_width, bar_height - 8.0)),
            ..default()
        },
        Transform::from_xyz(0.0, bar_y, 1.6),
    ));

    // --- Command menu window: border (lighter blue) ---
    commands.spawn((
        BattleEntity,
        MenuWindow,
        Visibility::Hidden,
        Sprite {
            color: Color::srgb(0.4, 0.6, 0.9),
            custom_size: Some(Vec2::new(160.0, 110.0)),
            ..default()
        },
        Transform::from_xyz(-150.0, -300.0, 1.7),
    ));
    // --- Command menu window: fill (dark navy, inset for a border) ---
    commands.spawn((
        BattleEntity,
        MenuWindow,
        Visibility::Hidden,
        Sprite {
            color: Color::srgb(0.05, 0.1, 0.35),
            custom_size: Some(Vec2::new(152.0, 102.0)),
            ..default()
        },
        Transform::from_xyz(-150.0, -300.0, 1.8),
    ));
}

#[derive(Clone)]
pub struct PlayerDef {
    pub name: String,
    pub sprite: String,
    pub stats: BattlerStats,
}
#[derive(Resource)]
pub struct PlayerLibrary {
    pub players: HashMap<String, PlayerDef>,
}

impl PlayerLibrary {
    pub fn new() -> Self {
        PlayerLibrary { players: HashMap::new() }
    }

    pub fn add_player(&mut self, id: String, player: PlayerDef) {
        self.players.insert(id, player);
    }

    pub fn get_player(&self, id: String) -> Option<&PlayerDef> {
        self.players.get(&id)
    }
}

pub fn update_atb(mut commands: Commands, mut query: Query<(Entity, &mut BattlerStats), Without<ActionReady>>, time: Res<Time>) {
    for (entity, mut stats) in query.iter_mut() {
        stats.atb_timer += stats.speed as f32 * time.delta_secs() * 10.0;
        if stats.atb_timer > 100.0 {
            stats.atb_timer = 100.0;
            commands.entity(entity).insert(ActionReady);
        }
    }
}

pub fn enemy_turn(
    mut enemy_query: Query<(Entity, &mut BattlerStats), (With<Enemy>, With<ActionReady>, Without<Player>)>,
    mut player_query: Query<(&mut BattlerStats, &Transform), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    if let Some((enemy_entity, mut enemy_stats)) = enemy_query.iter_mut().next() {
        if let Some((mut player_stats, player_transform)) = player_query.iter_mut().next() {
            let damage = calculate_damage(enemy_stats.attack, player_stats.defense);
            player_stats.hp = player_stats.hp.saturating_sub(damage);
            damage_writer.write(DamageEvent {
                amount: damage,
                position: player_transform.translation,
            });
            commands.entity(enemy_entity).remove::<ActionReady>();
            enemy_stats.atb_timer = 0.0;
        }
    }
}

pub fn check_battle_end(mut commands: Commands, mut player_query: Query<&BattlerStats, With<Player>>, mut enemy_query: Query<&BattlerStats, With<Enemy>>, mut next_state: ResMut<NextState<GameState>>) {
    if let Ok(player_stats) = player_query.single_mut() {
        if player_stats.hp == 0 {
            println!("Player has been defeated!");
            next_state.set(GameState::GameOver);
            return;
        }
    }

    if let Ok(enemy_stats) = enemy_query.single_mut() {
        if enemy_stats.hp == 0 {
            println!("Enemy has been defeated!");
            next_state.set(GameState::Field);
            return;
        }
    }
}

pub fn cleanup_battle(mut commands: Commands, query: Query<Entity, With<BattleEntity>>, mut player_query: Query<&mut Visibility, With<PlayerControlled>>, mut party_state: ResMut<PartyState>, stats: Query<&BattlerStats, With<Player>>) {
    if let Some(current) = stats.iter().next() {
        let mut saved = current.clone();
        party_state.members[0] = saved;
    }
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(mut visibility) = player_query.single_mut() {
        *visibility = Visibility::Visible;
    }
}


#[derive(Component)]
pub struct PlayerAtbUi {
    pub left_edge_x: f32,
    pub full_width: f32,
}

pub fn update_atb_ui(mut query: Query<&BattlerStats, With<Player>>, mut ui_query: Query<(&mut Transform, &PlayerAtbUi)>) {
    for player_stats in query.iter() {
        let fill_ratio = player_stats.atb_timer / 100.0;
        for (mut transform, atb_ui) in ui_query.iter_mut() {
            transform.scale.x = fill_ratio;
            transform.translation.x = atb_ui.left_edge_x + (fill_ratio * atb_ui.full_width) / 2.0;
        }
    }
}

#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct MpText;

pub fn update_hp_text(mut player_query: Query<&BattlerStats, With<Player>>, mut text_query: Query<&mut Text2d, With<HpText> >) {
    for stats in player_query.iter_mut() {
        for mut text in text_query.iter_mut() {
            *text = Text2d::new(format!("HP {}/{}", stats.hp, stats.max_hp));
        }
    }
}

pub fn update_mp_text(mut player_query: Query<&BattlerStats, With<Player>>, mut text_query: Query<&mut Text2d, With<MpText> >) {
    for stats in player_query.iter_mut() {
        for mut text in text_query.iter_mut() {
            *text = Text2d::new(format!("MP {}/{}", stats.mp, stats.max_mp));
        }
    }
}

#[derive(Resource)]
pub struct BattleMenu {
    pub selected_index: usize,
}

#[derive(Component)]
pub struct MenuOption {
    pub index: usize,
}

#[derive(Component)]
pub struct MenuWindow;

pub fn cursor_movement(input: Res<ButtonInput<KeyCode>>, mut menu: ResMut<BattleMenu>) {
    if input.just_pressed(KeyCode::ArrowUp) {
        menu.selected_index = (menu.selected_index + 3 - 1) % 3;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_index = (menu.selected_index + 1) % 3;
    }
}

pub fn update_menu_cursor(mut commands: Commands, menu: Res<BattleMenu>, mut query: Query<(Entity, &mut Visibility, &MenuOption)>) {
    for (entity, mut visibility, option) in query.iter_mut() {
        if option.index == menu.selected_index {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn draw_menu(
    menu: Res<BattleMenu>,
    player_query: Query<(), (With<Player>, With<ActionReady>)>,
    mut option_query: Query<(&MenuOption, &mut Text2d, &mut TextColor, &mut Visibility), Without<MenuWindow>>,
    mut window_query: Query<&mut Visibility, With<MenuWindow>>,
) {
    let player_ready = player_query.iter().next().is_some();

    // Toggle the command window panel with the same readiness
    for mut visibility in window_query.iter_mut() {
        *visibility = if player_ready { Visibility::Visible } else { Visibility::Hidden };
    }

    for (option, mut text, mut color, mut visibility) in option_query.iter_mut() {
        if !player_ready {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Visible;
        }

        let label = match option.index {
            0 => "Attack",
            1 => "Magic",
            2 => "Item",
            _ => "",
        };

        if option.index == menu.selected_index {
            *color = TextColor(Color::srgb(1.0, 1.0, 0.0));
            *text = Text2d::new(format!("> {}", label));
        } else {
            *color = TextColor(Color::srgb(1.0, 1.0, 1.0));
            *text = Text2d::new(label.to_string());
        }
    }
}


pub fn confirm_selection(
    input: Res<ButtonInput<KeyCode>>,
    menu: Res<BattleMenu>,
    mut player_query: Query<(Entity, &mut BattlerStats), (With<Player>, With<ActionReady>)>,
    mut enemy_query: Query<(&mut BattlerStats, &Transform), (With<Enemy>, Without<Player>)>, // + &Transform
    mut damage_writer: MessageWriter<DamageEvent>, // new param
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Space) {
        if let Some((player_entity, mut player_stats)) = player_query.iter_mut().next() {
            let mut acted = false;

            match menu.selected_index {
                0 => { // Attack
                    if let Some((mut enemy_stats, enemy_transform)) = enemy_query.iter_mut().next() {
                        let damage = calculate_damage(player_stats.attack, enemy_stats.defense);
                        enemy_stats.hp = enemy_stats.hp.saturating_sub(damage);
                        damage_writer.write(DamageEvent {
                            amount: damage,
                            position: enemy_transform.translation,
                        });
                        println!("Player attacks! Enemy HP is now: {}", enemy_stats.hp);
                        acted = true;
                    }
                }
                1 => println!("Player uses Magic!"),
                2 => println!("Player uses Item!"),
                _ => {}
            }

            if acted {
                commands.entity(player_entity).remove::<ActionReady>();
                player_stats.atb_timer = 0.0;
            }
        }
    }
}

fn calculate_damage(attack: u32, defense: u32) -> u32 {
    let base = if attack > defense { attack - defense } else { 1 };
    let variance = rand::thread_rng().gen_range(0.85..=1.15);
    ((base as f32 * variance).round() as u32).max(1)
}

#[derive(Message)]
pub struct DamageEvent {
    pub amount: u32,
    pub position: Vec3,   // where to spawn the number (the target's location)
}

#[derive(Component)]
pub struct DamageNumber {
    pub timer: f32,
}

pub fn spawn_damage_numbers(
    mut damage_reader: MessageReader<DamageEvent>,
    mut commands: Commands,
) {
    for event in damage_reader.read() {
        commands.spawn((
            BattleEntity,
            DamageNumber { timer: 0.0 },
            Text2d::new(format!("{}", event.amount)),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            TextFont { font_size: 28.0, ..default() },
            // spawn slightly above the target, z above other UI so it's on top
            Transform::from_xyz(event.position.x, event.position.y + 40.0, 5.0),
        ));
    }
}

pub fn update_damage_numbers(
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor)>,
    mut commands: Commands,
) {
    const LIFETIME: f32 = 1.0;
    const FLOAT_SPEED: f32 = 60.0;

    for (entity, mut number, mut transform, mut color) in query.iter_mut() {
        number.timer += time.delta_secs();

        // float upward
        transform.translation.y += FLOAT_SPEED * time.delta_secs();

        // fade alpha from 1 -> 0 over the lifetime
        let alpha = (1.0 - number.timer / LIFETIME).clamp(0.0, 1.0);
        color.0.set_alpha(alpha);

        if number.timer >= LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct GameOverScreen;

pub fn setup_game_over(mut commands: Commands) {
    // Black full-screen overlay
    commands.spawn((
        GameOverScreen,
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(2000.0, 2000.0)), // cover whole screen
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0), // high z, on top of everything
    ));
    // "GAME OVER" text
    commands.spawn((
        GameOverScreen,
        Text2d::new("GAME OVER"),
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        TextFont { font_size: 64.0, ..default() },
        Transform::from_xyz(0.0, 0.0, 11.0),
    ));
    // prompt
    commands.spawn((
        GameOverScreen,
        Text2d::new("Press Space to continue"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextFont { font_size: 24.0, ..default() },
        Transform::from_xyz(0.0, -60.0, 11.0),
    ));
}

pub fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    mut party_state: ResMut<PartyState>,
    scene_lib: Res<SceneLibrary>,
    mut scene_change: MessageWriter<SceneChangeRequest>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        // Revive the party to full
        if let Some(member) = party_state.members.get_mut(0) {
            member.hp = member.max_hp;
            member.mp = member.max_mp;
            member.atb_timer = 0.0;
        }
        // Look up town's spawn from the scene library (single source of truth)
        let spawn_pos = scene_lib
            .get_scene("town")               // adjust to your actual getter name
            .map(|scene| scene.default_player_pos)
            .unwrap_or(Vec2::ZERO);          // fallback if town isn't found

        scene_change.write(SceneChangeRequest {
            scene_id: "town".to_string(),
            player_pos: spawn_pos,
        });
        next_state.set(GameState::Field);
    }
}

pub fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}


