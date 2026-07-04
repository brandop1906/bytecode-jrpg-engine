use bevy::prelude::*;

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
    pub exp: u32,
    
}