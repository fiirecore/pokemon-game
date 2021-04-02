use super::Scene;
use super::scenes::SceneState;
use super::scenes::Scenes;
use super::scenes::title::TitleScene;
use super::scenes::main_menu::MainMenuScene;
use super::scenes::character::CharacterCreationScene;
use super::scenes::game::GameScene;

pub struct SceneManager {

	current: Scenes,

	title: TitleScene,
	main_menu: MainMenuScene,
	character: CharacterCreationScene,
	game: GameScene,

}

impl SceneManager {

	fn get(&self) -> &dyn Scene {
		match self.current {
		    Scenes::Title => &self.title,
		    Scenes::MainMenu => &self.main_menu,
			Scenes::CharacterCreation => &self.character,
		    Scenes::Game => &self.game,
		}
	}

	fn get_mut(&mut self) -> &mut dyn Scene {
		match self.current {
		    Scenes::Title => &mut self.title,
		    Scenes::MainMenu => &mut self.main_menu,
			Scenes::CharacterCreation => &mut self.character,
		    Scenes::Game => &mut self.game,
		}
	}

	pub async fn load(&mut self) {
		self.game.load().await;
	}
	
}

impl Scene for SceneManager {

	fn new() -> Self {
		Self {
			current: Scenes::default(),
			title: TitleScene::new(),
			main_menu: MainMenuScene::new(),
			character: CharacterCreationScene::new(),
			game: GameScene::new(),
		}
	}

    fn on_start(&mut self) {
		#[cfg(debug_assertions)] {
			let mut saves = macroquad::prelude::collections::storage::get_mut::<firecore_data::player::list::PlayerSaves>().expect("Could not get player saves");
			if saves.saves.is_empty() {
				self.current = Scenes::Title;
			} else {
				saves.select(0);
			}			
		}
		self.get_mut().on_start();
    }

    fn input(&mut self, delta: f32) {
        self.get_mut().input(delta);
    }

    fn update(&mut self, delta: f32) {
		match self.get().state() {
		    SceneState::Continue => {
				self.get_mut().update(delta);
			}
		    SceneState::Scene(scene) => {
				self.get_mut().quit();
				self.current = scene;
				self.get_mut().on_start();
			}
		}
	}

    fn render(&self) {
        self.get().render();
    }

    fn quit(&mut self) {
        self.get_mut().quit();
    }

    fn state(&self) -> SceneState {
        SceneState::Continue
    }
	
}
