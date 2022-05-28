use core::ops::Deref;

use pokengine::{
    engine::egui,
    pokedex::{
        item::Item,
        moves::{owned::OwnedMove, Move},
        pokemon::{owned::OwnedPokemon, Pokemon},
    },
};

use super::moves::{ButtonState, MovePanel};

pub struct LevelUpMovePanel<M: Deref<Target = Move> + Clone> {
    state: LevelUpState,

    panel: MovePanel,

    moves: Vec<M>,
}

enum LevelUpState {
    NotAlive,
    Text,
    Moves,
}

impl<M: Deref<Target = Move> + Clone> LevelUpMovePanel<M> {
    pub fn new() -> Self {
        Self {
            state: LevelUpState::NotAlive,
            panel: MovePanel::default(),
            moves: Vec::new(),
        }
    }

    pub fn spawn(&mut self, moves: Vec<M>) {
        self.state = LevelUpState::Text;
    }

    // pub fn update<P: Deref<Target = Pokemon> + Clone, I: Deref<Target = Item> + Clone>(
    //     &mut self,
    //     app: &mut App,
    //     plugins: &mut Plugins,
    //     text: &mut BattleText,
    //     delta: f32,
    //     pokemon: &mut OwnedPokemon<P, M, I>,
    // ) -> Option<(usize, M)> {
    //     match self.state {
    //         LevelUpState::Text => match text.alive() {
    //             true => {
    //                 // text.update(app, plugins, delta);
    //                 if !text.alive() {
    //                     self.state = LevelUpState::Moves;
    //                 }
    //                 None
    //             }
    //             false => match self.moves.first() {
    //                 Some(move_ref) => {
    //                     let state = text
    //                         .state
    //                         .get_or_insert_with(|| MessageState::new(1, Default::default()));
    //                     state.pages.push(MessagePage {
    //                         lines: vec![
    //                             format!("{} is trying to", pokemon.name()),
    //                             format!("learn {}", move_ref.name),
    //                         ],
    //                         wait: None,
    //                         color: TextColor::BLACK,
    //                     });
    //                     self.update(app, plugins, text, delta, pokemon)
    //                 }
    //                 None => {
    //                     self.state = LevelUpState::NotAlive;
    //                     None
    //                 }
    //             },
    //         },
    //         LevelUpState::Moves => {
    //             self.move_panel.input(app, plugins);
    //             let a = pressed(app, plugins, Control::A);
    //             if pressed(app, plugins, Control::B) || a {
    //                 self.state = LevelUpState::Text;
    //                 let pokemon_move = self.moves.remove(0);
    //                 if a {
    //                     self.move_panel.names[self.move_panel.cursor] =
    //                         Some((pokemon_move.clone(), TextColor::BLACK));
    //                     pokemon
    //                         .moves
    //                         .add(Some(self.move_panel.cursor), pokemon_move.clone());
    //                     return Some((self.move_panel.cursor, pokemon_move));
    //                 }
    //             }
    //             None
    //         }
    //         LevelUpState::NotAlive => None,
    //     }
    // }

    pub fn ui<P: Deref<Target = Pokemon> + Clone, I: Deref<Target = Item> + Clone>(
        &mut self,
        egui: &egui::Context,
        pokemon: &mut OwnedPokemon<P, M, I>,
    ) {
        match self.state {
            LevelUpState::NotAlive => (),
            LevelUpState::Moves => {
                egui::Window::new("Level Up")
                    .title_bar(false)
                    .show(egui, |ui| {
                        if let Some(state) = self.panel.ui(ui, pokemon) {
                            if let ButtonState::Clicked(index) = state {
                                pokemon.moves[index] = OwnedMove::from(self.moves.remove(0));
                            }
                        }
                    });
            }
            LevelUpState::Text => (),
        }
    }

    pub fn alive(&self) -> bool {
        !matches!(self.state, LevelUpState::NotAlive)
    }
}
