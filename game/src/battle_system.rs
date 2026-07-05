use bevy::ecs::entity;
use bevy::ecs::system::command;
use bevy::render::extract_component::ExtractComponent;
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
use crate::scene::FadePhase;
use crate::spells::*;

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
    time: Res<Time>,
    player_moving: Res<ButtonInput<KeyCode>>,
    current_scene: Res<SceneLibrary>,
    overlay_query: Query<&BattleStartOverlay>,   // guard: don't spawn twice
    mut commands: Commands,                        // to spawn the overlay
) {
    // If a battle-start fade is already underway, do nothing.
    if !overlay_query.is_empty() {
        return;
    }

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
                    println!("Encounter triggered! Fading to battle.");
                    commands.spawn((
                        BattleStartOverlay {
                            phase: FadePhase::FadingOut,
                            timer: 0.0,
                        },
                        Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.0), // start transparent
                            custom_size: Some(Vec2::new(2000.0, 2000.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 10.0),
                        // NOT SceneEntity — must survive the Field→Battle transition
                    ));
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
    pub exp_reward: u32,
}

#[derive(Component)]
pub struct ExpReward(pub u32);

#[derive(Resource, Default)]
pub struct PendingReward {
    pub exp: u32,
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
    enemy_lib: Res<EnemyLibrary>, player_lib: Res<PlayerLibrary>, party_state: Res<PartyState>, window_query: Query<&Window>, known: Res<KnownSpells>) {
    
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
        ExpReward(enemy.exp_reward),
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

    for (i, _spell_id) in known.spells.iter().enumerate() {
    commands.spawn((
        BattleEntity,
        Visibility::Hidden,
        SpellOption { index: i },
        Text2d::new(""),  // filled by the draw system
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-150.0, -180.0 - (i as f32 * 30.0), 2.0),
        TextFont { font_size: 24.0, ..default() },
    ));
}


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
    battle_end_query: Query<&BattleEndOverlay>,
) {
    if !battle_end_query.is_empty() {
        return;
    }

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

pub fn check_battle_end(
    mut commands: Commands,
    player_query: Query<&BattlerStats, With<Player>>,
    enemy_query: Query<(&BattlerStats, &ExpReward), With<Enemy>>,
    overlay_query: Query<&BattleEndOverlay>,   // to guard against spawning twice
    mut pending_reward: ResMut<PendingReward>,
) {
    // If a fade is already underway, do nothing.
    if !overlay_query.is_empty() {
        return;
    }

    let mut outcome = None;

    if let Ok(player_stats) = player_query.single() {
        if player_stats.hp == 0 {
            outcome = Some(BattleOutcome::GameOver);
        }
    }
    // only check enemy if player isn't already dead
    if outcome.is_none() {
        if let Ok((enemy_stats, exp_reward)) = enemy_query.single() {
            if enemy_stats.hp == 0 {
                pending_reward.exp = exp_reward.0;
                outcome = Some(BattleOutcome::Victory);
            }
        }
    }

    if let Some(outcome) = outcome {
        commands.spawn((
            BattleEndOverlay {
                phase: FadePhase::FadingOut,
                timer: 0.0,
                outcome,
            },
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0), // start transparent
                custom_size: Some(Vec2::new(2000.0, 2000.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 10.0),
            // NOT BattleEntity — must survive cleanup_battle
        ));
    }
}

pub fn update_battle_end_fade(
    mut overlay_query: Query<(&mut BattleEndOverlay, &mut Sprite)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    const BACKDROP_ALPHA: f32 = 0.9;  // fade-in settles here (dark backdrop, not clear)

    for (mut overlay, mut sprite) in &mut overlay_query {
        match overlay.phase {
            FadePhase::FadingOut => {
                overlay.timer += time.delta_secs();
                let alpha = (overlay.timer / 1.0).clamp(0.0, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha >= 1.0 {
                    // fully black — NOW change state
                    match overlay.outcome {
                        BattleOutcome::Victory => next_state.set(GameState::Victory),
                        BattleOutcome::GameOver => next_state.set(GameState::GameOver),
                    }
                    overlay.phase = FadePhase::FadingIn;
                    overlay.timer = 0.0;
                }
            }
            FadePhase::FadingIn => {
                let alpha = (1.0 - (overlay.timer / 1.0)).clamp(BACKDROP_ALPHA, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha > BACKDROP_ALPHA {
                    overlay.timer += time.delta_secs();   // only advance until settled
                }
            }
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

pub fn update_atb_ui(mut query: Query<&BattlerStats, With<Player>>, mut ui_query: Query<(&mut Transform, &PlayerAtbUi)>, battle_end_query: Query<&BattleEndOverlay>,) {
    if !battle_end_query.is_empty() {
        return;
    }
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
    pub layer: MenuLayer,
    pub selected_index: usize,
    pub selected_spell_index: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuLayer {
    Command,
    Spell,
}

#[derive(Resource)]
pub struct KnownSpells {
    pub spells: Vec<String>,
}

#[derive(Component)]
pub struct SpellOption {
    pub index: usize,
}

#[derive(Component)]
pub struct MenuOption {
    pub index: usize,
}

#[derive(Component)]
pub struct MenuWindow;

pub fn cursor_movement(input: Res<ButtonInput<KeyCode>>, mut menu: ResMut<BattleMenu>) {
    if menu.layer != MenuLayer::Command {
        return;
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        menu.selected_index = (menu.selected_index + 3 - 1) % 3;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_index = (menu.selected_index + 1) % 3;
    }
}

pub fn spell_cursor_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<BattleMenu>,
    known: Res<KnownSpells>,
) {
    if menu.layer != MenuLayer::Spell {
        return;
    }
    let count = known.spells.len();
    if count == 0 {
        return;
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        menu.selected_spell_index = (menu.selected_spell_index + count - 1) % count;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_spell_index = (menu.selected_spell_index + 1) % count;
    }
}

pub fn spell_cancel(input: Res<ButtonInput<KeyCode>>, mut menu: ResMut<BattleMenu>) {
    if menu.layer != MenuLayer::Spell {
        return;
    }
    if input.just_pressed(KeyCode::Escape) {
        menu.layer = MenuLayer::Command;
    }
}

pub fn spell_confirm(
    input: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<BattleMenu>,
    known: Res<KnownSpells>,
    spell_lib: Res<SpellLibrary>,
    battle_end_query: Query<&BattleEndOverlay>,
    mut player_query: Query<(Entity, &mut BattlerStats), (With<Player>, With<ActionReady>)>,
    mut enemy_query: Query<(&mut BattlerStats, &Transform), (With<Enemy>, Without<Player>)>,
    mut damage_writer: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    if !battle_end_query.is_empty() {
        return;
    }
    if menu.layer != MenuLayer::Spell {
        return;
    }

    if input.just_pressed(KeyCode::Space) {
        // Which spell is selected?
        let spell_id = match known.spells.get(menu.selected_spell_index) {
            Some(id) => id.clone(),
            None => return,
        };
        let spell = match spell_lib.get_spell(&spell_id) {
            Some(s) => s.clone(),
            None => return,
        };

        if let Some((player_entity, mut player_stats)) = player_query.iter_mut().next() {
            // Not enough MP? Refuse — don't consume the turn, stay in the menu.
            if player_stats.mp < spell.mp_cost {
                println!("Not enough MP for {}!", spell.name);
                return;
            }

            if let Some((mut enemy_stats, enemy_transform)) = enemy_query.iter_mut().next() {
                // Deduct MP
                player_stats.mp -= spell.mp_cost;
                // Magic damage: spell power + caster magic_attack - target magic_defense, floored at 1
                let raw = spell.power + player_stats.magic_attack;
                let damage = raw.saturating_sub(enemy_stats.magic_defense).max(1);
                enemy_stats.hp = enemy_stats.hp.saturating_sub(damage);
                damage_writer.write(DamageEvent {
                    amount: damage,
                    position: enemy_transform.translation,
                });

                // Consume the turn and return to the command layer
                commands.entity(player_entity).remove::<ActionReady>();
                player_stats.atb_timer = 0.0;
                menu.layer = MenuLayer::Command;
            }
        }
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

pub fn draw_spell_menu(
    menu: Res<BattleMenu>,
    known: Res<KnownSpells>,
    spell_lib: Res<SpellLibrary>,
    mut query: Query<(&SpellOption, &mut Text2d, &mut TextColor, &mut Visibility)>,
) {
    let show = menu.layer == MenuLayer::Spell;

    for (option, mut text, mut color, mut visibility) in query.iter_mut() {
        if !show {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;

        // Look up this spell's display name + cost
        let label = known.spells.get(option.index)
            .and_then(|id| spell_lib.get_spell(id))
            .map(|s| format!("{} ({} MP)", s.name, s.mp_cost))
            .unwrap_or_default();

        if option.index == menu.selected_spell_index {
            *color = TextColor(Color::srgb(1.0, 1.0, 0.0));
            *text = Text2d::new(format!("> {}", label));
        } else {
            *color = TextColor(Color::srgb(1.0, 1.0, 1.0));
            *text = Text2d::new(label);
        }
    }
}

pub fn confirm_selection(
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut menu: ResMut<BattleMenu>,
    battle_end_query: Query<&BattleEndOverlay>,
    mut player_query: Query<(Entity, &mut BattlerStats), (With<Player>, With<ActionReady>)>,
    mut enemy_query: Query<(&mut BattlerStats, &Transform), (With<Enemy>, Without<Player>)>,
    mut damage_writer: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    if !battle_end_query.is_empty() {
        return;
    }
    // Only handle command-layer input here
    if menu.layer != MenuLayer::Command {
        return;
    }

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
                        acted = true;
                    }
                }
                1 => { // Magic — enter the spell sub-menu (does NOT consume the turn)
                    menu.layer = MenuLayer::Spell;
                    menu.selected_spell_index = 0;
                    input.clear_just_pressed(KeyCode::Space);
                }
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


#[derive(Component)]
pub struct VictoryScreen;

pub fn setup_victory(
    mut commands: Commands,
    mut party_state: ResMut<PartyState>,
    pending_reward: Res<PendingReward>,   // was: enemy_query
) {
    let reward = pending_reward.exp;      // read the stashed value

    let mut levels_gained = 0;
    if let Some(member) = party_state.members.get_mut(0) {
        member.exp += reward;
        while member.exp >= member.level * 50 {
            member.exp -= member.level * 50;
            member.level += 1;
            member.max_hp += 20;
            member.hp = member.max_hp;
            member.attack += 3;
            member.defense += 2;
            member.max_mp += 5;
            member.mp = member.max_mp;
            levels_gained += 1;
        }
    }

    // --- results screen below ---

    commands.spawn((
        VictoryScreen,
        Sprite {
            color: Color::srgb(0.0, 0.0, 0.2),   // dark blue instead of black, for a "win" feel
            custom_size: Some(Vec2::new(2000.0, 2000.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
    // "VICTORY!" title
    commands.spawn((
        VictoryScreen,
        Text2d::new("VICTORY!"),
        TextColor(Color::srgb(1.0, 1.0, 0.3)),
        TextFont { font_size: 64.0, ..default() },
        Transform::from_xyz(0.0, 60.0, 11.0),
    ));
    // EXP line
    commands.spawn((
        VictoryScreen,
        Text2d::new(format!("Gained {} EXP", reward)),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextFont { font_size: 28.0, ..default() },
        Transform::from_xyz(0.0, 0.0, 11.0),
    ));

    if levels_gained > 0 {
        commands.spawn((
            VictoryScreen,
            Text2d::new(format!("Level up! Now level {}", party_state.members[0].level)),
            TextColor(Color::srgb(0.3, 1.0, 0.3)),
            TextFont { font_size: 28.0, ..default() },
            Transform::from_xyz(0.0, -40.0, 11.0),
        ));
    }
    // prompt
    commands.spawn((
        VictoryScreen,
        Text2d::new("Press Space to continue"),
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        TextFont { font_size: 24.0, ..default() },
        Transform::from_xyz(0.0, -100.0, 11.0),
    ));
}

pub fn victory_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Field);
    }
}

pub fn cleanup_victory(
    mut commands: Commands,
    query: Query<Entity, With<VictoryScreen>>,
    overlay_query: Query<Entity, With<BattleEndOverlay>>,   // add
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in overlay_query.iter() {   // add
        commands.entity(entity).despawn();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BattleOutcome {
    Victory,
    GameOver,
}

#[derive(Component)]
pub struct BattleEndOverlay {
    pub phase: FadePhase,
    pub timer: f32,
    pub outcome: BattleOutcome,
}

#[derive(Component)]
pub struct BattleStartOverlay {
    pub phase: FadePhase,
    pub timer: f32,
}

pub fn update_battle_start_fade(
    mut overlay_query: Query<(Entity, &mut BattleStartOverlay, &mut Sprite)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    const FADE_TIME: f32 = 0.5;  // snappy

    for (entity, mut overlay, mut sprite) in &mut overlay_query {
        match overlay.phase {
            FadePhase::FadingOut => {
                overlay.timer += time.delta_secs();
                let alpha = (overlay.timer / FADE_TIME).clamp(0.0, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha >= 1.0 {
                    next_state.set(GameState::Battle);   // switch under cover of black
                    overlay.phase = FadePhase::FadingIn;
                    overlay.timer = 0.0;
                }
            }
            FadePhase::FadingIn => {
                overlay.timer += time.delta_secs();
                let alpha = (1.0 - (overlay.timer / FADE_TIME)).clamp(0.0, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha <= 0.0 {
                    commands.entity(entity).despawn();   // fully clear — remove it
                }
            }
        }
    }
}