use crate::{
    actions::WorldActions,
    character::{
        npc::{trainer::TrainerDisable, NpcInteract},
        player::PlayerCharacter,
        Movement,
    },
    events::Sender,
    map::{
        movement::{can_move, can_swim, can_walk},
        MovementId, WarpDestination, WorldMap,
    },
    positions::{Coordinate, Direction, Location, Position},
};
use std::{collections::HashMap, ops::Deref};

use firecore_pokedex::{
    item::Item,
    moves::{Move, MoveId},
    pokemon::{owned::OwnedPokemon, Pokemon},
};
use rand::{Rng, SeedableRng};

use super::{battle::BattleEntry, chunk::Connection, movement::MovementResult};

use self::random::WorldRandoms;

mod random;
pub mod state;

mod tile;

pub type Maps = HashMap<Location, WorldMap>;

pub struct WorldMapManager<R: Rng + SeedableRng + Clone> {
    pub maps: Maps,
    pub tiles: tile::PaletteTileData,
    pub default: (Location, Position),
    sender: Sender<WorldActions>,
    randoms: WorldRandoms<R>, //
                              // add tile data fields if needed
}

impl<R: Rng + SeedableRng + Clone> WorldMapManager<R> {
    pub fn new(maps: Maps, default: (Location, Position), sender: Sender<WorldActions>) -> Self {
        Self {
            maps,
            tiles: Default::default(),
            default,
            sender,
            randoms: Default::default(),
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.randoms.seed(seed);
    }

    pub fn contains(&self, location: &Location) -> bool {
        self.maps.contains_key(location)
    }

    pub fn get(&self, location: &Location) -> Option<&WorldMap> {
        self.maps.get(location)
    }

    pub fn on_warp(&mut self, player: &mut PlayerCharacter) {
        self.on_map_change(player);
        self.on_tile(player);
    }

    pub fn on_map_change(&self, player: &PlayerCharacter) {
        if let Some(map) = self.maps.get(&player.location) {
            self.on_change(map);
        }
    }

    pub fn on_change(&self, map: &WorldMap) {
        self.sender.send(WorldActions::PlayMusic(map.music));
    }

    pub fn try_interact(&mut self, player: &mut PlayerCharacter) {
        if player.world.npc.active.is_none() {
            if let Some(map) = self.maps.get_mut(&player.location) {
                let pos = if map
                    .tile(player.position.coords)
                    .map(|tile| self.tiles.forwarding.contains(&tile))
                    .unwrap_or_default()
                {
                    player.position.in_direction(player.position.direction)
                } else {
                    player.position
                };
                for (id, npc) in map.npcs.iter_mut() {
                    if (npc.interact.is_some() || npc.trainer.is_some()) && npc.interact_from(&pos)
                    {
                        player.world.npc.active = Some(*id);
                    }
                }
            }
        }
    }

    pub fn post_battle<
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    >(
        &mut self,
        player: &mut PlayerCharacter,
        party: &mut [OwnedPokemon<P, M, I>],
        winner: bool,
        trainer: bool,
    ) {
        if let Some(entry) = player.world.battle.battling.take() {
            if winner {
                if trainer {
                    if let Some(trainer) = self
                        .maps
                        .get(&entry.location)
                        .map(|map| map.npcs.get(&entry.id).map(|npc| npc.trainer.as_ref()))
                        .flatten()
                        .flatten()
                    {
                        match &trainer.disable {
                            TrainerDisable::DisableSelf => {
                                player.world.battle.insert(&entry.location, entry.id);
                            }
                            TrainerDisable::Many(others) => {
                                player.world.battle.insert(&entry.location, entry.id);
                                player
                                    .world
                                    .battle
                                    .battled
                                    .get_mut(&entry.location)
                                    .unwrap()
                                    .extend(others);
                            }
                            TrainerDisable::None => (),
                        }
                    }
                }
            } else {
                let loc = player.world.heal.unwrap_or(self.default);
                player.location = loc.0;
                player.position = loc.1;
                player.location = player.location;
                party.iter_mut().for_each(|o| o.heal(None, None));
            }
        }
    }

    pub fn move_npcs(&mut self, player: &mut PlayerCharacter, delta: f32) {
        if let Some(map) = self.maps.get_mut(&player.location) {
            // Move Npcs

            for npc in player
                .world
                .scripts
                .npcs
                .values_mut()
                .filter(|(location, ..)| map.contains(location))
                .map(|(.., npc)| npc)
            {
                npc.character.do_move(delta);
            }

            for npc in map.npcs.values_mut() {
                npc.character.do_move(delta);
            }

            use crate::{character::npc::NpcMovement, positions::Destination};

            match player.world.npc.timer > 0.0 {
                false => {
                    player.world.npc.timer += 1.0;

                    const NPC_MOVE_CHANCE: f64 = 1.0 / 12.0;

                    for (index, npc) in map.npcs.iter_mut() {
                        if !npc.character.moving() && self.randoms.npc.gen_bool(NPC_MOVE_CHANCE) {
                            match npc.movement {
                                NpcMovement::Still => (),
                                NpcMovement::LookAround => {
                                    npc.character.position.direction =
                                        Direction::DIRECTIONS[self.randoms.npc.gen_range(0..4)];
                                    player.find_battle(&map.id, index, npc);
                                }
                                NpcMovement::WalkUpAndDown(steps) => {
                                    let origin =
                                        npc.origin.get_or_insert(npc.character.position.coords);
                                    let direction = if npc.character.position.coords.y
                                        <= origin.y - steps as i32
                                    {
                                        Direction::Down
                                    } else if npc.character.position.coords.y
                                        >= origin.y + steps as i32
                                    {
                                        Direction::Up
                                    } else if self.randoms.npc.gen_bool(0.5) {
                                        Direction::Down
                                    } else {
                                        Direction::Up
                                    };
                                    let coords =
                                        npc.character.position.coords.in_direction(direction);
                                    if can_move(
                                        npc.character.movement,
                                        map.movements[npc.character.position.coords.x as usize
                                            + npc.character.position.coords.y as usize
                                                * map.width as usize],
                                    ) {
                                        npc.character.position.direction = direction;
                                        if !player.find_battle(&map.id, index, npc)
                                            && coords.y != player.position.coords.y
                                        {
                                            npc.character.pathing.extend(
                                                &npc.character.position,
                                                Destination::to(&npc.character.position, coords),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                true => player.world.npc.timer -= delta,
            }
        }
    }

    pub fn update_interactions(
        &mut self,
        player: &mut PlayerCharacter,
        active: bool,
        finished: bool,
    ) {
        if let Some(id) = player.world.npc.active.as_ref() {
            let npc = self.maps.get_mut(&player.location).map(|map| map.npcs.get_mut(&id)).flatten();
            let npc = if let Some(npc) = npc {
                Some(npc)
            } else {
                player.world.scripts.npcs.get_mut(id).filter(|(location, ..)| &player.location == location).map(|(.., npc)| npc)
            };
            if let Some(npc) = npc {
                match active {
                    true => if finished {
                        if let Some(battle) =
                            BattleEntry::trainer(&mut player.world.battle, &player.location, id, npc)
                        {
                            self.sender.send(WorldActions::Battle(battle));
                        }
                        if player.frozen() {
                            player.unfreeze();
                        }
                        player.world.npc.active = None;
                    },
                    false => if !npc.character.moving() {

                        if !player.world.battle.battled(&player.location, id) {
                            if let Some(trainer) = npc.trainer.as_ref() {
                                if trainer.battle_on_interact {
                                    // Spawn text window
                                    self.sender.send(WorldActions::Message(
                                        Some((*id, true)),
                                        trainer.encounter_message.clone(),
                                        false,
                                    ));
                                    return player.position.direction = npc.character.position.direction.inverse();
                                }
                            }
                        }

                        match &npc.interact {
                            NpcInteract::Message(pages) => {
                                self.sender.send(WorldActions::Message(
                                    Some((*id, false)),
                                    pages.clone(),
                                    false,
                                ));
                                return player.position.direction = npc.character.position.direction.inverse();
                            }
                            NpcInteract::Script(_) => todo!(),
                            NpcInteract::Nothing => (),
                        }
                    }
                }
            }
        }
    }

    pub fn on_tile(
        &mut self,
        player: &mut PlayerCharacter,
        // party: &[OwnedPokemon<P, M, I>],
    ) {
        self.sender.send(WorldActions::OnTile);

        if let Some(map) = self.maps.get_mut(&player.location) {
            if let Some(current) = map.tile(player.position.coords) {
                if player.world.wild.encounters {
                    if let Some(wild) = &map.wild {
                        if wild.should_encounter(&mut self.randoms.wild) {
                            if let Some(tiles) = wild.tiles.as_ref() {
                                if tiles.iter().any(|tile| tile == &current) {
                                    self.sender.send(WorldActions::Battle(BattleEntry::wild(
                                        &mut self.randoms.wild,
                                        wild,
                                    )));
                                }
                            } else {
                                self.sender.send(WorldActions::Battle(BattleEntry::wild(
                                    &mut self.randoms.wild,
                                    wild,
                                )));
                            }
                        }
                    }
                }
            }

            if player.world.npc.active.is_none() {
                for (id, npc) in map.npcs.iter_mut().filter(|(_, npc)| npc.trainer.is_some()) {
                    player.find_battle(&map.id, id, npc);
                }
            }
        }

        // if let Some(tile_id) = map.tile(player.position.coords) {
        //     // look for player

        //     // try running scripts

        //     if player.world.scripts.actions.is_empty() {
        //         'scripts: for script in map.scripts.iter() {
        //             use worldlib::script::world::Condition;
        //             for condition in &script.conditions {
        //                 match condition {
        //                     Condition::Location(location) => {
        //                         if !location.in_bounds(&player.position.coords) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::Activate(direction) => {
        //                         if player.position.direction.ne(direction) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::NoRepeat => {
        //                         if player.world.scripts.executed.contains(&script.identifier) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::Script(script, happened) => {
        //                         if player.world.scripts.executed.contains(script).ne(happened) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                     Condition::PlayerHasPokemon(is_true) => {
        //                         if party.is_empty().eq(is_true) {
        //                             continue 'scripts;
        //                         }
        //                     }
        //                 }
        //             }
        //             player
        //                 .world
        //                 .scripts
        //                 .actions
        //                 .extend_from_slice(&script.actions);
        //             player.world.scripts.actions.push(
        //                 worldlib::script::world::WorldAction::Finish(script.identifier),
        //             );
        //             player.world.scripts.actions.reverse();
        //             break;
        //         }
        //     }
        // }
    }

    fn stop_player(&mut self, player: &mut PlayerCharacter) {
        player.stop_move();

        if let Some(map) = self.maps.get(&player.location) {
            if let Some(destination) = map.warp_at(player.position.coords) {
                // Warping does not trigger tile actions!
                player.world.warp = Some(*destination);
            } else if map.in_bounds(player.position.coords) {
                self.on_tile(player);
            }
        }
    }

    pub fn update(&mut self, player: &mut PlayerCharacter, delta: f32) {
        if player.do_move(delta) {
            self.stop_player(player);
        }
        self.move_npcs(player, delta);
    }

    pub fn try_move<
        P: Deref<Target = Pokemon>,
        M: Deref<Target = Move>,
        I: Deref<Target = Item>,
    >(
        &mut self,
        player: &mut PlayerCharacter,
        party: &[OwnedPokemon<P, M, I>],
        direction: Direction,
    ) {
        player.on_try_move(direction);

        let offset = direction.tile_offset();
        let coords = player.position.coords + offset;

        let movecode = self
            .get(&player.location)
            .map(|map| match map.chunk_movement(coords) {
                MovementResult::Option(code) => code,
                MovementResult::Chunk(direction, offset, connection) => self
                    .connection_movement(direction, offset, connection)
                    .map(|(coords, code)| {
                        player.character.position.coords = coords;
                        player.location = connection.0;
                        self.on_map_change(&player);
                        code
                    }),
            })
            .flatten()
            .unwrap_or(1);
        // .unwrap_or_else(|| self.walk_connections(coords).unwrap_or(1));

        if let Some(map) = self.get(&player.location) {
            if player.world.warp.is_none() {
                if let Some(destination) = map.warp_at(coords) {
                    player.world.warp = Some(*destination);
                    self.sender.send(WorldActions::BeginWarpTransition(coords));
                    return;
                }
            };

            let walk = map
                .tile(coords)
                .map(|tile| {
                    self.tiles
                        .cliffs
                        .get(&direction)
                        .map(|tiles| tiles.contains(&tile))
                })
                .flatten()
                .unwrap_or_default();

            let allow = if !walk {
                // checks if player is inside a solid tile or outside of map, lets them move if true
                // also checks if player is on a one way tile
                if map
                    .tile(player.position.coords)
                    .map(|tile| {
                        self.tiles
                            .cliffs
                            .values()
                            .any(|tiles| tiles.contains(&tile))
                    })
                    .unwrap_or(false)
                {
                    false
                } else {
                    map.local_movement(player.position.coords)
                        .map(|code| !can_move(player.movement, code))
                        .unwrap_or(true)
                }
            } else {
                walk
            };

            if player.movement == Movement::Swimming && can_walk(movecode) {
                player.movement = Movement::Walking
            }

            if can_move(player.movement, movecode) || allow || player.noclip {
                player.pathing.queue.push(direction);
                // self.player.offset =
                //     direction.pixel_offset(self.player.speed() * 60.0 * delta);
            } else if can_swim(movecode) && player.movement != Movement::Swimming {
                const SURF: &MoveId = unsafe { &MoveId::new_unchecked(1718777203) };

                if party
                    .iter()
                    .flat_map(|pokemon| pokemon.moves.iter())
                    .any(|m| &m.0.id == SURF)
                {
                    player.movement = Movement::Swimming;
                    player.pathing.queue.push(direction);
                }
            }
        }
    }

    pub fn connection_movement(
        &self,
        direction: Direction,
        offset: i32,
        connection: &Connection,
    ) -> Option<(Coordinate, MovementId)> {
        self.get(&connection.0)
            .map(|map| {
                let o = offset - connection.1;
                let position = Connection::offset(direction, map, o);
                let coords = position.in_direction(direction);
                map.local_movement(coords).map(|code| (position, code))
            })
            .flatten()
    }

    pub fn warp(&mut self, player: &mut PlayerCharacter, destination: WarpDestination) -> bool {
        match self.maps.contains_key(&destination.location) {
            true => {
                player.position.from_destination(destination.position);
                player.pathing.clear();
                player.location = destination.location;
                true
            }
            false => false,
        }
    }
}
