use bevy::prelude::*;
use bevy::text::TextBounds;
use std::collections::HashMap;

#[derive(Resource)]
pub struct ScriptVM {
    vm: vm::interpreter::VM,
    state: vm::interpreter::ExecutionResult,
    strings : Vec<String>,
    pending: Vec<PendingCommand>,
}

impl ScriptVM {
    pub fn new(bytecode: Vec<u8>, strings: Vec<String>) -> Self {
        ScriptVM {
            vm: vm::interpreter::VM::new(bytecode),
            state: vm::interpreter::ExecutionResult::Running,
            strings : strings,
            pending: Vec::new(),
        }
    }

    pub fn load_and_run(&mut self, bytecode: Vec<u8>, strings: Vec<String>) {
    self.vm.load_bytecode(bytecode);
    self.strings = strings;
    self.state = self.vm.run();
    }
}

pub struct PendingCommand {
    command: vm::commands::Command,
    retry_count: u8,
}

#[derive(Component)]
pub struct Solid; // Add a Solid component to represent solid entities.

#[derive(Component)]
pub struct FieldEntityId {
    pub id: u8,
}

#[derive(Component)]
pub struct WindowId(pub u8);

#[derive(Component)]
pub struct TextContent(pub String);

#[derive(Resource, Clone)]
pub struct ScriptLibrary {
    scripts: HashMap<u8, (Vec<u8>, Vec<String>)>,
}

impl ScriptLibrary {
    pub fn new() -> Self {
        ScriptLibrary { scripts: HashMap::new() }
    }

    pub fn add(&mut self, script_id: u8, bytecode: Vec<u8>, strings: Vec<String>) {
        self.scripts.insert(script_id, (bytecode, strings));
    }

    pub fn get(&self, script_id: u8) -> Option<&(Vec<u8>, Vec<String>)>  {
        self.scripts.get(&script_id)
    }
}   




pub fn process_vm_commands(mut script: ResMut<ScriptVM>, query_set_solid: Query<(Entity, &FieldEntityId)>, 
    query_window_close: Query<(Entity, &WindowId)>, mut commands: Commands) 
    {
    let commands_to_process: Vec<_> = script.vm.commands.drain(..).collect();
    for command in commands_to_process {
        match command {
            vm::commands::Command::SetSolid { character_id, enabled } => {
                let mut found = false;
                for (entity, field_entity_id) in query_set_solid.iter() {
                    if field_entity_id.id == character_id {
                        found = true;
                        if enabled {
                            commands.entity(entity).insert(Solid);
                        } else {
                            commands.entity(entity).remove::<Solid>();
                        }
                    }
                }
                if !found {
                    script.pending.push(PendingCommand { command: vm::commands::Command::SetSolid { character_id, enabled }, retry_count: 1})
                }
            }
            vm::commands::Command::WindowOpen { x, y, width, height, window_id } => {
                commands.spawn((
                    Transform::from_xyz(x as f32, y as f32, 2.0),
                    Sprite {
                        color: Color::srgba(0.0, 0.1, 0.3, 0.9),
                        custom_size: Some(Vec2::new(width as f32, height as f32)),
                        ..default()
                    },
                    WindowId(window_id),
                ));
            }

            vm::commands::Command::Message { window_id, message_id } => {
                let mut found = false;
                let text = script.strings[message_id as usize].clone();

                for (entity, id) in query_window_close.iter() {
                    
                    if id.0 == window_id {
                        commands.entity(entity).remove::<TextContent>();
                        commands.entity(entity).insert(TextContent(text.clone()));
                        found = true;
                    }
                }
                if !found {
                    script.pending.push(PendingCommand { command: vm::commands::Command::Message { window_id, message_id }, retry_count: 1 });
                }
            }

            vm::commands::Command::WindowClose { window_id } => {
                let mut found = false;
                for (entity, window_id_component) in query_window_close.iter() {
                    if window_id_component.0 == window_id {
                        found = true;
                        commands.entity(entity).despawn()
                    }
                }
                if !found {
                    script.pending.push(PendingCommand { command: vm::commands::Command::WindowClose { window_id }, retry_count: 1 });
                }
            }
            _ => {}
        }
    }
    let pendings_to_process: Vec<_> = script.pending.drain(..).collect();
    for pending in pendings_to_process {
        match pending.command  {
            vm::commands::Command::Message { window_id, message_id } => {
                let mut found = false;
                let text = script.strings[message_id as usize].clone();

                for (entity, id) in query_window_close.iter() {
                    
                    if id.0 == window_id {
                        commands.entity(entity).remove::<TextContent>();
                        commands.entity(entity).insert(TextContent(text.clone()));
                        found = true;
                    }
                }
                if !found {
                    if pending.retry_count >= 60 {
                        println!("Failed to find window id {} after 60 retries", window_id);
                    } else {  // retry again after a frame. 
                        println!("Retrying to find window id {} after 1 frame", window_id);
                        script.pending.push(PendingCommand { command: vm::commands::Command::Message { window_id, message_id }, retry_count: pending.retry_count + 1 });
                    }
                }
            }

            vm::commands::Command::SetSolid { character_id, enabled} => {
                let mut found = false;
                for (entity, field_entity_id) in query_set_solid.iter() {
                    if field_entity_id.id == character_id {
                        found = true;
                        if enabled {
                            commands.entity(entity).insert(Solid);
                        } else {
                            commands.entity(entity).remove::<Solid>();
                        }
                    }
                }
                if !found {
                    if pending.retry_count >= 60 {
                        println!("Failed to find character id {} after 60 retries", character_id);
                    } else {  // retry again after a frame. 
                        println!("Retrying to find character id {} after 1 frame", character_id);
                        script.pending.push(PendingCommand { command: vm::commands::Command::SetSolid { character_id, enabled }, retry_count: pending.retry_count + 1 });
                    }
                }
            }
            vm::commands::Command::WindowClose { window_id } => {
                let mut found = false;
                for (entity, window_id_component) in query_window_close.iter() {
                    if window_id_component.0 == window_id {
                        found = true;
                        commands.entity(entity).despawn()
                    }
                }
                if !found {
                    if pending.retry_count >= 60 {
                        println!("Failed to find window id {} after 60 retries", window_id);
                    } else {  // retry again after a frame. 
                        println!("Retrying to find window id {} after 1 frame", window_id);
                        script.pending.push(PendingCommand { command: vm::commands::Command::WindowClose { window_id }, retry_count: pending.retry_count + 1 });
                    }
                }
            }
            _ => {}
        } 
    }
}

pub fn render_text(query: Query<(Entity, &TextContent, &Sprite), Added<TextContent>>, mut commands: Commands) {
    for (entity, text_content, sprite) in query.iter() {
        commands.entity(entity).with_children(|parent| {
            let size = sprite.custom_size.unwrap();
            parent.spawn((
                Text2d::new(&text_content.0),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },  
                TextBounds::from(sprite.custom_size.unwrap()),
                Transform::from_xyz(-size.x / 2.0 + 160.0, size.y / 2.0 - 60.0, 1.0),
            ));
        });
    }
}

pub fn close_dialog_on_input(
    inputs: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<TextContent>>,
    mut commands: Commands,
    mut script: ResMut<ScriptVM>,
){
    if inputs.just_pressed(KeyCode::Space) {
        let mut had_dialog = false;
        for entity in query.iter() {
            commands.entity(entity).despawn_children();
            commands.entity(entity).remove::<TextContent>();
            had_dialog = true;
        }
        if had_dialog && script.state == vm::interpreter::ExecutionResult::Paused {
            script.state = script.vm.run();
        }
    }
}

