use bevy::prelude::*;
use scripting::*;
use player::*;
use scene::*;
use state::*;
use battle_system::*;
mod scripting;
mod walkmesh;
mod player;
mod scene;
mod state;
mod battle_system;

fn main() {

    let reactor = SceneDef {
        background: "reactor.png".to_string(),
        npcs: vec![],
        scripts: ScriptLibrary::new(),
        walkmeshes: vec![walkmesh::WalkableMesh::new(-620.0, -110.0, 620.0, -25.0)],
        scene_change: vec![
            ExitDef {
                target_scene: "town".to_string(),
                trigger_area: Rect::new(-640.0, -300.0, -600.0, 300.0),
                player_pos: Vec2::new(580.0, -100.0),
            }
        ],
        encounter_rate: 1.0,
        encounter_threshold: 5.0,
        default_player_pos: Vec2::new(0.0, 0.0),
    };

    let town = SceneDef {
        background: "town.png".to_string(),
        npcs: vec![
            NpcDef {
                sprite: "NPC_down.png".to_string(),
                name: "Villager".to_string(),
                position: Vec2::new(550.0, -100.0),
                field_identity: 1,
                solid: true,
            }
        ],
        encounter_rate: 0.0,
        encounter_threshold: 100.0,
        walkmeshes: vec![walkmesh::WalkableMesh::new(-620.0, -150.0, 620.0, -75.0)],
        scripts: {
            let mut lib = ScriptLibrary::new();
            lib.add(1, vm::assembler::assemble_scene("WINDOW 100,50,300,100,0\nMESSAGE 0,0\nMESSAGE 0,1\nWINCLOSE 0\nRET"), 
            vec!["Welcome to Midgar!".to_string(), "The reactor is just ahead.".to_string()]);
            lib
        },
        default_player_pos: Vec2::new(0.0, 0.0),
        scene_change: vec![
            ExitDef {
                target_scene: "reactor".to_string(),
                trigger_area: Rect::new(600.0, -300.0, 640.0, 300.0),
                player_pos: Vec2::new(-580.0, -100.0),
            }
        ],
    };

    let mut scene_lib = SceneLibrary::new();
    scene_lib.add_scene("town".to_string(), town);
    scene_lib.add_scene("reactor".to_string(), reactor);

    let mako_guard = EnemyDef {
        name: "Mako Guard".to_string(),
        sprite: "mako_guard_side.png".to_string(),
        stats: BattlerStats {
            hp: 30,
            max_hp: 30,
            mp: 0,
            max_mp: 0,
            attack: 5,
            defense: 2,
            magic_attack: 0,
            magic_defense: 0,
            speed: 3,
            level: 1,
            atb_timer: 0.0,
        },
    };

    let mut enemy_lib = battle_system::EnemyLibrary::new();
    enemy_lib.add_enemy("mako_guard".to_string(), mako_guard);

    let player = PlayerDef {
        name: "Zane".to_string(),
        sprite: "Zane_down.png".to_string(),
        stats: BattlerStats {
            hp: 100,
            max_hp: 100,
            mp: 50,
            max_mp: 50,
            attack: 10,
            defense: 5,
            magic_attack: 5,
            magic_defense: 3,
            speed: 5,
            level: 1,
            atb_timer: 0.0,
        },
    };

    let mut player_lib = battle_system::PlayerLibrary::new();
    player_lib.add_player("Zane".to_string(), player);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ScriptVM::new(vec![], vec![]))
        .insert_resource(walkmesh::WalkableArea::new())
        .insert_resource(ScriptLibrary::new())
        .insert_resource(scene_lib)
        .insert_resource(enemy_lib)
        .insert_resource(player_lib)
        .insert_resource(battle_system::EncounterTracker { danger: 0.0 })
        .insert_resource(BattleMenu { selected_index: 0 })
        .insert_state(GameState::Field)
        .init_resource::<Messages<SceneChangeRequest>>()
        .add_systems(Startup, (spawn_entity, scene_startup))
        .add_systems(Update, (move_player, process_vm_commands, render_text, 
            close_dialog_on_input, player_interact, detection, transition, update_fade, encounter_check_system)
            .run_if(in_state(GameState::Field)))
        .add_systems(OnEnter(GameState::Battle), setup_battle)
        .add_systems(OnExit(GameState::Battle), cleanup_battle)
        .add_systems(Update, (update_atb_ui, update_atb, update_hp_text, update_mp_text, draw_menu, cursor_movement,enemy_turn, confirm_selection, check_battle_end).run_if(in_state(GameState::Battle)))
        .run();
}   

pub fn spawn_entity(mut commands: Commands,  asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.insert_resource(PlayerImages {
        player_down: asset_server.load(r"Zane_down.png"),
        player_up: asset_server.load(r"Zane_up.png"),
        player_left: asset_server.load(r"Zane_left.png"),
        player_right: asset_server.load(r"Zane_right.png"),
    });
    commands.spawn((
        Sprite::from_image(asset_server.load(r"Zane_down.png")),
        Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(0.5)),
        Name::new("Player"), 
        Movement { speed_x: 200.0, speed_y: 200.0 },  // Add the Movement component with desired speed values.
        PlayerControlled, // Add the PlayerControlled component to mark this entity as player-controlled.
        Solid, // Add the Solid component to mark this entity as solid. This can be used for collision detection or other purposes.
        FieldEntityId { id: 0 } // Add the FieldEntityId component with an ID of 0. This can be used to identify entities within a game field or similar structure.
    ));
}
