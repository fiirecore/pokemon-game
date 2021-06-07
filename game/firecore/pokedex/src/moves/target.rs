use serde::{Deserialize, Serialize};
use deps::str::TinyStr16;

pub type PlayerId = TinyStr16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MoveTarget {

	User,
	// Team,
	Opponent,
	Opponents,
	AllButUser,
	// All,
	// Singular(Team),
	// Team(Team),
	// TeamButSelf,
	// ReachAll(u8),
}

pub const fn move_target_player() -> MoveTarget {
	MoveTarget::User
}

pub const fn move_target_opponent() -> MoveTarget {
	MoveTarget::Opponent
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MoveTargetInstance {
	Opponent(PlayerId, usize),
	Team(usize),
	User,
}

impl MoveTargetInstance {

    pub fn user() -> Vec<Self> {
		vec![Self::User]
    }

    pub fn opponent(opponent: PlayerId, index: usize) -> Vec<Self> {
		vec![Self::Opponent(opponent, index)]
    }

    pub fn team(index: usize) -> Vec<Self> {
        vec![Self::Team(index)]
    }

    pub fn opponents(opponent: PlayerId, size: usize) -> Vec<Self> {
        (0..size).into_iter().map(|index| Self::Opponent(opponent, index)).collect()
    }

    pub fn all_but_user(user: usize, opponent: PlayerId, size: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(size * 2 - 1);
		for i in 0..size {
			if i != user {
				vec.push(Self::Team(i));
			}
		}
		(0..size).into_iter().for_each(|index| vec.push(Self::Opponent(opponent, index)));
        vec
    }

}