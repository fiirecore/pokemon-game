use serde::{Deserialize, Serialize};
use ahash::AHashSet as HashSet;

use firecore_world::npc::NPC;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MapData {

    pub battled: HashSet<String>,

}

impl MapData {

    pub fn battle(&mut self, npc: &NPC) {
        if !self.battled.contains(&npc.identifier.name) {
            // if npc.trainer.is_some() {
                crate::util::battle_data::trainer_battle(&npc);
                if let Some(trainer) = &npc.trainer {
                    self.battled.insert(npc.identifier.name.clone());
                    for name in &trainer.disable_others {
                        self.battled.insert(name.clone());
                    }
                }
            // }
        } else {
            macroquad::prelude::info!("Player has already battled {}", npc.identifier.name);
        }
    }

}