use crate::battle::Battle;
use crate::gui::dynamic_text::DynamicText;
use firecore_pokedex::pokemon::battle::BattlePokemon;
use firecore_util::text::{Message, TextColor};
use macroquad::prelude::Vec2;
use super::pokemon::PokemonGui;

pub struct BattleText {

    pub text: DynamicText,
    pub faint_index: Option<usize>,

}

impl BattleText {

    pub fn new() -> Self {
        Self {
            text: DynamicText::with_size(5, Vec2::new(11.0, 11.0), Vec2::new(0.0, 113.0)),
            faint_index: None,
        }
    }

    pub fn reset_text(&mut self) {
        if let Some(messages) = self.text.messages.as_mut() {
            messages.clear();
        }
    }

    pub fn add_moves(&mut self, pokemon: &String, move_name: &String) {
        if let Some(messages) = self.text.messages.as_mut() {
            messages.push(
                Message::new(
                    vec![pokemon.clone() + " used " + move_name + "!"],
                    TextColor::White,
                    Some(0.5),
                )
            )
        }
        
    }

    pub fn add_faint_text(&mut self, name: &String) {
        if let Some(messages) = self.text.messages.as_mut() {
            self.faint_index = Some(messages.len());
            messages.push(
                Message::new(
                    vec![name.clone() + " fainted!"],
                    TextColor::White,
                    Some(1.0), 
                )            
            );
        }
    }

    pub fn player_level_up(&mut self, name: &String, exp: u32, level: Option<u8>) {
        if let Some(messages) = self.text.messages.as_mut() {
            messages.push(
                Message::new(
                    vec![
                        format!("{} gained {} exp!", name, exp)
                    ],
                    TextColor::White,
                    Some(0.5),
                )
            );
            if let Some(level) = level {
                messages.push(
                    Message::new(
                        vec![
                            format!("{} leveled up to level {}!", name, level)
                        ],
                        TextColor::White,
                        Some(0.5),
                    )
                );                
            }
        }        
    }

    pub fn on_move(&mut self, other_pokemon: &BattlePokemon, gui: &mut impl PokemonGui) {

        gui.update_gui(other_pokemon, false);

        if other_pokemon.faint() {
            let next = self.text.current_message() + 1;
            if let Some(messages) = self.text.messages.as_mut() {
                if next < messages.len() {
                    messages.remove(next);                
                }
            }            
            self.add_faint_text(other_pokemon.name());
        }

    }

    pub fn perform_player(&self, battle: &Battle) -> bool {

        self.text.can_continue && battle.player.next_move.queued && !battle.player.active().faint() && self.text.current_message() == if battle.player.active().base.speed >= battle.opponent.active().base.speed {
            0
        } else {
            1
        }
    }

    pub fn perform_opponent(&self, battle: &Battle) -> bool {
        self.text.can_continue && battle.opponent.next_move.queued && !battle.opponent.active().faint() && self.text.current_message() == if battle.player.active().base.speed >= battle.opponent.active().base.speed {
            1
        } else {
            0
        }
    }

}