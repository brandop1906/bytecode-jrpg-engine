use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpellType {
    Fire,
    Ice,
}

#[derive(Clone)]
pub struct SpellDef {
    pub name: String,
    pub mp_cost: u32,
    pub power: u32,
    pub magic_type: SpellType,
}

#[derive(Resource)]
pub struct SpellLibrary {
    pub magics: HashMap<String, SpellDef>,
}

impl SpellLibrary {
    pub fn new() -> Self {
        SpellLibrary {
            magics: HashMap::new(),
        }
    }

    pub fn add_spell(&mut self, spell_id: String, spell: SpellDef) {
        self.magics.insert(spell_id, spell);
    }

    pub fn get_spell(&self, spell_id: &str) -> Option<&SpellDef> {
        self.magics.get(spell_id)
    }
}