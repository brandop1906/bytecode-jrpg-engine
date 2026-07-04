use bevy::prelude::*;
use crate::stats::BattlerStats;

#[derive(Resource)]
pub struct PartyState {
    pub members: Vec<BattlerStats>,
}

