use std::ops::Deref;

use crate::pokedex::{
    item::Item,
    moves::Move,
    pokemon::{Pokemon},
    Dex,
};

use battlelib::{
    item::engine::ItemEngine,
    moves::engine::MoveEngine,
    prelude::{Battle, BattleAi},
};

use rand::{prelude::SmallRng, Rng};


use crate::game::battle_glue::{BattleId, BattleTrainerEntry};

mod manager;

pub use manager::*;

pub struct GameBattleWrapper<
    P: Deref<Target = Pokemon> + Clone,
    M: Deref<Target = Move> + Clone,
    I: Deref<Target = Item> + Clone,
> {
    battle: Battle<BattleId, P, M, I>,
    ai: BattleAi<SmallRng, BattleId, P, M, I>,
    trainer: Option<BattleTrainerEntry>,
}

impl<
        P: Deref<Target = Pokemon> + Clone,
        M: Deref<Target = Move> + Clone,
        I: Deref<Target = Item> + Clone,
    > GameBattleWrapper<P, M, I>
{

    pub fn update<'d>(
        &mut self,
        random: &mut (impl Rng + Clone + 'static),
        engine: &mut (impl MoveEngine + ItemEngine),
        delta: f32,
        pokedex: &'d dyn Dex<'d, Pokemon, P>,
        movedex: &'d dyn Dex<'d, Move, M>,
        itemdex: &'d dyn Dex<'d, Item, I>,
    ) {
        self.battle.update(random, engine, delta, movedex, itemdex);
        self.ai.update(pokedex, movedex, itemdex);
    }
}
