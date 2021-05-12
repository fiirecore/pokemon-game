use std::collections::VecDeque;

use crate::pokemon::BattleActionInstance;

#[derive(Debug)]
pub(crate) enum BattleManagerState {
	Transition,
	Opener,
	Introduction,
	Battle,
	Closer,
}

impl Default for BattleManagerState {
    fn default() -> Self {
        Self::Transition
    }
}


#[derive(Debug)]
pub enum BattleState {
	Selecting(usize),
	// Waiting (for opponent)
	Moving(MoveState),
}

impl Default for BattleState {
    fn default() -> Self {
        Self::Selecting(0)
    }
}

#[derive(Debug)]
pub enum MoveState {

	Start,
	SetupPokemon,
	Pokemon(MoveQueue), // queue of pokemon
	SetupPost,
	Post,
	End,

}

#[derive(Debug)]
pub struct MoveQueue {
	pub actions: VecDeque<BattleActionInstance>,
	pub current: Option<BattleActionInstance>,
}

impl MoveQueue {

	pub fn new(actions: VecDeque<BattleActionInstance>) -> Self {
		Self {
			actions,
			current: None,
		}
	}

	// pub fn interrupt(&mut self, instance: BattleActionInstance) {
	// 	self.actions.push_front(instance);
	// }

}