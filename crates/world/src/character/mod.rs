use crate::positions::{Destination, Direction, PixelOffset, Position};
use enum_map::Enum;
use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

use self::action::{ActionQueue, Actions};

pub mod action;
pub mod message;
pub mod npc;
pub mod player;
pub mod trainer;
// pub mod pathfind;

pub type CharacterFlag = tinystr::TinyStr8;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub name: String,

    pub position: Position,

    #[serde(default)]
    pub offset: PixelOffset,

    #[serde(default)]
    pub movement: MovementType,

    #[serde(default)]
    pub sprite: u8,

    #[serde(default)]
    pub locked: Counter,

    #[serde(default)]
    pub hidden: bool,

    #[serde(default)]
    pub noclip: bool,

    #[serde(default)]
    pub actions: Actions,

    #[serde(default)]
    pub flags: HashSet<CharacterFlag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, Deserialize, Serialize)]
pub enum MovementType {
    Walking,
    Running,
    Swimming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoMoveResult {
    Finished,
    Interact,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Counter(u8);

impl Character {

    // pub const INTERACT_LOCK: CharacterFlag = unsafe { CharacterFlag::new_unchecked(8386654075050290793) };

    pub fn new<S: Into<String>>(name: S, position: Position) -> Self {
        Self {
            name: name.into(),
            position,
            ..Default::default()
        }
    }

    pub fn moving(&self) -> bool {
        !self.actions.queue.is_empty() || !self.offset.is_zero()
    }

    pub fn update_sprite(&mut self) {
        self.sprite = if self.sprite == 0 { 2 } else { 0 }
    }

    pub fn on_try_move(&mut self, direction: Direction) {
        self.position.direction = direction;
    }

    pub fn stop_move(&mut self) {
        self.offset.reset();
    }

    pub fn locked(&self) -> bool {
        self.locked.active()
    }

    pub fn pathfind(&mut self, destination: Destination) {
        self.actions.extend(&self.position, destination);
        // match pathfind {
        //     true => {
        //         if let Some(path) = pathfind::pathfind(&self.position, destination, player, world) {
        //             self.pathing += path;
        //         }
        //     }
        //     false => ,
        // }
    }

    pub fn do_move(&mut self, delta: f32) -> Option<DoMoveResult> {
        if !self.locked() {
            match self.offset.is_zero() {
                true => {
                    if let Some(path) = (!self.actions.queue.is_empty()).then(|| self.actions.queue.remove(0)) {
                        match path {
                            ActionQueue::Move(direction) => {
                                self.position.direction = direction;
                                self.offset = direction.pixel_offset(self.speed() * 60.0 * delta);
                            }
                            ActionQueue::Look(direction) => {
                                self.position.direction = direction;
                            }
                            ActionQueue::Interact => return Some(DoMoveResult::Interact),
                        }
                    }
                    None
                }
                false => {
                    if self
                        .offset
                        .update(&self.position.direction, delta * self.speed() * 60.0)
                    {
                        self.position.coords += self.position.direction.tile_offset();
                        self.update_sprite();
                        Some(DoMoveResult::Finished)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn sees(&self, sight: u8, position: &Position) -> bool {
        let tracker = sight as i32;
        if position.elevation != self.position.elevation && self.position.elevation != 0 {
            return false;
        }
        match self.position.direction {
            Direction::Up => {
                if self.position.coords.x == position.coords.x
                    && self.position.coords.y > position.coords.y
                    && self.position.coords.y - tracker <= position.coords.y
                {
                    return true;
                }
            }
            Direction::Down => {
                if self.position.coords.x == position.coords.x
                    && self.position.coords.y < position.coords.y
                    && self.position.coords.y + tracker >= position.coords.y
                {
                    return true;
                }
            }
            Direction::Left => {
                if self.position.coords.y == position.coords.y
                    && self.position.coords.x > position.coords.x
                    && self.position.coords.x - tracker <= position.coords.x
                {
                    return true;
                }
            }
            Direction::Right => {
                if self.position.coords.y == position.coords.y
                    && self.position.coords.x < position.coords.x
                    && self.position.coords.x + tracker >= position.coords.x
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn queue_interact(&mut self, now: bool) {
        match now {
            true => self.actions.queue.insert(0, ActionQueue::Interact),
            false => self.actions.queue.push(ActionQueue::Interact),
        }
    }

    pub fn on_interact(&mut self) {
        self.locked.increment();
        self.stop_move();
        self.actions.clear();
    }

    pub fn end_interact(&mut self) {
        self.locked.decrement();
    }

    pub fn interact_from(&mut self, position: &Position) -> bool {
        self.can_interact_from(position)
            .map(|dir| {
                self.position.direction = dir;
                self.queue_interact(false);
                true
            })
            .unwrap_or_default()
    }

    pub fn can_interact_from(&self, position: &Position) -> Option<Direction> {
        if position.coords.x == self.position.coords.x {
            match position.direction {
                Direction::Up => {
                    if position.coords.y - 1 == self.position.coords.y {
                        Some(Direction::Down)
                    } else {
                        None
                    }
                }
                Direction::Down => {
                    if position.coords.y + 1 == self.position.coords.y {
                        Some(Direction::Up)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else if position.coords.y == self.position.coords.y {
            match position.direction {
                Direction::Right => {
                    if position.coords.x + 1 == self.position.coords.x {
                        Some(Direction::Left)
                    } else {
                        None
                    }
                }
                Direction::Left => {
                    if position.coords.x - 1 == self.position.coords.x {
                        Some(Direction::Right)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn speed(&self) -> f32 {
        match self.movement {
            MovementType::Walking => 1.0,
            MovementType::Running | MovementType::Swimming => 2.0,
        }
    }
}

impl Default for MovementType {
    fn default() -> Self {
        Self::Walking
    }
}

impl Counter {

    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    pub fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn active(&self) -> bool {
        self.0 != 0
    }

}