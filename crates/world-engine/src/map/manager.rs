use crate::engine::{
    controls::{pressed, Control},
    error::ImageError,
    graphics::{self, Color},
    math::{ivec2, IVec2},
    music,
    text::MessagePage,
    utils::Entity,
    Context, EngineContext,
};

use rand::prelude::SmallRng;

use worldlib::{
    actions::WorldActions,
    character::player::PlayerCharacter,
    events::{split, InputEvent, Receiver, Sender},
    map::{
        chunk::Connection, manager::WorldMapManager, warp::WarpDestination, Brightness, WorldMap,
    },
    positions::{Coordinate, Destination, Direction, Location, Position},
    serialized::SerializedWorld,
};

use crate::{
    battle::{BattleId, BattleMessage, BattleTrainerEntry},
    gui::TextWindow,
    map::{data::ClientWorldData, input::PlayerInput, warp::WarpTransition, RenderCoords},
    WorldMetaAction,
};

mod npc;

// pub mod script;

pub struct WorldManager {
    pub world: WorldMapManager<SmallRng>,

    data: ClientWorldData,

    warper: WarpTransition,
    text: TextWindow,
    input: PlayerInput,
    // screen: RenderCoords,
    sender: Sender<WorldMetaAction>,
    receiver: Receiver<WorldActions>,
    // events: EventReceiver<WorldEvents>,
}

impl WorldManager {
    pub fn new(
        ctx: &mut Context,
        actions: Sender<WorldMetaAction>,
        world: SerializedWorld,
    ) -> Result<Self, ImageError> {
        // let events = Default::default();

        let (sender, receiver) = split();

        Ok(Self {
            world: WorldMapManager::new(world.data, world.scripts, sender),

            data: ClientWorldData::new(ctx, world.textures)?,
            warper: WarpTransition::new(),
            text: TextWindow::new(ctx)?,
            input: PlayerInput::default(),
            sender: actions,
            // screen: RenderCoords::default(),
            // events,
            receiver,
        })
    }

    pub fn get(&self, location: &Location) -> Option<&WorldMap> {
        self.world.get(location)
    }

    pub fn start(&mut self, player: &mut PlayerCharacter) {
        self.world.on_warp(player);
        if let Some(battle) = player.world.battle.battling.as_ref() {
            let (id, trainer) = battle
                .trainer
                .as_ref()
                .map(|e| {
                    self.world
                        .get(&e.location)
                        .map(|map| {
                            map.npcs
                                .get(&e.id)
                                .map(|npc| {
                                    npc.trainer
                                        .as_ref()
                                        .map(|t| (npc, t))
                                        .map(|(npc, trainer)| (map, npc, trainer))
                                })
                                .flatten()
                        })
                        .flatten()
                        .map(|(map, npc, trainer)| {
                            (
                                BattleId::Trainer(e.id),
                                Some(BattleTrainerEntry {
                                    name: npc.character.name.clone(),
                                    bag: e.bag.clone(),
                                    badge: trainer.badge,
                                    sprite: npc.group,
                                    transition: map.settings.transition,
                                    defeat: trainer
                                        .defeat
                                        .iter()
                                        .map(|lines| MessagePage {
                                            lines: lines.clone(),
                                            wait: None,
                                            color: npc::color(
                                                self.world
                                                    .data
                                                    .npc
                                                    .groups
                                                    .get(&npc.group)
                                                    .map(|group| group.message)
                                                    .unwrap_or_default(),
                                            ),
                                        })
                                        .collect(),
                                    worth: trainer.worth,
                                }),
                            )
                        })
                })
                .flatten()
                .unwrap_or((BattleId::Wild, None));
            self.sender.send(WorldMetaAction::Battle(BattleMessage {
                id,
                party: battle.party.clone(),
                trainer,
                active: battle.active,
            }));
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.world.seed(seed);
    }

    pub fn post_battle(&mut self, player: &mut PlayerCharacter, winner: bool) {
        self.world.data.post_battle(player, winner)
    }

    pub fn spawn(&self) -> (Location, Position) {
        self.world.data.spawn
    }

    pub fn try_teleport(&mut self, player: &mut PlayerCharacter, location: Location) -> bool {
        if self.world.contains(&location) {
            self.teleport(player, location);
            true
        } else {
            false
        }
    }

    pub fn update(
        &mut self,
        ctx: &mut Context,
        eng: &mut EngineContext,
        player: &mut PlayerCharacter,
        delta: f32,
    ) {
        // } else if self.world_map.alive() {
        //     self.world_map.update(ctx);
        //     if pressed(ctx, Control::A) {
        //         if let Some(location) = self.world_map.despawn_get() {
        //             self.warp_to_location(location);
        //         }
        //     }

        self.text.update(ctx, eng, delta, player);

        self.data.update(delta, player);

        if self.warper.alive() {
            if let Some(music) = self.warper.update(&self.world.data, player, delta) {
                self.world.on_warp(player);
            }
        } else if player.world.warp.is_some() {
            self.warper.spawn();
            player.character.flags.insert(PlayerInput::INPUT_LOCK);
        }

        if let Some(direction) = self.input.update(ctx, eng, player, delta) {
            self.world.input(player, InputEvent::Move(direction));
        }

        if pressed(ctx, eng, Control::A) && !player.character.flags.contains(&PlayerInput::INPUT_LOCK) {        
            self.world.input(player, InputEvent::Interact);
        }

        self.world.update(player, delta);

        for action in self.receiver.try_iter() {
            match action {
                WorldActions::Battle(entry) => {
                    if !player.trainer.party.is_empty() {
                        player.character.locked.increment();
                        let active = entry.active;
                        let party = entry.party.clone();
                        let (id, t) = if let Some(trainer) = entry.trainer.as_ref() {
                            let (id, t) = (
                                BattleId::Trainer(trainer.id),
                                if let Some((map, npc)) = self
                                    .world
                                    .get(&trainer.location)
                                    .map(|map| map.npcs.get(&trainer.id).map(|npc| (map, npc)))
                                    .flatten()
                                {
                                    let trainer = npc.trainer.as_ref().unwrap();
                                    let group = npc::group(&self.world.data.npc.groups, &npc.group);
                                    let tgroup =
                                        npc::trainer(&self.world.data.npc.trainers, &trainer.group);
                                    Some(BattleTrainerEntry {
                                        name: format!("{} {}", tgroup.prefix, npc.character.name),
                                        bag: trainer.bag.clone(),
                                        badge: trainer.badge,
                                        sprite: npc.group,
                                        transition: map.settings.transition,
                                        defeat: trainer
                                            .defeat
                                            .iter()
                                            .map(|lines| MessagePage {
                                                lines: lines.clone(),
                                                wait: None,
                                                color: npc::color(group.message),
                                            })
                                            .collect(),
                                        worth: trainer.worth as _,
                                    })
                                } else {
                                    None
                                },
                            );
                            (id, t)
                        } else {
                            (BattleId::Wild, None)
                        };
                        player.world.battle.battling = Some(entry);
                        self.sender.send(WorldMetaAction::Battle(BattleMessage {
                            id,
                            party,
                            trainer: t,
                            active,
                        }))
                    };
                }
                WorldActions::PlayerJump => self.data.player.jump(player),
                // WorldActions::GivePokemon(pokemon) => {
                //     if let Some(pokemon) = pokemon.init(
                //         &mut self.data.randoms.general,
                //         crate::pokedex(),
                //         crate::movedex(),
                //         crate::itemdex(),
                //     ) {
                //         party.try_push(pokemon);
                //     }
                // }
                WorldActions::PlayMusic(music) => {
                    if let Some(playing) = music::get_current_music(eng) {
                        if playing != &music {
                            music::play_music(ctx, eng, &music);
                        }
                    } else {
                        music::play_music(ctx, eng, &music);
                    }
                }
                WorldActions::BeginWarpTransition(coords) => {
                    if let Some(map) = self.world.get(&player.location) {
                        if let Some(tile) = map.tile(coords) {
                            let palette = *tile.palette(&map.palettes);
                            let tile = tile.id();
                            self.warper
                                .queue(&self.world.data, player, palette, tile, coords);
                        }
                    }
                }
                WorldActions::OnTile => {
                    if let Some(map) = self.world.get(&player.location) {
                        on_tile(map, player, &mut self.data)
                    }
                }
                WorldActions::BreakObject(coordinate) => {
                    if let Some(map) = self.world.get(&player.location) {
                        if let Some(object) = map.object_at(&coordinate) {
                            self.data.object.add(coordinate, &object.group);
                        }
                    }
                }
            }
        }
    }

    // #[deprecated]
    // fn debug_input(&mut self, ctx: &Context, save: &mut PlayerData) {
    //     if is_key_pressed(ctx, Key::F3) {
    //         info!("Local Coordinates: {}", self.player.position.coords);

    //         match self.world.tile(self.player.position.coords) {
    //             Some(tile) => info!("Current Tile ID: {:x}", tile),
    //             None => info!("Currently out of bounds"),
    //         }

    //         info!("Player is {:?}", self.player.movement);
    //     }

    //     if is_key_pressed(ctx, Key::F5) {
    //         if let Some(map) = self.world.get() {
    //             info!("Resetting battled trainers in this map! ({})", map.name);
    //             save.world.get_map(&map.id).battled.clear();
    //         }
    //     }
    // }

    pub fn teleport(&mut self, player: &mut PlayerCharacter, location: Location) {
        if let Some(map) = self.world.data.maps.get(&location) {
            let coords = map.settings.fly_position.unwrap_or_else(|| {
                let mut count = 0u8;
                let mut first = None;
                let index = match map.movements.iter().enumerate().find(|(i, tile)| {
                    if WorldMap::can_move(0, **tile) {
                        count += 1;
                        if first.is_none() {
                            first = Some((*i, **tile));
                        }
                        if count == 8 {
                            return true;
                        }
                    }
                    false
                }) {
                    Some((index, ..)) => index,
                    None => first.map(|(index, ..)| index).unwrap_or_default(),
                } as i32;
                let x = index % map.width;
                let y = index / map.width;
                Coordinate { x, y }
            });
            let location = map.id;
            self.world.warp(
                player,
                WarpDestination {
                    location,
                    position: Destination {
                        coords,
                        direction: Some(Direction::Down),
                    },
                },
            );
        }
    }

    pub fn draw(&self, ctx: &mut Context, eng: &EngineContext, player: &PlayerCharacter) {
        let screen = RenderCoords::new(player);

        let color = match self.world.get(&player.location) {
            Some(current) => {
                let color = match current.settings.brightness {
                    Brightness::Day => Color::WHITE,
                    Brightness::Night => Color::rgb(0.6, 0.6, 0.6),
                };

                super::draw(
                    ctx,
                    eng,
                    current,
                    &player.world,
                    &self.data,
                    &screen,
                    true,
                    color,
                );

                match &current.chunk {
                    Some(chunk) => {
                        for (connection, direction, offset) in chunk
                            .connections
                            .iter()
                            .flat_map(|(d, connections)| connections.iter().map(move |c| (d, c)))
                            .flat_map(|(direction, Connection(location, offset))| {
                                self.world
                                    .data
                                    .maps
                                    .get(&location)
                                    .map(|map| (map, direction, offset))
                            })
                        {
                            fn map_offset(
                                direction: &Direction,
                                current: &WorldMap,
                                map: &WorldMap,
                                offset: i32,
                            ) -> IVec2 {
                                match direction {
                                    Direction::Down => ivec2(offset, current.height),
                                    Direction::Up => ivec2(offset, -map.height),
                                    Direction::Left => ivec2(-map.width, offset),
                                    Direction::Right => ivec2(current.width, offset),
                                }
                            }

                            super::draw(
                                ctx,
                                eng,
                                connection,
                                &player.world,
                                &self.data,
                                &screen.offset(map_offset(direction, current, connection, *offset)),
                                false,
                                color,
                            );
                        }
                    }
                    None => (),
                }

                color
            }
            None => {
                graphics::draw_text_left(
                    ctx,
                    eng,
                    &0,
                    "Cannot get map:",
                    0.0,
                    0.0,
                    Default::default(),
                );
                graphics::draw_text_left(
                    ctx,
                    eng,
                    &0,
                    player.location.map.as_deref().unwrap_or("None"),
                    0.0,
                    8.0,
                    Default::default(),
                );
                graphics::draw_text_left(
                    ctx,
                    eng,
                    &0,
                    player.location.index.as_str(),
                    0.0,
                    16.0,
                    Default::default(),
                );
                Color::WHITE
            }
        };

        if player.world.debug_draw {
            graphics::draw_text_left(
                ctx,
                eng,
                &1,
                player
                    .location
                    .map
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("No Base Map ID"),
                5.0,
                5.0,
                Default::default(),
            );
            graphics::draw_text_left(
                ctx,
                eng,
                &1,
                player.location.index.as_str(),
                5.0,
                15.0,
                Default::default(),
            );

            let coordinates = format!("{}", player.character.position.coords);
            graphics::draw_text_left(ctx, eng, &1, &coordinates, 5.0, 25.0, Default::default());
        } else {
            self.warper.draw_door(ctx, &self.data.tiles, &screen);
        }

        self.data.player.draw(ctx, player, color);
        if !player.world.debug_draw {
            self.data.player.bush.draw(ctx, &screen);
            self.warper.draw(ctx);
        }
        self.text.draw(ctx, eng, player);
    }
}

fn on_tile(
    map: &WorldMap,
    player: &PlayerCharacter,
    data: &mut ClientWorldData,
    // sender: &Sender<WorldActions>,
) {
    data.player.bush.check(map, player.position.coords);
    // check for wild encounter

    // if let Some(tile_id) = map.tile(player.position.coords) {

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
    //             player
    //                 .world
    //                 .scripts
    //                 .actions
    //                 .push(worldlib::script::world::WorldAction::Finish(
    //                     script.identifier,
    //                 ));
    //             player.world.scripts.actions.reverse();
    //             break;
    //         }
    //     }
    // }
}

// fn get_mut(world: &mut WorldMapManager) -> Option<&mut WorldMap> {
//     match world.data.location.as_ref() {
//         Some(cur) => world.maps.get_mut(cur),
//         None => None,
//     }
// }