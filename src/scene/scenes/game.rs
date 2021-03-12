use macroquad::prelude::collections::storage;

use crate::battle::battle_manager::BattleManager;
use crate::gui::game::pokemon_party_gui::PokemonPartyGui;
use crate::io::data::player::PlayerData;
use crate::scene::Scene;
use crate::util::Completable;
use crate::util::Input;
use frc_data::data::PersistantData;
use crate::world::map::manager::WorldManager;
use crate::util::Update;
use crate::util::Render;
use firecore_util::Entity;

use super::SceneState;

pub struct GameScene {

	state: SceneState,
	
	world_manager: WorldManager,
	battle_manager: BattleManager,
	party_gui: PokemonPartyGui,

	battling: bool,
	swapped: bool,

}

impl GameScene {
	
	pub fn new() -> GameScene {
		GameScene {

			state: SceneState::Continue,

			world_manager: WorldManager::new(),
			battle_manager: BattleManager::new(),
			party_gui: PokemonPartyGui::new(),

			battling: false,
			swapped: false,
		}
	}

	pub fn data_dirty(&mut self, player_data: &mut PlayerData) {
		self.save_data(player_data);
		unsafe { crate::io::data::player::DIRTY = false; }
	}

    pub fn save_data(&mut self, player_data: &mut PlayerData) {
        self.world_manager.save_data(player_data);
		player_data.save();
    }
	
}

#[async_trait::async_trait(?Send)]
impl Scene for GameScene {

	async fn load(&mut self) {
		self.world_manager.load().await;
	}

	async fn on_start(&mut self) {
		self.world_manager.on_start().await;
	}
	
	fn update(&mut self, delta: f32) {
		if unsafe { crate::io::data::player::DIRTY } {
			self.data_dirty(&mut storage::get_mut::<PlayerData>().expect("Could not get Player Data"));
		}
		if unsafe { crate::gui::game::pokemon_party_gui::SPAWN } {
			self.party_gui.spawn();
			unsafe { crate::gui::game::pokemon_party_gui::SPAWN = false; }
		}

		if !self.battling {

			self.world_manager.update(delta);

			if crate::util::battle_data::BATTLE_DATA.lock().is_some() {
				self.battling = true;
				self.swapped = true;
				self.battle_manager.on_start(&storage::get::<PlayerData>().expect("Could not get Player Data"), crate::util::battle_data::BATTLE_DATA.lock().take().unwrap());
			}

		} else {
			if self.swapped {
				// context.battle_context.reset();
				self.swapped = false;				
			}
			self.battle_manager.update(delta);
			if self.battle_manager.is_finished() {
				self.battle_manager.current_battle.update_data(&mut storage::get_mut::<PlayerData>().expect("Could not get Player Data"));
				self.battling = false;
				self.swapped = true;
				self.world_manager.play_music();
			}
		}
		self.party_gui.update(delta);
	}
	
	fn render(&self) {
		if !self.battling {
			self.world_manager.render();
		} else {
			if self.battle_manager.world_active() {
				self.world_manager.render();
			}
			self.battle_manager.render();
		}
		self.party_gui.render();
	}
	
	fn input(&mut self, delta: f32) {
		if self.party_gui.is_alive() {
			self.party_gui.input(delta);
		} else if !self.battling {
			self.world_manager.input(delta);
			#[cfg(target_arch = "wasm32")]
			if frc_input::pressed(frc_input::Control::Start) {
				self.state = SceneState::Scene(super::Scenes::TitleScene);
			}
		} else {
			self.battle_manager.input(delta);
		}
	}

	fn quit(&mut self) {
        self.save_data(&mut storage::get_mut::<PlayerData>().expect("Could not get player data"));
	}
	
	fn state(&self) -> SceneState {
		self.state
	}
	
}