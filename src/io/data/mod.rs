use serde::{Deserialize, Serialize};

use crate::util::input::Control;

pub mod configuration;
pub mod player;
pub mod text;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, enum_iterator::IntoEnumIterator)]
pub enum Direction { // move to util
	
	Up,
	Down,
	Left,
	Right,
	
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {

	pub map_id: String,
	pub map_index: u16,
	pub position: Position,

}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
pub struct Position {

	pub x: isize,
    pub y: isize,
	pub direction: Direction,
    #[serde(skip)]
    pub x_offset: f32,
    #[serde(skip)]
	pub y_offset: f32,

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatSet {

	pub hp: u8,
	pub atk: u8,
	pub def: u8,
	pub sp_atk: u8,
	pub sp_def: u8,
	pub speed: u8,

}

impl Direction {

	pub fn inverse(&self) -> Direction {
		match *self {
		    Direction::Up => Direction::Down,
		    Direction::Down => Direction::Up,
		    Direction::Left => Direction::Right,
		    Direction::Right => Direction::Left,
		}
	}

	pub fn value(&self) -> u8 {
		match *self {
			Direction::Up => 0,
			Direction::Down => 1,
			Direction::Left => 2,
			Direction::Right => 3,
		}
	}

	// Input

	pub fn keybind(&self) -> Control {
		match *self {
		    Direction::Up => Control::Up,
		    Direction::Down => Control::Down,
		    Direction::Left => Control::Left,
		    Direction::Right => Control::Right,
		}
	}

	pub fn offset(&self) -> (f32, f32) {
		match *self {
		    Direction::Up => (0.0, -1.0),
		    Direction::Down => (0.0, 1.0),
		    Direction::Left => (-1.0, 0.0),
		    Direction::Right => (1.0, 0.0),
		}
	}

}

impl Default for Direction {
    fn default() -> Self {
        Direction::Down
    }
}

impl Position {

	pub fn subtract(&self, x: isize, y: isize) -> Position {
		Position {
			x: self.x - x,
			y: self.y - y,
			..*self
		}
    }

}

impl StatSet {

	pub fn iv_random() -> Self {

		use macroquad::prelude::rand::gen_range;

		Self {
			hp: gen_range(0, 32),
			atk: gen_range(0, 32),
			def: gen_range(0, 32),
			sp_atk: gen_range(0, 32),
			sp_def: gen_range(0, 32),
			speed: gen_range(0, 32),
		}

	}

	pub fn uniform(stat: u8) -> Self {

		Self {
			hp: stat,
			atk: stat,
			def: stat,
			sp_atk: stat,
			sp_def: stat,
			speed: stat,
		}
		
	}

}