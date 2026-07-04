use bevy::ecs::entity;
use bevy::ecs::system::command;
use rand::Rng;
use bevy::prelude::*;
use crate::scene::*;
use crate::state::*;
use std::collections::HashMap;
use crate::player::*;

#[derive(Component, Clone)]
pub struct BattlerStats {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub attack: u32,
    pub defense: u32,
    pub magic_attack: u32,
    pub magic_defense: u32,
    pub speed: u32,
    pub level: u32,
    pub atb_timer: f32,
}

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
    enemy_lib: Res<EnemyLibrary>, player_lib: Res<PlayerLibrary>) {
    
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

    let player = player_lib.players.get("Zane").unwrap();
    commands.spawn((
        BattleEntity,
        Player,
        player.stats.clone(),
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

pub fn enemy_turn(mut commands: Commands, mut enemy_query: Query<(Entity, &mut BattlerStats), (With<Enemy>, With<ActionReady>, Without<Player>)>, mut player_query: Query<&mut BattlerStats, With<Player>>) {
    for (enemy_entity, mut enemy_stats) in enemy_query.iter_mut() {
        if let Ok(mut player_stats) = player_query.single_mut() {
            let damage = if enemy_stats.attack > player_stats.defense {
                enemy_stats.attack - player_stats.defense
            } else {
                1
            };
            player_stats.hp = player_stats.hp.saturating_sub(damage);
            println!("Enemy attacks! Player HP is now: {}", player_stats.hp);
        }
        commands.entity(enemy_entity).remove::<ActionReady>();
        enemy_stats.atb_timer = 0.0;
    }
}

pub fn player_turn(mut commands: Commands, mut player_query: Query<(Entity, &mut BattlerStats), (With<Player>, With<ActionReady>, Without<Enemy>)>, 
    mut enemy_query: Query<&mut BattlerStats, With<Enemy>>, input: Res<ButtonInput<KeyCode>>) {
    for (player_entity, mut player_stats) in player_query.iter_mut() {
        if input.just_pressed(KeyCode::Space) {
            if let Ok(mut enemy_stats) = enemy_query.single_mut() {
                let damage = if player_stats.attack > enemy_stats.defense {
                    player_stats.attack - enemy_stats.defense
                } else {
                    1
                };
                enemy_stats.hp = enemy_stats.hp.saturating_sub(damage);
                println!("Player attacks! Enemy HP is now: {}", enemy_stats.hp);
            }
            commands.entity(player_entity).remove::<ActionReady>();
            player_stats.atb_timer = 0.0;
        }
    }
}

pub fn check_battle_end(mut commands: Commands, mut player_query: Query<&BattlerStats, With<Player>>, mut enemy_query: Query<&BattlerStats, With<Enemy>>, mut next_state: ResMut<NextState<GameState>>) {
    if let Ok(player_stats) = player_query.single_mut() {
        if player_stats.hp == 0 {
            println!("Player has been defeated!");
            next_state.set(GameState::Field);
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

pub fn cleanup_battle(mut commands: Commands, query: Query<Entity, With<BattleEntity>>, mut player_query: Query<&mut Visibility, With<PlayerControlled>>) {
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