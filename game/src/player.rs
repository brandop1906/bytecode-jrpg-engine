use bevy::prelude::*;
use crate::scripting::*;
use crate::walkmesh;
use crate::scene::TransitionOverlay;
use crate::battle_system::BattleStartOverlay;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct Movement
    {
    pub speed_x: f32,
    pub speed_y: f32,
    }

    pub fn move_player(mut query: Query<(&mut Transform, &Movement, &mut Sprite)>, window_query: Query<&WindowId>, transition_query: Query<&TransitionOverlay>, 
    input: Res<ButtonInput<KeyCode>>, time: Res<Time>, walk_area: Res<walkmesh::WalkableArea>, 
    player_images: Res<PlayerImages>, battle_start_query: Query<&BattleStartOverlay>) {
        if !window_query.is_empty() {
            return;
        }
        if !transition_query.is_empty() {
            return;
        }
        if !battle_start_query.is_empty() {
            return;
        }

        for (mut transform, movement, mut sprite) in query.iter_mut() {
            let mut direction = Vec2::ZERO;
            if input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::KeyA) {
                direction.x -= 1.0;
            }
            if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) {
                direction.x += 1.0;
            }
            if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW) {
                direction.y += 1.0;
            }
            if input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS) {
                direction.y -= 1.0;
            }

            if direction != Vec2::ZERO {
                direction = direction.normalize();
                transform.translation.x += direction.x * movement.speed_x  * time.delta_secs();
                transform.translation.y += direction.y * movement.speed_y  * time.delta_secs();
            }

            if direction.y < 0.0 {
                sprite.image = player_images.player_down.clone();
            } else if direction.y > 0.0 {
                sprite.image = player_images.player_up.clone();
            }

            if direction.x < 0.0 {
                sprite.image = player_images.player_left.clone();
            } else if direction.x > 0.0 {
                sprite.image = player_images.player_right.clone();
            }

            //println!("Player position: ({}, {})", transform.translation.x, transform.translation.y);

            transform.translation = walk_area.clamp_position(transform.translation);
        }
    }

pub fn player_interact(query_player_pos:  Query<&Transform, With<PlayerControlled>>, 
    query_npc_pos:  Query<(&FieldEntityId, &Transform), Without<PlayerControlled>>,  
    input: Res<ButtonInput<KeyCode>>, window_query: Query<&WindowId>, mut script_vm: ResMut<ScriptVM>,
    script_lib: Res<ScriptLibrary>) {
    if !window_query.is_empty() {
        return;
    }
    for player_pos in query_player_pos.iter() {
        for (npc_id, npc_pos) in query_npc_pos.iter() {
            let distance = (player_pos.translation - npc_pos.translation).length();
            //println!("Distance to NPC {}: {}", npc_id.id, distance);
            if distance < 50.0 && input.just_pressed(KeyCode::Space) {
                if let Some((bytecode, strings)) = script_lib.get(npc_id.id) {
                    script_vm.load_and_run(bytecode.clone(), strings.clone());
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct PlayerImages {
    pub player_left: Handle<Image>,
    pub player_right: Handle<Image>,
    pub player_up: Handle<Image>,
    pub player_down: Handle<Image>,
}