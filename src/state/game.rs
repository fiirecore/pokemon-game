use crate::{
    command::{CommandProcessor, CommandResult},
    game::battle_glue::BattleId,
    saves::Player,
};
use battlelib::default_engine::{scripting::MoveScripts, EngineMoves};
use firecore_battle::pokedex::{item::Item, moves::Move, pokemon::Pokemon};
use firecore_battle_gui::{
    context::BattleGuiData,
    pokedex::{engine::audio, PokedexClientData},
};
use std::rc::Rc;
use worldlib::{events::*, serialized::SerializedWorld};

use crate::{
    engine::{
        error::ImageError,
        input::keyboard::{down as is_key_down, Key},
        Context,
    },
    game::battle_glue::BattleEntry,
    pokedex::gui::{bag::BagGui, party::PartyGui},
};

use crate::engine::log::warn;

use crate::{battle::BattleManager, world::manager::WorldManager};

use super::{MainStates, StateMessage};

pub enum GameStates {
    World,
    Battle,
}

impl Default for GameStates {
    fn default() -> Self {
        Self::World
    }
}

#[derive(Clone)]
pub enum GameActions {
    Battle(BattleEntry),
    CommandError(&'static str),
    Save,
    Exit,
}

pub(super) struct GameStateManager {
    state: GameStates,

    world: WorldManager,
    battle: BattleManager<&'static Pokemon, &'static Move, &'static Item>,

    pub save: Option<Player>,

    sender: Sender<StateMessage>,
    receiver: Receiver<GameActions>,
}

impl GameStateManager {
    pub fn new(
        ctx: &mut Context,
        dex: PokedexClientData,
        btl: BattleGuiData,
        wrld: SerializedWorld,
        battle: (EngineMoves, MoveScripts),
        sender: Sender<StateMessage>,
    ) -> Result<Self, ImageError> {
        let dex = Rc::new(dex);
        let party = Rc::new(PartyGui::new(&dex));
        let bag = Rc::new(BagGui::new(&dex));

        let (actions, receiver) = split();

        let world = WorldManager::new(ctx, dex.clone(), party.clone(), bag.clone(), actions, wrld)?;

        Ok(Self {
            state: GameStates::default(),

            world,
            battle: BattleManager::new(ctx, btl, dex, party, bag, battle),

            save: None,

            sender,
            receiver,
        })
    }

    pub fn seed(&mut self, seed: u64) {
        self.world.seed(seed);
        self.battle.seed(seed);
    }
}

impl GameStateManager {
    pub fn start(&mut self, _ctx: &mut Context) {
        if let Some(save) = self.save.as_mut() {
            match self.state {
                GameStates::World => self.world.start(&mut save.character),
                GameStates::Battle => (),
            }
        }
        // Ok(())
    }

    pub fn end(&mut self) {
        if let Some(save) = self.save.take() {
            self.sender.send(StateMessage::UpdateSave(save));
        }
        // Ok(())
    }

    pub fn update(&mut self, ctx: &mut Context, delta: f32, console: bool) {
        // Speed game up if spacebar is held down

        let delta = delta
            * if is_key_down(ctx, Key::Space) {
                4.0
            } else {
                1.0
            };

        if let Some(save) = self.save.as_mut() {
            for action in self.receiver.try_iter() {
                match action {
                    GameActions::Battle(entry) => match self.state {
                        GameStates::World => {
                            if self.battle.battle(
                                crate::dex::pokedex(),
                                crate::dex::movedex(),
                                crate::dex::itemdex(),
                                &mut save.character,
                                entry,
                            ) {
                                self.state = GameStates::Battle;
                            }
                        }
                        GameStates::Battle => warn!("Cannot start new battle, already in one!"),
                    },
                    GameActions::Save => self.sender.send(StateMessage::UpdateSave(save.clone())),
                    GameActions::Exit => {
                        audio::stop_music(ctx);
                        self.sender.send(StateMessage::Goto(MainStates::Menu));
                    }
                    GameActions::CommandError(error) => {
                        self.sender.send(StateMessage::CommandError(error))
                    }
                }
            }

            match self.state {
                GameStates::World => {
                    self.world.update(ctx, delta, &mut save.character, console);
                }
                GameStates::Battle => {
                    self.battle.update(
                        ctx,
                        crate::dex::pokedex(),
                        crate::dex::movedex(),
                        crate::dex::itemdex(),
                        delta,
                    );
                    if self.battle.finished {
                        save.character.input_frozen = false;
                        save.character.unfreeze();
                        if let Some(winner) = self.battle.winner() {
                            let winner = winner == &BattleId::Player;
                            let trainer = self.battle.update_data(winner, &mut save.character);
                            self.world.post_battle(&mut save.character, winner);
                        }
                        self.state = GameStates::World;
                        self.world.start(&mut save.character);
                    }
                }
            }
        } else if self.sender.is_empty() {
            self.sender.send(StateMessage::Goto(MainStates::Menu));
        }
        // Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        if let Some(save) = self.save.as_ref() {
            match self.state {
                GameStates::World => self.world.draw(ctx, &save.character),
                GameStates::Battle => {
                    if self.battle.world_active() {
                        self.world.draw(ctx, &save.character);
                    }
                    self.battle.draw(ctx);
                }
            }
        }

        // Ok(())
    }
}

impl CommandProcessor for GameStateManager {
    fn process(&mut self, command: CommandResult) {
        match self.state {
            GameStates::World => self.world.process(command),
            GameStates::Battle => (), //self.battle.process(result),
        }
    }
}
