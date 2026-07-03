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