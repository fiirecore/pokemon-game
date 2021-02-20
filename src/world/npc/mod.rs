use crate::util::Direction;
use crate::util::Position;
use serde::{Deserialize, Serialize};
use self::trainer::Trainer;

use super::player::Player;

pub mod trainer;

#[derive(Deserialize, Serialize)]
pub struct NPC {

    pub identifier: NPCIdentifier,
    pub position: Position, // Home position
    #[serde(skip)]
    pub offset: Option<(isize, isize)>, // Offset from home position, see if changing the struct to something that uses variables better would help
    // pub movement: Option<MovementType>,
    // pub encounter_message: Vec<Vec<String>>,
    pub trainer: Option<Trainer>,

}

#[derive(Debug, Deserialize, Serialize)]
pub struct NPCIdentifier {

    pub name: String,
    pub npc_type: String,

}

// #[derive(Clone, Debug, Deserialize)]
// pub enum MovementType {

//     Still,

// }

impl NPC {

    pub fn walk_to(&mut self, x: isize, y: isize) {
        match self.position.direction {
            Direction::Up => self.offset = Some((x, y + 1)),
            Direction::Down => self.offset = Some((x, y - 1)),
            Direction::Left => self.offset = Some((x + 1, y)),
            Direction::Right => self.offset = Some((x - 1, y)),
        }
    }

    pub fn should_move(&self) -> bool {
        if let Some(offset) = self.offset {
            self.position.x != offset.0 || self.position.y != offset.1
        } else {
            false
        }
    }

    pub fn interact(&mut self, direction: Option<Direction>, player: &mut Player) {
        if let Some(direction) = direction {
            self.position.direction = direction.inverse();
        }
        if self.trainer.is_some() {
            macroquad::prelude::info!("Trainer battle with {}", &self.identifier.name);
            // if !trainer.battled {
            //     trainer.battled = true;
                self.walk_to(player.position.local.x, player.position.local.y);
                player.freeze();
            //}
        }
    }

    pub fn after_interact(&mut self) {
        if self.trainer.is_some() {
            crate::util::battle_data::trainer_battle(&self);
        }
    }

}