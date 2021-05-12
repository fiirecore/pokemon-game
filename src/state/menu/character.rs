use game::{
	storage::{get_mut, player::PlayerSaves},
	macroquad::miniquad::date,
};

use super::{MenuState, MenuStateAction, MenuStates};

pub struct CharacterCreationState {
	action: Option<MenuStateAction>,
}

impl MenuState for CharacterCreationState {

	fn new() -> Self {
		Self {
			action: None,
		}
	}
	
	fn on_start(&mut self) {
		get_mut::<PlayerSaves>().expect("Could not get player saves!").select_new(&format!("Player{}", date::now() as u64 % 1000000));
		self.action = Some(MenuStateAction::Goto(MenuStates::MainMenu));
	}
	
	fn update(&mut self, _delta: f32) {}

	fn render(&self) {}

	fn quit(&mut self) {}

	fn action(&mut self) -> &mut Option<MenuStateAction> {
		&mut self.action
	}
}