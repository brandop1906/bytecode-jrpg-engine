use bevy::prelude::*;
use crate::walkmesh::*;
use crate::scripting::*;
use std::collections::HashMap;
use crate::player::PlayerControlled;

#[derive(Resource)]
pub struct SceneLibrary {
    scenes: HashMap<String, SceneDef>,
    current_scene: String,
}

impl SceneLibrary {
    pub fn new() -> Self {
        SceneLibrary { 
            scenes: HashMap::new(),
            current_scene: "".to_string(),
        }
    }
    pub fn add_scene(&mut self, scene_id: String, scene: SceneDef) {
        self.scenes.insert(scene_id, scene);
    }
    pub fn get_current_scene(&self) -> Option<&SceneDef> {
        self.scenes.get(&self.current_scene)
    }
    pub fn get_scene(&self, scene_id: &str) -> Option<&SceneDef> {
        self.scenes.get(scene_id)
    }
    pub fn set_current_scene(&mut self, scene_id: String) {
        if self.scenes.contains_key(&scene_id) {
            self.current_scene = scene_id;
        }
    }
}

#[derive(Message)]
pub struct SceneChangeRequest {
    pub scene_id: String,
    pub player_pos: Vec2,
}

#[derive(Clone)]
pub struct SceneDef {
    pub background: String,
    pub npcs: Vec<NpcDef>,
    pub walkmeshes: Vec<WalkableMesh>,
    pub scripts: ScriptLibrary,
    pub scene_change: Vec<ExitDef>,
    pub default_player_pos: Vec2,
    pub encounter_rate: f32,
    pub encounter_threshold: f32,
}

#[derive(Clone)]
pub struct NpcDef {
    pub sprite: String,
    pub name: String,
    pub position: Vec2,
    pub field_identity: u8,
    pub solid: bool,
    
}

#[derive(Clone)]
pub struct ExitDef {
    pub target_scene: String,
    pub trigger_area: Rect,
    pub player_pos: Vec2,
}

#[derive(Component)]
pub struct SceneEntity;

pub fn detection(position_query: Query<&Transform, With<PlayerControlled>>, 
    scenes: Res<SceneLibrary>, mut commands: Commands, transition_overlay_query: Query<Entity, With<TransitionOverlay>>) {
    let player_pos = position_query.single().unwrap().translation.truncate();
    if !transition_overlay_query.is_empty() {
        return;
    }
    if let Some(current_scene) = scenes.scenes.get(&scenes.current_scene) {
        for scene_change in &current_scene.scene_change {
            if scene_change.trigger_area.contains(player_pos) {
                commands.spawn((
                    TransitionOverlay {
                        phase: FadePhase::FadingOut,
                        timer: 0.0,
                        target_scene: scene_change.target_scene.clone(),
                        player_pos: scene_change.player_pos,
                    },
                    Sprite {
                        color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                        custom_size: Some(Vec2::new(1280.0, 720.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 3.0),
                ));
            }
        }
    }
}

pub fn transition(mut scene: MessageReader<SceneChangeRequest>, mut scenes: ResMut<SceneLibrary>, mut commands: Commands, 
    despawn: Query<Entity, With<SceneEntity>>, images: Res<AssetServer>, mut walkable_area: ResMut<WalkableArea>, 
    mut player_pos_query: Query<&mut Transform, With<PlayerControlled>>, mut script: ResMut<ScriptLibrary>,) {
        if let Some(event) = scene.read().next() {
            if let Some(new_scene) = scenes.scenes.get(&event.scene_id).cloned() {
                for entity in &despawn {
                    commands.entity(entity).despawn();
                }
                let background_handle = images.load(new_scene.background.clone());
                commands.spawn((
                    Sprite {
                        image: background_handle,
                        custom_size: Some(Vec2::new(1280.0, 720.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    SceneEntity {},
                ));
                for npc in &new_scene.npcs {
                    let entity = commands.spawn((Sprite {
                        image: images.load(npc.sprite.clone()),
                        ..default()
                        },
                        Transform::from_xyz(npc.position.x, npc.position.y, 1.0).with_scale(Vec3::splat(0.5)),
                        SceneEntity {},
                        FieldEntityId { id: npc.field_identity },
                        Name::new(npc.name.clone()))).id();
                    if npc.solid {
                        commands.entity(entity).insert(Solid);
                    }
                }
                walkable_area.set_walkable_mesh(new_scene.walkmeshes.clone());
                player_pos_query.single_mut().unwrap().translation = event.player_pos.extend(1.0);
                scenes.current_scene = event.scene_id.clone();
                *script = new_scene.scripts.clone();

            }
        }
    }

pub fn update_fade(mut transition_overlay_query: Query<(Entity, &mut TransitionOverlay, &mut Sprite)>, time: Res<Time>, 
    mut scene_change_event_writer: MessageWriter<SceneChangeRequest>, mut commands: Commands) {
    for (entity, mut overlay, mut sprite) in &mut transition_overlay_query {
        match overlay.phase {
            FadePhase::FadingOut => {
                overlay.timer += time.delta_secs();
                let alpha = (overlay.timer / 1.0).clamp(0.0, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha >= 1.0 {
                    scene_change_event_writer.write(SceneChangeRequest { scene_id: overlay.target_scene.clone(), player_pos: overlay.player_pos });
                    overlay.phase = FadePhase::FadingIn;
                    overlay.timer = 0.0;
                }
            }
            FadePhase::FadingIn => {
                overlay.timer += time.delta_secs();
                let alpha = (1.0 - (overlay.timer / 1.0)).clamp(0.0, 1.0);
                sprite.color = Color::srgba(0.0, 0.0, 0.0, alpha);
                if alpha <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn scene_startup(mut scene_change_event_writer: MessageWriter<SceneChangeRequest>) {
    scene_change_event_writer.write(SceneChangeRequest { scene_id: "town".to_string(), player_pos: Vec2::new(0.0, 0.0) });
}
    
#[derive(Component)]
pub struct TransitionOverlay {
    pub phase: FadePhase,
    pub timer: f32,
    pub target_scene: String,
    pub player_pos: Vec2, 
}

pub enum FadePhase {
    FadingOut,
    FadingIn,
}